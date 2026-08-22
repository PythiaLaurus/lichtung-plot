//! Shared geometry for distinct coefficient and forest semantic renderers.

use std::collections::HashSet;

use crate::Error;

use crate::layout::{AxisSpec, AxisTick, Layout};
use crate::render::backend::{with_clip, Backend, LineStyle, Point2D, TextAnchor, TextStyle};
use crate::spec::{
    AxisScale, CoefficientSignificanceMark, EstimateStatus, FacetScales, IntervalKind,
    PlotRenderContext, PlotRenderResult,
};
use crate::Plot;

pub(crate) const NOT_ESTIMABLE_TEXT: &str = "Not estimable";

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct EstimateTuple {
    pub(crate) estimate: f64,
    pub(crate) lower: Option<f64>,
    pub(crate) upper: Option<f64>,
}

#[expect(
    clippy::too_many_lines,
    reason = "one preparation boundary keeps exact status parsing and complete/null tuple validation together"
)]
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
    let Some(status_col) = chart.estimate_status_col() else {
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
        return estimates
            .into_iter()
            .enumerate()
            .map(|(index, estimate)| {
                let (lower, upper) = intervals.as_ref().map_or((None, None), |(lower, upper)| {
                    (lower.get(index).copied(), upper.get(index).copied())
                });
                Ok(Some(EstimateTuple {
                    estimate,
                    lower,
                    upper,
                }))
            })
            .collect();
    };

    let statuses = crate::data::strict_strings(data, status_col, family)?
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            EstimateStatus::parse_exact(&value).ok_or_else(|| {
                Error::Data(format!(
                    "{family}: status column {status_col:?} has invalid value {value:?} at source row {}; expected exactly {:?} or {:?}",
                    data.source_row(index),
                    EstimateStatus::Estimated.as_str(),
                    EstimateStatus::NotEstimable.as_str()
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
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

    statuses
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
                            "{family}: status {:?} requires a complete non-null finite estimate tuple at source row {}",
                            EstimateStatus::Estimated.as_str(),
                            data.source_row(index)
                        )));
                    };
                    if interval_cols.is_some() && (lower.is_none() || upper.is_none()) {
                        return Err(Error::Data(format!(
                            "{family}: status {:?} requires a complete non-null finite estimate tuple at source row {}",
                            EstimateStatus::Estimated.as_str(),
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
                    "{family}: status {:?} requires the complete estimate tuple to be null at source row {}",
                    EstimateStatus::NotEstimable.as_str(),
                    data.source_row(index)
                ))),
            }
        })
        .collect()
}

pub(crate) fn estimate_data_range(estimates: &[Option<EstimateTuple>]) -> Option<(f64, f64)> {
    estimates
        .iter()
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
    panel_data: crate::data::PlotData<'_>,
    family: &str,
    require_intervals: bool,
) -> Result<Option<Vec<Option<EstimateTuple>>>, Error> {
    if panel_data.is_full_plot()
        || !matches!(
            chart.facet_scales_spec(),
            FacetScales::Fixed | FacetScales::FreeY
        )
    {
        return Ok(None);
    }
    let source_rows = chart.configured_source_rows()?;
    let data = source_rows
        .as_deref()
        .map_or_else(|| chart.data(), |rows| chart.data().map_source_rows(rows));
    prepare_estimate_tuples(chart, data, family, require_intervals).map(Some)
}

pub(crate) fn draw_not_estimable(
    chart: &Plot,
    layout: &Layout,
    y: f64,
    vertical_guide_x: Option<f64>,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let palette = chart.palette();
    let panel = layout.projection.panel_box();
    let width = crate::layout::estimated_text_width(NOT_ESTIMABLE_TEXT, palette.detail_font_size);
    let x = not_estimable_text_x(panel, width, vertical_guide_x)?;
    let (_, y) = layout.projection.project(layout.projection.x_range().0, y);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "validated panel coordinates are finite and bounded by the canvas"
    )]
    backend.draw_text(
        x.round() as u32,
        y.round() as u32,
        NOT_ESTIMABLE_TEXT,
        &TextStyle {
            color: palette.foreground,
            font_family: palette.font_family,
            font_size: palette.detail_font_size,
            weight: crate::TextWeight::Normal,
            anchor: TextAnchor::Middle,
            rotation: 0.0,
        },
    )
}

fn not_estimable_text_x(
    panel: crate::layout::PanelBox,
    text_width: f64,
    vertical_guide_x: Option<f64>,
) -> Result<f64, Error> {
    const PANEL_PADDING: f64 = 6.0;
    const GUIDE_CLEARANCE: f64 = 6.0;

    let left_edge = panel.x0 + PANEL_PADDING;
    let right_edge = panel.x1 - PANEL_PADDING;
    if right_edge - left_edge < text_width {
        return Err(Error::Render(format!(
            "Not estimable row text requires {text_width:.1}px but the chart panel provides {:.1}px after padding; increase chart width",
            (right_edge - left_edge).max(0.0)
        )));
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
        (false, false) => Err(Error::Render(
            "Not estimable row text does not fit on either side of the vertical reference or zero line; increase chart width or move the line with exact explicit limits"
                .into(),
        )),
    }
}

#[expect(
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "validated row counts and bounded non-negative panel coordinates are converted to plot coordinates"
)]
pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
    family: &'static str,
    require_intervals: bool,
    require_reference: bool,
) -> Result<PlotRenderResult, Error> {
    let estimate_col = chart
        .x_col()
        .ok_or_else(|| Error::Data(format!("{family}: estimate column not set")))?;
    let label_col = chart
        .y_col()
        .ok_or_else(|| Error::Data(format!("{family}: term/label column not set")))?;
    if chart.hue_col().is_some() || chart.color_col().is_some() {
        return Err(Error::Data(format!(
            "{family}: hue and continuous color channels are not supported"
        )));
    }
    let labels = crate::data::strict_strings(context.data, label_col, family)?;
    let estimates = prepare_estimate_tuples(chart, context.data, family, require_intervals)?;
    let has_intervals = chart.interval_bound_cols().is_some();
    if chart.estimate_labels_spec().is_some() && has_intervals {
        return Err(Error::Data(format!(
            "{family}: numeric estimate labels are supported only for point-only displays without intervals"
        )));
    }
    let interval_semantics = if has_intervals {
        Some(chart.interval_semantics_spec().ok_or_else(|| {
            Error::Data(format!(
                "{family}: typed interval semantics are required; call interval_semantics(...)"
            ))
        })?)
    } else {
        None
    };
    let coefficient_significance = if let Some(column) = chart.coefficient_significance_col() {
        if !has_intervals {
            return Err(Error::Data(
                "coefficient: significance marks require complete confidence interval columns"
                    .into(),
            ));
        }
        if interval_semantics.map(crate::IntervalSemantics::kind) != Some(IntervalKind::Confidence)
        {
            return Err(Error::Data(
                "coefficient: significance marks require confidence intervals, not credible intervals"
                    .into(),
            ));
        }
        Some(
            crate::data::strict_strings(context.data, column, "coefficient significance")?
                .into_iter()
                .enumerate()
                .map(|(index, value)| {
                    CoefficientSignificanceMark::parse_exact(&value).ok_or_else(|| {
                        Error::Data(format!(
                            "coefficient: significance column {column:?} has invalid value {value:?} at source row {}; expected exactly \"\", \"*\", \"**\", or \"***\"",
                            context.data.source_row(index)
                        ))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        )
    } else {
        None
    };
    let reference = match chart.reference_line_value() {
        Some(value) if value.is_finite() => value,
        Some(value) => {
            return Err(Error::Data(format!(
                "{family}: reference value must be finite, got {value}"
            )))
        }
        None if require_reference => {
            return Err(Error::Data(format!(
                "{family}: an explicit scientific reference value is required"
            )))
        }
        None => 0.0,
    };
    if labels.is_empty()
        || labels.len() != estimates.len()
        || coefficient_significance
            .as_ref()
            .is_some_and(|marks| marks.len() != labels.len())
    {
        return Err(Error::Data(format!(
            "{family}: label, estimate, and interval columns must have equal non-zero lengths"
        )));
    }
    let mut seen = HashSet::with_capacity(labels.len());
    if labels.iter().any(|label| !seen.insert(label.clone())) {
        return Err(Error::Data(format!(
            "{family}: term/label values must be unique"
        )));
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
            )));
        }
        if estimate.is_none()
            && coefficient_significance
                .as_ref()
                .is_some_and(|marks| !marks[index].as_str().is_empty())
        {
            return Err(Error::Data(format!(
                "coefficient: not_estimable source row {} cannot carry a significance mark",
                context.data.source_row(index)
            )));
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
    let data_range = estimate_data_range(&transformed_estimates);
    let mut transformed_x_spec = chart.x_axis_spec().clone();
    if scale == AxisScale::Log10 {
        transformed_x_spec.limits = chart
            .x_axis_spec()
            .limits
            .map(|(lower, upper)| Ok::<_, Error>((transform(lower)?, transform(upper)?)))
            .transpose()?;
    }
    let facet_fallback = if data_range.is_none() && transformed_x_spec.limits.is_none() {
        fixed_facet_estimate_tuples(chart, context.data, family, require_intervals)?
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
    let row_count = labels.len() as f64;
    let y_axis = AxisSpec::default().limits(0.0, row_count);
    let y_ticks = labels
        .iter()
        .enumerate()
        .map(|(index, label)| AxisTick::new(row_count - index as f64 - 0.5, label.clone()))
        .collect();
    let palette = chart.palette();
    let display_labels =
        chart.resolve_label_domain("y-axis categories", labels.iter().map(String::as_str))?;
    let panel = crate::plots::horizontal_bar::panel_box_for_y_tick_labels(
        chart,
        context.panel,
        &display_labels,
    )?;
    let x_ticks = if scale == AxisScale::Log10 {
        log_ticks(x_range)
    } else {
        Vec::new()
    };
    let layout = if scale == AxisScale::Log10 {
        Layout::new_with_ticks_and_axis_specs_in_panel_box(
            x_range,
            (0.0, row_count),
            panel,
            chart.x_label_text().unwrap_or(estimate_col).to_owned(),
            chart.y_label_text().unwrap_or("").to_owned(),
            x_ticks,
            y_ticks,
            &x_axis,
            &y_axis,
        )
    } else {
        Layout::new_with_ticks_and_axis_specs_in_panel_box(
            x_range,
            (0.0, row_count),
            panel,
            chart.x_label_text().unwrap_or(estimate_col).to_owned(),
            chart.y_label_text().unwrap_or("").to_owned(),
            Vec::new(),
            y_ticks,
            &x_axis,
            &y_axis,
        )
    };
    let reference_line_x = (transformed_reference >= x_range.0
        && transformed_reference <= x_range.1)
        .then(|| layout.projection.project(transformed_reference, 0.0).0);
    with_clip(backend, layout.projection.panel_box(), |backend| {
        let (reference_x, y0) = layout.projection.project(transformed_reference, 0.0);
        let (_, y1) = layout.projection.project(transformed_reference, row_count);
        backend.draw_line(
            &[Point2D::new(reference_x, y0), Point2D::new(reference_x, y1)],
            &LineStyle::dashed(palette.foreground, 1.0, "4 3"),
        )?;
        for index in 0..labels.len() {
            let y = row_count - index as f64 - 0.5;
            let Some(tuple) = transformed_estimates[index] else {
                draw_not_estimable(chart, &layout, y, reference_line_x, backend)?;
                continue;
            };
            if let Some((lower, upper)) = tuple.lower.zip(tuple.upper) {
                let (left, py) = layout.projection.project(lower, y);
                let (right, _) = layout.projection.project(upper, y);
                backend.draw_line(
                    &[Point2D::new(left, py), Point2D::new(right, py)],
                    &LineStyle::solid(palette.foreground, 1.4),
                )?;
            }
            let (x, y) = layout.projection.project(tuple.estimate, y);
            backend.draw_scatter(
                &[Point2D::new(x, y)],
                &chart.point_style(palette.colors[0], 4),
            )?;
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
                    coefficient_significance.as_ref().and_then(|marks| {
                        let mark = marks[index].as_str();
                        (!mark.is_empty()).then(|| (mark.to_owned(), true))
                    })
                });
            if let Some((label, is_significance)) = adjacent_label {
                let width = crate::layout::estimated_text_width(&label, palette.detail_font_size);
                let panel = layout.projection.panel_box();
                let (label_x, anchor) = if x + 7.0 + width <= panel.x1 {
                    (x + 7.0, TextAnchor::Start)
                } else if x - 7.0 - width >= panel.x0 {
                    (x - 7.0, TextAnchor::End)
                } else {
                    let message = if is_significance {
                        format!(
                            "{family}: significance mark {label:?} does not fit beside its point; increase chart width or widen the x-axis range"
                        )
                    } else {
                        format!(
                            "{family}: formatted estimate label {label:?} does not fit beside its point; increase chart width, shorten the suffix, or reduce precision"
                        )
                    };
                    return Err(Error::Render(message));
                };
                backend.draw_text(
                    label_x.round() as u32,
                    y.round() as u32,
                    &label,
                    &TextStyle {
                        color: palette.foreground,
                        font_family: palette.font_family.clone(),
                        font_size: palette.detail_font_size,
                        weight: crate::TextWeight::Normal,
                        anchor,
                        rotation: 0.0,
                    },
                )?;
            }
        }
        Ok(())
    })?;
    Ok(PlotRenderResult::new(layout, Vec::new()))
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "finite logarithmic display powers are bounded to 32 emitted ticks"
)]
fn log_ticks(range: (f64, f64)) -> Vec<AxisTick> {
    let first = range.0.ceil() as i32;
    let last = range.1.floor() as i32;
    let powers = (first..=last).take(32).map(f64::from).collect::<Vec<_>>();
    let positions = if powers.len() >= 2 {
        powers
    } else {
        crate::layout::nice_ticks(range.0, range.1, 4)
    };
    positions
        .into_iter()
        .map(|position| AxisTick::new(position, crate::layout::format_tick(10_f64.powf(position))))
        .collect()
}
