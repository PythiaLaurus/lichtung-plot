//! Facet grid, marks, decorations, strips, and shared-label rendering.

use crate::Error;

use super::data::{FacetCell, FacetPanel};
use crate::builder::Plot;
use crate::decorations::plan::PanelPlan;
use crate::layout::canvas::try_wrap_text;
use crate::layout::{PanelBox, TextBlock};
use crate::render::backend::{Backend, Point2D, TextAnchor, TextStyle};
use crate::spec::{PlotKind, PlotRenderContext};
use crate::theme::ResolvedTheme;

#[expect(
    clippy::too_many_arguments,
    reason = "the immutable scene inputs preserve facet rendering order and geometry"
)]
pub(super) fn render_scene(
    source: &Plot,
    shared_plot: &Plot,
    kind: PlotKind,
    panels: &[FacetPanel],
    panel_plans: &[PanelPlan],
    horizontal_axis_label: Option<&str>,
    vertical_axis_label: Option<&str>,
    outer: PanelBox,
    shared_axis_origin: (f64, f64),
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    for (facet, panel_plan) in panels.iter().zip(panel_plans) {
        let panel_plot = shared_plot
            .clone()
            .with_render_scope(facet.render_scope())
            .with_significance_subset(facet.significance_selectors());
        crate::decorations::apply::apply_grid(&panel_plot, &panel_plan.result, theme, backend)?;
    }

    for (facet, panel_plan) in panels.iter().zip(panel_plans) {
        let panel_plot = shared_plot
            .clone()
            .with_render_scope(facet.render_scope())
            .with_significance_subset(facet.significance_selectors());
        let context = PlotRenderContext::new(
            crate::data::PlotData::with_rows(&panel_plot, &facet.rows, &facet.source_rows),
            panel_plan.panel_box,
            theme,
        )
        .with_facet_scales(source.facet_scales_spec());
        panel_plan
            .plot_with_planned_limits(&panel_plot)
            .render_panel(kind, context, backend)?;
        if !panel_plan.result.significance_brackets.is_empty() {
            crate::significance::render(
                &panel_plot,
                &panel_plan.result.significance_brackets,
                theme,
                backend,
            )?;
        }
        crate::decorations::apply::apply_axes(&panel_plot, &panel_plan.result, theme, backend)?;
        crate::decorations::apply::apply_planned_legend(
            &panel_plot,
            panel_plan.legend.as_ref(),
            backend,
        )?;
        draw_facet_strip(&panel_plot, facet, theme, backend)?;
    }
    draw_shared_facet_axis_labels(
        horizontal_axis_label,
        vertical_axis_label,
        outer,
        shared_axis_origin,
        theme,
        backend,
    )?;
    crate::decorations::apply::apply_canvas_text(source, theme, backend)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "small facet-line indexes are projected into backend coordinates"
)]
fn draw_facet_strip(
    plot: &Plot,
    facet: &FacetPanel,
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let label = facet_label(plot, &facet.cell);
    let lines = try_wrap_text(
        &label,
        theme.subtitle_font_size,
        (facet.panel.width() - 16.0).max(1.0),
        "facet strip",
    )?;
    let line_height = f64::from(theme.subtitle_font_size) + 6.0;
    crate::render::backend::with_layer(
        backend,
        &format!("facet-strip-{}", facet.panel_index),
        |backend| {
            for (index, line) in lines.iter().rev().enumerate() {
                backend.draw_text(
                    Point2D::new(
                        f64::midpoint(facet.panel.x0, facet.panel.x1).round(),
                        (facet.panel.y1 + 14.0 + index as f64 * line_height).round(),
                    ),
                    line,
                    &TextStyle {
                        color: theme.foreground,
                        font_family: theme.font_family.clone(),
                        font_size: theme.subtitle_font_size,
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
    reason = "small wrapped-line indexes are projected into backend coordinates"
)]
fn draw_shared_facet_axis_labels(
    horizontal_label: Option<&str>,
    vertical_label: Option<&str>,
    grid: PanelBox,
    axis_origin: (f64, f64),
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    if horizontal_label.is_none() && vertical_label.is_none() {
        return Ok(());
    }
    crate::render::backend::with_layer(backend, "facet-shared-axis-labels", |backend| {
        let base_style = TextStyle {
            color: theme.foreground,
            font_family: theme.font_family.clone(),
            font_size: theme.label_font_size,
            weight: crate::TextWeight::Normal,
            anchor: TextAnchor::Middle,
            rotation: 0.0,
        };
        if let Some(label) = horizontal_label {
            let block = TextBlock::layout(
                label,
                theme.label_font_size,
                grid.width().max(1.0),
                "shared facet x-axis title",
            )?;
            let first_y = axis_origin.1 - f64::from(theme.label_font_size);
            for (index, line) in block.lines().iter().enumerate() {
                backend.draw_text(
                    Point2D::new(
                        f64::midpoint(grid.x0, grid.x1).round(),
                        (first_y - index as f64 * block.line_height()).round(),
                    ),
                    line,
                    &base_style,
                )?;
            }
        }
        if let Some(label) = vertical_label {
            let block = TextBlock::layout(
                label,
                theme.label_font_size,
                grid.height().max(1.0),
                "shared facet y-axis title",
            )?;
            let first_x = axis_origin.0 - f64::from(theme.label_font_size) - 8.0;
            let style = TextStyle {
                rotation: -90.0,
                ..base_style
            };
            for (index, line) in block.lines().iter().enumerate() {
                backend.draw_text(
                    Point2D::new(
                        (first_x - index as f64 * block.line_height()).round(),
                        f64::midpoint(grid.y0, grid.y1).round(),
                    ),
                    line,
                    &style,
                )?;
            }
        }
        Ok(())
    })
}

pub(super) fn facet_label(plot: &Plot, cell: &FacetCell) -> String {
    cell.iter()
        .map(|selector| {
            format!(
                "{} = {}",
                plot.resolve_label(&selector.variable),
                plot.resolve_label(&selector.level)
            )
        })
        .collect::<Vec<_>>()
        .join(" | ")
}
