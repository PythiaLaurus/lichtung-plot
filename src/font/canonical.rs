// This is the private T5a3 integration surface; later text planning consumes it.
#![allow(dead_code)]

use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};

use fontdb::{Database, Family, Source, Stretch, Style, ID};
use sha2::{Digest, Sha256};
use unicode_properties::UnicodeGeneralCategory;
use unicode_segmentation::UnicodeSegmentation;

#[cfg(test)]
thread_local! {
    pub(crate) static FONT_DISCOVERY_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static FONT_PAIR_RESOLUTION_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static FONT_NORMALIZATION_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static FONT_SELECTION_PROBES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static FONT_SHAPE_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static SHAPED_GLYPH_RETAINS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static FONT_OUTLINE_GLYPHS: std::cell::RefCell<Vec<(FaceIdentity, u16)>> = const { std::cell::RefCell::new(Vec::new()) };
}

const DISCOVERY_ENTRY_LIMIT: u64 = 1_000_000;
const FONT_BYTE_LIMIT: u64 = 536_870_912;
const PROCESSING_VISIT_LIMIT: u64 = 1_000_000;
const TEXT_GENERATION_LIMIT: u128 = 1_000_000;
// Record existing logical units, but do not reject a render on their quotas.
// This does not disable arithmetic, allocation, font, layout, or PNG checks.
const ENFORCE_RESOURCE_QUOTAS: bool = false;
const SCALAR_MAP_WORDS: usize = 0x11_0000 / u64::BITS as usize;

/// Census producers can lend text or defer formatting until root admission.
pub(crate) enum CensusSource<'a> {
    Text(Cow<'a, str>),
    Formatted(std::fmt::Arguments<'a>),
}

impl<'a> From<Cow<'a, str>> for CensusSource<'a> {
    fn from(source: Cow<'a, str>) -> Self {
        Self::Text(source)
    }
}
impl<'a> From<&'a str> for CensusSource<'a> {
    fn from(source: &'a str) -> Self {
        Self::Text(Cow::Borrowed(source))
    }
}
impl<'a> From<&'a String> for CensusSource<'a> {
    fn from(source: &'a String) -> Self {
        Self::Text(Cow::Borrowed(source))
    }
}
impl From<String> for CensusSource<'_> {
    fn from(source: String) -> Self {
        Self::Text(Cow::Owned(source))
    }
}
impl<'a> From<std::fmt::Arguments<'a>> for CensusSource<'a> {
    fn from(source: std::fmt::Arguments<'a>) -> Self {
        Self::Formatted(source)
    }
}

#[derive(Default)]
struct CensusFormatCount {
    bytes: usize,
    scalars: u64,
    units: u64,
    previous: Option<char>,
    unit_has_base: bool,
    has_soft_hyphen: bool,
}

impl std::fmt::Write for CensusFormatCount {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        self.bytes = self.bytes.checked_add(text.len()).ok_or(std::fmt::Error)?;
        for scalar in text.chars() {
            self.scalars = self.scalars.checked_add(1).ok_or(std::fmt::Error)?;
            self.has_soft_hyphen |= scalar == '\u{00AD}';
            // Only attachment of a mark can extend a narrow selection unit.
            // Its grapheme boundary depends on the adjacent pair (GB9/GB9a
            // and control breaks), not RI/emoji/consonant lookbehind rules.
            let attached = self.unit_has_base
                && is_visible_mark(scalar)
                && self.previous.is_some_and(|previous| {
                    let mut pair = [0; 8];
                    let first = previous.encode_utf8(&mut pair).len();
                    let second = scalar.encode_utf8(&mut pair[first..]).len();
                    std::str::from_utf8(&pair[..first + second])
                        .expect("encoded scalars")
                        .graphemes(true)
                        .count()
                        == 1
                });
            if !attached {
                self.units = self.units.checked_add(1).ok_or(std::fmt::Error)?;
                self.unit_has_base = !is_visible_mark(scalar) && !is_nonpainting_control(scalar);
            }
            self.previous = Some(scalar);
        }
        Ok(())
    }
}

pub(crate) fn format_census_source(
    arguments: std::fmt::Arguments<'_>,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<String, FontError> {
    struct ReservedOutput<'a> {
        output: &'a mut String,
        length: usize,
    }
    impl std::fmt::Write for ReservedOutput<'_> {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            if text.len() > self.length.saturating_sub(self.output.len()) {
                return Err(std::fmt::Error);
            }
            self.output.push_str(text);
            Ok(())
        }
    }
    #[cfg(not(test))]
    if prepared_stage.is_none() {
        return Err(FontError::Reservation(
            "production census formatting requires the root prepared stage".into(),
        ));
    }
    let mut count = CensusFormatCount::default();
    count
        .write_fmt(arguments)
        .map_err(|_| FontError::Arithmetic("formatted census source count"))?;
    let resources = prepared_stage
        .as_ref()
        .map_or_else(OperationFontResources::new, |stage| stage.font_resources());
    resources.claim_visits(
        count
            .scalars
            .checked_add(count.units)
            .and_then(|work| work.checked_add(u64::from(count.has_soft_hyphen)))
            .ok_or(FontError::Arithmetic("formatted census source work"))?,
    )?;
    resources.claim_bytes(
        u64::try_from(count.bytes)
            .map_err(|_| FontError::Arithmetic("formatted census source bytes"))?,
    )?;
    let mut output = String::new();
    reserve_string_prepared_or_local(
        &mut output,
        count.bytes,
        "formatted census source",
        prepared_stage,
    )?;
    ReservedOutput {
        output: &mut output,
        length: count.bytes,
    }
    .write_fmt(arguments)
    .map_err(|_| FontError::Arithmetic("formatted census source materialization"))?;
    if output.len() != count.bytes {
        return Err(FontError::Arithmetic(
            "formatted census source count/materialization mismatch",
        ));
    }
    Ok(output)
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum FontError {
    #[error("text shaping output admission failed: {0:?}")]
    ShapeAdmission(TextGenerationClaimError),
    #[error("{resource} limit of {limit} was exceeded")]
    Limit { resource: &'static str, limit: u64 },
    #[error("font preflight arithmetic overflow while calculating {0}")]
    Arithmetic(&'static str),
    #[error("font preflight allocation refused for {0}")]
    Allocation(&'static str),
    #[error("{0}")]
    Reservation(String),
    #[error("{operation} '{path}': {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("font file changed while being read: '{0}'")]
    ChangedFile(PathBuf),
    #[error("cannot parse fontconfig file '{path}': {reason}")]
    ParseFontConfig { path: PathBuf, reason: String },
    #[error("cannot parse face index {index} in '{path}': {reason}")]
    ParseFace {
        path: PathBuf,
        index: u32,
        reason: String,
    },
    #[error("fontdb did not load every parsed face in '{0}'")]
    DatabaseLoad(PathBuf),
    #[error("no canonical system fonts were found")]
    NoSystemFonts,
    #[error("font candidate list is empty for pair {pair_ordinal}")]
    MissingFamily { pair_ordinal: u32 },
    #[error("font candidates for pair {pair_ordinal} do not contain U+{scalar:04X}")]
    MissingGlyph { pair_ordinal: u32, scalar: u32 },
    #[error("font pair {pair_ordinal} was prepared more than once")]
    DuplicatePreparedPair { pair_ordinal: u32 },
    #[error("font pair {pair_ordinal} was prepared after the operation census was sealed")]
    PreparedPairAfterSeal { pair_ordinal: u32 },
    #[error("font unit was not admitted in pair {pair_ordinal}")]
    UnknownUnit { pair_ordinal: u32 },
    #[error("unsupported text control U+{scalar:04X}")]
    UnsupportedControl { scalar: u32 },
    #[error("face {identity:?} has invalid design metrics")]
    InvalidMetrics { identity: FaceIdentity },
    #[error("shape face {identity:?}: {reason}")]
    Shape {
        identity: FaceIdentity,
        reason: &'static str,
    },
    #[error("outline glyph {glyph_id} from face {identity:?}: {reason}")]
    Outline {
        identity: FaceIdentity,
        glyph_id: u16,
        reason: &'static str,
    },
    #[error("canonical font environment synchronization was poisoned")]
    Synchronization,
}

#[derive(Debug, Clone, Copy)]
struct ResourceLimits {
    discovery_entries: u64,
    font_bytes: u64,
    processing_visits: u64,
}

impl ResourceLimits {
    const PRODUCTION: Self = Self {
        discovery_entries: DISCOVERY_ENTRY_LIMIT,
        font_bytes: FONT_BYTE_LIMIT,
        processing_visits: PROCESSING_VISIT_LIMIT,
    };
}

#[derive(Debug)]
struct FontBudget {
    resources: OperationFontResources,
    normalized_stacks: Vec<NormalizedStack>,
}

#[derive(Debug)]
struct NormalizedStack {
    source: FontStack,
    names: Vec<(String, bool)>,
}

impl FontBudget {
    fn new(limits: ResourceLimits) -> Self {
        Self {
            resources: OperationFontResources::with_limits(limits),
            normalized_stacks: Vec::new(),
        }
    }

    fn from_resources(resources: OperationFontResources) -> Self {
        Self {
            resources,
            normalized_stacks: Vec::new(),
        }
    }

    fn claim_discovery(&mut self, amount: u64) -> Result<(), FontError> {
        self.resources.claim_discovery(amount)?;
        self.claim_visits(amount)
    }

    fn claim_bytes(&mut self, amount: u64) -> Result<(), FontError> {
        self.resources.claim_bytes(amount)
    }

    fn claim_visits(&mut self, amount: u64) -> Result<(), FontError> {
        self.resources.claim_visits(amount)
    }
}

#[derive(Debug)]
struct FontResourceCounters {
    limits: ResourceLimits,
    discovery_entries: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct OperationFontResources {
    counters: Arc<Mutex<FontResourceCounters>>,
    generation: OperationGenerationResources,
}

impl OperationFontResources {
    pub(crate) fn new() -> Self {
        Self::with_limits(ResourceLimits::PRODUCTION)
    }

    fn with_limits(limits: ResourceLimits) -> Self {
        Self {
            counters: Arc::new(Mutex::new(FontResourceCounters {
                limits,
                discovery_entries: 0,
            })),
            generation: OperationGenerationResources {
                palette_cache: Arc::default(),
                counters: Arc::new(Mutex::new(GenerationResourceCounters {
                    slots: 0,
                    topology_slots: 0,
                    visits: 0,
                    calls: 0,
                    cache_entries: 0,
                    visit_limit: u128::from(limits.processing_visits),
                    bytes: 0,
                    byte_limit: limits.font_bytes,
                })),
            },
        }
    }

    pub(crate) fn claim_discovery(&self, amount: u64) -> Result<(), FontError> {
        let mut counters = self
            .counters
            .lock()
            .map_err(|_| FontError::Synchronization)?;
        let limit = counters.limits.discovery_entries;
        claim_font_resource(
            &mut counters.discovery_entries,
            amount,
            limit,
            "canonical system-font discovery entries",
        )
    }

    pub(crate) fn claim_bytes(&self, amount: u64) -> Result<(), FontError> {
        self.generation.claim_bytes(amount)
    }

    pub(crate) fn claim_visits(&self, amount: u64) -> Result<(), FontError> {
        self.generation
            .claim_visits(u128::from(amount))
            .map_err(|error| match error {
                TextGenerationClaimError::Arithmetic(resource) => FontError::Arithmetic(resource),
                TextGenerationClaimError::Limit {
                    resource, limit, ..
                } => FontError::Limit {
                    resource,
                    limit: u64::try_from(limit).expect("font visit limit originates as u64"),
                },
            })
    }

    fn limits(&self) -> Result<ResourceLimits, FontError> {
        self.counters
            .lock()
            .map(|counters| counters.limits)
            .map_err(|_| FontError::Synchronization)
    }

    pub(crate) fn usage(&self) -> Result<FontResourceUsage, FontError> {
        let counters = self
            .counters
            .lock()
            .map_err(|_| FontError::Synchronization)?;
        let generation = self
            .generation
            .counters
            .lock()
            .map_err(|_| FontError::Synchronization)?;
        Ok(FontResourceUsage {
            discovery_entries: u128::from(counters.discovery_entries),
            font_bytes: u128::from(generation.bytes),
            processing_visits: generation.visits,
        })
    }

    pub(crate) fn generation_resources(&self) -> OperationGenerationResources {
        self.generation.clone()
    }
}

fn claim_font_resource(
    used: &mut u64,
    amount: u64,
    limit: u64,
    resource: &'static str,
) -> Result<(), FontError> {
    let total = used
        .checked_add(amount)
        .ok_or(FontError::Arithmetic(resource))?;
    if ENFORCE_RESOURCE_QUOTAS && total > limit {
        return Err(FontError::Limit { resource, limit });
    }
    *used = total;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FaceIdentity {
    pub(crate) sha256: [u8; 32],
    pub(crate) face_index: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct RetainedFontFile {
    pub(crate) root_ordinal: u32,
    pub(crate) canonical_path: PathBuf,
    pub(crate) bytes: Arc<Vec<u8>>,
    pub(crate) sha256: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum NameRank {
    TypographicFamily = 0,
    LegacyFamily = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NameOccurrence {
    pub(crate) value: String,
    pub(crate) folded: String,
    pub(crate) rank: NameRank,
    pub(crate) record_ordinal: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DesignMetrics {
    pub(crate) units_per_em: u16,
    pub(crate) ascender: i16,
    pub(crate) descender: i16,
}

#[derive(Debug, Clone)]
pub(crate) struct FaceMetadata {
    pub(crate) identity: FaceIdentity,
    pub(crate) file_ordinal: usize,
    pub(crate) database_id: ID,
    pub(crate) names: Vec<NameOccurrence>,
    pub(crate) style: Style,
    pub(crate) stretch: Stretch,
    pub(crate) weight: u16,
    pub(crate) metrics: DesignMetrics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CandidateOccurrence {
    pub(crate) stack_entry_index: u32,
    pub(crate) family_match_rank: u8,
    pub(crate) style_distance: u8,
    pub(crate) stretch_distance: u8,
    pub(crate) weight_distance: u16,
    pub(crate) identity: FaceIdentity,
}

impl CandidateOccurrence {
    fn canonical_cmp(&self, other: &Self) -> Ordering {
        (
            self.stack_entry_index,
            self.family_match_rank,
            self.style_distance,
            self.stretch_distance,
            self.weight_distance,
            self.identity,
        )
            .cmp(&(
                other.stack_entry_index,
                other.family_match_rank,
                other.style_distance,
                other.stretch_distance,
                other.weight_distance,
                other.identity,
            ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateFace {
    occurrence: CandidateOccurrence,
    face_ordinal: usize,
}

#[derive(Debug, Clone, Copy)]
struct GlyphProbe {
    scalar: char,
    identity: FaceIdentity,
    covered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RequestedWeight {
    Normal,
    Semibold,
    Bold,
}

impl RequestedWeight {
    const fn value(self) -> u16 {
        match self {
            Self::Normal => 400,
            Self::Semibold => 600,
            Self::Bold => 700,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GenericFontFamily {
    Serif,
    SansSerif,
    Monospace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FontFamilyKind {
    Named(String),
    Generic(GenericFontFamily),
}

/// A literal family identity or one of the three host generic preferences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FontFamily {
    kind: FontFamilyKind,
}

impl FontFamily {
    pub(crate) fn named(name: impl Into<String>) -> crate::Result<Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(crate::Error::Data("font family name is empty".into()));
        }
        if name.chars().any(char::is_control) {
            return Err(crate::Error::Data(
                "font family name contains a control character".into(),
            ));
        }
        if name.trim() != name {
            return Err(crate::Error::Data(
                "font family name must already be trimmed".into(),
            ));
        }
        Ok(Self {
            kind: FontFamilyKind::Named(name),
        })
    }
    pub(crate) const fn generic(family: GenericFontFamily) -> Self {
        Self {
            kind: FontFamilyKind::Generic(family),
        }
    }
    pub(crate) fn name(&self) -> Option<&str> {
        match &self.kind {
            FontFamilyKind::Named(name) => Some(name),
            FontFamilyKind::Generic(_) => None,
        }
    }
    pub(crate) const fn generic_family(&self) -> Option<GenericFontFamily> {
        match self.kind {
            FontFamilyKind::Generic(family) => Some(family),
            FontFamilyKind::Named(_) => None,
        }
    }
}

/// Nonempty, ordered preferences; original named bytes and duplicates are retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FontStack {
    families: Arc<Vec<FontFamily>>,
}

impl FontStack {
    pub(crate) fn new(families: impl IntoIterator<Item = FontFamily>) -> crate::Result<Self> {
        let mut retained = Vec::new();
        for family in families {
            retained
                .try_reserve(1)
                .map_err(|_| crate::Error::Data("font stack allocation refused".into()))?;
            retained.push(family);
        }
        if retained.is_empty() {
            return Err(crate::Error::Data("font stack is empty".into()));
        }
        Ok(Self {
            families: Arc::new(retained),
        })
    }
    pub(crate) fn families(&self) -> &[FontFamily] {
        &self.families
    }

    /// One complete entry/name-byte copy pass; lengths are known without scanning names.
    pub(crate) fn byte_work(&self) -> Result<usize, FontError> {
        self.families
            .iter()
            .filter_map(FontFamily::name)
            .try_fold(self.len(), |total, name| {
                total
                    .checked_add(name.len())
                    .ok_or(FontError::Arithmetic("font stack byte work"))
            })
    }

    pub(crate) fn same(
        &self,
        other: &Self,
        resources: &OperationGenerationResources,
    ) -> Result<bool, FontError> {
        resources
            .claim_visits(1)
            .map_err(FontError::ShapeAdmission)?;
        if Arc::ptr_eq(&self.families, &other.families) {
            return Ok(true);
        }
        if self.len() != other.len() {
            return Ok(false);
        }
        for (left, right) in self.families.iter().zip(other.families.iter()) {
            resources
                .claim_visits(1)
                .map_err(FontError::ShapeAdmission)?;
            match (&left.kind, &right.kind) {
                (FontFamilyKind::Named(left), FontFamilyKind::Named(right)) => {
                    if !same_bytes(left, right, resources)? {
                        return Ok(false);
                    }
                }
                (FontFamilyKind::Generic(left), FontFamilyKind::Generic(right))
                    if left == right => {}
                _ => return Ok(false),
            }
        }
        Ok(true)
    }

    pub(crate) fn hash_into(
        &self,
        hash: &mut Sha256,
        resources: &OperationGenerationResources,
    ) -> Result<(), FontError> {
        resources
            .claim_visits(8)
            .map_err(FontError::ShapeAdmission)?;
        hash.update((self.len() as u64).to_le_bytes());
        for family in self.families.iter() {
            resources
                .claim_visits(1)
                .map_err(FontError::ShapeAdmission)?;
            match &family.kind {
                FontFamilyKind::Named(name) => {
                    resources
                        .claim_visits(9 + name.len() as u128)
                        .map_err(FontError::ShapeAdmission)?;
                    hash.update([0]);
                    hash.update((name.len() as u64).to_le_bytes());
                    hash.update(name.as_bytes());
                }
                FontFamilyKind::Generic(family) => {
                    resources
                        .claim_visits(2)
                        .map_err(FontError::ShapeAdmission)?;
                    hash.update([1, *family as u8]);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn copy_with_generation(
        &self,
        resources: &OperationGenerationResources,
    ) -> Result<Self, FontError> {
        resources
            .claim_visits(self.byte_work()? as u128)
            .map_err(FontError::ShapeAdmission)?;
        resources.claim_bytes(font_stack_arc_bytes()?)?;
        resources
            .claim_slots(self.len() as u128)
            .map_err(FontError::ShapeAdmission)?;
        resources.claim_bytes(
            self.len()
                .checked_mul(size_of::<FontFamily>())
                .ok_or(FontError::Arithmetic("font stack bytes"))? as u64,
        )?;
        let mut families = Vec::new();
        families
            .try_reserve_exact(self.len())
            .map_err(|_| FontError::Allocation("font stack entries"))?;
        for family in self {
            let kind = match &family.kind {
                FontFamilyKind::Named(name) => {
                    resources.claim_bytes(name.len() as u64)?;
                    let mut retained = String::new();
                    retained
                        .try_reserve_exact(name.len())
                        .map_err(|_| FontError::Allocation("font family name"))?;
                    retained.push_str(name);
                    FontFamilyKind::Named(retained)
                }
                FontFamilyKind::Generic(family) => FontFamilyKind::Generic(*family),
            };
            families.push(FontFamily { kind });
        }
        Ok(Self {
            families: Arc::new(families),
        })
    }

    pub(crate) fn share(
        &self,
        resources: &OperationGenerationResources,
    ) -> Result<Self, FontError> {
        resources
            .claim_visits(1)
            .map_err(FontError::ShapeAdmission)?;
        Ok(self.clone())
    }

    pub(crate) fn copy_with_stage(
        &self,
        resources: &OperationFontResources,
        stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<Self, FontError> {
        copy_stack_with_stage(self, resources, stage)
    }

    pub(crate) fn admit_copy<'a>(
        &'a self,
        resources: &OperationFontResources,
    ) -> Result<AdmittedFontStackCopy<'a>, FontError> {
        resources.claim_visits(self.byte_work()? as u64)?;
        Ok(AdmittedFontStackCopy {
            source: self,
            resources: resources.clone(),
        })
    }

    fn write_css(&self, output: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (index, family) in self.families.iter().enumerate() {
            if index != 0 {
                output.write_str(", ")?;
            }
            match &family.kind {
                FontFamilyKind::Generic(family) => output.write_str(match family {
                    GenericFontFamily::Serif => "serif",
                    GenericFontFamily::SansSerif => "sans-serif",
                    GenericFontFamily::Monospace => "monospace",
                })?,
                FontFamilyKind::Named(name) => {
                    if css_unquoted_family(name) {
                        output.write_str(name)?;
                    } else {
                        output.write_char('"')?;
                        for scalar in name.chars() {
                            if matches!(scalar, '"' | '\\') {
                                output.write_char('\\')?;
                            }
                            output.write_char(scalar)?;
                        }
                        output.write_char('"')?;
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn css(&self, resources: &OperationFontResources) -> Result<String, FontError> {
        // Count and materialization each consume the known UTF-8 input once.
        let work = self
            .byte_work()?
            .checked_mul(2)
            .ok_or(FontError::Arithmetic("font stack CSS work"))?;
        resources.claim_visits(work as u64)?;
        let mut count = CssCount(0);
        self.write_css(&mut count)
            .map_err(|_| FontError::Arithmetic("font stack CSS bytes"))?;
        resources.claim_bytes(count.0 as u64)?;
        let mut output = String::new();
        output
            .try_reserve_exact(count.0)
            .map_err(|_| FontError::Allocation("font stack CSS"))?;
        self.write_css(&mut output)
            .map_err(|_| FontError::Arithmetic("font stack CSS"))?;
        Ok(output)
    }
}

/// Single-use proof of copy work admission; holds the immutable source and its actual operation.
pub(crate) struct AdmittedFontStackCopy<'a> {
    source: &'a FontStack,
    resources: OperationFontResources,
}

impl AdmittedFontStackCopy<'_> {
    pub(crate) fn copy_with_stage(
        self,
        stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<FontStack, FontError> {
        copy_admitted_stack_with_stage(self, stage)
    }
}

fn font_stack_arc_bytes() -> Result<u64, FontError> {
    let layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(std::alloc::Layout::new::<Vec<FontFamily>>())
        .map_err(|_| FontError::Arithmetic("font stack Arc layout"))?
        .0
        .pad_to_align();
    u64::try_from(layout.size()).map_err(|_| FontError::Arithmetic("font stack Arc bytes"))
}

struct CssCount(usize);
impl std::fmt::Write for CssCount {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        self.0 = self.0.checked_add(text.len()).ok_or(std::fmt::Error)?;
        Ok(())
    }
}

fn css_unquoted_family(name: &str) -> bool {
    ![
        "serif",
        "sans-serif",
        "monospace",
        "cursive",
        "fantasy",
        "system-ui",
        "ui-serif",
        "ui-sans-serif",
        "ui-monospace",
        "ui-rounded",
        "math",
        "fangsong",
        "emoji",
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
        "default",
    ]
    .iter()
    .any(|keyword| {
        name.split(' ')
            .any(|word| word.eq_ignore_ascii_case(keyword))
    }) && name.split(' ').all(css_identifier)
}

fn css_identifier(word: &str) -> bool {
    let start = |c: char| c.is_ascii_alphabetic() || c == '_' || !c.is_ascii();
    let mut chars = word.chars();
    let valid_start = match chars.next() {
        Some('-') => chars.next().is_some_and(|c| start(c) || c == '-'),
        Some(c) => start(c),
        None => false,
    };
    valid_start && chars.all(|c| start(c) || c.is_ascii_digit() || c == '-')
}

impl std::ops::Deref for FontStack {
    type Target = [FontFamily];
    fn deref(&self) -> &Self::Target {
        self.families()
    }
}

impl<'a> IntoIterator for &'a FontStack {
    type Item = &'a FontFamily;
    type IntoIter = std::slice::Iter<'a, FontFamily>;
    fn into_iter(self) -> Self::IntoIter {
        self.families.iter()
    }
}

pub(crate) fn same_bytes(
    left: &str,
    right: &str,
    resources: &OperationGenerationResources,
) -> Result<bool, FontError> {
    resources
        .claim_visits(1)
        .map_err(FontError::ShapeAdmission)?;
    // Concurrent immutable borrows of this exact range already denote the same bytes.
    if std::ptr::eq(left, right) {
        return Ok(true);
    }
    if left.len() != right.len() {
        return Ok(false);
    }
    for index in 0..left.len() {
        resources
            .claim_visits(1)
            .map_err(FontError::ShapeAdmission)?;
        if left.as_bytes()[index] != right.as_bytes()[index] {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) fn builtin_font_stack() -> FontStack {
    FontStack::new(
        [
            "Noto Sans",
            "Noto Sans CJK SC",
            "DejaVu Sans",
            "Liberation Sans",
            "Arial",
        ]
        .into_iter()
        .map(|name| FontFamily::named(name).expect("built-in literal family"))
        .chain([FontFamily::generic(GenericFontFamily::SansSerif)]),
    )
    .expect("nonempty built-in font preferences")
}

#[cfg(test)]
pub(crate) fn named_stack(names: &[&str]) -> FontStack {
    FontStack::new(names.iter().map(|name| FontFamily::named(*name).unwrap())).unwrap()
}

#[cfg(test)]
pub(crate) fn test_sans_stack() -> &'static FontStack {
    static STACK: std::sync::OnceLock<FontStack> = std::sync::OnceLock::new();
    STACK.get_or_init(|| named_stack(&["Test Sans"]))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PairProvenance {
    pub(crate) pair_ordinal: u32,
    pub(crate) weight: RequestedWeight,
    pub(crate) candidates: Vec<FaceIdentity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScalarFaceSelection {
    Face(FaceIdentity),
    NonRenderingMandatoryBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SelectedScalarFace {
    pub(crate) scalar: char,
    pub(crate) selection: ScalarFaceSelection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectedScalarFaces(Vec<SelectedScalarFace>);

impl SelectedScalarFaces {
    #[expect(
        clippy::trivially_copy_pass_by_ref,
        reason = "matches slice binary_search_by_key call sites"
    )]
    pub(crate) fn binary_search_by_key<F>(&self, scalar: &char, mut key: F) -> Result<usize, usize>
    where
        F: FnMut(&SelectedScalarFace) -> char,
    {
        // Storage follows first observation rather than scalar-value order.
        self.0
            .iter()
            .position(|selected| key(selected) == *scalar)
            .ok_or(self.0.len())
    }
}

impl std::ops::Deref for SelectedScalarFaces {
    type Target = [SelectedScalarFace];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedFontPair {
    pub(crate) provenance: PairProvenance,
    pub(crate) primary: FaceIdentity,
    pub(crate) selected_scalars: SelectedScalarFaces,
    pub(crate) selected_units: Vec<SelectedFontUnit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectedFontUnit {
    pub(crate) source: String,
    pub(crate) face: Option<FaceIdentity>,
}

impl ResolvedFontPair {
    pub(crate) fn selection_for_unit(
        &self,
        unit: &str,
        scalar: char,
    ) -> Result<ScalarFaceSelection, FontError> {
        if unit.chars().count() > 1 && unit.chars().any(is_visible_mark) {
            let retained = self
                .selected_units
                .iter()
                .find(|entry| entry.source == unit)
                .ok_or(FontError::UnknownUnit {
                    pair_ordinal: self.provenance.pair_ordinal,
                })?;
            if let Some(face) = retained.face {
                return Ok(ScalarFaceSelection::Face(face));
            }
        }
        self.selected_scalars
            .iter()
            .find(|entry| entry.scalar == scalar)
            .map(|entry| entry.selection)
            .ok_or(FontError::MissingGlyph {
                pair_ordinal: self.provenance.pair_ordinal,
                scalar: scalar as u32,
            })
    }
}

fn is_visible_mark(scalar: char) -> bool {
    matches!(
        scalar.general_category(),
        unicode_properties::GeneralCategory::NonspacingMark
            | unicode_properties::GeneralCategory::SpacingMark
            | unicode_properties::GeneralCategory::EnclosingMark
    ) && !is_nonpainting_control(scalar)
}

/// Narrow base-plus-mark units, never whole emoji sequences or words.
pub(crate) fn selection_units(source: &str) -> impl Iterator<Item = &str> {
    source.graphemes(true).flat_map(|grapheme| {
        let mut remaining = grapheme;
        std::iter::from_fn(move || {
            let first = remaining.chars().next()?;
            let end = remaining
                .char_indices()
                .skip(1)
                .find(|(_, scalar)| {
                    !is_visible_mark(*scalar)
                        || is_visible_mark(first)
                        || is_nonpainting_control(first)
                })
                .map_or(remaining.len(), |(index, _)| index);
            let (unit, rest) = remaining.split_at(end);
            remaining = rest;
            Some(unit)
        })
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShapingDirection {
    LeftToRight,
    RightToLeft,
}

impl From<ShapingDirection> for rustybuzz::Direction {
    fn from(value: ShapingDirection) -> Self {
        match value {
            ShapingDirection::LeftToRight => Self::LeftToRight,
            ShapingDirection::RightToLeft => Self::RightToLeft,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GlyphInk {
    pub(crate) x_min: i16,
    pub(crate) y_min: i16,
    pub(crate) x_max: i16,
    pub(crate) y_max: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShapedGlyph {
    pub(crate) glyph_id: u16,
    pub(crate) cluster: u32,
    pub(crate) x_advance: i32,
    pub(crate) y_advance: i32,
    pub(crate) x_offset: i32,
    pub(crate) y_offset: i32,
    pub(crate) ink: Option<GlyphInk>,
}

#[derive(Debug, Clone)]
pub(crate) struct ShapedRun {
    pub(crate) identity: FaceIdentity,
    pub(crate) direction: ShapingDirection,
    pub(crate) script: rustybuzz::Script,
    pub(crate) metrics: DesignMetrics,
    pub(crate) glyphs: Vec<ShapedGlyph>,
}

#[derive(Debug)]
struct BinarySource(Arc<Vec<u8>>);

impl AsRef<[u8]> for BinarySource {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, Clone)]
struct SourceRoot {
    ordinal: u32,
    path: PathBuf,
    entry_prefix: Option<&'static str>,
}

#[derive(Debug, Default)]
struct HostGenericNames {
    serif: Option<String>,
    sans_serif: Option<String>,
    cursive: Option<String>,
    fantasy: Option<String>,
    monospace: Option<String>,
}

#[derive(Debug)]
struct HostSources {
    roots: Vec<SourceRoot>,
    generics: HostGenericNames,
}

#[derive(Debug)]
struct DiscoveryRecord {
    root_ordinal: u32,
    path: PathBuf,
    entry_prefix: Option<&'static str>,
    admit_file: bool,
}

pub(crate) struct ReservedDiscoveryRecords {
    records: Vec<DiscoveryRecord>,
}

struct DiscoveryFontHandle {
    file: File,
}

#[cfg(test)]
thread_local! {
    // Live handles, peak handles, opens, reads, retained files.
    static DISCOVERY_HANDLES: std::cell::Cell<[usize; 5]> = const { std::cell::Cell::new([0; 5]) };
    static DISCOVERY_AFTER_ADMISSION: std::cell::Cell<Option<fn(&Path)>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
impl Drop for DiscoveryFontHandle {
    fn drop(&mut self) {
        DISCOVERY_HANDLES.with(|state| {
            let mut counts = state.get();
            counts[0] -= 1;
            state.set(counts);
        });
    }
}

#[derive(Debug)]
pub(crate) struct CanonicalFontCore {
    database: Database,
    files: Vec<RetainedFontFile>,
    faces: Vec<FaceMetadata>,
    scalar_presence: Vec<u64>,
    prepared_pairs: Vec<PreparedPair>,
    incremental_pairs: Vec<IncrementalPair>,
    prepared_pairs_sealed: bool,
    generation_resources: OperationGenerationResources,
    budget: FontBudget,
}

#[derive(Debug)]
struct IncrementalPair {
    stack: FontStack,
    weight: RequestedWeight,
    candidates: Vec<CandidateFace>,
    selected_scalars: Vec<SelectedScalarFace>,
}

#[derive(Debug, Clone)]
struct PreparedPair {
    stack: FontStack,
    weight: RequestedWeight,
    resolved: ResolvedFontPair,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FontPairPreparationRequest<'a> {
    pair_ordinal: u32,
    stack: &'a FontStack,
    weight: RequestedWeight,
    required_scalars: &'a [char],
}

impl<'a> FontPairPreparationRequest<'a> {
    pub(crate) const fn new(
        pair_ordinal: u32,
        stack: &'a FontStack,
        weight: RequestedWeight,
        required_scalars: &'a [char],
    ) -> Self {
        Self {
            pair_ordinal,
            stack,
            weight,
            required_scalars,
        }
    }
}

struct PairPreparation<'a> {
    request: FontPairPreparationRequest<'a>,
    resolved_names: Vec<(String, bool)>,
    raw_candidate_count: usize,
    candidate_count: usize,
    scalar_count: usize,
    maximum_probes: usize,
    candidates: Vec<CandidateFace>,
    provenance: Vec<FaceIdentity>,
    selected_scalars: Vec<SelectedScalarFace>,
    probes: Vec<GlyphProbe>,
    retained_stack: Vec<FontFamily>,
}

impl<'a> PairPreparation<'a> {
    fn new(request: FontPairPreparationRequest<'a>) -> Self {
        Self {
            request,
            resolved_names: Vec::new(),
            raw_candidate_count: 0,
            candidate_count: 0,
            scalar_count: 0,
            maximum_probes: 0,
            candidates: Vec::new(),
            provenance: Vec::new(),
            selected_scalars: Vec::new(),
            probes: Vec::new(),
            retained_stack: Vec::new(),
        }
    }

    fn storage_is_reserved(&self) -> bool {
        self.resolved_names.capacity() >= self.request.stack.len()
            && self.candidates.capacity() >= self.raw_candidate_count
            && self.provenance.capacity() >= self.candidate_count
            && self.selected_scalars.capacity() >= self.scalar_count
            && self.probes.capacity() >= self.maximum_probes
            && self.retained_stack.capacity() >= self.request.stack.len()
            && self.retained_stack.len() == self.request.stack.len()
            && self
                .retained_stack
                .iter()
                .zip(self.request.stack)
                .all(|(retained, source)| match (&retained.kind, &source.kind) {
                    (FontFamilyKind::Named(retained), FontFamilyKind::Named(source)) => {
                        retained.capacity() >= source.len()
                    }
                    (FontFamilyKind::Generic(retained), FontFamilyKind::Generic(source)) => {
                        retained == source
                    }
                    _ => false,
                })
    }
}

/// One operation's retained font database and exact source bytes.
#[derive(Debug, Clone)]
pub(crate) struct PreparedFontEnvironment {
    core: Arc<RwLock<CanonicalFontCore>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FontResourceUsage {
    pub(crate) discovery_entries: u128,
    pub(crate) font_bytes: u128,
    pub(crate) processing_visits: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextGenerationClaimError {
    Arithmetic(&'static str),
    Limit {
        resource: &'static str,
        limit: u128,
        cumulative_claim: u128,
    },
}

#[derive(Debug)]
struct GenerationResourceCounters {
    slots: u128,
    topology_slots: u128,
    visits: u128,
    calls: u128,
    cache_entries: u128,
    visit_limit: u128,
    bytes: u64,
    byte_limit: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct OperationGenerationResources {
    counters: Arc<Mutex<GenerationResourceCounters>>,
    pub(crate) palette_cache: Arc<Mutex<crate::theme::PaletteCache>>,
}

/// Proof that the logical owner admitted a call before consulting its cache.
pub(crate) struct MeasurementCall(());

impl OperationGenerationResources {
    pub(crate) fn same_operation(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.counters, &other.counters)
    }

    #[cfg(test)]
    pub(crate) fn set_test_limits(&self, visits: u128, bytes: u64) {
        let mut counters = self.counters.lock().unwrap();
        counters.visit_limit = visits;
        counters.byte_limit = bytes;
    }

    #[cfg(test)]
    pub(crate) fn with_test_limits(visits: u128, bytes: u64) -> Self {
        Self {
            palette_cache: Arc::default(),
            counters: Arc::new(Mutex::new(GenerationResourceCounters {
                slots: 0,
                topology_slots: 0,
                visits: 0,
                calls: 0,
                cache_entries: 0,
                visit_limit: visits,
                bytes: 0,
                byte_limit: bytes,
            })),
        }
    }

    pub(crate) fn new() -> Self {
        Self {
            palette_cache: Arc::default(),
            counters: Arc::new(Mutex::new(GenerationResourceCounters {
                slots: 0,
                topology_slots: 0,
                visits: 0,
                calls: 0,
                cache_entries: 0,
                visit_limit: u128::from(PROCESSING_VISIT_LIMIT),
                bytes: 0,
                byte_limit: FONT_BYTE_LIMIT,
            })),
        }
    }

    pub(crate) fn claim_slots(&self, amount: u128) -> Result<(), TextGenerationClaimError> {
        let mut counters = self
            .counters
            .lock()
            .map_err(|_| TextGenerationClaimError::Arithmetic("text resource synchronization"))?;
        claim_text_generation_resource(&mut counters.slots, amount, "retained planning records")
    }

    pub(crate) fn claim_calls(&self, amount: u128) -> Result<(), TextGenerationClaimError> {
        let mut counters = self
            .counters
            .lock()
            .map_err(|_| TextGenerationClaimError::Arithmetic("text resource synchronization"))?;
        claim_text_generation_resource(
            &mut counters.calls,
            amount,
            "measurement/intrinsic call attempts",
        )
    }

    pub(crate) fn claim_topology_slots(
        &self,
        amount: u128,
    ) -> Result<(), TextGenerationClaimError> {
        let mut counters = self.counters.lock().map_err(|_| {
            TextGenerationClaimError::Arithmetic("topology resource synchronization")
        })?;
        claim_text_generation_resource(
            &mut counters.topology_slots,
            amount,
            "prepared topology slots",
        )
    }

    #[cfg(test)]
    pub(crate) fn topology_usage(&self) -> u128 {
        self.counters.lock().unwrap().topology_slots
    }

    pub(crate) fn claim_cache_entries(&self, amount: u128) -> Result<(), TextGenerationClaimError> {
        let mut counters = self
            .counters
            .lock()
            .map_err(|_| TextGenerationClaimError::Arithmetic("text resource synchronization"))?;
        claim_text_generation_resource(
            &mut counters.cache_entries,
            amount,
            "root-local cache entries",
        )
    }

    #[cfg(test)]
    pub(crate) fn call_cache_usage(&self) -> (u128, u128) {
        let counters = self.counters.lock().unwrap();
        (counters.calls, counters.cache_entries)
    }

    pub(crate) fn claim_bytes(&self, amount: u64) -> Result<(), FontError> {
        let mut counters = self
            .counters
            .lock()
            .map_err(|_| FontError::Synchronization)?;
        let limit = counters.byte_limit;
        claim_font_resource(&mut counters.bytes, amount, limit, "system-font bytes")
    }

    pub(crate) fn claim_visits(&self, amount: u128) -> Result<(), TextGenerationClaimError> {
        let mut counters = self
            .counters
            .lock()
            .map_err(|_| TextGenerationClaimError::Arithmetic("text resource synchronization"))?;
        let next =
            counters
                .visits
                .checked_add(amount)
                .ok_or(TextGenerationClaimError::Arithmetic(
                    "root processing visits",
                ))?;
        if ENFORCE_RESOURCE_QUOTAS && next > counters.visit_limit {
            return Err(TextGenerationClaimError::Limit {
                resource: "root processing visits",
                limit: counters.visit_limit,
                cumulative_claim: next,
            });
        }
        counters.visits = next;
        Ok(())
    }

    pub(crate) fn usage(&self) -> Result<(u128, u128), TextGenerationClaimError> {
        let counters = self
            .counters
            .lock()
            .map_err(|_| TextGenerationClaimError::Arithmetic("text resource synchronization"))?;
        Ok((counters.slots, counters.visits))
    }

    #[cfg(test)]
    pub(crate) fn byte_usage(&self) -> Result<u64, FontError> {
        self.counters
            .lock()
            .map(|counters| counters.bytes)
            .map_err(|_| FontError::Synchronization)
    }
}

impl PreparedFontEnvironment {
    pub(crate) fn font_resources(&self) -> Result<OperationFontResources, FontError> {
        Ok(self
            .core
            .read()
            .map_err(|_| FontError::Synchronization)?
            .budget
            .resources
            .clone())
    }
    pub(crate) fn discover_system() -> Result<Self, FontError> {
        #[cfg(test)]
        FONT_DISCOVERY_CALLS.with(|calls| calls.set(calls.get() + 1));
        let scalar_presence = scalar_presence_map()?;
        Ok(Self {
            core: Arc::new(RwLock::new(
                CanonicalFontCore::discover_system_with_resources(
                    FontBudget::new(ResourceLimits::PRODUCTION),
                    scalar_presence,
                    None,
                    None,
                )?,
            )),
        })
    }

    pub(crate) fn discover_system_with_prepared_resources(
        scalar_presence: Vec<u64>,
        font_resources: OperationFontResources,
        discovery: ReservedDiscoveryRecords,
        prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<Self, FontError> {
        #[cfg(test)]
        FONT_DISCOVERY_CALLS.with(|calls| calls.set(calls.get() + 1));
        let budget = FontBudget::from_resources(font_resources);
        Ok(Self {
            core: Arc::new(RwLock::new(
                CanonicalFontCore::discover_system_with_resources(
                    budget,
                    scalar_presence,
                    Some(discovery),
                    Some(prepared_stage),
                )?,
            )),
        })
    }

    pub(crate) fn reserve_system_discovery(
        prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<ReservedDiscoveryRecords, FontError> {
        let mut records = Vec::new();
        prepared_stage
            .reserve_prepared(
                &mut records,
                usize::try_from(DISCOVERY_ENTRY_LIMIT)
                    .map_err(|_| FontError::Arithmetic("discovery reservation"))?,
                "canonical font discovery records",
            )
            .map_err(|error| FontError::Reservation(error.to_string()))?;
        Ok(ReservedDiscoveryRecords { records })
    }

    pub(crate) fn resolve_pair(
        &self,
        pair_ordinal: u32,
        stack: &FontStack,
        weight: RequestedWeight,
        required_scalars: &[char],
    ) -> Result<ResolvedFontPair, FontError> {
        let mut core = self.core.write().map_err(|_| FontError::Synchronization)?;
        if core.prepared_pairs_sealed {
            return Err(FontError::PreparedPairAfterSeal { pair_ordinal });
        }
        core.resolve_pair(pair_ordinal, stack, weight, required_scalars, true)
    }

    pub(crate) fn claim_preflight_visits(&self, amount: u64) -> Result<(), FontError> {
        self.core
            .write()
            .map_err(|_| FontError::Synchronization)?
            .budget
            .claim_visits(amount)
    }

    pub(crate) fn claim_unicode_bytes(&self, amount: u64) -> Result<(), FontError> {
        self.core
            .read()
            .map_err(|_| FontError::Synchronization)?
            .budget
            .resources
            .claim_bytes(amount)
    }

    pub(crate) fn claim_text_generation_slots(
        &self,
        amount: u128,
    ) -> Result<(), TextGenerationClaimError> {
        let resources = self
            .core
            .read()
            .map_err(|_| TextGenerationClaimError::Arithmetic("text resource synchronization"))?
            .generation_resources
            .clone();
        resources.claim_slots(amount)
    }

    pub(crate) fn claim_processing_visits(
        &self,
        amount: u128,
    ) -> Result<(), TextGenerationClaimError> {
        let resources = self
            .core
            .read()
            .map_err(|_| TextGenerationClaimError::Arithmetic("text resource synchronization"))?
            .generation_resources
            .clone();
        resources.claim_visits(amount)
    }

    pub(crate) fn claim_measurement_call(
        &self,
    ) -> Result<MeasurementCall, TextGenerationClaimError> {
        self.core
            .read()
            .map_err(|_| TextGenerationClaimError::Arithmetic("text resource synchronization"))?
            .generation_resources
            .claim_calls(1)?;
        Ok(MeasurementCall(()))
    }

    pub(crate) fn claim_cache_entries(&self, amount: u128) -> Result<(), TextGenerationClaimError> {
        self.core
            .read()
            .map_err(|_| TextGenerationClaimError::Arithmetic("text resource synchronization"))?
            .generation_resources
            .claim_cache_entries(amount)
    }

    pub(crate) fn share_generation_resources(
        &self,
        resources: &OperationGenerationResources,
    ) -> Result<(), FontError> {
        let core = self.core.write().map_err(|_| FontError::Synchronization)?;
        // Rebinding a started environment would detach its existing clones and
        // erase admissions. Roots must supply their shared budget before prepare.
        if !Arc::ptr_eq(&core.generation_resources.counters, &resources.counters) {
            return Err(FontError::Arithmetic(
                "cannot replace an operation resource ledger",
            ));
        }
        Ok(())
    }

    pub(crate) fn generation_resources(&self) -> OperationGenerationResources {
        self.core.read().unwrap().generation_resources.clone()
    }

    pub(crate) fn reserve_prepared_pairs(
        &self,
        count: usize,
        prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), FontError> {
        let mut core = self.core.write().map_err(|_| FontError::Synchronization)?;
        reserve_prepared_or_local(
            &mut core.prepared_pairs,
            count,
            "prepared font pairs",
            prepared_stage,
        )
    }

    pub(crate) fn prepare_pairs<'a>(
        &self,
        requests: impl ExactSizeIterator<Item = FontPairPreparationRequest<'a>>,
        prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), FontError> {
        self.core
            .write()
            .map_err(|_| FontError::Synchronization)?
            .prepare_pairs_with_stage(requests, prepared_stage)
    }

    pub(crate) fn prepare_pair(
        &self,
        pair_ordinal: u32,
        stack: &FontStack,
        weight: RequestedWeight,
        required_scalars: &[char],
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), FontError> {
        let mut core = self.core.write().map_err(|_| FontError::Synchronization)?;
        if core.prepared_pairs_sealed {
            return Err(FontError::PreparedPairAfterSeal { pair_ordinal });
        }
        for prepared in &core.prepared_pairs {
            core.generation_resources
                .claim_visits(1)
                .map_err(FontError::ShapeAdmission)?;
            if prepared.resolved.provenance.pair_ordinal == pair_ordinal
                || (prepared.weight == weight
                    && prepared.stack.same(stack, &core.generation_resources)?)
            {
                return Err(FontError::DuplicatePreparedPair { pair_ordinal });
            }
        }
        let resolved = core.resolve_pair_with_stage(
            pair_ordinal,
            stack,
            weight,
            required_scalars,
            false,
            prepared_stage.as_deref_mut(),
        )?;
        let retained_stack = copy_stack_with_stage(stack, &core.budget.resources, prepared_stage)?;
        core.prepared_pairs.push(PreparedPair {
            stack: retained_stack,
            weight,
            resolved,
        });
        Ok(())
    }

    pub(crate) fn seal_prepared_pairs(&self) -> Result<(), FontError> {
        self.core
            .write()
            .map_err(|_| FontError::Synchronization)?
            .prepared_pairs_sealed = true;
        Ok(())
    }

    pub(crate) fn prepare_source_units(
        &self,
        pair_ordinal: u32,
        source: &str,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), FontError> {
        let mut core = self.core.write().map_err(|_| FontError::Synchronization)?;
        if core.prepared_pairs_sealed {
            return Err(FontError::PreparedPairAfterSeal { pair_ordinal });
        }
        // Staged requirements admitted scalar/unit traversal before their source copy.
        if prepared_stage.is_none() {
            core.budget.claim_visits(source.chars().count() as u64)?;
        }
        let index = core
            .prepared_pairs
            .iter()
            .position(|pair| pair.resolved.provenance.pair_ordinal == pair_ordinal)
            .ok_or(FontError::MissingFamily { pair_ordinal })?;
        for unit in selection_units(source)
            .filter(|unit| unit.chars().count() > 1 && unit.chars().any(is_visible_mark))
        {
            let pair = &core.prepared_pairs[index].resolved;
            let mut found = false;
            for entry in &pair.selected_units {
                core.budget.resources.claim_visits(1)?;
                if entry.source == unit {
                    found = true;
                    break;
                }
            }
            if found {
                continue;
            }
            let mut selected = None;
            for identity in &pair.provenance.candidates {
                core.budget.resources.claim_visits(1)?;
                let mut metadata = None;
                for face in &core.faces {
                    core.budget.resources.claim_visits(1)?;
                    if face.identity == *identity {
                        metadata = Some(face);
                        break;
                    }
                }
                let metadata = metadata.ok_or(FontError::InvalidMetrics {
                    identity: *identity,
                })?;
                let file = &core.files[metadata.file_ordinal];
                let parsed =
                    ttf_parser::Face::parse(&file.bytes, identity.face_index).map_err(|error| {
                        FontError::ParseFace {
                            path: file.canonical_path.clone(),
                            index: identity.face_index,
                            reason: error.to_string(),
                        }
                    })?;
                let mut covered = true;
                for scalar in unit.chars() {
                    core.budget.resources.claim_visits(1)?;
                    #[cfg(test)]
                    FONT_SELECTION_PROBES.with(|calls| calls.set(calls.get() + 1));
                    #[cfg(test)]
                    if !is_nonpainting_control(scalar) {
                        crate::orchestrator::measurements::count(
                            crate::orchestrator::measurements::Counter::FontUnitProbe,
                        );
                    }
                    if !is_nonpainting_control(scalar) && parsed.glyph_index(scalar).is_none() {
                        covered = false;
                        break;
                    }
                }
                if covered {
                    selected = Some(*identity);
                    break;
                }
            }
            let mut retained = String::new();
            core.budget.claim_bytes(unit.len() as u64)?;
            if let Some(stage) = prepared_stage.as_deref_mut() {
                stage
                    .reserve_prepared_string(&mut retained, unit.len(), "font unit source")
                    .map_err(|error| FontError::Reservation(error.to_string()))?;
            } else {
                retained
                    .try_reserve_exact(unit.len())
                    .map_err(|_| FontError::Allocation("font unit source"))?;
            }
            let resources = core.budget.resources.clone();
            reserve_cached_with_stage(
                &mut core.prepared_pairs[index].resolved.selected_units,
                1,
                &resources,
                prepared_stage.as_deref_mut(),
            )?;
            retained.push_str(unit);
            core.prepared_pairs[index]
                .resolved
                .selected_units
                .push(SelectedFontUnit {
                    source: retained,
                    face: selected,
                });
        }
        Ok(())
    }

    pub(crate) fn prepared_pairs_are_sealed(&self) -> Result<bool, FontError> {
        Ok(self
            .core
            .read()
            .map_err(|_| FontError::Synchronization)?
            .prepared_pairs_sealed)
    }

    pub(crate) fn prepared_pair(
        &self,
        stack: &FontStack,
        weight: RequestedWeight,
    ) -> Result<Option<ResolvedFontPair>, FontError> {
        let core = self.core.read().map_err(|_| FontError::Synchronization)?;
        for prepared in &core.prepared_pairs {
            core.generation_resources
                .claim_visits(1)
                .map_err(FontError::ShapeAdmission)?;
            if prepared.weight == weight
                && prepared.stack.same(stack, &core.generation_resources)?
            {
                return Ok(Some(prepared.resolved.clone()));
            }
        }
        Ok(None)
    }

    pub(crate) fn resource_usage(&self) -> Result<FontResourceUsage, FontError> {
        self.core
            .read()
            .map_err(|_| FontError::Synchronization)?
            .budget
            .resources
            .usage()
    }

    pub(crate) fn shape_run(
        &self,
        identity: FaceIdentity,
        text: &str,
        direction: ShapingDirection,
        script: rustybuzz::Script,
    ) -> Result<ShapedRun, FontError> {
        self.shape_run_with_context(identity, text, direction, script, ShapingContext::default())
    }

    pub(crate) fn shape_run_with_context(
        &self,
        identity: FaceIdentity,
        text: &str,
        direction: ShapingDirection,
        script: rustybuzz::Script,
        context: ShapingContext<'_>,
    ) -> Result<ShapedRun, FontError> {
        self.core
            .read()
            .map_err(|_| FontError::Synchronization)?
            .shape_run_with_context(identity, text, direction, script, context)
    }

    /// Extract one planned glyph from the exact retained face bytes.
    pub(crate) fn outline_glyph(
        &self,
        identity: FaceIdentity,
        glyph_id: u16,
        builder: &mut dyn ttf_parser::OutlineBuilder,
    ) -> Result<(DesignMetrics, Option<GlyphInk>), FontError> {
        #[cfg(test)]
        FONT_OUTLINE_GLYPHS.with(|glyphs| glyphs.borrow_mut().push((identity, glyph_id)));
        self.core
            .read()
            .map_err(|_| FontError::Synchronization)?
            .outline_glyph(identity, glyph_id, builder)
    }

    pub(crate) fn face_metrics(&self, identity: FaceIdentity) -> Result<DesignMetrics, FontError> {
        self.core
            .read()
            .map_err(|_| FontError::Synchronization)?
            .face_metrics(identity)
    }

    /// Replays the retained binary sources and generic-family state exactly; it
    /// never consults the host filesystem again.
    pub(crate) fn populate_database(&self, database: &mut Database) -> Result<(), FontError> {
        *database = self
            .core
            .read()
            .map_err(|_| FontError::Synchronization)?
            .database
            .clone();
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn synthetic() -> Self {
        Self {
            core: Arc::new(RwLock::new(tests::generated_core())),
        }
    }

    /// Retain generated OpenType layout tables for shaping conformance tests.
    #[cfg(test)]
    pub(crate) fn synthetic_shaping_fonts(fonts: Vec<Vec<u8>>) -> Self {
        let files = fonts
            .into_iter()
            .map(|bytes| {
                let bytes = Arc::new(bytes);
                RetainedFontFile {
                    root_ordinal: 0,
                    canonical_path: PathBuf::from("generated-shaping.ttf"),
                    sha256: Sha256::digest(bytes.as_slice()).into(),
                    bytes,
                }
            })
            .collect();
        Self {
            core: Arc::new(RwLock::new(
                CanonicalFontCore::from_retained(
                    files,
                    &HostGenericNames::default(),
                    FontBudget::new(ResourceLimits::PRODUCTION),
                    scalar_presence_map().unwrap(),
                )
                .unwrap(),
            )),
        }
    }

    /// Real retained SFNT faces with caller-chosen family names and cmap scalars.
    #[cfg(test)]
    pub(crate) fn synthetic_faces(faces: &[(&str, &[char])]) -> Self {
        let files = faces
            .iter()
            .map(|(family, coverage)| {
                let mut coverage = coverage.to_vec();
                coverage.sort_unstable();
                coverage.dedup();
                let bytes = Arc::new(tests::build_test_font_with_coverage(family, &coverage));
                RetainedFontFile {
                    root_ordinal: 0,
                    canonical_path: PathBuf::from("retained-synthetic.ttf"),
                    sha256: Sha256::digest(bytes.as_slice()).into(),
                    bytes,
                }
            })
            .collect();
        Self {
            core: Arc::new(RwLock::new(
                CanonicalFontCore::from_retained(
                    files,
                    &HostGenericNames::default(),
                    FontBudget::new(ResourceLimits::PRODUCTION),
                    scalar_presence_map().unwrap(),
                )
                .unwrap(),
            )),
        }
    }
}

#[allow(dead_code)]
impl CanonicalFontCore {
    pub(crate) fn discover_system() -> Result<Self, FontError> {
        Self::discover_system_with_resources(
            FontBudget::new(ResourceLimits::PRODUCTION),
            scalar_presence_map()?,
            None,
            None,
        )
    }

    fn discover_system_with_resources(
        mut budget: FontBudget,
        scalar_presence: Vec<u64>,
        discovery: Option<ReservedDiscoveryRecords>,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<Self, FontError> {
        let sources = platform_sources(&mut budget, prepared_stage.as_deref_mut())?;
        let files = discover_retained_files_with_stage(
            &sources.roots,
            &mut budget,
            discovery,
            prepared_stage.as_deref_mut(),
        )?;
        Self::from_retained_with_stage(
            files,
            &sources.generics,
            budget,
            scalar_presence,
            prepared_stage,
        )
    }

    pub(crate) fn into_database(self) -> Result<Database, FontError> {
        if self.database.is_empty() {
            return Err(FontError::NoSystemFonts);
        }
        Ok(self.database)
    }

    pub(crate) fn files(&self) -> &[RetainedFontFile] {
        &self.files
    }

    pub(crate) fn faces(&self) -> &[FaceMetadata] {
        &self.faces
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one function keeps global counting, reservation, and pair-order materialization explicit"
    )]
    fn prepare_pairs_with_stage<'a>(
        &mut self,
        requests: impl ExactSizeIterator<Item = FontPairPreparationRequest<'a>>,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), FontError> {
        let pair_count = requests.len();
        let mut preparations = Vec::new();
        reserve_prepared_or_local(
            &mut preparations,
            pair_count,
            "font pair preparation storage",
            prepared_stage.as_deref_mut(),
        )?;
        reserve_prepared_or_local(
            &mut self.prepared_pairs,
            pair_count,
            "prepared font pairs",
            prepared_stage.as_deref_mut(),
        )?;

        for request in requests {
            if self.prepared_pairs_sealed {
                return Err(FontError::PreparedPairAfterSeal {
                    pair_ordinal: request.pair_ordinal,
                });
            }
            for (ordinal, stack, weight) in self
                .prepared_pairs
                .iter()
                .map(|prepared| {
                    (
                        prepared.resolved.provenance.pair_ordinal,
                        &prepared.stack,
                        prepared.weight,
                    )
                })
                .chain(
                    preparations
                        .iter()
                        .map(|preparation: &PairPreparation<'_>| {
                            (
                                preparation.request.pair_ordinal,
                                preparation.request.stack,
                                preparation.request.weight,
                            )
                        }),
                )
            {
                self.generation_resources
                    .claim_visits(1)
                    .map_err(FontError::ShapeAdmission)?;
                if ordinal == request.pair_ordinal
                    || (weight == request.weight
                        && stack.same(request.stack, &self.generation_resources)?)
                {
                    return Err(FontError::DuplicatePreparedPair {
                        pair_ordinal: request.pair_ordinal,
                    });
                }
            }
            preparations.push(PairPreparation::new(request));
        }
        if preparations.len() != pair_count {
            return Err(FontError::Arithmetic(
                "font pair preparation count/materialization mismatch",
            ));
        }

        for preparation in &mut preparations {
            reserve_prepared_or_local(
                &mut preparation.resolved_names,
                preparation.request.stack.len(),
                "resolved font stack",
                prepared_stage.as_deref_mut(),
            )?;
        }
        for preparation in &mut preparations {
            #[cfg(test)]
            FONT_PAIR_RESOLUTION_CALLS.with(|calls| calls.set(calls.get() + 1));
            resolve_stack_names_into(
                &self.database,
                preparation.request.stack,
                &mut preparation.resolved_names,
                &mut self.budget,
                prepared_stage.as_deref_mut(),
            )?;
            preparation.raw_candidate_count = count_candidate_occurrences(
                &self.faces,
                &preparation.resolved_names,
                &mut self.budget,
            )?;
            preparation.candidate_count = count_unique_candidate_survivors(
                &self.faces,
                &preparation.resolved_names,
                &mut self.budget,
            )?;
            if preparation.candidate_count == 0 {
                return Err(FontError::MissingFamily {
                    pair_ordinal: preparation.request.pair_ordinal,
                });
            }
            let (scalar_count, rendering_scalar_count) = scalar_selection_capacities(
                preparation.request.required_scalars,
                &mut self.scalar_presence,
                &mut self.budget,
                false,
            )?;
            preparation.scalar_count = scalar_count;
            preparation.maximum_probes =
                maximum_glyph_probes(rendering_scalar_count, preparation.candidate_count)?;
        }

        for preparation in &mut preparations {
            reserve_prepared_or_local(
                &mut preparation.candidates,
                preparation.raw_candidate_count,
                "font candidate occurrences",
                prepared_stage.as_deref_mut(),
            )?;
            reserve_prepared_or_local(
                &mut preparation.provenance,
                preparation.candidate_count,
                "font provenance",
                prepared_stage.as_deref_mut(),
            )?;
            reserve_prepared_or_local(
                &mut preparation.selected_scalars,
                preparation.scalar_count,
                "selected scalar faces",
                prepared_stage.as_deref_mut(),
            )?;
            reserve_prepared_or_local(
                &mut preparation.probes,
                preparation.maximum_probes,
                "font glyph probes",
                prepared_stage.as_deref_mut(),
            )?;
            self.budget
                .resources
                .claim_visits(preparation.request.stack.byte_work()? as u64)?;
            self.budget.resources.claim_bytes(
                preparation
                    .request
                    .stack
                    .len()
                    .checked_mul(size_of::<FontFamily>())
                    .ok_or(FontError::Arithmetic("prepared font stack bytes"))?
                    as u64,
            )?;
            reserve_prepared_or_local(
                &mut preparation.retained_stack,
                preparation.request.stack.len(),
                "prepared font stack",
                prepared_stage.as_deref_mut(),
            )?;
            reserve_family_stack_names(
                &mut preparation.retained_stack,
                preparation.request.stack,
                &self.budget.resources,
                prepared_stage.as_deref_mut(),
            )?;
        }
        if preparations
            .iter()
            .any(|preparation| !preparation.storage_is_reserved())
        {
            return Err(FontError::Arithmetic(
                "font pair storage reservation mismatch",
            ));
        }

        for mut preparation in preparations {
            let request = preparation.request;
            materialize_candidate_occurrences(
                &self.faces,
                &preparation.resolved_names,
                request.weight,
                &mut self.budget,
                &mut preparation.candidates,
            )?;
            if preparation.candidates.len() != preparation.raw_candidate_count {
                return Err(FontError::Arithmetic(
                    "font candidate count/materialization mismatch",
                ));
            }
            canonicalize_candidate_occurrences(&mut preparation.candidates, &mut self.budget)?;
            if preparation.candidates.len() != preparation.candidate_count {
                return Err(FontError::Arithmetic(
                    "canonical candidate count/materialization mismatch",
                ));
            }
            preparation.provenance.extend(
                preparation
                    .candidates
                    .iter()
                    .map(|candidate| candidate.occurrence.identity),
            );
            let primary = preparation.provenance[0];

            let (scalar_count, rendering_scalar_count) = scalar_selection_capacities(
                request.required_scalars,
                &mut self.scalar_presence,
                &mut self.budget,
                false,
            )?;
            let maximum_probes =
                maximum_glyph_probes(rendering_scalar_count, preparation.candidates.len())?;
            if scalar_count != preparation.scalar_count
                || maximum_probes != preparation.maximum_probes
            {
                return Err(FontError::Arithmetic(
                    "font scalar capacity/materialization mismatch",
                ));
            }
            let files = &self.files;
            let faces = &self.faces;
            materialize_scalar_selection(
                request.pair_ordinal,
                &preparation.candidates,
                request.required_scalars,
                &mut self.scalar_presence,
                &mut self.budget,
                scalar_count,
                maximum_probes,
                &mut preparation.selected_scalars,
                &mut preparation.probes,
                |candidate, scalar| {
                    #[cfg(test)]
                    FONT_SELECTION_PROBES.with(|calls| calls.set(calls.get() + 1));
                    let face = &faces[candidate.face_ordinal];
                    let file = &files[face.file_ordinal];
                    let parsed = ttf_parser::Face::parse(&file.bytes, face.identity.face_index)
                        .map_err(|error| FontError::ParseFace {
                            path: file.canonical_path.clone(),
                            index: face.identity.face_index,
                            reason: error.to_string(),
                        })?;
                    #[cfg(test)]
                    crate::orchestrator::measurements::count(
                        crate::orchestrator::measurements::Counter::FontScalarProbe,
                    );
                    Ok(parsed.glyph_index(scalar).is_some())
                },
            )?;
            fill_reserved_family_stack(&mut preparation.retained_stack, request.stack)?;
            self.prepared_pairs.push(PreparedPair {
                stack: FontStack {
                    families: {
                        self.budget.resources.claim_bytes(font_stack_arc_bytes()?)?;
                        Arc::new(preparation.retained_stack)
                    },
                },
                weight: request.weight,
                resolved: ResolvedFontPair {
                    provenance: PairProvenance {
                        pair_ordinal: request.pair_ordinal,
                        weight: request.weight,
                        candidates: preparation.provenance,
                    },
                    primary,
                    selected_scalars: SelectedScalarFaces(preparation.selected_scalars),
                    selected_units: Vec::new(),
                },
            });
        }
        Ok(())
    }

    pub(crate) fn resolve_pair(
        &mut self,
        pair_ordinal: u32,
        stack: &FontStack,
        weight: RequestedWeight,
        required_scalars: &[char],
        charge_required_scalars: bool,
    ) -> Result<ResolvedFontPair, FontError> {
        self.resolve_pair_with_stage(
            pair_ordinal,
            stack,
            weight,
            required_scalars,
            charge_required_scalars,
            None,
        )
    }

    #[expect(
        clippy::too_many_lines,
        reason = "cache admission, incremental selection, and result copying remain ordered"
    )]
    fn resolve_pair_with_stage(
        &mut self,
        pair_ordinal: u32,
        stack: &FontStack,
        weight: RequestedWeight,
        required_scalars: &[char],
        charge_required_scalars: bool,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<ResolvedFontPair, FontError> {
        self.budget.claim_visits(stack.len() as u64)?;
        let mut cached_index = None;
        for (index, cached) in self.incremental_pairs.iter().enumerate() {
            self.budget.claim_visits(1)?;
            if cached.weight == weight && cached.stack.same(stack, &self.generation_resources)? {
                cached_index = Some(index);
                break;
            }
        }
        let index = if let Some(index) = cached_index {
            index
        } else {
            #[cfg(test)]
            FONT_PAIR_RESOLUTION_CALLS.with(|calls| calls.set(calls.get() + 1));
            let candidates = canonical_candidates_with_stage(
                &self.database,
                &self.faces,
                stack,
                weight,
                &mut self.budget,
                prepared_stage.as_deref_mut(),
            )?;
            if candidates.is_empty() {
                return Err(FontError::MissingFamily { pair_ordinal });
            }
            let retained_stack = copy_stack_with_stage(
                stack,
                &self.budget.resources,
                prepared_stage.as_deref_mut(),
            )?;
            self.generation_resources
                .claim_cache_entries(1)
                .map_err(FontError::ShapeAdmission)?;
            reserve_cached_with_stage(
                &mut self.incremental_pairs,
                1,
                &self.budget.resources,
                prepared_stage.as_deref_mut(),
            )?;
            let index = self.incremental_pairs.len();
            self.incremental_pairs.push(IncrementalPair {
                stack: retained_stack,
                weight,
                candidates,
                selected_scalars: Vec::new(),
            });
            index
        };
        let cached = &mut self.incremental_pairs[index];
        let mut missing = Vec::new();
        if charge_required_scalars {
            self.budget.claim_visits(required_scalars.len() as u64)?;
        }
        for scalar in required_scalars {
            let mut found = false;
            for candidate in cached
                .selected_scalars
                .iter()
                .map(|selected| selected.scalar)
                .chain(missing.iter().copied())
            {
                self.budget.claim_visits(1)?;
                if candidate == *scalar {
                    found = true;
                    break;
                }
            }
            if !found {
                reserve_cached_with_stage(
                    &mut missing,
                    1,
                    &self.budget.resources,
                    prepared_stage.as_deref_mut(),
                )?;
                missing.push(*scalar);
            }
        }
        if !missing.is_empty() {
            let files = &self.files;
            let faces = &self.faces;
            let selected = select_scalar_faces_with_stage(
                pair_ordinal,
                &cached.candidates,
                &missing,
                &mut self.scalar_presence,
                &mut self.budget,
                false,
                prepared_stage.as_deref_mut(),
                |candidate, scalar| {
                    #[cfg(test)]
                    FONT_SELECTION_PROBES.with(|calls| calls.set(calls.get() + 1));
                    let face = &faces[candidate.face_ordinal];
                    let file = &files[face.file_ordinal];
                    let parsed = ttf_parser::Face::parse(&file.bytes, face.identity.face_index)
                        .map_err(|error| FontError::ParseFace {
                            path: file.canonical_path.clone(),
                            index: face.identity.face_index,
                            reason: error.to_string(),
                        })?;
                    #[cfg(test)]
                    crate::orchestrator::measurements::count(
                        crate::orchestrator::measurements::Counter::FontScalarProbe,
                    );
                    Ok(parsed.glyph_index(scalar).is_some())
                },
            )?;
            reserve_cached_with_stage(
                &mut cached.selected_scalars,
                selected.len(),
                &self.budget.resources,
                prepared_stage.as_deref_mut(),
            )?;
            self.generation_resources
                .claim_cache_entries(selected.len() as u128)
                .map_err(FontError::ShapeAdmission)?;
            cached.selected_scalars.extend(selected);
        }
        // Return the operation's accumulated coverage. Copies are admitted;
        // cached metadata and successful scalar selections are never recomputed.
        let mut provenance = Vec::new();
        reserve_cached_with_stage(
            &mut provenance,
            cached.candidates.len(),
            &self.budget.resources,
            prepared_stage.as_deref_mut(),
        )?;
        provenance.extend(
            cached
                .candidates
                .iter()
                .map(|candidate| candidate.occurrence.identity),
        );
        let primary = provenance[0];
        let mut selected_scalars = Vec::new();
        reserve_cached_with_stage(
            &mut selected_scalars,
            cached.selected_scalars.len(),
            &self.budget.resources,
            prepared_stage,
        )?;
        selected_scalars.extend_from_slice(&cached.selected_scalars);

        Ok(ResolvedFontPair {
            provenance: PairProvenance {
                pair_ordinal,
                weight,
                candidates: provenance,
            },
            primary,
            selected_units: Vec::new(),
            selected_scalars: SelectedScalarFaces(selected_scalars),
        })
    }

    pub(crate) fn shape_run(
        &self,
        identity: FaceIdentity,
        text: &str,
        direction: ShapingDirection,
        script: rustybuzz::Script,
    ) -> Result<ShapedRun, FontError> {
        self.shape_run_with_context(identity, text, direction, script, ShapingContext::default())
    }

    fn shape_run_with_context(
        &self,
        identity: FaceIdentity,
        text: &str,
        direction: ShapingDirection,
        script: rustybuzz::Script,
        context: ShapingContext<'_>,
    ) -> Result<ShapedRun, FontError> {
        let metadata = self.find_face(identity)?.ok_or(FontError::Shape {
            identity,
            reason: "identity is not retained",
        })?;
        let file = &self.files[metadata.file_ordinal];
        shape_retained_run(file, metadata, text, direction, script, context, self)
    }

    pub(crate) fn face_metrics(&self, identity: FaceIdentity) -> Result<DesignMetrics, FontError> {
        self.find_face(identity)?
            .map(|face| face.metrics)
            .ok_or(FontError::Shape {
                identity,
                reason: "identity is not retained",
            })
    }

    fn find_face(&self, identity: FaceIdentity) -> Result<Option<&FaceMetadata>, FontError> {
        for face in &self.faces {
            self.budget.resources.claim_visits(1)?;
            if face.identity == identity {
                return Ok(Some(face));
            }
        }
        Ok(None)
    }

    fn outline_glyph(
        &self,
        identity: FaceIdentity,
        glyph_id: u16,
        builder: &mut dyn ttf_parser::OutlineBuilder,
    ) -> Result<(DesignMetrics, Option<GlyphInk>), FontError> {
        let metadata = self.find_face(identity)?.ok_or(FontError::Outline {
            identity,
            glyph_id,
            reason: "identity is not retained",
        })?;
        let file = &self.files[metadata.file_ordinal];
        let face = ttf_parser::Face::parse(&file.bytes, identity.face_index).map_err(|_| {
            FontError::Outline {
                identity,
                glyph_id,
                reason: "ttf-parser rejected the retained face",
            }
        })?;
        let ink = face
            .outline_glyph(ttf_parser::GlyphId(glyph_id), builder)
            .map(|bounds| GlyphInk {
                x_min: bounds.x_min,
                y_min: bounds.y_min,
                x_max: bounds.x_max,
                y_max: bounds.y_max,
            });
        Ok((metadata.metrics, ink))
    }

    fn from_retained(
        files: Vec<RetainedFontFile>,
        generics: &HostGenericNames,
        budget: FontBudget,
        scalar_presence: Vec<u64>,
    ) -> Result<Self, FontError> {
        Self::from_retained_with_stage(files, generics, budget, scalar_presence, None)
    }

    fn from_retained_with_stage(
        files: Vec<RetainedFontFile>,
        generics: &HostGenericNames,
        mut budget: FontBudget,
        scalar_presence: Vec<u64>,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<Self, FontError> {
        let generation_resources = budget.resources.generation_resources();
        let mut database = Database::new();

        let face_capacity = files.iter().try_fold(0_usize, |total, file| {
            let count = ttf_parser::fonts_in_collection(&file.bytes).unwrap_or(1);
            let count =
                usize::try_from(count).map_err(|_| FontError::Arithmetic("font face count"))?;
            total
                .checked_add(count)
                .ok_or(FontError::Arithmetic("font face count"))
        })?;
        budget.claim_visits(
            u64::try_from(face_capacity).map_err(|_| FontError::Arithmetic("font face count"))?,
        )?;
        let mut faces = Vec::new();
        reserve_prepared_or_local(
            &mut faces,
            face_capacity,
            "font face metadata",
            prepared_stage.as_deref_mut(),
        )?;

        for (file_ordinal, file) in files.iter().enumerate() {
            let ids = database.load_font_source(Source::Binary(Arc::new(BinarySource(
                Arc::clone(&file.bytes),
            ))));
            for database_id in ids {
                let info = database
                    .face(database_id)
                    .ok_or_else(|| FontError::DatabaseLoad(file.canonical_path.clone()))?;
                let index = info.index;
                let parsed = ttf_parser::Face::parse(&file.bytes, index).map_err(|error| {
                    FontError::ParseFace {
                        path: file.canonical_path.clone(),
                        index,
                        reason: error.to_string(),
                    }
                })?;

                let names =
                    decode_family_names(&parsed, &mut budget, prepared_stage.as_deref_mut())?;
                let identity = FaceIdentity {
                    sha256: file.sha256,
                    face_index: index,
                };
                let metrics = design_metrics(&parsed, identity)?;
                faces.push(FaceMetadata {
                    identity,
                    file_ordinal,
                    database_id,
                    names,
                    style: info.style,
                    stretch: info.stretch,
                    weight: info.weight.0,
                    metrics,
                });
            }
        }
        apply_host_generics(&mut database, generics, &faces);

        if scalar_presence.len() != SCALAR_MAP_WORDS {
            return Err(FontError::Arithmetic("Unicode scalar presence map"));
        }

        Ok(Self {
            database,
            files,
            faces,
            scalar_presence,
            prepared_pairs: Vec::new(),
            incremental_pairs: Vec::new(),
            prepared_pairs_sealed: false,
            generation_resources,
            budget,
        })
    }
}

fn claim_text_generation_resource(
    consumed: &mut u128,
    amount: u128,
    resource: &'static str,
) -> Result<(), TextGenerationClaimError> {
    let cumulative_claim = consumed
        .checked_add(amount)
        .ok_or(TextGenerationClaimError::Arithmetic(resource))?;
    if ENFORCE_RESOURCE_QUOTAS && cumulative_claim > TEXT_GENERATION_LIMIT {
        return Err(TextGenerationClaimError::Limit {
            resource,
            limit: TEXT_GENERATION_LIMIT,
            cumulative_claim,
        });
    }
    *consumed = cumulative_claim;
    Ok(())
}

fn design_metrics(
    face: &ttf_parser::Face<'_>,
    identity: FaceIdentity,
) -> Result<DesignMetrics, FontError> {
    let ascender = face
        .typographic_ascender()
        .unwrap_or_else(|| face.ascender());
    let descender = face
        .typographic_descender()
        .unwrap_or_else(|| face.descender());
    if ascender <= 0 || descender >= 0 {
        return Err(FontError::InvalidMetrics { identity });
    }
    Ok(DesignMetrics {
        units_per_em: face.units_per_em(),
        ascender,
        descender,
    })
}

fn decode_family_names(
    face: &ttf_parser::Face<'_>,
    budget: &mut FontBudget,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<Vec<NameOccurrence>, FontError> {
    let names = face.names();
    budget.claim_visits(u64::from(names.len()))?;
    let mut occurrence_count = 0;
    for record in names {
        if matches!(
            record.name_id,
            ttf_parser::name_id::TYPOGRAPHIC_FAMILY | ttf_parser::name_id::FAMILY
        ) {
            occurrence_count += usize::from(decodes_like_ttf_parser(&record, budget)?);
        }
    }
    budget
        .resources
        .generation
        .claim_slots(occurrence_count as u128)
        .map_err(FontError::ShapeAdmission)?;
    let mut occurrences = Vec::new();
    reserve_prepared_or_local(
        &mut occurrences,
        occurrence_count,
        "font family name occurrences",
        prepared_stage,
    )?;
    budget.claim_visits(u64::from(names.len()))?;
    for (record_ordinal, record) in names.into_iter().enumerate() {
        let rank = match record.name_id {
            ttf_parser::name_id::TYPOGRAPHIC_FAMILY => NameRank::TypographicFamily,
            ttf_parser::name_id::FAMILY => NameRank::LegacyFamily,
            _ => continue,
        };
        // Admit the bounded UTF-16 decode before entering ttf-parser's
        // allocator; normalization below separately charges its own passes.
        let encoded_bytes = u64::try_from(record.name.len())
            .map_err(|_| FontError::Arithmetic("font family name input"))?;
        if record.is_unicode() {
            budget.claim_visits(
                char::decode_utf16(
                    record
                        .name
                        .chunks_exact(2)
                        .map(|unit| u16::from_be_bytes([unit[0], unit[1]])),
                )
                .count() as u64,
            )?;
        }
        budget.claim_bytes(
            encoded_bytes
                .checked_mul(3)
                .ok_or(FontError::Arithmetic("font family name decode bytes"))?,
        )?;
        let Some(value) = record.to_string() else {
            continue;
        };
        occurrences.push(NameOccurrence {
            folded: fold_family_name(&value, budget)?,
            value,
            rank,
            record_ordinal: u16::try_from(record_ordinal)
                .map_err(|_| FontError::Arithmetic("name record ordinal"))?,
        });
    }
    if occurrences.len() != occurrence_count {
        return Err(FontError::Arithmetic(
            "font family-name count/materialization mismatch",
        ));
    }
    Ok(occurrences)
}

fn decodes_like_ttf_parser(
    record: &ttf_parser::name::Name<'_>,
    budget: &mut FontBudget,
) -> Result<bool, FontError> {
    if !record.is_unicode() {
        return Ok(false);
    }
    for scalar in char::decode_utf16(
        record
            .name
            .chunks_exact(2)
            .map(|unit| u16::from_be_bytes([unit[0], unit[1]])),
    ) {
        budget.claim_visits(1)?;
        if scalar.is_err() {
            return Ok(false);
        }
    }
    Ok(true)
}

fn canonical_candidates(
    database: &Database,
    faces: &[FaceMetadata],
    stack: &FontStack,
    requested_weight: RequestedWeight,
    budget: &mut FontBudget,
) -> Result<Vec<CandidateFace>, FontError> {
    canonical_candidates_with_stage(database, faces, stack, requested_weight, budget, None)
}

fn canonical_candidates_with_stage(
    database: &Database,
    faces: &[FaceMetadata],
    stack: &FontStack,
    requested_weight: RequestedWeight,
    budget: &mut FontBudget,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<Vec<CandidateFace>, FontError> {
    let mut resolved_names = Vec::new();
    reserve_prepared_or_local(
        &mut resolved_names,
        stack.len(),
        "resolved font stack",
        prepared_stage.as_deref_mut(),
    )?;
    resolve_stack_names_into(
        database,
        stack,
        &mut resolved_names,
        budget,
        prepared_stage.as_deref_mut(),
    )?;

    let count = count_candidate_occurrences(faces, &resolved_names, budget)?;
    let mut candidates = Vec::new();
    budget.claim_bytes(
        u64::try_from(
            count
                .checked_mul(std::mem::size_of::<CandidateFace>())
                .ok_or(FontError::Arithmetic("cached candidate bytes"))?,
        )
        .map_err(|_| FontError::Arithmetic("cached candidate bytes"))?,
    )?;
    reserve_prepared_or_local(
        &mut candidates,
        count,
        "font candidate occurrences",
        prepared_stage,
    )?;
    materialize_candidate_occurrences(
        faces,
        &resolved_names,
        requested_weight,
        budget,
        &mut candidates,
    )?;
    if candidates.len() != count {
        return Err(FontError::Arithmetic(
            "font candidate count/materialization mismatch",
        ));
    }
    canonicalize_candidate_occurrences(&mut candidates, budget)?;
    Ok(candidates)
}

fn resolve_stack_names_into(
    database: &Database,
    stack: &FontStack,
    resolved_names: &mut Vec<(String, bool)>,
    budget: &mut FontBudget,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), FontError> {
    budget.claim_visits(stack.len() as u64)?;
    for cached in &budget.normalized_stacks {
        budget.resources.claim_visits(1)?;
        if cached
            .source
            .same(stack, &budget.resources.generation_resources())?
        {
            for (name, generic) in &cached.names {
                resolved_names.push((
                    copy_name_with_stage(name, &budget.resources, prepared_stage.as_deref_mut())?,
                    *generic,
                ));
            }
            return Ok(());
        }
    }
    for family in stack {
        let (name, generic) = match &family.kind {
            FontFamilyKind::Named(name) => (name.as_str(), false),
            FontFamilyKind::Generic(GenericFontFamily::Serif) => {
                (database.family_name(&Family::Serif), true)
            }
            FontFamilyKind::Generic(GenericFontFamily::SansSerif) => {
                (database.family_name(&Family::SansSerif), true)
            }
            FontFamilyKind::Generic(GenericFontFamily::Monospace) => {
                (database.family_name(&Family::Monospace), true)
            }
        };
        resolved_names.push((fold_family_name(name, budget)?, generic));
    }
    if resolved_names.len() != stack.len() {
        return Err(FontError::Arithmetic(
            "resolved font stack count/materialization mismatch",
        ));
    }
    let source = copy_stack_with_stage(stack, &budget.resources, prepared_stage.as_deref_mut())?;
    let mut names = Vec::new();
    reserve_cached_with_stage(
        &mut names,
        resolved_names.len(),
        &budget.resources,
        prepared_stage.as_deref_mut(),
    )?;
    for (name, generic) in resolved_names.iter() {
        names.push((
            copy_name_with_stage(name, &budget.resources, prepared_stage.as_deref_mut())?,
            *generic,
        ));
    }
    budget
        .resources
        .generation
        .claim_cache_entries(1)
        .map_err(FontError::ShapeAdmission)?;
    reserve_cached_with_stage(
        &mut budget.normalized_stacks,
        1,
        &budget.resources,
        prepared_stage,
    )?;
    budget
        .normalized_stacks
        .push(NormalizedStack { source, names });
    Ok(())
}

fn fold_family_name(value: &str, budget: &mut FontBudget) -> Result<String, FontError> {
    #[cfg(test)]
    FONT_NORMALIZATION_CALLS.with(|calls| calls.set(calls.get() + 1));
    super::unicode16::nfkc_casefold(value, &mut |bytes, work| {
        budget.claim_bytes(bytes)?;
        budget.claim_visits(work)
    })
}

fn reserve_cached<T>(
    values: &mut Vec<T>,
    additional: usize,
    resources: &OperationFontResources,
) -> Result<(), FontError> {
    reserve_cached_with_stage(values, additional, resources, None)
}

fn reserve_cached_with_stage<T>(
    values: &mut Vec<T>,
    additional: usize,
    resources: &OperationFontResources,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), FontError> {
    let required = values
        .len()
        .checked_add(additional)
        .ok_or(FontError::Arithmetic("font cache length"))?;
    resources
        .generation
        .claim_slots(additional as u128)
        .map_err(FontError::ShapeAdmission)?;
    if required > values.capacity() {
        let bytes = required
            .checked_mul(std::mem::size_of::<T>())
            .ok_or(FontError::Arithmetic("font cache bytes"))?;
        resources.claim_bytes(
            u64::try_from(bytes).map_err(|_| FontError::Arithmetic("font cache bytes"))?,
        )?;
        reserve_prepared_or_local(values, additional, "operation font cache", prepared_stage)?;
    }
    Ok(())
}

fn copy_name_with_stage(
    source: &str,
    resources: &OperationFontResources,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<String, FontError> {
    resources.claim_bytes(source.len() as u64)?;
    let mut output = String::new();
    reserve_string_prepared_or_local(
        &mut output,
        source.len(),
        "cached font name",
        prepared_stage,
    )?;
    output.push_str(source);
    Ok(output)
}

fn copy_stack_with_stage(
    source: &FontStack,
    resources: &OperationFontResources,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<FontStack, FontError> {
    source
        .admit_copy(resources)?
        .copy_with_stage(prepared_stage)
}

fn copy_admitted_stack_with_stage(
    admitted: AdmittedFontStackCopy<'_>,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<FontStack, FontError> {
    let AdmittedFontStackCopy { source, resources } = admitted;
    resources.claim_bytes(font_stack_arc_bytes()?)?;
    let mut output = Vec::new();
    reserve_cached_with_stage(
        &mut output,
        source.len(),
        &resources,
        prepared_stage.as_deref_mut(),
    )?;
    for entry in source {
        output.push(FontFamily {
            kind: match &entry.kind {
                FontFamilyKind::Named(name) => {
                    let mut retained = String::new();
                    resources.claim_bytes(name.len() as u64)?;
                    reserve_string_prepared_or_local(
                        &mut retained,
                        name.len(),
                        "cached font name",
                        prepared_stage.as_deref_mut(),
                    )?;
                    retained.push_str(name);
                    FontFamilyKind::Named(retained)
                }
                FontFamilyKind::Generic(family) => FontFamilyKind::Generic(*family),
            },
        });
    }
    Ok(FontStack {
        families: Arc::new(output),
    })
}

fn materialize_candidate_occurrences(
    faces: &[FaceMetadata],
    resolved_names: &[(String, bool)],
    requested_weight: RequestedWeight,
    budget: &mut FontBudget,
    candidates: &mut Vec<CandidateFace>,
) -> Result<(), FontError> {
    for (stack_entry_index, (resolved_name, generic)) in resolved_names.iter().enumerate() {
        let stack_entry_index = u32::try_from(stack_entry_index)
            .map_err(|_| FontError::Arithmetic("font stack entry index"))?;
        for (face_ordinal, face) in faces.iter().enumerate() {
            for name in &face.names {
                budget.claim_visits(1)?;
                if name.folded != *resolved_name {
                    continue;
                }
                let stretch_distance = u8::try_from(face.stretch.to_number().abs_diff(5))
                    .map_err(|_| FontError::Arithmetic("font stretch distance"))?;
                candidates.push(CandidateFace {
                    occurrence: CandidateOccurrence {
                        stack_entry_index,
                        family_match_rank: if *generic { 2 } else { name.rank as u8 },
                        style_distance: style_distance(face.style),
                        stretch_distance,
                        weight_distance: face.weight.abs_diff(requested_weight.value()),
                        identity: face.identity,
                    },
                    face_ordinal,
                });
            }
        }
    }
    Ok(())
}

fn count_candidate_occurrences(
    faces: &[FaceMetadata],
    resolved_names: &[(String, bool)],
    budget: &mut FontBudget,
) -> Result<usize, FontError> {
    let mut count = 0_usize;
    for (resolved_name, _) in resolved_names {
        for face in faces {
            for name in &face.names {
                budget.claim_visits(1)?;
                if name.folded == *resolved_name {
                    count = count
                        .checked_add(1)
                        .ok_or(FontError::Arithmetic("font candidate occurrences"))?;
                }
            }
        }
    }
    Ok(count)
}

fn count_unique_candidate_survivors(
    faces: &[FaceMetadata],
    resolved_names: &[(String, bool)],
    budget: &mut FontBudget,
) -> Result<usize, FontError> {
    let mut count = 0_usize;
    for (face_index, face) in faces.iter().enumerate() {
        budget.claim_visits(1)?;
        if !face_matches_resolved_names(face, resolved_names, budget)? {
            continue;
        }
        let mut duplicate = false;
        for earlier in &faces[..face_index] {
            budget.claim_visits(1)?;
            if earlier.identity == face.identity
                && face_matches_resolved_names(earlier, resolved_names, budget)?
            {
                duplicate = true;
                break;
            }
        }
        if duplicate {
            continue;
        }
        count = count
            .checked_add(1)
            .ok_or(FontError::Arithmetic("canonical font candidates"))?;
    }
    Ok(count)
}

fn face_matches_resolved_names(
    face: &FaceMetadata,
    resolved_names: &[(String, bool)],
    budget: &mut FontBudget,
) -> Result<bool, FontError> {
    for (resolved_name, _) in resolved_names {
        for name in &face.names {
            budget.claim_visits(1)?;
            if name.folded == *resolved_name {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn canonicalize_candidate_occurrences(
    candidates: &mut Vec<CandidateFace>,
    budget: &mut FontBudget,
) -> Result<(), FontError> {
    let input = u64::try_from(candidates.len())
        .map_err(|_| FontError::Arithmetic("candidate occurrence sort"))?;
    budget.claim_visits(input)?;
    candidates.sort_by(|left, right| {
        left.occurrence
            .identity
            .cmp(&right.occurrence.identity)
            .then_with(|| left.occurrence.canonical_cmp(&right.occurrence))
    });

    budget.claim_visits(input)?;
    candidates.dedup_by(|right, left| right.occurrence.identity == left.occurrence.identity);

    let survivors = u64::try_from(candidates.len())
        .map_err(|_| FontError::Arithmetic("canonical candidate sort"))?;
    budget.claim_visits(survivors)?;
    candidates.sort_by(|left, right| left.occurrence.canonical_cmp(&right.occurrence));
    Ok(())
}

const fn style_distance(style: Style) -> u8 {
    match style {
        Style::Normal => 0,
        Style::Oblique => 1,
        Style::Italic => 2,
    }
}

fn select_scalar_faces<F>(
    pair_ordinal: u32,
    candidates: &[CandidateFace],
    required_scalars: &[char],
    presence: &mut [u64],
    budget: &mut FontBudget,
    charge_required_scalars: bool,
    has_glyph: F,
) -> Result<Vec<SelectedScalarFace>, FontError>
where
    F: FnMut(&CandidateFace, char) -> Result<bool, FontError>,
{
    select_scalar_faces_with_stage(
        pair_ordinal,
        candidates,
        required_scalars,
        presence,
        budget,
        charge_required_scalars,
        None,
        has_glyph,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "the scalar resolver keeps the pair, resources, reservation stage, and exact probe explicit"
)]
fn select_scalar_faces_with_stage<F>(
    pair_ordinal: u32,
    candidates: &[CandidateFace],
    required_scalars: &[char],
    presence: &mut [u64],
    budget: &mut FontBudget,
    charge_required_scalars: bool,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    has_glyph: F,
) -> Result<Vec<SelectedScalarFace>, FontError>
where
    F: FnMut(&CandidateFace, char) -> Result<bool, FontError>,
{
    let (scalar_count, rendering_scalar_count) =
        scalar_selection_capacities(required_scalars, presence, budget, charge_required_scalars)?;
    let maximum_probes = maximum_glyph_probes(rendering_scalar_count, candidates.len())?;

    let mut selected = Vec::new();
    reserve_prepared_or_local(
        &mut selected,
        scalar_count,
        "selected scalar faces",
        prepared_stage.as_deref_mut(),
    )?;
    let mut probes = Vec::new();
    reserve_prepared_or_local(
        &mut probes,
        maximum_probes,
        "font glyph probes",
        prepared_stage,
    )?;
    materialize_scalar_selection(
        pair_ordinal,
        candidates,
        required_scalars,
        presence,
        budget,
        scalar_count,
        maximum_probes,
        &mut selected,
        &mut probes,
        has_glyph,
    )?;
    Ok(selected)
}

fn scalar_selection_capacities(
    required_scalars: &[char],
    presence: &mut [u64],
    budget: &mut FontBudget,
    charge_required_scalars: bool,
) -> Result<(usize, usize), FontError> {
    if presence.len() != SCALAR_MAP_WORDS {
        return Err(FontError::Arithmetic("Unicode scalar presence map"));
    }
    presence.fill(0);
    if charge_required_scalars {
        budget.claim_visits(required_scalars.len() as u64)?;
    }
    for scalar in required_scalars {
        if scalar.is_control()
            && !is_nonrendering_mandatory_break(*scalar)
            && !is_nonpainting_control(*scalar)
        {
            return Err(FontError::UnsupportedControl {
                scalar: *scalar as u32,
            });
        }
        let value = *scalar as usize;
        presence[value / u64::BITS as usize] |= 1_u64 << (value % u64::BITS as usize);
    }

    let scalar_count = presence.iter().try_fold(0_usize, |total, word| {
        total
            .checked_add(word.count_ones() as usize)
            .ok_or(FontError::Arithmetic("required scalar count"))
    })?;
    let mandatory_break_count = presence
        .iter()
        .enumerate()
        .map(|(word_index, word)| {
            let mut bits = *word;
            let mut count = 0;
            while bits != 0 {
                let value = word_index * u64::BITS as usize + bits.trailing_zeros() as usize;
                if char::from_u32(u32::try_from(value).expect("Unicode scalar map index"))
                    .is_some_and(|scalar| {
                        is_nonrendering_mandatory_break(scalar) || is_nonpainting_control(scalar)
                    })
                {
                    count += 1;
                }
                bits &= bits - 1;
            }
            count
        })
        .sum::<usize>();
    let rendering_scalar_count = scalar_count
        .checked_sub(mandatory_break_count)
        .ok_or(FontError::Arithmetic("rendering scalar count"))?;
    Ok((scalar_count, rendering_scalar_count))
}

fn maximum_glyph_probes(
    rendering_scalar_count: usize,
    candidate_count: usize,
) -> Result<usize, FontError> {
    let maximum_probes = rendering_scalar_count
        .checked_mul(candidate_count)
        .ok_or(FontError::Arithmetic("required scalar probes"))?;
    let _ = u64::try_from(maximum_probes)
        .map_err(|_| FontError::Arithmetic("required scalar probes"))?;
    Ok(maximum_probes)
}

#[expect(
    clippy::too_many_arguments,
    reason = "materialization receives every pre-reserved payload and its exact census"
)]
fn materialize_scalar_selection<F>(
    pair_ordinal: u32,
    candidates: &[CandidateFace],
    required_scalars: &[char],
    presence: &mut [u64],
    budget: &mut FontBudget,
    scalar_count: usize,
    maximum_probes: usize,
    selected: &mut Vec<SelectedScalarFace>,
    probes: &mut Vec<GlyphProbe>,
    mut has_glyph: F,
) -> Result<(), FontError>
where
    F: FnMut(&CandidateFace, char) -> Result<bool, FontError>,
{
    for scalar in required_scalars {
        let value = *scalar as usize;
        let word = &mut presence[value / u64::BITS as usize];
        let bit = 1_u64 << (value % u64::BITS as usize);
        if *word & bit == 0 {
            continue;
        }
        *word &= !bit;
        let scalar = *scalar;
        if is_nonrendering_mandatory_break(scalar) {
            selected.push(SelectedScalarFace {
                scalar,
                selection: ScalarFaceSelection::NonRenderingMandatoryBreak,
            });
            continue;
        }
        if is_nonpainting_control(scalar) {
            selected.push(SelectedScalarFace {
                scalar,
                selection: ScalarFaceSelection::Face(candidates[0].occurrence.identity),
            });
            continue;
        }

        for candidate in candidates {
            budget.claim_visits(1)?;
            probes.push(GlyphProbe {
                scalar,
                identity: candidate.occurrence.identity,
                covered: has_glyph(candidate, scalar)?,
            });
        }
        let identity = probes
            .iter()
            .rev()
            .take(candidates.len())
            .rev()
            .find(|probe| probe.scalar == scalar && probe.covered)
            .map(|probe| probe.identity)
            .ok_or(FontError::MissingGlyph {
                pair_ordinal,
                scalar: scalar as u32,
            })?;
        selected.push(SelectedScalarFace {
            scalar,
            selection: ScalarFaceSelection::Face(identity),
        });
    }
    if probes.len() != maximum_probes || selected.len() != scalar_count {
        return Err(FontError::Arithmetic(
            "glyph probe or scalar-face count/materialization mismatch",
        ));
    }
    Ok(())
}

// UAX #14 BK, CR, LF, and NL controls partition paragraphs and do not paint.
// They remain in the charged census, but a glyph probe would incorrectly make
// newline coverage a font requirement. VT/FF/LS/PS have BK and NEL has NL.
pub(crate) const fn is_nonrendering_mandatory_break(scalar: char) -> bool {
    matches!(
        scalar,
        '\u{000A}' | '\u{000B}' | '\u{000C}' | '\u{000D}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
    )
}

/// Unicode 16 Default_Ignorable_Code_Point plus bidi controls.
pub(crate) const fn is_nonpainting_control(scalar: char) -> bool {
    matches!(scalar as u32, 0x00AD | 0x034F | 0x061C | 0x115F..=0x1160
        | 0x17B4..=0x17B5 | 0x180B..=0x180F | 0x200B..=0x200F
        | 0x202A..=0x202E | 0x2060..=0x206F | 0x3164 | 0xFE00..=0xFE0F
        | 0xFEFF | 0xFFA0 | 0xFFF0..=0xFFF8 | 0x1BCA0..=0x1BCA3
        | 0x1D173..=0x1D17A | 0xE0000..=0xE0FFF)
}

/// Context is clipped to the selected line, not to a font/script item. Empty
/// pre/post context denotes a real selected-line boundary (BOT/EOT).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ShapingContext<'a> {
    pub(crate) pre: &'a str,
    pub(crate) post: &'a str,
    /// Run-local source byte of a selected U+00AD, rendered as U+002D.
    pub(crate) soft_hyphen: Option<usize>,
}

#[expect(
    clippy::too_many_lines,
    reason = "context, third-party shaping, root admission, and first-party retention stay ordered"
)]
fn shape_retained_run(
    file: &RetainedFontFile,
    metadata: &FaceMetadata,
    text: &str,
    direction: ShapingDirection,
    script: rustybuzz::Script,
    context: ShapingContext<'_>,
    core: &CanonicalFontCore,
) -> Result<ShapedRun, FontError> {
    let input = text.chars().count()
        + context.pre.chars().rev().take(5).count()
        + context.post.chars().take(5).count();
    core.generation_resources
        .claim_visits(input as u128)
        .map_err(FontError::ShapeAdmission)?;
    if text.chars().any(is_nonrendering_mandatory_break) {
        return Err(FontError::Shape {
            identity: metadata.identity,
            reason: "run crosses a mandatory paragraph break",
        });
    }
    let face = rustybuzz::Face::from_slice(&file.bytes, metadata.identity.face_index).ok_or(
        FontError::Shape {
            identity: metadata.identity,
            reason: "rustybuzz rejected the retained face",
        },
    )?;
    let mut buffer = rustybuzz::UnicodeBuffer::new();
    if let Some(replacement) = context.soft_hyphen {
        if !text
            .get(replacement..)
            .is_some_and(|suffix| suffix.starts_with('\u{ad}'))
        {
            return Err(FontError::Shape {
                identity: metadata.identity,
                reason: "invalid selected soft-hyphen source",
            });
        }
        for (index, scalar) in text.char_indices() {
            buffer.add(
                if index == replacement { '-' } else { scalar },
                u32::try_from(index)
                    .map_err(|_| FontError::Arithmetic("shaping source cluster"))?,
            );
        }
    } else {
        buffer.push_str(text);
    }
    buffer.set_pre_context(context.pre);
    buffer.set_post_context(context.post);
    let mut flags = rustybuzz::BufferFlags::empty();
    if context.pre.is_empty() {
        flags |= rustybuzz::BufferFlags::BEGINNING_OF_TEXT;
    }
    if context.post.is_empty() {
        flags |= rustybuzz::BufferFlags::END_OF_TEXT;
    }
    buffer.set_flags(flags);
    buffer.set_direction(direction.into());
    buffer.set_script(script);
    buffer.set_language(
        rustybuzz::Language::from_str("und").map_err(|_| FontError::Shape {
            identity: metadata.identity,
            reason: "invalid fixed shaping language",
        })?,
    );
    #[cfg(test)]
    FONT_SHAPE_CALLS.with(|calls| calls.set(calls.get() + 1));
    let shaped = rustybuzz::shape(&face, &[], buffer);
    let count = shaped.len() as u128;
    core.generation_resources
        .claim_slots(count)
        .map_err(FontError::ShapeAdmission)?;
    core.budget.resources.claim_bytes(
        u64::try_from(
            shaped
                .len()
                .checked_mul(size_of::<ShapedGlyph>())
                .ok_or(FontError::Arithmetic("shaped glyph bytes"))?,
        )
        .map_err(|_| FontError::Arithmetic("shaped glyph bytes"))?,
    )?;
    let mut glyphs = Vec::new();
    try_reserve_exact(&mut glyphs, shaped.len(), "shaped glyphs")?;
    for (info, position) in shaped.glyph_infos().iter().zip(shaped.glyph_positions()) {
        #[cfg(test)]
        SHAPED_GLYPH_RETAINS.with(|count| count.set(count.get() + 1));
        let glyph_id = u16::try_from(info.glyph_id).map_err(|_| FontError::Shape {
            identity: metadata.identity,
            reason: "shaper emitted an out-of-range glyph id",
        })?;
        if glyph_id == 0 && !text.is_empty() {
            return Err(FontError::Shape {
                identity: metadata.identity,
                reason: "shaper emitted the missing-glyph id",
            });
        }
        let ink = face
            .glyph_bounding_box(ttf_parser::GlyphId(glyph_id))
            .map(|rect| GlyphInk {
                x_min: rect.x_min,
                y_min: rect.y_min,
                x_max: rect.x_max,
                y_max: rect.y_max,
            });
        glyphs.push(ShapedGlyph {
            glyph_id,
            cluster: info.cluster,
            x_advance: position.x_advance,
            y_advance: position.y_advance,
            x_offset: position.x_offset,
            y_offset: position.y_offset,
            ink,
        });
    }
    Ok(ShapedRun {
        identity: metadata.identity,
        direction,
        script,
        metrics: metadata.metrics,
        glyphs,
    })
}

#[allow(clippy::too_many_lines)]
fn discover_retained_files(
    roots: &[SourceRoot],
    budget: &mut FontBudget,
) -> Result<Vec<RetainedFontFile>, FontError> {
    discover_retained_files_with_stage(roots, budget, None, None)
}

#[allow(
    clippy::too_many_lines,
    reason = "bounded discovery keeps admission, retention, and mutation checks in one auditable path"
)]
fn discover_retained_files_with_stage(
    roots: &[SourceRoot],
    budget: &mut FontBudget,
    discovery: Option<ReservedDiscoveryRecords>,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<Vec<RetainedFontFile>, FontError> {
    let capacity = usize::try_from(budget.resources.limits()?.discovery_entries)
        .map_err(|_| FontError::Arithmetic("discovery reservation"))?;
    let mut discovered = discovery.map_or_else(Vec::new, |reserved| reserved.records);
    if discovered.capacity() < capacity {
        reserve_prepared_or_local(
            &mut discovered,
            capacity,
            "discovery records",
            prepared_stage.as_deref_mut(),
        )?;
    }
    for root in roots {
        match fs::symlink_metadata(&root.path) {
            Ok(_) => {
                budget.claim_discovery(1)?;
                discovered.push(DiscoveryRecord {
                    root_ordinal: root.ordinal,
                    path: root.path.clone(),
                    entry_prefix: root.entry_prefix,
                    admit_file: false,
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(FontError::Io {
                    operation: "inspect font root",
                    path: root.path.clone(),
                    source,
                });
            }
        }
    }

    let mut cursor = 0_usize;
    while cursor < discovered.len() {
        let root_ordinal = discovered[cursor].root_ordinal;
        let path = discovered[cursor].path.clone();
        let entry_prefix = discovered[cursor].entry_prefix;
        cursor = cursor
            .checked_add(1)
            .ok_or(FontError::Arithmetic("discovery cursor"))?;
        let metadata = fs::symlink_metadata(&path).map_err(|source| FontError::Io {
            operation: "inspect discovered font entry",
            path: path.clone(),
            source,
        })?;
        let file_type = metadata.file_type();
        let is_directory = if file_type.is_symlink() {
            fs::metadata(&path)
                .map_err(|source| FontError::Io {
                    operation: "inspect discovered font target",
                    path: path.clone(),
                    source,
                })?
                .is_dir()
        } else {
            file_type.is_dir()
        };
        if is_directory {
            if file_type.is_symlink() {
                continue;
            }
            let entries = fs::read_dir(&path).map_err(|source| FontError::Io {
                operation: "read font directory",
                path: path.clone(),
                source,
            })?;
            for entry in entries {
                let entry = entry.map_err(|source| FontError::Io {
                    operation: "read font directory entry",
                    path: path.clone(),
                    source,
                })?;
                budget.claim_discovery(1)?;
                if entry_prefix
                    .is_some_and(|prefix| !entry.file_name().to_string_lossy().starts_with(prefix))
                {
                    continue;
                }
                discovered.push(DiscoveryRecord {
                    root_ordinal,
                    path: entry.path(),
                    entry_prefix: None,
                    admit_file: true,
                });
            }
        }
    }

    for record in &mut discovered {
        if !record.admit_file {
            continue;
        }
        let metadata = fs::metadata(&record.path).map_err(|source| FontError::Io {
            operation: "inspect discovered font target",
            path: record.path.clone(),
            source,
        })?;
        if !metadata.is_file() {
            record.admit_file = false;
            continue;
        }
        let canonical_path = fs::canonicalize(&record.path).map_err(|source| FontError::Io {
            operation: "canonicalize font file",
            path: record.path.clone(),
            source,
        })?;
        if !has_fontdb_extension(&canonical_path) {
            record.admit_file = false;
            continue;
        }
        record.path = canonical_path;
    }
    discovered.retain(|record| record.admit_file);
    budget.claim_visits(
        u64::try_from(discovered.len())
            .map_err(|_| FontError::Arithmetic("discovery path sort"))?,
    )?;
    discovered.sort_unstable_by(|left, right| {
        native_path_cmp(&left.path, &right.path)
            .then_with(|| left.root_ordinal.cmp(&right.root_ordinal))
    });
    budget.claim_visits(
        u64::try_from(discovered.len())
            .map_err(|_| FontError::Arithmetic("discovery path deduplication"))?,
    )?;
    discovered.dedup_by(|right, left| right.path == left.path);
    budget.claim_visits(
        u64::try_from(discovered.len())
            .map_err(|_| FontError::Arithmetic("discovery root sort"))?,
    )?;
    discovered.sort_unstable_by(|left, right| {
        left.root_ordinal
            .cmp(&right.root_ordinal)
            .then_with(|| native_path_cmp(&left.path, &right.path))
    });

    let mut retained = Vec::new();
    reserve_prepared_or_local(
        &mut retained,
        discovered.len(),
        "retained font files",
        prepared_stage.as_deref_mut(),
    )?;
    for record in discovered {
        let canonical_path = record.path;
        let file = File::open(&canonical_path).map_err(|source| FontError::Io {
            operation: "open font file",
            path: canonical_path.clone(),
            source,
        })?;
        let mut handle = DiscoveryFontHandle { file };
        #[cfg(test)]
        DISCOVERY_HANDLES.with(|state| {
            let mut counts = state.get();
            counts[0] += 1;
            counts[1] = counts[1].max(counts[0]);
            counts[2] += 1;
            state.set(counts);
        });
        let metadata_before = handle.file.metadata().map_err(|source| FontError::Io {
            operation: "inspect font file",
            path: canonical_path.clone(),
            source,
        })?;
        if !metadata_before.is_file() {
            return Err(FontError::ChangedFile(canonical_path));
        }
        let length = metadata_before.len();
        let modified_before = metadata_before.modified().ok();
        budget.claim_bytes(length)?;
        let length =
            usize::try_from(length).map_err(|_| FontError::Arithmetic("font file length"))?;
        let mut bytes = Vec::new();
        reserve_prepared_or_local(
            &mut bytes,
            length,
            "retained font bytes",
            prepared_stage.as_deref_mut(),
        )?;
        #[cfg(test)]
        DISCOVERY_AFTER_ADMISSION.with(|hook| {
            if let Some(hook) = hook.take() {
                hook(&canonical_path);
            }
        });
        #[cfg(test)]
        DISCOVERY_HANDLES.with(|state| {
            let mut counts = state.get();
            counts[3] += 1;
            state.set(counts);
        });
        let sha256 = read_exact_font_bytes(&mut handle.file, &canonical_path, length, &mut bytes)?;
        let metadata_after = handle.file.metadata().map_err(|source| FontError::Io {
            operation: "reinspect font file",
            path: canonical_path.clone(),
            source,
        })?;
        if metadata_after.len()
            != u64::try_from(length).map_err(|_| FontError::Arithmetic("font file length"))?
            || modified_before.is_some_and(|modified| {
                metadata_after
                    .modified()
                    .map_or(true, |current| current != modified)
            })
        {
            return Err(FontError::ChangedFile(canonical_path));
        }
        retained.push(RetainedFontFile {
            root_ordinal: record.root_ordinal,
            canonical_path,
            bytes: Arc::new(bytes),
            sha256,
        });
        #[cfg(test)]
        DISCOVERY_HANDLES.with(|state| {
            let mut counts = state.get();
            counts[4] += 1;
            state.set(counts);
        });
        drop(handle);
    }
    Ok(retained)
}

fn read_exact_font_bytes(
    reader: &mut impl Read,
    path: &Path,
    expected_length: usize,
    bytes: &mut Vec<u8>,
) -> Result<[u8; 32], FontError> {
    let mut hasher = Sha256::new();
    let mut scratch = [0_u8; 16 * 1024];
    while bytes.len() < expected_length {
        let remaining = expected_length - bytes.len();
        let read_length = remaining.min(scratch.len());
        let read = reader
            .read(&mut scratch[..read_length])
            .map_err(|source| FontError::Io {
                operation: "read font file",
                path: path.to_path_buf(),
                source,
            })?;
        if read == 0 {
            return Err(FontError::ChangedFile(path.to_path_buf()));
        }
        hasher.update(&scratch[..read]);
        bytes.extend_from_slice(&scratch[..read]);
    }
    Ok(hasher.finalize().into())
}

fn has_fontdb_extension(path: &Path) -> bool {
    matches!(
        path.extension().and_then(std::ffi::OsStr::to_str),
        Some("ttf" | "ttc" | "TTF" | "TTC" | "otf" | "otc" | "OTF" | "OTC")
    )
}

#[cfg(unix)]
fn native_path_cmp(left: &Path, right: &Path) -> Ordering {
    use std::os::unix::ffi::OsStrExt;
    left.as_os_str()
        .as_bytes()
        .cmp(right.as_os_str().as_bytes())
}

#[cfg(windows)]
fn native_path_cmp(left: &Path, right: &Path) -> Ordering {
    use std::os::windows::ffi::OsStrExt;
    left.as_os_str()
        .encode_wide()
        .cmp(right.as_os_str().encode_wide())
}

#[cfg(not(any(unix, windows)))]
fn native_path_cmp(left: &Path, right: &Path) -> Ordering {
    left.as_os_str().cmp(right.as_os_str())
}

fn try_reserve_exact<T>(
    values: &mut Vec<T>,
    additional: usize,
    resource: &'static str,
) -> Result<(), FontError> {
    values
        .try_reserve_exact(additional)
        .map_err(|_| FontError::Allocation(resource))
}

fn reserve_prepared_or_local<T>(
    values: &mut Vec<T>,
    additional: usize,
    resource: &'static str,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), FontError> {
    if let Some(stage) = prepared_stage {
        stage
            .reserve_prepared(values, additional, resource)
            .map_err(|error| FontError::Reservation(error.to_string()))
    } else {
        try_reserve_exact(values, additional, resource)
    }
}

fn reserve_string_prepared_or_local(
    value: &mut String,
    additional: usize,
    resource: &'static str,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), FontError> {
    if let Some(stage) = prepared_stage {
        stage
            .reserve_prepared_string(value, additional, resource)
            .map_err(|error| FontError::Reservation(error.to_string()))
    } else {
        value
            .try_reserve_exact(additional)
            .map_err(|_| FontError::Allocation(resource))
    }
}

fn reserve_family_stack_names(
    retained: &mut Vec<FontFamily>,
    source: &FontStack,
    resources: &OperationFontResources,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), FontError> {
    for preference in source {
        retained.push(FontFamily {
            kind: match &preference.kind {
                FontFamilyKind::Named(name) => {
                    resources.claim_bytes(name.len() as u64)?;
                    let mut reserved = String::new();
                    if let Some(stage) = prepared_stage.as_deref_mut() {
                        stage
                            .reserve_prepared_string(
                                &mut reserved,
                                name.len(),
                                "prepared font family name",
                            )
                            .map_err(|error| FontError::Reservation(error.to_string()))?;
                    } else {
                        reserved
                            .try_reserve_exact(name.len())
                            .map_err(|_| FontError::Allocation("prepared font family name"))?;
                    }
                    FontFamilyKind::Named(reserved)
                }
                FontFamilyKind::Generic(family) => FontFamilyKind::Generic(*family),
            },
        });
    }
    Ok(())
}

fn fill_reserved_family_stack(
    retained: &mut [FontFamily],
    source: &FontStack,
) -> Result<(), FontError> {
    if retained.len() != source.len() {
        return Err(FontError::Arithmetic(
            "prepared font stack count/materialization mismatch",
        ));
    }
    for (retained, source) in retained.iter_mut().zip(source) {
        match (&mut retained.kind, &source.kind) {
            (FontFamilyKind::Named(retained), FontFamilyKind::Named(source)) => {
                if retained.capacity() < source.len() || !retained.is_empty() {
                    return Err(FontError::Arithmetic(
                        "prepared font family reservation mismatch",
                    ));
                }
                retained.push_str(source);
            }
            (FontFamilyKind::Generic(retained), FontFamilyKind::Generic(source))
                if retained == source => {}
            _ => {
                return Err(FontError::Arithmetic(
                    "prepared font stack materialization mismatch",
                ));
            }
        }
    }
    Ok(())
}

fn scalar_presence_map() -> Result<Vec<u64>, FontError> {
    let mut presence = Vec::new();
    try_reserve_exact(
        &mut presence,
        SCALAR_MAP_WORDS,
        "Unicode scalar presence map",
    )?;
    presence.resize(SCALAR_MAP_WORDS, 0_u64);
    Ok(presence)
}

fn apply_host_generics(
    database: &mut Database,
    generics: &HostGenericNames,
    _faces: &[FaceMetadata],
) {
    if let Some(name) = &generics.serif {
        database.set_serif_family(name);
    }
    if let Some(name) = &generics.sans_serif {
        database.set_sans_serif_family(name);
    }
    if let Some(name) = &generics.cursive {
        database.set_cursive_family(name);
    }
    if let Some(name) = &generics.fantasy {
        database.set_fantasy_family(name);
    }
    if let Some(name) = &generics.monospace {
        database.set_monospace_family(name);
    }
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
#[derive(Default)]
struct BoundedFontConfig {
    dirs: Vec<fontconfig_parser::DirData>,
    aliases: Vec<fontconfig_parser::Alias>,
    files: Vec<PathBuf>,
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
enum FontConfigTask {
    Parts {
        config_path: PathBuf,
        parts: std::vec::IntoIter<FontConfigDirective>,
    },
    Merge {
        path: PathBuf,
        admitted: bool,
    },
    Include(PathBuf),
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
enum FontConfigRead {
    Loaded {
        parts: Vec<FontConfigDirective>,
        complete: bool,
    },
    Ignored,
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
enum FontConfigDirective {
    Alias(fontconfig_parser::Alias),
    Dir(fontconfig_parser::Dir),
    Include(fontconfig_parser::Include),
    ResetDirs,
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
#[expect(
    clippy::too_many_lines,
    reason = "the iterative fontconfig traversal keeps include and directive order explicit"
)]
fn merge_fontconfig_root(
    config: &mut BoundedFontConfig,
    config_path: &Path,
    budget: &mut FontBudget,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<bool, FontError> {
    let FontConfigRead::Loaded { parts, complete } = read_bounded_fontconfig(
        config,
        config_path,
        false,
        budget,
        prepared_stage.as_deref_mut(),
    )?
    else {
        return Ok(false);
    };

    let mut tasks = Vec::new();
    reserve_prepared_or_local(
        &mut tasks,
        1,
        "fontconfig traversal",
        prepared_stage.as_deref_mut(),
    )?;
    tasks.push(FontConfigTask::Parts {
        config_path: config_path.to_path_buf(),
        parts: parts.into_iter(),
    });

    while let Some(task) = tasks.pop() {
        match task {
            FontConfigTask::Parts {
                config_path,
                mut parts,
            } => {
                let Some(part) = parts.next() else { continue };
                reserve_prepared_or_local(
                    &mut tasks,
                    1,
                    "fontconfig traversal",
                    prepared_stage.as_deref_mut(),
                )?;
                tasks.push(FontConfigTask::Parts {
                    config_path: config_path.clone(),
                    parts,
                });
                match part {
                    FontConfigDirective::Alias(alias) => {
                        reserve_prepared_or_local(
                            &mut config.aliases,
                            1,
                            "fontconfig aliases",
                            prepared_stage.as_deref_mut(),
                        )?;
                        config.aliases.push(alias);
                    }
                    FontConfigDirective::Dir(dir) => {
                        reserve_prepared_or_local(
                            &mut config.dirs,
                            1,
                            "fontconfig directories",
                            prepared_stage.as_deref_mut(),
                        )?;
                        config.dirs.push(fontconfig_parser::DirData {
                            path: dir.calculate_path(&config_path),
                            salt: dir.salt,
                        });
                    }
                    FontConfigDirective::Include(include) => {
                        reserve_prepared_or_local(
                            &mut tasks,
                            1,
                            "fontconfig traversal",
                            prepared_stage.as_deref_mut(),
                        )?;
                        tasks.push(FontConfigTask::Include(
                            include.calculate_path(&config_path),
                        ));
                    }
                    FontConfigDirective::ResetDirs => config.dirs.clear(),
                }
            }
            FontConfigTask::Merge { path, admitted } => {
                // Upstream ignores included syntax failures, but keeps their valid prefix.
                if let FontConfigRead::Loaded { parts, .. } = read_bounded_fontconfig(
                    config,
                    &path,
                    admitted,
                    budget,
                    prepared_stage.as_deref_mut(),
                )? {
                    reserve_prepared_or_local(
                        &mut tasks,
                        1,
                        "fontconfig traversal",
                        prepared_stage.as_deref_mut(),
                    )?;
                    tasks.push(FontConfigTask::Parts {
                        config_path: path,
                        parts: parts.into_iter(),
                    });
                }
            }
            FontConfigTask::Include(path) => {
                push_bounded_fontconfig_include(
                    &mut tasks,
                    &path,
                    budget,
                    prepared_stage.as_deref_mut(),
                )?;
            }
        }
    }
    Ok(complete)
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
fn read_bounded_fontconfig(
    config: &mut BoundedFontConfig,
    config_path: &Path,
    admitted: bool,
    budget: &mut FontBudget,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<FontConfigRead, FontError> {
    let canonical_path = match fs::canonicalize(config_path) {
        Ok(path) => path,
        Err(source) if !admitted && source.kind() == std::io::ErrorKind::NotFound => {
            return Ok(FontConfigRead::Ignored);
        }
        Err(source) => {
            return Err(FontError::Io {
                operation: "canonicalize fontconfig file",
                path: config_path.to_path_buf(),
                source,
            });
        }
    };
    if !admitted {
        budget.claim_discovery(1)?;
    }
    for path in &config.files {
        budget.claim_visits(1)?;
        if path == &canonical_path {
            return Ok(FontConfigRead::Loaded {
                parts: Vec::new(),
                complete: true,
            });
        }
    }
    reserve_prepared_or_local(
        &mut config.files,
        1,
        "fontconfig files",
        prepared_stage.as_deref_mut(),
    )?;
    config.files.push(canonical_path.clone());
    let mut file = File::open(&canonical_path).map_err(|source| FontError::Io {
        operation: "open admitted fontconfig file",
        path: canonical_path.clone(),
        source,
    })?;
    let metadata_before = file.metadata().map_err(|source| FontError::Io {
        operation: "inspect admitted fontconfig file",
        path: canonical_path.clone(),
        source,
    })?;
    if !metadata_before.is_file() {
        return Err(FontError::ChangedFile(canonical_path));
    }
    let length = metadata_before.len();
    let modified_before = metadata_before.modified().ok();
    budget.claim_bytes(length)?;
    let length =
        usize::try_from(length).map_err(|_| FontError::Arithmetic("fontconfig file length"))?;

    let mut source = String::new();
    reserve_string_prepared_or_local(
        &mut source,
        length,
        "fontconfig source bytes",
        prepared_stage.as_deref_mut(),
    )?;
    let read = file
        .by_ref()
        .take(u64::try_from(length).map_err(|_| FontError::Arithmetic("fontconfig file length"))?)
        .read_to_string(&mut source)
        .map_err(|source| FontError::Io {
            operation: "read fontconfig file",
            path: canonical_path.clone(),
            source,
        })?;
    let metadata_after = file.metadata().map_err(|source| FontError::Io {
        operation: "reinspect fontconfig file",
        path: canonical_path.clone(),
        source,
    })?;
    if read != length
        || metadata_after.len()
            != u64::try_from(length).map_err(|_| FontError::Arithmetic("fontconfig file length"))?
        || modified_before.is_some_and(|modified| {
            metadata_after
                .modified()
                .map_or(true, |current| current != modified)
        })
    {
        return Err(FontError::ChangedFile(canonical_path));
    }

    drop(file);
    match parse_bounded_fontconfig(&source, config_path, budget, prepared_stage) {
        Ok((parts, complete)) => Ok(FontConfigRead::Loaded { parts, complete }),
        Err(FontError::ParseFontConfig { .. }) => Ok(FontConfigRead::Ignored),
        Err(error) => Err(error),
    }
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
fn parse_bounded_fontconfig(
    source: &str,
    config_path: &Path,
    budget: &mut FontBudget,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(Vec<FontConfigDirective>, bool), FontError> {
    if source.contains("<!ENTITY") {
        return Err(fontconfig_parse_error(
            config_path,
            "internal entities are not admitted",
        ));
    }
    let markup = u64::try_from(source.bytes().filter(|byte| *byte == b'<').count())
        .map_err(|_| FontError::Arithmetic("fontconfig XML node capacity"))?;
    let node_capacity = markup
        .checked_mul(2)
        .and_then(|capacity| capacity.checked_add(1))
        .ok_or(FontError::Arithmetic("fontconfig XML node capacity"))?;
    budget.claim_visits(source.chars().count() as u64)?;
    let nodes_limit = u32::try_from(node_capacity)
        .map_err(|_| FontError::Arithmetic("fontconfig XML node capacity"))?;
    let document = roxmltree::Document::parse_with_options(
        source,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            nodes_limit,
        },
    )
    .map_err(|error| fontconfig_parse_error(config_path, error.to_string()))?;
    let root = document.root_element();
    if root.tag_name().name() != "fontconfig" {
        return Err(fontconfig_parse_error(
            config_path,
            "missing fontconfig root element",
        ));
    }
    let directive_count = root
        .children()
        .filter(|child| {
            child.is_element()
                && matches!(
                    child.tag_name().name(),
                    "alias" | "dir" | "include" | "reset-dirs"
                )
        })
        .count();
    budget.claim_discovery(
        u64::try_from(directive_count)
            .map_err(|_| FontError::Arithmetic("fontconfig directives"))?,
    )?;
    let mut directives = Vec::new();
    reserve_prepared_or_local(
        &mut directives,
        directive_count,
        "fontconfig directives",
        prepared_stage.as_deref_mut(),
    )?;
    for child in root.children().filter(roxmltree::Node::is_element) {
        let directive = match child.tag_name().name() {
            "alias" => parse_fontconfig_alias(child, config_path, prepared_stage.as_deref_mut())
                .map(FontConfigDirective::Alias)
                .map(Some),
            "dir" => parse_fontconfig_dir(child, config_path, prepared_stage.as_deref_mut())
                .map(FontConfigDirective::Dir)
                .map(Some),
            "include" => {
                parse_fontconfig_include(child, config_path, prepared_stage.as_deref_mut())
                    .map(FontConfigDirective::Include)
                    .map(Some)
            }
            "reset-dirs" => Ok(Some(FontConfigDirective::ResetDirs)),
            _ => Ok(None),
        };
        match directive {
            Ok(Some(directive)) => directives.push(directive),
            Ok(None) => {}
            Err(FontError::ParseFontConfig { .. }) => return Ok((directives, false)),
            Err(error) => return Err(error),
        }
    }
    Ok((directives, true))
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
fn parse_fontconfig_alias(
    node: roxmltree::Node<'_, '_>,
    config_path: &Path,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<fontconfig_parser::Alias, FontError> {
    let mut alias = fontconfig_parser::Alias::default();
    for child in node.children().filter(roxmltree::Node::is_element) {
        match child.tag_name().name() {
            "family" => {
                alias.alias = copy_fontconfig_string(
                    fontconfig_node_text(child, config_path)?,
                    "fontconfig alias family",
                    prepared_stage.as_deref_mut(),
                )?;
            }
            "prefer" => {
                copy_fontconfig_families(child, &mut alias.prefer, prepared_stage.as_deref_mut())?;
            }
            "accept" => {
                copy_fontconfig_families(child, &mut alias.accept, prepared_stage.as_deref_mut())?;
            }
            "default" => {
                copy_fontconfig_families(child, &mut alias.default, prepared_stage.as_deref_mut())?;
            }
            _ => {}
        }
    }
    Ok(alias)
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
fn copy_fontconfig_families(
    node: roxmltree::Node<'_, '_>,
    destination: &mut Vec<String>,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), FontError> {
    let count = node
        .children()
        .filter(|child| child.is_element() && child.tag_name().name() == "family")
        .count();
    reserve_prepared_or_local(
        destination,
        count,
        "fontconfig alias preferences",
        prepared_stage.as_deref_mut(),
    )?;
    for family in node
        .children()
        .filter(|child| child.is_element() && child.tag_name().name() == "family")
    {
        let Some(text) = family.text() else { continue };
        destination.push(copy_fontconfig_string(
            text,
            "fontconfig alias preference",
            prepared_stage.as_deref_mut(),
        )?);
    }
    Ok(())
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
fn parse_fontconfig_dir(
    node: roxmltree::Node<'_, '_>,
    config_path: &Path,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<fontconfig_parser::Dir, FontError> {
    let mut dir = fontconfig_parser::Dir::default();
    if let Some(prefix) = node.attribute("prefix") {
        dir.prefix = prefix.parse().map_err(|error: fontconfig_parser::Error| {
            fontconfig_parse_error(config_path, error.to_string())
        })?;
    }
    if let Some(salt) = node.attribute("salt") {
        dir.salt = copy_fontconfig_string(
            salt,
            "fontconfig directory salt",
            prepared_stage.as_deref_mut(),
        )?;
    }
    dir.path = copy_fontconfig_string(
        fontconfig_node_text(node, config_path)?,
        "fontconfig directory path",
        prepared_stage,
    )?;
    Ok(dir)
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
fn parse_fontconfig_include(
    node: roxmltree::Node<'_, '_>,
    config_path: &Path,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<fontconfig_parser::Include, FontError> {
    let mut include = fontconfig_parser::Include::default();
    if let Some(prefix) = node.attribute("prefix") {
        include.prefix = prefix.parse().map_err(|error: fontconfig_parser::Error| {
            fontconfig_parse_error(config_path, error.to_string())
        })?;
    }
    include.ignore_missing = node.attribute("ignore_missing") == Some("yes");
    include.path = copy_fontconfig_string(
        fontconfig_node_text(node, config_path)?,
        "fontconfig include path",
        prepared_stage,
    )?;
    Ok(include)
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
fn fontconfig_node_text<'a>(
    node: roxmltree::Node<'a, '_>,
    config_path: &Path,
) -> Result<&'a str, FontError> {
    node.text()
        .ok_or_else(|| fontconfig_parse_error(config_path, "missing directive text"))
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
fn copy_fontconfig_string(
    source: &str,
    resource: &'static str,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<String, FontError> {
    let mut retained = String::new();
    reserve_string_prepared_or_local(&mut retained, source.len(), resource, prepared_stage)?;
    retained.push_str(source);
    Ok(retained)
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
fn fontconfig_parse_error(config_path: &Path, reason: impl Into<String>) -> FontError {
    FontError::ParseFontConfig {
        path: config_path.to_path_buf(),
        reason: reason.into(),
    }
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
fn push_bounded_fontconfig_include(
    tasks: &mut Vec<FontConfigTask>,
    include_path: &Path,
    budget: &mut FontBudget,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), FontError> {
    let metadata = match fs::metadata(include_path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(source) => {
            return Err(FontError::Io {
                operation: "inspect fontconfig include",
                path: include_path.to_path_buf(),
                source,
            });
        }
    };
    if metadata.is_file() {
        budget.claim_discovery(1)?;
        reserve_prepared_or_local(tasks, 1, "fontconfig traversal", prepared_stage)?;
        tasks.push(FontConfigTask::Merge {
            path: include_path.to_path_buf(),
            admitted: true,
        });
        return Ok(());
    }
    if !metadata.is_dir() {
        return Ok(());
    }
    budget.claim_discovery(1)?;
    let entries = fs::read_dir(include_path).map_err(|source| FontError::Io {
        operation: "read fontconfig include directory",
        path: include_path.to_path_buf(),
        source,
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| FontError::Io {
            operation: "read fontconfig include entry",
            path: include_path.to_path_buf(),
            source,
        })?;
        budget.claim_discovery(1)?;
        let file_type = entry.file_type().map_err(|source| FontError::Io {
            operation: "inspect fontconfig include entry",
            path: entry.path(),
            source,
        })?;
        if !file_type.is_file() && !file_type.is_symlink() {
            continue;
        }
        reserve_prepared_or_local(
            &mut paths,
            1,
            "fontconfig include entries",
            prepared_stage.as_deref_mut(),
        )?;
        paths.push(entry.path());
    }
    budget.claim_visits(
        u64::try_from(paths.len())
            .map_err(|_| FontError::Arithmetic("fontconfig include path sort"))?,
    )?;
    paths.sort_unstable_by(|left, right| native_path_cmp(left, right));
    reserve_prepared_or_local(tasks, paths.len(), "fontconfig traversal", prepared_stage)?;
    for path in paths.into_iter().rev() {
        tasks.push(FontConfigTask::Merge {
            path,
            admitted: true,
        });
    }
    Ok(())
}

#[cfg(target_os = "windows")]
#[expect(
    clippy::too_many_lines,
    reason = "platform source ordering and generic-family extraction form one locked fontdb contract"
)]
fn platform_sources(
    _budget: &mut FontBudget,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<HostSources, FontError> {
    let mut paths = Vec::new();
    if let Some(system_root) = std::env::var_os("SYSTEMROOT") {
        paths.push(PathBuf::from(system_root).join("Fonts"));
    } else {
        paths.push(PathBuf::from(r"C:\Windows\Fonts\"));
    }
    if let Some(home) = std::env::var_os("USERPROFILE") {
        let home = PathBuf::from(home);
        paths.push(home.join(r"AppData\Local\Microsoft\Windows\Fonts"));
        paths.push(home.join(r"AppData\Roaming\Microsoft\Windows\Fonts"));
    }
    Ok(HostSources {
        roots: ordinal_roots(paths, prepared_stage)?,
        generics: HostGenericNames::default(),
    })
}

#[cfg(target_os = "macos")]
fn platform_sources(
    _budget: &mut FontBudget,
    _prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<HostSources, FontError> {
    let mut roots = vec![
        SourceRoot {
            ordinal: 0,
            path: PathBuf::from("/Library/Fonts"),
            entry_prefix: None,
        },
        SourceRoot {
            ordinal: 1,
            path: PathBuf::from("/System/Library/Fonts"),
            entry_prefix: None,
        },
        SourceRoot {
            ordinal: 2,
            path: PathBuf::from("/System/Library/AssetsV2"),
            entry_prefix: Some("com_apple_MobileAsset_Font"),
        },
        SourceRoot {
            ordinal: 3,
            path: PathBuf::from("/Network/Library/Fonts"),
            entry_prefix: None,
        },
    ];
    if let Some(home) = std::env::var_os("HOME") {
        roots.push(SourceRoot {
            ordinal: 4,
            path: PathBuf::from(home).join("Library/Fonts"),
            entry_prefix: None,
        });
    }
    Ok(HostSources {
        roots,
        generics: HostGenericNames::default(),
    })
}

#[cfg(target_os = "redox")]
fn platform_sources(
    _budget: &mut FontBudget,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<HostSources, FontError> {
    Ok(HostSources {
        roots: ordinal_roots(vec![PathBuf::from("/ui/fonts")], prepared_stage)?,
        generics: HostGenericNames::default(),
    })
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "redox"))
))]
#[expect(
    clippy::too_many_lines,
    reason = "platform source ordering and generic-family extraction form one locked fontdb contract"
)]
fn platform_sources(
    budget: &mut FontBudget,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<HostSources, FontError> {
    let home = std::env::var("HOME").ok();
    let mut config = BoundedFontConfig::default();
    if let Ok(config_file) = std::env::var("FONTCONFIG_FILE") {
        merge_fontconfig_root(
            &mut config,
            Path::new(&config_file),
            budget,
            prepared_stage.as_deref_mut(),
        )?;
    } else {
        let xdg_config_home = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| home.as_ref().map(|home| Path::new(home).join(".config")));
        let read_global = if let Some(path) = xdg_config_home {
            !merge_fontconfig_root(
                &mut config,
                &path.join("fontconfig/fonts.conf"),
                budget,
                prepared_stage.as_deref_mut(),
            )?
        } else {
            true
        };
        if read_global {
            merge_fontconfig_root(
                &mut config,
                Path::new("/etc/fonts/local.conf"),
                budget,
                prepared_stage.as_deref_mut(),
            )?;
        }
        merge_fontconfig_root(
            &mut config,
            Path::new("/etc/fonts/fonts.conf"),
            budget,
            prepared_stage.as_deref_mut(),
        )?;
    }

    let mut generics = HostGenericNames::default();
    for alias in &config.aliases {
        let name = alias
            .prefer
            .first()
            .or_else(|| alias.accept.first())
            .or_else(|| alias.default.first())
            .cloned();
        let Some(name) = name else { continue };
        match alias.alias.to_lowercase().as_str() {
            "serif" => generics.serif = Some(name),
            "sans-serif" | "sans serif" => generics.sans_serif = Some(name),
            "monospace" => generics.monospace = Some(name),
            "cursive" => generics.cursive = Some(name),
            "fantasy" => generics.fantasy = Some(name),
            _ => {}
        }
    }

    let paths = if config.dirs.is_empty() {
        let mut paths = vec![
            PathBuf::from("/usr/share/fonts/"),
            PathBuf::from("/usr/local/share/fonts/"),
        ];
        if let Some(home) = home {
            paths.push(Path::new(&home).join(".fonts"));
            paths.push(Path::new(&home).join(".local/share/fonts"));
        }
        paths
    } else {
        let path_count = config
            .dirs
            .iter()
            .filter(|directory| !directory.path.starts_with("~") || home.is_some())
            .count();
        let mut paths = Vec::new();
        reserve_prepared_or_local(
            &mut paths,
            path_count,
            "fontconfig font roots",
            prepared_stage.as_deref_mut(),
        )?;
        for directory in config.dirs {
            if directory.path.starts_with("~") {
                let Some(home) = &home else { continue };
                paths.push(
                    Path::new(home).join(
                        directory
                            .path
                            .strip_prefix("~")
                            .expect("tilde-prefixed font path has a tilde component"),
                    ),
                );
            } else {
                paths.push(directory.path);
            }
        }
        paths
    };
    Ok(HostSources {
        roots: ordinal_roots(paths, prepared_stage)?,
        generics,
    })
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "redox",
    all(unix, not(target_os = "android"))
)))]
fn platform_sources(
    _budget: &mut FontBudget,
    _prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<HostSources, FontError> {
    Ok(HostSources {
        roots: Vec::new(),
        generics: HostGenericNames::default(),
    })
}

fn ordinal_roots(
    paths: Vec<PathBuf>,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<Vec<SourceRoot>, FontError> {
    let mut roots = Vec::new();
    reserve_prepared_or_local(
        &mut roots,
        paths.len(),
        "canonical font source roots",
        prepared_stage,
    )?;
    for (ordinal, path) in paths.into_iter().enumerate() {
        roots.push(SourceRoot {
            ordinal: u32::try_from(ordinal)
                .map_err(|_| FontError::Arithmetic("canonical font source root ordinal"))?,
            path,
            entry_prefix: None,
        });
    }
    Ok(roots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_stack_comparisons_charge_the_terminating_byte_and_entry() {
        let left = FontStack::new([
            FontFamily::named("αZ").unwrap(),
            FontFamily::generic(GenericFontFamily::Serif),
        ])
        .unwrap();
        let mismatch = FontStack::new([
            FontFamily::named("αY").unwrap(),
            FontFamily::generic(GenericFontFamily::Monospace),
        ])
        .unwrap();
        // Stack length, first entry, name length, and three examined UTF-8 bytes.
        for remaining in [0, 5, 6] {
            let resources = OperationGenerationResources::with_test_limits(remaining, 0);
            let result = left.same(&mismatch, &resources);
            if remaining == 6 {
                assert!(!result.unwrap());
            } else {
                assert!(result.is_err());
            }
            assert_eq!(resources.usage().unwrap().1, remaining);
            assert_eq!(resources.byte_usage().unwrap(), 0);
        }
        // Equal names also examine the terminating generic entry.
        let independent = FontStack::new([
            FontFamily::named("αZ").unwrap(),
            FontFamily::generic(GenericFontFamily::Serif),
        ])
        .unwrap();
        for remaining in [6, 7] {
            let resources = OperationGenerationResources::with_test_limits(remaining, 0);
            let result = left.same(&independent, &resources);
            assert_eq!(result.is_ok(), remaining == 7);
            assert_eq!(resources.usage().unwrap().1, remaining);
        }
    }

    #[test]
    fn css_admits_known_utf8_passes_without_an_unpriced_scalar_prescan() {
        let stack = named_stack(&["α, A"]);
        // Each count/materialization pass visits one entry and five source bytes.
        for remaining in [0, 11, 12] {
            let resources = OperationFontResources::new();
            resources
                .generation_resources()
                .set_test_limits(remaining, 536_870_912);
            let result = stack.css(&resources);
            if remaining == 12 {
                assert_eq!(result.unwrap(), "\"α, A\"");
                assert_eq!(resources.usage().unwrap().processing_visits, 12);
                assert_eq!(resources.usage().unwrap().font_bytes, 7);
            } else {
                assert!(result.is_err());
                assert_eq!(resources.usage().unwrap().processing_visits, 0);
                assert_eq!(resources.usage().unwrap().font_bytes, 0);
            }
        }
    }

    #[test]
    fn prepared_font_pair_hit_admits_name_content_before_returning_a_pair() {
        let environment = PreparedFontEnvironment::synthetic();
        let stack = named_stack(&["Test Sans"]);
        environment
            .prepare_pair(0, &stack, RequestedWeight::Normal, &['A'], None)
            .unwrap();
        for remaining in [0, 12, 13] {
            let resources = environment.generation_resources();
            let before = resources.usage().unwrap().1;
            let bytes = resources.byte_usage().unwrap();
            resources.set_test_limits(before + remaining, bytes);
            let result = environment.prepared_pair(&stack, RequestedWeight::Normal);
            if remaining == 13 {
                assert_eq!(result.unwrap().unwrap().provenance.pair_ordinal, 0);
            } else {
                assert!(result.is_err());
            }
            assert_eq!(resources.usage().unwrap().1 - before, remaining);
            assert_eq!(resources.byte_usage().unwrap(), bytes);
        }
    }

    #[test]
    fn incremental_and_normalized_hits_price_content_without_reprobing_fonts() {
        for normalized in [false, true] {
            let environment = PreparedFontEnvironment::synthetic();
            let stack = named_stack(&["Test Sans"]);
            environment
                .resolve_pair(0, &stack, RequestedWeight::Normal, &['A'])
                .unwrap();
            // Entry scan + cache scan + stack comparison (12). Incremental
            // lookup additionally visits one supplied and one retained scalar.
            let work = if normalized { 14 } else { 16 };
            for remaining in [0, work - 1, work] {
                let resources = environment.generation_resources();
                let before = resources.usage().unwrap().1;
                resources.set_test_limits(before + remaining, 536_870_912);
                FONT_PAIR_RESOLUTION_CALLS.with(|calls| calls.set(0));
                FONT_NORMALIZATION_CALLS.with(|calls| calls.set(0));
                FONT_SELECTION_PROBES.with(|calls| calls.set(0));
                let success = if normalized {
                    let mut core = environment.core.write().unwrap();
                    let CanonicalFontCore {
                        database, budget, ..
                    } = &mut *core;
                    let mut names = Vec::with_capacity(1);
                    resolve_stack_names_into(database, &stack, &mut names, budget, None).is_ok()
                } else {
                    environment
                        .resolve_pair(1, &stack, RequestedWeight::Normal, &['A'])
                        .is_ok()
                };
                assert_eq!(success, remaining == work);
                assert_eq!(resources.usage().unwrap().1 - before, remaining);
                FONT_PAIR_RESOLUTION_CALLS.with(|calls| assert_eq!(calls.get(), 0));
                FONT_NORMALIZATION_CALLS.with(|calls| assert_eq!(calls.get(), 0));
                FONT_SELECTION_PROBES.with(|calls| assert_eq!(calls.get(), 0));
            }
        }
    }

    #[test]
    fn staged_stack_copy_prices_one_byte_pass_before_any_font_storage() {
        let source = FontStack::new([
            FontFamily::named("αX").unwrap(),
            FontFamily::named("A, B").unwrap(),
            FontFamily::generic(GenericFontFamily::Serif),
        ])
        .unwrap();
        let bytes = 3 * size_of::<FontFamily>()
            + 3
            + 4
            + usize::try_from(font_stack_arc_bytes().unwrap()).unwrap();
        let work = 3 + 3 + 4;
        for remaining in [0, work - 1, work] {
            let stopped = crate::layout::plan_exact_root(
                1,
                1,
                |stage| {
                    let resources = stage.font_resources();
                    resources
                        .generation_resources()
                        .set_test_limits(remaining, bytes as u64);
                    let copy = source.copy_with_stage(&resources, Some(stage));
                    if remaining == work {
                        let copy = copy.unwrap();
                        assert_eq!(copy, source);
                        assert_ne!(copy.families().as_ptr(), source.families().as_ptr());
                        assert_eq!(resources.usage().unwrap().processing_visits, work);
                        assert_eq!(resources.usage().unwrap().font_bytes, bytes as u128);
                    } else {
                        assert!(copy.is_err());
                        assert_eq!(resources.usage().unwrap().processing_visits, 0);
                        assert_eq!(resources.usage().unwrap().font_bytes, 0);
                        assert_eq!(resources.generation_resources().usage().unwrap().0, 0);
                    }
                    // Stop the enclosing root before unrelated feedback storage;
                    // the producer's success/refusal has been inspected above.
                    Err::<(), _>(crate::Error::Render("font copy boundary inspected".into()))
                },
                |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
                |(), _, (), _| Ok(()),
            )
            .err()
            .unwrap();
            assert!(stopped.to_string().contains("font copy boundary inspected"));
        }
    }

    #[test]
    fn literal_font_values_preserve_bytes_duplicates_and_css_semantics() {
        let discoveries = FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
        let named = FontFamily::named("A, B").unwrap();
        let stack = FontStack::new([
            named.clone(),
            FontFamily::named("sans-serif").unwrap(),
            named.clone(),
            FontFamily::generic(GenericFontFamily::SansSerif),
        ])
        .unwrap();
        assert_eq!(stack.families()[0].name(), Some("A, B"));
        assert_eq!(stack.families()[2], named);
        assert_eq!(stack.families()[1].generic_family(), None);
        assert_eq!(
            stack.families()[3].generic_family(),
            Some(GenericFontFamily::SansSerif)
        );
        assert_eq!(
            stack.css(&OperationFontResources::new()).unwrap(),
            "\"A, B\", \"sans-serif\", \"A, B\", sans-serif"
        );
        for name in [
            "serif",
            "MONOSPACE",
            "cursive",
            "fantasy",
            "system-ui",
            "ui-serif",
            "ui-sans-serif",
            "ui-monospace",
            "ui-rounded",
            "math",
            "fangsong",
            "inherit",
            "initial",
            "unset",
            "revert",
            "revert-layer",
            "default",
        ] {
            assert_eq!(
                named_stack(&[name])
                    .css(&OperationFontResources::new())
                    .unwrap(),
                format!("\"{name}\"")
            );
        }
        assert_eq!(
            named_stack(&["A\"B\\C", "A  B", "16 Font", "A; B"])
                .css(&OperationFontResources::new())
                .unwrap(),
            "\"A\\\"B\\\\C\", \"A  B\", \"16 Font\", \"A; B\""
        );
        assert_eq!(
            builtin_font_stack()
                .css(&OperationFontResources::new())
                .unwrap(),
            crate::DEFAULT_FONT_FAMILY_STACK
        );
        assert_eq!(
            named_stack(&["-Font", "--Font", "-16 Font", "sans-serif Display", "emoji"])
                .css(&OperationFontResources::new())
                .unwrap(),
            "-Font, --Font, \"-16 Font\", \"sans-serif Display\", \"emoji\""
        );
        assert_eq!(FONT_DISCOVERY_CALLS.with(std::cell::Cell::get), discoveries);
    }

    #[test]
    fn literal_comma_and_keyword_select_named_faces_independently_of_generics() {
        let environment = PreparedFontEnvironment::synthetic_faces(&[
            ("A, B", &['X']),
            ("A", &['X']),
            ("B", &['X']),
            ("sans-serif", &['X']),
        ]);
        environment
            .core
            .write()
            .unwrap()
            .database
            .set_sans_serif_family("B");
        let literal = named_stack(&["A, B"]);
        let separate = named_stack(&["A", "B"]);
        let keyword = named_stack(&["sans-serif"]);
        let generic = FontStack::new([FontFamily::generic(GenericFontFamily::SansSerif)]).unwrap();
        for (ordinal, stack) in [&literal, &separate, &keyword, &generic]
            .into_iter()
            .enumerate()
        {
            environment
                .prepare_pair(
                    u32::try_from(ordinal).unwrap(),
                    stack,
                    RequestedWeight::Normal,
                    &['X'],
                    None,
                )
                .unwrap();
        }
        environment.seal_prepared_pairs().unwrap();
        let pairs = [&literal, &separate, &keyword, &generic].map(|stack| {
            environment
                .prepared_pair(stack, RequestedWeight::Normal)
                .unwrap()
                .unwrap()
        });
        assert_eq!(pairs[0].provenance.candidates.len(), 1);
        assert_eq!(pairs[1].provenance.candidates.len(), 2);
        assert_ne!(pairs[0].primary, pairs[1].primary);
        assert_ne!(pairs[2].primary, pairs[3].primary);
        assert_eq!(pairs[3].primary, pairs[1].provenance.candidates[1]);
        assert!(environment
            .prepared_pair(&named_stack(&["A,B"]), RequestedWeight::Normal)
            .unwrap()
            .is_none());
        assert!(environment
            .prepared_pair(&named_stack(&["Noto Sans"]), RequestedWeight::Normal)
            .unwrap()
            .is_none());
    }

    #[test]
    fn literal_stack_clone_admits_exact_storage_and_refuses_before_copy() {
        let source = FontStack::new([
            FontFamily::named("A, B").unwrap(),
            FontFamily::named("A, B").unwrap(),
            FontFamily::generic(GenericFontFamily::Serif),
        ])
        .unwrap();
        let bytes = 3 * size_of::<FontFamily>()
            + 2 * "A, B".len()
            + usize::try_from(font_stack_arc_bytes().unwrap()).unwrap();
        for remaining in [bytes - 1, bytes] {
            let resources = OperationGenerationResources::with_test_limits(
                source.byte_work().unwrap() as u128,
                remaining as u64,
            );
            let result = source.copy_with_generation(&resources);
            if remaining == bytes {
                let retained = result.unwrap();
                assert_eq!(retained, source);
                assert_ne!(retained.families().as_ptr(), source.families().as_ptr());
                assert_ne!(
                    retained.families()[0].name().unwrap().as_ptr(),
                    source.families()[0].name().unwrap().as_ptr()
                );
                assert_eq!(resources.byte_usage().unwrap(), bytes as u64);
            } else {
                assert!(matches!(
                    result,
                    Err(FontError::Limit {
                        resource: "system-font bytes",
                        ..
                    })
                ));
            }
        }
        let resources = OperationGenerationResources::with_test_limits(
            source.byte_work().unwrap() as u128 - 1,
            bytes as u64,
        );
        assert!(source.copy_with_generation(&resources).is_err());
        assert_eq!(resources.byte_usage().unwrap(), 0);
    }

    #[test]
    fn retained_face_lookups_admit_each_comparison_before_shaping_metrics_or_outlining() {
        struct OutlineCalls(usize);
        impl ttf_parser::OutlineBuilder for OutlineCalls {
            fn move_to(&mut self, _x: f32, _y: f32) {
                self.0 += 1;
            }
            fn line_to(&mut self, _x: f32, _y: f32) {
                self.0 += 1;
            }
            fn quad_to(&mut self, _x1: f32, _y1: f32, _x: f32, _y: f32) {
                self.0 += 1;
            }
            fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _x: f32, _y: f32) {
                self.0 += 1;
            }
            fn close(&mut self) {
                self.0 += 1;
            }
        }
        for (target, examined) in [(0, 1_u128), (2, 3), (3, 3)] {
            for route in 0..3 {
                let total = examined + u128::from(route == 0 && target != 3);
                for remaining in [examined - 1, total] {
                    let environment = PreparedFontEnvironment::synthetic_faces(&[
                        ("First", &['A']),
                        ("Middle", &['A']),
                        ("Last", &['A']),
                    ]);
                    let identity = if target == 3 {
                        FaceIdentity {
                            sha256: [0; 32],
                            face_index: 0,
                        }
                    } else {
                        environment.core.read().unwrap().faces[target].identity
                    };
                    let before = environment.resource_usage().unwrap();
                    let records = environment.generation_resources().usage().unwrap().0;
                    environment
                        .claim_processing_visits(1_000_000 - before.processing_visits - remaining)
                        .unwrap();
                    FONT_SHAPE_CALLS.with(|calls| calls.set(0));
                    SHAPED_GLYPH_RETAINS.with(|calls| calls.set(0));
                    let mut outline = OutlineCalls(0);
                    let result = match route {
                        0 => environment
                            .shape_run(
                                identity,
                                "A",
                                ShapingDirection::LeftToRight,
                                rustybuzz::script::LATIN,
                            )
                            .map(|_| ()),
                        1 => environment.face_metrics(identity).map(|_| ()),
                        _ => environment
                            .outline_glyph(identity, 1, &mut outline)
                            .map(|_| ()),
                    };
                    if remaining < examined {
                        assert!(matches!(
                            result,
                            Err(FontError::Limit {
                                resource: "root processing visits",
                                ..
                            })
                        ));
                        FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 0));
                        SHAPED_GLYPH_RETAINS.with(|calls| assert_eq!(calls.get(), 0));
                        assert_eq!(outline.0, 0);
                        assert_eq!(
                            environment.generation_resources().usage().unwrap().0,
                            records
                        );
                        assert_eq!(
                            environment.resource_usage().unwrap().font_bytes,
                            before.font_bytes
                        );
                    } else if target == 3 {
                        assert!(matches!(
                            result,
                            Err(FontError::Shape {
                                reason: "identity is not retained",
                                ..
                            } | FontError::Outline {
                                reason: "identity is not retained",
                                ..
                            })
                        ));
                    } else {
                        result.unwrap();
                        if route == 2 {
                            assert!(outline.0 > 0);
                        }
                    }
                    assert_eq!(
                        environment.resource_usage().unwrap().processing_visits,
                        1_000_000
                    );
                }
            }
        }
    }

    #[test]
    fn generation_limits_are_independent_inclusive_and_shared() {
        for resource in 0..5 {
            let font = OperationFontResources::new();
            let generation = font.generation_resources();
            let claim = |amount| match resource {
                0 => generation.claim_slots(amount),
                1 => generation.claim_visits(amount),
                2 => generation.claim_calls(amount),
                3 => generation.claim_cache_entries(amount),
                _ => generation.claim_topology_slots(amount),
            };
            claim(999_999).unwrap();
            claim(1).unwrap();
            assert!(matches!(
                claim(1),
                Err(TextGenerationClaimError::Limit {
                    limit: 1_000_000,
                    cumulative_claim: 1_000_001,
                    ..
                })
            ));
            assert!(matches!(
                claim(u128::MAX),
                Err(TextGenerationClaimError::Arithmetic(_))
            ));
            let (records, visits) = generation.usage().unwrap();
            let (calls, entries) = generation.clone().call_cache_usage();
            for (index, used) in [records, visits, calls, entries, generation.topology_usage()]
                .into_iter()
                .enumerate()
            {
                assert_eq!(used, if index == resource { 1_000_000 } else { 0 });
            }
            font.claim_discovery(1_000_000).unwrap();
            assert!(font.claim_discovery(1).is_err());
            font.claim_bytes(FONT_BYTE_LIMIT).unwrap();
            assert!(font.claim_bytes(1).is_err());
            assert_eq!(generation.usage().unwrap(), (records, visits));
            assert_eq!(generation.call_cache_usage(), (calls, entries));
        }
        for resource in 0..3 {
            let resources = OperationGenerationResources::new();
            let claim = |amount| match resource {
                0 => resources.clone().claim_calls(amount),
                1 => resources.clone().claim_cache_entries(amount),
                _ => resources.clone().claim_topology_slots(amount),
            };
            claim(100_000).unwrap();
            let successes = std::thread::scope(|scope| {
                let a = scope.spawn(|| claim(600_000).is_ok());
                let b = scope.spawn(|| claim(600_000).is_ok());
                usize::from(a.join().unwrap()) + usize::from(b.join().unwrap())
            });
            assert_eq!(successes, 1);
            claim(300_000).unwrap();
            assert!(claim(1).is_err());
        }
    }

    #[test]
    fn font_cache_refusal_does_not_insert_or_prevent_existing_entry_reuse() {
        let environment = PreparedFontEnvironment::synthetic();
        let stack = named_stack(&["Test Sans"]);
        environment
            .resolve_pair(0, &stack, RequestedWeight::Normal, &['A'])
            .unwrap();
        let resources = environment.generation_resources();
        let used = resources.call_cache_usage().1;
        resources.claim_cache_entries(1_000_000 - used).unwrap();
        environment
            .resolve_pair(0, &stack, RequestedWeight::Normal, &['A'])
            .unwrap();
        assert!(matches!(
            environment.resolve_pair(
                1,
                &named_stack(&["Test Serif"]),
                RequestedWeight::Normal,
                &['A']
            ),
            Err(FontError::ShapeAdmission(TextGenerationClaimError::Limit {
                resource: "root-local cache entries",
                ..
            }))
        ));
        assert_eq!(
            environment
                .core
                .read()
                .unwrap()
                .budget
                .normalized_stacks
                .len(),
            1
        );
        assert_eq!(resources.call_cache_usage().1, 1_000_000);
    }

    #[test]
    fn root_visits_are_atomic_shared_and_cannot_be_rebound_after_preparation() {
        let font = OperationFontResources::new();
        let generation = font.generation_resources();
        font.claim_visits(100_000).unwrap();
        let successes = std::thread::scope(|scope| {
            let a = scope.spawn(|| font.clone().claim_visits(600_000).is_ok());
            let b = scope.spawn(|| generation.clone().claim_visits(600_000).is_ok());
            usize::from(a.join().unwrap()) + usize::from(b.join().unwrap())
        });
        assert_eq!(successes, 1);
        assert_eq!(generation.usage().unwrap().1, 700_000);
        generation.claim_visits(300_000).unwrap();
        assert!(font.claim_visits(1).is_err());
        assert!(generation.claim_visits(u128::MAX).is_err());
        assert_eq!(generation.usage().unwrap().1, 1_000_000);
        generation.claim_slots(1_000_000).unwrap();
        assert!(font.generation_resources().claim_slots(1).is_err());

        let environment = PreparedFontEnvironment::synthetic();
        let original = environment.generation_resources();
        environment.share_generation_resources(&original).unwrap();
        let before = original.usage().unwrap();
        assert!(environment
            .clone()
            .share_generation_resources(&generation)
            .is_err());
        assert_eq!(original.usage().unwrap(), before);
        environment.claim_processing_visits(1).unwrap();
        assert_eq!(original.usage().unwrap().1, before.1 + 1);
    }

    #[test]
    fn canonical_shaping_admits_only_consumed_context_before_the_engine() {
        let environment = PreparedFontEnvironment::synthetic();
        let stack = named_stack(&["Test Sans"]);
        let pair = environment
            .resolve_pair(0, &stack, RequestedWeight::Normal, &['A'])
            .unwrap();
        let context = "A".repeat(2048);
        let before = environment.resource_usage().unwrap().processing_visits;
        environment
            .claim_processing_visits(1_000_000 - before - 12)
            .unwrap();
        FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        let shaped = environment
            .shape_run_with_context(
                pair.primary,
                "A",
                ShapingDirection::LeftToRight,
                rustybuzz::script::LATIN,
                ShapingContext {
                    pre: &context,
                    post: &context,
                    soft_hyphen: None,
                },
            )
            .unwrap();
        assert_eq!(shaped.glyphs.len(), 1);
        assert_eq!(
            environment.resource_usage().unwrap().processing_visits,
            1_000_000
        );
        FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        SHAPED_GLYPH_RETAINS.with(|count| count.set(0));
        assert!(matches!(
            environment.shape_run(
                pair.primary,
                "A",
                ShapingDirection::LeftToRight,
                rustybuzz::script::LATIN
            ),
            Err(FontError::Limit {
                resource: "root processing visits",
                ..
            })
        ));
        FONT_SHAPE_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        SHAPED_GLYPH_RETAINS.with(|count| assert_eq!(count.get(), 0));
    }

    #[test]
    fn incremental_pairs_reuse_normalization_candidates_and_scalar_probes() {
        let environment = PreparedFontEnvironment::synthetic();
        let shared = environment.clone();
        FONT_NORMALIZATION_CALLS.with(|calls| calls.set(0));
        FONT_PAIR_RESOLUTION_CALLS.with(|calls| calls.set(0));
        FONT_SELECTION_PROBES.with(|calls| calls.set(0));
        let stack = named_stack(&["Test Sans"]);
        environment
            .resolve_pair(0, &stack, RequestedWeight::Normal, &['A'])
            .unwrap();
        let resources = environment.core.read().unwrap().budget.resources.clone();
        resources.claim_visits(900_000).unwrap();
        let before = resources.usage().unwrap().processing_visits;
        for ordinal in 1..=1000 {
            let stack = stack.clone();
            let pair = shared
                .resolve_pair(ordinal, &stack, RequestedWeight::Normal, &['A', 'A'])
                .unwrap();
            assert_eq!(pair.provenance.pair_ordinal, ordinal);
            assert_eq!(pair.selected_scalars.len(), 1);
        }
        let after = resources.usage().unwrap().processing_visits;
        assert!(after > before && after < 1_000_000);
        FONT_NORMALIZATION_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        FONT_PAIR_RESOLUTION_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        FONT_SELECTION_PROBES.with(|calls| assert_eq!(calls.get(), 1));

        let pair = shared
            .resolve_pair(
                1001,
                &stack,
                RequestedWeight::Normal,
                &['A', ' ', '\n', 'A'],
            )
            .unwrap();
        assert_eq!(pair.selected_scalars.len(), 3);
        FONT_SELECTION_PROBES.with(|calls| assert_eq!(calls.get(), 2));
        shared
            .resolve_pair(1002, &stack, RequestedWeight::Bold, &['A'])
            .unwrap();
        FONT_PAIR_RESOLUTION_CALLS.with(|calls| assert_eq!(calls.get(), 2));
        FONT_NORMALIZATION_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        FONT_SELECTION_PROBES.with(|calls| assert_eq!(calls.get(), 3));
        let used = resources.usage().unwrap().processing_visits;
        resources
            .claim_visits(u64::try_from(1_000_000 - used).unwrap())
            .unwrap();
        assert!(matches!(
            shared.resolve_pair(1003, &stack, RequestedWeight::Normal, &['A']),
            Err(FontError::Limit { .. })
        ));
    }

    #[test]
    fn sealed_census_reuses_one_normalization_per_stack_and_one_selection_per_pair() {
        let environment = PreparedFontEnvironment::synthetic();
        let stack = named_stack(&["Test Sans"]);
        FONT_NORMALIZATION_CALLS.with(|calls| calls.set(0));
        FONT_PAIR_RESOLUTION_CALLS.with(|calls| calls.set(0));
        FONT_SELECTION_PROBES.with(|calls| calls.set(0));
        environment
            .prepare_pairs(
                [
                    FontPairPreparationRequest::new(
                        0,
                        &stack,
                        RequestedWeight::Normal,
                        &['A', ' ', '\n'],
                    ),
                    FontPairPreparationRequest::new(1, &stack, RequestedWeight::Bold, &['A']),
                ]
                .into_iter(),
                None,
            )
            .unwrap();
        environment.seal_prepared_pairs().unwrap();
        for _ in 0..1000 {
            let pair = environment
                .clone()
                .prepared_pair(&stack, RequestedWeight::Normal)
                .unwrap()
                .unwrap();
            assert_eq!(pair.selected_scalars.len(), 3);
        }
        FONT_NORMALIZATION_CALLS.with(|calls| assert_eq!(calls.get(), 1));
        FONT_PAIR_RESOLUTION_CALLS.with(|calls| assert_eq!(calls.get(), 2));
        FONT_SELECTION_PROBES.with(|calls| assert_eq!(calls.get(), 3));
        assert!(matches!(
            environment.resolve_pair(0, &stack, RequestedWeight::Normal, &['A']),
            Err(FontError::PreparedPairAfterSeal { .. })
        ));
    }

    #[test]
    fn family_normalization_propagates_shared_root_refusals() {
        let resources = OperationFontResources::new();
        let mut budget = FontBudget::from_resources(resources.clone());
        resources.claim_visits(PROCESSING_VISIT_LIMIT).unwrap();
        assert!(matches!(
            fold_family_name("Test Sans", &mut budget),
            Err(FontError::Limit {
                resource: "root processing visits",
                ..
            })
        ));

        let resources = OperationFontResources::new();
        let mut budget = FontBudget::from_resources(resources.clone());
        resources.claim_bytes(FONT_BYTE_LIMIT).unwrap();
        assert!(matches!(
            fold_family_name("Test Sans", &mut budget),
            Err(FontError::Limit {
                resource: "system-font bytes",
                ..
            })
        ));
    }

    #[test]
    fn unicode_16_casefold_covers_ascii_compatibility_and_full_fold_cases() {
        assert_eq!(
            super::super::unicode16::folded_for_test("NOTO Sans"),
            "noto sans"
        );
        assert_eq!(
            super::super::unicode16::folded_for_test("Ｎｏｔｏ ﬃ"),
            "noto ffi"
        );
        assert_eq!(
            super::super::unicode16::folded_for_test("Straße"),
            super::super::unicode16::folded_for_test("STRASSE")
        );
        assert_eq!(
            super::super::unicode16::folded_for_test("ΟΣ"),
            super::super::unicode16::folded_for_test("ος")
        );
    }

    #[test]
    fn resource_limits_accept_equal_and_reject_plus_one() {
        let limits = ResourceLimits {
            discovery_entries: 2,
            font_bytes: 2,
            processing_visits: 4,
        };
        let mut budget = FontBudget::new(limits);
        budget.claim_discovery(2).unwrap();
        budget.claim_bytes(2).unwrap();
        budget.claim_visits(2).unwrap();
        assert!(matches!(
            budget.claim_discovery(1),
            Err(FontError::Limit { .. })
        ));
        assert!(matches!(
            budget.claim_bytes(1),
            Err(FontError::Limit { .. })
        ));
        assert!(matches!(
            budget.claim_visits(1),
            Err(FontError::Limit { .. })
        ));
    }

    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "android", target_os = "redox"))
    ))]
    #[test]
    fn bounded_fontconfig_preserves_include_order_resets_and_aliases() {
        let root = tempfile::tempdir().unwrap();
        let includes = root.path().join("conf.d");
        fs::create_dir(&includes).unwrap();
        fs::write(
            includes.join("10-first.conf"),
            "<fontconfig><dir>/first</dir><alias><family>serif</family><prefer><family>First Serif</family></prefer></alias></fontconfig>",
        )
        .unwrap();
        fs::write(
            includes.join("20-reset.conf"),
            "<fontconfig><reset-dirs/><dir>/after</dir><alias><family>sans-serif</family><accept><family>After Sans</family></accept></alias></fontconfig>",
        )
        .unwrap();
        let config_path = root.path().join("fonts.conf");
        fs::write(
            &config_path,
            "<fontconfig><dir>/before</dir><include prefix=\"relative\">conf.d</include><alias><family>monospace</family><default><family>Root Mono</family></default></alias></fontconfig>",
        )
        .unwrap();

        let mut config = BoundedFontConfig::default();
        let mut budget = FontBudget::new(ResourceLimits::PRODUCTION);
        assert!(merge_fontconfig_root(&mut config, &config_path, &mut budget, None).unwrap());
        assert_eq!(
            config
                .dirs
                .iter()
                .map(|directory| directory.path.as_path())
                .collect::<Vec<_>>(),
            [Path::new("/after")]
        );
        assert_eq!(
            config
                .aliases
                .iter()
                .map(|alias| alias.alias.as_str())
                .collect::<Vec<_>>(),
            ["serif", "sans-serif", "monospace"]
        );
        let mut upstream = fontconfig_parser::FontConfig::default();
        upstream.merge_config(&config_path).unwrap();
        assert_eq!(config.dirs, upstream.dirs);
        assert_eq!(config.aliases, upstream.aliases);
    }

    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "android", target_os = "redox"))
    ))]
    #[test]
    fn bounded_fontconfig_retains_directives_before_a_relevant_parse_failure() {
        let root = tempfile::tempdir().unwrap();
        let config_path = root.path().join("fonts.conf");
        fs::write(
            &config_path,
            "<fontconfig><dir>/first</dir><dir/><dir>/discarded</dir></fontconfig>",
        )
        .unwrap();

        let mut config = BoundedFontConfig::default();
        assert!(!merge_fontconfig_root(
            &mut config,
            &config_path,
            &mut FontBudget::new(ResourceLimits::PRODUCTION),
            None,
        )
        .unwrap());

        assert_eq!(config.dirs.len(), 1);
        assert_eq!(config.dirs[0].path, Path::new("/first"));
        let mut upstream = fontconfig_parser::FontConfig::default();
        assert!(upstream.merge_config(&config_path).is_err());
        assert_eq!(config.dirs, upstream.dirs);
    }

    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "android", target_os = "redox"))
    ))]
    #[test]
    fn bounded_fontconfig_root_status_drives_fallback_like_locked_parser() {
        let root = tempfile::tempdir().unwrap();
        let local = root.path().join("local.conf");
        let default = root.path().join("default.conf");
        fs::write(&local, "<fontconfig><dir>/local</dir></fontconfig>").unwrap();
        fs::write(&default, "<fontconfig><dir>/default</dir></fontconfig>").unwrap();
        for source in [
            None,
            Some("<fontconfig><dir>/valid</dir></fontconfig>"),
            Some("<fontconfig><dir>/prefix</dir><dir/><dir>/discarded</dir></fontconfig>"),
            Some(
                "<fontconfig><dir>/prefix</dir><include prefix=\"invalid\">bad</include></fontconfig>",
            ),
            Some("<fontconfig><dir>/prefix</dir><alias><family/></alias></fontconfig>"),
            Some("<fontconfig><dir>/malformed</dir>"),
            Some("<wrong-root/>"),
        ] {
            let user = root.path().join("user.conf");
            if let Some(source) = source {
                fs::write(&user, source).unwrap();
            }
            let mut config = BoundedFontConfig::default();
            let mut budget = FontBudget::new(ResourceLimits::PRODUCTION);
            let mut upstream = fontconfig_parser::FontConfig::default();
            let complete = merge_fontconfig_root(&mut config, &user, &mut budget, None).unwrap();
            let upstream_complete = upstream.merge_config(&user).is_ok();
            assert_eq!(complete, upstream_complete, "{source:?}");
            assert_eq!(config.dirs, upstream.dirs, "{source:?}");
            if !complete {
                assert!(merge_fontconfig_root(&mut config, &local, &mut budget, None).unwrap());
            }
            if !upstream_complete {
                upstream.merge_config(&local).unwrap();
            }
            assert!(merge_fontconfig_root(&mut config, &default, &mut budget, None).unwrap());
            upstream.merge_config(&default).unwrap();
            assert_eq!(config.dirs, upstream.dirs, "{source:?}");
            assert_eq!(config.aliases, upstream.aliases, "{source:?}");
            if source.is_some() {
                // Canonical deduplication is successful even after the first parse failed.
                assert!(merge_fontconfig_root(&mut config, &user, &mut budget, None).unwrap());
                upstream.merge_config(&user).unwrap();
                fs::remove_file(&user).unwrap();
            }
        }
    }

    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "android", target_os = "redox"))
    ))]
    #[test]
    fn bounded_fontconfig_included_failures_do_not_fail_parent() {
        let root = tempfile::tempdir().unwrap();
        let includes = root.path().join("conf.d");
        fs::create_dir(&includes).unwrap();
        fs::write(
            includes.join("10-partial.conf"),
            "<fontconfig><dir>/included</dir><dir/><dir>/discarded</dir></fontconfig>",
        )
        .unwrap();
        fs::write(
            includes.join("20-malformed.conf"),
            "<fontconfig><dir>/malformed</dir>",
        )
        .unwrap();
        fs::write(
            includes.join("30-last.conf"),
            "<fontconfig><dir>/last</dir><include prefix=\"relative\">../fonts.conf</include></fontconfig>",
        )
        .unwrap();
        for include in [
            "conf.d/10-partial.conf",
            "conf.d/20-malformed.conf",
            "conf.d",
            "absent",
        ] {
            let path = root.path().join("fonts.conf");
            fs::write(&path, format!(
                "<fontconfig><include prefix=\"relative\">{include}</include><dir>/parent</dir></fontconfig>"
            )).unwrap();
            let mut config = BoundedFontConfig::default();
            assert!(merge_fontconfig_root(
                &mut config,
                &path,
                &mut FontBudget::new(ResourceLimits::PRODUCTION),
                None
            )
            .unwrap());
            let mut upstream = fontconfig_parser::FontConfig::default();
            upstream.merge_config(&path).unwrap();
            assert_eq!(config.dirs, upstream.dirs, "{include}");
        }
    }

    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "android", target_os = "redox"))
    ))]
    #[test]
    fn bounded_fontconfig_empty_preferences_match_locked_parser() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("fonts.conf");
        fs::write(&path,
            "<fontconfig><alias><family>serif</family><prefer><family/><family>Preferred</family></prefer><accept><family/><family>Accepted</family></accept><default><family/><family>Default</family></default></alias><dir>/after</dir></fontconfig>"
        ).unwrap();
        let mut config = BoundedFontConfig::default();
        assert!(merge_fontconfig_root(
            &mut config,
            &path,
            &mut FontBudget::new(ResourceLimits::PRODUCTION),
            None
        )
        .unwrap());
        let mut upstream = fontconfig_parser::FontConfig::default();
        upstream.merge_config(&path).unwrap();
        assert_eq!(config.aliases, upstream.aliases);
        assert_eq!(config.dirs, upstream.dirs);
        assert_eq!(config.aliases[0].prefer, ["Preferred"]);
    }

    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "android", target_os = "redox"))
    ))]
    #[test]
    fn bounded_fontconfig_missing_optional_paths_but_not_admitted_io_are_ignored() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("absent.conf");
        let mut config = BoundedFontConfig::default();
        let mut budget = FontBudget::new(ResourceLimits::PRODUCTION);
        assert!(matches!(
            read_bounded_fontconfig(&mut config, &path, false, &mut budget, None),
            Ok(FontConfigRead::Ignored)
        ));
        assert!(matches!(
            read_bounded_fontconfig(&mut config, &path, true, &mut budget, None),
            Err(FontError::Io { .. })
        ));
        fs::write(&path, [0xff]).unwrap();
        assert!(matches!(
            merge_fontconfig_root(&mut config, &path, &mut budget, None),
            Err(FontError::Io { .. })
        ));

        let includes = root.path().join("conf.d");
        fs::create_dir(&includes).unwrap();
        std::os::unix::fs::symlink(root.path().join("missing"), includes.join("broken.conf"))
            .unwrap();
        let parent = root.path().join("parent.conf");
        fs::write(
            &parent,
            "<fontconfig><include prefix=\"relative\">conf.d</include></fontconfig>",
        )
        .unwrap();
        assert!(matches!(
            merge_fontconfig_root(&mut config, &parent, &mut budget, None),
            Err(FontError::Io { .. })
        ));
    }

    #[test]
    fn discovery_retains_one_handle_at_a_time_and_closes_on_byte_refusal() {
        let root = tempfile::tempdir().unwrap();
        for (name, bytes) in [
            ("a.ttf", b"first".as_slice()),
            ("b.otf", b"second".as_slice()),
        ] {
            fs::write(root.path().join(name), bytes).unwrap();
        }
        let roots = ordinal_roots(vec![root.path().to_path_buf()], None).unwrap();
        let limits = ResourceLimits {
            discovery_entries: 8,
            font_bytes: 11,
            processing_visits: 32,
        };
        DISCOVERY_HANDLES.with(|state| state.set([0; 5]));
        let files = discover_retained_files(&roots, &mut FontBudget::new(limits)).unwrap();
        DISCOVERY_HANDLES.with(|state| assert_eq!(state.get(), [0, 1, 2, 2, 2]));
        for (file, source) in files
            .iter()
            .zip([b"first".as_slice(), b"second".as_slice()])
        {
            assert_eq!(file.bytes.as_slice(), source);
            assert_eq!(file.sha256, <[u8; 32]>::from(Sha256::digest(source)));
        }
        DISCOVERY_HANDLES.with(|state| state.set([0; 5]));
        let mut budget = FontBudget::new(ResourceLimits {
            font_bytes: 5,
            ..limits
        });
        assert!(matches!(
            discover_retained_files(&roots, &mut budget),
            Err(FontError::Limit { .. })
        ));
        DISCOVERY_HANDLES.with(|state| assert_eq!(state.get(), [0, 1, 2, 1, 1]));
        assert_eq!(budget.resources.usage().unwrap().font_bytes, 5);
    }

    #[test]
    fn discovery_closes_handle_on_observed_file_changes() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("a.ttf");
        fs::write(root.path().join("b.ttf"), b"unopened").unwrap();
        let roots = ordinal_roots(vec![root.path().to_path_buf()], None).unwrap();
        let limits = ResourceLimits {
            discovery_entries: 8,
            font_bytes: 32,
            processing_visits: 32,
        };
        for hook in [
            (|path: &Path| fs::write(path, b"x").unwrap()) as fn(&Path),
            (|path: &Path| fs::write(path, b"longer than admitted").unwrap()) as fn(&Path),
        ] {
            fs::write(&path, b"original").unwrap();
            DISCOVERY_HANDLES.with(|state| state.set([0; 5]));
            DISCOVERY_AFTER_ADMISSION.with(|state| state.set(Some(hook)));
            assert!(
                matches!(discover_retained_files(&roots, &mut FontBudget::new(limits)),
                Err(FontError::ChangedFile(changed)) if changed == fs::canonicalize(&path).unwrap())
            );
            DISCOVERY_HANDLES.with(|state| assert_eq!(state.get(), [0, 1, 1, 1, 0]));
        }
    }

    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "android", target_os = "redox"))
    ))]
    #[test]
    fn bounded_fontconfig_parser_capacity_and_entity_rejection_are_explicit() {
        let path = Path::new("fixture.conf");
        let source = "<fontconfig><dir salt=\"s\">/fonts</dir></fontconfig>";
        let inputs = source.chars().count();
        let limits = ResourceLimits {
            discovery_entries: 1,
            font_bytes: 0,
            processing_visits: inputs as u64 + 1,
        };
        let mut budget = FontBudget::new(limits);
        let (parts, complete) = parse_bounded_fontconfig(source, path, &mut budget, None).unwrap();
        assert!(complete);
        assert_eq!(parts.len(), 1);
        assert_eq!(
            budget.resources.usage().unwrap().processing_visits,
            inputs as u128 + 1
        );
        assert!(matches!(
            parse_bounded_fontconfig(
                source,
                path,
                &mut FontBudget::new(ResourceLimits {
                    processing_visits: inputs as u64 - 1,
                    ..limits
                }),
                None
            ),
            Err(FontError::Limit {
                resource: "root processing visits",
                ..
            })
        ));
        assert!(matches!(
            parse_bounded_fontconfig(
                "<!DOCTYPE fontconfig [<!ENTITY path '/fonts'>]><fontconfig><dir>&path;</dir></fontconfig>",
                path,
                &mut FontBudget::new(ResourceLimits::PRODUCTION),
                None
            ),
            Err(FontError::ParseFontConfig { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn discovery_distinguishes_missing_roots_from_entry_io_failures() {
        let root = tempfile::tempdir().unwrap();
        let mut budget = FontBudget::new(ResourceLimits::PRODUCTION);
        let roots = ordinal_roots(vec![root.path().join("missing")], None).unwrap();
        assert!(discover_retained_files(&roots, &mut budget)
            .unwrap()
            .is_empty());
        std::os::unix::fs::symlink(root.path().join("absent"), root.path().join("broken.ttf"))
            .unwrap();
        let roots = ordinal_roots(vec![root.path().to_path_buf()], None).unwrap();
        assert!(matches!(
            discover_retained_files(&roots, &mut budget),
            Err(FontError::Io { .. })
        ));
    }

    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "android", target_os = "redox"))
    ))]
    #[test]
    fn bounded_fontconfig_rejects_bytes_before_reading_past_the_limit() {
        let root = tempfile::tempdir().unwrap();
        let config_path = root.path().join("fonts.conf");
        let source = "<fontconfig><dir>/fonts</dir></fontconfig>";
        fs::write(&config_path, source).unwrap();
        let equal_limits = ResourceLimits {
            discovery_entries: 2,
            font_bytes: source.len() as u64,
            processing_visits: 1_024,
        };
        assert!(merge_fontconfig_root(
            &mut BoundedFontConfig::default(),
            &config_path,
            &mut FontBudget::new(equal_limits),
            None,
        )
        .unwrap());
        let plus_one = ResourceLimits {
            font_bytes: source.len() as u64 - 1,
            ..equal_limits
        };
        assert!(matches!(
            merge_fontconfig_root(
                &mut BoundedFontConfig::default(),
                &config_path,
                &mut FontBudget::new(plus_one),
                None,
            ),
            Err(FontError::Limit {
                resource: "system-font bytes",
                ..
            })
        ));
    }

    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "android", target_os = "redox"))
    ))]
    #[test]
    fn bounded_fontconfig_include_entries_share_the_discovery_limit() {
        let root = tempfile::tempdir().unwrap();
        let includes = root.path().join("conf.d");
        fs::create_dir(&includes).unwrap();
        // Even an entry that is not a configuration file consumes admission.
        fs::create_dir(includes.join("ignored-directory")).unwrap();
        for name in ["10.conf", "20.conf"] {
            fs::write(
                includes.join(name),
                "<fontconfig><dir>/fonts</dir></fontconfig>",
            )
            .unwrap();
        }
        let config_path = root.path().join("fonts.conf");
        fs::write(
            &config_path,
            "<fontconfig><include prefix=\"relative\">conf.d</include></fontconfig>",
        )
        .unwrap();
        let limits = ResourceLimits {
            discovery_entries: 7,
            font_bytes: 1_024,
            processing_visits: 1_024,
        };
        assert!(matches!(
            merge_fontconfig_root(
                &mut BoundedFontConfig::default(),
                &config_path,
                &mut FontBudget::new(limits),
                None,
            ),
            Err(FontError::Limit {
                resource: "canonical system-font discovery entries",
                ..
            })
        ));
        let exact = ResourceLimits {
            discovery_entries: 8,
            ..limits
        };
        assert!(merge_fontconfig_root(
            &mut BoundedFontConfig::default(),
            &config_path,
            &mut FontBudget::new(exact),
            None,
        )
        .unwrap());
    }

    #[test]
    fn discovery_enforces_injected_entry_and_byte_boundaries() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("one.ttf"), [1_u8, 2]).unwrap();
        let roots = [SourceRoot {
            ordinal: 0,
            path: root.path().to_path_buf(),
            entry_prefix: None,
        }];
        let equal_limits = ResourceLimits {
            discovery_entries: 2,
            font_bytes: 2,
            processing_visits: 5,
        };
        assert_eq!(
            discover_retained_files(&roots, &mut FontBudget::new(equal_limits))
                .unwrap()
                .len(),
            1
        );

        let entry_plus_one = ResourceLimits {
            discovery_entries: 1,
            ..equal_limits
        };
        assert!(matches!(
            discover_retained_files(&roots, &mut FontBudget::new(entry_plus_one)),
            Err(FontError::Limit { .. })
        ));
        let byte_plus_one = ResourceLimits {
            font_bytes: 1,
            ..equal_limits
        };
        assert!(matches!(
            discover_retained_files(&roots, &mut FontBudget::new(byte_plus_one)),
            Err(FontError::Limit { .. })
        ));
        let work_plus_one = ResourceLimits {
            processing_visits: 4,
            ..equal_limits
        };
        assert!(matches!(
            discover_retained_files(&roots, &mut FontBudget::new(work_plus_one)),
            Err(FontError::Limit { .. })
        ));
    }

    #[test]
    fn retained_discovery_sorts_by_root_then_native_path_and_deduplicates() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        File::create(first.path().join("z.ttf")).unwrap();
        File::create(first.path().join("a.TTF")).unwrap();
        File::create(second.path().join("b.otf")).unwrap();
        File::create(second.path().join("ignored.TtF")).unwrap();
        let roots = vec![
            SourceRoot {
                ordinal: 1,
                path: second.path().to_path_buf(),
                entry_prefix: None,
            },
            SourceRoot {
                ordinal: 0,
                path: first.path().to_path_buf(),
                entry_prefix: None,
            },
            SourceRoot {
                ordinal: 2,
                path: first.path().to_path_buf(),
                entry_prefix: None,
            },
        ];
        let limits = ResourceLimits {
            discovery_entries: 32,
            font_bytes: 32,
            processing_visits: 32,
        };
        let files = discover_retained_files(&roots, &mut FontBudget::new(limits)).unwrap();
        let names: Vec<_> = files
            .iter()
            .map(|file| {
                (
                    file.root_ordinal,
                    file.canonical_path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                )
            })
            .collect();
        assert_eq!(
            names,
            [
                (0, "a.TTF".into()),
                (0, "z.ttf".into()),
                (1, "b.otf".into())
            ]
        );
    }

    #[test]
    fn direct_file_discovery_root_is_counted_but_not_admitted() {
        let directory = tempfile::tempdir().unwrap();
        let font = directory.path().join("direct.ttf");
        fs::write(&font, [1_u8, 2]).unwrap();
        let roots = [SourceRoot {
            ordinal: 0,
            path: font,
            entry_prefix: None,
        }];
        let limits = ResourceLimits {
            discovery_entries: 1,
            font_bytes: 0,
            processing_visits: 1,
        };
        let mut budget = FontBudget::new(limits);

        let files = discover_retained_files(&roots, &mut budget).unwrap();

        assert!(files.is_empty());
        assert_eq!(
            budget.resources.usage().unwrap(),
            FontResourceUsage {
                discovery_entries: 1,
                font_bytes: 0,
                processing_visits: 1,
            }
        );
    }

    #[test]
    fn retained_font_read_rejects_short_and_never_reads_past_admitted_length() {
        struct FailedRead;
        impl Read for FailedRead {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("injected admitted read failure"))
            }
        }

        let path = Path::new("changing-test.ttf");
        let mut short = std::io::Cursor::new([1_u8, 2]);
        let mut bytes = Vec::with_capacity(3);
        assert!(matches!(
            read_exact_font_bytes(&mut short, path, 3, &mut bytes),
            Err(FontError::ChangedFile(changed)) if changed == path
        ));

        let mut growing = std::io::Cursor::new([1_u8, 2, 3]);
        let mut bytes = Vec::with_capacity(2);
        let digest = read_exact_font_bytes(&mut growing, path, 2, &mut bytes).unwrap();
        let expected: [u8; 32] = Sha256::digest([1_u8, 2]).into();
        assert_eq!(bytes, [1, 2]);
        assert_eq!(growing.position(), 2);
        assert_eq!(digest, expected);

        assert!(matches!(
            read_exact_font_bytes(&mut FailedRead, path, 1, &mut Vec::with_capacity(1)),
            Err(FontError::Io {
                operation: "read font file",
                ..
            })
        ));
    }

    #[test]
    fn directory_symlinks_are_not_followed() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let root = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            File::create(outside.path().join("hidden.ttf")).unwrap();
            symlink(outside.path(), root.path().join("linked")).unwrap();
            let roots = [SourceRoot {
                ordinal: 0,
                path: root.path().to_path_buf(),
                entry_prefix: None,
            }];
            let limits = ResourceLimits {
                discovery_entries: 8,
                font_bytes: 8,
                processing_visits: 8,
            };
            let files = discover_retained_files(&roots, &mut FontBudget::new(limits)).unwrap();
            assert!(files.is_empty());
        }
    }

    #[test]
    fn canonical_candidate_order_retains_least_occurrence_per_identity() {
        let identity_a = identity(1, 0);
        let identity_b = identity(2, 0);
        let mut candidates = vec![
            candidate(identity_a, 1, 0, 0),
            candidate(identity_b, 0, 1, 0),
            candidate(identity_a, 0, 0, 20),
            candidate(identity_a, 0, 0, 10),
        ];
        let mut budget = FontBudget::new(ResourceLimits::PRODUCTION);
        canonicalize_candidate_occurrences(&mut candidates, &mut budget).unwrap();
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].occurrence.identity, identity_a);
        assert_eq!(candidates[0].occurrence.weight_distance, 10);
        assert_eq!(candidates[1].occurrence.identity, identity_b);
    }

    #[test]
    fn unmatched_family_produces_no_candidates() {
        let database = Database::new();
        let face = fake_face(identity(1, 0), "Test Sans");
        let mut budget = FontBudget::new(ResourceLimits::PRODUCTION);
        let candidates = canonical_candidates(
            &database,
            &[face],
            &named_stack(&["Other Sans"]),
            RequestedWeight::Normal,
            &mut budget,
        )
        .unwrap();
        assert!(candidates.is_empty());
    }

    #[test]
    fn missing_glyph_is_explicit_and_mandatory_controls_are_not_probed() {
        let candidates = [candidate(identity(1, 0), 0, 0, 0)];
        let mut budget = FontBudget::new(ResourceLimits::PRODUCTION);
        let mut presence = scalar_presence_map();
        let error = select_scalar_faces(
            4,
            &candidates,
            &['x'],
            &mut presence,
            &mut budget,
            true,
            |_, _| Ok(false),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            FontError::MissingGlyph {
                pair_ordinal: 4,
                scalar: 0x78
            }
        ));

        let controls = [
            '\n', '\r', '\u{000B}', '\u{000C}', '\u{0085}', '\u{2028}', '\u{2029}',
        ];
        let mut budget = FontBudget::new(ResourceLimits::PRODUCTION);
        let mut presence = scalar_presence_map();
        let selected = select_scalar_faces(
            0,
            &candidates,
            &controls,
            &mut presence,
            &mut budget,
            true,
            |_, _| panic!("mandatory line-break controls must not be probed"),
        )
        .unwrap();
        assert!(selected.iter().all(|selected| matches!(
            selected.selection,
            ScalarFaceSelection::NonRenderingMandatoryBreak
        )));
    }

    #[test]
    fn scalar_selection_preserves_first_observed_probe_order() {
        let first = identity(1, 0);
        let second = identity(2, 0);
        let candidates = [candidate(first, 0, 0, 0), candidate(second, 0, 0, 0)];
        let mut budget = FontBudget::new(ResourceLimits::PRODUCTION);
        let mut presence = scalar_presence_map();
        let mut probes = Vec::new();

        let selected = select_scalar_faces(
            0,
            &candidates,
            &['z', 'a', 'z', 'm'],
            &mut presence,
            &mut budget,
            true,
            |candidate, scalar| {
                probes.push((candidate.occurrence.identity, scalar));
                Ok(candidate.occurrence.identity == second || scalar != 'z')
            },
        )
        .unwrap();

        assert_eq!(
            probes,
            [
                (first, 'z'),
                (second, 'z'),
                (first, 'a'),
                (second, 'a'),
                (first, 'm'),
                (second, 'm')
            ]
        );
        assert_eq!(
            selected
                .iter()
                .map(|selection| selection.scalar)
                .collect::<Vec<_>>(),
            ['z', 'a', 'm']
        );
        assert_eq!(
            selected
                .iter()
                .map(|selection| selection.selection)
                .collect::<Vec<_>>(),
            [
                ScalarFaceSelection::Face(second),
                ScalarFaceSelection::Face(first),
                ScalarFaceSelection::Face(first),
            ]
        );
        let selected = SelectedScalarFaces(selected);
        for scalar in ['z', 'a', 'm'] {
            let index = selected
                .binary_search_by_key(&scalar, |selection| selection.scalar)
                .unwrap();
            assert_eq!(selected[index].scalar, scalar);
        }
    }

    #[test]
    fn scalar_selection_reports_first_observed_missing_scalar() {
        let first = identity(1, 0);
        let second = identity(2, 0);
        let candidates = [candidate(first, 0, 0, 0), candidate(second, 0, 0, 0)];
        let mut budget = FontBudget::new(ResourceLimits::PRODUCTION);
        let mut presence = scalar_presence_map();
        let mut probes = Vec::new();

        let error = select_scalar_faces(
            7,
            &candidates,
            &['z', 'a', 'z'],
            &mut presence,
            &mut budget,
            true,
            |candidate, scalar| {
                probes.push((candidate.occurrence.identity, scalar));
                Ok(false)
            },
        )
        .unwrap_err();

        assert!(matches!(
            error,
            FontError::MissingGlyph {
                pair_ordinal: 7,
                scalar: 0x7A
            }
        ));
        assert_eq!(probes, [(first, 'z'), (second, 'z')]);
    }

    #[test]
    fn sealed_environment_rejects_every_late_pair_resolution_route() {
        let environment = PreparedFontEnvironment::synthetic();
        let stack = named_stack(&["Test Sans"]);
        environment.reserve_prepared_pairs(1, None).unwrap();
        environment
            .prepare_pair(0, &stack, RequestedWeight::Normal, &['A'], None)
            .unwrap();
        environment.seal_prepared_pairs().unwrap();

        assert!(matches!(
            environment.resolve_pair(1, &stack, RequestedWeight::Normal, &['A']),
            Err(FontError::PreparedPairAfterSeal { pair_ordinal: 1 })
        ));
        assert!(matches!(
            environment.prepare_pairs(
                std::iter::once(FontPairPreparationRequest::new(
                    1,
                    &stack,
                    RequestedWeight::Bold,
                    &['A'],
                )),
                None,
            ),
            Err(FontError::PreparedPairAfterSeal { pair_ordinal: 1 })
        ));
    }

    #[test]
    fn batch_pair_preparation_preserves_order_and_exact_work() {
        let environment = PreparedFontEnvironment::synthetic();
        let stack = named_stack(&["Test Sans"]);
        let normal_scalars = ['A'];
        let bold_scalars = ['\n', 'A', '\n'];
        let before = environment.resource_usage().unwrap().processing_visits;
        FONT_PAIR_RESOLUTION_CALLS.with(|calls| calls.set(0));

        environment
            .prepare_pairs(
                [
                    FontPairPreparationRequest::new(
                        4,
                        &stack,
                        RequestedWeight::Normal,
                        &normal_scalars,
                    ),
                    FontPairPreparationRequest::new(
                        2,
                        &stack,
                        RequestedWeight::Bold,
                        &bold_scalars,
                    ),
                ]
                .into_iter(),
                None,
            )
            .unwrap();

        let after = environment.resource_usage().unwrap().processing_visits;
        assert!(after > before && after <= 1_000_000);
        assert_eq!(after, environment.generation_resources().usage().unwrap().1);
        FONT_PAIR_RESOLUTION_CALLS.with(|calls| assert_eq!(calls.get(), 2));
        let core = environment.core.read().unwrap();
        assert_eq!(
            core.prepared_pairs
                .iter()
                .map(|pair| pair.resolved.provenance.pair_ordinal)
                .collect::<Vec<_>>(),
            [4, 2]
        );
        assert_eq!(core.prepared_pairs[0].resolved.selected_scalars.len(), 1);
        assert_eq!(core.prepared_pairs[1].resolved.selected_scalars.len(), 2);
        assert!(matches!(
            core.prepared_pairs[1].resolved.selected_scalars[0].selection,
            ScalarFaceSelection::NonRenderingMandatoryBreak
        ));
    }

    #[test]
    fn batch_pair_preparation_rejects_duplicates_before_resolution() {
        let environment = PreparedFontEnvironment::synthetic();
        let stack = named_stack(&["Test Sans"]);
        let scalars = ['A'];
        let before = environment.resource_usage().unwrap().processing_visits;
        FONT_PAIR_RESOLUTION_CALLS.with(|calls| calls.set(0));

        let error = environment
            .prepare_pairs(
                [
                    FontPairPreparationRequest::new(0, &stack, RequestedWeight::Normal, &scalars),
                    FontPairPreparationRequest::new(1, &stack, RequestedWeight::Normal, &scalars),
                ]
                .into_iter(),
                None,
            )
            .unwrap_err();

        assert!(matches!(
            error,
            FontError::DuplicatePreparedPair { pair_ordinal: 1 }
        ));
        assert_eq!(
            environment.resource_usage().unwrap().processing_visits,
            before + 1 + 1 // one prior-request visit and the identical immutable stack handle
        );
        FONT_PAIR_RESOLUTION_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        assert!(environment.core.read().unwrap().prepared_pairs.is_empty());
    }

    #[test]
    fn batch_counts_every_pair_before_first_pair_materialization() {
        let environment = PreparedFontEnvironment::synthetic();
        let stack = named_stack(&["Test Sans"]);
        let scalars = ['A'];
        environment
            .claim_processing_visits(
                999_990 - environment.resource_usage().unwrap().processing_visits,
            )
            .unwrap();

        let error = environment
            .prepare_pairs(
                [
                    FontPairPreparationRequest::new(0, &stack, RequestedWeight::Normal, &scalars),
                    FontPairPreparationRequest::new(1, &stack, RequestedWeight::Bold, &scalars),
                ]
                .into_iter(),
                None,
            )
            .unwrap_err();

        assert!(matches!(error, FontError::Limit { .. }));
        assert!(environment.core.read().unwrap().prepared_pairs.is_empty());
    }

    #[test]
    fn generated_font_resolves_and_shapes_without_external_assets() {
        let mut core = generated_core();
        let pair = core
            .resolve_pair(
                0,
                &named_stack(&["TEST SANS"]),
                RequestedWeight::Normal,
                &['A'],
                true,
            )
            .unwrap();
        let run = core
            .shape_run(
                pair.primary,
                "A",
                ShapingDirection::LeftToRight,
                rustybuzz::script::LATIN,
            )
            .unwrap();
        assert_eq!(run.identity, pair.primary);
        assert_eq!(run.metrics.units_per_em, 1000);
        assert_eq!(run.glyphs.len(), 1);
        assert_eq!(run.glyphs[0].glyph_id, 1);
        assert_eq!(run.glyphs[0].cluster, 0);
        assert_eq!(run.glyphs[0].x_advance, 600);
        assert_eq!(
            run.glyphs[0].ink,
            Some(GlyphInk {
                x_min: 0,
                y_min: 0,
                x_max: 500,
                y_max: 700
            })
        );
    }

    #[test]
    fn host_database_generic_smoke() {
        let mut core = CanonicalFontCore::discover_system().unwrap();
        let family =
            super::super::unicode16::folded_for_test(core.database.family_name(&Family::SansSerif));
        let matches = core
            .faces
            .iter()
            .filter(|face| {
                face.names
                    .iter()
                    .any(|candidate| candidate.folded == family)
            })
            .count();
        let resolved = core.resolve_pair(
            0,
            &FontStack::new([FontFamily::generic(GenericFontFamily::SansSerif)]).unwrap(),
            RequestedWeight::Normal,
            &['A'],
            true,
        );
        if matches == 0 {
            assert!(matches!(resolved, Err(FontError::MissingFamily { .. })));
        } else {
            let resolved = resolved.unwrap();
            assert_eq!(resolved.primary, resolved.provenance.candidates[0]);
        }
    }

    fn identity(byte: u8, face_index: u32) -> FaceIdentity {
        FaceIdentity {
            sha256: [byte; 32],
            face_index,
        }
    }

    fn scalar_presence_map() -> Vec<u64> {
        let mut presence = Vec::new();
        presence.try_reserve_exact(SCALAR_MAP_WORDS).unwrap();
        presence.resize(SCALAR_MAP_WORDS, 0);
        presence
    }

    fn candidate(
        identity: FaceIdentity,
        stack_entry_index: u32,
        family_match_rank: u8,
        weight_distance: u16,
    ) -> CandidateFace {
        CandidateFace {
            occurrence: CandidateOccurrence {
                stack_entry_index,
                family_match_rank,
                style_distance: 0,
                stretch_distance: 0,
                weight_distance,
                identity,
            },
            face_ordinal: 0,
        }
    }

    fn fake_face(identity: FaceIdentity, name: &str) -> FaceMetadata {
        FaceMetadata {
            identity,
            file_ordinal: 0,
            database_id: ID::dummy(),
            names: vec![NameOccurrence {
                value: name.into(),
                folded: super::super::unicode16::folded_for_test(name),
                rank: NameRank::TypographicFamily,
                record_ordinal: 0,
            }],
            style: Style::Normal,
            stretch: Stretch::Normal,
            weight: 400,
            metrics: DesignMetrics {
                units_per_em: 1000,
                ascender: 800,
                descender: -200,
            },
        }
    }

    pub(super) fn generated_core() -> CanonicalFontCore {
        let bytes = Arc::new(build_test_font());
        let sha256: [u8; 32] = Sha256::digest(bytes.as_slice()).into();
        CanonicalFontCore::from_retained(
            vec![RetainedFontFile {
                root_ordinal: 0,
                canonical_path: PathBuf::from("generated-test.ttf"),
                bytes,
                sha256,
            }],
            &HostGenericNames::default(),
            FontBudget::new(ResourceLimits::PRODUCTION),
            scalar_presence_map(),
        )
        .unwrap()
    }

    #[test]
    fn formatted_census_counts_units_across_write_boundaries() {
        for mark in (0..=0x10_FFFF)
            .filter_map(char::from_u32)
            .filter(|scalar| is_visible_mark(*scalar))
        {
            for base in [
                'A',
                '\n',
                '\u{0301}',
                '\u{0600}',
                '\u{0915}',
                '\u{200D}',
                '\u{1F469}',
            ] {
                let source = format!("{base}{mark}\u{0301}");
                let mut count = CensusFormatCount::default();
                for scalar in source.chars() {
                    write!(&mut count, "{scalar}").unwrap();
                }
                assert_eq!(count.bytes, source.len());
                assert_eq!(count.scalars, 3);
                assert_eq!(
                    usize::try_from(count.units).unwrap(),
                    selection_units(&source).count(),
                    "{source:?}"
                );
            }
        }
    }

    #[test]
    fn formatted_census_admits_work_and_bytes_before_output_writes() {
        struct Observed<'a>(&'a std::cell::Cell<usize>);
        impl std::fmt::Display for Observed<'_> {
            fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.set(self.0.get() + 1);
                output.write_str("A")?;
                output.write_str("\u{0301}")
            }
        }
        for refusal in [None, Some("work"), Some("bytes")] {
            let result = crate::layout::plan_exact_root(
                320,
                240,
                |stage| {
                    if refusal == Some("work") {
                        stage
                            .font_resources()
                            .claim_visits(PROCESSING_VISIT_LIMIT)
                            .unwrap();
                    } else if refusal == Some("bytes") {
                        stage.font_resources().claim_bytes(FONT_BYTE_LIMIT).unwrap();
                    }
                    let writes = std::cell::Cell::new(0);
                    let result =
                        format_census_source(format_args!("{}", Observed(&writes)), Some(stage));
                    if refusal.is_some() {
                        assert!(matches!(result, Err(FontError::Limit { .. })));
                        assert_eq!(
                            writes.get(),
                            1,
                            "only the allocation-free count pass may run"
                        );
                    } else {
                        assert_eq!(result.unwrap(), "A\u{0301}");
                        assert_eq!(writes.get(), 2);
                        let usage = stage.font_resources().usage().unwrap();
                        assert_eq!(usage.font_bytes, 3);
                        assert_eq!(usage.processing_visits, 3);
                    }
                    Ok(())
                },
                |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
                |(), _, (), _| Ok(()),
            );
            if refusal == Some("bytes") {
                assert!(result
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("system-font bytes"));
            } else {
                result.unwrap();
            }
        }
    }

    #[test]
    fn retained_units_prefer_complete_faces_without_changing_scalar_or_primary_selection() {
        let files = [
            ("Base", vec!['A']),
            ("Complete", vec!['A', '\u{0301}']),
            ("Mark", vec!['\u{0301}']),
        ]
        .into_iter()
        .map(|(family, coverage)| {
            let bytes = Arc::new(build_test_font_with_coverage(family, &coverage));
            RetainedFontFile {
                root_ordinal: 0,
                canonical_path: PathBuf::from("never-on-disk.ttf"),
                sha256: Sha256::digest(bytes.as_slice()).into(),
                bytes,
            }
        })
        .collect();
        let environment = PreparedFontEnvironment {
            core: Arc::new(RwLock::new(
                CanonicalFontCore::from_retained(
                    files,
                    &HostGenericNames::default(),
                    FontBudget::new(ResourceLimits::PRODUCTION),
                    scalar_presence_map(),
                )
                .unwrap(),
            )),
        };
        let complete = named_stack(&["Base", "Complete"]);
        let split = named_stack(&["Base", "Mark"]);
        environment
            .prepare_pairs(
                [
                    FontPairPreparationRequest::new(
                        0,
                        &complete,
                        RequestedWeight::Normal,
                        &['A', '\u{0301}', '\u{200D}', '\u{FE0F}'],
                    ),
                    FontPairPreparationRequest::new(
                        1,
                        &split,
                        RequestedWeight::Normal,
                        &['A', '\u{0301}'],
                    ),
                ]
                .into_iter(),
                None,
            )
            .unwrap();
        for ordinal in 0..2 {
            environment
                .prepare_source_units(ordinal, "A A\u{0301}", None)
                .unwrap();
        }
        environment.seal_prepared_pairs().unwrap();
        let pair = environment
            .prepared_pair(&complete, RequestedWeight::Normal)
            .unwrap()
            .unwrap();
        assert_eq!(
            pair.selection_for_unit("A", 'A').unwrap(),
            ScalarFaceSelection::Face(pair.primary)
        );
        let unit_face = pair.provenance.candidates[1];
        for scalar in ['A', '\u{0301}'] {
            assert_eq!(
                pair.selection_for_unit("A\u{0301}", scalar).unwrap(),
                ScalarFaceSelection::Face(unit_face)
            );
        }
        assert!(matches!(
            pair.selection_for_unit("A\u{0301}\u{0301}", 'A'),
            Err(FontError::UnknownUnit { .. })
        ));
        assert!(matches!(
            environment.prepare_source_units(0, "A\u{0301}", None),
            Err(FontError::PreparedPairAfterSeal { .. })
        ));
        let split_pair = environment
            .prepared_pair(&split, RequestedWeight::Normal)
            .unwrap()
            .unwrap();
        assert_eq!(
            split_pair.selection_for_unit("A\u{0301}", 'A').unwrap(),
            ScalarFaceSelection::Face(split_pair.primary)
        );
        assert_eq!(
            split_pair
                .selection_for_unit("A\u{0301}", '\u{0301}')
                .unwrap(),
            ScalarFaceSelection::Face(split_pair.provenance.candidates[1])
        );
        environment
            .shape_run(
                unit_face,
                "A\u{0301}",
                ShapingDirection::LeftToRight,
                rustybuzz::script::LATIN,
            )
            .unwrap();
    }

    #[test]
    fn units_are_narrow_and_controls_do_not_require_nominal_coverage() {
        assert_eq!(
            selection_units("A\u{0301}B\u{0301}\u{200D}A\u{FE0F}").collect::<Vec<_>>(),
            ["A\u{0301}", "B\u{0301}", "\u{200D}", "A", "\u{FE0F}"]
        );
        let mut core = generated_core();
        let stack = named_stack(&["Test Sans"]);
        core.resolve_pair(
            0,
            &stack,
            RequestedWeight::Normal,
            &['\n', '\u{200C}', '\u{200D}', '\u{FE0F}', '\u{2067}'],
            true,
        )
        .unwrap();
        assert!(matches!(
            core.resolve_pair(0, &stack, RequestedWeight::Normal, &['\u{0301}'], true),
            Err(FontError::MissingGlyph { .. })
        ));
        assert!(matches!(
            core.resolve_pair(0, &stack, RequestedWeight::Normal, &['\t'], true),
            Err(FontError::UnsupportedControl { .. })
        ));
    }

    #[test]
    fn decoder_may_admit_zero_faces_from_a_retained_file() {
        let mut files = generated_core().files;
        files.push(RetainedFontFile {
            root_ordinal: 0,
            canonical_path: PathBuf::from("unsupported.ttf"),
            bytes: Arc::new(vec![0; 16]),
            sha256: [0; 32],
        });
        let core = CanonicalFontCore::from_retained(
            files,
            &HostGenericNames::default(),
            FontBudget::new(ResourceLimits::PRODUCTION),
            scalar_presence_map(),
        )
        .unwrap();
        assert_eq!(core.faces.len(), 1);
    }

    #[test]
    fn partial_collection_preserves_actual_decoder_face_index() {
        let mut font = build_test_font();
        let tables = usize::from(u16::from_be_bytes(font[4..6].try_into().unwrap()));
        for table in 0..tables {
            let position = 12 + table * 16 + 8;
            let offset = u32::from_be_bytes(font[position..position + 4].try_into().unwrap());
            set_u32(&mut font, position, offset + 32);
        }
        let mut collection = b"ttcf".to_vec();
        push_u32(&mut collection, 0x0001_0000);
        push_u32(&mut collection, 2);
        push_u32(&mut collection, 20);
        push_u32(&mut collection, 32);
        collection.extend_from_slice(&[0; 12]);
        collection.extend_from_slice(&font);
        let bytes = Arc::new(collection);
        let core = CanonicalFontCore::from_retained(
            vec![RetainedFontFile {
                root_ordinal: 0,
                canonical_path: PathBuf::from("partial.ttc"),
                sha256: Sha256::digest(bytes.as_slice()).into(),
                bytes,
            }],
            &HostGenericNames::default(),
            FontBudget::new(ResourceLimits::PRODUCTION),
            scalar_presence_map(),
        )
        .unwrap();
        assert_eq!(core.faces.len(), 1);
        assert_eq!(core.faces[0].identity.face_index, 1);
        assert_eq!(
            core.database.face(core.faces[0].database_id).unwrap().index,
            1
        );
        core.shape_run(
            core.faces[0].identity,
            "A",
            ShapingDirection::LeftToRight,
            rustybuzz::script::LATIN,
        )
        .unwrap();
    }

    fn build_test_font() -> Vec<u8> {
        build_test_font_with_coverage("Test Sans", &[' ', 'A', '\u{05D0}'])
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn build_test_font_with_coverage(family_name: &str, coverage: &[char]) -> Vec<u8> {
        let family = utf16_be(family_name);
        let postscript = utf16_be("TestSans-Regular");
        let mut name = Vec::new();
        push_u16(&mut name, 0);
        push_u16(&mut name, 3);
        push_u16(&mut name, 42);
        let mut offset = 0_u16;
        for (name_id, value) in [(1_u16, &family), (16_u16, &family), (6_u16, &postscript)] {
            push_u16(&mut name, 3);
            push_u16(&mut name, 1);
            push_u16(&mut name, 0x0409);
            push_u16(&mut name, name_id);
            push_u16(&mut name, u16::try_from(value.len()).unwrap());
            push_u16(&mut name, offset);
            offset += u16::try_from(value.len()).unwrap();
        }
        name.extend_from_slice(&family);
        name.extend_from_slice(&family);
        name.extend_from_slice(&postscript);

        let mut cmap = Vec::new();
        push_u16(&mut cmap, 0);
        push_u16(&mut cmap, 1);
        push_u16(&mut cmap, 3);
        push_u16(&mut cmap, 10);
        push_u32(&mut cmap, 12);
        push_u16(&mut cmap, 12);
        push_u16(&mut cmap, 0);
        push_u32(&mut cmap, u32::try_from(16 + coverage.len() * 12).unwrap());
        push_u32(&mut cmap, 0);
        push_u32(&mut cmap, u32::try_from(coverage.len()).unwrap());
        for scalar in coverage {
            push_u32(&mut cmap, *scalar as u32);
            push_u32(&mut cmap, *scalar as u32);
            push_u32(&mut cmap, if *scalar == ' ' { 2 } else { 1 });
        }

        let mut head = vec![0_u8; 54];
        set_u32(&mut head, 0, 0x0001_0000);
        set_u32(&mut head, 12, 0x5F0F_3CF5);
        set_u16(&mut head, 18, 1000);
        set_i16(&mut head, 36, 0);
        set_i16(&mut head, 38, 0);
        set_i16(&mut head, 40, 500);
        set_i16(&mut head, 42, 700);
        set_i16(&mut head, 50, 0);

        let mut hhea = vec![0_u8; 36];
        set_u32(&mut hhea, 0, 0x0001_0000);
        set_i16(&mut hhea, 4, 800);
        set_i16(&mut hhea, 6, -200);
        set_u16(&mut hhea, 10, 600);
        set_u16(&mut hhea, 34, 3);

        let mut maxp = vec![0_u8; 32];
        set_u32(&mut maxp, 0, 0x0001_0000);
        set_u16(&mut maxp, 4, 3);

        let mut os2 = vec![0_u8; 78];
        set_u16(&mut os2, 0, 0);
        set_u16(&mut os2, 4, 400);
        set_u16(&mut os2, 6, 5);
        set_u16(&mut os2, 62, 0x0040);
        set_u16(&mut os2, 64, 0x0020);
        set_u16(&mut os2, 66, 0x0041);
        set_i16(&mut os2, 68, 800);
        set_i16(&mut os2, 70, -200);
        set_u16(&mut os2, 74, 800);
        set_u16(&mut os2, 76, 200);

        let hmtx = vec![0x01, 0xF4, 0, 0, 0x02, 0x58, 0, 0, 0, 0xFA, 0, 0];
        let loca = vec![0, 0, 0, 0, 0, 17, 0, 17];
        let mut glyf = vec![0_u8; 34];
        set_i16(&mut glyf, 0, 1);
        set_i16(&mut glyf, 2, 0);
        set_i16(&mut glyf, 4, 0);
        set_i16(&mut glyf, 6, 500);
        set_i16(&mut glyf, 8, 700);
        set_u16(&mut glyf, 10, 3);
        set_u16(&mut glyf, 12, 0);
        glyf[14..18].fill(1);
        for (offset, value) in [(18, 0), (20, 500), (22, 0), (24, -500)] {
            set_i16(&mut glyf, offset, value);
        }
        for (offset, value) in [(26, 0), (28, 0), (30, 700), (32, 0)] {
            set_i16(&mut glyf, offset, value);
        }

        build_sfnt(vec![
            (*b"OS/2", os2),
            (*b"cmap", cmap),
            (*b"glyf", glyf),
            (*b"head", head),
            (*b"hhea", hhea),
            (*b"hmtx", hmtx),
            (*b"loca", loca),
            (*b"maxp", maxp),
            (*b"name", name),
        ])
    }

    fn build_sfnt(mut tables: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
        tables.sort_by_key(|(tag, _)| *tag);
        let directory_len = 12 + tables.len() * 16;
        let mut offset = directory_len;
        let mut records = Vec::new();
        for (tag, data) in &tables {
            records.push((*tag, offset, data.len()));
            offset += (data.len() + 3) & !3;
        }
        let mut font = Vec::with_capacity(offset);
        push_u32(&mut font, 0x0001_0000);
        push_u16(&mut font, u16::try_from(tables.len()).unwrap());
        push_u16(&mut font, 0);
        push_u16(&mut font, 0);
        push_u16(&mut font, 0);
        for (tag, offset, length) in &records {
            font.extend_from_slice(tag);
            push_u32(&mut font, 0);
            push_u32(&mut font, u32::try_from(*offset).unwrap());
            push_u32(&mut font, u32::try_from(*length).unwrap());
        }
        for (_, data) in tables {
            font.extend_from_slice(&data);
            while font.len() % 4 != 0 {
                font.push(0);
            }
        }
        font
    }

    fn utf16_be(value: &str) -> Vec<u8> {
        value.encode_utf16().flat_map(u16::to_be_bytes).collect()
    }

    fn push_u16(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn push_u32(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn set_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }

    fn set_i16(bytes: &mut [u8], offset: usize, value: i16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    }

    fn set_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
    }
}
