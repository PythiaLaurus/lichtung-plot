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
    observations: Vec<std::sync::Arc<Observation>>,
    misses: Vec<FitMiss>,
}

/// A round borrows one operation's budgets. No child can start another round.
pub(crate) struct FeedbackRound {
    state: RoundState,
    resources: crate::font::canonical::OperationGenerationResources,
    ordinal: usize,
    guide_trials: Vec<(ComponentAddress, u8, u32)>,
    guide_events: Vec<(ComponentAddress, u8, u32)>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ChildFeedbackInputs(Vec<FeedbackEntry>);

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ChildFeedback {
    observations: Vec<std::sync::Arc<Observation>>,
    requests: Vec<std::sync::Arc<FeedbackRequest>>,
    guide_events: Vec<(ComponentAddress, u8, u32)>,
}

#[derive(Clone, Copy)]
pub(crate) struct ChildCheckpoint(usize, usize, usize, usize);

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

fn reserve<T>(
    resources: &crate::font::canonical::OperationGenerationResources,
    values: &mut Vec<T>,
    count: usize,
) -> Result<(), Error> {
    if count > values.capacity() - values.len() {
        let bytes = values
            .len()
            .checked_add(count)
            .and_then(|capacity| capacity.checked_mul(std::mem::size_of::<T>()))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| Error::Render("feedback storage overflow".into()))?;
        resources.claim_bytes(bytes).map_err(claim_error)?;
    }
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
    let mut descendants = Vec::new();
    reserve(resources, &mut descendants, address.descendants.len())?;
    descendants.extend_from_slice(&address.descendants);
    Ok(ComponentAddress {
        root: address.root,
        descendants: descendants.into_boxed_slice(),
    })
}

impl FeedbackRound {
    pub(crate) fn child_inputs(
        &self,
        owners: &[ComponentAddress],
    ) -> Result<ChildFeedbackInputs, Error> {
        let mut entries = Vec::new();
        for entry in &self.state.consumed {
            if owners.iter().any(|owner| entry.target.is_within(owner))
                && entry.value != FeedbackValue::Insets([0; 4])
            {
                self.resources.claim_slots(1).map_err(claim_error)?;
                reserve(&self.resources, &mut entries, 1)?;
                entries.push(FeedbackEntry {
                    target: copy_address(&self.resources, &entry.target)?,
                    kind: entry.kind,
                    value: entry.value.clone(),
                });
            }
        }
        Ok(ChildFeedbackInputs(entries))
    }

    pub(crate) fn child_stable(&self, owners: &[ComponentAddress]) -> bool {
        let relevant = |entry: &&FeedbackEntry| {
            owners.iter().any(|owner| entry.target.is_within(owner))
                && entry.value != FeedbackValue::Insets([0; 4])
        };
        let consumed = self.state.consumed.iter().filter(relevant);
        let next = self.state.next.iter().filter(relevant);
        // next is not sorted until the round finishes.
        consumed.clone().count() == next.clone().count()
            && consumed
                .clone()
                .all(|entry| next.clone().any(|other| entry == other))
            && !self.state.misses.iter().any(|miss| {
                owners
                    .iter()
                    .any(|owner| miss.failure.reason.address.is_within(owner))
            })
    }

    pub(crate) fn child_inputs_equal(
        &self,
        inputs: &ChildFeedbackInputs,
        owners: &[ComponentAddress],
    ) -> bool {
        self.state
            .consumed
            .iter()
            .filter(|entry| {
                owners.iter().any(|owner| entry.target.is_within(owner))
                    && entry.value != FeedbackValue::Insets([0; 4])
            })
            .eq(&inputs.0)
    }

    pub(crate) fn child_checkpoint(&self) -> ChildCheckpoint {
        ChildCheckpoint(
            self.state.observations.len(),
            self.state.requests.len(),
            self.state.misses.len(),
            self.guide_events.len(),
        )
    }

    pub(crate) fn retain_child_feedback(
        &self,
        start: ChildCheckpoint,
    ) -> Result<Option<ChildFeedback>, Error> {
        if self.state.misses.len() != start.2
            || self.state.requests[start.1..].iter().any(|request| {
                self.state
                    .consumed
                    .iter()
                    .find(|entry| {
                        entry.target == request.entry.target && entry.kind == request.entry.kind
                    })
                    .map_or(
                        request.entry.value != FeedbackValue::Insets([0; 4]),
                        |entry| entry.value != request.entry.value,
                    )
            })
        {
            return Ok(None);
        }
        let mut observations = Vec::new();
        let mut requests = Vec::new();
        let mut guide_events = Vec::new();
        self.resources
            .claim_slots(
                (self.state.observations.len() - start.0 + self.state.requests.len() - start.1
                    + self.guide_events.len()
                    - start.3) as u128,
            )
            .map_err(claim_error)?;
        reserve(
            &self.resources,
            &mut observations,
            self.state.observations.len() - start.0,
        )?;
        reserve(
            &self.resources,
            &mut requests,
            self.state.requests.len() - start.1,
        )?;
        reserve(
            &self.resources,
            &mut guide_events,
            self.guide_events.len() - start.3,
        )?;
        observations.extend(self.state.observations[start.0..].iter().cloned());
        requests.extend(self.state.requests[start.1..].iter().cloned());
        for (owner, maximum, ordinal) in &self.guide_events[start.3..] {
            guide_events.push((copy_address(&self.resources, owner)?, *maximum, *ordinal));
        }
        Ok(Some(ChildFeedback {
            observations,
            requests,
            guide_events,
        }))
    }

    pub(crate) fn replay_child(&mut self, child: &ChildFeedback) -> Result<(), Error> {
        for (owner, maximum, ordinal) in &child.guide_events {
            self.claim_guide_trial(owner, *maximum, *ordinal)?;
        }
        for observation in &child.observations {
            self.admit_observation()?;
            self.state
                .observations
                .push(std::sync::Arc::clone(observation));
        }
        for request in &child.requests {
            // request() also observes its source; that observation is already in
            // the retained stream. Recreate only the current-round request here.
            let count = self.state.observations.len();
            self.request(
                &request.source,
                &request.entry.target,
                request.entry.kind,
                request.entry.value.clone(),
            )?;
            self.state.observations.truncate(count);
        }
        Ok(())
    }

    pub(crate) fn attempt_ordinal(&self, address: &ComponentAddress) -> Result<u32, Error> {
        for (owner, _, ordinal) in &self.guide_trials {
            self.resources.claim_visits(1).map_err(claim_error)?;
            if owner == address {
                return Ok(*ordinal);
            }
        }
        Ok(0)
    }
    pub(crate) fn claim_guide_trial(
        &mut self,
        owner: &ComponentAddress,
        maximum: u8,
        ordinal: u32,
    ) -> Result<(), Error> {
        let mut found = None;
        for entry in &mut self.guide_trials {
            self.resources.claim_visits(1).map_err(claim_error)?;
            if entry.0 == *owner {
                found = Some(entry);
                break;
            }
        }
        if let Some((_, used, current)) = found {
            if *used >= maximum {
                return Err(Error::Render(format!("guide planning limit of {maximum} edge/flow trials per owner per root round exceeded")));
            }
            *used += 1;
            *current = ordinal;
        } else {
            self.resources.claim_slots(1).map_err(claim_error)?;
            reserve(&self.resources, &mut self.guide_trials, 1)?;
            self.guide_trials
                .push((copy_address(&self.resources, owner)?, 1, ordinal));
        }
        self.resources.claim_slots(1).map_err(claim_error)?;
        reserve(&self.resources, &mut self.guide_events, 1)?;
        self.guide_events
            .push((copy_address(&self.resources, owner)?, maximum, ordinal));
        Ok(())
    }

    pub(crate) fn begin_trial(&mut self) -> Result<TrialCheckpoint, Error> {
        let mut next = Vec::new();
        self.resources
            .claim_slots(self.state.next.len() as u128)
            .map_err(claim_error)?;
        reserve(&self.resources, &mut next, self.state.next.len())?;
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
        self.state.consumed = self
            .state
            .next
            .iter()
            .map(|entry| FeedbackEntry {
                target: entry.target.clone(),
                kind: entry.kind,
                value: entry.value.clone(),
            })
            .collect();
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
            reserve(&resources, &mut consumed, previous.next.len())?;
            reserve(&resources, &mut next, previous.next.len())?;
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
            guide_events: Vec::new(),
        })
    }

    pub(crate) fn begin_owner(&mut self, address: &ComponentAddress) -> Result<(), Error> {
        self.admit_observation()?;
        let address = copy_address(&self.resources, address)?;
        self.state
            .observations
            .push(std::sync::Arc::new(Observation::Address(address)));
        Ok(())
    }

    fn observe(&mut self, observation: Observation) -> Result<(), Error> {
        self.admit_observation()?;
        self.state
            .observations
            .push(std::sync::Arc::new(observation));
        Ok(())
    }

    fn admit_observation(&mut self) -> Result<(), Error> {
        self.resources.claim_slots(1).map_err(claim_error)?;
        self.resources
            .claim_bytes(std::mem::size_of::<Observation>() as u64)
            .map_err(claim_error)?;
        reserve(&self.resources, &mut self.state.observations, 1)?;
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
            .push(std::sync::Arc::new(Observation::TextPlan(plan.clone())));
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
            .claim_bytes(u64::try_from(value.len()).map_err(claim_error)?)
            .map_err(claim_error)?;
        self.admit_observation()?;
        let mut text = String::new();
        text.try_reserve_exact(value.len())
            .map_err(|error| Error::Render(format!("feedback string allocation: {error}")))?;
        text.push_str(value);
        self.state
            .observations
            .push(std::sync::Arc::new(Observation::Text(text)));
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
        match &canvas.root_text {
            crate::layout::canvas::CanvasRootText::Planned(plan) => {
                self.observe_count(0)?;
                return plan.observe(self);
            }
            crate::layout::canvas::CanvasRootText::Unmeasured => self.observe_count(1)?,
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
        let request = self.request(source, target, kind, FeedbackValue::Insets(normalized))?;
        #[cfg(test)]
        if kind == FeedbackKind::AxisGutter && request.is_some() {
            crate::orchestrator::measurements::gutter_change(
                target,
                self.insets(target, kind),
                self.next_insets(target, kind),
            );
        }
        Ok(request)
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
        reserve(&self.resources, &mut self.state.requests, 1)?;
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
            reserve(&self.resources, &mut self.state.next, 1)?;
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
            .claim_visits(requests.len() as u128)
            .map_err(claim_error)?;
        reserve(&self.resources, &mut self.state.misses, 1)?;
        let mut feedback = Vec::new();
        reserve(&self.resources, &mut feedback, requests.len())?;
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
        let ordinal = self.attempt_ordinal(address)?;
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
        reserve(&self.resources, &mut self.state.misses, 1)?;
        self.state.misses.push(FitMiss {
            failure,
            feedback: Vec::new(),
        });
        Ok(())
    }

    pub(super) fn finish(mut self) -> Result<RoundState, Error> {
        self.resources
            .claim_visits((self.state.next.len() + self.state.misses.len()) as u128)
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
    pub(super) fn feedback_unchanged(&self) -> bool {
        // Insets default to zero when absent; domains default to None. Both
        // vectors are in canonical target/kind order after finishing a round.
        let effective = |entry: &&FeedbackEntry| entry.value != FeedbackValue::Insets([0; 4]);
        self.consumed
            .iter()
            .filter(effective)
            .eq(self.next.iter().filter(effective))
    }

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

    #[test]
    fn interior_consequences_keep_source_target_quantization_and_operation_budget() {
        use crate::layout::interior::{ClearanceAxis, DomainExpansion, InteriorReserveRequest};
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let mut round = FeedbackRound::new(None, resources.clone(), 0).unwrap();
        let target = ComponentAddress::root();
        let source = target
            .child(super::super::AddressRole::MarkLayer, 0)
            .unwrap();
        let before = resources.byte_usage().unwrap();
        let request = InteriorReserveRequest::OwnedInk([1.0 / 2048.0, 0.0, 0.0, 0.0])
            .submit(&mut round, &source, &target)
            .unwrap();
        assert!(request.is_some());
        assert_eq!(round.state.requests[0].source, source);
        assert_eq!(round.state.requests[0].entry.target, target);
        assert_eq!(
            round.state.requests[0].entry.value,
            FeedbackValue::Insets([1, 0, 0, 0])
        );
        assert!(resources.byte_usage().unwrap() > before);
        DomainExpansion {
            axis: ClearanceAxis::X,
            limits: (-1.0, 2.0),
        }
        .submit(&mut round, &source, &target)
        .unwrap();
        assert_eq!(
            round.state.requests[1].entry.kind,
            FeedbackKind::AutomaticDomainX
        );
        assert!(InteriorReserveRequest::OwnedInk([-1.0; 4])
            .submit(&mut round, &source, &target)
            .is_err());
        assert!(DomainExpansion {
            axis: ClearanceAxis::Y,
            limits: (2.0, 1.0)
        }
        .submit(&mut round, &source, &target)
        .is_err());
        let checkpoint = round.begin_trial().unwrap();
        InteriorReserveRequest::AxisGutter([2.0; 4])
            .submit(&mut round, &source, &target)
            .unwrap();
        round
            .miss(
                &source,
                FitKind::TrackOrResidual,
                &[],
                "rejected interior trial",
            )
            .unwrap();
        let charged = resources.byte_usage().unwrap();
        assert!(matches!(
            round.finish_trial(checkpoint),
            TrialOutcome::Miss(_)
        ));
        assert_eq!(resources.byte_usage().unwrap(), charged);
        resources.claim_bytes(536_870_912 - charged).unwrap();
        assert!(InteriorReserveRequest::OwnedInk([3.0; 4])
            .submit(&mut round, &source, &target)
            .is_err());
    }

    #[test]
    fn axis_semantics_and_standard_typography_are_feedback_identity() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .erase();
        let domains = [crate::spec::AxisDomain::Continuous; 2];
        let theme = crate::Theme::Publication.resolve();
        let standard = crate::layout::StandardPanelInputs::new(&theme).unwrap();
        let state = |plot: &crate::Plot, standard: &crate::layout::StandardPanelInputs| {
            let resources = crate::font::canonical::OperationGenerationResources::new();
            let mut round = FeedbackRound::new(None, resources, 0).unwrap();
            for input in plot.axis_semantic_inputs(domains) {
                input.observe_feedback(&mut round).unwrap();
            }
            standard.observe_feedback(&mut round).unwrap();
            round.finish().unwrap()
        };
        let automatic = state(&plot, &standard);
        let shared = plot.clone().with_resolved_facet_ranges(
            crate::spec::PlotKind::Scatter,
            Some((0.0, 1.0)),
            None,
        );
        assert_ne!(automatic, state(&shared, &standard));
        assert_eq!(
            automatic,
            state(&plot.with_planned_x_limits((0.0, 1.0)), &standard)
        );
        let mut changed_theme = theme.clone();
        changed_theme.typography.tick_label.line_height_multiplier += 0.25;
        let changed = crate::layout::StandardPanelInputs::new(&changed_theme).unwrap();
        assert_ne!(state(&shared, &standard), state(&shared, &changed));
    }

    #[test]
    fn guide_lookups_charge_examined_entries_and_refuse_before_mutation() {
        let owners = [0, 1, 2, 3].map(|slot| {
            ComponentAddress::root()
                .child(super::super::AddressRole::CanvasCell, slot)
                .unwrap()
        });
        for target in [0, 2, 3] {
            let examined = (target + 1).min(3) as u128;
            for remaining in [0, examined - 1, examined] {
                let resources = crate::font::canonical::OperationGenerationResources::new();
                let mut round = FeedbackRound::new(None, resources.clone(), 0).unwrap();
                for (index, owner) in owners[..3].iter().enumerate() {
                    let before = resources.usage().unwrap().1;
                    round.claim_guide_trial(owner, 8, 0).unwrap();
                    assert_eq!(resources.usage().unwrap().1 - before, index as u128);
                }
                let before = resources.usage().unwrap().1;
                assert_eq!(round.attempt_ordinal(&owners[target]).unwrap(), 0);
                assert_eq!(resources.usage().unwrap().1 - before, examined);
                resources
                    .claim_visits(1_000_000 - resources.usage().unwrap().1 - remaining)
                    .unwrap();
                let records = resources.usage().unwrap().0;
                let trials = round.guide_trials.clone();
                let events = round.guide_events.clone();
                let capacities = (round.guide_trials.capacity(), round.guide_events.capacity());
                let result = round.claim_guide_trial(&owners[target], 8, 1);
                if remaining == examined {
                    result.unwrap();
                } else {
                    assert!(result
                        .unwrap_err()
                        .to_string()
                        .contains("root processing visits"));
                    assert_eq!(round.guide_trials, trials);
                    assert_eq!(round.guide_events, events);
                    assert_eq!(resources.usage().unwrap().0, records);
                    assert_eq!(
                        (round.guide_trials.capacity(), round.guide_events.capacity()),
                        capacities
                    );
                }
                assert_eq!(resources.usage().unwrap().1, 1_000_000);
                assert!(round
                    .attempt_ordinal(&owners[0])
                    .unwrap_err()
                    .to_string()
                    .contains("root processing visits"));
            }
        }
    }

    #[test]
    fn child_replay_retains_exact_observations_but_reissues_current_round_requests() {
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let owner = ComponentAddress::root()
            .child(super::super::AddressRole::CanvasCell, 0)
            .unwrap();
        let mut seed = FeedbackRound::new(None, resources.clone(), 0).unwrap();
        seed.request_insets(&owner, &owner, FeedbackKind::AxisGutter, [1.0; 4])
            .unwrap();
        let seed = seed.finish().unwrap();
        let mut original = FeedbackRound::new(Some(&seed), resources.clone(), 1).unwrap();
        let start = original.child_checkpoint();
        original.claim_guide_trial(&owner, 2, 0).unwrap();
        original.claim_guide_trial(&owner, 2, 1).unwrap();
        original.observe_text(&text_plan()).unwrap();
        original
            .observe_box(PanelBox::new(1.0, 9.0, 2.0, 8.0))
            .unwrap();
        original
            .request_insets(&owner, &owner, FeedbackKind::AxisGutter, [1.0; 4])
            .unwrap();
        let retained = original.retain_child_feedback(start).unwrap().unwrap();
        let state = original.finish().unwrap();
        let mut replay = FeedbackRound::new(Some(&state), resources, 2).unwrap();
        replay.replay_child(&retained).unwrap();
        assert!(!std::sync::Arc::ptr_eq(
            &state.requests[0],
            &replay.state.requests[0]
        ));
        assert!(std::sync::Arc::ptr_eq(
            &state.observations[0],
            &replay.state.observations[0]
        ));
        assert_eq!(replay.attempt_ordinal(&owner).unwrap(), 1);
        assert!(replay
            .claim_guide_trial(&owner, 2, 2)
            .unwrap_err()
            .to_string()
            .contains("planning limit"));
        assert_eq!(state, replay.finish().unwrap());
    }

    #[test]
    fn changed_requests_and_provisional_misses_cannot_be_retained_as_child_results() {
        let owner = ComponentAddress::root();
        let mut round = FeedbackRound::new(
            None,
            crate::font::canonical::OperationGenerationResources::new(),
            0,
        )
        .unwrap();
        let start = round.child_checkpoint();
        let request = round
            .request_insets(&owner, &owner, FeedbackKind::AxisGutter, [1.0; 4])
            .unwrap();
        assert!(round.retain_child_feedback(start).unwrap().is_none());
        round
            .miss(&owner, FitKind::Tick, &[request], "unresolved")
            .unwrap();
        assert!(round.retain_child_feedback(start).unwrap().is_none());
    }

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
            let Observation::TextPlan(retained) = round.state.observations[0].as_ref() else {
                panic!("one shared text observation")
            };
            assert!(retained.shares_payload(&plan));
            assert!(round.observe_text(&plan).is_err());
            assert!(round.observe_text_block(&block).is_err());
            assert_eq!(round.state.observations.len(), 1);
        }
    }

    #[test]
    fn effective_feedback_uses_getter_defaults_not_request_or_observation_identity() {
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let address = ComponentAddress::root();
        let mut round = FeedbackRound::new(None, resources.clone(), 0).unwrap();
        for kind in [
            FeedbackKind::AxisGutter,
            FeedbackKind::SharedAlignment,
            FeedbackKind::OwnedInk,
        ] {
            assert!(round
                .request_insets(&address, &address, kind, [-0.0; 4])
                .unwrap()
                .is_none());
        }
        let mut state = round.finish().unwrap();
        assert!(!state.requests.is_empty());
        assert!(state.feedback_unchanged());
        // Explicit zeros are also equivalent to an absent consumed entry.
        state.next.push(FeedbackEntry {
            target: address.clone(),
            kind: FeedbackKind::AxisGutter,
            value: FeedbackValue::Insets([0; 4]),
        });
        assert!(state.feedback_unchanged());
        let mut round = FeedbackRound::new(Some(&state), resources, 1).unwrap();
        round
            .observe_string("different current observations")
            .unwrap();
        assert!(round.state.feedback_unchanged());
        round
            .request_domain(
                &address,
                &address,
                FeedbackKind::AutomaticDomainX,
                (0.0, 1.0),
            )
            .unwrap();
        assert!(!round.finish().unwrap().feedback_unchanged());
    }

    #[test]
    fn unchanged_effective_feedback_does_not_clear_a_provisional_miss() {
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let address = ComponentAddress::root();
        let mut round = FeedbackRound::new(None, resources, 0).unwrap();
        let request = round
            .request_insets(&address, &address, FeedbackKind::AxisGutter, [1.0; 4])
            .unwrap();
        round
            .miss(
                &address,
                FitKind::Tick,
                &[request],
                "unresolved current ink",
            )
            .unwrap();
        // Exercise the driver's guard even if net feedback returns to its input.
        round.state.next.clear();
        let state = round.finish().unwrap();
        assert!(state.feedback_unchanged());
        assert!(state.terminal_error().is_none());
        assert!(state
            .unchanged_error()
            .unwrap()
            .to_string()
            .contains("unresolved current ink"));
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
