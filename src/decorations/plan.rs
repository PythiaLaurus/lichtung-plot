//! Immutable panel and legend planning shared by every renderer.

use crate::Error;

use crate::decorations::legend::{Legend, LegendBox};
use crate::layout::fit::{FitFailure, FitScope};
use crate::layout::interior::{ClearanceAxis, DomainExpansion, InteriorReserveRequest};
use crate::layout::kernel::{FeedbackKind, FeedbackRound, FitKind, FrameRequirements};
use crate::layout::AddressRole;
use crate::layout::ComponentAddress;
use crate::layout::PanelBox;
use crate::render::geometry::GeometrySnapshot;
use crate::spec::{
    LegendClearance, LegendDefault, LegendPolicy, LegendPosition, PlotKind, PlotRenderContext,
    PlotRenderResult,
};
use crate::Plot;

pub(crate) mod children;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LegendRole {
    Panel,
    Suppressed,
}

pub(crate) struct ResolvedLegend {
    pub(crate) legend: Legend,
    pub(crate) bounds: LegendBox,
}

/// One measured outside guide and its containing residual body share a single
/// owner-local gap and the same assigned-frame constraint.
pub(crate) struct AssignedGuideFrame {
    edge: LegendPosition,
    minimum_body: (f64, f64),
    gap: f64,
}

impl AssignedGuideFrame {
    pub(crate) const fn new(edge: LegendPosition, minimum_body: (f64, f64), gap: f64) -> Self {
        Self {
            edge,
            minimum_body,
            gap,
        }
    }

    fn horizontal(&self) -> bool {
        matches!(
            self.edge,
            LegendPosition::OutsideTopLeft
                | LegendPosition::OutsideTop
                | LegendPosition::OutsideTopRight
                | LegendPosition::OutsideBottomLeft
                | LegendPosition::OutsideBottom
                | LegendPosition::OutsideBottomRight
        )
    }

    fn vertical(&self) -> bool {
        matches!(
            self.edge,
            LegendPosition::OutsideLeftTop
                | LegendPosition::OutsideLeft
                | LegendPosition::OutsideLeftBottom
                | LegendPosition::OutsideRightTop
                | LegendPosition::OutsideRight
                | LegendPosition::OutsideRightBottom
        )
    }

    pub(crate) fn available(&self, cell: PanelBox, gutters: [f64; 4]) -> (f64, f64) {
        let horizontal = self.horizontal();
        (
            cell.width()
                - gutters[0]
                - gutters[1]
                - if horizontal {
                    0.0
                } else {
                    self.minimum_body.0 + self.gap
                },
            cell.height()
                - gutters[2]
                - gutters[3]
                - if horizontal {
                    self.minimum_body.1 + self.gap
                } else {
                    0.0
                },
        )
    }

    pub(crate) fn assign_fit(
        &self,
        cell: PanelBox,
        guide: Option<(f64, f64)>,
        gutters: [f64; 4],
        scope: &FitScope,
    ) -> Result<crate::layout::fit::FitOutcome<PanelBox>, Error> {
        let mut insets = [0.0; 4];
        if let Some((width, height)) = guide {
            match self.edge {
                LegendPosition::OutsideLeftTop
                | LegendPosition::OutsideLeft
                | LegendPosition::OutsideLeftBottom => insets[0] = width + self.gap,
                LegendPosition::OutsideRightTop
                | LegendPosition::OutsideRight
                | LegendPosition::OutsideRightBottom => insets[1] = width + self.gap,
                LegendPosition::OutsideBottomLeft
                | LegendPosition::OutsideBottom
                | LegendPosition::OutsideBottomRight => insets[2] = height + self.gap,
                LegendPosition::OutsideTopLeft
                | LegendPosition::OutsideTop
                | LegendPosition::OutsideTopRight => insets[3] = height + self.gap,
                _ => {}
            }
        }
        FrameRequirements {
            minimum_body: self.minimum_body,
            natural_insets: insets,
        }
        .assign_fit(cell, gutters, scope)
    }

    pub(crate) fn place(&self, residual: PanelBox, guide: (f64, f64)) -> PanelBox {
        let (width, height) = guide;
        let align = match self.edge {
            LegendPosition::OutsideRightTop
            | LegendPosition::OutsideLeftTop
            | LegendPosition::OutsideTopRight
            | LegendPosition::OutsideBottomRight => 1.0,
            LegendPosition::OutsideRightBottom
            | LegendPosition::OutsideLeftBottom
            | LegendPosition::OutsideTopLeft
            | LegendPosition::OutsideBottomLeft => 0.0,
            _ => 0.5,
        };
        let (left, bottom) = match self.edge {
            LegendPosition::OutsideRightTop
            | LegendPosition::OutsideRight
            | LegendPosition::OutsideRightBottom => (
                residual.x1 + self.gap,
                residual.y0 + (residual.height() - height).max(0.0) * align,
            ),
            LegendPosition::OutsideLeftTop
            | LegendPosition::OutsideLeft
            | LegendPosition::OutsideLeftBottom => (
                residual.x0 - self.gap - width,
                residual.y0 + (residual.height() - height).max(0.0) * align,
            ),
            LegendPosition::OutsideTopLeft
            | LegendPosition::OutsideTop
            | LegendPosition::OutsideTopRight => (
                residual.x0 + (residual.width() - width).max(0.0) * align,
                residual.y1 + self.gap,
            ),
            LegendPosition::OutsideBottomLeft
            | LegendPosition::OutsideBottom
            | LegendPosition::OutsideBottomRight => (
                residual.x0 + (residual.width() - width).max(0.0) * align,
                residual.y0 - self.gap - height,
            ),
            _ => unreachable!("inside guides are collision overlays, not outside tracks"),
        };
        PanelBox::new(left, left + width, bottom, bottom + height)
    }
}

impl ResolvedLegend {
    pub(crate) fn observe_feedback(&self, round: &mut FeedbackRound) -> Result<(), Error> {
        round.observe_box(PanelBox::new(
            self.bounds.left,
            self.bounds.right,
            self.bounds.bottom,
            self.bounds.top,
        ))?;
        let (x, y) = self.bounds.anchor();
        round.observe_number(x)?;
        round.observe_number(y)?;
        self.legend.observe_feedback(round)
    }
}

pub(crate) struct PanelPlan {
    // Content requests expose their preferred frame here; embedded fill may
    // arrange a taller physical panel_box under the owning Grid's row maximum.
    pub(crate) frame_box: PanelBox,
    pub(crate) panel_box: PanelBox,
    pub(crate) result: PlotRenderResult,
    pub(crate) legend: Option<ResolvedLegend>,
    planned_x_limits: Option<(f64, f64)>,
    planned_y_limits: Option<(f64, f64)>,
}

pub(crate) fn place_outside_legend(
    legend: Legend,
    position: LegendPosition,
    cell: PanelBox,
    minimum_body: (f64, f64),
    scope: &FitScope,
) -> Result<crate::layout::fit::FitOutcome<(PanelBox, ResolvedLegend)>, Error> {
    let frame = AssignedGuideFrame::new(position, minimum_body, legend.metrics.outside_gap);
    let panel_box = match frame.assign_fit(
        cell,
        Some((legend.compute_box_width(), legend.compute_box_height())),
        [0.0; 4],
        scope,
    )? {
        crate::layout::fit::FitOutcome::Measured(panel) => panel,
        crate::layout::fit::FitOutcome::Miss(reason) => {
            return Ok(crate::layout::fit::FitOutcome::Miss(reason))
        }
    };
    let bounds = resolve_box(
        &legend,
        &frame,
        position,
        LegendDefault::LowestOverlap,
        panel_box,
        panel_box,
        &[],
    );
    Ok(crate::layout::fit::FitOutcome::Measured((
        panel_box,
        ResolvedLegend { legend, bounds },
    )))
}

pub(crate) const fn is_outside(position: LegendPosition) -> bool {
    matches!(
        position,
        LegendPosition::OutsideRightTop
            | LegendPosition::OutsideRight
            | LegendPosition::OutsideRightBottom
            | LegendPosition::OutsideLeftTop
            | LegendPosition::OutsideLeft
            | LegendPosition::OutsideLeftBottom
            | LegendPosition::OutsideTopLeft
            | LegendPosition::OutsideTop
            | LegendPosition::OutsideTopRight
            | LegendPosition::OutsideBottomLeft
            | LegendPosition::OutsideBottom
            | LegendPosition::OutsideBottomRight
    )
}

const fn is_inside(position: LegendPosition) -> bool {
    !is_outside(position)
}

impl PanelPlan {
    pub(crate) fn plot_with_planned_limits(&self, plot: &Plot) -> Plot {
        let mut plot = plot.clone();
        if let Some(limits) = self.planned_x_limits {
            plot = plot.with_planned_x_limits(limits);
        }
        if let Some(limits) = self.planned_y_limits {
            plot = plot.with_planned_y_limits(limits);
        }
        plot
    }
}

#[cfg(test)]
pub(crate) fn plan_panel(
    plot: &Plot,
    kind: PlotKind,
    cell: PanelBox,
    role: LegendRole,
    theme: &crate::theme::ResolvedTheme,
    round: &mut FeedbackRound,
    address: &ComponentAddress,
) -> Result<crate::layout::fit::FitOutcome<PanelPlan>, Error> {
    let source_rows = plot.configured_source_rows()?;
    let context = source_rows.map_or_else(
        || PlotRenderContext::new(plot.data(), cell, theme),
        |rows| PlotRenderContext::with_source_rows(plot.data(), rows, cell, theme),
    );
    let guide = if role == LegendRole::Panel && kind != PlotKind::Matrix {
        plot.prepared_discrete_guide()?
            .map(|descriptor| {
                crate::decorations::legend::discrete::GuideSlot::new(
                    descriptor,
                    address,
                    plot.legend_position_spec(),
                )
            })
            .transpose()?
    } else {
        None
    };
    plan_panel_with_context(plot, kind, role, context, guide.as_ref(), round, address)
}

#[expect(
    clippy::too_many_lines,
    reason = "the two-pass panel lifecycle keeps discovery, reservation, collision resolution, and frozen output together"
)]
pub(crate) fn plan_panel_with_context(
    plot: &Plot,
    kind: PlotKind,
    role: LegendRole,
    mut context: PlotRenderContext<'_>,
    discrete_guide: Option<&crate::decorations::legend::discrete::GuideSlot>,
    round: &mut FeedbackRound,
    address: &ComponentAddress,
) -> Result<crate::layout::fit::FitOutcome<PanelPlan>, Error> {
    use crate::layout::fit::FitOutcome;
    use crate::layout::kernel::TrialOutcome;

    let minimum = plot.intrinsic_body_minimum(context.theme)?;
    let position = plot.legend_position_spec();
    let bypass_outside_guide = role == LegendRole::Suppressed
        || discrete_guide.is_none()
        || plot.legend_policy_spec() == LegendPolicy::Hidden
        || (position != LegendPosition::Auto && !is_outside(position));
    if context.content_height
        && !matches!(kind, PlotKind::Heatmap | PlotKind::Beeswarm)
        && (!context.fill_content_height || bypass_outside_guide)
    {
        let (panel, preference) = content_frame(plot, context, round, address, None)?;
        context.panel = panel;
        context.content_preference = Some(preference);
    }
    if let FitOutcome::Miss(failure) = (FrameRequirements {
        minimum_body: minimum,
        natural_insets: [0.0; 4],
    })
    .assign_fit(context.panel, [0.0; 4], &FitScope::new(address, 0))?
    {
        return Ok(FitOutcome::Miss(failure));
    }
    if bypass_outside_guide {
        return plan_panel_attempt(
            plot,
            kind,
            role,
            context,
            discrete_guide,
            round,
            address,
            None,
            None,
        );
    }
    let initial_scope = FitScope::new(address, 0);
    let (discovered, discovery) =
        match measure_geometry(plot, kind, context.with_fit_scope(&initial_scope))? {
            FitOutcome::Measured(value) => value,
            FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
        };
    if discovered.decoration_mode == crate::spec::DecorationMode::SelfManaged {
        if context.content_height && context.fill_content_height {
            let (_, preference) = content_frame(plot, context, round, address, None)?;
            context.content_preference = Some(preference);
        }
        return plan_panel_attempt(
            plot,
            kind,
            role,
            context,
            discrete_guide,
            round,
            address,
            None,
            Some((discovered, discovery)),
        );
    }
    let automatic_edges = [
        LegendPosition::OutsideRight,
        LegendPosition::OutsideTop,
        LegendPosition::OutsideBottom,
        LegendPosition::OutsideLeft,
    ];
    let edges: &[LegendPosition] = if position == LegendPosition::Auto {
        &automatic_edges
    } else {
        std::slice::from_ref(&position)
    };
    let mut first_miss = None;
    for (edge_index, &edge) in edges.iter().enumerate() {
        let horizontal = matches!(
            edge,
            LegendPosition::OutsideTopLeft
                | LegendPosition::OutsideTop
                | LegendPosition::OutsideTopRight
                | LegendPosition::OutsideBottomLeft
                | LegendPosition::OutsideBottom
                | LegendPosition::OutsideBottomRight
        );
        let mut compact_changes = true;
        for compact in [false, true] {
            if compact && !compact_changes {
                continue;
            }
            round.claim_guide_trial(
                address,
                if position == LegendPosition::Auto {
                    8
                } else {
                    2
                },
                u32::try_from(edge_index * 2 + usize::from(compact))
                    .map_err(|_| Error::Render("guide attempt ordinal overflowed".into()))?,
            )?;
            let scope = FitScope::new(
                address,
                u32::try_from(edge_index * 2 + usize::from(compact))
                    .map_err(|_| Error::Render("guide attempt ordinal overflowed".into()))?,
            );
            #[cfg(test)]
            let guide_trial =
                crate::orchestrator::measurements::GuideTrial::new(address, edge, compact);
            let guide_scope = FitScope::new(
                &discrete_guide.expect("contributing panel guide").owner,
                scope.ordinal,
            );
            let gutters = round.insets(address, FeedbackKind::AxisGutter);
            let ink = round.insets(address, FeedbackKind::OwnedInk);
            let gap = context.theme.legend_outside_gap()?;
            let frame = AssignedGuideFrame::new(edge, minimum, gap);
            let available = frame.available(
                context.panel,
                [
                    gutters[0] + ink[0],
                    gutters[1] + ink[1],
                    gutters[2] + ink[2],
                    gutters[3] + ink[3],
                ],
            );
            let guide = match Legend::measure_assigned(
                std::sync::Arc::clone(
                    &discrete_guide.expect("contributing panel guide").descriptor,
                ),
                (
                    available.0,
                    (!context.content_height).then_some(available.1),
                ),
                horizontal,
                compact,
                context.theme,
                &mut compact_changes,
                &guide_scope,
            )? {
                FitOutcome::Measured(guide) => guide,
                FitOutcome::Miss(reason) => {
                    FitFailure::retain_least(&mut first_miss, reason);
                    #[cfg(test)]
                    guide_trial.reject();
                    continue;
                }
            };
            let size = (guide.compute_box_width(), guide.compute_box_height());
            let trial_context = if context.content_height {
                let (panel, preference) =
                    content_frame(plot, context, round, address, Some((horizontal, size, gap)))?;
                let mut trial = context.with_panel(panel);
                trial.content_preference = Some(preference);
                trial
            } else {
                context
            };
            #[cfg(test)]
            guide_trial.body();
            let mut assigned_gutters = gutters;
            for (gutter, reserve) in assigned_gutters.iter_mut().zip(ink) {
                *gutter += reserve;
            }
            let panel = match frame.assign_fit(
                trial_context.panel,
                Some(size),
                assigned_gutters,
                &scope,
            )? {
                FitOutcome::Measured(panel) => panel,
                FitOutcome::Miss(reason) => {
                    FitFailure::retain_least(&mut first_miss, reason);
                    #[cfg(test)]
                    guide_trial.reject();
                    continue;
                }
            };
            if let FitOutcome::Miss(reason) = check_assigned_body(
                plot,
                &discovered,
                context.panel,
                panel,
                context.theme,
                &scope,
            )? {
                FitFailure::retain_least(&mut first_miss, reason);
                #[cfg(test)]
                guide_trial.reject();
                continue;
            }
            let checkpoint = round.begin_trial()?;
            round.observe_count(edge as usize)?;
            round.observe_count(usize::from(compact))?;

            let trial_plot = plot.clone().with_legend_position(edge);
            let outcome = plan_panel_attempt(
                &trial_plot,
                kind,
                role,
                trial_context,
                discrete_guide,
                round,
                address,
                Some(guide),
                None,
            )?;
            let plan = match outcome {
                FitOutcome::Measured(plan) => plan,
                FitOutcome::Miss(failure) => {
                    #[cfg(test)]
                    guide_trial.reject();
                    round.record_failure(failure)?;
                    if let TrialOutcome::Miss(failure) = round.finish_trial(checkpoint) {
                        FitFailure::retain_least(&mut first_miss, failure);
                    }
                    continue;
                }
            };
            match round.finish_trial(checkpoint) {
                _outcome @ (TrialOutcome::Fit | TrialOutcome::Provisional) => {
                    #[cfg(test)]
                    #[expect(
                        clippy::used_underscore_binding,
                        reason = "outcome is observed only in tests"
                    )]
                    guide_trial.complete(&_outcome);
                    return Ok(FitOutcome::Measured(plan));
                }
                TrialOutcome::Miss(reason) => {
                    #[cfg(test)]
                    guide_trial.reject();
                    FitFailure::retain_least(&mut first_miss, reason);
                }
            }
        }
    }
    Ok(FitOutcome::Miss(first_miss.unwrap_or_else(|| {
        FitScope::new(address, 0).failure(
            FitKind::LegendOrGuide,
            "outside guide policy found no fitting containing frame",
        )
    })))
}

fn content_frame(
    plot: &Plot,
    context: PlotRenderContext<'_>,
    round: &FeedbackRound,
    address: &ComponentAddress,
    guide: Option<(bool, (f64, f64), f64)>,
) -> Result<(PanelBox, f64), Error> {
    let mut plot = plot.clone();
    if let Some(domain) = round.feedback_domain(address, FeedbackKind::AutomaticDomainX) {
        plot = plot.with_planned_x_limits(domain);
    }
    if let Some(domain) = round.feedback_domain(address, FeedbackKind::AutomaticDomainY) {
        plot = plot.with_planned_y_limits(domain);
    }
    let mut insets = round.insets(address, FeedbackKind::AxisGutter);
    for (inset, ink) in insets
        .iter_mut()
        .zip(round.insets(address, FeedbackKind::OwnedInk))
    {
        *inset += ink;
    }
    let hard = plot.intrinsic_body_minimum(context.theme)?;
    let preference = crate::layout::interior::sizing::body_preference(&plot, context.theme, hard)?;
    let mut width = context.panel.width() - insets[0] - insets[1];
    if let Some((false, (guide_width, _), gap)) = guide {
        width -= guide_width + gap;
    }
    if width < hard.0 {
        return Err(Error::Render(
            "content frame cannot preserve the scientific body width".into(),
        ));
    }
    let mut height = plot.natural_body_height(context.theme, width, preference.1)?;
    if let Some((horizontal, (_, guide_height), gap)) = guide {
        height = if horizontal {
            height + guide_height + gap
        } else {
            height.max(guide_height)
        };
    }
    height += insets[2] + insets[3];
    let assigned = if context.fill_content_height {
        height.max(context.panel.height())
    } else {
        height
    };
    Ok((
        PanelBox::new(
            context.panel.x0,
            context.panel.x1,
            context.panel.y0,
            context.panel.y0 + assigned,
        ),
        height,
    ))
}

#[allow(
    clippy::too_many_lines,
    reason = "one body owner evaluates all assigned fit predicates"
)]
pub(crate) fn check_assigned_body(
    plot: &Plot,
    discovered: &PlotRenderResult,
    cell: PanelBox,
    assigned: PanelBox,
    theme: &crate::theme::ResolvedTheme,
    scope: &FitScope,
) -> Result<crate::layout::fit::FitOutcome<()>, Error> {
    use crate::layout::fit::{self, FitOutcome};
    #[cfg(test)]
    crate::orchestrator::measurements::count(
        crate::orchestrator::measurements::Counter::GuideBodyCheck,
    );
    let old = discovered.layout.projection.panel_box();
    let width = assigned.width() - (cell.width() - old.width());
    let height = assigned.height() - (cell.height() - old.height());
    if !width.is_finite() || !height.is_finite() {
        return Err(Error::Render(
            "assigned body extent arithmetic is not finite".into(),
        ));
    }
    if width <= 0.0 || height <= 0.0 {
        return Ok(FitOutcome::Miss(scope.failure(
            FitKind::TrackOrResidual,
            "guide leaves no assigned data panel",
        )));
    }
    let mut miss = None;

    let mut layout = discovered.layout.clone();
    layout.projection = crate::layout::Projection::new(
        layout.projection.x_range(),
        layout.projection.y_range(),
        (old.x0, old.x0 + width, old.y0, old.y0 + height),
    )
    .with_categories(layout.projection.category_counts());
    // This partial check covers indivisible sources and titles only. Automatic
    // numeric sources are regenerated by the complete frame trial below.
    if discovered.x_domain == crate::spec::AxisDomain::Continuous
        && plot.explicit_x_ticks().is_none()
    {
        layout.x_axis.ticks.clear();
    }
    if discovered.y_domain == crate::spec::AxisDomain::Continuous
        && plot.explicit_y_ticks().is_none()
    {
        layout.y_axis.ticks.clear();
    }
    if let FitOutcome::Miss(reason) = fit::outcome(layout.measure_axes(
        theme,
        scope,
        plot.axis_semantic_inputs([discovered.x_domain, discovered.y_domain]),
    ))? {
        FitFailure::retain_least(&mut miss, reason);
    }
    let annotations = match fit::outcome(crate::decorations::panel_text::measure_plans(
        plot,
        &discovered.layout,
        theme,
        scope,
    ))? {
        FitOutcome::Measured(plans) => plans,
        FitOutcome::Miss(failure) => {
            FitFailure::retain_least(&mut miss, failure);
            Vec::new()
        }
    };
    for (index, annotation) in annotations.iter().enumerate() {
        if annotation.text_plan.width() + 16.0 > width
            || annotation.text_plan.height() + 16.0 > height
        {
            FitFailure::retain_least(
                &mut miss,
                scope.child(AddressRole::Annotation, index)?.failure(
                    FitKind::Text,
                    "panel annotation does not fit the assigned body after guide reservation",
                ),
            );
        }
    }

    let panel = PanelBox::new(
        assigned.x0 + old.x0 - cell.x0,
        assigned.x0 + old.x0 - cell.x0 + width,
        assigned.y0 + old.y0 - cell.y0,
        assigned.y0 + old.y0 - cell.y0 + height,
    );
    for (index, annotation) in annotations.iter().enumerate() {
        let a = annotation.bounds_in(panel);
        for other in &annotations[index + 1..] {
            let b = other.bounds_in(panel);
            if a.x0 < b.x1 && a.x1 > b.x0 && a.y0 < b.y1 && a.y1 > b.y0 {
                FitFailure::retain_least(
                    &mut miss,
                    scope.child(AddressRole::Annotation, index)?.failure(
                        FitKind::OverlayCollision,
                        "panel annotations collide in the assigned containing frame",
                    ),
                );
            }
        }
    }
    Ok(match miss {
        Some(failure) => FitOutcome::Miss(failure),
        None => FitOutcome::Measured(()),
    })
}

#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::similar_names,
    reason = "one immutable frame attempt owns discovery, feedback and retained geometry"
)]
fn plan_panel_attempt(
    plot: &Plot,
    kind: PlotKind,
    role: LegendRole,
    context: PlotRenderContext<'_>,
    discrete_guide: Option<&crate::decorations::legend::discrete::GuideSlot>,
    round: &mut FeedbackRound,
    address: &ComponentAddress,
    assigned_guide: Option<Legend>,
    initial: Option<(PlotRenderResult, GeometrySnapshot)>,
) -> Result<crate::layout::fit::FitOutcome<PanelPlan>, Error> {
    use crate::layout::fit::FitOutcome;
    let scope = FitScope::new(address, round.attempt_ordinal(address));
    let context = context.with_fit_scope(&scope);
    let minimum = plot.intrinsic_body_minimum(context.theme)?;
    let cell = context.panel;
    round.begin_owner(address)?;
    round.observe_box(cell)?;
    let legend_default = kind.legend_default(plot.user_y_limits().is_some());

    let mut planning_plot = plot.clone();
    if role == LegendRole::Suppressed {
        planning_plot = planning_plot.with_suppressed_legend();
    }
    let planned_x_limits = round
        .feedback_domain(address, FeedbackKind::AutomaticDomainX)
        .map(|range| {
            plot.x_axis_spec()
                .limits
                .map_or(range, |base| (base.0.min(range.0), base.1.max(range.1)))
        });
    let planned_y_limits = round
        .feedback_domain(address, FeedbackKind::AutomaticDomainY)
        .map(|range| {
            plot.y_axis_spec()
                .limits
                .map_or(range, |base| (base.0.min(range.0), base.1.max(range.1)))
        });
    if let Some(limits) = planned_x_limits {
        planning_plot = planning_plot.with_planned_x_limits(limits);
    }
    if let Some(limits) = planned_y_limits {
        planning_plot = planning_plot.with_planned_y_limits(limits);
    }
    let discovery_panel = if let Some(guide) = &assigned_guide {
        let mut gutters = round.insets(address, FeedbackKind::AxisGutter);
        for (gutter, ink) in gutters
            .iter_mut()
            .zip(round.insets(address, FeedbackKind::OwnedInk))
        {
            *gutter += ink;
        }
        let frame = AssignedGuideFrame::new(
            planning_plot.legend_position_spec(),
            minimum,
            guide.metrics.outside_gap,
        );
        match frame.assign_fit(
            cell,
            Some((guide.compute_box_width(), guide.compute_box_height())),
            gutters,
            &scope,
        )? {
            FitOutcome::Measured(panel) => panel,
            FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
        }
    } else {
        cell
    };
    // An empty guide can reuse discovery only at the consumed domain. Free
    // facets otherwise keep replaying the pre-feedback provisional payload.
    let initial = initial.filter(|(result, _)| {
        (result.x_domain == crate::spec::AxisDomain::Categorical
            || planned_x_limits.is_none_or(|range| range == result.layout.projection.x_range()))
            && (result.y_domain == crate::spec::AxisDomain::Categorical
                || planned_y_limits.is_none_or(|range| range == result.layout.projection.y_range()))
    });
    let (discovered, discovery) = if let Some(initial) = initial {
        initial
    } else {
        let width = if assigned_guide.is_some() {
            let gutters = round.insets(address, FeedbackKind::AxisGutter);
            let ink = round.insets(address, FeedbackKind::OwnedInk);
            (discovery_panel.width() + gutters[0] + gutters[1] + ink[0] + ink[1] - minimum.0)
                .max(0.0)
        } else {
            context.axis_label_width
        };
        match measure_geometry(
            &planning_plot,
            kind,
            context
                .with_panel(discovery_panel)
                .with_axis_label_width(width),
        )? {
            FitOutcome::Measured(value) => value,
            FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
        }
    };
    let visible = role == LegendRole::Panel
        && planning_plot.legend_policy_spec() != LegendPolicy::Hidden
        && discrete_guide.is_some();
    let legend = if let Some(guide) = assigned_guide {
        Some(guide)
    } else if visible {
        let guide_scope = FitScope::new(
            &discrete_guide.expect("visible panel guide").owner,
            scope.ordinal,
        );
        match Legend::measure_assigned(
            std::sync::Arc::clone(&discrete_guide.expect("visible panel guide").descriptor),
            (discovery_panel.width(), discovery_panel.height()),
            false,
            false,
            context.theme,
            &mut false,
            &guide_scope,
        )? {
            FitOutcome::Measured(guide) => Some(guide),
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        }
    } else {
        None
    };
    let frame = AssignedGuideFrame::new(
        planning_plot.legend_position_spec(),
        minimum,
        legend
            .as_ref()
            .map_or(0.0, |guide| guide.metrics.outside_gap),
    );
    let mut assigned_gutters = round.insets(address, FeedbackKind::AxisGutter);
    let owned_ink = round.insets(address, FeedbackKind::OwnedInk);
    for (gutter, ink) in assigned_gutters.iter_mut().zip(owned_ink) {
        *gutter += ink;
    }
    let panel_box = match frame.assign_fit(
        cell,
        legend
            .as_ref()
            .map(|guide| (guide.compute_box_width(), guide.compute_box_height())),
        assigned_gutters,
        &scope,
    )? {
        FitOutcome::Measured(panel) => panel,
        FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
    };

    let (result, geometry) = if panel_box == discovery_panel {
        (discovered, discovery)
    } else {
        match measure_geometry(
            &planning_plot,
            kind,
            context.with_panel(panel_box).with_axis_label_width(
                (panel_box.width() + assigned_gutters[0] + assigned_gutters[1] - minimum.0)
                    .max(0.0),
            ),
        )? {
            FitOutcome::Measured(value) => value,
            FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
        }
    };
    let position = planning_plot.legend_position_spec();
    let panel_axes = &result.axes;
    let chrome = if let Some(axes) = &panel_axes {
        Some(axes.geometry_snapshot()?)
    } else if let Some(crate::layout::interior::SelectedMarkPlan {
        payload: crate::layout::interior::MarkPayload::Matrix(payload),
        ..
    }) = result.marks.as_deref()
    {
        payload.observe_axes(round)?;
        round.begin_owner(address)?;
        Some(payload.axis_geometry()?)
    } else {
        None
    };
    let mut gutters = result.grid_axis_gutters.unwrap_or([0.0; 4]);
    if let Some(axes) = &panel_axes {
        let required = axes.requirements();
        if !planning_plot.parent_owns_axis_gutters() {
            // The root cell excludes only root-owned padding/title/caption.
            // Replace the entire axis reserve, allowing contraction as well as
            // growth without spending another component's reserved space.

            let clearance = context.theme.panel_edge_length()?;
            for (gutter, value) in gutters.iter_mut().zip([
                required.left,
                required.right,
                required.bottom,
                required.top,
            ]) {
                let quanta = crate::layout::kernel::LayoutScalar::from_css_pixels(
                    value,
                    crate::layout::kernel::SpatialRounding::Ceil,
                )?;
                // Retain the renderer's whole-pixel outward envelope after the
                // complete exact comfort+canonical-axis requirement.
                *gutter = crate::theme::composition::hard(&clearance.add_quanta(quanta)?)?.ceil();
            }
        }
    }
    if let Some(chrome) = &chrome {
        let axes = result.layout.projection.panel_box();
        // Parent tracks retain an outward pixel envelope of the frozen ink.
        if matches!(
            position,
            LegendPosition::OutsideLeftTop
                | LegendPosition::OutsideLeft
                | LegendPosition::OutsideLeftBottom
        ) {
            gutters[0] = chrome
                .bounds()
                .iter()
                .map(|ink| (axes.x0 - ink.left).max(0.0))
                .fold(gutters[0], f64::max)
                .ceil();
        }
        if matches!(
            position,
            LegendPosition::OutsideBottomLeft
                | LegendPosition::OutsideBottom
                | LegendPosition::OutsideBottomRight
        ) {
            gutters[2] = chrome
                .bounds()
                .iter()
                .map(|ink| (axes.y0 - ink.bottom).max(0.0))
                .fold(gutters[2], f64::max)
                .ceil();
        }
    }
    let gutter_request =
        InteriorReserveRequest::AxisGutter(gutters).submit(round, address, address)?;
    if gutter_request.is_some() {
        round.miss(
            address,
            FitKind::Tick,
            &[gutter_request],
            "axis titles or tick labels require a changed parent gutter",
        )?;
    }
    let mut auto_candidate_index = None;
    let mut auto_annotation_miss = false;
    let mut resolved = match crate::layout::fit::outcome(
        legend
            .map(|legend| {
                let bounds = if position == LegendPosition::Auto {
                    let preferred = initial_auto_candidate(
                        &legend,
                        result.layout.projection.panel_box(),
                        geometry.bounds(),
                        legend_default,
                    );
                    let candidate = auto_candidate_avoiding_panel_text(&result, &legend, preferred);
                    auto_annotation_miss = candidate.is_none();
                    // Keep provisional geometry for the failed preferred attempt;
                    // only a resolving pending assignment can make its miss provisional.
                    let index = candidate.unwrap_or(preferred);
                    auto_candidate_index = Some(index);
                    legend.inside_candidates(result.layout.projection.pixel_box())[index]
                } else {
                    resolve_box(
                        &legend,
                        &frame,
                        position,
                        legend_default,
                        result.layout.projection.panel_box(),
                        guide_panel(panel_box, assigned_gutters, position),
                        geometry.bounds(),
                    )
                };
                Ok::<_, crate::layout::fit::PlanError>(ResolvedLegend { legend, bounds })
            })
            .transpose(),
    )? {
        FitOutcome::Measured(legend) => legend,
        FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
    };
    let mut domain_request = None;
    let mut next_x = result.layout.projection.x_range();
    let mut next_y = result.layout.projection.y_range();
    let mark_expansions = result
        .marks
        .as_deref()
        .map_or([None; 2], |marks| match &marks.payload {
            crate::layout::interior::MarkPayload::Scatter(payload) => payload.domain_expansions,
            crate::layout::interior::MarkPayload::Line(payload) => payload.domain_expansions,
            crate::layout::interior::MarkPayload::Qq(payload) => payload.domain_expansions,
            crate::layout::interior::MarkPayload::Regression(payload) => payload.domain_expansions,
            crate::layout::interior::MarkPayload::Density(payload) => payload.domain_expansions,
            crate::layout::interior::MarkPayload::Histogram(payload) => payload.domain_expansions,
            crate::layout::interior::MarkPayload::Interaction(payload) => payload.domain_expansions,
            crate::layout::interior::MarkPayload::Beeswarm(payload) => payload.domain_expansions,
            crate::layout::interior::MarkPayload::Waterfall(payload) => payload.domain_expansions,
            crate::layout::interior::MarkPayload::Marginal(payload) => payload.domain_expansions,
            crate::layout::interior::MarkPayload::Estimate(payload) => payload.domain_expansions,
            crate::layout::interior::MarkPayload::Dumbbell(payload) => payload.domain_expansions,
            crate::layout::interior::MarkPayload::Bars(payload) => [payload.domain_expansion, None],
            crate::layout::interior::MarkPayload::Distribution(payload) => {
                crate::layout::interior::merge_mark_domain_expansions(
                    payload.domain_expansions,
                    [
                        None,
                        result
                            .significance
                            .as_ref()
                            .filter(|plan| plan.y_range != next_y)
                            .map(|plan| DomainExpansion {
                                axis: ClearanceAxis::Y,
                                limits: plan.y_range,
                            }),
                    ],
                )
            }
            _ => [None; 2],
        });
    let mut mark_requests = [None; 2];
    let mut count = 0;
    for expansion in mark_expansions.into_iter().flatten() {
        mark_requests[count] = expansion.submit(round, address, address)?;
        count += 1;
        match expansion.axis {
            ClearanceAxis::X => {
                next_x = (
                    next_x.0.min(expansion.limits.0),
                    next_x.1.max(expansion.limits.1),
                );
            }
            ClearanceAxis::Y => {
                next_y = (
                    next_y.0.min(expansion.limits.0),
                    next_y.1.max(expansion.limits.1),
                );
            }
        }
    }
    if count != 0 {
        round.miss(
            address,
            FitKind::OverlayCollision,
            &mark_requests[..count],
            "mark ink or significance requires automatic domain clearance",
        )?;
    }
    if let Some(legend) = &resolved {
        let collision = overlap_score_within_panel(
            legend.bounds,
            geometry.bounds(),
            legend.legend.metrics.font_size.max(6.0),
            result.layout.projection.panel_box(),
        );
        if is_inside(position) && collision > 0.0 && !auto_annotation_miss {
            let mut replacement_bounds = None;
            let expansion = if position == LegendPosition::Auto {
                let clearance = match crate::layout::fit::outcome(automatic_clearance(
                    kind,
                    plot,
                    &result,
                    &legend.legend,
                    geometry.bounds(),
                    legend_default,
                    auto_candidate_index.unwrap_or(0),
                ))? {
                    FitOutcome::Measured(clearance) => clearance,
                    FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
                };
                if let Some(clearance) = clearance {
                    auto_candidate_index = Some(clearance.candidate_index);
                    replacement_bounds = Some(
                        legend
                            .legend
                            .inside_candidates(result.layout.projection.pixel_box())
                            [clearance.candidate_index],
                    );
                    clearance.expansion
                } else {
                    None
                }
            } else {
                let preferred_edge = clearance_edge(
                    planning_plot.legend_position_spec(),
                    legend.bounds,
                    result.layout.projection.panel_box(),
                );
                explicit_clearance(
                    kind,
                    plot,
                    &result,
                    &legend.legend,
                    legend.bounds,
                    preferred_edge,
                )?
            };
            if let Some(bounds) = replacement_bounds {
                resolved.as_mut().expect("resolved legend exists").bounds = bounds;
            }
            if let Some(DomainExpansion { axis, limits }) = expansion {
                domain_request =
                    DomainExpansion { axis, limits }.submit(round, address, address)?;
                match axis {
                    ClearanceAxis::X => next_x = (next_x.0.min(limits.0), next_x.1.max(limits.1)),
                    ClearanceAxis::Y => next_y = (next_y.0.min(limits.0), next_y.1.max(limits.1)),
                }
            }
        }
    }
    let mut ink_request = None;
    round.observe_box(panel_box)?;
    round.observe_layout(&result.layout)?;
    round.observe_count(usize::from(result.numeric_presentation.is_some()))?;
    if let Some(presentation) = &result.numeric_presentation {
        presentation.observe_feedback(round)?;
    }
    if let Some(text) = &panel_axes {
        text.observe_feedback(round)?;
    }
    if let Some(crate::layout::interior::SelectedMarkPlan {
        payload: crate::layout::interior::MarkPayload::Marginal(payload),
        ..
    }) = result.marks.as_deref()
    {
        payload.observe_subpanels(round)?;
        round.begin_owner(address)?;
    }
    if let Some(crate::layout::interior::SelectedMarkPlan {
        payload: crate::layout::interior::MarkPayload::Beeswarm(payload),
        ..
    }) = result.marks.as_deref()
    {
        payload.observe_sources(round)?;
    }
    if let Some(crate::layout::interior::SelectedMarkPlan {
        payload: crate::layout::interior::MarkPayload::Waterfall(payload),
        ..
    }) = result.marks.as_deref()
    {
        payload.observe_sources(round)?;
    }
    if let Some(chrome) = &chrome {
        let (width, height) = plot.canvas_dimensions()?;
        let mut overflow = [0.0_f64; 4];
        for bounds in chrome.bounds() {
            round.observe_box(PanelBox::new(
                bounds.left,
                bounds.right,
                bounds.bottom,
                bounds.top,
            ))?;
            overflow[0] = overflow[0].max(-bounds.left);
            overflow[1] = overflow[1].max(bounds.right - f64::from(width));
            overflow[2] = overflow[2].max(-bounds.bottom);
            overflow[3] = overflow[3].max(bounds.top - f64::from(height));
        }
        // Measured composition axes are reserved by their parent grid, including
        // overflow from its provisional cell. A child reserve would count it twice.
        // A pending axis gutter already moves this ink inward next round. Only
        // the residual overflow belongs to the containing-frame ink reserve.
        let next_axis_gutters = round.next_insets(address, FeedbackKind::AxisGutter);
        let current_axis_gutters = round.insets(address, FeedbackKind::AxisGutter);
        for (index, extra) in overflow.iter_mut().enumerate() {
            *extra = (*extra - (next_axis_gutters[index] - current_axis_gutters[index]).max(0.0))
                .max(0.0);
        }
        if !planning_plot.parent_owns_axis_gutters() && overflow.iter().any(|value| *value > 0.0) {
            let mut reserves = owned_ink;
            for (reserve, extra) in reserves.iter_mut().zip(overflow) {
                // Reserve outward rather than clamping a frozen text anchor.
                *reserve += extra.ceil();
            }
            ink_request =
                InteriorReserveRequest::OwnedInk(reserves).submit(round, address, address)?;
            round.miss(
                address,
                FitKind::TrackOrResidual,
                &[ink_request],
                "axis ink requires a containing-frame reserve",
            )?;
        }
        chrome.observe_feedback(round)?;
    }
    let mut next_gutters = round.next_insets(address, FeedbackKind::AxisGutter);
    for (gutter, ink) in next_gutters
        .iter_mut()
        .zip(round.next_insets(address, FeedbackKind::OwnedInk))
    {
        *gutter += ink;
    }
    let next_frame = match frame.assign_fit(
        cell,
        resolved.as_ref().map(|guide| {
            (
                guide.legend.compute_box_width(),
                guide.legend.compute_box_height(),
            )
        }),
        next_gutters,
        &FitScope::new(address, round.attempt_ordinal(address)),
    )? {
        crate::layout::fit::FitOutcome::Measured(panel) => panel,
        crate::layout::fit::FitOutcome::Miss(reason) => {
            // Content height and outside flow use consumed insets. Changed
            // gutters must reach their parent before residual height is final.
            if context.content_height || (is_outside(position) && resolved.is_some()) {
                round.miss(
                    address,
                    reason.reason.kind,
                    &[gutter_request, ink_request],
                    reason.message,
                )?;
            } else {
                round.record_failure(reason)?;
            }
            panel_box
        }
    };
    let axes = result.layout.projection.panel_box();
    let next_axes = PanelBox::new(
        axes.x0 + next_frame.x0 - panel_box.x0,
        axes.x1 + next_frame.x1 - panel_box.x1,
        axes.y0 + next_frame.y0 - panel_box.y0,
        axes.y1 + next_frame.y1 - panel_box.y1,
    );
    let next_projection = crate::layout::Projection::new(next_x, next_y, next_axes.as_tuple())
        .with_categories(result.layout.projection.category_counts());
    if let Some(legend) = &resolved {
        let mut resolving_auto_bounds = None;
        if auto_annotation_miss && (gutter_request.is_some() || ink_request.is_some()) {
            let annotations =
                match crate::layout::fit::outcome(crate::decorations::panel_text::measure_plans(
                    &planning_plot,
                    &result.layout,
                    context.theme,
                    &scope,
                ))? {
                    FitOutcome::Measured(plans) => plans,
                    FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
                };
            let candidates = legend.legend.inside_candidates(next_axes.as_tuple());
            let preferred = auto_candidate_index.expect("automatic preferred attempt");
            for offset in 0..candidates.len() {
                #[cfg(test)]
                crate::orchestrator::measurements::count(
                    crate::orchestrator::measurements::Counter::InsideLegendCandidate,
                );
                let bounds = candidates[(preferred + offset) % candidates.len()];
                if legend_fits_panel(bounds, next_axes)
                    && crate::decorations::panel_text::conflict_resolved_by_assignment(
                        &annotations,
                        next_axes,
                        bounds,
                    )
                {
                    resolving_auto_bounds = Some(bounds);
                    break;
                }
            }
        }
        let next_bounds = if position == LegendPosition::Auto {
            resolving_auto_bounds.unwrap_or_else(|| {
                legend.legend.inside_candidates(next_axes.as_tuple())
                    [auto_candidate_index.expect("automatic legend candidate")]
            })
        } else {
            resolve_box(
                &legend.legend,
                &frame,
                position,
                legend_default,
                next_axes,
                guide_panel(next_frame, next_gutters, position),
                geometry.bounds(),
            )
        };
        let allocation_requests = [gutter_request, ink_request];
        if is_outside(position) {
            if let Some(chrome) = &chrome {
                for ink in chrome.bounds() {
                    #[cfg(test)]
                    crate::orchestrator::measurements::count(
                        crate::orchestrator::measurements::Counter::GeometryOverlapScan,
                    );
                    let overlaps =
                        |left: f64, right: f64, bottom: f64, top: f64, guide: LegendBox| {
                            left < guide.right
                                && right > guide.left
                                && bottom < guide.top
                                && top > guide.bottom
                        };
                    if overlaps(ink.left, ink.right, ink.bottom, ink.top, legend.bounds) {
                        let dx = next_axes.x0 - axes.x0;
                        let dy = next_axes.y0 - axes.y0;
                        let resolved = !overlaps(
                            ink.left + dx,
                            ink.right + dx,
                            ink.bottom + dy,
                            ink.top + dy,
                            next_bounds,
                        );
                        round.miss(
                            address,
                            FitKind::TrackOrResidual,
                            if resolved { &allocation_requests } else { &[] },
                            "outside guide overlaps measured axis ink in its containing frame",
                        )?;
                        break;
                    }
                }
            }
        }
        if is_inside(position) {
            let corner = crate::decorations::panel_text::conflicting_legend_corner(
                &result.panel_text,
                legend.bounds,
            );
            if let Some(corner) = corner {
                let resolved_by_assignment = (!auto_annotation_miss
                    || resolving_auto_bounds.is_some())
                    && crate::decorations::panel_text::conflict_resolved_by_assignment(
                        &result.panel_text,
                        next_axes,
                        next_bounds,
                    );
                let message = if auto_annotation_miss {
                    "automatic legend cannot avoid the configured panel text annotations; hide the legend or choose an outside legend position".to_owned()
                } else {
                    format!("panel text annotation at {corner:?} is incompatible with the resolved inside legend; hide the legend or choose an outside legend position")
                };
                round.miss(
                    address,
                    FitKind::OverlayCollision,
                    if resolved_by_assignment {
                        &allocation_requests
                    } else {
                        &[]
                    },
                    message,
                )?;
            }
            let fits = legend_fits_panel(legend.bounds, axes);
            let overlap = overlap_score_within_panel(
                legend.bounds,
                geometry.bounds(),
                legend.legend.metrics.font_size.max(6.0),
                axes,
            );
            if !fits || overlap > 0.0 {
                let annotations = &result.panel_text;
                let allocation_can_help = legend_fits_panel(next_bounds, next_axes)
                    && geometry.overlap_after_assignment(
                        axes,
                        next_axes,
                        crate::render::geometry::GeometryBox {
                            left: next_bounds.left,
                            right: next_bounds.right,
                            bottom: next_bounds.bottom,
                            top: next_bounds.top,
                            weight: 1.0,
                        },
                        legend.legend.metrics.font_size.max(6.0),
                        |plan| {
                            annotations
                                .iter()
                                .find(|annotation| annotation.text_plan == *plan)
                                .map(|annotation| annotation.placement_in(next_axes))
                        },
                    )? <= 0.0;
                let requests = [
                    if fits { domain_request } else { None },
                    if allocation_can_help {
                        gutter_request
                    } else {
                        None
                    },
                    if allocation_can_help {
                        ink_request
                    } else {
                        None
                    },
                ];
                let placement = if position == LegendPosition::Auto {
                    "automatic legend".to_owned()
                } else {
                    format!("explicit inside legend {position:?}")
                };
                round.miss(address, FitKind::OverlayCollision, &requests, format!("{placement} cannot fit inside the plot without overlapping recorded geometry (in_bounds={fits}, overlap_score={overlap:.3}); increase the canvas, shorten legend labels, choose an outside position, or provide more automatic axis range"))?;
            }
        } else {
            let fits_track = |panel: PanelBox| {
                (!frame.vertical() || legend.legend.compute_box_height() <= panel.height())
                    && (!frame.horizontal() || legend.legend.compute_box_width() <= panel.width())
            };
            if !fits_track(panel_box) {
                round.miss(
                    address,
                    FitKind::TrackOrResidual,
                    if fits_track(next_frame) {
                        &allocation_requests
                    } else {
                        &[]
                    },
                    "measured outside legend does not fit its assigned frame track",
                )?;
            }
        }
    }
    for annotation in &result.deferred_annotation_text {
        round.observe_text(&annotation.plan)?;
    }
    if let Some(significance) = &result.significance {
        round.observe_number(significance.y_range.0)?;
        round.observe_number(significance.y_range.1)?;
        for bracket in &significance.brackets {
            round.observe_text(&bracket.label.plan)?;
        }
    }
    round.observe_count(geometry.bounds().len())?;
    let (width, height) = plot.canvas_dimensions()?;
    for bounds in geometry.bounds() {
        round.observe_box(PanelBox::new(
            bounds.left,
            bounds.right,
            bounds.bottom,
            bounds.top,
        ))?;
        round.observe_number(bounds.weight)?;
        if bounds.left < 0.0
            || bounds.right > f64::from(width)
            || bounds.bottom < 0.0
            || (!context.content_height && bounds.top > f64::from(height))
        {
            if result.grid_axis_gutters.is_some()
                && chrome
                    .as_ref()
                    .is_some_and(|axes| axes.bounds().contains(bounds))
            {
                // The Grid's frozen outer axes have their own concrete gutter/ink request above.
                continue;
            }
            let declared = result
                .deferred_annotation_text
                .iter()
                .map(|text| &text.plan)
                .chain(
                    result
                        .producer_text
                        .iter()
                        .filter(|_| kind == PlotKind::Waterfall)
                        .map(|text| &text.plan),
                )
                .chain(
                    result
                        .significance
                        .iter()
                        .flat_map(|plan| plan.brackets.iter().map(|bracket| &bracket.label.plan)),
                );
            let resolved_by_projection = geometry.text_overflow_resolved(
                *bounds,
                PanelBox::new(0.0, f64::from(width), 0.0, f64::from(height)),
                &result.layout.projection,
                &next_projection,
                declared,
            )?;
            let requests = [
                gutter_request,
                ink_request,
                domain_request,
                mark_requests[0],
                mark_requests[1],
            ];
            round.miss(
                address,
                FitKind::OverlayCollision,
                if resolved_by_projection {
                    &requests
                } else {
                    &[]
                },
                "unowned painted geometry lies outside the exact root",
            )?;
        }
    }
    round.observe_count(usize::from(auto_annotation_miss))?;
    round.observe_count(auto_candidate_index.map_or(0, |index| index + 1))?;
    round.observe_count(usize::from(resolved.is_some()))?;
    if let Some(legend) = &resolved {
        legend.observe_feedback(round)?;
    }
    geometry.observe_feedback(round)?;
    Ok(FitOutcome::Measured(PanelPlan {
        frame_box: if context.fill_content_height {
            PanelBox::new(
                cell.x0,
                cell.x1,
                cell.y0,
                cell.y0 + context.content_preference.unwrap_or(cell.height()),
            )
        } else if context.content_height
            && matches!(
                kind,
                PlotKind::Heatmap | PlotKind::Beeswarm | PlotKind::Matrix
            )
        {
            PanelBox::new(
                cell.x0,
                cell.x1,
                cell.y0,
                result.layout.projection.panel_box().y1 + assigned_gutters[3],
            )
        } else {
            cell
        },
        panel_box,
        result,
        legend: resolved,
        planned_x_limits,
        planned_y_limits,
    }))
}

fn measure_geometry(
    plot: &Plot,
    kind: PlotKind,
    context: PlotRenderContext<'_>,
) -> Result<crate::layout::fit::FitOutcome<(PlotRenderResult, GeometrySnapshot)>, Error> {
    use crate::layout::fit::{self, FitOutcome};
    match kind {
        PlotKind::Scatter
        | PlotKind::Line
        | PlotKind::Trajectory
        | PlotKind::LinePath
        | PlotKind::Histogram
        | PlotKind::Density
        | PlotKind::Qq
        | PlotKind::Regression
        | PlotKind::Dumbbell
        | PlotKind::PointInterval
        | PlotKind::Coefficient
        | PlotKind::Forest
        | PlotKind::Bar
        | PlotKind::HorizontalBar
        | PlotKind::StackedBar
        | PlotKind::StackedHorizontalBar
        | PlotKind::Boxplot
        | PlotKind::Violin
        | PlotKind::Raincloud
        | PlotKind::Interaction
        | PlotKind::Heatmap
        | PlotKind::Beeswarm
        | PlotKind::Waterfall
        | PlotKind::Marginal
        | PlotKind::Matrix => match fit::outcome(plot.measure_selected_interior(context))? {
            FitOutcome::Measured(result) => {
                let mut geometry = GeometrySnapshot::new();
                geometry.append(
                    &result
                        .marks
                        .as_ref()
                        .ok_or_else(|| Error::Render("selected mark plan missing".into()))?
                        .geometry,
                )?;
                geometry.append(
                    &result
                        .annotations
                        .as_ref()
                        .ok_or_else(|| Error::Render("selected annotation plan missing".into()))?
                        .geometry,
                )?;
                Ok(FitOutcome::Measured((result, geometry)))
            }
            FitOutcome::Miss(failure) => Ok(FitOutcome::Miss(failure)),
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClearanceEdge {
    Left,
    Right,
    Bottom,
    Top,
}

#[derive(Debug, Clone, Copy)]
struct AutomaticClearance {
    candidate_index: usize,
    expansion: Option<DomainExpansion>,
}

fn clearance_edge(position: LegendPosition, legend: LegendBox, panel: PanelBox) -> ClearanceEdge {
    match position {
        LegendPosition::InsideRightTop
        | LegendPosition::InsideRight
        | LegendPosition::InsideRightBottom => ClearanceEdge::Right,
        LegendPosition::InsideLeftTop
        | LegendPosition::InsideLeft
        | LegendPosition::InsideLeftBottom => ClearanceEdge::Left,
        LegendPosition::InsideBottomLeft
        | LegendPosition::InsideBottom
        | LegendPosition::InsideBottomRight => ClearanceEdge::Bottom,
        LegendPosition::Auto
        | LegendPosition::InsideTopLeft
        | LegendPosition::InsideTop
        | LegendPosition::InsideTopRight => ClearanceEdge::Top,
        _ => {
            let distances = [
                (legend.left - panel.x0, ClearanceEdge::Left),
                (panel.x1 - legend.right, ClearanceEdge::Right),
                (legend.bottom - panel.y0, ClearanceEdge::Bottom),
                (panel.y1 - legend.top, ClearanceEdge::Top),
            ];
            distances
                .into_iter()
                .min_by(|left, right| left.0.total_cmp(&right.0))
                .map_or(ClearanceEdge::Top, |(_, edge)| edge)
        }
    }
}

fn automatic_clearance(
    kind: PlotKind,
    plot: &Plot,
    result: &PlotRenderResult,
    legend: &Legend,
    geometry: &[crate::render::geometry::GeometryBox],
    default: LegendDefault,
    current_candidate: usize,
) -> Result<Option<AutomaticClearance>, crate::layout::fit::PlanError> {
    match default {
        LegendDefault::TopRight(clearance) => top_right_clearance(
            kind,
            plot,
            result,
            legend,
            geometry,
            clearance,
            current_candidate,
        ),
        LegendDefault::LowestOverlap | LegendDefault::SelfManaged => {
            lowest_overlap_clearance(kind, plot, result, legend, geometry)
        }
    }
}

fn top_right_clearance(
    kind: PlotKind,
    plot: &Plot,
    result: &PlotRenderResult,
    legend: &Legend,
    geometry: &[crate::render::geometry::GeometryBox],
    clearance: LegendClearance,
    current_candidate: usize,
) -> Result<Option<AutomaticClearance>, crate::layout::fit::PlanError> {
    let margin = legend.metrics.font_size.max(6.0);
    let candidates = legend.inside_candidates(result.layout.projection.pixel_box());
    for (candidate_index, bounds) in candidates
        .iter()
        .copied()
        .enumerate()
        .skip(current_candidate)
    {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::InsideLegendCandidate,
        );
        if crate::decorations::panel_text::conflicting_legend_corner(&result.panel_text, bounds)
            .is_some()
        {
            continue;
        }
        if !legend_fits_panel(bounds, result.layout.projection.panel_box()) {
            continue;
        }
        if overlap_score_within_panel(
            bounds,
            geometry,
            margin,
            result.layout.projection.panel_box(),
        ) <= 0.0
        {
            return Ok(Some(AutomaticClearance {
                candidate_index,
                expansion: None,
            }));
        }
        for edge in candidate_edges(candidate_index, clearance) {
            if let Some(expansion) = expanded_axis_limits(kind, plot, result, legend, bounds, edge)?
            {
                return Ok(Some(AutomaticClearance {
                    candidate_index,
                    expansion: Some(expansion),
                }));
            }
        }
    }
    Ok(None)
}

fn candidate_edges(index: usize, clearance: LegendClearance) -> [ClearanceEdge; 2] {
    let horizontal = match index {
        0 | 2 => ClearanceEdge::Right,
        _ => ClearanceEdge::Left,
    };
    let vertical = match index {
        0 | 1 => ClearanceEdge::Top,
        _ => ClearanceEdge::Bottom,
    };
    match clearance {
        LegendClearance::HorizontalMaximum => [horizontal, vertical],
        LegendClearance::VerticalMaximum => [vertical, horizontal],
    }
}

fn lowest_overlap_clearance(
    kind: PlotKind,
    plot: &Plot,
    result: &PlotRenderResult,
    legend: &Legend,
    geometry: &[crate::render::geometry::GeometryBox],
) -> Result<Option<AutomaticClearance>, crate::layout::fit::PlanError> {
    let margin = legend.metrics.font_size.max(6.0);
    let candidates = legend.inside_candidates(result.layout.projection.pixel_box());
    let edges = [
        [ClearanceEdge::Right, ClearanceEdge::Top],
        [ClearanceEdge::Left, ClearanceEdge::Top],
        [ClearanceEdge::Right, ClearanceEdge::Bottom],
        [ClearanceEdge::Left, ClearanceEdge::Bottom],
    ];

    let mut best = None;
    for (candidate_index, bounds) in candidates.into_iter().enumerate() {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::InsideLegendCandidate,
        );
        if !legend_fits_panel(bounds, result.layout.projection.panel_box())
            || crate::decorations::panel_text::conflicting_legend_corner(&result.panel_text, bounds)
                .is_some()
        {
            continue;
        }
        for (edge_index, edge) in edges[candidate_index].into_iter().enumerate() {
            let Some(DomainExpansion { axis, limits }) =
                expanded_axis_limits(kind, plot, result, legend, bounds, edge)?
            else {
                continue;
            };
            let range = match axis {
                ClearanceAxis::X => result.layout.projection.x_range(),
                ClearanceAxis::Y => result.layout.projection.y_range(),
            };
            let candidate = (
                overlap_score_within_panel(
                    bounds,
                    geometry,
                    margin,
                    result.layout.projection.panel_box(),
                ),
                expansion_ratio(range, limits)?,
                candidate_index,
                edge_index,
                axis,
                limits,
            );
            if best.as_ref().is_none_or(
                |current: &(f64, f64, usize, usize, ClearanceAxis, (f64, f64))| {
                    candidate
                        .0
                        .total_cmp(&current.0)
                        .then_with(|| candidate.1.total_cmp(&current.1))
                        .then_with(|| candidate.2.cmp(&current.2))
                        .then_with(|| candidate.3.cmp(&current.3))
                        .is_lt()
                },
            ) {
                best = Some(candidate);
            }
        }
    }
    Ok(best.map(
        |(_, _, candidate_index, _, axis, limits)| AutomaticClearance {
            candidate_index,
            expansion: Some(DomainExpansion { axis, limits }),
        },
    ))
}

fn explicit_clearance(
    kind: PlotKind,
    plot: &Plot,
    result: &PlotRenderResult,
    legend: &Legend,
    bounds: LegendBox,
    preferred_edge: ClearanceEdge,
) -> Result<Option<DomainExpansion>, Error> {
    let mut best = None;
    for (index, edge) in [
        preferred_edge,
        ClearanceEdge::Left,
        ClearanceEdge::Right,
        ClearanceEdge::Bottom,
        ClearanceEdge::Top,
    ]
    .into_iter()
    .enumerate()
    {
        let Some(DomainExpansion { axis, limits }) =
            expanded_axis_limits(kind, plot, result, legend, bounds, edge)?
        else {
            continue;
        };
        let range = match axis {
            ClearanceAxis::X => result.layout.projection.x_range(),
            ClearanceAxis::Y => result.layout.projection.y_range(),
        };
        let expansion = expansion_ratio(range, limits)?;
        if best
            .as_ref()
            .is_none_or(|current: &(f64, usize, ClearanceAxis, (f64, f64))| {
                expansion
                    .total_cmp(&current.0)
                    .then_with(|| index.cmp(&current.1))
                    .is_lt()
            })
        {
            best = Some((expansion, index, axis, limits));
        }
    }
    Ok(best.map(|(_, _, axis, limits)| DomainExpansion { axis, limits }))
}

fn clearance_finite(value: f64, operation: &str) -> Result<f64, Error> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(Error::Render(format!(
            "legend clearance arithmetic overflow in {operation}"
        )))
    }
}

fn expansion_ratio(range: (f64, f64), limits: (f64, f64)) -> Result<f64, Error> {
    let original = clearance_finite(range.1 - range.0, "original domain span")?;
    let expanded = clearance_finite(limits.1 - limits.0, "expanded domain span")?;
    if original <= 0.0 || expanded <= 0.0 {
        return Err(Error::Render(
            "legend clearance requires positive domain spans".into(),
        ));
    }
    let ratio = clearance_finite(expanded / original, "expansion ratio")?;
    if ratio < 1.0 {
        return Err(Error::Render(
            "legend clearance ratio cannot shrink a domain".into(),
        ));
    }
    Ok(ratio)
}

fn legend_fits_panel(legend: LegendBox, panel: PanelBox) -> bool {
    const EPSILON: f64 = 1e-6;
    legend.left >= panel.x0 - EPSILON
        && legend.right <= panel.x1 + EPSILON
        && legend.bottom >= panel.y0 - EPSILON
        && legend.top <= panel.y1 + EPSILON
}

fn overlap_score_within_panel(
    legend: LegendBox,
    geometry: &[crate::render::geometry::GeometryBox],
    margin: f64,
    panel: PanelBox,
) -> f64 {
    let legend = crate::render::geometry::GeometryBox {
        left: legend.left,
        right: legend.right,
        bottom: legend.bottom,
        top: legend.top,
        weight: 1.0,
    };
    geometry
        .iter()
        .filter_map(|bounds| {
            #[cfg(test)]
            crate::orchestrator::measurements::count(
                crate::orchestrator::measurements::Counter::GeometryOverlapScan,
            );
            let clipped = crate::render::geometry::GeometryBox {
                left: bounds.left.max(panel.x0),
                right: bounds.right.min(panel.x1),
                bottom: bounds.bottom.max(panel.y0),
                top: bounds.top.min(panel.y1),
                weight: bounds.weight,
            };
            (clipped.left <= clipped.right && clipped.bottom <= clipped.top).then_some(clipped)
        })
        .map(|bounds| bounds.overlap_area(legend, margin))
        .sum()
}

fn expanded_axis_limits(
    kind: PlotKind,
    plot: &Plot,
    result: &PlotRenderResult,
    legend: &Legend,
    bounds: LegendBox,
    edge: ClearanceEdge,
) -> Result<Option<DomainExpansion>, Error> {
    let panel = result.layout.projection.panel_box();
    let width = clearance_finite(panel.width(), "panel width")?;
    let height = clearance_finite(panel.height(), "panel height")?;
    if width <= 0.0 || height <= 0.0 {
        return Err(Error::Render(
            "legend clearance requires positive panel spans".into(),
        ));
    }
    let margin = legend.metrics.font_size.max(6.0);
    let (axis, range, fraction, expand_minimum) = match edge {
        ClearanceEdge::Left if result.x_domain == crate::spec::AxisDomain::Continuous => (
            ClearanceAxis::X,
            result.layout.projection.x_range(),
            (panel.x1 - bounds.right - margin) / width,
            true,
        ),
        ClearanceEdge::Right if result.x_domain == crate::spec::AxisDomain::Continuous => (
            ClearanceAxis::X,
            result.layout.projection.x_range(),
            (bounds.left - margin - panel.x0) / width,
            false,
        ),
        ClearanceEdge::Bottom if result.y_domain == crate::spec::AxisDomain::Continuous => (
            ClearanceAxis::Y,
            result.layout.projection.y_range(),
            (panel.y1 - bounds.top - margin) / height,
            true,
        ),
        ClearanceEdge::Top if result.y_domain == crate::spec::AxisDomain::Continuous => (
            ClearanceAxis::Y,
            result.layout.projection.y_range(),
            (bounds.bottom - margin - panel.y0) / height,
            false,
        ),
        _ => return Ok(None),
    };
    if matches!(axis, ClearanceAxis::X) && plot.user_x_limits().is_some()
        || matches!(axis, ClearanceAxis::Y) && plot.user_y_limits().is_some()
    {
        return Ok(None);
    }
    let (minimum, maximum) = range;
    if bar_expansion_moves_zero_anchor(kind, axis, range, expand_minimum) {
        return Ok(None);
    }
    clearance_finite(minimum, "domain minimum")?;
    clearance_finite(maximum, "domain maximum")?;
    let span = clearance_finite(maximum - minimum, "original domain span")?;
    let fraction = clearance_finite(fraction, "available panel fraction")?;
    if span <= 0.0 {
        return Err(Error::Render(
            "legend clearance requires a positive domain span".into(),
        ));
    }
    if fraction <= 0.05 {
        return Ok(None);
    }
    let expanded_span = clearance_finite(
        span / fraction.clamp(0.05, 0.95) * 1.05,
        "expanded domain span",
    )?;
    let limits = if expand_minimum {
        (maximum - expanded_span, maximum)
    } else {
        (minimum, minimum + expanded_span)
    };
    clearance_finite(limits.0, "expanded domain minimum")?;
    clearance_finite(limits.1, "expanded domain maximum")?;
    expansion_ratio(range, limits)?;
    Ok((limits != range).then_some(DomainExpansion { axis, limits }))
}

fn bar_expansion_moves_zero_anchor(
    kind: PlotKind,
    axis: ClearanceAxis,
    range: (f64, f64),
    expand_minimum: bool,
) -> bool {
    let numeric_bar_axis = matches!(
        (kind, axis),
        (PlotKind::Bar | PlotKind::StackedBar, ClearanceAxis::Y)
            | (
                PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar,
                ClearanceAxis::X
            )
    );
    numeric_bar_axis && ((expand_minimum && range.0 == 0.0) || (!expand_minimum && range.1 == 0.0))
}

fn auto_candidate_avoiding_panel_text(
    result: &PlotRenderResult,
    legend: &Legend,
    preferred: usize,
) -> Option<usize> {
    let panel = result.layout.projection.panel_box();
    let candidates = legend.inside_candidates(panel.as_tuple());
    for offset in 0..candidates.len() {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::InsideLegendCandidate,
        );
        let index = (preferred + offset) % candidates.len();
        if crate::decorations::panel_text::conflict_resolved_by_assignment(
            &result.panel_text,
            panel,
            candidates[index],
        ) {
            return Some(index);
        }
    }
    None
}

fn guide_panel(mut panel: PanelBox, gutters: [f64; 4], position: LegendPosition) -> PanelBox {
    match position {
        LegendPosition::OutsideLeftTop
        | LegendPosition::OutsideLeft
        | LegendPosition::OutsideLeftBottom => panel.x0 -= gutters[0],
        LegendPosition::OutsideRightTop
        | LegendPosition::OutsideRight
        | LegendPosition::OutsideRightBottom => panel.x1 += gutters[1],
        LegendPosition::OutsideBottomLeft
        | LegendPosition::OutsideBottom
        | LegendPosition::OutsideBottomRight => panel.y0 -= gutters[2],
        LegendPosition::OutsideTopLeft
        | LegendPosition::OutsideTop
        | LegendPosition::OutsideTopRight => panel.y1 += gutters[3],
        _ => {}
    }
    panel
}

fn initial_auto_candidate(
    legend: &Legend,
    panel: PanelBox,
    geometry: &[crate::render::geometry::GeometryBox],
    default: LegendDefault,
) -> usize {
    let pixel_box = panel.as_tuple();
    match default {
        LegendDefault::TopRight(_) | LegendDefault::SelfManaged => 0,
        LegendDefault::LowestOverlap => {
            let margin = legend.metrics.font_size.max(6.0);
            legend
                .inside_candidates(pixel_box)
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    #[cfg(test)]
                    {
                        crate::orchestrator::measurements::count(
                            crate::orchestrator::measurements::Counter::InsideLegendCandidate,
                        );
                        crate::orchestrator::measurements::count(
                            crate::orchestrator::measurements::Counter::InsideLegendCandidate,
                        );
                    }
                    overlap_score_within_panel(**left, geometry, margin, panel).total_cmp(
                        &overlap_score_within_panel(**right, geometry, margin, panel),
                    )
                })
                .map_or(0, |(index, _)| index)
        }
    }
}

fn resolve_box(
    legend: &Legend,
    frame: &AssignedGuideFrame,
    position: LegendPosition,
    default: LegendDefault,
    axes: PanelBox,
    reserved_panel: PanelBox,
    geometry: &[crate::render::geometry::GeometryBox],
) -> LegendBox {
    let pixel_box = axes.as_tuple();
    match position {
        LegendPosition::Auto => legend.inside_candidates(pixel_box)
            [initial_auto_candidate(legend, axes, geometry, default)],
        LegendPosition::InsideTopLeft | LegendPosition::InsideLeftTop => {
            legend.aligned_inside_box(pixel_box, 0.0, 1.0)
        }
        LegendPosition::InsideTop => legend.aligned_inside_box(pixel_box, 0.5, 1.0),
        LegendPosition::InsideTopRight | LegendPosition::InsideRightTop => {
            legend.aligned_inside_box(pixel_box, 1.0, 1.0)
        }
        LegendPosition::InsideRight => legend.aligned_inside_box(pixel_box, 1.0, 0.5),
        LegendPosition::InsideRightBottom | LegendPosition::InsideBottomRight => {
            legend.aligned_inside_box(pixel_box, 1.0, 0.0)
        }
        LegendPosition::InsideBottomLeft | LegendPosition::InsideLeftBottom => {
            legend.aligned_inside_box(pixel_box, 0.0, 0.0)
        }
        LegendPosition::InsideBottom => legend.aligned_inside_box(pixel_box, 0.5, 0.0),
        LegendPosition::InsideLeft => legend.aligned_inside_box(pixel_box, 0.0, 0.5),
        LegendPosition::OutsideRightTop
        | LegendPosition::OutsideRight
        | LegendPosition::OutsideRightBottom
        | LegendPosition::OutsideLeftTop
        | LegendPosition::OutsideLeft
        | LegendPosition::OutsideLeftBottom
        | LegendPosition::OutsideTopLeft
        | LegendPosition::OutsideTop
        | LegendPosition::OutsideTopRight
        | LegendPosition::OutsideBottomLeft
        | LegendPosition::OutsideBottom
        | LegendPosition::OutsideBottomRight => {
            let rect = frame.place(
                reserved_panel,
                (legend.compute_box_width(), legend.compute_box_height()),
            );
            legend.box_at(rect.x0, rect.y0)
        }
    }
}

#[cfg(test)]
pub(crate) fn plan_panel_for_test<K: crate::kind::Marker>(
    plot: &Plot<K>,
    kind: PlotKind,
    cell: PanelBox,
    role: LegendRole,
    theme: &crate::theme::ResolvedTheme,
) -> Result<PanelPlan, Error> {
    let mut output = None;
    let source = crate::kind::into_erased(plot.clone());
    crate::layout::plan_exact_root(
        plot.canvas_dimensions()?.0,
        plot.canvas_dimensions()?.1,
        |stage| {
            let inputs =
                crate::plots::shared_x::prepare_plot(&source, &stage.operation(), Some(theme))?;
            inputs.prepare_discrete_guides(
                &source,
                theme,
                &stage.operation(),
                &ComponentAddress::root(),
                false,
            )?;
            Ok(inputs)
        },
        |inputs, assigned, round| {
            let plot = inputs.attach(source.clone())?;
            Ok(crate::layout::MeasuredLayout::new(
                assigned,
                crate::layout::fit::measured(plan_panel(
                    &plot,
                    kind,
                    cell,
                    role,
                    theme,
                    round,
                    &ComponentAddress::root(),
                ))
                .map_err(crate::layout::fit::PlanError::into_error)?,
            ))
        },
        |_, _, plan, _| {
            output = Some(plan);
            Ok(())
        },
    )?;
    Ok(output.expect("converged panel was finalized"))
}
