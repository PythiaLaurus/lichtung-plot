//! Operation-local parent inputs and observations for the mode-aware root driver.

use super::{ComponentAddress, LayoutScalar, SpatialRounding};
use crate::layout::fit::{FitFailure, FitScope};
pub(crate) use crate::layout::fit::{FitKind, FitReason};
use crate::layout::{Layout, PixelRect, ResolvedTextPlan};
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
    RootRequest(super::RootRequest),
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
    pub(crate) fn observe_root_request(
        &mut self,
        request: super::RootRequest,
    ) -> Result<(), Error> {
        self.observe(Observation::RootRequest(request))
    }
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

    pub(crate) fn observe_box(&mut self, panel: PixelRect) -> Result<(), Error> {
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
        self.insets_quanta(target, kind)
            .map(|value| value as f64 / 1024.0)
    }

    pub(crate) fn insets_quanta(&self, target: &ComponentAddress, kind: FeedbackKind) -> [i64; 4] {
        let index = find_index(&self.state.consumed, |entry| {
            entry.target == *target
                && entry.kind == kind
                && matches!(entry.value, FeedbackValue::Insets(_))
        });
        index.map_or([0; 4], |index| {
            let FeedbackValue::Insets(values) = self.state.consumed[index].value else {
                unreachable!("matched inset feedback")
            };
            values
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

    pub(crate) fn facet_feedback_domain(&self, kind: FeedbackKind) -> Option<(f64, f64)> {
        self.state
            .consumed
            .iter()
            .filter_map(|entry| {
                if entry.kind != kind
                    || entry.target.root.role != super::AddressRole::Root
                    || entry.target.root.slot_index != 0
                    || !matches!(
                        entry.target.descendants(),
                        [segment] if segment.role == super::AddressRole::FacetCell
                    )
                {
                    return None;
                }
                let FeedbackValue::Domain(values) = entry.value else {
                    return None;
                };
                Some((f64::from_bits(values[0]), f64::from_bits(values[1])))
            })
            .fold(None, |union, range| {
                Some(union.map_or(range, |(lower, upper): (f64, f64)| {
                    (lower.min(range.0), upper.max(range.1))
                }))
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
                a.metric_source,
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
                b.metric_source,
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
        cell: PixelRect,
        gutters: [f64; 4],
        scope: &FitScope,
    ) -> Result<crate::layout::fit::FitOutcome<PixelRect>, Error> {
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
        Ok(crate::layout::fit::FitOutcome::Measured(PixelRect::new(
            left, right, bottom, top,
        )))
    }
}
