//! Canvas measurement, panel placement, and final panel planning.

use crate::Error;

use super::data::FacetPanel;
use super::rendering::facet_label;
use crate::builder::Plot;
use crate::decorations::legend::discrete::scoped::{
    ConsumedGuideDependency, DependencyUse, GuideAllocation, GuideConsumption, GuideOrientation,
    SUPPRESSED_GUIDES,
};
use crate::decorations::plan::{LegendRole, PanelPlan};
use crate::layout::canvas::PanelRequirements;
use crate::layout::kernel::{FeedbackKind, FeedbackRound, FitKind, FrameRequirements};
use crate::layout::{AddressRole, ComponentAddress};
use crate::layout::{PanelBox, PhysicalTextAlignment, RootFramePlan};
use crate::spec::{FacetScales, PlotRenderContext};
use crate::theme::ResolvedTheme;

pub(super) fn canvas_layout(
    plot: &Plot,
    theme: &ResolvedTheme,
    horizontal_axis_label: Option<&str>,
    vertical_axis_label: Option<&str>,
    round: &mut FeedbackRound,
) -> Result<RootFramePlan, crate::layout::fit::PlanError> {
    let mut canvas = RootFramePlan::measure(
        plot.canvas_dimensions()?,
        theme,
        plot.prose(),
        crate::decorations::title_block::plan::RootAlignments::resolve(
            theme,
            plot.prose_alignments(),
            [None; 2],
        ),
        (
            crate::layout::canvas::MINIMUM_PANEL_SPAN,
            crate::layout::canvas::MINIMUM_PANEL_SPAN,
        ),
    )?;
    let (panel, extra_left, extra_bottom) = adjusted_shared_axis_panel(
        horizontal_axis_label,
        vertical_axis_label,
        theme,
        [0.0; 4],
        canvas.panel_box,
        round,
        &ComponentAddress::root().child(AddressRole::RootBody, 0)?,
    )?;
    canvas.panel_box = panel;
    canvas.insets.left += extra_left;
    canvas.insets.bottom += extra_bottom;
    Ok(canvas)
}

pub(super) fn reserve_rewrapped_shared_axis_titles(
    horizontal_axis_label: Option<&str>,
    vertical_axis_label: Option<&str>,
    theme: &ResolvedTheme,
    reserved_panel: PanelBox,
    candidate_panel: PanelBox,
    round: &mut FeedbackRound,
) -> Result<PanelBox, crate::layout::fit::PlanError> {
    let baseline = shared_axis_requirements(
        horizontal_axis_label,
        vertical_axis_label,
        theme,
        reserved_panel,
    )?;
    adjusted_shared_axis_panel(
        horizontal_axis_label,
        vertical_axis_label,
        theme,
        baseline,
        candidate_panel,
        round,
        &ComponentAddress::root().child(AddressRole::FacetGrid, 0)?,
    )
    .map(|(panel, _, _)| panel)
}

/// Shared titles own their measured perpendicular extents and theme clearance.
/// Root padding and the scientific Grid remain independent reservations.
pub(super) fn shared_axis_requirements(
    horizontal: Option<&str>,
    vertical: Option<&str>,
    theme: &ResolvedTheme,
    panel: PanelBox,
) -> Result<[f64; 4], crate::layout::fit::PlanError> {
    let mut insets = [0.0; 4];

    let lengths = theme.axes.get_spacing().lengths(
        theme.typography.axes.tick_label.get_font_size(),
        theme.typography.axes.axis_title.get_font_size(),
    )?;
    for (source, span, index, gap, role) in [
        (
            horizontal,
            panel.width(),
            2,
            &lengths.x_title_without_tick_labels_gap,
            AddressRole::AxisBottom,
        ),
        (
            vertical,
            panel.height(),
            0,
            &lengths.y_title_edge_padding,
            AddressRole::AxisLeft,
        ),
    ] {
        if let Some(source) = source {
            let plan = crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                source,
                &theme.typography.axes.axis_title,
                span,
                PhysicalTextAlignment::Center,
                "shared facet axis title",
                &crate::layout::fit::FitScope::root().child(role, 0)?,
            ))?;
            insets[index] = crate::theme::AxisLengths::hard(&gap.add_pixels(plan.height())?)?;
        }
    }
    Ok(insets)
}

fn adjusted_shared_axis_panel(
    horizontal_axis_label: Option<&str>,
    vertical_axis_label: Option<&str>,
    theme: &ResolvedTheme,
    baseline: [f64; 4],
    candidate: PanelBox,
    round: &mut FeedbackRound,
    address: &ComponentAddress,
) -> Result<(PanelBox, f64, f64), crate::layout::fit::PlanError> {
    let consumed = round.insets(address, FeedbackKind::AxisGutter);
    let initial =
        shared_axis_requirements(horizontal_axis_label, vertical_axis_label, theme, candidate)?;
    let mut allocated = consumed;
    for index in [0, 2] {
        allocated[index] = allocated[index].max((initial[index] - baseline[index]).max(0.0));
    }
    let positive_span = crate::layout::canvas::MINIMUM_PANEL_SPAN;
    let panel = crate::layout::fit::measured(
        FrameRequirements {
            minimum_body: (positive_span, positive_span),
            natural_insets: [0.0; 4],
        }
        .assign_fit(
            candidate,
            allocated,
            &crate::layout::fit::FitScope::new(address, 0),
        ),
    )?;
    let measured =
        shared_axis_requirements(horizontal_axis_label, vertical_axis_label, theme, panel)?;
    let extra_bottom = (measured[2] - baseline[2]).max(0.0);
    let extra_left = (measured[0] - baseline[0]).max(0.0);
    if candidate.width() - extra_left < positive_span
        || candidate.height() - extra_bottom < positive_span
    {
        return Err(crate::layout::fit::FitScope::new(address, 0).failure(FitKind::TrackOrResidual,
                "multiline shared facet axis titles leave no positive child region; increase chart size"
            ).into());
    }
    round.begin_owner(address)?;
    round.observe_box(panel)?;
    if let Some(request) = round.request_insets(
        address,
        address,
        FeedbackKind::AxisGutter,
        [extra_left, 0.0, extra_bottom, 0.0],
    )? {
        round.miss(
            address,
            FitKind::Tick,
            &[Some(request)],
            "shared facet axis title requires a changed gutter",
        )?;
    }
    Ok((panel, allocated[0], allocated[2]))
}

#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::type_complexity,
    reason = "facet measurement keeps immutable grid, scale, theme, and child-plan inputs explicit"
)]
pub(super) fn native_parent(
    plot: &Plot,
    panels: &mut [FacetPanel],
    outer: PanelBox,
    shape: (usize, usize),
    facet_scales: FacetScales,
    theme: &ResolvedTheme,
    round: &mut FeedbackRound,
    probes: &[std::sync::Arc<crate::spec::PlotRenderResult>],
    probe_cells: &[PanelBox],
    minima: &[(f64, f64)],
    containing_trial: bool,
    discrete_guide: Option<&std::sync::Arc<crate::decorations::legend::discrete::PreparedGuide>>,
    scope: &crate::layout::fit::FitScope,
    children: &mut crate::decorations::plan::children::ChildPlans,
    content_sizing: bool,
    inputs: Option<&crate::layout::interior::PreparedInteriorInputs>,
) -> Result<crate::layout::fit::FitOutcome<(Plot, Vec<std::sync::Arc<PanelPlan>>)>, Error> {
    use crate::layout::fit::{self, FitOutcome};
    let (rows, columns) = shape;
    let resources = theme.prepared_font_environment()?.operation();
    let data = plot.data().with_operation(&resources);
    let grid = ComponentAddress::root().child(AddressRole::FacetGrid, 0)?;
    let mut requirements = Vec::new();
    crate::data::source::reserve_source(&mut requirements, panels.len())?;
    for (index, facet) in panels.iter().enumerate() {
        let address = grid.child(
            AddressRole::FacetCell,
            u32::try_from(index).map_err(|_| Error::Render("facet address overflowed".into()))?,
        )?;
        let [left, right, bottom, top] = round.insets(&address, FeedbackKind::AxisGutter);
        let label = facet_label(plot, &facet.cell, data)?;
        let measured = PanelRequirements::measure(
            probes[index]
                .axes
                .as_ref()
                .ok_or_else(|| Error::Render("facet probe is missing its measured axes".into()))?,
            theme,
            Some(&label),
            probe_cells[facet.row * columns + facet.column],
            &scope.child(AddressRole::FacetCell, index)?,
        )
        .map_err(fit::PlanError::into_error)?;
        requirements.push((
            facet.row,
            facet.column,
            PanelRequirements {
                left: left.max(measured.left),
                right: right.max(measured.right),
                bottom: bottom.max(measured.bottom),
                top: top.max(measured.top),
            },
        ));
    }
    let track_sizing = content_sizing.then(|| super::tracks::ContentSizing {
        plot,
        inputs: inputs.expect("content facet root retains prepared inputs"),
        round,
        natural_width: false,
        natural_height: false,
    });
    let cells = match super::tracks::allocate(
        data,
        outer,
        (rows, columns),
        panels,
        minima,
        &requirements,
        theme,
        &scope.child(AddressRole::FacetGrid, 0)?,
        track_sizing.as_ref(),
    )? {
        FitOutcome::Measured(grid) => grid.cells,
        FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
    };
    let mut child_miss = None;
    let mut panel_plans = Vec::new();
    crate::data::source::reserve_source(&mut panel_plans, panels.len())?;
    let mut legend_owner = !containing_trial;
    let guide_slot = discrete_guide
        .map(|descriptor| {
            crate::decorations::legend::discrete::GuideSlot::new(
                std::sync::Arc::clone(descriptor),
                &ComponentAddress::root(),
                plot.legend_position_spec(),
            )
        })
        .transpose()?;
    for (index, facet) in panels.iter().enumerate() {
        let mut panel_plot = plot
            .clone()
            .with_render_scope(facet.render_scope()?)
            .with_significance_subset(facet.significance_selectors());
        let domain_address = ComponentAddress::root().child(
            AddressRole::FacetCell,
            u32::try_from(index).map_err(|_| Error::Render("facet address overflowed".into()))?,
        )?;
        if matches!(facet_scales, FacetScales::Free | FacetScales::FreeX) {
            if let Some(range) =
                round.feedback_domain(&domain_address, FeedbackKind::AutomaticDomainX)
            {
                panel_plot = panel_plot.with_planned_x_limits(range);
            }
        }
        if matches!(facet_scales, FacetScales::Free | FacetScales::FreeY) {
            if let Some(range) =
                round.feedback_domain(&domain_address, FeedbackKind::AutomaticDomainY)
            {
                panel_plot = panel_plot.with_planned_y_limits(range);
            }
        }
        let cell = cells[facet.row * columns + facet.column];
        if containing_trial {
            if let FitOutcome::Miss(reason) = crate::decorations::plan::check_assigned_body(
                &panel_plot,
                &probes[index],
                probe_cells[facet.row * columns + facet.column],
                cell,
                theme,
                &scope.child(AddressRole::FacetCell, index)?,
            )? {
                fit::FitFailure::retain_least(&mut child_miss, reason);
            }
            let label = facet_label(&panel_plot, &facet.cell, data)?;
            let appearance = &theme.typography.composition.facet_header;
            let strip_scope = scope
                .child(AddressRole::FacetCell, index)?
                .child(AddressRole::FacetHeader, 0)?;
            let text_width = crate::theme::composition::capacity(
                cell.width(),
                &theme.facet_strip_lengths()?.horizontal_inset,
            )?;
            if text_width <= 0.0 {
                fit::FitFailure::retain_least(
                    &mut child_miss,
                    strip_scope.failure(FitKind::Text, "facet strip has no assigned text width"),
                );
            } else if let FitOutcome::Miss(reason) = fit::text(
                theme,
                &label,
                appearance,
                text_width,
                PhysicalTextAlignment::Center,
                "facet strip",
                &strip_scope,
            )? {
                fit::FitFailure::retain_least(&mut child_miss, reason);
            }
        }
        let cell_scope = scope.child(AddressRole::FacetCell, index)?;
        let role = if legend_owner {
            LegendRole::Panel
        } else {
            LegendRole::Suppressed
        };
        let dependency = guide_slot
            .as_ref()
            .filter(|_| containing_trial || role == LegendRole::Panel)
            .map(|slot| {
                let (use_kind, allocation) = if role == LegendRole::Panel {
                    (
                        DependencyUse::DescriptorAndAllocation {
                            descriptor: std::sync::Arc::clone(&slot.descriptor),
                            orientation: GuideOrientation::from_horizontal(false),
                            compact: false,
                        },
                        cell,
                    )
                } else {
                    (DependencyUse::AllocationOnly, outer)
                };
                ConsumedGuideDependency::new(
                    slot.owner.clone(),
                    use_kind,
                    GuideAllocation::bounds(allocation)?,
                    round,
                )
            })
            .transpose()?;
        let guides = GuideConsumption {
            dependencies: dependency.as_slice(),
            ..SUPPRESSED_GUIDES
        };
        let context = PlotRenderContext::new(
            crate::data::PlotData::with_rows(&panel_plot, &facet.rows, &facet.source_rows),
            cell,
            theme,
        )
        .with_facet_scales(facet_scales)
        .with_fit_scope(&cell_scope)
        .with_guides(&guides);
        let result = match children.plan(
            &panel_plot,
            role,
            context,
            if role == LegendRole::Panel {
                guide_slot.as_ref()
            } else {
                None
            },
            round,
            &domain_address,
        )? {
            FitOutcome::Measured(result) => result,
            FitOutcome::Miss(failure) => {
                fit::FitFailure::retain_least(&mut child_miss, failure);
                continue;
            }
        };
        if result.legend.is_some() {
            legend_owner = false;
        }
        let label = facet_label(&panel_plot, &facet.cell, data)?;
        let cell = cells[facet.row * columns + facet.column];
        let requirement =
            match fit::outcome(PanelRequirements::measure(
                result.result.axes.as_ref().ok_or_else(|| {
                    Error::Render("facet panel is missing its measured axes".into())
                })?,
                theme,
                Some(&label),
                cell,
                &cell_scope,
            ))? {
                FitOutcome::Measured(requirement) => requirement,
                FitOutcome::Miss(failure) => {
                    fit::FitFailure::retain_least(&mut child_miss, failure);
                    continue;
                }
            };
        let address = grid.child(
            AddressRole::FacetCell,
            u32::try_from(index).map_err(|_| Error::Render("facet address overflowed".into()))?,
        )?;
        round.begin_owner(&address)?;
        round.observe_box(cell)?;
        round.observe_layout(&result.result.layout)?;
        if let Some(request) = round.request_insets(
            &address,
            &address,
            FeedbackKind::AxisGutter,
            [
                requirement.left,
                requirement.right,
                requirement.bottom,
                requirement.top,
            ],
        )? {
            round.miss(
                &address,
                FitKind::Tick,
                &[Some(request)],
                "facet cell requires measured decoration gutters",
            )?;
        }
        panel_plans.push(result);
    }
    if let Some(failure) = child_miss {
        return Ok(FitOutcome::Miss(failure));
    }
    for facet in panels {
        facet.panel = cells[facet.row * columns + facet.column];
    }
    Ok(FitOutcome::Measured((plot.clone(), panel_plans)))
}

pub(super) fn feedback_plot(
    mut plot: Plot,
    panels: &[FacetPanel],
    facet_scales: FacetScales,
    round: &FeedbackRound,
) -> Result<Plot, Error> {
    let fix_x = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeY);
    let fix_y = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeX);
    for (fixed, kind) in [
        (fix_x, FeedbackKind::AutomaticDomainX),
        (fix_y, FeedbackKind::AutomaticDomainY),
    ] {
        if !fixed {
            continue;
        }
        let mut shared: Option<(f64, f64)> = None;
        for index in 0..panels.len() {
            let address = ComponentAddress::root().child(
                AddressRole::FacetCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("facet address overflowed".into()))?,
            )?;
            if let Some(range) = round.feedback_domain(&address, kind) {
                shared = Some(
                    shared.map_or(range, |prior| (prior.0.min(range.0), prior.1.max(range.1))),
                );
            }
        }
        if let Some(range) = shared {
            plot = if kind == FeedbackKind::AutomaticDomainX {
                plot.with_planned_x_limits(range)
            } else {
                plot.with_planned_y_limits(range)
            };
        }
    }
    Ok(plot)
}
