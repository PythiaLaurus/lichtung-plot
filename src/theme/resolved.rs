//! Publication-grade visual themes.
//!
//! Every theme is hardcoded so visual regressions remain reviewable and
//! re-renders are byte-stable under the same font environment.
//! Each built-in theme resolves privately to colors, fonts, and layout style.

#![deny(missing_docs)]

use std::sync::{Arc, Mutex, OnceLock};

use sha2::{Digest, Sha256};

use super::{
    AnnotationTextRoles, AxisSpacing, AxisTextRoles, AxisTheme, CompositionTextRoles,
    ContrastTextRoles, DataFillTheme, DataLineTheme, DataOutlineTheme, FixedFillTheme, FrameSides,
    FrameTheme, GuideTextRoles, LegendTheme, OutlineTheme, PointOutlineTheme, PointTheme,
    RootTextRoles, SolidStrokeTheme, StrokeTheme, TickTheme, TypographyTheme,
};
use crate::font::canonical::{OperationGenerationResources, PreparedFontEnvironment};
use crate::layout::{
    PhysicalTextAlignment, ResolvedTextPlan, TextLineHeight, TextPlanInput, TextPlanProvenance,
};
use crate::render::backend::{StrokePattern, TextAnchor};
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
            pub(crate) fn resolve(self, resources: &OperationGenerationResources) -> Result<ResolvedTheme, Error> {
                #[cfg(test)]
                THEME_RESOLUTION_CALLS.with(|calls| calls.set(calls.get() + 1));
                match self { $(Self::$variant => $resolver(resources)),+ }
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
    /// Independent categorical palette and continuous scale defaults.
    pub(crate) color_scales: ColorScales,
    /// Legacy paint source retained for later legend, mark, and annotation roles.
    pub(crate) foreground: (u8, u8, u8),
    /// Font size in actual CSS pixels for annotations, status text, and color scales.
    pub(crate) detail_font_size: u32,
    /// Legacy label-size source retained for later body and annotation roles.
    pub(crate) label_font_size: u32,
    /// Legacy Canvas-composition title size retained until T5b.
    pub(crate) composition_title_font_size: u32,
    /// Legacy facet-header size source retained until T5b composition migration.
    pub(crate) facet_header_font_size: u32,
    /// Canvas-root and Plot-panel paints.
    pub(crate) surfaces: super::SurfaceTheme,
    /// An explicit Plot inside Canvas composes over the Canvas owner's surface.
    pub(crate) canvas_surface_override: Option<Color>,
    /// Complete nested semantic text roles.
    pub(crate) typography: TypographyTheme,
    /// Legend typography-adjacent chrome with owner-local lengths.
    pub(crate) legend: LegendTheme,
    /// Nonsemantic axis chrome and its local spacing.
    pub(crate) axes: AxisTheme,
    /// Complete stroke roles for currently migrated marks.
    pub(crate) marks: MarkAppearance,
    /// Resolved non-text annotation appearance.
    pub(crate) annotations: AnnotationAppearance,
    /// Root and panel composition relationships.
    pub(crate) composition: super::CompositionTheme,
    font_environment: Arc<OnceLock<Result<PreparedFontEnvironment, String>>>,
    text_queries: Arc<Mutex<TextQueryCache>>,
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
    fn profile_entry(
        &self,
        input: &TextPlanInput<'_>,
        prepared_only: bool,
        environment: &PreparedFontEnvironment,
    ) -> Result<Option<usize>, Error> {
        let (mut low, mut high) = (0, self.profiles.len());
        while low < high {
            let middle = low + (high - low) / 2;
            let index = self.profiles[middle];
            match self.compare_profile(index, input, prepared_only, environment)? {
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
                std::cmp::Ordering::Equal => {
                    if self.entries[index]
                        .appearance
                        .stack
                        .same(input.stack, &environment.generation_resources())
                        .map_err(|error| Error::Render(error.to_string()))?
                    {
                        return Ok(Some(index));
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
                            if self.compare_profile(index, input, prepared_only, environment)?
                                != std::cmp::Ordering::Equal
                            {
                                break;
                            }
                            if self.entries[index]
                                .appearance
                                .stack
                                .same(input.stack, &environment.generation_resources())
                                .map_err(|error| Error::Render(error.to_string()))?
                            {
                                return Ok(Some(index));
                            }
                        }
                    }
                    return Ok(None);
                }
            }
        }
        Ok(None)
    }

    fn compare_profile(
        &self,
        index: usize,
        input: &TextPlanInput<'_>,
        prepared_only: bool,
        environment: &PreparedFontEnvironment,
    ) -> Result<std::cmp::Ordering, Error> {
        environment
            .claim_processing_visits(1)
            .map_err(|error| Error::Render(format!("text profile lookup: {error:?}")))?;
        let entry = &self.entries[index];
        let metadata = (
            entry.source.len(),
            entry.appearance.font_size,
            entry.appearance.weight as u8,
            entry.prepared_only,
        )
            .cmp(&(
                input.source.len(),
                input.font_size,
                input.weight as u8,
                prepared_only,
            ));
        if metadata != std::cmp::Ordering::Equal {
            return Ok(metadata);
        }
        if std::ptr::eq(entry.source.as_str(), input.source) {
            return Ok(std::cmp::Ordering::Equal);
        }
        for index in 0..input.source.len() {
            environment
                .claim_processing_visits(1)
                .map_err(|error| Error::Render(format!("text profile content: {error:?}")))?;
            let ordering = entry.source.as_bytes()[index].cmp(&input.source.as_bytes()[index]);
            if ordering != std::cmp::Ordering::Equal {
                return Ok(ordering);
            }
        }
        Ok(std::cmp::Ordering::Equal)
    }

    fn profile_insertion_point(
        &self,
        input: &TextPlanInput<'_>,
        prepared_only: bool,
        environment: &PreparedFontEnvironment,
    ) -> Result<usize, Error> {
        let (mut low, mut high) = (0, self.profiles.len());
        while low < high {
            let middle = low + (high - low) / 2;
            if self.compare_profile(self.profiles[middle], input, prepared_only, environment)?
                == std::cmp::Ordering::Less
            {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        Ok(low)
    }

    fn query_key(
        &self,
        input: &TextPlanInput<'_>,
        preferred_lines: Option<usize>,
        prepared_only: bool,
        environment: &PreparedFontEnvironment,
    ) -> Result<TextQueryKey, Error> {
        let shaping_entry = self.profile_entry(input, prepared_only, environment)?;
        let resources = environment.generation_resources();
        let shaping_state = if let Some(index) = shaping_entry {
            self.entries[index].shaping_state.clone()
        } else {
            CachedTextQuery::shaping_state(input, prepared_only, &resources)?
        };
        Ok(TextQueryKey {
            shaping_entry,
            shaping_fingerprint: shaping_state.clone().finalize().into(),
            fingerprint: CachedTextQuery::presentation_fingerprint(
                shaping_state.clone(),
                input,
                preferred_lines,
                &resources,
            )?,
            shaping_state,
            observed_entries: self.entries.len(),
        })
    }
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
        shaping_entry: Option<usize>,
    ) -> Result<Option<usize>, Error> {
        let (input, preferred_lines, _prepared_only) = key;
        let start = Self::lower_bound(index, fingerprint, environment)?;
        for candidate in &index[start..] {
            environment
                .claim_processing_visits(1)
                .map_err(|error| Error::Render(format!("text query lookup: {error:?}")))?;
            if candidate.fingerprint != fingerprint {
                break;
            }
            let query = &self.entries[candidate.entry];
            if shaping_entry == Some(query.shaping_entry)
                && (!exact
                    || (query.width == input.maximum_width_quanta
                        && query.matches_presentation(
                            input,
                            preferred_lines,
                            &environment.generation_resources(),
                        )?))
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
    shaping_entry: usize,
    shaping_state: Sha256,
    source: Arc<String>,
    appearance: TextTheme,
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
        prepared_only: bool,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<([u8; 32], [u8; 32]), Error> {
        let state = Self::shaping_state(input, prepared_only, resources)?;
        Ok((
            state.clone().finalize().into(),
            Self::presentation_fingerprint(state, input, preferred_lines, resources)?,
        ))
    }

    fn shaping_state(
        input: &TextPlanInput<'_>,
        prepared_only: bool,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<Sha256, Error> {
        resources
            .claim_visits(8 + input.source.len() as u128)
            .map_err(|error| Error::Render(format!("text query hashing: {error:?}")))?;
        let mut hash = Sha256::new();
        // The complete, unnormalized source fixes paragraph bidi and scripts;
        // language and Unicode policy are fixed for this renderer. The memo is
        // shared only together with its immutable operation font environment.
        hash.update((input.source.len() as u64).to_le_bytes());
        hash.update(input.source.as_bytes());
        input
            .stack
            .hash_into(&mut hash, resources)
            .map_err(|error| Error::Render(format!("text query hashing: {error}")))?;
        resources
            .claim_visits(6)
            .map_err(|error| Error::Render(format!("text query hashing: {error:?}")))?;
        hash.update(input.font_size.to_le_bytes());
        hash.update([
            match input.weight {
                TextWeight::Normal => 0,
                TextWeight::Semibold => 1,
                TextWeight::Bold => 2,
            },
            u8::from(prepared_only),
        ]);
        Ok(hash)
    }

    fn presentation_fingerprint(
        mut hash: Sha256,
        input: &TextPlanInput<'_>,
        preferred_lines: Option<usize>,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<[u8; 32], Error> {
        resources
            .claim_visits(
                36 + input.provenance.owner.len() as u128
                    + if preferred_lines.is_some() { 8 } else { 0 },
            )
            .map_err(|error| Error::Render(format!("text query hashing: {error:?}")))?;
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
        Ok(hash.finalize().into())
    }

    fn matches_presentation(
        &self,
        input: &TextPlanInput<'_>,
        preferred_lines: Option<usize>,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<bool, Error> {
        Ok(self.appearance.paint == input.paint
            && input.line_height
                == TextLineHeight::Multiplier(self.appearance.line_height_multiplier)
            && self.alignment == input.alignment
            && self.pair_ordinal == input.provenance.pair_ordinal
            && self.exact_override == input.exact_override
            && self.preferred_lines == preferred_lines
            && crate::font::canonical::same_bytes(&self.owner, &input.provenance.owner, resources)
                .map_err(|error| Error::Render(error.to_string()))?)
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
            && self.marks.primary_point == other.marks.primary_point
            && self.marks.raw_overlay_point == other.marks.raw_overlay_point
            && self.typography == other.typography
    }
}

impl ResolvedTheme {
    fn non_text_eq(&self, other: &Self) -> bool {
        self.color_scales == other.color_scales
            && self.foreground == other.foreground
            && self.detail_font_size == other.detail_font_size
            && self.label_font_size == other.label_font_size
            && self.composition_title_font_size == other.composition_title_font_size
            && self.facet_header_font_size == other.facet_header_font_size
            && self.surfaces == other.surfaces
            && self.canvas_surface_override == other.canvas_surface_override
            && self.marks.non_point_eq(&other.marks)
            && self.annotations == other.annotations
    }
}

impl ResolvedTheme {
    pub(crate) fn legend_outside_gap(&self) -> Result<f64, Error> {
        self.legend.outside_gap_pixels(
            self.typography.guides.legend_label.font_size(),
            &self.prepared_font_environment()?.generation_resources(),
        )
    }
    pub(crate) fn legend_shared_gap(&self) -> Result<f64, Error> {
        self.legend.shared_gap_pixels(
            self.typography.guides.legend_label.font_size(),
            &self.prepared_font_environment()?.generation_resources(),
        )
    }
    pub(crate) fn same(
        &self,
        other: &Self,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<bool, Error> {
        self.color_scales
            .admit_comparison(&other.color_scales, resources)?;
        self.marks
            .accents
            .admit_comparison(&other.marks.accents, resources)?;
        if !self.axes.same(&other.axes, resources)? {
            return Ok(false);
        }
        if !self.legend.same(&other.legend, resources)? {
            return Ok(false);
        }
        if !self.composition.same(&other.composition, resources)? {
            return Ok(false);
        }
        if !self
            .marks
            .primary_point
            .same(&other.marks.primary_point, resources)?
            || !self
                .marks
                .raw_overlay_point
                .same(&other.marks.raw_overlay_point, resources)?
        {
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
            if !left.same(right, resources)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(crate) fn admitted_clone(&self, data: crate::data::PlotData<'_>) -> Result<Self, Error> {
        data.require_source_resources()?;
        let point_ratios = usize::from(self.marks.primary_point.outline.is_some())
            + usize::from(self.marks.raw_overlay_point.outline.is_some());
        data.claim_source_visits(
            self.typography.roles().len()
                + 4
                + 6
                + point_ratios
                + self.legend.relationship_count()
                + 18,
        )?;
        // All fonts, four scale/accent payloads, six axis relationships and each
        // structurally present point-outline relationship are shared once.
        Ok(self.clone())
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
        environment
            .claim_measurement_call()
            .map_err(|error| Error::Render(format!("axis intrinsic call admission: {error:?}")))?;
        let appearance = self.typography.axes().tick_label();
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
            let (cached, query_key) =
                self.cached_text_query(&input, Some(preferred_lines), true, &call)?;
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
                        TextQueryKey {
                            fingerprint: CachedTextQuery::presentation_fingerprint(
                                query_key.shaping_state.clone(),
                                &input,
                                None,
                                &environment.generation_resources(),
                            )?,
                            shaping_state: query_key.shaping_state.clone(),
                            ..query_key
                        },
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
            self.surfaces.plot_panel(),
            self.canvas_surface_override
                .unwrap_or(self.surfaces.canvas_root()),
            self.annotations.planning_backdrop,
            self.typography.annotations().contrast().light(),
            self.typography.annotations().contrast().dark(),
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
        appearance: &TextTheme,
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
        appearance: &TextTheme,
        maximum_width_quanta: i64,
        alignment: PhysicalTextAlignment,
        owner: &str,
        exact_override: bool,
        prepared_only: bool,
        scope: &crate::layout::fit::FitScope,
    ) -> Result<ResolvedTextPlan, crate::layout::fit::PlanError> {
        let input = TextPlanInput {
            source,
            stack: appearance.stack(),
            weight: appearance.weight(),
            font_size: appearance.font_size(),
            line_height: TextLineHeight::Multiplier(appearance.line_height()),
            maximum_width_quanta,
            alignment,
            paint: appearance.color(),
            provenance: TextPlanProvenance {
                pair_ordinal: pair_ordinal(appearance.weight()),
                owner: owner.to_owned(),
            },
            exact_override,
        };
        let environment = self.prepared_font_environment()?;
        let call = environment
            .claim_measurement_call()
            .map_err(|error| Error::Render(format!("text query call admission: {error:?}")))?;
        #[cfg(test)]
        let before = environment
            .resource_usage()
            .map_err(|error| Error::Render(error.to_string()))?
            .processing_visits;
        let (cached, query_key) = self.cached_text_query(&input, None, prepared_only, &call)?;
        #[cfg(test)]
        if input.source.len() > 1000 && std::env::var_os("LICHTUNG_FONT_WORK_AUDIT").is_some() {
            eprintln!("font_query owner={owner:?} bytes={} width={maximum_width_quanta} before={before} lookup={} route={}", input.source.len(), environment.resource_usage().map_err(|error| Error::Render(error.to_string()))?.processing_visits - before, match &cached { Some(CachedTextResult::Plan(_)) => "hit", Some(CachedTextResult::Reflow(_)) => "reflow", _ => "cold" });
        }
        let result = match cached {
            Some(CachedTextResult::Plan(plan)) => return Ok(plan),
            Some(CachedTextResult::Reflow(plan)) => {
                let (source, stack) = plan
                    .analysis_profile()
                    .map_err(|error| Error::Render(error.to_string()))?;
                plan.reflow_in_call(
                    TextPlanInput {
                        source,
                        stack,
                        ..input.clone()
                    },
                    &call,
                )
            }
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
            query_key,
        )?;
        #[cfg(test)]
        if input.source.len() > 1000 && std::env::var_os("LICHTUNG_FONT_WORK_AUDIT").is_some() {
            eprintln!(
                "font_query_done owner={owner:?} bytes={} width={maximum_width_quanta} visits={}",
                input.source.len(),
                environment
                    .resource_usage()
                    .map_err(|error| Error::Render(error.to_string()))?
                    .processing_visits
                    - before
            );
        }
        Ok(plan)
    }

    fn cached_text_query(
        &self,
        input: &TextPlanInput<'_>,
        preferred_lines: Option<usize>,
        prepared_only: bool,
        _call: &crate::font::canonical::MeasurementCall,
    ) -> Result<(Option<CachedTextResult>, TextQueryKey), Error> {
        let environment = self.prepared_font_environment()?;
        let cache = self
            .text_queries
            .lock()
            .map_err(|_| Error::Render("text query cache lock poisoned".into()))?;
        let query_key = cache.query_key(input, preferred_lines, prepared_only, environment)?;
        let shaping_fingerprint = query_key.shaping_fingerprint;
        let fingerprint = query_key.fingerprint;
        let key = (input, preferred_lines, prepared_only);
        if let Some(index) = cache.find(
            &cache.exact,
            fingerprint,
            key,
            true,
            environment,
            query_key.shaping_entry,
        )? {
            return Ok((Some(cache.entries[index].result.clone()), query_key));
        }
        if let Some(index) = cache.find(
            &cache.shaping,
            shaping_fingerprint,
            key,
            false,
            environment,
            query_key.shaping_entry,
        )? {
            let query = &cache.entries[index];
            environment
                .claim_processing_visits(1)
                .map_err(|error| Error::Render(format!("text query lookup: {error:?}")))?;
            if query.matches_width(input.maximum_width_quanta)
                && query.matches_presentation(
                    input,
                    preferred_lines,
                    &environment.generation_resources(),
                )?
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
        reason = "key proof, ordered admission, index insertion and payload freezing stay in one cache transaction"
    )]
    fn cache_text_query(
        &self,
        input: &TextPlanInput<'_>,
        appearance: &TextTheme,
        preferred_lines: Option<usize>,
        prepared_only: bool,
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
        let environment = self.prepared_font_environment()?;
        let resources = environment.generation_resources();
        if query_key.shaping_entry.is_none() && cache.entries.len() != query_key.observed_entries {
            query_key.shaping_entry = cache.profile_entry(input, prepared_only, environment)?;
        }
        let source_bytes = if query_key.shaping_entry.is_some() {
            0
        } else {
            input.source.len()
        };
        let key_bytes = source_bytes
            .checked_add(input.provenance.owner.len())
            .ok_or_else(|| Error::Render("text query cache byte count overflow".into()))?;
        let shaping_fingerprint = query_key.shaping_fingerprint;
        let fingerprint = query_key.fingerprint;
        let key = (input, preferred_lines, prepared_only);
        if cache
            .find(
                &cache.exact,
                fingerprint,
                key,
                true,
                environment,
                query_key.shaping_entry,
            )?
            .is_some()
        {
            return Ok(());
        }
        let new_shaping = matches!(&result, CachedTextResult::Plan(_))
            && cache
                .find(
                    &cache.shaping,
                    shaping_fingerprint,
                    key,
                    false,
                    environment,
                    query_key.shaping_entry,
                )?
                .is_none();
        resources
            .claim_visits((source_bytes + input.provenance.owner.len()) as u128)
            .map_err(|error| Error::Render(format!("text query key copy: {error:?}")))?;
        environment
            .claim_cache_entries(1)
            .map_err(|error| Error::Render(format!("text query cache admission: {error:?}")))?;
        environment
            .claim_text_generation_slots(2 + 2 * u128::from(new_shaping))
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
            TextQueryCache::reserve(&mut cache.profiles, environment)?;
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
        let profile_position = if new_shaping {
            Some(cache.profile_insertion_point(input, prepared_only, environment)?)
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
            appearance
                .stack()
                .share(&resources)
                .map_err(|error| Error::Render(error.to_string()))?,
            appearance.color(),
            appearance.font_size(),
            appearance.weight(),
            appearance.line_height(),
        )?;
        let source = if let Some(index) = query_key.shaping_entry {
            resources
                .claim_visits(1)
                .map_err(|error| Error::Render(format!("text source sharing: {error:?}")))?;
            Arc::clone(&cache.entries[index].source)
        } else {
            let layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
                .extend(std::alloc::Layout::new::<String>())
                .map_err(|_| Error::Render("text source Arc layout overflow".into()))?
                .0
                .pad_to_align();
            environment
                .claim_unicode_bytes(layout.size() as u64)
                .map_err(|error| Error::Render(error.to_string()))?;
            Arc::new(copy(input.source)?)
        };
        cache.entries.push(CachedTextQuery {
            shaping_entry: query_key.shaping_entry.unwrap_or(exact_record.entry),
            shaping_state: query_key.shaping_state,
            unwrapped_width: match &result {
                CachedTextResult::Plan(plan) => plan
                    .unwrapped_fit_width(environment)
                    .map_err(|error| Error::Render(format!("cached paragraph fit: {error}")))?,
                CachedTextResult::PreferredWidth(_) | CachedTextResult::Reflow(_) => None,
            },
            source,
            appearance,
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
pub(crate) struct SurfaceTheme {
    /// Canvas root paint.
    pub(crate) canvas_root: Color,
    /// Plot panel paint.
    pub(crate) plot_panel: Color,
}

impl SurfaceTheme {
    /// Construct complete surface paints; transparent paints are valid.
    #[expect(
        clippy::unnecessary_wraps,
        reason = "complete component construction uniformly returns Result; Color already validates every surface operand"
    )]
    pub(crate) fn new(canvas_root: Color, plot_panel: Color) -> Result<Self, Error> {
        Ok(Self {
            canvas_root,
            plot_panel,
        })
    }

    pub(crate) fn canvas_root(&self) -> Color {
        self.canvas_root
    }

    pub(crate) fn plot_panel(&self) -> Color {
        self.plot_panel
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "construction-only surface edits await complete theme customization"
        )
    )]
    pub(crate) fn with_canvas_root(self, canvas_root: Color) -> Result<Self, Error> {
        Self::new(canvas_root, self.plot_panel)
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "construction-only surface edits await complete theme customization"
        )
    )]
    pub(crate) fn with_plot_panel(self, plot_panel: Color) -> Result<Self, Error> {
        Self::new(self.canvas_root, plot_panel)
    }
}

/// Complete text appearance for one semantic typography role.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextTheme {
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

impl TextTheme {
    /// Construct one complete text role, using actual CSS pixels and a multiplier.
    pub(crate) fn new(
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

    pub(crate) fn stack(&self) -> &crate::font::canonical::FontStack {
        &self.stack
    }

    pub(crate) fn color(&self) -> Color {
        self.paint
    }

    pub(crate) fn font_size(&self) -> u32 {
        self.font_size
    }

    pub(crate) fn weight(&self) -> TextWeight {
        self.weight
    }

    /// Inspect the retained multiplier, without multiplying by the font size.
    pub(crate) fn line_height(&self) -> f64 {
        self.line_height_multiplier
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "construction-only stack replacement awaits complete theme customization; operation copies retain their admitted stack path"
        )
    )]
    pub(crate) fn with_stack(
        self,
        stack: crate::font::canonical::FontStack,
    ) -> Result<Self, Error> {
        Self::new(
            stack,
            self.paint,
            self.font_size,
            self.weight,
            self.line_height_multiplier,
        )
    }

    pub(crate) fn with_color(self, color: Color) -> Result<Self, Error> {
        Self::new(
            self.stack,
            color,
            self.font_size,
            self.weight,
            self.line_height_multiplier,
        )
    }

    pub(crate) fn with_font_size(self, font_size: u32) -> Result<Self, Error> {
        Self::new(
            self.stack,
            self.paint,
            font_size,
            self.weight,
            self.line_height_multiplier,
        )
    }

    pub(crate) fn with_weight(self, weight: TextWeight) -> Result<Self, Error> {
        Self::new(
            self.stack,
            self.paint,
            self.font_size,
            weight,
            self.line_height_multiplier,
        )
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "construction-only leading edits await complete theme customization"
        )
    )]
    pub(crate) fn with_line_height(self, line_height: f64) -> Result<Self, Error> {
        Self::new(
            self.stack,
            self.paint,
            self.font_size,
            self.weight,
            line_height,
        )
    }

    pub(crate) fn same(
        &self,
        other: &Self,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<bool, Error> {
        Ok(self.font_size == other.font_size
            && self.weight == other.weight
            && self.paint == other.paint
            && self.line_height_multiplier.to_bits() == other.line_height_multiplier.to_bits()
            && self
                .stack
                .same(&other.stack, resources)
                .map_err(|error| Error::Render(error.to_string()))?)
    }

    pub(crate) fn copy_with_generation(
        &self,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<Self, Error> {
        Self::new(
            self.stack
                .copy_with_generation(resources)
                .map_err(|error| Error::Render(error.to_string()))?,
            self.color(),
            self.font_size(),
            self.weight(),
            self.line_height(),
        )
    }
    /// Resolve this role's current line pitch in pixels.
    pub(crate) fn line_pitch_pixels(&self) -> f64 {
        f64::from(self.font_size) * self.line_height_multiplier
    }
}

/// Mark strokes are independent of structurally similar axis chrome.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MarkAppearance {
    /// Stable non-categorical mark slots, independent of categorical palette edits.
    pub(crate) accents: AccentPalette,
    /// Shared primary-point paint/outline before family-specific size selection.
    pub(crate) primary_point: PointTheme,
    /// Ordinary bar bodies in either orientation and originating guide glyphs.
    pub(crate) bar_body: DataFillTheme,
    pub(crate) line_band: DataFillTheme,
    pub(crate) regression_band: DataFillTheme,
    pub(crate) density_area: DataFillTheme,
    pub(crate) marginal_density: DataFillTheme,
    pub(crate) matrix_density: DataFillTheme,
    pub(crate) box_body: DataFillTheme,
    pub(crate) violin_body: DataFillTheme,
    pub(crate) raincloud_box: FixedFillTheme,
    pub(crate) histogram_bin: DataFillTheme,
    pub(crate) waterfall_bar: DataFillTheme,
    /// Boxplot/Violin raw observations and Raincloud strips/guide points.
    pub(crate) raw_overlay_point: PointTheme,
    pub(crate) raw_overlay_radius: u32,
    pub(crate) heatmap_cell_outline: OutlineTheme,
    pub(crate) heatmap_highlight_outline: OutlineTheme,
    /// Non-data missing paint shared by Heatmap cells, merged rows and keys.
    pub(crate) heatmap_missing_fill: Color,
    pub(crate) heatmap_missing_row_outline: OutlineTheme,
    pub(crate) heatmap_missing_key_outline: OutlineTheme,
    pub(crate) heatmap_missing_cross: StrokeTheme,
    pub(crate) bar_zero: StrokeTheme,
    pub(crate) waterfall_connector: StrokeTheme,
    pub(crate) waterfall_reference: StrokeTheme,
    pub(crate) raincloud_median: StrokeTheme,
    pub(crate) beeswarm_reference: StrokeTheme,
    pub(crate) interaction_interval: DataLineTheme,
    pub(crate) interaction_connector: DataLineTheme,
    /// Sizes of built-in point roles; plotted groups and their guides share these.
    pub(crate) point_sizes: PointSizeAppearance,
    /// Bar uncertainty stems/caps in either orientation.
    pub(crate) uncertainty_stroke: StrokeTheme,
    /// Distribution summary and whisker strokes, including their guide glyphs.
    pub(crate) summary_stroke: StrokeTheme,
    /// A Line series' data-colored path, reused in its legend glyph.
    pub(crate) series_line: DataLineTheme,
    /// A Trajectory or LinePath series' data-colored path.
    pub(crate) path_line: DataLineTheme,
    /// Histogram KDE overlay independently resolved from the bin outline.
    pub(crate) histogram_curve: StrokeTheme,
    /// Histogram rug ticks independently resolved from the KDE overlay.
    pub(crate) histogram_rug: StrokeTheme,
    /// Density group curve, reused exactly by its originating guide glyph.
    pub(crate) density_line: DataLineTheme,
    /// Regression fit line independently resolved from its observations/band.
    pub(crate) regression_fit: StrokeTheme,
    /// Estimate intervals do not borrow axis or bar uncertainty strokes.
    pub(crate) estimate_interval: StrokeTheme,
    /// PointInterval's solid reference at an exact caller value.
    pub(crate) point_interval_reference: StrokeTheme,
    /// Coefficient/Forest reference has its own fixed medium-dash grammar.
    pub(crate) estimate_reference: StrokeTheme,
    /// Q-Q reference is independent of the estimate-display reference role.
    pub(crate) qq_reference: StrokeTheme,
    /// Dumbbell pairs use one neutral connecting stroke.
    pub(crate) dumbbell_connector: StrokeTheme,
}

impl MarkAppearance {
    // The two point relationships have their own admitted comparison. Exhaustive
    // destructuring prevents any retained field escaping theme identity.
    fn non_point_eq(&self, other: &Self) -> bool {
        macro_rules! fields {
            ($($field:ident),+ $(,)?) => {{
                let Self { primary_point: _, raw_overlay_point: _, $($field),+ } = self;
                true $(&& *$field == other.$field)+
            }};
        }
        fields!(
            accents,
            bar_body,
            line_band,
            regression_band,
            density_area,
            marginal_density,
            matrix_density,
            box_body,
            violin_body,
            raincloud_box,
            histogram_bin,
            waterfall_bar,
            raw_overlay_radius,
            heatmap_cell_outline,
            heatmap_highlight_outline,
            heatmap_missing_fill,
            heatmap_missing_row_outline,
            heatmap_missing_key_outline,
            heatmap_missing_cross,
            bar_zero,
            waterfall_connector,
            waterfall_reference,
            raincloud_median,
            beeswarm_reference,
            interaction_interval,
            interaction_connector,
            point_sizes,
            uncertainty_stroke,
            summary_stroke,
            series_line,
            path_line,
            histogram_curve,
            histogram_rug,
            density_line,
            regression_fit,
            estimate_interval,
            point_interval_reference,
            estimate_reference,
            qq_reference,
            dumbbell_connector
        )
    }
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

/// Annotation paths resolve independently of axes, marks, and label typography.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AnnotationAppearance {
    pub(crate) bracket_stroke: SolidStrokeTheme,
    /// Clearance from a labeled supplemental point's painted radius.
    pub(crate) point_label_gap: f64,
    pub(crate) waterfall_shelf_padding: f64,
    pub(crate) waterfall_shelf_gap: f64,
    pub(crate) waterfall_shelf_row_gap: f64,
    /// Opaque calculation-only backdrop when output surfaces are translucent.
    pub(crate) planning_backdrop: Color,
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

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "the four closed presets provide one compact table of preserved scalar values"
)]
fn resolved_theme(
    resources: &OperationGenerationResources,
    colors: &[(u8, u8, u8)],
    foreground: (u8, u8, u8),
    axis_width: f32,
    bracket_width: f32,
    title_font_size: u32,
    subtitle_font_size: u32,
    caption_font_size: u32,
    axis_title_font_size: u32,
    grid_color: (u8, u8, u8),
) -> Result<ResolvedTheme, Error> {
    let primary_outline = PointOutlineTheme::publication(resources)?;
    let overlay_outline = PointOutlineTheme::publication(resources)?;
    let stack = crate::font::canonical::builtin_font_stack();
    let foreground_paint = color_from_rgb(foreground);
    let muted_paint = color_from_rgb(MUTED_TEXT);
    let opaque = |&(r, g, b): &(u8, u8, u8)| OpaqueColor::rgb(r, g, b);
    let color_scales = ColorScales::new(
        QualitativePalette::extendable_in(colors.iter().map(opaque), resources)?,
        SequentialGradient::new_in(crate::theme::HEATMAP_SEQUENTIAL_STOPS, resources)?,
        DivergingGradient::new_in(crate::theme::HEATMAP_DIVERGING_STOPS, resources)?,
    )?;
    let accents = AccentPalette::new_in(colors.iter().map(opaque), resources)?;
    let stroke = |width, pattern| StrokeTheme::new(foreground_paint, width, pattern);
    let filled = |alpha, width| {
        DataFillTheme::new(
            alpha,
            Some(DataOutlineTheme::fixed(OutlineTheme::new(
                foreground_paint,
                width,
            )?)),
        )
    };
    let text = |paint, font_size, line_height| {
        TextTheme::new(
            stack.clone(),
            paint,
            font_size,
            TextWeight::Normal,
            line_height,
        )
    };

    Ok(ResolvedTheme {
        color_scales,
        foreground,
        detail_font_size: 14,
        label_font_size: axis_title_font_size,
        composition_title_font_size: title_font_size,
        facet_header_font_size: subtitle_font_size,
        surfaces: SurfaceTheme::new(Color::rgb(255, 255, 255), Color::rgb(255, 255, 255))?,
        canvas_surface_override: None,
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
        legend: LegendTheme::builtin(resources)?,
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
                super::RelativeSpacing::ratio_in(8, 16, resources)?,
                super::RelativeSpacing::ratio_in(6, 16, resources)?,
                super::RelativeSpacing::ratio_in(8, axis_title_font_size, resources)?,
                super::RelativeSpacing::ratio_in(16, axis_title_font_size, resources)?,
                super::RelativeSpacing::ratio_in(16, axis_title_font_size, resources)?,
                super::RelativeSpacing::ratio_in(12, axis_title_font_size, resources)?,
            )?,
        )?,
        marks: MarkAppearance {
            accents,
            primary_point: PointTheme::new(209, crate::PointShape::Circle, Some(primary_outline))?,
            bar_body: filled(255, 1.0)?,
            line_band: DataFillTheme::new(46, None)?,
            regression_band: DataFillTheme::new(64, None)?,
            density_area: DataFillTheme::new(56, None)?,
            marginal_density: DataFillTheme::new(115, Some(DataOutlineTheme::mapped(1.0, 255)?))?,
            matrix_density: DataFillTheme::new(102, Some(DataOutlineTheme::mapped(1.0, 255)?))?,
            box_body: filled(255, 1.0)?,
            violin_body: filled(178, 1.0)?,
            raincloud_box: FixedFillTheme::new(
                Color::rgb(255, 255, 255),
                Some(OutlineTheme::new(foreground_paint, 1.0)?),
            )?,
            histogram_bin: filled(209, 0.8)?,
            waterfall_bar: filled(255, 0.8)?,
            raw_overlay_point: PointTheme::new(
                209,
                crate::PointShape::Circle,
                Some(overlay_outline),
            )?,
            raw_overlay_radius: 2,
            heatmap_cell_outline: OutlineTheme::new(Color::rgb(255, 255, 255), 0.5)?,
            heatmap_highlight_outline: OutlineTheme::new(foreground_paint, 2.5)?,
            heatmap_missing_fill: Color::rgb(224, 224, 224),
            heatmap_missing_row_outline: OutlineTheme::new(Color::rgb(255, 255, 255), 0.5)?,
            heatmap_missing_key_outline: OutlineTheme::new(Color::rgb(120, 120, 120), 0.8)?,
            heatmap_missing_cross: StrokeTheme::new(
                Color::rgb(120, 120, 120),
                1.0,
                StrokePattern::Solid,
            )?,
            bar_zero: stroke(0.8, StrokePattern::Solid)?,
            waterfall_connector: stroke(0.7, StrokePattern::ShortDash)?,
            waterfall_reference: stroke(0.7, StrokePattern::MediumDash)?,
            raincloud_median: stroke(1.5, StrokePattern::Solid)?,
            beeswarm_reference: stroke(0.8, StrokePattern::Solid)?,
            interaction_interval: DataLineTheme::new(1.2, 255, StrokePattern::Solid)?,
            interaction_connector: DataLineTheme::new(1.8, 255, StrokePattern::Solid)?,
            point_sizes: PointSizeAppearance::publication(),
            uncertainty_stroke: stroke(axis_width, StrokePattern::Solid)?,
            summary_stroke: stroke(axis_width, StrokePattern::Solid)?,
            series_line: DataLineTheme::new(1.8, 255, StrokePattern::Solid)?,
            path_line: DataLineTheme::new(1.4, 255, StrokePattern::Solid)?,
            histogram_curve: stroke(1.0, StrokePattern::Solid)?,
            histogram_rug: stroke(0.7, StrokePattern::Solid)?,
            density_line: DataLineTheme::new(1.6, 255, StrokePattern::Solid)?,
            regression_fit: stroke(1.5, StrokePattern::Solid)?,
            estimate_interval: stroke(1.4, StrokePattern::Solid)?,
            point_interval_reference: stroke(0.8, StrokePattern::Solid)?,
            estimate_reference: stroke(1.0, StrokePattern::MediumDash)?,
            qq_reference: stroke(1.2, StrokePattern::Solid)?,
            dumbbell_connector: stroke(1.2, StrokePattern::Solid)?,
        },
        annotations: AnnotationAppearance {
            bracket_stroke: SolidStrokeTheme::new(foreground_paint, bracket_width)?,
            point_label_gap: 2.0,
            waterfall_shelf_padding: 4.0,
            waterfall_shelf_gap: 4.0,
            waterfall_shelf_row_gap: 4.0,
            planning_backdrop: Color::rgb(255, 255, 255),
        },
        composition: super::CompositionTheme::builtin(
            title_font_size,
            subtitle_font_size,
            caption_font_size,
            axis_title_font_size,
            resources,
        )?,
        font_environment: Arc::new(OnceLock::new()),
        text_queries: Arc::default(),
    })
}

fn plus_six_line_height(font_size: u32) -> f64 {
    (f64::from(font_size) + 6.0) / f64::from(font_size)
}

fn publication_theme(resources: &OperationGenerationResources) -> Result<ResolvedTheme, Error> {
    // Okabe-Ito 8-color qualitative palette (colorblind-safe).
    resolved_theme(
        resources,
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

fn nature_theme(resources: &OperationGenerationResources) -> Result<ResolvedTheme, Error> {
    resolved_theme(
        resources,
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

fn science_theme(resources: &OperationGenerationResources) -> Result<ResolvedTheme, Error> {
    resolved_theme(
        resources,
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

fn chi_theme(resources: &OperationGenerationResources) -> Result<ResolvedTheme, Error> {
    resolved_theme(
        resources,
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
    use crate::SignificanceMarkStyle;

    #[test]
    fn text_theme_construction_and_edits_preserve_complete_independent_inputs() {
        let stack = crate::font::canonical::named_stack(&["Literal, Sans", "serif"]);
        let source = TextTheme::new(
            stack.clone(),
            Color::rgba(13, 29, 47, 61),
            23,
            TextWeight::Semibold,
            1.75,
        )
        .unwrap();
        assert_eq!(source.stack(), &stack);
        assert_eq!(source.color(), Color::rgba(13, 29, 47, 61));
        assert_eq!(source.font_size(), 23);
        assert_eq!(source.weight(), TextWeight::Semibold);
        assert_eq!(source.line_height().to_bits(), 1.75_f64.to_bits());
        assert_eq!(source.line_pitch_pixels().to_bits(), 40.25_f64.to_bits());

        let replacement = crate::font::canonical::named_stack(&["Other face"]);
        let cases = [
            (
                source.clone().with_stack(replacement.clone()).unwrap(),
                TextTheme {
                    stack: replacement,
                    ..source.clone()
                },
            ),
            (
                source
                    .clone()
                    .with_color(Color::rgba(71, 83, 97, 0))
                    .unwrap(),
                TextTheme {
                    paint: Color::rgba(71, 83, 97, 0),
                    ..source.clone()
                },
            ),
            (
                source.clone().with_font_size(31).unwrap(),
                TextTheme {
                    font_size: 31,
                    ..source.clone()
                },
            ),
            (
                source.clone().with_weight(TextWeight::Bold).unwrap(),
                TextTheme {
                    weight: TextWeight::Bold,
                    ..source.clone()
                },
            ),
            (
                source.clone().with_line_height(2.125).unwrap(),
                TextTheme {
                    line_height_multiplier: 2.125,
                    ..source.clone()
                },
            ),
        ];
        for (edited, expected) in cases {
            assert_eq!(edited, expected);
            assert_ne!(edited, source);
        }
        let repeated = source
            .clone()
            .with_font_size(31)
            .unwrap()
            .with_font_size(7)
            .unwrap();
        assert_eq!(repeated.font_size(), 7);
        assert_eq!(repeated.line_height().to_bits(), 1.75_f64.to_bits());
        assert_eq!(source.font_size(), 23);
        assert_eq!(source.stack(), &stack);
    }

    #[test]
    fn text_theme_refuses_invalid_size_and_leading_without_shrinking_valid_extremes() {
        let stack = crate::font::canonical::named_stack(&["Caller literal"]);
        let source = TextTheme::new(
            stack.clone(),
            Color::rgba(3, 5, 7, 0),
            19,
            TextWeight::Normal,
            1.3,
        )
        .unwrap();
        assert!(TextTheme::new(stack.clone(), source.color(), 0, source.weight(), 1.3).is_err());
        assert!(source.clone().with_font_size(0).is_err());
        for leading in [0.0, -0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                TextTheme::new(stack.clone(), source.color(), 19, source.weight(), leading)
                    .is_err()
            );
            assert!(source.clone().with_line_height(leading).is_err());
        }
        for leading in [f64::from_bits(1), f64::MAX] {
            let extreme = TextTheme::new(
                stack.clone(),
                source.color(),
                u32::MAX,
                TextWeight::Bold,
                leading,
            )
            .unwrap();
            assert_eq!(extreme.font_size(), u32::MAX);
            assert_eq!(extreme.line_height().to_bits(), leading.to_bits());
        }
        // Internal negative fixtures may still poison a field. Every construction
        // edit validates the complete result, rather than accepting a bad sibling.
        let poisoned = TextTheme {
            line_height_multiplier: f64::NAN,
            ..source.clone()
        };
        assert!(poisoned.clone().with_stack(stack).is_err());
        assert!(poisoned.clone().with_color(Color::rgb(0, 0, 0)).is_err());
        assert!(poisoned.clone().with_font_size(11).is_err());
        assert!(poisoned.with_weight(TextWeight::Bold).is_err());
        assert_eq!(source.font_size(), 19);
        assert_eq!(source.line_height().to_bits(), 1.3_f64.to_bits());
    }

    #[test]
    fn surface_theme_retains_transparency_and_independent_root_and_panel_edits() {
        let source =
            SurfaceTheme::new(Color::rgba(11, 23, 37, 0), Color::rgba(41, 59, 73, 127)).unwrap();
        assert_eq!(source.canvas_root(), Color::rgba(11, 23, 37, 0));
        assert_eq!(source.plot_panel(), Color::rgba(41, 59, 73, 127));
        let root = source
            .clone()
            .with_canvas_root(Color::rgb(83, 97, 109))
            .unwrap();
        assert_eq!(root.canvas_root(), Color::rgb(83, 97, 109));
        assert_eq!(root.plot_panel(), source.plot_panel());
        let panel = source
            .clone()
            .with_plot_panel(Color::rgba(113, 131, 149, 255))
            .unwrap();
        assert_eq!(panel.canvas_root(), source.canvas_root());
        assert_eq!(panel.plot_panel(), Color::rgb(113, 131, 149));
        let repeated = panel
            .with_plot_panel(Color::rgba(151, 163, 179, 0))
            .unwrap();
        assert_eq!(repeated.plot_panel(), Color::rgba(151, 163, 179, 0));
        assert_eq!(source.plot_panel(), Color::rgba(41, 59, 73, 127));
    }

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
        let stack = crate::font::canonical::builtin_font_stack();
        let foreground = color_from_rgb(expected.foreground);
        let text = |paint, font_size, line_height_multiplier| {
            TextTheme::new(
                stack.clone(),
                paint,
                font_size,
                TextWeight::Normal,
                line_height_multiplier,
            )
            .unwrap()
        };

        ResolvedTheme {
            color_scales: ColorScales::new(
                QualitativePalette::extendable(
                    colors.iter().map(|&(r, g, b)| OpaqueColor::rgb(r, g, b)),
                )
                .unwrap(),
                SequentialGradient::new(crate::theme::HEATMAP_SEQUENTIAL_STOPS).unwrap(),
                DivergingGradient::new(crate::theme::HEATMAP_DIVERGING_STOPS).unwrap(),
            )
            .unwrap(),
            foreground: expected.foreground,
            detail_font_size: 14,
            label_font_size: expected.axis_title_size,
            composition_title_font_size: expected.title_size,
            facet_header_font_size: expected.subtitle_size,
            surfaces: SurfaceTheme::new(
                Color::rgba(255, 255, 255, 255),
                Color::rgba(255, 255, 255, 255),
            )
            .unwrap(),
            canvas_surface_override: None,
            typography: TypographyTheme::new(
                RootTextRoles::new(
                    text(
                        foreground,
                        expected.title_size,
                        (f64::from(expected.title_size) + 6.0) / f64::from(expected.title_size),
                    ),
                    text(
                        Color::rgb(80, 80, 80),
                        expected.subtitle_size,
                        (f64::from(expected.subtitle_size) + 6.0)
                            / f64::from(expected.subtitle_size),
                    ),
                    text(
                        Color::rgb(80, 80, 80),
                        expected.caption_size,
                        (f64::from(expected.caption_size) + 6.0) / f64::from(expected.caption_size),
                    ),
                )
                .unwrap(),
                AxisTextRoles::new(
                    text(foreground, expected.axis_title_size, 1.2),
                    text(foreground, 16, 1.2),
                )
                .unwrap(),
                GuideTextRoles::new(
                    text(Color::rgb(60, 60, 60), 12, 1.2),
                    text(Color::rgb(40, 40, 40), 11, 1.2),
                )
                .unwrap(),
                CompositionTextRoles::new(
                    text(foreground, expected.subtitle_size, 1.2),
                    text(foreground, expected.title_size, 1.2),
                    text(foreground, expected.title_size, 1.2),
                    text(foreground, expected.axis_title_size, 1.2),
                )
                .unwrap(),
                AnnotationTextRoles::new(
                    text(foreground, 16, 1.2),
                    text(foreground, 14, 1.2),
                    text(foreground, 14, 1.2),
                    text(foreground, 10, 1.2),
                    text(foreground, 14, 1.2),
                    ContrastTextRoles::new(
                        text(Color::rgb(255, 255, 255), 11, 1.2),
                        text(Color::rgb(0, 0, 0), 11, 1.2),
                    )
                    .unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
            legend: {
                use crate::theme::{
                    GradientLengths, LegendFrameTheme, LegendSpacing, RelativeSpacing,
                };
                LegendTheme::new(
                    LegendSpacing::new(
                        RelativeSpacing::from_pixels(8.8, 11).unwrap(),
                        RelativeSpacing::ratio(1, 1).unwrap(),
                        RelativeSpacing::from_pixels(8.8, 11).unwrap(),
                        RelativeSpacing::from_pixels(8.8, 11).unwrap(),
                        RelativeSpacing::from_pixels(17.6, 11).unwrap(),
                        RelativeSpacing::from_pixels(3.3, 11).unwrap(),
                        RelativeSpacing::from_pixels(13.2, 11).unwrap(),
                        RelativeSpacing::from_pixels(7.8, 12).unwrap(),
                        RelativeSpacing::new(0.2).unwrap(),
                    )
                    .unwrap(),
                    GradientLengths::new(
                        RelativeSpacing::ratio(40, 11).unwrap(),
                        RelativeSpacing::ratio(160, 11).unwrap(),
                    )
                    .unwrap(),
                    Some(
                        LegendFrameTheme::new(
                            Color::rgba(255, 255, 255, 204),
                            Some(OutlineTheme::new(Color::rgb(200, 200, 200), 1.0).unwrap()),
                            RelativeSpacing::ratio(2, 11).unwrap(),
                        )
                        .unwrap(),
                    ),
                )
                .unwrap()
            },
            axes: AxisTheme::new(
                Some(
                    FrameTheme::new(
                        FrameSides::new(true, true, true, true).unwrap(),
                        StrokeTheme {
                            paint: foreground,
                            width: expected.axis_width,
                            pattern: StrokePattern::Solid,
                        },
                    )
                    .unwrap(),
                ),
                Some(
                    FrameTheme::new(
                        FrameSides::new(false, false, true, true).unwrap(),
                        StrokeTheme {
                            paint: foreground,
                            width: expected.axis_width,
                            pattern: StrokePattern::Solid,
                        },
                    )
                    .unwrap(),
                ),
                Some(
                    TickTheme::new(
                        StrokeTheme {
                            paint: foreground,
                            width: expected.axis_width,
                            pattern: StrokePattern::Solid,
                        },
                        5,
                    )
                    .unwrap(),
                ),
                Some(StrokeTheme {
                    paint: color_from_rgb(expected.grid_color),
                    width: 0.5,
                    pattern: StrokePattern::MediumDash,
                }),
                AxisSpacing::new(
                    super::super::RelativeSpacing::ratio(8, 16).unwrap(),
                    super::super::RelativeSpacing::ratio(6, 16).unwrap(),
                    super::super::RelativeSpacing::ratio(8, expected.axis_title_size).unwrap(),
                    super::super::RelativeSpacing::ratio(16, expected.axis_title_size).unwrap(),
                    super::super::RelativeSpacing::ratio(16, expected.axis_title_size).unwrap(),
                    super::super::RelativeSpacing::ratio(12, expected.axis_title_size).unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
            marks: MarkAppearance {
                accents: AccentPalette::new(
                    colors.iter().map(|&(r, g, b)| OpaqueColor::rgb(r, g, b)),
                )
                .unwrap(),
                primary_point: PointTheme {
                    fill_alpha: 209,
                    shape: crate::PointShape::Circle,
                    outline: Some(
                        PointOutlineTheme::publication(&OperationGenerationResources::new())
                            .unwrap(),
                    ),
                },
                bar_body: DataFillTheme::new(
                    255,
                    Some(DataOutlineTheme::fixed(
                        OutlineTheme::new(foreground, 1.0).unwrap(),
                    )),
                )
                .unwrap(),
                line_band: DataFillTheme::new(46, None).unwrap(),
                regression_band: DataFillTheme::new(64, None).unwrap(),
                density_area: DataFillTheme::new(56, None).unwrap(),
                marginal_density: DataFillTheme::new(
                    115,
                    Some(DataOutlineTheme::mapped(1.0, 255).unwrap()),
                )
                .unwrap(),
                matrix_density: DataFillTheme::new(
                    102,
                    Some(DataOutlineTheme::mapped(1.0, 255).unwrap()),
                )
                .unwrap(),
                box_body: DataFillTheme::new(
                    255,
                    Some(DataOutlineTheme::fixed(
                        OutlineTheme::new(foreground, 1.0).unwrap(),
                    )),
                )
                .unwrap(),
                violin_body: DataFillTheme::new(
                    178,
                    Some(DataOutlineTheme::fixed(
                        OutlineTheme::new(foreground, 1.0).unwrap(),
                    )),
                )
                .unwrap(),
                raincloud_box: FixedFillTheme::new(
                    Color::rgb(255, 255, 255),
                    Some(OutlineTheme::new(foreground, 1.0).unwrap()),
                )
                .unwrap(),
                histogram_bin: DataFillTheme::new(
                    209,
                    Some(DataOutlineTheme::fixed(
                        OutlineTheme::new(foreground, 0.8).unwrap(),
                    )),
                )
                .unwrap(),
                waterfall_bar: DataFillTheme::new(
                    255,
                    Some(DataOutlineTheme::fixed(
                        OutlineTheme::new(foreground, 0.8).unwrap(),
                    )),
                )
                .unwrap(),
                raw_overlay_point: PointTheme {
                    fill_alpha: 209,
                    shape: crate::PointShape::Circle,
                    outline: Some(
                        PointOutlineTheme::publication(&OperationGenerationResources::new())
                            .unwrap(),
                    ),
                },
                raw_overlay_radius: 2,
                heatmap_cell_outline: OutlineTheme {
                    paint: Color::rgb(255, 255, 255),
                    width: 0.5,
                },
                heatmap_highlight_outline: OutlineTheme {
                    paint: foreground,
                    width: 2.5,
                },
                heatmap_missing_fill: Color::rgb(224, 224, 224),
                heatmap_missing_row_outline: OutlineTheme {
                    paint: Color::rgb(255, 255, 255),
                    width: 0.5,
                },
                heatmap_missing_key_outline: OutlineTheme {
                    paint: Color::rgb(120, 120, 120),
                    width: 0.8,
                },
                heatmap_missing_cross: StrokeTheme {
                    paint: Color::rgb(120, 120, 120),
                    width: 1.0,
                    pattern: StrokePattern::Solid,
                },
                bar_zero: StrokeTheme {
                    paint: foreground,
                    width: 0.8,
                    pattern: StrokePattern::Solid,
                },
                waterfall_connector: StrokeTheme {
                    paint: foreground,
                    width: 0.7,
                    pattern: StrokePattern::ShortDash,
                },
                waterfall_reference: StrokeTheme {
                    paint: foreground,
                    width: 0.7,
                    pattern: StrokePattern::MediumDash,
                },
                raincloud_median: StrokeTheme {
                    paint: foreground,
                    width: 1.5,
                    pattern: StrokePattern::Solid,
                },
                beeswarm_reference: StrokeTheme {
                    paint: foreground,
                    width: 0.8,
                    pattern: StrokePattern::Solid,
                },
                interaction_interval: DataLineTheme {
                    width: 1.2,
                    alpha: 255,
                    pattern: StrokePattern::Solid,
                },
                interaction_connector: DataLineTheme {
                    width: 1.8,
                    alpha: 255,
                    pattern: StrokePattern::Solid,
                },
                point_sizes: PointSizeAppearance::publication(),
                uncertainty_stroke: StrokeTheme {
                    paint: foreground,
                    width: expected.axis_width,
                    pattern: StrokePattern::Solid,
                },
                summary_stroke: StrokeTheme {
                    paint: foreground,
                    width: expected.axis_width,
                    pattern: StrokePattern::Solid,
                },
                series_line: DataLineTheme {
                    width: 1.8,
                    alpha: 255,
                    pattern: StrokePattern::Solid,
                },
                path_line: DataLineTheme {
                    width: 1.4,
                    alpha: 255,
                    pattern: StrokePattern::Solid,
                },
                histogram_curve: StrokeTheme {
                    paint: foreground,
                    width: 1.0,
                    pattern: StrokePattern::Solid,
                },
                histogram_rug: StrokeTheme {
                    paint: foreground,
                    width: 0.7,
                    pattern: StrokePattern::Solid,
                },
                density_line: DataLineTheme {
                    width: 1.6,
                    alpha: 255,
                    pattern: StrokePattern::Solid,
                },
                regression_fit: StrokeTheme {
                    paint: foreground,
                    width: 1.5,
                    pattern: StrokePattern::Solid,
                },
                estimate_interval: StrokeTheme {
                    paint: foreground,
                    width: 1.4,
                    pattern: StrokePattern::Solid,
                },
                point_interval_reference: StrokeTheme {
                    paint: foreground,
                    width: 0.8,
                    pattern: StrokePattern::Solid,
                },
                estimate_reference: StrokeTheme {
                    paint: foreground,
                    width: 1.0,
                    pattern: StrokePattern::MediumDash,
                },
                qq_reference: StrokeTheme {
                    paint: foreground,
                    width: 1.2,
                    pattern: StrokePattern::Solid,
                },
                dumbbell_connector: StrokeTheme {
                    paint: foreground,
                    width: 1.2,
                    pattern: StrokePattern::Solid,
                },
            },
            annotations: AnnotationAppearance {
                bracket_stroke: SolidStrokeTheme {
                    paint: foreground,
                    width: expected.bracket_width,
                },
                point_label_gap: 2.0,
                waterfall_shelf_padding: 4.0,
                waterfall_shelf_gap: 4.0,
                waterfall_shelf_row_gap: 4.0,
                planning_backdrop: Color::rgb(255, 255, 255),
            },
            composition: crate::theme::CompositionTheme::builtin(
                expected.title_size,
                expected.subtitle_size,
                expected.caption_size,
                expected.axis_title_size,
                &OperationGenerationResources::new(),
            )
            .unwrap(),
            font_environment: Arc::new(OnceLock::new()),
            text_queries: Arc::default(),
        }
    }

    #[test]
    fn built_in_themes_preserve_every_resolved_value() {
        for theme in BUILT_IN_THEMES {
            assert_eq!(
                theme.resolve(&OperationGenerationResources::new()).unwrap(),
                expected_theme(theme),
                "{theme:?}"
            );
        }
    }

    #[test]
    fn built_in_themes_have_complete_foundational_records() {
        for theme in BUILT_IN_THEMES {
            let resolved = theme.resolve(&OperationGenerationResources::new()).unwrap();
            let text_roles = [
                &resolved.typography.root.title,
                &resolved.typography.root.subtitle,
                &resolved.typography.root.caption,
                &resolved.typography.axes.axis_title,
                &resolved.typography.axes.tick_label,
                &resolved.typography.annotations.significance_label,
                &resolved.typography.annotations.estimate_detail,
                &resolved.typography.annotations.waterfall_total,
                &resolved.typography.annotations.point_layer_label,
                &resolved.typography.annotations.panel_text,
                &resolved.typography.annotations.contrast.light,
                &resolved.typography.annotations.contrast.dark,
                &resolved.typography.guides.legend_title,
                &resolved.typography.guides.legend_label,
                &resolved.typography.composition.facet_header,
                &resolved.typography.composition.composition_header,
                &resolved.typography.composition.text_panel_title,
                &resolved.typography.composition.body,
            ];

            assert!(
                !resolved.color_scales.qualitative().colors().is_empty(),
                "{theme:?} has no colors"
            );
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
                !role.stack.families().is_empty()
                    && role.stack == crate::font::canonical::builtin_font_stack()
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
            let resolved = theme.resolve(&OperationGenerationResources::new()).unwrap();
            let foreground = color_from_rgb(resolved.foreground);
            assert_eq!(
                resolved.surfaces.canvas_root,
                Color::rgba(255, 255, 255, 255)
            );
            assert_eq!(
                resolved.surfaces.plot_panel,
                Color::rgba(255, 255, 255, 255)
            );
            assert_eq!(resolved.typography.root.title.paint, foreground);
            assert_eq!(
                resolved.typography.root.subtitle.paint,
                Color::rgb(80, 80, 80)
            );
            assert_eq!(
                resolved.typography.root.caption.paint,
                Color::rgb(80, 80, 80)
            );
            assert_eq!(resolved.typography.axes.axis_title.paint, foreground);
            assert_eq!(resolved.typography.axes.tick_label.paint, foreground);
            assert_eq!(
                resolved.typography.annotations.significance_label.paint,
                foreground
            );
        }
    }

    #[test]
    fn private_uncertainty_and_summary_strokes_are_independent_of_axis_chrome() {
        let mut theme = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let axis = theme.axes.frame.as_ref().unwrap().stroke.backend_style();
        theme.marks.uncertainty_stroke.paint = Color::rgb(11, 23, 37);
        theme.marks.uncertainty_stroke.width = 2.5;
        theme.marks.summary_stroke.paint = Color::rgb(43, 59, 71);
        theme.marks.summary_stroke.width = 3.5;

        let uncertainty = theme.marks.uncertainty_stroke.backend_style();
        let summary = theme.marks.summary_stroke.backend_style();
        assert_eq!(uncertainty.color, Color::rgb(11, 23, 37));
        assert_eq!(uncertainty.width.to_bits(), 2.5_f32.to_bits());
        assert_eq!(summary.color, Color::rgb(43, 59, 71));
        assert_eq!(summary.width.to_bits(), 3.5_f32.to_bits());
        assert_eq!(
            theme.axes.frame.as_ref().unwrap().stroke.backend_style(),
            axis
        );
        assert_ne!(uncertainty, summary);
    }

    #[test]
    fn custom_bracket_stroke_changes_annotation_identity_without_recoloring_text_or_axes() {
        let mut theme = Theme::Nature
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let original = theme.clone();
        theme.annotations.bracket_stroke.paint = Color::rgb(15, 29, 41);
        theme.annotations.bracket_stroke.width = 2.25;
        assert_ne!(theme, original);
        assert_eq!(
            theme.annotations.bracket_stroke.paint,
            Color::rgb(15, 29, 41)
        );
        assert_eq!(
            theme.annotations.bracket_stroke.width.to_bits(),
            2.25_f32.to_bits()
        );
        assert_eq!(
            theme.typography.annotations.significance_label,
            original.typography.annotations.significance_label
        );
        assert_eq!(theme.axes, original.axes);
        assert_eq!(theme.marks, original.marks);
    }

    #[test]
    fn point_label_gap_is_independent_of_point_and_text_appearance() {
        let mut theme = Theme::CHI
            .resolve(&OperationGenerationResources::new())
            .unwrap();
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
        let mut theme = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        theme.surfaces.canvas_root = Color::rgba(0, 0, 0, 0);
        theme.surfaces.plot_panel = Color::rgba(0, 0, 0, 0);
        theme.annotations.planning_backdrop = Color::rgb(0, 0, 0);
        let fill = Color::rgba(0, 0, 0, 0);
        assert_eq!(
            theme.contrasting_text(fill).unwrap(),
            &theme.typography.annotations.contrast.light
        );
        theme.canvas_surface_override = Some(Color::rgb(255, 255, 255));
        assert_eq!(
            theme.contrasting_text(fill).unwrap(),
            &theme.typography.annotations.contrast.dark
        );
    }

    #[test]
    fn histogram_curve_and_rug_styles_do_not_inherit_axis_or_bin_strokes() {
        let mut theme = Theme::Science
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let original = theme.clone();
        theme.marks.histogram_curve.paint = Color::rgb(25, 49, 73);
        theme.marks.histogram_curve.width = 2.5;
        theme.marks.histogram_rug.paint = Color::rgb(97, 121, 145);
        theme.marks.histogram_rug.width = 0.5;
        assert_ne!(theme, original);
        assert_eq!(theme.axes, original.axes);
        assert_eq!(
            theme.marks.uncertainty_stroke,
            original.marks.uncertainty_stroke
        );
        assert_eq!(
            theme.marks.histogram_curve.backend_style().color,
            Color::rgb(25, 49, 73)
        );
        assert_eq!(
            theme.marks.histogram_rug.backend_style().color,
            Color::rgb(97, 121, 145)
        );
    }

    #[test]
    fn estimate_reference_and_dumbbell_roles_do_not_alias_each_other() {
        let mut theme = Theme::Nature
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let original = theme.clone();
        theme.marks.estimate_interval.paint = Color::rgb(23, 47, 71);
        theme.marks.estimate_interval.width = 2.25;
        theme.marks.point_interval_reference.paint = Color::rgb(31, 53, 79);
        theme.marks.estimate_reference.width = 3.0;
        theme.marks.qq_reference.paint = Color::rgb(41, 67, 89);
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
            let typography = theme
                .resolve(&OperationGenerationResources::new())
                .unwrap()
                .typography;
            for role in [
                &typography.root.title,
                &typography.root.subtitle,
                &typography.root.caption,
            ] {
                let pitch = role.line_height_multiplier * f64::from(role.font_size);
                let expected = f64::from(role.font_size) + 6.0;
                assert!((pitch - expected).abs() < 1.0e-12, "{theme:?}");
            }
            for role in [
                &typography.axes.axis_title,
                &typography.axes.tick_label,
                &typography.annotations.significance_label,
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
                theme
                    .resolve(&OperationGenerationResources::new())
                    .unwrap()
                    .composition
                    .root_title_alignment(),
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
            let resolved = theme.resolve(&OperationGenerationResources::new()).unwrap();
            let (_, expected) = expected_preset(theme);
            let frame = resolved.axes.frame().expect("built-in frame");
            let ticks = resolved.axes.ticks().expect("built-in ticks");
            let grid = resolved.axes.grid().expect("built-in grid");

            assert_eq!(frame.stroke.paint, color_from_rgb(expected.foreground));
            assert!((frame.stroke.width - expected.axis_width).abs() < f32::EPSILON);
            assert_eq!(frame.stroke.pattern, StrokePattern::Solid);
            assert_eq!(frame.sides().drawing_order(), [true; 4]);
            let cell_frame = resolved.axes.cell_frame().expect("built-in cell frame");
            assert_eq!(
                cell_frame.sides().drawing_order(),
                [true, false, false, true]
            );
            assert_eq!(cell_frame.stroke(), &frame.stroke);
            assert_eq!(ticks.stroke, frame.stroke);
            assert_eq!(ticks.length, 5);
            assert_eq!(grid.paint, color_from_rgb(expected.grid_color));
            assert!((grid.width - 0.5).abs() < f32::EPSILON);
            assert_eq!(grid.pattern, StrokePattern::MediumDash);
        }
    }

    #[test]
    fn significance_label_defaults_and_exact_precedence_are_complete() {
        for theme in BUILT_IN_THEMES {
            let typography = theme
                .resolve(&OperationGenerationResources::new())
                .unwrap()
                .typography;
            let resources = crate::font::canonical::OperationGenerationResources::new();
            let default = typography
                .effective_significance_label(None, &resources)
                .unwrap();
            assert_eq!(default.font_size, 16, "{theme:?}");
            assert_eq!(default.weight, TextWeight::Normal, "{theme:?}");
            assert!(
                (default.line_height_multiplier - 1.2).abs() < f64::EPSILON,
                "{theme:?}"
            );

            let exact = SignificanceMarkStyle::new(23, TextWeight::Bold, 9.0)
                .expect("valid exact significance style");
            let effective = typography
                .effective_significance_label(Some(exact), &resources)
                .unwrap();
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
            let resolved = theme.resolve(&OperationGenerationResources::new()).unwrap();
            let lengths = resolved.root_title_lengths().unwrap();
            let resources = resolved
                .prepared_font_environment()
                .unwrap()
                .generation_resources();
            assert_eq!(
                crate::theme::composition::hard(&lengths.title_subtitle_gap, &resources)
                    .unwrap()
                    .to_bits(),
                0.0_f64.to_bits()
            );
            assert_eq!(
                crate::theme::composition::hard(
                    &lengths.title_block_to_content_clearance,
                    &resources
                )
                .unwrap()
                .to_bits(),
                14.0_f64.to_bits()
            );
            assert_eq!(
                crate::theme::composition::hard(&lengths.caption_to_content_clearance, &resources)
                    .unwrap()
                    .to_bits(),
                12.0_f64.to_bits()
            );
        }
    }

    #[test]
    fn theme_clone_discovers_one_shared_font_environment() {
        FONT_ENVIRONMENT_DISCOVERIES.with(|calls| calls.set(0));
        let theme = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let clone = theme.clone();

        theme.prepare_font_environment().unwrap();
        clone.prepare_font_environment().unwrap();

        assert!(theme.shares_font_environment_with(&clone));
        FONT_ENVIRONMENT_DISCOVERIES.with(|calls| assert_eq!(calls.get(), 1));
    }

    #[test]
    fn separately_resolved_theme_can_share_the_root_font_owner() {
        FONT_ENVIRONMENT_DISCOVERIES.with(|calls| calls.set(0));
        let root = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let mut plot = Theme::Science
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        plot.share_font_environment_from(&root);

        root.prepare_font_environment().unwrap();
        plot.prepare_font_environment().unwrap();

        assert!(root.shares_font_environment_with(&plot));
        FONT_ENVIRONMENT_DISCOVERIES.with(|calls| assert_eq!(calls.get(), 1));
    }

    #[test]
    fn runtime_font_preparation_does_not_affect_appearance_equality() {
        let prepared = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let unresolved = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        prepared.prepare_font_environment().unwrap();

        assert_eq!(prepared, unresolved);
        assert!(!prepared.shares_font_environment_with(&unresolved));
    }

    #[test]
    fn role_weight_changes_metrics_when_the_host_resolves_a_distinct_face() {
        let theme = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let normal = theme
            .measure_text(
                "Minimum WWW maximum",
                &theme.typography.root.title,
                PhysicalTextAlignment::Center,
                "title",
                false,
            )
            .unwrap();
        let mut bold_role = theme.typography.root.title.clone();
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
        let theme = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let plan = theme
            .plan_text(
                "planned text",
                &theme.typography.root.title,
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
                &theme.typography.root.title,
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
        let root = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let mut child = Theme::Science
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        child.share_font_environment_from(&root);
        let appearance = &root.typography.root.title;
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
        changed.paint = Color::rgb(3, 2, 1);
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
        assert_eq!(changed.style.paint, Color::rgb(3, 2, 1));
        assert!(!first.shares_payload(&changed));
        assert!(first.shares_analysis(&changed));
        let other_root = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        assert!(!Arc::ptr_eq(&root.text_queries, &other_root.text_queries));
    }

    #[test]
    fn text_analysis_is_font_scoped_not_presentation_scoped() {
        let root = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        root.font_environment
            .set(Ok(PreparedFontEnvironment::synthetic()))
            .unwrap();
        let mut appearance = root.typography.root.caption.clone();
        appearance.stack = crate::font::canonical::named_stack(&["Test Sans"]);
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
        appearance.paint = Color::rgb(4, 5, 6);
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
        let root = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
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
        reason = "one root verifies index scaling, stable collision order, and exact charged comparisons"
    )]
    fn indexed_text_queries_charge_search_and_matching_buckets_only() {
        let root = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        root.font_environment
            .set(Ok(PreparedFontEnvironment::synthetic()))
            .unwrap();
        let environment = root.prepared_font_environment().unwrap();
        let resources = environment.generation_resources();
        environment.share_generation_resources(&resources).unwrap();
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
        let input = indexed_query_input("A", &appearance, "owner 000");
        let (_, fingerprint) =
            CachedTextQuery::fingerprints(&input, None, true, &resources).unwrap();
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
        // Profile metadata/source (2), immutable font handle (1), only the
        // presentation hash (36+9), owner equality (10), and index comparisons.
        assert_eq!(after.1 - before.1, comparisons + 1 + 2 + 1 + 45 + 10);
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
                    environment,
                    Some(expected)
                )
                .unwrap(),
            Some(expected)
        );
        assert_eq!(
            resources.usage().unwrap().1 - before,
            8 + cache.exact.len() as u128 + 10
        );
        let before = resources.usage().unwrap().1;
        assert_eq!(
            cache
                .find(
                    &cache.shaping,
                    [0; 32],
                    (&collision, None, true),
                    false,
                    environment,
                    Some(expected)
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
                environment,
                Some(expected)
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
                    environment,
                    Some(expected)
                )
                .unwrap(),
            Some(expected)
        );
    }

    #[test]
    fn indexed_query_fingerprints_include_exact_presentation_and_provenance() {
        let appearance = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap()
            .typography
            .root
            .caption;
        let input = indexed_query_input("A", &appearance, "owner");
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let fingerprints = CachedTextQuery::fingerprints(&input, None, true, &resources).unwrap();
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
            let changed = CachedTextQuery::fingerprints(&changed, None, true, &resources).unwrap();
            assert_eq!(changed.0, fingerprints.0);
            assert_ne!(changed.1, fingerprints.1);
        }
        assert_ne!(
            CachedTextQuery::fingerprints(&input, Some(2), true, &resources)
                .unwrap()
                .1,
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
                CachedTextQuery::fingerprints(&changed, None, true, &resources)
                    .unwrap()
                    .0,
                fingerprints.0
            );
        }
        assert_ne!(
            CachedTextQuery::fingerprints(&input, None, false, &resources)
                .unwrap()
                .0,
            fingerprints.0
        );
    }

    #[test]
    fn query_hashing_admits_every_font_byte_before_the_complete_hash_pass() {
        use crate::font::canonical::{named_stack, OperationGenerationResources};
        let appearance = TextTheme {
            stack: named_stack(&["α, A"]),
            ..Theme::Publication
                .resolve(&OperationGenerationResources::new())
                .unwrap()
                .typography
                .root
                .caption
        };
        let input = indexed_query_input("A", &appearance, "owner");
        // 58 fixed bytes, source/owner bytes, one entry visit, tag+length, five name bytes.
        let work = 58 + 1 + 5 + 1 + 9 + 5;
        for remaining in [0, work - 1, work] {
            let resources = OperationGenerationResources::with_test_limits(remaining, 0);
            let result = CachedTextQuery::fingerprints(&input, None, true, &resources);
            assert_eq!(result.is_ok(), remaining == work);
            if remaining == work {
                assert_eq!(resources.usage().unwrap().1, work);
            }
            assert_eq!(resources.byte_usage().unwrap(), 0);
        }
    }

    #[test]
    fn cached_text_hit_refuses_content_work_without_allocating_or_reshaping() {
        use crate::font::canonical::{named_stack, FONT_SHAPE_CALLS};
        let theme = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        theme
            .font_environment
            .set(Ok(PreparedFontEnvironment::synthetic()))
            .unwrap();
        let appearance = TextTheme {
            stack: named_stack(&["Test Sans"]),
            font_size: 10,
            ..theme.typography.root.caption.clone()
        };
        let first = theme
            .plan_text(
                "A",
                &appearance,
                100.0,
                PhysicalTextAlignment::Start,
                "owner",
                false,
            )
            .unwrap();
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        // Profile metadata/source 2, shared font 1, presentation hash 41,
        // binary/candidate lookup 2 and owner bytes/length 6: 52.
        for remaining in [0, 51, 52] {
            let before = resources.usage().unwrap();
            let bytes = resources.byte_usage().unwrap();
            resources.set_test_limits(before.1 + remaining, bytes);
            FONT_SHAPE_CALLS.with(|calls| calls.set(0));
            let result = theme.plan_text(
                "A",
                &appearance,
                100.0,
                PhysicalTextAlignment::Start,
                "owner",
                false,
            );
            if remaining == 52 {
                assert!(first.shares_payload(&result.unwrap()));
            } else {
                assert!(result
                    .unwrap_err()
                    .to_string()
                    .contains("root processing visits"));
            }
            assert_eq!(resources.usage().unwrap().1 - before.1, remaining);
            assert_eq!(resources.usage().unwrap().0, before.0);
            assert_eq!(resources.byte_usage().unwrap(), bytes);
            FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        }
    }

    #[test]
    fn caption_publication_work_evidence_keeps_complete_required_sources() {
        let sentence = "Measured responses remained stable across independent trials under controlled conditions. ";
        for (name, caption, size, words) in [
            (
                "three_paragraphs_200",
                [sentence.repeat(7), sentence.repeat(7), sentence.repeat(6)].join("\n"),
                (800, 900),
                20,
            ),
            ("long_2048", "Measured ".repeat(2048), (1600, 8000), 2048),
        ] {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("caption.svg");
            crate::orchestrator::measurements::start();
            let result = crate::Plot::scatter(
                crate::NumericValues::new("x", [0.0, 1.0]),
                crate::NumericValues::new("y", [0.0, 1.0]),
            )
            .caption(&caption)
            .size(size.0, size.1)
            .svg_preserve_text(&path);
            let evidence = crate::orchestrator::measurements::finish();
            eprintln!("font_work_evidence,{name},visits={},bytes={},records={},calls={},cache_entries={},source_bytes={},proposal_visits={},exact_trials={}", evidence.visits, evidence.bytes, evidence.records, evidence.measurement_calls, evidence.cache_entries, caption.len(), evidence.counters[crate::orchestrator::measurements::Counter::TextProposalVisit as usize], evidence.counters[crate::orchestrator::measurements::Counter::TextExactTrial as usize]);
            result.unwrap();
            let svg = std::fs::read_to_string(&path).unwrap();
            let retained = svg
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
                .filter(|word| *word == "Measured")
                .count();
            assert_eq!(retained, words);
        }
    }

    #[test]
    fn presentation_queries_share_source_profile_and_preserve_literal_stack_identity() {
        use crate::font::canonical::{named_stack, FONT_SHAPE_CALLS};
        let theme = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        theme
            .font_environment
            .set(Ok(PreparedFontEnvironment::synthetic_faces(&[
                ("A, B", &['A', ' ']),
                ("A", &['A', ' ']),
                ("B", &['A', ' ']),
            ])))
            .unwrap();
        let mut appearance = TextTheme {
            stack: named_stack(&["A, B"]),
            font_size: 10,
            ..theme.typography.root.caption.clone()
        };
        let source = "A ".repeat(128);
        let environment = theme.prepared_font_environment().unwrap();
        environment
            .prepare_pair(
                0,
                &appearance.stack,
                crate::font::canonical::RequestedWeight::Normal,
                &['A', ' '],
                None,
            )
            .unwrap();
        environment
            .prepare_pair(
                1,
                &named_stack(&["A", "B"]),
                crate::font::canonical::RequestedWeight::Normal,
                &['A', ' '],
                None,
            )
            .unwrap();
        theme
            .plan_text(
                &source,
                &appearance,
                1600.0,
                PhysicalTextAlignment::Start,
                "first",
                false,
            )
            .unwrap();
        FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        for owner in ["second", "third"] {
            theme
                .plan_text(
                    &source,
                    &appearance,
                    1600.0,
                    PhysicalTextAlignment::Start,
                    owner,
                    false,
                )
                .unwrap();
        }
        FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        {
            let cache = theme.text_queries.lock().unwrap();
            assert_eq!(cache.profiles.len(), 1);
            assert_eq!(cache.entries.len(), 3);
            assert!(cache
                .entries
                .iter()
                .all(|entry| Arc::ptr_eq(&cache.entries[0].source, &entry.source)));
        }
        appearance.stack = named_stack(&["A", "B"]);
        theme
            .plan_text(
                &source,
                &appearance,
                1600.0,
                PhysicalTextAlignment::Start,
                "separate",
                false,
            )
            .unwrap();
        assert_eq!(theme.text_queries.lock().unwrap().profiles.len(), 2);
        // Return to an independently constructed equal literal: byte equality
        // locates the original profile; pointer identity is not its definition.
        appearance.stack = named_stack(&["A, B"]);
        FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        theme
            .plan_text(
                &source,
                &appearance,
                1600.0,
                PhysicalTextAlignment::Start,
                "first",
                false,
            )
            .unwrap();
        FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
    }

    #[test]
    fn retained_profile_does_not_trust_a_reused_mutable_input_address() {
        let theme = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
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
    fn effective_significance_copy_uses_actual_role_storage_and_refuses_before_copy() {
        use crate::font::canonical::{named_stack, FontFamily, OperationGenerationResources};
        let mut theme = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        theme.typography.annotations.significance_label.stack = named_stack(&["αX", "A, B"]);
        let bytes = 2 * size_of::<FontFamily>()
            + 3
            + 4
            + size_of::<[std::sync::atomic::AtomicUsize; 2]>()
            + size_of::<Vec<FontFamily>>();
        let work = 2 + 3 + 4;
        let exact = SignificanceMarkStyle::new(23, TextWeight::Bold, 9.0).unwrap();
        for (visits, limit) in [
            (0, bytes),
            (work - 1, bytes),
            (work, bytes - 1),
            (work, bytes),
        ] {
            let resources = OperationGenerationResources::with_test_limits(visits, limit as u64);
            let result = theme
                .typography
                .effective_significance_label(Some(exact), &resources);
            if visits == work && limit == bytes {
                let result = result.unwrap();
                assert_eq!(
                    result.stack,
                    theme.typography.annotations.significance_label.stack
                );
                assert_eq!(result.font_size, 23);
                assert_eq!(result.weight, TextWeight::Bold);
                assert_eq!(resources.byte_usage().unwrap(), bytes as u64);
                assert_eq!(resources.usage().unwrap(), (2, work));
            } else {
                assert!(result.is_err());
            }
            if visits < work {
                assert_eq!(resources.byte_usage().unwrap(), 0);
            }
        }
    }

    #[test]
    fn admitted_theme_clone_covers_all_eighteen_roles_with_exact_refusal_boundaries() {
        use crate::font::canonical::{
            named_stack, OperationGenerationResources, FONT_DISCOVERY_CALLS,
        };
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [0.0]),
            crate::NumericValues::new("y", [0.0]),
        )
        .erase();
        let mut theme = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        theme.typography.root.title.stack = named_stack(&["αX", "A, B"]);
        let roles = theme.typography.roles();
        assert_eq!(roles.len(), 18);
        let bytes = 0_u64; // all palette/gradient payloads and font entries are immutable shares
        let work = 60; // prior forty-two handles plus eighteen composition relationships
        let discoveries = FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
        for (visits, limit) in [(0, bytes), (work - 1, bytes), (work, bytes)] {
            let resources = OperationGenerationResources::with_test_limits(visits, limit);
            let copy = theme.admitted_clone(plot.data().with_axis_resources(&resources));
            if visits == work && limit == bytes {
                let copy = copy.unwrap();
                assert_eq!(copy, theme);
                assert_eq!(resources.usage().unwrap().1, work);
                assert_eq!(resources.byte_usage().unwrap(), bytes);
                for (retained, original) in copy.typography.roles().into_iter().zip(roles) {
                    assert_eq!(
                        retained.stack.families().as_ptr(),
                        original.stack.families().as_ptr()
                    );
                    for (retained, original) in retained
                        .stack
                        .families()
                        .iter()
                        .zip(original.stack.families())
                    {
                        if let (Some(retained), Some(original)) = (retained.name(), original.name())
                        {
                            assert_eq!(retained.as_ptr(), original.as_ptr());
                        }
                    }
                }
            } else {
                assert!(copy.is_err());
            }
        }
        assert_eq!(FONT_DISCOVERY_CALLS.with(std::cell::Cell::get), discoveries);
        assert!(theme.admitted_clone(plot.data()).is_err());
    }

    #[test]
    fn retained_theme_matching_prices_all_eighteen_stack_profiles() {
        use crate::font::canonical::{named_stack, OperationGenerationResources};
        let theme = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let mut retained = theme.clone();
        let work: usize = 63; // sixty payloads plus axis, guide and composition metadata witnesses
        for remaining in [0, work - 1, work] {
            let resources = OperationGenerationResources::with_test_limits(remaining as u128, 0);
            let result = retained.same(&theme, &resources);
            assert_eq!(result.is_ok(), remaining == work);
            assert_eq!(resources.usage().unwrap().1, remaining as u128);
            assert_eq!(resources.byte_usage().unwrap(), 0);
        }
        retained.typography.root.title.stack = named_stack(&["A, B"]);
        let resources = OperationGenerationResources::with_test_limits(46, 0);
        assert!(!retained.same(&theme, &resources).unwrap());
        assert_eq!(resources.usage().unwrap().1, 46); // forty-five witnesses, then differing font entry count
    }

    #[test]
    fn text_and_intrinsic_cache_hits_admit_calls_before_lookup() {
        for intrinsic in [false, true] {
            let mut root = Theme::Publication
                .resolve(&OperationGenerationResources::new())
                .unwrap();
            root.font_environment
                .set(Ok(PreparedFontEnvironment::synthetic()))
                .unwrap();
            root.typography.axes.tick_label.stack =
                crate::font::canonical::named_stack(&["Test Sans"]);
            let query = || {
                if intrinsic {
                    root.intrinsic_tick_label_width(["A"].into_iter(), 100.0)
                        .map(|_| ())
                } else {
                    root.measure_text(
                        "A",
                        &root.typography.axes.tick_label,
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
        let root = Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        root.font_environment
            .set(Ok(PreparedFontEnvironment::synthetic()))
            .unwrap();
        let mut appearance = root.typography.root.caption.clone();
        appearance.stack = crate::font::canonical::named_stack(&["Test Sans"]);
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
        let key = root
            .text_queries
            .lock()
            .unwrap()
            .query_key(
                &input,
                None,
                true,
                root.prepared_font_environment().unwrap(),
            )
            .unwrap();
        root.cache_text_query(
            &input,
            &appearance,
            None,
            true,
            CachedTextResult::Plan(first.clone()),
            key,
        )
        .unwrap();
        assert_eq!(resources.call_cache_usage().1, 1_000_000);
        let usage = resources.usage().unwrap().0;
        let bytes = resources.byte_usage().unwrap();
        let input = indexed_query_input("A", &appearance, "refused");
        let key = root
            .text_queries
            .lock()
            .unwrap()
            .query_key(
                &input,
                None,
                true,
                root.prepared_font_environment().unwrap(),
            )
            .unwrap();
        let error = root
            .cache_text_query(
                &input,
                &appearance,
                None,
                true,
                CachedTextResult::Plan(first.clone()),
                key,
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
