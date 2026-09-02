//! Immutable, generation-font-backed text planning.

#![allow(dead_code)]

use std::ops::Range;
use std::str::FromStr;

use unicode_bidi::ParagraphBidiInfo;
use unicode_script::{Script, UnicodeScript};
use unicode_segmentation::UnicodeSegmentation;

use crate::font::canonical::{
    self, DesignMetrics, FaceIdentity, FamilyPreference, FamilyStackError, FontError, GlyphInk,
    PairProvenance, PreparedFontEnvironment, RequestedWeight, ResolvedFontPair,
    ScalarFaceSelection, ShapedRun as CanonicalShapedRun, ShapingDirection,
};
use crate::render::backend::Rgba;
use crate::TextWeight;

mod requirements;
#[allow(unused_imports)]
pub(crate) use requirements::{
    foundational_plot_requirements, PreparedFontPairRequirement, PreparedTextCensus,
    PreparedTextRequirement, PreparedTextRole,
};

pub(crate) const CSS_PIXEL_QUANTA: i64 = 1024;
#[cfg(test)]
thread_local! {
    static PARAGRAPH_BIDI_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PhysicalTextAlignment {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum TextLineHeight {
    Multiplier(f64),
    ResolvedQuanta(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextPlanProvenance {
    pub(crate) pair_ordinal: u32,
    pub(crate) owner: String,
}

#[derive(Debug, Clone)]
pub(crate) struct TextPlanInput<'a> {
    pub(crate) source: &'a str,
    pub(crate) stack: &'a str,
    pub(crate) weight: TextWeight,
    pub(crate) font_size: u32,
    pub(crate) line_height: TextLineHeight,
    pub(crate) maximum_width_quanta: i64,
    pub(crate) alignment: PhysicalTextAlignment,
    pub(crate) paint: Rgba,
    pub(crate) provenance: TextPlanProvenance,
    pub(crate) exact_override: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TextSourceSpan {
    pub(crate) start_byte: u32,
    pub(crate) end_byte: u32,
}

impl TextSourceSpan {
    fn new(start: usize, end: usize) -> Result<Self, TextPlanError> {
        Ok(Self {
            start_byte: u32::try_from(start).map_err(|_| TextPlanError::SourceTooLong)?,
            end_byte: u32::try_from(end).map_err(|_| TextPlanError::SourceTooLong)?,
        })
    }

    fn range(self) -> Range<usize> {
        self.start_byte as usize..self.end_byte as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TextInkBounds {
    pub(crate) left: i64,
    pub(crate) bottom: i64,
    pub(crate) right: i64,
    pub(crate) top: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlannedGlyph {
    pub(crate) face: FaceIdentity,
    pub(crate) glyph_id: u16,
    pub(crate) cluster: u32,
    pub(crate) design_x_advance: i32,
    pub(crate) design_y_advance: i32,
    pub(crate) design_x_offset: i32,
    pub(crate) design_y_offset: i32,
    pub(crate) design_ink: Option<GlyphInk>,
    pub(crate) x_advance_quanta: i64,
    pub(crate) y_advance_quanta: i64,
    pub(crate) x_offset_quanta: i64,
    pub(crate) y_offset_quanta: i64,
    pub(crate) ink_quanta: Option<TextInkBounds>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ShapedTextRun {
    pub(crate) source_span: TextSourceSpan,
    pub(crate) face: FaceIdentity,
    pub(crate) direction: ShapingDirection,
    pub(crate) script: Script,
    pub(crate) units_per_em: u16,
    pub(crate) design_ascender: i16,
    pub(crate) design_descender: i16,
    pub(crate) ascender_quanta: i64,
    pub(crate) descender_quanta: i64,
    pub(crate) design_x_advance: i64,
    pub(crate) design_y_advance: i64,
    pub(crate) logical_advance_quanta: i64,
    pub(crate) logical_y_advance_quanta: i64,
    pub(crate) ink_quanta: Option<TextInkBounds>,
    pub(crate) glyphs: Vec<PlannedGlyph>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextLinePlan {
    pub(crate) source_span: TextSourceSpan,
    pub(crate) ending_grapheme_index: u32,
    pub(crate) translation_x_quanta: i64,
    pub(crate) translation_y_quanta: i64,
    pub(crate) logical_advance_quanta: i64,
    pub(crate) ink_quanta: Option<TextInkBounds>,
    pub(crate) base_direction: ShapingDirection,
    pub(crate) runs: Vec<ShapedTextRun>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedTextStyle {
    pub(crate) stack_source: String,
    pub(crate) stack: Vec<FamilyPreference>,
    pub(crate) weight: TextWeight,
    pub(crate) font_size: u32,
    pub(crate) line_height_quanta: i64,
    pub(crate) alignment: PhysicalTextAlignment,
    pub(crate) paint: Rgba,
    pub(crate) provenance: TextPlanProvenance,
    pub(crate) exact_override: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedTextPlan {
    pub(crate) source: String,
    pub(crate) style: ResolvedTextStyle,
    pub(crate) pair_provenance: PairProvenance,
    pub(crate) lines: Vec<TextLinePlan>,
    pub(crate) line_pitch_quanta: i64,
    pub(crate) first_baseline_quanta: Option<i64>,
    pub(crate) last_baseline_quanta: Option<i64>,
    pub(crate) width_quanta: i64,
    pub(crate) height_quanta: i64,
}

impl ResolvedTextPlan {
    #[expect(
        clippy::cast_precision_loss,
        reason = "immutable plans retain exact i64 quanta; layout adapters use pixels"
    )]
    pub(crate) fn width(&self) -> f64 {
        self.width_quanta as f64 / CSS_PIXEL_QUANTA as f64
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "immutable plans retain exact i64 quanta; layout adapters use pixels"
    )]
    pub(crate) fn fit_width(&self) -> f64 {
        let rounded_advance = self
            .lines
            .iter()
            .map(|line| line.logical_advance_quanta)
            .max()
            .unwrap_or(0);
        let conservative_ceil = rounded_advance.saturating_add(i64::from(rounded_advance > 0));
        conservative_ceil as f64 / CSS_PIXEL_QUANTA as f64
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "immutable plans retain exact i64 quanta; layout adapters use pixels"
    )]
    pub(crate) fn height(&self) -> f64 {
        self.height_quanta as f64 / CSS_PIXEL_QUANTA as f64
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "immutable plans retain exact i64 quanta; layout adapters use pixels"
    )]
    pub(crate) fn line_height(&self) -> f64 {
        self.line_pitch_quanta as f64 / CSS_PIXEL_QUANTA as f64
    }

    pub(crate) fn line_texts(&self) -> Vec<&str> {
        self.lines
            .iter()
            .map(|line| &self.source[line.source_span.range()])
            .collect()
    }

    pub(crate) fn into_line_texts(self) -> Vec<String> {
        self.lines
            .iter()
            .map(|line| self.source[line.source_span.range()].to_owned())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "text bytes {source_span:?} require {required_width_quanta} quanta but only {maximum_width_quanta} are available"
)]
pub(crate) struct TextFitMiss {
    pub(crate) source_span: TextSourceSpan,
    pub(crate) maximum_width_quanta: i64,
    pub(crate) required_width_quanta: i64,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TextPlanError {
    #[error(transparent)]
    Stack(#[from] FamilyStackError),
    #[error(transparent)]
    Font(#[from] FontError),
    #[error(transparent)]
    Fit(#[from] TextFitMiss),
    #[error("font size must be positive")]
    InvalidFontSize,
    #[error("maximum text width must be nonnegative")]
    InvalidMaximumWidth,
    #[error("line height must be finite and positive")]
    InvalidLineHeight,
    #[error("text source exceeds the supported u32 byte range")]
    SourceTooLong,
    #[error("text planning arithmetic overflow while calculating {0}")]
    Arithmetic(&'static str),
    #[error("text {resource} limit of {limit} exceeded by cumulative claim {cumulative_claim}")]
    Limit {
        resource: &'static str,
        limit: u128,
        cumulative_claim: u128,
    },
    #[error("text planning could not reserve {0}")]
    Allocation(&'static str),
    #[error("cannot map Unicode script '{0}' into rustybuzz")]
    Script(String),
    #[error("resolved scalar U+{0:04X} is not backed by a rendering face")]
    InvalidScalarSelection(u32),
    #[error("prepared text census does not contain the requested font stack and weight")]
    MissingPreparedPair,
    #[error("prepared text census does not contain scalar U+{0:04X}")]
    MissingPreparedScalar(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeferredAnnotationTextPlan {
    pub(crate) plan: ResolvedTextPlan,
    pub(crate) exact_override: bool,
}

impl DeferredAnnotationTextPlan {
    pub(crate) fn from_resolved(plan: ResolvedTextPlan) -> Self {
        let exact_override = plan.style.exact_override;
        Self {
            plan,
            exact_override,
        }
    }
}

impl PreparedFontEnvironment {
    pub(crate) fn plan_text(
        &self,
        input: TextPlanInput<'_>,
    ) -> Result<ResolvedTextPlan, TextPlanError> {
        plan_text(self, input, false)
    }

    pub(crate) fn plan_prepared_text(
        &self,
        input: TextPlanInput<'_>,
    ) -> Result<ResolvedTextPlan, TextPlanError> {
        plan_text(self, input, self.prepared_pairs_are_sealed()?)
    }
}

fn plan_text(
    environment: &PreparedFontEnvironment,
    input: TextPlanInput<'_>,
    prepared_only: bool,
) -> Result<ResolvedTextPlan, TextPlanError> {
    let maximum_width_quanta = input.maximum_width_quanta;
    complete_text_alternatives(environment, input, prepared_only)?
        .into_maximum_plan(maximum_width_quanta)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawTextAttemptKind {
    Greedy(i64),
    Unwrapped,
}

#[derive(Debug)]
enum RawTextAttemptOutcome {
    Admitted(Range<usize>),
    Rejected(TextFitMiss),
}

#[derive(Debug)]
struct RawTextAttempt {
    kind: RawTextAttemptKind,
    outcome: RawTextAttemptOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SelectedTextLine {
    paragraph_index: usize,
    source_span: TextSourceSpan,
    ending_grapheme_index: u32,
}

#[derive(Debug)]
struct AdmittedTextAlternative {
    raw_attempt_index: usize,
    plan: ResolvedTextPlan,
}

#[derive(Debug)]
struct CompleteTextAlternatives {
    raw_attempts: Vec<RawTextAttempt>,
    raw_lines: Vec<SelectedTextLine>,
    admitted: Vec<AdmittedTextAlternative>,
}

impl CompleteTextAlternatives {
    fn into_maximum_plan(
        mut self,
        maximum_width_quanta: i64,
    ) -> Result<ResolvedTextPlan, TextPlanError> {
        let target_kind = if maximum_width_quanta == i64::MAX {
            // The unwrapped vector is also the i64::MAX greedy vector, without
            // introducing an unbounded endpoint into the completed width set.
            RawTextAttemptKind::Unwrapped
        } else {
            RawTextAttemptKind::Greedy(maximum_width_quanta)
        };
        let target_index = self
            .raw_attempts
            .iter()
            .position(|attempt| attempt.kind == target_kind)
            .ok_or(TextPlanError::Arithmetic("maximum-width text attempt"))?;
        let target_range = match &self.raw_attempts[target_index].outcome {
            RawTextAttemptOutcome::Admitted(range) => range.clone(),
            RawTextAttemptOutcome::Rejected(miss) => return Err(miss.clone().into()),
        };
        let selected_index = self
            .admitted
            .iter()
            .position(|alternative| {
                same_ending_vector(
                    &self.raw_lines,
                    target_range.clone(),
                    admitted_range(&self.raw_attempts, alternative.raw_attempt_index),
                )
            })
            .ok_or(TextPlanError::Arithmetic(
                "admitted maximum-width text attempt",
            ))?;
        Ok(self.admitted.swap_remove(selected_index).plan)
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "complete text planning keeps count, reservation, materialization, and freezing ordered"
)]
fn complete_text_alternatives(
    environment: &PreparedFontEnvironment,
    mut input: TextPlanInput<'_>,
    prepared_only: bool,
) -> Result<CompleteTextAlternatives, TextPlanError> {
    if input.font_size == 0 {
        return Err(TextPlanError::InvalidFontSize);
    }
    if input.maximum_width_quanta < 0 {
        return Err(TextPlanError::InvalidMaximumWidth);
    }
    let _ = u32::try_from(input.source.len()).map_err(|_| TextPlanError::SourceTooLong)?;

    let stack = canonical::parse_family_stack(input.stack)?;
    let line_height_quanta = resolve_line_height(input.line_height, input.font_size)?;
    let required_scalars = input.source.chars().collect::<Vec<_>>();
    let weight = requested_weight(input.weight);
    let prepared = environment.prepared_pair(&stack, weight)?;
    let pair = match prepared {
        Some(pair) => {
            if let Some(scalar) = required_scalars.iter().find(|scalar| {
                !pair
                    .selected_scalars
                    .iter()
                    .any(|selection| selection.scalar == **scalar)
            }) {
                if prepared_only {
                    return Err(TextPlanError::MissingPreparedScalar(*scalar as u32));
                }
                environment.resolve_pair(
                    pair.provenance.pair_ordinal,
                    &stack,
                    weight,
                    &required_scalars,
                )?
            } else {
                pair
            }
        }
        None if prepared_only => return Err(TextPlanError::MissingPreparedPair),
        None => environment.resolve_pair(
            input.provenance.pair_ordinal,
            &stack,
            weight,
            &required_scalars,
        )?,
    };
    input.provenance.pair_ordinal = pair.provenance.pair_ordinal;
    let primary_metrics = environment.face_metrics(pair.primary)?;
    let source_scalar_count = u128::try_from(required_scalars.len())
        .map_err(|_| TextPlanError::Arithmetic("text scalar count"))?;
    claim_generation_work(environment, source_scalar_count)?;
    let paragraph_seeds = mandatory_paragraph_seeds(input.source)?;
    let transitions = PreparedTextTransitions::prepare(
        environment,
        &pair,
        input.source,
        paragraph_seeds,
        input.font_size,
        input.maximum_width_quanta,
    )?;
    let (raw_attempts, raw_lines) = materialize_raw_attempts(environment, &transitions)?;
    let admitted_raw_indices = canonical_admitted_indices(environment, &raw_attempts, &raw_lines)?;

    let mut admitted = Vec::new();
    admitted
        .try_reserve_exact(admitted_raw_indices.len())
        .map_err(|_| TextPlanError::Allocation("admitted text plans"))?;
    for raw_attempt_index in admitted_raw_indices {
        let range = admitted_range(&raw_attempts, raw_attempt_index);
        let mut built_lines = Vec::new();
        built_lines
            .try_reserve_exact(range.len())
            .map_err(|_| TextPlanError::Allocation("selected text lines"))?;
        for selected in &raw_lines[range] {
            let paragraph = transitions
                .paragraphs
                .get(selected.paragraph_index)
                .ok_or(TextPlanError::Arithmetic("selected text paragraph"))?;
            let built = if input.source[paragraph.span.range()].is_empty() {
                empty_line(
                    selected.source_span,
                    selected.ending_grapheme_index,
                    primary_metrics,
                    input.font_size,
                )?
            } else {
                prepare_line(
                    environment,
                    &pair,
                    input.source,
                    paragraph.span,
                    selected.source_span,
                    selected.ending_grapheme_index,
                    input.font_size,
                    paragraph
                        .bidi
                        .as_ref()
                        .ok_or(TextPlanError::Arithmetic("paragraph bidi plan"))?,
                )?
            };
            built_lines.push(built);
        }
        admitted.push(AdmittedTextAlternative {
            raw_attempt_index,
            plan: finalize_plan(
                input.clone(),
                stack.clone(),
                pair.provenance.clone(),
                line_height_quanta,
                built_lines,
            )?,
        });
    }

    Ok(CompleteTextAlternatives {
        raw_attempts,
        raw_lines,
        admitted,
    })
}

#[derive(Debug)]
struct ParagraphTransitions {
    boundaries: Vec<usize>,
    width_range: Range<usize>,
}

impl ParagraphTransitions {
    fn source_span(
        &self,
        paragraph_span: TextSourceSpan,
        start_index: usize,
        end_index: usize,
    ) -> Result<TextSourceSpan, TextPlanError> {
        Self::source_span_from_boundaries(paragraph_span, &self.boundaries, start_index, end_index)
    }

    fn source_span_from_boundaries(
        paragraph_span: TextSourceSpan,
        boundaries: &[usize],
        start_index: usize,
        end_index: usize,
    ) -> Result<TextSourceSpan, TextPlanError> {
        let start = boundaries
            .get(start_index)
            .copied()
            .ok_or(TextPlanError::Arithmetic("text transition start"))?;
        let end = boundaries
            .get(end_index)
            .copied()
            .ok_or(TextPlanError::Arithmetic("text transition end"))?;
        TextSourceSpan::new(
            (paragraph_span.start_byte as usize)
                .checked_add(start)
                .ok_or(TextPlanError::Arithmetic("text transition start"))?,
            (paragraph_span.start_byte as usize)
                .checked_add(end)
                .ok_or(TextPlanError::Arithmetic("text transition end"))?,
        )
    }

    fn width(
        &self,
        widths_quanta: &[i64],
        start_index: usize,
        end_index: usize,
    ) -> Result<i64, TextPlanError> {
        let index = transition_index(self.boundaries.len(), start_index, end_index)?;
        let index = self
            .width_range
            .start
            .checked_add(index)
            .ok_or(TextPlanError::Arithmetic("text transition width"))?;
        widths_quanta
            .get(index)
            .copied()
            .ok_or(TextPlanError::Arithmetic("text transition width"))
    }

    fn furthest_fitting_end(
        &self,
        widths_quanta: &[i64],
        start_index: usize,
        maximum_width_quanta: i64,
    ) -> Result<Option<usize>, TextPlanError> {
        let mut selected = None;
        for end_index in start_index + 1..self.boundaries.len() {
            if self.width(widths_quanta, start_index, end_index)? <= maximum_width_quanta {
                selected = Some(end_index);
            }
        }
        Ok(selected)
    }
}

#[derive(Debug, Clone, Copy)]
struct ParagraphSeed {
    span: TextSourceSpan,
    scalar_count: u128,
}

struct PreparedParagraph<'a> {
    span: TextSourceSpan,
    grapheme_start: usize,
    grapheme_ends: Vec<usize>,
    transitions: ParagraphTransitions,
    bidi: Option<ParagraphBidiInfo<'a>>,
}

struct PreparedTextTransitions<'a> {
    paragraphs: Vec<PreparedParagraph<'a>>,
    widths_quanta: Vec<i64>,
    completed_widths_quanta: Vec<i64>,
    rendered_scalar_count: u128,
}

impl<'a> PreparedTextTransitions<'a> {
    #[expect(
        clippy::too_many_lines,
        reason = "transition preparation keeps count, reservation, shaping, and width completion ordered"
    )]
    fn prepare(
        environment: &PreparedFontEnvironment,
        pair: &ResolvedFontPair,
        source: &'a str,
        paragraph_seeds: Vec<ParagraphSeed>,
        font_size: u32,
        maximum_width_quanta: i64,
    ) -> Result<Self, TextPlanError> {
        let mut transition_count = 0_u128;
        let paragraph_count = u128::try_from(paragraph_seeds.len())
            .map_err(|_| TextPlanError::Arithmetic("text paragraph slots"))?;
        let mut transition_storage_slots = paragraph_count
            .checked_add(2)
            .ok_or(TextPlanError::Arithmetic("text transition slots"))?;
        for seed in &paragraph_seeds {
            let paragraph = &source[seed.span.range()];
            claim_generation_work(
                environment,
                checked_multiply(seed.scalar_count, 3, "text transition count passes")?,
            )?;
            let grapheme_count = paragraph.graphemes(true).count();
            let boundary_count = if paragraph.is_empty() {
                1
            } else {
                legal_line_breaks(paragraph)
                    .len()
                    .checked_add(1)
                    .ok_or(TextPlanError::Arithmetic("text transition boundaries"))?
            };
            let grapheme_slots = u128::try_from(grapheme_count)
                .map_err(|_| TextPlanError::Arithmetic("text grapheme slots"))?
                .checked_add(1)
                .ok_or(TextPlanError::Arithmetic("text grapheme slots"))?;
            let boundary_slots = u128::try_from(boundary_count)
                .map_err(|_| TextPlanError::Arithmetic("text transition slots"))?
                .checked_add(1)
                .ok_or(TextPlanError::Arithmetic("text transition slots"))?;
            transition_storage_slots = transition_storage_slots
                .checked_add(grapheme_slots)
                .and_then(|slots| slots.checked_add(boundary_slots))
                .ok_or(TextPlanError::Arithmetic("text transition slots"))?;
            if !paragraph.is_empty() {
                transition_storage_slots = transition_storage_slots
                    .checked_add(boundary_slots)
                    .ok_or(TextPlanError::Arithmetic("text transition slots"))?;
                transition_count = transition_count
                    .checked_add(checked_transition_count(boundary_count)?)
                    .ok_or(TextPlanError::Arithmetic("text transition count"))?;
            }
        }
        transition_storage_slots = transition_storage_slots
            .checked_add(transition_count)
            .ok_or(TextPlanError::Arithmetic("text transition slots"))?;

        claim_collection_count(environment, transition_storage_slots)?;
        let transition_count = usize::try_from(transition_count)
            .map_err(|_| TextPlanError::Arithmetic("text transition count"))?;
        let mut widths_quanta = Vec::new();
        widths_quanta
            .try_reserve_exact(transition_count)
            .map_err(|_| TextPlanError::Allocation("text transition widths"))?;

        let finite_maximum_count = u128::from(maximum_width_quanta != i64::MAX);
        let completed_width_capacity = u128::try_from(transition_count)
            .map_err(|_| TextPlanError::Arithmetic("completed text widths"))?
            .checked_add(3)
            .and_then(|count| count.checked_add(finite_maximum_count))
            .ok_or(TextPlanError::Arithmetic("completed text widths"))?;
        let completed_width_slots = completed_width_capacity
            .checked_add(1)
            .ok_or(TextPlanError::Arithmetic("completed text width slots"))?;
        claim_collection_count(environment, completed_width_slots)?;
        let mut completed_widths_quanta = Vec::new();
        completed_widths_quanta
            .try_reserve_exact(
                usize::try_from(completed_width_capacity)
                    .map_err(|_| TextPlanError::Arithmetic("completed text widths"))?,
            )
            .map_err(|_| TextPlanError::Allocation("completed text widths"))?;

        let mut paragraphs = Vec::new();
        paragraphs
            .try_reserve_exact(paragraph_seeds.len())
            .map_err(|_| TextPlanError::Allocation("text paragraphs"))?;
        let mut grapheme_start = 0_usize;
        let mut transition_cursor = 0_usize;
        let mut transition_scalar_work = 0_u128;
        let mut rendered_scalar_count = 0_u128;
        claim_collection_materialization(environment, transition_storage_slots)?;
        for seed in paragraph_seeds {
            rendered_scalar_count = rendered_scalar_count
                .checked_add(seed.scalar_count)
                .ok_or(TextPlanError::Arithmetic("rendered text scalar count"))?;
            let paragraph = &source[seed.span.range()];
            if paragraph.is_empty() {
                let mut boundaries = Vec::new();
                boundaries
                    .try_reserve_exact(1)
                    .map_err(|_| TextPlanError::Allocation("text transition boundaries"))?;
                boundaries.push(0);
                paragraphs.push(PreparedParagraph {
                    span: seed.span,
                    grapheme_start,
                    grapheme_ends: Vec::new(),
                    transitions: ParagraphTransitions {
                        boundaries,
                        width_range: transition_cursor..transition_cursor,
                    },
                    bidi: None,
                });
                continue;
            }

            claim_generation_work(
                environment,
                checked_multiply(
                    seed.scalar_count,
                    3,
                    "text transition materialization passes",
                )?,
            )?;
            let grapheme_count = paragraph.graphemes(true).count();
            let mut grapheme_ends = Vec::new();
            grapheme_ends
                .try_reserve_exact(grapheme_count)
                .map_err(|_| TextPlanError::Allocation("text grapheme boundaries"))?;
            grapheme_ends.extend(
                paragraph
                    .grapheme_indices(true)
                    .map(|(start, grapheme)| start + grapheme.len()),
            );
            let legal_boundaries = legal_line_breaks_at_graphemes(paragraph, &grapheme_ends);
            let boundary_count = legal_boundaries
                .len()
                .checked_add(1)
                .ok_or(TextPlanError::Arithmetic("text transition boundaries"))?;
            let paragraph_transition_count =
                usize::try_from(checked_transition_count(boundary_count)?)
                    .map_err(|_| TextPlanError::Arithmetic("text transition count"))?;
            let transition_end = transition_cursor
                .checked_add(paragraph_transition_count)
                .ok_or(TextPlanError::Arithmetic("text transition range"))?;
            let mut boundaries = Vec::new();
            boundaries
                .try_reserve_exact(boundary_count)
                .map_err(|_| TextPlanError::Allocation("text transition boundaries"))?;
            boundaries.push(0);
            boundaries.extend(legal_boundaries);

            claim_generation_work(environment, seed.scalar_count)?;
            let scalar_prefixes = scalar_prefixes_at_boundaries(paragraph, &boundaries)?;
            transition_scalar_work = transition_scalar_work
                .checked_add(transition_scalar_work_for_prefixes(&scalar_prefixes)?)
                .ok_or(TextPlanError::Arithmetic("text transition scalar work"))?;
            claim_generation_work(environment, seed.scalar_count)?;
            #[cfg(test)]
            PARAGRAPH_BIDI_CALLS.with(|calls| calls.set(calls.get() + 1));
            let bidi = ParagraphBidiInfo::new(paragraph, None);
            let next_grapheme_start = grapheme_start
                .checked_add(grapheme_ends.len())
                .ok_or(TextPlanError::Arithmetic("grapheme index"))?;
            paragraphs.push(PreparedParagraph {
                span: seed.span,
                grapheme_start,
                grapheme_ends,
                transitions: ParagraphTransitions {
                    boundaries,
                    width_range: transition_cursor..transition_end,
                },
                bidi: Some(bidi),
            });
            grapheme_start = next_grapheme_start;
            transition_cursor = transition_end;
        }
        debug_assert_eq!(transition_cursor, transition_count);

        claim_collection_materialization(environment, completed_width_slots)?;
        claim_generation_work(environment, transition_scalar_work)?;

        let mut hard_minimum_quanta = 0_i64;
        let mut natural_unwrapped_width_quanta = 0_i64;
        for paragraph in &paragraphs {
            let Some(bidi) = &paragraph.bidi else {
                continue;
            };
            for start_index in 0..paragraph.transitions.boundaries.len() - 1 {
                for end_index in start_index + 1..paragraph.transitions.boundaries.len() {
                    let line_span = paragraph.transitions.source_span(
                        paragraph.span,
                        start_index,
                        end_index,
                    )?;
                    let measured = transition_measurement(
                        environment,
                        pair,
                        source,
                        paragraph.span,
                        line_span,
                        font_size,
                        bidi,
                    )?;
                    widths_quanta.push(measured.width_quanta);
                    completed_widths_quanta.push(measured.width_quanta);
                    if end_index == start_index + 1 {
                        hard_minimum_quanta = hard_minimum_quanta.max(measured.ink_span_quanta);
                    }
                    if start_index == 0 && end_index + 1 == paragraph.transitions.boundaries.len() {
                        natural_unwrapped_width_quanta =
                            natural_unwrapped_width_quanta.max(measured.ink_span_quanta);
                    }
                }
            }
        }
        debug_assert_eq!(widths_quanta.len(), transition_count);

        claim_sort_and_dedup_work(environment, completed_widths_quanta.len())?;
        completed_widths_quanta.sort_unstable();
        completed_widths_quanta.dedup();
        completed_widths_quanta.push(0);
        completed_widths_quanta.push(hard_minimum_quanta);
        completed_widths_quanta.push(natural_unwrapped_width_quanta);
        if maximum_width_quanta != i64::MAX {
            completed_widths_quanta.push(maximum_width_quanta);
        }
        debug_assert!(
            completed_widths_quanta.len()
                <= usize::try_from(completed_width_capacity).unwrap_or(usize::MAX)
        );
        claim_generation_work(
            environment,
            u128::try_from(completed_widths_quanta.len())
                .map_err(|_| TextPlanError::Arithmetic("completed text width filtering"))?,
        )?;
        completed_widths_quanta.retain(|width| *width >= 0 && *width <= maximum_width_quanta);
        claim_sort_and_dedup_work(environment, completed_widths_quanta.len())?;
        completed_widths_quanta.sort_unstable();
        completed_widths_quanta.dedup();

        Ok(Self {
            paragraphs,
            widths_quanta,
            completed_widths_quanta,
            rendered_scalar_count,
        })
    }
}

fn materialize_raw_attempts(
    environment: &PreparedFontEnvironment,
    transitions: &PreparedTextTransitions<'_>,
) -> Result<(Vec<RawTextAttempt>, Vec<SelectedTextLine>), TextPlanError> {
    let raw_attempt_count = transitions
        .completed_widths_quanta
        .len()
        .checked_add(1)
        .ok_or(TextPlanError::Arithmetic("raw text attempt count"))?;
    let raw_attempt_count_u128 = u128::try_from(raw_attempt_count)
        .map_err(|_| TextPlanError::Arithmetic("raw text attempt count"))?;
    let wrapping_work = transitions
        .rendered_scalar_count
        .checked_mul(raw_attempt_count_u128)
        .ok_or(TextPlanError::Arithmetic("text wrapping work"))?;

    claim_generation_work(environment, wrapping_work)?;
    let mut admitted_line_count = u128::try_from(transitions.paragraphs.len())
        .map_err(|_| TextPlanError::Arithmetic("unwrapped text line count"))?;
    for &width in &transitions.completed_widths_quanta {
        let mut line_count = 0_u128;
        let miss = greedy_text_attempt(transitions, width, |_| {
            line_count = line_count
                .checked_add(1)
                .ok_or(TextPlanError::Arithmetic("text alternative line count"))?;
            Ok(())
        })?;
        if miss.is_none() {
            admitted_line_count = admitted_line_count
                .checked_add(line_count)
                .ok_or(TextPlanError::Arithmetic("text alternative line count"))?;
        }
    }
    let raw_slots = raw_attempt_count_u128
        .checked_add(admitted_line_count)
        .ok_or(TextPlanError::Arithmetic("raw text alternative slots"))?;
    claim_collection_count(environment, raw_slots)?;

    let mut raw_attempts = Vec::new();
    raw_attempts
        .try_reserve_exact(raw_attempt_count)
        .map_err(|_| TextPlanError::Allocation("raw text attempts"))?;
    let admitted_line_count = usize::try_from(admitted_line_count)
        .map_err(|_| TextPlanError::Arithmetic("text alternative line count"))?;
    let mut raw_lines = Vec::new();
    raw_lines
        .try_reserve_exact(admitted_line_count)
        .map_err(|_| TextPlanError::Allocation("raw text alternative lines"))?;
    claim_collection_materialization(environment, raw_slots)?;
    claim_generation_work(environment, wrapping_work)?;

    let minimum_wrap_width = minimum_wrap_width(transitions)?;
    for &width in &transitions.completed_widths_quanta {
        let line_start = raw_lines.len();
        let miss = if width >= minimum_wrap_width {
            greedy_text_attempt(transitions, width, |line| {
                raw_lines.push(line);
                Ok(())
            })?
        } else {
            greedy_text_attempt(transitions, width, |_| Ok(()))?
        };
        let outcome = match miss {
            Some(miss) if width < minimum_wrap_width => RawTextAttemptOutcome::Rejected(miss),
            None if width >= minimum_wrap_width => {
                RawTextAttemptOutcome::Admitted(line_start..raw_lines.len())
            }
            _ => {
                return Err(TextPlanError::Arithmetic(
                    "text attempt count/materialization",
                ))
            }
        };
        raw_attempts.push(RawTextAttempt {
            kind: RawTextAttemptKind::Greedy(width),
            outcome,
        });
    }

    let line_start = raw_lines.len();
    for paragraph_index in 0..transitions.paragraphs.len() {
        raw_lines.push(full_paragraph_line(transitions, paragraph_index)?);
    }
    raw_attempts.push(RawTextAttempt {
        kind: RawTextAttemptKind::Unwrapped,
        outcome: RawTextAttemptOutcome::Admitted(line_start..raw_lines.len()),
    });
    debug_assert_eq!(raw_attempts.len(), raw_attempt_count);
    debug_assert_eq!(raw_lines.len(), admitted_line_count);
    Ok((raw_attempts, raw_lines))
}

fn greedy_text_attempt(
    transitions: &PreparedTextTransitions<'_>,
    maximum_width_quanta: i64,
    mut emit: impl FnMut(SelectedTextLine) -> Result<(), TextPlanError>,
) -> Result<Option<TextFitMiss>, TextPlanError> {
    for (paragraph_index, paragraph) in transitions.paragraphs.iter().enumerate() {
        if paragraph.bidi.is_none() {
            emit(full_paragraph_line(transitions, paragraph_index)?)?;
            continue;
        }

        let mut start_index = 0_usize;
        while start_index + 1 < paragraph.transitions.boundaries.len() {
            let required_width_quanta = paragraph.transitions.width(
                &transitions.widths_quanta,
                start_index,
                start_index + 1,
            )?;
            if required_width_quanta > maximum_width_quanta {
                return Ok(Some(TextFitMiss {
                    source_span: paragraph.transitions.source_span(
                        paragraph.span,
                        start_index,
                        start_index + 1,
                    )?,
                    maximum_width_quanta,
                    required_width_quanta,
                }));
            }
            let Some(end_index) = paragraph.transitions.furthest_fitting_end(
                &transitions.widths_quanta,
                start_index,
                maximum_width_quanta,
            )?
            else {
                return Err(TextPlanError::Arithmetic(
                    "greedy text transition selection",
                ));
            };
            let end = paragraph.transitions.boundaries[end_index];
            emit(SelectedTextLine {
                paragraph_index,
                source_span: paragraph.transitions.source_span(
                    paragraph.span,
                    start_index,
                    end_index,
                )?,
                ending_grapheme_index: ending_grapheme_index(
                    &paragraph.grapheme_ends,
                    end,
                    paragraph.grapheme_start,
                )?,
            })?;
            start_index = end_index;
        }
    }
    Ok(None)
}

fn full_paragraph_line(
    transitions: &PreparedTextTransitions<'_>,
    paragraph_index: usize,
) -> Result<SelectedTextLine, TextPlanError> {
    let paragraph = transitions
        .paragraphs
        .get(paragraph_index)
        .ok_or(TextPlanError::Arithmetic("unwrapped text paragraph"))?;
    let ending_grapheme_index = paragraph
        .grapheme_start
        .checked_add(paragraph.grapheme_ends.len())
        .ok_or(TextPlanError::Arithmetic("ending grapheme index"))?;
    Ok(SelectedTextLine {
        paragraph_index,
        source_span: paragraph.span,
        ending_grapheme_index: u32::try_from(ending_grapheme_index)
            .map_err(|_| TextPlanError::Arithmetic("ending grapheme index"))?,
    })
}

fn minimum_wrap_width(transitions: &PreparedTextTransitions<'_>) -> Result<i64, TextPlanError> {
    let mut minimum = 0_i64;
    for paragraph in &transitions.paragraphs {
        for start_index in 0..paragraph.transitions.boundaries.len().saturating_sub(1) {
            minimum = minimum.max(paragraph.transitions.width(
                &transitions.widths_quanta,
                start_index,
                start_index + 1,
            )?);
        }
    }
    Ok(minimum)
}

fn canonical_admitted_indices(
    environment: &PreparedFontEnvironment,
    raw_attempts: &[RawTextAttempt],
    raw_lines: &[SelectedTextLine],
) -> Result<Vec<usize>, TextPlanError> {
    claim_generation_work(
        environment,
        u128::try_from(raw_attempts.len())
            .map_err(|_| TextPlanError::Arithmetic("admitted text count"))?,
    )?;
    let admitted_count = raw_attempts
        .iter()
        .filter(|attempt| matches!(attempt.outcome, RawTextAttemptOutcome::Admitted(_)))
        .count();
    let admitted_count_u128 = u128::try_from(admitted_count)
        .map_err(|_| TextPlanError::Arithmetic("admitted text count"))?;
    let admitted_slots = admitted_count_u128
        .checked_add(1)
        .ok_or(TextPlanError::Arithmetic("admitted text slots"))?;
    claim_collection_count(environment, admitted_slots)?;
    let mut admitted = Vec::new();
    admitted
        .try_reserve_exact(admitted_count)
        .map_err(|_| TextPlanError::Allocation("admitted text alternatives"))?;
    claim_collection_materialization(environment, admitted_slots)?;
    admitted.extend(
        raw_attempts
            .iter()
            .enumerate()
            .filter_map(|(index, attempt)| {
                matches!(attempt.outcome, RawTextAttemptOutcome::Admitted(_)).then_some(index)
            }),
    );

    claim_sort_and_dedup_work(environment, admitted.len())?;
    admitted.sort_unstable_by(|left, right| {
        let left_range = admitted_range(raw_attempts, *left);
        let right_range = admitted_range(raw_attempts, *right);
        left_range
            .len()
            .cmp(&right_range.len())
            .then_with(|| ending_vector_cmp(raw_lines, left_range, right_range))
            .then_with(|| left.cmp(right))
    });
    admitted.dedup_by(|left, right| {
        same_ending_vector(
            raw_lines,
            admitted_range(raw_attempts, *left),
            admitted_range(raw_attempts, *right),
        )
    });
    Ok(admitted)
}

fn admitted_range(raw_attempts: &[RawTextAttempt], index: usize) -> Range<usize> {
    match &raw_attempts[index].outcome {
        RawTextAttemptOutcome::Admitted(range) => range.clone(),
        RawTextAttemptOutcome::Rejected(_) => {
            unreachable!("only admitted raw text attempts enter canonical ordering")
        }
    }
}

fn ending_vector_cmp(
    raw_lines: &[SelectedTextLine],
    left: Range<usize>,
    right: Range<usize>,
) -> std::cmp::Ordering {
    raw_lines[left]
        .iter()
        .map(|line| line.ending_grapheme_index)
        .cmp(
            raw_lines[right]
                .iter()
                .map(|line| line.ending_grapheme_index),
        )
}

fn same_ending_vector(
    raw_lines: &[SelectedTextLine],
    left: Range<usize>,
    right: Range<usize>,
) -> bool {
    left.len() == right.len()
        && raw_lines[left]
            .iter()
            .zip(&raw_lines[right])
            .all(|(left, right)| left.ending_grapheme_index == right.ending_grapheme_index)
}

fn claim_generation_work(
    environment: &PreparedFontEnvironment,
    amount: u128,
) -> Result<(), TextPlanError> {
    claim_text_generation(environment.claim_text_generation_work(amount))
}

fn claim_collection_count(
    environment: &PreparedFontEnvironment,
    slots: u128,
) -> Result<(), TextPlanError> {
    claim_generation_work(environment, slots)?;
    claim_text_generation(environment.claim_text_generation_slots(slots))
}

fn claim_collection_materialization(
    environment: &PreparedFontEnvironment,
    slots: u128,
) -> Result<(), TextPlanError> {
    claim_generation_work(environment, slots)
}

fn claim_sort_and_dedup_work(
    environment: &PreparedFontEnvironment,
    count: usize,
) -> Result<(), TextPlanError> {
    let count = u128::try_from(count)
        .map_err(|_| TextPlanError::Arithmetic("text sort and deduplication work"))?;
    claim_generation_work(
        environment,
        checked_multiply(count, 2, "text sort and deduplication work")?,
    )
}

fn checked_multiply(
    value: u128,
    factor: u128,
    resource: &'static str,
) -> Result<u128, TextPlanError> {
    value
        .checked_mul(factor)
        .ok_or(TextPlanError::Arithmetic(resource))
}

fn claim_text_generation(
    claim: Result<(), canonical::TextGenerationClaimError>,
) -> Result<(), TextPlanError> {
    match claim {
        Ok(()) => Ok(()),
        Err(canonical::TextGenerationClaimError::Arithmetic(resource)) => {
            Err(TextPlanError::Arithmetic(resource))
        }
        Err(canonical::TextGenerationClaimError::Limit {
            resource,
            limit,
            cumulative_claim,
        }) => Err(TextPlanError::Limit {
            resource,
            limit,
            cumulative_claim,
        }),
    }
}

fn checked_transition_count(boundary_count: usize) -> Result<u128, TextPlanError> {
    let count = u128::try_from(boundary_count)
        .map_err(|_| TextPlanError::Arithmetic("text transition count"))?;
    count
        .checked_mul(
            count
                .checked_sub(1)
                .ok_or(TextPlanError::Arithmetic("text transition count"))?,
        )
        .and_then(|value| value.checked_div(2))
        .ok_or(TextPlanError::Arithmetic("text transition count"))
}

fn transition_index(
    boundary_count: usize,
    start_index: usize,
    end_index: usize,
) -> Result<usize, TextPlanError> {
    if start_index >= end_index || end_index >= boundary_count {
        return Err(TextPlanError::Arithmetic("text transition index"));
    }
    let count = u128::try_from(boundary_count)
        .map_err(|_| TextPlanError::Arithmetic("text transition index"))?;
    let start = u128::try_from(start_index)
        .map_err(|_| TextPlanError::Arithmetic("text transition index"))?;
    let end = u128::try_from(end_index)
        .map_err(|_| TextPlanError::Arithmetic("text transition index"))?;
    let row_offset = start
        .checked_mul(
            count
                .checked_mul(2)
                .and_then(|value| value.checked_sub(start))
                .and_then(|value| value.checked_sub(1))
                .ok_or(TextPlanError::Arithmetic("text transition index"))?,
        )
        .and_then(|value| value.checked_div(2))
        .ok_or(TextPlanError::Arithmetic("text transition index"))?;
    usize::try_from(
        row_offset
            .checked_add(end - start - 1)
            .ok_or(TextPlanError::Arithmetic("text transition index"))?,
    )
    .map_err(|_| TextPlanError::Arithmetic("text transition index"))
}

fn scalar_prefixes_at_boundaries(
    paragraph: &str,
    boundaries: &[usize],
) -> Result<Vec<u128>, TextPlanError> {
    let mut prefixes = Vec::new();
    prefixes
        .try_reserve_exact(boundaries.len())
        .map_err(|_| TextPlanError::Allocation("text transition scalar prefixes"))?;
    let mut scalars = paragraph.char_indices().peekable();
    let mut count = 0_u128;
    for &boundary in boundaries {
        while scalars.peek().is_some_and(|(index, _)| *index < boundary) {
            scalars.next();
            count = count
                .checked_add(1)
                .ok_or(TextPlanError::Arithmetic("text transition scalar work"))?;
        }
        prefixes.push(count);
    }
    Ok(prefixes)
}

fn transition_scalar_work_for_prefixes(prefixes: &[u128]) -> Result<u128, TextPlanError> {
    let mut work = 0_u128;
    for start_index in 0..prefixes.len().saturating_sub(1) {
        for end_index in start_index + 1..prefixes.len() {
            work = work
                .checked_add(
                    prefixes[end_index]
                        .checked_sub(prefixes[start_index])
                        .ok_or(TextPlanError::Arithmetic("text transition scalar work"))?,
                )
                .ok_or(TextPlanError::Arithmetic("text transition scalar work"))?;
        }
    }
    Ok(work)
}

fn requested_weight(weight: TextWeight) -> RequestedWeight {
    match weight {
        TextWeight::Normal => RequestedWeight::Normal,
        TextWeight::Semibold => RequestedWeight::Semibold,
        TextWeight::Bold => RequestedWeight::Bold,
    }
}

fn mandatory_paragraphs(source: &str) -> Result<Vec<TextSourceSpan>, TextPlanError> {
    Ok(mandatory_paragraph_seeds(source)?
        .into_iter()
        .map(|paragraph| paragraph.span)
        .collect())
}

fn mandatory_paragraph_seeds(source: &str) -> Result<Vec<ParagraphSeed>, TextPlanError> {
    let mut paragraphs = Vec::new();
    let mut start = 0_usize;
    let mut scalar_count = 0_u128;
    let mut scalars = source.char_indices().peekable();
    while let Some((index, scalar)) = scalars.next() {
        if !canonical::is_nonrendering_mandatory_break(scalar) {
            scalar_count = scalar_count
                .checked_add(1)
                .ok_or(TextPlanError::Arithmetic("paragraph scalar count"))?;
            continue;
        }
        paragraphs.push(ParagraphSeed {
            span: TextSourceSpan::new(start, index)?,
            scalar_count,
        });
        let mut break_end = index + scalar.len_utf8();
        if scalar == '\r' && scalars.peek().is_some_and(|(_, next)| *next == '\n') {
            let (lf_index, lf) = scalars
                .next()
                .ok_or(TextPlanError::Arithmetic("CRLF segmentation"))?;
            break_end = lf_index + lf.len_utf8();
        }
        start = break_end;
        scalar_count = 0;
    }
    paragraphs.push(ParagraphSeed {
        span: TextSourceSpan::new(start, source.len())?,
        scalar_count,
    });
    Ok(paragraphs)
}

/// Exact Unicode 16 UAX #14 opportunities, restricted to Unicode 16 UAX #29
/// grapheme boundaries before transition shaping.
fn legal_line_breaks(paragraph: &str) -> Vec<usize> {
    let grapheme_ends = paragraph
        .grapheme_indices(true)
        .map(|(start, grapheme)| start + grapheme.len())
        .collect::<Vec<_>>();
    legal_line_breaks_at_graphemes(paragraph, &grapheme_ends)
}

fn legal_line_breaks_at_graphemes(paragraph: &str, grapheme_ends: &[usize]) -> Vec<usize> {
    if paragraph.is_empty() {
        return Vec::new();
    }
    let mut breaks = super::unicode16::linebreaks(paragraph)
        .into_iter()
        .filter_map(|(index, _)| {
            (index > 0 && grapheme_ends.binary_search(&index).is_ok()).then_some(index)
        })
        .collect::<Vec<_>>();
    if breaks.last().copied() != Some(paragraph.len()) {
        breaks.push(paragraph.len());
    }
    breaks.sort_unstable();
    breaks.dedup();
    breaks
}

fn ending_grapheme_index(
    grapheme_ends: &[usize],
    end: usize,
    paragraph_start_index: usize,
) -> Result<u32, TextPlanError> {
    let local = grapheme_ends
        .binary_search(&end)
        .map_err(|_| TextPlanError::Arithmetic("ending grapheme boundary"))?
        .checked_add(1)
        .ok_or(TextPlanError::Arithmetic("ending grapheme index"))?;
    u32::try_from(
        paragraph_start_index
            .checked_add(local)
            .ok_or(TextPlanError::Arithmetic("ending grapheme index"))?,
    )
    .map_err(|_| TextPlanError::Arithmetic("ending grapheme index"))
}

#[derive(Debug)]
struct BuiltLine {
    plan: TextLinePlan,
    advance: ExactRational,
    ascent: i64,
    descent: i64,
}

fn empty_line(
    span: TextSourceSpan,
    ending_grapheme_index: u32,
    primary_metrics: DesignMetrics,
    font_size: u32,
) -> Result<BuiltLine, TextPlanError> {
    let (ascent, descent) = scaled_metrics(primary_metrics, font_size)?;
    let descent = descent
        .checked_neg()
        .ok_or(TextPlanError::Arithmetic("primary face descender"))?;
    Ok(BuiltLine {
        plan: TextLinePlan {
            source_span: span,
            ending_grapheme_index,
            translation_x_quanta: 0,
            translation_y_quanta: 0,
            logical_advance_quanta: 0,
            ink_quanta: None,
            base_direction: ShapingDirection::LeftToRight,
            runs: Vec::new(),
        },
        advance: ExactRational::ZERO,
        ascent,
        descent,
    })
}

#[expect(
    clippy::too_many_arguments,
    reason = "selected-line shaping retains exact paragraph and line identities"
)]
fn prepare_line(
    environment: &PreparedFontEnvironment,
    pair: &ResolvedFontPair,
    source: &str,
    paragraph_span: TextSourceSpan,
    line_span: TextSourceSpan,
    ending_grapheme_index: u32,
    font_size: u32,
    bidi: &ParagraphBidiInfo<'_>,
) -> Result<BuiltLine, TextPlanError> {
    let paragraph = &source[paragraph_span.range()];
    let line_local_start = (line_span.start_byte - paragraph_span.start_byte) as usize;
    let line_local_end = (line_span.end_byte - paragraph_span.start_byte) as usize;
    let line_scalar_count = u128::try_from(source[line_span.range()].chars().count())
        .map_err(|_| TextPlanError::Arithmetic("selected line scalar work"))?;
    claim_generation_work(
        environment,
        checked_multiply(line_scalar_count, 2, "selected line scalar work")?,
    )?;
    let (levels, bidi_runs) = bidi.visual_runs(line_local_start..line_local_end);

    let mut runs = Vec::new();
    let mut exact_ink: Option<ExactInk> = None;
    let mut pen_x = ExactRational::ZERO;
    let mut pen_y = ExactRational::ZERO;
    let mut ascent = 0_i64;
    let mut descent = 0_i64;

    for bidi_run in bidi_runs {
        let direction = if levels[bidi_run.start].is_rtl() {
            ShapingDirection::RightToLeft
        } else {
            ShapingDirection::LeftToRight
        };
        let mut groups = itemize_bidi_run(pair, paragraph, paragraph_span.start_byte, bidi_run)?;
        if direction == ShapingDirection::RightToLeft {
            groups.reverse();
        }
        for group in groups {
            let text = &source[group.span.range()];
            let shaped = environment.shape_run(
                group.face,
                text,
                direction,
                rustybuzz_script(group.script)?,
            )?;
            claim_generation_work(
                environment,
                u128::try_from(shaped.glyphs.len())
                    .map_err(|_| TextPlanError::Arithmetic("selected line glyph work"))?,
            )?;
            let built = scale_run(shaped, group.span, group.script, font_size)?;
            ascent = ascent.max(built.plan.ascender_quanta);
            descent = descent.max(
                built
                    .plan
                    .descender_quanta
                    .checked_neg()
                    .ok_or(TextPlanError::Arithmetic("face descender"))?,
            );
            if let Some(ink) = built.ink {
                exact_ink = Some(match exact_ink {
                    Some(current) => current.union(ink.translated(pen_x, pen_y)?)?,
                    None => ink.translated(pen_x, pen_y)?,
                });
            }
            pen_x = pen_x.add(built.advance_x, "line advance")?;
            pen_y = pen_y.add(built.advance_y, "line advance")?;
            runs.push(built.plan);
        }
    }

    let ink_quanta = exact_ink.map(ExactInk::outward).transpose()?;
    if let Some(ink) = ink_quanta {
        ascent = ascent.max(ink.top.max(0));
        descent = descent.max(
            ink.bottom
                .min(0)
                .checked_neg()
                .ok_or(TextPlanError::Arithmetic("line ink below baseline"))?,
        );
    }
    let logical_advance_quanta = pen_x.nearest_even_i64("line logical advance")?;
    if logical_advance_quanta < 0 {
        return Err(TextPlanError::Arithmetic("negative line logical advance"));
    }

    Ok(BuiltLine {
        plan: TextLinePlan {
            source_span: line_span,
            ending_grapheme_index,
            translation_x_quanta: 0,
            translation_y_quanta: 0,
            logical_advance_quanta,
            ink_quanta,
            base_direction: if bidi.paragraph_level.is_rtl() {
                ShapingDirection::RightToLeft
            } else {
                ShapingDirection::LeftToRight
            },
            runs,
        },
        advance: pen_x,
        ascent,
        descent,
    })
}

#[derive(Debug, Clone, Copy)]
struct TransitionMeasurement {
    width_quanta: i64,
    ink_span_quanta: i64,
}

fn transition_measurement(
    environment: &PreparedFontEnvironment,
    pair: &ResolvedFontPair,
    source: &str,
    paragraph_span: TextSourceSpan,
    line_span: TextSourceSpan,
    font_size: u32,
    bidi: &ParagraphBidiInfo<'_>,
) -> Result<TransitionMeasurement, TextPlanError> {
    let paragraph = &source[paragraph_span.range()];
    let line_local_start = (line_span.start_byte - paragraph_span.start_byte) as usize;
    let line_local_end = (line_span.end_byte - paragraph_span.start_byte) as usize;
    let (levels, bidi_runs) = bidi.visual_runs(line_local_start..line_local_end);
    let mut pen_x = ExactRational::ZERO;
    let mut pen_y = ExactRational::ZERO;
    let mut exact_ink: Option<ExactInk> = None;

    for bidi_run in bidi_runs {
        let direction = if levels[bidi_run.start].is_rtl() {
            ShapingDirection::RightToLeft
        } else {
            ShapingDirection::LeftToRight
        };
        let mut groups = itemize_bidi_run(pair, paragraph, paragraph_span.start_byte, bidi_run)?;
        if direction == ShapingDirection::RightToLeft {
            groups.reverse();
        }
        for group in groups {
            let shaped = environment.shape_run(
                group.face,
                &source[group.span.range()],
                direction,
                rustybuzz_script(group.script)?,
            )?;
            claim_generation_work(
                environment,
                u128::try_from(shaped.glyphs.len())
                    .map_err(|_| TextPlanError::Arithmetic("text transition glyph work"))?,
            )?;
            for glyph in shaped.glyphs {
                if let Some(ink) = glyph.ink {
                    let ink = exact_glyph_ink(
                        ink,
                        glyph.x_offset,
                        glyph.y_offset,
                        font_size,
                        shaped.metrics.units_per_em,
                    )?
                    .translated(pen_x, pen_y)?;
                    exact_ink = Some(match exact_ink {
                        Some(current) => current.union(ink)?,
                        None => ink,
                    });
                }
                pen_x = pen_x.add(
                    scale_design(
                        i64::from(glyph.x_advance),
                        font_size,
                        shaped.metrics.units_per_em,
                    )?,
                    "text transition advance",
                )?;
                pen_y = pen_y.add(
                    scale_design(
                        i64::from(glyph.y_advance),
                        font_size,
                        shaped.metrics.units_per_em,
                    )?,
                    "text transition advance",
                )?;
            }
        }
    }
    let width_quanta = pen_x.ceil_i64("wrap advance")?;
    if width_quanta < 0 {
        return Err(TextPlanError::Arithmetic(
            "negative text transition advance",
        ));
    }
    let ink_span_quanta = exact_ink.map_or(Ok(0), |ink| {
        let ink = ink.outward()?;
        ink.right
            .checked_sub(ink.left)
            .ok_or(TextPlanError::Arithmetic("text transition ink span"))
    })?;
    Ok(TransitionMeasurement {
        width_quanta,
        ink_span_quanta,
    })
}

#[derive(Debug, Clone, Copy)]
struct ItemGroup {
    span: TextSourceSpan,
    face: FaceIdentity,
    script: Script,
}

#[derive(Debug, Clone, Copy)]
struct ScalarItem {
    start: usize,
    end: usize,
    face: FaceIdentity,
    script: Script,
}

fn itemize_bidi_run(
    pair: &ResolvedFontPair,
    paragraph: &str,
    paragraph_source_start: u32,
    bidi_run: Range<usize>,
) -> Result<Vec<ItemGroup>, TextPlanError> {
    let items = paragraph[bidi_run.clone()]
        .char_indices()
        .map(|(relative, scalar)| {
            let start = bidi_run.start + relative;
            Ok(ScalarItem {
                start,
                end: start + scalar.len_utf8(),
                face: selected_face(pair, scalar)?,
                script: scalar.script(),
            })
        })
        .collect::<Result<Vec<_>, TextPlanError>>()?;

    let mut groups: Vec<ItemGroup> = Vec::new();
    for item in items {
        let absolute_start = (paragraph_source_start as usize)
            .checked_add(item.start)
            .ok_or(TextPlanError::Arithmetic("run source span"))?;
        let absolute_end = (paragraph_source_start as usize)
            .checked_add(item.end)
            .ok_or(TextPlanError::Arithmetic("run source span"))?;
        if let Some(previous) = groups.last_mut() {
            if previous.face == item.face
                && previous.script == item.script
                && previous.span.end_byte as usize == absolute_start
            {
                previous.span.end_byte =
                    u32::try_from(absolute_end).map_err(|_| TextPlanError::SourceTooLong)?;
                continue;
            }
        }
        groups.push(ItemGroup {
            span: TextSourceSpan::new(absolute_start, absolute_end)?,
            face: item.face,
            script: item.script,
        });
    }
    Ok(groups)
}

fn selected_face(pair: &ResolvedFontPair, scalar: char) -> Result<FaceIdentity, TextPlanError> {
    let selected = pair
        .selected_scalars
        .binary_search_by_key(&scalar, |selected| selected.scalar)
        .ok()
        .and_then(|index| pair.selected_scalars.get(index))
        .ok_or(TextPlanError::InvalidScalarSelection(scalar as u32))?;
    match selected.selection {
        ScalarFaceSelection::Face(identity) => Ok(identity),
        ScalarFaceSelection::NonRenderingMandatoryBreak => {
            Err(TextPlanError::InvalidScalarSelection(scalar as u32))
        }
    }
}

fn rustybuzz_script(script: Script) -> Result<rustybuzz::Script, TextPlanError> {
    rustybuzz::Script::from_str(script.short_name())
        .map_err(|_| TextPlanError::Script(script.short_name().to_owned()))
}

#[derive(Debug)]
struct BuiltRun {
    plan: ShapedTextRun,
    advance_x: ExactRational,
    advance_y: ExactRational,
    ink: Option<ExactInk>,
}

fn scale_run(
    shaped: CanonicalShapedRun,
    source_span: TextSourceSpan,
    script: Script,
    font_size: u32,
) -> Result<BuiltRun, TextPlanError> {
    let mut glyphs = Vec::with_capacity(shaped.glyphs.len());
    let mut pen_x = ExactRational::ZERO;
    let mut pen_y = ExactRational::ZERO;
    let mut exact_ink: Option<ExactInk> = None;
    let mut design_horizontal_advance = 0_i64;
    let mut design_vertical_advance = 0_i64;

    for glyph in shaped.glyphs {
        let x_advance = scale_design(
            glyph.x_advance.into(),
            font_size,
            shaped.metrics.units_per_em,
        )?;
        let y_advance = scale_design(
            glyph.y_advance.into(),
            font_size,
            shaped.metrics.units_per_em,
        )?;
        let x_offset = scale_design(
            glyph.x_offset.into(),
            font_size,
            shaped.metrics.units_per_em,
        )?;
        let y_offset = scale_design(
            glyph.y_offset.into(),
            font_size,
            shaped.metrics.units_per_em,
        )?;
        let glyph_exact_ink = glyph
            .ink
            .map(|ink| {
                exact_glyph_ink(
                    ink,
                    glyph.x_offset,
                    glyph.y_offset,
                    font_size,
                    shaped.metrics.units_per_em,
                )
            })
            .transpose()?;
        if let Some(ink) = glyph_exact_ink {
            exact_ink = Some(match exact_ink {
                Some(current) => current.union(ink.translated(pen_x, pen_y)?)?,
                None => ink.translated(pen_x, pen_y)?,
            });
        }
        let cluster = source_span
            .start_byte
            .checked_add(glyph.cluster)
            .ok_or(TextPlanError::Arithmetic("glyph cluster"))?;
        glyphs.push(PlannedGlyph {
            face: shaped.identity,
            glyph_id: glyph.glyph_id,
            cluster,
            design_x_advance: glyph.x_advance,
            design_y_advance: glyph.y_advance,
            design_x_offset: glyph.x_offset,
            design_y_offset: glyph.y_offset,
            design_ink: glyph.ink,
            x_advance_quanta: x_advance.nearest_even_i64("glyph x advance")?,
            y_advance_quanta: y_advance.nearest_even_i64("glyph y advance")?,
            x_offset_quanta: x_offset.nearest_even_i64("glyph x offset")?,
            y_offset_quanta: y_offset.nearest_even_i64("glyph y offset")?,
            ink_quanta: glyph_exact_ink.map(ExactInk::outward).transpose()?,
        });
        pen_x = pen_x.add(x_advance, "run x advance")?;
        pen_y = pen_y.add(y_advance, "run y advance")?;
        design_horizontal_advance = design_horizontal_advance
            .checked_add(i64::from(glyph.x_advance))
            .ok_or(TextPlanError::Arithmetic("design run x advance"))?;
        design_vertical_advance = design_vertical_advance
            .checked_add(i64::from(glyph.y_advance))
            .ok_or(TextPlanError::Arithmetic("design run y advance"))?;
    }

    let (ascender_quanta, descender_quanta) = scaled_metrics(shaped.metrics, font_size)?;
    Ok(BuiltRun {
        plan: ShapedTextRun {
            source_span,
            face: shaped.identity,
            direction: shaped.direction,
            script,
            units_per_em: shaped.metrics.units_per_em,
            design_ascender: shaped.metrics.ascender,
            design_descender: shaped.metrics.descender,
            ascender_quanta,
            descender_quanta,
            design_x_advance: design_horizontal_advance,
            design_y_advance: design_vertical_advance,
            logical_advance_quanta: pen_x.nearest_even_i64("run logical advance")?,
            logical_y_advance_quanta: pen_y.nearest_even_i64("run logical y advance")?,
            ink_quanta: exact_ink.map(ExactInk::outward).transpose()?,
            glyphs,
        },
        advance_x: pen_x,
        advance_y: pen_y,
        ink: exact_ink,
    })
}

fn exact_glyph_ink(
    ink: GlyphInk,
    x_offset: i32,
    y_offset: i32,
    font_size: u32,
    units_per_em: u16,
) -> Result<ExactInk, TextPlanError> {
    Ok(ExactInk {
        left: scale_design(
            i64::from(x_offset)
                .checked_add(i64::from(ink.x_min))
                .ok_or(TextPlanError::Arithmetic("glyph ink left"))?,
            font_size,
            units_per_em,
        )?,
        bottom: scale_design(
            i64::from(y_offset)
                .checked_add(i64::from(ink.y_min))
                .ok_or(TextPlanError::Arithmetic("glyph ink bottom"))?,
            font_size,
            units_per_em,
        )?,
        right: scale_design(
            i64::from(x_offset)
                .checked_add(i64::from(ink.x_max))
                .ok_or(TextPlanError::Arithmetic("glyph ink right"))?,
            font_size,
            units_per_em,
        )?,
        top: scale_design(
            i64::from(y_offset)
                .checked_add(i64::from(ink.y_max))
                .ok_or(TextPlanError::Arithmetic("glyph ink top"))?,
            font_size,
            units_per_em,
        )?,
    })
}

fn scaled_metrics(metrics: DesignMetrics, font_size: u32) -> Result<(i64, i64), TextPlanError> {
    let ascender = scale_design(i64::from(metrics.ascender), font_size, metrics.units_per_em)?
        .nearest_even_i64("face ascender")?;
    let descender = scale_design(
        i64::from(metrics.descender),
        font_size,
        metrics.units_per_em,
    )?
    .nearest_even_i64("face descender")?;
    if ascender <= 0 || descender >= 0 {
        return Err(TextPlanError::Arithmetic("scaled face metrics"));
    }
    Ok((ascender, descender))
}

fn finalize_plan(
    input: TextPlanInput<'_>,
    stack: Vec<FamilyPreference>,
    pair_provenance: PairProvenance,
    line_height_quanta: i64,
    mut built_lines: Vec<BuiltLine>,
) -> Result<ResolvedTextPlan, TextPlanError> {
    let ascent = built_lines
        .iter()
        .map(|line| line.ascent)
        .max()
        .unwrap_or(0);
    let descent = built_lines
        .iter()
        .map(|line| line.descent)
        .max()
        .unwrap_or(0);
    let metric_height = ascent
        .checked_add(descent)
        .ok_or(TextPlanError::Arithmetic("text metric height"))?;
    let line_pitch_quanta = metric_height.max(line_height_quanta);

    let width_quanta = built_lines.iter().try_fold(0_i64, |width, line| {
        let span = line.plan.ink_quanta.map_or(Ok(0), |ink| {
            ink.right
                .checked_sub(ink.left)
                .ok_or(TextPlanError::Arithmetic("line ink span"))
        })?;
        Ok::<_, TextPlanError>(width.max(span))
    })?;

    for (line_index, line) in built_lines.iter_mut().enumerate() {
        let (line_left, line_span) = line.plan.ink_quanta.map_or(Ok((0, 0)), |ink| {
            Ok::<_, TextPlanError>((
                ink.left,
                ink.right
                    .checked_sub(ink.left)
                    .ok_or(TextPlanError::Arithmetic("line ink span"))?,
            ))
        })?;
        let alignment_space = width_quanta
            .checked_sub(line_span)
            .ok_or(TextPlanError::Arithmetic("line alignment slack"))?;
        let alignment_offset = match input.alignment {
            PhysicalTextAlignment::Start => 0,
            PhysicalTextAlignment::Center => ExactRational::new(i128::from(alignment_space), 2)?
                .nearest_even_i64("center line alignment")?,
            PhysicalTextAlignment::End => alignment_space,
        };
        line.plan.translation_x_quanta = alignment_offset
            .checked_sub(line_left)
            .ok_or(TextPlanError::Arithmetic("line x translation"))?;
        let line_index =
            i64::try_from(line_index).map_err(|_| TextPlanError::Arithmetic("line index"))?;
        line.plan.translation_y_quanta = line_pitch_quanta
            .checked_mul(line_index)
            .and_then(|offset| ascent.checked_add(offset))
            .ok_or(TextPlanError::Arithmetic("line baseline"))?;
    }

    let line_count =
        i64::try_from(built_lines.len()).map_err(|_| TextPlanError::Arithmetic("line count"))?;
    let height_quanta = if line_count == 0 {
        0
    } else {
        line_pitch_quanta
            .checked_mul(line_count - 1)
            .and_then(|pitch| metric_height.checked_add(pitch))
            .ok_or(TextPlanError::Arithmetic("text height"))?
    };
    let first_baseline_quanta = (line_count > 0).then_some(ascent);
    let last_baseline_quanta = if line_count == 0 {
        None
    } else {
        Some(
            line_pitch_quanta
                .checked_mul(line_count - 1)
                .and_then(|pitch| ascent.checked_add(pitch))
                .ok_or(TextPlanError::Arithmetic("last baseline"))?,
        )
    };

    Ok(ResolvedTextPlan {
        source: input.source.to_owned(),
        style: ResolvedTextStyle {
            stack_source: input.stack.to_owned(),
            stack,
            weight: input.weight,
            font_size: input.font_size,
            line_height_quanta,
            alignment: input.alignment,
            paint: input.paint,
            provenance: input.provenance,
            exact_override: input.exact_override,
        },
        pair_provenance,
        lines: built_lines.into_iter().map(|line| line.plan).collect(),
        line_pitch_quanta,
        first_baseline_quanta,
        last_baseline_quanta,
        width_quanta,
        height_quanta,
    })
}

#[derive(Debug, Clone, Copy)]
struct ExactInk {
    left: ExactRational,
    bottom: ExactRational,
    right: ExactRational,
    top: ExactRational,
}

impl ExactInk {
    fn translated(self, x: ExactRational, y: ExactRational) -> Result<Self, TextPlanError> {
        Ok(Self {
            left: self.left.add(x, "ink translation")?,
            bottom: self.bottom.add(y, "ink translation")?,
            right: self.right.add(x, "ink translation")?,
            top: self.top.add(y, "ink translation")?,
        })
    }

    fn union(self, other: Self) -> Result<Self, TextPlanError> {
        Ok(Self {
            left: self.left.min(other.left)?,
            bottom: self.bottom.min(other.bottom)?,
            right: self.right.max(other.right)?,
            top: self.top.max(other.top)?,
        })
    }

    fn outward(self) -> Result<TextInkBounds, TextPlanError> {
        Ok(TextInkBounds {
            left: self.left.floor_i64("ink left")?,
            bottom: self.bottom.floor_i64("ink bottom")?,
            right: self.right.ceil_i64("ink right")?,
            top: self.top.ceil_i64("ink top")?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExactRational {
    numerator: i128,
    denominator: i128,
}

impl ExactRational {
    const ZERO: Self = Self {
        numerator: 0,
        denominator: 1,
    };

    fn new(numerator: i128, denominator: i128) -> Result<Self, TextPlanError> {
        if denominator <= 0 {
            return Err(TextPlanError::Arithmetic("rational denominator"));
        }
        let divisor = gcd(numerator, denominator);
        Ok(Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        })
    }

    fn add(self, other: Self, resource: &'static str) -> Result<Self, TextPlanError> {
        let common = gcd(self.denominator, other.denominator);
        let left_factor = other.denominator / common;
        let right_factor = self.denominator / common;
        let numerator = self
            .numerator
            .checked_mul(left_factor)
            .and_then(|left| {
                other
                    .numerator
                    .checked_mul(right_factor)
                    .and_then(|right| left.checked_add(right))
            })
            .ok_or(TextPlanError::Arithmetic(resource))?;
        let denominator = self
            .denominator
            .checked_mul(left_factor)
            .ok_or(TextPlanError::Arithmetic(resource))?;
        Self::new(numerator, denominator)
    }

    fn floor_i64(self, resource: &'static str) -> Result<i64, TextPlanError> {
        i64::try_from(self.numerator.div_euclid(self.denominator))
            .map_err(|_| TextPlanError::Arithmetic(resource))
    }

    fn ceil_i64(self, resource: &'static str) -> Result<i64, TextPlanError> {
        let quotient = self.numerator.div_euclid(self.denominator);
        let remainder = self.numerator.rem_euclid(self.denominator);
        let rounded = if remainder == 0 {
            quotient
        } else {
            quotient
                .checked_add(1)
                .ok_or(TextPlanError::Arithmetic(resource))?
        };
        i64::try_from(rounded).map_err(|_| TextPlanError::Arithmetic(resource))
    }

    fn nearest_even_i64(self, resource: &'static str) -> Result<i64, TextPlanError> {
        let quotient = self.numerator.div_euclid(self.denominator);
        let remainder = self.numerator.rem_euclid(self.denominator);
        let doubled = remainder
            .checked_mul(2)
            .ok_or(TextPlanError::Arithmetic(resource))?;
        let increment = doubled > self.denominator
            || (doubled == self.denominator && quotient.rem_euclid(2) != 0);
        let rounded = if increment {
            quotient
                .checked_add(1)
                .ok_or(TextPlanError::Arithmetic(resource))?
        } else {
            quotient
        };
        i64::try_from(rounded).map_err(|_| TextPlanError::Arithmetic(resource))
    }

    fn min(self, other: Self) -> Result<Self, TextPlanError> {
        Ok(if self.cmp(other)?.is_le() {
            self
        } else {
            other
        })
    }

    fn max(self, other: Self) -> Result<Self, TextPlanError> {
        Ok(if self.cmp(other)?.is_ge() {
            self
        } else {
            other
        })
    }

    fn cmp(self, other: Self) -> Result<std::cmp::Ordering, TextPlanError> {
        self.numerator
            .checked_mul(other.denominator)
            .and_then(|left| {
                other
                    .numerator
                    .checked_mul(self.denominator)
                    .map(|right| left.cmp(&right))
            })
            .ok_or(TextPlanError::Arithmetic("rational comparison"))
    }
}

fn gcd(mut left: i128, mut right: i128) -> i128 {
    left = left.abs();
    right = right.abs();
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left.max(1)
}

fn scale_design(
    design: i64,
    font_size: u32,
    units_per_em: u16,
) -> Result<ExactRational, TextPlanError> {
    let numerator = i128::from(design)
        .checked_mul(i128::from(font_size))
        .and_then(|value| value.checked_mul(i128::from(CSS_PIXEL_QUANTA)))
        .ok_or(TextPlanError::Arithmetic("design-unit scaling"))?;
    ExactRational::new(numerator, i128::from(units_per_em))
}

fn resolve_line_height(line_height: TextLineHeight, font_size: u32) -> Result<i64, TextPlanError> {
    match line_height {
        TextLineHeight::Multiplier(multiplier) => {
            if !multiplier.is_finite() || multiplier <= 0.0 {
                return Err(TextPlanError::InvalidLineHeight);
            }
            binary64_product(
                multiplier,
                i128::from(font_size) * i128::from(CSS_PIXEL_QUANTA),
            )?
            .ceil_i64("resolved line height")
        }
        TextLineHeight::ResolvedQuanta(height) if height > 0 => Ok(height),
        TextLineHeight::ResolvedQuanta(_) => Err(TextPlanError::InvalidLineHeight),
    }
}

fn binary64_product(value: f64, factor: i128) -> Result<ExactRational, TextPlanError> {
    let bits = value.to_bits();
    let exponent_bits = ((bits >> 52) & 0x7FF) as i32;
    let fraction = bits & ((1_u64 << 52) - 1);
    let (significand, exponent) = if exponent_bits == 0 {
        (fraction, -1022 - 52)
    } else {
        (fraction | (1_u64 << 52), exponent_bits - 1023 - 52)
    };
    let mut numerator = i128::from(significand)
        .checked_mul(factor)
        .ok_or(TextPlanError::Arithmetic("binary64 rational"))?;
    let mut denominator = 1_i128;
    if exponent >= 0 {
        numerator = checked_shift_left(numerator, exponent.cast_unsigned(), "binary64 rational")?;
    } else {
        denominator = checked_shift_left(1, exponent.unsigned_abs(), "binary64 rational")?;
    }
    ExactRational::new(numerator, denominator)
}

pub(crate) fn maximum_width_quanta(maximum_width: f64) -> Result<i64, TextPlanError> {
    binary64_product(maximum_width, i128::from(CSS_PIXEL_QUANTA))?.floor_i64("maximum text width")
}

fn checked_shift_left(
    value: i128,
    shift: u32,
    resource: &'static str,
) -> Result<i128, TextPlanError> {
    let shifted = value
        .checked_shl(shift)
        .ok_or(TextPlanError::Arithmetic(resource))?;
    if shift != 0 && shifted.checked_shr(shift) != Some(value) {
        return Err(TextPlanError::Arithmetic(resource));
    }
    Ok(shifted)
}

/// Pixel-space view over an immutable operation-owned text plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextBlock {
    plan: ResolvedTextPlan,
}

impl TextBlock {
    pub(crate) const fn from_resolved(plan: ResolvedTextPlan) -> Self {
        Self { plan }
    }

    pub(crate) fn lines(&self) -> Vec<&str> {
        self.plan.line_texts()
    }

    pub(crate) fn into_lines(self) -> Vec<String> {
        self.plan.into_line_texts()
    }

    pub(crate) fn width(&self) -> f64 {
        self.plan.width()
    }

    pub(crate) fn height(&self) -> f64 {
        self.plan.height()
    }

    pub(crate) fn line_height(&self) -> f64 {
        self.plan.line_height()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prepared_transitions(source: &str) -> PreparedTextTransitions<'_> {
        let environment = PreparedFontEnvironment::synthetic();
        let stack = canonical::parse_family_stack("Test Sans").unwrap();
        let pair = environment
            .resolve_pair(
                0,
                &stack,
                RequestedWeight::Normal,
                &source.chars().collect::<Vec<_>>(),
            )
            .unwrap();
        PreparedTextTransitions::prepare(
            &environment,
            &pair,
            source,
            mandatory_paragraph_seeds(source).unwrap(),
            10,
            i64::MAX,
        )
        .unwrap()
    }

    fn input(
        source: &str,
        maximum_width_quanta: i64,
        alignment: PhysicalTextAlignment,
    ) -> TextPlanInput<'_> {
        TextPlanInput {
            source,
            stack: "  Test Sans  ",
            weight: TextWeight::Normal,
            font_size: 10,
            line_height: TextLineHeight::Multiplier(1.2),
            maximum_width_quanta,
            alignment,
            paint: Rgba::rgb(1, 2, 3),
            provenance: TextPlanProvenance {
                pair_ordinal: 0,
                owner: "test".into(),
            },
            exact_override: false,
        }
    }

    fn plan(
        source: &str,
        maximum_width_quanta: i64,
        alignment: PhysicalTextAlignment,
    ) -> ResolvedTextPlan {
        PreparedFontEnvironment::synthetic()
            .plan_text(input(source, maximum_width_quanta, alignment))
            .unwrap()
    }

    fn alternatives(source: &str, maximum_width_quanta: i64) -> CompleteTextAlternatives {
        complete_text_alternatives(
            &PreparedFontEnvironment::synthetic(),
            input(source, maximum_width_quanta, PhysicalTextAlignment::Start),
            false,
        )
        .unwrap()
    }

    fn raw_ending_vector(
        alternatives: &CompleteTextAlternatives,
        raw_attempt_index: usize,
    ) -> Option<Vec<u32>> {
        match &alternatives.raw_attempts[raw_attempt_index].outcome {
            RawTextAttemptOutcome::Admitted(range) => Some(
                alternatives.raw_lines[range.clone()]
                    .iter()
                    .map(|line| line.ending_grapheme_index)
                    .collect(),
            ),
            RawTextAttemptOutcome::Rejected(_) => None,
        }
    }

    fn admitted_ending_vectors(alternatives: &CompleteTextAlternatives) -> Vec<Vec<u32>> {
        alternatives
            .admitted
            .iter()
            .map(|alternative| {
                alternative
                    .plan
                    .lines
                    .iter()
                    .map(|line| line.ending_grapheme_index)
                    .collect()
            })
            .collect()
    }

    fn line_texts(plan: &ResolvedTextPlan) -> Vec<&str> {
        plan.lines
            .iter()
            .map(|line| &plan.source[line.source_span.range()])
            .collect()
    }

    #[test]
    fn stack_parser_trims_names_and_closes_generics() {
        assert_eq!(
            canonical::parse_family_stack(" Test Sans , serif, SANS-SERIF, monospace").unwrap(),
            [
                FamilyPreference::Named("Test Sans".into()),
                FamilyPreference::Serif,
                FamilyPreference::SansSerif,
                FamilyPreference::Monospace,
            ]
        );
        assert!(matches!(
            canonical::parse_family_stack("Test Sans,,serif"),
            Err(FamilyStackError::EmptyEntry { entry_index: 1 })
        ));
        assert!(matches!(
            canonical::parse_family_stack("cursive"),
            Err(FamilyStackError::UnsupportedGeneric { .. })
        ));
    }

    #[test]
    fn empty_present_text_has_one_primary_metric_line() {
        let plan = plan("", 0, PhysicalTextAlignment::Start);

        assert_eq!(line_texts(&plan), [""]);
        assert_eq!(plan.width_quanta, 0);
        assert_eq!(plan.height_quanta, 10 * CSS_PIXEL_QUANTA);
        assert_eq!(plan.first_baseline_quanta, Some(8 * CSS_PIXEL_QUANTA));
        assert_eq!(plan.last_baseline_quanta, Some(8 * CSS_PIXEL_QUANTA));
        assert!(plan.lines[0].runs.is_empty());
    }

    #[test]
    fn mandatory_breaks_preserve_crlf_and_all_blank_paragraphs() {
        let plan = plan("\r\nA\n\n", i64::MAX, PhysicalTextAlignment::Start);

        assert_eq!(line_texts(&plan), ["", "A", "", ""]);
        assert_eq!(
            plan.lines
                .iter()
                .map(|line| line.ending_grapheme_index)
                .collect::<Vec<_>>(),
            [0, 1, 1, 1]
        );
    }

    #[test]
    fn whitespace_is_preserved_and_contributes_logical_advance_not_ink() {
        let plan = plan(" ", i64::MAX, PhysicalTextAlignment::Start);

        assert_eq!(line_texts(&plan), [" "]);
        assert_eq!(plan.lines[0].logical_advance_quanta, 2560);
        assert_eq!(plan.width_quanta, 0);
        assert!(plan.lines[0].ink_quanta.is_none());
    }

    #[test]
    fn greedy_wrap_keeps_exact_spans_and_line_metrics() {
        let plan = plan("AA A", 14_848, PhysicalTextAlignment::Center);

        assert_eq!(line_texts(&plan), ["AA ", "A"]);
        assert_eq!(plan.width_quanta, 11_264);
        assert_eq!(plan.line_pitch_quanta, 12_288);
        assert_eq!(plan.first_baseline_quanta, Some(8192));
        assert_eq!(plan.last_baseline_quanta, Some(20_480));
        assert_eq!(plan.height_quanta, 22_528);
        assert_eq!(plan.lines[0].translation_x_quanta, 0);
        assert_eq!(plan.lines[1].translation_x_quanta, 3072);
        assert_eq!(plan.lines[0].translation_y_quanta, 8192);
        assert_eq!(plan.lines[1].translation_y_quanta, 20_480);
    }

    #[test]
    fn every_legal_transition_is_shaped_once_and_reused_for_fitting() {
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let transitions = prepared_transitions("A A A");
        let paragraph = &transitions.paragraphs[0].transitions;

        assert_eq!(paragraph.boundaries, [0, 2, 4, 5]);
        assert_eq!(
            transitions.widths_quanta,
            [8_704, 17_408, 23_552, 8_704, 14_848, 6_144]
        );
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 17));

        assert_eq!(
            paragraph
                .furthest_fitting_end(&transitions.widths_quanta, 0, 8_704)
                .unwrap(),
            Some(1)
        );
        assert_eq!(
            paragraph
                .furthest_fitting_end(&transitions.widths_quanta, 0, 17_408)
                .unwrap(),
            Some(2)
        );
        assert_eq!(
            paragraph
                .furthest_fitting_end(&transitions.widths_quanta, 0, 23_552)
                .unwrap(),
            Some(3)
        );
        assert_eq!(
            paragraph
                .furthest_fitting_end(&transitions.widths_quanta, 1, 14_848)
                .unwrap(),
            Some(3)
        );
        assert_eq!(
            paragraph
                .furthest_fitting_end(&transitions.widths_quanta, 0, 8_703)
                .unwrap(),
            None
        );
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 17));
    }

    #[test]
    fn completed_widths_include_distinct_complete_paragraph_ink_width() {
        let transitions = prepared_transitions("A A");

        assert_eq!(
            transitions.completed_widths_quanta,
            [0, 5_120, 6_144, 8_704, 13_824, 14_848]
        );
    }

    #[test]
    fn raw_attempts_use_completed_width_order_then_explicit_unwrapped() {
        let alternatives = alternatives("A A A", 17_408);

        assert_eq!(
            alternatives
                .raw_attempts
                .iter()
                .map(|attempt| attempt.kind)
                .collect::<Vec<_>>(),
            [
                RawTextAttemptKind::Greedy(0),
                RawTextAttemptKind::Greedy(5_120),
                RawTextAttemptKind::Greedy(6_144),
                RawTextAttemptKind::Greedy(8_704),
                RawTextAttemptKind::Greedy(14_848),
                RawTextAttemptKind::Greedy(17_408),
                RawTextAttemptKind::Unwrapped,
            ]
        );
    }

    #[test]
    fn explicit_unwrapped_duplicate_counts_but_shapes_one_admitted_vector() {
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let alternatives = alternatives("A", i64::MAX);

        assert_eq!(alternatives.raw_attempts.len(), 4);
        assert_eq!(raw_ending_vector(&alternatives, 2), Some(vec![1]));
        assert_eq!(raw_ending_vector(&alternatives, 3), Some(vec![1]));
        assert_eq!(admitted_ending_vectors(&alternatives), [vec![1]]);
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 2));
    }

    #[test]
    fn admitted_alternatives_deduplicate_and_sort_by_line_count_then_vector() {
        let alternatives = alternatives("A A A", i64::MAX);

        assert_eq!(
            admitted_ending_vectors(&alternatives),
            [vec![5], vec![2, 5], vec![4, 5], vec![2, 4, 5]]
        );
    }

    #[test]
    fn rejected_attempts_remain_in_raw_width_order() {
        let alternatives = alternatives("A A A", 7_000);
        let rejected = alternatives
            .raw_attempts
            .iter()
            .filter_map(|attempt| match (&attempt.kind, &attempt.outcome) {
                (RawTextAttemptKind::Greedy(width), RawTextAttemptOutcome::Rejected(miss)) => {
                    Some((*width, miss.clone()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            rejected.iter().map(|(width, _)| *width).collect::<Vec<_>>(),
            [0, 5_120, 6_144, 7_000]
        );
        assert!(rejected.iter().all(|(_, miss)| {
            miss.source_span == TextSourceSpan::new(0, 2).unwrap()
                && miss.required_width_quanta == 8_704
                && miss.maximum_width_quanta < 8_704
        }));
        assert_eq!(raw_ending_vector(&alternatives, 4), Some(vec![5]));
    }

    #[test]
    fn transition_slot_refusal_precedes_all_paragraph_shaping() {
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let environment = PreparedFontEnvironment::synthetic();
        environment.claim_text_generation_slots(999_995).unwrap();

        let error = environment
            .plan_text(input("A A\nA A", i64::MAX, PhysicalTextAlignment::Start))
            .unwrap_err();

        assert!(matches!(
            error,
            TextPlanError::Limit {
                resource: "candidate-generation slots",
                limit: 1_000_000,
                cumulative_claim,
            } if cumulative_claim > 1_000_000
        ));
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
    }

    #[test]
    fn paragraph_bidi_is_computed_once_for_transitions_and_selected_lines() {
        PARAGRAPH_BIDI_CALLS.with(|calls| calls.set(0));
        let plan = plan("A A A", i64::MAX, PhysicalTextAlignment::Start);

        assert_eq!(line_texts(&plan), ["A A A"]);
        PARAGRAPH_BIDI_CALLS.with(|calls| assert_eq!(calls.get(), 1));
    }

    #[test]
    fn mixed_direction_non_latin_text_retains_exact_runs_and_glyph_identity() {
        let plan = plan("Aא", i64::MAX, PhysicalTextAlignment::Start);
        let line = &plan.lines[0];
        assert_eq!(line.base_direction, ShapingDirection::LeftToRight);
        let latin = line
            .runs
            .iter()
            .find(|run| run.script == Script::Latin)
            .unwrap();
        let hebrew = line
            .runs
            .iter()
            .find(|run| run.script == Script::Hebrew)
            .unwrap();

        assert_eq!(&plan.source[latin.source_span.range()], "A");
        assert_eq!(latin.direction, ShapingDirection::LeftToRight);
        assert_eq!(&plan.source[hebrew.source_span.range()], "א");
        assert_eq!(hebrew.direction, ShapingDirection::RightToLeft);
        assert_eq!(latin.face, hebrew.face);
        assert!(latin.glyphs.iter().all(|glyph| glyph.glyph_id != 0));
        assert!(hebrew.glyphs.iter().all(|glyph| glyph.glyph_id != 0));
    }

    #[test]
    fn shaping_runs_split_at_direct_common_script_boundaries() {
        let plan = plan("A A", i64::MAX, PhysicalTextAlignment::Start);
        let runs = &plan.lines[0].runs;

        assert_eq!(
            runs.iter()
                .map(|run| (&plan.source[run.source_span.range()], run.script))
                .collect::<Vec<_>>(),
            [
                ("A", Script::Latin),
                (" ", Script::Common),
                ("A", Script::Latin)
            ]
        );
    }

    #[test]
    fn transition_pairs_are_counted_per_mandatory_paragraph() {
        let paragraphs = mandatory_paragraphs("A A\nA A").unwrap();
        let total = paragraphs
            .iter()
            .map(|span| {
                let boundary_count = legal_line_breaks(&"A A\nA A"[span.range()]).len() + 1;
                checked_transition_count(boundary_count).unwrap()
            })
            .sum::<u128>();

        assert_eq!(total, 6);
    }

    #[test]
    fn transition_pair_count_is_checked_at_the_slot_limit() {
        assert_eq!(checked_transition_count(1_414).unwrap(), 998_991);
        assert_eq!(checked_transition_count(1_415).unwrap(), 1_000_405);
    }

    #[test]
    fn long_breakable_input_returns_typed_generation_work_limit_before_shaping() {
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let source = "A ".repeat(144);
        let error = PreparedFontEnvironment::synthetic()
            .plan_text(input(&source, i64::MAX, PhysicalTextAlignment::Start))
            .unwrap_err();

        assert!(matches!(
            error,
            TextPlanError::Limit {
                resource: "candidate-generation work units",
                limit: 1_000_000,
                cumulative_claim,
            } if cumulative_claim > 1_000_000
        ));
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
    }

    #[test]
    fn physical_end_alignment_uses_block_ink_width() {
        let plan = plan("AA A", 14_848, PhysicalTextAlignment::End);

        assert_eq!(plan.lines[0].translation_x_quanta, 0);
        assert_eq!(plan.lines[1].translation_x_quanta, 6144);
    }

    #[test]
    fn scaled_glyphs_use_nearest_even_and_outward_ink() {
        let environment = PreparedFontEnvironment::synthetic();
        let mut request = input("A", i64::MAX, PhysicalTextAlignment::Start);
        request.font_size = 11;
        let plan = environment.plan_text(request).unwrap();
        let glyph = &plan.lines[0].runs[0].glyphs[0];

        assert_eq!(glyph.glyph_id, 1);
        assert_eq!(glyph.cluster, 0);
        assert_eq!(glyph.design_x_advance, 600);
        assert_eq!(glyph.x_advance_quanta, 6758);
        assert_eq!(
            glyph.ink_quanta,
            Some(TextInkBounds {
                left: 0,
                bottom: 0,
                right: 5632,
                top: 7885,
            })
        );
    }

    #[test]
    fn intact_overwide_sequence_returns_typed_fit_miss() {
        let error = PreparedFontEnvironment::synthetic()
            .plan_text(input("AA", 6000, PhysicalTextAlignment::Start))
            .unwrap_err();

        assert!(matches!(
            error,
            TextPlanError::Fit(TextFitMiss {
                source_span: TextSourceSpan {
                    start_byte: 0,
                    end_byte: 2
                },
                maximum_width_quanta: 6000,
                required_width_quanta: 12_288,
            })
        ));
    }

    #[test]
    fn environment_clone_and_database_replay_are_immutable() {
        let environment = PreparedFontEnvironment::synthetic();
        let clone = environment.clone();
        let first = environment
            .plan_text(input("A A", i64::MAX, PhysicalTextAlignment::Start))
            .unwrap();
        let second = clone
            .plan_text(input("A A", i64::MAX, PhysicalTextAlignment::Start))
            .unwrap();
        assert_eq!(first, second);

        let mut first_database = fontdb::Database::new();
        let mut second_database = fontdb::Database::new();
        environment.populate_database(&mut first_database).unwrap();
        clone.populate_database(&mut second_database).unwrap();
        assert_eq!(first_database.faces().count(), 1);
        assert_eq!(second_database.faces().count(), 1);
        assert_eq!(
            first_database.faces().next().unwrap().index,
            second_database.faces().next().unwrap().index
        );
    }

    #[test]
    fn deferred_annotation_wraps_without_remeasurement() {
        let environment = PreparedFontEnvironment::synthetic();
        let mut request = input("A", i64::MAX, PhysicalTextAlignment::Center);
        request.exact_override = true;
        let plan = environment.plan_text(request).unwrap();
        let deferred = DeferredAnnotationTextPlan::from_resolved(plan.clone());

        assert!(deferred.exact_override);
        assert_eq!(deferred.plan, plan);
    }
}
