//! Natural root composition at one chosen width. Public constructors stay exact.

use crate::decorations::legend::descriptor::{ContinuousDescriptor, PreparedGuideDescriptor};
use crate::layout::fit::{self, FitScope};
use crate::layout::kernel::{FeedbackKind, FeedbackRound, RootSize};
use crate::layout::{ComponentAddress, PhysicalTextAlignment};
use crate::theme::ResolvedTheme;
use crate::{Error, Plot, PlotKind};

pub(crate) struct GuidedPreference {
    pub(crate) descriptor: PreparedGuideDescriptor,
    pub(crate) position: crate::LegendPosition,
}

impl GuidedPreference {
    pub(crate) fn horizontal(&self) -> bool {
        matches!(
            self.position,
            crate::LegendPosition::OutsideTopLeft
                | crate::LegendPosition::OutsideTop
                | crate::LegendPosition::OutsideTopRight
                | crate::LegendPosition::OutsideBottomLeft
                | crate::LegendPosition::OutsideBottom
                | crate::LegendPosition::OutsideBottomRight
        )
    }

    pub(crate) fn preference(&self, theme: &ResolvedTheme) -> Result<(f64, f64), Error> {
        match &self.descriptor {
            PreparedGuideDescriptor::Discrete((title, entries)) => {
                crate::decorations::legend::Legend::natural_preference(title, entries, theme)
            }
            PreparedGuideDescriptor::Continuous(descriptor) => {
                continuous_preference(descriptor, theme, &FitScope::root())
            }
        }
    }

    pub(crate) fn at_width(
        &self,
        width: f64,
        theme: &ResolvedTheme,
        scope: &FitScope,
    ) -> Result<(f64, f64), Error> {
        let outcome = match &self.descriptor {
            PreparedGuideDescriptor::Discrete((title, entries)) => {
                let legend = fit::measured(crate::decorations::legend::Legend::measure_assigned(
                    title,
                    std::sync::Arc::clone(entries),
                    (width, None),
                    self.horizontal(),
                    false,
                    theme,
                    &mut false,
                    scope,
                ))
                .map_err(fit::PlanError::into_error)?;
                (legend.compute_box_width(), legend.compute_box_height())
            }
            PreparedGuideDescriptor::Continuous(descriptor) => {
                let guide = fit::measured(
                    crate::decorations::legend::MeasuredContinuousGuide::measure(
                        descriptor,
                        (width, None),
                        false,
                        false,
                        theme,
                        &mut false,
                        scope,
                    ),
                )
                .map_err(fit::PlanError::into_error)?;
                (guide.width, guide.height)
            }
        };
        Ok(outcome)
    }
}

pub(crate) fn continuous_width(
    descriptor: &ContinuousDescriptor,
    theme: &ResolvedTheme,
) -> Result<f64, Error> {
    let (metrics, frame_minimum) =
        crate::decorations::legend::MeasuredContinuousGuide::allocation_metrics(descriptor, theme)?;
    let mut width = text_width(theme, &descriptor.title, &theme.typography.legend_title)?;
    for label in descriptor
        .labels
        .iter()
        .map(String::as_str)
        .chain(descriptor.missing.iter().map(|key| key.label.as_str()))
    {
        theme
            .prepared_font_environment()?
            .claim_processing_visits(1)
            .map_err(|error| Error::Render(format!("continuous preference value: {error:?}")))?;
        let plan = theme.measure_text(
            label,
            &theme.typography.legend_label,
            PhysicalTextAlignment::Start,
            "continuous preference value",
            false,
        )?;
        width =
            width.max(metrics.handle_w + metrics.handle_text + plan.width().max(plan.fit_width()));
    }
    crate::layout::kernel::required_length(
        (width + 2.0 * metrics.pad + 1.0 / 1024.0).max(frame_minimum),
    )
}

pub(crate) fn continuous_preference(
    descriptor: &ContinuousDescriptor,
    theme: &ResolvedTheme,
    scope: &FitScope,
) -> Result<(f64, f64), Error> {
    let width = continuous_width(descriptor, theme)?;
    let guide = fit::measured(
        crate::decorations::legend::MeasuredContinuousGuide::measure(
            descriptor,
            (width, None),
            false,
            false,
            theme,
            &mut false,
            scope,
        ),
    )
    .map_err(fit::PlanError::into_error)?;
    Ok((guide.width, guide.height))
}

fn scoped_guide<'a, T>(
    panels: &'a [(Option<String>, T)],
    plot: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<&'a T, Error> {
    for (scope, marks) in panels {
        data.claim_source_visits(1)?;
        if scope.as_deref() == plot.axis_scope() {
            return Ok(marks);
        }
    }
    Err(Error::Render(
        "scoped continuous guide preference is missing".into(),
    ))
}

/// The prepared scope owns the guide. No panel assignment or edge trial occurs
/// while this intrinsic input is retained.
pub(crate) fn guided_preference(
    plot: &Plot,
    inputs: &super::PreparedInteriorInputs,
    theme: &ResolvedTheme,
) -> Result<Option<GuidedPreference>, Error> {
    let resources = theme.prepared_font_environment()?.generation_resources();
    resources
        .claim_calls(1)
        .map_err(|error| Error::Render(format!("guided intrinsic call: {error:?}")))?;
    let data = plot.data().with_axis_resources(&resources);
    if matches!(plot.kind(), PlotKind::Heatmap | PlotKind::Beeswarm) {
        if plot.continuous_guide_suppressed() {
            return Ok(None);
        }
        let descriptor = match plot.kind() {
            PlotKind::Heatmap => {
                let prepared = inputs.heatmap.as_ref().ok_or_else(|| {
                    Error::Render("scoped Heatmap guide preference is missing".into())
                })?;
                let marks = scoped_guide(&prepared.panels, plot, data)?;
                let semantics = marks.guide_semantics(theme, data)?;
                let values = marks.guide_values();
                let indices: &[usize] =
                    if semantics.domain.0.to_bits() == semantics.domain.1.to_bits() {
                        &[0]
                    } else if semantics.center.is_some_and(|center| {
                        center > semantics.domain.0 && center < semantics.domain.1
                    }) {
                        &[0, 2, 1]
                    } else {
                        &[0, 1]
                    };
                ContinuousDescriptor {
                    field: data.copy_source_string(plot.color_dimension_key().unwrap_or(""))?,
                    title: data.copy_source_string(&marks.color_label)?,
                    stops: semantics.stops,
                    center: semantics.center,
                    missing: semantics.missing,
                    domain: semantics.domain,
                    exact: plot.heatmap_color_limits_spec().is_some(),
                    labels: data.collect_source(
                        indices
                            .iter()
                            .map(|&index| data.copy_source_string(&values[index])),
                    )?,
                }
            }
            PlotKind::Beeswarm => {
                let prepared = inputs.beeswarm.as_ref().ok_or_else(|| {
                    Error::Render("scoped Beeswarm guide preference is missing".into())
                })?;
                let marks = scoped_guide(&prepared.panels, plot, data)?;
                let semantics = marks.guide_semantics(data)?;
                let values = marks.guide_values();
                let indices: &[usize] =
                    if semantics.domain.0.to_bits() == semantics.domain.1.to_bits() {
                        &[1]
                    } else {
                        &[1, 0]
                    };
                ContinuousDescriptor {
                    field: data.copy_source_string(plot.color_dimension_key().unwrap_or(""))?,
                    title: data.copy_source_string(&marks.color_label)?,
                    stops: semantics.stops,
                    center: semantics.center,
                    missing: semantics.missing,
                    domain: semantics.domain,
                    exact: plot.beeswarm_color_limits_spec().is_some(),
                    labels: data.collect_source(
                        indices
                            .iter()
                            .map(|&index| data.copy_source_string(&values[index])),
                    )?,
                }
            }
            _ => unreachable!("continuous kind selected"),
        };
        return Ok(Some(GuidedPreference {
            descriptor: PreparedGuideDescriptor::Continuous(descriptor),
            position: crate::LegendPosition::OutsideRight,
        }));
    }
    let position = plot.legend_position_spec();
    if plot.legend_policy_spec() == crate::LegendPolicy::Hidden
        || (position != crate::LegendPosition::Auto
            && !crate::decorations::plan::is_outside(position))
        || plot.kind() == PlotKind::Matrix
    {
        return Ok(None);
    }
    let mut entries = inputs.guide_entries(plot, theme, data, plot.axis_scope())?;
    crate::decorations::legend::resolve_entries(plot, &mut entries, &resources)?;
    if entries.is_empty() {
        return Ok(None);
    }
    let title = crate::decorations::legend::resolved_title(&entries, &resources)?;
    let entries = crate::decorations::legend::Legend::admit_entries(entries, theme)?;
    Ok(Some(GuidedPreference {
        descriptor: PreparedGuideDescriptor::Discrete((title, entries)),
        position,
    }))
}

pub(crate) fn guided_body_height(
    plot: &Plot,
    theme: &ResolvedTheme,
    width: f64,
    guide: Option<&GuidedPreference>,
    scope: &FitScope,
) -> Result<f64, Error> {
    let hard = plot.intrinsic_body_minimum(theme)?;
    let preference = body_preference(plot, theme, hard)?;
    let Some(guide) = guide else {
        return plot.natural_body_height(theme, width, preference.1);
    };
    let natural = guide.preference(theme)?;
    if guide.horizontal() {
        let measured = guide.at_width(width, theme, scope)?;
        return Ok(plot.natural_body_height(theme, width, preference.1)?
            + measured.1
            + theme.legend.outside_gap);
    }
    let available = width - hard.0 - theme.legend.outside_gap;
    if available <= 0.0
        || (available < natural.0
            && matches!(guide.descriptor, PreparedGuideDescriptor::Discrete(_)))
    {
        // This proposal does not select an Auto edge. The existing containing
        // frame trials report the selected width-dependent content height.
        return Ok(plot
            .natural_body_height(theme, width, preference.1)?
            .max(natural.1));
    }
    let allocated = if matches!(guide.descriptor, PreparedGuideDescriptor::Continuous(_)) {
        continuous_width(
            match &guide.descriptor {
                PreparedGuideDescriptor::Continuous(value) => value,
                PreparedGuideDescriptor::Discrete(_) => unreachable!(),
            },
            theme,
        )?
        .min(available)
    } else {
        natural.0.min(available)
    };
    let measured = guide.at_width(allocated, theme, scope)?;
    Ok(plot
        .natural_body_height(
            theme,
            width - measured.0 - theme.legend.outside_gap,
            preference.1,
        )?
        .max(measured.1))
}

pub(crate) fn integral(value: f64) -> Result<u32, Error> {
    let quanta = crate::layout::kernel::ceil_spatial_quanta(value)?;
    let pixels = (i128::from(quanta) + 1023) / 1024;
    u32::try_from(pixels)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| {
            Error::Render(
                "natural root dimension is not representable as positive CSS pixels".into(),
            )
        })
}

#[expect(
    clippy::cast_precision_loss,
    reason = "root-bounded spatial quanta convert exactly to CSS pixels"
)]
fn pixels(value: i64) -> f64 {
    value as f64 / 1024.0
}

pub(crate) fn text_width(
    theme: &ResolvedTheme,
    source: &str,
    role: &crate::theme::TextTheme,
) -> Result<f64, Error> {
    let (hard, natural) =
        fit::intrinsic_widths(theme, source, role, "natural root text preference")?;
    if natural <= hard {
        return Ok(hard);
    }
    let ch = theme
        .measure_text(
            "0",
            role,
            PhysicalTextAlignment::Start,
            "natural root 66ch",
            false,
        )?
        .width();
    Ok(hard.max(natural.min(66.0 * ch)))
}

pub(crate) fn body_preference(
    plot: &Plot,
    theme: &ResolvedTheme,
    hard: (f64, f64),
) -> Result<(f64, f64), Error> {
    use super::{InteriorRoute, SpecializedGrid};
    match InteriorRoute::classify(plot.kind()) {
        InteriorRoute::StandardCoordinate(_) => {
            let preference = super::preference::mark_preference(plot, theme, hard)?;
            Ok((
                pixels(preference.body_quanta.0),
                pixels(preference.body_quanta.1),
            ))
        }
        InteriorRoute::OneCategoricalAxis(_) => {
            let preference = super::preference::categorical_preference(plot, theme, hard)?;
            Ok((
                pixels(preference.body_quanta.0),
                pixels(preference.body_quanta.1),
            ))
        }
        InteriorRoute::SpecializedGrid(SpecializedGrid::Marginal) => {
            crate::plots::marginal::natural_body_preference(plot, theme)
        }
        InteriorRoute::SpecializedGrid(SpecializedGrid::Matrix) => {
            crate::plots::matrix::natural_body_preference(plot, theme)
        }
        InteriorRoute::SpecializedGrid(SpecializedGrid::Heatmap) => {
            crate::plots::heatmap::natural_body_preference(plot, theme)
        }
        InteriorRoute::SpecializedGrid(SpecializedGrid::Beeswarm | SpecializedGrid::Waterfall) => {
            let ch = theme
                .measure_text(
                    "0",
                    &theme.typography.tick_label,
                    PhysicalTextAlignment::Start,
                    "specialized numeric 66ch",
                    false,
                )?
                .width();
            let count = plot
                .prepared_axis_inputs()
                .and_then(|axes| axes.categories[1].as_ref())
                .map_or(0, Vec::len);
            let count = u32::try_from(count).map_err(|_| {
                Error::Render("specialized category preference count overflow".into())
            })?;
            let height = hard
                .1
                .max(2.0 * f64::from(theme.typography.tick_label.font_size) * f64::from(count));
            let width = hard.0.max(66.0 * ch).max(4.0 * height / 3.0);
            Ok((width, height.max(3.0 * width / 4.0)))
        }
    }
}

/// Measure natural outer height after width selection, under consumed root feedback.
#[expect(
    clippy::too_many_lines,
    reason = "root text, axes, guide and scientific body compose one sizing request"
)]
pub(crate) fn plot_size(
    plot: &Plot,
    inputs: &super::PreparedInteriorInputs,
    theme: &ResolvedTheme,
    requested: RootSize,
    round: &FeedbackRound,
) -> Result<RootSize, Error> {
    let (requested_width, requested_height) = requested.requested_dimensions();
    if let (Some(width), Some(height)) = (requested_width, requested_height) {
        return Ok(RootSize::new(width, height));
    }
    let resources = theme.prepared_font_environment()?.generation_resources();
    resources
        .claim_calls(1)
        .map_err(|error| Error::Render(format!("natural root sizing call: {error:?}")))?;
    let data = plot.data().with_axis_resources(&resources);
    let hard = plot.intrinsic_body_minimum(theme)?;
    let preference = body_preference(plot, theme, hard)?;
    let axes = plot.prepared_axis_inputs();
    let outward = theme.axes.ticks.as_ref().map_or(0.0, |ticks| {
        f64::from(ticks.length) + f64::from(ticks.stroke.width) / 2.0
    }) + theme
        .axes
        .frame
        .as_ref()
        .map_or(0.0, |frame| f64::from(frame.stroke.width) / 2.0);
    let chrome = theme.typography.axis_title.line_pitch_pixels()
        + theme.axes.spacing.y_tick_label_gap
        + theme.axes.spacing.y_title_tick_label_gap
        + theme.axes.spacing.y_title_edge_padding
        + outward
        + 4.0;
    let mut y_width = 0.0_f64;
    if let Some(axes) = axes {
        if let Some(labels) = &axes.display_categories[1] {
            for label in labels {
                data.claim_source_visits(1)?;
                y_width = y_width.max(
                    theme
                        .measure_text(
                            label,
                            &theme.typography.tick_label,
                            PhysicalTextAlignment::End,
                            "natural category gutter",
                            false,
                        )?
                        .width(),
                );
            }
        } else {
            for value in [
                axes.ranges[1].0,
                f64::midpoint(axes.ranges[1].0, axes.ranges[1].1),
                axes.ranges[1].1,
            ] {
                data.claim_source_visits(1)?;
                let label = crate::layout::format_guide_label(value, 0, data)?;
                y_width = y_width.max(
                    theme
                        .measure_text(
                            &label,
                            &theme.typography.tick_label,
                            PhysicalTextAlignment::End,
                            "natural numeric gutter",
                            false,
                        )?
                        .width(),
                );
            }
        }
    }
    let guide = guided_preference(plot, inputs, theme)?;
    let (guide_width, guide_height) = if plot.kind() == PlotKind::Matrix {
        crate::plots::matrix::natural_guide_preference(plot, theme)?
    } else if let Some(guide) = &guide {
        guide.preference(theme)?
    } else {
        (0.0, 0.0)
    };
    let guide_width = guide_width
        + if guide_width > 0.0 {
            theme.legend.outside_gap
        } else {
            0.0
        };
    let horizontal = guide.as_ref().is_some_and(GuidedPreference::horizontal);
    let mut outer_width =
        preference.0 + y_width + chrome + 32.0 + if horizontal { 0.0 } else { guide_width };
    if horizontal {
        outer_width = outer_width.max(guide_width + 32.0);
    }
    for (source, role) in [plot.title_text(), plot.subtitle_text(), plot.caption_text()]
        .into_iter()
        .zip([
            &theme.typography.title,
            &theme.typography.subtitle,
            &theme.typography.caption,
        ])
    {
        if let Some(source) = source {
            outer_width = outer_width.max(text_width(theme, source, role)? + 32.0);
        }
    }
    let width = requested_width.map_or_else(|| integral(outer_width), Ok)?;
    let owner = ComponentAddress::root().child(
        crate::layout::AddressRole::PlotBody,
        0,
        &theme.prepared_font_environment()?.generation_resources(),
    )?;
    let gutters = round.insets(&owner, FeedbackKind::AxisGutter)?;
    let ink = round.insets(&owner, FeedbackKind::OwnedInk)?;
    let left = if gutters[0] > 0.0 {
        gutters[0]
    } else {
        y_width + chrome
    };
    // This initial content proposal is not a guide placement. The owning panel
    // trial derives its height from the selected guide edge at this fixed width.
    let body_width = (f64::from(width) - 32.0 - left - gutters[1] - ink[0] - ink[1]).max(hard.0);
    let body_height = plot.natural_body_height(theme, body_width, preference.1)?;
    let sources = [plot.title_text(), plot.subtitle_text(), plot.caption_text()];
    let (top, caption) =
        crate::decorations::title_block::plan::RootTitleBlockPlan::content_reserves(
            theme,
            sources,
            f64::from(width) - 32.0,
            crate::decorations::title_block::plan::RootAlignments::resolve(
                theme,
                [plot.title_alignment, plot.subtitle_alignment],
                [None; 2],
            ),
            &FitScope::root(),
        )
        .map_err(fit::PlanError::into_error)?;
    let bottom = if gutters[2] > 0.0 {
        gutters[2]
    } else {
        theme.typography.tick_label.line_pitch_pixels()
            + theme.typography.axis_title.line_pitch_pixels()
            + theme.axes.spacing.x_tick_label_gap
            + theme.axes.spacing.x_title_tick_label_gap
            + outward
            + 4.0
    };
    let height = integral(
        top + caption
            + body_height.max(guide_height)
            + bottom
            + gutters[3]
            + ink[2]
            + ink[3]
            + 20.0,
    )?;
    Ok(RootSize::new(width, height))
}

impl Plot {
    pub(crate) fn natural_body_height(
        &self,
        theme: &ResolvedTheme,
        width: f64,
        preference: f64,
    ) -> Result<f64, Error> {
        let hard = self.intrinsic_body_minimum(theme)?;
        let height = match self.kind() {
            PlotKind::Heatmap => crate::plots::heatmap::natural_body_height(self, theme, width),
            PlotKind::Beeswarm => crate::plots::beeswarm::natural_body_height(self, theme, width),
            PlotKind::Waterfall => crate::plots::waterfall::natural_body_height(self, theme, width),
            PlotKind::Matrix => Ok(width.max(hard.1)),
            PlotKind::Marginal => crate::plots::marginal::natural_body_height(self, theme, width),
            _ => {
                let density = match super::InteriorRoute::classify(self.kind()) {
                    super::InteriorRoute::StandardCoordinate(_) => {
                        super::preference::mark_preference(self, theme, hard)?.density_side
                    }
                    super::InteriorRoute::OneCategoricalAxis(_) => {
                        let categorical =
                            super::preference::categorical_preference(self, theme, hard)?;
                        if categorical.axis == crate::layout::AxisSide::Left {
                            pixels(categorical.body_quanta.1)
                        } else {
                            0.0
                        }
                    }
                    super::InteriorRoute::SpecializedGrid(_) => preference,
                };
                Ok(hard.1.max(3.0 * width / 4.0).max(density))
            }
        }?;
        crate::layout::kernel::required_length(height)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn continuous_intrinsic_width_admits_broad_frame_and_missing_key_chrome() {
        let mut theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        theme.legend.frame_stroke_width = 30.0;
        theme.legend.frame_radius = 50.0;
        let descriptor = crate::decorations::legend::descriptor::ContinuousDescriptor {
            field: "signal".into(),
            title: "Signal".into(),
            stops: vec![
                crate::render::backend::GradientStop::new(0.0, (0, 0, 0)),
                crate::render::backend::GradientStop::new(1.0, (255, 255, 255)),
            ],
            center: None,
            missing: Some(crate::decorations::legend::descriptor::MissingKey {
                fill: crate::theme::Color::rgb(224, 224, 224),
                outline: crate::theme::OutlineTheme {
                    paint: crate::theme::Color::rgb(120, 120, 120),
                    width: 40.0,
                },
                label: "Missing".into(),
            }),
            domain: (0.0, 1.0),
            exact: false,
            labels: vec!["0".into(), "1".into()],
        };
        let (width, height) = super::continuous_preference(
            &descriptor,
            &theme,
            &crate::layout::fit::FitScope::root(),
        )
        .unwrap();
        assert!(width >= 130.0 && height >= 130.0);
        assert!(width < 300.0 && height < 500.0);
    }

    #[test]
    fn upward_css_conversion_preserves_extreme_representability() {
        assert_eq!(super::integral(1.0 / 1024.0).unwrap(), 1);
        assert_eq!(super::integral(300.0 + 1.0 / 1024.0).unwrap(), 301);
        assert_eq!(super::integral(f64::from(u32::MAX)).unwrap(), u32::MAX);
        for value in [
            0.0,
            -1.0,
            f64::INFINITY,
            f64::NAN,
            f64::from(u32::MAX) + 1.0 / 1024.0,
        ] {
            assert!(super::integral(value).is_err());
        }
    }
}
