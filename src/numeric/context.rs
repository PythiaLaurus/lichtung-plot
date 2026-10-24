//! Library-created borrowed numeric views; no geometry or mutable renderer escapes.

use super::format::NumericFormat;
use super::prefixes::selection::SelectedScale;
use super::set::FormattedSet;
use crate::{
    AxisScale, FacetCell, NumericFactor, NumericLocale, NumericRate, NumericUnit, PlotKind, Result,
    Text,
};

/// Scientific coordinate axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateAxis {
    /// Horizontal coordinate.
    X,
    /// Vertical coordinate.
    Y,
}
/// Complete numeric consumer, independent of its selected typography.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericRole {
    /// Horizontal ticks.
    XAxis,
    /// Vertical ticks.
    YAxis,
    /// Continuous color guide.
    ContinuousColor,
    /// Active finite cell values.
    HeatmapValues,
    /// Available point-only estimates.
    CoefficientLabels,
    /// Eligible base/terminal totals.
    WaterfallTotals,
}
/// Whether a set is an intrinsic preference or a reached actual source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericPurpose {
    /// Low/mid/high preference, not a required unique source.
    IntrinsicProbe,
    /// Actual semantic occurrences.
    ActualSet,
}
/// Scientific domain provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericRangeOrigin {
    /// Locally automatic.
    Automatic,
    /// Caller-configured limits.
    Explicit,
    /// Parent-resolved scientific limits.
    ParentResolved,
}

/// Exact operand representation, retaining binary64 source bits and power exponents.
#[derive(Debug, Clone, Copy)]
pub enum NumericOperand {
    /// The represented finite ordinary operand.
    Binary64(f64),
    /// A mathematical power, not a rounded physical `powf` result.
    PowerOfTen {
        /// The exact represented exponent.
        exponent: f64,
    },
}

/// Raw semantic identity, before display aliases.
#[derive(Debug, Clone)]
pub enum NumericIdentity<'a> {
    /// A generated position in its original complete source order.
    Tick {
        /// Owning coordinate.
        axis: CoordinateAxis,
        /// Original occurrence.
        source_index: usize,
    },
    /// A final-domain guide position.
    GuideTick {
        /// Original occurrence.
        source_index: usize,
    },
    /// Raw cell keys.
    HeatmapCell {
        /// Raw row level.
        row: &'a str,
        /// Raw column level.
        column: &'a str,
    },
    /// Raw available estimate category.
    EstimateRow {
        /// Raw category level.
        category: &'a str,
    },
    /// Base total.
    WaterfallBase,
    /// Terminal total.
    WaterfallTerminal,
    /// An intrinsic low/mid/high preference.
    Probe {
        /// Probe occurrence.
        index: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OwnerInfo {
    pub(crate) kind: Option<PlotKind>,
    pub(crate) canvas: Option<(usize, usize)>,
    pub(crate) facet: Option<FacetCell>,
    pub(crate) shared: bool,
    pub(crate) scope: Option<String>,
}

/// Semantic producer description, never a kernel/backend handle.
#[derive(Debug, Clone)]
pub struct NumericOwner<'a>(pub(crate) &'a OwnerInfo);
impl<'a> NumericOwner<'a> {
    /// The local family, or absence for a genuinely shared heterogeneous owner.
    pub const fn plot_kind(&self) -> Option<PlotKind> {
        self.0.kind
    }
    /// Explicit Canvas anchor coordinate.
    pub const fn canvas_cell(&self) -> Option<(usize, usize)> {
        self.0.canvas
    }
    /// Raw facet levels.
    pub fn facet_cell(&self) -> Option<&'a FacetCell> {
        self.0.facet.as_ref()
    }
    /// Whether this one result is shared by contributing owners.
    pub const fn is_shared(&self) -> bool {
        self.0.shared
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Identity {
    Tick(CoordinateAxis, usize),
    GuideTick(usize),
    Heatmap(String, String),
    Estimate(String),
    Base,
    Terminal,
    Probe(usize),
}
impl Identity {
    fn view(&self) -> NumericIdentity<'_> {
        match self {
            Self::Tick(axis, index) => NumericIdentity::Tick {
                axis: *axis,
                source_index: *index,
            },
            Self::GuideTick(index) => NumericIdentity::GuideTick {
                source_index: *index,
            },
            Self::Heatmap(row, column) => NumericIdentity::HeatmapCell { row, column },
            Self::Estimate(category) => NumericIdentity::EstimateRow { category },
            Self::Base => NumericIdentity::WaterfallBase,
            Self::Terminal => NumericIdentity::WaterfallTerminal,
            Self::Probe(index) => NumericIdentity::Probe { index: *index },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextValue {
    pub(crate) bits: u64,
    pub(crate) power: bool,
    pub(crate) identity: Identity,
}
impl ContextValue {
    fn operand(&self) -> NumericOperand {
        if self.power {
            NumericOperand::PowerOfTen {
                exponent: f64::from_bits(self.bits),
            }
        } else {
            NumericOperand::Binary64(f64::from_bits(self.bits))
        }
    }
}

/// One exact source occurrence in a complete set.
#[derive(Debug, Clone)]
pub struct NumericValue<'a>(pub(crate) &'a ContextValue);
impl<'a> NumericValue<'a> {
    /// Original ordinary or mathematical operand.
    pub fn operand(&self) -> NumericOperand {
        self.0.operand()
    }
    /// Original semantic identity.
    pub fn identity(&self) -> NumericIdentity<'a> {
        self.0.identity.view()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DomainInfo {
    pub(crate) limits: (u64, u64),
    pub(crate) data: Option<(u64, u64)>,
    pub(crate) scale: AxisScale,
    pub(crate) origin: NumericRangeOrigin,
}

/// Final scientific domain, without CSS geometry or invented logarithmic inverses.
#[derive(Debug, Clone)]
pub struct NumericDomain<'a>(pub(crate) &'a DomainInfo);
impl NumericDomain<'_> {
    /// Lower mathematical endpoint.
    pub fn lower(&self) -> NumericOperand {
        endpoint(self.0.limits.0, self.0.scale)
    }
    /// Upper mathematical endpoint.
    pub fn upper(&self) -> NumericOperand {
        endpoint(self.0.limits.1, self.0.scale)
    }
    /// Raw bounds only when genuinely retained as finite endpoints.
    pub fn data_limits(&self) -> Option<(f64, f64)> {
        self.0
            .data
            .map(|(a, b)| (f64::from_bits(a), f64::from_bits(b)))
    }
    /// Resolved scientific scale coordinates.
    pub fn scale_limits(&self) -> (f64, f64) {
        (
            f64::from_bits(self.0.limits.0),
            f64::from_bits(self.0.limits.1),
        )
    }
    /// Scientific projection scale.
    pub const fn scale(&self) -> AxisScale {
        self.0.scale
    }
    /// Original domain provenance.
    pub const fn origin(&self) -> NumericRangeOrigin {
        self.0.origin
    }
}
fn endpoint(bits: u64, scale: AxisScale) -> NumericOperand {
    if scale == AxisScale::Log10 {
        NumericOperand::PowerOfTen {
            exponent: f64::from_bits(bits),
        }
    } else {
        NumericOperand::Binary64(f64::from_bits(bits))
    }
}

/// Resolved exact factors and literal selected affixes for one member/common set.
#[derive(Debug, Clone)]
pub struct NumericFactors<'a> {
    pub(crate) multiplier: f64,
    pub(crate) decimal_scale: i16,
    pub(crate) rate: NumericRate,
    pub(crate) scale: &'a SelectedScale,
    pub(crate) unit: Option<&'a NumericUnit>,
}
impl<'a> NumericFactors<'a> {
    /// Exact represented public multiplier.
    pub const fn multiplier(&self) -> f64 {
        self.multiplier
    }
    /// Exact decimal shift.
    pub const fn decimal_scale(&self) -> i16 {
        self.decimal_scale
    }
    /// Explicit rate factor.
    pub const fn rate(&self) -> NumericRate {
        self.rate
    }
    /// A resolved divisor, never AutomaticDecimal.
    pub const fn divisor(&self) -> &'a NumericFactor {
        &self.scale.divisor
    }
    /// Selected literal prefix, not the outer literal affix.
    pub fn prefix(&self) -> &'a str {
        &self.scale.prefix
    }
    /// Selected literal suffix, not the outer literal affix.
    pub fn suffix(&self) -> &'a str {
        &self.scale.suffix
    }
    /// Explicit unit policy.
    pub const fn unit(&self) -> Option<&'a NumericUnit> {
        self.unit
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextInfo {
    pub(crate) owner: OwnerInfo,
    pub(crate) role: NumericRole,
    pub(crate) purpose: NumericPurpose,
    pub(crate) values: Vec<ContextValue>,
    pub(crate) domain: Option<DomainInfo>,
    pub(crate) generation: Option<crate::TickGeneration>,
    pub(crate) window: Option<[(u64, u64); 2]>,
}

/// Ordered complete semantic input and nonrecursive built-in helper.
#[derive(Debug, Clone)]
pub struct NumericSetContext<'a> {
    pub(crate) info: &'a ContextInfo,
    pub(crate) values: &'a [NumericValue<'a>],
    pub(crate) format: &'a NumericFormat,
    pub(crate) metadata: &'a FormattedSet,
    pub(crate) operands: &'a [f64],
    pub(crate) power: bool,
    pub(crate) required: bool,
    pub(crate) helper: &'a std::cell::OnceCell<FormattedSet>,
}
impl<'a> NumericSetContext<'a> {
    /// Semantic producer.
    pub fn owner(&self) -> NumericOwner<'a> {
        NumericOwner(&self.info.owner)
    }
    /// Actual consumer.
    pub const fn role(&self) -> NumericRole {
        self.info.role
    }
    /// Preference versus actual source.
    pub const fn purpose(&self) -> NumericPurpose {
        self.info.purpose
    }
    /// Every original occurrence, including duplicates, in source order.
    pub const fn values(&self) -> &'a [NumericValue<'a>] {
        self.values
    }
    /// Resolved domain when this owner has one.
    pub fn domain(&self) -> Option<NumericDomain<'a>> {
        self.info.domain.as_ref().map(NumericDomain)
    }
    /// Explicit selected locale.
    pub fn locale(&self) -> &'a NumericLocale {
        self.format.locale_ref()
    }
    /// Common resolved factor/affix only when every member agrees.
    pub fn common_factors(&self) -> Option<NumericFactors<'a>> {
        let first = self.metadata.scales.first()?;
        self.metadata
            .scales
            .iter()
            .all(|scale| scale == first)
            .then(|| self.factors(0))
    }
    /// Built-in conversion of this same complete set, bypassing only the function.
    pub fn format_builtin(&self) -> Result<NumericLabels> {
        let builtin = self.builtin()?;
        Ok(NumericLabels {
            labels: crate::data::source::collect_source(
                builtin
                    .labels
                    .iter()
                    .map(|label| crate::data::source::copy_source_string(label)),
            )?,
            explanation: builtin
                .explanation
                .as_ref()
                .map(Text::try_copy)
                .transpose()?,
        })
    }
    fn builtin(&self) -> Result<&FormattedSet> {
        if self.helper.get().is_none() {
            let result = self.format.function_helper(
                self.operands,
                self.power,
                &self.metadata.scales,
                self.required,
                self.info.purpose == NumericPurpose::IntrinsicProbe,
            )?;
            self.helper.set(result).map_err(|_| {
                crate::Error::Render("numeric helper was initialized recursively".into())
            })?;
        }
        self.helper
            .get()
            .ok_or_else(|| crate::Error::Render("numeric helper result is missing".into()))
    }
    fn factors(&self, index: usize) -> NumericFactors<'a> {
        let (multiplier, decimal_scale, rate) = self.format.factor_inputs();
        NumericFactors {
            multiplier,
            decimal_scale,
            rate,
            scale: &self.metadata.scales[index],
            unit: self.metadata.unit.as_ref(),
        }
    }
}

/// One original occurrence, retaining its complete set and helper precision.
#[derive(Debug, Clone)]
pub struct NumericValueContext<'a> {
    pub(crate) set: &'a NumericSetContext<'a>,
    pub(crate) index: usize,
}
impl<'a> NumericValueContext<'a> {
    /// The complete enclosing semantic set.
    pub const fn set(&self) -> &NumericSetContext<'a> {
        self.set
    }
    /// Original occurrence index.
    pub const fn index(&self) -> usize {
        self.index
    }
    /// Original operand and identity.
    pub fn value(&self) -> &NumericValue<'a> {
        &self.set.values[self.index]
    }
    /// Resolved member factor, even with PerValue selection.
    pub fn factors(&self) -> NumericFactors<'a> {
        self.set.factors(self.index)
    }
    /// Built-in text under the enclosing complete set's precision/factors.
    pub fn format_builtin(&self) -> Result<String> {
        crate::data::source::copy_source_string(&self.set.builtin()?.labels[self.index])
    }
}

/// Owned final callback strings and an optional replacement common explanation.
#[derive(Debug, Clone)]
pub struct NumericLabels {
    labels: Vec<String>,
    explanation: Option<Text>,
}
impl NumericLabels {
    /// Retain final strings without postformatting.
    pub const fn new(labels: Vec<String>) -> Self {
        Self {
            labels,
            explanation: None,
        }
    }
    /// Replace the one configured/generated explanation, retaining ordinary Text semantics.
    #[must_use]
    pub fn explanation(mut self, value: impl Into<Text>) -> Self {
        self.explanation = Some(value.into());
        self
    }
    /// Final strings in occurrence order.
    pub fn labels(&self) -> &[String] {
        &self.labels
    }
    /// Optional replacement explanation.
    pub const fn get_explanation(&self) -> Option<&Text> {
        self.explanation.as_ref()
    }
    /// Consume the owned result.
    pub fn into_parts(self) -> (Vec<String>, Option<Text>) {
        (self.labels, self.explanation)
    }
}
