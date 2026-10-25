//! Certified geometric index bounds in the actual projection representation.

use super::{refine, LatticeOperand};
use crate::numeric::{
    binary,
    decimal::{ExactDecimal, ResolvedFormat},
    logarithm::{self, LogBounds},
};
use crate::{Error, Result};
use num_bigint::{BigInt, BigUint, Sign};

pub(crate) struct LatticeRange {
    pub(crate) first: BigInt,
    pub(crate) last: BigInt,
    anchor: f64,
    ratio: f64,
    direction: i8,
}

impl LatticeRange {
    pub(crate) fn new(range: (f64, f64), log: bool, anchor: f64, ratio: f64) -> Result<Self> {
        let _ = LatticeOperand::new(anchor, ratio, BigInt::ZERO)?;
        if !range.0.is_finite() || !range.1.is_finite() || range.0 >= range.1 {
            return Err(Error::Data(
                "multiplicative domain must be finite and ascending".into(),
            ));
        }
        let direction = if ratio > 1.0 { 1 } else { -1 };
        if !log && range.1 <= 0.0 {
            return Ok(Self {
                first: BigInt::from(1_u8),
                last: BigInt::ZERO,
                anchor,
                ratio,
                direction,
            });
        }
        if !log && range.0 <= 0.0 {
            return Err(Error::Render(
                "multiplicative view contains an unbounded positive lattice near zero".into(),
            ));
        }
        let lower = Boundary::new(range.0, true, log)?;
        let upper = Boundary::new(range.1, false, log)?;
        let (low, low_exact) = quotient(&lower, anchor, ratio, direction)?;
        let (high, high_exact) = quotient(&upper, anchor, ratio, direction)?;
        let first = if low_exact && lower.inclusive {
            low
        } else {
            low + 1_u8
        };
        let last = if high_exact && !upper.inclusive {
            high - 1_u8
        } else {
            high
        };
        Ok(Self {
            first,
            last,
            anchor,
            ratio,
            direction,
        })
    }
    pub(crate) fn count(&self) -> BigInt {
        if self.last < self.first {
            BigInt::ZERO
        } else {
            &self.last - &self.first + 1_u8
        }
    }
    pub(crate) fn operand(&self, index: &BigInt) -> Result<LatticeOperand> {
        LatticeOperand::new(self.anchor, self.ratio, index * self.direction)
    }
}

enum BoundaryValue {
    Linear(ExactDecimal),
    Scale(BigInt),
}
struct Boundary {
    value: BoundaryValue,
    inclusive: bool,
}
impl Boundary {
    fn new(value: f64, lower: bool, log: bool) -> Result<Self> {
        let center = binary::units(value)?;
        let neighbor = if lower {
            if value.next_down().is_finite() {
                binary::units(value.next_down())?
            } else {
                &center - (binary::units(value.next_up())? - &center)
            }
        } else if value.next_up().is_finite() {
            binary::units(value.next_up())?
        } else {
            &center + (&center - binary::units(value.next_down())?)
        };
        let numerator = center + neighbor;
        let boundary = if log {
            BoundaryValue::Scale(numerator)
        } else {
            if numerator.sign() != Sign::Plus {
                return Err(Error::Render(
                    "geometric lower boundary is not positive".into(),
                ));
            }
            BoundaryValue::Linear(ExactDecimal::ratio(
                false,
                numerator.into_parts().1,
                BigUint::from(1_u8) << 1075_usize,
                BigInt::ZERO,
            )?)
        };
        Ok(Self {
            value: boundary,
            inclusive: value.to_bits() & 1 == 0,
        })
    }
    fn logs(&self, precision: usize) -> Result<LogBounds> {
        match &self.value {
            BoundaryValue::Linear(value) => {
                logarithm::rational(&value.coefficient, &value.denominator, precision)
            }
            BoundaryValue::Scale(numerator) => {
                let scaled = numerator << precision;
                let unit = BigInt::from(1_u8) << 1075_usize;
                Ok(LogBounds {
                    lower: logarithm::floor(&scaled, &unit),
                    upper: logarithm::ceil(&scaled, &unit),
                })
            }
        }
    }
    fn equals(&self, value: &LatticeOperand) -> Result<bool> {
        let neutral = ResolvedFormat::default();
        match &self.value {
            BoundaryValue::Linear(boundary) => value.equals(&neutral, boundary),
            BoundaryValue::Scale(numerator) => Ok(value
                .exact_decade(&neutral)?
                .is_some_and(|decade| (decade << 1075_usize) == *numerator)),
        }
    }
}

fn quotient(boundary: &Boundary, anchor: f64, ratio: f64, direction: i8) -> Result<(BigInt, bool)> {
    let mut precision = 128_usize;
    loop {
        let limits = boundary.logs(precision)?;
        let anchor_bounds = logarithm::represented(anchor, precision)?;
        let ratio_bounds = logarithm::represented(ratio, precision)?;
        let (ratio_low, ratio_high) = if direction < 0 {
            (-ratio_bounds.upper, -ratio_bounds.lower)
        } else {
            (ratio_bounds.lower, ratio_bounds.upper)
        };
        if ratio_low <= BigInt::ZERO {
            precision = refine(precision)?;
            continue;
        }
        let lower = limits.lower - anchor_bounds.upper;
        let upper = limits.upper - anchor_bounds.lower;
        let low = logarithm::floor(
            &lower,
            if lower.sign() == Sign::Minus {
                &ratio_low
            } else {
                &ratio_high
            },
        );
        let high = logarithm::floor(
            &upper,
            if upper.sign() == Sign::Minus {
                &ratio_high
            } else {
                &ratio_low
            },
        );
        if low == high {
            let operand = LatticeOperand::new(anchor, ratio, &low * direction)?;
            return Ok((low, boundary.equals(&operand)?));
        }
        if high == &low + 1_u8 {
            let operand = LatticeOperand::new(anchor, ratio, &high * direction)?;
            if boundary.equals(&operand)? {
                return Ok((high, true));
            }
        }
        precision = refine(precision)?;
    }
}
