//! The sole owned normalized scientific presentation policy. Exact resolved
//! views borrow its affixes; adaptive precision is selected by a semantic set.

use super::decimal::{
    self, NegativeZero, NumericNotation, ResolvedFormat, ResolvedPrecision, TrailingZeros,
};
use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NumericPrecision {
    Fractional(u8),
    Significant(u8),
    AdaptiveFractional { min: u8, max: u8 },
    AdaptiveSignificant { min: u8, max: u8 },
}

impl NumericPrecision {
    pub(crate) fn bounds(self) -> Result<(u8, u8)> {
        let (min, max, significant) = match self {
            Self::Fractional(digits) => (digits, digits, false),
            Self::Significant(digits) => (digits, digits, true),
            Self::AdaptiveFractional { min, max } => (min, max, false),
            Self::AdaptiveSignificant { min, max } => (min, max, true),
        };
        if min > max || max > 17 || (significant && min == 0) {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NumericFormat {
    notation: NumericNotation,
    precision: NumericPrecision,
    decimal_scale: i16,
    prefix: Option<String>,
    suffix: Option<String>,
    negative_zero: NegativeZero,
    trailing_zeros: TrailingZeros,
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
        })
    }

    pub(crate) fn new(notation: NumericNotation, precision: NumericPrecision) -> Result<Self> {
        let value = Self {
            notation,
            precision,
            ..Self::default()
        };
        value.validate()?;
        Ok(value)
    }

    pub(crate) const fn notation(&self) -> NumericNotation {
        self.notation
    }
    pub(crate) const fn precision(&self) -> NumericPrecision {
        self.precision
    }
    pub(crate) const fn decimal_scale(&self) -> i16 {
        self.decimal_scale
    }
    pub(crate) fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
    pub(crate) fn suffix(&self) -> Option<&str> {
        self.suffix.as_deref()
    }
    pub(crate) const fn negative_zero(&self) -> NegativeZero {
        self.negative_zero
    }
    pub(crate) const fn trailing_zeros(&self) -> TrailingZeros {
        self.trailing_zeros
    }

    pub(crate) fn with_notation(mut self, value: NumericNotation) -> Result<Self> {
        self.notation = value;
        self.validate()?;
        Ok(self)
    }

    pub(crate) fn with_precision(mut self, value: NumericPrecision) -> Result<Self> {
        self.precision = value;
        self.validate()?;
        Ok(self)
    }

    pub(crate) fn with_decimal_scale(mut self, value: i16) -> Result<Self> {
        self.decimal_scale = value;
        self.validate()?;
        Ok(self)
    }

    pub(crate) fn with_prefix(mut self, value: impl Into<String>) -> Result<Self> {
        self.prefix = Some(value.into());
        self.validate()?;
        Ok(self)
    }

    pub(crate) fn with_suffix(mut self, value: impl Into<String>) -> Result<Self> {
        self.suffix = Some(value.into());
        self.validate()?;
        Ok(self)
    }

    pub(crate) fn with_negative_zero(mut self, value: NegativeZero) -> Result<Self> {
        self.negative_zero = value;
        self.validate()?;
        Ok(self)
    }

    pub(crate) fn with_trailing_zeros(mut self, value: TrailingZeros) -> Result<Self> {
        self.trailing_zeros = value;
        self.validate()?;
        Ok(self)
    }

    fn validate(&self) -> Result<()> {
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
        })
    }

    pub(crate) fn format_ordinary(&self, value: f64) -> Result<String> {
        let digits = match self.precision {
            NumericPrecision::Fractional(digits) | NumericPrecision::Significant(digits) => digits,
            NumericPrecision::AdaptiveFractional { .. }
            | NumericPrecision::AdaptiveSignificant { .. } => {
                return Err(Error::Render(
                    "adaptive numeric formatting requires its complete semantic set".into(),
                ));
            }
        };
        decimal::format_finite(value, &self.resolve_at(digits)?)
    }

    pub(crate) fn generated_alphabet(&self) -> &'static [char] {
        match self.notation {
            NumericNotation::Decimal => {
                &['-', '.', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9']
            }
            NumericNotation::Scientific | NumericNotation::Engineering => &[
                '-', '.', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'e',
            ],
        }
    }

    /// Finite first/last scalar contexts, not sample numbers. Canonical source
    /// preparation extracts each literal/boundary base-plus-mark selection unit.
    pub(crate) fn boundary_sources(&self) -> Result<Vec<String>> {
        let mut sources = Vec::new();
        let mut join = |a: &str, b: &str| -> Result<()> {
            let mut text = String::new();
            let length = a
                .len()
                .checked_add(b.len())
                .ok_or_else(|| Error::Render("numeric affix boundary length overflow".into()))?;
            crate::data::source::reserve_source_string(&mut text, length)?;
            text.push_str(a);
            text.push_str(b);
            crate::data::source::reserve_source(&mut sources, 1)?;
            sources.push(text);
            Ok(())
        };
        if let Some(prefix) = self.prefix() {
            for first in ["-", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9"] {
                join(prefix, first)?;
            }
        }
        if let Some(suffix) = self.suffix() {
            for last in ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"] {
                join(last, suffix)?;
            }
        }
        Ok(sources)
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<()> {
        round.observe_count(self.notation as usize)?;
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
        for affix in [self.prefix(), self.suffix()] {
            round.observe_count(usize::from(affix.is_some()))?;
            if let Some(text) = affix {
                round.observe_string(text)?;
            }
        }
        round.observe_count(self.negative_zero as usize)?;
        round.observe_count(self.trailing_zeros as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_policy_edits_preserve_literal_affixes_and_require_resolved_precision() {
        let format = NumericFormat::new(NumericNotation::Decimal, NumericPrecision::Fractional(2))
            .unwrap()
            .with_decimal_scale(2)
            .unwrap()
            .with_prefix("[")
            .unwrap()
            .with_suffix("\u{0301} units]")
            .unwrap();
        assert_eq!(
            format.format_ordinary(0.125).unwrap(),
            "[12.50\u{0301} units]"
        );
        let normalized = format
            .clone()
            .with_negative_zero(NegativeZero::Normalize)
            .unwrap();
        assert_ne!(format, normalized);
        assert_eq!(
            format.format_ordinary(0.125).unwrap(),
            normalized.format_ordinary(0.125).unwrap()
        );
        assert_eq!(
            normalized.format_ordinary(-0.000_001).unwrap(),
            "[0.00\u{0301} units]"
        );
        let engineering = normalized
            .with_notation(NumericNotation::Engineering)
            .unwrap()
            .with_precision(NumericPrecision::Significant(3))
            .unwrap()
            .with_decimal_scale(0)
            .unwrap()
            .with_trailing_zeros(TrailingZeros::Trim)
            .unwrap();
        assert_eq!(
            engineering.format_ordinary(999.5).unwrap(),
            "[1e3\u{0301} units]"
        );
        assert_eq!(
            format.format_ordinary(-0.0).unwrap(),
            "[-0.00\u{0301} units]"
        );

        let adaptive = format
            .clone()
            .with_precision(NumericPrecision::AdaptiveFractional { min: 1, max: 3 })
            .unwrap();
        assert!(matches!(
            adaptive.format_ordinary(0.125),
            Err(Error::Render(_))
        ));
        assert!(adaptive.resolve_at(0).is_err());
        assert!(adaptive.resolve_at(4).is_err());
        assert_eq!(
            decimal::format_finite(0.125, &adaptive.resolve_at(1).unwrap()).unwrap(),
            "[12.5\u{0301} units]"
        );

        for precision in [
            NumericPrecision::Fractional(18),
            NumericPrecision::Significant(0),
            NumericPrecision::Significant(18),
            NumericPrecision::AdaptiveFractional { min: 2, max: 1 },
            NumericPrecision::AdaptiveSignificant { min: 0, max: 3 },
            NumericPrecision::AdaptiveSignificant { min: 1, max: 18 },
        ] {
            assert!(matches!(
                format.clone().with_precision(precision),
                Err(Error::Data(_))
            ));
        }
        for scale in [-325, 325] {
            assert!(matches!(
                format.clone().with_decimal_scale(scale),
                Err(Error::Data(_))
            ));
        }
        for affix in ["", "\n"] {
            assert!(matches!(
                format.clone().with_prefix(affix),
                Err(Error::Data(_))
            ));
            assert!(matches!(
                format.clone().with_suffix(affix),
                Err(Error::Data(_))
            ));
        }
    }
}
