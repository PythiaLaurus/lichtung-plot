//! Facet grid, marks, decorations, strips, and shared-label rendering.

use crate::Error;

use super::data::{FacetCell, FacetPanel};
use crate::builder::Plot;
use crate::decorations::plan::PanelPlan;
use crate::layout::figure::try_wrap_text;
use crate::layout::{PanelBox, TextBlock};
use crate::render::backend::{Backend, TextAnchor, TextStyle};
use crate::spec::{PlotKind, PlotRenderContext};
use crate::theme::ThemePalette;

#[expect(
    clippy::too_many_arguments,
    reason = "the immutable scene inputs preserve facet rendering order and geometry"
)]
pub(super) fn render_scene(
    source: &Plot,
    chart: &Plot,
    kind: PlotKind,
    panels: Vec<FacetPanel>,
    plans: Vec<PanelPlan>,
    horizontal_axis_label: Option<&str>,
    vertical_axis_label: Option<&str>,
    outer: PanelBox,
    shared_axis_origin: (f64, f64),
    palette: &ThemePalette,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    for (facet, plan) in panels.iter().zip(&plans) {
        let panel_chart = chart
            .clone()
            .with_render_scope(facet.render_scope())
            .with_significance_subset(facet.significance_selectors());
        crate::decorations::apply::apply_grid(
            &panel_chart,
            &plan.result,
            &panel_chart.palette(),
            backend,
        )?;
    }

    for (facet, plan) in panels.into_iter().zip(plans) {
        let panel_chart = chart
            .clone()
            .with_render_scope(facet.render_scope())
            .with_significance_subset(facet.significance_selectors());
        let context = PlotRenderContext::new(
            crate::data::PlotData::with_rows(&panel_chart, &facet.rows, &facet.source_rows),
            plan.panel,
        )
        .with_facet_scales(source.facet_scales_spec());
        plan.chart_for_render(&panel_chart)
            .render_panel(kind, context, backend)?;
        if !plan.result.significance_brackets.is_empty() {
            crate::significance::render(&panel_chart, &plan.result.significance_brackets, backend)?;
        }
        crate::decorations::apply::apply_axes(
            &panel_chart,
            &plan.result,
            &panel_chart.palette(),
            backend,
        )?;
        crate::decorations::apply::apply_planned_legend(
            &panel_chart,
            plan.legend.as_ref(),
            backend,
        )?;
        draw_facet_strip(&panel_chart, &facet, backend)?;
    }
    draw_shared_facet_axis_labels(
        horizontal_axis_label,
        vertical_axis_label,
        outer,
        shared_axis_origin,
        palette,
        backend,
    )?;
    crate::decorations::apply::apply_figure_text(source, &source.palette(), backend)
}

#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "validated finite facet coordinates are rounded for the integer text backend"
)]
fn draw_facet_strip(
    chart: &Plot,
    facet: &FacetPanel,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let label = facet_label(chart, &facet.cell);
    let palette = chart.palette();
    let lines = try_wrap_text(
        &label,
        palette.subtitle_font_size,
        (facet.panel.width() - 16.0).max(1.0),
        "facet strip",
    )?;
    let line_height = f64::from(palette.subtitle_font_size) + 6.0;
    crate::render::backend::with_layer(
        backend,
        &format!("facet-strip-{}", facet.panel_index),
        |backend| {
            for (index, line) in lines.iter().rev().enumerate() {
                backend.draw_text(
                    f64::midpoint(facet.panel.x0, facet.panel.x1).round() as u32,
                    (facet.panel.y1 + 14.0 + index as f64 * line_height).round() as u32,
                    line,
                    &TextStyle {
                        color: palette.foreground,
                        font_family: palette.font_family.clone(),
                        font_size: palette.subtitle_font_size,
                        weight: crate::TextWeight::Normal,
                        anchor: TextAnchor::Middle,
                        rotation: 0.0,
                    },
                )?;
            }
            Ok(())
        },
    )
}

#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "validated finite canvas coordinates are rounded for the integer text backend"
)]
fn draw_shared_facet_axis_labels(
    horizontal_label: Option<&str>,
    vertical_label: Option<&str>,
    grid: PanelBox,
    axis_origin: (f64, f64),
    palette: &ThemePalette,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    if horizontal_label.is_none() && vertical_label.is_none() {
        return Ok(());
    }
    crate::render::backend::with_layer(backend, "facet-shared-axis-labels", |backend| {
        let base_style = TextStyle {
            color: palette.foreground,
            font_family: palette.font_family.clone(),
            font_size: palette.label_font_size,
            weight: crate::TextWeight::Normal,
            anchor: TextAnchor::Middle,
            rotation: 0.0,
        };
        if let Some(label) = horizontal_label {
            let block = TextBlock::layout(
                label,
                palette.label_font_size,
                grid.width().max(1.0),
                "shared facet x-axis title",
            )?;
            let first_y = axis_origin.1 - f64::from(palette.label_font_size);
            for (index, line) in block.lines().iter().enumerate() {
                backend.draw_text(
                    f64::midpoint(grid.x0, grid.x1).round() as u32,
                    (first_y - index as f64 * block.line_height()).round() as u32,
                    line,
                    &base_style,
                )?;
            }
        }
        if let Some(label) = vertical_label {
            let block = TextBlock::layout(
                label,
                palette.label_font_size,
                grid.height().max(1.0),
                "shared facet y-axis title",
            )?;
            let first_x = axis_origin.0 - f64::from(palette.label_font_size) - 8.0;
            let style = TextStyle {
                rotation: -90.0,
                ..base_style
            };
            for (index, line) in block.lines().iter().enumerate() {
                backend.draw_text(
                    (first_x - index as f64 * block.line_height()).round() as u32,
                    f64::midpoint(grid.y0, grid.y1).round() as u32,
                    line,
                    &style,
                )?;
            }
        }
        Ok(())
    })
}

pub(super) fn facet_label(chart: &Plot, cell: &FacetCell) -> String {
    cell.iter()
        .map(|selector| {
            format!(
                "{} = {}",
                chart.resolve_label(&selector.variable),
                chart.resolve_label(&selector.level)
            )
        })
        .collect::<Vec<_>>()
        .join(" | ")
}
