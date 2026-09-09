//! Operation-local parent inputs and observations for the exact-root driver.

use super::{ComponentAddress, LayoutScalar, SpatialRounding};
use crate::layout::fit::{FitFailure, FitScope};
pub(crate) use crate::layout::fit::{FitKind, FitReason};
use crate::layout::{Layout, PanelBox, ResolvedTextPlan};
use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum FeedbackKind {
    AxisGutter,
    SharedAlignment,
    OwnedInk,
    AutomaticDomainX,
    AutomaticDomainY,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FeedbackValue {
    Insets([i64; 4]),
    Domain([u64; 2]),
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Clone))]
struct FeedbackEntry {
    target: ComponentAddress,
    kind: FeedbackKind,
    value: FeedbackValue,
}

#[derive(Debug, PartialEq, Eq)]
struct FeedbackRequest {
    source: ComponentAddress,
    entry: FeedbackEntry,
}

/// A handle to a concrete changed request in this round, never a fit boolean.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FeedbackRequestId {
    index: usize,
    round: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct FitMiss {
    failure: FitFailure,
    feedback: Vec<std::sync::Arc<FeedbackRequest>>,
}

#[derive(Debug, PartialEq, Eq)]
enum Observation {
    Address(ComponentAddress),
    Box([i64; 4]),
    Number(u64),
    Count(usize),
    Text(String),
    TextPlan(ResolvedTextPlan),
    TextPlacement {
        position: [i64; 2],
        rotation: u64,
        anchor: crate::render::backend::TextAnchor,
        origin: crate::render::backend::PlannedTextOrigin,
    },
    LegendGlyph {
        point: Option<PointGlyphState>,
        line: Option<(
            crate::render::backend::Rgba,
            u64,
            crate::render::backend::StrokePattern,
        )>,
        fill: Option<FillGlyphState>,
    },
}

#[derive(Debug, PartialEq, Eq)]
struct PointGlyphState {
    color: (u8, u8, u8),
    radius: u32,
    shape: crate::PointShape,
    opacity: u64,
    outline: Option<((u8, u8, u8), u64)>,
}

#[derive(Debug, PartialEq, Eq)]
struct FillGlyphState {
    color: (u8, u8, u8),
    stroke: (u8, u8, u8),
    stroke_width: u64,
    opacity: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct RoundState {
    consumed: Vec<FeedbackEntry>,
    next: Vec<FeedbackEntry>,
    requests: Vec<std::sync::Arc<FeedbackRequest>>,
    observations: Vec<Observation>,
    misses: Vec<FitMiss>,
}

/// A round borrows one operation's budgets. No child can start another round.
pub(crate) struct FeedbackRound {
    state: RoundState,
    resources: crate::font::canonical::OperationGenerationResources,
    ordinal: usize,
    guide_trials: Vec<(ComponentAddress, u8, u32)>,
}

pub(crate) struct TrialCheckpoint {
    observations: usize,
    requests: usize,
    misses: usize,
    next: Vec<FeedbackEntry>,
}

pub(crate) enum TrialOutcome {
    Fit,
    Provisional,
    Miss(FitFailure),
}

fn number(value: f64) -> Result<u64, Error> {
    if !value.is_finite() {
        return Err(Error::Render("layout feedback input must be finite".into()));
    }
    Ok(if value == 0.0 { 0 } else { value.to_bits() })
}

fn reserve<T>(values: &mut Vec<T>, count: usize) -> Result<(), Error> {
    values
        .try_reserve_exact(count)
        .map_err(|error| Error::Render(format!("layout feedback allocation failed: {error}")))
}

fn claim_error(error: impl std::fmt::Debug) -> Error {
    Error::Render(format!("layout feedback admission: {error:?}"))
}

fn copy_address(
    resources: &crate::font::canonical::OperationGenerationResources,
    address: &ComponentAddress,
) -> Result<ComponentAddress, Error> {
    resources
        .claim_slots(address.descendants.len() as u128)
        .map_err(claim_error)?;
    resources
        .claim_work(address.descendants.len() as u128)
        .map_err(claim_error)?;
    let mut descendants = Vec::new();
    reserve(&mut descendants, address.descendants.len())?;
    descendants.extend_from_slice(&address.descendants);
    Ok(ComponentAddress {
        root: address.root,
        descendants: descendants.into_boxed_slice(),
    })
}

impl FeedbackRound {
    pub(crate) fn attempt_ordinal(&self, address: &ComponentAddress) -> u32 {
        self.guide_trials
            .iter()
            .find(|(owner, _, _)| owner == address)
            .map_or(0, |(_, _, ordinal)| *ordinal)
    }
    pub(crate) fn claim_guide_trial(
        &mut self,
        owner: &ComponentAddress,
        maximum: u8,
        ordinal: u32,
    ) -> Result<(), Error> {
        self.resources.claim_work(1).map_err(claim_error)?;
        if let Some((_, used, current)) = self
            .guide_trials
            .iter_mut()
            .find(|(address, _, _)| address == owner)
        {
            if *used >= maximum {
                return Err(Error::Render(format!("guide planning limit of {maximum} edge/flow trials per owner per root round exceeded")));
            }
            *used += 1;
            *current = ordinal;
        } else {
            self.resources.claim_slots(1).map_err(claim_error)?;
            reserve(&mut self.guide_trials, 1)?;
            self.guide_trials
                .push((copy_address(&self.resources, owner)?, 1, ordinal));
        }
        Ok(())
    }

    pub(crate) fn begin_trial(&mut self) -> Result<TrialCheckpoint, Error> {
        let mut next = Vec::new();
        self.resources
            .claim_slots(self.state.next.len() as u128)
            .map_err(claim_error)?;
        reserve(&mut next, self.state.next.len())?;
        for entry in &self.state.next {
            next.push(FeedbackEntry {
                target: copy_address(&self.resources, &entry.target)?,
                kind: entry.kind,
                value: entry.value.clone(),
            });
        }
        Ok(TrialCheckpoint {
            observations: self.state.observations.len(),
            requests: self.state.requests.len(),
            misses: self.state.misses.len(),
            next,
        })
    }

    pub(crate) fn finish_trial(&mut self, checkpoint: TrialCheckpoint) -> TrialOutcome {
        if self.state.misses[checkpoint.misses..]
            .iter()
            .any(|miss| miss.feedback.is_empty())
        {
            let miss = self
                .state
                .misses
                .drain(checkpoint.misses..)
                .filter(|miss| miss.feedback.is_empty())
                .min_by(|a, b| a.failure.reason.cmp(&b.failure.reason))
                .expect("terminal trial miss");
            self.state.observations.truncate(checkpoint.observations);
            self.state.requests.truncate(checkpoint.requests);
            self.state.next = checkpoint.next;
            TrialOutcome::Miss(miss.failure)
        } else if self.state.misses.len() > checkpoint.misses {
            TrialOutcome::Provisional
        } else {
            TrialOutcome::Fit
        }
    }
    #[cfg(test)]
    pub(crate) fn seed_insets_for_test(
        &mut self,
        address: &ComponentAddress,
        values: [f64; 4],
    ) -> Result<(), Error> {
        self.request_insets(address, address, FeedbackKind::AxisGutter, values)?;
        self.state.consumed.clone_from(&self.state.next);
        self.state.requests.clear();
        Ok(())
    }
    pub(super) fn new(
        previous: Option<&RoundState>,
        resources: crate::font::canonical::OperationGenerationResources,
        ordinal: usize,
    ) -> Result<Self, Error> {
        let mut consumed = Vec::new();
        let mut next = Vec::new();
        if let Some(previous) = previous {
            resources
                .claim_slots((previous.next.len() as u128) * 2)
                .map_err(claim_error)?;
            reserve(&mut consumed, previous.next.len())?;
            reserve(&mut next, previous.next.len())?;
            for entry in &previous.next {
                consumed.push(FeedbackEntry {
                    target: copy_address(&resources, &entry.target)?,
                    kind: entry.kind,
                    value: entry.value.clone(),
                });
                next.push(FeedbackEntry {
                    target: copy_address(&resources, &entry.target)?,
                    kind: entry.kind,
                    value: entry.value.clone(),
                });
            }
        }
        Ok(Self {
            state: RoundState {
                consumed,
                next,
                requests: Vec::new(),
                observations: Vec::new(),
                misses: Vec::new(),
            },
            resources,
            ordinal,
            guide_trials: Vec::new(),
        })
    }

    pub(crate) fn begin_owner(&mut self, address: &ComponentAddress) -> Result<(), Error> {
        self.resources.claim_work(1).map_err(claim_error)?;
        self.admit_observation()?;
        let address = copy_address(&self.resources, address)?;
        self.state.observations.push(Observation::Address(address));
        Ok(())
    }

    fn observe(&mut self, observation: Observation) -> Result<(), Error> {
        self.admit_observation()?;
        self.state.observations.push(observation);
        Ok(())
    }

    fn admit_observation(&mut self) -> Result<(), Error> {
        self.resources.claim_slots(1).map_err(claim_error)?;
        self.resources.claim_work(1).map_err(claim_error)?;
        reserve(&mut self.state.observations, 1)?;
        Ok(())
    }

    pub(crate) fn observe_number(&mut self, value: f64) -> Result<(), Error> {
        self.observe(Observation::Number(number(value)?))
    }

    pub(crate) fn observe_count(&mut self, value: usize) -> Result<(), Error> {
        self.observe(Observation::Count(value))
    }

    pub(crate) fn observe_placement(
        &mut self,
        placement: crate::render::backend::PlannedTextPlacement,
    ) -> Result<(), Error> {
        self.observe(Observation::TextPlacement {
            position: [
                LayoutScalar::from_css_pixels(placement.position.x, SpatialRounding::NearestEven)?
                    .0,
                LayoutScalar::from_css_pixels(placement.position.y, SpatialRounding::NearestEven)?
                    .0,
            ],
            rotation: number(placement.rotation)?,
            anchor: placement.anchor,
            origin: placement.origin,
        })
    }

    pub(crate) fn observe_legend_glyph(
        &mut self,
        point: Option<&crate::render::backend::PointStyle>,
        line: Option<&crate::render::backend::LineStyle>,
        fill: Option<&crate::render::backend::FillStyle>,
    ) -> Result<(), Error> {
        let point = point
            .map(|style| {
                crate::render::backend::validate_point_style(style)?;
                Ok::<_, Error>(PointGlyphState {
                    color: style.color,
                    radius: style.radius,
                    shape: style.shape,
                    opacity: number(style.opacity)?,
                    outline: style
                        .outline
                        .map(|outline| {
                            Ok::<_, Error>((outline.color, number(outline.width_ratio)?))
                        })
                        .transpose()?,
                })
            })
            .transpose()?;
        let line = line
            .map(|style| {
                crate::render::backend::validate_line_style(style)?;
                Ok::<_, Error>((style.color, number(f64::from(style.width))?, style.pattern))
            })
            .transpose()?;
        let fill = fill
            .map(|style| {
                crate::render::backend::validate_fill_style(style)?;
                Ok::<_, Error>(FillGlyphState {
                    color: style.color,
                    stroke: style.stroke,
                    stroke_width: number(f64::from(style.stroke_width))?,
                    opacity: number(style.opacity)?,
                })
            })
            .transpose()?;
        self.observe(Observation::LegendGlyph { point, line, fill })
    }

    pub(crate) fn observe_box(&mut self, panel: PanelBox) -> Result<(), Error> {
        if panel.x1 < panel.x0 || panel.y1 < panel.y0 {
            return Err(Error::Render(
                "layout feedback box has reversed edges".into(),
            ));
        }
        let edges = [
            LayoutScalar::from_css_pixels(panel.x0, SpatialRounding::Floor)?.0,
            LayoutScalar::from_css_pixels(panel.x1, SpatialRounding::Ceil)?.0,
            LayoutScalar::from_css_pixels(panel.y0, SpatialRounding::Floor)?.0,
            LayoutScalar::from_css_pixels(panel.y1, SpatialRounding::Ceil)?.0,
        ];
        self.observe(Observation::Box(edges))
    }

    pub(crate) fn observe_text(&mut self, plan: &ResolvedTextPlan) -> Result<(), Error> {
        // The producer admitted the immutable payload. Retaining another handle
        // admits only this observation, before incrementing the shared ownership.
        self.admit_observation()?;
        self.state
            .observations
            .push(Observation::TextPlan(plan.clone()));
        Ok(())
    }

    pub(crate) fn observe_text_block(
        &mut self,
        block: &crate::layout::TextBlock,
    ) -> Result<(), Error> {
        self.observe_text(block.plan())
    }

    pub(crate) fn observe_string(&mut self, value: &str) -> Result<(), Error> {
        self.resources
            .claim_work(value.len() as u128)
            .map_err(claim_error)?;
        self.admit_observation()?;
        let mut text = String::new();
        text.try_reserve_exact(value.len())
            .map_err(|error| Error::Render(format!("feedback string allocation: {error}")))?;
        text.push_str(value);
        self.state.observations.push(Observation::Text(text));
        Ok(())
    }

    pub(crate) fn observe_canvas(
        &mut self,
        canvas: &crate::layout::CanvasLayout,
    ) -> Result<(), Error> {
        self.observe_box(canvas.panel_box)?;
        for inset in [
            canvas.insets.left,
            canvas.insets.right,
            canvas.insets.bottom,
            canvas.insets.top,
        ] {
            self.observe_number(inset)?;
        }
        for baseline in [canvas.title_y, canvas.subtitle_y, canvas.caption_y] {
            self.observe_count(usize::from(baseline.is_some()))?;
            if let Some(baseline) = baseline {
                self.observe_number(baseline)?;
            }
        }
        for (plan, baselines) in [
            (&canvas.title_plan, &canvas.title_lines_y),
            (&canvas.subtitle_plan, &canvas.subtitle_lines_y),
            (&canvas.caption_plan, &canvas.caption_lines_y),
        ] {
            self.observe_count(usize::from(plan.is_some()))?;
            if let Some(plan) = plan {
                self.observe_text(plan)?;
            }
            self.observe_count(baselines.len())?;
            for baseline in baselines {
                self.observe_number(*baseline)?;
            }
        }
        Ok(())
    }

    pub(crate) fn observe_layout(&mut self, layout: &Layout) -> Result<(), Error> {
        self.observe_box(layout.projection.panel_box())?;
        for range in [layout.projection.x_range(), layout.projection.y_range()] {
            self.observe_number(range.0)?;
            self.observe_number(range.1)?;
        }
        for axis in [&layout.x_axis, &layout.y_axis] {
            self.observe_count(usize::from(axis.explicit_ticks))?;
            self.observe_count(usize::from(axis.assigned_tick_width.is_some()))?;
            if let Some(width) = axis.assigned_tick_width {
                self.observe_number(width)?;
            }
            self.observe_string(&axis.label)?;
            self.observe_count(axis.ticks.len())?;
            for tick in &axis.ticks {
                self.observe_number(tick.position)?;
                self.observe_string(&tick.label)?;
            }
        }
        Ok(())
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "legacy pixel adapters consume exact-root-bounded layout quanta"
    )]
    pub(crate) fn insets(&self, target: &ComponentAddress, kind: FeedbackKind) -> [f64; 4] {
        self.state
            .consumed
            .iter()
            .find_map(|entry| {
                if entry.target == *target && entry.kind == kind {
                    if let FeedbackValue::Insets(values) = entry.value {
                        return Some(values.map(|value| value as f64 / 1024.0));
                    }
                }
                None
            })
            .unwrap_or([0.0; 4])
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "legacy pixel adapters consume exact-root-bounded layout quanta"
    )]
    pub(crate) fn next_insets(&self, target: &ComponentAddress, kind: FeedbackKind) -> [f64; 4] {
        self.state
            .next
            .iter()
            .find_map(|entry| {
                if entry.target == *target && entry.kind == kind {
                    if let FeedbackValue::Insets(values) = entry.value {
                        return Some(values.map(|value| value as f64 / 1024.0));
                    }
                }
                None
            })
            .unwrap_or([0.0; 4])
    }

    pub(crate) fn request_insets(
        &mut self,
        source: &ComponentAddress,
        target: &ComponentAddress,
        kind: FeedbackKind,
        values: [f64; 4],
    ) -> Result<Option<FeedbackRequestId>, Error> {
        if !matches!(
            kind,
            FeedbackKind::AxisGutter | FeedbackKind::SharedAlignment | FeedbackKind::OwnedInk
        ) {
            return Err(Error::Render(
                "domain feedback cannot contain insets".into(),
            ));
        }
        let mut normalized = [0; 4];
        for (output, value) in normalized.iter_mut().zip(values) {
            if value < 0.0 {
                return Err(Error::Render(
                    "layout feedback reserve must be nonnegative".into(),
                ));
            }
            *output = LayoutScalar::from_css_pixels(value, SpatialRounding::Ceil)?.0;
        }
        if kind != FeedbackKind::AxisGutter {
            if let Some(FeedbackEntry {
                value: FeedbackValue::Insets(prior),
                ..
            }) = self
                .state
                .next
                .iter()
                .find(|entry| entry.target == *target && entry.kind == kind)
            {
                for (value, prior) in normalized.iter_mut().zip(prior) {
                    *value = (*value).max(*prior);
                }
            }
        }
        self.request(source, target, kind, FeedbackValue::Insets(normalized))
    }

    pub(crate) fn domain(
        &self,
        target: &ComponentAddress,
        kind: FeedbackKind,
    ) -> Option<(f64, f64)> {
        self.state.consumed.iter().find_map(|entry| {
            if entry.target == *target && entry.kind == kind {
                if let FeedbackValue::Domain(values) = entry.value {
                    return Some((f64::from_bits(values[0]), f64::from_bits(values[1])));
                }
            }
            None
        })
    }

    pub(crate) fn request_domain(
        &mut self,
        source: &ComponentAddress,
        target: &ComponentAddress,
        kind: FeedbackKind,
        mut range: (f64, f64),
    ) -> Result<Option<FeedbackRequestId>, Error> {
        number(range.0)?;
        number(range.1)?;
        if !matches!(
            kind,
            FeedbackKind::AutomaticDomainX | FeedbackKind::AutomaticDomainY
        ) || range.0 >= range.1
        {
            return Err(Error::Render("invalid automatic domain feedback".into()));
        }
        if let Some(entry) = self
            .state
            .next
            .iter()
            .find(|entry| entry.target == *target && entry.kind == kind)
        {
            if let FeedbackValue::Domain(prior) = entry.value {
                range.0 = range.0.min(f64::from_bits(prior[0]));
                range.1 = range.1.max(f64::from_bits(prior[1]));
            }
        }
        self.request(
            source,
            target,
            kind,
            FeedbackValue::Domain([number(range.0)?, number(range.1)?]),
        )
    }

    fn request(
        &mut self,
        source: &ComponentAddress,
        target: &ComponentAddress,
        kind: FeedbackKind,
        value: FeedbackValue,
    ) -> Result<Option<FeedbackRequestId>, Error> {
        self.begin_owner(source)?;
        if kind == FeedbackKind::AxisGutter
            && self.state.requests.iter().any(|request| {
                request.entry.target == *target
                    && request.entry.kind == kind
                    && request.entry.value != value
            })
        {
            return Err(Error::Render(
                "conflicting measured gutter identities in one root round".into(),
            ));
        }
        self.resources.claim_slots(1).map_err(claim_error)?;
        reserve(&mut self.state.requests, 1)?;
        let id = FeedbackRequestId {
            index: self.state.requests.len(),
            round: self.ordinal,
        };
        self.state
            .requests
            .push(std::sync::Arc::new(FeedbackRequest {
                source: copy_address(&self.resources, source)?,
                entry: FeedbackEntry {
                    target: copy_address(&self.resources, target)?,
                    kind,
                    value: value.clone(),
                },
            }));
        if value == FeedbackValue::Insets([0; 4])
            && !self
                .state
                .next
                .iter()
                .any(|entry| entry.target == *target && entry.kind == kind)
        {
            return Ok(None);
        }
        let changed = self
            .state
            .consumed
            .iter()
            .find(|entry| entry.target == *target && entry.kind == kind)
            .is_none_or(|entry| entry.value != value);
        if let Some(entry) = self
            .state
            .next
            .iter_mut()
            .find(|entry| entry.target == *target && entry.kind == kind)
        {
            entry.value = value;
        } else {
            self.resources.claim_slots(1).map_err(claim_error)?;
            reserve(&mut self.state.next, 1)?;
            self.state.next.push(FeedbackEntry {
                target: copy_address(&self.resources, target)?,
                kind,
                value,
            });
        }
        Ok(changed.then_some(id))
    }

    pub(crate) fn miss(
        &mut self,
        address: &ComponentAddress,
        kind: FitKind,
        requests: &[Option<FeedbackRequestId>],
        message: impl Into<String>,
    ) -> Result<(), Error> {
        self.resources
            .claim_slots(1 + requests.len() as u128)
            .map_err(claim_error)?;
        self.resources
            .claim_work(requests.len() as u128)
            .map_err(claim_error)?;
        reserve(&mut self.state.misses, 1)?;
        let mut feedback = Vec::new();
        reserve(&mut feedback, requests.len())?;
        for id in requests.iter().flatten() {
            let request = self
                .state
                .requests
                .get(id.index)
                .filter(|_| id.round == self.ordinal)
                .ok_or_else(|| {
                    Error::Render("fit miss references a request from another round".into())
                })?;
            if request.source != *address && request.entry.target != *address {
                return Err(Error::Render(
                    "fit miss references an unrelated feedback owner".into(),
                ));
            }
            feedback.push(std::sync::Arc::clone(request));
        }
        let ordinal = self
            .guide_trials
            .iter()
            .find(|(owner, _, _)| owner == address)
            .map_or(0, |(_, _, ordinal)| *ordinal);
        self.state.misses.push(FitMiss {
            failure: FitFailure {
                reason: FitReason {
                    kind,
                    address: copy_address(&self.resources, address)?,
                    ordinal,
                },
                message: message.into(),
            },
            feedback,
        });
        Ok(())
    }

    pub(crate) fn record_failure(&mut self, failure: FitFailure) -> Result<(), Error> {
        self.resources.claim_slots(1).map_err(claim_error)?;
        reserve(&mut self.state.misses, 1)?;
        self.state.misses.push(FitMiss {
            failure,
            feedback: Vec::new(),
        });
        Ok(())
    }

    pub(super) fn finish(mut self) -> Result<RoundState, Error> {
        self.resources
            .claim_work((self.state.next.len() + self.state.misses.len()) as u128)
            .map_err(claim_error)?;
        self.state
            .next
            .sort_by(|a, b| (&a.target, a.kind).cmp(&(&b.target, b.kind)));
        self.state.misses.sort_by(|a, b| {
            (&a.failure.reason, !a.feedback.is_empty())
                .cmp(&(&b.failure.reason, !b.feedback.is_empty()))
        });
        self.state
            .misses
            .dedup_by(|a, b| a.failure.reason == b.failure.reason);
        Ok(self.state)
    }
}

impl RoundState {
    pub(super) fn terminal_error(&self) -> Option<Error> {
        self.state_error(false)
    }

    pub(super) fn unchanged_error(&self) -> Option<Error> {
        self.state_error(true)
    }

    fn state_error(&self, include_provisional: bool) -> Option<Error> {
        self.misses
            .iter()
            .find(|miss| include_provisional || miss.feedback.is_empty())
            .map(|miss| Error::Render(miss.failure.to_string()))
    }
}

/// Intrinsic, unassigned frame requirements. The parent owns the residual body;
/// an outside guide contributes its natural nonfill extent, not a child choice.
pub(crate) struct FrameRequirements {
    pub(crate) minimum_body: (f64, f64),
    pub(crate) natural_insets: [f64; 4],
}

impl FrameRequirements {
    pub(crate) fn assign(&self, cell: PanelBox, gutters: [f64; 4]) -> Result<PanelBox, Error> {
        match self.assign_fit(cell, gutters, &FitScope::root())? {
            crate::layout::fit::FitOutcome::Measured(panel) => Ok(panel),
            crate::layout::fit::FitOutcome::Miss(reason) => Err(reason.into_error()),
        }
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "assigned frame coordinates remain bounded by exact public u32 root dimensions"
    )]
    pub(crate) fn assign_fit(
        &self,
        cell: PanelBox,
        gutters: [f64; 4],
        scope: &FitScope,
    ) -> Result<crate::layout::fit::FitOutcome<PanelBox>, Error> {
        let checked = |value: i128| {
            i64::try_from(value)
                .map_err(|_| Error::Render("assigned frame arithmetic overflowed".into()))
        };
        let mut edges = [0_i64; 4];
        for ((edge, natural), gutter) in edges.iter_mut().zip(self.natural_insets).zip(gutters) {
            if natural < 0.0 || gutter < 0.0 {
                return Err(Error::Render("invalid assigned frame inset".into()));
            }
            *edge = checked(
                i128::from(LayoutScalar::from_css_pixels(natural, SpatialRounding::Ceil)?.0)
                    + i128::from(LayoutScalar::from_css_pixels(gutter, SpatialRounding::Ceil)?.0),
            )?;
        }
        let mut assigned = [0_i64; 4];
        for ((output, coordinate), inset) in assigned
            .iter_mut()
            .zip([cell.x0, cell.x1, cell.y0, cell.y1])
            .zip([edges[0], -edges[1], edges[2], -edges[3]])
        {
            *output = checked(
                i128::from(
                    LayoutScalar::from_css_pixels(coordinate, SpatialRounding::NearestEven)?.0,
                ) + i128::from(inset),
            )?;
        }
        let width = checked(i128::from(assigned[1]) - i128::from(assigned[0]))?;
        let height = checked(i128::from(assigned[3]) - i128::from(assigned[2]))?;
        let minimum_width =
            LayoutScalar::from_css_pixels(self.minimum_body.0, SpatialRounding::Ceil)?.0;
        let minimum_height =
            LayoutScalar::from_css_pixels(self.minimum_body.1, SpatialRounding::Ceil)?.0;
        if minimum_width <= 0 || minimum_height <= 0 {
            return Err(Error::Render("frame minima must be positive".into()));
        }
        if width < minimum_width || height < minimum_height {
            return Ok(crate::layout::fit::FitOutcome::Miss(scope.failure(
                FitKind::TrackOrResidual,
                "assigned frame leaves insufficient body after measured gutters",
            )));
        }
        let [left, right, bottom, top] = assigned.map(|value| value as f64 / 1024.0);
        Ok(crate::layout::fit::FitOutcome::Measured(PanelBox::new(
            left, right, bottom, top,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_plan() -> ResolvedTextPlan {
        crate::font::canonical::PreparedFontEnvironment::synthetic()
            .plan_text(crate::layout::TextPlanInput {
                source: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                stack: "Test Sans",
                weight: crate::TextWeight::Normal,
                font_size: 10,
                line_height: crate::layout::TextLineHeight::Multiplier(1.2),
                maximum_width_quanta: i64::MAX,
                alignment: crate::layout::PhysicalTextAlignment::Start,
                paint: crate::render::backend::Rgba::rgb(0, 0, 0),
                provenance: crate::layout::TextPlanProvenance {
                    pair_ordinal: 0,
                    owner: "feedback test".into(),
                },
                exact_override: false,
            })
            .unwrap()
    }

    #[test]
    fn rejected_ancestor_attempts_do_not_reset_child_guide_budget() {
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let owner = ComponentAddress::root();
        let mut round = FeedbackRound::new(None, resources, 0).unwrap();
        for ordinal in 0..8 {
            let checkpoint = round.begin_trial().unwrap();
            round.claim_guide_trial(&owner, 8, ordinal).unwrap();
            round
                .miss(
                    &owner,
                    FitKind::TrackOrResidual,
                    &[],
                    "containing frame failed",
                )
                .unwrap();
            assert!(matches!(
                round.finish_trial(checkpoint),
                TrialOutcome::Miss(_)
            ));
        }
        assert!(round.state.misses.is_empty());
        assert!(round
            .claim_guide_trial(&owner, 8, 8)
            .unwrap_err()
            .to_string()
            .contains("planning limit"));
    }

    #[test]
    fn text_observations_share_admitted_payload_and_refuse_before_retention() {
        let plan = text_plan();
        let block = crate::layout::TextBlock::from_resolved(plan.clone());
        for as_block in [false, true] {
            let resources = crate::font::canonical::OperationGenerationResources::new();
            resources.claim_slots(999_999).unwrap();
            let mut round = FeedbackRound::new(None, resources, 0).unwrap();
            if as_block {
                round.observe_text_block(&block).unwrap();
            } else {
                round.observe_text(&plan).unwrap();
            }
            let [Observation::TextPlan(retained)] = round.state.observations.as_slice() else {
                panic!("one shared text observation")
            };
            assert!(retained.shares_payload(&plan));
            assert!(round.observe_text(&plan).is_err());
            assert!(round.observe_text_block(&block).is_err());
            assert_eq!(round.state.observations.len(), 1);
        }
    }

    #[test]
    fn provisional_misses_retain_concrete_requests_and_reject_stale_handles() {
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let address = ComponentAddress::root();
        let mut round = FeedbackRound::new(None, resources.clone(), 0).unwrap();
        let request = round
            .request_insets(
                &address,
                &address,
                FeedbackKind::AxisGutter,
                [1.0, 0.0, 0.0, 0.0],
            )
            .unwrap();
        round
            .miss(&address, FitKind::Tick, &[request], "pending gutter")
            .unwrap();
        assert!(std::sync::Arc::ptr_eq(
            &round.state.misses[0].feedback[0],
            &round.state.requests[0]
        ));
        let state = round.finish().unwrap();
        let mut next = FeedbackRound::new(Some(&state), resources, 1).unwrap();
        assert!(next
            .miss(&address, FitKind::Tick, &[request], "stale")
            .unwrap_err()
            .to_string()
            .contains("another round"));
        let unchanged = next
            .request_insets(
                &address,
                &address,
                FeedbackKind::AxisGutter,
                [1.0, 0.0, 0.0, 0.0],
            )
            .unwrap();
        assert!(unchanged.is_none());
        next.miss(&address, FitKind::Tick, &[unchanged], "no changed input")
            .unwrap();
        assert!(next
            .finish()
            .unwrap()
            .terminal_error()
            .unwrap()
            .to_string()
            .contains("no changed input"));
    }
}
