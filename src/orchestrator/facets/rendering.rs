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
        for text in text {
            geometry.text(text.placement, &text.plan, None)?;
        }
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
    strip_text: &[Vec<PlacedText>],
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
) -> Result<Vec<PlacedText>, crate::layout::fit::PlanError> {
    let resources = theme.prepared_font_environment()?.operation();
    let spacing = theme.facet_strip_lengths()?;
    let plans = strip_plans(
        plot,
        &facet.cell,
        plot.data().with_operation(&resources),
        theme,
        facet.panel.width(),
        &crate::layout::fit::FitScope::root()
            .child(crate::layout::AddressRole::FacetCell, facet.panel_index)?
            .child(crate::layout::AddressRole::FacetHeader, 0)?,
    )?;
    let mut top = spacing
        .content_clearance
        .add_pixels(plans.iter().map(|plan| plan.occupied_size().1).sum())?;
    let mut placed = Vec::new();
    crate::data::source::reserve_source(&mut placed, plans.len())?;
    for plan in plans {
        let first = plan.first_baseline_quanta.unwrap_or(0);
        let first_y = top
            .subtract(&crate::theme::spacing::ExactLength::pixels(
                first as f64 / 1024.0,
            )?)
            .ok_or_else(|| {
                Error::Render("facet strip baseline exceeds its complete extent".into())
            })?
            .after_pixels(facet.panel.y1)?;
        let placement = crate::render::backend::PlannedTextPlacement::baseline(
            Point2D::new(f64::midpoint(facet.panel.x0, facet.panel.x1), first_y),
            TextAnchor::Middle,
            0.0,
        );
        let region = PanelBox::new(
            facet.panel.x0,
            facet.panel.x1,
            top.after_pixels(facet.panel.y1)? - plan.occupied_size().1,
            top.after_pixels(facet.panel.y1)?,
        );
        let placement = crate::layout::text::placement::resolve(placement, &plan, Some(region))?;
        top = top
            .subtract(&crate::theme::spacing::ExactLength::pixels(
                plan.occupied_size().1,
            )?)
            .ok_or_else(|| Error::Render("facet strip sequence underflow".into()))?;
        let plan = crate::layout::text::placement::contain(
            plan,
            placement,
            region,
            &crate::layout::fit::ROOT_SCOPE,
        )?;
        placed.push(PlacedText { plan, placement });
    }
    Ok(placed)
}

fn draw_facet_strip(
    layer: &str,
    text: &[PlacedText],
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    crate::render::backend::with_layer(backend, layer, |backend| {
        for text in text {
            text.draw(backend)?;
        }
        Ok(())
    })
}

pub(super) fn plan_shared_facet_axis_labels(
    horizontal_label: Option<&crate::Text>,
    vertical_label: Option<&crate::Text>,
    grid: PanelBox,
    axis_origin: (f64, f64),
    theme: &ResolvedTheme,
) -> Result<Vec<PlacedText>, crate::layout::fit::PlanError> {
    let mut plans = Vec::new();

    crate::render::backend::reserve_text_records(
        &mut plans,
        usize::from(horizontal_label.is_some()) + usize::from(vertical_label.is_some()),
    )?;
    if let Some(label) = horizontal_label {
        let appearance = label.appearance(&theme.typography.axes.axis_title);
        let lengths = theme.axes.get_spacing().lengths(
            theme.typography.axes.tick_label.get_font_size(),
            appearance.theme.get_font_size(),
        )?;
        let plan = crate::layout::fit::measured(crate::layout::fit::single_text(
            theme,
            label.as_str(),
            appearance,
            grid.width(),
            label
                .resolved_alignment(crate::TextAlignment::Center)
                .into(),
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
        let placement = crate::layout::Axis::title_placement(
            &plan,
            Point2D::new(f64::midpoint(grid.x0, grid.x1), middle_y),
            crate::layout::AxisSide::Bottom,
        )?;
        plans.push(PlacedText { plan, placement });
    }
    if let Some(label) = vertical_label {
        let appearance = label.appearance(&theme.typography.axes.axis_title);
        let lengths = theme.axes.get_spacing().lengths(
            theme.typography.axes.tick_label.get_font_size(),
            appearance.theme.get_font_size(),
        )?;
        let plan = crate::layout::fit::measured(crate::layout::fit::single_text(
            theme,
            label.as_str(),
            appearance,
            grid.height(),
            label
                .resolved_alignment(crate::TextAlignment::Center)
                .into(),
            "shared facet y-axis title",
            &crate::layout::fit::FitScope::root().child(crate::layout::AddressRole::AxisLeft, 0)?,
        ))?;
        let middle_x = crate::theme::AxisLengths::before(
            &lengths
                .y_title_edge_padding
                .add_pixels(plan.height() / 2.0)?,
            axis_origin.0,
        )?;
        let placement = crate::layout::Axis::title_placement(
            &plan,
            Point2D::new(middle_x, f64::midpoint(grid.y0, grid.y1)),
            crate::layout::AxisSide::Left,
        )?;
        let reverse = plan
            .request
            .as_ref()
            .is_none_or(|request| request.instructions().orientation.is_none());
        let text = PlacedText { plan, placement };
        plans.push(if reverse {
            text.with_reversed_lines()
        } else {
            text
        });
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

pub(super) fn strip_sources(
    plot: &Plot,
    cell: &FacetCell,
    data: crate::data::PlotData<'_>,
    theme: &ResolvedTheme,
) -> Result<Vec<crate::Text>, Error> {
    if theme.text_layouts.facet.iter().all(Option::is_none) {
        return Ok(vec![facet_label(plot, cell, data)?.into()]);
    }
    data.source_resources()?;
    crate::data::source::collect_source(cell.iter().enumerate().map(|(index, selector)| {
        let role = if cell.as_slice().len() == 1 { 1 } else { index };
        let variable = plot.resolve_label(&selector.variable)?;
        let level = plot.resolve_label(&selector.level)?;
        let length = variable
            .len()
            .checked_add(level.len())
            .and_then(|length| length.checked_add(3))
            .ok_or_else(|| Error::Render("facet text source length overflow".into()))?;
        let mut source = String::new();
        crate::data::source::reserve_source_string(&mut source, length)?;
        source.push_str(&variable);
        source.push_str(" = ");
        source.push_str(&level);
        let source = crate::Text::from(source);
        Ok(if let Some(layout) = &theme.text_layouts.facet[role] {
            source.layout(layout.clone())
        } else {
            source
        })
    }))
}

pub(super) fn strip_plans(
    plot: &Plot,
    cell: &FacetCell,
    data: crate::data::PlotData<'_>,
    theme: &ResolvedTheme,
    width: f64,
    scope: &crate::layout::fit::FitScope,
) -> Result<Vec<crate::layout::ResolvedTextPlan>, crate::layout::fit::PlanError> {
    let width =
        crate::theme::composition::capacity(width, &theme.facet_strip_lengths()?.horizontal_inset)?;
    crate::layout::fit::collect(
        "facet strip roles",
        strip_sources(plot, cell, data, theme)?
            .iter()
            .map(|source| {
                crate::layout::fit::measured(crate::layout::fit::single_text(
                    theme,
                    source.as_str(),
                    source.appearance(&theme.typography.composition.facet_header),
                    width,
                    PhysicalTextAlignment::Center,
                    "facet strip",
                    scope,
                ))
            }),
    )
}

pub(super) fn strip_widths(
    plot: &Plot,
    cell: &FacetCell,
    data: crate::data::PlotData<'_>,
    theme: &ResolvedTheme,
) -> Result<(f64, f64), Error> {
    strip_sources(plot, cell, data, theme)?.iter().try_fold(
        (0.0_f64, 0.0_f64),
        |(hard, preferred), text| {
            let widths = crate::layout::fit::intrinsic_widths(
                theme,
                text.as_str(),
                text.appearance(&theme.typography.composition.facet_header),
                "facet strip",
            )?;
            Ok((hard.max(widths.0), preferred.max(widths.1)))
        },
    )
}

pub(super) fn strip_requirement(
    axes: &crate::layout::PanelAxisPlan,
    plot: &Plot,
    cell: &FacetCell,
    data: crate::data::PlotData<'_>,
    theme: &ResolvedTheme,
    panel: PanelBox,
    scope: &crate::layout::fit::FitScope,
) -> Result<crate::layout::canvas::PanelRequirements, crate::layout::fit::PlanError> {
    let mut requirement = crate::layout::canvas::PanelRequirements::measure_header(axes, theme)?;
    let plans = strip_plans(plot, cell, data, theme, panel.width(), scope)?;
    let height = plans.iter().map(|plan| plan.occupied_size().1).sum();
    requirement.top = requirement.top.max(crate::theme::composition::hard(
        &theme
            .facet_strip_lengths()?
            .content_clearance
            .add_pixels(height)?,
    )?);
    Ok(requirement)
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
            .font_size(17.0)
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
        let strips = plan_facet_strip(&plot, &facet, &theme).unwrap();
        let strip = &strips[0];
        let origin = (57.25, 67.75);
        let axes = plan_shared_facet_axis_labels(
            Some(&"Horizontal\nSecond".into()),
            Some(&"Vertical\nSecond".into()),
            grid,
            origin,
            &theme,
        )
        .unwrap();
        for role in [strip, &axes[0], &axes[1]] {
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
