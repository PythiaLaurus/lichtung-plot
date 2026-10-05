//! Immutable, generation-font-backed text planning.

#![allow(dead_code)]

use std::ops::Range;
use std::str::FromStr;

use unicode_bidi::{BidiDataSource, HardcodedBidiData, ParagraphBidiInfo};
use unicode_script::{Script, UnicodeScript};
use unicode_segmentation::UnicodeSegmentation;

use crate::font::canonical::{
    self, DesignMetrics, FaceIdentity, FontError, FontStack, GlyphInk, PairProvenance,
    PreparedFontEnvironment, RequestedWeight, ResolvedFontPair, ScalarFaceSelection,
    ShapedRun as CanonicalShapedRun, ShapingDirection,
};
use crate::theme::Color;
use crate::TextWeight;

mod requirements;
#[cfg(test)]
mod shaping_tests;
#[allow(unused_imports)]
pub(crate) use requirements::{
    foundational_plot_requirements, PreparedFontPairRequirement, PreparedTextCensus,
    PreparedTextRequirement, PreparedTextRole,
};

pub(crate) const CSS_PIXEL_QUANTA: i64 = 1024;
#[cfg(test)]
thread_local! {
    static PARAGRAPH_BIDI_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static TEXT_WORK: std::cell::Cell<TextWork> = const { std::cell::Cell::new(TextWork {
        proposal_visits: 0, exact_trials: 0, shaping_scalars: 0, shaped_glyphs: 0,
    }) };
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TextWork {
    proposal_visits: usize,
    exact_trials: usize,
    shaping_scalars: usize,
    shaped_glyphs: usize,
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
    pub(crate) stack: &'a FontStack,
    pub(crate) weight: TextWeight,
    pub(crate) font_size: u32,
    pub(crate) line_height: TextLineHeight,
    pub(crate) maximum_width_quanta: i64,
    pub(crate) alignment: PhysicalTextAlignment,
    pub(crate) paint: Color,
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
    /// Absolute within the line, baseline-relative and y-up. Rounded once.
    pub(crate) glyph_origin_x_quanta: i64,
    pub(crate) glyph_origin_y_quanta: i64,
    pub(crate) ink_quanta: Option<TextInkBounds>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ShapedTextRun {
    pub(crate) source_span: TextSourceSpan,
    pub(crate) face: FaceIdentity,
    pub(crate) direction: ShapingDirection,
    /// Paragraph-resolved embedding level after this line's L1 reset.
    pub(crate) bidi_level: u8,
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
    /// Selected discretionary break: render U+002D for this original U+00AD span.
    pub(crate) selected_soft_hyphen: Option<TextSourceSpan>,
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
    pub(crate) stack: FontStack,
    pub(crate) weight: TextWeight,
    pub(crate) font_size: u32,
    pub(crate) line_height_quanta: i64,
    pub(crate) alignment: PhysicalTextAlignment,
    pub(crate) paint: Color,
    pub(crate) provenance: TextPlanProvenance,
    pub(crate) exact_override: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedTextPlan(
    std::sync::Arc<ResolvedTextPayload>,
    Option<std::sync::Arc<TextAnalysis>>,
);

// Cached analysis is operational state, not part of rendered-plan equality.
impl PartialEq for ResolvedTextPlan {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for ResolvedTextPlan {}

#[derive(Debug)]
struct TextAnalysis {
    environment: PreparedFontEnvironment,
    original: std::sync::Arc<ResolvedTextPayload>,
    paragraphs: Vec<AnalyzedParagraph>,
}

#[derive(Debug)]
struct AnalyzedParagraph {
    span: TextSourceSpan,
    grapheme_start: usize,
    grapheme_ends: Vec<usize>,
    bidi: Option<ParagraphBidiData>,
    items: Vec<ScalarItem>,
    hyphen_face: Option<FaceIdentity>,
    whole_width: i64,
    whole: ParagraphWhole,
    wrapping: std::sync::OnceLock<ParagraphWrapping>,
    hard_width: std::sync::OnceLock<i64>,
}

#[derive(Debug)]
enum ParagraphWhole {
    Retained(BuiltLine),
    OriginalLine {
        index: usize,
        advance: ExactRational,
        ascent: i64,
        descent: i64,
    },
}

#[derive(Debug)]
struct ParagraphWrapping {
    boundaries: Vec<usize>,
    prefixes: Vec<ExactRational>,
}

#[derive(Debug)]
struct ParagraphBidiData {
    original_classes: Vec<unicode_bidi::BidiClass>,
    levels: Vec<unicode_bidi::Level>,
    paragraph_level: unicode_bidi::Level,
    is_pure_ltr: bool,
}

impl From<ParagraphBidiInfo<'_>> for ParagraphBidiData {
    fn from(info: ParagraphBidiInfo<'_>) -> Self {
        Self {
            original_classes: info.original_classes,
            levels: info.levels,
            paragraph_level: info.paragraph_level,
            is_pure_ltr: info.is_pure_ltr,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Clone))]
pub(crate) struct ResolvedTextPayload {
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

impl std::ops::Deref for ResolvedTextPlan {
    type Target = ResolvedTextPayload;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ResolvedTextPlan {
    /// Lend the immutable original profile after the cache has verified the caller's complete input.
    pub(crate) fn analysis_profile(&self) -> Result<(&str, &FontStack), TextPlanError> {
        let original = &self
            .1
            .as_ref()
            .ok_or(TextPlanError::Arithmetic("missing text analysis"))?
            .original;
        Ok((&original.source, &original.style.stack))
    }

    pub(crate) fn same_stack(&self, stack: &FontStack) -> Result<bool, TextPlanError> {
        let environment = &self
            .1
            .as_ref()
            .ok_or(TextPlanError::Arithmetic("missing text analysis"))?
            .environment;
        Ok(self
            .style
            .stack
            .same(stack, &environment.generation_resources())?)
    }
    pub(crate) fn copy_stack(&self) -> Result<FontStack, TextPlanError> {
        let environment = &self
            .1
            .as_ref()
            .ok_or(TextPlanError::Arithmetic("missing text analysis"))?
            .environment;
        Ok(self
            .style
            .stack
            .copy_with_generation(&environment.generation_resources())?)
    }

    pub(crate) fn intrinsic_from_analysis(
        &self,
        preferred_lines: usize,
    ) -> Result<TextIntrinsicRequirements, TextPlanError> {
        let environment = &self
            .1
            .as_ref()
            .ok_or(TextPlanError::Arithmetic("missing text analysis"))?
            .environment;
        let call = environment
            .claim_measurement_call()
            .map_err(text_generation_error)?;
        self.intrinsic_in_call(preferred_lines, &call)
    }

    pub(crate) fn intrinsic_in_call(
        &self,
        preferred_lines: usize,
        _call: &canonical::MeasurementCall,
    ) -> Result<TextIntrinsicRequirements, TextPlanError> {
        let analysis = self
            .1
            .as_ref()
            .ok_or(TextPlanError::Arithmetic("missing text analysis"))?;
        let environment = &analysis.environment;
        let original = &analysis.original;
        let mut result = TextIntrinsicRequirements {
            hard_width_quanta: 0,
            natural_width_quanta: 0,
            preferred_width_quanta: 0,
        };
        for paragraph in &analysis.paragraphs {
            claim_processing_visits(environment, 1)?;
            if paragraph.span.start_byte == paragraph.span.end_byte {
                continue;
            }
            let (whole, _, _, _) = paragraph.whole(original);
            if let Some(ink) = whole.ink_quanta {
                result.natural_width_quanta = result.natural_width_quanta.max(
                    ink.right
                        .checked_sub(ink.left)
                        .ok_or(TextPlanError::Arithmetic("intrinsic ink width"))?,
                );
            }
            let wrapping = paragraph.wrapping(
                environment,
                &original.source,
                whole,
                original.style.font_size,
            )?;
            let last = wrapping.boundaries.len() - 1;
            if paragraph.hard_width.get().is_none() {
                let mut hard = 0;
                for start in 0..last {
                    claim_processing_visits(environment, 1)?;
                    let width = if last == 1 {
                        paragraph.whole_width
                    } else {
                        let end = wrapping.boundaries[start + 1];
                        let line = prepare_line(
                            environment,
                            &original.source,
                            paragraph.span,
                            TextSourceSpan::new(
                                paragraph.span.start_byte as usize + wrapping.boundaries[start],
                                paragraph.span.start_byte as usize + end,
                            )?,
                            ending_grapheme_index(
                                &paragraph.grapheme_ends,
                                end,
                                paragraph.grapheme_start,
                            )?,
                            original.style.font_size,
                            paragraph
                                .bidi
                                .as_ref()
                                .ok_or(TextPlanError::Arithmetic("empty intrinsic paragraph"))?,
                            &paragraph.items,
                            paragraph.hyphen_face,
                        )?;
                        exact_line_width(environment, &line)?
                    };
                    hard = hard.max(width);
                }
                environment
                    .claim_cache_entries(1)
                    .map_err(text_generation_error)?;
                let _ = paragraph.hard_width.set(hard);
            }
            result.hard_width_quanta = result.hard_width_quanta.max(
                *paragraph
                    .hard_width
                    .get()
                    .ok_or(TextPlanError::Arithmetic("intrinsic width initialization"))?,
            );
            let lines = (preferred_lines / analysis.paragraphs.len().max(1)).max(1);
            let chunk = last.div_ceil(lines);
            for start in (0..last).step_by(chunk) {
                claim_processing_visits(environment, 1)?;
                let end = start.saturating_add(chunk).min(last);
                result.preferred_width_quanta = result.preferred_width_quanta.max(
                    prefix_difference(wrapping.prefixes[end], wrapping.prefixes[start])?
                        .ceil_i64("preferred width")?,
                );
            }
        }
        result.preferred_width_quanta = result.preferred_width_quanta.max(result.hard_width_quanta);
        Ok(result)
    }

    /// Re-select lines from this operation's immutable source/font analysis.
    pub(crate) fn reflow(&self, input: TextPlanInput<'_>) -> Result<Self, TextPlanError> {
        let environment = &self
            .1
            .as_ref()
            .ok_or(TextPlanError::Arithmetic("missing text analysis"))?
            .environment;
        let call = environment
            .claim_measurement_call()
            .map_err(text_generation_error)?;
        self.reflow_in_call(input, &call)
    }

    pub(crate) fn reflow_in_call(
        &self,
        mut input: TextPlanInput<'_>,
        _call: &canonical::MeasurementCall,
    ) -> Result<Self, TextPlanError> {
        let analysis = self
            .1
            .as_ref()
            .ok_or(TextPlanError::Arithmetic("missing text analysis"))?;
        let environment = &analysis.environment;
        let original = &analysis.original;
        let resources = environment.generation_resources();
        if !canonical::same_bytes(input.source, &original.source, &resources)?
            || !input.stack.same(&original.style.stack, &resources)?
            || input.weight != original.style.weight
            || input.font_size != original.style.font_size
        {
            return Err(TextPlanError::Arithmetic("text analysis profile mismatch"));
        }
        if input.maximum_width_quanta < 0 {
            return Err(TextPlanError::InvalidMaximumWidth);
        }
        let line_height = resolve_line_height(input.line_height, input.font_size, &resources)?;
        let mut lines = Vec::new();
        for paragraph in &analysis.paragraphs {
            claim_processing_visits(environment, 1)?;
            let (whole, advance, ascent, descent) = paragraph.whole(original);
            if paragraph.whole_width <= input.maximum_width_quanta {
                retain_line(
                    environment,
                    &mut lines,
                    copy_line(environment, whole, advance, ascent, descent)?,
                )?;
                continue;
            }
            let wrapping = paragraph.wrapping(environment, input.source, whole, input.font_size)?;
            let boundaries = &wrapping.boundaries;
            let bidi = paragraph
                .bidi
                .as_ref()
                .ok_or(TextPlanError::Arithmetic("empty paragraph reflow"))?;
            let mut start = 0;
            while start + 1 < boundaries.len() {
                let (end, line) = select_line(
                    environment,
                    &wrapping.prefixes,
                    start,
                    input.maximum_width_quanta,
                    (start == 0).then_some(paragraph.whole_width),
                    TextSourceSpan::new(
                        paragraph.span.start_byte as usize + boundaries[start],
                        paragraph.span.start_byte as usize + boundaries[start + 1],
                    )?,
                    |end| {
                        prepare_line(
                            environment,
                            input.source,
                            paragraph.span,
                            TextSourceSpan::new(
                                paragraph.span.start_byte as usize + boundaries[start],
                                paragraph.span.start_byte as usize + boundaries[end],
                            )?,
                            ending_grapheme_index(
                                &paragraph.grapheme_ends,
                                boundaries[end],
                                paragraph.grapheme_start,
                            )?,
                            input.font_size,
                            bidi,
                            &paragraph.items,
                            paragraph.hyphen_face,
                        )
                    },
                )?;
                retain_line(environment, &mut lines, line)?;
                start = end;
            }
        }
        let mut candidates = Vec::new();
        let provenance = &original.pair_provenance;
        claim_collection_count(environment, provenance.candidates.len() as u128)?;
        super::unicode16::reserve(
            &mut candidates,
            provenance.candidates.len(),
            &mut |bytes, work| claim_unicode(environment, bytes, work),
        )?;
        candidates.extend_from_slice(&provenance.candidates);
        input.provenance.pair_ordinal = provenance.pair_ordinal;
        let stack = input.stack;
        let mut plan = finalize_plan(
            environment,
            input,
            stack,
            PairProvenance {
                pair_ordinal: provenance.pair_ordinal,
                weight: provenance.weight,
                candidates,
            },
            line_height,
            lines,
        )?;
        plan.1 = Some(std::sync::Arc::clone(analysis));
        Ok(plan)
    }

    pub(crate) fn unwrapped_fit_width(
        &self,
        environment: &PreparedFontEnvironment,
    ) -> Result<Option<i64>, TextPlanError> {
        let mut previous_end = None;
        let mut required = 0;
        for line in &self.lines {
            claim_processing_visits(environment, 1)?;
            if let Some(end) = previous_end {
                let separator = &self.source[end..line.source_span.start_byte as usize];
                if separator.is_empty() {
                    return Ok(None);
                }
                for scalar in separator.chars() {
                    claim_processing_visits(environment, 1)?;
                    if !canonical::is_nonrendering_mandatory_break(scalar) {
                        return Ok(None);
                    }
                }
            }
            previous_end = Some(line.source_span.end_byte as usize);
            let mut advance = ExactRational::ZERO;
            for run in &line.runs {
                claim_processing_visits(environment, 1)?;
                advance = advance.add(
                    scale_design(run.design_x_advance, self.style.font_size, run.units_per_em)?,
                    "cached paragraph advance",
                )?;
            }
            let ink_width = line.ink_quanta.map_or(Ok(0), |ink| {
                ink.right
                    .checked_sub(ink.left)
                    .ok_or(TextPlanError::Arithmetic("cached paragraph ink"))
            })?;
            required = required
                .max(advance.ceil_i64("cached paragraph advance")?)
                .max(ink_width);
        }
        Ok(Some(required))
    }

    #[cfg(test)]
    pub(crate) fn edit_for_test(&mut self) -> &mut ResolvedTextPayload {
        std::sync::Arc::make_mut(&mut self.0)
    }

    pub(crate) fn shares_payload(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }

    #[cfg(test)]
    pub(crate) fn shares_analysis(&self, other: &Self) -> bool {
        match (&self.1, &other.1) {
            (Some(left), Some(right)) => std::sync::Arc::ptr_eq(left, right),
            _ => false,
        }
    }
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

    /// Standalone rendered lines for string-based adapters; the immutable plan
    /// continues to retain the original source and discretionary-break span.
    pub(crate) fn into_line_texts(
        self,
        environment: &PreparedFontEnvironment,
    ) -> Result<Vec<String>, TextPlanError> {
        let mut lines = Vec::new();
        claim_collection_count(environment, self.lines.len() as u128)?;
        super::unicode16::reserve(&mut lines, self.lines.len(), &mut |bytes, work| {
            claim_unicode(environment, bytes, work)
        })?;
        for line in &self.lines {
            let text = &self.source[line.source_span.range()];
            let length = text.len() - usize::from(line.selected_soft_hyphen.is_some());
            claim_unicode(environment, length as u64, 0)?;
            let mut retained = String::new();
            retained
                .try_reserve_exact(length)
                .map_err(|_| TextPlanError::Allocation("owned text line"))?;
            if let Some(soft) = line.selected_soft_hyphen {
                retained.push_str(
                    &self.source[line.source_span.start_byte as usize..soft.start_byte as usize],
                );
                retained.push('-');
                retained.push_str(
                    &self.source[soft.end_byte as usize..line.source_span.end_byte as usize],
                );
            } else {
                retained.push_str(text);
            }
            lines.push(retained);
        }
        Ok(lines)
    }
}

impl AnalyzedParagraph {
    fn whole<'a>(
        &'a self,
        original: &'a ResolvedTextPayload,
    ) -> (&'a TextLinePlan, ExactRational, i64, i64) {
        match &self.whole {
            ParagraphWhole::Retained(line) => (&line.plan, line.advance, line.ascent, line.descent),
            ParagraphWhole::OriginalLine {
                index,
                advance,
                ascent,
                descent,
            } => (&original.lines[*index], *advance, *ascent, *descent),
        }
    }

    fn wrapping(
        &self,
        environment: &PreparedFontEnvironment,
        source: &str,
        whole: &TextLinePlan,
        font_size: u32,
    ) -> Result<&ParagraphWrapping, TextPlanError> {
        if self.wrapping.get().is_none() {
            let mut boundaries = legal_line_breaks_at_graphemes(
                &source[self.span.range()],
                &self.grapheme_ends,
                environment,
            )?;
            claim_collection_count(environment, 1)?;
            super::unicode16::reserve(&mut boundaries, 1, &mut |bytes, work| {
                claim_unicode(environment, bytes, work)
            })?;
            boundaries.insert(0, 0);
            let prefixes = proposal_prefixes(environment, whole, &boundaries, font_size)?;
            environment
                .claim_cache_entries(1)
                .map_err(text_generation_error)?;
            let _ = self.wrapping.set(ParagraphWrapping {
                boundaries,
                prefixes,
            });
        }
        self.wrapping.get().ok_or(TextPlanError::Arithmetic(
            "paragraph wrapping initialization",
        ))
    }
}

fn copy_line(
    environment: &PreparedFontEnvironment,
    line: &TextLinePlan,
    advance: ExactRational,
    ascent: i64,
    descent: i64,
) -> Result<BuiltLine, TextPlanError> {
    let mut runs = Vec::new();
    claim_collection_count(environment, line.runs.len() as u128)?;
    super::unicode16::reserve(&mut runs, line.runs.len(), &mut |bytes, work| {
        claim_unicode(environment, bytes, work)
    })?;
    for run in &line.runs {
        let mut glyphs = Vec::new();
        claim_collection_count(environment, run.glyphs.len() as u128)?;
        super::unicode16::reserve(&mut glyphs, run.glyphs.len(), &mut |bytes, work| {
            claim_unicode(environment, bytes, work)
        })?;
        glyphs.extend_from_slice(&run.glyphs);
        runs.push(ShapedTextRun { glyphs, ..*run });
    }
    Ok(BuiltLine {
        plan: TextLinePlan { runs, ..*line },
        advance,
        ascent,
        descent,
    })
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
    Spacing(#[from] crate::Error),
    #[error(transparent)]
    Font(FontError),
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

impl From<FontError> for TextPlanError {
    fn from(error: FontError) -> Self {
        match error {
            FontError::ShapeAdmission(claim) => match claim_text_generation(Err(claim)) {
                Err(error) => error,
                Ok(()) => unreachable!("failed shaping admission"),
            },
            error => Self::Font(error),
        }
    }
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
        let call = self
            .claim_measurement_call()
            .map_err(text_generation_error)?;
        self.plan_text_in_call(input, false, &call)
    }

    pub(crate) fn plan_text_in_call(
        &self,
        input: TextPlanInput<'_>,
        prepared_only: bool,
        _call: &canonical::MeasurementCall,
    ) -> Result<ResolvedTextPlan, TextPlanError> {
        match measure_text(
            self,
            input,
            prepared_only && self.prepared_pairs_are_sealed()?,
            None,
        )? {
            TextMeasurement::Assigned(plan) => Ok(plan),
            TextMeasurement::Intrinsic(_) => unreachable!("assigned text request"),
        }
    }

    pub(crate) fn plan_prepared_text(
        &self,
        input: TextPlanInput<'_>,
    ) -> Result<ResolvedTextPlan, TextPlanError> {
        let call = self
            .claim_measurement_call()
            .map_err(text_generation_error)?;
        self.plan_text_in_call(input, true, &call)
    }

    pub(crate) fn text_intrinsic_requirements(
        &self,
        input: TextPlanInput<'_>,
        preferred_lines: usize,
    ) -> Result<TextIntrinsicRequirements, TextPlanError> {
        self.claim_measurement_call()
            .map_err(text_generation_error)?;
        match measure_text(
            self,
            input,
            self.prepared_pairs_are_sealed()?,
            Some(preferred_lines),
        )? {
            TextMeasurement::Intrinsic(requirements) => Ok(requirements),
            TextMeasurement::Assigned(_) => unreachable!("intrinsic text request"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[expect(
    clippy::struct_field_names,
    reason = "explicit quantum units distinguish these widths from the pixel-space adapter lengths"
)]
pub(crate) struct TextIntrinsicRequirements {
    pub(crate) hard_width_quanta: i64,
    pub(crate) natural_width_quanta: i64,
    pub(crate) preferred_width_quanta: i64,
}

enum TextMeasurement {
    Intrinsic(TextIntrinsicRequirements),
    Assigned(ResolvedTextPlan),
}

#[expect(
    clippy::too_many_lines,
    reason = "constrained text planning keeps count, reservation, materialization, and freezing ordered"
)]
fn measure_text(
    environment: &PreparedFontEnvironment,
    mut input: TextPlanInput<'_>,
    prepared_only: bool,
    intrinsic_lines: Option<usize>,
) -> Result<TextMeasurement, TextPlanError> {
    let prepared_only = prepared_only || environment.prepared_pairs_are_sealed()?;
    if input.font_size == 0 {
        return Err(TextPlanError::InvalidFontSize);
    }
    if input.maximum_width_quanta < 0 {
        return Err(TextPlanError::InvalidMaximumWidth);
    }
    let _ = u32::try_from(input.source.len()).map_err(|_| TextPlanError::SourceTooLong)?;

    let stack = input.stack;
    let line_height_quanta = resolve_line_height(
        input.line_height,
        input.font_size,
        &environment.generation_resources(),
    )?;
    claim_processing_visits(environment, input.source.chars().count() as u128)?;
    let scalar_count = input.source.chars().count();
    let mut required_scalars = Vec::new();
    super::unicode16::reserve(
        &mut required_scalars,
        scalar_count + 1,
        &mut |bytes, work| claim_unicode(environment, bytes, work),
    )?;
    required_scalars.extend(input.source.chars());
    if input.source.contains('\u{ad}') {
        required_scalars.push('-');
    }
    let weight = requested_weight(input.weight);
    let prepared = environment.prepared_pair(stack, weight)?;
    let mut pair = match prepared {
        Some(pair) if prepared_only => pair,
        Some(pair) => {
            if required_scalars.iter().any(|scalar| {
                !pair
                    .selected_scalars
                    .iter()
                    .any(|selection| selection.scalar == *scalar)
            }) {
                environment.resolve_pair(
                    pair.provenance.pair_ordinal,
                    stack,
                    weight,
                    &required_scalars,
                )?
            } else {
                pair
            }
        }
        None if prepared_only => return Err(TextPlanError::MissingPreparedPair),
        None => {
            environment.prepare_pair(
                input.provenance.pair_ordinal,
                stack,
                weight,
                &required_scalars,
                None,
            )?;
            environment
                .prepared_pair(stack, weight)?
                .ok_or(TextPlanError::MissingPreparedPair)?
        }
    };
    input.provenance.pair_ordinal = pair.provenance.pair_ordinal;
    if !prepared_only {
        environment.prepare_source_units(pair.provenance.pair_ordinal, input.source, None)?;
        pair.selected_units = environment
            .prepared_pair(stack, weight)?
            .ok_or(TextPlanError::MissingPreparedPair)?
            .selected_units;
    }
    let primary_metrics = environment.face_metrics(pair.primary)?;
    let source_scalar_count = u128::try_from(required_scalars.len())
        .map_err(|_| TextPlanError::Arithmetic("text scalar count"))?;
    claim_processing_visits(environment, source_scalar_count)?;
    let paragraph_seeds = mandatory_paragraph_seeds(environment, input.source)?;
    let mut built_lines = Vec::new();
    let mut analyzed_paragraphs = Vec::new();
    let mut requirements = TextIntrinsicRequirements {
        hard_width_quanta: 0,
        natural_width_quanta: 0,
        preferred_width_quanta: 0,
    };
    let paragraph_count = paragraph_seeds.len();
    let mut grapheme_start = 0_usize;
    claim_collection_count(
        environment,
        source_scalar_count + paragraph_count as u128 + 1,
    )?;
    for seed in paragraph_seeds {
        let paragraph = &input.source[seed.span.range()];
        claim_processing_visits(environment, seed.scalar_count)?;
        let mut grapheme_ends = Vec::new();
        for (start, grapheme) in paragraph.grapheme_indices(true) {
            claim_collection_count(environment, 1)?;
            super::unicode16::reserve(&mut grapheme_ends, 1, &mut |bytes, work| {
                claim_unicode(environment, bytes, work)
            })?;
            grapheme_ends.push(start + grapheme.len());
        }
        let next_grapheme = grapheme_start
            .checked_add(grapheme_ends.len())
            .ok_or(TextPlanError::Arithmetic("grapheme index"))?;
        let ending = u32::try_from(next_grapheme)
            .map_err(|_| TextPlanError::Arithmetic("grapheme index"))?;
        if paragraph.is_empty() {
            if intrinsic_lines.is_none() {
                let whole = empty_line(seed.span, ending, primary_metrics, input.font_size)?;
                retain_paragraph(
                    environment,
                    &mut analyzed_paragraphs,
                    AnalyzedParagraph {
                        span: seed.span,
                        grapheme_start,
                        grapheme_ends,
                        bidi: None,
                        items: Vec::new(),
                        hyphen_face: None,
                        whole_width: 0,
                        whole: ParagraphWhole::OriginalLine {
                            index: built_lines.len(),
                            advance: whole.advance,
                            ascent: whole.ascent,
                            descent: whole.descent,
                        },
                        wrapping: std::sync::OnceLock::new(),
                        hard_width: std::sync::OnceLock::new(),
                    },
                )?;
                retain_line(environment, &mut built_lines, whole)?;
            }
            continue;
        }
        claim_processing_visits(environment, seed.scalar_count)?;
        #[cfg(test)]
        PARAGRAPH_BIDI_CALLS.with(|calls| calls.set(calls.get() + 1));
        claim_unicode(
            environment,
            (paragraph.len()
                * (size_of::<unicode_bidi::BidiClass>() + size_of::<unicode_bidi::Level>()))
                as u64,
            0,
        )?;
        let bidi = ParagraphBidiData::from(ParagraphBidiInfo::new(paragraph, None));
        let items = selected_paragraph_items(environment, &pair, paragraph)?;
        let hyphen_face = paragraph
            .contains('\u{ad}')
            .then(|| selected_face(&pair, "-", '-'))
            .transpose()?;
        let whole = prepare_line(
            environment,
            input.source,
            seed.span,
            seed.span,
            ending,
            input.font_size,
            &bidi,
            &items,
            hyphen_face,
        )?;
        let whole_width = exact_line_width(environment, &whole)?;
        if intrinsic_lines.is_none() && whole_width <= input.maximum_width_quanta {
            retain_paragraph(
                environment,
                &mut analyzed_paragraphs,
                AnalyzedParagraph {
                    span: seed.span,
                    grapheme_start,
                    grapheme_ends,
                    bidi: Some(bidi),
                    items,
                    hyphen_face,
                    whole_width,
                    whole: ParagraphWhole::OriginalLine {
                        index: built_lines.len(),
                        advance: whole.advance,
                        ascent: whole.ascent,
                        descent: whole.descent,
                    },
                    wrapping: std::sync::OnceLock::new(),
                    hard_width: std::sync::OnceLock::new(),
                },
            )?;
            retain_line(environment, &mut built_lines, whole)?;
            grapheme_start = next_grapheme;
            continue;
        }
        let mut boundaries =
            legal_line_breaks_at_graphemes(paragraph, &grapheme_ends, environment)?;
        claim_collection_count(environment, 1)?;
        super::unicode16::reserve(&mut boundaries, 1, &mut |bytes, work| {
            claim_unicode(environment, bytes, work)
        })?;
        boundaries.insert(0, 0);
        let prefixes = proposal_prefixes(environment, &whole.plan, &boundaries, input.font_size)?;
        let last = boundaries.len() - 1;
        let shape = |start: usize, end: usize| {
            let span = TextSourceSpan::new(
                seed.span.start_byte as usize + boundaries[start],
                seed.span.start_byte as usize + boundaries[end],
            )?;
            prepare_line(
                environment,
                input.source,
                seed.span,
                span,
                ending_grapheme_index(&grapheme_ends, boundaries[end], grapheme_start)?,
                input.font_size,
                &bidi,
                &items,
                hyphen_face,
            )
        };
        if let Some(preferred_lines) = intrinsic_lines {
            requirements.natural_width_quanta =
                requirements.natural_width_quanta.max(ink_width(&whole)?);
            for start in 0..last {
                let width = if last == 1 {
                    whole_width
                } else {
                    exact_line_width(environment, &shape(start, start + 1)?)?
                };
                requirements.hard_width_quanta = requirements.hard_width_quanta.max(width);
            }
            let lines = (preferred_lines / paragraph_count.max(1)).max(1);
            let chunk = last.div_ceil(lines);
            for start in (0..last).step_by(chunk) {
                claim_processing_visits(environment, 1)?;
                let end = start.saturating_add(chunk).min(last);
                let width = prefix_difference(prefixes[end], prefixes[start])?
                    .ceil_i64("preferred width")?;
                requirements.preferred_width_quanta =
                    requirements.preferred_width_quanta.max(width);
            }
        } else {
            // A failed endpoint is the extension barrier for this line start.
            // No failed glyph payload or prior line-start trial table is retained.
            let mut start = 0;
            while start < last {
                let (end, selected) = select_line(
                    environment,
                    &prefixes,
                    start,
                    input.maximum_width_quanta,
                    (start == 0).then_some(whole_width),
                    TextSourceSpan::new(
                        seed.span.start_byte as usize + boundaries[start],
                        seed.span.start_byte as usize + boundaries[start + 1],
                    )?,
                    |end| shape(start, end),
                )?;
                retain_line(environment, &mut built_lines, selected)?;
                start = end;
            }
            environment
                .claim_cache_entries(1)
                .map_err(text_generation_error)?;
            retain_paragraph(
                environment,
                &mut analyzed_paragraphs,
                AnalyzedParagraph {
                    span: seed.span,
                    grapheme_start,
                    grapheme_ends,
                    bidi: Some(bidi),
                    items,
                    hyphen_face,
                    whole_width,
                    whole: ParagraphWhole::Retained(whole),
                    wrapping: std::sync::OnceLock::from(ParagraphWrapping {
                        boundaries,
                        prefixes,
                    }),
                    hard_width: std::sync::OnceLock::new(),
                },
            )?;
        }
        grapheme_start = next_grapheme;
    }
    if intrinsic_lines.is_some() {
        requirements.preferred_width_quanta = requirements
            .preferred_width_quanta
            .max(requirements.hard_width_quanta);
        return Ok(TextMeasurement::Intrinsic(requirements));
    }
    let mut plan = finalize_plan(
        environment,
        input,
        stack,
        pair.provenance,
        line_height_quanta,
        built_lines,
    )?;
    claim_collection_count(environment, 1)?;
    claim_unicode(
        environment,
        (size_of::<TextAnalysis>() + 2 * size_of::<usize>()) as u64,
        1,
    )?;
    plan.1 = Some(std::sync::Arc::new(TextAnalysis {
        environment: environment.clone(),
        original: std::sync::Arc::clone(&plan.0),
        paragraphs: analyzed_paragraphs,
    }));
    Ok(TextMeasurement::Assigned(plan))
}

fn retain_paragraph(
    environment: &PreparedFontEnvironment,
    paragraphs: &mut Vec<AnalyzedParagraph>,
    paragraph: AnalyzedParagraph,
) -> Result<(), TextPlanError> {
    claim_collection_count(environment, 1)?;
    super::unicode16::reserve(paragraphs, 1, &mut |bytes, work| {
        claim_unicode(environment, bytes, work)
    })?;
    paragraphs.push(paragraph);
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct ParagraphSeed {
    span: TextSourceSpan,
    scalar_count: u128,
}

fn select_line(
    environment: &PreparedFontEnvironment,
    prefixes: &[ExactRational],
    start: usize,
    maximum_width_quanta: i64,
    failed_paragraph_width: Option<i64>,
    first_span: TextSourceSpan,
    mut shape: impl FnMut(usize) -> Result<BuiltLine, TextPlanError>,
) -> Result<(usize, BuiltLine), TextPlanError> {
    let last = prefixes.len() - 1;
    let mut end = start + 1;
    for candidate in start + 1..=last {
        claim_processing_visits(environment, 1)?;
        #[cfg(test)]
        TEXT_WORK.with(|work| {
            work.update(|mut work| {
                work.proposal_visits += 1;
                crate::orchestrator::measurements::count(
                    crate::orchestrator::measurements::Counter::TextProposalVisit,
                );
                work
            });
        });
        if prefix_difference(prefixes[candidate], prefixes[start])?.ceil_i64("proposal width")?
            > maximum_width_quanta
        {
            end = candidate.saturating_sub(1).max(start + 1);
            break;
        }
        end = candidate;
    }
    let mut failed = failed_paragraph_width.map(|_| last);
    let mut selected = loop {
        claim_processing_visits(environment, 1)?;
        let (trial, width) = if let Some(width) = failed_paragraph_width.filter(|_| end == last) {
            (None, width)
        } else {
            let trial = shape(end)?;
            let width = exact_line_width(environment, &trial)?;
            (Some(trial), width)
        };
        if width <= maximum_width_quanta {
            break trial.ok_or(TextPlanError::Arithmetic("failed paragraph trial"))?;
        }
        failed = Some(end);
        if end == start + 1 {
            return Err(TextFitMiss {
                source_span: first_span,
                maximum_width_quanta,
                required_width_quanta: width,
            }
            .into());
        }
        end -= 1;
    };
    while end < last {
        claim_processing_visits(environment, 1)?;
        if failed == Some(end + 1) {
            break;
        }
        let trial = shape(end + 1)?;
        if exact_line_width(environment, &trial)? > maximum_width_quanta {
            break;
        }
        selected = trial;
        end += 1;
    }
    Ok((end, selected))
}

fn retain_line(
    environment: &PreparedFontEnvironment,
    lines: &mut Vec<BuiltLine>,
    line: BuiltLine,
) -> Result<(), TextPlanError> {
    claim_collection_count(environment, 2)?;
    super::unicode16::reserve(lines, 1, &mut |bytes, work| {
        claim_unicode(environment, bytes, work)
    })?;
    lines.push(line);
    Ok(())
}

fn ink_width(line: &BuiltLine) -> Result<i64, TextPlanError> {
    line.plan.ink_quanta.map_or(Ok(0), |ink| {
        ink.right
            .checked_sub(ink.left)
            .ok_or(TextPlanError::Arithmetic("line ink span"))
    })
}

fn exact_line_width(
    environment: &PreparedFontEnvironment,
    line: &BuiltLine,
) -> Result<i64, TextPlanError> {
    claim_processing_visits(environment, 1)?;
    #[cfg(test)]
    TEXT_WORK.with(|work| {
        work.update(|mut work| {
            work.exact_trials += 1;
            crate::orchestrator::measurements::count(
                crate::orchestrator::measurements::Counter::TextExactTrial,
            );
            work
        });
    });
    Ok(line.advance.ceil_i64("wrap advance")?.max(ink_width(line)?))
}

fn prefix_difference(
    end: ExactRational,
    start: ExactRational,
) -> Result<ExactRational, TextPlanError> {
    end.add(
        ExactRational::new(
            start
                .numerator
                .checked_neg()
                .ok_or(TextPlanError::Arithmetic("proposal difference"))?,
            start.denominator,
        )?,
        "proposal difference",
    )
}

fn proposal_prefixes(
    environment: &PreparedFontEnvironment,
    whole: &TextLinePlan,
    boundaries: &[usize],
    font_size: u32,
) -> Result<Vec<ExactRational>, TextPlanError> {
    claim_collection_count(environment, boundaries.len() as u128)?;
    let mut prefixes = Vec::new();
    super::unicode16::reserve(&mut prefixes, boundaries.len(), &mut |bytes, visits| {
        claim_unicode(environment, bytes, visits)
    })?;
    prefixes.resize(boundaries.len(), ExactRational::ZERO);
    // Rustybuzz clusters are monotone within each directional run. Traverse in
    // logical order, and defer the entire cluster to its first legal endpoint.
    let mut clusters = Vec::new();
    for (run_order, run) in whole.runs.iter().enumerate() {
        let mut cursor = if run.direction == ShapingDirection::RightToLeft {
            run.glyphs.len()
        } else {
            0
        };
        let rtl = run.direction == ShapingDirection::RightToLeft;
        while if rtl {
            cursor > 0
        } else {
            cursor < run.glyphs.len()
        } {
            let index = if rtl { cursor - 1 } else { cursor };
            let glyph_order = index;
            let cluster = run.glyphs[index].cluster;
            let mut total = 0_i64;
            loop {
                let index = if rtl { cursor - 1 } else { cursor };
                claim_processing_visits(environment, 1)?;
                total = total
                    .checked_add(i64::from(run.glyphs[index].design_x_advance))
                    .ok_or(TextPlanError::Arithmetic("cluster advance"))?;
                if rtl {
                    cursor -= 1;
                } else {
                    cursor += 1;
                }
                if (rtl && cursor == 0) || (!rtl && cursor == run.glyphs.len()) {
                    break;
                }
                let next = if rtl { cursor - 1 } else { cursor };
                if run.glyphs[next].cluster != cluster {
                    break;
                }
            }
            let end = if (rtl && cursor == 0) || (!rtl && cursor == run.glyphs.len()) {
                run.source_span.end_byte
            } else {
                run.glyphs[if rtl { cursor - 1 } else { cursor }].cluster
            };
            let magnitude = total
                .checked_abs()
                .ok_or(TextPlanError::Arithmetic("cluster magnitude"))?;
            claim_collection_count(environment, 1)?;
            super::unicode16::reserve(&mut clusters, 1, &mut |bytes, visits| {
                claim_unicode(environment, bytes, visits)
            })?;
            clusters.push((
                end,
                run_order,
                glyph_order,
                cluster,
                scale_design(magnitude, font_size, run.units_per_em)?,
            ));
        }
    }
    claim_processing_visits(environment, clusters.len() as u128)?;
    clusters.sort_unstable_by_key(|&(end, run, glyph, _, _)| (end, run, glyph));
    let mut boundary = 1;
    for (end, _, _, _, advance) in clusters {
        claim_processing_visits(environment, 1)?;
        let end = (end - whole.source_span.start_byte) as usize;
        while boundaries[boundary] < end {
            claim_processing_visits(environment, 1)?;
            boundary += 1;
        }
        prefixes[boundary] = prefixes[boundary].add(advance, "cluster prefix")?;
    }
    for index in 1..prefixes.len() {
        claim_processing_visits(environment, 1)?;
        prefixes[index] = prefixes[index].add(prefixes[index - 1], "paragraph prefix")?;
    }
    Ok(prefixes)
}

fn claim_processing_visits(
    environment: &PreparedFontEnvironment,
    amount: u128,
) -> Result<(), TextPlanError> {
    claim_text_generation(environment.claim_processing_visits(amount))
}

fn claim_collection_count(
    environment: &PreparedFontEnvironment,
    slots: u128,
) -> Result<(), TextPlanError> {
    claim_text_generation(environment.claim_text_generation_slots(slots))
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
    claim.map_err(text_generation_error)
}

fn text_generation_error(error: canonical::TextGenerationClaimError) -> TextPlanError {
    match error {
        canonical::TextGenerationClaimError::Arithmetic(resource) => {
            TextPlanError::Arithmetic(resource)
        }
        canonical::TextGenerationClaimError::Limit {
            resource,
            limit,
            cumulative_claim,
        } => TextPlanError::Limit {
            resource,
            limit,
            cumulative_claim,
        },
    }
}

fn requested_weight(weight: TextWeight) -> RequestedWeight {
    match weight {
        TextWeight::Normal => RequestedWeight::Normal,
        TextWeight::Semibold => RequestedWeight::Semibold,
        TextWeight::Bold => RequestedWeight::Bold,
    }
}

fn mandatory_paragraph_seeds(
    environment: &PreparedFontEnvironment,
    source: &str,
) -> Result<Vec<ParagraphSeed>, TextPlanError> {
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
        super::unicode16::reserve(&mut paragraphs, 1, &mut |bytes, work| {
            claim_unicode(environment, bytes, work)
        })?;
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
    super::unicode16::reserve(&mut paragraphs, 1, &mut |bytes, work| {
        claim_unicode(environment, bytes, work)
    })?;
    paragraphs.push(ParagraphSeed {
        span: TextSourceSpan::new(start, source.len())?,
        scalar_count,
    });
    Ok(paragraphs)
}

/// Exact Unicode 16 UAX #14 opportunities, restricted to Unicode 16 UAX #29
/// grapheme boundaries before contextual line trials.
fn legal_line_breaks(
    paragraph: &str,
    environment: &PreparedFontEnvironment,
) -> Result<Vec<usize>, TextPlanError> {
    u32::try_from(paragraph.len()).map_err(|_| TextPlanError::SourceTooLong)?;
    claim_unicode(environment, 0, paragraph.chars().count() as u64)?;
    let mut grapheme_ends = Vec::new();
    for (start, grapheme) in paragraph.grapheme_indices(true) {
        super::unicode16::reserve(&mut grapheme_ends, 1, &mut |bytes, work| {
            claim_unicode(environment, bytes, work)
        })?;
        grapheme_ends.push(start + grapheme.len());
    }
    legal_line_breaks_at_graphemes(paragraph, &grapheme_ends, environment)
}

fn claim_unicode(
    environment: &PreparedFontEnvironment,
    bytes: u64,
    work: u64,
) -> Result<(), TextPlanError> {
    if bytes != 0 {
        environment.claim_unicode_bytes(bytes)?;
    }
    if work != 0 {
        claim_processing_visits(environment, u128::from(work))?;
    }
    Ok(())
}

fn paragraph_linebreaks(
    paragraph: &str,
    claim: &mut impl FnMut(u64, u64) -> Result<(), TextPlanError>,
) -> Result<Vec<(usize, super::unicode16::BreakOpportunity)>, TextPlanError> {
    if paragraph.is_empty() {
        return Ok(Vec::new());
    }
    // ASCII letters have identical AL/non-East-Asian line-break properties.
    // LB28 forbids every internal AL/AL boundary, and no other rule counts
    // consecutive AL characters. Evaluate each run once, retaining its source
    // start and the following boundary; shaping always uses the original text.
    claim(0, paragraph.chars().count() as u64)?;
    let mut reduced = String::new();
    claim(paragraph.len() as u64, 0)?;
    reduced
        .try_reserve_exact(paragraph.len())
        .map_err(|_| TextPlanError::Allocation("line-break analysis source"))?;
    let mut offsets = Vec::new();
    let mut alphabetic = false;
    for (offset, scalar) in paragraph.char_indices() {
        let next_alphabetic = scalar.is_ascii_alphabetic();
        if !alphabetic || !next_alphabetic {
            super::unicode16::reserve(&mut offsets, 1, claim)?;
            offsets.push((reduced.len(), offset));
            reduced.push(scalar);
        }
        alphabetic = next_alphabetic;
    }
    super::unicode16::reserve(&mut offsets, 1, claim)?;
    offsets.push((reduced.len(), paragraph.len()));
    let mut opportunities = super::unicode16::linebreaks(&reduced, claim)?;
    let mut mapped = 0;
    for (index, _) in &mut opportunities {
        loop {
            claim(0, 1)?;
            if offsets[mapped].0 >= *index {
                break;
            }
            mapped += 1;
        }
        *index = offsets[mapped].1;
    }
    Ok(opportunities)
}

fn legal_line_breaks_at_graphemes(
    paragraph: &str,
    grapheme_ends: &[usize],
    environment: &PreparedFontEnvironment,
) -> Result<Vec<usize>, TextPlanError> {
    let mut claim = |bytes, work| claim_unicode(environment, bytes, work);
    let opportunities = paragraph_linebreaks(paragraph, &mut claim)?;
    let mut breaks = Vec::new();
    let mut grapheme = 0;
    // Both inputs are sorted and unique. Intersect without sorting or another
    // unbounded binary-search pass, charging before every context comparison.
    for (index, _) in opportunities {
        while grapheme < grapheme_ends.len() {
            claim(0, 1)?;
            if grapheme_ends[grapheme] >= index {
                break;
            }
            grapheme += 1;
        }
        claim(0, 1)?;
        if index > 0 && grapheme_ends.get(grapheme) == Some(&index) {
            claim_collection_count(environment, 1)?;
            super::unicode16::reserve(&mut breaks, 1, &mut claim)?;
            breaks.push(index);
        }
    }
    Ok(breaks)
}

#[test]
fn legal_line_breaks_share_root_work_and_byte_refusals() {
    let environment = PreparedFontEnvironment::synthetic();
    let clone = environment.clone();
    environment
        .claim_processing_visits(
            1_000_000 - environment.resource_usage().unwrap().processing_visits,
        )
        .unwrap();
    assert!(matches!(
        legal_line_breaks("A A", &clone),
        Err(TextPlanError::Limit {
            resource: "root processing visits",
            limit: 1_000_000,
            ..
        })
    ));

    let environment = PreparedFontEnvironment::synthetic();
    let used = environment.resource_usage().unwrap().font_bytes;
    environment
        .claim_unicode_bytes(u64::try_from(536_870_912 - used).unwrap())
        .unwrap();
    assert!(matches!(
        legal_line_breaks("A A", &environment.clone()),
        Err(TextPlanError::Font(FontError::Limit {
            resource: "system-font bytes",
            ..
        }))
    ));
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
            selected_soft_hyphen: None,
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
#[expect(
    clippy::too_many_lines,
    reason = "contextual shaping keeps budget admission, glyph retention, and exact line geometry ordered"
)]
fn prepare_line(
    environment: &PreparedFontEnvironment,
    source: &str,
    paragraph_span: TextSourceSpan,
    line_span: TextSourceSpan,
    ending_grapheme_index: u32,
    font_size: u32,
    bidi: &ParagraphBidiData,
    items: &[ScalarItem],
    hyphen_face: Option<FaceIdentity>,
) -> Result<BuiltLine, TextPlanError> {
    let selected_soft_hyphen = (line_span.end_byte < paragraph_span.end_byte
        && source[line_span.range()].ends_with('\u{ad}'))
    .then(|| {
        TextSourceSpan::new(
            line_span.end_byte as usize - '\u{ad}'.len_utf8(),
            line_span.end_byte as usize,
        )
    })
    .transpose()?;
    let line_local_start = (line_span.start_byte - paragraph_span.start_byte) as usize;
    let line_local_end = (line_span.end_byte - paragraph_span.start_byte) as usize;
    let line_scalar_count = u128::try_from(source[line_span.range()].chars().count())
        .map_err(|_| TextPlanError::Arithmetic("selected line scalar work"))?;
    claim_processing_visits(environment, line_scalar_count)?;
    claim_unicode(
        environment,
        u64::try_from(checked_multiply(
            (line_local_end - line_local_start) as u128,
            (size_of::<unicode_bidi::BidiClass>()
                + 2 * size_of::<unicode_bidi::Level>()
                + size_of::<Range<usize>>()) as u128,
            "selected line bidi storage",
        )?)
        .map_err(|_| TextPlanError::Arithmetic("selected line bidi storage"))?,
        0,
    )?;
    let (levels, bidi_runs) = line_visual_runs(
        bidi,
        &source[paragraph_span.range()],
        line_local_start..line_local_end,
    )?;

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
        let mut groups = itemize_bidi_run(
            environment,
            items,
            paragraph_span.start_byte,
            bidi_run.start + line_local_start..bidi_run.end + line_local_start,
        )?;
        if let Some(soft) = selected_soft_hyphen {
            if let Some(index) = groups
                .iter()
                .position(|group| group.span.range().contains(&(soft.start_byte as usize)))
            {
                let hyphen = ItemGroup {
                    span: soft,
                    face: hyphen_face.ok_or(TextPlanError::MissingPreparedScalar('-' as u32))?,
                    script: groups[index].script,
                };
                // Keep same-face GSUB/GPOS context intact through the replacement.
                if groups[index].face != hyphen.face {
                    if groups[index].span.start_byte == soft.start_byte {
                        groups[index] = hyphen;
                    } else {
                        super::unicode16::reserve(&mut groups, 1, &mut |bytes, work| {
                            claim_unicode(environment, bytes, work)
                        })?;
                        groups[index].span.end_byte = soft.start_byte;
                        groups.insert(index + 1, hyphen);
                    }
                }
            }
        }
        if direction == ShapingDirection::RightToLeft {
            groups.reverse();
        }
        for group in groups {
            let text = &source[group.span.range()];
            #[cfg(test)]
            TEXT_WORK.with(|work| {
                work.update(|mut work| {
                    work.shaping_scalars += text.chars().count();
                    work
                });
            });
            let shaped = environment.shape_run_with_context(
                group.face,
                text,
                direction,
                rustybuzz_script(group.script)?,
                canonical::ShapingContext {
                    pre: &source[line_span.start_byte as usize..group.span.start_byte as usize],
                    post: if selected_soft_hyphen
                        .is_some_and(|soft| soft.start_byte == group.span.end_byte)
                    {
                        "-"
                    } else {
                        &source[group.span.end_byte as usize..line_span.end_byte as usize]
                    },
                    soft_hyphen: selected_soft_hyphen
                        .filter(|soft| group.span.range().contains(&(soft.start_byte as usize)))
                        .map(|soft| (soft.start_byte - group.span.start_byte) as usize),
                },
            )?;
            #[cfg(test)]
            TEXT_WORK.with(|work| {
                work.update(|mut work| {
                    work.shaped_glyphs += shaped.glyphs.len();
                    work
                });
            });
            claim_collection_count(environment, 1)?;
            super::unicode16::reserve(&mut runs, 1, &mut |bytes, work| {
                claim_unicode(environment, bytes, work)
            })?;
            let built = scale_run(
                environment,
                shaped,
                group.span,
                group.script,
                font_size,
                pen_x,
                pen_y,
                levels[bidi_run.start].number(),
            )?;
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
                    Some(current) => current.union(ink)?,
                    None => ink,
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
            selected_soft_hyphen,
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

/// Slice already-resolved paragraph levels, not independently resolved text.
/// The library then applies L1 and L2 with work proportional to this line only.
fn line_visual_runs(
    bidi: &ParagraphBidiData,
    paragraph: &str,
    line: Range<usize>,
) -> Result<(Vec<unicode_bidi::Level>, Vec<Range<usize>>), TextPlanError> {
    let mut original_classes = Vec::new();
    let mut levels = Vec::new();
    original_classes
        .try_reserve_exact(line.len())
        .map_err(|_| TextPlanError::Allocation("line bidi classes"))?;
    levels
        .try_reserve_exact(line.len())
        .map_err(|_| TextPlanError::Allocation("line bidi levels"))?;
    original_classes.extend_from_slice(&bidi.original_classes[line.clone()]);
    levels.extend_from_slice(&bidi.levels[line.clone()]);
    let selected = ParagraphBidiInfo {
        text: &paragraph[line.clone()],
        original_classes,
        levels,
        paragraph_level: bidi.paragraph_level,
        is_pure_ltr: bidi.is_pure_ltr,
    };
    Ok(selected.visual_runs(0..selected.text.len()))
}

fn selected_paragraph_items(
    environment: &PreparedFontEnvironment,
    pair: &ResolvedFontPair,
    paragraph: &str,
) -> Result<Vec<ScalarItem>, TextPlanError> {
    let mut units = canonical::selection_units(paragraph);
    let mut unit = "";
    let mut unit_end = 0;
    let mut items = paragraph_items(environment, paragraph, |start, scalar| {
        if start == unit_end {
            unit = units
                .next()
                .ok_or(TextPlanError::Arithmetic("source unit boundary"))?;
            unit_end += unit.len();
        }
        selected_face(pair, unit, scalar)
    })?;
    // Controls retain their source and shaping context, but do not impose
    // nominal coverage or split a fallback run back onto the primary face.
    let mut following_face = pair.primary;
    for item in items.iter_mut().rev() {
        claim_processing_visits(environment, 1)?;
        let scalar = paragraph[item.start..item.end]
            .chars()
            .next()
            .ok_or(TextPlanError::Arithmetic("source scalar boundary"))?;
        if canonical::is_nonpainting_control(scalar) {
            item.face = following_face;
        } else {
            following_face = item.face;
        }
    }
    let mut preceding_face = None;
    for item in &mut items {
        claim_processing_visits(environment, 1)?;
        let scalar = paragraph[item.start..item.end]
            .chars()
            .next()
            .ok_or(TextPlanError::Arithmetic("source scalar boundary"))?;
        if canonical::is_nonpainting_control(scalar) {
            if let Some(face) = preceding_face {
                item.face = face;
            }
        } else {
            preceding_face = Some(item.face);
        }
    }
    Ok(items)
}

fn paragraph_items(
    environment: &PreparedFontEnvironment,
    paragraph: &str,
    mut face_for: impl FnMut(usize, char) -> Result<FaceIdentity, TextPlanError>,
) -> Result<Vec<ScalarItem>, TextPlanError> {
    let mut items: Vec<ScalarItem> = Vec::new();
    let mut extensions = unicode_script::ScriptExtension::default();
    let mut segment_start = 0;
    let mut segment_script = None;
    for (start, scalar) in paragraph.char_indices() {
        super::unicode16::reserve(&mut items, 1, &mut |bytes, work| {
            claim_unicode(environment, bytes, work)
        })?;
        claim_processing_visits(environment, 1)?;
        let next = extensions.intersection(scalar.script_extension());
        if next.is_empty() {
            let script = segment_script.unwrap_or(Script::Common);
            for item in &mut items[segment_start..] {
                claim_processing_visits(environment, 1)?;
                item.script = script;
            }
            segment_start = items.len();
            extensions = scalar.script_extension();
            segment_script = None;
        } else {
            extensions = next;
        }
        if !matches!(
            scalar.script(),
            Script::Common | Script::Inherited | Script::Unknown
        ) {
            segment_script = Some(scalar.script());
        }
        items.push(ScalarItem {
            start,
            end: start + scalar.len_utf8(),
            face: face_for(start, scalar)?,
            script: scalar.script(),
        });
    }
    let script = segment_script.unwrap_or(Script::Common);
    for item in &mut items[segment_start..] {
        claim_processing_visits(environment, 1)?;
        item.script = script;
    }

    // The key set is bounded by Unicode's bracket repertoire, not source length.
    // Per-key stacks avoid scanning arbitrary nesting for unmatched closers.
    let mut openings: Vec<(char, Vec<usize>)> = Vec::new();
    let mut stack: Vec<(usize, usize)> = Vec::new();
    for (index, scalar) in paragraph.chars().enumerate() {
        claim_processing_visits(environment, 1)?;
        let Some(bracket) = HardcodedBidiData.bidi_matched_opening_bracket(scalar) else {
            // Inherited characters stay with their resolved base, including a
            // closing bracket whose context was restored above. Unrestricted
            // Common characters carry that context forward; Script_Extensions
            // such as the Kana prolonged sound mark remain constrained.
            if index > 0
                && (scalar.script() == Script::Inherited || scalar.script_extension().is_common())
            {
                items[index].script = items[index - 1].script;
            }
            continue;
        };
        let mut key = None;
        for (candidate, (opening, _)) in openings.iter().enumerate() {
            claim_processing_visits(environment, 1)?;
            if *opening == bracket.opening {
                key = Some(candidate);
                break;
            }
        }
        if bracket.is_open {
            let key = if let Some(key) = key {
                key
            } else {
                super::unicode16::reserve(&mut openings, 1, &mut |bytes, work| {
                    claim_unicode(environment, bytes, work)
                })?;
                openings.push((bracket.opening, Vec::new()));
                openings.len() - 1
            };
            let indices = &mut openings[key].1;
            super::unicode16::reserve(indices, 1, &mut |bytes, work| {
                claim_unicode(environment, bytes, work)
            })?;
            super::unicode16::reserve(&mut stack, 1, &mut |bytes, work| {
                claim_unicode(environment, bytes, work)
            })?;
            indices.push(stack.len());
            stack.push((key, index));
        } else if let Some(position) = key.and_then(|key| openings[key].1.last()).copied() {
            items[index].script = items[stack[position].1].script;
            while stack.len() > position {
                claim_processing_visits(environment, 1)?;
                let (key, _) = stack.pop().expect("matched bracket stack");
                openings[key].1.pop();
            }
        }
    }
    Ok(items)
}

fn itemize_bidi_run(
    environment: &PreparedFontEnvironment,
    items: &[ScalarItem],
    paragraph_source_start: u32,
    bidi_run: Range<usize>,
) -> Result<Vec<ItemGroup>, TextPlanError> {
    let mut limits = [0; 2];
    for (limit, boundary) in limits.iter_mut().zip([bidi_run.start, bidi_run.end]) {
        let (mut low, mut high) = (0, items.len());
        while low < high {
            claim_processing_visits(environment, 1)?;
            let middle = low + (high - low) / 2;
            if items[middle].start < boundary {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        *limit = low;
    }
    let [first, last] = limits;
    let mut groups: Vec<ItemGroup> = Vec::new();
    for item in &items[first..last] {
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
        super::unicode16::reserve(&mut groups, 1, &mut |bytes, work| {
            claim_unicode(environment, bytes, work)
        })?;
        groups.push(ItemGroup {
            span: TextSourceSpan::new(absolute_start, absolute_end)?,
            face: item.face,
            script: item.script,
        });
    }
    Ok(groups)
}

fn selected_face(
    pair: &ResolvedFontPair,
    unit: &str,
    scalar: char,
) -> Result<FaceIdentity, TextPlanError> {
    if canonical::is_nonpainting_control(scalar) {
        return Ok(pair.primary);
    }
    let selection = pair
        .selection_for_unit(unit, scalar)
        .map_err(|error| match error {
            FontError::MissingGlyph { .. } => TextPlanError::MissingPreparedScalar(scalar as u32),
            error => TextPlanError::Font(error),
        })?;
    match selection {
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

#[expect(
    clippy::too_many_lines,
    reason = "freeze origins and ink in one pass while retaining exact advances for wrapping"
)]
#[expect(
    clippy::too_many_arguments,
    reason = "root admission accompanies exact run geometry"
)]
fn scale_run(
    environment: &PreparedFontEnvironment,
    shaped: CanonicalShapedRun,
    source_span: TextSourceSpan,
    script: Script,
    font_size: u32,
    line_pen_x: ExactRational,
    line_pen_y: ExactRational,
    bidi_level: u8,
) -> Result<BuiltRun, TextPlanError> {
    let mut glyphs = Vec::new();
    claim_collection_count(environment, shaped.glyphs.len() as u128)?;
    super::unicode16::reserve(&mut glyphs, shaped.glyphs.len(), &mut |bytes, work| {
        claim_unicode(environment, bytes, work)
    })?;
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
        let origin_x = line_pen_x
            .add(pen_x, "glyph origin")?
            .add(x_offset, "glyph origin")?
            .nearest_even_i64("glyph origin")?;
        let origin_y = line_pen_y
            .add(pen_y, "glyph origin")?
            .add(y_offset, "glyph origin")?
            .nearest_even_i64("glyph origin")?;
        let glyph_exact_ink = glyph
            .ink
            .map(|ink| {
                exact_glyph_ink(ink, 0, 0, font_size, shaped.metrics.units_per_em)?.translated(
                    ExactRational::new(i128::from(origin_x), 1)?,
                    ExactRational::new(i128::from(origin_y), 1)?,
                )
            })
            .transpose()?;
        if let Some(ink) = glyph_exact_ink {
            exact_ink = Some(match exact_ink {
                Some(current) => current.union(ink)?,
                None => ink,
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
            glyph_origin_x_quanta: origin_x,
            glyph_origin_y_quanta: origin_y,
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
            bidi_level,
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
        .ceil_i64("face ascender")?;
    let descender = scale_design(
        i64::from(metrics.descender),
        font_size,
        metrics.units_per_em,
    )?
    .floor_i64("face descender")?;
    if ascender <= 0 || descender >= 0 {
        return Err(TextPlanError::Arithmetic("scaled face metrics"));
    }
    Ok((ascender, descender))
}

#[expect(
    clippy::too_many_lines,
    reason = "final geometry and root-admitted payload freezing remain ordered"
)]
fn finalize_plan(
    environment: &PreparedFontEnvironment,
    input: TextPlanInput<'_>,
    stack: &FontStack,
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

    let mut source = String::new();
    let stack_source = stack.css(&environment.font_resources()?)?;
    let stack = stack.copy_with_stage(&environment.font_resources()?, None)?;
    let mut lines = Vec::new();
    claim_unicode(
        environment,
        u64::try_from(
            input
                .source
                .len()
                .checked_add(size_of::<ResolvedTextPayload>() + 2 * size_of::<usize>())
                .ok_or(TextPlanError::Arithmetic("text payload bytes"))?,
        )
        .map_err(|_| TextPlanError::Arithmetic("text payload bytes"))?,
        0,
    )?;
    source
        .try_reserve_exact(input.source.len())
        .map_err(|_| TextPlanError::Allocation("text source"))?;
    super::unicode16::reserve(&mut lines, built_lines.len(), &mut |bytes, work| {
        claim_unicode(environment, bytes, work)
    })?;
    source.push_str(input.source);
    lines.extend(built_lines.into_iter().map(|line| line.plan));
    Ok(ResolvedTextPlan(
        std::sync::Arc::new(ResolvedTextPayload {
            source,
            style: ResolvedTextStyle {
                stack_source,
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
            lines,
            line_pitch_quanta,
            first_baseline_quanta,
            last_baseline_quanta,
            width_quanta,
            height_quanta,
        }),
        None,
    ))
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

fn resolve_line_height(
    line_height: TextLineHeight,
    font_size: u32,
    resources: &canonical::OperationGenerationResources,
) -> Result<i64, TextPlanError> {
    match line_height {
        TextLineHeight::Multiplier(multiplier) => {
            if !multiplier.is_finite() || multiplier <= 0.0 {
                return Err(TextPlanError::InvalidLineHeight);
            }
            Ok(
                crate::theme::RelativeSpacing::new_in(multiplier, resources)?
                    .product(crate::theme::LocalMetric::css_pixels(font_size)?, resources)?
                    .round(super::kernel::SpatialRounding::Ceil, resources)?
                    .quanta(),
            )
        }
        TextLineHeight::ResolvedQuanta(height) if height > 0 => Ok(height),
        TextLineHeight::ResolvedQuanta(_) => Err(TextPlanError::InvalidLineHeight),
    }
}

pub(crate) fn maximum_width_quanta(maximum_width: f64) -> Result<i64, TextPlanError> {
    Ok(super::kernel::LayoutScalar::from_css_pixels(
        maximum_width,
        super::kernel::SpatialRounding::Floor,
    )?
    .quanta())
}

/// Pixel-space view over an immutable operation-owned text plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextBlock {
    plan: ResolvedTextPlan,
}

impl TextBlock {
    pub(crate) const fn plan(&self) -> &ResolvedTextPlan {
        &self.plan
    }
    pub(crate) const fn from_resolved(plan: ResolvedTextPlan) -> Self {
        Self { plan }
    }

    pub(crate) fn lines(&self) -> Vec<&str> {
        self.plan.line_texts()
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

    #[test]
    fn represented_title_leading_uses_the_exact_local_product_on_plan_and_reflow() {
        let environment = PreparedFontEnvironment::synthetic();
        let mut request = input("A\nA", i64::MAX, PhysicalTextAlignment::Start);
        request.font_size = 20;
        request.line_height = TextLineHeight::Multiplier(1.3);
        assert_eq!(1.3_f64.to_bits(), 0x3ff4_cccc_cccc_cccd);
        let plan = environment.plan_text(request.clone()).unwrap();
        assert_eq!(plan.style.line_height_quanta, 26625);
        assert_eq!(plan.line_pitch_quanta, 26625);
        assert_eq!(plan.lines.len(), 2);
        assert_eq!(plan.reflow(request).unwrap().line_pitch_quanta, 26625);
        assert_eq!(maximum_width_quanta(f64::from_bits(1)).unwrap(), 0);
    }

    #[test]
    fn leading_refuses_at_the_real_operation_entry_before_shaping() {
        for byte_limit in [false, true] {
            let environment = PreparedFontEnvironment::synthetic();
            let resources = environment.generation_resources();
            let used_visits = resources.usage().unwrap().1;
            let used_bytes = resources.byte_usage().unwrap();
            resources.set_test_limits(
                if byte_limit { u128::MAX } else { used_visits },
                if byte_limit { used_bytes } else { u64::MAX },
            );
            canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
            let result = environment.plan_text(input("A", i64::MAX, PhysicalTextAlignment::Start));
            assert!(
                matches!(result, Err(TextPlanError::Spacing(crate::Error::Render(message))) if message.contains("admission"))
            );
            canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
            assert_eq!(resources.byte_usage().unwrap(), used_bytes);
        }
    }

    pub(super) fn input(
        source: &str,
        maximum_width_quanta: i64,
        alignment: PhysicalTextAlignment,
    ) -> TextPlanInput<'_> {
        TextPlanInput {
            source,
            stack: canonical::test_sans_stack(),
            weight: TextWeight::Normal,
            font_size: 10,
            line_height: TextLineHeight::Multiplier(1.2),
            maximum_width_quanta,
            alignment,
            paint: Color::rgb(1, 2, 3),
            provenance: TextPlanProvenance {
                pair_ordinal: 0,
                owner: "test".into(),
            },
            exact_override: false,
        }
    }

    pub(super) fn plan(
        source: &str,
        maximum_width_quanta: i64,
        alignment: PhysicalTextAlignment,
    ) -> ResolvedTextPlan {
        PreparedFontEnvironment::synthetic()
            .plan_text(input(source, maximum_width_quanta, alignment))
            .unwrap()
    }

    fn line_texts(plan: &ResolvedTextPlan) -> Vec<&str> {
        plan.lines
            .iter()
            .map(|line| &plan.source[line.source_span.range()])
            .collect()
    }

    #[test]
    fn resource_audit_direct_text_calls_include_reflow_and_intrinsics_without_double_charging() {
        for route in 0..5 {
            let environment = PreparedFontEnvironment::synthetic();
            let request = input("A A A", i64::MAX, PhysicalTextAlignment::Start);
            let plan = environment.plan_text(request.clone()).unwrap();
            let resources = environment.generation_resources();
            assert_eq!(resources.call_cache_usage().0, 1);
            resources
                .claim_calls(999_999 - resources.call_cache_usage().0)
                .unwrap();
            let query = || match route {
                0 => environment.plan_text(request.clone()).map(|_| ()),
                1 => environment.plan_prepared_text(request.clone()).map(|_| ()),
                2 => plan.reflow(request.clone()).map(|_| ()),
                3 => plan.intrinsic_from_analysis(1).map(|_| ()),
                _ => environment
                    .text_intrinsic_requirements(request.clone(), 1)
                    .map(|_| ()),
            };
            query().unwrap();
            assert_eq!(resources.call_cache_usage().0, 1_000_000);
            let before = (
                resources.usage().unwrap(),
                resources.byte_usage().unwrap(),
                resources.call_cache_usage(),
            );
            canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
            assert!(matches!(
                query(),
                Err(TextPlanError::Limit {
                    resource: "measurement/intrinsic call attempts",
                    ..
                })
            ));
            assert_eq!(
                (
                    resources.usage().unwrap(),
                    resources.byte_usage().unwrap(),
                    resources.call_cache_usage()
                ),
                before
            );
            canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        }
    }

    #[test]
    fn resource_audit_lazy_text_cache_refusal_is_fatal_and_hits_do_not_insert() {
        let environment = PreparedFontEnvironment::synthetic();
        let request = input("A A A", i64::MAX, PhysicalTextAlignment::Start);
        let plan = environment.plan_text(request).unwrap();
        let resources = environment.generation_resources();
        resources
            .claim_cache_entries(999_999 - resources.call_cache_usage().1)
            .unwrap();
        assert!(matches!(
            plan.intrinsic_from_analysis(1),
            Err(TextPlanError::Limit {
                resource: "root-local cache entries",
                ..
            })
        ));
        let paragraph = &plan.1.as_ref().unwrap().paragraphs[0];
        assert!(paragraph.wrapping.get().is_some());
        assert!(paragraph.hard_width.get().is_none());
        assert_eq!(resources.call_cache_usage().1, 1_000_000);

        let environment = PreparedFontEnvironment::synthetic();
        let plan = environment
            .plan_text(input("A A A", i64::MAX, PhysicalTextAlignment::Start))
            .unwrap();
        let expected = plan.intrinsic_from_analysis(1).unwrap();
        let resources = environment.generation_resources();
        resources
            .claim_cache_entries(1_000_000 - resources.call_cache_usage().1)
            .unwrap();
        let calls = resources.call_cache_usage().0;
        let repeated = plan.intrinsic_from_analysis(1).unwrap();
        assert_eq!(repeated.hard_width_quanta, expected.hard_width_quanta);
        assert_eq!(resources.call_cache_usage(), (calls + 1, 1_000_000));
    }

    #[test]
    fn intrinsic_widths_are_separate_from_assigned_line_capacity() {
        let environment = PreparedFontEnvironment::synthetic();
        let source = "AAAA AAAA AAAA AAAA";
        let one = environment
            .text_intrinsic_requirements(input(source, i64::MAX, PhysicalTextAlignment::Start), 1)
            .unwrap();
        let two = environment
            .text_intrinsic_requirements(input(source, i64::MAX, PhysicalTextAlignment::Start), 2)
            .unwrap();
        assert_eq!(one.hard_width_quanta, two.hard_width_quanta);
        assert_eq!(one.natural_width_quanta, two.natural_width_quanta);
        assert!(two.hard_width_quanta < two.natural_width_quanta);
        let unwrapped = environment
            .plan_text(input(source, i64::MAX, PhysicalTextAlignment::Start))
            .unwrap();
        assert_eq!(one.natural_width_quanta, unwrapped.width_quanta);
        assert_eq!(
            one.preferred_width_quanta,
            unwrapped.lines[0].logical_advance_quanta
        );
        assert!(two.preferred_width_quanta < one.preferred_width_quanta);
        let measured = environment
            .plan_text(input(
                source,
                two.preferred_width_quanta,
                PhysicalTextAlignment::Start,
            ))
            .unwrap();
        assert_eq!(measured.lines.len(), 2);
        assert_eq!(measured.source, source);
    }

    #[test]
    fn literal_families_reject_untrimmed_names_and_keep_keywords_named() {
        use canonical::{FontFamily, GenericFontFamily};
        for name in [
            " Test Sans",
            "Test Sans ",
            "",
            "A\nB",
            "A\u{7f}B",
            "\u{2003}Test Sans",
        ] {
            assert!(FontFamily::named(name).is_err(), "{name:?}");
        }
        assert!(FontStack::new([]).is_err());
        let named = FontFamily::named("sans-serif").unwrap();
        assert_eq!(named.name(), Some("sans-serif"));
        assert_ne!(named, FontFamily::generic(GenericFontFamily::SansSerif));
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
    fn exact_fit_retains_the_whole_paragraph_without_reshaping() {
        TEXT_WORK.with(|work| work.set(TextWork::default()));
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let plan = plan("A A A", 23_552, PhysicalTextAlignment::Start);
        assert_eq!(line_texts(&plan), ["A A A"]);
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        TEXT_WORK.with(|work| {
            assert_eq!(
                work.get(),
                TextWork {
                    proposal_visits: 0,
                    exact_trials: 1,
                    shaping_scalars: 5,
                    shaped_glyphs: 5,
                }
            );
        });
    }

    #[test]
    fn requested_width_selects_only_its_greedy_lines() {
        for (width, expected, shapes) in [
            (8_704, vec!["A ", "A ", "A"], 6),
            (14_848, vec!["A ", "A A"], 4),
            (17_408, vec!["A A ", "A"], 3),
            (20_000, vec!["A A ", "A"], 3),
            (23_552, vec!["A A A"], 1),
            (i64::MAX, vec!["A A A"], 1),
        ] {
            crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
            let plan = plan("A A A", width, PhysicalTextAlignment::Start);
            assert_eq!(line_texts(&plan), expected);
            assert_eq!(line_texts(&plan).concat(), plan.source);
            crate::font::canonical::FONT_SHAPE_CALLS
                .with(|calls| assert_eq!(calls.get(), shapes, "width {width}"));
        }
    }

    #[test]
    fn unwrapped_shapes_only_one_final_line() {
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let plan = plan("A", i64::MAX, PhysicalTextAlignment::Start);
        assert_eq!(line_texts(&plan), ["A"]);
        assert_eq!(plan.lines[0].ending_grapheme_index, 1);
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 1));
    }

    #[test]
    fn requested_plan_retains_style_and_pair_provenance() {
        let environment = PreparedFontEnvironment::synthetic();
        let mut request = input("A A A", 17_408, PhysicalTextAlignment::End);
        request.provenance.pair_ordinal = 7;
        request.provenance.owner = "requested label".into();
        request.exact_override = true;
        let expected = request.clone();
        let plan = environment.plan_text(request).unwrap();
        assert_eq!(plan.source, expected.source);
        assert_eq!(plan.style.provenance, expected.provenance);
        assert_eq!(plan.pair_provenance.pair_ordinal, 7);
        assert_eq!(&plan.style.stack, expected.stack);
        assert_eq!(plan.style.weight, expected.weight);
        assert_eq!(plan.style.font_size, expected.font_size);
        assert_eq!(plan.style.paint, expected.paint);
        assert_eq!(plan.style.alignment, expected.alignment);
        assert!(plan.style.exact_override);
    }

    #[test]
    fn reflow_accepts_exact_stack_clones_and_rejects_different_literal_profiles() {
        let environment = PreparedFontEnvironment::synthetic();
        let request = input("A A", i64::MAX, PhysicalTextAlignment::Start);
        let plan = environment.plan_text(request.clone()).unwrap();
        let exact = plan.copy_stack().unwrap();
        let mut changed = request;
        changed.stack = &exact;
        assert_eq!(plan.reflow(changed.clone()).unwrap().style.stack, exact);
        for stack in [
            canonical::named_stack(&["TEST SANS"]),
            canonical::named_stack(&["Test Sans", "Test Sans"]),
            canonical::named_stack(&["Test Sans, Test Sans"]),
        ] {
            let mut changed = changed.clone();
            changed.stack = &stack;
            assert!(matches!(
                plan.reflow(changed.clone()),
                Err(TextPlanError::Arithmetic("text analysis profile mismatch"))
            ));
        }
    }

    #[test]
    fn requested_fit_miss_does_not_materialize_an_unwrapped_plan() {
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let error = PreparedFontEnvironment::synthetic()
            .plan_text(input("A A A", 7_000, PhysicalTextAlignment::Start))
            .unwrap_err();
        assert!(matches!(
            error,
            TextPlanError::Fit(TextFitMiss {
                source_span: TextSourceSpan {
                    start_byte: 0,
                    end_byte: 2
                },
                maximum_width_quanta: 7_000,
                required_width_quanta: 8_704,
            })
        ));
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 2));
    }

    #[test]
    fn paragraph_slot_refusal_precedes_all_paragraph_shaping() {
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let environment = PreparedFontEnvironment::synthetic();
        environment
            .claim_text_generation_slots(
                999_995 - environment.generation_resources().usage().unwrap().0,
            )
            .unwrap();

        let error = environment
            .plan_text(input("A A\nA A", i64::MAX, PhysicalTextAlignment::Start))
            .unwrap_err();

        assert!(matches!(
            error,
            TextPlanError::Limit {
                resource: "retained planning records",
                limit: 1_000_000,
                cumulative_claim,
            } if cumulative_claim > 1_000_000
        ));
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
    }

    #[test]
    fn selected_record_refusal_does_not_reshape_the_paragraph() {
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let environment = PreparedFontEnvironment::synthetic();
        let reference = PreparedFontEnvironment::synthetic();
        reference
            .plan_text(input("A", i64::MAX, PhysicalTextAlignment::Start))
            .unwrap();
        let total = reference.generation_resources().usage().unwrap().0;
        environment
            .claim_text_generation_slots(1_000_001 - total)
            .unwrap();
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let error = environment
            .plan_text(input("A", i64::MAX, PhysicalTextAlignment::Start))
            .unwrap_err();
        assert!(matches!(
            error,
            TextPlanError::Limit {
                resource: "retained planning records",
                limit: 1_000_000,
                cumulative_claim: 1_000_001,
            }
        ));
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 1));
    }

    #[test]
    fn selected_plan_budget_is_exact_and_shared_by_environment_clones() {
        let environment = PreparedFontEnvironment::synthetic();
        let clone = environment.clone();
        let reference = PreparedFontEnvironment::synthetic();
        reference
            .plan_text(input("A", i64::MAX, PhysicalTextAlignment::Start))
            .unwrap();
        let total = reference.generation_resources().usage().unwrap().0;
        environment
            .claim_text_generation_slots(1_000_000 - total)
            .unwrap();
        let plan = clone
            .plan_text(input("A", i64::MAX, PhysicalTextAlignment::Start))
            .unwrap();
        assert_eq!(line_texts(&plan), ["A"]);
        assert!(matches!(
            environment.claim_text_generation_slots(1),
            Err(canonical::TextGenerationClaimError::Limit {
                resource: "retained planning records",
                limit: 1_000_000,
                cumulative_claim: 1_000_001,
            })
        ));
    }

    #[test]
    fn paragraph_bidi_is_computed_once_for_proposals_and_selected_lines() {
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
    fn shaping_runs_resolve_common_script_from_paragraph_context() {
        let plan = plan("A A", i64::MAX, PhysicalTextAlignment::Start);
        let runs = &plan.lines[0].runs;

        assert_eq!(
            runs.iter()
                .map(|run| (&plan.source[run.source_span.range()], run.script))
                .collect::<Vec<_>>(),
            [("A A", Script::Latin)]
        );
    }

    #[test]
    fn legal_wrapping_preserves_mandatory_paragraph_context() {
        PARAGRAPH_BIDI_CALLS.with(|calls| calls.set(0));
        let plan = plan("A A\nA A", 8_704, PhysicalTextAlignment::Start);
        assert_eq!(line_texts(&plan), ["A ", "A", "A ", "A"]);
        PARAGRAPH_BIDI_CALLS.with(|calls| assert_eq!(calls.get(), 2));
    }

    #[test]
    fn original_144_repeat_wraps_with_linear_selected_work() {
        TEXT_WORK.with(|work| work.set(TextWork::default()));
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let source = "A ".repeat(144);
        let plan = plan(&source, 87_040, PhysicalTextAlignment::Start);
        assert_eq!(line_texts(&plan).concat(), source);
        assert_eq!(plan.lines.len(), 15);
        // Whole paragraph, selected lines, and the first failing extension.
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 30));
        TEXT_WORK.with(|work| {
            assert_eq!(
                work.get(),
                TextWork {
                    proposal_visits: 158,
                    exact_trials: 30,
                    shaping_scalars: 884,
                    shaped_glyphs: 884,
                }
            );
        });
    }

    #[test]
    fn synthetic_trials_retreat_reuse_failure_and_stop_at_the_first_failed_extension() {
        let template = plan("A", i64::MAX, PhysicalTextAlignment::Start).lines[0].clone();
        for (prefixes, widths, maximum, paragraph_failure, expected_end, expected_trials) in [
            (
                vec![0, 2, 4, 6, 8],
                vec![0, 2, 5, 8, 9],
                6,
                Some(9),
                2,
                vec![3, 2],
            ),
            (
                vec![0, 4, 8, 12, 16],
                vec![0, 2, 3, 9, 4],
                6,
                Some(9),
                2,
                vec![1, 2, 3],
            ),
            (
                vec![0, 1, 2, 3],
                vec![0, 2, 6, 6],
                5,
                Some(6),
                1,
                vec![2, 1],
            ),
        ] {
            let environment = PreparedFontEnvironment::synthetic();
            let prefixes = prefixes
                .into_iter()
                .map(|width| ExactRational::new(width, 1).unwrap())
                .collect::<Vec<_>>();
            let mut trials = Vec::new();
            let (end, _) = select_line(
                &environment,
                &prefixes,
                0,
                maximum,
                paragraph_failure,
                TextSourceSpan::new(0, 1).unwrap(),
                |end| {
                    trials.push(end);
                    let mut line = template.clone();
                    line.ink_quanta = None;
                    Ok(BuiltLine {
                        plan: line,
                        advance: ExactRational::new(widths[end], 1)?,
                        ascent: 0,
                        descent: 0,
                    })
                },
            )
            .unwrap();
            assert_eq!(end, expected_end);
            assert_eq!(trials, expected_trials);
        }
    }

    #[test]
    fn synthetic_trial_work_has_an_exact_shared_budget_and_refuses_before_shaping() {
        let template = plan("A", i64::MAX, PhysicalTextAlignment::Start).lines[0].clone();
        let prefixes = [0, 2, 4, 6, 8].map(|width| ExactRational::new(width, 1).unwrap());
        for remaining in [0, 9] {
            let environment = PreparedFontEnvironment::synthetic();
            environment
                .claim_processing_visits(
                    1_000_000 - remaining - environment.resource_usage().unwrap().processing_visits,
                )
                .unwrap();
            let mut trials = Vec::new();
            let result = select_line(
                &environment.clone(),
                &prefixes,
                0,
                6,
                Some(9),
                TextSourceSpan::new(0, 1).unwrap(),
                |end| {
                    trials.push(end);
                    let mut line = template.clone();
                    line.ink_quanta = None;
                    Ok(BuiltLine {
                        plan: line,
                        advance: ExactRational::new(if end == 3 { 8 } else { 5 }, 1)?,
                        ascent: 0,
                        descent: 0,
                    })
                },
            );
            if remaining == 0 {
                assert!(matches!(
                    result,
                    Err(TextPlanError::Limit {
                        resource: "root processing visits",
                        cumulative_claim: 1_000_001,
                        ..
                    })
                ));
                assert!(trials.is_empty());
            } else {
                assert_eq!(result.unwrap().0, 2);
                assert_eq!(trials, [3, 2]);
                assert!(matches!(
                    environment.claim_processing_visits(1),
                    Err(canonical::TextGenerationClaimError::Limit {
                        cumulative_claim: 1_000_001,
                        ..
                    })
                ));
            }
        }
    }

    #[test]
    fn exact_trial_checks_outward_ink_and_upward_advance_as_one_budget_unit() {
        let template = plan("A", i64::MAX, PhysicalTextAlignment::Start).lines[0].clone();
        for (advance, ink_right) in [
            (ExactRational::new(13, 2).unwrap(), 6),
            (ExactRational::new(6, 1).unwrap(), 7),
        ] {
            let environment = PreparedFontEnvironment::synthetic();
            environment
                .claim_processing_visits(
                    999_999 - environment.resource_usage().unwrap().processing_visits,
                )
                .unwrap();
            let mut line = template.clone();
            line.ink_quanta = Some(TextInkBounds {
                left: 0,
                right: ink_right,
                bottom: 0,
                top: 1,
            });
            let line = BuiltLine {
                plan: line,
                advance,
                ascent: 1,
                descent: 0,
            };
            assert_eq!(exact_line_width(&environment, &line).unwrap(), 7);
            assert!(matches!(
                environment.claim_processing_visits(1),
                Err(canonical::TextGenerationClaimError::Limit {
                    cumulative_claim: 1_000_001,
                    ..
                })
            ));
        }
    }

    #[test]
    fn proposal_storage_refuses_bytes_before_prefix_or_cluster_reservation() {
        let whole = plan("A", i64::MAX, PhysicalTextAlignment::Start).lines[0].clone();
        let prefix_bytes = 2 * size_of::<ExactRational>() as u64;
        let cluster_bytes = size_of::<(u32, usize, usize, u32, ExactRational)>() as u64;
        for remaining in [
            0,
            prefix_bytes - 1,
            prefix_bytes,
            prefix_bytes + cluster_bytes - 1,
            prefix_bytes + cluster_bytes,
        ] {
            let environment = PreparedFontEnvironment::synthetic();
            let before = environment.resource_usage().unwrap();
            environment
                .claim_unicode_bytes(
                    u64::try_from(536_870_912 - before.font_bytes).unwrap() - remaining,
                )
                .unwrap();
            let records = environment.generation_resources().usage().unwrap().0;
            let result = proposal_prefixes(&environment, &whole, &[0, 1], 10);
            if remaining == prefix_bytes + cluster_bytes {
                assert_eq!(result.unwrap().len(), 2);
                assert_eq!(
                    environment.resource_usage().unwrap().font_bytes,
                    536_870_912
                );
            } else {
                assert!(matches!(
                    result,
                    Err(TextPlanError::Font(FontError::Limit {
                        resource: "system-font bytes",
                        ..
                    }))
                ));
                assert_eq!(
                    environment.resource_usage().unwrap().processing_visits
                        - before.processing_visits,
                    u128::from(remaining >= prefix_bytes),
                    "refuse before processing the unadmitted storage"
                );
            }
            assert_eq!(
                environment.generation_resources().usage().unwrap().0 - records,
                2 + u128::from(remaining >= prefix_bytes)
            );
        }
    }

    #[test]
    fn proposal_map_defers_whole_clusters_past_internal_legal_graphemes() {
        let plan = plan("AAA", i64::MAX, PhysicalTextAlignment::Start);
        let mut line = plan.lines[0].clone();
        line.runs[0].glyphs[1].cluster = 0;
        let mut whole = BuiltLine {
            plan: line,
            advance: ExactRational::ZERO,
            ascent: 0,
            descent: 0,
        };
        let prefixes = proposal_prefixes(
            &PreparedFontEnvironment::synthetic(),
            &whole.plan,
            &[0, 1, 2, 3],
            10,
        )
        .unwrap();
        assert_eq!(
            prefixes
                .iter()
                .map(|width| width.ceil_i64("test prefix").unwrap())
                .collect::<Vec<_>>(),
            [0, 0, 12_288, 18_432]
        );
        // Magnitude is taken after summing the cluster, not per glyph.
        whole.plan.runs[0].glyphs[0].design_x_advance = -600;
        let prefixes = proposal_prefixes(
            &PreparedFontEnvironment::synthetic(),
            &whole.plan,
            &[0, 1, 2, 3],
            10,
        )
        .unwrap();
        assert_eq!(
            prefixes
                .iter()
                .map(|width| width.ceil_i64("test prefix").unwrap())
                .collect::<Vec<_>>(),
            [0, 0, 0, 6144]
        );
    }

    #[test]
    fn ordinary_caption_workloads_render_in_adequate_public_boxes() {
        let sentence = "Measured responses remained stable across independent trials under controlled conditions. ";
        let captions = [
            "A ".repeat(144),
            sentence.repeat(20),
            [sentence.repeat(7), sentence.repeat(7), sentence.repeat(6)].join("\n"),
            "Measured ".repeat(2048),
        ];
        assert_eq!(captions[1].split_whitespace().count(), 200);
        assert_eq!(captions[2].split_whitespace().count(), 200);
        for (index, caption) in captions.iter().enumerate() {
            let environment = PreparedFontEnvironment::discover_system().unwrap();
            let mut request = input(
                caption,
                768 * CSS_PIXEL_QUANTA,
                PhysicalTextAlignment::Start,
            );
            let stack = canonical::builtin_font_stack();
            request.stack = &stack;
            request.font_size = 15;
            TEXT_WORK.with(|work| work.set(TextWork::default()));
            let planned = environment.plan_text(request).unwrap();
            assert_eq!(planned.line_texts().concat(), caption.replace('\n', ""));
            assert!(planned.lines.len() > 1);
            TEXT_WORK.with(|work| {
                let work = work.get();
                assert!(
                    work.shaping_scalars <= 4 * caption.chars().count(),
                    "{work:?}"
                );
                assert!(work.exact_trials <= 2 * planned.lines.len() + 1, "{work:?}");
                assert!(
                    work.proposal_visits
                        <= caption.split_whitespace().count() + planned.lines.len(),
                    "{work:?}"
                );
                eprintln!(
                    "caption {index}: {work:?}, selected lines {}",
                    planned.lines.len()
                );
            });
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("caption.svg");
            crate::Plot::scatter(
                crate::NumericValues::new("x", [0.0, 1.0]),
                crate::NumericValues::new("y", [0.0, 1.0]),
            )
            .caption(caption)
            .size(
                if index == 3 { 1600 } else { 800 },
                if index == 3 { 8000 } else { 900 },
            )
            .svg_preserve_text(&path)
            .unwrap();
            let svg = std::fs::read_to_string(path).unwrap();
            let words = svg
                .split("<text ")
                .skip(1)
                .flat_map(|element| {
                    element
                        .split_once('>')
                        .unwrap()
                        .1
                        .split_once("</text>")
                        .unwrap()
                        .0
                        .split_whitespace()
                })
                .filter(|word| *word == if index == 0 { "A" } else { "Measured" })
                .count();
            assert_eq!(
                words,
                match index {
                    0 => 144,
                    3 => 2048,
                    _ => 20,
                }
            );
        }
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
