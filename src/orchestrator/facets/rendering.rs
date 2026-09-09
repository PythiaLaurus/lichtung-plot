//! Facet grid, marks, decorations, strips, and shared-label rendering.

use crate::Error;

use super::data::{FacetCell, FacetPanel};
use crate::builder::Plot;
use crate::decorations::plan::PanelPlan;
use crate::layout::{PanelBox, PhysicalTextAlignment};
use crate::render::backend::{Backend, PlacedText, Point2D, SurfaceRole, TextAnchor};
use crate::spec::{PlotKind, PlotRenderContext};
use crate::theme::ResolvedTheme;

pub(super) fn observe_arranged_chrome(
    plan: &super::FacetPlan,
    theme: &ResolvedTheme,
    round: &mut crate::layout::kernel::FeedbackRound,
) -> Result<(), Error> {
    let mut geometry = crate::render::geometry::GeometryBackend::new(theme)?;
    geometry.begin(plan.plot.width(), plan.plot.height())?;
    let result = (|| {
        for (facet, text) in plan.panels.iter().zip(&plan.strip_text) {
            round.observe_box(facet.panel)?;
            draw_facet_strip(facet, text, &mut geometry)?;
        }
        draw_shared_facet_axis_labels(&plan.shared_axis_text, &mut geometry)
    })();
    crate::render::backend::finish_scope(result, geometry.end(), "arranged facet chrome")?;
    geometry.observe_feedback(round)
}

#[expect(
    clippy::too_many_arguments,
    reason = "the immutable scene inputs preserve facet rendering order and geometry"
)]
pub(super) fn render_scene(
    source: &Plot,
    shared_plot: &Plot,
    kind: PlotKind,
    panels: &[FacetPanel],
    panel_plans: &[std::sync::Arc<PanelPlan>],
    outside_legend: Option<&crate::decorations::plan::ResolvedLegend>,
    strip_text: &[PlacedText],
    shared_axis_text: &[PlacedText],
    canvas_layout: &crate::layout::CanvasLayout,
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    for panel_plan in panel_plans {
        backend.draw_surface(
            panel_plan.panel_box,
            theme.surfaces.plot_panel,
            SurfaceRole::PlotPanel,
        )?;
    }
    for (facet, panel_plan) in panels.iter().zip(panel_plans) {
        let panel_plot = shared_plot
            .clone()
            .with_render_scope(facet.render_scope())
            .with_significance_subset(facet.significance_selectors());
        crate::decorations::apply::apply_grid(&panel_plot, &panel_plan.result, theme, backend)?;
    }

    for (index, ((facet, panel_plan), strip)) in
        panels.iter().zip(panel_plans).zip(strip_text).enumerate()
    {
        let panel_plot = shared_plot
            .clone()
            .with_render_scope(facet.render_scope())
            .with_significance_subset(facet.significance_selectors());
        let context = PlotRenderContext::new(
            crate::data::PlotData::with_rows(&panel_plot, &facet.rows, &facet.source_rows),
            panel_plan.panel_box,
            theme,
        )
        .with_facet_scales(source.facet_scales_spec())
        .with_deferred_annotation_text(&panel_plan.result.deferred_annotation_text)
        .with_retained_body(&panel_plan.result);
        panel_plan
            .plot_with_planned_limits(&panel_plot)
            .render_panel(kind, context, backend)?;
        if let Some(significance) = &panel_plan.result.significance {
            crate::significance::render(&panel_plot, significance, theme, backend)?;
        }
        crate::decorations::apply::apply_axes(&panel_plot, panel_plan, theme, backend)?;
        crate::decorations::apply::apply_planned_legend(
            &panel_plot,
            if index == 0 {
                outside_legend.or(panel_plan.legend.as_ref())
            } else {
                panel_plan.legend.as_ref()
            },
            backend,
        )?;
        draw_facet_strip(facet, strip, backend)?;
    }
    draw_shared_facet_axis_labels(shared_axis_text, backend)?;
    crate::decorations::apply::apply_canvas_text(source, canvas_layout, backend)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "small facet-line indexes are projected into backend coordinates"
)]
pub(super) fn plan_facet_strip(
    plot: &Plot,
    facet: &FacetPanel,
    theme: &ResolvedTheme,
) -> Result<PlacedText, Error> {
    let label = facet_label(plot, &facet.cell);
    let plan = theme.plan_legacy_text(
        &label,
        theme.facet_header_font_size,
        (facet.panel.width() - 16.0).max(1.0),
        PhysicalTextAlignment::Center,
        "facet strip",
    )?;
    let first_y =
        facet.panel.y1 + 14.0 + plan.lines.len().saturating_sub(1) as f64 * plan.line_height();
    Ok(PlacedText::from_first_line(
        plan,
        Point2D::new(
            f64::midpoint(facet.panel.x0, facet.panel.x1).round(),
            first_y.round(),
        ),
        TextAnchor::Middle,
        0.0,
    ))
}

fn draw_facet_strip(
    facet: &FacetPanel,
    text: &PlacedText,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    crate::render::backend::with_layer(
        backend,
        &format!("facet-strip-{}", facet.panel_index),
        |backend| text.draw(backend),
    )
}

#[expect(
    clippy::cast_precision_loss,
    reason = "small wrapped-line indexes are projected into backend coordinates"
)]
pub(super) fn plan_shared_facet_axis_labels(
    horizontal_label: Option<&str>,
    vertical_label: Option<&str>,
    grid: PanelBox,
    axis_origin: (f64, f64),
    theme: &ResolvedTheme,
) -> Result<Vec<PlacedText>, Error> {
    let mut plans = Vec::new();
    crate::render::backend::reserve_text_records(
        &mut plans,
        usize::from(horizontal_label.is_some()) + usize::from(vertical_label.is_some()),
        theme.prepared_font_environment()?,
    )?;
    if let Some(label) = horizontal_label {
        let plan = theme.plan_text(
            label,
            &theme.typography.axis_title,
            grid.width().max(1.0),
            PhysicalTextAlignment::Center,
            "shared facet x-axis title",
            false,
        )?;
        let first_y = axis_origin.1 - f64::from(theme.typography.axis_title.font_size);
        plans.push(PlacedText::from_first_line(
            plan,
            Point2D::new(f64::midpoint(grid.x0, grid.x1).round(), first_y.round()),
            TextAnchor::Middle,
            0.0,
        ));
    }
    if let Some(label) = vertical_label {
        let plan = theme.plan_text(
            label,
            &theme.typography.axis_title,
            grid.height().max(1.0),
            PhysicalTextAlignment::Center,
            "shared facet y-axis title",
            false,
        )?;
        let first_x = axis_origin.0 - f64::from(theme.typography.axis_title.font_size) - 8.0;
        let first_x = first_x - plan.lines.len().saturating_sub(1) as f64 * plan.line_height();
        plans.push(
            PlacedText::from_first_line(
                plan,
                Point2D::new(first_x.round(), f64::midpoint(grid.y0, grid.y1).round()),
                TextAnchor::Middle,
                -90.0,
            )
            .with_reversed_lines(),
        );
    }
    Ok(plans)
}

fn draw_shared_facet_axis_labels(
    plans: &[PlacedText],
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    if plans.is_empty() {
        return Ok(());
    }
    crate::render::backend::with_layer(backend, "facet-shared-axis-labels", |backend| {
        for text in plans {
            text.draw(backend)?;
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
