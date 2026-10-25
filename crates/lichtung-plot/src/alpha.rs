//! Owned byte alpha and explicit, eagerly materialized explanatory semantics.

use std::sync::Arc;

use crate::data::source::{collect_source, reserve_source, source_arc};
use crate::numeric::set::FormattedSet;
use crate::{CategoryValues, Error, NumericValues, Result};

mod encoding;
mod guide;
pub(crate) mod marks;
mod materialize;

pub use encoding::AlphaEncoding;
pub(crate) use encoding::{AlphaUnitKind, PreparedAlpha, ResolvedAlpha};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AlphaTarget {
    Point,
    Line,
    Fill,
}

impl AlphaTarget {
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Point => 0,
            Self::Line => 1,
            Self::Fill => 2,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct AlphaInputs([AlphaEncoding; 3]);

impl AlphaInputs {
    pub(crate) fn get(&self, target: AlphaTarget) -> &AlphaEncoding {
        &self.0[target.index()]
    }

    pub(crate) fn set(&mut self, target: AlphaTarget, encoding: AlphaEncoding) {
        self.0[target.index()] = encoding;
    }
}

/// Explicit explanatory semantics carried by immutable [`AlphaValues`].
///
/// Semantics are declared by construction, never inferred from byte cardinality
/// or a custom function's outputs. Only declared mappings produce alpha sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaSemantics {
    /// Caller-computed bytes with no automatic alpha explanation.
    Undeclared,
    /// A declared continuous source/domain illustrated by numeric symbols.
    Continuous,
    /// A complete ordered raw category-to-alpha mapping.
    Categorical,
    /// A complete ordered interval-to-alpha mapping with equality on the right.
    Threshold,
}

/// Owned byte alpha values and their complete immutable mapping declaration.
///
/// Construction eagerly materializes custom functions; no callback survives in a
/// Plot, Canvas or output. Alpha zero preserves scientific membership. Attach with
/// [`AlphaEncoding`] to select observations, explicit collective keys or bins.
///
/// ```
/// use lichtung_plot::{AlphaSemantics, AlphaValues, NumericValues};
/// let alpha = AlphaValues::linear(
///     NumericValues::new("confidence", [0.0, 0.25, 0.5, 0.75, 1.0]),
///     (0.0, 1.0), (0, 255),
/// )?;
/// assert_eq!(alpha.semantics(), AlphaSemantics::Continuous);
/// assert_eq!((0..alpha.len()).map(|i| alpha.get(i).unwrap()).collect::<Vec<_>>(),
///     [0, 64, 128, 191, 255]);
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
///
/// The representation is private:
/// ```compile_fail
/// use lichtung_plot::AlphaValues;
/// fn private(values: AlphaValues) { let _ = values.declaration; }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlphaValues {
    name: Arc<String>,
    data: Arc<AlphaData>,
    declaration: Arc<AlphaDeclaration>,
}

#[derive(Debug, PartialEq, Eq)]
struct AlphaData {
    values: Vec<u8>,
    members: AlphaMembers,
}

#[derive(Debug, PartialEq, Eq)]
enum AlphaMembers {
    Undeclared,
    Numeric(Vec<u64>),
    Categorical(Vec<usize>),
    Threshold(Vec<(u64, usize)>),
}

/// Recipes retain the complete declaration independently of selected occurrences.
#[derive(Debug, PartialEq, Eq)]
enum AlphaDeclaration {
    Undeclared,
    Continuous {
        domain: [u64; 2],
        rule: ContinuousRule,
        examples: FormattedSet,
        alphas: Vec<u8>,
    },
    Categorical(Vec<(String, u8)>),
    Threshold {
        boundaries: FormattedSet,
        alphas: Vec<u8>,
        labels: Vec<String>,
    },
}

#[derive(Debug, PartialEq, Eq)]
enum ContinuousRule {
    Linear((u8, u8)),
    Log10 {
        range: (u8, u8),
        coordinates: [u64; 2],
    },
    Custom(Vec<(u64, u8)>),
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "public consuming factories materialize and release one complete immutable source input"
)]
impl AlphaValues {
    /// Own arbitrary caller-computed bytes without declaring guide semantics.
    ///
    /// Names must be nonempty, already trimmed and control-free. Empty values
    /// are valid; attachment checks the actual unit alignment before fonts.
    pub fn new(name: impl Into<String>, values: impl IntoIterator<Item = u8>) -> Result<Self> {
        let name = validated_name(name.into())?;
        Self::from_parts(
            name,
            collect_input(values)?,
            AlphaMembers::Undeclared,
            AlphaDeclaration::Undeclared,
        )
    }

    /// Map finite source values linearly over an explicit increasing domain.
    ///
    /// Clamp out-of-domain values. Ascending, descending and constant byte ranges
    /// are valid; quantization is nearest/ties-to-even. Missing/nonfinite sources
    /// and nonfinite/nonincreasing domains reject during construction.
    pub fn linear(source: NumericValues, domain: (f64, f64), range: (u8, u8)) -> Result<Self> {
        materialize::builtin(&source, domain, range, false)
    }

    /// Map positive finite source values over an explicit positive log10 domain.
    ///
    /// Endpoints must be strictly increasing and remain distinguishable after
    /// binary64 log10. Nonpositive sources reject rather than clamping. Other
    /// clamping/quantization rules match [`Self::linear`].
    pub fn log10(source: NumericValues, domain: (f64, f64), range: (u8, u8)) -> Result<Self> {
        materialize::builtin(&source, domain, range, true)
    }

    /// Declare a continuous mapping and eagerly materialize its finite values.
    ///
    /// Invoke `f` once for every distinct clamped source coordinate and required
    /// numeric representative, in numeric order. Retain the complete evaluated
    /// table and original members; no function, inverse or smooth ramp is stored.
    /// The domain must be finite and strictly increasing, and sources nonmissing
    /// and finite. Up to five original-domain examples explain this declaration.
    pub fn continuous(
        source: NumericValues,
        domain: (f64, f64),
        f: impl FnMut(f64) -> u8,
    ) -> Result<Self> {
        materialize::continuous(&source, domain, f)
    }

    /// Declare a categorical map, invoking `f` once per first-observed raw level.
    ///
    /// Null sources reject. Complete table order survives [`Self::select`]; only
    /// actual consumed categories appear in automatic explanations, including zero.
    pub fn categorical(source: CategoryValues, f: impl FnMut(&str) -> u8) -> Result<Self> {
        materialize::categorical(&source, f)
    }

    /// Declare increasing finite boundaries and exactly one more alpha byte.
    ///
    /// Intervals are `< b0`, `[b0,b1)`, ..., `>= blast`; equality belongs to the
    /// right. No boundaries plus one byte declares `all values`. Sources must be
    /// finite and nonmissing. Labels come from the actual bounds, not caller text.
    pub fn threshold(
        source: NumericValues,
        boundaries: impl IntoIterator<Item = f64>,
        alphas: impl IntoIterator<Item = u8>,
    ) -> Result<Self> {
        materialize::threshold(&source, boundaries, alphas)
    }

    /// Select original occurrences while retaining the complete shared declaration.
    ///
    /// Repeated indices are allowed; out-of-range indices reject. Construct pooled
    /// custom/category maps once and select per Plot to preserve Shared recipes.
    pub fn select(&self, indices: impl IntoIterator<Item = usize>) -> Result<Self> {
        let indices = collect_input(indices)?;
        let values = collect_source(indices.iter().map(|&index| {
            self.get(index).ok_or_else(|| {
                Error::Data(format!(
                    "alpha values {:?} selection index {index} is outside length {}",
                    self.name(),
                    self.len(),
                ))
            })
        }))?;
        let members = match &self.data.members {
            AlphaMembers::Undeclared => AlphaMembers::Undeclared,
            AlphaMembers::Numeric(members) => {
                AlphaMembers::Numeric(collect_source(indices.iter().map(|&i| Ok(members[i])))?)
            }
            AlphaMembers::Categorical(members) => {
                AlphaMembers::Categorical(collect_source(indices.iter().map(|&i| Ok(members[i])))?)
            }
            AlphaMembers::Threshold(members) => {
                AlphaMembers::Threshold(collect_source(indices.iter().map(|&i| Ok(members[i])))?)
            }
        };
        Ok(Self {
            name: Arc::clone(&self.name),
            data: source_arc(AlphaData { values, members })?,
            declaration: Arc::clone(&self.declaration),
        })
    }

    /// Raw source identity used for automatic titles and diagnostics.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Number of retained occurrences.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.values.len()
    }

    /// Whether no occurrences are retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.values.is_empty()
    }

    /// One occurrence's final byte, or None outside the retained input.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<u8> {
        self.data.values.get(index).copied()
    }

    /// Explicit explanatory classification, independent of selected byte values.
    #[must_use]
    pub fn semantics(&self) -> AlphaSemantics {
        match self.declaration.as_ref() {
            AlphaDeclaration::Undeclared => AlphaSemantics::Undeclared,
            AlphaDeclaration::Continuous { .. } => AlphaSemantics::Continuous,
            AlphaDeclaration::Categorical(_) => AlphaSemantics::Categorical,
            AlphaDeclaration::Threshold { .. } => AlphaSemantics::Threshold,
        }
    }

    fn from_parts(
        name: String,
        values: Vec<u8>,
        members: AlphaMembers,
        declaration: AlphaDeclaration,
    ) -> Result<Self> {
        Ok(Self {
            name: source_arc(name)?,
            data: source_arc(AlphaData { values, members })?,
            declaration: source_arc(declaration)?,
        })
    }
}

fn validated_name(name: String) -> Result<String> {
    if name.is_empty() || name.trim() != name || name.chars().any(char::is_control) {
        return Err(Error::Data(
            "alpha source names must be nonempty, already trimmed and control-free".into(),
        ));
    }
    Ok(name)
}

fn numeric_key(value: f64) -> u64 {
    if value == 0.0 {
        0
    } else {
        value.to_bits()
    }
}

fn collect_input<T>(values: impl IntoIterator<Item = T>) -> Result<Vec<T>> {
    let values = values.into_iter();
    let mut retained = Vec::new();
    let (minimum, maximum) = values.size_hint();
    if maximum == Some(minimum) {
        reserve_source(&mut retained, minimum)?;
    }
    for value in values {
        if retained.len() == retained.capacity() {
            let additional = retained.capacity().max(1);
            reserve_source(&mut retained, additional)?;
        }
        retained.push(value);
    }
    Ok(retained)
}

#[cfg(test)]
mod tests;
