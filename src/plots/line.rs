//! Ordered numeric series with optional caller-provided ribbons.

use std::collections::HashSet;

use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::layout::{AxisTick, Layout};
use crate::render::backend::{
    with_clip, Backend, FillStyle, LineStyle, Point2D, PointStyle, TextAnchor,
};
use crate::spec::{AxisDomain, LinePathStyle, PlotRenderContext, PlotRenderResult};
use crate::theme::ResolvedTheme;
use crate::Plot;

#[derive(Debug)]
struct SeriesData {
    identity: crate::data::CollectivePair,
    hue_index: Option<usize>,
    points: Vec<SeriesPoint>,
}

#[derive(Debug)]
struct PreparedPointLayer {
    points: Vec<(f64, f64)>,
    labels: Option<Vec<String>>,
    legend_label: Option<String>,
    style: PointStyle,
}

type SeriesPoint = (f64, f64, Option<(f64, f64)>);

#[derive(Debug)]
struct TrajectoryData {
    groups: Vec<TrajectoryGroup>,
    hue_levels: Vec<String>,
    condition_levels: Option<Vec<String>>,
}

#[derive(Debug)]
struct TrajectoryGroup {
    identity: Option<crate::data::CollectivePair>,
    points: Vec<TrajectoryPoint>,
    hue_index: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
struct TrajectoryPoint {
    x: f64,
    y: f64,
    hue_index: Option<usize>,
}

pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let theme = context.theme;
    render_series(chart, context, theme, backend)
}

pub(crate) fn render_trajectory(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let theme = context.theme;
    render_grouped(chart, context, theme, backend, false)
}

pub(crate) fn render_path(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let theme = context.theme;
    render_grouped(chart, context, theme, backend, true)
}

fn render_series(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let (series, primary_color_slots) = prepare(chart, context.data)?;
    let mut text = context.producer_text()?;
    let point_layers = prepare_point_layers(chart, primary_color_slots, theme)?;
    let layout = context.axis_layout(|| {
        let x_range = crate::layout::continuous_auto_range(
            crate::data::min_max(
                series
                    .iter()
                    .flat_map(|series| series.points.iter().map(|point| point.0))
                    .chain(
                        point_layers
                            .iter()
                            .flat_map(|layer| layer.points.iter().map(|point| point.0)),
                    ),
                "line",
            )?,
            chart.x_axis_spec(),
        );
        let y_data_range = crate::data::min_max(
            series
                .iter()
                .flat_map(|series| {
                    series.points.iter().flat_map(|point| {
                        point
                            .2
                            .map_or([point.1, point.1], |(lower, upper)| [lower, upper])
                    })
                })
                .chain(
                    point_layers
                        .iter()
                        .flat_map(|layer| layer.points.iter().map(|point| point.1)),
                ),
            "line",
        )?;
        let y_range = crate::layout::continuous_auto_range(y_data_range, chart.y_axis_spec());
        let x_axis = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
        let y_axis = crate::layout::display_axis_spec(chart.y_axis_spec(), y_range);
        let panel = context.panel;
        context.layout_with_axis_specs(
            x_range,
            y_range,
            panel,
            chart.x_label_text().unwrap_or("").to_owned(),
            chart.y_label_text().unwrap_or("").to_owned(),
            &x_axis,
            &y_axis,
            theme,
        )
    })?;
    draw_series(chart, &layout, &series, theme, backend)?;
    draw_point_layers(&layout, &point_layers, theme, &mut text, backend)?;

    let mut legends = series
        .iter()
        .filter(|series| series.identity.hue.is_some())
        .map(|series| {
            let color = theme.colors[series.hue_index.unwrap_or(0) % theme.colors.len()];
            let identity = chart.collective_legend_identity(&series.identity)?;
            let point = chart.point_style(color, 3);
            let line = LineStyle::solid(color, 1.8);
            let fill = FillStyle {
                color,
                stroke: color,
                stroke_width: 0.0,
                opacity: 0.18,
            };
            let has_fill = series.points.iter().all(|point| point.2.is_some());
            Ok(match (chart.markers_visible(), has_fill) {
                (true, true) => LegendEntry::point_line_fill(identity, point, line, fill),
                (true, false) => LegendEntry::point_line(identity, point, line),
                (false, true) => LegendEntry::line_fill(identity, line, fill),
                (false, false) => LegendEntry::line(identity, line),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    legends.extend(point_layers.iter().filter_map(|layer| {
        layer.legend_label.as_ref().map(|label| {
            LegendEntry::point(chart.legend_identity(label.clone()), layer.style.clone())
                .without_hue_level()
        })
    }));
    let mut result = PlotRenderResult::new(layout, legends);
    result.producer_text = text.finish()?;
    Ok(result)
}

pub(crate) fn shared_x_domain(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<(f64, f64), Error> {
    let (ranges, categories) = axis_inputs(chart, data, None)?;
    if categories.is_some() {
        return Err(Error::Data(
            "canvas x-range group has a categorical x axis".into(),
        ));
    }
    Ok(ranges[0])
}

#[expect(
    clippy::type_complexity,
    reason = "two scientific ranges and optional ordered condition levels"
)]
pub(crate) fn axis_inputs(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
    facet: Option<crate::data::FacetAxisSources<'_>>,
) -> Result<([(f64, f64); 2], Option<Vec<String>>), Error> {
    // Every retained observation contributes to these domains. Series sorting,
    // duplicate checks, and path/colour payloads belong to mark preparation.
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("line: x column not set".into()))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("line: y column not set".into()))?;
    let y_values = crate::data::strict_finite_f64(data, y_col, "line axis")?;
    data.claim_source_visits(y_values.len())?;
    let mut y = crate::data::min_max(y_values, "line axis")?;
    let mut levels = None;
    let mut x = if data.column(x_col)?.is_numeric() {
        let values = crate::data::strict_finite_f64(data, x_col, "line axis")?;
        data.claim_source_visits(values.len())?;
        crate::data::min_max(values, "line axis")?
    } else if chart.kind() == crate::PlotKind::Trajectory {
        let labels =
            crate::data::strict_normalized_strings(data, x_col, "trajectory", "condition")?;
        let categories = if let Some(global) = facet.map_or_else(
            || chart.global_line_condition_levels(),
            |sources| sources.conditions,
        ) {
            data.collect_source(global.iter().map(|label| data.copy_source_string(label)))?
        } else {
            data.source_levels(&labels)?
        };
        let range = (0.0, categorical_index(categories.len())?);
        levels = Some(categories);
        range
    } else {
        return Err(Error::Data("line axis: x must be numeric".into()));
    };
    if chart.kind() == crate::PlotKind::Line {
        if let Some((lower, upper)) = chart.interval_bound_cols() {
            let lower = crate::data::strict_finite_f64(data, lower, "line axis")?;
            let upper = crate::data::strict_finite_f64(data, upper, "line axis")?;
            data.claim_source_visits(lower.len())?;
            data.claim_source_visits(upper.len())?;
            y = crate::data::min_max(lower.into_iter().chain(upper), "line axis")?;
        }
        for (index, layer) in chart.point_layers().iter().enumerate() {
            data.claim_source_visits(1)?;
            let mut buffer = [0_u8; 64];
            let mut cursor = std::io::Cursor::new(buffer.as_mut_slice());
            std::io::Write::write_fmt(&mut cursor, format_args!("line point layer {index}"))
                .expect("bounded layer index");
            let length = usize::try_from(cursor.position()).expect("bounded layer index");
            let context = std::str::from_utf8(&buffer[..length]).expect("ASCII layer index");
            let xr = data.source_numeric_range(layer.x_values(), context)?;
            let yr = data.source_numeric_range(layer.y_values(), context)?;
            x = (x.0.min(xr.0), x.1.max(xr.1));
            y = (y.0.min(yr.0), y.1.max(yr.1));
        }
    }
    if levels.is_none() {
        x = crate::layout::continuous_auto_range(x, chart.x_axis_spec());
    }
    Ok((
        [
            x,
            crate::layout::continuous_auto_range(y, chart.y_axis_spec()),
        ],
        levels,
    ))
}

fn prepare_point_layers(
    chart: &Plot,
    primary_color_slots: usize,
    theme: &ResolvedTheme,
) -> Result<Vec<PreparedPointLayer>, Error> {
    chart
        .point_layers()
        .iter()
        .enumerate()
        .map(|(index, layer)| {
            let context = format!("line point layer {index}");
            let x = crate::data::strict_numeric_values(layer.x_values(), &context)?;
            let y = crate::data::strict_numeric_values(layer.y_values(), &context)?;
            if x.is_empty() || x.len() != y.len() {
                return Err(Error::Data(format!(
                    "{context}: x and y columns must have equal non-zero lengths"
                )));
            }
            let labels = layer
                .label_values()
                .map(|values| crate::data::strict_category_values(values, &context))
                .transpose()?;
            if labels
                .as_ref()
                .is_some_and(|labels| labels.len() != x.len())
            {
                return Err(Error::Data(format!(
                    "{context}: labels must align one-to-one with x and y"
                )));
            }
            Ok(PreparedPointLayer {
                points: x.into_iter().zip(y).collect(),
                labels,
                legend_label: layer.legend_label_text().map(str::to_owned),
                style: layer
                    .point_style(theme.colors[(primary_color_slots + index) % theme.colors.len()]),
            })
        })
        .collect()
}

fn draw_point_layers(
    layout: &Layout,
    layers: &[PreparedPointLayer],
    theme: &ResolvedTheme,
    text: &mut crate::spec::ProducerText<'_>,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    if layers.is_empty() {
        return Ok(());
    }
    with_clip(backend, layout.projection.panel_box(), |backend| {
        for (layer_index, layer) in layers.iter().enumerate() {
            let projected = layer
                .points
                .iter()
                .map(|point| {
                    let projected = layout.projection.project(point.0, point.1);
                    Point2D::new(projected.0, projected.1)
                })
                .collect::<Vec<_>>();
            backend.draw_scatter(&projected, &layer.style)?;
            if let Some(labels) = &layer.labels {
                let x_range = layout.projection.x_range();
                let y_range = layout.projection.y_range();
                let label_offset = f64::from(layer.style.radius) + 2.0;
                for ((source, point), label) in layer.points.iter().zip(&projected).zip(labels) {
                    if source.0 < x_range.0
                        || source.0 > x_range.1
                        || source.1 < y_range.0
                        || source.1 > y_range.1
                    {
                        continue;
                    }
                    let owner = format!("line point layer {layer_index} label");
                    let mut appearance =
                        theme.legacy_text_appearance(10, crate::TextWeight::Normal);
                    appearance.line_height_multiplier = 1.2;
                    appearance.paint = layer.style.color.into();
                    let plan = text
                        .plan(label, &owner, || {
                            Ok(theme.measure_legacy_appearance_text(
                                label,
                                &appearance,
                                crate::layout::PhysicalTextAlignment::Start,
                                &owner,
                                false,
                            )?)
                        })
                        .map_err(crate::layout::fit::PlanError::into_error)?;
                    text.draw(
                        crate::render::backend::PlacedText::middle(
                            plan,
                            Point2D::new(
                                (point.x + label_offset).max(0.0).round(),
                                (point.y + label_offset).max(0.0).round(),
                            ),
                            TextAnchor::Start,
                            0.0,
                        ),
                        backend,
                    )?;
                }
            }
        }
        Ok(())
    })
}

fn render_grouped(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
    ordered_path: bool,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let trajectory = prepare_trajectory(chart, context.data, ordered_path)?;
    let family = if ordered_path {
        "line path"
    } else {
        "trajectory"
    };
    let x_domain = if trajectory.condition_levels.is_some() {
        AxisDomain::Categorical
    } else {
        AxisDomain::Continuous
    };
    let layout = context.axis_layout(|| {
        let x_values = trajectory
            .groups
            .iter()
            .flat_map(|group| group.points.iter().map(|point| point.x));
        let (x_range, x_ticks) = trajectory.condition_levels.as_ref().map_or_else(
            || {
                let range = crate::data::min_max(x_values, family)?;
                Ok::<_, Error>((
                    crate::layout::continuous_auto_range(range, chart.x_axis_spec()),
                    Vec::new(),
                ))
            },
            |levels| {
                let ticks = levels
                    .iter()
                    .enumerate()
                    .map(|(index, label)| {
                        categorical_index(index)
                            .map(|index| AxisTick::new(index + 0.5, label.clone()))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(((0.0, categorical_index(levels.len())?), ticks))
            },
        )?;
        let y_range = crate::layout::continuous_auto_range(
            crate::data::min_max(
                trajectory
                    .groups
                    .iter()
                    .flat_map(|group| group.points.iter().map(|point| point.y)),
                family,
            )?,
            chart.y_axis_spec(),
        );
        let x_axis = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
        let y_axis = crate::layout::display_axis_spec(chart.y_axis_spec(), y_range);
        let panel = context.panel;
        context.layout_with_ticks(
            x_range,
            y_range,
            panel,
            chart.x_label_text().unwrap_or("").to_owned(),
            chart.y_label_text().unwrap_or("").to_owned(),
            x_ticks,
            Vec::new(),
            &x_axis,
            &y_axis,
            theme,
        )
    })?;
    let hue_paths = ordered_path && chart.line_path_style_spec() == LinePathStyle::Hue;
    with_clip(backend, layout.projection.panel_box(), |backend| {
        for group in &trajectory.groups {
            let projected = group
                .points
                .iter()
                .map(|point| {
                    let (x, y) = layout.projection.project(point.x, point.y);
                    Point2D::new(x, y)
                })
                .collect::<Vec<_>>();
            let color = if hue_paths {
                hue_path_color(chart, &trajectory.hue_levels, group.hue_index, theme)?
            } else {
                theme.foreground
            };
            backend.draw_line(&projected, &LineStyle::solid(color, 1.4))?;
        }
        if !ordered_path || chart.markers_visible() {
            for group in &trajectory.groups {
                for point in &group.points {
                    let (x, y) = layout.projection.project(point.x, point.y);
                    let color = if hue_paths {
                        hue_path_color(chart, &trajectory.hue_levels, group.hue_index, theme)?
                    } else {
                        point.hue_index.map_or(theme.colors[0], |index| {
                            theme.colors[index % theme.colors.len()]
                        })
                    };
                    backend.draw_scatter(&[Point2D::new(x, y)], &chart.point_style(color, 4))?;
                }
            }
        }
        Ok(())
    })?;
    let legends = trajectory_legends(chart, &trajectory, theme, hue_paths)?;
    Ok(PlotRenderResult::new(layout, legends).with_axis_domains(x_domain, AxisDomain::Continuous))
}

fn trajectory_legends(
    chart: &Plot,
    trajectory: &TrajectoryData,
    theme: &ResolvedTheme,
    hue_paths: bool,
) -> Result<Vec<LegendEntry>, Error> {
    if hue_paths {
        return trajectory
            .groups
            .iter()
            .filter_map(|group| group.identity.as_ref().map(|identity| (group, identity)))
            .map(|(group, identity)| {
                let color = hue_path_color(chart, &trajectory.hue_levels, group.hue_index, theme)?;
                let identity = chart.collective_legend_identity(identity)?;
                let line = LineStyle::solid(color, 1.4);
                if chart.markers_visible() {
                    Ok(LegendEntry::point_line(
                        identity,
                        chart.point_style(color, 4),
                        line,
                    ))
                } else {
                    Ok(LegendEntry::line(identity, line))
                }
            })
            .collect();
    }
    if chart.kind() == crate::PlotKind::LinePath && !chart.markers_visible() {
        return Ok(Vec::new());
    }
    trajectory
        .hue_levels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let color = theme.colors[index % theme.colors.len()];
            Ok(LegendEntry::point(
                chart.legend_identity(label.clone()),
                chart.point_style(color, 4),
            ))
        })
        .collect()
}

fn hue_path_color(
    chart: &Plot,
    hue_levels: &[String],
    hue_index: Option<usize>,
    theme: &ResolvedTheme,
) -> Result<(u8, u8, u8), Error> {
    let index = hue_index.ok_or_else(|| {
        Error::Render("line path: hue-styled path has no resolved hue index".into())
    })?;
    let label = hue_levels.get(index).ok_or_else(|| {
        Error::Render("line path: hue-styled path index exceeds resolved hues".into())
    })?;
    chart.categorical_color("line path", label, index, theme)
}

fn draw_series(
    chart: &Plot,
    layout: &Layout,
    series: &[SeriesData],
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    with_clip(backend, layout.projection.panel_box(), |backend| {
        for series in series {
            let color_index = series.hue_index.unwrap_or(0);
            let color = theme.colors[color_index % theme.colors.len()];
            if series.points.iter().all(|point| point.2.is_some()) {
                let mut polygon = series
                    .points
                    .iter()
                    .map(|point| {
                        let (x, y) = layout.projection.project(point.0, point.2.unwrap().1);
                        Point2D::new(x, y)
                    })
                    .collect::<Vec<_>>();
                polygon.extend(series.points.iter().rev().map(|point| {
                    let (x, y) = layout.projection.project(point.0, point.2.unwrap().0);
                    Point2D::new(x, y)
                }));
                backend.draw_polygon(
                    &polygon,
                    &FillStyle {
                        color,
                        stroke: color,
                        stroke_width: 0.0,
                        opacity: 0.18,
                    },
                )?;
            }
            let projected = series
                .points
                .iter()
                .map(|point| {
                    let (x, y) = layout.projection.project(point.0, point.1);
                    Point2D::new(x, y)
                })
                .collect::<Vec<_>>();
            backend.draw_line(&projected, &LineStyle::solid(color, 1.8))?;
            if chart.markers_visible() {
                backend.draw_scatter(&projected, &chart.point_style(color, 3))?;
            }
        }
        Ok(())
    })
}

fn prepare<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
) -> Result<(Vec<SeriesData>, usize), Error> {
    let data = data.into();
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("line: x column not set".into()))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("line: y column not set".into()))?;
    let x = crate::data::strict_finite_f64(data, x_col, "line")?;
    let y = crate::data::strict_finite_f64(data, y_col, "line")?;
    let bounds = match chart.interval_bound_cols() {
        Some((lower, upper)) => Some((
            crate::data::strict_finite_f64(data, lower, "line")?,
            crate::data::strict_finite_f64(data, upper, "line")?,
        )),
        None => None,
    };
    let channels = crate::data::observed_collectives(
        data,
        chart.group_col(),
        chart.group_name(),
        chart.hue_col(),
        chart.hue_name(),
        None,
        "line",
    )?;
    if x.is_empty() || x.len() != y.len() || channels.rows.len() != x.len() {
        return Err(Error::Data(
            "line: aligned x, y, group, and hue columns must have equal non-zero lengths".into(),
        ));
    }
    if bounds
        .as_ref()
        .is_some_and(|(lower, upper)| lower.len() != x.len() || upper.len() != x.len())
    {
        return Err(Error::Data(
            "line: interval columns must align with x and y".into(),
        ));
    }

    let hue_levels = chart.resolved_hue_levels(channels.hue_levels.clone());
    let identities = chart.resolved_collective_pairs(channels.identities.clone());
    let mut output = Vec::with_capacity(identities.len());
    for identity in identities {
        let mut points = Vec::new();
        for index in 0..x.len() {
            if channels.rows[index] != identity {
                continue;
            }
            let interval = bounds
                .as_ref()
                .map(|(lower, upper)| (lower[index], upper[index]));
            if interval.is_some_and(|(lower, upper)| lower > y[index] || y[index] > upper) {
                let source_row = data.source_row(index);
                return Err(Error::Data(format!(
                    "line: source row {source_row} must satisfy lower <= y <= upper"
                )));
            }
            points.push((x[index], y[index], interval));
        }
        if points.is_empty() {
            continue;
        }
        points.sort_by(|left, right| left.0.total_cmp(&right.0));
        let mut seen = HashSet::with_capacity(points.len());
        if points.iter().any(|point| {
            let canonical = if point.0 == 0.0 { 0 } else { point.0.to_bits() };
            !seen.insert(canonical)
        }) {
            return Err(Error::Data(format!(
                "line: duplicate x value within series {}",
                identity.description(channels.collapsed)
            )));
        }
        let hue_index = identity
            .hue
            .as_ref()
            .and_then(|hue| hue_levels.iter().position(|level| level == hue));
        output.push(SeriesData {
            identity,
            hue_index,
            points,
        });
    }
    let primary_color_slots = if chart.hue_col().is_some() {
        hue_levels.len()
    } else {
        1
    };
    Ok((output, primary_color_slots))
}

#[expect(
    clippy::too_many_lines,
    reason = "trajectory normalization, duplicate validation, and stable grouping form one ordered preflight"
)]
fn prepare_trajectory<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
    ordered_path: bool,
) -> Result<TrajectoryData, Error> {
    let data = data.into();
    let context = if ordered_path {
        "line path"
    } else {
        "trajectory"
    };
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data(format!("{context}: x column not set")))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data(format!("{context}: y column not set")))?;
    let group_col = chart.group_col().ok_or_else(|| {
        Error::Data(format!(
            "{context}: explicit graphical group column not set"
        ))
    })?;
    let channels = crate::data::observed_collectives(
        data,
        Some(group_col),
        chart.group_name(),
        chart.hue_col(),
        chart.hue_name(),
        None,
        context,
    )?;
    let groups = channels
        .rows
        .iter()
        .map(|identity| {
            identity.group.clone().ok_or_else(|| {
                Error::Data(format!(
                    "{context}: graphical group channel did not resolve"
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let hues = chart.hue_col().map(|_| {
        channels
            .rows
            .iter()
            .map(|identity| {
                identity
                    .hue
                    .clone()
                    .ok_or_else(|| Error::Data(format!("{context}: hue channel did not resolve")))
            })
            .collect::<Result<Vec<_>, Error>>()
    });
    let hues = hues.transpose()?;
    let y = crate::data::strict_finite_f64(data, y_col, context)?;
    let x_source = data
        .column(x_col)
        .map_err(|error| Error::Data(format!("{context}: column {x_col:?}: {error}")))?;
    let (x, condition_labels, condition_levels) = if x_source.is_numeric() {
        (
            crate::data::strict_finite_f64(data, x_col, context)?,
            None,
            None,
        )
    } else if ordered_path {
        return Err(Error::Data(
            "line path: x must be numeric so source-ordered path geometry remains explicit".into(),
        ));
    } else {
        let labels =
            crate::data::strict_normalized_strings(data, x_col, "trajectory", "condition")?;
        let levels = chart.resolved_line_condition_levels(crate::data::stable_levels(&labels));
        let positions = labels
            .iter()
            .map(|label| {
                levels
                    .iter()
                    .position(|level| level == label)
                    .ok_or_else(|| {
                        Error::Data(format!(
                            "trajectory: condition {label:?} is absent from global order"
                        ))
                    })
                    .and_then(categorical_index)
                    .map(|index| index + 0.5)
            })
            .collect::<Result<Vec<_>, _>>()?;
        (positions, Some(labels), Some(levels))
    };
    let row_count = groups.len();
    if row_count == 0
        || x.len() != row_count
        || y.len() != row_count
        || hues
            .as_ref()
            .is_some_and(|values| values.len() != row_count)
    {
        return Err(Error::Data(format!(
            "{context}: aligned group, condition, y, and hue columns must have equal non-zero lengths"
        )));
    }

    if !ordered_path {
        let mut seen_numeric = HashSet::with_capacity(row_count);
        let mut seen_categorical = HashSet::with_capacity(row_count);
        for row in 0..row_count {
            let duplicate = condition_labels.as_ref().map_or_else(
                || {
                    let normalized = if x[row] == 0.0 { 0 } else { x[row].to_bits() };
                    !seen_numeric.insert((groups[row].clone(), normalized))
                },
                |labels| !seen_categorical.insert((groups[row].clone(), labels[row].clone())),
            );
            if duplicate {
                return Err(Error::Data(format!(
                    "trajectory: duplicate normalized (group, condition) ({:?}, {:?}) at source row {}",
                    groups[row],
                    condition_labels
                        .as_ref()
                        .map_or_else(|| x[row].to_string(), |labels| labels[row].clone()),
                    data.source_row(row)
                )));
            }
        }
    }

    let observed_group_levels = crate::data::stable_levels(&groups);
    let group_levels = chart.resolved_group_levels(observed_group_levels);
    let observed_hue_levels = hues
        .as_ref()
        .map_or_else(Vec::new, |values| crate::data::stable_levels(values));
    let hue_levels = chart.resolved_hue_levels(observed_hue_levels);
    if ordered_path && chart.line_path_style_spec() == LinePathStyle::Hue {
        if hues.is_none() {
            return Err(Error::Data(
                "line path: LinePathStyle::Hue requires a categorical hue column".into(),
            ));
        }
        let identities = chart.resolved_collective_pairs(channels.identities.clone());
        let mut output = Vec::new();
        for identity in identities {
            let rows = (0..row_count)
                .filter(|row| channels.rows[*row] == identity)
                .collect::<Vec<_>>();
            if rows.is_empty() {
                continue;
            }
            if rows.len() < 2 {
                return Err(Error::Data(format!(
                    "line path: {} requires at least two retained points",
                    identity.description(channels.collapsed)
                )));
            }
            let hue = identity.hue.as_ref().ok_or_else(|| {
                Error::Data(
                    "line path: LinePathStyle::Hue requires a categorical hue column".into(),
                )
            })?;
            let hue_index = hue_levels
                .iter()
                .position(|level| level == hue)
                .ok_or_else(|| {
                    Error::Data(format!(
                        "line path: hue {hue:?} is absent from the resolved hue order"
                    ))
                })?;
            let points = rows
                .into_iter()
                .map(|row| TrajectoryPoint {
                    x: x[row],
                    y: y[row],
                    hue_index: Some(hue_index),
                })
                .collect();
            output.push(TrajectoryGroup {
                identity: Some(identity.clone()),
                points,
                hue_index: Some(hue_index),
            });
        }
        return Ok(TrajectoryData {
            groups: output,
            hue_levels,
            condition_levels,
        });
    }
    let mut output = Vec::new();
    for label in group_levels {
        let group_rows = (0..row_count)
            .filter(|row| groups[*row] == label)
            .collect::<Vec<_>>();
        if group_rows.is_empty() {
            continue;
        }
        let mut points = (0..row_count)
            .filter(|row| groups[*row] == label)
            .map(|row| TrajectoryPoint {
                x: x[row],
                y: y[row],
                hue_index: hues
                    .as_ref()
                    .and_then(|values| hue_levels.iter().position(|level| level == &values[row])),
            })
            .collect::<Vec<_>>();
        if points.len() < 2 {
            return Err(Error::Data(format!(
                "trajectory: graphical group {label:?} requires at least two retained conditions"
            )));
        }
        if !ordered_path {
            points.sort_by(|left, right| left.x.total_cmp(&right.x));
        }
        output.push(TrajectoryGroup {
            identity: None,
            points,
            hue_index: None,
        });
    }
    Ok(TrajectoryData {
        groups: output,
        hue_levels,
        condition_levels,
    })
}

fn categorical_index(index: usize) -> Result<f64, Error> {
    u32::try_from(index).map(f64::from).map_err(|_| {
        Error::Data(
            "trajectory: categorical condition count exceeds the supported coordinate range".into(),
        )
    })
}

#[cfg(test)]
mod axis_tests {
    use super::*;
    use crate::{CategoryValues, NumericValues};

    #[test]
    fn domain_only_sources_match_grouped_mark_preparation_and_refuse_before_work() {
        let x = NumericValues::new("x", [2.0, 1.0, 2.0, 1.0]);
        let y = NumericValues::new("y", [20.0, 10.0, -40.0, -30.0]);
        let groups = CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap();
        let line = Plot::line(x.clone(), y.clone())
            .group(groups.clone())
            .erase();
        let numeric = Plot::trajectory(x, y.clone(), groups.clone()).erase();
        let categorical = Plot::trajectory(
            CategoryValues::new("condition", [" after ", "before", "after", "before"]).unwrap(),
            y,
            groups,
        )
        .erase();
        let resources = crate::font::canonical::OperationGenerationResources::new();
        for plot in [&line, &numeric, &categorical] {
            let (ranges, categories) =
                axis_inputs(plot, plot.data().with_axis_resources(&resources), None).unwrap();
            if plot.kind() == crate::PlotKind::Line {
                let (series, _) = prepare(plot, plot.data()).unwrap();
                assert_eq!(
                    ranges,
                    [
                        crate::layout::continuous_auto_range(
                            crate::data::min_max(
                                series
                                    .iter()
                                    .flat_map(|series| series.points.iter().map(|point| point.0)),
                                "test"
                            )
                            .unwrap(),
                            plot.x_axis_spec()
                        ),
                        crate::layout::continuous_auto_range(
                            crate::data::min_max(
                                series
                                    .iter()
                                    .flat_map(|series| series.points.iter().map(|point| point.1)),
                                "test"
                            )
                            .unwrap(),
                            plot.y_axis_spec()
                        ),
                    ]
                );
            } else {
                let prepared = prepare_trajectory(plot, plot.data(), false).unwrap();
                assert_eq!(categories, prepared.condition_levels);
                assert_eq!(
                    ranges[1],
                    crate::layout::continuous_auto_range(
                        crate::data::min_max(
                            prepared
                                .groups
                                .iter()
                                .flat_map(|group| group.points.iter().map(|point| point.y)),
                            "test"
                        )
                        .unwrap(),
                        plot.y_axis_spec()
                    )
                );
            }
            for (visits, bytes) in [(0, 1000), (1000, 0)] {
                let resources =
                    crate::font::canonical::OperationGenerationResources::with_test_limits(
                        visits, bytes,
                    );
                let error = axis_inputs(plot, plot.data().with_axis_resources(&resources), None)
                    .unwrap_err();
                assert!(matches!(error, Error::Render(_)), "{error}");
            }
        }
    }
}
