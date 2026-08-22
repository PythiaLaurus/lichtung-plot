//! Cumulative signed-delta waterfall plot.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]

use crate::layout::{AxisTick, Layout, PanelBox};
use crate::render::backend::{
    with_clip, Backend, FillStyle, LineStyle, Point2D, TextAnchor, TextStyle,
};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::Error;
use crate::Plot;

pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    let label_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("waterfall: label column not set".into()))?;
    let delta_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("waterfall: delta column not set".into()))?;
    let (base, terminal) = chart
        .waterfall_totals()
        .ok_or_else(|| Error::Data("waterfall: base and terminal values not set".into()))?;
    let labels = crate::data::strict_strings(context.data, label_col, "waterfall")?;
    let display_labels =
        chart.resolve_label_domain("y-axis categories", labels.iter().map(String::as_str))?;
    let deltas = crate::data::strict_finite_f64(context.data, delta_col, "waterfall")?;
    if labels.is_empty() || labels.len() != deltas.len() {
        return Err(Error::Data(
            "waterfall: labels and deltas must have equal non-zero lengths".into(),
        ));
    }
    if !base.is_finite() || !terminal.is_finite() {
        return Err(Error::Data(
            "waterfall: base and terminal values must be finite".into(),
        ));
    }
    let computed = base + deltas.iter().sum::<f64>();
    let tolerance = 1e-8_f64.max(terminal.abs() * 1e-8);
    if (computed - terminal).abs() > tolerance {
        return Err(Error::Data(format!(
            "waterfall: base plus deltas is {computed}, expected terminal {terminal}"
        )));
    }
    let mut running = base;
    let mut spans = Vec::with_capacity(deltas.len());
    let mut range = (base.min(terminal), base.max(terminal));
    for delta in &deltas {
        let next = running + delta;
        range.0 = range.0.min(running).min(next);
        range.1 = range.1.max(running).max(next);
        spans.push((running, next));
        running = next;
    }
    let x_range = crate::layout::continuous_auto_range(range, chart.x_axis_spec());
    let x_axis_spec = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
    let y_ticks = labels
        .iter()
        .enumerate()
        .map(|(index, label)| AxisTick::new(index as f64 + 0.5, label.clone()))
        .collect::<Vec<_>>();
    let panel = crate::plots::horizontal_bar::panel_box_for_y_tick_labels(
        chart,
        context.panel,
        &display_labels,
    )?;
    let initial_layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
        x_range,
        (0.0, labels.len() as f64),
        panel,
        chart.x_label_text().unwrap_or("").to_string(),
        chart.y_label_text().unwrap_or("").to_string(),
        Vec::new(),
        y_ticks.clone(),
        &x_axis_spec,
        chart.y_axis_spec(),
    );
    let palette = chart.palette();
    let (base_label, terminal_label) = chart
        .waterfall_labels_spec()
        .map_or(("base", "total"), |labels| {
            (labels.base(), labels.terminal())
        });
    let annotations = plan_annotations(
        &initial_layout,
        panel,
        labels.len() as f64,
        [
            (base, format!("{base_label} {base:.3}")),
            (terminal, format!("{terminal_label} {terminal:.3}")),
        ],
        palette.detail_font_size,
    )?;
    if annotations.height > panel.height() - MINIMUM_MARK_PANEL_HEIGHT {
        return Err(Error::Render(format!(
            "waterfall: total annotations require {:.0}px while preserving a {MINIMUM_MARK_PANEL_HEIGHT:.0}px mark panel; increase chart height or shorten the configured labels",
            annotations.height
        )));
    }
    let mut mark_panel = panel;
    mark_panel.y1 -= annotations.height;
    crate::plots::horizontal_bar::validate_y_tick_label_rows(chart, mark_panel, &display_labels)?;
    let layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
        x_range,
        (0.0, labels.len() as f64),
        mark_panel,
        chart.x_label_text().unwrap_or("").to_string(),
        chart.y_label_text().unwrap_or("").to_string(),
        Vec::new(),
        y_ticks,
        &x_axis_spec,
        chart.y_axis_spec(),
    );
    with_clip(backend, layout.projection.panel_box(), |backend| {
        for (index, ((start, end), delta)) in spans.iter().zip(&deltas).enumerate() {
            let y0 = index as f64 + 0.2;
            let y1 = index as f64 + 0.8;
            let (left, bottom) = layout.projection.project(*start, y0);
            let (right, top) = layout.projection.project(*end, y1);
            let color = if *delta >= 0.0 {
                palette.colors[0]
            } else {
                palette.colors[1 % palette.colors.len()]
            };
            backend.draw_polygon(
                &[
                    Point2D::new(left, bottom),
                    Point2D::new(right, bottom),
                    Point2D::new(right, top),
                    Point2D::new(left, top),
                ],
                &FillStyle {
                    color,
                    stroke: palette.foreground,
                    stroke_width: 0.8,
                    opacity: 1.0,
                },
            )?;
            if index + 1 < spans.len() {
                let (x, from_y) = layout.projection.project(*end, y1);
                let (_, to_y) = layout.projection.project(*end, index as f64 + 1.2);
                backend.draw_line(
                    &[Point2D::new(x, from_y), Point2D::new(x, to_y)],
                    &LineStyle::dashed(palette.foreground, 0.7, "3 2"),
                )?;
            }
        }
        for value in [base, terminal] {
            let (x, y0) = layout.projection.project(value, 0.0);
            let (_, y1) = layout.projection.project(value, labels.len() as f64);
            backend.draw_line(
                &[Point2D::new(x, y0), Point2D::new(x, y1)],
                &LineStyle::dashed(palette.foreground, 0.7, "4 3"),
            )?;
        }
        Ok(())
    })?;
    let total_style = TextStyle {
        color: palette.foreground,
        font_family: palette.font_family,
        font_size: palette.detail_font_size,
        weight: crate::TextWeight::Normal,
        anchor: TextAnchor::Middle,
        rotation: 0.0,
    };
    for annotation in annotations.items {
        backend.draw_text(
            annotation.x as u32,
            (layout.projection.panel_box().y1
                + ANNOTATION_PADDING
                + f64::from(palette.detail_font_size) / 2.0
                + annotation.row as f64
                    * (f64::from(palette.detail_font_size) + ANNOTATION_ROW_GAP))
                .round() as u32,
            &annotation.text,
            &total_style,
        )?;
    }
    Ok(PlotRenderResult::new(layout, Vec::new()))
}

const ANNOTATION_PADDING: f64 = 4.0;
const ANNOTATION_GAP: f64 = 4.0;
const ANNOTATION_ROW_GAP: f64 = 4.0;
const MINIMUM_MARK_PANEL_HEIGHT: f64 = 24.0;

struct Annotation {
    text: String,
    x: f64,
    row: usize,
}

struct AnnotationPlan {
    items: Vec<Annotation>,
    height: f64,
}

fn plan_annotations(
    layout: &Layout,
    panel: PanelBox,
    source_y: f64,
    candidates: [(f64, String); 2],
    font_size: u32,
) -> Result<AnnotationPlan, Error> {
    let mut rows: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut items = Vec::new();
    for (value, text) in candidates {
        if !layout.projection.contains_data(value, source_y) {
            continue;
        }
        let width = crate::layout::estimated_text_width(&text, font_size);
        if width + 2.0 * ANNOTATION_PADDING > panel.width() {
            return Err(Error::Render(format!(
                "waterfall: total annotation {text:?} requires {:.0}px but only {:.0}px is available; increase chart width or shorten the configured label",
                width + 2.0 * ANNOTATION_PADDING,
                panel.width()
            )));
        }
        let (projected_x, _) = layout.projection.project(value, source_y);
        let half_width = width / 2.0;
        let x = projected_x
            .clamp(
                panel.x0 + ANNOTATION_PADDING + half_width,
                panel.x1 - ANNOTATION_PADDING - half_width,
            )
            .round()
            .clamp(
                panel.x0 + ANNOTATION_PADDING + half_width,
                panel.x1 - ANNOTATION_PADDING - half_width,
            );
        let bounds = (x - half_width, x + half_width);
        let row = rows
            .iter()
            .position(|occupied| {
                occupied.iter().all(|(left, right)| {
                    bounds.1 + ANNOTATION_GAP <= *left || *right + ANNOTATION_GAP <= bounds.0
                })
            })
            .unwrap_or(rows.len());
        if row == rows.len() {
            rows.push(Vec::new());
        }
        rows[row].push(bounds);
        items.push(Annotation { text, x, row });
    }
    let height = if rows.is_empty() {
        0.0
    } else {
        2.0 * ANNOTATION_PADDING
            + rows.len() as f64 * f64::from(font_size)
            + rows.len().saturating_sub(1) as f64 * ANNOTATION_ROW_GAP
    };
    Ok(AnnotationPlan { items, height })
}
