//! Shared geometry for distinct coefficient and forest semantic renderers.

use crate::Error;

use crate::layout::{AxisSpec, AxisTick, Layout, PhysicalTextAlignment, ResolvedTextPlan};
use crate::render::backend::{
    with_clip, Backend, LineStyle, PlannedTextPlacement, Point2D, TextAnchor,
};
use crate::spec::{AxisScale, EstimateStatus, FacetScales, PlotRenderContext, PlotRenderResult};
use crate::theme::ResolvedTheme;
use crate::Plot;

pub(crate) const NOT_ESTIMABLE_TEXT: &str = "Not estimable";

struct AdjacentTextPlan {
    plan: ResolvedTextPlan,
    position: Point2D,
    anchor: TextAnchor,
    deferred_annotation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct EstimateTuple {
    pub(crate) estimate: f64,
    pub(crate) lower: Option<f64>,
    pub(crate) upper: Option<f64>,
}

pub(crate) fn prepare_estimate_tuples(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
    family: &str,
    require_intervals: bool,
) -> Result<Vec<Option<EstimateTuple>>, Error> {
    let estimate_col = chart
        .x_col()
        .ok_or_else(|| Error::Data(format!("{family}: estimate column not set")))?;
    let interval_cols = match chart.interval_bound_cols() {
        Some(columns) => Some(columns),
        None if require_intervals => {
            return Err(Error::Data(format!(
                "{family}: complete interval columns are required"
            )))
        }
        None => None,
    };
    let Some(statuses) = data.estimate_statuses()? else {
        let estimates = crate::data::strict_finite_f64(data, estimate_col, family)?;
        let intervals = interval_cols
            .map(|(lower_col, upper_col)| {
                Ok::<_, Error>((
                    crate::data::strict_finite_f64(data, lower_col, family)?,
                    crate::data::strict_finite_f64(data, upper_col, family)?,
                ))
            })
            .transpose()?;
        if intervals.as_ref().is_some_and(|(lower, upper)| {
            lower.len() != estimates.len() || upper.len() != estimates.len()
        }) {
            return Err(Error::Data(format!(
                "{family}: estimate and interval columns must have equal lengths"
            )));
        }
        return data.collect_source(estimates.into_iter().enumerate().map(|(index, estimate)| {
            let (lower, upper) = intervals.as_ref().map_or((None, None), |(lower, upper)| {
                (lower.get(index).copied(), upper.get(index).copied())
            });
            Ok(Some(EstimateTuple {
                estimate,
                lower,
                upper,
            }))
        }));
    };

    let estimates = crate::data::nullable_finite_f64(data, estimate_col, family)?;
    let intervals = interval_cols
        .map(|(lower_col, upper_col)| {
            Ok::<_, Error>((
                crate::data::nullable_finite_f64(data, lower_col, family)?,
                crate::data::nullable_finite_f64(data, upper_col, family)?,
            ))
        })
        .transpose()?;
    if statuses.len() != estimates.len()
        || intervals.as_ref().is_some_and(|(lower, upper)| {
            lower.len() != estimates.len() || upper.len() != estimates.len()
        })
    {
        return Err(Error::Data(format!(
            "{family}: status, estimate, and interval columns must have equal lengths"
        )));
    }

    data.collect_source(statuses
        .into_iter()
        .enumerate()
        .map(|(index, status)| {
            let estimate = estimates[index];
            let (lower, upper) = intervals.as_ref().map_or((None, None), |(lower, upper)| {
                (lower[index], upper[index])
            });
            match status {
                EstimateStatus::Estimated => {
                    let Some(estimate) = estimate else {
                        return Err(Error::Data(format!(
                            "{family}: EstimateStatus::Estimated requires a complete non-null finite estimate tuple at source row {}",
                            data.source_row(index)
                        )));
                    };
                    if interval_cols.is_some() && (lower.is_none() || upper.is_none()) {
                        return Err(Error::Data(format!(
                            "{family}: EstimateStatus::Estimated requires a complete non-null finite estimate tuple at source row {}",
                            data.source_row(index)
                        )));
                    }
                    Ok(Some(EstimateTuple {
                        estimate,
                        lower,
                        upper,
                    }))
                }
                EstimateStatus::NotEstimable
                    if estimate.is_none() && lower.is_none() && upper.is_none() =>
                {
                    Ok(None)
                }
                EstimateStatus::NotEstimable => Err(Error::Data(format!(
                    "{family}: EstimateStatus::NotEstimable requires the complete estimate tuple to be null at source row {}",
                    data.source_row(index)
                ))),
            }
        }))
}

pub(crate) fn estimate_data_range<'a>(
    estimates: impl IntoIterator<Item = &'a Option<EstimateTuple>>,
) -> Option<(f64, f64)> {
    estimates
        .into_iter()
        .filter_map(|estimate| {
            estimate.map(|tuple| {
                tuple
                    .lower
                    .zip(tuple.upper)
                    .map_or((tuple.estimate, tuple.estimate), |(lower, upper)| {
                        (lower, upper)
                    })
            })
        })
        .fold(None, |range, (lower, upper)| {
            Some(
                range.map_or((lower, upper), |(minimum, maximum): (f64, f64)| {
                    (minimum.min(lower), maximum.max(upper))
                }),
            )
        })
}

pub(crate) fn fixed_facet_estimate_tuples(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    family: &str,
    require_intervals: bool,
) -> Result<Option<Vec<Option<EstimateTuple>>>, Error> {
    if context.data.is_full_plot()
        || !matches!(
            context.facet_scales(),
            Some(FacetScales::Fixed | FacetScales::FreeY)
        )
    {
        return Ok(None);
    }
    let source_rows = chart.configured_source_rows()?;
    let data = source_rows.map_or_else(|| chart.data(), |rows| chart.data().map_source_rows(rows));
    prepare_estimate_tuples(chart, data, family, require_intervals).map(Some)
}

pub(crate) fn plan_not_estimable(
    theme: &ResolvedTheme,
    layout: &Layout,
    vertical_guide_x: Option<f64>,
    scope: &crate::layout::fit::FitScope,
) -> Result<(ResolvedTextPlan, f64), crate::layout::fit::PlanError> {
    let text_plan = theme.measure_legacy_text(
        NOT_ESTIMABLE_TEXT,
        theme.detail_font_size,
        PhysicalTextAlignment::Center,
        "not estimable status",
    )?;
    let x = not_estimable_text_x(
        layout.projection.panel_box(),
        text_plan.width(),
        vertical_guide_x,
        scope,
    )?;
    Ok((text_plan, x))
}

pub(crate) fn draw_planned_not_estimable(
    text_plan: &ResolvedTextPlan,
    x: f64,
    layout: &Layout,
    y: f64,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let (_, y) = layout.projection.project(layout.projection.x_range().0, y);
    backend.draw_planned_text(
        PlannedTextPlacement::middle(Point2D::new(x.round(), y.round()), TextAnchor::Middle, 0.0),
        text_plan,
    )
}

fn not_estimable_text_x(
    panel: crate::layout::PanelBox,
    text_width: f64,
    vertical_guide_x: Option<f64>,
    scope: &crate::layout::fit::FitScope,
) -> Result<f64, crate::layout::fit::PlanError> {
    const PANEL_PADDING: f64 = 6.0;
    const GUIDE_CLEARANCE: f64 = 6.0;

    let left_edge = panel.x0 + PANEL_PADDING;
    let right_edge = panel.x1 - PANEL_PADDING;
    if right_edge - left_edge < text_width {
        return Err(scope.failure(crate::layout::fit::FitKind::Text, format!(
            "Not estimable row text requires {text_width:.1}px but the chart panel provides {:.1}px after padding; increase chart width",
            (right_edge - left_edge).max(0.0)
        )).into());
    }
    let Some(guide_x) = vertical_guide_x else {
        return Ok(f64::midpoint(left_edge, right_edge));
    };
    let left_right = guide_x - GUIDE_CLEARANCE;
    let right_left = guide_x + GUIDE_CLEARANCE;
    let left_width = left_right - left_edge;
    let right_width = right_edge - right_left;
    let left_fits = left_width >= text_width;
    let right_fits = right_width >= text_width;
    match (left_fits, right_fits) {
        (true, true) if right_width > left_width => Ok(f64::midpoint(right_left, right_edge)),
        (true, _) => Ok(f64::midpoint(left_edge, left_right)),
        (false, true) => Ok(f64::midpoint(right_left, right_edge)),
        (false, false) => Err(scope.failure(crate::layout::fit::FitKind::OverlayCollision,
            "Not estimable row text does not fit on either side of the vertical reference or zero line; increase chart width or move the line with exact explicit limits"
        ).into()),
    }
}

pub(crate) fn axis_categories(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
    family: &str,
) -> Result<Vec<String>, Error> {
    let column = chart
        .y_col()
        .ok_or_else(|| Error::Data(format!("{family}: term/label column not set")))?;
    let labels = crate::data::strict_strings(data, column, family)?;
    for (index, label) in labels.iter().enumerate() {
        data.claim_source_visits(index)?;
        if labels[..index].contains(label) {
            return Err(Error::Data(format!(
                "{family}: term/label values must be unique"
            )));
        }
    }
    Ok(labels)
}

pub(crate) fn shared_x_domain(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<(f64, f64), Error> {
    let forest = chart.kind() == crate::spec::PlotKind::Forest;
    let family = if forest { "forest" } else { "coefficient" };
    let reference = if forest {
        chart.forest_reference_value()
    } else {
        chart.coefficient_reference_line_value().unwrap_or(0.0)
    };
    let estimates = prepare_estimate_tuples(chart, data, family, forest)?;
    let transform = |value: f64| -> Result<f64, Error> {
        if !value.is_finite() {
            return Err(Error::Data(format!("{family}: axis values must be finite")));
        }
        match chart.axis_scale_spec() {
            AxisScale::Linear => Ok(value),
            AxisScale::Log10 if value > 0.0 => Ok(value.log10()),
            AxisScale::Log10 => Err(Error::Data(format!("{family}: logarithmic scale requires strictly positive estimates, bounds, and reference values"))),
        }
    };
    let reference = transform(reference)?;
    let mut range: Option<(f64, f64)> = None;
    for estimate in estimates.into_iter().flatten() {
        for value in [Some(estimate.estimate), estimate.lower, estimate.upper]
            .into_iter()
            .flatten()
        {
            let value = transform(value)?;
            range = Some(range.map_or((value, value), |range| {
                (range.0.min(value), range.1.max(value))
            }));
        }
    }
    if let Some(limits) = chart.x_axis_spec().limits {
        let range = (transform(limits.0)?, transform(limits.1)?);
        if range.0 >= range.1 {
            return Err(Error::Data(format!("{family}: logarithmic x-axis limits must remain distinguishable after transformation")));
        }
        return Ok(range);
    }
    let range = range.ok_or_else(|| {
        Error::Data(format!(
            "{family}: all rows are not_estimable; set explicit x-axis limits"
        ))
    })?;
    Ok(crate::layout::continuous_auto_range(
        (range.0.min(reference), range.1.max(reference)),
        &AxisSpec::default(),
    ))
}

#[expect(
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    reason = "validated row counts are converted to plot coordinates"
)]
pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
    family: &'static str,
    require_intervals: bool,
    reference: f64,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let theme = context.theme;
    let estimate_col = chart
        .x_col()
        .ok_or_else(|| Error::Data(format!("{family}: estimate column not set")))?;
    if chart.hue_col().is_some() || chart.color_col().is_some() {
        return Err(Error::Data(format!(
            "{family}: hue and continuous color channels are not supported"
        ))
        .into());
    }
    let labels = axis_categories(chart, context.data, family)?;
    let estimates = prepare_estimate_tuples(chart, context.data, family, require_intervals)?;
    let has_intervals = chart.interval_bound_cols().is_some();
    if chart.estimate_labels_spec().is_some() && has_intervals {
        return Err(Error::Data(format!(
            "{family}: numeric estimate labels are supported only for point-only displays without intervals"
        )).into());
    }
    let mut coefficient_significance = vec![None; labels.len()];
    for annotation in chart.significance_annotations() {
        let Some(target) = annotation.at_target() else {
            return Err(Error::Data(
                "coefficient: only attached significance annotations are supported".into(),
            )
            .into());
        };
        if target.series_name().is_some() {
            return Err(Error::Data(
                "coefficient: series-qualified significance targets are not supported".into(),
            )
            .into());
        }
        let index = labels
            .iter()
            .position(|label| label == target.category_name())
            .ok_or_else(|| {
                Error::Data(format!(
                    "coefficient significance references unknown term {:?}; available terms: {}",
                    target.category_name(),
                    labels.join(", ")
                ))
            })?;
        if coefficient_significance[index].is_some() {
            return Err(Error::Data(format!(
                "coefficient significance target {:?} is annotated more than once",
                target.category_name()
            ))
            .into());
        }
        coefficient_significance[index] = Some(annotation.mark().glyph()?.to_owned());
    }
    if !reference.is_finite() {
        return Err(Error::Data(format!(
            "{family}: reference value must be finite, got {reference}"
        ))
        .into());
    }
    if labels.is_empty() || labels.len() != estimates.len() {
        return Err(Error::Data(format!(
            "{family}: label, estimate, and interval columns must have equal non-zero lengths"
        ))
        .into());
    }
    for (index, estimate) in estimates.iter().enumerate() {
        if estimate.is_some_and(|tuple| {
            tuple
                .lower
                .zip(tuple.upper)
                .is_some_and(|(lower, upper)| lower > tuple.estimate || tuple.estimate > upper)
        }) {
            let source_row = context.data.source_row(index);
            return Err(Error::Data(format!(
                "{family}: source row {source_row} must satisfy lower <= estimate <= upper"
            ))
            .into());
        }
        if estimate.is_none() && coefficient_significance[index].is_some() {
            return Err(Error::Data(format!(
                "coefficient: not_estimable source row {} cannot carry a significance mark",
                context.data.source_row(index)
            ))
            .into());
        }
    }
    let scale = chart.axis_scale_spec();
    let transform = |value: f64| -> Result<f64, Error> {
        match scale {
            AxisScale::Linear => Ok(value),
            AxisScale::Log10 if value > 0.0 => Ok(value.log10()),
            AxisScale::Log10 => Err(Error::Data(format!(
                "{family}: logarithmic scale requires strictly positive estimates, bounds, and reference values"
            ))),
        }
    };
    let transformed_estimates = estimates
        .iter()
        .map(|estimate| {
            estimate
                .map(|tuple| {
                    Ok::<_, Error>(EstimateTuple {
                        estimate: transform(tuple.estimate)?,
                        lower: tuple.lower.map(transform).transpose()?,
                        upper: tuple.upper.map(transform).transpose()?,
                    })
                })
                .transpose()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let transformed_reference = transform(reference)?;
    let row_count = labels.len() as f64;
    let panel = context.panel;
    let layout = context.axis_layout(|| {
    let data_range = estimate_data_range(&transformed_estimates);
    let mut transformed_x_spec = chart.x_axis_spec().clone();
    if scale == AxisScale::Log10 {
        transformed_x_spec.limits = chart
            .x_axis_spec()
            .limits
            .map(|(lower, upper)| Ok::<_, Error>((transform(lower)?, transform(upper)?)))
            .transpose()?;
        if transformed_x_spec
            .limits
            .is_some_and(|(lower, upper)| lower >= upper)
        {
            return Err(Error::Data(format!(
                "{family}: logarithmic x-axis limits must remain distinguishable after transformation"
            )).into());
        }
        transformed_x_spec.exact_ticks = chart.projected_explicit_x_ticks(context.data)?;
    }
    let facet_fallback = if data_range.is_none() && transformed_x_spec.limits.is_none() {
        fixed_facet_estimate_tuples(chart, context, family, require_intervals)?
            .map(|estimates| {
                estimates
                    .into_iter()
                    .map(|estimate| {
                        estimate
                            .map(|tuple| {
                                Ok::<_, Error>(EstimateTuple {
                                    estimate: transform(tuple.estimate)?,
                                    lower: tuple.lower.map(transform).transpose()?,
                                    upper: tuple.upper.map(transform).transpose()?,
                                })
                            })
                            .transpose()
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(|estimates| estimate_data_range(&estimates))
            })
            .transpose()?
            .flatten()
    } else {
        None
    };
    let x_range = if let Some(range) = chart.resolved_facet_x_range() {
        range
    } else if let Some(data_range) = data_range {
        crate::layout::continuous_auto_range(
            (
                data_range.0.min(transformed_reference),
                data_range.1.max(transformed_reference),
            ),
            &transformed_x_spec,
        )
    } else if let Some(data_range) = facet_fallback {
        let provisional = (
            data_range.0.min(transformed_reference),
            data_range.1.max(transformed_reference),
        );
        if provisional.0 < provisional.1 {
            provisional
        } else {
            crate::layout::continuous_auto_range(provisional, &transformed_x_spec)
        }
    } else {
        transformed_x_spec.limits.ok_or_else(|| {
            Error::Data(format!(
                "{family}: all rows are not_estimable; set explicit x-axis limits"
            ))
        })?
    };
    let x_axis = crate::layout::display_axis_spec(&transformed_x_spec, x_range);
    let y_axis = AxisSpec::default().limits(0.0, row_count);
        let y_ticks = labels
            .iter()
            .enumerate()
            .map(|(index, label)| AxisTick::new(row_count - index as f64 - 0.5, label.clone()))
            .collect();
        let log_plan = if x_axis.exact_ticks.is_some() {
            None
        } else if scale == AxisScale::Log10 {
            Some(log_tick_plan(x_range, x_axis.interval, panel, theme, context.fit_scope())?)
        } else {
            None
        };
        let source = log_plan.as_ref().map(|plan| plan.source);
        let x_ticks = log_plan.map_or_else(Vec::new, |plan| plan.ticks);
        let mut layout = context.layout_with_ticks(
            x_range,
            (0.0, row_count),
            panel,
            chart.x_label_text().unwrap_or(estimate_col).to_owned(),
            chart.y_label_text().unwrap_or("").to_owned(),
            x_ticks,
            y_ticks,
            &x_axis,
            &y_axis,
            theme,
        )?;
        if let Some(source) = source {
            layout.x_axis.source = source;
        }
        Ok(layout)
    })?;
    let x_range = layout.projection.x_range();
    let reference_line_x = (transformed_reference >= x_range.0
        && transformed_reference <= x_range.1)
        .then(|| layout.projection.project(transformed_reference, 0.0).0);
    let significance_label = theme
        .typography
        .effective_significance_label(chart.significance_mark_style_override());
    let not_estimable_plan = estimates
        .iter()
        .position(Option::is_none)
        .map(|index| {
            plan_not_estimable(
                theme,
                &layout,
                reference_line_x,
                &context
                    .fit_scope()
                    .child(crate::layout::AddressRole::CategoricalValueLabel, index)?,
            )
        })
        .transpose()?;
    let detail_label =
        theme.legacy_text_appearance(theme.detail_font_size, crate::TextWeight::Normal);
    let frozen_significance = context.deferred_annotation_text();
    let mut frozen_significance_index = 0_usize;
    let adjacent_label_plans = crate::layout::fit::collect(theme, "estimate adjacent labels", (0..labels.len())
        .map(|index| {
            let Some(tuple) = transformed_estimates[index] else {
                return Ok(None);
            };
            let original_estimate = estimates[index]
                .ok_or_else(|| {
                    Error::Render(format!(
                        "{family}: prepared estimated row is missing its original estimate"
                    ))
                })?
                .estimate;
            let adjacent_label = chart
                .estimate_labels_spec()
                .map(|format| (format.format(original_estimate), false))
                .or_else(|| {
                    coefficient_significance[index]
                        .clone()
                        .map(|mark| (mark, true))
                });
            let Some((source, is_significance)) = adjacent_label else {
                return Ok(None);
            };
            let scope = context.fit_scope().child(if is_significance { crate::layout::AddressRole::Annotation } else { crate::layout::AddressRole::CategoricalValueLabel }, index)?;
            let (x, y) = layout
                .projection
                .project(tuple.estimate, row_count - index as f64 - 0.5);
            let (appearance, gap, owner, exact_override) = if is_significance {
                (
                    &significance_label,
                    chart.significance_mark_style_spec().gap_spec(),
                    "coefficient attached significance label",
                    chart.significance_mark_style_override().is_some(),
                )
            } else {
                (&detail_label, 7.0, "coefficient estimate label", false)
            };
            let panel = layout.projection.panel_box();
            let frozen = if is_significance {
                frozen_significance
                    .map(|plans| {
                        let plan = plans.get(frozen_significance_index).ok_or_else(|| {
                            Error::Render(format!(
                                "{family}: frozen attached significance plan count is too short"
                            ))
                        })?;
                        frozen_significance_index += 1;
                        if plan.plan.source != source
                            || plan.plan.style.stack_source != appearance.stack
                            || plan.plan.style.paint != appearance.paint
                            || plan.plan.style.font_size != appearance.font_size
                            || plan.plan.style.weight != appearance.weight
                            || plan.plan.style.exact_override != exact_override
                            || plan.exact_override != exact_override
                        {
                            return Err(Error::Render(format!(
                                "{family}: frozen attached significance plan does not match its source and style"
                            )));
                        }
                        Ok(plan.plan.clone())
                    })
                    .transpose()?
            } else {
                None
            };
            let (position, anchor, plan) = if let Some(plan) = frozen {
                let (position, anchor, fits) = match plan.style.alignment {
                    PhysicalTextAlignment::Start => (
                        Point2D::new((x + gap).round(), y.round()),
                        TextAnchor::Start,
                        x + gap + plan.width() <= panel.x1,
                    ),
                    PhysicalTextAlignment::End => (
                        Point2D::new((x - gap).round(), y.round()),
                        TextAnchor::End,
                        x - gap - plan.width() >= panel.x0,
                    ),
                    PhysicalTextAlignment::Center => {
                        return Err(Error::Render(format!(
                            "{family}: frozen attached significance plan has invalid alignment"
                        )).into());
                    }
                };
                if !fits {
                    return Err(scope.failure(crate::layout::fit::FitKind::OverlayCollision, format!(
                        "{family}: frozen attached significance plan no longer fits beside its point"
                    )).into());
                }
                (position, anchor, plan)
            } else {
                let measure = |alignment| {
                    if is_significance {
                        theme.measure_text(&source, appearance, alignment, owner, exact_override)
                    } else {
                        theme.measure_legacy_appearance_text(
                            &source,
                            appearance,
                            alignment,
                            owner,
                            exact_override,
                        )
                    }
                };
                let start_plan = measure(PhysicalTextAlignment::Start)?;
                let width = start_plan.width();
                if x + gap + width <= panel.x1 {
                    (
                        Point2D::new((x + gap).round(), y.round()),
                        TextAnchor::Start,
                        start_plan,
                    )
                } else if x - gap - width >= panel.x0 {
                    let end_plan = measure(PhysicalTextAlignment::End)?;
                    (
                        Point2D::new((x - gap).round(), y.round()),
                        TextAnchor::End,
                        end_plan,
                    )
                } else {
                    let message = if is_significance {
                        format!(
                            "{family}: significance mark {source:?} does not fit beside its point; increase chart width or widen the x-axis range"
                        )
                    } else {
                        format!(
                            "{family}: formatted estimate label {source:?} does not fit beside its point; increase chart width, shorten the suffix, or reduce precision"
                        )
                    };
                    return Err(scope.failure(crate::layout::fit::FitKind::OverlayCollision, message).into());
                }
            };
            Ok(Some(AdjacentTextPlan {
                plan,
                position,
                anchor,
                deferred_annotation: is_significance,
            }))
        })
        )?;
    if frozen_significance.is_some_and(|plans| plans.len() != frozen_significance_index) {
        return Err(Error::Render(format!(
            "{family}: frozen attached significance plan count is too long"
        ))
        .into());
    }
    with_clip(backend, layout.projection.panel_box(), |backend| {
        let (reference_x, y0) = layout.projection.project(transformed_reference, 0.0);
        let (_, y1) = layout.projection.project(transformed_reference, row_count);
        backend.draw_line(
            &[Point2D::new(reference_x, y0), Point2D::new(reference_x, y1)],
            &LineStyle::medium_dashed(theme.foreground, 1.0),
        )?;
        for index in 0..labels.len() {
            let y = row_count - index as f64 - 0.5;
            let Some(tuple) = transformed_estimates[index] else {
                let (plan, x) = not_estimable_plan
                    .as_ref()
                    .ok_or_else(|| Error::Render("not estimable text plan is missing".into()))?;
                draw_planned_not_estimable(plan, *x, &layout, y, backend)?;
                continue;
            };
            if let Some((lower, upper)) = tuple.lower.zip(tuple.upper) {
                let (left, py) = layout.projection.project(lower, y);
                let (right, _) = layout.projection.project(upper, y);
                backend.draw_line(
                    &[Point2D::new(left, py), Point2D::new(right, py)],
                    &LineStyle::solid(theme.foreground, 1.4),
                )?;
            }
            let (x, y) = layout.projection.project(tuple.estimate, y);
            backend.draw_scatter(
                &[Point2D::new(x, y)],
                &chart.point_style(theme.colors[0], 4),
            )?;
            if let Some(adjacent) = &adjacent_label_plans[index] {
                backend.draw_planned_text(
                    PlannedTextPlacement::middle(adjacent.position, adjacent.anchor, 0.0),
                    &adjacent.plan,
                )?;
            }
        }
        Ok(())
    })?;
    let deferred_annotation_text = adjacent_label_plans
        .into_iter()
        .flatten()
        .filter(|plan| plan.deferred_annotation)
        .map(|plan| crate::layout::DeferredAnnotationTextPlan::from_resolved(plan.plan))
        .collect();
    Ok(PlotRenderResult::new(layout, Vec::new())
        .with_deferred_annotation_text(deferred_annotation_text))
}

pub(crate) fn log_tick_plan(
    range: (f64, f64),
    interval: Option<crate::layout::TickInterval>,
    panel: crate::layout::PanelBox,
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<crate::layout::NumericTickPlan, crate::layout::fit::PlanError> {
    let projection = crate::layout::Projection::new(range, (0.0, 1.0), panel.as_tuple());
    crate::layout::numeric_tick_plan(
        range,
        interval,
        true,
        &projection,
        crate::layout::AxisSide::Bottom,
        theme,
        false,
        scope,
    )
}

#[cfg(test)]
fn log_ticks(
    range: (f64, f64),
    panel: crate::layout::PanelBox,
    theme: &ResolvedTheme,
) -> Result<Vec<AxisTick>, Error> {
    Ok(
        log_tick_plan(range, None, panel, theme, &crate::layout::fit::ROOT_SCOPE)
            .map_err(crate::layout::fit::PlanError::into_error)?
            .ticks,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick_theme() -> ResolvedTheme {
        crate::Theme::Publication.resolve()
    }

    #[test]
    fn not_estimable_text_clears_vertical_guide_by_measured_half_width() {
        let panel = crate::layout::PanelBox::new(0.0, 200.0, 0.0, 100.0);
        let width = 80.0;
        let guide = 100.0;
        let center = not_estimable_text_x(
            panel,
            width,
            Some(guide),
            &crate::layout::fit::FitScope::root(),
        )
        .unwrap();

        assert!(guide - (center + width / 2.0) >= 6.0);
    }

    #[test]
    fn wide_log_ticks_cover_the_full_domain_without_exceeding_the_maximum() {
        let theme = tick_theme();
        let ticks = log_ticks(
            (-100.0, 100.0),
            crate::layout::PanelBox::new(0.0, 2_000.0, 0.0, 100.0),
            &theme,
        )
        .unwrap();

        assert!(ticks.len() > 5);
        assert!(ticks.first().is_some_and(|tick| tick.position <= -90.0));
        assert!(ticks.last().is_some_and(|tick| tick.position >= 90.0));
    }

    #[test]
    fn extreme_log_ticks_keep_every_label_finite_and_nonzero() {
        let theme = tick_theme();
        let ticks = log_ticks(
            (-400.0, 400.0),
            crate::layout::PanelBox::new(0.0, 2_000.0, 0.0, 100.0),
            &theme,
        )
        .unwrap();

        assert!(ticks.iter().all(|tick| {
            tick.label != "0"
                && tick.label != "inf"
                && tick
                    .label
                    .trim_end_matches(['k', 'M'])
                    .parse::<f64>()
                    .is_ok_and(f64::is_finite)
        }));
    }

    #[test]
    fn upper_extreme_log_ticks_have_unique_expression_labels() {
        let theme = tick_theme();
        let ticks = log_ticks(
            (308.0, f64::MAX.log10()),
            crate::layout::PanelBox::new(0.0, 2_000.0, 0.0, 100.0),
            &theme,
        )
        .unwrap();

        assert!(ticks.iter().enumerate().all(|(index, tick)| {
            !ticks[..index]
                .iter()
                .any(|previous| previous.label == tick.label)
                && tick
                    .label
                    .strip_prefix("10^")
                    .is_some_and(|exponent| exponent.parse::<f64>().is_ok_and(f64::is_finite))
        }));
    }

    #[test]
    fn lower_subnormal_log_ticks_omit_duplicate_rounded_values() {
        let lower = f64::from_bits(1).log10();
        let theme = tick_theme();
        let ticks = log_ticks(
            (lower, -323.0),
            crate::layout::PanelBox::new(0.0, 2_000.0, 0.0, 100.0),
            &theme,
        )
        .unwrap();

        assert!(ticks.iter().enumerate().all(|(index, tick)| {
            !ticks[..index]
                .iter()
                .any(|previous| previous.label == tick.label)
        }));
    }
}
