// This is the private T5a3 integration surface; later text planning consumes it.
#![allow(dead_code)]

use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};

use fontdb::{Database, Family, Source, Stretch, Style, ID};
use sha2::{Digest, Sha256};

#[cfg(test)]
thread_local! {
    pub(crate) static FONT_DISCOVERY_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static FONT_PAIR_RESOLUTION_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static FONT_SHAPE_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static FONT_OUTLINE_GLYPHS: std::cell::RefCell<Vec<(FaceIdentity, u16)>> = const { std::cell::RefCell::new(Vec::new()) };
}

const DISCOVERY_ENTRY_LIMIT: u64 = 1_000_000;
const FONT_BYTE_LIMIT: u64 = 536_870_912;
const FONT_WORK_LIMIT: u64 = 1_000_000;
const TEXT_GENERATION_LIMIT: u128 = 1_000_000;
const SCALAR_MAP_WORDS: usize = 0x11_0000 / u64::BITS as usize;

#[derive(Debug, thiserror::Error)]
pub(crate) enum FontError {
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

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum FamilyStackError {
    #[error("font stack is empty")]
    Empty,
    #[error("font stack entry {entry_index} is empty")]
    EmptyEntry { entry_index: usize },
    #[error("font stack entry {entry_index} contains a control character")]
    ControlCharacter { entry_index: usize },
    #[error("font stack entry {entry_index} uses unsupported CSS generic family '{name}'")]
    UnsupportedGeneric { entry_index: usize, name: String },
    #[error("font stack allocation refused for {0}")]
    Allocation(&'static str),
    #[error("{0}")]
    Reservation(String),
}

#[derive(Debug, Clone, Copy)]
struct ResourceLimits {
    discovery_entries: u64,
    font_bytes: u64,
    logical_work: u64,
}

impl ResourceLimits {
    const PRODUCTION: Self = Self {
        discovery_entries: DISCOVERY_ENTRY_LIMIT,
        font_bytes: FONT_BYTE_LIMIT,
        logical_work: FONT_WORK_LIMIT,
    };
}

#[derive(Debug)]
struct FontBudget {
    resources: OperationFontResources,
}

impl FontBudget {
    fn new(limits: ResourceLimits) -> Self {
        Self {
            resources: OperationFontResources::with_limits(limits),
        }
    }

    fn from_resources(resources: OperationFontResources) -> Self {
        Self { resources }
    }

    fn claim_discovery(&mut self, amount: u64) -> Result<(), FontError> {
        self.resources.claim_discovery(amount)?;
        self.claim_work(amount)
    }

    fn claim_bytes(&mut self, amount: u64) -> Result<(), FontError> {
        self.resources.claim_bytes(amount)
    }

    fn claim_work(&mut self, amount: u64) -> Result<(), FontError> {
        self.resources.claim_work(amount)
    }
}

#[derive(Debug)]
struct FontResourceCounters {
    limits: ResourceLimits,
    discovery_entries: u64,
    font_bytes: u64,
    logical_work: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct OperationFontResources {
    counters: Arc<Mutex<FontResourceCounters>>,
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
                font_bytes: 0,
                logical_work: 0,
            })),
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
        let mut counters = self
            .counters
            .lock()
            .map_err(|_| FontError::Synchronization)?;
        let limit = counters.limits.font_bytes;
        claim_font_resource(&mut counters.font_bytes, amount, limit, "system-font bytes")
    }

    pub(crate) fn claim_work(&self, amount: u64) -> Result<(), FontError> {
        let mut counters = self
            .counters
            .lock()
            .map_err(|_| FontError::Synchronization)?;
        let limit = counters.limits.logical_work;
        claim_font_resource(
            &mut counters.logical_work,
            amount,
            limit,
            "font-preflight logical work units",
        )
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
        Ok(FontResourceUsage {
            discovery_entries: u128::from(counters.discovery_entries),
            font_bytes: u128::from(counters.font_bytes),
            logical_work: u128::from(counters.logical_work),
        })
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
    if total > limit {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FamilyPreference {
    Named(String),
    Serif,
    SansSerif,
    Monospace,
}

pub(crate) fn parse_family_stack(stack: &str) -> Result<Vec<FamilyPreference>, FamilyStackError> {
    parse_family_stack_with_stage(stack, None)
}

pub(crate) fn parse_family_stack_with_stage(
    stack: &str,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<Vec<FamilyPreference>, FamilyStackError> {
    if stack.trim().is_empty() {
        return Err(FamilyStackError::Empty);
    }
    let entry_count = stack.split(',').count();
    let mut parsed = Vec::new();
    if let Some(stage) = prepared_stage.as_deref_mut() {
        stage
            .reserve_prepared(&mut parsed, entry_count, "prepared font stack entries")
            .map_err(|error| FamilyStackError::Reservation(error.to_string()))?;
    } else {
        parsed
            .try_reserve_exact(entry_count)
            .map_err(|_| FamilyStackError::Allocation("font stack entries"))?;
    }

    for (entry_index, entry) in stack.split(',').enumerate() {
        let entry = entry.trim();
        if entry.is_empty() {
            return Err(FamilyStackError::EmptyEntry { entry_index });
        }
        if entry.chars().any(char::is_control) {
            return Err(FamilyStackError::ControlCharacter { entry_index });
        }
        let folded = entry.to_ascii_lowercase();
        let preference = match folded.as_str() {
            "serif" => FamilyPreference::Serif,
            "sans-serif" => FamilyPreference::SansSerif,
            "monospace" => FamilyPreference::Monospace,
            "cursive" | "fantasy" | "system-ui" | "ui-serif" | "ui-sans-serif" | "ui-monospace"
            | "ui-rounded" | "math" | "fangsong" => {
                return Err(FamilyStackError::UnsupportedGeneric {
                    entry_index,
                    name: entry.to_owned(),
                });
            }
            _ => {
                let mut name = String::new();
                if let Some(stage) = prepared_stage.as_deref_mut() {
                    stage
                        .reserve_prepared_string(
                            &mut name,
                            entry.len(),
                            "prepared font family name",
                        )
                        .map_err(|error| FamilyStackError::Reservation(error.to_string()))?;
                } else {
                    name.try_reserve_exact(entry.len())
                        .map_err(|_| FamilyStackError::Allocation("font family name"))?;
                }
                name.push_str(entry);
                FamilyPreference::Named(name)
            }
        };
        parsed.push(preference);
    }
    Ok(parsed)
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

struct OpenedFontFile {
    root_ordinal: u32,
    canonical_path: PathBuf,
    file: File,
    length: usize,
    modified: Option<std::time::SystemTime>,
    bytes: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct CanonicalFontCore {
    database: Database,
    files: Vec<RetainedFontFile>,
    faces: Vec<FaceMetadata>,
    scalar_presence: Vec<u64>,
    prepared_pairs: Vec<PreparedPair>,
    prepared_pairs_sealed: bool,
    generation_resources: OperationGenerationResources,
    budget: FontBudget,
}

#[derive(Debug, Clone)]
struct PreparedPair {
    stack: Vec<FamilyPreference>,
    weight: RequestedWeight,
    resolved: ResolvedFontPair,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FontPairPreparationRequest<'a> {
    pair_ordinal: u32,
    stack: &'a [FamilyPreference],
    weight: RequestedWeight,
    required_scalars: &'a [char],
}

impl<'a> FontPairPreparationRequest<'a> {
    pub(crate) const fn new(
        pair_ordinal: u32,
        stack: &'a [FamilyPreference],
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
    retained_stack: Vec<FamilyPreference>,
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
                .all(|(retained, source)| match (retained, source) {
                    (FamilyPreference::Named(retained), FamilyPreference::Named(source)) => {
                        retained.capacity() >= source.len()
                    }
                    (FamilyPreference::Serif, FamilyPreference::Serif)
                    | (FamilyPreference::SansSerif, FamilyPreference::SansSerif)
                    | (FamilyPreference::Monospace, FamilyPreference::Monospace) => true,
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
    pub(crate) logical_work: u128,
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

#[derive(Debug, Default)]
struct GenerationResourceCounters {
    slots: u128,
    work: u128,
}

#[derive(Debug, Clone)]
pub(crate) struct OperationGenerationResources {
    counters: Arc<Mutex<GenerationResourceCounters>>,
}

impl OperationGenerationResources {
    pub(crate) fn new() -> Self {
        Self {
            counters: Arc::new(Mutex::new(GenerationResourceCounters::default())),
        }
    }

    pub(crate) fn claim_slots(&self, amount: u128) -> Result<(), TextGenerationClaimError> {
        let mut counters = self
            .counters
            .lock()
            .map_err(|_| TextGenerationClaimError::Arithmetic("text resource synchronization"))?;
        claim_text_generation_resource(&mut counters.slots, amount, "candidate-generation slots")
    }

    pub(crate) fn claim_work(&self, amount: u128) -> Result<(), TextGenerationClaimError> {
        let mut counters = self
            .counters
            .lock()
            .map_err(|_| TextGenerationClaimError::Arithmetic("text resource synchronization"))?;
        claim_text_generation_resource(
            &mut counters.work,
            amount,
            "candidate-generation work units",
        )
    }

    #[cfg(test)]
    pub(crate) fn usage(&self) -> Result<(u128, u128), TextGenerationClaimError> {
        let counters = self
            .counters
            .lock()
            .map_err(|_| TextGenerationClaimError::Arithmetic("text resource synchronization"))?;
        Ok((counters.slots, counters.work))
    }
}

impl PreparedFontEnvironment {
    pub(crate) fn discover_system() -> Result<Self, FontError> {
        #[cfg(test)]
        FONT_DISCOVERY_CALLS.with(|calls| calls.set(calls.get() + 1));
        let scalar_presence = scalar_presence_map()?;
        Ok(Self {
            core: Arc::new(RwLock::new(
                CanonicalFontCore::discover_system_with_resources(
                    FontBudget::new(ResourceLimits::PRODUCTION),
                    scalar_presence,
                    OperationGenerationResources::new(),
                    None,
                    None,
                )?,
            )),
        })
    }

    pub(crate) fn discover_system_with_prepared_resources(
        scalar_presence: Vec<u64>,
        font_resources: OperationFontResources,
        generation_resources: OperationGenerationResources,
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
                    generation_resources,
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
        stack: &[FamilyPreference],
        weight: RequestedWeight,
        required_scalars: &[char],
    ) -> Result<ResolvedFontPair, FontError> {
        let mut core = self.core.write().map_err(|_| FontError::Synchronization)?;
        if core.prepared_pairs_sealed {
            return Err(FontError::PreparedPairAfterSeal { pair_ordinal });
        }
        core.resolve_pair(pair_ordinal, stack, weight, required_scalars, true)
    }

    pub(crate) fn claim_preflight_work(&self, amount: u64) -> Result<(), FontError> {
        self.core
            .write()
            .map_err(|_| FontError::Synchronization)?
            .budget
            .claim_work(amount)
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

    pub(crate) fn claim_text_generation_work(
        &self,
        amount: u128,
    ) -> Result<(), TextGenerationClaimError> {
        let resources = self
            .core
            .read()
            .map_err(|_| TextGenerationClaimError::Arithmetic("text resource synchronization"))?
            .generation_resources
            .clone();
        resources.claim_work(amount)
    }

    pub(crate) fn share_generation_resources(
        &self,
        resources: OperationGenerationResources,
    ) -> Result<(), FontError> {
        self.core
            .write()
            .map_err(|_| FontError::Synchronization)?
            .generation_resources = resources;
        Ok(())
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
        stack: &[FamilyPreference],
        weight: RequestedWeight,
        required_scalars: &[char],
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), FontError> {
        let mut core = self.core.write().map_err(|_| FontError::Synchronization)?;
        if core.prepared_pairs_sealed {
            return Err(FontError::PreparedPairAfterSeal { pair_ordinal });
        }
        if core.prepared_pairs.iter().any(|prepared| {
            prepared.resolved.provenance.pair_ordinal == pair_ordinal
                || (prepared.stack == stack && prepared.weight == weight)
        }) {
            return Err(FontError::DuplicatePreparedPair { pair_ordinal });
        }
        let resolved = core.resolve_pair_with_stage(
            pair_ordinal,
            stack,
            weight,
            required_scalars,
            false,
            prepared_stage.as_deref_mut(),
        )?;
        let mut retained_stack = Vec::new();
        reserve_prepared_or_local(
            &mut retained_stack,
            stack.len(),
            "prepared font stack",
            prepared_stage,
        )?;
        retained_stack.extend_from_slice(stack);
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

    pub(crate) fn prepared_pairs_are_sealed(&self) -> Result<bool, FontError> {
        Ok(self
            .core
            .read()
            .map_err(|_| FontError::Synchronization)?
            .prepared_pairs_sealed)
    }

    pub(crate) fn prepared_pair(
        &self,
        stack: &[FamilyPreference],
        weight: RequestedWeight,
    ) -> Result<Option<ResolvedFontPair>, FontError> {
        Ok(self
            .core
            .read()
            .map_err(|_| FontError::Synchronization)?
            .prepared_pairs
            .iter()
            .find(|prepared| prepared.stack == stack && prepared.weight == weight)
            .map(|prepared| prepared.resolved.clone()))
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
        #[cfg(test)]
        FONT_SHAPE_CALLS.with(|calls| calls.set(calls.get() + 1));
        self.core
            .read()
            .map_err(|_| FontError::Synchronization)?
            .shape_run(identity, text, direction, script)
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
}

#[allow(dead_code)]
impl CanonicalFontCore {
    pub(crate) fn discover_system() -> Result<Self, FontError> {
        Self::discover_system_with_resources(
            FontBudget::new(ResourceLimits::PRODUCTION),
            scalar_presence_map()?,
            OperationGenerationResources::new(),
            None,
            None,
        )
    }

    fn discover_system_with_resources(
        mut budget: FontBudget,
        scalar_presence: Vec<u64>,
        generation_resources: OperationGenerationResources,
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
            generation_resources,
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
            if self.prepared_pairs.iter().any(|prepared| {
                prepared.resolved.provenance.pair_ordinal == request.pair_ordinal
                    || (prepared.stack == request.stack && prepared.weight == request.weight)
            }) || preparations
                .iter()
                .any(|preparation: &PairPreparation<'_>| {
                    preparation.request.pair_ordinal == request.pair_ordinal
                        || (preparation.request.stack == request.stack
                            && preparation.request.weight == request.weight)
                })
            {
                return Err(FontError::DuplicatePreparedPair {
                    pair_ordinal: request.pair_ordinal,
                });
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
            )?;
            preparation.raw_candidate_count = count_candidate_occurrences(
                &self.faces,
                &preparation.resolved_names,
                &mut self.budget,
            )?;
            preparation.candidate_count =
                count_unique_candidate_survivors(&self.faces, &preparation.resolved_names)?;
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
            reserve_prepared_or_local(
                &mut preparation.retained_stack,
                preparation.request.stack.len(),
                "prepared font stack",
                prepared_stage.as_deref_mut(),
            )?;
            reserve_family_stack_names(
                &mut preparation.retained_stack,
                preparation.request.stack,
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
                    let face = &faces[candidate.face_ordinal];
                    let file = &files[face.file_ordinal];
                    let parsed = ttf_parser::Face::parse(&file.bytes, face.identity.face_index)
                        .map_err(|error| FontError::ParseFace {
                            path: file.canonical_path.clone(),
                            index: face.identity.face_index,
                            reason: error.to_string(),
                        })?;
                    Ok(parsed.glyph_index(scalar).is_some())
                },
            )?;
            fill_reserved_family_stack(&mut preparation.retained_stack, request.stack)?;
            self.prepared_pairs.push(PreparedPair {
                stack: preparation.retained_stack,
                weight: request.weight,
                resolved: ResolvedFontPair {
                    provenance: PairProvenance {
                        pair_ordinal: request.pair_ordinal,
                        weight: request.weight,
                        candidates: preparation.provenance,
                    },
                    primary,
                    selected_scalars: SelectedScalarFaces(preparation.selected_scalars),
                },
            });
        }
        Ok(())
    }

    pub(crate) fn resolve_pair(
        &mut self,
        pair_ordinal: u32,
        stack: &[FamilyPreference],
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

    fn resolve_pair_with_stage(
        &mut self,
        pair_ordinal: u32,
        stack: &[FamilyPreference],
        weight: RequestedWeight,
        required_scalars: &[char],
        charge_required_scalars: bool,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<ResolvedFontPair, FontError> {
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

        let mut provenance = Vec::new();
        reserve_prepared_or_local(
            &mut provenance,
            candidates.len(),
            "font provenance",
            prepared_stage.as_deref_mut(),
        )?;
        provenance.extend(
            candidates
                .iter()
                .map(|candidate| candidate.occurrence.identity),
        );
        let primary = provenance[0];
        let files = &self.files;
        let faces = &self.faces;
        let selected_scalars = select_scalar_faces_with_stage(
            pair_ordinal,
            &candidates,
            required_scalars,
            &mut self.scalar_presence,
            &mut self.budget,
            charge_required_scalars,
            prepared_stage,
            |candidate, scalar| {
                let face = &faces[candidate.face_ordinal];
                let file = &files[face.file_ordinal];
                let parsed = ttf_parser::Face::parse(&file.bytes, face.identity.face_index)
                    .map_err(|error| FontError::ParseFace {
                        path: file.canonical_path.clone(),
                        index: face.identity.face_index,
                        reason: error.to_string(),
                    })?;
                Ok(parsed.glyph_index(scalar).is_some())
            },
        )?;

        Ok(ResolvedFontPair {
            provenance: PairProvenance {
                pair_ordinal,
                weight,
                candidates: provenance,
            },
            primary,
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
        let metadata = self
            .faces
            .iter()
            .find(|face| face.identity == identity)
            .ok_or(FontError::Shape {
                identity,
                reason: "identity is not retained",
            })?;
        let file = &self.files[metadata.file_ordinal];
        shape_retained_run(file, metadata, text, direction, script)
    }

    pub(crate) fn face_metrics(&self, identity: FaceIdentity) -> Result<DesignMetrics, FontError> {
        self.faces
            .iter()
            .find(|face| face.identity == identity)
            .map(|face| face.metrics)
            .ok_or(FontError::Shape {
                identity,
                reason: "identity is not retained",
            })
    }

    fn outline_glyph(
        &self,
        identity: FaceIdentity,
        glyph_id: u16,
        builder: &mut dyn ttf_parser::OutlineBuilder,
    ) -> Result<(DesignMetrics, Option<GlyphInk>), FontError> {
        let metadata = self
            .faces
            .iter()
            .find(|face| face.identity == identity)
            .ok_or(FontError::Outline {
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
        generation_resources: OperationGenerationResources,
    ) -> Result<Self, FontError> {
        Self::from_retained_with_stage(
            files,
            generics,
            budget,
            scalar_presence,
            generation_resources,
            None,
        )
    }

    fn from_retained_with_stage(
        files: Vec<RetainedFontFile>,
        generics: &HostGenericNames,
        mut budget: FontBudget,
        scalar_presence: Vec<u64>,
        generation_resources: OperationGenerationResources,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<Self, FontError> {
        let mut database = Database::new();

        let face_capacity = files.iter().try_fold(0_usize, |total, file| {
            let count = ttf_parser::fonts_in_collection(&file.bytes).unwrap_or(1);
            let count =
                usize::try_from(count).map_err(|_| FontError::Arithmetic("font face count"))?;
            total
                .checked_add(count)
                .ok_or(FontError::Arithmetic("font face count"))
        })?;
        budget.claim_work(
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
            let expected = ttf_parser::fonts_in_collection(&file.bytes).unwrap_or(1);
            let ids = database.load_font_source(Source::Binary(Arc::new(BinarySource(
                Arc::clone(&file.bytes),
            ))));
            if ids.len()
                != usize::try_from(expected)
                    .map_err(|_| FontError::Arithmetic("fontdb face count"))?
            {
                return Err(FontError::DatabaseLoad(file.canonical_path.clone()));
            }

            for (index, database_id) in ids.into_iter().enumerate() {
                let index =
                    u32::try_from(index).map_err(|_| FontError::Arithmetic("font face index"))?;
                let parsed = ttf_parser::Face::parse(&file.bytes, index).map_err(|error| {
                    FontError::ParseFace {
                        path: file.canonical_path.clone(),
                        index,
                        reason: error.to_string(),
                    }
                })?;
                let info = database
                    .face(database_id)
                    .ok_or_else(|| FontError::DatabaseLoad(file.canonical_path.clone()))?;
                if info.index != index {
                    return Err(FontError::DatabaseLoad(file.canonical_path.clone()));
                }

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
    if cumulative_claim > TEXT_GENERATION_LIMIT {
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
    let occurrence_count = names
        .into_iter()
        .filter(|record| {
            matches!(
                record.name_id,
                ttf_parser::name_id::TYPOGRAPHIC_FAMILY | ttf_parser::name_id::FAMILY
            ) && decodes_like_ttf_parser(record)
        })
        .count();
    budget.claim_work(
        u64::try_from(occurrence_count)
            .map_err(|_| FontError::Arithmetic("font family name occurrences"))?,
    )?;
    let mut occurrences = Vec::new();
    reserve_prepared_or_local(
        &mut occurrences,
        occurrence_count,
        "font family name occurrences",
        prepared_stage,
    )?;
    for (record_ordinal, record) in names.into_iter().enumerate() {
        let rank = match record.name_id {
            ttf_parser::name_id::TYPOGRAPHIC_FAMILY => NameRank::TypographicFamily,
            ttf_parser::name_id::FAMILY => NameRank::LegacyFamily,
            _ => continue,
        };
        let Some(value) = record.to_string() else {
            continue;
        };
        occurrences.push(NameOccurrence {
            folded: super::unicode16::nfkc_casefold(&value),
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

fn decodes_like_ttf_parser(record: &ttf_parser::name::Name<'_>) -> bool {
    record.is_unicode()
        && char::decode_utf16(
            record
                .name
                .chunks_exact(2)
                .map(|unit| u16::from_be_bytes([unit[0], unit[1]])),
        )
        .all(|scalar| scalar.is_ok())
}

fn canonical_candidates(
    database: &Database,
    faces: &[FaceMetadata],
    stack: &[FamilyPreference],
    requested_weight: RequestedWeight,
    budget: &mut FontBudget,
) -> Result<Vec<CandidateFace>, FontError> {
    canonical_candidates_with_stage(database, faces, stack, requested_weight, budget, None)
}

fn canonical_candidates_with_stage(
    database: &Database,
    faces: &[FaceMetadata],
    stack: &[FamilyPreference],
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
    resolve_stack_names_into(database, stack, &mut resolved_names)?;

    let count = count_candidate_occurrences(faces, &resolved_names, budget)?;
    let mut candidates = Vec::new();
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
    stack: &[FamilyPreference],
    resolved_names: &mut Vec<(String, bool)>,
) -> Result<(), FontError> {
    for family in stack {
        let (name, generic) = match family {
            FamilyPreference::Named(name) => (name.as_str(), false),
            FamilyPreference::Serif => (database.family_name(&Family::Serif), true),
            FamilyPreference::SansSerif => (database.family_name(&Family::SansSerif), true),
            FamilyPreference::Monospace => (database.family_name(&Family::Monospace), true),
        };
        resolved_names.push((super::unicode16::nfkc_casefold(name), generic));
    }
    if resolved_names.len() != stack.len() {
        return Err(FontError::Arithmetic(
            "resolved font stack count/materialization mismatch",
        ));
    }
    Ok(())
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
                if name.folded != *resolved_name {
                    continue;
                }
                budget.claim_work(1)?;
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
                if name.folded == *resolved_name {
                    budget.claim_work(1)?;
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
) -> Result<usize, FontError> {
    let mut count = 0_usize;
    for (face_index, face) in faces.iter().enumerate() {
        if !face_matches_resolved_names(face, resolved_names)
            || faces[..face_index].iter().any(|earlier| {
                earlier.identity == face.identity
                    && face_matches_resolved_names(earlier, resolved_names)
            })
        {
            continue;
        }
        count = count
            .checked_add(1)
            .ok_or(FontError::Arithmetic("canonical font candidates"))?;
    }
    Ok(count)
}

fn face_matches_resolved_names(face: &FaceMetadata, resolved_names: &[(String, bool)]) -> bool {
    resolved_names
        .iter()
        .any(|(resolved_name, _)| face.names.iter().any(|name| name.folded == *resolved_name))
}

fn canonicalize_candidate_occurrences(
    candidates: &mut Vec<CandidateFace>,
    budget: &mut FontBudget,
) -> Result<(), FontError> {
    let input = u64::try_from(candidates.len())
        .map_err(|_| FontError::Arithmetic("candidate occurrence sort"))?;
    budget.claim_work(input)?;
    candidates.sort_by(|left, right| {
        left.occurrence
            .identity
            .cmp(&right.occurrence.identity)
            .then_with(|| left.occurrence.canonical_cmp(&right.occurrence))
    });

    budget.claim_work(input)?;
    candidates.dedup_by(|right, left| right.occurrence.identity == left.occurrence.identity);

    let survivors = u64::try_from(candidates.len())
        .map_err(|_| FontError::Arithmetic("canonical candidate sort"))?;
    budget.claim_work(survivors)?;
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
    for scalar in required_scalars {
        if charge_required_scalars {
            budget.claim_work(1)?;
        }
        let value = *scalar as usize;
        presence[value / u64::BITS as usize] |= 1_u64 << (value % u64::BITS as usize);
    }

    let scalar_count = presence.iter().try_fold(0_usize, |total, word| {
        total
            .checked_add(word.count_ones() as usize)
            .ok_or(FontError::Arithmetic("required scalar count"))
    })?;
    let mandatory_break_count = [
        '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{0085}', '\u{2028}', '\u{2029}',
    ]
    .into_iter()
    .filter(|scalar| {
        let value = *scalar as usize;
        presence[value / u64::BITS as usize] & (1_u64 << (value % u64::BITS as usize)) != 0
    })
    .count();
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

        for candidate in candidates {
            budget.claim_work(1)?;
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

fn shape_retained_run(
    file: &RetainedFontFile,
    metadata: &FaceMetadata,
    text: &str,
    direction: ShapingDirection,
    script: rustybuzz::Script,
) -> Result<ShapedRun, FontError> {
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
    buffer.push_str(text);
    buffer.set_direction(direction.into());
    buffer.set_script(script);
    buffer.set_language(
        rustybuzz::Language::from_str("und").map_err(|_| FontError::Shape {
            identity: metadata.identity,
            reason: "invalid fixed shaping language",
        })?,
    );
    let shaped = rustybuzz::shape(&face, &[], buffer);
    let mut glyphs = Vec::new();
    try_reserve_exact(&mut glyphs, shaped.len(), "shaped glyphs")?;
    for (info, position) in shaped.glyph_infos().iter().zip(shaped.glyph_positions()) {
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
            Err(_) => {}
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
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        let file_type = metadata.file_type();
        let is_directory = if file_type.is_symlink() {
            fs::metadata(&path).is_ok_and(|target| target.is_dir())
        } else {
            file_type.is_dir()
        };
        if is_directory {
            if file_type.is_symlink() {
                continue;
            }
            let Ok(entries) = fs::read_dir(&path) else {
                continue;
            };
            for entry in entries {
                let Ok(entry) = entry else {
                    continue;
                };
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
        if !record.admit_file
            || !fs::metadata(&record.path).is_ok_and(|metadata| metadata.is_file())
        {
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
    budget.claim_work(
        u64::try_from(discovered.len())
            .map_err(|_| FontError::Arithmetic("discovery path sort"))?,
    )?;
    discovered.sort_unstable_by(|left, right| {
        native_path_cmp(&left.path, &right.path)
            .then_with(|| left.root_ordinal.cmp(&right.root_ordinal))
    });
    budget.claim_work(
        u64::try_from(discovered.len())
            .map_err(|_| FontError::Arithmetic("discovery path deduplication"))?,
    )?;
    discovered.dedup_by(|right, left| right.path == left.path);
    budget.claim_work(
        u64::try_from(discovered.len())
            .map_err(|_| FontError::Arithmetic("discovery root sort"))?,
    )?;
    discovered.sort_unstable_by(|left, right| {
        left.root_ordinal
            .cmp(&right.root_ordinal)
            .then_with(|| native_path_cmp(&left.path, &right.path))
    });

    let mut opened = Vec::new();
    reserve_prepared_or_local(
        &mut opened,
        discovered.len(),
        "opened font files",
        prepared_stage.as_deref_mut(),
    )?;
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
        let metadata_before = file.metadata().map_err(|source| FontError::Io {
            operation: "inspect font file",
            path: canonical_path.clone(),
            source,
        })?;
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
        opened.push(OpenedFontFile {
            root_ordinal: record.root_ordinal,
            canonical_path,
            file,
            length,
            modified: modified_before,
            bytes,
        });
    }

    for mut opened_file in opened {
        let sha256 = read_exact_font_bytes(
            &mut opened_file.file,
            &opened_file.canonical_path,
            opened_file.length,
            &mut opened_file.bytes,
        )?;
        let metadata_after = opened_file
            .file
            .metadata()
            .map_err(|source| FontError::Io {
                operation: "reinspect font file",
                path: opened_file.canonical_path.clone(),
                source,
            })?;
        if metadata_after.len()
            != u64::try_from(opened_file.length)
                .map_err(|_| FontError::Arithmetic("font file length"))?
            || opened_file.modified.is_some_and(|modified| {
                metadata_after
                    .modified()
                    .map_or(true, |current| current != modified)
            })
        {
            return Err(FontError::ChangedFile(opened_file.canonical_path));
        }
        retained.push(RetainedFontFile {
            root_ordinal: opened_file.root_ordinal,
            canonical_path: opened_file.canonical_path,
            bytes: Arc::new(opened_file.bytes),
            sha256,
        });
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
    retained: &mut Vec<FamilyPreference>,
    source: &[FamilyPreference],
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), FontError> {
    for preference in source {
        retained.push(match preference {
            FamilyPreference::Named(name) => {
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
                FamilyPreference::Named(reserved)
            }
            FamilyPreference::Serif => FamilyPreference::Serif,
            FamilyPreference::SansSerif => FamilyPreference::SansSerif,
            FamilyPreference::Monospace => FamilyPreference::Monospace,
        });
    }
    Ok(())
}

fn fill_reserved_family_stack(
    retained: &mut [FamilyPreference],
    source: &[FamilyPreference],
) -> Result<(), FontError> {
    if retained.len() != source.len() {
        return Err(FontError::Arithmetic(
            "prepared font stack count/materialization mismatch",
        ));
    }
    for (retained, source) in retained.iter_mut().zip(source) {
        match (retained, source) {
            (FamilyPreference::Named(retained), FamilyPreference::Named(source)) => {
                if retained.capacity() < source.len() || !retained.is_empty() {
                    return Err(FontError::Arithmetic(
                        "prepared font family reservation mismatch",
                    ));
                }
                retained.push_str(source);
            }
            (FamilyPreference::Serif, FamilyPreference::Serif)
            | (FamilyPreference::SansSerif, FamilyPreference::SansSerif)
            | (FamilyPreference::Monospace, FamilyPreference::Monospace) => {}
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
    Loaded(Vec<FontConfigDirective>),
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
    let FontConfigRead::Loaded(parts) = read_bounded_fontconfig(
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
                if let FontConfigRead::Loaded(parts) = read_bounded_fontconfig(
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
    Ok(true)
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
    let Ok(canonical_path) = fs::canonicalize(config_path) else {
        return Ok(FontConfigRead::Ignored);
    };
    if !admitted {
        budget.claim_discovery(1)?;
    }
    budget.claim_work(
        u64::try_from(config.files.len())
            .map_err(|_| FontError::Arithmetic("fontconfig path deduplication"))?,
    )?;
    if config.files.contains(&canonical_path) {
        return Ok(FontConfigRead::Ignored);
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
        return Ok(FontConfigRead::Ignored);
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

    match parse_bounded_fontconfig(&source, config_path, budget, prepared_stage) {
        Ok(parts) => Ok(FontConfigRead::Loaded(parts)),
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
) -> Result<Vec<FontConfigDirective>, FontError> {
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
    let attributes = u64::try_from(source.bytes().filter(|byte| *byte == b'=').count())
        .map_err(|_| FontError::Arithmetic("fontconfig XML attribute capacity"))?;
    budget.claim_work(
        node_capacity
            .checked_add(attributes)
            .ok_or(FontError::Arithmetic("fontconfig XML parser capacity"))?,
    )?;
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
            Err(FontError::ParseFontConfig { .. }) => break,
            Err(error) => return Err(error),
        }
    }
    Ok(directives)
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
            "prefer" => copy_fontconfig_families(
                child,
                config_path,
                &mut alias.prefer,
                prepared_stage.as_deref_mut(),
            )?,
            "accept" => copy_fontconfig_families(
                child,
                config_path,
                &mut alias.accept,
                prepared_stage.as_deref_mut(),
            )?,
            "default" => copy_fontconfig_families(
                child,
                config_path,
                &mut alias.default,
                prepared_stage.as_deref_mut(),
            )?,
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
    config_path: &Path,
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
        destination.push(copy_fontconfig_string(
            fontconfig_node_text(family, config_path)?,
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
    let Ok(metadata) = fs::metadata(include_path) else {
        return Ok(());
    };
    if metadata.is_file() {
        reserve_prepared_or_local(tasks, 1, "fontconfig traversal", prepared_stage)?;
        tasks.push(FontConfigTask::Merge {
            path: include_path.to_path_buf(),
            admitted: false,
        });
        return Ok(());
    }
    if !metadata.is_dir() {
        return Ok(());
    }
    budget.claim_discovery(1)?;
    let Ok(entries) = fs::read_dir(include_path) else {
        return Ok(());
    };
    let mut paths = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else { continue };
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_file() && !file_type.is_symlink() {
            continue;
        }
        budget.claim_discovery(1)?;
        reserve_prepared_or_local(
            &mut paths,
            1,
            "fontconfig include entries",
            prepared_stage.as_deref_mut(),
        )?;
        paths.push(entry.path());
    }
    budget.claim_work(
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
    fn unicode_16_casefold_covers_ascii_compatibility_and_full_fold_cases() {
        assert_eq!(
            super::super::unicode16::nfkc_casefold("NOTO Sans"),
            "noto sans"
        );
        assert_eq!(
            super::super::unicode16::nfkc_casefold("Ｎｏｔｏ ﬃ"),
            "noto ffi"
        );
        assert_eq!(
            super::super::unicode16::nfkc_casefold("Straße"),
            super::super::unicode16::nfkc_casefold("STRASSE")
        );
        assert_eq!(
            super::super::unicode16::nfkc_casefold("ΟΣ"),
            super::super::unicode16::nfkc_casefold("ος")
        );
    }

    #[test]
    fn resource_limits_accept_equal_and_reject_plus_one() {
        let limits = ResourceLimits {
            discovery_entries: 2,
            font_bytes: 2,
            logical_work: 4,
        };
        let mut budget = FontBudget::new(limits);
        budget.claim_discovery(2).unwrap();
        budget.claim_bytes(2).unwrap();
        budget.claim_work(2).unwrap();
        assert!(matches!(
            budget.claim_discovery(1),
            Err(FontError::Limit { .. })
        ));
        assert!(matches!(
            budget.claim_bytes(1),
            Err(FontError::Limit { .. })
        ));
        assert!(matches!(budget.claim_work(1), Err(FontError::Limit { .. })));
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
        assert!(merge_fontconfig_root(
            &mut config,
            &config_path,
            &mut FontBudget::new(ResourceLimits::PRODUCTION),
            None,
        )
        .unwrap());

        assert_eq!(config.dirs.len(), 1);
        assert_eq!(config.dirs[0].path, Path::new("/first"));
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
            logical_work: 1_024,
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
            discovery_entries: 6,
            font_bytes: 1_024,
            logical_work: 1_024,
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
            discovery_entries: 7,
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
            logical_work: 5,
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
            logical_work: 4,
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
            logical_work: 32,
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
            logical_work: 1,
        };
        let mut budget = FontBudget::new(limits);

        let files = discover_retained_files(&roots, &mut budget).unwrap();

        assert!(files.is_empty());
        assert_eq!(
            budget.resources.usage().unwrap(),
            FontResourceUsage {
                discovery_entries: 1,
                font_bytes: 0,
                logical_work: 1,
            }
        );
    }

    #[test]
    fn retained_font_read_rejects_short_and_never_reads_past_admitted_length() {
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
                logical_work: 8,
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
            &[FamilyPreference::Named("Other Sans".into())],
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
        let stack = parse_family_stack("Test Sans").unwrap();
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
        let stack = parse_family_stack("Test Sans").unwrap();
        let normal_scalars = ['A'];
        let bold_scalars = ['\n', 'A', '\n'];
        let before = environment.resource_usage().unwrap().logical_work;
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

        let after = environment.resource_usage().unwrap().logical_work;
        assert_eq!(after - before, 20);
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
        let stack = parse_family_stack("Test Sans").unwrap();
        let scalars = ['A'];
        let before = environment.resource_usage().unwrap().logical_work;

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
        assert_eq!(environment.resource_usage().unwrap().logical_work, before);
        assert!(environment.core.read().unwrap().prepared_pairs.is_empty());
    }

    #[test]
    fn batch_counts_every_pair_before_first_pair_materialization() {
        let environment = PreparedFontEnvironment::synthetic();
        let stack = parse_family_stack("Test Sans").unwrap();
        let scalars = ['A'];
        environment.core.write().unwrap().budget = FontBudget::new(ResourceLimits {
            logical_work: 10,
            ..ResourceLimits::PRODUCTION
        });

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
                &[FamilyPreference::Named("TEST SANS".into())],
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
            super::super::unicode16::nfkc_casefold(core.database.family_name(&Family::SansSerif));
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
            &[FamilyPreference::SansSerif],
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
                folded: super::super::unicode16::nfkc_casefold(name),
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
            OperationGenerationResources::new(),
        )
        .unwrap()
    }

    #[allow(clippy::too_many_lines)]
    fn build_test_font() -> Vec<u8> {
        let family = utf16_be("Test Sans");
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
        push_u16(&mut cmap, 1);
        push_u32(&mut cmap, 12);
        push_u16(&mut cmap, 4);
        push_u16(&mut cmap, 48);
        push_u16(&mut cmap, 0);
        push_u16(&mut cmap, 8);
        push_u16(&mut cmap, 8);
        push_u16(&mut cmap, 2);
        push_u16(&mut cmap, 0);
        push_u16(&mut cmap, 0x0020);
        push_u16(&mut cmap, 0x0041);
        push_u16(&mut cmap, 0x05D0);
        push_u16(&mut cmap, 0xFFFF);
        push_u16(&mut cmap, 0);
        push_u16(&mut cmap, 0x0020);
        push_u16(&mut cmap, 0x0041);
        push_u16(&mut cmap, 0x05D0);
        push_u16(&mut cmap, 0xFFFF);
        push_u16(&mut cmap, 0xFFE2);
        push_u16(&mut cmap, 0xFFC0);
        push_u16(&mut cmap, 0xFA31);
        push_u16(&mut cmap, 1);
        push_u16(&mut cmap, 0);
        push_u16(&mut cmap, 0);
        push_u16(&mut cmap, 0);
        push_u16(&mut cmap, 0);

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
