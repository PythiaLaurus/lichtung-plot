//! Interaction profiles from precomputed estimates and intervals.

use std::collections::HashSet;

use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::layout::{AxisTick, Layout};
use crate::render::backend::{with_clip, Backend, LineStyle, Point2D, PointStyle};
use crate::spec::{LinePattern, PlotRenderContext, PlotRenderResult, PointShape};
use crate::Plot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct TraceStyle {
    point_shape: PointShape,
    line_pattern: LinePattern,
}

#[expect(
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    reason = "validated category indices and panel coordinates are bounded by the canvas"
)]
pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    chart.interaction_source_spec().ok_or_else(|| {
        Error::Data("interaction: estimate source is required; call interaction_source(...)".into())
    })?;
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("interaction: x factor column not set".into()))?;
    let estimate_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("interaction: estimate column not set".into()))?;
    chart
        .group_col()
        .ok_or_else(|| Error::Data("interaction: graphical group column not set".into()))?;
    let (lower_col, upper_col) = chart.interval_bound_cols().ok_or_else(|| {
        Error::Data("interaction: complete precomputed interval columns are required".into())
    })?;
    chart.interval_semantics_spec().ok_or_else(|| {
        Error::Data(
            "interaction: typed interval semantics are required; call interval_semantics(...)"
                .into(),
        )
    })?;
    let x = crate::data::strict_strings(context.data, x_col, "interaction")?;
    let channels = crate::data::collective_channels(
        context.data,
        chart.group_col(),
        chart.hue_col(),
        None,
        "interaction",
    )?;
    let trace = channels.groups.as_ref().ok_or_else(|| {
        Error::Data("interaction: graphical group channel did not resolve".into())
    })?;
    let estimate = crate::data::strict_finite_f64(context.data, estimate_col, "interaction")?;
    let lower = crate::data::strict_finite_f64(context.data, lower_col, "interaction")?;
    let upper = crate::data::strict_finite_f64(context.data, upper_col, "interaction")?;
    if x.is_empty()
        || [trace.len(), estimate.len(), lower.len(), upper.len()]
            .iter()
            .any(|length| *length != x.len())
    {
        return Err(Error::Data(
            "interaction: all channels must have equal non-zero lengths".into(),
        ));
    }
    let mut x_levels = Vec::new();
    let trace_levels = chart.resolved_group_levels(channels.group_levels.clone());
    let hue_levels = chart.resolved_hue_levels(channels.hue_levels.clone());
    let trace_styles = resolve_trace_styles(chart, context, trace, &trace_levels)?;
    let mut cells = HashSet::new();
    for index in 0..x.len() {
        if lower[index] > estimate[index] || estimate[index] > upper[index] {
            let source_row = context.data.source_row(index);
            return Err(Error::Data(format!(
                "interaction: source row {source_row} must satisfy lower <= estimate <= upper"
            )));
        }
        if !cells.insert((x[index].clone(), trace[index].clone())) {
            let source_row = context.data.source_row(index);
            return Err(Error::Data(format!(
                "interaction: duplicate x/trace cell {:?}/{:?} at source row {source_row}",
                x[index], trace[index]
            )));
        }
        if !x_levels.contains(&x[index]) {
            x_levels.push(x[index].clone());
        }
    }
    let y_range = crate::layout::continuous_auto_range(
        crate::data::min_max(lower.iter().chain(&upper).copied(), "interaction")?,
        chart.y_axis_spec(),
    );
    let y_axis = crate::layout::display_axis_spec(chart.y_axis_spec(), y_range);
    let x_ticks = x_levels
        .iter()
        .enumerate()
        .map(|(index, label)| AxisTick::new(index as f64 + 0.5, label.clone()))
        .collect();
    let panel = context.panel;
    let panel = crate::plots::horizontal_bar::panel_box_for_x_tick_labels(chart, panel, &x_levels)?;
    let layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
        (0.0, x_levels.len() as f64),
        y_range,
        panel,
        chart.x_label_text().unwrap_or(x_col).to_owned(),
        chart.y_label_text().unwrap_or(estimate_col).to_owned(),
        x_ticks,
        Vec::new(),
        chart.x_axis_spec(),
        &y_axis,
    );
    with_clip(backend, layout.projection.panel_box(), |backend| {
        for (trace_index, trace_level) in trace_levels.iter().enumerate() {
            let hue = (0..trace.len())
                .find(|row| &trace[*row] == trace_level)
                .and_then(|row| channels.hues.as_ref().map(|hues| hues[row].as_str()));
            let color_index = hue
                .and_then(|hue| hue_levels.iter().position(|level| level == hue))
                .unwrap_or(0);
            let color =
                chart.categorical_color("interaction", hue.unwrap_or(trace_level), color_index)?;
            let mut points = Vec::new();
            for (x_index, x_level) in x_levels.iter().enumerate() {
                if let Some(row) =
                    (0..x.len()).find(|row| &x[*row] == x_level && &trace[*row] == trace_level)
                {
                    let (px, py) = layout
                        .projection
                        .project(x_index as f64 + 0.5, estimate[row]);
                    let (_, low) = layout.projection.project(x_index as f64 + 0.5, lower[row]);
                    let (_, high) = layout.projection.project(x_index as f64 + 0.5, upper[row]);
                    backend.draw_line(
                        &[Point2D::new(px, low), Point2D::new(px, high)],
                        &LineStyle::solid(color, 1.2),
                    )?;
                    points.push(Point2D::new(px, py));
                }
            }
            if chart.interaction_connectors_visible() && points.len() > 1 {
                let line_style = trace_styles.as_ref().map_or_else(
                    || LineStyle::solid(color, 1.8),
                    |styles| styles[trace_index].line_pattern.line_style(color, 1.8),
                );
                backend.draw_line(&points, &line_style)?;
            }
            if chart.markers_visible() {
                let point_shape = trace_styles
                    .as_ref()
                    .map_or(PointShape::Circle, |styles| styles[trace_index].point_shape);
                backend.draw_scatter(
                    &points,
                    &chart.point_style(color, 4).with_shape(point_shape),
                )?;
            }
        }
        Ok(())
    })?;
    let legends = if let Some(styles) = trace_styles {
        trace_levels
            .iter()
            .zip(styles)
            .map(|(trace_level, style)| {
                let hue = (0..trace.len())
                    .find(|row| &trace[*row] == trace_level)
                    .and_then(|row| channels.hues.as_ref().map(|hues| hues[row].as_str()));
                let color_index = hue
                    .and_then(|hue| hue_levels.iter().position(|level| level == hue))
                    .unwrap_or(0);
                let color = chart.categorical_color(
                    "interaction",
                    hue.unwrap_or(trace_level),
                    color_index,
                )?;
                Ok(LegendEntry {
                    label: trace_level.clone(),
                    style: PointStyle::new(color, 3).with_shape(style.point_shape),
                    line_style: Some(style.line_pattern.line_style(color, 1.8)),
                })
            })
            .collect::<Result<Vec<_>, Error>>()?
    } else {
        hue_levels
            .iter()
            .enumerate()
            .map(|(index, label)| {
                Ok(LegendEntry::point(
                    label.clone(),
                    PointStyle::new(chart.categorical_color("interaction", label, index)?, 3),
                ))
            })
            .collect::<Result<Vec<_>, Error>>()?
    };
    Ok(PlotRenderResult::new(layout, legends))
}

fn resolve_trace_styles(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    traces: &[String],
    trace_levels: &[String],
) -> Result<Option<Vec<TraceStyle>>, Error> {
    let Some((point_shape_col, line_pattern_col)) = chart.interaction_style_cols() else {
        return Ok(None);
    };
    let point_shapes = crate::data::strict_strings(context.data, point_shape_col, "interaction")?;
    let line_patterns = crate::data::strict_strings(context.data, line_pattern_col, "interaction")?;
    if point_shapes.len() != traces.len() || line_patterns.len() != traces.len() {
        return Err(Error::Data(
            "interaction: trace style columns must align one-to-one with trace rows".into(),
        ));
    }

    let mut styles = vec![None; trace_levels.len()];
    for row in 0..traces.len() {
        let point_shape = PointShape::parse_exact(&point_shapes[row]).ok_or_else(|| {
            Error::Data(format!(
                "interaction: point-shape column {point_shape_col:?} has invalid value {:?} at source row {}; expected exactly {:?} or {:?}",
                point_shapes[row],
                context.data.source_row(row),
                PointShape::Circle.as_str(),
                PointShape::TriangleUp.as_str()
            ))
        })?;
        let line_pattern = LinePattern::parse_exact(&line_patterns[row]).ok_or_else(|| {
            Error::Data(format!(
                "interaction: line-pattern column {line_pattern_col:?} has invalid value {:?} at source row {}; expected exactly {:?} or {:?}",
                line_patterns[row],
                context.data.source_row(row),
                LinePattern::Solid.as_str(),
                LinePattern::Dashed.as_str()
            ))
        })?;
        let style = TraceStyle {
            point_shape,
            line_pattern,
        };
        let index = trace_levels
            .iter()
            .position(|level| level == &traces[row])
            .ok_or_else(|| {
                Error::Data(format!(
                    "interaction: trace {:?} at source row {} is absent from the resolved trace order",
                    traces[row],
                    context.data.source_row(row)
                ))
            })?;
        if styles[index].is_some_and(|existing| existing != style) {
            return Err(Error::Data(format!(
                "interaction: trace {:?} has inconsistent point-shape or line-pattern values at source row {}; each trace requires one exact style pair",
                traces[row],
                context.data.source_row(row)
            )));
        }
        styles[index] = Some(style);
    }

    styles
        .into_iter()
        .zip(trace_levels)
        .map(|(style, trace)| {
            style.ok_or_else(|| {
                Error::Data(format!(
                    "interaction: trace {trace:?} has no point-shape and line-pattern values"
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}
