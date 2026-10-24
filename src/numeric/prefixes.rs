//! One normalized scale/prefix request for exact numeric operands.

use std::sync::Arc;

use super::locale::validate_token;
use crate::{Error, Result};
use num_bigint::BigInt;

pub(crate) mod selection;
#[cfg(test)]
mod tests;

/// Whether a scale is chosen separately, for the complete set, or explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericSelection<T> {
    /// Choose separately for each member.
    PerValue,
    /// Choose one common factor from the entire semantic set.
    WholeSet,
    /// Retain a caller-selected factor or rule.
    Fixed(T),
}

/// Handling of a rounding carry into the next automatic prefix threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericPromotion {
    /// Select only from the original unrounded magnitude.
    KeepThreshold,
    /// Resolve the final threshold from the original signed rounding cell.
    PromoteOnRounding,
}

/// A standard SI prefix, including neutral, deci/centi/deca/hecto and 2022 additions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SiPrefix(i8);

const SI: &[(i8, &str)] = &[
    (-30, "q"),
    (-27, "r"),
    (-24, "y"),
    (-21, "z"),
    (-18, "a"),
    (-15, "f"),
    (-12, "p"),
    (-9, "n"),
    (-6, "µ"),
    (-3, "m"),
    (-2, "c"),
    (-1, "d"),
    (0, ""),
    (1, "da"),
    (2, "h"),
    (3, "k"),
    (6, "M"),
    (9, "G"),
    (12, "T"),
    (15, "P"),
    (18, "E"),
    (21, "Z"),
    (24, "Y"),
    (27, "R"),
    (30, "Q"),
];

impl SiPrefix {
    /// Validate a standard decimal exponent.
    pub fn from_exponent(exponent: i8) -> Result<Self> {
        if SI.iter().any(|(e, _)| *e == exponent) {
            Ok(Self(exponent))
        } else {
            Err(Error::Data("unsupported SI prefix exponent".into()))
        }
    }
    /// Exact decimal exponent.
    pub const fn exponent(&self) -> i8 {
        self.0
    }
    /// Standard symbol; micro is U+00B5.
    pub fn symbol(&self) -> &str {
        SI.iter()
            .find(|(e, _)| *e == self.0)
            .expect("validated SI prefix")
            .1
    }
}

/// An exact divisor or an automatic common decimal-power request.
#[derive(Debug, Clone)]
pub enum NumericFactor {
    /// Use the decade of the greatest nonzero magnitude in the whole set.
    AutomaticDecimal,
    /// The exact represented nonzero finite divisor.
    Value(f64),
    /// A mathematical power with an arbitrary-size signed ASCII integral exponent.
    PowerOfTen(String),
}

impl PartialEq for NumericFactor {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::AutomaticDecimal, Self::AutomaticDecimal) => true,
            (Self::Value(a), Self::Value(b)) => a.to_bits() == b.to_bits(),
            (Self::PowerOfTen(a), Self::PowerOfTen(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for NumericFactor {}

impl NumericFactor {
    pub(super) fn try_copy(&self) -> Result<Self> {
        Ok(match self {
            Self::AutomaticDecimal => Self::AutomaticDecimal,
            Self::Value(value) => Self::Value(*value),
            Self::PowerOfTen(value) => {
                Self::PowerOfTen(crate::data::source::copy_source_string(value)?)
            }
        })
    }

    fn observe(&self, round: &mut crate::layout::kernel::FeedbackRound) -> Result<()> {
        match self {
            Self::AutomaticDecimal => round.observe_count(0),
            Self::Value(value) => {
                round.observe_count(1)?;
                round.observe_number(*value)
            }
            Self::PowerOfTen(value) => {
                round.observe_count(2)?;
                round.observe_string(value)
            }
        }
    }
    fn normalize(self) -> Result<Self> {
        match self {
            Self::Value(value) if !value.is_finite() || value == 0.0 => Err(Error::Data(
                "numeric divisor must be nonzero and finite".into(),
            )),
            Self::PowerOfTen(source) => {
                let digits = source.strip_prefix(['+', '-']).unwrap_or(&source);
                if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(Error::Data(
                        "decimal factor exponent must be a signed ASCII integer".into(),
                    ));
                }
                let exponent = BigInt::parse_bytes(source.as_bytes(), 10)
                    .ok_or_else(|| Error::Data("invalid decimal factor exponent".into()))?;
                Ok(Self::PowerOfTen(exponent.to_string()))
            }
            _ => Ok(self),
        }
    }
}

/// An unrounded threshold, exact divisor and literal compact affixes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericCompactRule(Arc<CompactRule>);

#[derive(Debug, PartialEq, Eq)]
struct CompactRule {
    threshold: u64,
    divisor: u64,
    prefix: String,
    suffix: String,
}

impl NumericCompactRule {
    /// Construct a finite threshold, positive finite divisor and single-line affixes.
    pub fn new(
        threshold: f64,
        divisor: f64,
        prefix: impl Into<String>,
        suffix: impl Into<String>,
    ) -> Result<Self> {
        if !threshold.is_finite() || !divisor.is_finite() || divisor <= 0.0 {
            return Err(Error::Data(
                "compact thresholds must be finite and divisors positive finite".into(),
            ));
        }
        let (prefix, suffix) = (prefix.into(), suffix.into());
        validate_token(&prefix, true)?;
        validate_token(&suffix, true)?;
        Ok(Self(Arc::new(CompactRule {
            threshold: if threshold == 0.0 {
                0
            } else {
                threshold.to_bits()
            },
            divisor: divisor.to_bits(),
            prefix,
            suffix,
        })))
    }
    /// Unrounded magnitude threshold.
    pub fn threshold(&self) -> f64 {
        f64::from_bits(self.0.threshold)
    }
    /// Exact represented divisor.
    pub fn divisor(&self) -> f64 {
        f64::from_bits(self.0.divisor)
    }
    /// Literal prefix inside the format's outer prefix.
    pub fn prefix(&self) -> &str {
        &self.0.prefix
    }
    /// Literal suffix inside the format's outer suffix.
    pub fn suffix(&self) -> &str {
        &self.0.suffix
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PrefixMode {
    None,
    Si(NumericSelection<SiPrefix>, NumericPromotion),
    Compact(
        Arc<Vec<NumericCompactRule>>,
        NumericSelection<usize>,
        NumericPromotion,
    ),
    Factor(NumericFactor),
}

/// The sole selected SI, compact or exact factor policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericPrefixes(pub(super) Arc<PrefixMode>);

impl NumericPrefixes {
    pub(super) fn observe(&self, round: &mut crate::layout::kernel::FeedbackRound) -> Result<()> {
        match self.0.as_ref() {
            PrefixMode::None => round.observe_count(0),
            PrefixMode::Si(selection, promotion) => {
                round.observe_count(1)?;
                observe_selection(
                    *selection,
                    |prefix| usize::from(prefix.0.unsigned_abs()) * 2 + usize::from(prefix.0 < 0),
                    round,
                )?;
                round.observe_count(*promotion as usize)
            }
            PrefixMode::Compact(rules, selection, promotion) => {
                round.observe_count(2)?;
                observe_selection(*selection, |index| index, round)?;
                round.observe_count(*promotion as usize)?;
                round.observe_count(rules.len())?;
                for rule in rules.iter() {
                    round.observe_number(rule.threshold())?;
                    round.observe_number(rule.divisor())?;
                    round.observe_string(rule.prefix())?;
                    round.observe_string(rule.suffix())?;
                }
                Ok(())
            }
            PrefixMode::Factor(factor) => {
                round.observe_count(3)?;
                factor.observe(round)
            }
        }
    }
    /// No scale selection.
    pub fn none() -> Self {
        Self(Arc::new(PrefixMode::None))
    }
    /// Standard SI selection, saturating automatic engineering prefixes at -30/30.
    pub fn si(selection: NumericSelection<SiPrefix>, promotion: NumericPromotion) -> Result<Self> {
        fixed_promotion(matches!(selection, NumericSelection::Fixed(_)), promotion)?;
        Ok(Self(Arc::new(PrefixMode::Si(selection, promotion))))
    }
    /// The scales 1.4.0 short-scale K/M/B/T rules, normalized to custom rules.
    pub fn decimal_compact(
        selection: NumericSelection<usize>,
        promotion: NumericPromotion,
    ) -> Result<Self> {
        let rules = crate::data::source::collect_source(
            [(1e3, "K"), (1e6, "M"), (1e9, "B"), (1e12, "T")]
                .into_iter()
                .map(|(threshold, suffix)| {
                    NumericCompactRule::new(threshold, threshold, "", suffix)
                }),
        )?;
        Self::compact(rules, selection, promotion)
    }
    /// Strictly increasing thresholds with one explicit selection/promotion policy.
    pub fn compact(
        rules: Vec<NumericCompactRule>,
        selection: NumericSelection<usize>,
        promotion: NumericPromotion,
    ) -> Result<Self> {
        fixed_promotion(matches!(selection, NumericSelection::Fixed(_)), promotion)?;
        if rules
            .windows(2)
            .any(|pair| pair[0].threshold() >= pair[1].threshold())
        {
            return Err(Error::Data(
                "compact thresholds must be strictly increasing".into(),
            ));
        }
        if let NumericSelection::Fixed(index) = selection {
            if index >= rules.len() {
                return Err(Error::Data(
                    "fixed compact rule index is out of range".into(),
                ));
            }
        }
        if rules.is_empty() {
            Ok(Self::none())
        } else {
            Ok(Self(Arc::new(PrefixMode::Compact(
                Arc::new(rules),
                selection,
                promotion,
            ))))
        }
    }
    /// Choose one exact or automatic common divisor, replacing SI/compact selection.
    pub fn factor(value: NumericFactor) -> Result<Self> {
        Ok(Self(Arc::new(PrefixMode::Factor(value.normalize()?))))
    }
}

fn observe_selection<T>(
    selection: NumericSelection<T>,
    index: impl FnOnce(T) -> usize,
    round: &mut crate::layout::kernel::FeedbackRound,
) -> Result<()> {
    match selection {
        NumericSelection::PerValue => round.observe_count(0),
        NumericSelection::WholeSet => round.observe_count(1),
        NumericSelection::Fixed(value) => {
            round.observe_count(2)?;
            round.observe_count(index(value))
        }
    }
}

fn fixed_promotion(fixed: bool, promotion: NumericPromotion) -> Result<()> {
    if fixed && promotion != NumericPromotion::KeepThreshold {
        Err(Error::Data(
            "fixed prefix selection requires KeepThreshold".into(),
        ))
    } else {
        Ok(())
    }
}
