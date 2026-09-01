//! Vertical bar plot implementation.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]

use crate::Error;

use crate::layout::{AxisTick, Layout, Projection};
use crate::plots::bar_data::{self, BarMode, PreparedBar, PreparedBarSignificance, PreparedBars};
use crate::render::backend::{with_clip, Backend, FillStyle, LineStyle, Point2D, TextAnchor};
use crate::spec::{BarSpacing, PlotKind, PlotRenderContext, PlotRenderResult};
use crate::Plot;

/// Render a vertical bar plot.
pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    let theme = context.theme;

    let mode = match chart.kind() {
        PlotKind::StackedBar => BarMode::Stacked,
        _ if chart.group_col().is_some() => BarMode::Grouped,
        _ => BarMode::Single,
    };
    let prepared = bar_data::prepare(chart, context.data, mode)?;
    let legend_entries = bar_data::legend_entries(&prepared, chart, theme);
    let mut base_y_range =
        crate::layout::anchored_auto_range(prepared.value_range, chart.y_axis_spec(), 0.0);
    let mut rendered_maxima = vec![0.0_f64; prepared.categories.len()];
    let anchor_count = prepared.series.len().max(1);
    let mut grouped_maxima = vec![vec![f64::NEG_INFINITY; anchor_count]; prepared.categories.len()];
    for bar in &prepared.bars {
        let maximum = bar
            .interval
            .map_or(bar.start.max(bar.end), |(_, upper)| bar.start.max(upper));
        rendered_maxima[bar.category_index] = rendered_maxima[bar.category_index].max(maximum);
        let anchor = bar.series_index.unwrap_or(0);
        grouped_maxima[bar.category_index][anchor] =
            grouped_maxima[bar.category_index][anchor].max(maximum);
    }
    for point in &prepared.points {
        rendered_maxima[point.category_index] =
            rendered_maxima[point.category_index].max(point.value);
        grouped_maxima[point.category_index][point.series_index.unwrap_or(0)] =
            grouped_maxima[point.category_index][point.series_index.unwrap_or(0)].max(point.value);
    }
    if mode == BarMode::Stacked {
        for (category_index, maximum) in rendered_maxima.iter().enumerate() {
            grouped_maxima[category_index].fill(*maximum);
        }
    }
    let panel = context.panel;
    let panel = crate::plots::horizontal_bar::panel_box_for_x_tick_labels(
        chart,
        panel,
        &prepared.categories,
        theme,
    )?;
    let mark_style = chart.significance_mark_style_spec();
    let significance_label = theme
        .typography
        .effective_significance_label(chart.significance_mark_style_override());
    base_y_range = range_for_bar_marks(chart, &prepared, panel, base_y_range, &significance_label);
    let value_per_pixel = (base_y_range.1 - base_y_range.0) / panel.height();
    for point in &prepared.points {
        let clearance = point.value + f64::from(point.style.radius) * value_per_pixel;
        rendered_maxima[point.category_index] =
            rendered_maxima[point.category_index].max(clearance);
        grouped_maxima[point.category_index][point.series_index.unwrap_or(0)] =
            grouped_maxima[point.category_index][point.series_index.unwrap_or(0)].max(clearance);
    }
    for mark in prepared.significance.iter().filter(|mark| mark.positive) {
        let clearance = mark.edge
            + (mark_style.gap_spec() + significance_label.line_height() + 2.0) * value_per_pixel;
        rendered_maxima[mark.category_index] = rendered_maxima[mark.category_index].max(clearance);
        if let Some(series_index) = mark.series_index {
            grouped_maxima[mark.category_index][series_index] =
                grouped_maxima[mark.category_index][series_index].max(clearance);
        } else {
            for maximum in &mut grouped_maxima[mark.category_index] {
                if maximum.is_finite() {
                    *maximum = maximum.max(clearance);
                }
            }
        }
    }
    let significance = if prepared.series.is_empty() {
        crate::significance::plan(
            chart,
            &prepared.categories,
            &rendered_maxima,
            base_y_range,
            panel,
            theme,
        )?
    } else {
        crate::significance::plan_grouped(
            chart,
            &prepared.categories,
            &prepared.series,
            &grouped_maxima,
            0.76,
            base_y_range,
            panel,
            mode == BarMode::Stacked,
            theme,
        )?
    };
    let y_range = significance.y_range;
    let y_axis_spec = crate::layout::display_axis_spec(chart.y_axis_spec(), y_range);
    let x_count = prepared.categories.len() as f64;
    let x_ticks = prepared
        .categories
        .iter()
        .enumerate()
        .map(|(i, label)| AxisTick::new(i as f64 + 0.5, label.clone()))
        .collect();
    let layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
        (0.0, x_count),
        y_range,
        panel,
        chart.x_label_text().unwrap_or("").to_string(),
        chart.y_label_text().unwrap_or("").to_string(),
        x_ticks,
        Vec::new(),
        chart.x_axis_spec(),
        &y_axis_spec,
        theme,
    );
    validate_bar_significance_collisions(chart, &prepared, mode, &layout, &significance_label)?;

    with_clip(backend, layout.projection.panel_box(), |backend| {
        for bar in &prepared.bars {
            let (x_center, width) = bar_geometry(bar, &prepared, mode, chart.bar_spacing_spec());
            let color_index = bar.hue_index.unwrap_or_else(|| {
                if chart.group_col().is_some() {
                    0
                } else {
                    bar.category_index
                }
            });
            draw_bar(
                backend,
                &layout.projection,
                x_center,
                width,
                bar.start,
                bar.end,
                &FillStyle {
                    color: theme.colors[color_index % theme.colors.len()],
                    stroke: theme.foreground,
                    stroke_width: 1.0,
                    opacity: 1.0,
                },
            )?;
            if let Some((lower, upper)) = bar.interval {
                let style = LineStyle::solid(theme.foreground, theme.interval_width);
                draw_error_bar(
                    backend,
                    &layout.projection,
                    x_center,
                    width,
                    lower,
                    upper,
                    &style,
                )?;
            }
        }
        for mark in &prepared.significance {
            draw_bar_significance(
                chart,
                mark,
                &prepared,
                mode,
                &layout,
                &significance_label,
                backend,
            )?;
        }
        for point in &prepared.points {
            let x_center = point_center(point.category_index, point.series_index, &prepared, mode);
            let projected = layout.projection.project(x_center, point.value);
            backend.draw_scatter(&[Point2D::new(projected.0, projected.1)], &point.style)?;
        }
        Ok(())
    })?;
    if legend_entries.is_empty() {
        Ok(PlotRenderResult::new(layout, Vec::new()).with_significance_plan(significance))
    } else {
        Ok(PlotRenderResult::new(layout, legend_entries).with_significance_plan(significance))
    }
}

fn range_for_bar_marks(
    chart: &Plot,
    prepared: &PreparedBars,
    panel: crate::layout::PanelBox,
    range: (f64, f64),
    label: &crate::theme::TextAppearance,
) -> (f64, f64) {
    if prepared.significance.is_empty() || chart.user_y_limits().is_some() {
        return range;
    }
    let style = chart.significance_mark_style_spec();
    let units_per_pixel = (range.1 - range.0) / panel.height();
    let clearance = (style.gap_spec() + label.line_height() + 2.0) * units_per_pixel;
    let mut extent = prepared.value_range;
    for mark in &prepared.significance {
        if mark.positive {
            extent.1 = extent.1.max(mark.edge + clearance);
        } else {
            extent.0 = extent.0.min(mark.edge - clearance);
        }
    }
    crate::layout::anchored_auto_range(extent, chart.y_axis_spec(), 0.0)
}

fn draw_bar_significance(
    chart: &Plot,
    mark: &PreparedBarSignificance,
    prepared: &PreparedBars,
    mode: BarMode,
    layout: &Layout,
    label: &crate::theme::TextAppearance,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let style = chart.significance_mark_style_spec();
    let x = significance_center(mark.category_index, mark.series_index, prepared, mode);
    let (x, edge_y) = layout.projection.project(x, mark.edge);
    let y = if mark.positive {
        edge_y + style.gap_spec()
    } else {
        edge_y - style.gap_spec() - f64::from(label.font_size)
    };
    let panel = layout.projection.panel_box();
    if y < panel.y0 || y + label.line_height() > panel.y1 {
        return Err(Error::Render(format!(
            "bar significance mark {:?} does not clear its confidence interval and panel boundary; widen the y-axis range",
            mark.label
        )));
    }
    backend.draw_text(
        Point2D::new(x.round(), y.round()),
        &mark.label,
        &label.backend_style(TextAnchor::Middle, 0.0),
    )
}

fn validate_bar_significance_collisions(
    chart: &Plot,
    prepared: &PreparedBars,
    mode: BarMode,
    layout: &Layout,
    label: &crate::theme::TextAppearance,
) -> Result<(), Error> {
    let style = chart.significance_mark_style_spec();
    let mut bounds = Vec::new();
    for mark in &prepared.significance {
        let center = significance_center(mark.category_index, mark.series_index, prepared, mode);
        let (x, edge_y) = layout.projection.project(center, mark.edge);
        let width = crate::layout::estimated_text_width(&mark.label, label.font_size);
        let height = label.line_height();
        let y0 = if mark.positive {
            edge_y + style.gap_spec()
        } else {
            edge_y - style.gap_spec() - height
        };
        let candidate = (x - width / 2.0, x + width / 2.0, y0, y0 + height);
        if bounds.iter().any(|&(x0, x1, y0, y1)| {
            candidate.0 < x1 && x0 < candidate.1 && candidate.2 < y1 && y0 < candidate.3
        }) {
            return Err(Error::Render(
                "bar significance marks overlap; enlarge the chart or reduce mark typography"
                    .into(),
            ));
        }
        bounds.push(candidate);
    }
    Ok(())
}

fn significance_center(
    category_index: usize,
    series_index: Option<usize>,
    prepared: &PreparedBars,
    mode: BarMode,
) -> f64 {
    series_index.map_or(category_index as f64 + 0.5, |series_index| {
        point_center(category_index, Some(series_index), prepared, mode)
    })
}

fn point_center(
    category_index: usize,
    series_index: Option<usize>,
    prepared: &PreparedBars,
    mode: BarMode,
) -> f64 {
    let category_center = category_index as f64 + 0.5;
    if mode != BarMode::Grouped {
        return category_center;
    }
    let series_count = prepared.series.len().max(1) as f64;
    let slot = 0.76 / series_count;
    category_center - 0.76 / 2.0 + slot * (series_index.unwrap_or(0) as f64 + 0.5)
}

fn bar_geometry(
    bar: &PreparedBar,
    prepared: &PreparedBars,
    mode: BarMode,
    spacing: BarSpacing,
) -> (f64, f64) {
    let category_center = bar.category_index as f64 + 0.5;
    match mode {
        BarMode::Single | BarMode::Stacked => (category_center, 0.6),
        BarMode::Grouped => {
            let series_count = prepared.series.len().max(1) as f64;
            let slot_width = 0.76;
            let slot = slot_width / series_count;
            let series_index = bar.series_index.unwrap_or(0) as f64;
            let center = category_center - slot_width / 2.0 + slot * (series_index + 0.5);
            let width = match spacing {
                BarSpacing::Comfort => slot * 0.82,
                BarSpacing::Compact => slot,
            };
            (center, width)
        }
    }
}

fn draw_bar(
    backend: &mut dyn Backend,
    proj: &Projection,
    x_center: f64,
    width: f64,
    y0: f64,
    y1: f64,
    fill: &FillStyle,
) -> Result<(), Error> {
    let (left, bottom) = proj.project(x_center - width / 2.0, y0);
    let (right, top) = proj.project(x_center + width / 2.0, y1);
    backend.draw_polygon(
        &[
            Point2D::new(left, bottom),
            Point2D::new(left, top),
            Point2D::new(right, top),
            Point2D::new(right, bottom),
        ],
        fill,
    )
}

fn draw_error_bar(
    backend: &mut dyn Backend,
    proj: &Projection,
    x_center: f64,
    width: f64,
    lower: f64,
    upper: f64,
    style: &LineStyle,
) -> Result<(), Error> {
    let (x_px, y_top_px) = proj.project(x_center, upper);
    let (_, y_bot_px) = proj.project(x_center, lower);
    let cap_px = (proj.project(width, 0.0).0 - proj.project(0.0, 0.0).0).abs() * 0.2;
    backend.draw_line(
        &[Point2D::new(x_px, y_top_px), Point2D::new(x_px, y_bot_px)],
        style,
    )?;
    backend.draw_line(
        &[
            Point2D::new(x_px - cap_px, y_top_px),
            Point2D::new(x_px + cap_px, y_top_px),
        ],
        style,
    )?;
    backend.draw_line(
        &[
            Point2D::new(x_px - cap_px, y_bot_px),
            Point2D::new(x_px + cap_px, y_bot_px),
        ],
        style,
    )
}
