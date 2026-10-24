//! Built-in construction data and private operation-local appearance resolution.

#![deny(missing_docs)]

use std::sync::{Arc, Mutex, OnceLock};

use sha2::{Digest, Sha256};

use super::{
    AnnotationTextRoles, AnnotationTheme, AxisSpacing, AxisTextRoles, AxisTheme,
    CompositionTextRoles, ContrastTextRoles, FrameSides, FrameTheme, GuideTextRoles, LegendTheme,
    MarkTheme, RootTextRoles, StrokeTheme, Theme, TickTheme, TypographyTheme,
};
use crate::font::canonical::PreparedFontEnvironment;
use crate::layout::{
    PhysicalTextAlignment, ResolvedTextPlan, TextLineHeight, TextPlanInput, TextPlanProvenance,
};
use crate::render::backend::StrokePattern;
use crate::theme::{color_from_rgb, Color};
use crate::theme::{
    AccentPalette, ColorScales, DivergingGradient, OpaqueColor, QualitativePalette,
    SequentialGradient,
};
use crate::{Error, TextWeight};

/// Built-in axis tick-label font size in actual CSS pixels.
const TICK_FONT_SIZE: u32 = 16;

const MUTED_TEXT: (u8, u8, u8) = (80, 80, 80);
const SIGNIFICANCE_FONT_SIZE: u32 = 16;

#[cfg(test)]
thread_local! {
    static FONT_ENVIRONMENT_DISCOVERIES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// A theme's full resolved visual specification. Layout and renderers consume
/// this value to produce concrete backend styles and layout decisions.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedTheme {
    appearance: Theme,
    /// An explicit Plot inside Canvas composes over the Canvas owner's surface.
    pub(crate) canvas_surface_override: Option<Color>,
    font_environment: Arc<OnceLock<Result<PreparedFontEnvironment, String>>>,
    text_queries: Arc<Mutex<TextQueryCache>>,
    /// Complete guide measurements share this operation's prepared font scope.
    pub(crate) guide_measurements: Arc<Mutex<crate::decorations::legend::GuideMeasurements>>,
}

impl std::ops::Deref for ResolvedTheme {
    type Target = Theme;
    fn deref(&self) -> &Theme {
        &self.appearance
    }
}

impl std::ops::DerefMut for ResolvedTheme {
    fn deref_mut(&mut self) -> &mut Theme {
        &mut self.appearance
    }
}

#[derive(Debug, Default)]
struct TextQueryCache {
    profiles: Vec<usize>,
    entries: Vec<CachedTextQuery>,
    exact: Vec<TextQueryIndex>,
    shaping: Vec<TextQueryIndex>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TextQueryIndex {
    fingerprint: [u8; 32],
    entry: usize,
}

impl TextQueryCache {
    fn profile_entry(&self, input: &TextPlanInput<'_>) -> Option<usize> {
        let (mut low, mut high) = (0, self.profiles.len());
        while low < high {
            let middle = low + (high - low) / 2;
            let index = self.profiles[middle];
            match self.compare_profile(index, input) {
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
                std::cmp::Ordering::Equal => {
                    if self.entries[index].appearance.stack.same(input.stack) {
                        return Some(index);
                    }
                    // Distinct literal stacks can share source/size/weight. They
                    // stay distinct; inspect only this equal-source interval.
                    for direction in [false, true] {
                        let mut position = middle;
                        loop {
                            position = if direction {
                                position + 1
                            } else {
                                let Some(previous) = position.checked_sub(1) else {
                                    break;
                                };
                                previous
                            };
                            if position == self.profiles.len() {
                                break;
                            }
                            let index = self.profiles[position];
                            if self.compare_profile(index, input) != std::cmp::Ordering::Equal {
                                break;
                            }
                            if self.entries[index].appearance.stack.same(input.stack) {
                                return Some(index);
                            }
                        }
                    }
                    return None;
                }
            }
        }
        None
    }

    fn compare_profile(&self, index: usize, input: &TextPlanInput<'_>) -> std::cmp::Ordering {
        let entry = &self.entries[index];
        let metadata = (
            entry.source.len(),
            entry.appearance.font_size,
            entry.appearance.weight as u8,
        )
            .cmp(&(input.source.len(), input.font_size, input.weight as u8));
        if metadata != std::cmp::Ordering::Equal {
            return metadata;
        }
        if std::ptr::eq(entry.source.as_str(), input.source) {
            return std::cmp::Ordering::Equal;
        }
        for index in 0..input.source.len() {
            let ordering = entry.source.as_bytes()[index].cmp(&input.source.as_bytes()[index]);
            if ordering != std::cmp::Ordering::Equal {
                return ordering;
            }
        }
        std::cmp::Ordering::Equal
    }

    fn profile_insertion_point(&self, input: &TextPlanInput<'_>) -> usize {
        let (mut low, mut high) = (0, self.profiles.len());
        while low < high {
            let middle = low + (high - low) / 2;
            if self.compare_profile(self.profiles[middle], input) == std::cmp::Ordering::Less {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        low
    }

    fn query_key(&self, input: &TextPlanInput<'_>, preferred_lines: Option<usize>) -> TextQueryKey {
        let shaping_entry = self.profile_entry(input);

        let shaping_state = if let Some(index) = shaping_entry {
            self.entries[index].shaping_state.clone()
        } else {
            CachedTextQuery::shaping_state(input)
        };
        TextQueryKey {
            shaping_entry,
            shaping_fingerprint: shaping_state.clone().finalize().into(),
            fingerprint: CachedTextQuery::presentation_fingerprint(
                shaping_state.clone(),
                input,
                preferred_lines,
            ),
            shaping_state,
            observed_entries: self.entries.len(),
        }
    }
    fn lower_bound(index: &[TextQueryIndex], fingerprint: [u8; 32]) -> usize {
        let (mut low, mut high) = (0, index.len());
        while low < high {
            let middle = low + (high - low) / 2;
            if index[middle].fingerprint < fingerprint {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        low
    }

    fn find_query(
        &self,
        index: &[TextQueryIndex],
        fingerprint: [u8; 32],
        key: (&TextPlanInput<'_>, Option<usize>),
        exact: bool,

        shaping_entry: Option<usize>,
    ) -> Option<usize> {
        let (input, preferred_lines) = key;
        let start = Self::lower_bound(index, fingerprint);
        for candidate in &index[start..] {
            if candidate.fingerprint != fingerprint {
                break;
            }
            let query = &self.entries[candidate.entry];
            if shaping_entry == Some(query.shaping_entry)
                && (!exact
                    || (query.width == input.maximum_width_quanta
                        && query.matches_presentation(input, preferred_lines)))
            {
                return Some(candidate.entry);
            }
        }
        None
    }

    fn reserve<T>(values: &mut Vec<T>) -> Result<(), Error> {
        if values.len() < values.capacity() {
            return Ok(());
        }
        let capacity = values
            .len()
            .checked_add(1)
            .and_then(|required| {
                values
                    .capacity()
                    .checked_mul(2)
                    .map(|doubled| required.max(doubled))
            })
            .ok_or_else(|| Error::Render("text query cache capacity overflow".into()))?;

        values
            .try_reserve_exact(capacity - values.len())
            .map_err(|_| Error::Render("cannot reserve text query cache".into()))
    }

    fn insertion_point(index: &[TextQueryIndex], value: TextQueryIndex) -> usize {
        let (mut low, mut high) = (0, index.len());
        while low < high {
            let middle = low + (high - low) / 2;
            if index[middle] < value {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        low
    }
}

#[derive(Debug)]
struct CachedTextQuery {
    shaping_entry: usize,
    shaping_state: Sha256,
    source: Arc<String>,
    appearance: TextTheme,
    alignment: PhysicalTextAlignment,
    owner: String,
    pair_ordinal: u32,
    exact_override: bool,
    width: i64,
    preferred_lines: Option<usize>,
    unwrapped_width: Option<i64>,
    result: CachedTextResult,
}

#[derive(Debug, Clone)]
enum CachedTextResult {
    Plan(ResolvedTextPlan),
    /// Lookup-only result: reuse analysis, never this plan's width-specific lines.
    Reflow(ResolvedTextPlan),
    PreferredWidth(i64),
}

struct TextQueryKey {
    observed_entries: usize,
    shaping_entry: Option<usize>,
    shaping_state: Sha256,
    shaping_fingerprint: [u8; 32],
    fingerprint: [u8; 32],
}

impl CachedTextQuery {
    #[cfg(test)]
    fn fingerprints(
        input: &TextPlanInput<'_>,
        preferred_lines: Option<usize>,
    ) -> ([u8; 32], [u8; 32]) {
        let state = Self::shaping_state(input);
        (
            state.clone().finalize().into(),
            Self::presentation_fingerprint(state, input, preferred_lines),
        )
    }

    fn shaping_state(input: &TextPlanInput<'_>) -> Sha256 {
        let mut hash = Sha256::new();
        // The complete, unnormalized source fixes paragraph bidi and scripts;
        // language and Unicode policy are fixed for this renderer. The memo is
        // shared only together with its immutable operation font environment.
        hash.update((input.source.len() as u64).to_le_bytes());
        hash.update(input.source.as_bytes());
        input.stack.hash_into(&mut hash);

        hash.update(input.font_size.to_le_bytes());
        hash.update([match input.weight {
            TextWeight::Normal => 0,
            TextWeight::Semibold => 1,
            TextWeight::Bold => 2,
        }]);
        hash
    }

    fn presentation_fingerprint(
        mut hash: Sha256,
        input: &TextPlanInput<'_>,
        preferred_lines: Option<usize>,
    ) -> [u8; 32] {
        hash.update((input.provenance.owner.len() as u64).to_le_bytes());
        hash.update(input.provenance.owner.as_bytes());
        hash.update(input.provenance.pair_ordinal.to_le_bytes());
        hash.update(input.maximum_width_quanta.to_le_bytes());
        hash.update([match input.alignment {
            PhysicalTextAlignment::Start => 0,
            PhysicalTextAlignment::Center => 1,
            PhysicalTextAlignment::End => 2,
        }]);
        match input.line_height {
            TextLineHeight::Multiplier(value) => {
                hash.update([0]);
                hash.update(value.to_bits().to_le_bytes());
            }
            TextLineHeight::ResolvedQuanta(value) => {
                hash.update([1]);
                hash.update(value.to_le_bytes());
            }
        }
        hash.update([
            input.paint.red(),
            input.paint.green(),
            input.paint.blue(),
            input.paint.get_alpha(),
            u8::from(input.exact_override),
        ]);
        hash.update([u8::from(preferred_lines.is_some())]);
        if let Some(lines) = preferred_lines {
            hash.update((lines as u64).to_le_bytes());
        }
        hash.finalize().into()
    }

    fn matches_presentation(
        &self,
        input: &TextPlanInput<'_>,
        preferred_lines: Option<usize>,
    ) -> bool {
        self.appearance.paint == input.paint
            && input.line_height
                == TextLineHeight::Multiplier(self.appearance.line_height_multiplier)
            && self.alignment == input.alignment
            && self.pair_ordinal == input.provenance.pair_ordinal
            && self.exact_override == input.exact_override
            && self.preferred_lines == preferred_lines
            && crate::font::canonical::same_bytes(&self.owner, &input.provenance.owner)
    }

    fn matches_width(&self, width: i64) -> bool {
        self.width == width
            || self
                .unwrapped_width
                .is_some_and(|required| required <= width)
    }
}

impl PartialEq for ResolvedTheme {
    fn eq(&self, other: &Self) -> bool {
        self.non_text_eq(other)
            && self.axes == other.axes
            && self.legend == other.legend
            && self.composition == other.composition
            && self.typography == other.typography
    }
}

impl ResolvedTheme {
    pub(crate) fn new(appearance: Theme) -> Result<Self, Error> {
        appearance.validate()?;
        Ok(Self {
            appearance,
            canvas_surface_override: None,
            font_environment: Arc::new(OnceLock::new()),
            text_queries: Arc::default(),
            guide_measurements: Arc::default(),
        })
    }

    fn non_text_eq(&self, other: &Self) -> bool {
        self.color_scales == other.color_scales
            && self.surfaces == other.surfaces
            && self.canvas_surface_override == other.canvas_surface_override
            && self.marks == other.marks
            && self.annotations == other.annotations
    }
}

impl ResolvedTheme {
    pub(crate) fn legend_outside_gap(&self) -> Result<f64, Error> {
        self.legend
            .outside_gap_pixels(self.typography.guides.legend_label.get_font_size())
    }
    pub(crate) fn legend_shared_gap(&self) -> Result<f64, Error> {
        self.legend
            .shared_gap_pixels(self.typography.guides.legend_label.get_font_size())
    }
    pub(crate) fn same_validated(&self, other: &Self) -> Result<bool, Error> {
        if !self.axes.same_validated(&other.axes)? {
            return Ok(false);
        }
        if !self.legend.same(&other.legend) {
            return Ok(false);
        }
        if !self.composition.same(&other.composition) {
            return Ok(false);
        }
        if !self.non_text_eq(other) {
            return Ok(false);
        }
        for (left, right) in self
            .typography
            .roles()
            .into_iter()
            .zip(other.typography.roles())
        {
            if !left.same(right) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Initialize this operation's canonical system-font environment.
    pub(crate) fn prepare_font_environment(&self) -> Result<(), Error> {
        self.prepared_font_environment().map(|_| ())
    }

    pub(crate) fn prepare_font_environment_from_census(
        &self,
        census: &mut crate::layout::PreparedTextCensus,
        presence: Vec<u64>,
        prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<(), Error> {
        self.font_environment
            .get_or_init(|| {
                #[cfg(test)]
                FONT_ENVIRONMENT_DISCOVERIES.with(|calls| calls.set(calls.get() + 1));
                census
                    .prepare_system_fonts(presence, prepared_stage)
                    .map_err(|error| error.to_string())
            })
            .as_ref()
            .map(|_| ())
            .map_err(|error| Error::Render(format!("font preflight: {error}")))
    }

    /// Make this separately resolved theme use the root operation's environment.
    pub(crate) fn share_font_environment_from(&mut self, root: &Self) {
        self.font_environment = Arc::clone(&root.font_environment);
        self.text_queries = Arc::clone(&root.text_queries);
        self.guide_measurements = Arc::clone(&root.guide_measurements);
    }

    /// Plan one foundational appearance within an exact finite pixel width.
    #[cfg(test)]
    pub(crate) fn plan_text(
        &self,
        source: &str,
        appearance: &TextTheme,
        maximum_width: f64,
        alignment: PhysicalTextAlignment,
        owner: &str,
        exact_override: bool,
    ) -> Result<ResolvedTextPlan, Error> {
        if !maximum_width.is_finite() || maximum_width <= 0.0 {
            return Err(Error::Render(format!(
                "{owner} has no positive finite width available for text layout"
            )));
        }
        let maximum_width_quanta = crate::layout::maximum_width_quanta(maximum_width)
            .map_err(|error| Error::Render(format!("{owner} text planning: {error}")))?;
        self.plan_text_quanta(
            source,
            appearance,
            maximum_width_quanta,
            alignment,
            owner,
            exact_override,
        )
    }

    /// Common label width from contextual legal spans and the assigned row capacity.
    #[expect(
        clippy::cast_precision_loss,
        reason = "axis label requirements are normalized layout quanta consumed by pixel adapters"
    )]
    pub(crate) fn intrinsic_tick_label_width<'a>(
        &self,
        labels: impl ExactSizeIterator<Item = &'a str>,
        available_height: f64,
    ) -> Result<f64, crate::Error> {
        self.prepare_font_environment()?;
        let environment = self.prepared_font_environment()?;

        let appearance = self.typography.get_axes().get_tick_label();
        let height = crate::layout::maximum_width_quanta(available_height)
            .map_err(|error| crate::Error::Render(format!("axis row height: {error}")))?;
        let count = i64::try_from(labels.len().max(1))
            .map_err(|_| crate::Error::Render("axis row count overflowed".into()))?;
        let pitch = crate::layout::maximum_width_quanta(appearance.line_pitch_pixels())
            .map_err(|error| crate::Error::Render(format!("axis row pitch: {error}")))?;
        if height < 0 || pitch <= 0 {
            return Err(crate::Error::Render(
                "axis row allocation requires nonnegative height and positive text pitch".into(),
            ));
        }
        let preferred_lines = usize::try_from(((height / count - 4096).max(0) / pitch).max(1))
            .map_err(|_| crate::Error::Render("axis row capacity overflowed".into()))?;
        let mut width = 1_i64;
        for source in labels {
            let mut input = crate::layout::TextPlanInput {
                source,
                stack: &appearance.stack,
                weight: appearance.weight,
                font_size: appearance.font_size,
                line_height: crate::layout::TextLineHeight::Multiplier(
                    appearance.line_height_multiplier,
                ),
                maximum_width_quanta: i64::MAX,
                alignment: PhysicalTextAlignment::End,
                paint: appearance.paint,
                provenance: crate::layout::TextPlanProvenance {
                    pair_ordinal: 0,
                    owner: "y-axis tick label".into(),
                },
                exact_override: false,
            };
            input.provenance.pair_ordinal = environment
                .admit_text(&appearance.stack, appearance.weight.into(), source, None)
                .map_err(|error| {
                    Error::Render(format!("axis intrinsic text admission: {error}"))
                })?;

            let (cached, query_key) = self.cached_text_query(&input, Some(preferred_lines))?;
            let preferred_width = if let Some(CachedTextResult::PreferredWidth(width)) = cached {
                width
            } else {
                let analysis = if let Some(CachedTextResult::Reflow(plan)) = cached {
                    plan
                } else {
                    let plan = environment.plan_text(input.clone()).map_err(|error| {
                        crate::Error::Render(format!("axis intrinsic text analysis: {error}"))
                    })?;
                    self.cache_text_query(
                        &input,
                        appearance,
                        None,
                        CachedTextResult::Plan(plan.clone()),
                        TextQueryKey {
                            fingerprint: CachedTextQuery::presentation_fingerprint(
                                query_key.shaping_state.clone(),
                                &input,
                                None,
                            ),
                            shaping_state: query_key.shaping_state.clone(),
                            ..query_key
                        },
                    )?;
                    plan
                };
                let requirements =
                    analysis
                        .intrinsic_from_analysis(preferred_lines)
                        .map_err(|error| {
                            crate::Error::Render(format!(
                                "axis intrinsic text requirements: {error}"
                            ))
                        })?;
                self.cache_text_query(
                    &input,
                    appearance,
                    Some(preferred_lines),
                    CachedTextResult::PreferredWidth(requirements.preferred_width_quanta),
                    query_key,
                )?;
                requirements.preferred_width_quanta
            };
            width = width.max(preferred_width);
        }
        Ok(width as f64 / 1024.0)
    }

    /// Measure one foundational appearance without introducing a wrap boundary.
    pub(crate) fn measure_text(
        &self,
        source: &str,
        appearance: &TextTheme,
        alignment: PhysicalTextAlignment,
        owner: &str,
        exact_override: bool,
    ) -> Result<ResolvedTextPlan, Error> {
        self.plan_text_quanta(
            source,
            appearance,
            i64::MAX,
            alignment,
            owner,
            exact_override,
        )
    }

    /// Select a complete text role over one final variable-fill paint before
    /// font preparation, using owner-local panel/root surfaces and backdrop.
    pub(crate) fn contrasting_text(&self, fill: Color) -> Result<&TextTheme, Error> {
        super::contrast::select(
            fill,
            self.surfaces.get_plot_panel(),
            self.canvas_surface_override
                .unwrap_or(self.surfaces.get_canvas_root()),
            self.annotations.get_planning_backdrop().into(),
            self.typography.get_annotations().get_contrast().get_light(),
            self.typography.get_annotations().get_contrast().get_dark(),
        )
    }

    pub(crate) fn measure_legacy_appearance_text(
        &self,
        source: &str,
        appearance: &TextTheme,
        alignment: PhysicalTextAlignment,
        owner: &str,
        exact_override: bool,
    ) -> Result<ResolvedTextPlan, Error> {
        self.plan_text_quanta_scoped(
            source,
            appearance,
            i64::MAX,
            alignment,
            owner,
            exact_override,
            &crate::layout::fit::ROOT_SCOPE,
        )
        .map_err(crate::layout::fit::PlanError::into_error)
    }

    fn plan_text_quanta(
        &self,
        source: &str,
        appearance: &TextTheme,
        maximum_width_quanta: i64,
        alignment: PhysicalTextAlignment,
        owner: &str,
        exact_override: bool,
    ) -> Result<ResolvedTextPlan, Error> {
        self.plan_text_quanta_scoped(
            source,
            appearance,
            maximum_width_quanta,
            alignment,
            owner,
            exact_override,
            &crate::layout::fit::ROOT_SCOPE,
        )
        .map_err(crate::layout::fit::PlanError::into_error)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the shared adapter preserves every immutable text-plan input and the consuming fit scope"
    )]
    pub(crate) fn plan_text_quanta_scoped(
        &self,
        source: &str,
        appearance: &TextTheme,
        maximum_width_quanta: i64,
        alignment: PhysicalTextAlignment,
        owner: &str,
        exact_override: bool,
        scope: &crate::layout::fit::FitScope,
    ) -> Result<ResolvedTextPlan, crate::layout::fit::PlanError> {
        let mut input = TextPlanInput {
            source,
            stack: appearance.get_stack(),
            weight: appearance.get_weight(),
            font_size: appearance.get_font_size(),
            line_height: TextLineHeight::Multiplier(appearance.get_line_height()),
            maximum_width_quanta,
            alignment,
            paint: appearance.get_color(),
            provenance: TextPlanProvenance {
                pair_ordinal: 0,
                owner: owner.to_owned(),
            },
            exact_override,
        };
        let environment = self.prepared_font_environment()?;
        input.provenance.pair_ordinal = environment
            .admit_text(
                appearance.get_stack(),
                appearance.get_weight().into(),
                source,
                None,
            )
            .map_err(|error| Error::Render(format!("{owner} text admission: {error}")))?;

        let (cached, query_key) = self.cached_text_query(&input, None)?;

        let result = match cached {
            Some(CachedTextResult::Plan(plan)) => return Ok(plan),
            Some(CachedTextResult::Reflow(plan)) => {
                let (source, stack) = plan
                    .analysis_profile()
                    .map_err(|error| Error::Render(error.to_string()))?;
                plan.reflow(TextPlanInput {
                    source,
                    stack,
                    ..input.clone()
                })
            }
            _ => environment.plan_text(input.clone()),
        };
        let plan = result.map_err(|error| match error {
            crate::layout::TextPlanError::Fit(miss) => scope
                .failure(
                    crate::layout::fit::FitKind::Text,
                    format!("{owner} text planning: {miss}"),
                )
                .into(),
            error => crate::layout::fit::PlanError::Fatal(Error::Render(format!(
                "{owner} text planning: {error}"
            ))),
        })?;
        self.cache_text_query(
            &input,
            appearance,
            None,
            CachedTextResult::Plan(plan.clone()),
            query_key,
        )?;

        Ok(plan)
    }

    fn cached_text_query(
        &self,
        input: &TextPlanInput<'_>,
        preferred_lines: Option<usize>,
    ) -> Result<(Option<CachedTextResult>, TextQueryKey), Error> {
        let cache = self
            .text_queries
            .lock()
            .map_err(|_| Error::Render("text query cache lock poisoned".into()))?;
        let query_key = cache.query_key(input, preferred_lines);
        let shaping_fingerprint = query_key.shaping_fingerprint;
        let fingerprint = query_key.fingerprint;
        let key = (input, preferred_lines);
        if let Some(index) = cache.find_query(
            &cache.exact,
            fingerprint,
            key,
            true,
            query_key.shaping_entry,
        ) {
            return Ok((Some(cache.entries[index].result.clone()), query_key));
        }
        if let Some(index) = cache.find_query(
            &cache.shaping,
            shaping_fingerprint,
            key,
            false,
            query_key.shaping_entry,
        ) {
            let query = &cache.entries[index];

            if query.matches_width(input.maximum_width_quanta)
                && query.matches_presentation(input, preferred_lines)
            {
                return Ok((Some(query.result.clone()), query_key));
            }
            if let CachedTextResult::Plan(plan) = &query.result {
                return Ok((Some(CachedTextResult::Reflow(plan.clone())), query_key));
            }
        }
        Ok((None, query_key))
    }

    #[expect(
        clippy::too_many_lines,
        reason = "key proof, storage reservation, index insertion and payload freezing stay in one cache transaction"
    )]
    fn cache_text_query(
        &self,
        input: &TextPlanInput<'_>,
        appearance: &TextTheme,
        preferred_lines: Option<usize>,
        result: CachedTextResult,
        mut query_key: TextQueryKey,
    ) -> Result<(), Error> {
        let mut cache = self
            .text_queries
            .lock()
            .map_err(|_| Error::Render("text query cache lock poisoned".into()))?;
        // This root-local memo is optional and bounded, never an all-width table.
        if cache.entries.len() == 256 {
            return Ok(());
        }

        if query_key.shaping_entry.is_none() && cache.entries.len() != query_key.observed_entries {
            query_key.shaping_entry = cache.profile_entry(input);
        }

        let shaping_fingerprint = query_key.shaping_fingerprint;
        let fingerprint = query_key.fingerprint;
        let key = (input, preferred_lines);
        if cache
            .find_query(
                &cache.exact,
                fingerprint,
                key,
                true,
                query_key.shaping_entry,
            )
            .is_some()
        {
            return Ok(());
        }
        let new_shaping = matches!(&result, CachedTextResult::Plan(_))
            && cache
                .find_query(
                    &cache.shaping,
                    shaping_fingerprint,
                    key,
                    false,
                    query_key.shaping_entry,
                )
                .is_none();

        TextQueryCache::reserve(&mut cache.entries)?;
        TextQueryCache::reserve(&mut cache.exact)?;
        if new_shaping {
            TextQueryCache::reserve(&mut cache.shaping)?;
            TextQueryCache::reserve(&mut cache.profiles)?;
        }
        let exact_record = TextQueryIndex {
            fingerprint,
            entry: cache.entries.len(),
        };
        let shaping_record = TextQueryIndex {
            fingerprint: shaping_fingerprint,
            entry: cache.entries.len(),
        };
        let exact_position = TextQueryCache::insertion_point(&cache.exact, exact_record);
        let shaping_position = if new_shaping {
            Some(TextQueryCache::insertion_point(
                &cache.shaping,
                shaping_record,
            ))
        } else {
            None
        };
        let profile_position = if new_shaping {
            Some(cache.profile_insertion_point(input))
        } else {
            None
        };
        let copy = |text: &str| -> Result<String, Error> {
            let mut owned = String::new();
            owned
                .try_reserve_exact(text.len())
                .map_err(|_| Error::Render("cannot reserve text query key".into()))?;
            owned.push_str(text);
            Ok(owned)
        };
        let appearance = TextTheme::new(
            appearance.get_stack().clone(),
            appearance.get_color(),
            appearance.get_font_size(),
            appearance.get_weight(),
            appearance.get_line_height(),
        )?;
        let source = if let Some(index) = query_key.shaping_entry {
            Arc::clone(&cache.entries[index].source)
        } else {
            Arc::new(copy(input.source)?)
        };
        cache.entries.push(CachedTextQuery {
            shaping_entry: query_key.shaping_entry.unwrap_or(exact_record.entry),
            shaping_state: query_key.shaping_state,
            unwrapped_width: match &result {
                CachedTextResult::Plan(plan) => plan
                    .unwrapped_fit_width()
                    .map_err(|error| Error::Render(format!("cached paragraph fit: {error}")))?,
                CachedTextResult::PreferredWidth(_) | CachedTextResult::Reflow(_) => None,
            },
            source,
            appearance,
            alignment: input.alignment,
            owner: copy(&input.provenance.owner)?,
            pair_ordinal: input.provenance.pair_ordinal,
            exact_override: input.exact_override,
            width: input.maximum_width_quanta,
            preferred_lines,
            result,
        });
        cache.exact.insert(exact_position, exact_record);
        if let Some(position) = shaping_position {
            cache.shaping.insert(position, shaping_record);
        }
        if let Some(position) = profile_position {
            cache.profiles.insert(position, exact_record.entry);
        }
        Ok(())
    }

    /// Borrow this operation's prepared canonical font environment.
    pub(crate) fn prepared_font_environment(&self) -> Result<&PreparedFontEnvironment, Error> {
        #[cfg(test)]
        if self.font_environment.get().is_none() {
            self.font_environment.get_or_init(|| {
                FONT_ENVIRONMENT_DISCOVERIES.with(|calls| calls.set(calls.get() + 1));
                PreparedFontEnvironment::discover_system().map_err(|error| error.to_string())
            });
        }
        let prepared = self.font_environment.get().ok_or_else(|| {
            Error::Render("font environment was requested before canonical text preparation".into())
        })?;
        prepared
            .as_ref()
            .map_err(|error| Error::Render(format!("font preflight: {error}")))
    }
}

#[cfg(test)]
const fn pair_ordinal(weight: TextWeight) -> u32 {
    match weight {
        TextWeight::Normal => 0,
        TextWeight::Semibold => 1,
        TextWeight::Bold => 2,
    }
}

/// Paints applied to renderer-owned drawing surfaces.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceTheme {
    /// Canvas root paint.
    pub(crate) canvas_root: Color,
    /// Plot panel paint.
    pub(crate) plot_panel: Color,
}

impl SurfaceTheme {
    /// Construct complete surface paints; transparent paints are valid.
    pub fn new(canvas_root: Color, plot_panel: Color) -> Result<Self, Error> {
        Ok(Self {
            canvas_root,
            plot_panel,
        })
    }

    /// Inspect the complete Canvas root paint.
    pub fn get_canvas_root(&self) -> Color {
        self.canvas_root
    }

    /// Inspect the complete Plot panel paint.
    pub fn get_plot_panel(&self) -> Color {
        self.plot_panel
    }

    /// Replace only the Canvas root paint.
    pub fn canvas_root(self, canvas_root: Color) -> Result<Self, Error> {
        Self::new(canvas_root, self.plot_panel)
    }

    /// Replace only the Plot panel paint.
    pub fn plot_panel(self, plot_panel: Color) -> Result<Self, Error> {
        Self::new(self.canvas_root, plot_panel)
    }
}

/// Complete text appearance for one semantic typography role.
#[derive(Debug, Clone, PartialEq)]
pub struct TextTheme {
    /// Ordered literal named and host-generic family preferences.
    pub(crate) stack: crate::font::canonical::FontStack,
    /// Text paint.
    pub(crate) paint: Color,
    /// Font size in actual CSS pixels.
    pub(crate) font_size: u32,
    /// Closed renderer-owned font weight.
    pub(crate) weight: TextWeight,
    /// Finite positive multiple of the font size used for line pitch.
    pub(crate) line_height_multiplier: f64,
}

// Public construction/edits admit only finite positive leading, so complete
// field equality is reflexive and has no signed-zero normalization ambiguity.
impl Eq for TextTheme {}

impl TextTheme {
    /// Construct one complete text role, using actual CSS pixels and a multiplier.
    pub fn new(
        stack: crate::font::canonical::FontStack,
        color: Color,
        font_size: u32,
        weight: TextWeight,
        line_height: f64,
    ) -> Result<Self, Error> {
        let value = Self {
            stack,
            paint: color,
            font_size,
            weight,
            line_height_multiplier: line_height,
        };
        value.validate()?;
        Ok(value)
    }

    pub(crate) fn validate(&self) -> Result<(), Error> {
        if self.font_size == 0 {
            return Err(Error::Data(
                "text font size must be positive CSS pixels".into(),
            ));
        }
        if !self.line_height_multiplier.is_finite() || self.line_height_multiplier <= 0.0 {
            return Err(Error::Data(
                "text line height must be a finite positive multiplier".into(),
            ));
        }
        Ok(())
    }

    /// Borrow this role's complete ordered font preferences.
    pub fn get_stack(&self) -> &crate::font::canonical::FontStack {
        &self.stack
    }

    /// Inspect the complete text paint.
    pub fn get_color(&self) -> Color {
        self.paint
    }

    /// Inspect the positive font size in actual CSS pixels.
    pub fn get_font_size(&self) -> u32 {
        self.font_size
    }

    /// Inspect the closed font weight.
    pub fn get_weight(&self) -> TextWeight {
        self.weight
    }

    /// Inspect the retained multiplier, without multiplying by the font size.
    pub fn get_line_height(&self) -> f64 {
        self.line_height_multiplier
    }

    /// Replace only the complete font stack.
    pub fn stack(self, stack: crate::font::canonical::FontStack) -> Result<Self, Error> {
        Self::new(
            stack,
            self.paint,
            self.font_size,
            self.weight,
            self.line_height_multiplier,
        )
    }

    /// Replace only the complete text paint.
    pub fn color(self, color: Color) -> Result<Self, Error> {
        Self::new(
            self.stack,
            color,
            self.font_size,
            self.weight,
            self.line_height_multiplier,
        )
    }

    /// Replace only the positive CSS font size.
    pub fn font_size(self, font_size: u32) -> Result<Self, Error> {
        Self::new(
            self.stack,
            self.paint,
            font_size,
            self.weight,
            self.line_height_multiplier,
        )
    }

    /// Replace only the font weight.
    pub fn weight(self, weight: TextWeight) -> Result<Self, Error> {
        Self::new(
            self.stack,
            self.paint,
            self.font_size,
            weight,
            self.line_height_multiplier,
        )
    }

    /// Replace only the finite positive line-height multiplier.
    pub fn line_height(self, line_height: f64) -> Result<Self, Error> {
        Self::new(
            self.stack,
            self.paint,
            self.font_size,
            self.weight,
            line_height,
        )
    }

    pub(crate) fn same(&self, other: &Self) -> bool {
        self.font_size == other.font_size
            && self.weight == other.weight
            && self.paint == other.paint
            && self.line_height_multiplier.to_bits() == other.line_height_multiplier.to_bits()
            && self.stack.same(&other.stack)
    }

    pub(crate) fn try_copy(&self) -> Result<Self, Error> {
        Self::new(
            self.stack
                .try_copy()
                .map_err(|error| Error::Render(error.to_string()))?,
            self.get_color(),
            self.get_font_size(),
            self.get_weight(),
            self.get_line_height(),
        )
    }
    /// Resolve this role's current line pitch in pixels.
    pub(crate) fn line_pitch_pixels(&self) -> f64 {
        f64::from(self.font_size) * self.line_height_multiplier
    }
}

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "the four closed presets provide one compact table of preserved scalar values"
)]
fn resolved_theme(
    colors: &[(u8, u8, u8)],
    foreground: (u8, u8, u8),
    axis_width: f32,
    bracket_width: f32,
    title_font_size: u32,
    subtitle_font_size: u32,
    caption_font_size: u32,
    axis_title_font_size: u32,
    grid_color: (u8, u8, u8),
) -> Result<Theme, Error> {
    let stack = crate::font::canonical::builtin_font_stack();
    let foreground_paint = color_from_rgb(foreground);
    let muted_paint = color_from_rgb(MUTED_TEXT);
    let opaque = |&(r, g, b): &(u8, u8, u8)| OpaqueColor::rgb(r, g, b);
    let color_scales = ColorScales::new(
        QualitativePalette::extendable(colors.iter().map(opaque))?,
        SequentialGradient::new(crate::theme::HEATMAP_SEQUENTIAL_STOPS)?,
        DivergingGradient::new(crate::theme::HEATMAP_DIVERGING_STOPS)?,
    )?;
    let accents = AccentPalette::new(colors.iter().map(opaque))?;
    let stroke = |width, pattern| StrokeTheme::new(foreground_paint, width, pattern);
    let text = |paint, font_size, line_height| {
        TextTheme::new(
            stack.clone(),
            paint,
            font_size,
            TextWeight::Normal,
            line_height,
        )
    };

    let value = Theme {
        color_scales,
        surfaces: SurfaceTheme::new(Color::rgb(255, 255, 255), Color::rgb(255, 255, 255))?,
        typography: TypographyTheme::new(
            RootTextRoles::new(
                text(
                    foreground_paint,
                    title_font_size,
                    plus_six_line_height(title_font_size),
                )?,
                text(
                    muted_paint,
                    subtitle_font_size,
                    plus_six_line_height(subtitle_font_size),
                )?,
                text(
                    muted_paint,
                    caption_font_size,
                    plus_six_line_height(caption_font_size),
                )?,
            )?,
            AxisTextRoles::new(
                text(foreground_paint, axis_title_font_size, 1.2)?,
                text(foreground_paint, TICK_FONT_SIZE, 1.2)?,
            )?,
            GuideTextRoles::new(
                text(Color::rgb(60, 60, 60), 12, 1.2)?,
                text(Color::rgb(40, 40, 40), 11, 1.2)?,
            )?,
            CompositionTextRoles::new(
                text(foreground_paint, subtitle_font_size, 1.2)?,
                text(foreground_paint, title_font_size, 1.2)?,
                text(foreground_paint, title_font_size, 1.2)?,
                text(foreground_paint, axis_title_font_size, 1.2)?,
            )?,
            AnnotationTextRoles::new(
                text(foreground_paint, SIGNIFICANCE_FONT_SIZE, 1.2)?,
                text(foreground_paint, 14, 1.2)?,
                text(foreground_paint, 14, 1.2)?,
                text(foreground_paint, 10, 1.2)?,
                text(foreground_paint, 14, 1.2)?,
                ContrastTextRoles::new(
                    text(Color::rgb(255, 255, 255), 11, 1.2)?,
                    text(Color::rgb(0, 0, 0), 11, 1.2)?,
                )?,
            )?,
        )?,
        legend: LegendTheme::builtin()?,
        axes: AxisTheme::new(
            Some(FrameTheme::new(
                FrameSides::new(true, true, true, true)?,
                stroke(axis_width, StrokePattern::Solid)?,
            )?),
            Some(FrameTheme::new(
                FrameSides::new(false, false, true, true)?,
                stroke(axis_width, StrokePattern::Solid)?,
            )?),
            Some(TickTheme::new(
                stroke(axis_width, StrokePattern::Solid)?,
                5,
            )?),
            Some(StrokeTheme::new(
                color_from_rgb(grid_color),
                0.5,
                StrokePattern::MediumDash,
            )?),
            AxisSpacing::new(
                super::RelativeSpacing::ratio(8, 16)?,
                super::RelativeSpacing::ratio(6, 16)?,
                super::RelativeSpacing::ratio(8, axis_title_font_size)?,
                super::RelativeSpacing::ratio(16, axis_title_font_size)?,
                super::RelativeSpacing::ratio(16, axis_title_font_size)?,
                super::RelativeSpacing::ratio(12, axis_title_font_size)?,
            )?,
        )?,
        marks: MarkTheme::builtin(accents, opaque(&foreground), axis_width)?,
        annotations: AnnotationTheme::builtin(foreground_paint, bracket_width)?,
        composition: super::CompositionTheme::builtin(
            title_font_size,
            subtitle_font_size,
            caption_font_size,
            axis_title_font_size,
        )?,
    };
    Theme::new(
        value.surfaces,
        value.typography,
        value.axes,
        value.legend,
        value.composition,
        value.marks,
        value.annotations,
        value.color_scales,
    )
}

fn plus_six_line_height(font_size: u32) -> f64 {
    (f64::from(font_size) + 6.0) / f64::from(font_size)
}

pub(super) fn publication_theme() -> Result<Theme, Error> {
    // Okabe-Ito 8-color qualitative palette (colorblind-safe).
    resolved_theme(
        &[
            (0, 114, 178),
            (213, 94, 0),
            (0, 158, 115),
            (204, 121, 167),
            (86, 180, 233),
            (230, 159, 0),
            (240, 228, 66),
            (0, 0, 0),
        ],
        (0, 0, 0),
        0.8,
        1.0,
        20,
        15,
        15,
        17,
        (220, 220, 220),
    )
}

pub(super) fn nature_theme() -> Result<Theme, Error> {
    resolved_theme(
        &[
            (40, 86, 130),
            (130, 160, 188),
            (200, 215, 230),
            (120, 130, 90),
            (180, 175, 130),
        ],
        (40, 40, 40),
        0.8,
        0.8,
        19,
        14,
        14,
        16,
        (230, 230, 230),
    )
}

pub(super) fn science_theme() -> Result<Theme, Error> {
    resolved_theme(
        &[
            (255, 0, 0),
            (0, 0, 255),
            (0, 175, 0),
            (255, 175, 0),
            (160, 32, 240),
        ],
        (0, 0, 0),
        1.0,
        1.0,
        21,
        16,
        15,
        17,
        (200, 200, 200),
    )
}

pub(super) fn chi_theme() -> Result<Theme, Error> {
    resolved_theme(
        &[
            (171, 199, 232),
            (243, 169, 175),
            (174, 224, 174),
            (244, 213, 152),
            (197, 176, 213),
        ],
        (60, 60, 60),
        0.8,
        0.8,
        19,
        15,
        15,
        17,
        (220, 220, 220),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn indexed_query_input<'a>(
        source: &'a str,
        appearance: &'a TextTheme,
        owner: &str,
    ) -> TextPlanInput<'a> {
        TextPlanInput {
            source,
            stack: &appearance.stack,
            weight: appearance.weight,
            font_size: appearance.font_size,
            line_height: TextLineHeight::Multiplier(appearance.line_height_multiplier),
            maximum_width_quanta: 100 * 1024,
            alignment: PhysicalTextAlignment::Start,
            paint: appearance.paint,
            provenance: TextPlanProvenance {
                pair_ordinal: pair_ordinal(appearance.weight),
                owner: owner.into(),
            },
            exact_override: false,
        }
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one root verifies indexed lookup, stable collision order, and complete matching"
    )]
    fn indexed_text_queries_preserve_matching_and_collision_order() {
        let root = Theme::publication().resolve().unwrap();
        root.font_environment
            .set(Ok(PreparedFontEnvironment::synthetic()))
            .unwrap();
        let environment = root.prepared_font_environment().unwrap();
        let resources = environment.operation();
        environment.validate_operation(&resources).unwrap();
        let mut appearance = root.typography.root.caption.clone();
        appearance.stack = crate::font::canonical::named_stack(&["Test Sans"]);
        appearance.font_size = 10;
        let mut first = None;
        for index in 0..128 {
            let plan = root
                .plan_text(
                    "A",
                    &appearance,
                    100.0,
                    PhysicalTextAlignment::Start,
                    &format!("owner {index:03}"),
                    false,
                )
                .unwrap();
            if index == 0 {
                first = Some(plan);
            }
        }
        {
            let cache = root.text_queries.lock().unwrap();
            assert_eq!(cache.entries.len(), 128);
            assert_eq!(cache.exact.len(), 128);
            assert_eq!(
                cache.shaping.len(),
                1,
                "one analysis representative per compatible font/source profile"
            );
            assert!(cache.exact.windows(2).all(|pair| pair[0] < pair[1]));
        }

        let hit = root
            .plan_text(
                "A",
                &appearance,
                100.0,
                PhysicalTextAlignment::Start,
                "owner 000",
                false,
            )
            .unwrap();
        assert!(first.unwrap().shares_payload(&hit));

        root.plan_text(
            "AA",
            &appearance,
            100.0,
            PhysicalTextAlignment::Start,
            "collision",
            false,
        )
        .unwrap();
        let collision = indexed_query_input("AA", &appearance, "collision");
        let mut cache = root.text_queries.lock().unwrap();
        let expected = cache.entries.len() - 1;
        let TextQueryCache { exact, shaping, .. } = &mut *cache;
        for index in [exact, shaping] {
            for entry in index.iter_mut() {
                entry.fingerprint = [0; 32];
            }
            index.sort_unstable();
        }

        assert_eq!(
            cache.find_query(
                &cache.exact,
                [0; 32],
                (&collision, None),
                true,
                Some(expected)
            ),
            Some(expected)
        );

        assert_eq!(
            cache.find_query(
                &cache.shaping,
                [0; 32],
                (&collision, None),
                false,
                Some(expected)
            ),
            Some(expected)
        );

        let mut wrong_provenance = collision.clone();
        wrong_provenance.provenance.pair_ordinal += 1;
        assert!(cache
            .find_query(
                &cache.exact,
                [0; 32],
                (&wrong_provenance, None),
                true,
                Some(expected)
            )
            .is_none());
        assert_eq!(
            cache.find_query(
                &cache.shaping,
                [0; 32],
                (&wrong_provenance, None),
                false,
                Some(expected)
            ),
            Some(expected)
        );
    }

    #[test]
    fn indexed_query_fingerprints_include_exact_presentation_and_provenance() {
        let appearance = Theme::publication()
            .resolve()
            .unwrap()
            .typography
            .root
            .caption
            .clone();
        let input = indexed_query_input("A", &appearance, "owner");

        let fingerprints = CachedTextQuery::fingerprints(&input, None);
        for change in 0..7 {
            let mut changed = input.clone();
            match change {
                0 => changed.maximum_width_quanta += 1,
                1 => changed.alignment = PhysicalTextAlignment::End,
                2 => changed.paint = Color::rgba(4, 5, 6, 7),
                3 => changed.line_height = TextLineHeight::ResolvedQuanta(1000),
                4 => changed.exact_override = true,
                5 => changed.provenance.owner.push('x'),
                _ => changed.provenance.pair_ordinal += 1,
            }
            let changed = CachedTextQuery::fingerprints(&changed, None);
            assert_eq!(changed.0, fingerprints.0);
            assert_ne!(changed.1, fingerprints.1);
        }
        assert_ne!(
            CachedTextQuery::fingerprints(&input, Some(2)).1,
            fingerprints.1
        );
        for change in 0..4 {
            let mut changed = input.clone();
            let other = crate::font::canonical::named_stack(&["Other"]);
            match change {
                0 => changed.source = "AA",
                1 => changed.stack = &other,
                2 => changed.font_size += 1,
                _ => changed.weight = TextWeight::Bold,
            }
            assert_ne!(
                CachedTextQuery::fingerprints(&changed, None).0,
                fingerprints.0
            );
        }
    }

    #[test]
    fn retained_profile_does_not_trust_a_reused_mutable_input_address() {
        let theme = Theme::publication().resolve().unwrap();
        theme
            .font_environment
            .set(Ok(PreparedFontEnvironment::synthetic_faces(&[(
                "Literal",
                &['A', 'X'],
            )])))
            .unwrap();
        let appearance = TextTheme {
            stack: crate::font::canonical::named_stack(&["Literal"]),
            font_size: 10,
            ..theme.typography.root.caption.clone()
        };
        let mut source = String::with_capacity(16);
        source.push('A');
        let address = source.as_ptr();
        let first = theme
            .plan_text(
                &source,
                &appearance,
                100.0,
                PhysicalTextAlignment::Start,
                "mutable input",
                false,
            )
            .unwrap();
        source.replace_range(.., "X");
        assert_eq!(source.as_ptr(), address);
        let second = theme
            .plan_text(
                &source,
                &appearance,
                100.0,
                PhysicalTextAlignment::Start,
                "mutable input",
                false,
            )
            .unwrap();
        assert!(!first.shares_payload(&second));
        assert_eq!(second.source, "X");
        assert_eq!(theme.text_queries.lock().unwrap().profiles.len(), 2);
    }

    #[test]
    fn later_font_admissions_preserve_cached_shapes_and_complete_request_payloads() {
        let theme = Theme::publication().resolve().unwrap();
        theme
            .font_environment
            .set(Ok(PreparedFontEnvironment::synthetic_faces(&[
                ("Base", &['A']),
                ("Complete", &['A', '\u{301}']),
            ])))
            .unwrap();
        let appearance = TextTheme {
            stack: crate::font::canonical::named_stack(&["Base", "Complete"]),
            ..theme.typography.root.caption.clone()
        };
        let measure = |source| {
            theme
                .measure_text(
                    source,
                    &appearance,
                    PhysicalTextAlignment::Start,
                    "actual source",
                    true,
                )
                .unwrap()
        };
        let first = measure("A");
        let marked = measure("A\u{301}");
        assert_ne!(first.lines[0].runs[0].face, marked.lines[0].runs[0].face);
        let counts = crate::font::canonical::planning_counts();
        assert!(first.shares_payload(&measure("A")));
        assert!(marked.shares_payload(&measure("A\u{301}")));
        assert_eq!(crate::font::canonical::planning_counts(), counts);
        theme
            .prepared_font_environment()
            .unwrap()
            .operation()
            .seal_fonts()
            .unwrap();
        let counts = crate::font::canonical::planning_counts();
        assert!(first.shares_payload(&measure("A")));
        assert!(marked.shares_payload(&measure("A\u{301}")));
        assert_eq!(crate::font::canonical::planning_counts(), counts);
        assert_eq!(theme.text_queries.lock().unwrap().entries.len(), 2);
    }

    #[test]
    fn text_cache_stops_optional_insertion_without_refusing_rendering() {
        let root = Theme::publication().resolve().unwrap();
        root.font_environment
            .set(Ok(PreparedFontEnvironment::synthetic()))
            .unwrap();
        let mut appearance = root.typography.root.caption.clone();
        appearance.stack = crate::font::canonical::named_stack(&["Test Sans"]);
        let render = |owner: &str| {
            root.measure_text("A", &appearance, PhysicalTextAlignment::Start, owner, false)
                .unwrap()
        };
        let first = render("first");
        for index in 1..256 {
            render(&format!("owner {index}"));
        }
        assert_eq!(root.text_queries.lock().unwrap().entries.len(), 256);
        assert_eq!(render("uncached owner").source, "A");
        assert_eq!(root.text_queries.lock().unwrap().entries.len(), 256);
        assert!(first.shares_payload(&render("first")));
    }
}
