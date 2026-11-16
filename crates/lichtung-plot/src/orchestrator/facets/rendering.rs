//! Facet grid, marks, decorations, strips, and shared-label rendering.

use crate::Error;

use crate::builder::PlotDescription;
use crate::layout::{PhysicalTextAlignment, PixelRect};
use crate::orchestrator::root::GridSlot;
use crate::render::backend::{Backend, PlacedText, Point2D, SurfaceRole, TextAnchor};
use crate::spec::PlotKind;
use crate::FacetCell;

pub(super) fn observe_arranged_chrome(
    plan: &super::OwnedFacetPlan,

    round: &mut crate::layout::kernel::FeedbackRound,
) -> Result<(), Error> {
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    for child in &plan.children {
        round.observe_box(plan.cells[child.slot.0])?;
        for text in &child.strips {
            geometry.text(text.placement, &text.plan, None)?;
        }
    }
    for text in &plan.shared_titles {
        geometry.text(text.placement, &text.plan, None)?;
    }
    for (_, legend) in plan.parent_guide.iter().chain(
        plan.children
            .iter()
            .filter_map(|child| child.legend.as_ref()),
    ) {
        legend.geometry(&mut geometry)?;
    }
    geometry.observe_feedback(round)
}

pub(super) fn render_scene(
    source: &PlotDescription,
    kind: PlotKind,
    children: &[super::OwnedFacetChild],
    parent_guide: Option<&(String, crate::decorations::legend::LegendDrawPlan)>,
    shared_axis_text: &[PlacedText],
    canvas_layout: &crate::layout::RootFramePlan,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    for child in children {
        let panel_plan = &child.plot;
        backend.draw_surface(
            panel_plan.panel_box,
            panel_plan.surface,
            SurfaceRole::PlotPanel,
        )?;
    }
    for child in children {
        let panel_plan = &child.plot;
        crate::render::backend::with_layer_role(
            backend,
            &panel_plan.layers.as_ref().expect("facet plot layers").grid,
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

    for (index, child) in children.iter().enumerate() {
        let panel_plan = &child.plot;
        crate::layout::interior::InteriorRoute::classify(kind).validate(source)?;
        panel_plan.result.draw_body(backend)?;
        crate::significance::render_selected(&panel_plan.result, backend)?;
        crate::render::backend::with_layer(
            backend,
            &panel_plan.layers.as_ref().expect("facet plot layers").axes,
            |backend| {
                panel_plan
                    .result
                    .axes
                    .as_ref()
                    .ok_or_else(|| Error::Render("facet plan is missing immutable axes".into()))?
                    .draw(backend)
            },
        )?;
        let legend = if index == 0 {
            parent_guide.or(child.legend.as_ref())
        } else {
            child.legend.as_ref()
        };
        if let Some((layer, legend)) = legend {
            crate::render::backend::with_layer(backend, layer, |backend| legend.draw(backend))?;
        }
        draw_facet_strip(&child.strip_layer, &child.strips, backend)?;
    }
    draw_shared_facet_axis_labels(shared_axis_text, backend)?;
    canvas_layout.root_text.draw(source.prose(), backend)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "small facet-line indexes are projected into backend coordinates"
)]
pub(super) fn plan_facet_strip(
    plot: &PlotDescription,
    cell: &FacetCell,
    slot: GridSlot,
    panel: PixelRect,
    theme: &crate::text::TextRequestContext<'_, '_>,
) -> Result<Vec<PlacedText>, crate::layout::fit::PlanError> {
    theme.text.prepared_font_environment()?;
    let spacing = theme.facet_strip_lengths()?;
    let plans = strip_plans(
        plot,
        cell,
        theme,
        panel.width(),
        &crate::layout::fit::FitScope::root()
            .child(crate::layout::AddressRole::FacetCell, slot.0)?
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
            .after_pixels(panel.y1)?;
        let placement = crate::render::backend::PlannedTextPlacement::baseline(
            Point2D::new(f64::midpoint(panel.x0, panel.x1), first_y),
            TextAnchor::Middle,
            0.0,
        );
        let region = PixelRect::new(
            panel.x0,
            panel.x1,
            top.after_pixels(panel.y1)? - plan.occupied_size().1,
            top.after_pixels(panel.y1)?,
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
    grid: PixelRect,
    axis_origin: (f64, f64),
    theme: &crate::text::TextRequestContext<'_, '_>,
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

pub(super) fn facet_label(plot: &PlotDescription, cell: &FacetCell) -> Result<String, Error> {
    let mut label = String::new();
    for (index, (_, values, raw_level)) in super::data::fields(plot, cell).iter().enumerate() {
        let variable = plot.resolve_label(values.name())?;
        let level = plot.resolve_label(raw_level)?;
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
    plot: &PlotDescription,
    cell: &FacetCell,
    theme: &crate::text::TextRequestContext<'_, '_>,
) -> Result<Vec<crate::Text>, Error> {
    if theme.text_layouts.facet.iter().all(Option::is_none) {
        return Ok(vec![facet_label(plot, cell)?.into()]);
    }
    crate::data::source::collect_source(
        super::data::fields(plot, cell).into_iter().enumerate().map(
            |(index, (_, values, raw_level))| {
                let role = if cell.dimensions() == 1 { 1 } else { index };
                let variable = plot.resolve_label(values.name())?;
                let level = plot.resolve_label(raw_level)?;
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
            },
        ),
    )
}

pub(super) fn strip_plans(
    plot: &PlotDescription,
    cell: &FacetCell,
    theme: &crate::text::TextRequestContext<'_, '_>,
    width: f64,
    scope: &crate::layout::fit::FitScope,
) -> Result<Vec<crate::layout::ResolvedTextPlan>, crate::layout::fit::PlanError> {
    let width =
        crate::theme::composition::capacity(width, &theme.facet_strip_lengths()?.horizontal_inset)?;
    crate::layout::fit::collect(
        "facet strip roles",
        strip_sources(plot, cell, theme)?.iter().map(|source| {
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
    plot: &PlotDescription,
    cell: &FacetCell,
    theme: &crate::text::TextRequestContext<'_, '_>,
) -> Result<(f64, f64), Error> {
    strip_sources(plot, cell, theme)?.iter().try_fold(
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
    axes: &crate::layout::AxisSystemPlan,
    plot: &PlotDescription,
    cell: &FacetCell,
    theme: &crate::text::TextRequestContext<'_, '_>,
    panel: PixelRect,
    scope: &crate::layout::fit::FitScope,
) -> Result<crate::layout::canvas::EdgeSpaceRequirements, crate::layout::fit::PlanError> {
    let mut requirement =
        crate::layout::canvas::EdgeSpaceRequirements::measure_header(axes, theme)?;
    let plans = strip_plans(plot, cell, theme, panel.width(), scope)?;
    let height = plans.iter().map(|plan| plan.occupied_size().1).sum();
    requirement.top = requirement.top.max(crate::theme::composition::hard(
        &theme
            .facet_strip_lengths()?
            .content_clearance
            .add_pixels(height)?,
    )?);
    Ok(requirement)
}
