//! Immutable panel and legend planning shared by every renderer.

use crate::Error;

use crate::decorations::legend::{Legend, LegendBox};
use crate::layout::fit::{FitFailure, FitScope};
use crate::layout::kernel::{FeedbackKind, FeedbackRound, FitKind, FrameRequirements};
use crate::layout::AddressRole;
use crate::layout::ComponentAddress;
use crate::layout::PanelBox;
use crate::render::backend::{self, Backend};
use crate::render::geometry::GeometryBackend;
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
    pub(crate) panel_box: PanelBox,
    pub(crate) result: PlotRenderResult,
    pub(crate) legend: Option<ResolvedLegend>,
    pub(crate) axis_text: Option<crate::layout::PanelAxisTextPlan>,
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
    let mut requirements = frame_requirements(Some(&legend), position);
    requirements.minimum_body = minimum_body;
    let panel_box = match requirements.assign_fit(cell, [0.0; 4], scope)? {
        crate::layout::fit::FitOutcome::Measured(panel) => panel,
        crate::layout::fit::FitOutcome::Miss(reason) => {
            return Ok(crate::layout::fit::FitOutcome::Miss(reason))
        }
    };
    let bounds = resolve_box(
        &legend,
        position,
        LegendDefault::LowestOverlap,
        panel_box,
        panel_box,
        cell,
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
    pub(crate) fn plot_with_planned_limits<K: crate::kind::Marker>(&self, plot: &Plot<K>) -> Plot {
        let plot = crate::kind::into_erased(plot.clone());
        let plot = self.planned_x_limits.map_or_else(
            || plot.clone(),
            |limits| plot.clone().with_planned_x_limits(limits),
        );
        self.planned_y_limits
            .map_or(plot.clone(), |limits| plot.with_planned_y_limits(limits))
    }

    #[cfg(test)]
    pub(crate) const fn planned_x_limits(&self) -> Option<(f64, f64)> {
        self.planned_x_limits
    }

    #[cfg(test)]
    pub(crate) const fn planned_y_limits(&self) -> Option<(f64, f64)> {
        self.planned_y_limits
    }
}

pub(crate) fn plan_panel<K: crate::kind::Marker>(
    plot: &Plot<K>,
    kind: PlotKind,
    cell: PanelBox,
    role: LegendRole,
    theme: &crate::theme::ResolvedTheme,
    round: &mut FeedbackRound,
    address: &ComponentAddress,
) -> Result<crate::layout::fit::FitOutcome<PanelPlan>, Error> {
    let plot = crate::kind::into_erased(plot.clone());
    let source_rows = plot.configured_source_rows()?;
    let context = source_rows.map_or_else(
        || PlotRenderContext::new(plot.data(), cell, theme),
        |rows| PlotRenderContext::with_source_rows(plot.data(), rows, cell, theme),
    );
    plan_panel_with_context(&plot, kind, role, context, None, round, address)
}

#[expect(
    clippy::too_many_lines,
    reason = "the two-pass panel lifecycle keeps discovery, reservation, collision resolution, and frozen output together"
)]
pub(crate) fn plan_panel_with_context(
    plot: &Plot,
    kind: PlotKind,
    role: LegendRole,
    context: PlotRenderContext<'_>,
    legend_entries: Option<&[crate::decorations::legend::LegendEntry]>,
    round: &mut FeedbackRound,
    address: &ComponentAddress,
) -> Result<crate::layout::fit::FitOutcome<PanelPlan>, Error> {
    use crate::layout::fit::FitOutcome;
    use crate::layout::kernel::TrialOutcome;
    let position = plot.legend_position_spec();
    if role == LegendRole::Suppressed
        || plot.legend_policy_spec() == LegendPolicy::Hidden
        || (position != LegendPosition::Auto && !is_outside(position))
    {
        return plan_panel_attempt(
            plot,
            kind,
            role,
            context,
            legend_entries,
            round,
            address,
            None,
            None,
        );
    }
    let initial_scope = FitScope::new(address, 0);
    let (mut discovered, discovery) =
        match measure_geometry(plot, kind, context.with_fit_scope(&initial_scope))? {
            FitOutcome::Measured(value) => value,
            FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
        };
    if let Some(entries) = legend_entries {
        discovered.legend_entries = entries.to_vec();
    }
    if discovered.legend_entries.is_empty()
        || discovered.decoration_mode == crate::spec::DecorationMode::SelfManaged
    {
        return plan_panel_attempt(
            plot,
            kind,
            role,
            context,
            legend_entries,
            round,
            address,
            None,
            Some((discovered, discovery)),
        );
    }
    let title = crate::decorations::legend::resolved_title(&discovered.legend_entries)?;
    let entries: std::sync::Arc<[crate::decorations::legend::LegendEntry]> =
        std::mem::take(&mut discovered.legend_entries).into();
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
            let mut guide_scope =
                scope.child(crate::decorations::legend::address_role(position), 0)?;
            guide_scope.ordinal = scope.ordinal;
            let gutters = round.insets(address, FeedbackKind::AxisGutter);
            let ink = round.insets(address, FeedbackKind::OwnedInk);
            let metrics = crate::decorations::legend::LegendMetrics::from_canvas(
                f64::from(plot.width()),
                f64::from(plot.height()),
            );
            let gap = metrics.pad.max(8.0);
            let available = (
                context.panel.width()
                    - gutters[0]
                    - gutters[1]
                    - ink[0]
                    - ink[1]
                    - if horizontal { 0.0 } else { 80.0 + gap },
                context.panel.height()
                    - gutters[2]
                    - gutters[3]
                    - ink[2]
                    - ink[3]
                    - if horizontal { 60.0 + gap } else { 0.0 },
            );
            let guide = match Legend::measure_outside(
                &title,
                std::sync::Arc::clone(&entries),
                (plot.width(), plot.height()),
                available,
                horizontal,
                compact,
                context.theme,
                &mut compact_changes,
                &guide_scope,
            )? {
                FitOutcome::Measured(guide) => guide,
                FitOutcome::Miss(reason) => {
                    FitFailure::retain_least(&mut first_miss, reason);
                    continue;
                }
            };
            let requirements = frame_requirements(Some(&guide), edge);
            let mut assigned_gutters = gutters;
            for (gutter, reserve) in assigned_gutters.iter_mut().zip(ink) {
                *gutter += reserve;
            }
            let panel = match requirements.assign_fit(context.panel, assigned_gutters, &scope)? {
                FitOutcome::Measured(panel) => panel,
                FitOutcome::Miss(reason) => {
                    FitFailure::retain_least(&mut first_miss, reason);
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
                context,
                legend_entries,
                round,
                address,
                Some(guide),
                None,
            )?;
            let plan = match outcome {
                FitOutcome::Measured(plan) => plan,
                FitOutcome::Miss(failure) => {
                    round.record_failure(failure)?;
                    if let TrialOutcome::Miss(failure) = round.finish_trial(checkpoint) {
                        FitFailure::retain_least(&mut first_miss, failure);
                    }
                    continue;
                }
            };
            match round.finish_trial(checkpoint) {
                TrialOutcome::Fit | TrialOutcome::Provisional => {
                    return Ok(FitOutcome::Measured(plan))
                }
                TrialOutcome::Miss(reason) => {
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

#[allow(
    clippy::too_many_lines,
    reason = "one body owner evaluates all assigned fit predicates"
)]
#[expect(
    clippy::cast_precision_loss,
    reason = "assigned axes and annotations share one bounded body fit check"
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
    for (source, span, owner, role) in [
        (
            &discovered.layout.x_axis.label,
            width,
            "x-axis title",
            AddressRole::AxisBottom,
        ),
        (
            &discovered.layout.y_axis.label,
            height,
            "y-axis title",
            AddressRole::AxisLeft,
        ),
    ] {
        if !source.is_empty() {
            if let FitOutcome::Miss(reason) = fit::text(
                theme,
                source,
                &theme.typography.axis_title,
                span,
                crate::layout::PhysicalTextAlignment::Center,
                owner,
                &scope.child(role, 0)?,
            )? {
                FitFailure::retain_least(&mut miss, reason);
            }
        }
    }
    if discovered.x_domain == crate::spec::AxisDomain::Categorical
        || plot.explicit_x_ticks().is_some()
    {
        let span = width / discovered.layout.x_axis.ticks.len().max(1) as f64;
        for tick in &discovered.layout.x_axis.ticks {
            if let FitOutcome::Miss(reason) = fit::text(
                theme,
                &tick.label,
                &theme.typography.tick_label,
                span,
                crate::layout::PhysicalTextAlignment::Center,
                "x-axis tick label",
                &scope.child(AddressRole::AxisBottom, 0)?,
            )? {
                FitFailure::retain_least(&mut miss, reason);
            }
        }
    }
    if discovered.y_domain == crate::spec::AxisDomain::Categorical
        || plot.explicit_y_ticks().is_some()
    {
        let labels = &discovered.layout.y_axis.ticks;
        let available = discovered.layout.y_axis.tick_label_width(
            PanelBox::new(0.0, width, 0.0, height),
            crate::layout::AxisSide::Left,
            theme,
        )?;
        let row_height = height / labels.len().max(1) as f64;
        for tick in labels.iter().filter(|tick| !tick.label.is_empty()) {
            match fit::text(
                theme,
                &tick.label,
                &theme.typography.tick_label,
                available,
                crate::layout::PhysicalTextAlignment::End,
                "y-axis tick label",
                &scope.child(AddressRole::AxisLeft, 0)?,
            )? {
                FitOutcome::Measured(plan) if plan.height() + 4.0 > row_height => {
                    FitFailure::retain_least(
                        &mut miss,
                        scope.child(AddressRole::AxisLeft, 0)?.failure(
                            FitKind::Tick,
                            "y-axis labels exceed their assigned rows after guide reservation",
                        ),
                    );
                }
                FitOutcome::Measured(_) => {}
                FitOutcome::Miss(reason) => FitFailure::retain_least(&mut miss, reason),
            }
        }
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
    let pairs = (annotations.len() as u128) * (annotations.len().saturating_sub(1) as u128) / 2;
    theme
        .prepared_font_environment()?
        .claim_text_generation_work(pairs)
        .map_err(|error| Error::Render(format!("assigned annotation collision work: {error:?}")))?;
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
    legend_entries: Option<&[crate::decorations::legend::LegendEntry]>,
    round: &mut FeedbackRound,
    address: &ComponentAddress,
    assigned_guide: Option<Legend>,
    initial: Option<(PlotRenderResult, GeometryBackend)>,
) -> Result<crate::layout::fit::FitOutcome<PanelPlan>, Error> {
    use crate::layout::fit::FitOutcome;
    let scope = FitScope::new(address, round.attempt_ordinal(address));
    let context = context.with_fit_scope(&scope);
    let cell = context.panel;
    round.begin_owner(address)?;
    round.observe_box(cell)?;
    let legend_default = kind.legend_default(plot.user_y_limits().is_some());
    let mut planning_plot = match role {
        LegendRole::Panel => plot.clone(),
        LegendRole::Suppressed => plot.clone().with_suppressed_legend(),
    };
    let planned_x_limits = round
        .domain(address, FeedbackKind::AutomaticDomainX)
        .map(|range| {
            plot.x_axis_spec()
                .limits
                .map_or(range, |base| (base.0.min(range.0), base.1.max(range.1)))
        });
    let planned_y_limits = round
        .domain(address, FeedbackKind::AutomaticDomainY)
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
        match frame_requirements(Some(guide), planning_plot.legend_position_spec())
            .assign_fit(cell, gutters, &scope)?
        {
            FitOutcome::Measured(panel) => panel,
            FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
        }
    } else {
        cell
    };
    let (mut discovered, discovery) = match initial {
        Some(initial) => initial,
        None => {
            match measure_geometry(&planning_plot, kind, context.with_panel(discovery_panel))? {
                FitOutcome::Measured(value) => value,
                FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
            }
        }
    };
    if let Some(entries) = legend_entries {
        discovered.legend_entries = entries.to_vec();
    }
    let visible = role == LegendRole::Panel
        && planning_plot.legend_policy_spec() != LegendPolicy::Hidden
        && !discovered.legend_entries.is_empty();
    let legend = if let Some(guide) = assigned_guide {
        Some(guide)
    } else if visible {
        Some(Legend::new(
            &crate::decorations::legend::resolved_title(&discovered.legend_entries)?,
            discovered.legend_entries.clone(),
            f64::from(planning_plot.width()),
            f64::from(planning_plot.height()),
            context.theme,
        )?)
    } else {
        None
    };
    let requirements = frame_requirements(legend.as_ref(), planning_plot.legend_position_spec());
    let mut assigned_gutters = round.insets(address, FeedbackKind::AxisGutter);
    let owned_ink = round.insets(address, FeedbackKind::OwnedInk);
    for (gutter, ink) in assigned_gutters.iter_mut().zip(owned_ink) {
        *gutter += ink;
    }
    let panel_box = match requirements.assign_fit(cell, assigned_gutters, &scope)? {
        FitOutcome::Measured(panel) => panel,
        FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
    };

    let (result, geometry) = if panel_box == discovery_panel {
        (discovered, discovery)
    } else {
        match measure_geometry(&planning_plot, kind, context.with_panel(panel_box))? {
            FitOutcome::Measured(value) => value,
            FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
        }
    };
    let position = planning_plot.legend_position_spec();
    let axis_text = (result.decoration_mode == crate::spec::DecorationMode::Standard)
        .then(|| result.layout.plan_axis_text(context.theme))
        .transpose()?;
    let chrome = if let Some(text) = &axis_text {
        let mut chrome = GeometryBackend::new(context.theme)?;
        chrome.begin(plot.width(), plot.height())?;
        let drawn = result
            .layout
            .draw_decorations_with_text(context.theme, text, &mut chrome);
        backend::finish_scope(drawn, chrome.end(), "assigned axis ink measurement")?;
        Some(chrome)
    } else {
        None
    };
    let mut gutters = match crate::layout::fit::outcome(planning_plot.measured_axis_title_gutters(
        kind,
        panel_box,
        &result.layout,
        context.theme,
        &scope,
    ))? {
        FitOutcome::Measured(gutters) => gutters,
        FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
    };
    if planning_plot.y_axis_spec().exact_ticks.is_some() && !planning_plot.axis_text_is_measured() {
        let required = match crate::layout::fit::outcome(
            crate::layout::canvas::PanelRequirements::measure_header(
                &result.layout,
                context.theme,
                None,
                None,
                panel_box,
                &scope,
            ),
        )? {
            FitOutcome::Measured(requirement) => requirement,
            FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
        };
        let base = planning_plot.base_canvas_layout(context.theme);
        gutters[0] = gutters[0].max((required.left - base.insets.left).max(0.0).ceil());
    }
    if let Some(chrome) = &chrome {
        let axes = result.layout.projection.panel_box();
        // The current axis adapter snaps anchors to CSS pixels. An integral
        // envelope keeps that snap residue from becoming another gutter request.
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
        round.request_insets(address, address, FeedbackKind::AxisGutter, gutters)?;
    if gutter_request.is_some() && gutters.iter().any(|value| *value > 0.0) {
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
                    let candidate = auto_candidate_avoiding_panel_text(
                        &planning_plot,
                        &result,
                        &legend,
                        preferred,
                        context.theme,
                        &scope,
                    )?;
                    auto_annotation_miss = candidate.is_none();
                    // Keep provisional geometry for the failed preferred attempt;
                    // only a resolving pending assignment can make its miss provisional.
                    let index = candidate.unwrap_or(preferred);
                    auto_candidate_index = Some(index);
                    legend.inside_candidates(result.layout.projection.pixel_box())[index]
                } else {
                    resolve_box(
                        &legend,
                        position,
                        legend_default,
                        result.layout.projection.panel_box(),
                        guide_panel(panel_box, assigned_gutters, position),
                        cell,
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
    if let Some(legend) = &resolved {
        let collision = overlap_score_within_panel(
            legend.bounds,
            geometry.bounds(),
            legend.legend.metrics.font_size.max(6.0),
            result.layout.projection.panel_box(),
        );
        if is_inside(position)
            && collision > 0.0
            && gutter_request.is_none()
            && !auto_annotation_miss
        {
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
                    context.theme,
                    &scope,
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
                let feedback_kind = match axis {
                    ClearanceAxis::X => FeedbackKind::AutomaticDomainX,
                    ClearanceAxis::Y => FeedbackKind::AutomaticDomainY,
                };
                domain_request = round.request_domain(address, address, feedback_kind, limits)?;
                match axis {
                    ClearanceAxis::X => next_x = limits,
                    ClearanceAxis::Y => next_y = limits,
                }
            }
        }
    }
    let mut ink_request = None;
    round.observe_box(panel_box)?;
    round.observe_layout(&result.layout)?;
    if let Some(text) = &axis_text {
        text.observe_feedback(round)?;
    }
    if let Some(chrome) = &chrome {
        let mut overflow = [0.0_f64; 4];
        for bounds in chrome.bounds() {
            round.observe_box(PanelBox::new(
                bounds.left,
                bounds.right,
                bounds.bottom,
                bounds.top,
            ))?;
            overflow[0] = overflow[0].max(-bounds.left);
            overflow[1] = overflow[1].max(bounds.right - f64::from(plot.width()));
            overflow[2] = overflow[2].max(-bounds.bottom);
            overflow[3] = overflow[3].max(bounds.top - f64::from(plot.height()));
        }
        // Measured composition axes are reserved by their parent grid, including
        // overflow from its provisional cell. A child reserve would count it twice.
        if !planning_plot.axis_text_is_measured() && overflow.iter().any(|value| *value > 0.0) {
            let mut reserves = owned_ink;
            for (reserve, extra) in reserves.iter_mut().zip(overflow) {
                // The active axis adapter rounds text anchors to CSS pixels.
                // A subpixel request cannot move that painted anchor.
                *reserve += extra.ceil();
            }
            ink_request =
                round.request_insets(address, address, FeedbackKind::OwnedInk, reserves)?;
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
    let next_frame = match requirements.assign_fit(
        cell,
        next_gutters,
        &FitScope::new(address, round.attempt_ordinal(address)),
    )? {
        crate::layout::fit::FitOutcome::Measured(panel) => panel,
        crate::layout::fit::FitOutcome::Miss(reason) => {
            round.record_failure(reason)?;
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
    let next_projection = crate::layout::Projection::new(next_x, next_y, next_axes.as_tuple());
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
                let bounds = candidates[(preferred + offset) % candidates.len()];
                if legend_fits_panel(bounds, next_axes)
                    && annotations
                        .iter()
                        .all(|plan| plan.avoids_legend_in(next_axes, bounds))
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
                position,
                legend_default,
                next_axes,
                guide_panel(next_frame, next_gutters, position),
                cell,
                geometry.bounds(),
            )
        };
        let allocation_requests = [gutter_request, ink_request];
        if is_outside(position) {
            if let Some(chrome) = &chrome {
                for ink in chrome.bounds() {
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
            let corner = match crate::layout::fit::outcome(
                crate::decorations::panel_text::conflicting_legend_corner(
                    &planning_plot,
                    &result.layout,
                    legend.bounds,
                    context.theme,
                    &scope,
                ),
            )? {
                FitOutcome::Measured(corner) => corner,
                FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
            };
            if let Some(corner) = corner {
                let resolved_by_assignment = (!auto_annotation_miss
                    || resolving_auto_bounds.is_some())
                    && match crate::layout::fit::outcome(
                        crate::decorations::panel_text::conflict_resolved_by_assignment(
                            &planning_plot,
                            &result.layout,
                            next_axes,
                            next_bounds,
                            context.theme,
                            &scope,
                        ),
                    )? {
                        FitOutcome::Measured(resolved) => resolved,
                        FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
                    };
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
                let annotations = match crate::layout::fit::outcome(
                    crate::decorations::panel_text::measure_plans(
                        &planning_plot,
                        &result.layout,
                        context.theme,
                        &scope,
                    ),
                )? {
                    FitOutcome::Measured(plans) => plans,
                    FitOutcome::Miss(failure) => return Ok(FitOutcome::Miss(failure)),
                };
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
                (requirements.natural_insets[0] == 0.0 && requirements.natural_insets[1] == 0.0
                    || legend.legend.compute_box_height() <= panel.height())
                    && (requirements.natural_insets[2] == 0.0
                        && requirements.natural_insets[3] == 0.0
                        || legend.legend.compute_box_width() <= panel.width())
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
    for bounds in geometry.bounds() {
        round.observe_box(PanelBox::new(
            bounds.left,
            bounds.right,
            bounds.bottom,
            bounds.top,
        ))?;
        round.observe_number(bounds.weight)?;
        if bounds.left < 0.0
            || bounds.right > f64::from(plot.width())
            || bounds.bottom < 0.0
            || bounds.top > f64::from(plot.height())
        {
            let declared = result
                .deferred_annotation_text
                .iter()
                .map(|text| &text.plan)
                .chain(
                    result
                        .significance
                        .iter()
                        .flat_map(|plan| plan.brackets.iter().map(|bracket| &bracket.label.plan)),
                );
            let resolved_by_projection = geometry.text_overflow_resolved(
                *bounds,
                PanelBox::new(0.0, f64::from(plot.width()), 0.0, f64::from(plot.height())),
                &result.layout.projection,
                &next_projection,
                declared,
            )?;
            let requests = [gutter_request, ink_request, domain_request];
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
        panel_box,
        result,
        legend: resolved,
        axis_text,
        planned_x_limits,
        planned_y_limits,
    }))
}

fn record_brackets(
    plot: &Plot,
    result: &PlotRenderResult,
    theme: &crate::theme::ResolvedTheme,
    geometry: &mut GeometryBackend,
) -> Result<(), Error> {
    if let Some(significance) = &result.significance {
        crate::significance::render(plot, significance, theme, geometry)?;
    }
    Ok(())
}

fn measure_geometry(
    plot: &Plot,
    kind: PlotKind,
    context: PlotRenderContext<'_>,
) -> Result<crate::layout::fit::FitOutcome<(PlotRenderResult, GeometryBackend)>, Error> {
    use crate::layout::fit::{self, FitOutcome};
    let mut geometry = GeometryBackend::new(context.theme)?;
    geometry.begin(plot.width(), plot.height())?;
    let result = plot
        .measure_panel(kind, context, &mut geometry)
        .and_then(|result| {
            record_brackets(plot, &result, context.theme, &mut geometry)?;
            Ok(result)
        });
    let result = backend::finish_scope(fit::outcome(result), geometry.end(), "geometry discovery")?;
    match result {
        FitOutcome::Measured(result) => Ok(FitOutcome::Measured((result, geometry))),
        FitOutcome::Miss(failure) => Ok(FitOutcome::Miss(failure)),
    }
}

#[cfg(test)]
fn discover_geometry(
    plot: &Plot,
    kind: PlotKind,
    context: PlotRenderContext<'_>,
) -> Result<(PlotRenderResult, GeometryBackend), Error> {
    crate::layout::fit::measured(measure_geometry(plot, kind, context))
        .map_err(crate::layout::fit::PlanError::into_error)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClearanceEdge {
    Left,
    Right,
    Bottom,
    Top,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClearanceAxis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy)]
struct AutomaticClearance {
    candidate_index: usize,
    expansion: Option<DomainExpansion>,
}

#[derive(Debug, Clone, Copy)]
struct DomainExpansion {
    axis: ClearanceAxis,
    limits: (f64, f64),
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

#[expect(
    clippy::too_many_arguments,
    reason = "automatic legend clearance consumes one immutable planned scene and theme"
)]
fn automatic_clearance(
    kind: PlotKind,
    plot: &Plot,
    result: &PlotRenderResult,
    legend: &Legend,
    geometry: &[crate::render::geometry::GeometryBox],
    default: LegendDefault,
    current_candidate: usize,
    theme: &crate::theme::ResolvedTheme,
    scope: &FitScope,
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
            theme,
            scope,
        ),
        LegendDefault::LowestOverlap | LegendDefault::SelfManaged => {
            lowest_overlap_clearance(kind, plot, result, legend, geometry, theme, scope)
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "candidate selection keeps the immutable legend-planning inputs explicit"
)]
fn top_right_clearance(
    kind: PlotKind,
    plot: &Plot,
    result: &PlotRenderResult,
    legend: &Legend,
    geometry: &[crate::render::geometry::GeometryBox],
    clearance: LegendClearance,
    current_candidate: usize,
    theme: &crate::theme::ResolvedTheme,
    scope: &FitScope,
) -> Result<Option<AutomaticClearance>, crate::layout::fit::PlanError> {
    let margin = legend.metrics.font_size.max(6.0);
    let candidates = legend.inside_candidates(result.layout.projection.pixel_box());
    for (candidate_index, bounds) in candidates
        .iter()
        .copied()
        .enumerate()
        .skip(current_candidate)
    {
        if crate::decorations::panel_text::conflicting_legend_corner(
            plot,
            &result.layout,
            bounds,
            theme,
            scope,
        )?
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
    theme: &crate::theme::ResolvedTheme,
    scope: &FitScope,
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
        if !legend_fits_panel(bounds, result.layout.projection.panel_box())
            || crate::decorations::panel_text::conflicting_legend_corner(
                plot,
                &result.layout,
                bounds,
                theme,
                scope,
            )?
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
    plot: &Plot,
    result: &PlotRenderResult,
    legend: &Legend,
    preferred: usize,
    theme: &crate::theme::ResolvedTheme,
    scope: &FitScope,
) -> Result<Option<usize>, crate::layout::fit::PlanError> {
    let panel = result.layout.projection.panel_box();
    let annotations =
        crate::decorations::panel_text::measure_plans(plot, &result.layout, theme, scope)?;
    let candidates = legend.inside_candidates(panel.as_tuple());
    for offset in 0..candidates.len() {
        let index = (preferred + offset) % candidates.len();
        if annotations
            .iter()
            .all(|plan| plan.avoids_legend_in(panel, candidates[index]))
        {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn frame_requirements(legend: Option<&Legend>, position: LegendPosition) -> FrameRequirements {
    let mut natural_insets = [0.0; 4];
    if let Some(legend) = legend {
        let gap = legend.metrics.pad.max(8.0);
        match position {
            LegendPosition::OutsideLeftTop
            | LegendPosition::OutsideLeft
            | LegendPosition::OutsideLeftBottom => {
                natural_insets[0] = legend.compute_box_width() + gap;
            }
            LegendPosition::OutsideRightTop
            | LegendPosition::OutsideRight
            | LegendPosition::OutsideRightBottom => {
                natural_insets[1] = legend.compute_box_width() + gap;
            }
            LegendPosition::OutsideBottomLeft
            | LegendPosition::OutsideBottom
            | LegendPosition::OutsideBottomRight => {
                natural_insets[2] = legend.compute_box_height() + gap;
            }
            LegendPosition::OutsideTopLeft
            | LegendPosition::OutsideTop
            | LegendPosition::OutsideTopRight => {
                natural_insets[3] = legend.compute_box_height() + gap;
            }
            _ => {}
        }
    }
    // The legacy family adapter still owns this hard body floor. Native interior
    // migrations replace it with their traced scientific minimum requirements.
    FrameRequirements {
        minimum_body: (80.0, 60.0),
        natural_insets,
    }
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
    position: LegendPosition,
    default: LegendDefault,
    axes: PanelBox,
    reserved_panel: PanelBox,
    cell: PanelBox,
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
        LegendPosition::OutsideRightTop => outside_right(legend, reserved_panel, cell, 1.0),
        LegendPosition::OutsideRight => outside_right(legend, reserved_panel, cell, 0.5),
        LegendPosition::OutsideRightBottom => outside_right(legend, reserved_panel, cell, 0.0),
        LegendPosition::OutsideLeftTop => outside_left(legend, reserved_panel, cell, 1.0),
        LegendPosition::OutsideLeft => outside_left(legend, reserved_panel, cell, 0.5),
        LegendPosition::OutsideLeftBottom => outside_left(legend, reserved_panel, cell, 0.0),
        LegendPosition::OutsideTopLeft => outside_top(legend, reserved_panel, cell, 0.0),
        LegendPosition::OutsideTop => outside_top(legend, reserved_panel, cell, 0.5),
        LegendPosition::OutsideTopRight => outside_top(legend, reserved_panel, cell, 1.0),
        LegendPosition::OutsideBottomLeft => outside_bottom(legend, reserved_panel, cell, 0.0),
        LegendPosition::OutsideBottom => outside_bottom(legend, reserved_panel, cell, 0.5),
        LegendPosition::OutsideBottomRight => outside_bottom(legend, reserved_panel, cell, 1.0),
    }
}

fn outside_right(legend: &Legend, panel: PanelBox, _cell: PanelBox, align: f64) -> LegendBox {
    let gap = legend.metrics.pad.max(8.0);
    let height = legend.compute_box_height();
    legend.box_at(
        panel.x1 + gap,
        panel.y0 + (panel.height() - height).max(0.0) * align,
    )
}

fn outside_left(legend: &Legend, panel: PanelBox, _cell: PanelBox, align: f64) -> LegendBox {
    let gap = legend.metrics.pad.max(8.0);
    let height = legend.compute_box_height();
    legend.box_at(
        panel.x0 - gap - legend.compute_box_width(),
        panel.y0 + (panel.height() - height).max(0.0) * align,
    )
}

fn outside_top(legend: &Legend, panel: PanelBox, _cell: PanelBox, align: f64) -> LegendBox {
    let gap = legend.metrics.pad.max(8.0);
    legend.box_at(
        panel.x0 + (panel.width() - legend.compute_box_width()).max(0.0) * align,
        panel.y1 + gap,
    )
}

fn outside_bottom(legend: &Legend, panel: PanelBox, _cell: PanelBox, align: f64) -> LegendBox {
    let gap = legend.metrics.pad.max(8.0);
    legend.box_at(
        panel.x0 + (panel.width() - legend.compute_box_width()).max(0.0) * align,
        panel.y0 - gap - legend.compute_box_height(),
    )
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
    crate::layout::plan_exact_root(
        plot.width(),
        plot.height(),
        |_| Ok(()),
        |(), assigned, round| {
            Ok(crate::layout::MeasuredLayout::new(
                assigned,
                crate::layout::fit::measured(plan_panel(
                    plot,
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
        |(), _, plan, _| {
            output = Some(plan);
            Ok(())
        },
    )?;
    Ok(output.expect("converged panel was finalized"))
}

#[cfg(test)]
mod tests;
