//! The independent cell/estimate/total common-text block outside scientific marks.

use super::slot::ExplanationPlan;
use crate::builder::PlotDescription;
use crate::layout::fit::{FitKind, FitScope, PlanError};
use crate::layout::interior::preference::panel;
use crate::orchestrator::registration::{PlotRegistration, PreparedFamily};
use crate::render::backend::PlacedText;
use crate::spec::PlotMeasureContext;
use crate::{Error, PlotKind, Result, Text, TextTheme};

fn role<'a>(
    plot: &PlotDescription,
    theme: &'a crate::text::TextRequestContext<'_, '_>,
) -> Result<&'a TextTheme> {
    match plot.kind() {
        PlotKind::Heatmap => theme.contrasting_text(theme.surfaces.get_plot_panel()),
        PlotKind::Waterfall => Ok(&theme.typography.annotations.waterfall_total),
        _ => Ok(&theme.typography.annotations.estimate_detail),
    }
}

fn sources(
    plot: &PlotDescription,
    registered: Option<&PlotRegistration<'_>>,
    mut visit: impl FnMut(&Text) -> Result<()>,
) -> Result<()> {
    match plot.kind() {
        PlotKind::Heatmap => {
            if let Some(marks) = family(registered, "Heatmap", |family| match family {
                PreparedFamily::Heatmap(marks) => Some(marks),
                _ => None,
            })? {
                if let Some(text) = marks.numeric_explanation() {
                    visit(text)?;
                }
            }
        }
        PlotKind::Coefficient => {
            if let Some(marks) = family(registered, "Coefficient", |family| match family {
                PreparedFamily::Estimate { marks, .. } => Some(marks),
                _ => None,
            })? {
                if let Some(text) = marks.numeric_explanation() {
                    visit(text)?;
                }
            }
        }
        PlotKind::Waterfall => {
            if let Some(marks) = family(registered, "Waterfall", |family| match family {
                PreparedFamily::Waterfall(marks) => Some(marks),
                _ => None,
            })? {
                for text in marks.total_explanations() {
                    visit(text)?;
                }
            }
        }
        _ => (),
    }
    Ok(())
}

fn family<'a, T>(
    registered: Option<&'a PlotRegistration<'_>>,
    owner: &str,
    select: impl Fn(&'a PreparedFamily) -> Option<&'a T>,
) -> Result<Option<&'a T>> {
    if registered.is_none_or(|registered| select(&registered.family).is_none()) {
        return Ok(None);
    }
    Ok(select(panel(registered, owner)?))
}

fn source<'a>(context: &PlotMeasureContext<'a, '_>) -> Result<Option<&'a Text>> {
    let plot = context.registered.source;
    match plot.kind() {
        PlotKind::Heatmap => {
            Ok(
                family(Some(context.registered), "Heatmap", |family| match family {
                    PreparedFamily::Heatmap(marks) => Some(marks),
                    _ => None,
                })?
                .and_then(crate::plots::heatmap::HeatmapData::numeric_explanation),
            )
        }
        PlotKind::Coefficient => {
            Ok(family(
                Some(context.registered),
                "Coefficient",
                |family| match family {
                    PreparedFamily::Estimate { marks, .. } => Some(marks),
                    _ => None,
                },
            )?
            .and_then(crate::plots::interval_estimate::PreparedEstimate::numeric_explanation))
        }
        PlotKind::Waterfall => {
            if !matches!(&context.registered.family, PreparedFamily::Waterfall(_)) {
                return Ok(None);
            }
            context
                .registered
                .axes
                .coordinate()
                .ok_or_else(|| Error::Render("total explanation lacks prepared axes".into()))?;
            let x = context.axis_range(crate::layout::AxisSide::Bottom)?;
            let y = context.axis_range(crate::layout::AxisSide::Left)?;
            let PreparedFamily::Waterfall(marks) = panel(Some(context.registered), "Waterfall")?
            else {
                return Ok(None);
            };
            marks.total_explanation(&crate::layout::Projection::new(
                x,
                y,
                context.assignment.local.body.as_tuple(),
            ))
        }
        _ => Ok(None),
    }
}

fn gap(theme: &crate::text::TextRequestContext<'_, '_>) -> Result<f64> {
    theme
        .panel_edge_length()?
        .pixels_rounded(crate::layout::kernel::SpatialRounding::Ceil)
}

pub(crate) fn preferred_width(
    plot: &PlotDescription,
    registered: Option<&PlotRegistration<'_>>,
    operation: &crate::font::canonical::OperationContext,
    theme: &crate::text::TextRequestContext<'_, '_>,
) -> Result<f64> {
    let mut width = 0.0_f64;
    sources(plot, registered, |text| {
        width = width.max(super::slot::preferred_width(
            text,
            role(plot, theme)?,
            theme,
        )?);
        Ok(())
    })?;
    Ok(width.max(axis_preferences(registered, operation, theme)?.0))
}

pub(crate) fn axis_preferences(
    registered: Option<&PlotRegistration<'_>>,
    operation: &crate::font::canonical::OperationContext,
    theme: &crate::text::TextRequestContext<'_, '_>,
) -> Result<(f64, f64)> {
    let mut preferred = [0.0_f64; 2];
    // The axis owner later measures the actual selected set. Intrinsic probes
    // include common explanations without inventing scientific tick positions.
    if let Some(axes) = registered.and_then(|registered| registered.axes.coordinate()) {
        for (side, preferred) in preferred.iter_mut().enumerate() {
            let base = axes.axis(registered.expect("prepared axis occurrence").source, side);
            if base.exact_ticks()?.is_some() || axes.categories(side).is_some() {
                continue;
            }
            let registered = registered.expect("prepared axis occurrence");
            if let Some(set) = base.numeric_probe(registered.axis_numeric_binding(operation)?)? {
                if let Some(text) = set.explanation {
                    let preference = super::slot::preferred_width(
                        &text,
                        &theme.typography.axes.tick_label,
                        theme,
                    )?;
                    *preferred = preferred.max(preference);
                }
            }
        }
    }
    Ok((preferred[0], preferred[1]))
}

pub(crate) fn natural_height(
    plot: &PlotDescription,
    registered: Option<&PlotRegistration<'_>>,
    theme: &crate::text::TextRequestContext<'_, '_>,
    width: f64,
) -> Result<f64> {
    let mut height = 0.0_f64;
    sources(plot, registered, |text| {
        let plan = ExplanationPlan::measure(
            text,
            role(plot, theme)?,
            width,
            crate::TextAlignment::Start,
            theme,
            &FitScope::root(),
        )
        .map_err(PlanError::into_error)?;
        height = height.max(plan.height + gap(theme)?);
        Ok(())
    })?;
    Ok(height)
}

pub(crate) fn reserve(
    factory: &crate::orchestrator::assignment::PlotMeasureAssignmentFactory<'_, '_>,
    assignment: crate::orchestrator::assignment::PlotMeasureAssignment,
    guides: crate::decorations::legend::discrete::scoped::GuideConsumption<'_>,
    request: &crate::text::TextRequestContext<'_, '_>,
    fit_scope: &FitScope,
) -> std::result::Result<
    (
        crate::orchestrator::assignment::PlotMeasureAssignment,
        Option<PlacedText>,
    ),
    PlanError,
> {
    let (body, text) = {
        let context = factory.context(&assignment, guides, fit_scope)?;
        let Some(text) = source(&context)? else {
            return Ok((assignment, None));
        };
        let plan = ExplanationPlan::measure(
            text,
            role(context.registered.source, request)?,
            assignment.body.width(),
            crate::TextAlignment::Start,
            request,
            fit_scope,
        )?;
        let mut body = assignment.body;
        body.y1 -= plan.height + gap(request)?;
        if body.height() <= 0.0 {
            return Err(fit_scope
                .failure(
                    FitKind::Text,
                    "required numeric explanation leaves no mark body",
                )
                .into());
        }
        (body, plan.at(assignment.body.x0, assignment.body.y1))
    };
    Ok((
        factory.apply_change(
            assignment,
            crate::orchestrator::assignment::PlotAssignmentChange::NumericExplanation { body },
        )?,
        Some(text),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decorations::legend::discrete::scoped::SUPPRESSED_GUIDES;
    use crate::layout::PixelRect;
    use crate::orchestrator::assignment::PlotAllocationInputs;
    use crate::{DataCoordinate, DataPosition, TextLayout, TextOrientation};

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one real Root/factory contract distinguishes Probe direction, Full's absent reference and actual category failure"
    )]
    fn probe_numeric_explanation_keeps_pre_reserve_data_reference_and_full_stays_unprojected() {
        let appearance = crate::Theme::publication().resolve().unwrap();
        let text = crate::text::TextService::default();
        let request = crate::text::TextRequestContext::new(&appearance, &text);
        for endpoint in ["second", "absent"] {
            let layout = TextLayout::default()
                .anchor(crate::TextAnchor::center())
                .orientation(TextOrientation::data_direction(
                    DataPosition::new(
                        DataCoordinate::numeric(1.0).unwrap(),
                        DataCoordinate::category("first"),
                    ),
                    DataPosition::new(
                        DataCoordinate::numeric(3.0).unwrap(),
                        DataCoordinate::category(endpoint),
                    ),
                ));
            let format = crate::NumericFormat::new(
                crate::NumericNotation::Decimal,
                crate::NumericPrecision::Fractional(0),
            )
            .unwrap()
            .unit(Some(crate::NumericUnit::new(
                "t",
                crate::NumericUnitPlacement::Shared,
            )))
            .unwrap()
            .explanation(crate::NumericExplanation::automatic().layout(layout));
            let source = crate::Plot::coefficient(
                crate::CategoryValues::new("term", ["first", "second"]).unwrap(),
                crate::CoefficientInput::estimates(crate::NumericValues::new(
                    "estimate",
                    [1.0, 3.0],
                )),
            )
            .estimate_labels(format)
            .x_limits(0.0, 4.0)
            .size(800, 800)
            .erase();
            crate::orchestrator::with_prepared_plot_for_test(
                &source,
                &request,
                |_| Ok(()),
                |plot, root, request, _, round| {
                    // Root's real census and font preparation precede direction resolution.
                    request.text.prepared_font_environment().unwrap();
                    let registered = &root.plots[0];
                    let measurement = root
                        .assignment_factory()
                        .assign(plot.sizing_request().assigned_size().unwrap());
                    let factory = root
                        .plot_assignment_factory(&measurement, &registered.scope.id)
                        .unwrap();
                    let panel = PixelRect::new(20.0, 620.0, 20.0, 620.0);
                    let entry = factory
                        .assign(
                            PlotAllocationInputs::for_test(
                                measurement.measurement_size,
                                panel,
                                round,
                                &registered.scope.address,
                            ),
                            round,
                            None,
                        )
                        .unwrap();
                    let scope = FitScope::root();
                    if endpoint == "second" {
                        let error = crate::orchestrator::measure_plot(
                            &factory,
                            entry.clone(),
                            SUPPRESSED_GUIDES,
                            &scope,
                        )
                        .unwrap_err();
                        assert!(
                            matches!(error, PlanError::Fatal(Error::Data(message))
                                if message == "data text direction requires actual scientific axes")
                        );
                    }
                    let measured = crate::orchestrator::measure_probe(
                        &factory,
                        entry,
                        SUPPRESSED_GUIDES,
                        &scope,
                    );
                    if endpoint == "absent" {
                        assert!(
                            matches!(measured.unwrap_err(), PlanError::Fatal(Error::Data(message))
                                if message == "annotation category \"absent\" is absent from the actual axis")
                        );
                        return;
                    }
                    let (residual, result) = measured.unwrap();
                    let explanation = result
                        .annotations
                        .as_ref()
                        .unwrap()
                        .explanation
                        .as_ref()
                        .expect("the real shared unit generates a required explanation");
                    assert_eq!(explanation.plan.source, "t");
                    assert!(!explanation.plan.lines.is_empty());
                    // Half the numeric span and one of two equal top-down row
                    // pitches form a 45-degree direction in the original square.
                    // Reprojecting after reservation would give a smaller angle.
                    assert!((explanation.placement.rotation - 45.0).abs() < 1e-10);
                    assert_eq!(
                        [residual.body.x0, residual.body.x1, residual.body.y0].map(f64::to_bits),
                        [panel.x0, panel.x1, panel.y0].map(f64::to_bits)
                    );
                    assert!(residual.body.y1 < panel.y1);
                    assert!(residual.body.height() > 0.0);
                },
            )
            .unwrap();
        }
    }
}
