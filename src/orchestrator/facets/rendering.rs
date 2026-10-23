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

    round: &mut crate::layout::kernel::FeedbackRound,
) -> Result<(), Error> {
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    for (facet, text) in plan.panels.iter().zip(&plan.strip_text) {
        round.observe_box(facet.panel)?;
        geometry.text(text.placement, &text.plan, None)?;
    }
    for text in &plan.shared_axis_text {
        geometry.text(text.placement, &text.plan, None)?;
    }
    for legend in plan.legends.iter().flatten() {
        legend.geometry(&mut geometry)?;
    }
    geometry.observe_feedback(round)
}

#[expect(
    clippy::too_many_arguments,
    reason = "the immutable scene inputs preserve facet rendering order and geometry"
)]
pub(super) fn render_scene(
    source: &Plot,
    panel_plots: &[Plot],
    kind: PlotKind,
    panels: &[FacetPanel],
    panel_plans: &[std::sync::Arc<PanelPlan>],
    legends: &[Option<crate::decorations::legend::LegendDrawPlan>],
    strip_layers: &[String],
    panel_layers: &[[String; 3]],
    strip_text: &[PlacedText],
    shared_axis_text: &[PlacedText],
    canvas_layout: &crate::layout::RootFramePlan,
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
    for (layers, panel_plan) in panel_layers.iter().zip(panel_plans) {
        crate::render::backend::with_layer_role(
            backend,
            &layers[0],
            crate::render::backend::LayerRole::Grid,
            |backend| {
                panel_plan
                    .result
                    .axes
                    .as_ref()
                    .ok_or_else(|| Error::Render("facet plan is missing immutable axes".into()))?
                    .draw_grid(backend)
            },
        )?;
    }

    for (index, ((facet, panel_plan), strip)) in
        panels.iter().zip(panel_plans).zip(strip_text).enumerate()
    {
        let panel_plot = &panel_plots[index];
        let context = PlotRenderContext::new(
            crate::data::PlotData::with_rows(panel_plot, &facet.rows, &facet.source_rows),
            panel_plan.panel_box,
            theme,
        )
        .with_facet_scales(source.facet_scales_spec())
        .with_retained_body(&panel_plan.result);
        panel_plot.render_panel(kind, context, backend)?;
        crate::significance::render_selected(&panel_plan.result, backend)?;
        crate::render::backend::with_layer(backend, &panel_layers[index][1], |backend| {
            panel_plan
                .result
                .axes
                .as_ref()
                .ok_or_else(|| Error::Render("facet plan is missing immutable axes".into()))?
                .draw(backend)
        })?;
        if let Some(legend) = &legends[index] {
            crate::render::backend::with_layer(backend, &panel_layers[index][2], |backend| {
                legend.draw(backend)
            })?;
        }
        draw_facet_strip(&strip_layers[index], strip, backend)?;
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
) -> Result<PlacedText, crate::layout::fit::PlanError> {
    let resources = theme.prepared_font_environment()?.operation();
    let spacing = theme.facet_strip_lengths()?;
    let label = facet_label(plot, &facet.cell, plot.data().with_operation(&resources))?;
    let plan = crate::layout::fit::measured(crate::layout::fit::text(
        theme,
        &label,
        &theme.typography.composition.facet_header,
        crate::theme::composition::capacity(facet.panel.width(), &spacing.horizontal_inset)?,
        PhysicalTextAlignment::Center,
        "facet strip",
        &crate::layout::fit::FitScope::root()
            .child(crate::layout::AddressRole::FacetCell, facet.panel_index)?
            .child(crate::layout::AddressRole::FacetHeader, 0)?,
    ))?;
    let top = spacing.content_clearance.add_pixels(plan.height())?;
    let first = plan
        .first_baseline_quanta
        .ok_or_else(|| Error::Render("facet strip has no retained baseline".into()))?;
    let first_y = top
        .subtract(&crate::theme::spacing::ExactLength::pixels(
            first as f64 / 1024.0,
        )?)
        .ok_or_else(|| Error::Render("facet strip baseline exceeds its complete extent".into()))?
        .after_pixels(facet.panel.y1)?;
    Ok(PlacedText {
        plan,
        placement: crate::render::backend::PlannedTextPlacement::baseline(
            Point2D::new(f64::midpoint(facet.panel.x0, facet.panel.x1), first_y),
            TextAnchor::Middle,
            0.0,
        ),
    })
}

fn draw_facet_strip(
    layer: &str,
    text: &PlacedText,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    crate::render::backend::with_layer(backend, layer, |backend| text.draw(backend))
}

pub(super) fn plan_shared_facet_axis_labels(
    horizontal_label: Option<&str>,
    vertical_label: Option<&str>,
    grid: PanelBox,
    axis_origin: (f64, f64),
    theme: &ResolvedTheme,
) -> Result<Vec<PlacedText>, crate::layout::fit::PlanError> {
    let mut plans = Vec::new();

    let lengths = theme.axes.get_spacing().lengths(
        theme.typography.axes.tick_label.get_font_size(),
        theme.typography.axes.axis_title.get_font_size(),
    )?;
    crate::render::backend::reserve_text_records(
        &mut plans,
        usize::from(horizontal_label.is_some()) + usize::from(vertical_label.is_some()),
    )?;
    if let Some(label) = horizontal_label {
        let plan = crate::layout::fit::measured(crate::layout::fit::text(
            theme,
            label,
            &theme.typography.axes.axis_title,
            grid.width(),
            PhysicalTextAlignment::Center,
            "shared facet x-axis title",
            &crate::layout::fit::FitScope::root()
                .child(crate::layout::AddressRole::AxisBottom, 0)?,
        ))?;
        let middle_y = crate::theme::AxisLengths::before(
            &lengths
                .x_title_without_tick_labels_gap
                .add_pixels(plan.height() / 2.0)?,
            axis_origin.1,
        )?;
        plans.push(PlacedText::middle(
            plan,
            Point2D::new(f64::midpoint(grid.x0, grid.x1), middle_y),
            TextAnchor::Middle,
            0.0,
        ));
    }
    if let Some(label) = vertical_label {
        let plan = crate::layout::fit::measured(crate::layout::fit::text(
            theme,
            label,
            &theme.typography.axes.axis_title,
            grid.height(),
            PhysicalTextAlignment::Center,
            "shared facet y-axis title",
            &crate::layout::fit::FitScope::root().child(crate::layout::AddressRole::AxisLeft, 0)?,
        ))?;
        let middle_x = crate::theme::AxisLengths::before(
            &lengths
                .y_title_edge_padding
                .add_pixels(plan.height() / 2.0)?,
            axis_origin.0,
        )?;
        plans.push(
            PlacedText::middle(
                plan,
                Point2D::new(middle_x, f64::midpoint(grid.y0, grid.y1)),
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

pub(super) fn facet_label(
    plot: &Plot,
    cell: &FacetCell,
    data: crate::data::PlotData<'_>,
) -> Result<String, Error> {
    data.source_resources()?;
    let mut label = String::new();
    for (index, selector) in cell.iter().enumerate() {
        let variable = plot.resolve_label(&selector.variable)?;
        let level = plot.resolve_label(&selector.level)?;
        let length = variable
            .len()
            .checked_add(level.len())
            .and_then(|length| length.checked_add(6))
            .ok_or_else(|| Error::Render("facet header length overflow".into()))?;
        crate::data::source::reserve_source_string(&mut label, length)?;
        if index != 0 {
            label.push_str(" | ");
        }
        label.push_str(&variable);
        label.push_str(" = ");
        label.push_str(&level);
    }
    Ok(label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facet_label_preserves_raw_names_and_source_rows() {
        use crate::font::canonical::OperationContext;
        let plot = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [2.0, 3.0]),
        )
        .labels(crate::Labels::raw())
        .erase();
        let cell = vec![super::super::data::FacetSelector {
            column: "facet",
            variable: "ab".into(),
            level: "cd".into(),
        }];
        let rows = [1];
        let ids = [29];
        {
            let resources = OperationContext::new();

            let data =
                crate::data::PlotData::with_rows(&plot, &rows, &ids).with_operation(&resources);
            let result = facet_label(&plot, &cell, data);
            assert_eq!(result.unwrap(), "ab = cd");

            assert_eq!(data.source_row(0), 29);
        }
    }

    #[test]
    fn shared_multiline_titles_use_measured_clearance_and_keep_centered_strips() {
        let plot = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .labels(crate::Labels::raw())
        .erase();
        let mut theme = crate::Theme::publication().resolve().unwrap();
        theme.typography.composition.facet_header = theme
            .typography
            .composition
            .facet_header
            .clone()
            .font_size(17)
            .unwrap();
        theme.typography.axes.axis_title.line_height_multiplier = 1.25;
        let grid = PanelBox::new(71.25, 372.75, 83.25, 304.75);
        let facet = FacetPanel {
            panel_index: 0,
            row: 0,
            column: 0,
            cell: vec![super::super::data::FacetSelector {
                column: "facet",
                variable: "Group".into(),
                level: "First\nSecond".into(),
            }],
            rows: vec![0, 1],
            source_rows: vec![0, 1],
            panel: grid,
        };
        let strip = plan_facet_strip(&plot, &facet, &theme).unwrap();
        let origin = (57.25, 67.75);
        let axes = plan_shared_facet_axis_labels(
            Some("Horizontal\nSecond"),
            Some("Vertical\nSecond"),
            grid,
            origin,
            &theme,
        )
        .unwrap();
        for role in [&strip, &axes[0], &axes[1]] {
            assert_eq!(role.plan.lines.len(), 2);
        }
        assert_eq!(
            strip.placement.position.x.to_bits(),
            f64::midpoint(grid.x0, grid.x1).to_bits()
        );
        assert_eq!(strip.plan.source, "Group = First\nSecond");

        let reserve = crate::theme::composition::hard(
            &theme
                .facet_strip_lengths()
                .unwrap()
                .content_clearance
                .add_quanta(crate::layout::kernel::LayoutScalar::from_quanta(
                    strip.plan.height_quanta,
                ))
                .unwrap(),
        )
        .unwrap();
        let mut strip_ink = crate::render::geometry::GeometrySnapshot::new();
        strip_ink.text(strip.placement, &strip.plan, None).unwrap();
        assert!(!strip_ink.bounds().is_empty());
        assert!(strip_ink.bounds().iter().all(|ink| ink.bottom >= grid.y1
            && ink.top <= grid.y1 + reserve
            && ink.left >= grid.x0
            && ink.right <= grid.x1));
        let mut horizontal = crate::render::geometry::GeometrySnapshot::new();
        horizontal
            .text(axes[0].placement, &axes[0].plan, None)
            .unwrap();
        assert!(horizontal
            .bounds()
            .iter()
            .all(|ink| ink.top <= origin.1 - 16.0));
        let mut vertical = crate::render::geometry::GeometrySnapshot::new();
        vertical
            .text(axes[1].placement, &axes[1].plan, None)
            .unwrap();
        assert!(vertical
            .bounds()
            .iter()
            .all(|ink| ink.right <= origin.0 - 12.0));
        assert!(axes[1].placement.reverse_lines);
    }
}
