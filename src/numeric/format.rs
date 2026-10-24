//! The sole owned normalized scientific presentation policy. Exact resolved
//! views borrow its affixes; adaptive precision is selected by a semantic set.

use super::decimal::{
    NegativeZero, NumericNotation, NumericRounding, ResolvedFormat, ResolvedPrecision,
    TrailingZeros, MAX_FRACTIONAL_DIGITS, MAX_SIGNIFICANT_DIGITS,
};
use super::emission::{NumericField, NumericRate, NumericSign, NumericSymbol, Presentation};
use super::locale::NumericLocale;
use super::prefixes::{selection::SelectedScale, NumericPrefixes};
use crate::{Error, Result};

/// Exact precision or one uniformly selected precision for a complete numeric set.
/// Construction through [`NumericFormat`] validates all bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericPrecision {
    /// Exactly this many fractional digits, from zero through twenty.
    Fractional(u8),
    /// Exactly this many significant digits, from one through twenty-one.
    Significant(u8),
    /// The smallest common fractional precision distinguishing the complete set.
    AdaptiveFractional {
        /// Inclusive lower bound, from zero through twenty.
        min: u8,
        /// Inclusive upper bound, at least `min` and at most twenty.
        max: u8,
    },
    /// The smallest common significant precision distinguishing the complete set.
    AdaptiveSignificant {
        /// Inclusive lower bound, from one through twenty-one.
        min: u8,
        /// Inclusive upper bound, at least `min` and at most twenty-one.
        max: u8,
    },
}

impl NumericPrecision {
    pub(crate) fn bounds(self) -> Result<(u8, u8)> {
        let (min, max, significant) = match self {
            Self::Fractional(digits) => (digits, digits, false),
            Self::Significant(digits) => (digits, digits, true),
            Self::AdaptiveFractional { min, max } => (min, max, false),
            Self::AdaptiveSignificant { min, max } => (min, max, true),
        };
        let limit = if significant {
            MAX_SIGNIFICANT_DIGITS
        } else {
            MAX_FRACTIONAL_DIGITS
        };
        if min > max || max > limit || (significant && min == 0) {
            return Err(Error::Data(
                "numeric presentation precision bounds are invalid".into(),
            ));
        }
        Ok((min, max))
    }

    fn resolved(self, digits: u8) -> Result<ResolvedPrecision> {
        let (min, max) = self.bounds()?;
        if !(min..=max).contains(&digits) {
            return Err(Error::Data(
                "numeric presentation selected precision is outside its bounds".into(),
            ));
        }
        Ok(match self {
            Self::Fractional(_) | Self::AdaptiveFractional { .. } => {
                ResolvedPrecision::Fractional(digits)
            }
            Self::Significant(_) | Self::AdaptiveSignificant { .. } => {
                ResolvedPrecision::Significant(digits)
            }
        })
    }
}

/// Immutable scientific numeric presentation with explicitly selected locale.
///
/// Rounding defaults to nearest with ties to even, after exact decimal scaling.
/// Affixes are literal: a percent suffix alone does not scale the scientific value.
/// Adaptive precision is selected uniformly for each complete semantic set,
/// independently of geometric fit. The default is Decimal/Fractional(2), scale
/// zero, absent affixes, preserved negative zero and retained trailing zeros.
///
/// ```
/// use lichtung_plot::{NumericFormat, NumericNotation, NumericPrecision, NumericValues, Plot};
/// let percent = NumericFormat::new(NumericNotation::Decimal, NumericPrecision::Fractional(1))?
///     .decimal_scale(2)?
///     .suffix("%")?;
/// assert_eq!(percent.get_decimal_scale(), 2);
/// let plot = Plot::scatter(NumericValues::new("x", [1.0, 2.0]),
///                          NumericValues::new("rate", [0.125, 0.875]))
///     .y_tick_format(percent);
/// # let _ = plot;
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
///
/// Fields and the implementation module stay private:
/// ```compile_fail
/// use lichtung_plot::NumericFormat;
/// let _ = NumericFormat::default().decimal_scale;
/// ```
/// ```compile_fail
/// use lichtung_plot::numeric::format::NumericFormat;
/// ```
/// Precision edits take an explicit precision policy:
/// ```compile_fail
/// use lichtung_plot::NumericFormat;
/// let _ = NumericFormat::default().precision(2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericFormat {
    notation: NumericNotation,
    precision: NumericPrecision,
    decimal_scale: i16,
    prefix: Option<String>,
    suffix: Option<String>,
    negative_zero: NegativeZero,
    trailing_zeros: TrailingZeros,
    rounding: NumericRounding,
    multiplier_bits: u64,
    rate: NumericRate,
    presentation: Presentation,
    prefixes: NumericPrefixes,
    pub(super) units: std::sync::Arc<super::unit::UnitPolicy>,
}

impl Default for NumericFormat {
    fn default() -> Self {
        Self {
            notation: NumericNotation::Decimal,
            precision: NumericPrecision::Fractional(2),
            decimal_scale: 0,
            prefix: None,
            suffix: None,
            negative_zero: NegativeZero::Preserve,
            trailing_zeros: TrailingZeros::Keep,
            rounding: NumericRounding::NearestEven,
            multiplier_bits: 1.0_f64.to_bits(),
            rate: NumericRate::Value,
            presentation: Presentation::default(),
            prefixes: NumericPrefixes::none(),
            units: std::sync::Arc::new(super::unit::UnitPolicy::default()),
        }
    }
}

impl NumericFormat {
    pub(crate) fn try_copy(&self) -> Result<Self> {
        Ok(Self {
            notation: self.notation,
            precision: self.precision,
            decimal_scale: self.decimal_scale,
            prefix: self
                .prefix
                .as_deref()
                .map(crate::data::source::copy_source_string)
                .transpose()?,
            suffix: self
                .suffix
                .as_deref()
                .map(crate::data::source::copy_source_string)
                .transpose()?,
            negative_zero: self.negative_zero,
            trailing_zeros: self.trailing_zeros,
            rounding: self.rounding,
            multiplier_bits: self.multiplier_bits,
            rate: self.rate,
            presentation: self.presentation.clone(),
            prefixes: self.prefixes.clone(),
            units: self.units.clone(),
        })
    }

    /// Construct a validated format with scale zero, no affixes, Preserve, Keep
    /// and NearestEven rounding.
    pub fn new(notation: NumericNotation, precision: NumericPrecision) -> Result<Self> {
        let value = Self {
            notation,
            precision,
            ..Self::default()
        };
        value.validate()?;
        Ok(value)
    }

    /// The selected decimal/exponent notation.
    pub const fn get_notation(&self) -> NumericNotation {
        self.notation
    }
    /// The exact precision or complete adaptive bounds.
    pub const fn get_precision(&self) -> NumericPrecision {
        self.precision
    }
    /// The exact signed decimal display shift, from -324 through 324.
    pub const fn get_decimal_scale(&self) -> i16 {
        self.decimal_scale
    }
    /// Literal text before the sign and number, or absence.
    pub fn get_prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
    /// Literal text immediately after the number, or absence.
    pub fn get_suffix(&self) -> Option<&str> {
        self.suffix.as_deref()
    }
    /// Actual and rounded negative-zero handling.
    pub const fn get_negative_zero(&self) -> NegativeZero {
        self.negative_zero
    }
    /// Fractional trailing-zero handling.
    pub const fn get_trailing_zeros(&self) -> TrailingZeros {
        self.trailing_zeros
    }

    /// Replace notation while retaining and validating every other field.
    pub fn notation(mut self, value: NumericNotation) -> Result<Self> {
        self.notation = value;
        self.validate()?;
        Ok(self)
    }

    /// Replace precision; fractional bounds are 0..=20 and significant bounds 1..=21.
    /// Adaptive `min` must not exceed `max`.
    pub fn precision(mut self, value: NumericPrecision) -> Result<Self> {
        self.precision = value;
        self.validate()?;
        Ok(self)
    }

    /// Replace the exact decimal display shift within -324..=324.
    /// This never changes scientific coordinates or color mapping.
    pub fn decimal_scale(mut self, value: i16) -> Result<Self> {
        self.decimal_scale = value;
        self.validate()?;
        Ok(self)
    }

    /// Replace the literal prefix with nonempty, control-free text.
    /// Spaces and combining marks are preserved exactly.
    pub fn prefix(mut self, value: impl Into<String>) -> Result<Self> {
        self.prefix = Some(value.into());
        self.validate()?;
        Ok(self)
    }

    /// Replace the literal suffix with nonempty, control-free text.
    /// No scaling or spacing is inferred from its contents.
    pub fn suffix(mut self, value: impl Into<String>) -> Result<Self> {
        self.suffix = Some(value.into());
        self.validate()?;
        Ok(self)
    }

    /// Replace handling of actual or rounded negative zero.
    pub fn negative_zero(mut self, value: NegativeZero) -> Result<Self> {
        self.negative_zero = value;
        self.validate()?;
        Ok(self)
    }

    /// Replace fractional zero trimming; integral place-value zeros are retained.
    pub fn trailing_zeros(mut self, value: TrailingZeros) -> Result<Self> {
        self.trailing_zeros = value;
        self.validate()?;
        Ok(self)
    }

    /// Replace exact rounding at the selected decimal quantum.
    /// Signed floor/ceiling and tie handling apply before negative-zero normalization.
    pub fn rounding(mut self, value: NumericRounding) -> Result<Self> {
        self.rounding = value;
        self.validate()?;
        Ok(self)
    }

    /// Multiply by the exact represented finite factor before the one rounding.
    /// Negative factors reverse display signs; either signed zero selects zero.
    pub fn multiplier(mut self, value: f64) -> Result<Self> {
        self.multiplier_bits = if value == 0.0 { 0 } else { value.to_bits() };
        self.validate()?;
        Ok(self)
    }

    /// Select complete explicit locale tokens, without host inference.
    pub fn locale(mut self, value: NumericLocale) -> Result<Self> {
        self.presentation.locale = value;
        self.validate()?;
        Ok(self)
    }
    /// Enable integer grouping using the selected locale's optional pattern.
    pub fn grouping(mut self, value: bool) -> Result<Self> {
        self.presentation.grouping = value;
        self.validate()?;
        Ok(self)
    }
    /// Select sign presentation, after numeric rounding and zero normalization.
    pub fn sign(mut self, value: NumericSign) -> Result<Self> {
        self.presentation.sign = value;
        self.validate()?;
        Ok(self)
    }
    /// Select currency or an applicable nondecimal integer-radix prefix.
    pub fn symbol(mut self, value: NumericSymbol) -> Result<Self> {
        self.presentation.symbol = value;
        self.validate()?;
        Ok(self)
    }
    /// Set or clear minimum Unicode-scalar field width.
    pub fn field(mut self, value: Option<NumericField>) -> Result<Self> {
        self.presentation.field = value;
        self.validate()?;
        Ok(self)
    }
    /// Select an exact rate factor and the locale's automatic suffix.
    pub fn rate(mut self, value: NumericRate) -> Result<Self> {
        self.rate = value;
        self.validate()?;
        Ok(self)
    }

    /// Replace the sole SI, compact or common exact-divisor policy.
    pub fn prefixes(mut self, value: NumericPrefixes) -> Result<Self> {
        self.prefixes = value;
        self.validate()?;
        Ok(self)
    }

    pub(crate) fn prefix_policy(&self) -> &NumericPrefixes {
        &self.prefixes
    }

    pub(super) fn present(&self, ascii: &str, scale: &SelectedScale) -> Result<String> {
        self.presentation.emit(
            ascii,
            self.notation,
            self.rate,
            super::emission::Affixes {
                outer_prefix: self.prefix.as_deref().unwrap_or_default(),
                outer_suffix: self.suffix.as_deref().unwrap_or_default(),
                prefix: &scale.prefix,
                suffix: &scale.suffix,
            },
        )
    }

    pub(super) fn validate(&self) -> Result<()> {
        if let Some(unit) = &self.units.unit {
            unit.validate()?;
        }
        if self.presentation.symbol == NumericSymbol::RadixPrefix
            && !matches!(self.notation, NumericNotation::Integer(radix) if radix != super::decimal::NumericRadix::Decimal)
        {
            return Err(Error::Data(
                "radix prefix requires nondecimal integer notation".into(),
            ));
        }
        let (min, _) = self.precision.bounds()?;
        self.resolve_at(min)?.validate()
    }

    pub(crate) fn resolve_at(&self, digits: u8) -> Result<ResolvedFormat<'_>> {
        Ok(ResolvedFormat {
            notation: self.notation,
            precision: self.precision.resolved(digits)?,
            decimal_scale: self.decimal_scale,
            prefix: self.prefix.as_deref(),
            suffix: self.suffix.as_deref(),
            negative_zero: self.negative_zero,
            trailing_zeros: self.trailing_zeros,
            rounding: self.rounding,
            multiplier: f64::from_bits(self.multiplier_bits),
            rate: self.rate,
            divisor: None,
        })
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<()> {
        let (notation, radix) = match self.notation {
            NumericNotation::Decimal => (0, 0),
            NumericNotation::Scientific => (1, 0),
            NumericNotation::Engineering => (2, 0),
            NumericNotation::General => (3, 0),
            NumericNotation::Integer(radix) => (4, radix as usize),
        };
        round.observe_count(notation)?;
        round.observe_count(radix)?;
        let (min, max) = self.precision.bounds()?;
        round.observe_count(match self.precision {
            NumericPrecision::Fractional(_) => 0,
            NumericPrecision::Significant(_) => 1,
            NumericPrecision::AdaptiveFractional { .. } => 2,
            NumericPrecision::AdaptiveSignificant { .. } => 3,
        })?;
        round.observe_count(usize::from(min))?;
        round.observe_count(usize::from(max))?;
        round.observe_number(f64::from(self.decimal_scale))?;
        round.observe_number(f64::from_bits(self.multiplier_bits))?;
        round.observe_count(self.rate as usize)?;
        self.presentation.observe(round)?;
        self.prefixes.observe(round)?;
        self.units.observe(round)?;
        for affix in [self.get_prefix(), self.get_suffix()] {
            round.observe_count(usize::from(affix.is_some()))?;
            if let Some(text) = affix {
                round.observe_string(text)?;
            }
        }
        round.observe_count(self.negative_zero as usize)?;
        round.observe_count(self.trailing_zeros as usize)?;
        round.observe_count(self.rounding as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_policy_edits_preserve_literal_affixes_and_semantic_precision_bounds() {
        let format = NumericFormat::new(NumericNotation::Decimal, NumericPrecision::Fractional(2))
            .unwrap()
            .decimal_scale(2)
            .unwrap()
            .prefix("[")
            .unwrap()
            .suffix("\u{0301} units]")
            .unwrap();
        assert_eq!(
            format.format_singleton(0.125, false).unwrap(),
            "[12.50\u{0301} units]"
        );
        let normalized = format
            .clone()
            .negative_zero(NegativeZero::Normalize)
            .unwrap();
        assert_ne!(format, normalized);
        assert_eq!(
            format.format_singleton(0.125, false).unwrap(),
            normalized.format_singleton(0.125, false).unwrap()
        );
        assert_eq!(
            normalized.format_singleton(-0.000_001, false).unwrap(),
            "[0.00\u{0301} units]"
        );
        let engineering = normalized
            .notation(NumericNotation::Engineering)
            .unwrap()
            .precision(NumericPrecision::Significant(3))
            .unwrap()
            .decimal_scale(0)
            .unwrap()
            .trailing_zeros(TrailingZeros::Trim)
            .unwrap();
        assert_eq!(
            engineering.format_singleton(999.5, false).unwrap(),
            "[1e3\u{0301} units]"
        );
        assert_eq!(
            format.format_singleton(-0.0, false).unwrap(),
            "[-0.00\u{0301} units]"
        );

        let adaptive = format
            .clone()
            .precision(NumericPrecision::AdaptiveFractional { min: 1, max: 3 })
            .unwrap();
        assert!(adaptive.resolve_at(0).is_err());
        assert!(adaptive.resolve_at(4).is_err());
        assert_eq!(
            adaptive.format_singleton(0.125, false).unwrap(),
            "[12.5\u{0301} units]"
        );

        for precision in [
            NumericPrecision::Fractional(21),
            NumericPrecision::Significant(0),
            NumericPrecision::Significant(22),
            NumericPrecision::AdaptiveFractional { min: 2, max: 1 },
            NumericPrecision::AdaptiveSignificant { min: 0, max: 3 },
            NumericPrecision::AdaptiveSignificant { min: 1, max: 22 },
        ] {
            assert!(matches!(
                format.clone().precision(precision),
                Err(Error::Data(_))
            ));
        }
        for scale in [-325, 325] {
            assert!(matches!(
                format.clone().decimal_scale(scale),
                Err(Error::Data(_))
            ));
        }
        for affix in ["", "\n"] {
            assert!(matches!(format.clone().prefix(affix), Err(Error::Data(_))));
            assert!(matches!(format.clone().suffix(affix), Err(Error::Data(_))));
        }
    }
}
