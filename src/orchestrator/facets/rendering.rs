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
    let mut geometry = crate::render::geometry::GeometrySnapshot::new(theme)?;
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
        crate::significance::render_selected(panel_plot, &panel_plan.result, theme, backend)?;
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
    let resources = theme.prepared_font_environment()?.generation_resources();
    let label = facet_label(
        plot,
        &facet.cell,
        plot.data().with_axis_resources(&resources),
    )?;
    let plan = crate::layout::fit::measured(crate::layout::fit::text(
        theme,
        &label,
        &theme.legacy_text_appearance(theme.facet_header_font_size, crate::TextWeight::Normal),
        facet.panel.width() - 16.0,
        PhysicalTextAlignment::Center,
        "facet strip",
        &crate::layout::fit::FitScope::root()
            .child(
                crate::layout::AddressRole::FacetCell,
                facet.panel_index,
                &theme.prepared_font_environment()?.generation_resources(),
            )?
            .child(
                crate::layout::AddressRole::FacetHeader,
                0,
                &theme.prepared_font_environment()?.generation_resources(),
            )?,
    ))?;
    let first_y =
        facet.panel.y1 + 14.0 + plan.lines.len().saturating_sub(1) as f64 * plan.line_height();
    Ok(PlacedText::from_deferred_first_line(
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
    crate::render::backend::reserve_text_records(
        &mut plans,
        usize::from(horizontal_label.is_some()) + usize::from(vertical_label.is_some()),
        theme.prepared_font_environment()?,
    )?;
    if let Some(label) = horizontal_label {
        let plan = crate::layout::fit::measured(crate::layout::fit::text(
            theme,
            label,
            &theme.typography.axis_title,
            grid.width(),
            PhysicalTextAlignment::Center,
            "shared facet x-axis title",
            &crate::layout::fit::FitScope::root().child(
                crate::layout::AddressRole::AxisBottom,
                0,
                &theme.prepared_font_environment()?.generation_resources(),
            )?,
        ))?;
        let middle_y = axis_origin.1
            - theme.axes.spacing.x_title_without_tick_labels_gap
            - plan.height() / 2.0;
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
            &theme.typography.axis_title,
            grid.height(),
            PhysicalTextAlignment::Center,
            "shared facet y-axis title",
            &crate::layout::fit::FitScope::root().child(
                crate::layout::AddressRole::AxisLeft,
                0,
                &theme.prepared_font_environment()?.generation_resources(),
            )?,
        ))?;
        let middle_x =
            axis_origin.0 - theme.axes.spacing.y_title_edge_padding - plan.height() / 2.0;
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
        data.claim_source_visits(1)?;
        let variable = plot.resolve_source_label(&selector.variable, data)?;
        let level = plot.resolve_source_label(&selector.level, data)?;
        let length = variable
            .len()
            .checked_add(level.len())
            .and_then(|length| length.checked_add(6))
            .ok_or_else(|| Error::Render("facet header length overflow".into()))?;
        data.reserve_source_string(&mut label, length)?;
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
    fn facet_label_admits_each_owned_string_and_refuses_before_unfunded_copies() {
        use crate::font::canonical::OperationGenerationResources;
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
        // Two two-byte label copies and a ten-byte output reservation retain
        // three String headers. The source work is one selector + four bytes.
        for available in [2, 3, 4] {
            let resources = OperationGenerationResources::with_test_limits(5, 14);
            resources.claim_slots(1_000_000 - available).unwrap();
            let data = crate::data::PlotData::with_rows(&plot, &rows, &ids)
                .with_axis_resources(&resources);
            let result = facet_label(&plot, &cell, data);
            if available >= 3 {
                assert_eq!(result.unwrap(), "ab = cd");
            } else {
                assert!(result.unwrap_err().to_string().contains("record admission"));
            }
            assert_eq!(resources.usage().unwrap().1, 5);
            assert_eq!(
                resources.byte_usage().unwrap(),
                if available >= 3 { 14 } else { 4 }
            );
            assert_eq!(data.source_row(0), 29);
        }
        for bytes in [0, 13, 14, 15] {
            let resources = OperationGenerationResources::with_test_limits(5, bytes);
            let data = crate::data::PlotData::with_rows(&plot, &rows, &ids)
                .with_axis_resources(&resources);
            let result = facet_label(&plot, &cell, data);
            if bytes >= 14 {
                assert_eq!(result.unwrap(), "ab = cd");
            } else {
                assert!(result
                    .unwrap_err()
                    .to_string()
                    .contains("storage admission"));
            }
            assert_eq!(
                resources.byte_usage().unwrap(),
                if bytes >= 14 {
                    14
                } else if bytes == 0 {
                    0
                } else {
                    4
                }
            );
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
        let mut theme = crate::Theme::Publication.resolve();
        theme.facet_header_font_size = 17;
        theme.typography.axis_title.line_height_multiplier = 1.25;
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
            f64::midpoint(grid.x0, grid.x1).round().to_bits()
        );
        let mut horizontal = crate::render::geometry::GeometrySnapshot::new(&theme).unwrap();
        horizontal
            .text(axes[0].placement, &axes[0].plan, None)
            .unwrap();
        assert!(horizontal
            .bounds()
            .iter()
            .all(|ink| ink.top <= origin.1 - theme.axes.spacing.x_title_without_tick_labels_gap));
        let mut vertical = crate::render::geometry::GeometrySnapshot::new(&theme).unwrap();
        vertical
            .text(axes[1].placement, &axes[1].plan, None)
            .unwrap();
        assert!(vertical
            .bounds()
            .iter()
            .all(|ink| ink.right <= origin.0 - theme.axes.spacing.y_title_edge_padding));
        assert!(axes[1].placement.reverse_lines);
    }
}
