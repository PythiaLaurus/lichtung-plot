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
    // The cell width and content height are one quantized, replaceable parent
    // input. They are not padding and never restart automatic width selection.
    ContentHeight,
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
            crate::theme::Color,
            u64,
            crate::render::backend::StrokePattern,
        )>,
        fill: Option<FillGlyphState>,
    },
}

#[derive(Debug, PartialEq, Eq)]
struct PointGlyphState {
    color: crate::theme::Color,
    radius: u32,
    shape: crate::PointShape,
    outline: Option<(crate::theme::Color, u64)>,
}

#[derive(Debug, PartialEq, Eq)]
struct FillGlyphState {
    color: crate::theme::Color,
    stroke: crate::theme::Color,
    stroke_width: u64,
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
    let needed = values
        .len()
        .checked_add(count)
        .ok_or_else(|| Error::Render("feedback storage overflow".into()))?;
    if needed <= values.capacity() {
        return Ok(());
    }
    // Unknown feedback streams grow geometrically. Reserving exactly one more
    // entry repeatedly allocates/reprices every prefix and makes cumulative
    // storage quadratic even though each payload is retained only once.
    let capacity = needed.max(values.capacity().saturating_mul(2));
    let bytes = capacity
        .checked_mul(std::mem::size_of::<T>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(|| Error::Render("feedback storage overflow".into()))?;
    // Admit the complete new buffer, not only its delta. Earlier buffers and
    // rejected work remain charged; spare capacity creates no record claims.
    resources.claim_bytes(bytes).map_err(claim_error)?;
    values
        .try_reserve_exact(capacity - values.len())
        .map_err(|error| Error::Render(format!("layout feedback allocation failed: {error}")))
}

fn claim_error(error: impl std::fmt::Debug) -> Error {
    Error::Render(format!("layout feedback admission: {error:?}"))
}

fn arc_payload_bytes<T>() -> u64 {
    (std::mem::size_of::<T>() + 2 * std::mem::size_of::<usize>()) as u64
}

/// Searches admit only entries actually examined, including a terminating match.
fn find_index<T>(
    resources: &crate::font::canonical::OperationGenerationResources,
    entries: &[T],
    mut matches: impl FnMut(&T) -> bool,
) -> Result<Option<usize>, Error> {
    for (index, entry) in entries.iter().enumerate() {
        resources.claim_visits(1).map_err(claim_error)?;
        if matches(entry) {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

impl FeedbackRound {
    pub(crate) fn resources(&self) -> &crate::font::canonical::OperationGenerationResources {
        &self.resources
    }
    fn owned_by(
        &self,
        address: &ComponentAddress,
        owners: &[ComponentAddress],
    ) -> Result<bool, Error> {
        Ok(find_index(&self.resources, owners, |owner| address.is_within(owner))?.is_some())
    }

    fn relevant(&self, entry: &FeedbackEntry, owners: &[ComponentAddress]) -> Result<bool, Error> {
        self.resources.claim_visits(1).map_err(claim_error)?;
        Ok(self.owned_by(&entry.target, owners)? && entry.value != FeedbackValue::Insets([0; 4]))
    }

    pub(crate) fn child_inputs(
        &self,
        owners: &[ComponentAddress],
    ) -> Result<ChildFeedbackInputs, Error> {
        let mut entries = Vec::new();
        for entry in &self.state.consumed {
            if self.relevant(entry, owners)? {
                self.resources.claim_slots(1).map_err(claim_error)?;
                reserve(&self.resources, &mut entries, 1)?;
                entries.push(FeedbackEntry {
                    target: entry.target.clone(),
                    kind: entry.kind,
                    value: entry.value.clone(),
                });
            }
        }
        Ok(ChildFeedbackInputs(entries))
    }

    pub(crate) fn child_stable(&self, owners: &[ComponentAddress]) -> Result<bool, Error> {
        let count = |entries: &[FeedbackEntry]| -> Result<usize, Error> {
            let mut count = 0;
            for entry in entries {
                count += usize::from(self.relevant(entry, owners)?);
            }
            Ok(count)
        };
        // next is not sorted until the round finishes.
        if count(&self.state.consumed)? != count(&self.state.next)? {
            return Ok(false);
        }
        for entry in &self.state.consumed {
            if !self.relevant(entry, owners)? {
                continue;
            }
            let mut found = false;
            for other in &self.state.next {
                if self.relevant(other, owners)? && entry == other {
                    found = true;
                    break;
                }
            }
            if !found {
                return Ok(false);
            }
        }
        for miss in &self.state.misses {
            self.resources.claim_visits(1).map_err(claim_error)?;
            if self.owned_by(&miss.failure.reason.address, owners)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(crate) fn child_inputs_equal(
        &self,
        inputs: &ChildFeedbackInputs,
        owners: &[ComponentAddress],
    ) -> Result<bool, Error> {
        let mut index = 0;
        for entry in &self.state.consumed {
            if self.relevant(entry, owners)? {
                let Some(input) = inputs.0.get(index) else {
                    return Ok(false);
                };
                self.resources.claim_visits(1).map_err(claim_error)?;
                if entry != input {
                    return Ok(false);
                }
                index += 1;
            }
        }
        Ok(index == inputs.0.len())
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
        if self.state.misses.len() != start.2 {
            return Ok(None);
        }
        for request in &self.state.requests[start.1..] {
            self.resources.claim_visits(1).map_err(claim_error)?;
            let prior = find_index(&self.resources, &self.state.consumed, |entry| {
                entry.target == request.entry.target && entry.kind == request.entry.kind
            })?;
            if prior.map_or(
                request.entry.value != FeedbackValue::Insets([0; 4]),
                |index| self.state.consumed[index].value != request.entry.value,
            ) {
                return Ok(None);
            }
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
            guide_events.push((owner.clone(), *maximum, *ordinal));
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
            self.admit_observation_reference()?;
            self.state
                .observations
                .push(std::sync::Arc::clone(observation));
        }
        for request in &child.requests {
            // The source observation is already in the retained stream. Only
            // the new current-round request owns a fresh payload.
            self.publish_request(
                &request.source,
                &request.entry.target,
                request.entry.kind,
                request.entry.value.clone(),
            )?;
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
            self.guide_trials.push((owner.clone(), 1, ordinal));
        }
        self.resources.claim_slots(1).map_err(claim_error)?;
        reserve(&self.resources, &mut self.guide_events, 1)?;
        self.guide_events.push((owner.clone(), maximum, ordinal));
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
                target: entry.target.clone(),
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

    pub(crate) fn finish_trial(
        &mut self,
        checkpoint: TrialCheckpoint,
    ) -> Result<TrialOutcome, Error> {
        let mut least = None;
        for (index, miss) in self.state.misses.iter().enumerate().skip(checkpoint.misses) {
            self.resources.claim_visits(1).map_err(claim_error)?;
            if miss.feedback.is_empty()
                && least.is_none_or(|prior: usize| {
                    miss.failure.reason < self.state.misses[prior].failure.reason
                })
            {
                least = Some(index);
            }
        }
        if let Some(index) = least {
            let miss = self.state.misses.remove(index);
            self.state.misses.truncate(checkpoint.misses);
            self.state.observations.truncate(checkpoint.observations);
            self.state.requests.truncate(checkpoint.requests);
            self.state.next = checkpoint.next;
            Ok(TrialOutcome::Miss(miss.failure))
        } else if self.state.misses.len() > checkpoint.misses {
            Ok(TrialOutcome::Provisional)
        } else {
            Ok(TrialOutcome::Fit)
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
                    target: entry.target.clone(),
                    kind: entry.kind,
                    value: entry.value.clone(),
                });
                next.push(FeedbackEntry {
                    target: entry.target.clone(),
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
        let address = address.clone();
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
            .claim_bytes(arc_payload_bytes::<Observation>())
            .map_err(claim_error)?;
        self.admit_observation_reference()
    }

    fn admit_observation_reference(&mut self) -> Result<(), Error> {
        self.resources.claim_slots(1).map_err(claim_error)?;
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
        canvas: &crate::layout::RootFramePlan,
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
        canvas.root_text.observe(self)
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
    pub(crate) fn insets(
        &self,
        target: &ComponentAddress,
        kind: FeedbackKind,
    ) -> Result<[f64; 4], Error> {
        let index = find_index(&self.resources, &self.state.consumed, |entry| {
            entry.target == *target
                && entry.kind == kind
                && matches!(entry.value, FeedbackValue::Insets(_))
        })?;
        Ok(index.map_or([0.0; 4], |index| {
            let FeedbackValue::Insets(values) = self.state.consumed[index].value else {
                unreachable!("matched inset feedback")
            };
            values.map(|value| value as f64 / 1024.0)
        }))
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "legacy pixel adapters consume exact-root-bounded layout quanta"
    )]
    pub(crate) fn next_insets(
        &self,
        target: &ComponentAddress,
        kind: FeedbackKind,
    ) -> Result<[f64; 4], Error> {
        let index = find_index(&self.resources, &self.state.next, |entry| {
            entry.target == *target
                && entry.kind == kind
                && matches!(entry.value, FeedbackValue::Insets(_))
        })?;
        Ok(index.map_or([0.0; 4], |index| {
            let FeedbackValue::Insets(values) = self.state.next[index].value else {
                unreachable!("matched inset feedback")
            };
            values.map(|value| value as f64 / 1024.0)
        }))
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
            FeedbackKind::AxisGutter
                | FeedbackKind::SharedAlignment
                | FeedbackKind::OwnedInk
                | FeedbackKind::ContentHeight
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
        if matches!(kind, FeedbackKind::SharedAlignment | FeedbackKind::OwnedInk) {
            if let Some(index) = find_index(&self.resources, &self.state.next, |entry| {
                entry.target == *target && entry.kind == kind
            })? {
                if let FeedbackValue::Insets(prior) = self.state.next[index].value {
                    for (value, prior) in normalized.iter_mut().zip(prior) {
                        *value = (*value).max(prior);
                    }
                }
            }
        }
        self.begin_owner(source)?;
        let request =
            self.publish_request(source, target, kind, FeedbackValue::Insets(normalized))?;
        #[cfg(test)]
        #[expect(
            clippy::cast_precision_loss,
            reason = "observer-only gutter coordinates are exact-root-bounded layout quanta"
        )]
        if kind == FeedbackKind::AxisGutter && request.is_some() {
            // Observer-only reads must not add test-only work to the ledger.
            let consumed = self.state.consumed.iter().find_map(|entry| {
                (entry.target == *target && entry.kind == kind).then_some(&entry.value)
            });
            let prior = match consumed {
                Some(FeedbackValue::Insets(values)) => values.map(|value| value as f64 / 1024.0),
                _ => [0.0; 4],
            };
            crate::orchestrator::measurements::gutter_change(
                target,
                prior,
                normalized.map(|value| value as f64 / 1024.0),
            );
        }
        Ok(request)
    }

    pub(crate) fn domain(
        &self,
        target: &ComponentAddress,
        kind: FeedbackKind,
    ) -> Result<Option<(f64, f64)>, Error> {
        let index = find_index(&self.resources, &self.state.consumed, |entry| {
            entry.target == *target
                && entry.kind == kind
                && matches!(entry.value, FeedbackValue::Domain(_))
        })?;
        Ok(index.map(|index| {
            let FeedbackValue::Domain(values) = self.state.consumed[index].value else {
                unreachable!("matched domain feedback")
            };
            (f64::from_bits(values[0]), f64::from_bits(values[1]))
        }))
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
        if let Some(index) = find_index(&self.resources, &self.state.next, |entry| {
            entry.target == *target && entry.kind == kind
        })? {
            let entry = &self.state.next[index];
            if let FeedbackValue::Domain(prior) = entry.value {
                range.0 = range.0.min(f64::from_bits(prior[0]));
                range.1 = range.1.max(f64::from_bits(prior[1]));
            }
        }
        self.begin_owner(source)?;
        self.publish_request(
            source,
            target,
            kind,
            FeedbackValue::Domain([number(range.0)?, number(range.1)?]),
        )
    }

    fn publish_request(
        &mut self,
        source: &ComponentAddress,
        target: &ComponentAddress,
        kind: FeedbackKind,
        value: FeedbackValue,
    ) -> Result<Option<FeedbackRequestId>, Error> {
        if kind == FeedbackKind::AxisGutter
            && find_index(&self.resources, &self.state.requests, |request| {
                request.entry.target == *target
                    && request.entry.kind == kind
                    && request.entry.value != value
            })?
            .is_some()
        {
            return Err(Error::Render(
                "conflicting measured gutter identities in one root round".into(),
            ));
        }
        let absent_zero = value == FeedbackValue::Insets([0; 4])
            && find_index(&self.resources, &self.state.next, |entry| {
                entry.target == *target && entry.kind == kind
            })?
            .is_none();
        // Finish all dependency searches before retaining or applying the
        // request. A refused scan must not publish a partial request identity.
        let (changed, next) = if absent_zero {
            (false, None)
        } else {
            let consumed = find_index(&self.resources, &self.state.consumed, |entry| {
                entry.target == *target && entry.kind == kind
            })?;
            let changed = consumed.is_none_or(|index| self.state.consumed[index].value != value);
            let next = find_index(&self.resources, &self.state.next, |entry| {
                entry.target == *target && entry.kind == kind
            })?;
            (changed, next)
        };
        self.resources.claim_slots(1).map_err(claim_error)?;
        self.resources
            .claim_bytes(arc_payload_bytes::<FeedbackRequest>())
            .map_err(claim_error)?;
        self.resources.claim_slots(1).map_err(claim_error)?;
        reserve(&self.resources, &mut self.state.requests, 1)?;
        let id = FeedbackRequestId {
            index: self.state.requests.len(),
            round: self.ordinal,
        };
        self.state
            .requests
            .push(std::sync::Arc::new(FeedbackRequest {
                source: source.clone(),
                entry: FeedbackEntry {
                    target: target.clone(),
                    kind,
                    value: value.clone(),
                },
            }));
        if absent_zero {
            return Ok(None);
        }
        if let Some(index) = next {
            let entry = &mut self.state.next[index];
            entry.value = value;
        } else {
            self.resources.claim_slots(1).map_err(claim_error)?;
            reserve(&self.resources, &mut self.state.next, 1)?;
            self.state.next.push(FeedbackEntry {
                target: target.clone(),
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
                    address: address.clone(),
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
        self.resources
            .claim_visits(self.state.misses.len() as u128)
            .map_err(claim_error)?;
        self.state
            .misses
            .dedup_by(|a, b| a.failure.reason == b.failure.reason);
        Ok(self.state)
    }
}

impl RoundState {
    pub(super) fn feedback_unchanged(
        &self,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<bool, Error> {
        // Insets default to zero when absent; domains default to None. Both
        // vectors are in canonical target/kind order after finishing a round.
        let mut consumed = self.consumed.iter();
        let mut next = self.next.iter();
        loop {
            match (
                effective_entry(resources, &mut consumed)?,
                effective_entry(resources, &mut next)?,
            ) {
                (None, None) => return Ok(true),
                (Some(left), Some(right)) => {
                    if !equal_feedback_entry(resources, left, right)? {
                        return Ok(false);
                    }
                }
                _ => return Ok(false),
            }
        }
    }

    pub(super) fn terminal_error(
        &self,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<Option<Error>, Error> {
        self.state_error(false, resources)
    }

    pub(super) fn unchanged_error(
        &self,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<Option<Error>, Error> {
        self.state_error(true, resources)
    }

    fn state_error(
        &self,
        include_provisional: bool,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<Option<Error>, Error> {
        let index = find_index(resources, &self.misses, |miss| {
            include_provisional || miss.feedback.is_empty()
        })?;
        Ok(index.map(|index| Error::Render(self.misses[index].failure.to_string())))
    }

    pub(super) fn repeats_history(
        &self,
        history: &[Self],
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<bool, Error> {
        for prior in history {
            resources.claim_visits(1).map_err(claim_error)?;
            if self.same_state(prior, resources)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn same_state(
        &self,
        other: &Self,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<bool, Error> {
        Ok(
            equal_entries(resources, &self.consumed, &other.consumed, |a, b| {
                equal_feedback_entry(resources, a, b)
            })? && equal_entries(resources, &self.next, &other.next, |a, b| {
                equal_feedback_entry(resources, a, b)
            })? && equal_entries(resources, &self.requests, &other.requests, |a, b| {
                equal_request(resources, a, b)
            })? && equal_entries(
                resources,
                &self.observations,
                &other.observations,
                |a, b| {
                    if std::sync::Arc::ptr_eq(a, b) {
                        return Ok(true);
                    }
                    match (a.as_ref(), b.as_ref()) {
                        (Observation::Address(a), Observation::Address(b)) => {
                            equal_address(resources, a, b)
                        }
                        (Observation::Text(a), Observation::Text(b)) => {
                            equal_string(resources, a, b)
                        }
                        (Observation::TextPlan(a), Observation::TextPlan(b)) => {
                            equal_text_plan(resources, a, b)
                        }
                        _ => Ok(a == b),
                    }
                },
            )? && equal_entries(resources, &self.misses, &other.misses, |a, b| {
                Ok(a.failure.reason.kind == b.failure.reason.kind
                    && a.failure.reason.ordinal == b.failure.reason.ordinal
                    && equal_address(
                        resources,
                        &a.failure.reason.address,
                        &b.failure.reason.address,
                    )?
                    && equal_string(resources, &a.failure.message, &b.failure.message)?
                    && equal_entries(resources, &a.feedback, &b.feedback, |a, b| {
                        equal_request(resources, a, b)
                    })?)
            })?,
        )
    }
}

fn effective_entry<'a>(
    resources: &crate::font::canonical::OperationGenerationResources,
    entries: &mut std::slice::Iter<'a, FeedbackEntry>,
) -> Result<Option<&'a FeedbackEntry>, Error> {
    for entry in entries {
        resources.claim_visits(1).map_err(claim_error)?;
        if entry.value != FeedbackValue::Insets([0; 4]) {
            return Ok(Some(entry));
        }
    }
    Ok(None)
}

fn equal_entries<T>(
    resources: &crate::font::canonical::OperationGenerationResources,
    left: &[T],
    right: &[T],
    mut equal: impl FnMut(&T, &T) -> Result<bool, Error>,
) -> Result<bool, Error> {
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.iter().zip(right) {
        resources.claim_visits(1).map_err(claim_error)?;
        if !equal(left, right)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn equal_string(
    resources: &crate::font::canonical::OperationGenerationResources,
    left: &str,
    right: &str,
) -> Result<bool, Error> {
    equal_entries(resources, left.as_bytes(), right.as_bytes(), |a, b| {
        Ok(a == b)
    })
}

fn equal_address(
    resources: &crate::font::canonical::OperationGenerationResources,
    left: &ComponentAddress,
    right: &ComponentAddress,
) -> Result<bool, Error> {
    Ok(left.root == right.root
        && equal_entries(
            resources,
            left.descendants(),
            right.descendants(),
            |a, b| Ok(a == b),
        )?)
}

fn equal_feedback_entry(
    resources: &crate::font::canonical::OperationGenerationResources,
    left: &FeedbackEntry,
    right: &FeedbackEntry,
) -> Result<bool, Error> {
    Ok(equal_address(resources, &left.target, &right.target)?
        && left.kind == right.kind
        && left.value == right.value)
}

fn equal_request(
    resources: &crate::font::canonical::OperationGenerationResources,
    left: &std::sync::Arc<FeedbackRequest>,
    right: &std::sync::Arc<FeedbackRequest>,
) -> Result<bool, Error> {
    Ok(std::sync::Arc::ptr_eq(left, right)
        || (equal_address(resources, &left.source, &right.source)?
            && equal_feedback_entry(resources, &left.entry, &right.entry)?))
}

#[expect(
    clippy::too_many_lines,
    reason = "rendered-plan equality admits each variable string, candidate, line, run, and glyph before its comparison"
)]
fn equal_text_plan(
    resources: &crate::font::canonical::OperationGenerationResources,
    left: &ResolvedTextPlan,
    right: &ResolvedTextPlan,
) -> Result<bool, Error> {
    if left.shares_payload(right) {
        return Ok(true);
    }
    let metadata = |plan: &ResolvedTextPlan| {
        (
            plan.line_pitch_quanta,
            plan.first_baseline_quanta,
            plan.last_baseline_quanta,
            plan.width_quanta,
            plan.height_quanta,
        )
    };
    let style = |plan: &ResolvedTextPlan| {
        (
            plan.style.weight,
            plan.style.font_size,
            plan.style.line_height_quanta,
            plan.style.alignment,
            plan.style.paint,
            plan.style.provenance.pair_ordinal,
            plan.style.exact_override,
        )
    };
    if metadata(left) != metadata(right)
        || style(left) != style(right)
        || left.pair_provenance.pair_ordinal != right.pair_provenance.pair_ordinal
        || left.pair_provenance.weight != right.pair_provenance.weight
        || !equal_string(resources, &left.source, &right.source)?
        || !equal_string(
            resources,
            &left.style.stack_source,
            &right.style.stack_source,
        )?
        || !equal_string(
            resources,
            &left.style.provenance.owner,
            &right.style.provenance.owner,
        )?
        || !equal_entries(
            resources,
            &left.pair_provenance.candidates,
            &right.pair_provenance.candidates,
            |a, b| Ok(a == b),
        )?
        || !equal_entries(
            resources,
            &left.style.stack,
            &right.style.stack,
            |a, b| match (a, b) {
                (
                    crate::font::canonical::FamilyPreference::Named(a),
                    crate::font::canonical::FamilyPreference::Named(b),
                ) => equal_string(resources, a, b),
                _ => Ok(a == b),
            },
        )?
    {
        return Ok(false);
    }
    equal_entries(resources, &left.lines, &right.lines, |a, b| {
        if (
            a.source_span,
            a.selected_soft_hyphen,
            a.ending_grapheme_index,
            a.translation_x_quanta,
            a.translation_y_quanta,
            a.logical_advance_quanta,
            a.ink_quanta,
            a.base_direction,
        ) != (
            b.source_span,
            b.selected_soft_hyphen,
            b.ending_grapheme_index,
            b.translation_x_quanta,
            b.translation_y_quanta,
            b.logical_advance_quanta,
            b.ink_quanta,
            b.base_direction,
        ) {
            return Ok(false);
        }
        equal_entries(resources, &a.runs, &b.runs, |a, b| {
            if (
                a.source_span,
                a.face,
                a.direction,
                a.bidi_level,
                a.script,
                a.units_per_em,
                a.design_ascender,
                a.design_descender,
                a.ascender_quanta,
                a.descender_quanta,
            ) != (
                b.source_span,
                b.face,
                b.direction,
                b.bidi_level,
                b.script,
                b.units_per_em,
                b.design_ascender,
                b.design_descender,
                b.ascender_quanta,
                b.descender_quanta,
            ) || (
                a.design_x_advance,
                a.design_y_advance,
                a.logical_advance_quanta,
                a.logical_y_advance_quanta,
                a.ink_quanta,
            ) != (
                b.design_x_advance,
                b.design_y_advance,
                b.logical_advance_quanta,
                b.logical_y_advance_quanta,
                b.ink_quanta,
            ) {
                return Ok(false);
            }
            equal_entries(resources, &a.glyphs, &b.glyphs, |a, b| Ok(a == b))
        })
    })
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
    fn large_feedback_stream_prices_geometric_buffers_without_quadratic_prefix_storage() {
        const COUNT: usize = 16_384;
        let payload_bytes =
            (std::mem::size_of::<Observation>() + 2 * std::mem::size_of::<usize>()) as u64;
        let reference_bytes = std::mem::size_of::<std::sync::Arc<Observation>>() as u64;
        // Capacities 1,2,4,...,16384 retain 32767 reference slots across all
        // allocations. Exact one-entry growth instead prices 134225920 slots.
        let buffers = 32_767 * reference_bytes;
        let bytes = COUNT as u64 * payload_bytes + buffers;
        let resources =
            crate::font::canonical::OperationGenerationResources::with_test_limits(0, bytes);
        let mut round = FeedbackRound::new(None, resources.clone(), 0).unwrap();
        for value in 0..COUNT {
            round.observe_count(value).unwrap();
        }
        assert_eq!(round.state.observations.len(), COUNT);
        assert_eq!(round.state.observations.capacity(), COUNT);
        assert_eq!(resources.usage().unwrap(), (2 * COUNT as u128, 0));
        assert_eq!(resources.byte_usage().unwrap(), bytes);
        assert!((COUNT as u64 * (COUNT as u64 + 1) / 2) * reference_bytes > 536_870_912);
        let before = resources.usage().unwrap();
        assert!(round.observe_count(COUNT).is_err());
        assert_eq!(round.state.observations.len(), COUNT);
        assert_eq!(round.state.observations.capacity(), COUNT);
        assert_eq!(resources.usage().unwrap(), (before.0 + 1, 0));
        assert_eq!(resources.byte_usage().unwrap(), bytes);
    }

    #[test]
    fn feedback_growth_refuses_before_buffer_or_reference_mutation_at_the_exact_byte_boundary() {
        let reference_bytes = std::mem::size_of::<std::sync::Arc<Observation>>() as u64;
        for available in [
            4 * reference_bytes - 1,
            4 * reference_bytes,
            4 * reference_bytes + 1,
        ] {
            let resources = crate::font::canonical::OperationGenerationResources::new();
            let mut round = FeedbackRound::new(None, resources.clone(), 0).unwrap();
            round.observe_count(1).unwrap();
            round.observe_count(2).unwrap();
            assert_eq!(round.state.observations.capacity(), 2);
            let before = (resources.usage().unwrap(), resources.byte_usage().unwrap());
            let first = std::sync::Arc::clone(&round.state.observations[0]);
            let owners = std::sync::Arc::strong_count(&first);
            resources.set_test_limits(before.0 .1, before.1 + available);
            let result = round.admit_observation_reference();
            assert_eq!(result.is_ok(), available >= 4 * reference_bytes);
            assert_eq!(resources.usage().unwrap(), (before.0 .0 + 1, before.0 .1));
            assert_eq!(round.state.observations.len(), 2);
            assert_eq!(std::sync::Arc::strong_count(&first), owners);
            assert_eq!(
                round.state.observations.capacity(),
                if available >= 4 * reference_bytes {
                    4
                } else {
                    2
                }
            );
            assert_eq!(
                resources.byte_usage().unwrap() - before.1,
                if available >= 4 * reference_bytes {
                    4 * reference_bytes
                } else {
                    0
                }
            );
        }
    }

    #[test]
    fn observation_replay_admits_only_references_and_fresh_observations_admit_both_classes() {
        let payload_bytes =
            (std::mem::size_of::<Observation>() + 2 * std::mem::size_of::<usize>()) as u64;
        let reference_bytes = std::mem::size_of::<std::sync::Arc<Observation>>() as u64;
        for replay in [false, true] {
            let expected = if replay {
                reference_bytes
            } else {
                payload_bytes + reference_bytes
            };
            for available in [0, expected - 1, expected, expected + 1] {
                let resources = crate::font::canonical::OperationGenerationResources::new();
                let mut producer = FeedbackRound::new(None, resources.clone(), 0).unwrap();
                let checkpoint = producer.child_checkpoint();
                producer.observe_count(42).unwrap();
                let retained = producer.retain_child_feedback(checkpoint).unwrap().unwrap();
                let before = (resources.usage().unwrap(), resources.byte_usage().unwrap());
                let owners = std::sync::Arc::strong_count(&retained.observations[0]);
                resources.set_test_limits(before.0 .1, before.1 + available);
                let mut consumer = FeedbackRound::new(None, resources.clone(), 1).unwrap();
                let result = if replay {
                    consumer.replay_child(&retained)
                } else {
                    consumer.observe_count(42)
                };
                assert_eq!(result.is_ok(), available >= expected);
                assert_eq!(
                    resources.usage().unwrap().0 - before.0 .0,
                    if replay || available < payload_bytes {
                        1
                    } else {
                        2
                    }
                );
                assert_eq!(
                    resources.byte_usage().unwrap() - before.1,
                    if available >= expected {
                        expected
                    } else if !replay && available >= payload_bytes {
                        payload_bytes
                    } else {
                        0
                    }
                );
                if available < expected {
                    assert!(consumer.state.observations.is_empty());
                    assert_eq!(consumer.state.observations.capacity(), 0);
                    assert_eq!(
                        std::sync::Arc::strong_count(&retained.observations[0]),
                        owners
                    );
                } else if replay {
                    assert!(std::sync::Arc::ptr_eq(
                        &consumer.state.observations[0],
                        &retained.observations[0]
                    ));
                }
            }
        }
    }

    #[test]
    fn fresh_request_payload_and_reference_refusals_preserve_admitted_prefix_without_publication() {
        let observation_bytes = (std::mem::size_of::<Observation>()
            + 2 * std::mem::size_of::<usize>()
            + std::mem::size_of::<std::sync::Arc<Observation>>())
            as u64;
        let payload_bytes =
            (std::mem::size_of::<FeedbackRequest>() + 2 * std::mem::size_of::<usize>()) as u64;
        let reference_bytes = std::mem::size_of::<std::sync::Arc<FeedbackRequest>>() as u64;
        let expected = observation_bytes + payload_bytes + reference_bytes;
        for available in [
            observation_bytes + payload_bytes - 1,
            expected - 1,
            expected,
            expected + 1,
        ] {
            let resources = crate::font::canonical::OperationGenerationResources::with_test_limits(
                0, available,
            );
            let mut round = FeedbackRound::new(None, resources.clone(), 0).unwrap();
            let root = ComponentAddress::root();
            let result = round.request_insets(&root, &root, FeedbackKind::AxisGutter, [0.0; 4]);
            assert_eq!(result.is_ok(), available >= expected);
            assert_eq!(round.state.observations.len(), 1);
            assert!(round.state.next.is_empty());
            if available >= expected {
                assert!(result.unwrap().is_none());
                assert_eq!(round.state.requests.len(), 1);
                assert_eq!(resources.byte_usage().unwrap(), expected);
            } else {
                assert!(round.state.requests.is_empty());
                assert_eq!(round.state.requests.capacity(), 0);
                assert_eq!(
                    resources.byte_usage().unwrap(),
                    if available < observation_bytes + payload_bytes {
                        observation_bytes
                    } else {
                        observation_bytes + payload_bytes
                    }
                );
            }
        }
        let resources = crate::font::canonical::OperationGenerationResources::new();
        resources.claim_slots(999_997).unwrap();
        let mut round = FeedbackRound::new(None, resources.clone(), 0).unwrap();
        assert!(round
            .request_insets(
                &ComponentAddress::root(),
                &ComponentAddress::root(),
                FeedbackKind::AxisGutter,
                [0.0; 4]
            )
            .is_err());
        assert_eq!(resources.usage().unwrap(), (1_000_000, 0));
        assert_eq!(
            resources.byte_usage().unwrap(),
            observation_bytes + payload_bytes
        );
        assert!(round.state.requests.is_empty());
        assert_eq!(round.state.requests.capacity(), 0);
    }

    #[test]
    fn observation_payload_and_replay_reference_have_independent_record_boundaries() {
        for replay in [false, true] {
            for available in [0, 1, 2] {
                let resources = crate::font::canonical::OperationGenerationResources::new();
                let mut producer = FeedbackRound::new(None, resources.clone(), 0).unwrap();
                let checkpoint = producer.child_checkpoint();
                producer.observe_count(42).unwrap();
                let retained = producer.retain_child_feedback(checkpoint).unwrap().unwrap();
                resources
                    .claim_slots(1_000_000 - resources.usage().unwrap().0 - available)
                    .unwrap();
                let before = (resources.usage().unwrap(), resources.byte_usage().unwrap());
                let mut consumer = FeedbackRound::new(None, resources.clone(), 1).unwrap();
                let result = if replay {
                    consumer.replay_child(&retained)
                } else {
                    consumer.observe_count(42)
                };
                let expected = if replay { 1 } else { 2 };
                assert_eq!(result.is_ok(), available >= expected);
                assert_eq!(
                    resources.usage().unwrap().0 - before.0 .0,
                    available.min(expected)
                );
                if available < expected {
                    assert!(consumer.state.observations.is_empty());
                    assert_eq!(consumer.state.observations.capacity(), 0);
                    assert_eq!(
                        resources.byte_usage().unwrap() - before.1,
                        if replay || available == 0 {
                            0
                        } else {
                            (std::mem::size_of::<Observation>() + 2 * std::mem::size_of::<usize>())
                                as u64
                        }
                    );
                }
            }
        }
    }

    #[test]
    fn content_height_replaces_width_and_height_instead_of_accumulating_ink() {
        let resources = super::super::ResourceState::new();
        let owner = ComponentAddress::root();
        let mut first =
            FeedbackRound::new(None, resources.generation_resources.clone(), 0).unwrap();
        first
            .request_insets(
                &owner,
                &owner,
                FeedbackKind::ContentHeight,
                [400.0, 0.0, 300.0, 0.0],
            )
            .unwrap();
        let previous = first.finish().unwrap();
        let mut second =
            FeedbackRound::new(Some(&previous), resources.generation_resources.clone(), 1).unwrap();
        second
            .request_insets(
                &owner,
                &owner,
                FeedbackKind::ContentHeight,
                [200.0, 0.0, 150.0, 0.0],
            )
            .unwrap();
        assert_eq!(
            second
                .next_insets(&owner, FeedbackKind::ContentHeight)
                .unwrap()
                .map(f64::to_bits),
            [200.0_f64, 0.0, 150.0, 0.0].map(f64::to_bits)
        );
        second
            .request_insets(
                &owner,
                &owner,
                FeedbackKind::ContentHeight,
                [180.0, 0.0, 120.0, 0.0],
            )
            .unwrap();
        assert_eq!(
            second
                .next_insets(&owner, FeedbackKind::ContentHeight)
                .unwrap()
                .map(f64::to_bits),
            [180.0_f64, 0.0, 120.0, 0.0].map(f64::to_bits)
        );
    }

    fn scan_entry(slot: u32, value: FeedbackValue) -> FeedbackEntry {
        FeedbackEntry {
            target: ComponentAddress::root()
                .child(
                    super::super::AddressRole::CanvasCell,
                    slot,
                    &crate::font::canonical::OperationGenerationResources::new(),
                )
                .unwrap(),
            kind: if matches!(value, FeedbackValue::Domain(_)) {
                FeedbackKind::AutomaticDomainX
            } else {
                FeedbackKind::AxisGutter
            },
            value,
        }
    }

    fn empty_state() -> RoundState {
        FeedbackRound::new(
            None,
            crate::font::canonical::OperationGenerationResources::new(),
            0,
        )
        .unwrap()
        .state
    }

    #[test]
    fn root_convergence_admits_effective_entries_and_stops_at_first_difference() {
        for changed in [false, true] {
            // Six examined entries, plus two one-step target comparisons for
            // equality. A first effective mismatch stops after four entries
            // and its one-step target, leaving both final entries unused.
            let expected = if changed { 5 } else { 8 };
            for available in [0, expected - 1, expected, expected + 1] {
                let resources =
                    crate::font::canonical::OperationGenerationResources::with_test_limits(
                        available, 0,
                    );
                let mut state = empty_state();
                for slot in 0..3 {
                    let value = if slot == 0 { [0; 4] } else { [1024; 4] };
                    state
                        .consumed
                        .push(scan_entry(slot, FeedbackValue::Insets(value)));
                    state.next.push(scan_entry(
                        slot,
                        FeedbackValue::Insets(if changed && slot == 1 {
                            [2048; 4]
                        } else {
                            value
                        }),
                    ));
                }
                let result = state.feedback_unchanged(&resources);
                if available < expected {
                    assert!(result
                        .unwrap_err()
                        .to_string()
                        .contains("root processing visits"));
                } else {
                    assert_eq!(result.unwrap(), !changed);
                }
                assert_eq!(resources.usage().unwrap(), (0, available.min(expected)));
            }
        }
    }

    #[test]
    fn root_diagnostic_search_admits_only_examined_misses_and_preserves_least_reason() {
        let mut state = empty_state();
        let request = std::sync::Arc::new(FeedbackRequest {
            source: ComponentAddress::root(),
            entry: scan_entry(0, FeedbackValue::Insets([1024; 4])),
        });
        for (index, kind) in [FitKind::Constraint, FitKind::Text, FitKind::Tick]
            .into_iter()
            .enumerate()
        {
            state.misses.push(FitMiss {
                failure: FitScope::root().failure(kind, format!("miss {index}")),
                feedback: if index < 2 {
                    vec![request.clone()]
                } else {
                    vec![]
                },
            });
        }
        for terminal in [false, true] {
            let expected = if terminal { 3 } else { 1 };
            for available in [0, expected - 1, expected, expected + 1] {
                let resources =
                    crate::font::canonical::OperationGenerationResources::with_test_limits(
                        available, 0,
                    );
                let result = if terminal {
                    state.terminal_error(&resources)
                } else {
                    state.unchanged_error(&resources)
                };
                if available < expected {
                    assert!(result
                        .unwrap_err()
                        .to_string()
                        .contains("root processing visits"));
                } else {
                    assert!(result.unwrap().unwrap().to_string().contains(if terminal {
                        "miss 2"
                    } else {
                        "miss 0"
                    }));
                }
                assert_eq!(resources.usage().unwrap(), (0, available.min(expected)));
            }
        }
    }

    #[test]
    fn cycle_history_comparisons_admit_nested_text_and_stop_at_match_or_refusal() {
        let mut state = empty_state();
        state
            .observations
            .push(std::sync::Arc::new(Observation::Text("ab".into())));
        let mut first = empty_state();
        first
            .observations
            .push(std::sync::Arc::new(Observation::Text("xb".into())));
        let mut matching = empty_state();
        matching
            .observations
            .push(std::sync::Arc::new(Observation::Text("ab".into())));
        let mut unused = empty_state();
        unused
            .observations
            .push(std::sync::Arc::new(Observation::Text("ab".into())));
        let history = [first, matching, unused];
        // First state: history entry + observation + first differing byte.
        // Second: history entry + observation + both matching bytes. No tail.
        for available in [0, 6, 7, 8] {
            let resources = crate::font::canonical::OperationGenerationResources::with_test_limits(
                available, 0,
            );
            let result = state.repeats_history(&history, &resources);
            if available < 7 {
                assert!(result
                    .unwrap_err()
                    .to_string()
                    .contains("root processing visits"));
            } else {
                assert!(result.unwrap());
            }
            assert_eq!(resources.usage().unwrap(), (0, available.min(7)));
        }
        let resources =
            crate::font::canonical::OperationGenerationResources::with_test_limits(2, 0);
        assert!(state
            .repeats_history(std::slice::from_ref(&state), &resources)
            .unwrap());
        assert_eq!(
            resources.usage().unwrap().1,
            2,
            "shared observation skips payload comparison"
        );
    }

    #[test]
    fn admitted_text_history_equality_matches_rendered_identity_and_checks_glyphs() {
        let plan = text_plan();
        let independent = text_plan();
        assert!(!plan.shares_payload(&independent));
        assert_eq!(plan.pair_provenance.candidates.len(), 1);
        assert_eq!(plan.lines.len(), 1);
        assert_eq!(plan.lines[0].runs.len(), 1);
        assert_eq!(plan.lines[0].runs[0].glyphs.len(), 32);
        // 32 source bytes, 9 stack-source bytes, 13 owner bytes, one candidate,
        // one named family + its 9 bytes, one line, one run, and 32 glyph pairs.
        for available in [0, 98, 99, 100] {
            let bounded = crate::font::canonical::OperationGenerationResources::with_test_limits(
                available, 0,
            );
            let result = equal_text_plan(&bounded, &plan, &independent);
            if available < 99 {
                assert!(result
                    .unwrap_err()
                    .to_string()
                    .contains("root processing visits"));
            } else {
                assert!(result.unwrap());
            }
            assert_eq!(bounded.usage().unwrap(), (0, available.min(99)));
        }
        let resources = crate::font::canonical::OperationGenerationResources::new();
        assert!(equal_text_plan(&resources, &plan, &independent).unwrap());
        let mut changed = independent.clone();
        changed.edit_for_test().lines[0].runs[0].glyphs[0].glyph_id += 1;
        assert_ne!(plan, changed);
        let before = resources.usage().unwrap().1;
        assert!(!equal_text_plan(&resources, &plan, &changed).unwrap());
        assert_eq!(
            resources.usage().unwrap().1 - before,
            68,
            "the first differing glyph leaves 31 unexamined glyphs"
        );
        resources.set_test_limits(resources.usage().unwrap().1, 536_870_912);
        assert!(equal_text_plan(&resources, &plan, &plan).unwrap());
        assert!(equal_text_plan(&resources, &plan, &independent)
            .unwrap_err()
            .to_string()
            .contains("root processing visits"));
    }

    #[test]
    fn feedback_getters_admit_terminating_matches_and_stop_on_refusal() {
        for domain in [false, true] {
            for target in [0, 2, 3] {
                let examined = u128::from((target + 1).min(3));
                for available in [0, examined - 1, examined, examined + 1] {
                    let resources = crate::font::canonical::OperationGenerationResources::new();
                    let mut round = FeedbackRound::new(None, resources.clone(), 0).unwrap();
                    for slot in 0..3 {
                        let value = if domain {
                            FeedbackValue::Domain([0.0_f64.to_bits(), 2.0_f64.to_bits()])
                        } else {
                            FeedbackValue::Insets([1024; 4])
                        };
                        round.state.consumed.push(scan_entry(slot, value.clone()));
                        round.state.next.push(scan_entry(slot, value));
                    }
                    let owner = scan_entry(target, FeedbackValue::Insets([0; 4])).target;
                    let before = resources.usage().unwrap();
                    resources.set_test_limits(available, 536_870_912);
                    if domain {
                        let result = round.domain(&owner, FeedbackKind::AutomaticDomainX);
                        if available >= examined {
                            assert_eq!(result.unwrap(), (target < 3).then_some((0.0, 2.0)));
                        } else {
                            assert!(result
                                .unwrap_err()
                                .to_string()
                                .contains("root processing visits"));
                        }
                    } else {
                        let result = round.insets(&owner, FeedbackKind::AxisGutter);
                        if available >= examined {
                            assert_eq!(
                                result.unwrap().map(f64::to_bits),
                                if target < 3 {
                                    [1.0_f64.to_bits(); 4]
                                } else {
                                    [0.0_f64.to_bits(); 4]
                                }
                            );
                        } else {
                            assert!(result
                                .unwrap_err()
                                .to_string()
                                .contains("root processing visits"));
                        }
                        resources
                            .set_test_limits(resources.usage().unwrap().1 + available, 536_870_912);
                        assert_eq!(
                            round.next_insets(&owner, FeedbackKind::AxisGutter).is_ok(),
                            available >= examined
                        );
                    }
                    assert_eq!(resources.usage().unwrap().0, before.0);
                    let multiplier = if domain { 1 } else { 2 };
                    assert_eq!(
                        resources.usage().unwrap().1,
                        available.min(examined) * multiplier
                    );
                    assert_eq!(round.state.consumed, round.state.next);
                }
            }
        }
    }

    #[test]
    fn child_dependency_scans_price_quadratic_searches_and_cached_inputs_exactly() {
        for owner_count in [1, 2] {
            // Three consumed + three next entries for the count passes, three
            // consumed entries for membership, and 1+2+3 next entries searched.
            // Each entry first admits its scan, then every examined owner.
            let stability_visits = 15 * (1 + owner_count);
            for available in [stability_visits - 1, stability_visits, stability_visits + 1] {
                let resources = crate::font::canonical::OperationGenerationResources::new();
                let mut round = FeedbackRound::new(None, resources.clone(), 0).unwrap();
                for slot in 0..3 {
                    round
                        .state
                        .consumed
                        .push(scan_entry(slot, FeedbackValue::Insets([1024; 4])));
                    round
                        .state
                        .next
                        .push(scan_entry(slot, FeedbackValue::Insets([1024; 4])));
                }
                let unrelated = scan_entry(99, FeedbackValue::Insets([0; 4])).target;
                let mut owners = Vec::new();
                if owner_count == 2 {
                    owners.push(unrelated);
                }
                owners.push(ComponentAddress::root());
                resources.set_test_limits(available, 536_870_912);
                let result = round.child_stable(&owners);
                if available < stability_visits {
                    assert!(result
                        .unwrap_err()
                        .to_string()
                        .contains("root processing visits"));
                } else {
                    assert!(result.unwrap());
                }
                assert_eq!(
                    resources.usage().unwrap().1,
                    available.min(stability_visits)
                );
            }
        }

        let resources = crate::font::canonical::OperationGenerationResources::new();
        let mut first = FeedbackRound::new(None, resources.clone(), 0).unwrap();
        for slot in 0..3 {
            first
                .state
                .next
                .push(scan_entry(slot, FeedbackValue::Insets([1024; 4])));
        }
        let state = first.finish().unwrap();
        let mut second = FeedbackRound::new(Some(&state), resources.clone(), 1).unwrap();
        let owners = [ComponentAddress::root()];
        let before = resources.usage().unwrap().1;
        let inputs = second.child_inputs(&owners).unwrap();
        assert_eq!(resources.usage().unwrap().1 - before, 6);
        let checkpoint = second.child_checkpoint();
        second.observe_count(7).unwrap();
        let cached = second.retain_child_feedback(checkpoint).unwrap().unwrap();
        let state = second.finish().unwrap();
        let mut third = FeedbackRound::new(Some(&state), resources.clone(), 2).unwrap();
        let before = resources.usage().unwrap().1;
        // Three entry scans + three owner matches + three cached comparisons,
        // followed by the independently counted 30-visit stability check.
        resources.set_test_limits(before + 39, 536_870_912);
        assert!(third.child_inputs_equal(&inputs, &owners).unwrap());
        assert!(third.child_stable(&owners).unwrap());
        third.replay_child(&cached).unwrap();
        assert_eq!(resources.usage().unwrap().1 - before, 39);
        assert!(third
            .child_inputs_equal(&inputs, &owners)
            .unwrap_err()
            .to_string()
            .contains("root processing visits"));
        assert_eq!(third.state.observations, state.observations);
    }

    #[test]
    fn feedback_request_searches_refuse_before_publishing_or_changing_dependencies() {
        for target in [0, 2, 3] {
            // OwnedInk first finds its merge operand in next, then request()
            // searches consumed and next. Each search stops on its match.
            let examined = 3 * u128::from((target + 1).min(3));
            for available in [examined - 1, examined, examined + 1] {
                let resources = crate::font::canonical::OperationGenerationResources::new();
                let mut round = FeedbackRound::new(None, resources.clone(), 1).unwrap();
                for slot in 0..3 {
                    let mut consumed = scan_entry(slot, FeedbackValue::Insets([1024; 4]));
                    consumed.kind = FeedbackKind::OwnedInk;
                    let mut next = scan_entry(slot, FeedbackValue::Insets([1024; 4]));
                    next.kind = FeedbackKind::OwnedInk;
                    round.state.consumed.push(consumed);
                    round.state.next.push(next);
                }
                let owner = scan_entry(target, FeedbackValue::Insets([0; 4])).target;
                resources.set_test_limits(available, 536_870_912);
                let result = round.request_insets(&owner, &owner, FeedbackKind::OwnedInk, [2.0; 4]);
                if available < examined {
                    assert!(result
                        .unwrap_err()
                        .to_string()
                        .contains("root processing visits"));
                    assert!(round.state.requests.is_empty());
                    assert_eq!(round.state.consumed, round.state.next);
                } else {
                    assert!(result.unwrap().is_some());
                    assert_eq!(round.state.requests.len(), 1);
                    assert_eq!(
                        round.state.next[usize::try_from(target).unwrap()].value,
                        FeedbackValue::Insets([2048; 4])
                    );
                }
                assert_eq!(resources.usage().unwrap().1, available.min(examined));
            }
        }
    }

    #[test]
    fn terminal_trial_scan_admits_every_examined_miss_before_rollback() {
        for available in [0, 2, 3, 4] {
            let resources = crate::font::canonical::OperationGenerationResources::new();
            let mut round = FeedbackRound::new(None, resources.clone(), 1).unwrap();
            let checkpoint = round.begin_trial().unwrap();
            round.observe_count(7).unwrap();
            let scope = FitScope::root();
            for kind in [FitKind::Tick, FitKind::Constraint, FitKind::Text] {
                round
                    .record_failure(scope.failure(kind, "trial failure"))
                    .unwrap();
            }
            resources.set_test_limits(available, 536_870_912);
            let result = round.finish_trial(checkpoint);
            if available < 3 {
                assert!(result
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("root processing visits"));
                assert_eq!(round.state.misses.len(), 3);
                assert_eq!(round.state.observations.len(), 1);
            } else {
                let TrialOutcome::Miss(failure) = result.unwrap() else {
                    panic!("terminal trial must return its least diagnostic")
                };
                assert_eq!(failure.reason.kind, FitKind::Constraint);
                assert!(round.state.misses.is_empty());
                assert!(round.state.observations.is_empty());
            }
            assert_eq!(resources.usage().unwrap().1, available.min(3));
        }
    }

    #[test]
    fn cached_input_mismatch_stops_before_unused_feedback_tail() {
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let mut round = FeedbackRound::new(None, resources.clone(), 1).unwrap();
        round.state.consumed = (0..3)
            .map(|slot| scan_entry(slot, FeedbackValue::Insets([1024; 4])))
            .collect();
        let inputs = ChildFeedbackInputs(vec![scan_entry(0, FeedbackValue::Insets([2048; 4]))]);
        resources.set_test_limits(3, 536_870_912);
        assert!(!round
            .child_inputs_equal(&inputs, &[ComponentAddress::root()])
            .unwrap());
        assert_eq!(resources.usage().unwrap().1, 3);
    }

    #[test]
    fn interior_consequences_keep_source_target_quantization_and_operation_budget() {
        use crate::layout::interior::{ClearanceAxis, DomainExpansion, InteriorReserveRequest};
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let mut round = FeedbackRound::new(None, resources.clone(), 0).unwrap();
        let target = ComponentAddress::root();
        let source = target
            .child(super::super::AddressRole::MarkLayer, 0, &resources)
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
            round.finish_trial(checkpoint).unwrap(),
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
                .child(
                    super::super::AddressRole::CanvasCell,
                    slot,
                    &crate::font::canonical::OperationGenerationResources::new(),
                )
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
            .child(super::super::AddressRole::CanvasCell, 0, &resources)
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
                paint: crate::theme::Color::rgb(0, 0, 0),
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
                round.finish_trial(checkpoint).unwrap(),
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
    fn multiple_guide_owners_keep_independent_budgets_through_rejected_trials() {
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let owners = [0, 1].map(|index| {
            ComponentAddress::root()
                .child(super::super::AddressRole::CanvasCell, index, &resources)
                .unwrap()
        });
        let mut round = FeedbackRound::new(None, resources.clone(), 0).unwrap();
        for ordinal in 0..8 {
            for owner in &owners {
                round.claim_guide_trial(owner, 8, ordinal).unwrap();
                let trial = round.begin_trial().unwrap();
                round
                    .miss(
                        owner,
                        FitKind::TrackOrResidual,
                        &[],
                        "containing frame rejected this guide trial",
                    )
                    .unwrap();
                assert!(matches!(
                    round.finish_trial(trial).unwrap(),
                    TrialOutcome::Miss(_)
                ));
            }
        }
        assert!(round.state.misses.is_empty());
        for owner in &owners {
            assert_eq!(round.attempt_ordinal(owner).unwrap(), 7);
            assert!(round
                .claim_guide_trial(owner, 8, 8)
                .unwrap_err()
                .to_string()
                .contains("planning limit"));
        }
        let state = round.finish().unwrap();
        let mut next = FeedbackRound::new(Some(&state), resources, 1).unwrap();
        for owner in &owners {
            next.claim_guide_trial(owner, 8, 0).unwrap();
            assert_eq!(next.attempt_ordinal(owner).unwrap(), 0);
        }
    }

    #[test]
    fn text_observations_share_admitted_payload_and_refuse_before_retention() {
        let plan = text_plan();
        let block = crate::layout::TextBlock::from_resolved(plan.clone());
        for as_block in [false, true] {
            let resources = crate::font::canonical::OperationGenerationResources::new();
            resources.claim_slots(999_998).unwrap();
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
        assert!(state.feedback_unchanged(&resources).unwrap());
        // Explicit zeros are also equivalent to an absent consumed entry.
        state.next.push(FeedbackEntry {
            target: address.clone(),
            kind: FeedbackKind::AxisGutter,
            value: FeedbackValue::Insets([0; 4]),
        });
        assert!(state.feedback_unchanged(&resources).unwrap());
        let mut round = FeedbackRound::new(Some(&state), resources.clone(), 1).unwrap();
        round
            .observe_string("different current observations")
            .unwrap();
        assert!(round.state.feedback_unchanged(&resources).unwrap());
        round
            .request_domain(
                &address,
                &address,
                FeedbackKind::AutomaticDomainX,
                (0.0, 1.0),
            )
            .unwrap();
        assert!(!round
            .finish()
            .unwrap()
            .feedback_unchanged(&resources)
            .unwrap());
    }

    #[test]
    fn unchanged_effective_feedback_does_not_clear_a_provisional_miss() {
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let address = ComponentAddress::root();
        let mut round = FeedbackRound::new(None, resources.clone(), 0).unwrap();
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
        assert!(state.feedback_unchanged(&resources).unwrap());
        assert!(state.terminal_error(&resources).unwrap().is_none());
        assert!(state
            .unchanged_error(&resources)
            .unwrap()
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
        let mut next = FeedbackRound::new(Some(&state), resources.clone(), 1).unwrap();
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
            .terminal_error(&resources)
            .unwrap()
            .unwrap()
            .to_string()
            .contains("no changed input"));
    }
}
