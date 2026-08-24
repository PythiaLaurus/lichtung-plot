//! Horizontal bar plot implementation.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]

use crate::Error;

use crate::layout::{AxisTick, Layout, PanelBox, Projection};
use crate::plots::bar_data::{self, BarMode, PreparedBar, PreparedBarSignificance, PreparedBars};
use crate::render::backend::{
    with_clip, Backend, FillStyle, LineStyle, Point2D, TextAnchor, TextStyle,
};
use crate::spec::{BarSpacing, PlotKind, PlotRenderContext, PlotRenderResult};
use crate::Plot;

pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    let palette = chart.palette();

    let mode = match chart.kind() {
        PlotKind::StackedHorizontalBar => BarMode::Stacked,
        _ if chart.group_col().is_some() => BarMode::Grouped,
        _ => BarMode::Single,
    };
    let prepared = bar_data::prepare(chart, context.data, mode)?;
    let display_categories = chart.resolve_label_domain(
        "y-axis categories",
        prepared.categories.iter().map(String::as_str),
    )?;
    let legend_entries = bar_data::legend_entries(&prepared, chart);
    let mut base_x_range =
        crate::layout::anchored_auto_range(prepared.value_range, chart.x_axis_spec(), 0.0);
    let y_count = prepared.categories.len() as f64;
    let y_ticks = prepared
        .categories
        .iter()
        .enumerate()
        .map(|(i, label)| {
            AxisTick::new(
                i as f64 + 0.5,
                if chart.y_tick_labels_visible() {
                    label.clone()
                } else {
                    String::new()
                },
            )
        })
        .collect();
    let semantic_panel = context.panel;
    let panel_box = if chart.y_tick_labels_visible() {
        panel_box_for_y_tick_labels(chart, semantic_panel, &display_categories)?
    } else {
        semantic_panel
    };
    base_x_range = range_for_bar_marks(chart, &prepared, panel_box, base_x_range);
    let anchor_count = prepared.series.len().max(1);
    let mut rendered_maxima =
        vec![vec![f64::NEG_INFINITY; anchor_count]; prepared.categories.len()];
    for bar in &prepared.bars {
        let anchor = bar.series_index.unwrap_or(0);
        rendered_maxima[bar.category_index][anchor] = rendered_maxima[bar.category_index][anchor]
            .max(
                bar.interval
                    .map_or(bar.start.max(bar.end), |(_, upper)| bar.start.max(upper)),
            );
    }
    for point in &prepared.points {
        rendered_maxima[point.category_index][point.series_index.unwrap_or(0)] =
            rendered_maxima[point.category_index][point.series_index.unwrap_or(0)].max(point.value);
    }
    let value_per_pixel = (base_x_range.1 - base_x_range.0) / panel_box.width();
    for point in &prepared.points {
        let clearance = point.value + f64::from(point.style.radius) * value_per_pixel;
        rendered_maxima[point.category_index][point.series_index.unwrap_or(0)] =
            rendered_maxima[point.category_index][point.series_index.unwrap_or(0)].max(clearance);
    }
    let mark_style = chart.significance_mark_style_spec();
    for mark in prepared.significance.iter().filter(|mark| mark.positive) {
        let width = crate::layout::estimated_text_width(&mark.label, mark_style.font_size_spec());
        let clearance = mark.edge + (mark_style.gap_spec() + width + 2.0) * value_per_pixel;
        if let Some(series_index) = mark.series_index {
            rendered_maxima[mark.category_index][series_index] =
                rendered_maxima[mark.category_index][series_index].max(clearance);
        } else {
            for maximum in &mut rendered_maxima[mark.category_index] {
                if maximum.is_finite() {
                    *maximum = maximum.max(clearance);
                }
            }
        }
    }
    if mode == BarMode::Stacked {
        for maxima in &mut rendered_maxima {
            let maximum = maxima.iter().copied().fold(0.0_f64, f64::max);
            maxima.fill(maximum);
        }
    }
    let significance = crate::significance::plan_horizontal(
        chart,
        &prepared.categories,
        &prepared.series,
        &rendered_maxima,
        0.74,
        base_x_range,
        panel_box,
        mode == BarMode::Stacked,
    )?;
    let x_range = significance.y_range;
    let x_axis_spec = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
    let layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
        x_range,
        (0.0, y_count),
        panel_box,
        chart.x_label_text().unwrap_or("").to_string(),
        chart.y_label_text().unwrap_or("").to_string(),
        Vec::new(),
        y_ticks,
        &x_axis_spec,
        chart.y_axis_spec(),
    );
    validate_bar_significance_collisions(chart, &prepared, mode, &layout)?;

    with_clip(backend, layout.projection.panel_box(), |backend| {
        if chart.zero_line_visible() && x_range.0 <= 0.0 && x_range.1 >= 0.0 {
            let (x, y0) = layout.projection.project(0.0, 0.0);
            let (_, y1) = layout.projection.project(0.0, y_count);
            backend.draw_line(
                &[Point2D::new(x, y0), Point2D::new(x, y1)],
                &LineStyle::solid(palette.foreground, 0.8),
            )?;
        }
        for bar in &prepared.bars {
            let (y_center, height) = bar_geometry(bar, &prepared, mode, chart.bar_spacing_spec());
            let color_index = bar.hue_index.unwrap_or_else(|| {
                if chart.group_col().is_some() {
                    0
                } else {
                    bar.category_index
                }
            });
            let color = if chart.colors_by_sign() {
                if bar.end >= bar.start {
                    palette.colors[0]
                } else {
                    palette.colors[1 % palette.colors.len()]
                }
            } else {
                palette.colors[color_index % palette.colors.len()]
            };
            draw_bar(
                backend,
                &layout.projection,
                bar.start,
                bar.end,
                y_center,
                height,
                &FillStyle {
                    color,
                    stroke: palette.foreground,
                    stroke_width: 1.0,
                    opacity: 1.0,
                },
            )?;
            if let Some((lower, upper)) = bar.interval {
                let style = LineStyle {
                    color: palette.foreground,
                    width: palette.axis_width,
                    dash: String::new(),
                };
                draw_error_bar(
                    backend,
                    &layout.projection,
                    lower,
                    upper,
                    y_center,
                    height,
                    &style,
                )?;
            }
        }
        for mark in &prepared.significance {
            draw_bar_significance(chart, mark, &prepared, mode, &layout, backend)?;
        }
        for point in &prepared.points {
            let y_center = point_center(point.category_index, point.series_index, &prepared, mode);
            let projected = layout.projection.project(point.value, y_center);
            backend.draw_scatter(&[Point2D::new(projected.0, projected.1)], &point.style)?;
        }
        Ok(())
    })?;
    if legend_entries.is_empty() {
        Ok(PlotRenderResult::new(layout, Vec::new())
            .with_significance_brackets(significance.brackets))
    } else {
        Ok(PlotRenderResult::new(layout, legend_entries)
            .with_significance_brackets(significance.brackets))
    }
}

fn range_for_bar_marks(
    chart: &Plot,
    prepared: &PreparedBars,
    panel: PanelBox,
    range: (f64, f64),
) -> (f64, f64) {
    if prepared.significance.is_empty() || chart.user_x_limits().is_some() {
        return range;
    }
    let style = chart.significance_mark_style_spec();
    let units_per_pixel = (range.1 - range.0) / panel.width();
    let mut extent = prepared.value_range;
    for mark in &prepared.significance {
        let width = crate::layout::estimated_text_width(&mark.label, style.font_size_spec());
        let clearance = (style.gap_spec() + width + 2.0) * units_per_pixel;
        if mark.positive {
            extent.1 = extent.1.max(mark.edge + clearance);
        } else {
            extent.0 = extent.0.min(mark.edge - clearance);
        }
    }
    crate::layout::anchored_auto_range(extent, chart.x_axis_spec(), 0.0)
}

fn draw_bar_significance(
    chart: &Plot,
    mark: &PreparedBarSignificance,
    prepared: &PreparedBars,
    mode: BarMode,
    layout: &Layout,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let style = chart.significance_mark_style_spec();
    let y = significance_center(mark.category_index, mark.series_index, prepared, mode);
    let (edge_x, y) = layout.projection.project(mark.edge, y);
    let width = crate::layout::estimated_text_width(&mark.label, style.font_size_spec());
    let panel = layout.projection.panel_box();
    let (x, anchor, fits) = if mark.positive {
        let x = edge_x + style.gap_spec();
        (x, TextAnchor::Start, x + width <= panel.x1)
    } else {
        let x = edge_x - style.gap_spec();
        (x, TextAnchor::End, x - width >= panel.x0)
    };
    if !fits {
        return Err(Error::Render(format!(
            "horizontal bar significance mark {:?} does not clear its confidence interval and panel boundary; widen the x-axis range",
            mark.label
        )));
    }
    backend.draw_text(
        x.round() as u32,
        y.round() as u32,
        &mark.label,
        &TextStyle {
            color: chart.palette().foreground,
            font_family: chart.palette().font_family,
            font_size: style.font_size_spec(),
            weight: style.weight_spec(),
            anchor,
            rotation: 0.0,
        },
    )
}

fn validate_bar_significance_collisions(
    chart: &Plot,
    prepared: &PreparedBars,
    mode: BarMode,
    layout: &Layout,
) -> Result<(), Error> {
    let style = chart.significance_mark_style_spec();
    let mut bounds = Vec::new();
    for mark in &prepared.significance {
        let y = significance_center(mark.category_index, mark.series_index, prepared, mode);
        let (edge_x, y) = layout.projection.project(mark.edge, y);
        let width = crate::layout::estimated_text_width(&mark.label, style.font_size_spec());
        let height = f64::from(style.font_size_spec()) * 1.2;
        let (x0, x1) = if mark.positive {
            (edge_x + style.gap_spec(), edge_x + style.gap_spec() + width)
        } else {
            (edge_x - style.gap_spec() - width, edge_x - style.gap_spec())
        };
        let candidate = (x0, x1, y - height / 2.0, y + height / 2.0);
        if bounds.iter().any(|&(x0, x1, y0, y1)| {
            candidate.0 < x1 && x0 < candidate.1 && candidate.2 < y1 && y0 < candidate.3
        }) {
            return Err(Error::Render(
                "horizontal bar significance marks overlap; enlarge the chart or reduce mark typography"
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
    let slot = 0.74 / series_count;
    category_center - 0.74 / 2.0 + slot * (series_index.unwrap_or(0) as f64 + 0.5)
}

fn bar_geometry(
    bar: &PreparedBar,
    prepared: &PreparedBars,
    mode: BarMode,
    spacing: BarSpacing,
) -> (f64, f64) {
    let category_center = bar.category_index as f64 + 0.5;
    match mode {
        BarMode::Single | BarMode::Stacked => (category_center, 0.55),
        BarMode::Grouped => {
            let series_count = prepared.series.len().max(1) as f64;
            let slot_width = 0.74;
            let slot = slot_width / series_count;
            let series_index = bar.series_index.unwrap_or(0) as f64;
            let center = category_center - slot_width / 2.0 + slot * (series_index + 0.5);
            let height = match spacing {
                BarSpacing::Comfort => slot * 0.82,
                BarSpacing::Compact => slot,
            };
            (center, height)
        }
    }
}

fn draw_bar(
    backend: &mut dyn Backend,
    proj: &Projection,
    x0: f64,
    x1: f64,
    y_center: f64,
    height: f64,
    fill: &FillStyle,
) -> Result<(), Error> {
    let (left, bottom) = proj.project(x0, y_center - height / 2.0);
    let (right, top) = proj.project(x1, y_center + height / 2.0);
    backend.draw_polygon(
        &[
            Point2D::new(left, bottom),
            Point2D::new(right, bottom),
            Point2D::new(right, top),
            Point2D::new(left, top),
        ],
        fill,
    )
}

fn draw_error_bar(
    backend: &mut dyn Backend,
    proj: &Projection,
    lower: f64,
    upper: f64,
    y_center: f64,
    height: f64,
    style: &LineStyle,
) -> Result<(), Error> {
    let (x_left, y_px) = proj.project(lower, y_center);
    let (x_right, _) = proj.project(upper, y_center);
    let cap_px = (proj.project(0.0, height).1 - proj.project(0.0, 0.0).1).abs() * 0.2;
    backend.draw_line(
        &[Point2D::new(x_left, y_px), Point2D::new(x_right, y_px)],
        style,
    )?;
    backend.draw_line(
        &[
            Point2D::new(x_left, y_px - cap_px),
            Point2D::new(x_left, y_px + cap_px),
        ],
        style,
    )?;
    backend.draw_line(
        &[
            Point2D::new(x_right, y_px - cap_px),
            Point2D::new(x_right, y_px + cap_px),
        ],
        style,
    )
}

pub(crate) fn panel_box_for_y_tick_labels(
    chart: &Plot,
    panel: PanelBox,
    labels: &[String],
) -> Result<PanelBox, Error> {
    panel_box_for_y_tick_labels_impl(chart, panel, labels, true)
}

pub(crate) fn panel_box_for_elided_y_tick_labels(
    chart: &Plot,
    panel: PanelBox,
    labels: &[String],
) -> Result<PanelBox, Error> {
    panel_box_for_y_tick_labels_impl(chart, panel, labels, false)
}

fn panel_box_for_y_tick_labels_impl(
    chart: &Plot,
    mut panel: PanelBox,
    labels: &[String],
    validate_row_height: bool,
) -> Result<PanelBox, Error> {
    if labels.is_empty() || chart.composed_axis_text_is_measured() {
        return Ok(panel);
    }
    let palette = chart.palette();
    let blocks = y_tick_label_blocks(chart, panel, labels)?;
    let max_label_width = blocks
        .iter()
        .map(crate::layout::TextBlock::width)
        .fold(0.0_f64, f64::max);
    if max_label_width <= 0.0 {
        return Ok(panel);
    }
    if validate_row_height {
        validate_y_tick_label_row_height(panel, labels.len(), &blocks)?;
    }

    let min_panel_w = 80.0;
    let required_left = crate::layout::y_axis_left_inset_for_tick_labels(
        max_label_width,
        palette.tick_length,
        palette.label_font_size,
    );
    let base_left = chart.base_figure_layout().panel_box.x0;
    let additional_inset = (required_left - base_left.min(panel.x0)).max(0.0);
    if additional_inset > 0.0 {
        panel.x0 = (panel.x0 + additional_inset).min(panel.x1 - min_panel_w);
    }
    Ok(panel)
}

pub(crate) fn validate_y_tick_label_rows(
    chart: &Plot,
    panel: PanelBox,
    labels: &[String],
) -> Result<(), Error> {
    if labels.is_empty() {
        return Ok(());
    }
    let blocks = y_tick_label_blocks(chart, panel, labels)?;
    validate_y_tick_label_row_height(panel, labels.len(), &blocks)
}

fn y_tick_label_blocks(
    chart: &Plot,
    panel: PanelBox,
    labels: &[String],
) -> Result<Vec<crate::layout::TextBlock>, Error> {
    let palette = chart.palette();
    let maximum_width = (panel.x1
        - 80.0
        - f64::from(palette.tick_length)
        - f64::from(palette.label_font_size)
        - 20.0)
        .max(1.0);
    labels
        .iter()
        .map(|label| {
            crate::layout::TextBlock::layout(
                label,
                palette.tick_font_size,
                maximum_width,
                "y-axis category label",
            )
        })
        .collect()
}

fn validate_y_tick_label_row_height(
    panel: PanelBox,
    label_count: usize,
    blocks: &[crate::layout::TextBlock],
) -> Result<(), Error> {
    let row_height = panel.height() / label_count.max(1) as f64;
    if blocks.iter().any(|block| block.height() > row_height) {
        return Err(Error::Render(
            "multiline y-axis category labels do not fit their rows; increase chart height or shorten labels"
                .into(),
        ));
    }
    Ok(())
}

pub(crate) fn panel_box_for_x_tick_labels(
    chart: &Plot,
    mut panel: PanelBox,
    labels: &[String],
) -> Result<PanelBox, Error> {
    if labels.is_empty() || chart.axis_text_is_measured() {
        return Ok(panel);
    }
    let palette = chart.palette();
    let display =
        chart.resolve_label_domain("x-axis categories", labels.iter().map(String::as_str))?;
    let maximum_width = panel.width() / labels.len() as f64 * 0.9;
    let maximum_height = display
        .iter()
        .map(|label| {
            crate::layout::TextBlock::layout(
                label,
                palette.tick_font_size,
                maximum_width.max(1.0),
                "x-axis category label",
            )
            .map(|block| block.height())
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .fold(0.0_f64, f64::max);
    let extra = (maximum_height - f64::from(palette.tick_font_size) * 1.2).max(0.0);
    if panel.height() - extra < 60.0 {
        return Err(Error::Render(
            "multiline x-axis category labels do not fit while preserving the minimum 60px plot panel; increase chart height"
                .into(),
        ));
    }
    panel.y0 += extra;
    Ok(panel)
}
