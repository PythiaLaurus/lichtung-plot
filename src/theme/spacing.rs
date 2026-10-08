//! Immutable reduced rational identities and exact local lengths.

use std::cmp::Ordering;
use std::sync::Arc;

use num_bigint::BigUint;

use crate::layout::kernel::{LayoutScalar, SpatialRounding};
use crate::{Error, Result};

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RelativeSpacing(Arc<SpacingPayload>);

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

    pub(crate) fn new(value: f64) -> Result<Self> {
        validate(value)?;

        let rational = dyadic(value);
        Ok(Self(Arc::new(SpacingPayload {
            rational,
            approximate: if value == 0.0 { 0.0 } else { value },
        })))
    }

    pub(crate) fn ratio(numerator: u32, denominator: u32) -> Result<Self> {
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

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "inspection is reserved for the named nonspatial consumers and public cutover"
        )
    )]
    pub(crate) fn as_f64(&self) -> f64 {
        self.0.approximate
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.0.rational.numerator.bits() == 0
    }

    #[cfg(test)]
    pub(crate) fn shares_payload_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub(crate) fn product(&self, metric: LocalMetric) -> ExactLength {
        let rational = &self.0.rational;

        ExactLength(Rational {
            numerator: &rational.numerator * metric.0,
            denominator: rational.denominator.clone(),
        })
    }
}

/// A declared positive role em/radius or canonical local span, never a root size.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LocalMetric(u64);

impl LocalMetric {
    pub(crate) fn css_pixels(value: u32) -> Result<Self> {
        Self::scalar(LayoutScalar::exact_css_pixels(value)?)
    }

    pub(crate) fn scalar(value: LayoutScalar) -> Result<Self> {
        match u64::try_from(value.quanta()) {
            Ok(value) if value > 0 => Ok(Self(value)),
            _ => Err(invalid("local metric must be positive")),
        }
    }
}

/// Unrounded nonnegative quanta. Aggregate before selecting a spatial boundary.
#[derive(Debug)]
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
            .product(LocalMetric::css_pixels(pixels).unwrap())
            .round(rounding)
            .unwrap()
            .quanta()
    }

    #[test]
    fn constructors_have_one_reduced_exact_identity() {
        let ratio = RelativeSpacing::ratio(6, 14).unwrap();
        assert_eq!(ratio, RelativeSpacing::ratio(3, 7).unwrap());
        assert_eq!(ratio.0.rational.numerator, BigUint::from(3_u8));
        assert_eq!(ratio.0.rational.denominator, BigUint::from(7_u8));
        assert_ne!(ratio, RelativeSpacing::new(6.0 / 14.0).unwrap());
        assert_ne!(
            RelativeSpacing::new(0.6).unwrap(),
            RelativeSpacing::ratio(3, 5).unwrap()
        );
        assert_eq!(
            RelativeSpacing::ratio(0, 17).unwrap(),
            RelativeSpacing::new(-0.0).unwrap()
        );
        assert_eq!(RelativeSpacing::new(-0.0).unwrap().as_f64().to_bits(), 0);
        assert_eq!(resolved(&ratio, 14, SpatialRounding::Ceil), 6144);
        assert_eq!(
            resolved(
                &RelativeSpacing::ratio(5, 14).unwrap(),
                14,
                SpatialRounding::Ceil
            ),
            5120
        );
        // Binary64 5/14 lies above the mathematical rational.
        assert_eq!(
            resolved(
                &RelativeSpacing::new(5.0 / 14.0).unwrap(),
                14,
                SpatialRounding::Ceil
            ),
            5121
        );
    }

    #[test]
    fn validation_and_local_metric_bounds_are_checked() {
        for value in [-1.0, f64::NEG_INFINITY, f64::INFINITY, f64::NAN] {
            assert!(RelativeSpacing::new(value).is_err());
            assert!(RelativeSpacing::from_pixels(value, 11).is_err());
        }
        assert!(RelativeSpacing::ratio(1, 0).is_err());
        assert!(RelativeSpacing::from_pixels(1.0, 0).is_err());
        assert!(LocalMetric::css_pixels(0).is_err());
        assert!(LocalMetric::scalar(LayoutScalar::from_quanta(-1)).is_err());
        assert_eq!(
            LocalMetric::css_pixels(u32::MAX).unwrap().0,
            4_398_046_510_080
        );
        assert_eq!(
            LocalMetric::scalar(LayoutScalar::from_quanta(i64::MAX))
                .unwrap()
                .0,
            i64::MAX as u64
        );
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
            .product(LocalMetric::css_pixels(1).unwrap());
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
    fn pixel_factories_preserve_all_pinned_source_bits_through_exact_division() {
        for (pixels, bits, em, nearest, hard) in [
            (8.8, 0x4021_9999_9999_999a, 11, 9011, 9012),
            (17.6, 0x4031_9999_9999_999a, 11, 18022, 18023),
            (3.3, 0x400a_6666_6666_6666, 11, 3379, 3380),
            (13.2, 0x402a_6666_6666_6666, 11, 13517, 13517),
            (7.8, 0x401f_3333_3333_3333, 12, 7987, 7988),
        ] {
            assert_eq!(f64::to_bits(pixels), bits);
            let value = RelativeSpacing::from_pixels(pixels, em).unwrap();

            let restored = value.product(LocalMetric::css_pixels(em).unwrap());
            let source = RelativeSpacing::new(pixels)
                .unwrap()
                .product(LocalMetric::css_pixels(1).unwrap());
            assert_eq!(restored.compare(&source), Ordering::Equal);
            assert_eq!(
                restored
                    .round(SpatialRounding::NearestEven)
                    .unwrap()
                    .quanta(),
                nearest
            );
            assert_eq!(
                restored.round(SpatialRounding::Ceil).unwrap().quanta(),
                hard
            );
            assert_eq!(value.as_f64().to_bits(), (pixels / f64::from(em)).to_bits());
        }
        assert_ne!(
            RelativeSpacing::from_pixels(8.8, 11).unwrap(),
            RelativeSpacing::new(8.8 / 11.0).unwrap()
        );
        assert_ne!(
            RelativeSpacing::from_pixels(8.8, 11).unwrap(),
            RelativeSpacing::ratio(4, 5).unwrap()
        );
        assert_eq!(
            RelativeSpacing::from_pixels(0.0, 11).unwrap(),
            RelativeSpacing::ratio(0, 1).unwrap()
        );
        assert_eq!(
            RelativeSpacing::from_pixels(-0.0, 11)
                .unwrap()
                .as_f64()
                .to_bits(),
            0
        );
        for pixels in [2, 11, 40, 160] {
            assert_eq!(
                RelativeSpacing::from_pixels(f64::from(pixels), 11).unwrap(),
                RelativeSpacing::ratio(pixels, 11).unwrap()
            );
            assert_eq!(
                resolved(
                    &RelativeSpacing::from_pixels(f64::from(pixels), 11).unwrap(),
                    11,
                    SpatialRounding::Ceil
                ),
                i64::from(pixels) * 1024
            );
        }
        assert_eq!(
            RelativeSpacing::from_pixels(f64::MAX, 1)
                .unwrap()
                .as_f64()
                .to_bits(),
            f64::MAX.to_bits()
        );
    }

    #[test]
    fn exact_aggregate_retains_the_edge_inset_quantum_and_repeated_gap_sum() {
        let em = LocalMetric::css_pixels(11).unwrap();
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
            .product(LocalMetric::css_pixels(1).unwrap());
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

    #[test]
    fn inspection_rounds_rational_ties_to_even_without_integer_overflow() {
        assert_eq!(
            RelativeSpacing::ratio(u32::MAX, u32::MAX)
                .unwrap()
                .as_f64()
                .to_bits(),
            1.0_f64.to_bits()
        );
        assert_eq!(
            RelativeSpacing::ratio(5, 14).unwrap().as_f64().to_bits(),
            0x3fd6_db6d_b6db_6db7
        );
        assert_eq!(
            RelativeSpacing::ratio(u32::MAX, 1)
                .unwrap()
                .as_f64()
                .to_bits(),
            4_294_967_295.0_f64.to_bits()
        );
        // P can supply actual binary64 halfway values through exact division.
        assert_eq!(
            RelativeSpacing::from_pixels(f64::from_bits(3), 2)
                .unwrap()
                .as_f64()
                .to_bits(),
            2
        );
        assert_eq!(
            RelativeSpacing::from_pixels(f64::from_bits(5), 2)
                .unwrap()
                .as_f64()
                .to_bits(),
            2
        );
        assert_eq!(
            RelativeSpacing::from_pixels(f64::from_bits(1), u32::MAX)
                .unwrap()
                .as_f64()
                .to_bits(),
            0
        );
    }

    #[test]
    fn spacing_clone_shares_exact_retained_integers() {
        let value = RelativeSpacing::new(f64::from_bits(1)).unwrap();

        let copy = value.clone();
        assert!(Arc::ptr_eq(&value.0, &copy.0));
        assert_eq!(value, copy);
    }
}
