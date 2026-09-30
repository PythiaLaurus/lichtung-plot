//! Publication-grade visual themes.
//!
//! Every theme is hardcoded so visual regressions remain reviewable and
//! re-renders are byte-stable under the same font environment.
//! Each built-in theme resolves privately to colors, fonts, and layout style.

#![deny(missing_docs)]

use std::sync::{Arc, Mutex, OnceLock};

use sha2::{Digest, Sha256};

use crate::font::canonical::PreparedFontEnvironment;
use crate::layout::{
    PhysicalTextAlignment, ResolvedTextPlan, TextLineHeight, TextPlanInput, TextPlanProvenance,
};
use crate::render::backend::{LineStyle, Rgba, StrokePattern, TextAnchor};
use crate::{Error, SignificanceMarkStyle, TextWeight};

/// Built-in axis tick-label font size in points.
const TICK_FONT_SIZE: u32 = 16;

const MUTED_TEXT: (u8, u8, u8) = (80, 80, 80);
const SIGNIFICANCE_FONT_SIZE: u32 = 16;

#[cfg(test)]
thread_local! {
    pub(crate) static THEME_RESOLUTION_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static FONT_ENVIRONMENT_DISCOVERIES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

macro_rules! define_themes {
    ($($variant:ident => ($documentation:literal, $resolver:ident)),+ $(,)?) => {
        /// Predefined publication themes with hardcoded journal/venue palettes.
        ///
        /// Attach one complete selection with [`crate::Plot::theme`] or
        /// [`crate::Canvas::theme`].
        ///
        /// ```compile_fail
        /// let _ = lichtung_plot::Theme::Publication.resolve();
        /// ```
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Theme {
            $(
                #[doc = $documentation]
                $variant,
            )+
        }

        impl Theme {
            pub(crate) fn resolve(self) -> ResolvedTheme {
                #[cfg(test)]
                THEME_RESOLUTION_CALLS.with(|calls| calls.set(calls.get() + 1));
                match self { $(Self::$variant => $resolver()),+ }
            }
        }
    };
}

define_themes! {
    Publication => ("Default clean theme with high contrast and generous whitespace.", publication_theme),
    Nature => ("Nature-style muted blues and greens without top/right axes.", nature_theme),
    Science => ("Science-style high-saturation palette with bold axis labels.", science_theme),
    CHI => ("ACM CHI/HCI pastel palette with accessible contrast.", chi_theme),
}

/// A theme's full resolved visual specification. Layout and renderers consume
/// this value to produce concrete backend styles and layout decisions.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedTheme {
    /// Series colors in draw order. Index 0 is the first series.
    pub(crate) colors: Vec<(u8, u8, u8)>,
    /// Legacy stack source retained for later legend, mark, and annotation roles.
    pub(crate) font_family: String,
    /// Legacy paint source retained for later legend, mark, and annotation roles.
    pub(crate) foreground: (u8, u8, u8),
    /// Font size in points for annotations, status text, and color scales.
    pub(crate) detail_font_size: u32,
    /// Legacy label-size source retained for later body and annotation roles.
    pub(crate) label_font_size: u32,
    /// Legacy Canvas-composition title size retained until T5b.
    pub(crate) composition_title_font_size: u32,
    /// Legacy facet-header size source retained until T5b composition migration.
    pub(crate) facet_header_font_size: u32,
    /// Canvas-root and Plot-panel paints.
    pub(crate) surfaces: SurfaceAppearance,
    /// An explicit Plot inside Canvas composes over the Canvas owner's surface.
    pub(crate) canvas_surface_override: Option<Rgba>,
    /// Complete foundational text roles.
    pub(crate) typography: FoundationalTypography,
    /// Legend typography-adjacent chrome with owner-local lengths.
    pub(crate) legend: LegendAppearance,
    /// Nonsemantic axis chrome and its local spacing.
    pub(crate) axes: AxisAppearance,
    /// Complete stroke roles for currently migrated marks.
    pub(crate) marks: MarkAppearance,
    /// Resolved non-text annotation appearance.
    pub(crate) annotations: AnnotationAppearance,
    /// Root and panel composition relationships.
    pub(crate) composition: CompositionAppearance,
    font_environment: Arc<OnceLock<Result<PreparedFontEnvironment, String>>>,
    text_queries: Arc<Mutex<TextQueryCache>>,
}

#[derive(Debug, Default)]
struct TextQueryCache {
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
    fn lower_bound(
        index: &[TextQueryIndex],
        fingerprint: [u8; 32],
        environment: &PreparedFontEnvironment,
    ) -> Result<usize, Error> {
        let (mut low, mut high) = (0, index.len());
        while low < high {
            environment
                .claim_processing_visits(1)
                .map_err(|error| Error::Render(format!("text query lookup: {error:?}")))?;
            let middle = low + (high - low) / 2;
            if index[middle].fingerprint < fingerprint {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        Ok(low)
    }

    fn find(
        &self,
        index: &[TextQueryIndex],
        fingerprint: [u8; 32],
        key: (&TextPlanInput<'_>, Option<usize>, bool),
        exact: bool,
        environment: &PreparedFontEnvironment,
    ) -> Result<Option<usize>, Error> {
        let (input, preferred_lines, prepared_only) = key;
        let start = Self::lower_bound(index, fingerprint, environment)?;
        for candidate in &index[start..] {
            environment
                .claim_processing_visits(1)
                .map_err(|error| Error::Render(format!("text query lookup: {error:?}")))?;
            if candidate.fingerprint != fingerprint {
                break;
            }
            let query = &self.entries[candidate.entry];
            if query.matches_shaping(input, prepared_only)
                && (!exact
                    || (query.width == input.maximum_width_quanta
                        && query.matches_presentation(input, preferred_lines)))
            {
                return Ok(Some(candidate.entry));
            }
        }
        Ok(None)
    }

    fn reserve<T>(values: &mut Vec<T>, environment: &PreparedFontEnvironment) -> Result<(), Error> {
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
        let bytes = capacity
            .checked_mul(size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| Error::Render("text query cache byte count overflow".into()))?;
        environment
            .claim_unicode_bytes(bytes)
            .map_err(|error| Error::Render(format!("text query cache: {error}")))?;
        values
            .try_reserve_exact(capacity - values.len())
            .map_err(|_| Error::Render("cannot reserve text query cache".into()))
    }

    fn insertion_point(
        index: &[TextQueryIndex],
        value: TextQueryIndex,
        environment: &PreparedFontEnvironment,
    ) -> Result<usize, Error> {
        let (mut low, mut high) = (0, index.len());
        while low < high {
            environment
                .claim_processing_visits(1)
                .map_err(|error| Error::Render(format!("text query index: {error:?}")))?;
            let middle = low + (high - low) / 2;
            if index[middle] < value {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        Ok(low)
    }
}

#[derive(Debug)]
struct CachedTextQuery {
    source: String,
    appearance: TextAppearance,
    alignment: PhysicalTextAlignment,
    owner: String,
    pair_ordinal: u32,
    exact_override: bool,
    prepared_only: bool,
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

impl CachedTextQuery {
    fn fingerprints(
        input: &TextPlanInput<'_>,
        preferred_lines: Option<usize>,
        prepared_only: bool,
    ) -> ([u8; 32], [u8; 32]) {
        let mut hash = Sha256::new();
        // The complete, unnormalized source fixes paragraph bidi and scripts;
        // language and Unicode policy are fixed for this renderer. The memo is
        // shared only together with its immutable operation font environment.
        for text in [input.source, input.stack] {
            hash.update((text.len() as u64).to_le_bytes());
            hash.update(text.as_bytes());
        }
        hash.update(input.font_size.to_le_bytes());
        hash.update([
            match input.weight {
                TextWeight::Normal => 0,
                TextWeight::Semibold => 1,
                TextWeight::Bold => 2,
            },
            u8::from(prepared_only),
        ]);
        let shaping = hash.clone().finalize().into();
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
            input.paint.alpha(),
            u8::from(input.exact_override),
        ]);
        hash.update([u8::from(preferred_lines.is_some())]);
        if let Some(lines) = preferred_lines {
            hash.update((lines as u64).to_le_bytes());
        }
        (shaping, hash.finalize().into())
    }

    fn matches_shaping(&self, input: &TextPlanInput<'_>, prepared_only: bool) -> bool {
        self.source == input.source
            && self.appearance.stack == input.stack
            && self.appearance.font_size == input.font_size
            && self.appearance.weight == input.weight
            && self.prepared_only == prepared_only
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
            && self.owner == input.provenance.owner
            && self.pair_ordinal == input.provenance.pair_ordinal
            && self.exact_override == input.exact_override
            && self.preferred_lines == preferred_lines
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
        self.colors == other.colors
            && self.font_family == other.font_family
            && self.foreground == other.foreground
            && self.detail_font_size == other.detail_font_size
            && self.label_font_size == other.label_font_size
            && self.composition_title_font_size == other.composition_title_font_size
            && self.facet_header_font_size == other.facet_header_font_size
            && self.surfaces == other.surfaces
            && self.canvas_surface_override == other.canvas_surface_override
            && self.typography == other.typography
            && self.axes == other.axes
            && self.marks == other.marks
            && self.annotations == other.annotations
            && self.composition == other.composition
    }
}

impl ResolvedTheme {
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
    }

    /// Plan one foundational appearance within an exact finite pixel width.
    #[cfg(test)]
    pub(crate) fn plan_text(
        &self,
        source: &str,
        appearance: &TextAppearance,
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
        environment
            .claim_measurement_call()
            .map_err(|error| Error::Render(format!("axis intrinsic call admission: {error:?}")))?;
        let appearance = &self.typography.tick_label;
        let height = crate::layout::maximum_width_quanta(available_height)
            .map_err(|error| crate::Error::Render(format!("axis row height: {error}")))?;
        let count = i64::try_from(labels.len().max(1))
            .map_err(|_| crate::Error::Render("axis row count overflowed".into()))?;
        let pitch = crate::layout::maximum_width_quanta(appearance.line_height())
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
            let input = crate::layout::TextPlanInput {
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
            let call = environment.claim_measurement_call().map_err(|error| {
                Error::Render(format!("text intrinsic call admission: {error:?}"))
            })?;
            let cached = self.cached_text_query(&input, Some(preferred_lines), true, &call)?;
            let preferred_width = if let Some(CachedTextResult::PreferredWidth(width)) = cached {
                width
            } else {
                let analysis = if let Some(CachedTextResult::Reflow(plan)) = cached {
                    plan
                } else {
                    let plan = environment
                        .plan_text_in_call(input.clone(), true, &call)
                        .map_err(|error| {
                            crate::Error::Render(format!("axis intrinsic text analysis: {error}"))
                        })?;
                    self.cache_text_query(
                        &input,
                        appearance,
                        None,
                        true,
                        CachedTextResult::Plan(plan.clone()),
                    )?;
                    plan
                };
                let requirements =
                    analysis
                        .intrinsic_in_call(preferred_lines, &call)
                        .map_err(|error| {
                            crate::Error::Render(format!(
                                "axis intrinsic text requirements: {error}"
                            ))
                        })?;
                self.cache_text_query(
                    &input,
                    appearance,
                    Some(preferred_lines),
                    true,
                    CachedTextResult::PreferredWidth(requirements.preferred_width_quanta),
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
        appearance: &TextAppearance,
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

    /// Build the current private appearance for a legacy T5b body-text role.
    pub(crate) fn legacy_text_appearance(
        &self,
        font_size: u32,
        weight: TextWeight,
    ) -> TextAppearance {
        TextAppearance {
            stack: self.font_family.clone(),
            paint: self.foreground.into(),
            font_size,
            weight,
            line_height_multiplier: 1.2,
        }
    }

    /// Select a complete text role over one final variable-fill paint before
    /// font preparation, using owner-local panel/root surfaces and backdrop.
    pub(crate) fn contrasting_text(&self, fill: Rgba) -> Result<&TextAppearance, Error> {
        super::contrast::select(
            fill,
            self.surfaces.plot_panel,
            self.canvas_surface_override
                .unwrap_or(self.surfaces.canvas_root),
            self.annotations.planning_backdrop,
            &self.typography.contrast_light,
            &self.typography.contrast_dark,
        )
    }

    pub(crate) fn measure_legacy_appearance_text(
        &self,
        source: &str,
        appearance: &TextAppearance,
        alignment: PhysicalTextAlignment,
        owner: &str,
        exact_override: bool,
    ) -> Result<ResolvedTextPlan, Error> {
        self.plan_text_quanta_with_policy(
            source,
            appearance,
            i64::MAX,
            alignment,
            owner,
            exact_override,
            true,
            &crate::layout::fit::ROOT_SCOPE,
        )
        .map_err(crate::layout::fit::PlanError::into_error)
    }

    fn plan_text_quanta(
        &self,
        source: &str,
        appearance: &TextAppearance,
        maximum_width_quanta: i64,
        alignment: PhysicalTextAlignment,
        owner: &str,
        exact_override: bool,
    ) -> Result<ResolvedTextPlan, Error> {
        self.plan_text_quanta_with_policy(
            source,
            appearance,
            maximum_width_quanta,
            alignment,
            owner,
            exact_override,
            true,
            &crate::layout::fit::ROOT_SCOPE,
        )
        .map_err(crate::layout::fit::PlanError::into_error)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the shared adapter preserves every immutable text-plan input while selecting the phase-owned preparation policy"
    )]
    pub(crate) fn plan_text_quanta_with_policy(
        &self,
        source: &str,
        appearance: &TextAppearance,
        maximum_width_quanta: i64,
        alignment: PhysicalTextAlignment,
        owner: &str,
        exact_override: bool,
        prepared_only: bool,
        scope: &crate::layout::fit::FitScope,
    ) -> Result<ResolvedTextPlan, crate::layout::fit::PlanError> {
        let input = TextPlanInput {
            source,
            stack: &appearance.stack,
            weight: appearance.weight,
            font_size: appearance.font_size,
            line_height: TextLineHeight::Multiplier(appearance.line_height_multiplier),
            maximum_width_quanta,
            alignment,
            paint: appearance.paint,
            provenance: TextPlanProvenance {
                pair_ordinal: pair_ordinal(appearance.weight),
                owner: owner.to_owned(),
            },
            exact_override,
        };
        let environment = self.prepared_font_environment()?;
        let call = environment
            .claim_measurement_call()
            .map_err(|error| Error::Render(format!("text query call admission: {error:?}")))?;
        let result = match self.cached_text_query(&input, None, prepared_only, &call)? {
            Some(CachedTextResult::Plan(plan)) => return Ok(plan),
            Some(CachedTextResult::Reflow(plan)) => plan.reflow_in_call(input.clone(), &call),
            _ => environment.plan_text_in_call(input.clone(), prepared_only, &call),
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
            prepared_only,
            CachedTextResult::Plan(plan.clone()),
        )?;
        Ok(plan)
    }

    fn cached_text_query(
        &self,
        input: &TextPlanInput<'_>,
        preferred_lines: Option<usize>,
        prepared_only: bool,
        _call: &crate::font::canonical::MeasurementCall,
    ) -> Result<Option<CachedTextResult>, Error> {
        let environment = self.prepared_font_environment()?;
        let cache = self
            .text_queries
            .lock()
            .map_err(|_| Error::Render("text query cache lock poisoned".into()))?;
        let (shaping_fingerprint, fingerprint) =
            CachedTextQuery::fingerprints(input, preferred_lines, prepared_only);
        let key = (input, preferred_lines, prepared_only);
        if let Some(index) = cache.find(&cache.exact, fingerprint, key, true, environment)? {
            return Ok(Some(cache.entries[index].result.clone()));
        }
        if let Some(index) =
            cache.find(&cache.shaping, shaping_fingerprint, key, false, environment)?
        {
            let query = &cache.entries[index];
            environment
                .claim_processing_visits(1)
                .map_err(|error| Error::Render(format!("text query lookup: {error:?}")))?;
            if query.matches_width(input.maximum_width_quanta)
                && query.matches_presentation(input, preferred_lines)
            {
                return Ok(Some(query.result.clone()));
            }
            if let CachedTextResult::Plan(plan) = &query.result {
                return Ok(Some(CachedTextResult::Reflow(plan.clone())));
            }
        }
        Ok(None)
    }

    fn cache_text_query(
        &self,
        input: &TextPlanInput<'_>,
        appearance: &TextAppearance,
        preferred_lines: Option<usize>,
        prepared_only: bool,
        result: CachedTextResult,
    ) -> Result<(), Error> {
        let mut cache = self
            .text_queries
            .lock()
            .map_err(|_| Error::Render("text query cache lock poisoned".into()))?;
        // This root-local memo is optional and bounded, never an all-width table.
        if cache.entries.len() == 256 {
            return Ok(());
        }
        let environment = self.prepared_font_environment()?;
        let key_bytes = input
            .source
            .len()
            .checked_add(input.stack.len())
            .and_then(|bytes| bytes.checked_add(input.provenance.owner.len()))
            .ok_or_else(|| Error::Render("text query cache byte count overflow".into()))?;
        let (shaping_fingerprint, fingerprint) =
            CachedTextQuery::fingerprints(input, preferred_lines, prepared_only);
        let key = (input, preferred_lines, prepared_only);
        if cache
            .find(&cache.exact, fingerprint, key, true, environment)?
            .is_some()
        {
            return Ok(());
        }
        let new_shaping = matches!(&result, CachedTextResult::Plan(_))
            && cache
                .find(&cache.shaping, shaping_fingerprint, key, false, environment)?
                .is_none();
        environment
            .claim_cache_entries(1)
            .map_err(|error| Error::Render(format!("text query cache admission: {error:?}")))?;
        environment
            .claim_text_generation_slots(2 + u128::from(new_shaping))
            .map_err(|error| Error::Render(format!("text query cache: {error:?}")))?;
        environment
            .claim_unicode_bytes(
                u64::try_from(key_bytes)
                    .map_err(|_| Error::Render("text query cache byte count overflow".into()))?,
            )
            .map_err(|error| Error::Render(format!("text query cache: {error}")))?;
        TextQueryCache::reserve(&mut cache.entries, environment)?;
        TextQueryCache::reserve(&mut cache.exact, environment)?;
        if new_shaping {
            TextQueryCache::reserve(&mut cache.shaping, environment)?;
        }
        let exact_record = TextQueryIndex {
            fingerprint,
            entry: cache.entries.len(),
        };
        let shaping_record = TextQueryIndex {
            fingerprint: shaping_fingerprint,
            entry: cache.entries.len(),
        };
        let exact_position =
            TextQueryCache::insertion_point(&cache.exact, exact_record, environment)?;
        let shaping_position = if new_shaping {
            Some(TextQueryCache::insertion_point(
                &cache.shaping,
                shaping_record,
                environment,
            )?)
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
        cache.entries.push(CachedTextQuery {
            unwrapped_width: match &result {
                CachedTextResult::Plan(plan) => plan
                    .unwrapped_fit_width(environment)
                    .map_err(|error| Error::Render(format!("cached paragraph fit: {error}")))?,
                CachedTextResult::PreferredWidth(_) | CachedTextResult::Reflow(_) => None,
            },
            source: copy(input.source)?,
            appearance: TextAppearance {
                stack: copy(&appearance.stack)?,
                ..*appearance
            },
            alignment: input.alignment,
            owner: copy(&input.provenance.owner)?,
            pair_ordinal: input.provenance.pair_ordinal,
            exact_override: input.exact_override,
            prepared_only,
            width: input.maximum_width_quanta,
            preferred_lines,
            result,
        });
        cache.exact.insert(exact_position, exact_record);
        if let Some(position) = shaping_position {
            cache.shaping.insert(position, shaping_record);
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

    #[cfg(test)]
    fn shares_font_environment_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.font_environment, &other.font_environment)
    }
}

const fn pair_ordinal(weight: TextWeight) -> u32 {
    match weight {
        TextWeight::Normal => 0,
        TextWeight::Semibold => 1,
        TextWeight::Bold => 2,
    }
}

/// Paints applied to renderer-owned drawing surfaces.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SurfaceAppearance {
    /// Canvas root paint.
    pub(crate) canvas_root: Rgba,
    /// Plot panel paint.
    pub(crate) plot_panel: Rgba,
}

/// Complete text appearance for one semantic typography role.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextAppearance {
    /// CSS-compatible system-font family stack.
    pub(crate) stack: String,
    /// Text paint.
    pub(crate) paint: Rgba,
    /// Font size in points.
    pub(crate) font_size: u32,
    /// Closed renderer-owned font weight.
    pub(crate) weight: TextWeight,
    /// Finite positive multiple of the font size used for line pitch.
    pub(crate) line_height_multiplier: f64,
}

impl TextAppearance {
    /// Resolve this role's current line pitch in pixels.
    pub(crate) fn line_height(&self) -> f64 {
        f64::from(self.font_size) * self.line_height_multiplier
    }
}

/// Foundational role-based typography resolved before measurement.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FoundationalTypography {
    /// Root and panel title role.
    pub(crate) title: TextAppearance,
    /// Root and panel subtitle role.
    pub(crate) subtitle: TextAppearance,
    /// Root caption role.
    pub(crate) caption: TextAppearance,
    /// X- and y-axis title role.
    pub(crate) axis_title: TextAppearance,
    /// X- and y-axis tick-label role.
    pub(crate) tick_label: TextAppearance,
    /// Significance-label default before an exact caller override.
    pub(crate) significance_label: TextAppearance,
    /// Estimate value labels and the required not-estimable marker.
    pub(crate) estimate_detail: TextAppearance,
    /// Required Waterfall start/end total labels.
    pub(crate) waterfall_total: TextAppearance,
    /// Supplemental line point labels; exact layer paint overrides only color.
    pub(crate) point_layer_label: TextAppearance,
    /// Caller-owned text inside one plot panel.
    pub(crate) panel_text: TextAppearance,
    /// Complete candidates selected over a variable fill before font discovery.
    pub(crate) contrast_light: TextAppearance,
    pub(crate) contrast_dark: TextAppearance,
    pub(crate) legend_title: TextAppearance,
    pub(crate) legend_label: TextAppearance,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LegendAppearance {
    pub(crate) padding: f64,
    pub(crate) key_width: f64,
    pub(crate) key_label_gap: f64,
    pub(crate) outside_gap: f64,
    pub(crate) shared_gap: f64,
    pub(crate) gradient_min_length: f64,
    pub(crate) gradient_preferred_length: f64,
    pub(crate) row_gap: f64,
    pub(crate) column_gap: f64,
    pub(crate) title_gap: f64,
    pub(crate) frame_fill: Rgba,
    pub(crate) frame_outline: Rgba,
    pub(crate) frame_stroke_width: f32,
    pub(crate) frame_radius: f64,
}

impl FoundationalTypography {
    /// Resolve the effective significance-label typography while preserving an
    /// exact caller size and weight over the theme defaults.
    pub(crate) fn effective_significance_label(
        &self,
        exact: Option<SignificanceMarkStyle>,
    ) -> TextAppearance {
        let mut effective = self.significance_label.clone();
        if let Some(exact) = exact {
            effective.font_size = exact.font_size_spec();
            effective.weight = exact.weight_spec();
        }
        effective
    }
}

/// One complete axis stroke style.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StrokeAppearance {
    /// Stroke paint.
    pub(crate) paint: Rgba,
    /// Finite positive stroke width in pixels.
    pub(crate) width: f32,
    /// Closed stroke pattern.
    pub(crate) pattern: StrokePattern,
}

impl StrokeAppearance {
    /// Build the private backend line style for this complete stroke.
    pub(crate) fn backend_style(&self) -> LineStyle {
        LineStyle {
            color: self.paint,
            width: self.width,
            pattern: self.pattern,
        }
    }
}

/// Mark strokes are independent of structurally similar axis chrome.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MarkAppearance {
    /// Shared primary-point paint/outline before family-specific size selection.
    pub(crate) primary_point: PointAppearance,
    /// Ordinary bar bodies in either orientation and originating guide glyphs.
    pub(crate) bar_body: DataFillAppearance,
    pub(crate) line_band: DataFillAppearance,
    pub(crate) regression_band: DataFillAppearance,
    pub(crate) density_area: DataFillAppearance,
    pub(crate) marginal_density: DataFillAppearance,
    pub(crate) matrix_density: DataFillAppearance,
    pub(crate) box_body: DataFillAppearance,
    pub(crate) violin_body: DataFillAppearance,
    pub(crate) raincloud_box: FixedFillAppearance,
    pub(crate) histogram_bin: DataFillAppearance,
    pub(crate) waterfall_bar: DataFillAppearance,
    /// Boxplot/Violin raw observations and Raincloud strips/guide points.
    pub(crate) raw_overlay_point: PointAppearance,
    pub(crate) raw_overlay_radius: u32,
    pub(crate) heatmap_cell_outline: FillOutlineAppearance,
    pub(crate) heatmap_highlight_outline: FillOutlineAppearance,
    /// Non-data missing paint shared by Heatmap cells, merged rows and keys.
    pub(crate) heatmap_missing_fill: Rgba,
    pub(crate) heatmap_missing_row_outline: FillOutlineAppearance,
    pub(crate) heatmap_missing_key_outline: FillOutlineAppearance,
    pub(crate) heatmap_missing_cross: StrokeAppearance,
    pub(crate) bar_zero: StrokeAppearance,
    pub(crate) waterfall_connector: StrokeAppearance,
    pub(crate) waterfall_reference: StrokeAppearance,
    pub(crate) raincloud_median: StrokeAppearance,
    pub(crate) beeswarm_reference: StrokeAppearance,
    pub(crate) interaction_interval: DataLineAppearance,
    pub(crate) interaction_connector: DataLineAppearance,
    /// Sizes of built-in point roles; plotted groups and their guides share these.
    pub(crate) point_sizes: PointSizeAppearance,
    /// Bar uncertainty stems/caps in either orientation.
    pub(crate) uncertainty_stroke: StrokeAppearance,
    /// Distribution summary and whisker strokes, including their guide glyphs.
    pub(crate) summary_stroke: StrokeAppearance,
    /// A Line series' data-colored path, reused in its legend glyph.
    pub(crate) series_line: DataLineAppearance,
    /// A Trajectory or LinePath series' data-colored path.
    pub(crate) path_line: DataLineAppearance,
    /// Histogram KDE overlay independently resolved from the bin outline.
    pub(crate) histogram_curve: StrokeAppearance,
    /// Histogram rug ticks independently resolved from the KDE overlay.
    pub(crate) histogram_rug: StrokeAppearance,
    /// Density group curve, reused exactly by its originating guide glyph.
    pub(crate) density_line: DataLineAppearance,
    /// Regression fit line independently resolved from its observations/band.
    pub(crate) regression_fit: StrokeAppearance,
    /// Estimate intervals do not borrow axis or bar uncertainty strokes.
    pub(crate) estimate_interval: StrokeAppearance,
    /// PointInterval's solid reference at an exact caller value.
    pub(crate) point_interval_reference: StrokeAppearance,
    /// Coefficient/Forest reference has its own fixed medium-dash grammar.
    pub(crate) estimate_reference: StrokeAppearance,
    /// Q-Q reference is independent of the estimate-display reference role.
    pub(crate) qq_reference: StrokeAppearance,
    /// Dumbbell pairs use one neutral connecting stroke.
    pub(crate) dumbbell_connector: StrokeAppearance,
}

/// A filled mark consumes one opaque scale color and independent fill/outline
/// roles; neither paint inherits alpha from the other.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DataFillAppearance {
    pub(crate) fill_alpha: u8,
    pub(crate) outline_paint: FillOutlinePaint,
    pub(crate) outline_width: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FillOutlinePaint {
    Fixed(Rgba),
    Data,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FixedFillAppearance {
    pub(crate) paint: Rgba,
    pub(crate) outline_paint: Rgba,
    pub(crate) outline_width: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FillOutlineAppearance {
    pub(crate) paint: Rgba,
    pub(crate) width: f32,
}

impl FillOutlineAppearance {
    pub(crate) fn backend_style(&self, fill: Rgba) -> crate::render::backend::FillStyle {
        crate::render::backend::FillStyle::new(fill, self.paint, self.width)
    }
}

impl FixedFillAppearance {
    pub(crate) fn backend_style(&self) -> crate::render::backend::FillStyle {
        crate::render::backend::FillStyle::new(self.paint, self.outline_paint, self.outline_width)
    }
}

impl DataFillAppearance {
    pub(crate) fn backend_style(&self, color: (u8, u8, u8)) -> crate::render::backend::FillStyle {
        crate::render::backend::FillStyle::new(
            Rgba::from(color).with_alpha(self.fill_alpha),
            match self.outline_paint {
                FillOutlinePaint::Fixed(paint) => paint,
                FillOutlinePaint::Data => color.into(),
            },
            self.outline_width,
        )
    }
}

/// One complete default point paint and its independently painted outline.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PointAppearance {
    pub(crate) fill_alpha: u8,
    pub(crate) shape: crate::PointShape,
    pub(crate) outline: crate::render::backend::PointOutline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PointSizeAppearance {
    pub(crate) scatter: u32,
    pub(crate) path: u32,
    pub(crate) trajectory: u32,
    pub(crate) regression: u32,
    pub(crate) qq: u32,
    pub(crate) marginal: u32,
    pub(crate) matrix: u32,
    pub(crate) interval_estimate: u32,
    pub(crate) dumbbell: u32,
    pub(crate) interaction: u32,
    pub(crate) beeswarm: u32,
    pub(crate) box_flier: u32,
}

impl PointSizeAppearance {
    fn publication() -> Self {
        Self {
            scatter: 3,
            path: 3,
            trajectory: 4,
            regression: 3,
            qq: 3,
            marginal: 3,
            matrix: 2,
            interval_estimate: 4,
            dumbbell: 4,
            interaction: 4,
            beeswarm: 3,
            box_flier: 2,
        }
    }
}

impl PointAppearance {
    pub(crate) fn backend_style(
        &self,
        color: (u8, u8, u8),
        radius: u32,
        outline_visible: bool,
    ) -> crate::render::backend::PointStyle {
        crate::render::backend::PointStyle {
            color: Rgba::from(color).with_alpha(self.fill_alpha),
            radius,
            outline: outline_visible.then_some(self.outline),
            shape: self.shape,
        }
    }
}

/// One complete data-colored mark stroke; the scale supplies opaque RGB only.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DataLineAppearance {
    pub(crate) width: f32,
    pub(crate) alpha: u8,
    pub(crate) pattern: StrokePattern,
}

impl DataLineAppearance {
    pub(crate) fn backend_style(&self, color: (u8, u8, u8)) -> LineStyle {
        LineStyle {
            color: Rgba::new(color.0, color.1, color.2, self.alpha),
            width: self.width,
            pattern: self.pattern,
        }
    }
}

/// Annotation paths resolve independently of axes, marks, and label typography.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AnnotationAppearance {
    pub(crate) bracket_stroke: BracketStrokeAppearance,
    /// Clearance from a labeled supplemental point's painted radius.
    pub(crate) point_label_gap: f64,
    /// Opaque calculation-only backdrop when output surfaces are translucent.
    pub(crate) planning_backdrop: Rgba,
}

/// Bracket paths have a fixed solid topology; only their paint and width vary.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BracketStrokeAppearance {
    pub(crate) paint: Rgba,
    pub(crate) width: f32,
}

/// Structurally present frame appearance.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FrameAppearance {
    /// Complete frame stroke.
    pub(crate) stroke: StrokeAppearance,
    /// Original preset setting retained until T5a5 normalizes frame topology.
    pub(crate) source_show_top_right_spines: bool,
}

/// Structurally present tick-mark appearance.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TickAppearance {
    /// Complete tick stroke.
    pub(crate) stroke: StrokeAppearance,
    /// Tick length in pixels.
    pub(crate) length: u32,
}

/// Role-local axis spacing in pixels.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AxisSpacing {
    /// X tick-mark to tick-label gap.
    pub(crate) x_tick_label_gap: f64,
    /// Y tick-mark to tick-label gap.
    pub(crate) y_tick_label_gap: f64,
    /// X tick-label to axis-title gap.
    pub(crate) x_title_tick_label_gap: f64,
    /// X tick-mark to axis-title gap when labels are absent.
    pub(crate) x_title_without_tick_labels_gap: f64,
    /// Y tick-label to axis-title gap.
    pub(crate) y_title_tick_label_gap: f64,
    /// Minimum y-axis title clearance from the canvas edge.
    pub(crate) y_title_edge_padding: f64,
}

/// Structurally resolved axis, frame, tick, and grid appearance.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AxisAppearance {
    /// Optional complete frame style; `None` means no frame.
    pub(crate) frame: Option<FrameAppearance>,
    /// Optional complete tick style; `None` means no tick marks.
    pub(crate) ticks: Option<TickAppearance>,
    /// Optional complete grid style; `None` means no grid.
    pub(crate) grid: Option<StrokeAppearance>,
    /// Axis-local spacing relationships.
    pub(crate) spacing: AxisSpacing,
}

/// Physical horizontal alignment of a root text role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TitleAlignment {
    /// Align to the physical left edge of the local span.
    #[allow(
        dead_code,
        reason = "private alignment is activated by future public themes"
    )]
    Start,
    /// Align to the physical midpoint of the local span.
    Center,
    /// Align to the physical right edge of the local span.
    #[allow(
        dead_code,
        reason = "private alignment is activated by future public themes"
    )]
    End,
}

impl TitleAlignment {
    /// Resolve this alignment into one physical x coordinate and backend anchor.
    pub(crate) fn resolve_physical(self, span_start: f64, span_end: f64) -> (f64, TextAnchor) {
        match self {
            Self::Start => (span_start, TextAnchor::Start),
            Self::Center => (
                span_start + (span_end - span_start) / 2.0,
                TextAnchor::Middle,
            ),
            Self::End => (span_end, TextAnchor::End),
        }
    }
}

/// Root title-block appearance and local spacing.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RootTitleBlockAppearance {
    /// Gap between adjacent title and subtitle line boxes.
    pub(crate) title_subtitle_gap: f64,
    /// Clearance between the top title block and root content.
    pub(crate) title_block_to_content_clearance: f64,
    /// Clearance between root content and the caption.
    pub(crate) caption_to_content_clearance: f64,
}

/// Root and TextPanel composition appearance.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CompositionAppearance {
    pub(crate) root_title_alignment: TitleAlignment,
    pub(crate) root_subtitle_alignment: Option<TitleAlignment>,
    pub(crate) root_caption_alignment: TitleAlignment,
    /// Plot or Canvas root title block.
    pub(crate) root_title_block: RootTitleBlockAppearance,
    pub(crate) text_panel_inset: f64,
    pub(crate) text_panel_gap: f64,
    pub(crate) facet_gap: f64,
    pub(crate) canvas_grid_gap: f64,
    pub(crate) column_header_padding: f64,
    pub(crate) row_header_padding: f64,
    pub(crate) row_title_horizontal_padding: f64,
    pub(crate) row_title_vertical_padding: f64,
}

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "the four closed presets provide one compact table of preserved scalar values"
)]
fn resolved_theme(
    colors: Vec<(u8, u8, u8)>,
    foreground: (u8, u8, u8),
    axis_width: f32,
    bracket_width: f32,
    title_font_size: u32,
    subtitle_font_size: u32,
    caption_font_size: u32,
    axis_title_font_size: u32,
    grid_color: (u8, u8, u8),
    source_show_top_right_spines: bool,
) -> ResolvedTheme {
    let stack = crate::DEFAULT_FONT_FAMILY_STACK.to_owned();
    let foreground_paint = Rgba::from(foreground);
    let muted_paint = Rgba::from(MUTED_TEXT);

    ResolvedTheme {
        colors,
        font_family: stack.clone(),
        foreground,
        detail_font_size: 14,
        label_font_size: axis_title_font_size,
        composition_title_font_size: title_font_size,
        facet_header_font_size: subtitle_font_size,
        surfaces: SurfaceAppearance {
            canvas_root: Rgba::rgb(255, 255, 255),
            plot_panel: Rgba::rgb(255, 255, 255),
        },
        canvas_surface_override: None,
        typography: FoundationalTypography {
            title: text_appearance(
                &stack,
                foreground_paint,
                title_font_size,
                plus_six_line_height(title_font_size),
            ),
            subtitle: text_appearance(
                &stack,
                muted_paint,
                subtitle_font_size,
                plus_six_line_height(subtitle_font_size),
            ),
            caption: text_appearance(
                &stack,
                muted_paint,
                caption_font_size,
                plus_six_line_height(caption_font_size),
            ),
            axis_title: text_appearance(&stack, foreground_paint, axis_title_font_size, 1.2),
            tick_label: text_appearance(&stack, foreground_paint, TICK_FONT_SIZE, 1.2),
            significance_label: text_appearance(
                &stack,
                foreground_paint,
                SIGNIFICANCE_FONT_SIZE,
                1.2,
            ),
            estimate_detail: text_appearance(&stack, foreground_paint, 14, 1.2),
            waterfall_total: text_appearance(&stack, foreground_paint, 14, 1.2),
            point_layer_label: text_appearance(&stack, foreground_paint, 10, 1.2),
            panel_text: text_appearance(&stack, foreground_paint, 14, 1.2),
            contrast_light: text_appearance(&stack, Rgba::rgb(255, 255, 255), 11, 1.2),
            contrast_dark: text_appearance(&stack, Rgba::rgb(0, 0, 0), 11, 1.2),
            legend_title: text_appearance(&stack, Rgba::rgb(60, 60, 60), 12, 1.2),
            legend_label: text_appearance(&stack, Rgba::rgb(40, 40, 40), 11, 1.2),
        },
        legend: LegendAppearance {
            padding: 8.8,
            key_width: 11.0,
            key_label_gap: 8.8,
            outside_gap: 8.8,
            shared_gap: 17.6,
            gradient_min_length: 40.0,
            gradient_preferred_length: 160.0,
            row_gap: 3.3,
            column_gap: 13.2,
            title_gap: 7.8,
            frame_fill: Rgba::new(255, 255, 255, 204),
            frame_outline: Rgba::rgb(200, 200, 200),
            frame_stroke_width: 1.0,
            frame_radius: 2.0,
        },
        axes: AxisAppearance {
            frame: Some(FrameAppearance {
                stroke: stroke_appearance(foreground_paint, axis_width, StrokePattern::Solid),
                source_show_top_right_spines,
            }),
            ticks: Some(TickAppearance {
                stroke: stroke_appearance(foreground_paint, axis_width, StrokePattern::Solid),
                length: 5,
            }),
            grid: Some(stroke_appearance(
                Rgba::from(grid_color),
                0.5,
                StrokePattern::MediumDash,
            )),
            spacing: AxisSpacing {
                x_tick_label_gap: 8.0,
                y_tick_label_gap: 6.0,
                x_title_tick_label_gap: 8.0,
                x_title_without_tick_labels_gap: 16.0,
                y_title_tick_label_gap: 16.0,
                y_title_edge_padding: 12.0,
            },
        },
        marks: MarkAppearance {
            primary_point: PointAppearance {
                fill_alpha: 209,
                shape: crate::PointShape::Circle,
                outline: crate::render::backend::PointOutline::default(),
            },
            bar_body: DataFillAppearance {
                fill_alpha: 255,
                outline_paint: FillOutlinePaint::Fixed(foreground_paint),
                outline_width: 1.0,
            },
            line_band: DataFillAppearance {
                fill_alpha: 46,
                outline_paint: FillOutlinePaint::Data,
                outline_width: 0.0,
            },
            regression_band: DataFillAppearance {
                fill_alpha: 64,
                outline_paint: FillOutlinePaint::Data,
                outline_width: 0.0,
            },
            density_area: DataFillAppearance {
                fill_alpha: 56,
                outline_paint: FillOutlinePaint::Data,
                outline_width: 0.0,
            },
            marginal_density: DataFillAppearance {
                fill_alpha: 115,
                outline_paint: FillOutlinePaint::Data,
                outline_width: 1.0,
            },
            matrix_density: DataFillAppearance {
                fill_alpha: 102,
                outline_paint: FillOutlinePaint::Data,
                outline_width: 1.0,
            },
            box_body: DataFillAppearance {
                fill_alpha: 255,
                outline_paint: FillOutlinePaint::Fixed(foreground_paint),
                outline_width: 1.0,
            },
            violin_body: DataFillAppearance {
                fill_alpha: 178,
                outline_paint: FillOutlinePaint::Fixed(foreground_paint),
                outline_width: 1.0,
            },
            raincloud_box: FixedFillAppearance {
                paint: Rgba::rgb(255, 255, 255),
                outline_paint: foreground_paint,
                outline_width: 1.0,
            },
            histogram_bin: DataFillAppearance {
                fill_alpha: 209,
                outline_paint: FillOutlinePaint::Fixed(foreground_paint),
                outline_width: 0.8,
            },
            waterfall_bar: DataFillAppearance {
                fill_alpha: 255,
                outline_paint: FillOutlinePaint::Fixed(foreground_paint),
                outline_width: 0.8,
            },
            raw_overlay_point: PointAppearance {
                fill_alpha: 209,
                shape: crate::PointShape::Circle,
                outline: crate::render::backend::PointOutline::default(),
            },
            raw_overlay_radius: 2,
            heatmap_cell_outline: FillOutlineAppearance {
                paint: Rgba::rgb(255, 255, 255),
                width: 0.5,
            },
            heatmap_highlight_outline: FillOutlineAppearance {
                paint: foreground_paint,
                width: 2.5,
            },
            heatmap_missing_fill: Rgba::rgb(224, 224, 224),
            heatmap_missing_row_outline: FillOutlineAppearance {
                paint: Rgba::rgb(255, 255, 255),
                width: 0.5,
            },
            heatmap_missing_key_outline: FillOutlineAppearance {
                paint: Rgba::rgb(120, 120, 120),
                width: 0.8,
            },
            heatmap_missing_cross: StrokeAppearance {
                paint: Rgba::rgb(120, 120, 120),
                width: 1.0,
                pattern: StrokePattern::Solid,
            },
            bar_zero: StrokeAppearance {
                paint: foreground_paint,
                width: 0.8,
                pattern: StrokePattern::Solid,
            },
            waterfall_connector: StrokeAppearance {
                paint: foreground_paint,
                width: 0.7,
                pattern: StrokePattern::ShortDash,
            },
            waterfall_reference: StrokeAppearance {
                paint: foreground_paint,
                width: 0.7,
                pattern: StrokePattern::MediumDash,
            },
            raincloud_median: StrokeAppearance {
                paint: foreground_paint,
                width: 1.5,
                pattern: StrokePattern::Solid,
            },
            beeswarm_reference: StrokeAppearance {
                paint: foreground_paint,
                width: 0.8,
                pattern: StrokePattern::Solid,
            },
            interaction_interval: DataLineAppearance {
                width: 1.2,
                alpha: 255,
                pattern: StrokePattern::Solid,
            },
            interaction_connector: DataLineAppearance {
                width: 1.8,
                alpha: 255,
                pattern: StrokePattern::Solid,
            },
            point_sizes: PointSizeAppearance::publication(),
            uncertainty_stroke: stroke_appearance(
                foreground_paint,
                axis_width,
                StrokePattern::Solid,
            ),
            summary_stroke: stroke_appearance(foreground_paint, axis_width, StrokePattern::Solid),
            series_line: DataLineAppearance {
                width: 1.8,
                alpha: 255,
                pattern: StrokePattern::Solid,
            },
            path_line: DataLineAppearance {
                width: 1.4,
                alpha: 255,
                pattern: StrokePattern::Solid,
            },
            histogram_curve: stroke_appearance(foreground_paint, 1.0, StrokePattern::Solid),
            histogram_rug: stroke_appearance(foreground_paint, 0.7, StrokePattern::Solid),
            density_line: DataLineAppearance {
                width: 1.6,
                alpha: 255,
                pattern: StrokePattern::Solid,
            },
            regression_fit: stroke_appearance(foreground_paint, 1.5, StrokePattern::Solid),
            estimate_interval: stroke_appearance(foreground_paint, 1.4, StrokePattern::Solid),
            point_interval_reference: stroke_appearance(
                foreground_paint,
                0.8,
                StrokePattern::Solid,
            ),
            estimate_reference: stroke_appearance(foreground_paint, 1.0, StrokePattern::MediumDash),
            qq_reference: stroke_appearance(foreground_paint, 1.2, StrokePattern::Solid),
            dumbbell_connector: stroke_appearance(foreground_paint, 1.2, StrokePattern::Solid),
        },
        annotations: AnnotationAppearance {
            bracket_stroke: BracketStrokeAppearance {
                paint: foreground_paint,
                width: bracket_width,
            },
            point_label_gap: 2.0,
            planning_backdrop: Rgba::rgb(255, 255, 255),
        },
        composition: CompositionAppearance {
            root_title_alignment: TitleAlignment::Center,
            root_subtitle_alignment: None,
            root_caption_alignment: TitleAlignment::Start,
            root_title_block: RootTitleBlockAppearance {
                title_subtitle_gap: 0.0,
                title_block_to_content_clearance: 14.0,
                caption_to_content_clearance: 12.0,
            },
            text_panel_inset: 16.0,
            text_panel_gap: 12.0,
            facet_gap: 12.0,
            canvas_grid_gap: 12.0,
            column_header_padding: 16.0,
            row_header_padding: 20.0,
            row_title_horizontal_padding: 16.0,
            row_title_vertical_padding: 8.0,
        },
        font_environment: Arc::new(OnceLock::new()),
        text_queries: Arc::default(),
    }
}

fn text_appearance(
    stack: &str,
    paint: Rgba,
    font_size: u32,
    line_height_multiplier: f64,
) -> TextAppearance {
    TextAppearance {
        stack: stack.to_owned(),
        paint,
        font_size,
        weight: TextWeight::Normal,
        line_height_multiplier,
    }
}

fn stroke_appearance(paint: Rgba, width: f32, pattern: StrokePattern) -> StrokeAppearance {
    StrokeAppearance {
        paint,
        width,
        pattern,
    }
}

fn plus_six_line_height(font_size: u32) -> f64 {
    (f64::from(font_size) + 6.0) / f64::from(font_size)
}

fn publication_theme() -> ResolvedTheme {
    // Okabe-Ito 8-color qualitative palette (colorblind-safe).
    resolved_theme(
        vec![
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
        false,
    )
}

fn nature_theme() -> ResolvedTheme {
    resolved_theme(
        vec![
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
        false,
    )
}

fn science_theme() -> ResolvedTheme {
    resolved_theme(
        vec![
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
        true,
    )
}

fn chi_theme() -> ResolvedTheme {
    resolved_theme(
        vec![
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
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUILT_IN_THEMES: [Theme; 4] = [
        Theme::Publication,
        Theme::Nature,
        Theme::Science,
        Theme::CHI,
    ];

    #[derive(Clone, Copy)]
    struct ExpectedPreset {
        foreground: (u8, u8, u8),
        axis_width: f32,
        bracket_width: f32,
        title_size: u32,
        subtitle_size: u32,
        caption_size: u32,
        axis_title_size: u32,
        grid_color: (u8, u8, u8),
        top_right_spines: bool,
    }

    fn expected_preset(theme: Theme) -> (Vec<(u8, u8, u8)>, ExpectedPreset) {
        match theme {
            Theme::Publication => (
                vec![
                    (0, 114, 178),
                    (213, 94, 0),
                    (0, 158, 115),
                    (204, 121, 167),
                    (86, 180, 233),
                    (230, 159, 0),
                    (240, 228, 66),
                    (0, 0, 0),
                ],
                ExpectedPreset {
                    foreground: (0, 0, 0),
                    axis_width: 0.8,
                    bracket_width: 1.0,
                    title_size: 20,
                    subtitle_size: 15,
                    caption_size: 15,
                    axis_title_size: 17,
                    grid_color: (220, 220, 220),
                    top_right_spines: false,
                },
            ),
            Theme::Nature => (
                vec![
                    (40, 86, 130),
                    (130, 160, 188),
                    (200, 215, 230),
                    (120, 130, 90),
                    (180, 175, 130),
                ],
                ExpectedPreset {
                    foreground: (40, 40, 40),
                    axis_width: 0.8,
                    bracket_width: 0.8,
                    title_size: 19,
                    subtitle_size: 14,
                    caption_size: 14,
                    axis_title_size: 16,
                    grid_color: (230, 230, 230),
                    top_right_spines: false,
                },
            ),
            Theme::Science => (
                vec![
                    (255, 0, 0),
                    (0, 0, 255),
                    (0, 175, 0),
                    (255, 175, 0),
                    (160, 32, 240),
                ],
                ExpectedPreset {
                    foreground: (0, 0, 0),
                    axis_width: 1.0,
                    bracket_width: 1.0,
                    title_size: 21,
                    subtitle_size: 16,
                    caption_size: 15,
                    axis_title_size: 17,
                    grid_color: (200, 200, 200),
                    top_right_spines: true,
                },
            ),
            Theme::CHI => (
                vec![
                    (171, 199, 232),
                    (243, 169, 175),
                    (174, 224, 174),
                    (244, 213, 152),
                    (197, 176, 213),
                ],
                ExpectedPreset {
                    foreground: (60, 60, 60),
                    axis_width: 0.8,
                    bracket_width: 0.8,
                    title_size: 19,
                    subtitle_size: 15,
                    caption_size: 15,
                    axis_title_size: 17,
                    grid_color: (220, 220, 220),
                    top_right_spines: false,
                },
            ),
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one complete built-in owner fixture keeps every resolved role explicit"
    )]
    fn expected_theme(theme: Theme) -> ResolvedTheme {
        let (colors, expected) = expected_preset(theme);
        let stack = crate::DEFAULT_FONT_FAMILY_STACK.to_owned();
        let foreground = Rgba::from(expected.foreground);
        let text = |paint, font_size, line_height_multiplier| TextAppearance {
            stack: stack.clone(),
            paint,
            font_size,
            weight: TextWeight::Normal,
            line_height_multiplier,
        };

        ResolvedTheme {
            colors,
            font_family: stack.clone(),
            foreground: expected.foreground,
            detail_font_size: 14,
            label_font_size: expected.axis_title_size,
            composition_title_font_size: expected.title_size,
            facet_header_font_size: expected.subtitle_size,
            surfaces: SurfaceAppearance {
                canvas_root: Rgba::new(255, 255, 255, 255),
                plot_panel: Rgba::new(255, 255, 255, 255),
            },
            canvas_surface_override: None,
            typography: FoundationalTypography {
                title: text(
                    foreground,
                    expected.title_size,
                    (f64::from(expected.title_size) + 6.0) / f64::from(expected.title_size),
                ),
                subtitle: text(
                    Rgba::rgb(80, 80, 80),
                    expected.subtitle_size,
                    (f64::from(expected.subtitle_size) + 6.0) / f64::from(expected.subtitle_size),
                ),
                caption: text(
                    Rgba::rgb(80, 80, 80),
                    expected.caption_size,
                    (f64::from(expected.caption_size) + 6.0) / f64::from(expected.caption_size),
                ),
                axis_title: text(foreground, expected.axis_title_size, 1.2),
                tick_label: text(foreground, 16, 1.2),
                significance_label: text(foreground, 16, 1.2),
                estimate_detail: text(foreground, 14, 1.2),
                waterfall_total: text(foreground, 14, 1.2),
                point_layer_label: text(foreground, 10, 1.2),
                panel_text: text(foreground, 14, 1.2),
                contrast_light: text(Rgba::rgb(255, 255, 255), 11, 1.2),
                contrast_dark: text(Rgba::rgb(0, 0, 0), 11, 1.2),
                legend_title: text(Rgba::rgb(60, 60, 60), 12, 1.2),
                legend_label: text(Rgba::rgb(40, 40, 40), 11, 1.2),
            },
            legend: LegendAppearance {
                padding: 8.8,
                key_width: 11.0,
                key_label_gap: 8.8,
                outside_gap: 8.8,
                shared_gap: 17.6,
                gradient_min_length: 40.0,
                gradient_preferred_length: 160.0,
                row_gap: 3.3,
                column_gap: 13.2,
                title_gap: 7.8,
                frame_fill: Rgba::new(255, 255, 255, 204),
                frame_outline: Rgba::rgb(200, 200, 200),
                frame_stroke_width: 1.0,
                frame_radius: 2.0,
            },
            axes: AxisAppearance {
                frame: Some(FrameAppearance {
                    stroke: StrokeAppearance {
                        paint: foreground,
                        width: expected.axis_width,
                        pattern: StrokePattern::Solid,
                    },
                    source_show_top_right_spines: expected.top_right_spines,
                }),
                ticks: Some(TickAppearance {
                    stroke: StrokeAppearance {
                        paint: foreground,
                        width: expected.axis_width,
                        pattern: StrokePattern::Solid,
                    },
                    length: 5,
                }),
                grid: Some(StrokeAppearance {
                    paint: Rgba::from(expected.grid_color),
                    width: 0.5,
                    pattern: StrokePattern::MediumDash,
                }),
                spacing: AxisSpacing {
                    x_tick_label_gap: 8.0,
                    y_tick_label_gap: 6.0,
                    x_title_tick_label_gap: 8.0,
                    x_title_without_tick_labels_gap: 16.0,
                    y_title_tick_label_gap: 16.0,
                    y_title_edge_padding: 12.0,
                },
            },
            marks: MarkAppearance {
                primary_point: PointAppearance {
                    fill_alpha: 209,
                    shape: crate::PointShape::Circle,
                    outline: crate::render::backend::PointOutline::default(),
                },
                bar_body: DataFillAppearance {
                    fill_alpha: 255,
                    outline_paint: FillOutlinePaint::Fixed(foreground),
                    outline_width: 1.0,
                },
                line_band: DataFillAppearance {
                    fill_alpha: 46,
                    outline_paint: FillOutlinePaint::Data,
                    outline_width: 0.0,
                },
                regression_band: DataFillAppearance {
                    fill_alpha: 64,
                    outline_paint: FillOutlinePaint::Data,
                    outline_width: 0.0,
                },
                density_area: DataFillAppearance {
                    fill_alpha: 56,
                    outline_paint: FillOutlinePaint::Data,
                    outline_width: 0.0,
                },
                marginal_density: DataFillAppearance {
                    fill_alpha: 115,
                    outline_paint: FillOutlinePaint::Data,
                    outline_width: 1.0,
                },
                matrix_density: DataFillAppearance {
                    fill_alpha: 102,
                    outline_paint: FillOutlinePaint::Data,
                    outline_width: 1.0,
                },
                box_body: DataFillAppearance {
                    fill_alpha: 255,
                    outline_paint: FillOutlinePaint::Fixed(foreground),
                    outline_width: 1.0,
                },
                violin_body: DataFillAppearance {
                    fill_alpha: 178,
                    outline_paint: FillOutlinePaint::Fixed(foreground),
                    outline_width: 1.0,
                },
                raincloud_box: FixedFillAppearance {
                    paint: Rgba::rgb(255, 255, 255),
                    outline_paint: foreground,
                    outline_width: 1.0,
                },
                histogram_bin: DataFillAppearance {
                    fill_alpha: 209,
                    outline_paint: FillOutlinePaint::Fixed(foreground),
                    outline_width: 0.8,
                },
                waterfall_bar: DataFillAppearance {
                    fill_alpha: 255,
                    outline_paint: FillOutlinePaint::Fixed(foreground),
                    outline_width: 0.8,
                },
                raw_overlay_point: PointAppearance {
                    fill_alpha: 209,
                    shape: crate::PointShape::Circle,
                    outline: crate::render::backend::PointOutline::default(),
                },
                raw_overlay_radius: 2,
                heatmap_cell_outline: FillOutlineAppearance {
                    paint: Rgba::rgb(255, 255, 255),
                    width: 0.5,
                },
                heatmap_highlight_outline: FillOutlineAppearance {
                    paint: foreground,
                    width: 2.5,
                },
                heatmap_missing_fill: Rgba::rgb(224, 224, 224),
                heatmap_missing_row_outline: FillOutlineAppearance {
                    paint: Rgba::rgb(255, 255, 255),
                    width: 0.5,
                },
                heatmap_missing_key_outline: FillOutlineAppearance {
                    paint: Rgba::rgb(120, 120, 120),
                    width: 0.8,
                },
                heatmap_missing_cross: StrokeAppearance {
                    paint: Rgba::rgb(120, 120, 120),
                    width: 1.0,
                    pattern: StrokePattern::Solid,
                },
                bar_zero: StrokeAppearance {
                    paint: foreground,
                    width: 0.8,
                    pattern: StrokePattern::Solid,
                },
                waterfall_connector: StrokeAppearance {
                    paint: foreground,
                    width: 0.7,
                    pattern: StrokePattern::ShortDash,
                },
                waterfall_reference: StrokeAppearance {
                    paint: foreground,
                    width: 0.7,
                    pattern: StrokePattern::MediumDash,
                },
                raincloud_median: StrokeAppearance {
                    paint: foreground,
                    width: 1.5,
                    pattern: StrokePattern::Solid,
                },
                beeswarm_reference: StrokeAppearance {
                    paint: foreground,
                    width: 0.8,
                    pattern: StrokePattern::Solid,
                },
                interaction_interval: DataLineAppearance {
                    width: 1.2,
                    alpha: 255,
                    pattern: StrokePattern::Solid,
                },
                interaction_connector: DataLineAppearance {
                    width: 1.8,
                    alpha: 255,
                    pattern: StrokePattern::Solid,
                },
                point_sizes: PointSizeAppearance::publication(),
                uncertainty_stroke: StrokeAppearance {
                    paint: foreground,
                    width: expected.axis_width,
                    pattern: StrokePattern::Solid,
                },
                summary_stroke: StrokeAppearance {
                    paint: foreground,
                    width: expected.axis_width,
                    pattern: StrokePattern::Solid,
                },
                series_line: DataLineAppearance {
                    width: 1.8,
                    alpha: 255,
                    pattern: StrokePattern::Solid,
                },
                path_line: DataLineAppearance {
                    width: 1.4,
                    alpha: 255,
                    pattern: StrokePattern::Solid,
                },
                histogram_curve: StrokeAppearance {
                    paint: foreground,
                    width: 1.0,
                    pattern: StrokePattern::Solid,
                },
                histogram_rug: StrokeAppearance {
                    paint: foreground,
                    width: 0.7,
                    pattern: StrokePattern::Solid,
                },
                density_line: DataLineAppearance {
                    width: 1.6,
                    alpha: 255,
                    pattern: StrokePattern::Solid,
                },
                regression_fit: StrokeAppearance {
                    paint: foreground,
                    width: 1.5,
                    pattern: StrokePattern::Solid,
                },
                estimate_interval: StrokeAppearance {
                    paint: foreground,
                    width: 1.4,
                    pattern: StrokePattern::Solid,
                },
                point_interval_reference: StrokeAppearance {
                    paint: foreground,
                    width: 0.8,
                    pattern: StrokePattern::Solid,
                },
                estimate_reference: StrokeAppearance {
                    paint: foreground,
                    width: 1.0,
                    pattern: StrokePattern::MediumDash,
                },
                qq_reference: StrokeAppearance {
                    paint: foreground,
                    width: 1.2,
                    pattern: StrokePattern::Solid,
                },
                dumbbell_connector: StrokeAppearance {
                    paint: foreground,
                    width: 1.2,
                    pattern: StrokePattern::Solid,
                },
            },
            annotations: AnnotationAppearance {
                bracket_stroke: BracketStrokeAppearance {
                    paint: foreground,
                    width: expected.bracket_width,
                },
                point_label_gap: 2.0,
                planning_backdrop: Rgba::rgb(255, 255, 255),
            },
            composition: CompositionAppearance {
                root_title_alignment: TitleAlignment::Center,
                root_subtitle_alignment: None,
                root_caption_alignment: TitleAlignment::Start,
                root_title_block: RootTitleBlockAppearance {
                    title_subtitle_gap: 0.0,
                    title_block_to_content_clearance: 14.0,
                    caption_to_content_clearance: 12.0,
                },
                text_panel_inset: 16.0,
                text_panel_gap: 12.0,
                facet_gap: 12.0,
                canvas_grid_gap: 12.0,
                column_header_padding: 16.0,
                row_header_padding: 20.0,
                row_title_horizontal_padding: 16.0,
                row_title_vertical_padding: 8.0,
            },
            font_environment: Arc::new(OnceLock::new()),
            text_queries: Arc::default(),
        }
    }

    #[test]
    fn built_in_themes_preserve_every_resolved_value() {
        for theme in BUILT_IN_THEMES {
            assert_eq!(theme.resolve(), expected_theme(theme), "{theme:?}");
        }
    }

    #[test]
    fn built_in_themes_have_complete_foundational_records() {
        for theme in BUILT_IN_THEMES {
            let resolved = theme.resolve();
            let text_roles = [
                &resolved.typography.title,
                &resolved.typography.subtitle,
                &resolved.typography.caption,
                &resolved.typography.axis_title,
                &resolved.typography.tick_label,
                &resolved.typography.significance_label,
            ];

            assert!(!resolved.colors.is_empty(), "{theme:?} has no colors");
            assert_eq!(resolved.font_family, crate::DEFAULT_FONT_FAMILY_STACK);
            assert!(resolved.marks.uncertainty_stroke.width.is_finite());
            assert!(resolved.marks.uncertainty_stroke.width > 0.0);
            assert!(resolved.marks.summary_stroke.width.is_finite());
            assert!(resolved.marks.summary_stroke.width > 0.0);
            assert!(resolved.annotations.bracket_stroke.width.is_finite());
            assert!(resolved.annotations.bracket_stroke.width > 0.0);
            assert_eq!(resolved.detail_font_size, 14);
            let (_, expected) = expected_preset(theme);
            assert_eq!(resolved.label_font_size, expected.axis_title_size);
            assert_eq!(resolved.composition_title_font_size, expected.title_size);
            assert_eq!(resolved.facet_header_font_size, expected.subtitle_size);
            assert!(text_roles.iter().all(|role| {
                !role.stack.trim().is_empty()
                    && role.font_size > 0
                    && role.line_height_multiplier.is_finite()
                    && role.line_height_multiplier > 0.0
            }));

            let frame = resolved.axes.frame.as_ref().expect("built-in frame");
            let ticks = resolved.axes.ticks.as_ref().expect("built-in ticks");
            let grid = resolved.axes.grid.as_ref().expect("built-in grid");
            assert!(frame.stroke.width.is_finite() && frame.stroke.width > 0.0);
            assert!(ticks.stroke.width.is_finite() && ticks.stroke.width > 0.0);
            assert!(grid.width.is_finite() && grid.width > 0.0);
            assert!(ticks.length > 0);
        }
    }

    #[test]
    fn built_in_foundational_paints_are_complete_rgba() {
        for theme in BUILT_IN_THEMES {
            let resolved = theme.resolve();
            let foreground = Rgba::from(resolved.foreground);
            assert_eq!(resolved.surfaces.canvas_root, Rgba::new(255, 255, 255, 255));
            assert_eq!(resolved.surfaces.plot_panel, Rgba::new(255, 255, 255, 255));
            assert_eq!(resolved.typography.title.paint, foreground);
            assert_eq!(resolved.typography.subtitle.paint, Rgba::rgb(80, 80, 80));
            assert_eq!(resolved.typography.caption.paint, Rgba::rgb(80, 80, 80));
            assert_eq!(resolved.typography.axis_title.paint, foreground);
            assert_eq!(resolved.typography.tick_label.paint, foreground);
            assert_eq!(resolved.typography.significance_label.paint, foreground);
        }
    }

    #[test]
    fn private_uncertainty_and_summary_strokes_are_independent_of_axis_chrome() {
        let mut theme = Theme::Publication.resolve();
        let axis = theme.axes.frame.as_ref().unwrap().stroke.backend_style();
        theme.marks.uncertainty_stroke.paint = Rgba::rgb(11, 23, 37);
        theme.marks.uncertainty_stroke.width = 2.5;
        theme.marks.summary_stroke.paint = Rgba::rgb(43, 59, 71);
        theme.marks.summary_stroke.width = 3.5;

        let uncertainty = theme.marks.uncertainty_stroke.backend_style();
        let summary = theme.marks.summary_stroke.backend_style();
        assert_eq!(uncertainty.color, Rgba::rgb(11, 23, 37));
        assert_eq!(uncertainty.width.to_bits(), 2.5_f32.to_bits());
        assert_eq!(summary.color, Rgba::rgb(43, 59, 71));
        assert_eq!(summary.width.to_bits(), 3.5_f32.to_bits());
        assert_eq!(
            theme.axes.frame.as_ref().unwrap().stroke.backend_style(),
            axis
        );
        assert_ne!(uncertainty, summary);
    }

    #[test]
    fn custom_bracket_stroke_changes_annotation_identity_without_recoloring_text_or_axes() {
        let mut theme = Theme::Nature.resolve();
        let original = theme.clone();
        theme.annotations.bracket_stroke.paint = Rgba::rgb(15, 29, 41);
        theme.annotations.bracket_stroke.width = 2.25;
        assert_ne!(theme, original);
        assert_eq!(
            theme.annotations.bracket_stroke.paint,
            Rgba::rgb(15, 29, 41)
        );
        assert_eq!(
            theme.annotations.bracket_stroke.width.to_bits(),
            2.25_f32.to_bits()
        );
        assert_eq!(
            theme.typography.significance_label,
            original.typography.significance_label
        );
        assert_eq!(theme.axes, original.axes);
        assert_eq!(theme.marks, original.marks);
    }

    #[test]
    fn point_label_gap_is_independent_of_point_and_text_appearance() {
        let mut theme = Theme::CHI.resolve();
        let original = theme.clone();
        theme.annotations.point_label_gap = 7.0;
        assert_ne!(theme, original);
        assert_eq!(
            theme.annotations.point_label_gap.to_bits(),
            7.0_f64.to_bits()
        );
        assert_eq!(theme.marks, original.marks);
        assert_eq!(theme.typography, original.typography);
        assert_eq!(theme.axes, original.axes);
    }

    #[test]
    fn composed_plot_contrast_uses_the_canvas_owner_surface() {
        let mut theme = Theme::Publication.resolve();
        theme.surfaces.canvas_root = Rgba::new(0, 0, 0, 0);
        theme.surfaces.plot_panel = Rgba::new(0, 0, 0, 0);
        theme.annotations.planning_backdrop = Rgba::rgb(0, 0, 0);
        let fill = Rgba::new(0, 0, 0, 0);
        assert_eq!(
            theme.contrasting_text(fill).unwrap(),
            &theme.typography.contrast_light
        );
        theme.canvas_surface_override = Some(Rgba::rgb(255, 255, 255));
        assert_eq!(
            theme.contrasting_text(fill).unwrap(),
            &theme.typography.contrast_dark
        );
    }

    #[test]
    fn histogram_curve_and_rug_styles_do_not_inherit_axis_or_bin_strokes() {
        let mut theme = Theme::Science.resolve();
        let original = theme.clone();
        theme.marks.histogram_curve.paint = Rgba::rgb(25, 49, 73);
        theme.marks.histogram_curve.width = 2.5;
        theme.marks.histogram_rug.paint = Rgba::rgb(97, 121, 145);
        theme.marks.histogram_rug.width = 0.5;
        assert_ne!(theme, original);
        assert_eq!(theme.axes, original.axes);
        assert_eq!(
            theme.marks.uncertainty_stroke,
            original.marks.uncertainty_stroke
        );
        assert_eq!(
            theme.marks.histogram_curve.backend_style().color,
            Rgba::rgb(25, 49, 73)
        );
        assert_eq!(
            theme.marks.histogram_rug.backend_style().color,
            Rgba::rgb(97, 121, 145)
        );
    }

    #[test]
    fn estimate_reference_and_dumbbell_roles_do_not_alias_each_other() {
        let mut theme = Theme::Nature.resolve();
        let original = theme.clone();
        theme.marks.estimate_interval.paint = Rgba::rgb(23, 47, 71);
        theme.marks.estimate_interval.width = 2.25;
        theme.marks.point_interval_reference.paint = Rgba::rgb(31, 53, 79);
        theme.marks.estimate_reference.width = 3.0;
        theme.marks.qq_reference.paint = Rgba::rgb(41, 67, 89);
        theme.marks.dumbbell_connector.width = 2.0;
        assert_ne!(theme, original);
        assert_eq!(theme.axes, original.axes);
        assert_eq!(
            theme.marks.uncertainty_stroke,
            original.marks.uncertainty_stroke
        );
        assert_eq!(
            theme.marks.estimate_reference.pattern,
            StrokePattern::MediumDash
        );
        assert_eq!(
            theme.marks.point_interval_reference.pattern,
            StrokePattern::Solid
        );
        assert_ne!(
            theme.marks.estimate_interval,
            theme.marks.estimate_reference
        );
        assert_ne!(theme.marks.qq_reference, theme.marks.dumbbell_connector);
    }

    #[test]
    fn built_in_line_heights_preserve_current_geometry() {
        for theme in BUILT_IN_THEMES {
            let typography = theme.resolve().typography;
            for role in [&typography.title, &typography.subtitle, &typography.caption] {
                let pitch = role.line_height_multiplier * f64::from(role.font_size);
                let expected = f64::from(role.font_size) + 6.0;
                assert!((pitch - expected).abs() < 1.0e-12, "{theme:?}");
            }
            for role in [
                &typography.axis_title,
                &typography.tick_label,
                &typography.significance_label,
            ] {
                assert!(
                    (role.line_height_multiplier - 1.2).abs() < f64::EPSILON,
                    "{theme:?}"
                );
            }
        }
    }

    #[test]
    fn all_built_ins_center_the_root_title_block() {
        for theme in BUILT_IN_THEMES {
            assert_eq!(
                theme.resolve().composition.root_title_alignment,
                TitleAlignment::Center,
                "{theme:?}"
            );
        }
    }

    #[test]
    fn root_alignment_resolves_physical_x_and_anchor() {
        let span = (12.0, 92.0);
        assert_eq!(
            TitleAlignment::Start.resolve_physical(span.0, span.1),
            (12.0, TextAnchor::Start)
        );
        assert_eq!(
            TitleAlignment::Center.resolve_physical(span.0, span.1),
            (52.0, TextAnchor::Middle)
        );
        assert_eq!(
            TitleAlignment::End.resolve_physical(span.0, span.1),
            (92.0, TextAnchor::End)
        );
    }

    #[test]
    fn built_in_axis_chrome_is_structural_and_preserves_sources() {
        for theme in BUILT_IN_THEMES {
            let resolved = theme.resolve();
            let (_, expected) = expected_preset(theme);
            let frame = resolved.axes.frame.expect("built-in frame");
            let ticks = resolved.axes.ticks.expect("built-in ticks");
            let grid = resolved.axes.grid.expect("built-in grid");

            assert_eq!(frame.stroke.paint, Rgba::from(expected.foreground));
            assert!((frame.stroke.width - expected.axis_width).abs() < f32::EPSILON);
            assert_eq!(frame.stroke.pattern, StrokePattern::Solid);
            assert_eq!(
                frame.source_show_top_right_spines,
                expected.top_right_spines
            );
            assert_eq!(ticks.stroke, frame.stroke);
            assert_eq!(ticks.length, 5);
            assert_eq!(grid.paint, Rgba::from(expected.grid_color));
            assert!((grid.width - 0.5).abs() < f32::EPSILON);
            assert_eq!(grid.pattern, StrokePattern::MediumDash);
        }
    }

    #[test]
    fn significance_label_defaults_and_exact_precedence_are_complete() {
        for theme in BUILT_IN_THEMES {
            let typography = theme.resolve().typography;
            let default = typography.effective_significance_label(None);
            assert_eq!(default.font_size, 16, "{theme:?}");
            assert_eq!(default.weight, TextWeight::Normal, "{theme:?}");
            assert!(
                (default.line_height_multiplier - 1.2).abs() < f64::EPSILON,
                "{theme:?}"
            );

            let exact = SignificanceMarkStyle::new(23, TextWeight::Bold, 9.0)
                .expect("valid exact significance style");
            let effective = typography.effective_significance_label(Some(exact));
            assert_eq!(effective.stack, default.stack, "{theme:?}");
            assert_eq!(effective.paint, default.paint, "{theme:?}");
            assert_eq!(effective.font_size, 23, "{theme:?}");
            assert_eq!(effective.weight, TextWeight::Bold, "{theme:?}");
            assert!(
                (effective.line_height_multiplier - 1.2).abs() < f64::EPSILON,
                "{theme:?}"
            );
        }
    }

    #[test]
    fn composition_spacing_preserves_active_values() {
        for theme in BUILT_IN_THEMES {
            let composition = theme.resolve().composition;
            assert!(composition.root_title_block.title_subtitle_gap.abs() < f64::EPSILON);
            assert!(
                (composition
                    .root_title_block
                    .title_block_to_content_clearance
                    - 14.0)
                    .abs()
                    < f64::EPSILON
            );
            assert!(
                (composition.root_title_block.caption_to_content_clearance - 12.0).abs()
                    < f64::EPSILON
            );
        }
    }

    #[test]
    fn theme_clone_discovers_one_shared_font_environment() {
        FONT_ENVIRONMENT_DISCOVERIES.with(|calls| calls.set(0));
        let theme = Theme::Publication.resolve();
        let clone = theme.clone();

        theme.prepare_font_environment().unwrap();
        clone.prepare_font_environment().unwrap();

        assert!(theme.shares_font_environment_with(&clone));
        FONT_ENVIRONMENT_DISCOVERIES.with(|calls| assert_eq!(calls.get(), 1));
    }

    #[test]
    fn separately_resolved_theme_can_share_the_root_font_owner() {
        FONT_ENVIRONMENT_DISCOVERIES.with(|calls| calls.set(0));
        let root = Theme::Publication.resolve();
        let mut plot = Theme::Science.resolve();
        plot.share_font_environment_from(&root);

        root.prepare_font_environment().unwrap();
        plot.prepare_font_environment().unwrap();

        assert!(root.shares_font_environment_with(&plot));
        FONT_ENVIRONMENT_DISCOVERIES.with(|calls| assert_eq!(calls.get(), 1));
    }

    #[test]
    fn runtime_font_preparation_does_not_affect_appearance_equality() {
        let prepared = Theme::Publication.resolve();
        let unresolved = Theme::Publication.resolve();
        prepared.prepare_font_environment().unwrap();

        assert_eq!(prepared, unresolved);
        assert!(!prepared.shares_font_environment_with(&unresolved));
    }

    #[test]
    fn role_weight_changes_metrics_when_the_host_resolves_a_distinct_face() {
        let theme = Theme::Publication.resolve();
        let normal = theme
            .measure_text(
                "Minimum WWW maximum",
                &theme.typography.title,
                PhysicalTextAlignment::Center,
                "title",
                false,
            )
            .unwrap();
        let mut bold_role = theme.typography.title.clone();
        bold_role.weight = TextWeight::Bold;
        let bold = theme
            .measure_text(
                "Minimum WWW maximum",
                &bold_role,
                PhysicalTextAlignment::Center,
                "title",
                false,
            )
            .unwrap();

        if normal.pair_provenance.candidates.first() != bold.pair_provenance.candidates.first() {
            let normal_metrics = (
                normal.width_quanta,
                normal.height_quanta,
                normal.lines[0].logical_advance_quanta,
                normal.lines[0].ink_quanta,
            );
            let bold_metrics = (
                bold.width_quanta,
                bold.height_quanta,
                bold.lines[0].logical_advance_quanta,
                bold.lines[0].ink_quanta,
            );
            assert_ne!(normal_metrics, bold_metrics);
        }
    }

    #[test]
    fn text_block_views_do_not_rediscover_or_fallback() {
        FONT_ENVIRONMENT_DISCOVERIES.with(|calls| calls.set(0));
        let theme = Theme::Publication.resolve();
        let plan = theme
            .plan_text(
                "planned text",
                &theme.typography.title,
                300.0,
                PhysicalTextAlignment::Center,
                "title",
                false,
            )
            .unwrap();
        let block = crate::layout::TextBlock::from_resolved(plan);

        assert_eq!(block.lines(), ["planned text"]);
        assert!(theme
            .plan_text(
                "indivisible",
                &theme.typography.title,
                1.0,
                PhysicalTextAlignment::Center,
                "title",
                false,
            )
            .is_err());
        FONT_ENVIRONMENT_DISCOVERIES.with(|calls| assert_eq!(calls.get(), 1));
    }

    #[test]
    fn root_text_queries_share_payloads_but_not_profiles_or_constraints() {
        let root = Theme::Publication.resolve();
        let mut child = Theme::Science.resolve();
        child.share_font_environment_from(&root);
        let appearance = &root.typography.title;
        let first = root
            .plan_text(
                "Measured responses",
                appearance,
                400.0,
                PhysicalTextAlignment::Start,
                "title",
                false,
            )
            .unwrap();
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let second = child
            .plan_text(
                "Measured responses",
                appearance,
                400.0,
                PhysicalTextAlignment::Start,
                "title",
                false,
            )
            .unwrap();
        assert!(first.shares_payload(&second));
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        let narrower = root
            .plan_text(
                "Measured responses",
                appearance,
                390.0,
                PhysicalTextAlignment::Start,
                "title",
                false,
            )
            .unwrap();
        assert!(first.shares_payload(&narrower));
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        assert!(root
            .plan_text(
                "Measured responses",
                appearance,
                1.0,
                PhysicalTextAlignment::Start,
                "title",
                false
            )
            .is_err());
        let mut changed = appearance.clone();
        changed.paint = Rgba::rgb(3, 2, 1);
        let changed = root
            .plan_text(
                "Measured responses",
                &changed,
                400.0,
                PhysicalTextAlignment::Start,
                "title",
                false,
            )
            .unwrap();
        assert_eq!(changed.style.paint, Rgba::rgb(3, 2, 1));
        assert!(!first.shares_payload(&changed));
        assert!(first.shares_analysis(&changed));
        let other_root = Theme::Publication.resolve();
        assert!(!Arc::ptr_eq(&root.text_queries, &other_root.text_queries));
    }

    #[test]
    fn text_analysis_is_font_scoped_not_presentation_scoped() {
        let root = Theme::Publication.resolve();
        root.font_environment
            .set(Ok(PreparedFontEnvironment::synthetic()))
            .unwrap();
        let mut appearance = root.typography.caption.clone();
        appearance.stack = "Test Sans".into();
        appearance.font_size = 10;
        let first = root
            .plan_text(
                "A A A A",
                &appearance,
                20.0,
                PhysicalTextAlignment::Start,
                "first",
                false,
            )
            .unwrap();
        appearance.paint = Rgba::rgb(4, 5, 6);
        appearance.line_height_multiplier = 1.5;
        let restyled = root
            .plan_text(
                "A A A A",
                &appearance,
                15.0,
                PhysicalTextAlignment::End,
                "restyled",
                true,
            )
            .unwrap();
        assert!(first.shares_analysis(&restyled));
        assert!(!first.shares_payload(&restyled));
        assert_eq!(restyled.style.paint, appearance.paint);
        assert_eq!(restyled.style.line_height_quanta, 15 * 1024);
        assert_eq!(restyled.style.alignment, PhysicalTextAlignment::End);
        assert_eq!(restyled.style.provenance.owner, "restyled");
        assert!(restyled.style.exact_override);
        appearance.font_size = 11;
        let resized = root
            .plan_text(
                "A A A A",
                &appearance,
                15.0,
                PhysicalTextAlignment::End,
                "restyled",
                true,
            )
            .unwrap();
        assert!(!first.shares_analysis(&resized));
        appearance.font_size = 10;
        appearance.weight = crate::TextWeight::Bold;
        let reweighted = root
            .plan_text(
                "A A A A",
                &appearance,
                15.0,
                PhysicalTextAlignment::End,
                "restyled",
                true,
            )
            .unwrap();
        assert!(!first.shares_analysis(&reweighted));
    }

    #[test]
    fn intrinsic_query_memo_reuses_the_root_prepared_fonts() {
        let root = Theme::Publication.resolve();
        let first = root
            .intrinsic_tick_label_width(["Measured responses"].into_iter(), 100.0)
            .unwrap();
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let second = root
            .clone()
            .intrinsic_tick_label_width(["Measured responses"].into_iter(), 100.0)
            .unwrap();
        assert_eq!(first.to_bits(), second.to_bits());
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
    }

    fn indexed_query_input<'a>(
        source: &'a str,
        appearance: &'a TextAppearance,
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
        reason = "one root verifies index scaling, stable collision order, and exact charged comparisons"
    )]
    fn indexed_text_queries_charge_search_and_matching_buckets_only() {
        let root = Theme::Publication.resolve();
        root.font_environment
            .set(Ok(PreparedFontEnvironment::synthetic()))
            .unwrap();
        let environment = root.prepared_font_environment().unwrap();
        let resources = environment.generation_resources();
        environment.share_generation_resources(&resources).unwrap();
        let mut appearance = root.typography.caption.clone();
        appearance.stack = "Test Sans".into();
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
        let input = indexed_query_input("A", &appearance, "owner 000");
        let (_, fingerprint) = CachedTextQuery::fingerprints(&input, None, true);
        let comparisons = {
            let cache = root.text_queries.lock().unwrap();
            assert_eq!(cache.entries.len(), 128);
            assert_eq!(cache.exact.len(), 128);
            assert_eq!(
                cache.shaping.len(),
                1,
                "one analysis representative per compatible font/source profile"
            );
            assert!(cache.exact.windows(2).all(|pair| pair[0] < pair[1]));
            let (mut low, mut high, mut comparisons) = (0, cache.exact.len(), 0);
            while low < high {
                comparisons += 1;
                let middle = low + (high - low) / 2;
                if cache.exact[middle].fingerprint < fingerprint {
                    low = middle + 1;
                } else {
                    high = middle;
                }
            }
            comparisons
        };
        assert!(comparisons <= 8);
        let before = resources.usage().unwrap();
        let bytes = environment.resource_usage().unwrap().font_bytes;
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
        let after = resources.usage().unwrap();
        assert_eq!(after.1 - before.1, comparisons + 1);
        assert_eq!(after.0, before.0);
        assert_eq!(environment.resource_usage().unwrap().font_bytes, bytes);

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
        let before = resources.usage().unwrap().1;
        assert_eq!(
            cache
                .find(
                    &cache.exact,
                    [0; 32],
                    (&collision, None, true),
                    true,
                    environment
                )
                .unwrap(),
            Some(expected)
        );
        assert_eq!(
            resources.usage().unwrap().1 - before,
            8 + cache.exact.len() as u128
        );
        let before = resources.usage().unwrap().1;
        assert_eq!(
            cache
                .find(
                    &cache.shaping,
                    [0; 32],
                    (&collision, None, true),
                    false,
                    environment
                )
                .unwrap(),
            Some(expected)
        );
        assert_eq!(resources.usage().unwrap().1 - before, 4);
        let mut wrong_provenance = collision.clone();
        wrong_provenance.provenance.pair_ordinal += 1;
        assert!(cache
            .find(
                &cache.exact,
                [0; 32],
                (&wrong_provenance, None, true),
                true,
                environment
            )
            .unwrap()
            .is_none());
        assert_eq!(
            cache
                .find(
                    &cache.shaping,
                    [0; 32],
                    (&wrong_provenance, None, true),
                    false,
                    environment
                )
                .unwrap(),
            Some(expected)
        );
    }

    #[test]
    fn indexed_query_fingerprints_include_exact_presentation_and_provenance() {
        let appearance = Theme::Publication.resolve().typography.caption;
        let input = indexed_query_input("A", &appearance, "owner");
        let fingerprints = CachedTextQuery::fingerprints(&input, None, true);
        for change in 0..7 {
            let mut changed = input.clone();
            match change {
                0 => changed.maximum_width_quanta += 1,
                1 => changed.alignment = PhysicalTextAlignment::End,
                2 => changed.paint = Rgba::new(4, 5, 6, 7),
                3 => changed.line_height = TextLineHeight::ResolvedQuanta(1000),
                4 => changed.exact_override = true,
                5 => changed.provenance.owner.push('x'),
                _ => changed.provenance.pair_ordinal += 1,
            }
            let changed = CachedTextQuery::fingerprints(&changed, None, true);
            assert_eq!(changed.0, fingerprints.0);
            assert_ne!(changed.1, fingerprints.1);
        }
        assert_ne!(
            CachedTextQuery::fingerprints(&input, Some(2), true).1,
            fingerprints.1
        );
        for change in 0..4 {
            let mut changed = input.clone();
            match change {
                0 => changed.source = "AA",
                1 => changed.stack = "Other",
                2 => changed.font_size += 1,
                _ => changed.weight = TextWeight::Bold,
            }
            assert_ne!(
                CachedTextQuery::fingerprints(&changed, None, true).0,
                fingerprints.0
            );
        }
        assert_ne!(
            CachedTextQuery::fingerprints(&input, None, false).0,
            fingerprints.0
        );
    }

    #[test]
    fn text_and_intrinsic_cache_hits_admit_calls_before_lookup() {
        for intrinsic in [false, true] {
            let mut root = Theme::Publication.resolve();
            root.font_environment
                .set(Ok(PreparedFontEnvironment::synthetic()))
                .unwrap();
            root.typography.tick_label.stack = "Test Sans".into();
            let query = || {
                if intrinsic {
                    root.intrinsic_tick_label_width(["A"].into_iter(), 100.0)
                        .map(|_| ())
                } else {
                    root.measure_text(
                        "A",
                        &root.typography.tick_label,
                        PhysicalTextAlignment::Start,
                        "call boundary",
                        false,
                    )
                    .map(|_| ())
                }
            };
            query().unwrap();
            let resources = root
                .prepared_font_environment()
                .unwrap()
                .generation_resources();
            assert_eq!(resources.call_cache_usage().0, 1 + u128::from(intrinsic));
            let calls = 1 + u128::from(intrinsic);
            resources
                .claim_calls(1_000_000 - calls - resources.call_cache_usage().0)
                .unwrap();
            let entries = resources.call_cache_usage().1;
            let records = resources.usage().unwrap().0;
            crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
            query().unwrap();
            assert_eq!(resources.call_cache_usage(), (1_000_000, entries));
            assert_eq!(resources.usage().unwrap().0, records);
            let usage = resources.usage().unwrap();
            let bytes = resources.byte_usage().unwrap();
            let error = query().unwrap_err();
            assert!(error
                .to_string()
                .contains("measurement/intrinsic call attempts"));
            assert_eq!(resources.usage().unwrap(), usage);
            assert_eq!(resources.byte_usage().unwrap(), bytes);
            crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        }
    }

    #[test]
    fn text_cache_insertion_limit_is_independent_and_refusal_remains_fatal() {
        let root = Theme::Publication.resolve();
        root.font_environment
            .set(Ok(PreparedFontEnvironment::synthetic()))
            .unwrap();
        let mut appearance = root.typography.caption.clone();
        appearance.stack = "Test Sans".into();
        let first = root
            .measure_text(
                "A",
                &appearance,
                PhysicalTextAlignment::Start,
                "first",
                false,
            )
            .unwrap();
        let resources = root
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        resources
            .claim_cache_entries(999_999 - resources.call_cache_usage().1)
            .unwrap();
        let input = indexed_query_input("A", &appearance, "last admitted");
        root.cache_text_query(
            &input,
            &appearance,
            None,
            true,
            CachedTextResult::Plan(first.clone()),
        )
        .unwrap();
        assert_eq!(resources.call_cache_usage().1, 1_000_000);
        let usage = resources.usage().unwrap().0;
        let bytes = resources.byte_usage().unwrap();
        let input = indexed_query_input("A", &appearance, "refused");
        let error = root
            .cache_text_query(
                &input,
                &appearance,
                None,
                true,
                CachedTextResult::Plan(first.clone()),
            )
            .unwrap_err();
        assert!(error.to_string().contains("root-local cache entries"));
        assert_eq!(resources.usage().unwrap().0, usage);
        assert_eq!(resources.byte_usage().unwrap(), bytes);
        assert_eq!(root.text_queries.lock().unwrap().entries.len(), 2);
        let error = root
            .plan_text_quanta_with_policy(
                "A",
                &appearance,
                i64::MAX,
                PhysicalTextAlignment::Start,
                "refused",
                false,
                true,
                &crate::layout::fit::ROOT_SCOPE,
            )
            .unwrap_err();
        assert!(matches!(error, crate::layout::fit::PlanError::Fatal(_)));
        let hit = root
            .measure_text(
                "A",
                &appearance,
                PhysicalTextAlignment::Start,
                "first",
                false,
            )
            .unwrap();
        assert!(first.shares_payload(&hit));
        assert_eq!(resources.call_cache_usage().1, 1_000_000);
    }

    #[test]
    fn text_query_index_storage_is_admitted_before_allocation() {
        let environment = PreparedFontEnvironment::synthetic();
        let used = environment.resource_usage().unwrap().font_bytes;
        environment
            .claim_unicode_bytes(u64::try_from(536_870_912 - used).unwrap())
            .unwrap();
        let mut index = Vec::<TextQueryIndex>::new();
        assert!(TextQueryCache::reserve(&mut index, &environment).is_err());
        assert_eq!(index.capacity(), 0);
    }
}
