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
    AuthoredText(crate::Text),
    NumericEmitter(crate::numeric::function::Emitter),
    PositionFunction(crate::spec::ticks::function::Function),
    TextLayout(Option<crate::TextLayout>),
    NumericContext(std::sync::Arc<crate::numeric::context::ContextInfo>),
    TextPlan(ResolvedTextPlan),
    Guide(std::sync::Arc<crate::decorations::legend::discrete::PreparedGuide>),
    Alpha(std::sync::Arc<crate::decorations::legend::alpha::PreparedAlphaInputs>),
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
    outline: Option<(crate::theme::Color, crate::theme::RelativeSpacing)>,
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

/// One deterministic feedback round. No child can start another round.
pub(crate) struct FeedbackRound {
    state: RoundState,
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

fn reserve<T>(values: &mut Vec<T>, count: usize) -> Result<(), Error> {
    let needed = values
        .len()
        .checked_add(count)
        .ok_or_else(|| Error::Render("feedback storage overflow".into()))?;
    if needed <= values.capacity() {
        return Ok(());
    }
    // Unknown feedback streams grow geometrically instead of reallocating each prefix.
    let capacity = needed.max(values.capacity().saturating_mul(2));

    values
        .try_reserve_exact(capacity - values.len())
        .map_err(|error| Error::Render(format!("layout feedback allocation failed: {error}")))
}

/// Search stops at the first matching entry.
fn find_index<T>(entries: &[T], mut matches: impl FnMut(&T) -> bool) -> Option<usize> {
    for (index, entry) in entries.iter().enumerate() {
        if matches(entry) {
            return Some(index);
        }
    }
    None
}

impl FeedbackRound {
    pub(crate) fn observe_text_layout(
        &mut self,
        layout: Option<&crate::TextLayout>,
    ) -> Result<(), Error> {
        self.observe(Observation::TextLayout(layout.cloned()))
    }
    fn owned_by(address: &ComponentAddress, owners: &[ComponentAddress]) -> bool {
        find_index(owners, |owner| address.is_within(owner)).is_some()
    }

    fn relevant(entry: &FeedbackEntry, owners: &[ComponentAddress]) -> bool {
        Self::owned_by(&entry.target, owners) && entry.value != FeedbackValue::Insets([0; 4])
    }

    pub(crate) fn child_inputs(
        &self,
        owners: &[ComponentAddress],
    ) -> Result<ChildFeedbackInputs, Error> {
        let mut entries = Vec::new();
        for entry in &self.state.consumed {
            if Self::relevant(entry, owners) {
                reserve(&mut entries, 1)?;
                entries.push(FeedbackEntry {
                    target: entry.target.clone(),
                    kind: entry.kind,
                    value: entry.value.clone(),
                });
            }
        }
        Ok(ChildFeedbackInputs(entries))
    }

    pub(crate) fn child_stable(&self, owners: &[ComponentAddress]) -> bool {
        let count = |entries: &[FeedbackEntry]| {
            entries
                .iter()
                .filter(|entry| Self::relevant(entry, owners))
                .count()
        };
        // next is not sorted until the round finishes.
        if count(&self.state.consumed) != count(&self.state.next) {
            return false;
        }
        for entry in &self.state.consumed {
            if !Self::relevant(entry, owners) {
                continue;
            }
            let mut found = false;
            for other in &self.state.next {
                if Self::relevant(other, owners) && entry == other {
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        for miss in &self.state.misses {
            if Self::owned_by(&miss.failure.reason.address, owners) {
                return false;
            }
        }
        true
    }

    pub(crate) fn child_inputs_equal(
        &self,
        inputs: &ChildFeedbackInputs,
        owners: &[ComponentAddress],
    ) -> bool {
        let mut index = 0;
        for entry in &self.state.consumed {
            if Self::relevant(entry, owners) {
                let Some(input) = inputs.0.get(index) else {
                    return false;
                };

                if entry != input {
                    return false;
                }
                index += 1;
            }
        }
        index == inputs.0.len()
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
            let prior = find_index(&self.state.consumed, |entry| {
                entry.target == request.entry.target && entry.kind == request.entry.kind
            });
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

        reserve(&mut observations, self.state.observations.len() - start.0)?;
        reserve(&mut requests, self.state.requests.len() - start.1)?;
        reserve(&mut guide_events, self.guide_events.len() - start.3)?;
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
            self.reserve_observation()?;
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

    pub(crate) fn attempt_ordinal(&self, address: &ComponentAddress) -> u32 {
        for (owner, _, ordinal) in &self.guide_trials {
            if owner == address {
                return *ordinal;
            }
        }
        0
    }
    pub(crate) fn claim_guide_trial(
        &mut self,
        owner: &ComponentAddress,
        maximum: u8,
        ordinal: u32,
    ) -> Result<(), Error> {
        let mut found = None;
        for entry in &mut self.guide_trials {
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
            reserve(&mut self.guide_trials, 1)?;
            self.guide_trials.push((owner.clone(), 1, ordinal));
        }

        reserve(&mut self.guide_events, 1)?;
        self.guide_events.push((owner.clone(), maximum, ordinal));
        Ok(())
    }

    pub(crate) fn begin_trial(&mut self) -> Result<TrialCheckpoint, Error> {
        let mut next = Vec::new();

        reserve(&mut next, self.state.next.len())?;
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

    pub(crate) fn finish_trial(&mut self, checkpoint: TrialCheckpoint) -> TrialOutcome {
        let mut least = None;
        for (index, miss) in self.state.misses.iter().enumerate().skip(checkpoint.misses) {
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
    pub(super) fn new(previous: Option<&RoundState>, ordinal: usize) -> Result<Self, Error> {
        let mut consumed = Vec::new();
        let mut next = Vec::new();
        if let Some(previous) = previous {
            reserve(&mut consumed, previous.next.len())?;
            reserve(&mut next, previous.next.len())?;
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
            ordinal,
            guide_trials: Vec::new(),
            guide_events: Vec::new(),
        })
    }

    pub(crate) fn begin_owner(&mut self, address: &ComponentAddress) -> Result<(), Error> {
        self.reserve_observation()?;
        let address = address.clone();
        self.state
            .observations
            .push(std::sync::Arc::new(Observation::Address(address)));
        Ok(())
    }

    fn observe(&mut self, observation: Observation) -> Result<(), Error> {
        self.reserve_observation()?;
        self.state
            .observations
            .push(std::sync::Arc::new(observation));
        Ok(())
    }

    fn reserve_observation(&mut self) -> Result<(), Error> {
        reserve(&mut self.state.observations, 1)?;
        Ok(())
    }

    pub(crate) fn observe_number(&mut self, value: f64) -> Result<(), Error> {
        self.observe(Observation::Number(number(value)?))
    }

    pub(crate) fn observe_count(&mut self, value: usize) -> Result<(), Error> {
        self.observe(Observation::Count(value))
    }

    pub(crate) fn observe_guide(
        &mut self,
        descriptor: &std::sync::Arc<crate::decorations::legend::discrete::PreparedGuide>,
    ) -> Result<(), Error> {
        self.observe(Observation::Guide(std::sync::Arc::clone(descriptor)))
    }

    pub(crate) fn observe_alpha(
        &mut self,
        inputs: &std::sync::Arc<crate::decorations::legend::alpha::PreparedAlphaInputs>,
    ) -> Result<(), Error> {
        self.observe(Observation::Alpha(std::sync::Arc::clone(inputs)))
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
                        .as_ref()
                        .map(|outline| (outline.color, outline.width_ratio.clone())),
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
        self.reserve_observation()?;
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

    pub(crate) fn observe_font_stack(
        &mut self,
        stack: &crate::font::canonical::FontStack,
    ) -> Result<(), Error> {
        self.observe_count(stack.families().len())?;
        for family in stack.families() {
            self.observe_count(usize::from(family.name().is_some()))?;
            if let Some(name) = family.name() {
                self.observe_string(name)?;
            } else {
                self.observe_count(family.generic_family().expect("generic family") as usize)?;
            }
        }
        Ok(())
    }

    pub(crate) fn observe_authored_text(&mut self, value: &crate::Text) -> Result<(), Error> {
        self.observe(Observation::AuthoredText(value.try_copy()?))
    }

    pub(crate) fn observe_numeric_emitter(
        &mut self,
        value: &crate::numeric::function::Emitter,
    ) -> Result<(), Error> {
        self.observe(Observation::NumericEmitter(value.clone()))
    }
    pub(crate) fn observe_position_function(
        &mut self,
        value: &crate::spec::ticks::function::Function,
    ) -> Result<(), Error> {
        self.observe(Observation::PositionFunction(value.clone()))
    }
    pub(crate) fn observe_numeric_context(
        &mut self,
        value: &std::sync::Arc<crate::numeric::context::ContextInfo>,
    ) -> Result<(), Error> {
        self.observe(Observation::NumericContext(value.clone()))
    }

    pub(crate) fn observe_string(&mut self, value: &str) -> Result<(), Error> {
        self.reserve_observation()?;
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
            if let Some(numeric) = &axis.numeric {
                numeric.observe_feedback(self)?;
            }
            self.observe_count(usize::from(axis.tick_labels_visible))?;
            self.observe_count(usize::from(axis.explicit_ticks))?;
            self.observe_count(usize::from(axis.assigned_tick_width.is_some()))?;
            if let Some(width) = axis.assigned_tick_width {
                self.observe_number(width)?;
            }
            self.observe_count(usize::from(axis.label.is_some()))?;
            if let Some(label) = &axis.label {
                self.observe_authored_text(label)?;
            }
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
        let index = find_index(&self.state.consumed, |entry| {
            entry.target == *target
                && entry.kind == kind
                && matches!(entry.value, FeedbackValue::Insets(_))
        });
        index.map_or([0.0; 4], |index| {
            let FeedbackValue::Insets(values) = self.state.consumed[index].value else {
                unreachable!("matched inset feedback")
            };
            values.map(|value| value as f64 / 1024.0)
        })
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "legacy pixel adapters consume exact-root-bounded layout quanta"
    )]
    pub(crate) fn next_insets(&self, target: &ComponentAddress, kind: FeedbackKind) -> [f64; 4] {
        let index = find_index(&self.state.next, |entry| {
            entry.target == *target
                && entry.kind == kind
                && matches!(entry.value, FeedbackValue::Insets(_))
        });
        index.map_or([0.0; 4], |index| {
            let FeedbackValue::Insets(values) = self.state.next[index].value else {
                unreachable!("matched inset feedback")
            };
            values.map(|value| value as f64 / 1024.0)
        })
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
            if let Some(index) = find_index(&self.state.next, |entry| {
                entry.target == *target && entry.kind == kind
            }) {
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

    pub(crate) fn feedback_domain(
        &self,
        target: &ComponentAddress,
        kind: FeedbackKind,
    ) -> Option<(f64, f64)> {
        let index = find_index(&self.state.consumed, |entry| {
            entry.target == *target
                && entry.kind == kind
                && matches!(entry.value, FeedbackValue::Domain(_))
        });
        index.map(|index| {
            let FeedbackValue::Domain(values) = self.state.consumed[index].value else {
                unreachable!("matched domain feedback")
            };
            (f64::from_bits(values[0]), f64::from_bits(values[1]))
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
        if let Some(index) = find_index(&self.state.next, |entry| {
            entry.target == *target && entry.kind == kind
        }) {
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
            && find_index(&self.state.requests, |request| {
                request.entry.target == *target
                    && request.entry.kind == kind
                    && request.entry.value != value
            })
            .is_some()
        {
            return Err(Error::Render(
                "conflicting measured gutter identities in one root round".into(),
            ));
        }
        let absent_zero = value == FeedbackValue::Insets([0; 4])
            && find_index(&self.state.next, |entry| {
                entry.target == *target && entry.kind == kind
            })
            .is_none();
        // Finish all dependency searches before retaining or applying the
        // request. A refused scan must not publish a partial request identity.
        let (changed, next) = if absent_zero {
            (false, None)
        } else {
            let consumed = find_index(&self.state.consumed, |entry| {
                entry.target == *target && entry.kind == kind
            });
            let changed = consumed.is_none_or(|index| self.state.consumed[index].value != value);
            let next = find_index(&self.state.next, |entry| {
                entry.target == *target && entry.kind == kind
            });
            (changed, next)
        };

        reserve(&mut self.state.requests, 1)?;
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
            reserve(&mut self.state.next, 1)?;
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
        let ordinal = self.attempt_ordinal(address);
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
        reserve(&mut self.state.misses, 1)?;
        self.state.misses.push(FitMiss {
            failure,
            feedback: Vec::new(),
        });
        Ok(())
    }

    pub(super) fn finish(mut self) -> RoundState {
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
        self.state
    }
}

impl RoundState {
    pub(super) fn feedback_unchanged(&self) -> bool {
        // Insets default to zero when absent; domains default to None. Both
        // vectors are in canonical target/kind order after finishing a round.
        let effective = |entries: &&FeedbackEntry| entries.value != FeedbackValue::Insets([0; 4]);
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
        let index = find_index(&self.misses, |miss| {
            include_provisional || miss.feedback.is_empty()
        });
        index.map(|index| Error::Render(self.misses[index].failure.to_string()))
    }

    pub(super) fn repeats_history(&self, history: &[Self]) -> bool {
        history.iter().any(|prior| self.same_state(prior))
    }

    fn same_state(&self, other: &Self) -> bool {
        self.consumed == other.consumed
            && self.next == other.next
            && self.requests == other.requests
            && equal_entries(&self.observations, &other.observations, |a, b| {
                if std::sync::Arc::ptr_eq(a, b) {
                    return true;
                }
                match (a.as_ref(), b.as_ref()) {
                    (Observation::TextPlan(a), Observation::TextPlan(b)) => equal_text_plan(a, b),
                    _ => a == b,
                }
            })
            && self.misses == other.misses
    }
}

fn equal_entries<T>(left: &[T], right: &[T], mut equal: impl FnMut(&T, &T) -> bool) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| equal(left, right))
}

fn equal_text_plan(left: &ResolvedTextPlan, right: &ResolvedTextPlan) -> bool {
    if left.shares_payload(right) {
        return true;
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
        || left.source != right.source
        || left.style.provenance.owner != right.style.provenance.owner
        || left.pair_provenance.candidates != right.pair_provenance.candidates
        || left.style.stack != right.style.stack
    {
        return false;
    }
    equal_entries(&left.lines, &right.lines, |a, b| {
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
            return false;
        }
        equal_entries(&a.runs, &b.runs, |a, b| {
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
                return false;
            }
            a.glyphs == b.glyphs
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
    fn feedback_reservation_preserves_the_observed_prefix_and_reference_owners() {
        let mut round = FeedbackRound::new(None, 0).unwrap();
        round.observe_count(1).unwrap();
        round.observe_count(2).unwrap();
        assert_eq!(round.state.observations.capacity(), 2);
        let first = std::sync::Arc::clone(&round.state.observations[0]);
        let owners = std::sync::Arc::strong_count(&first);
        round.reserve_observation().unwrap();
        assert_eq!(round.state.observations.len(), 2);
        assert_eq!(round.state.observations.capacity(), 4);
        assert_eq!(std::sync::Arc::strong_count(&first), owners);
        assert!(std::sync::Arc::ptr_eq(&first, &round.state.observations[0]));
    }

    #[test]
    fn observation_replay_shares_payload_while_fresh_observation_owns_its_value() {
        for replay in [false, true] {
            let mut producer = FeedbackRound::new(None, 0).unwrap();
            let checkpoint = producer.child_checkpoint();
            producer.observe_count(42).unwrap();
            let retained = producer.retain_child_feedback(checkpoint).unwrap().unwrap();
            let mut consumer = FeedbackRound::new(None, 1).unwrap();
            if replay {
                consumer.replay_child(&retained).unwrap();
            } else {
                consumer.observe_count(42).unwrap();
            }
            assert_eq!(consumer.state.observations.len(), 1);
            assert_eq!(consumer.state.observations[0], retained.observations[0]);
            assert_eq!(
                std::sync::Arc::ptr_eq(&consumer.state.observations[0], &retained.observations[0]),
                replay
            );
        }
    }

    #[test]
    fn unchanged_inset_request_retains_dependency_without_changed_insets() {
        let mut round = FeedbackRound::new(None, 0).unwrap();
        let root = ComponentAddress::root();
        assert!(round
            .request_insets(&root, &root, FeedbackKind::AxisGutter, [0.0; 4])
            .unwrap()
            .is_none());
        assert_eq!(round.state.observations.len(), 1);
        assert_eq!(round.state.requests.len(), 1);
        assert!(round.state.next.is_empty());
    }

    #[test]
    fn guide_feedback_retains_exact_ratio_identity_with_only_reference_storage() {
        let mut exact = crate::render::backend::PointStyle::new((0, 0, 0), 3);
        exact.outline.as_mut().unwrap().width_ratio =
            crate::theme::RelativeSpacing::ratio(3, 5).unwrap();
        let mut dyadic = exact.clone();
        dyadic.outline.as_mut().unwrap().width_ratio =
            crate::theme::RelativeSpacing::new(0.6).unwrap();
        let mut equivalent = exact.clone();
        equivalent.outline.as_mut().unwrap().width_ratio =
            crate::theme::RelativeSpacing::ratio(6, 10).unwrap();
        let observe = |style: &crate::render::backend::PointStyle| {
            let mut round = FeedbackRound::new(None, 0).unwrap();
            round.observe_legend_glyph(Some(style), None, None).unwrap();
            round.state.observations.pop().unwrap()
        };

        assert_eq!(observe(&exact), observe(&equivalent));
        assert_ne!(observe(&exact), observe(&dyadic));
    }

    #[test]
    fn alpha_feedback_retains_hidden_undeclared_recipes_and_actual_members() {
        use crate::alpha::{AlphaEncoding, AlphaTarget, AlphaValues};
        use crate::decorations::legend::alpha::{AlphaUnit, PreparedAlphaInputs};
        use std::sync::Arc;

        let prepare = |hidden: bool, custom: bool, unused: u8, source_row: usize| {
            let values = if custom {
                AlphaValues::continuous(
                    crate::NumericValues::new("confidence", [0.25, 0.375, 0.75]),
                    (0.0, 1.0),
                    |value| {
                        if value.to_bits() == 0.375_f64.to_bits() {
                            unused
                        } else {
                            0
                        }
                    },
                )
            } else {
                AlphaValues::new("confidence", [0; 3])
            }
            .unwrap();
            let plot = crate::Plot::scatter(
                crate::NumericValues::new("x", [0.0, f64::NAN, 1.0]),
                crate::NumericValues::new("y", [0.0, 1.0, 1.0]),
            )
            .source_rows(crate::SourceRows::new([source_row, 200, 300]).unwrap())
            .with_alpha(AlphaTarget::Point, AlphaEncoding::observations(values))
            .labels(crate::Labels::raw().alias("confidence", "\u{10ffff}"));
            let plot = if hidden {
                plot.legend(crate::LegendPolicy::Hidden)
            } else {
                plot
            }
            .erase();
            let operation = crate::font::canonical::OperationContext::new();
            let theme = crate::Theme::publication().resolve().unwrap();
            let inputs =
                crate::plots::shared_x::prepare_plot(&plot, &operation, Some(&theme)).unwrap();
            let discoveries =
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
            inputs
                .prepare_discrete_guides(
                    &plot,
                    &theme,
                    &operation,
                    &ComponentAddress::root(),
                    false,
                )
                .unwrap();
            let guides = inputs.discrete_guides.get().unwrap();
            assert!(guides.root.is_none());
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                discoveries
            );
            Arc::clone(guides.alpha_for_scope(None).unwrap())
        };
        let observe = |inputs: &Arc<PreparedAlphaInputs>| {
            let mut round = FeedbackRound::new(None, 0).unwrap();
            round.observe_alpha(inputs).unwrap();
            round.finish()
        };
        let hidden = prepare(true, true, 0, 100);
        let independent = prepare(true, true, 0, 100);
        assert!(!Arc::ptr_eq(&hidden, &independent));
        assert!(observe(&hidden).same_state(&observe(&independent)));
        assert!(!observe(&hidden).same_state(&observe(&prepare(true, true, 99, 100))));
        assert!(!observe(&hidden).same_state(&observe(&prepare(true, true, 0, 400))));
        let undeclared = prepare(false, false, 0, 100);
        assert!(!observe(&hidden).same_state(&observe(&undeclared)));
        assert_eq!(hidden.sources[0].glyphs, undeclared.sources[0].glyphs);
        assert_eq!(
            hidden.sources[0].glyphs[0]
                .members
                .iter()
                .map(|member| &member.unit)
                .collect::<Vec<_>>(),
            [
                &AlphaUnit::Observation {
                    slot: 0,
                    source_row: 100
                },
                &AlphaUnit::Observation {
                    slot: 2,
                    source_row: 300
                },
            ]
        );
        let mut producer = FeedbackRound::new(None, 0).unwrap();
        let checkpoint = producer.child_checkpoint();
        producer.observe_alpha(&hidden).unwrap();
        let retained = producer.retain_child_feedback(checkpoint).unwrap().unwrap();
        let mut consumer = FeedbackRound::new(None, 1).unwrap();
        consumer.replay_child(&retained).unwrap();
        assert!(producer.finish().same_state(&consumer.finish()));
    }

    #[test]
    fn content_height_replaces_width_and_height_instead_of_accumulating_ink() {
        let owner = ComponentAddress::root();
        let mut first = FeedbackRound::new(None, 0).unwrap();
        first
            .request_insets(
                &owner,
                &owner,
                FeedbackKind::ContentHeight,
                [400.0, 0.0, 300.0, 0.0],
            )
            .unwrap();
        let previous = first.finish();
        let mut second = FeedbackRound::new(Some(&previous), 1).unwrap();
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
                .map(f64::to_bits),
            [180.0_f64, 0.0, 120.0, 0.0].map(f64::to_bits)
        );
    }

    fn scan_entry(slot: u32, value: FeedbackValue) -> FeedbackEntry {
        FeedbackEntry {
            target: ComponentAddress::root()
                .child(super::super::AddressRole::CanvasCell, slot)
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
        FeedbackRound::new(None, 0).unwrap().state
    }

    #[test]
    fn root_convergence_compares_effective_entries() {
        for changed in [false, true] {
            {
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
                let result = state.feedback_unchanged();
                assert_eq!(result, !changed);
            }
        }
    }

    #[test]
    fn root_diagnostic_search_preserves_least_reason() {
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
            {
                let result = if terminal {
                    state.terminal_error()
                } else {
                    state.unchanged_error()
                };
                {
                    assert!(result.unwrap().to_string().contains(if terminal {
                        "miss 2"
                    } else {
                        "miss 0"
                    }));
                }
            }
        }
    }

    #[test]
    fn cycle_history_compares_nested_text_and_finds_matching_state() {
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
        assert!(state.repeats_history(&history));

        assert!(state.repeats_history(std::slice::from_ref(&state)));
    }

    #[test]
    fn text_history_equality_matches_rendered_identity_and_checks_glyphs() {
        let plan = text_plan();
        let independent = text_plan();
        assert!(!plan.shares_payload(&independent));
        assert_eq!(plan.pair_provenance.candidates.len(), 1);
        assert_eq!(plan.lines.len(), 1);
        assert_eq!(plan.lines[0].runs.len(), 1);
        assert_eq!(plan.lines[0].runs[0].glyphs.len(), 32);
        assert!(equal_text_plan(&plan, &independent));
        let mut changed = independent.clone();
        changed.edit_for_test().lines[0].runs[0].glyphs[0].glyph_id += 1;
        assert_ne!(plan, changed);

        assert!(!equal_text_plan(&plan, &changed));

        assert!(equal_text_plan(&plan, &plan));
    }

    #[test]
    fn child_dependencies_preserve_stability_and_cached_inputs() {
        for owner_count in [1, 2] {
            {
                let mut round = FeedbackRound::new(None, 0).unwrap();
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

                let result = round.child_stable(&owners);
                assert!(result);
            }
        }

        let mut first = FeedbackRound::new(None, 0).unwrap();
        for slot in 0..3 {
            first
                .state
                .next
                .push(scan_entry(slot, FeedbackValue::Insets([1024; 4])));
        }
        let state = first.finish();
        let mut second = FeedbackRound::new(Some(&state), 1).unwrap();
        let owners = [ComponentAddress::root()];

        let inputs = second.child_inputs(&owners).unwrap();

        let checkpoint = second.child_checkpoint();
        second.observe_count(7).unwrap();
        let cached = second.retain_child_feedback(checkpoint).unwrap().unwrap();
        let state = second.finish();
        let mut third = FeedbackRound::new(Some(&state), 2).unwrap();

        assert!(third.child_inputs_equal(&inputs, &owners));
        assert!(third.child_stable(&owners));
        third.replay_child(&cached).unwrap();

        assert!(third.child_inputs_equal(&inputs, &owners));
        assert_eq!(third.state.observations, state.observations);
    }

    #[test]
    fn terminal_trial_returns_least_miss_and_rolls_back_observations() {
        {
            let mut round = FeedbackRound::new(None, 1).unwrap();
            let checkpoint = round.begin_trial().unwrap();
            round.observe_count(7).unwrap();
            let scope = FitScope::root();
            for kind in [FitKind::Tick, FitKind::Constraint, FitKind::Text] {
                round
                    .record_failure(scope.failure(kind, "trial failure"))
                    .unwrap();
            }

            let result = round.finish_trial(checkpoint);
            {
                let TrialOutcome::Miss(failure) = result else {
                    panic!("terminal trial must return its least diagnostic")
                };
                assert_eq!(failure.reason.kind, FitKind::Constraint);
                assert!(round.state.misses.is_empty());
                assert!(round.state.observations.is_empty());
            }
        }
    }

    #[test]
    fn cached_input_mismatch_stops_before_unused_feedback_tail() {
        let mut round = FeedbackRound::new(None, 1).unwrap();
        round.state.consumed = (0..3)
            .map(|slot| scan_entry(slot, FeedbackValue::Insets([1024; 4])))
            .collect();
        let inputs = ChildFeedbackInputs(vec![scan_entry(0, FeedbackValue::Insets([2048; 4]))]);

        assert!(!round.child_inputs_equal(&inputs, &[ComponentAddress::root()]));
    }

    #[test]
    fn interior_consequences_keep_source_target_and_quantization_after_rejected_trial() {
        use crate::layout::interior::{ClearanceAxis, DomainExpansion, InteriorReserveRequest};

        let mut round = FeedbackRound::new(None, 0).unwrap();
        let target = ComponentAddress::root();
        let source = target
            .child(super::super::AddressRole::MarkLayer, 0)
            .unwrap();

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

        assert!(matches!(
            round.finish_trial(checkpoint),
            TrialOutcome::Miss(_)
        ));

        assert!(InteriorReserveRequest::OwnedInk([3.0; 4])
            .submit(&mut round, &source, &target)
            .unwrap()
            .is_some());
        assert_eq!(
            round
                .next_insets(&target, FeedbackKind::OwnedInk)
                .map(f64::to_bits),
            [3.0_f64.to_bits(); 4]
        );
    }

    #[test]
    fn axis_semantics_and_standard_typography_are_feedback_identity() {
        let source = crate::Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        );
        let plot = source.clone().erase();
        let domains = [crate::spec::AxisDomain::Continuous; 2];
        let theme = crate::Theme::publication().resolve().unwrap();
        let standard = crate::layout::StandardPanelInputs::new(&theme).unwrap();
        let state = |plot: &crate::Plot, standard: &crate::layout::StandardPanelInputs| {
            let mut round = FeedbackRound::new(None, 0).unwrap();
            for input in plot.axis_semantic_inputs(domains) {
                input.observe_feedback(&mut round).unwrap();
            }
            standard.observe_feedback(&mut round).unwrap();
            round.finish()
        };
        let automatic = state(&plot, &standard);
        let shared = plot.clone().with_resolved_facet_ranges(
            crate::spec::PlotKind::Scatter,
            Some((0.0, 1.0)),
            None,
        );
        assert_ne!(automatic, state(&shared, &standard));
        let mut generation_states = vec![state(&plot, &standard)];
        for side in 0..2 {
            for generation in [
                crate::TickGeneration::additive(1.0).unwrap(),
                crate::TickGeneration::additive(1.0_f64.next_up()).unwrap(),
                crate::TickGeneration::target(5).unwrap(),
                crate::TickGeneration::target(6).unwrap(),
                crate::TickGeneration::maximum(5).unwrap(),
                crate::TickGeneration::maximum(6).unwrap(),
            ] {
                let changed = if side == 0 {
                    source.clone().x_tick_generation(generation)
                } else {
                    source.clone().y_tick_generation(generation)
                }
                .erase();
                let selected = state(&changed, &standard);
                assert!(generation_states.iter().all(|other| *other != selected));
                assert_eq!(selected, state(&changed.clone(), &standard));
                generation_states.push(selected);
            }
        }
        assert_eq!(
            automatic,
            state(&plot.with_planned_x_limits((0.0, 1.0)), &standard)
        );
        let mut changed_theme = theme.clone();
        changed_theme
            .typography
            .axes
            .tick_label
            .line_height_multiplier += 0.25;
        let changed = crate::layout::StandardPanelInputs::new(&changed_theme).unwrap();
        assert_ne!(state(&shared, &standard), state(&shared, &changed));
    }

    #[test]
    fn child_replay_retains_exact_observations_but_reissues_current_round_requests() {
        let owner = ComponentAddress::root()
            .child(super::super::AddressRole::CanvasCell, 0)
            .unwrap();
        let mut seed = FeedbackRound::new(None, 0).unwrap();
        seed.request_insets(&owner, &owner, FeedbackKind::AxisGutter, [1.0; 4])
            .unwrap();
        let seed = seed.finish();
        let mut original = FeedbackRound::new(Some(&seed), 1).unwrap();
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
        let state = original.finish();
        let mut replay = FeedbackRound::new(Some(&state), 2).unwrap();
        replay.replay_child(&retained).unwrap();
        assert!(!std::sync::Arc::ptr_eq(
            &state.requests[0],
            &replay.state.requests[0]
        ));
        assert!(std::sync::Arc::ptr_eq(
            &state.observations[0],
            &replay.state.observations[0]
        ));
        assert_eq!(replay.attempt_ordinal(&owner), 1);
        assert!(replay
            .claim_guide_trial(&owner, 2, 2)
            .unwrap_err()
            .to_string()
            .contains("planning limit"));
        assert_eq!(state, replay.finish());
    }

    #[test]
    fn changed_requests_and_provisional_misses_cannot_be_retained_as_child_results() {
        let owner = ComponentAddress::root();
        let mut round = FeedbackRound::new(None, 0).unwrap();
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
                direction: crate::TextDirection::Auto,
                source: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                stack: crate::font::canonical::test_sans_stack(),
                weight: crate::TextWeight::Normal,
                font_size: 10.0,
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
        let owner = ComponentAddress::root();
        let mut round = FeedbackRound::new(None, 0).unwrap();
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
    fn multiple_guide_owners_keep_independent_budgets_through_rejected_trials() {
        let owners = [0, 1].map(|index| {
            ComponentAddress::root()
                .child(super::super::AddressRole::CanvasCell, index)
                .unwrap()
        });
        let mut round = FeedbackRound::new(None, 0).unwrap();
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
                assert!(matches!(round.finish_trial(trial), TrialOutcome::Miss(_)));
            }
        }
        assert!(round.state.misses.is_empty());
        for owner in &owners {
            assert_eq!(round.attempt_ordinal(owner), 7);
            assert!(round
                .claim_guide_trial(owner, 8, 8)
                .unwrap_err()
                .to_string()
                .contains("planning limit"));
        }
        let state = round.finish();
        let mut next = FeedbackRound::new(Some(&state), 1).unwrap();
        for owner in &owners {
            next.claim_guide_trial(owner, 8, 0).unwrap();
            assert_eq!(next.attempt_ordinal(owner), 0);
        }
    }

    #[test]
    fn text_observations_share_the_selected_payload() {
        let plan = text_plan();
        let block = crate::layout::TextBlock::from_resolved(plan.clone());
        for as_block in [false, true] {
            let mut round = FeedbackRound::new(None, 0).unwrap();
            if as_block {
                round.observe_text_block(&block).unwrap();
            } else {
                round.observe_text(&plan).unwrap();
            }
            let Observation::TextPlan(retained) = round.state.observations[0].as_ref() else {
                panic!("one shared text observation")
            };
            assert!(retained.shares_payload(&plan));
            round.observe_text(&plan).unwrap();
            round.observe_text_block(&block).unwrap();
            assert_eq!(round.state.observations.len(), 3);
            for observation in &round.state.observations {
                let Observation::TextPlan(retained) = observation.as_ref() else {
                    panic!("text observation")
                };
                assert!(retained.shares_payload(&plan));
            }
        }
    }

    #[test]
    fn effective_feedback_uses_getter_defaults_not_request_or_observation_identity() {
        let address = ComponentAddress::root();
        let mut round = FeedbackRound::new(None, 0).unwrap();
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
        let mut state = round.finish();
        assert!(!state.requests.is_empty());
        assert!(state.feedback_unchanged());
        // Explicit zeros are also equivalent to an absent consumed entry.
        state.next.push(FeedbackEntry {
            target: address.clone(),
            kind: FeedbackKind::AxisGutter,
            value: FeedbackValue::Insets([0; 4]),
        });
        assert!(state.feedback_unchanged());
        let mut round = FeedbackRound::new(Some(&state), 1).unwrap();
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
        assert!(!round.finish().feedback_unchanged());
    }

    #[test]
    fn unchanged_effective_feedback_does_not_clear_a_provisional_miss() {
        let address = ComponentAddress::root();
        let mut round = FeedbackRound::new(None, 0).unwrap();
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
        let state = round.finish();
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
        let address = ComponentAddress::root();
        let mut round = FeedbackRound::new(None, 0).unwrap();
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
        let state = round.finish();
        let mut next = FeedbackRound::new(Some(&state), 1).unwrap();
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
            .terminal_error()
            .unwrap()
            .to_string()
            .contains("no changed input"));
    }
}
