//! Interaction profiles from precomputed estimates and intervals.

use std::collections::HashSet;

use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::layout::{AxisTick, Layout};
use crate::render::backend::{with_clip, Backend, LineStyle, Point2D};
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
    let x = crate::data::strict_strings(context.data, x_col, "interaction")?;
    let channels = crate::data::observed_collectives(
        context.data,
        chart.group_col(),
        chart.group_name(),
        chart.hue_col(),
        chart.hue_name(),
        None,
        "interaction",
    )?;
    let estimate = crate::data::strict_finite_f64(context.data, estimate_col, "interaction")?;
    let lower = crate::data::strict_finite_f64(context.data, lower_col, "interaction")?;
    let upper = crate::data::strict_finite_f64(context.data, upper_col, "interaction")?;
    if x.is_empty()
        || [
            channels.rows.len(),
            estimate.len(),
            lower.len(),
            upper.len(),
        ]
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
    let identities = chart.resolved_collective_pairs(channels.identities.clone());
    let trace_styles = resolve_trace_styles(chart, &trace_levels)?;
    let mut cells = HashSet::new();
    for index in 0..x.len() {
        if lower[index] > estimate[index] || estimate[index] > upper[index] {
            let source_row = context.data.source_row(index);
            return Err(Error::Data(format!(
                "interaction: source row {source_row} must satisfy lower <= estimate <= upper"
            )));
        }
        if !cells.insert((x[index].clone(), channels.rows[index].clone())) {
            let source_row = context.data.source_row(index);
            return Err(Error::Data(format!(
                "interaction: duplicate (condition, group, hue) cell ({:?}, {:?}, {:?}) at source row {source_row}",
                x[index], channels.rows[index].group, channels.rows[index].hue
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
    let panel = crate::plots::horizontal_bar::panel_box_for_x_tick_labels(
        chart,
        panel,
        &x_levels,
        context.theme,
    )?;
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
        for identity in &identities {
            if !channels.rows.contains(identity) {
                continue;
            }
            let trace_level = identity.group.as_ref().ok_or_else(|| {
                Error::Data("interaction: graphical group channel did not resolve".into())
            })?;
            let trace_index = trace_levels
                .iter()
                .position(|level| level == trace_level)
                .ok_or_else(|| {
                    Error::Data(format!(
                        "interaction: graphical group {trace_level:?} is absent from the resolved order"
                    ))
                })?;
            let hue = identity.hue.as_deref();
            let color_index = hue
                .and_then(|hue| hue_levels.iter().position(|level| level == hue))
                .unwrap_or(0);
            let color = chart.categorical_color(
                "interaction",
                hue.unwrap_or(trace_level),
                color_index,
                context.theme,
            )?;
            let mut points = Vec::new();
            for (x_index, x_level) in x_levels.iter().enumerate() {
                if let Some(row) =
                    (0..x.len()).find(|row| &x[*row] == x_level && &channels.rows[*row] == identity)
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
    let legends = identities
        .iter()
        .filter(|identity| channels.rows.contains(identity))
        .filter(|identity| identity.hue.is_some() || trace_styles.is_some())
        .map(|identity| {
            let trace_level = identity.group.as_ref().ok_or_else(|| {
                Error::Render("interaction legend identity has no graphical group".into())
            })?;
            let trace_index = trace_levels
                .iter()
                .position(|level| level == trace_level)
                .ok_or_else(|| {
                    Error::Render("interaction legend group is absent from resolved order".into())
                })?;
            let hue = identity.hue.as_deref();
            let color_index = hue
                .and_then(|hue| hue_levels.iter().position(|level| level == hue))
                .unwrap_or(0);
            let color = chart.categorical_color(
                "interaction",
                hue.unwrap_or(trace_level),
                color_index,
                context.theme,
            )?;
            let style = trace_styles
                .as_ref()
                .map_or(TraceStyle::default(), |styles| styles[trace_index]);
            let legend_identity = chart.collective_legend_identity(identity)?;
            let point = chart.point_style(color, 4).with_shape(style.point_shape);
            let line = style.line_pattern.line_style(color, 1.8);
            match (
                chart.markers_visible(),
                chart.interaction_connectors_visible(),
            ) {
                (true, true) => Ok(LegendEntry::point_line(legend_identity, point, line)),
                (true, false) => Ok(LegendEntry::point(legend_identity, point)),
                (false, true) => Ok(LegendEntry::line(legend_identity, line)),
                (false, false) => Err(Error::Render(
                    "interaction legend has no visible glyph component".into(),
                )),
            }
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(PlotRenderResult::new(layout, legends))
}

fn resolve_trace_styles(
    chart: &Plot,
    trace_levels: &[String],
) -> Result<Option<Vec<TraceStyle>>, Error> {
    let Some(configured) = chart.interaction_styles_spec() else {
        return Ok(None);
    };

    let mut styles = vec![None; trace_levels.len()];
    for configured_style in configured {
        let style = TraceStyle {
            point_shape: configured_style.point_shape(),
            line_pattern: configured_style.line_pattern(),
        };
        let index = trace_levels
            .iter()
            .position(|level| level == configured_style.group())
            .ok_or_else(|| {
                Error::Data(format!(
                    "interaction: style references unknown graphical group {:?}",
                    configured_style.group()
                ))
            })?;
        if styles[index].is_some() {
            return Err(Error::Data(format!(
                "interaction: graphical group {:?} has more than one configured style",
                configured_style.group()
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
                    "interaction: graphical group {trace:?} has no configured style"
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}
