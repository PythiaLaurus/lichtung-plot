//! Canvas measurement, panel placement, and final panel planning.

use crate::Error;

use crate::builder::PlotDescription;
use crate::decorations::legend::discrete::scoped::{
    ConsumedGuideDependency, DependencyUse, GuideAllocation, GuideConsumption, GuideOrientation,
    SUPPRESSED_GUIDES,
};
use crate::decorations::plan::LegendRole;
use crate::layout::canvas::EdgeSpaceRequirements;
use crate::layout::kernel::{FeedbackKind, FeedbackRound, FitKind, FrameRequirements};
use crate::layout::{AddressRole, ComponentAddress};
use crate::layout::{PixelRect, RootFramePlan};
use crate::orchestrator::assignment::{
    GuideTrialAssignment, ParentInsets, PlotAllocationInputs, PlotMeasureDimensions, RootAssignment,
};
use crate::orchestrator::registration::RowSelection;
use crate::orchestrator::root::{GridSlot, PreparedFacetTopology, Root};
use crate::spec::FacetScales;
use crate::FacetCell;

pub(super) fn canvas_layout(
    plot: &PlotDescription,
    size: crate::layout::RootSize,
    theme: &crate::text::TextRequestContext<'_, '_>,
    horizontal_axis_label: Option<&crate::Text>,
    vertical_axis_label: Option<&crate::Text>,
    round: &mut FeedbackRound,
) -> Result<RootFramePlan, crate::layout::fit::PlanError> {
    let mut canvas = RootFramePlan::measure(
        size.dimensions()?,
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
    horizontal_axis_label: Option<&crate::Text>,
    vertical_axis_label: Option<&crate::Text>,
    theme: &crate::text::TextRequestContext<'_, '_>,
    reserved_panel: PixelRect,
    candidate_panel: PixelRect,
    round: &mut FeedbackRound,
) -> Result<PixelRect, crate::layout::fit::PlanError> {
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
    horizontal: Option<&crate::Text>,
    vertical: Option<&crate::Text>,
    theme: &crate::text::TextRequestContext<'_, '_>,
    panel: PixelRect,
) -> Result<[f64; 4], crate::layout::fit::PlanError> {
    let mut insets = [0.0; 4];

    for (source, span, index, role) in [
        (horizontal, panel.width(), 2, AddressRole::AxisBottom),
        (vertical, panel.height(), 0, AddressRole::AxisLeft),
    ] {
        if let Some(source) = source {
            let appearance = source.appearance(&theme.typography.axes.axis_title);
            let lengths = theme.axes.get_spacing().lengths(
                theme.typography.axes.tick_label.get_font_size(),
                appearance.theme.get_font_size(),
            )?;
            let gap = if index == 2 {
                &lengths.x_title_without_tick_labels_gap
            } else {
                &lengths.y_title_edge_padding
            };
            let plan = crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                source.as_str(),
                appearance,
                span,
                source
                    .resolved_alignment(crate::TextAlignment::Center)
                    .into(),
                "shared facet axis title",
                &crate::layout::fit::FitScope::root().child(role, 0)?,
            ))?;
            insets[index] = crate::theme::AxisLengths::hard(&gap.add_pixels(plan.height())?)?;
        }
    }
    Ok(insets)
}

fn adjusted_shared_axis_panel(
    horizontal_axis_label: Option<&crate::Text>,
    vertical_axis_label: Option<&crate::Text>,
    theme: &crate::text::TextRequestContext<'_, '_>,
    baseline: [f64; 4],
    candidate: PixelRect,
    round: &mut FeedbackRound,
    address: &ComponentAddress,
) -> Result<(PixelRect, f64, f64), crate::layout::fit::PlanError> {
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
    reason = "facet measurement keeps immutable grid, scale, theme, and child-plan inputs explicit"
)]
pub(super) fn native_parent(
    plot: &PlotDescription,
    root: &Root<'_>,
    topology: &PreparedFacetTopology,
    measurement: &RootAssignment,
    panels: &[(GridSlot, FacetCell, RowSelection)],
    windows: [Option<(f64, f64)>; 2],
    outer: PixelRect,
    shape: (usize, usize),
    theme: &crate::text::TextRequestContext<'_, '_>,
    round: &mut FeedbackRound,
    probes: &[std::sync::Arc<crate::spec::PlotRenderResult>],
    probe_cells: &[PixelRect],
    minima: &[(f64, f64)],
    containing_trial: bool,
    discrete_guide: Option<&std::sync::Arc<crate::decorations::legend::discrete::PreparedGuide>>,
    scope: &crate::layout::fit::FitScope,
    children: &mut crate::decorations::plan::children::ChildPlans,
    content_sizing: bool,
) -> Result<
    crate::layout::fit::FitOutcome<(Vec<super::OwnedFacetChild>, super::tracks::GridAssignment)>,
    Error,
> {
    use crate::layout::fit::{self, FitOutcome};
    let (rows, columns) = shape;
    theme.text.prepared_font_environment()?;
    let grid_owner = ComponentAddress::root().child(AddressRole::FacetGrid, 0)?;
    let mut requirements = Vec::new();
    crate::data::source::reserve_source(&mut requirements, panels.len())?;
    for (index, (slot, _, _)) in panels.iter().enumerate() {
        let address = grid_owner.child(
            AddressRole::FacetCell,
            u32::try_from(index).map_err(|_| Error::Render("facet address overflowed".into()))?,
        )?;
        let [left, right, bottom, top] = round.insets(&address, FeedbackKind::AxisGutter);
        let registered = &root.plots[index];
        let measured = super::rendering::strip_requirement(
            probes[index]
                .axes
                .as_ref()
                .ok_or_else(|| Error::Render("facet probe is missing its measured axes".into()))?,
            registered.source,
            registered
                .scope
                .numeric_owner
                .facet
                .as_ref()
                .expect("prepared facet occurrence"),
            theme,
            probe_cells[slot.0],
            &scope.child(AddressRole::FacetCell, index)?,
        )
        .map_err(fit::PlanError::into_error)?;
        requirements.push((
            slot.0 / columns,
            slot.0 % columns,
            EdgeSpaceRequirements {
                left: left.max(measured.left),
                right: right.max(measured.right),
                bottom: bottom.max(measured.bottom),
                top: top.max(measured.top),
            },
        ));
    }
    let track_sizing = content_sizing.then_some(super::tracks::ContentSizing {
        root,
        topology,
        windows,
        round,
        natural_width: false,
        natural_height: false,
    });
    let grid = match super::tracks::allocate(
        outer,
        (rows, columns),
        panels,
        minima,
        &requirements,
        theme,
        &scope.child(AddressRole::FacetGrid, 0)?,
        track_sizing.as_ref(),
    )? {
        FitOutcome::Measured(grid) => grid,
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
    for (index, (slot, _, _)) in panels.iter().enumerate() {
        let id = topology.slots[slot.0]
            .plot_id
            .as_ref()
            .expect("facet producer");
        let factory = root.plot_assignment_factory(measurement, id)?;
        let registered = root.plot(id)?;
        let domain_address = &registered.scope.address;
        let cell = grid.cells[slot.0];
        let assignment = factory.assign(
            PlotAllocationInputs {
                dimensions: PlotMeasureDimensions {
                    canvas: measurement.measurement_size,
                    output_override: None,
                },
                cell,
                frame: cell,
                body: cell,
                axis_label_width: cell.width(),
                content_height: false,
                fill_content_height: false,
                content_preference: None,
                insets: ParentInsets::consumed(round, domain_address),
                guide: GuideTrialAssignment {
                    edge: plot.legend_position_spec(),
                    compact: false,
                    allocation: None,
                },
            },
            round,
            None,
        )?;
        #[cfg(test)]
        super::observe_factory_phase(id, None, &assignment);
        if containing_trial {
            let body_scope = scope.child(AddressRole::FacetCell, index)?;
            let context = factory.context(&assignment, SUPPRESSED_GUIDES, &body_scope)?;
            if let FitOutcome::Miss(reason) = crate::decorations::plan::check_assigned_body(
                &context,
                &probes[index],
                probe_cells[slot.0],
                cell,
                theme,
                &body_scope,
            )? {
                fit::FitFailure::retain_least(&mut child_miss, reason);
            }
            let strip_scope = scope
                .child(AddressRole::FacetCell, index)?
                .child(AddressRole::FacetHeader, 0)?;
            if let FitOutcome::Miss(reason) = fit::outcome(super::rendering::strip_plans(
                registered.source,
                registered
                    .scope
                    .numeric_owner
                    .facet
                    .as_ref()
                    .expect("prepared facet occurrence"),
                theme,
                cell.width(),
                &strip_scope,
            ))? {
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
        let result = match children.plan(
            &factory,
            assignment,
            role,
            GuideConsumption {
                local_guide: if role == LegendRole::Panel {
                    guide_slot.as_ref()
                } else {
                    None
                },
                ..guides
            },
            &cell_scope,
            round,
            domain_address,
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
        let cell = grid.cells[slot.0];
        let requirement =
            match fit::outcome(super::rendering::strip_requirement(
                result.result.axes.as_ref().ok_or_else(|| {
                    Error::Render("facet panel is missing its measured axes".into())
                })?,
                registered.source,
                registered
                    .scope
                    .numeric_owner
                    .facet
                    .as_ref()
                    .expect("prepared facet occurrence"),
                theme,
                cell,
                &cell_scope,
            ))? {
                FitOutcome::Measured(requirement) => requirement,
                FitOutcome::Miss(failure) => {
                    fit::FitFailure::retain_least(&mut child_miss, failure);
                    continue;
                }
            };
        let address = grid_owner.child(
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
        panel_plans.push(super::OwnedFacetChild {
            slot: *slot,
            plot: result,
            strips: Vec::new(),
            strip_layer: String::new(),
            legend: None,
        });
    }
    if let Some(failure) = child_miss {
        return Ok(FitOutcome::Miss(failure));
    }
    Ok(FitOutcome::Measured((panel_plans, grid)))
}

pub(super) fn feedback_domains(
    plot: &PlotDescription,
    panels: &[(GridSlot, FacetCell, RowSelection)],
    shared: [Option<(f64, f64)>; 2],
    facet_scales: FacetScales,
    round: &FeedbackRound,
) -> Result<[Option<(f64, f64)>; 2], Error> {
    let mut windows = [plot.x_axis_spec(), plot.y_axis_spec()]
        .map(|axis| axis.window.map(crate::layout::RawWindow::pair));
    for side in 0..2 {
        if shared[side].is_some()
            && !(side == 0
                && plot.kind() == crate::PlotKind::Forest
                && plot.axis_scale_spec() == crate::AxisScale::Log10)
        {
            windows[side] = shared[side];
        }
    }
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
            windows[usize::from(kind == FeedbackKind::AutomaticDomainY)] = Some(range);
        }
    }
    Ok(windows)
}
