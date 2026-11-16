//! Immutable reduced rational identities and exact local lengths.

use std::cmp::Ordering;
use std::sync::Arc;

use num_bigint::BigUint;

use crate::layout::kernel::{LayoutScalar, SpatialRounding};
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Rational {
    numerator: BigUint,
    denominator: BigUint,
}

#[derive(Debug)]
struct SpacingPayload {
    rational: Rational,
    // Immutable nearest-even inspection only. Never an identity/spatial operand.
    approximate: f64,
}

impl PartialEq for SpacingPayload {
    fn eq(&self, other: &Self) -> bool {
        self.rational == other.rational
    }
}

impl Eq for SpacingPayload {}

/// Exact nonnegative dimensionless spacing, resolved against its consuming role's metric.
///
/// `new` retains the represented binary64 rational; `ratio` retains the reduced
/// mathematical integer ratio. Approximate inspection is never a layout operand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelativeSpacing(Arc<SpacingPayload>);

impl RelativeSpacing {
    pub(crate) fn compare(&self, other: &Self) -> Ordering {
        if Arc::ptr_eq(&self.0, &other.0) {
            return Ordering::Equal;
        }
        let left = &self.0.rational;
        let right = &other.0.rational;

        (&left.numerator * &right.denominator).cmp(&(&right.numerator * &left.denominator))
    }
    pub(crate) fn same(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.0, &other.0) {
            return true;
        }
        for (left, right) in [
            (&self.0.rational.numerator, &other.0.rational.numerator),
            (&self.0.rational.denominator, &other.0.rational.denominator),
        ] {
            if left.iter_u64_digits().len() != right.iter_u64_digits().len() {
                return false;
            }
            let (mut left, mut right) = (left.iter_u64_digits(), right.iter_u64_digits());
            while left.len() != 0 {
                if left.next() != right.next() {
                    return false;
                }
            }
        }
        true
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<()> {
        for value in [&self.0.rational.numerator, &self.0.rational.denominator] {
            let mut digits = value.iter_u64_digits();
            round.observe_count(digits.len())?;
            while digits.len() != 0 {
                let digit = digits.next().expect("remaining normalized rational digit");
                round.observe_count((digit >> 32) as usize)?;
                round.observe_count((digit & 0xffff_ffff) as usize)?;
            }
        }
        Ok(())
    }

    /// Retain a finite nonnegative binary64 ratio, normalizing signed zero.
    pub fn new(value: f64) -> Result<Self> {
        validate(value)?;

        let rational = dyadic(value);
        Ok(Self(Arc::new(SpacingPayload {
            rational,
            approximate: if value == 0.0 { 0.0 } else { value },
        })))
    }

    /// Retain a mathematical integer ratio; denominator must be positive.
    pub fn ratio(numerator: u32, denominator: u32) -> Result<Self> {
        if denominator == 0 {
            return Err(invalid("ratio denominator must be positive"));
        }

        // Bounded Euclidean gcd on mathematical integers, before BigUint storage.
        let (mut left, mut right) = (numerator, denominator);
        while right != 0 {
            (left, right) = (right, left % right);
        }

        Ok(Self(Arc::new(SpacingPayload {
            rational: Rational {
                numerator: BigUint::from(numerator / left),
                denominator: BigUint::from(denominator / left),
            },
            // Both integer operands are exactly representable; IEEE division is
            // nearest-even inspection of this rational, including canonical zero.
            approximate: f64::from(numerator) / f64::from(denominator),
        })))
    }

    pub(crate) fn from_pixels(pixels: f64, em: u32) -> Result<Self> {
        validate(pixels)?;
        if em == 0 {
            return Err(invalid("source em must be positive"));
        }

        let source = dyadic(pixels);

        let denominator = &source.denominator * em;
        let rational = reduce(source.numerator, denominator);
        Ok(Self(Arc::new(SpacingPayload {
            rational,
            approximate: if pixels == 0.0 {
                0.0
            } else {
                pixels / f64::from(em)
            },
        })))
    }

    /// Inspect the nearest binary64 approximation without changing exact identity.
    pub fn as_f64(&self) -> f64 {
        self.0.approximate
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.0.rational.numerator.bits() == 0
    }

    pub(crate) fn product(&self, metric: LocalMetric) -> ExactLength {
        let rational = &self.0.rational;
        let metric = match metric {
            LocalMetric::CssPixels(bits) => {
                let mut value = dyadic(f64::from_bits(bits));
                value.numerator *= 1024_u16;
                value
            }
            LocalMetric::Quanta(value) => Rational {
                numerator: BigUint::from(value),
                denominator: BigUint::from(1_u8),
            },
        };

        ExactLength(Rational {
            numerator: &rational.numerator * metric.numerator,
            denominator: &rational.denominator * metric.denominator,
        })
    }
}

/// A declared positive role em/radius or canonical local span, never a root size.
#[derive(Debug, Clone, Copy)]
pub(crate) enum LocalMetric {
    CssPixels(u64),
    Quanta(u64),
}

impl LocalMetric {
    pub(crate) fn css_pixels(value: f64) -> Result<Self> {
        if !value.is_finite() || value <= 0.0 {
            return Err(invalid("local metric must be finite and positive"));
        }
        Ok(Self::CssPixels(value.to_bits()))
    }

    pub(crate) fn scalar(value: LayoutScalar) -> Result<Self> {
        match u64::try_from(value.quanta()) {
            Ok(value) if value > 0 => Ok(Self::Quanta(value)),
            _ => Err(invalid("local metric must be positive")),
        }
    }
}

/// Unrounded nonnegative quanta. Aggregate before selecting a spatial boundary.
#[derive(Debug, Clone)]
pub(crate) struct ExactLength(Rational);

// Test oracles may compare independently measured exact consequences. Runtime
// matching uses the exact compare method, not this trait.
#[cfg(test)]
impl PartialEq for ExactLength {
    fn eq(&self, other: &Self) -> bool {
        &self.0.numerator * &other.0.denominator == &other.0.numerator * &self.0.denominator
    }
}

impl ExactLength {
    /// Retain canonical integer quanta without decoding a pixel projection.
    pub(crate) fn quanta(value: LayoutScalar) -> Result<Self> {
        let value =
            u64::try_from(value.quanta()).map_err(|_| invalid("length must be nonnegative"))?;

        Ok(Self(Rational {
            numerator: BigUint::from(value),
            denominator: BigUint::from(1_u8),
        }))
    }
    /// Unrounded remainder; a negative remainder is unavailable capacity, not zero.
    pub(crate) fn subtract(&self, other: &Self) -> Option<Self> {
        let left = &self.0.numerator * &other.0.denominator;
        let right = &other.0.numerator * &self.0.denominator;
        if left < right {
            return None;
        }
        Some(Self(Rational {
            numerator: left - right,
            denominator: &self.0.denominator * &other.0.denominator,
        }))
    }
    pub(crate) fn before_pixels(&self, anchor: f64) -> Result<f64> {
        scalar_pixels(self.before(anchor)?)
    }
    pub(crate) fn after_pixels(&self, anchor: f64) -> Result<f64> {
        let value = self.before_pixels(-anchor)?;
        Ok(if value == 0.0 { 0.0 } else { -value })
    }
    pub(crate) fn divided_by(&self, value: f64) -> Result<Self> {
        if !value.is_finite() || value <= 0.0 {
            return Err(invalid("length divisor must be finite and positive"));
        }
        let divisor = dyadic(value);

        Ok(Self(Rational {
            numerator: &self.0.numerator * divisor.denominator,
            denominator: &self.0.denominator * divisor.numerator,
        }))
    }

    pub(crate) fn pixels_rounded(&self, rounding: SpatialRounding) -> Result<f64> {
        scalar_pixels(self.round(rounding)?)
    }
    /// Resolve a whole position, retaining anchor parity through half-quantum
    /// ties instead of independently rounding the offset before subtraction.
    pub(crate) fn before(&self, anchor: f64) -> Result<LayoutScalar> {
        let origin = Self::pixels(anchor.abs())?;

        let left = &origin.0.numerator * &self.0.denominator;
        let right = &self.0.numerator * &origin.0.denominator;
        let (negative, numerator) = if anchor.is_sign_negative() {
            (true, left + right)
        } else if left < right {
            (true, right - left)
        } else {
            (false, left - right)
        };
        let value = Self(Rational {
            numerator,
            denominator: &origin.0.denominator * &self.0.denominator,
        })
        .round(SpatialRounding::NearestEven)?;
        let quanta = if negative {
            value
                .quanta()
                .checked_neg()
                .ok_or_else(|| invalid("axis position overflow"))?
        } else {
            value.quanta()
        };
        Ok(LayoutScalar::from_quanta(quanta))
    }
    /// Exact nonnegative represented pixel ink/occupancy.
    pub(crate) fn pixels(value: f64) -> Result<Self> {
        validate(value)?;
        let rational = dyadic(value);

        Ok(Self(Rational {
            numerator: rational.numerator << 10_usize,
            denominator: rational.denominator,
        }))
    }

    pub(crate) fn add_pixels(&self, value: f64) -> Result<Self> {
        Ok(self.add_length(&Self::pixels(value)?))
    }
    pub(crate) fn add_length(&self, other: &Self) -> Self {
        let numerator =
            &self.0.numerator * &other.0.denominator + &other.0.numerator * &self.0.denominator;
        let denominator = &self.0.denominator * &other.0.denominator;
        Self(reduce(numerator, denominator))
    }

    pub(crate) fn times(&self, count: u32) -> Self {
        Self(Rational {
            numerator: &self.0.numerator * count,
            denominator: self.0.denominator.clone(),
        })
    }

    pub(crate) fn add_quanta(&self, value: LayoutScalar) -> Result<Self> {
        let value =
            u64::try_from(value.quanta()).map_err(|_| invalid("length must be nonnegative"))?;

        Ok(Self(Rational {
            numerator: &self.0.numerator + &self.0.denominator * value,
            denominator: self.0.denominator.clone(),
        }))
    }

    pub(crate) fn compare(&self, other: &Self) -> Ordering {
        (&self.0.numerator * &other.0.denominator).cmp(&(&other.0.numerator * &self.0.denominator))
    }

    pub(crate) fn round(&self, rounding: SpatialRounding) -> Result<LayoutScalar> {
        let quotient = &self.0.numerator / &self.0.denominator;
        let remainder = &self.0.numerator % &self.0.denominator;
        let mut digits = quotient.iter_u64_digits();
        let value = digits.next().unwrap_or(0);
        if digits.next().is_some() {
            return Err(invalid("length exceeds layout quanta"));
        }
        let increment = match rounding {
            SpatialRounding::Floor => false,
            SpatialRounding::Ceil => remainder.bits() != 0,
            SpatialRounding::NearestEven => {
                match (&remainder << 1_usize).cmp(&self.0.denominator) {
                    Ordering::Less => false,
                    Ordering::Equal => value & 1 != 0,
                    Ordering::Greater => true,
                }
            }
        };
        value
            .checked_add(u64::from(increment))
            .and_then(|value| i64::try_from(value).ok())
            .map(LayoutScalar::from_quanta)
            .ok_or_else(|| invalid("rounded length exceeds layout quanta"))
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "the checked quanta bound guarantees exact integer conversion"
)]
fn scalar_pixels(value: LayoutScalar) -> Result<f64> {
    if value.quanta().unsigned_abs() > (1_u64 << 53) {
        return Err(invalid("length cannot retain exact CSS quanta"));
    }
    Ok(value.quanta() as f64 / 1024.0)
}

fn validate(value: f64) -> Result<()> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(invalid("spacing must be finite and nonnegative"))
    }
}

fn dyadic(value: f64) -> Rational {
    let bits = value.to_bits();
    let exponent = i32::try_from((bits >> 52) & 0x7ff).expect("11-bit exponent");
    let fraction = bits & ((1_u64 << 52) - 1);
    let (mut significand, mut shift) = if exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1_u64 << 52), exponent - 1075)
    };
    if significand == 0 {
        return Rational {
            numerator: BigUint::from(0_u8),
            denominator: BigUint::from(1_u8),
        };
    }
    let zeros = significand.trailing_zeros();
    significand >>= zeros;
    shift += i32::try_from(zeros).expect("at most 53 bits");

    let numerator = BigUint::from(significand);
    let denominator = BigUint::from(1_u8);
    if shift >= 0 {
        Rational {
            numerator: numerator << shift.unsigned_abs() as usize,
            denominator,
        }
    } else {
        Rational {
            numerator,
            denominator: denominator << shift.unsigned_abs() as usize,
        }
    }
}

fn reduce(numerator: BigUint, denominator: BigUint) -> Rational {
    let (mut left, mut right) = (numerator.clone(), denominator.clone());
    while right.bits() != 0 {
        let remainder = &left % &right;
        (left, right) = (right, remainder);
    }

    Rational {
        numerator: numerator / &left,
        denominator: denominator / &left,
    }
}

fn invalid(message: &str) -> Error {
    Error::Render(format!("relative spacing: {message}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolved(value: &RelativeSpacing, pixels: u32, rounding: SpatialRounding) -> i64 {
        value
            .product(LocalMetric::css_pixels(f64::from(pixels)).unwrap())
            .round(rounding)
            .unwrap()
            .quanta()
    }

    #[test]
    fn finite_binary64_extremes_roundtrip_and_spatial_overflow_refuses() {
        for bits in [
            1,
            0x000f_ffff_ffff_ffff,
            0x0010_0000_0000_0000,
            0x3fb9_9999_9999_999a,
            0x7fef_ffff_ffff_ffff,
        ] {
            let value = RelativeSpacing::new(f64::from_bits(bits)).unwrap();
            assert_eq!(value.as_f64().to_bits(), bits);
        }
        let tiny = RelativeSpacing::new(f64::from_bits(1)).unwrap();
        assert_eq!(tiny.0.rational.numerator, BigUint::from(1_u8));
        assert_eq!(tiny.0.rational.denominator.bits(), 1075);
        assert_eq!(resolved(&tiny, u32::MAX, SpatialRounding::Floor), 0);
        assert_eq!(resolved(&tiny, u32::MAX, SpatialRounding::NearestEven), 0);
        assert_eq!(resolved(&tiny, u32::MAX, SpatialRounding::Ceil), 1);

        let large = RelativeSpacing::new(f64::MAX)
            .unwrap()
            .product(LocalMetric::css_pixels(1.0).unwrap());
        assert!(large.round(SpatialRounding::Ceil).is_err());
        let maximum = RelativeSpacing::ratio(1, 1)
            .unwrap()
            .product(LocalMetric::scalar(LayoutScalar::from_quanta(i64::MAX)).unwrap());
        assert_eq!(
            maximum
                .round(SpatialRounding::NearestEven)
                .unwrap()
                .quanta(),
            i64::MAX
        );
        assert!(maximum
            .add_quanta(LayoutScalar::from_quanta(1))
            .unwrap()
            .round(SpatialRounding::Floor)
            .is_err());
    }

    #[test]
    fn nearest_even_ties_floor_and_ceil_are_independent_boundaries() {
        for (numerator, nearest, floor, ceil) in
            [(1, 0, 0, 1), (3, 2, 1, 2), (5, 2, 2, 3), (7, 4, 3, 4)]
        {
            let value = RelativeSpacing::ratio(numerator, 2048).unwrap();
            assert_eq!(resolved(&value, 1, SpatialRounding::NearestEven), nearest);
            assert_eq!(resolved(&value, 1, SpatialRounding::Floor), floor);
            assert_eq!(resolved(&value, 1, SpatialRounding::Ceil), ceil);
        }
    }

    #[test]
    fn exact_aggregate_retains_the_edge_inset_quantum_and_repeated_gap_sum() {
        let em = LocalMetric::css_pixels(11.0).unwrap();
        let padding = RelativeSpacing::from_pixels(8.8, 11).unwrap().product(em);
        let comfort = RelativeSpacing::new(0.2).unwrap().product(em);
        let edge = padding.add_length(&comfort);
        assert_eq!(
            edge.round(SpatialRounding::NearestEven).unwrap().quanta(),
            11264
        );
        assert_eq!(edge.round(SpatialRounding::Ceil).unwrap().quanta(), 11265);
        assert_eq!(
            edge.times(2).round(SpatialRounding::Ceil).unwrap().quanta(),
            22529
        );
        let gap = RelativeSpacing::ratio(1, 3072)
            .unwrap()
            .product(LocalMetric::css_pixels(1.0).unwrap());
        assert_eq!(
            gap.times(3)
                .add_quanta(LayoutScalar::from_quanta(1024))
                .unwrap()
                .round(SpatialRounding::Ceil)
                .unwrap()
                .quanta(),
            1025
        );
        assert_eq!(gap.compare(&edge), Ordering::Less);
        assert!(gap.add_quanta(LayoutScalar::from_quanta(-1)).is_err());
    }
}
