//! Categorical beeswarm plot with a continuous color scale.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]

use std::collections::BTreeMap;

use crate::layout::{AxisTick, Layout, PanelBox};
use crate::render::backend::{
    with_clip, Backend, GradientStop, LineStyle, Point2D, TextAnchor, TextStyle,
};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::Chart;
use crate::Error;

pub(crate) fn render(
    chart: &Chart<'_>,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    let category_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("beeswarm: categorical row column not set".into()))?;
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("beeswarm: x column not set".into()))?;
    let color_col = chart
        .color_col()
        .ok_or_else(|| Error::Data("beeswarm: continuous color column not set".into()))?;
    let categories = crate::data::strict_strings(context.data, category_col, "beeswarm")?;
    let xs = crate::data::strict_finite_f64(context.data, x_col, "beeswarm")?;
    let colors = crate::data::strict_finite_f64(context.data, color_col, "beeswarm")?;
    if categories.len() != xs.len() || xs.len() != colors.len() || xs.is_empty() {
        return Err(Error::Data(
            "beeswarm: category/x/color columns must have equal non-zero lengths".into(),
        ));
    }
    let mut levels = Vec::new();
    for category in &categories {
        if !levels.contains(category) {
            levels.push(category.clone());
        }
    }
    let display_levels =
        chart.resolve_label_domain("y-axis categories", levels.iter().map(String::as_str))?;
    let x_data = crate::data::min_max(xs.iter().copied(), "beeswarm")?;
    let color_range = match chart.beeswarm_color_limits_spec() {
        Some((minimum, maximum)) => {
            if !minimum.is_finite() || !maximum.is_finite() || minimum >= maximum {
                return Err(Error::Data(format!(
                    "beeswarm: color limits must be finite and strictly ascending; got {minimum}..{maximum}"
                )));
            }
            (minimum, maximum)
        }
        None => crate::data::min_max(colors.iter().copied(), "beeswarm")?,
    };
    let x_range = crate::layout::continuous_auto_range(x_data, chart.x_axis_spec());
    let x_axis_spec = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
    let detail_font_size = chart.palette().detail_font_size;
    let high_label = format_colorbar_value(color_range.1);
    let low_label = format_colorbar_value(color_range.0);
    let color_label = chart.resolved_color_label();
    let color_label_block = crate::layout::TextBlock::layout(
        &color_label,
        detail_font_size,
        (context.panel.height().min(150.0) * 0.9).max(1.0),
        "beeswarm colorbar title",
    )?;
    let mut panel = crate::plots::horizontal_bar::panel_box_for_y_tick_labels(
        chart,
        context.panel,
        &display_levels,
    )?;
    let value_width = [&high_label, &low_label]
        .into_iter()
        .map(|label| crate::layout::estimated_text_width(label, detail_font_size))
        .fold(0.0_f64, f64::max);
    let colorbar_reserve = (34.0 + value_width).max(
        65.0 + color_label_block.lines().len().saturating_sub(1) as f64
            * color_label_block.line_height()
            + f64::from(detail_font_size) / 2.0,
    ) + 4.0;
    if panel.width() - colorbar_reserve < 100.0 {
        return Err(Error::Render(
            "beeswarm: colorbar title does not fit while preserving the minimum 100px plot panel; increase chart width"
                .into(),
        ));
    }
    panel.x1 -= colorbar_reserve;
    let y_ticks = levels
        .iter()
        .enumerate()
        .map(|(index, label)| AxisTick::new(index as f64 + 0.5, label.clone()))
        .collect();
    let layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
        x_range,
        (0.0, levels.len() as f64),
        panel,
        chart.x_label_text().unwrap_or("").to_string(),
        chart.y_label_text().unwrap_or("").to_string(),
        Vec::new(),
        y_ticks,
        &x_axis_spec,
        chart.y_axis_spec(),
    );
    with_clip(backend, layout.projection.panel_box(), |backend| {
        if chart.zero_line_visible() && x_range.0 <= 0.0 && x_range.1 >= 0.0 {
            let (x, y0) = layout.projection.project(0.0, 0.0);
            let (_, y1) = layout.projection.project(0.0, levels.len() as f64);
            backend.draw_line(
                &[Point2D::new(x, y0), Point2D::new(x, y1)],
                &LineStyle::solid(chart.palette().foreground, 0.8),
            )?;
        }

        let mut rows: Vec<(usize, f64, f64, usize)> = categories
            .iter()
            .zip(&xs)
            .zip(&colors)
            .enumerate()
            .map(|(row, ((category, x), color))| {
                let category_index = levels
                    .iter()
                    .position(|level| level == category)
                    .unwrap_or(0);
                (category_index, *x, *color, row)
            })
            .collect();
        rows.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.total_cmp(&b.1))
                .then_with(|| a.2.total_cmp(&b.2))
                .then_with(|| a.3.cmp(&b.3))
        });
        let mut occupied: BTreeMap<(usize, i64), usize> = BTreeMap::new();
        for (category, x, color, _) in rows {
            let bin = (((x - x_range.0) / (x_range.1 - x_range.0)) * 48.0).floor() as i64;
            let count = occupied.entry((category, bin)).or_default();
            let layer = (*count).div_ceil(2);
            let direction = if (*count).is_multiple_of(2) {
                1.0
            } else {
                -1.0
            };
            let jitter = if *count == 0 {
                0.0
            } else {
                direction * (layer as f64 * 0.09).min(0.36)
            };
            *count += 1;
            let (px, py) = layout.projection.project(x, category as f64 + 0.5 + jitter);
            backend.draw_scatter(
                &[Point2D::new(px, py)],
                &chart.point_style(continuous_color(color, color_range), 3),
            )?;
        }
        Ok(())
    })?;
    draw_colorbar(
        chart,
        backend,
        &layout,
        &color_label_block,
        color_range,
        &high_label,
        &low_label,
    )?;
    Ok(PlotRenderResult::new(layout, Vec::new()))
}

fn draw_colorbar(
    chart: &Chart<'_>,
    backend: &mut dyn Backend,
    layout: &Layout,
    label: &crate::layout::TextBlock,
    range: (f64, f64),
    high_label: &str,
    low_label: &str,
) -> Result<(), Error> {
    let (_, panel_right, panel_bottom, panel_top) = layout.projection.pixel_box();
    let left = panel_right + 18.0;
    let height = (panel_top - panel_bottom).min(150.0);
    let bottom = panel_bottom + (panel_top - panel_bottom - height) / 2.0;
    let stops = [
        GradientStop::new(0.0, continuous_color(range.0, range)),
        GradientStop::new(1.0, continuous_color(range.1, range)),
    ];
    backend.draw_linear_gradient(
        PanelBox::new(left, left + 10.0, bottom, bottom + height),
        Point2D::new(left + 5.0, bottom),
        Point2D::new(left + 5.0, bottom + height),
        &stops,
    )?;
    let value_style = TextStyle {
        color: chart.palette().foreground,
        font_family: chart.palette().font_family,
        font_size: chart.palette().detail_font_size,
        weight: crate::TextWeight::Normal,
        anchor: TextAnchor::Start,
        rotation: 0.0,
    };
    backend.draw_text(
        (left + 16.0) as u32,
        (bottom + height - 4.0).max(0.0) as u32,
        high_label,
        &value_style,
    )?;
    backend.draw_text(
        (left + 16.0) as u32,
        (bottom - 13.0).max(0.0) as u32,
        low_label,
        &value_style,
    )?;
    let title_style = TextStyle {
        anchor: TextAnchor::Middle,
        rotation: -90.0,
        ..value_style
    };
    for (index, line) in label.lines().iter().enumerate() {
        backend.draw_text(
            (left + 47.0 + index as f64 * label.line_height()) as u32,
            (bottom + height / 2.0) as u32,
            line,
            &title_style,
        )?;
    }
    Ok(())
}

fn continuous_color(value: f64, range: (f64, f64)) -> (u8, u8, u8) {
    let span = range.1 - range.0;
    let t = if span.to_bits() == 0 {
        0.5
    } else if value <= range.0 {
        0.0
    } else if value >= range.1 {
        1.0
    } else if span.is_finite() {
        (value - range.0) / span
    } else {
        let scale = range.0.abs().max(range.1.abs());
        let minimum = range.0 / scale;
        let maximum = range.1 / scale;
        (value / scale - minimum) / (maximum - minimum)
    };
    let t = t.clamp(0.0, 1.0);
    let low = (49.0, 130.0, 189.0);
    let high = (222.0, 45.0, 38.0);
    (
        (low.0 + (high.0 - low.0) * t).round() as u8,
        (low.1 + (high.1 - low.1) * t).round() as u8,
        (low.2 + (high.2 - low.2) * t).round() as u8,
    )
}

fn format_colorbar_value(value: f64) -> String {
    if value.abs() >= 1_000_000.0 || (value.abs().to_bits() != 0 && value.abs() < 0.0001) {
        format!("{value:.3e}")
    } else {
        format!("{value:.3}")
    }
}
