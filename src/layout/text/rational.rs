//! Exact signed metrics, with inline arithmetic for ordinary CSS sizes.

#![allow(
    clippy::many_single_char_names,
    reason = "rational arithmetic uses numerator/denominator pairs and Euclidean remainders"
)]

use super::TextPlanError;
use num_bigint::BigInt;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ExactRational {
    Small(i128, i128),
    Wide(Arc<(BigInt, BigInt)>),
}

impl ExactRational {
    pub const ZERO: Self = Self::Small(0, 1);

    pub fn new(n: i128, d: i128) -> Result<Self, TextPlanError> {
        if d <= 0 {
            return Err(TextPlanError::Arithmetic("rational denominator"));
        }
        let divisor = gcd(n.unsigned_abs(), d.unsigned_abs());
        let divisor =
            i128::try_from(divisor).map_err(|_| TextPlanError::Arithmetic("rational divisor"))?;
        Ok(Self::Small(n / divisor, d / divisor))
    }

    fn wide(mut n: BigInt, mut d: BigInt) -> Result<Self, TextPlanError> {
        if d <= BigInt::ZERO {
            return Err(TextPlanError::Arithmetic("rational denominator"));
        }
        let (mut a, mut b) = (n.magnitude().clone(), d.magnitude().clone());
        while b.bits() != 0 {
            let r = &a % &b;
            (a, b) = (b, r);
        }
        let divisor = BigInt::from(a);
        n /= &divisor;
        d /= divisor;
        if let (Ok(n), Ok(d)) = (i128::try_from(&n), i128::try_from(&d)) {
            return Ok(Self::Small(n, d));
        }
        Ok(Self::Wide(Arc::new((n, d))))
    }

    fn parts(&self) -> (BigInt, BigInt) {
        match self {
            Self::Small(n, d) => ((*n).into(), (*d).into()),
            Self::Wide(pair) => pair.as_ref().clone(),
        }
    }

    pub fn design(design: i64, size: f64, em: u16) -> Result<Self, TextPlanError> {
        if !size.is_finite() || size <= 0.0 {
            return Err(TextPlanError::InvalidFontSize);
        }
        if design == 0 {
            return Self::new(0, em.into());
        }
        let bits = size.to_bits();
        let exponent = i32::try_from((bits >> 52) & 0x7ff).expect("eleven-bit exponent");
        let mantissa = (bits & ((1_u64 << 52) - 1)) | if exponent == 0 { 0 } else { 1_u64 << 52 };
        let power = if exponent == 0 {
            -1074
        } else {
            exponent - 1023 - 52
        } + 10;
        let mut n = i128::from(design) * i128::from(mantissa);
        let mut d = i128::from(em);
        if power < 0 {
            let cancel = n.unsigned_abs().trailing_zeros().min(power.unsigned_abs());
            n >>= cancel;
            if let Some(denominator) = 1_i128
                .checked_shl(power.unsigned_abs() - cancel)
                .filter(|&v| v > 0)
                .and_then(|p| d.checked_mul(p))
            {
                return Self::new(n, denominator);
            }
            return Self::wide(n.into(), BigInt::from(d) << (power.unsigned_abs() - cancel));
        }
        let cancel = d.unsigned_abs().trailing_zeros().min(power.unsigned_abs());
        d >>= cancel;
        if let Some(numerator) = 1_i128
            .checked_shl(power.unsigned_abs() - cancel)
            .filter(|&v| v > 0)
            .and_then(|p| n.checked_mul(p))
        {
            return Self::new(numerator, d);
        }
        Self::wide(BigInt::from(n) << (power.unsigned_abs() - cancel), d.into())
    }

    pub fn negated(&self) -> Result<Self, TextPlanError> {
        if let Self::Small(n, d) = self {
            if let Some(n) = n.checked_neg() {
                return Ok(Self::Small(n, *d));
            }
        }
        let (n, d) = self.parts();
        Self::wide(-n, d)
    }

    pub fn add(&self, other: &Self, _resource: &'static str) -> Result<Self, TextPlanError> {
        if let (Self::Small(a, b), Self::Small(c, d)) = (self, other) {
            let g = i128::try_from(gcd(b.unsigned_abs(), d.unsigned_abs()))
                .expect("positive denominators");
            if let (Some(n), Some(d)) = (
                a.checked_mul(d / g)
                    .and_then(|a| c.checked_mul(b / g).and_then(|c| a.checked_add(c))),
                b.checked_mul(d / g),
            ) {
                return Self::new(n, d);
            }
        }
        let (a, b) = self.parts();
        let (c, d) = other.parts();
        Self::wide(a * &d + c * &b, b * d)
    }

    fn rounded(&self, mode: u8, resource: &'static str) -> Result<i64, TextPlanError> {
        if let Self::Small(n, d) = self {
            let q = n.div_euclid(*d);
            let r = n.rem_euclid(*d);
            let increment = if mode == 0 {
                false
            } else if mode == 1 {
                r != 0
            } else {
                r > d - r || (r == d - r && q % 2 != 0)
            };
            return q
                .checked_add(i128::from(increment))
                .and_then(|v| i64::try_from(v).ok())
                .ok_or(TextPlanError::Arithmetic(resource));
        }
        let (n, d) = self.parts();
        let mut q = &n / &d;
        let mut r = &n % &d;
        if r < BigInt::ZERO {
            q -= 1;
            r += &d;
        }
        let increment = if mode == 0 {
            false
        } else if mode == 1 {
            r != BigInt::ZERO
        } else {
            let twice = &r << 1;
            twice > d || (twice == d && &q % 2 != BigInt::ZERO)
        };
        if increment {
            q += 1;
        }
        i64::try_from(q).map_err(|_| TextPlanError::Arithmetic(resource))
    }
    pub fn floor_i64(&self, resource: &'static str) -> Result<i64, TextPlanError> {
        self.rounded(0, resource)
    }
    pub fn ceil_i64(&self, resource: &'static str) -> Result<i64, TextPlanError> {
        self.rounded(1, resource)
    }
    pub fn nearest_even_i64(&self, resource: &'static str) -> Result<i64, TextPlanError> {
        self.rounded(2, resource)
    }

    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if let (Self::Small(a, b), Self::Small(c, d)) = (self, other) {
            if let (Some(a), Some(b)) = (a.checked_mul(*d), c.checked_mul(*b)) {
                return a.cmp(&b);
            }
        }
        let (a, b) = self.parts();
        let (c, d) = other.parts();
        (a * d).cmp(&(c * b))
    }
    pub fn min(self, other: Self) -> Self {
        if self.cmp(&other).is_le() {
            self
        } else {
            other
        }
    }
    pub fn max(self, other: Self) -> Self {
        if self.cmp(&other).is_ge() {
            self
        } else {
            other
        }
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        (a, b) = (b, r);
    }
    a.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractional_design_units_retain_dyadic_values_until_real_spatial_boundaries() {
        // 801 * 10.25 * 1024 / 2048 = 4105.125 quanta.
        let metric = ExactRational::design(801, 10.25, 2048).unwrap();
        assert_eq!(metric, ExactRational::new(32841, 8).unwrap());
        assert_eq!(metric.floor_i64("test").unwrap(), 4105);
        assert_eq!(metric.ceil_i64("test").unwrap(), 4106);
        assert_eq!(metric.nearest_even_i64("test").unwrap(), 4105);
        for sign in [-1, 1] {
            let tiny = ExactRational::design(sign, f64::from_bits(1), 2048).unwrap();
            assert_eq!(
                tiny.floor_i64("test").unwrap(),
                if sign < 0 { -1 } else { 0 }
            );
            assert_eq!(tiny.ceil_i64("test").unwrap(), i64::from(sign > 0));
            assert_eq!(tiny.nearest_even_i64("test").unwrap(), 0);
            let one = ExactRational::new(1, 1).unwrap();
            assert_eq!(
                one.add(&tiny, "test").unwrap().ceil_i64("test").unwrap(),
                if sign > 0 { 2 } else { 1 }
            );
            assert_eq!(
                tiny.add(&tiny.negated().unwrap(), "test").unwrap(),
                ExactRational::ZERO
            );
        }
        assert!(ExactRational::design(1, f64::MAX, 2048)
            .unwrap()
            .ceil_i64("test")
            .is_err());
    }
}
