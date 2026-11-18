// This is the private T5a3 integration surface; later text planning consumes it.
#![allow(dead_code)]

use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock, Weak};

use fontdb::{Database, Family, Source, Stretch, Style, ID};
use sha2::{Digest, Sha256};
use unicode_properties::UnicodeGeneralCategory;
use unicode_segmentation::UnicodeSegmentation;

use crate::text::{is_nonpainting_control, is_nonrendering_mandatory_break};

mod admission;
mod index;
use admission::FontScanIssue;

#[cfg(test)]
thread_local! {
    pub(crate) static FONT_DISCOVERY_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static FONT_PAIR_RESOLUTION_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static FONT_NORMALIZATION_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static FONT_SELECTION_PROBES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static FONT_SOURCE_ADMISSIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static FONT_FINAL_SEALS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static FONT_SHAPE_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static FONT_OUTLINE_GLYPHS: std::cell::RefCell<Vec<(FaceIdentity, u16)>> = const { std::cell::RefCell::new(Vec::new()) };
    pub(crate) static FONT_METRIC_QUERIES: std::cell::RefCell<Vec<FaceIdentity>> = const { std::cell::RefCell::new(Vec::new()) };
    static LOAD_OBSERVATIONS: std::cell::RefCell<LoadObservations> = std::cell::RefCell::new(LoadObservations::default());
}

#[cfg(test)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct LoadObservations {
    metadata_bytes: std::collections::BTreeMap<PathBuf, usize>,
    full_bytes: usize,
    hash_bytes: usize,
    materialized: Vec<(PathBuf, usize)>,
    face_checks: Vec<(PathBuf, u32)>,
    frozen_files: Vec<(PathBuf, usize)>,
}

#[cfg(test)]
pub(crate) fn planning_counts() -> [usize; 6] {
    [
        FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        FONT_PAIR_RESOLUTION_CALLS.with(std::cell::Cell::get),
        FONT_SELECTION_PROBES.with(std::cell::Cell::get),
        FONT_SOURCE_ADMISSIONS.with(std::cell::Cell::get),
        FONT_FINAL_SEALS.with(std::cell::Cell::get),
        FONT_SHAPE_CALLS.with(std::cell::Cell::get),
    ]
}

const SCALAR_MAP_WORDS: usize = 0x11_0000 / u64::BITS as usize;

/// Census producers can lend text or defer formatting until root preparation.
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
}

impl std::fmt::Write for CensusFormatCount {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        self.bytes = self.bytes.checked_add(text.len()).ok_or(std::fmt::Error)?;
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
    #[error("font source changed during indexing or materialization: '{0}'")]
    ChangedFile(PathBuf),
    #[error("cannot parse fontconfig file '{path}': {reason}")]
    ParseFontConfig { path: PathBuf, reason: String },
    #[error("cannot parse face index {index} in '{path}': {reason}")]
    ParseFace {
        path: PathBuf,
        index: u32,
        reason: String,
    },
    #[error("{context}: {source}")]
    Admission {
        context: String,
        #[source]
        source: Box<Self>,
    },
    #[error("font candidate list is empty for pair {pair_ordinal}")]
    MissingFamily { pair_ordinal: u32 },
    #[error("font candidates for pair {pair_ordinal} do not contain U+{scalar:04X}")]
    MissingGlyph { pair_ordinal: u32, scalar: u32 },
    #[error("font pair {pair_ordinal} was prepared more than once")]
    DuplicatePreparedPair { pair_ordinal: u32 },
    #[error("font pair {pair_ordinal} was prepared after the root's fonts were sealed")]
    PreparedPairAfterSeal { pair_ordinal: u32 },
    #[error("font operation is already sealed")]
    OperationSealed,
    #[error("frozen text does not match admitted font pair {pair_ordinal}")]
    UnadmittedText { pair_ordinal: u32 },
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
    #[error("selected glyph {glyph_id} from face {identity:?} requires unsupported {paint:?} paint without a usable outline")]
    UnsupportedGlyphPaint {
        identity: FaceIdentity,
        glyph_id: u16,
        paint: GlyphPaint,
    },
    #[error("canonical font environment synchronization was poisoned")]
    Synchronization,
}

#[derive(Debug)]
struct FontPreparation {
    operation: OperationContext,
    normalized_stacks: Vec<NormalizedStack>,
}

#[derive(Debug)]
struct NormalizedStack {
    source: FontStack,
    names: Vec<String>,
}

impl FontPreparation {
    fn new() -> Self {
        Self {
            operation: OperationContext::new(),
            normalized_stacks: Vec::new(),
        }
    }

    fn for_operation(operation: OperationContext) -> Self {
        Self {
            operation,
            normalized_stacks: Vec::new(),
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NameOccurrence {
    pub(crate) value: String,
    pub(crate) folded: String,
    pub(crate) record_ordinal: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MetricSource {
    Typographic,
    Hhea,
    InkOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DesignMetrics {
    pub(crate) units_per_em: u16,
    pub(crate) ascender: i16,
    pub(crate) descender: i16,
    pub(crate) source: MetricSource,
}

impl DesignMetrics {
    fn validate(self, identity: FaceIdentity) -> Result<Self, FontError> {
        if !(16..=16384).contains(&self.units_per_em) {
            return Err(FontError::InvalidMetrics { identity });
        }
        Ok(self)
    }
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
    pub(crate) style_distance: u8,
    pub(crate) stretch_distance: u8,
    pub(crate) weight_distance: u16,
    pub(crate) identity: FaceIdentity,
}

impl CandidateOccurrence {
    fn canonical_cmp(&self, other: &Self) -> Ordering {
        (
            self.stack_entry_index,
            self.style_distance,
            self.stretch_distance,
            self.weight_distance,
            self.identity,
        )
            .cmp(&(
                other.stack_entry_index,
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

impl From<crate::TextWeight> for RequestedWeight {
    fn from(weight: crate::TextWeight) -> Self {
        match weight {
            crate::TextWeight::Normal => Self::Normal,
            crate::TextWeight::Semibold => Self::Semibold,
            crate::TextWeight::Bold => Self::Bold,
        }
    }
}

/// Closed host-resolved generic font preferences, distinct from literal names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericFontFamily {
    /// Host serif preference.
    Serif,
    /// Host sans-serif preference.
    SansSerif,
    /// Host monospace preference.
    Monospace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FontFamilyKind {
    Named(String),
    Generic(GenericFontFamily),
}

/// A literal family identity or one of the three host generic preferences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontFamily {
    kind: FontFamilyKind,
}

impl FontFamily {
    /// Construct a nonempty, already trimmed, control-free literal family name.
    /// Punctuation and CSS generic spellings remain part of this one literal name.
    pub fn named(name: impl Into<String>) -> crate::Result<Self> {
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
    /// Construct a host generic preference without parsing a string.
    pub const fn generic(family: GenericFontFamily) -> Self {
        Self {
            kind: FontFamilyKind::Generic(family),
        }
    }
    /// Borrow the literal name, or None for a generic preference.
    pub fn name(&self) -> Option<&str> {
        match &self.kind {
            FontFamilyKind::Named(name) => Some(name),
            FontFamilyKind::Generic(_) => None,
        }
    }
    /// Inspect the generic preference, or None for a literal name.
    pub const fn generic_family(&self) -> Option<GenericFontFamily> {
        match self.kind {
            FontFamilyKind::Generic(family) => Some(family),
            FontFamilyKind::Named(_) => None,
        }
    }
}

/// Nonempty, ordered preferences; original named bytes and duplicates are retained.
///
/// Obtain a built-in profile from a complete text role, or construct typed entries:
/// ```
/// use lichtung_plot::{FontFamily, FontStack, GenericFontFamily, Theme};
/// let profile = Theme::publication().typography().get_root().get_title().get_stack().clone();
/// assert_eq!(profile.families()[0].name(), Some("Noto Sans"));
/// let stack = FontStack::new([
///     FontFamily::named("A, B")?, // One literal identity, not two CSS entries.
///     FontFamily::generic(GenericFontFamily::SansSerif),
/// ])?;
/// assert_eq!(stack.families().len(), 2);
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
///
/// No raw stack parser or default-font constant is public:
/// ```compile_fail
/// let _ = lichtung_plot::FontStack::new(["Arial, sans-serif"]);
/// ```
/// ```compile_fail
/// use lichtung_plot::DEFAULT_FONT_FAMILY;
/// ```
/// ```compile_fail
/// use lichtung_plot::DEFAULT_FONT_FAMILY_STACK;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontStack {
    families: Arc<Vec<FontFamily>>,
}

impl FontStack {
    /// Retain a nonempty ordered sequence, preserving literal bytes and duplicates.
    pub fn new(families: impl IntoIterator<Item = FontFamily>) -> crate::Result<Self> {
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
    /// Borrow the complete ordered preference sequence.
    pub fn families(&self) -> &[FontFamily] {
        &self.families
    }

    pub(crate) fn same(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.families, &other.families) {
            return true;
        }
        if self.len() != other.len() {
            return false;
        }
        for (left, right) in self.families.iter().zip(other.families.iter()) {
            match (&left.kind, &right.kind) {
                (FontFamilyKind::Named(left), FontFamilyKind::Named(right)) => {
                    if !same_bytes(left, right) {
                        return false;
                    }
                }
                (FontFamilyKind::Generic(left), FontFamilyKind::Generic(right))
                    if left == right => {}
                _ => return false,
            }
        }
        true
    }

    pub(crate) fn hash_into(&self, hash: &mut Sha256) {
        hash.update((self.len() as u64).to_le_bytes());
        for family in self.families.iter() {
            match &family.kind {
                FontFamilyKind::Named(name) => {
                    hash.update([0]);
                    hash.update((name.len() as u64).to_le_bytes());
                    hash.update(name.as_bytes());
                }
                FontFamilyKind::Generic(family) => {
                    hash.update([1, *family as u8]);
                }
            }
        }
    }

    pub(crate) fn try_copy(&self) -> Result<Self, FontError> {
        let mut families = Vec::new();
        families
            .try_reserve_exact(self.len())
            .map_err(|_| FontError::Allocation("font stack entries"))?;
        for family in self {
            let kind = match &family.kind {
                FontFamilyKind::Named(name) => {
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

    pub(crate) fn copy_with_stage(
        &self,

        stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<Self, FontError> {
        copy_stack_with_stage(self, stage)
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

    pub(crate) fn css(&self) -> Result<String, FontError> {
        // Count and materialization each consume the known UTF-8 input once.

        let mut count = CssCount(0);
        self.write_css(&mut count)
            .map_err(|_| FontError::Arithmetic("font stack CSS bytes"))?;

        let mut output = String::new();
        output
            .try_reserve_exact(count.0)
            .map_err(|_| FontError::Allocation("font stack CSS"))?;
        self.write_css(&mut output)
            .map_err(|_| FontError::Arithmetic("font stack CSS"))?;
        Ok(output)
    }
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

pub(crate) fn same_bytes(left: &str, right: &str) -> bool {
    // Concurrent immutable borrows of this exact range already denote the same bytes.
    if std::ptr::eq(left, right) {
        return true;
    }
    if left.len() != right.len() {
        return false;
    }
    for index in 0..left.len() {
        if left.as_bytes()[index] != right.as_bytes()[index] {
            return false;
        }
    }
    true
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
    fn covers_source(&self, source: &str) -> bool {
        source
            .chars()
            .chain(source.contains('\u{ad}').then_some('-'))
            .all(|scalar| {
                self.selected_scalars
                    .iter()
                    .any(|entry| entry.scalar == scalar)
            })
            && selection_units(source)
                .filter(|unit| unit.chars().count() > 1 && unit.chars().any(is_visible_mark))
                .all(|unit| self.selected_units.iter().any(|entry| entry.source == unit))
    }

    fn try_copy(
        &self,
        mut stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<Self, FontError> {
        let mut candidates = Vec::new();
        reserve_cached_with_stage(
            &mut candidates,
            self.provenance.candidates.len(),
            stage.as_deref_mut(),
        )?;
        candidates.extend_from_slice(&self.provenance.candidates);
        let mut selected_scalars = Vec::new();
        reserve_cached_with_stage(
            &mut selected_scalars,
            self.selected_scalars.len(),
            stage.as_deref_mut(),
        )?;
        selected_scalars.extend_from_slice(&self.selected_scalars);
        let mut selected_units = Vec::new();
        reserve_cached_with_stage(
            &mut selected_units,
            self.selected_units.len(),
            stage.as_deref_mut(),
        )?;
        for unit in &self.selected_units {
            let mut source = String::new();
            if let Some(stage) = stage.as_deref_mut() {
                stage
                    .reserve_prepared_string(
                        &mut source,
                        unit.source.len(),
                        "font unit source copy",
                    )
                    .map_err(|error| FontError::Reservation(error.to_string()))?;
            } else {
                source
                    .try_reserve_exact(unit.source.len())
                    .map_err(|_| FontError::Allocation("font unit source copy"))?;
            }
            source.push_str(&unit.source);
            selected_units.push(SelectedFontUnit {
                source,
                face: unit.face,
            });
        }
        Ok(Self {
            provenance: PairProvenance {
                pair_ordinal: self.provenance.pair_ordinal,
                weight: self.provenance.weight,
                candidates,
            },
            primary: self.primary,
            selected_scalars: SelectedScalarFaces(selected_scalars),
            selected_units,
        })
    }

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
pub(crate) enum GlyphPaint {
    RasterImage,
    SvgImage,
    ColorLayers,
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

struct DiscoveryFontHandle {
    file: File,
}

#[cfg(test)]
thread_local! {
    // Live handles, peak handles, opens (metadata and materialization).
    static DISCOVERY_HANDLES: std::cell::Cell<[usize; 3]> = const { std::cell::Cell::new([0; 3]) };
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
    index: Vec<index::IndexedFontFile>,
    // Unique while admission is open; shared read-only only after sealing.
    fonts: Arc<FrozenFontRegistry>,
    scalar_presence: Vec<u64>,
    prepared_pairs: Vec<PreparedPair>,
    prepared_pairs_sealed: bool,
    operation: OperationContext,
    preparation: FontPreparation,
}

#[derive(Debug)]
struct PreparedPair {
    stack: FontStack,
    weight: RequestedWeight,
    candidates: Vec<CandidateFace>,
    resolved: ResolvedFontPair,
    sources: Vec<String>,
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
    resolved_names: Vec<String>,
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

/// The materialized candidate file/face collection. Uniquely owned while open,
/// shared with Output only after sealing, without index/source/operation state.
#[derive(Debug)]
pub(crate) struct FrozenFontRegistry {
    files: Vec<RetainedFontFile>,
    faces: Vec<FaceMetadata>,
    issues: Vec<FontScanIssue>,
}

pub(crate) struct SealedTextVerifier<'a> {
    environment: &'a PreparedFontEnvironment,
}

impl<'a> SealedTextVerifier<'a> {
    pub(crate) fn new(environment: &'a PreparedFontEnvironment) -> Self {
        Self { environment }
    }
}

impl SealedTextVerifier<'_> {
    pub(crate) fn validate_measured(
        &self,
        text: &crate::layout::ResolvedTextPlan,
    ) -> Result<(), FontError> {
        self.environment.validate_admitted_text(
            &text.style.stack,
            &text.pair_provenance,
            &text.source,
        )
    }

    pub(crate) fn retain_fonts(&self) -> Result<Arc<FrozenFontRegistry>, FontError> {
        let core = self
            .environment
            .core
            .read()
            .map_err(|_| FontError::Synchronization)?;
        if !core.prepared_pairs_sealed {
            return Err(FontError::Arithmetic("font registry shared before sealing"));
        }
        #[cfg(test)]
        LOAD_OBSERVATIONS.with_borrow_mut(|observed| {
            observed.frozen_files = core
                .fonts
                .files
                .iter()
                .map(|file| (file.canonical_path.clone(), file.bytes.len()))
                .collect();
        });
        Ok(Arc::clone(&core.fonts))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct OperationContext {
    // Shared by clones independently of accounting and mutable cache contents.
    identity: Arc<()>,
    pub(crate) palette_cache: Arc<Mutex<crate::theme::PaletteCache>>,
    fonts: Arc<Mutex<FontOperation>>,
    pub(crate) numeric_presentations: Arc<Mutex<crate::numeric::table::PresentationTable>>,
    pub(crate) tick_positions: Arc<Mutex<crate::spec::ticks::function::PositionTable>>,
}

#[derive(Debug, Default)]
struct FontOperation {
    environments: Vec<Weak<RwLock<CanonicalFontCore>>>,
    sealed: bool,
}

impl OperationContext {
    #[cfg(test)]
    pub(crate) fn liveness_probe(&self) -> impl Fn() -> bool + use<> {
        let palettes = Arc::downgrade(&self.palette_cache);
        let fonts = Arc::downgrade(&self.fonts);
        let numbers = Arc::downgrade(&self.numeric_presentations);
        let positions = Arc::downgrade(&self.tick_positions);
        move || {
            palettes.strong_count() != 0
                || fonts.strong_count() != 0
                || numbers.strong_count() != 0
                || positions.strong_count() != 0
        }
    }

    pub(crate) fn seal_fonts(&self) -> Result<(), FontError> {
        self.tick_positions
            .lock()
            .map_err(|_| FontError::Synchronization)?
            .seal();
        self.numeric_presentations
            .lock()
            .map_err(|_| FontError::Synchronization)?
            .seal();
        let mut fonts = self.fonts.lock().map_err(|_| FontError::Synchronization)?;
        if fonts.sealed {
            return Err(FontError::OperationSealed);
        }
        for environment in fonts.environments.iter().filter_map(Weak::upgrade) {
            let mut core = environment
                .write()
                .map_err(|_| FontError::Synchronization)?;
            core.prepared_pairs_sealed = true;
            #[cfg(test)]
            FONT_FINAL_SEALS.with(|calls| calls.set(calls.get() + 1));
        }
        fonts.sealed = true;
        Ok(())
    }

    pub(crate) fn same_operation(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
    }

    pub(crate) fn root_identity(&self) -> crate::orchestrator::registration::RootIdentity {
        crate::orchestrator::registration::RootIdentity(Arc::clone(&self.identity))
    }

    pub(crate) fn new() -> Self {
        Self {
            identity: Arc::new(()),
            palette_cache: Arc::default(),
            fonts: Arc::default(),
            numeric_presentations: Arc::default(),
            tick_positions: Arc::default(),
        }
    }
}

impl PreparedFontEnvironment {
    /// Geometry fixtures can serialize an already measured plan while continuing
    /// to measure other sources. Fork only the materialized immutable sources;
    /// unlike production capture this does not share the open registry or seal
    /// its operation, and cannot discover or load another font.
    #[cfg(test)]
    pub(crate) fn materialized_snapshot_for_test(&self) -> Arc<FrozenFontRegistry> {
        let core = self.core.read().unwrap();
        Arc::new(FrozenFontRegistry {
            files: core.fonts.files.clone(),
            faces: core.fonts.faces.clone(),
            issues: Vec::new(),
        })
    }

    #[cfg(test)]
    pub(crate) fn liveness_probe(&self) -> impl Fn() -> bool + use<> {
        let core = Arc::downgrade(&self.core);
        move || core.strong_count() != 0
    }

    /// Verify frozen source coverage and pair provenance without adding fonts,
    /// copying selections, probing faces or shaping text.
    pub(crate) fn validate_admitted_text(
        &self,
        stack: &FontStack,
        provenance: &PairProvenance,
        source: &str,
    ) -> Result<(), FontError> {
        let core = self.core.read().map_err(|_| FontError::Synchronization)?;
        if core.prepared_pairs_sealed
            && core.prepared_pairs.iter().any(|pair| {
                pair.stack.same(stack)
                    && pair.resolved.provenance == *provenance
                    && pair.resolved.covers_source(source)
            })
        {
            Ok(())
        } else {
            Err(FontError::UnadmittedText {
                pair_ordinal: provenance.pair_ordinal,
            })
        }
    }

    /// Admit one reached source into this operation before any metric or shaping
    /// query. Both initial census and bounded planning use this same transaction.
    pub(crate) fn admit_text(
        &self,
        stack: &FontStack,
        weight: RequestedWeight,
        source: &str,
        mut stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<u32, FontError> {
        u32::try_from(source.len())
            .map_err(|_| FontError::Arithmetic("admitted text source byte range"))?;
        let mut core = self.core.write().map_err(|_| FontError::Synchronization)?;
        if let Some(stage) = stage.as_ref() {
            if !core.operation.same_operation(&stage.operation()) {
                return Err(FontError::Arithmetic(
                    "cannot replace a prepared font operation",
                ));
            }
        }
        let existing = core
            .prepared_pairs
            .iter()
            .position(|pair| pair.weight == weight && pair.stack.same(stack));
        if let Some(index) = existing {
            if core.prepared_pairs[index]
                .sources
                .iter()
                .any(|known| known == source)
            {
                return Ok(core.prepared_pairs[index].resolved.provenance.pair_ordinal);
            }
        }
        let ordinal = if let Some(index) = existing {
            core.prepared_pairs[index].resolved.provenance.pair_ordinal
        } else {
            core.prepared_pairs
                .iter()
                .map(|pair| pair.resolved.provenance.pair_ordinal)
                .max()
                .map_or(Ok(0), |ordinal| {
                    ordinal
                        .checked_add(1)
                        .ok_or(FontError::Arithmetic("font pair ordinal overflow"))
                })?
        };
        if core.prepared_pairs_sealed {
            return if existing
                .is_some_and(|index| core.prepared_pairs[index].resolved.covers_source(source))
            {
                Ok(ordinal)
            } else {
                Err(FontError::PreparedPairAfterSeal {
                    pair_ordinal: ordinal,
                })
            };
        }
        let count = source
            .chars()
            .count()
            .checked_add(usize::from(source.contains('\u{ad}')))
            .ok_or(FontError::Arithmetic("font source scalar count overflow"))?;
        let mut scalars = Vec::new();
        reserve_cached_with_stage(&mut scalars, count, stage.as_deref_mut())?;
        scalars.extend(source.chars());
        if source.contains('\u{ad}') {
            scalars.push('-');
        }
        let index = core
            .admit_scalars_in(ordinal, stack, weight, &scalars, stage.as_deref_mut())
            .map_err(|error| core.admission_error(error, Some(stack)))?;
        core.prepare_source_units(ordinal, source, stage.as_deref_mut())?;
        let mut retained = String::new();
        if let Some(stage) = stage.as_deref_mut() {
            stage
                .reserve_prepared_string(&mut retained, source.len(), "admitted text source")
                .map_err(|error| FontError::Reservation(error.to_string()))?;
        } else {
            retained
                .try_reserve_exact(source.len())
                .map_err(|_| FontError::Allocation("admitted text source"))?;
        }
        reserve_cached_with_stage(&mut core.prepared_pairs[index].sources, 1, stage)?;
        retained.push_str(source);
        core.prepared_pairs[index].sources.push(retained);
        #[cfg(test)]
        FONT_SOURCE_ADMISSIONS.with(|calls| calls.set(calls.get() + 1));
        Ok(ordinal)
    }

    fn for_core(core: CanonicalFontCore) -> Result<Self, FontError> {
        let operation = core.operation.clone();
        let core = Arc::new(RwLock::new(core));
        let mut fonts = operation
            .fonts
            .lock()
            .map_err(|_| FontError::Synchronization)?;
        if fonts.sealed {
            return Err(FontError::OperationSealed);
        }
        fonts
            .environments
            .try_reserve(1)
            .map_err(|_| FontError::Allocation("operation font environments"))?;
        fonts.environments.push(Arc::downgrade(&core));
        Ok(Self { core })
    }

    #[cfg(test)]
    pub(crate) fn discover_system() -> Result<Self, FontError> {
        #[cfg(test)]
        FONT_DISCOVERY_CALLS.with(|calls| calls.set(calls.get() + 1));
        let scalar_presence = scalar_presence_map()?;
        Self::for_core(CanonicalFontCore::discover_prepared(
            FontPreparation::new(),
            scalar_presence,
            None,
        )?)
    }

    pub(crate) fn discover_for_operation(
        scalar_presence: Vec<u64>,
        prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<Self, FontError> {
        let operation = prepared_stage.operation();
        if operation
            .fonts
            .lock()
            .map_err(|_| FontError::Synchronization)?
            .sealed
        {
            return Err(FontError::OperationSealed);
        }
        #[cfg(test)]
        FONT_DISCOVERY_CALLS.with(|calls| calls.set(calls.get() + 1));
        let preparation = FontPreparation::for_operation(operation);
        Self::for_core(CanonicalFontCore::discover_prepared(
            preparation,
            scalar_presence,
            Some(prepared_stage),
        )?)
    }

    #[cfg(test)]
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
        core.resolve_pair(pair_ordinal, stack, weight, required_scalars)
    }

    pub(crate) fn validate_operation(&self, resources: &OperationContext) -> Result<(), FontError> {
        let core = self.core.read().map_err(|_| FontError::Synchronization)?;
        // Prepared fonts and their clones remain bound to their original operation.
        if !core.operation.same_operation(resources) {
            return Err(FontError::Arithmetic(
                "cannot replace a prepared font operation",
            ));
        }
        Ok(())
    }

    pub(crate) fn operation(&self) -> Result<OperationContext, FontError> {
        Ok(self
            .core
            .read()
            .map_err(|_| FontError::Synchronization)?
            .operation
            .clone())
    }

    #[cfg(test)]
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
        let mut core = self.core.write().map_err(|_| FontError::Synchronization)?;
        core.prepare_pairs_with_stage(requests, prepared_stage)
            .map_err(|error| core.admission_error(error, None))
    }

    #[cfg(test)]
    pub(crate) fn prepare_pair(
        &self,
        pair_ordinal: u32,
        stack: &FontStack,
        weight: RequestedWeight,
        required_scalars: &[char],
        prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), FontError> {
        self.prepare_pairs(
            std::iter::once(FontPairPreparationRequest::new(
                pair_ordinal,
                stack,
                weight,
                required_scalars,
            )),
            prepared_stage,
        )
    }

    #[cfg(test)]
    pub(crate) fn prepare_source_units(
        &self,
        pair_ordinal: u32,
        source: &str,
        prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), FontError> {
        let mut core = self.core.write().map_err(|_| FontError::Synchronization)?;
        core.prepare_source_units(pair_ordinal, source, prepared_stage)
    }
}

impl CanonicalFontCore {
    fn prepare_source_units(
        &mut self,
        pair_ordinal: u32,
        source: &str,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), FontError> {
        if self.prepared_pairs_sealed {
            return Err(FontError::PreparedPairAfterSeal { pair_ordinal });
        }
        // Scalar coverage is complete before whole combining units are selected.
        let index = self
            .prepared_pairs
            .iter()
            .position(|pair| pair.resolved.provenance.pair_ordinal == pair_ordinal)
            .ok_or(FontError::MissingFamily { pair_ordinal })?;
        for unit in selection_units(source)
            .filter(|unit| unit.chars().count() > 1 && unit.chars().any(is_visible_mark))
        {
            let pair = &self.prepared_pairs[index].resolved;
            let mut found = false;
            for entry in &pair.selected_units {
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
                let mut metadata = None;
                for face in &self.fonts.faces {
                    if face.identity == *identity {
                        metadata = Some(face);
                        break;
                    }
                }
                let metadata = metadata.ok_or(FontError::InvalidMetrics {
                    identity: *identity,
                })?;
                let file = &self.fonts.files[metadata.file_ordinal];
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
                    #[cfg(test)]
                    FONT_SELECTION_PROBES.with(|calls| calls.set(calls.get() + 1));
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

            if let Some(stage) = prepared_stage.as_deref_mut() {
                stage
                    .reserve_prepared_string(&mut retained, unit.len(), "font unit source")
                    .map_err(|error| FontError::Reservation(error.to_string()))?;
            } else {
                retained
                    .try_reserve_exact(unit.len())
                    .map_err(|_| FontError::Allocation("font unit source"))?;
            }

            reserve_cached_with_stage(
                &mut self.prepared_pairs[index].resolved.selected_units,
                1,
                prepared_stage.as_deref_mut(),
            )?;
            retained.push_str(unit);
            self.prepared_pairs[index]
                .resolved
                .selected_units
                .push(SelectedFontUnit {
                    source: retained,
                    face: selected,
                });
        }
        Ok(())
    }
}

impl PreparedFontEnvironment {
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
            if prepared.weight == weight && prepared.stack.same(stack) {
                return prepared.resolved.try_copy(None).map(Some);
            }
        }
        Ok(None)
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
        self.core
            .read()
            .map_err(|_| FontError::Synchronization)?
            .fonts
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
    #[cfg(test)]
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
        Self::for_core(tests::generated_core()).unwrap()
    }

    /// Retain generated OpenType layout tables for shaping conformance tests.
    #[cfg(test)]
    pub(crate) fn synthetic_shaping_fonts(fonts: Vec<Vec<u8>>) -> Self {
        Self::for_core(
            CanonicalFontCore::from_generated(
                fonts,
                &HostGenericNames::default(),
                FontPreparation::new(),
                scalar_presence_map().unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
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
                tests::build_test_font_with_coverage(family, &coverage)
            })
            .collect();
        Self::for_core(
            CanonicalFontCore::from_generated(
                files,
                &HostGenericNames::default(),
                FontPreparation::new(),
                scalar_presence_map().unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    }
}

#[allow(dead_code)]
impl CanonicalFontCore {
    fn discover_prepared(
        preparation: FontPreparation,
        scalar_presence: Vec<u64>,
        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<Self, FontError> {
        let sources = platform_sources(prepared_stage.as_deref_mut())?;
        let files = discover_indexed_files(&sources.roots, prepared_stage)?;
        Self::from_index(files, &sources.generics, preparation, scalar_presence)
    }

    pub(crate) fn files(&self) -> &[RetainedFontFile] {
        &self.fonts.files
    }

    pub(crate) fn faces(&self) -> &[FaceMetadata] {
        &self.fonts.faces
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
                if ordinal == request.pair_ordinal
                    || (weight == request.weight && stack.same(request.stack))
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
                &mut self.preparation,
                prepared_stage.as_deref_mut(),
            )?;
        }
        // Every relevant file is snapshotted and admitted before any published
        // pair is counted, ranked or selected. Later pairs can only append files;
        // this fixed index cannot introduce a new candidate into an old pair.
        for preparation in &preparations {
            self.materialize_matching(&preparation.resolved_names, prepared_stage.as_deref_mut())?;
        }
        for preparation in &mut preparations {
            preparation.raw_candidate_count =
                count_candidate_occurrences(&self.fonts.faces, &preparation.resolved_names);
            preparation.candidate_count =
                count_unique_candidate_survivors(&self.fonts.faces, &preparation.resolved_names)?;
            if preparation.candidate_count == 0 {
                return Err(self.admission_error(
                    FontError::MissingFamily {
                        pair_ordinal: preparation.request.pair_ordinal,
                    },
                    Some(preparation.request.stack),
                ));
            }
            let (scalar_count, rendering_scalar_count) = scalar_selection_capacities(
                preparation.request.required_scalars,
                &mut self.scalar_presence,
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
                &self.fonts.faces,
                &preparation.resolved_names,
                request.weight,
                &mut preparation.candidates,
            )?;
            if preparation.candidates.len() != preparation.raw_candidate_count {
                return Err(FontError::Arithmetic(
                    "font candidate count/materialization mismatch",
                ));
            }
            canonicalize_candidate_occurrences(&mut preparation.candidates);
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

            let (scalar_count, rendering_scalar_count) =
                scalar_selection_capacities(request.required_scalars, &mut self.scalar_presence)?;
            let maximum_probes =
                maximum_glyph_probes(rendering_scalar_count, preparation.candidates.len())?;
            if scalar_count != preparation.scalar_count
                || maximum_probes != preparation.maximum_probes
            {
                return Err(FontError::Arithmetic(
                    "font scalar capacity/materialization mismatch",
                ));
            }
            let files = &self.fonts.files;
            let faces = &self.fonts.faces;
            materialize_scalar_selection(
                request.pair_ordinal,
                &preparation.candidates,
                request.required_scalars,
                &mut self.scalar_presence,
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
                    Ok(parsed.glyph_index(scalar).is_some())
                },
            )?;
            fill_reserved_family_stack(&mut preparation.retained_stack, request.stack)?;
            self.prepared_pairs.push(PreparedPair {
                stack: FontStack {
                    families: { Arc::new(preparation.retained_stack) },
                },
                weight: request.weight,
                candidates: preparation.candidates,
                sources: Vec::new(),
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

    #[cfg(test)]
    pub(crate) fn resolve_pair(
        &mut self,
        pair_ordinal: u32,
        stack: &FontStack,
        weight: RequestedWeight,
        required_scalars: &[char],
    ) -> Result<ResolvedFontPair, FontError> {
        let index = self.admit_scalars_in(pair_ordinal, stack, weight, required_scalars, None)?;
        self.prepared_pairs[index].resolved.try_copy(None)
    }

    fn admit_scalars_in(
        &mut self,
        pair_ordinal: u32,
        stack: &FontStack,
        weight: RequestedWeight,
        required_scalars: &[char],

        mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<usize, FontError> {
        let mut cached_index = None;
        for (index, cached) in self.prepared_pairs.iter().enumerate() {
            if cached.weight == weight && cached.stack.same(stack) {
                cached_index = Some(index);
                break;
            }
        }
        let Some(index) = cached_index else {
            let index = self.prepared_pairs.len();
            self.prepare_pairs_with_stage(
                std::iter::once(FontPairPreparationRequest::new(
                    pair_ordinal,
                    stack,
                    weight,
                    required_scalars,
                )),
                prepared_stage.as_deref_mut(),
            )?;
            return Ok(index);
        };
        let cached = &mut self.prepared_pairs[index];
        let mut missing = Vec::new();

        for scalar in required_scalars {
            let mut found = false;
            for candidate in cached
                .resolved
                .selected_scalars
                .iter()
                .map(|selected| selected.scalar)
                .chain(missing.iter().copied())
            {
                if candidate == *scalar {
                    found = true;
                    break;
                }
            }
            if !found {
                reserve_cached_with_stage(&mut missing, 1, prepared_stage.as_deref_mut())?;
                missing.push(*scalar);
            }
        }
        if !missing.is_empty() {
            let files = &self.fonts.files;
            let faces = &self.fonts.faces;
            let selected = select_scalar_faces_with_stage(
                cached.resolved.provenance.pair_ordinal,
                &cached.candidates,
                &missing,
                &mut self.scalar_presence,
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
                    Ok(parsed.glyph_index(scalar).is_some())
                },
            )?;
            reserve_cached_with_stage(
                &mut cached.resolved.selected_scalars.0,
                selected.len(),
                prepared_stage,
            )?;

            cached.resolved.selected_scalars.0.extend(selected);
        }
        // Cached metadata and successful scalar selections are never recomputed.
        Ok(index)
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
        let metadata = self.find_face(identity).ok_or(FontError::Shape {
            identity,
            reason: "identity is not retained",
        })?;
        let file = &self.fonts.files[metadata.file_ordinal];
        shape_retained_run(file, metadata, text, direction, script, context)
    }

    pub(crate) fn face_metrics(&self, identity: FaceIdentity) -> Result<DesignMetrics, FontError> {
        #[cfg(test)]
        FONT_METRIC_QUERIES.with(|queries| queries.borrow_mut().push(identity));
        let face = self.find_face(identity).ok_or(FontError::Shape {
            identity,
            reason: "identity is not retained",
        })?;
        face.metrics.validate(identity)
    }

    fn find_face(&self, identity: FaceIdentity) -> Option<&FaceMetadata> {
        self.fonts
            .faces
            .iter()
            .find(|&face| face.identity == identity)
            .map(|v| v as _)
    }

    fn outline_glyph(
        &self,
        identity: FaceIdentity,
        glyph_id: u16,
        builder: &mut dyn ttf_parser::OutlineBuilder,
    ) -> Result<(DesignMetrics, Option<GlyphInk>), FontError> {
        self.fonts.outline_glyph(identity, glyph_id, builder)
    }

    #[cfg(test)]
    fn from_generated(
        files: Vec<Vec<u8>>,
        generics: &HostGenericNames,
        preparation: FontPreparation,
        scalar_presence: Vec<u64>,
    ) -> Result<Self, FontError> {
        let index = files
            .into_iter()
            .map(index::generated)
            .collect::<Result<Vec<_>, _>>()?;
        let mut core = Self::from_index(index, generics, preparation, scalar_presence)?;
        // Parser/geometry fixtures explicitly request the complete generated
        // collection, through the same index/materialization/admission path.
        for ordinal in 0..core.index.len() {
            core.materialize_file(ordinal, None)?;
        }
        Ok(core)
    }

    fn from_index(
        index: Vec<index::IndexedFontFile>,
        generics: &HostGenericNames,
        preparation: FontPreparation,
        scalar_presence: Vec<u64>,
    ) -> Result<Self, FontError> {
        let operation = preparation.operation.clone();
        let mut database = Database::new();

        apply_host_generics(&mut database, generics);

        if scalar_presence.len() != SCALAR_MAP_WORDS {
            return Err(FontError::Arithmetic("Unicode scalar presence map"));
        }

        Ok(Self {
            database,
            index,
            fonts: Arc::new(FrozenFontRegistry {
                files: Vec::new(),
                faces: Vec::new(),
                issues: Vec::new(),
            }),
            scalar_presence,
            prepared_pairs: Vec::new(),
            prepared_pairs_sealed: false,
            operation,
            preparation,
        })
    }

    fn materialize_matching(
        &mut self,
        names: &[String],
        mut stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), FontError> {
        if self.prepared_pairs_sealed {
            return Err(FontError::OperationSealed);
        }
        for ordinal in 0..self.index.len() {
            if !self.index[ordinal].is_materialized() && self.index[ordinal].matches(names) {
                self.materialize_file(ordinal, stage.as_deref_mut())?;
            }
        }
        Ok(())
    }

    fn materialize_file(
        &mut self,
        ordinal: usize,
        mut stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    ) -> Result<(), FontError> {
        if self.prepared_pairs_sealed {
            return Err(FontError::OperationSealed);
        }
        let indexed = self
            .index
            .get_mut(ordinal)
            .ok_or(FontError::Arithmetic("indexed font ordinal"))?;
        if indexed.is_materialized() {
            return Ok(());
        }
        let file = indexed.materialize(stage.as_deref_mut())?;
        let fonts = Arc::get_mut(&mut self.fonts)
            .ok_or(FontError::Arithmetic("font registry shared before sealing"))?;
        let face_start = fonts.faces.len();
        let file_ordinal = fonts.files.len();
        reserve_prepared_or_local(
            &mut fonts.files,
            1,
            "materialized font files",
            stage.as_deref_mut(),
        )?;
        admission::admit_file(
            &file,
            file_ordinal,
            &mut self.database,
            &mut fonts.faces,
            &mut fonts.issues,
            stage,
        )?;
        for face in &fonts.faces[face_start..] {
            if !indexed
                .faces
                .iter()
                .any(|entry| entry.index == face.identity.face_index && entry.names == face.names)
            {
                return Err(FontError::Arithmetic(
                    "font index/admission family mismatch",
                ));
            }
        }
        fonts.files.push(file);
        indexed.finish_materialization();
        // Read witnesses have served their purpose; the first complete snapshot
        // now owns every subsequent query. No reopening this path is permitted.
        Ok(())
    }

    #[expect(
        clippy::unnecessary_debug_formatting,
        reason = "diagnostics preserve escaped lossless native paths"
    )]
    fn admission_error(&self, error: FontError, stack: Option<&FontStack>) -> FontError {
        if !matches!(
            error,
            FontError::MissingFamily { .. } | FontError::MissingGlyph { .. }
        ) {
            return error;
        }
        let mut error = self.fonts.admission_error(error, stack);
        if let FontError::Admission { context, .. } = &mut error {
            for indexed in self.index.iter().filter(|file| !file.is_materialized()) {
                for (index, reason) in &indexed.issues {
                    let entry = match admission::format_diagnostic(format_args!(
                        "; index {:?} index={index:?}: {reason} (not materialized; no full-file SHA)",
                        indexed.path,
                    )) {
                        Ok(entry) => entry,
                        Err(error) => return error,
                    };
                    if let Err(error) = reserve_string_prepared_or_local(
                        context,
                        entry.len(),
                        "font index diagnostics",
                        None,
                    ) {
                        return error;
                    }
                    context.push_str(&entry);
                }
            }
        }
        error
    }
}

impl FrozenFontRegistry {
    #[expect(
        clippy::unnecessary_debug_formatting,
        reason = "diagnostics preserve escaped lossless native paths"
    )]
    fn admission_error(&self, error: FontError, stack: Option<&FontStack>) -> FontError {
        if !matches!(
            error,
            FontError::MissingFamily { .. } | FontError::MissingGlyph { .. }
        ) {
            return error;
        }
        let context = (|| {
            let mut context =
                admission::format_diagnostic(format_args!("requested stack {stack:?}"))?;
            for issue in &self.issues {
                let file = self
                    .files
                    .get(issue.file_ordinal)
                    .ok_or(FontError::Arithmetic("font diagnostic file identity"))?;
                let entry = admission::format_diagnostic(format_args!(
                    "; scan {:?} sha256={:02x?} index={:?}: {:?}",
                    file.canonical_path, file.sha256, issue.face_index, issue.reason,
                ))?;
                reserve_string_prepared_or_local(
                    &mut context,
                    entry.len(),
                    "font failure diagnostics",
                    None,
                )?;
                context.push_str(&entry);
            }
            Ok::<_, FontError>(context)
        })();
        match context {
            Ok(context) => FontError::Admission {
                context,
                source: Box::new(error),
            },
            Err(error) => error,
        }
    }

    pub(crate) fn outline_glyph(
        &self,
        identity: FaceIdentity,
        glyph_id: u16,
        builder: &mut dyn ttf_parser::OutlineBuilder,
    ) -> Result<(DesignMetrics, Option<GlyphInk>), FontError> {
        #[cfg(test)]
        FONT_OUTLINE_GLYPHS.with(|glyphs| glyphs.borrow_mut().push((identity, glyph_id)));
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
        let metrics = metadata.metrics.validate(identity)?;
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
        validate_selected_glyph_paint(&face, identity, glyph_id, ink.is_some())?;
        Ok((metrics, ink))
    }
}

/// Only inspect paint for an actually used glyph with no outline result.
/// Paint queries expose declarations or encoded data, not a full validity proof.
fn validate_selected_glyph_paint(
    face: &ttf_parser::Face<'_>,
    identity: FaceIdentity,
    glyph_id: u16,
    has_outline: bool,
) -> Result<(), FontError> {
    if has_outline {
        return Ok(());
    }
    let glyph = ttf_parser::GlyphId(glyph_id);
    let paint = if face.glyph_raster_image(glyph, u16::MAX).is_some() {
        Some(GlyphPaint::RasterImage)
    } else if face.glyph_svg_image(glyph).is_some() {
        Some(GlyphPaint::SvgImage)
    } else if face.is_color_glyph(glyph) {
        Some(GlyphPaint::ColorLayers)
    } else {
        None
    };
    if let Some(paint) = paint {
        return Err(FontError::UnsupportedGlyphPaint {
            identity,
            glyph_id,
            paint,
        });
    }
    Ok(())
}

fn design_metrics(face: &ttf_parser::Face<'_>) -> DesignMetrics {
    // Keep the existing typo-first policy without component-wise zero fallbacks.
    // No usable nominal interval is not proof that actual glyph ink is unusable.
    let typo = face
        .typographic_ascender()
        .zip(face.typographic_descender());
    let hhea = face.tables().hhea;
    let (ascender, descender, source) = if let Some((a, d)) = typo.filter(|(a, d)| a > d) {
        (a, d, MetricSource::Typographic)
    } else if hhea.ascender > hhea.descender {
        (hhea.ascender, hhea.descender, MetricSource::Hhea)
    } else {
        (0, 0, MetricSource::InkOnly)
    };
    DesignMetrics {
        units_per_em: face.units_per_em(),
        ascender,
        descender,
        source,
    }
}

fn decode_family_names(
    names: ttf_parser::name::Names<'_>,

    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<Vec<NameOccurrence>, FontError> {
    let mut occurrence_count = 0;
    for ordinal in 0..names.len() {
        let Some(record) = names.get(ordinal) else {
            continue;
        };
        if matches!(
            record.name_id,
            ttf_parser::name_id::TYPOGRAPHIC_FAMILY | ttf_parser::name_id::FAMILY
        ) {
            occurrence_count += usize::from(decodes_like_ttf_parser(&record));
        }
    }

    let mut occurrences = Vec::new();
    reserve_prepared_or_local(
        &mut occurrences,
        occurrence_count,
        "font family name occurrences",
        prepared_stage,
    )?;

    for record_ordinal in 0..names.len() {
        let Some(record) = names.get(record_ordinal) else {
            continue;
        };
        if !matches!(
            record.name_id,
            ttf_parser::name_id::TYPOGRAPHIC_FAMILY | ttf_parser::name_id::FAMILY
        ) {
            continue;
        }
        let Some(value) = admission::decode_name(&record)? else {
            continue;
        };
        occurrences.push(NameOccurrence {
            folded: fold_family_name(&value)?,
            value,
            record_ordinal,
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
    if !record.is_unicode() {
        return false;
    }
    for scalar in char::decode_utf16(
        record
            .name
            .chunks_exact(2)
            .map(|unit| u16::from_be_bytes([unit[0], unit[1]])),
    ) {
        if scalar.is_err() {
            return false;
        }
    }
    true
}

fn resolve_stack_names_into(
    database: &Database,
    stack: &FontStack,
    resolved_names: &mut Vec<String>,
    preparation: &mut FontPreparation,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), FontError> {
    for cached in &preparation.normalized_stacks {
        if cached.source.same(stack) {
            for name in &cached.names {
                resolved_names.push(copy_name_with_stage(name, prepared_stage.as_deref_mut())?);
            }
            return Ok(());
        }
    }
    for family in stack {
        let name = match &family.kind {
            FontFamilyKind::Named(name) => name.as_str(),
            FontFamilyKind::Generic(GenericFontFamily::Serif) => {
                database.family_name(&Family::Serif)
            }
            FontFamilyKind::Generic(GenericFontFamily::SansSerif) => {
                database.family_name(&Family::SansSerif)
            }
            FontFamilyKind::Generic(GenericFontFamily::Monospace) => {
                database.family_name(&Family::Monospace)
            }
        };
        resolved_names.push(fold_family_name(name)?);
    }
    if resolved_names.len() != stack.len() {
        return Err(FontError::Arithmetic(
            "resolved font stack count/materialization mismatch",
        ));
    }
    let source = copy_stack_with_stage(stack, prepared_stage.as_deref_mut())?;
    let mut names = Vec::new();
    reserve_cached_with_stage(
        &mut names,
        resolved_names.len(),
        prepared_stage.as_deref_mut(),
    )?;
    for name in resolved_names.iter() {
        names.push(copy_name_with_stage(name, prepared_stage.as_deref_mut())?);
    }

    reserve_cached_with_stage(&mut preparation.normalized_stacks, 1, prepared_stage)?;
    preparation
        .normalized_stacks
        .push(NormalizedStack { source, names });
    Ok(())
}

fn fold_family_name(value: &str) -> Result<String, FontError> {
    #[cfg(test)]
    FONT_NORMALIZATION_CALLS.with(|calls| calls.set(calls.get() + 1));
    super::unicode16::nfkc_casefold(value)
}

fn reserve_cached<T>(values: &mut Vec<T>, additional: usize) -> Result<(), FontError> {
    reserve_cached_with_stage(values, additional, None)
}

fn reserve_cached_with_stage<T>(
    values: &mut Vec<T>,
    additional: usize,

    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), FontError> {
    let required = values
        .len()
        .checked_add(additional)
        .ok_or(FontError::Arithmetic("font cache length"))?;

    if required > values.capacity() {
        reserve_prepared_or_local(values, additional, "operation font cache", prepared_stage)?;
    }
    Ok(())
}

fn copy_name_with_stage(
    source: &str,

    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<String, FontError> {
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
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<FontStack, FontError> {
    let mut output = Vec::new();
    reserve_cached_with_stage(&mut output, source.len(), prepared_stage.as_deref_mut())?;
    for entry in source {
        output.push(FontFamily {
            kind: match &entry.kind {
                FontFamilyKind::Named(name) => {
                    let mut retained = String::new();

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
    resolved_names: &[String],
    requested_weight: RequestedWeight,

    candidates: &mut Vec<CandidateFace>,
) -> Result<(), FontError> {
    for (face_ordinal, face) in faces.iter().enumerate() {
        let Some(stack_entry_index) = first_matching_stack_entry(&face.names, resolved_names)
        else {
            continue;
        };
        let stack_entry_index = u32::try_from(stack_entry_index)
            .map_err(|_| FontError::Arithmetic("font stack entry index"))?;
        let stretch_distance = u8::try_from(face.stretch.to_number().abs_diff(5))
            .map_err(|_| FontError::Arithmetic("font stretch distance"))?;
        candidates.push(CandidateFace {
            occurrence: CandidateOccurrence {
                stack_entry_index,
                style_distance: style_distance(face.style),
                stretch_distance,
                weight_distance: face.weight.abs_diff(requested_weight.value()),
                identity: face.identity,
            },
            face_ordinal,
        });
    }
    Ok(())
}

fn count_candidate_occurrences(faces: &[FaceMetadata], resolved_names: &[String]) -> usize {
    faces
        .iter()
        .filter(|face| first_matching_stack_entry(&face.names, resolved_names).is_some())
        .count()
}

fn count_unique_candidate_survivors(
    faces: &[FaceMetadata],
    resolved_names: &[String],
) -> Result<usize, FontError> {
    let mut count = 0_usize;
    for (face_index, face) in faces.iter().enumerate() {
        if first_matching_stack_entry(&face.names, resolved_names).is_none() {
            continue;
        }
        let mut duplicate = false;
        for earlier in &faces[..face_index] {
            if earlier.identity == face.identity
                && first_matching_stack_entry(&earlier.names, resolved_names).is_some()
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

fn first_matching_stack_entry(
    names: &[NameOccurrence],
    resolved_names: &[String],
) -> Option<usize> {
    // Family records establish membership only. A face matching multiple names
    // belongs at its first matching stack position, regardless of name ID.
    resolved_names
        .iter()
        .position(|resolved| names.iter().any(|name| name.folded == *resolved))
}

fn canonicalize_candidate_occurrences(candidates: &mut Vec<CandidateFace>) {
    candidates.sort_by(|left, right| {
        left.occurrence
            .identity
            .cmp(&right.occurrence.identity)
            .then_with(|| left.occurrence.canonical_cmp(&right.occurrence))
    });

    candidates.dedup_by(|right, left| right.occurrence.identity == left.occurrence.identity);

    candidates.sort_by(|left, right| left.occurrence.canonical_cmp(&right.occurrence));
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
        None,
        has_glyph,
    )
}

fn select_scalar_faces_with_stage<F>(
    pair_ordinal: u32,
    candidates: &[CandidateFace],
    required_scalars: &[char],
    presence: &mut [u64],

    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
    has_glyph: F,
) -> Result<Vec<SelectedScalarFace>, FontError>
where
    F: FnMut(&CandidateFace, char) -> Result<bool, FontError>,
{
    let (scalar_count, rendering_scalar_count) =
        scalar_selection_capacities(required_scalars, presence)?;
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
) -> Result<(usize, usize), FontError> {
    if presence.len() != SCALAR_MAP_WORDS {
        return Err(FontError::Arithmetic("Unicode scalar presence map"));
    }
    presence.fill(0);

    for scalar in required_scalars {
        if crate::text::is_unsupported_control(*scalar) {
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

/// Context is clipped to the selected line, not to a font/script item. Empty
/// pre/post context denotes a real selected-line boundary (BOT/EOT).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ShapingContext<'a> {
    pub(crate) pre: &'a str,
    pub(crate) post: &'a str,
    /// Run-local source byte of a selected U+00AD, rendered as U+002D.
    pub(crate) soft_hyphen: Option<usize>,
}

fn shape_retained_run(
    file: &RetainedFontFile,
    metadata: &FaceMetadata,
    text: &str,
    direction: ShapingDirection,
    script: rustybuzz::Script,
    context: ShapingContext<'_>,
) -> Result<ShapedRun, FontError> {
    if text.chars().any(is_nonrendering_mandatory_break) {
        return Err(FontError::Shape {
            identity: metadata.identity,
            reason: "run crosses a mandatory paragraph break",
        });
    }
    let metrics = metadata.metrics.validate(metadata.identity)?;
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
        validate_selected_glyph_paint(&face, metadata.identity, glyph_id, ink.is_some())?;
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
        metrics,
        glyphs,
    })
}

#[allow(
    clippy::too_many_lines,
    reason = "discovery keeps traversal, retention, and mutation checks in one auditable path"
)]
fn discover_indexed_files(
    roots: &[SourceRoot],
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<Vec<index::IndexedFontFile>, FontError> {
    let mut discovered = Vec::new();
    for root in roots {
        match fs::symlink_metadata(&root.path) {
            Ok(_) => {
                reserve_discovery_record(&mut discovered, prepared_stage.as_deref_mut())?;
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
                if entry_prefix
                    .is_some_and(|prefix| !entry.file_name().to_string_lossy().starts_with(prefix))
                {
                    continue;
                }
                reserve_discovery_record(&mut discovered, prepared_stage.as_deref_mut())?;
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
    discovered.sort_unstable_by(|left, right| {
        native_path_cmp(&left.path, &right.path)
            .then_with(|| left.root_ordinal.cmp(&right.root_ordinal))
    });
    discovered.dedup_by(|right, left| right.path == left.path);
    discovered.sort_unstable_by(|left, right| {
        left.root_ordinal
            .cmp(&right.root_ordinal)
            .then_with(|| native_path_cmp(&left.path, &right.path))
    });

    let mut retained = Vec::new();
    reserve_prepared_or_local(
        &mut retained,
        discovered.len(),
        "indexed font files",
        prepared_stage.as_deref_mut(),
    )?;
    for record in discovered {
        retained.push(index::index_file(
            record.root_ordinal,
            record.path,
            prepared_stage.as_deref_mut(),
        )?);
    }
    Ok(retained)
}

fn reserve_discovery_record(
    records: &mut Vec<DiscoveryRecord>,
    prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), FontError> {
    let required = records
        .len()
        .checked_add(1)
        .ok_or(FontError::Arithmetic("discovery record length"))?;
    if required > records.capacity() {
        let target = required.max(
            records
                .capacity()
                .checked_mul(2)
                .ok_or(FontError::Arithmetic("discovery record capacity"))?,
        );
        target
            .checked_mul(std::mem::size_of::<DiscoveryRecord>())
            .ok_or(FontError::Arithmetic("discovery record bytes"))?;
        reserve_prepared_or_local(
            records,
            target - records.len(),
            "discovery records",
            prepared_stage,
        )?;
    }
    Ok(())
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
        #[cfg(test)]
        LOAD_OBSERVATIONS.with_borrow_mut(|observed| {
            observed.full_bytes += read;
            observed.hash_bytes += read;
        });
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

    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<(), FontError> {
    for preference in source {
        retained.push(FontFamily {
            kind: match &preference.kind {
                FontFamilyKind::Named(name) => {
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

fn apply_host_generics(database: &mut Database, generics: &HostGenericNames) {
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
fn merge_fontconfig_root(
    config: &mut BoundedFontConfig,
    config_path: &Path,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<bool, FontError> {
    let FontConfigRead::Loaded { parts, complete } =
        read_bounded_fontconfig(config, config_path, false, prepared_stage.as_deref_mut())?
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
            FontConfigTask::Merge { path } => {
                // Upstream ignores included syntax failures, but keeps their valid prefix.
                if let FontConfigRead::Loaded { parts, .. } =
                    read_bounded_fontconfig(config, &path, true, prepared_stage.as_deref_mut())?
                {
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
                push_bounded_fontconfig_include(&mut tasks, &path, prepared_stage.as_deref_mut())?;
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
    must_exist: bool,
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<FontConfigRead, FontError> {
    let canonical_path = match fs::canonicalize(config_path) {
        Ok(path) => path,
        Err(source) if !must_exist && source.kind() == std::io::ErrorKind::NotFound => {
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
    for path in &config.files {
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
        operation: "open fontconfig file",
        path: canonical_path.clone(),
        source,
    })?;
    let metadata_before = file.metadata().map_err(|source| FontError::Io {
        operation: "inspect fontconfig file",
        path: canonical_path.clone(),
        source,
    })?;
    if !metadata_before.is_file() {
        return Err(FontError::ChangedFile(canonical_path));
    }
    let length = metadata_before.len();
    let modified_before = metadata_before.modified().ok();
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
    match parse_bounded_fontconfig(&source, config_path, prepared_stage) {
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
        reserve_prepared_or_local(tasks, 1, "fontconfig traversal", prepared_stage)?;
        tasks.push(FontConfigTask::Merge {
            path: include_path.to_path_buf(),
        });
        return Ok(());
    }
    if !metadata.is_dir() {
        return Ok(());
    }
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
    paths.sort_unstable_by(|left, right| native_path_cmp(left, right));
    reserve_prepared_or_local(tasks, paths.len(), "fontconfig traversal", prepared_stage)?;
    for path in paths.into_iter().rev() {
        tasks.push(FontConfigTask::Merge { path });
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn platform_sources(
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
fn platform_sources(
    mut prepared_stage: Option<&mut crate::layout::kernel::PreparedStage<'_>>,
) -> Result<HostSources, FontError> {
    let home = std::env::var("HOME").ok();
    let mut config = BoundedFontConfig::default();
    if let Ok(config_file) = std::env::var("FONTCONFIG_FILE") {
        merge_fontconfig_root(
            &mut config,
            Path::new(&config_file),
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
                prepared_stage.as_deref_mut(),
            )?
        } else {
            true
        };
        if read_global {
            merge_fontconfig_root(
                &mut config,
                Path::new("/etc/fonts/local.conf"),
                prepared_stage.as_deref_mut(),
            )?;
        }
        merge_fontconfig_root(
            &mut config,
            Path::new("/etc/fonts/fonts.conf"),
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

    mod admission;
    mod loading;

    struct Outline(usize);
    impl ttf_parser::OutlineBuilder for Outline {
        fn move_to(&mut self, _: f32, _: f32) {
            self.0 += 1;
        }
        fn line_to(&mut self, _: f32, _: f32) {
            self.0 += 1;
        }
        fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {
            self.0 += 1;
        }
        fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
            self.0 += 1;
        }
        fn close(&mut self) {
            self.0 += 1;
        }
    }

    #[test]
    fn poisoned_environment_returns_synchronization_without_discarding_fonts() {
        let environment = PreparedFontEnvironment::synthetic();
        let operation = environment.operation().unwrap();
        let file_bytes = Arc::clone(&environment.core.read().unwrap().fonts.files[0].bytes);
        let poisoned = environment.clone();
        let panic = std::thread::spawn(move || {
            let _guard = poisoned.core.write().unwrap();
            panic!("intentional writer panic to test a subsequent call's boundary");
        })
        .join();
        assert!(panic.is_err());
        assert!(matches!(
            environment.operation(),
            Err(FontError::Synchronization)
        ));
        assert!(matches!(
            environment.admit_text(test_sans_stack(), RequestedWeight::Normal, "A", None),
            Err(FontError::Synchronization)
        ));
        assert!(matches!(
            operation.seal_fonts(),
            Err(FontError::Synchronization)
        ));
        let core = environment.core.read().unwrap_err().into_inner();
        let fonts = &core.fonts;
        assert!(Arc::ptr_eq(&file_bytes, &fonts.files[0].bytes));
        assert_eq!(fonts.faces.len(), 1);
        assert!(fonts
            .issues
            .iter()
            .all(|issue| !matches!(issue.reason, super::admission::ScanReason::Parser { .. })));
    }

    #[test]
    fn retained_face_lookups_require_the_exact_identity() {
        let environment = PreparedFontEnvironment::synthetic_faces(&[
            ("First", &['A']),
            ("Middle", &['A']),
            ("Last", &['A']),
        ]);
        for target in [0, 2, 3] {
            let identity = if target == 3 {
                FaceIdentity {
                    sha256: [0; 32],
                    face_index: 0,
                }
            } else {
                environment.core.read().unwrap().fonts.faces[target].identity
            };
            for route in 0..3 {
                let mut outline = Outline(0);
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
                if target == 3 {
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
            }
        }
    }

    #[test]
    fn zero_descent_is_not_an_invalid_face_or_permission_to_reselect() {
        let valid = build_test_font_with_coverage("Valid Metrics", &['A']);
        let mut invalid = build_test_font_with_coverage("Invalid Metrics", &['A', 'B']);
        let os2 = ttf_parser::RawFace::parse(&invalid, 0)
            .unwrap()
            .table_records
            .into_iter()
            .find(|record| record.tag == ttf_parser::Tag::from_bytes(b"OS/2"))
            .unwrap();
        // A zero descent is meaningful, not a request for hhea/win replacement.
        set_i16(&mut invalid, usize::try_from(os2.offset).unwrap() + 70, 0);
        let core = CanonicalFontCore::from_generated(
            vec![valid, invalid],
            &HostGenericNames::default(),
            FontPreparation::new(),
            scalar_presence_map(),
        )
        .unwrap();
        let valid = core.faces()[0].identity;
        let invalid = core.faces()[1].identity;
        let environment = PreparedFontEnvironment::for_core(core).unwrap();
        let stack = named_stack(&["Valid Metrics", "Invalid Metrics"]);
        let pair = environment
            .resolve_pair(0, &stack, RequestedWeight::Normal, &['A'])
            .unwrap();
        assert_eq!(pair.provenance.candidates, [valid, invalid]);
        assert_eq!(pair.primary, valid);
        assert_eq!(environment.face_metrics(valid).unwrap().descender, -200);
        environment
            .shape_run(
                valid,
                "A",
                ShapingDirection::LeftToRight,
                rustybuzz::script::LATIN,
            )
            .unwrap();

        environment
            .admit_text(&stack, RequestedWeight::Normal, "B", None)
            .unwrap();
        let pair = environment
            .prepared_pair(&stack, RequestedWeight::Normal)
            .unwrap()
            .unwrap();
        assert_eq!(
            pair.selection_for_unit("B", 'B').unwrap(),
            ScalarFaceSelection::Face(invalid),
        );
        assert_eq!(environment.face_metrics(invalid).unwrap().descender, 0);
        assert_eq!(
            environment
                .shape_run(
                    invalid,
                    "B",
                    ShapingDirection::LeftToRight,
                    rustybuzz::script::LATIN,
                )
                .unwrap()
                .metrics
                .source,
            MetricSource::Typographic,
        );
        let mut outline = Outline(0);
        environment.outline_glyph(invalid, 1, &mut outline).unwrap();
        assert!(outline.0 > 0);
        let invalid_primary = named_stack(&["Invalid Metrics", "Valid Metrics"]);
        let pair = environment
            .resolve_pair(1, &invalid_primary, RequestedWeight::Normal, &[])
            .unwrap();
        assert_eq!(pair.primary, invalid);
        assert_eq!(environment.face_metrics(pair.primary).unwrap().descender, 0);
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
            stack.css().unwrap(),
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
            assert_eq!(named_stack(&[name]).css().unwrap(), format!("\"{name}\""));
        }
        assert_eq!(
            named_stack(&["A\"B\\C", "A  B", "16 Font", "A; B"])
                .css()
                .unwrap(),
            "\"A\\\"B\\\\C\", \"A  B\", \"16 Font\", \"A; B\""
        );
        assert_eq!(
            builtin_font_stack().css().unwrap(),
            "Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif"
        );
        assert_eq!(
            named_stack(&["-Font", "--Font", "-16 Font", "sans-serif Display", "emoji"])
                .css()
                .unwrap(),
            "-Font, --Font, \"-16 Font\", \"sans-serif Display\", \"emoji\""
        );
        assert_eq!(FONT_DISCOVERY_CALLS.with(std::cell::Cell::get), discoveries);
    }

    #[test]
    fn registered_pairs_reuse_normalization_and_selection_after_final_sealing() {
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
        environment.operation().unwrap().seal_fonts().unwrap();
        {
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

    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "android", target_os = "redox"))
    ))]
    #[test]
    fn bounded_fontconfig_missing_optional_paths_but_not_admitted_io_are_ignored() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("absent.conf");
        let mut config = BoundedFontConfig::default();
        assert!(matches!(
            read_bounded_fontconfig(&mut config, &path, false, None),
            Ok(FontConfigRead::Ignored)
        ));
        assert!(matches!(
            read_bounded_fontconfig(&mut config, &path, true, None),
            Err(FontError::Io { .. })
        ));
        fs::write(&path, [0xff]).unwrap();
        assert!(matches!(
            merge_fontconfig_root(&mut config, &path, None),
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
            merge_fontconfig_root(&mut config, &parent, None),
            Err(FontError::Io { .. })
        ));
    }

    #[test]
    fn discovery_closes_handle_on_observed_file_changes() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("a.ttf");
        fs::write(root.path().join("b.ttf"), b"unopened").unwrap();
        let roots = ordinal_roots(vec![root.path().to_path_buf()], None).unwrap();
        for hook in [
            (|path: &Path| fs::write(path, b"x").unwrap()) as fn(&Path),
            (|path: &Path| fs::write(path, b"longer than admitted").unwrap()) as fn(&Path),
        ] {
            fs::write(&path, b"original").unwrap();
            DISCOVERY_HANDLES.with(|state| state.set([0; 3]));
            DISCOVERY_AFTER_ADMISSION.with(|state| state.set(Some(hook)));
            assert!(matches!(discover_indexed_files(&roots, None),
                Err(FontError::ChangedFile(changed)) if changed == fs::canonicalize(&path).unwrap()));
            DISCOVERY_HANDLES.with(|state| assert_eq!(state.get(), [0, 1, 1]));
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
        let (parts, complete) = parse_bounded_fontconfig(source, path, None).unwrap();
        assert!(complete);
        assert_eq!(parts.len(), 1);
        assert!(matches!(
            parse_bounded_fontconfig(
                "<!DOCTYPE fontconfig [<!ENTITY path '/fonts'>]><fontconfig><dir>&path;</dir></fontconfig>",
                path,
                None
            ),
            Err(FontError::ParseFontConfig { .. })
        ));
    }

    #[test]
    fn canonical_candidate_order_retains_least_occurrence_per_identity() {
        let identity_a = identity(1, 0);
        let identity_b = identity(2, 0);
        let identity_c = identity(2, 1);
        let mut candidates = vec![
            candidate(identity_a, 1, 0),
            candidate(identity_c, 0, 0),
            candidate(identity_b, 0, 0),
            candidate(identity_a, 0, 20),
            candidate(identity_a, 0, 10),
        ];

        canonicalize_candidate_occurrences(&mut candidates);
        assert_eq!(
            candidates,
            [
                candidate(identity_b, 0, 0),
                candidate(identity_c, 0, 0),
                candidate(identity_a, 0, 10),
            ]
        );
    }

    #[test]
    fn missing_glyph_is_explicit_and_mandatory_controls_are_not_probed() {
        let candidates = [candidate(identity(1, 0), 0, 0)];

        let mut presence = scalar_presence_map();
        let error = select_scalar_faces(4, &candidates, &['x'], &mut presence, |_, _| Ok(false))
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

        let mut presence = scalar_presence_map();
        let selected = select_scalar_faces(0, &candidates, &controls, &mut presence, |_, _| {
            panic!("mandatory line-break controls must not be probed")
        })
        .unwrap();
        assert!(selected.iter().all(|selected| matches!(
            selected.selection,
            ScalarFaceSelection::NonRenderingMandatoryBreak
        )));
    }

    #[test]
    fn sealed_environment_rejects_every_late_pair_resolution_route() {
        let environment = PreparedFontEnvironment::synthetic();
        let stack = named_stack(&["Test Sans"]);
        environment.reserve_prepared_pairs(1, None).unwrap();
        environment
            .prepare_pair(0, &stack, RequestedWeight::Normal, &['A'], None)
            .unwrap();
        environment.operation().unwrap().seal_fonts().unwrap();

        assert!(matches!(
            environment.resolve_pair(1, &stack, RequestedWeight::Normal, &['A']),
            Err(FontError::PreparedPairAfterSeal { pair_ordinal: 1 })
        ));
        let counts = planning_counts();
        assert_eq!(
            environment
                .admit_text(&stack, RequestedWeight::Normal, "A", None)
                .unwrap(),
            0
        );
        assert!(matches!(
            environment.admit_text(&stack, RequestedWeight::Normal, "B", None),
            Err(FontError::PreparedPairAfterSeal { pair_ordinal: 0 })
        ));
        assert!(matches!(
            environment.admit_text(&stack, RequestedWeight::Bold, "A", None),
            Err(FontError::PreparedPairAfterSeal { pair_ordinal: 1 })
        ));
        assert!(matches!(
            environment.operation().unwrap().seal_fonts(),
            Err(FontError::OperationSealed)
        ));
        assert_eq!(planning_counts(), counts);
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
        weight_distance: u16,
    ) -> CandidateFace {
        CandidateFace {
            occurrence: CandidateOccurrence {
                stack_entry_index,
                style_distance: 0,
                stretch_distance: 0,
                weight_distance,
                identity,
            },
            face_ordinal: 0,
        }
    }

    pub(super) fn generated_core() -> CanonicalFontCore {
        CanonicalFontCore::from_generated(
            vec![build_test_font()],
            &HostGenericNames::default(),
            FontPreparation::new(),
            scalar_presence_map(),
        )
        .unwrap()
    }

    #[test]
    fn retained_units_prefer_complete_faces_without_changing_scalar_or_primary_selection() {
        let files = [
            ("Base", vec!['A']),
            ("Complete", vec!['A', '\u{0301}']),
            ("Mark", vec!['\u{0301}']),
        ]
        .into_iter()
        .map(|(family, coverage)| build_test_font_with_coverage(family, &coverage))
        .collect();
        let environment = PreparedFontEnvironment::for_core(
            CanonicalFontCore::from_generated(
                files,
                &HostGenericNames::default(),
                FontPreparation::new(),
                scalar_presence_map(),
            )
            .unwrap(),
        )
        .unwrap();
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
        environment.operation().unwrap().seal_fonts().unwrap();
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
    fn materialized_collection_can_include_rejected_faces_without_retaining_the_operation() {
        let core = CanonicalFontCore::from_generated(
            vec![build_test_font(), vec![0; 16]],
            &HostGenericNames::default(),
            FontPreparation::new(),
            scalar_presence_map(),
        )
        .unwrap();
        assert_eq!(core.fonts.faces.len(), 1);
        let file_storage = core.fonts.files.as_ptr();
        let environment = PreparedFontEnvironment::for_core(core).unwrap();
        let alive = environment.liveness_probe();
        let operation = environment.operation().unwrap().liveness_probe();
        environment.operation().unwrap().seal_fonts().unwrap();
        let registry = SealedTextVerifier::new(&environment)
            .retain_fonts()
            .unwrap();
        assert!(Arc::ptr_eq(
            &registry,
            &environment.core.read().unwrap().fonts
        ));
        assert_eq!(
            registry.files.len(),
            2,
            "keep the complete collection, including files with no admitted faces"
        );
        assert_eq!(registry.files.as_ptr(), file_storage);
        drop(environment);
        assert!(!alive() && !operation());
        assert_eq!(registry.files[1].bytes.as_slice(), &[0; 16]);
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
