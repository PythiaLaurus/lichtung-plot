//! Retained rational geometric operands, with symbolic indices and factors.

use super::decimal::{ExactDecimal, ResolvedFormat};
use super::logarithm::{self, LogBounds};
use crate::{Error, Result};

pub(crate) mod generation;
#[cfg(test)]
mod tests;
use num_bigint::{BigInt, BigUint, Sign};
use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LatticeOperand {
    pub(crate) anchor_bits: u64,
    pub(crate) ratio_bits: u64,
    pub(crate) index: BigInt,
    pub(crate) index_text: String,
}

impl LatticeOperand {
    pub(crate) fn new(anchor: f64, ratio: f64, index: BigInt) -> Result<Self> {
        if !anchor.is_finite()
            || anchor <= 0.0
            || !ratio.is_finite()
            || ratio <= 0.0
            || ratio.to_bits() == 1.0_f64.to_bits()
        {
            return Err(Error::Data("multiplicative ticks require a finite positive anchor and positive ratio other than one".into()));
        }
        let mut index_text = String::new();
        crate::data::source::reserve_source_string(
            &mut index_text,
            usize::try_from(index.bits().div_ceil(3))
                .map_err(|_| Error::Render("multiplicative index text length overflow".into()))?
                .checked_add(2)
                .ok_or_else(|| Error::Render("multiplicative index text length overflow".into()))?,
        )?;
        write!(index_text, "{index}")
            .map_err(|_| Error::Render("multiplicative index formatting failed".into()))?;
        Ok(Self {
            anchor_bits: anchor.to_bits(),
            ratio_bits: ratio.to_bits(),
            index,
            index_text,
        })
    }

    pub(crate) fn logs(&self, format: &ResolvedFormat<'_>, precision: usize) -> Result<LogBounds> {
        let anchor = logarithm::represented(f64::from_bits(self.anchor_bits), precision)?;
        let ratio = logarithm::represented(f64::from_bits(self.ratio_bits), precision)?;
        let factor = scaled_factor(format)?;
        let effective = logarithm::rational(&factor.coefficient, &factor.denominator, precision)?;
        let shift = &factor.exponent << precision;
        let (ratio_low, ratio_high) = if self.index.sign() == Sign::Minus {
            (ratio.upper, ratio.lower)
        } else {
            (ratio.lower, ratio.upper)
        };
        Ok(LogBounds {
            lower: anchor.lower + &self.index * ratio_low + effective.lower + &shift,
            upper: anchor.upper + &self.index * ratio_high + effective.upper + shift,
        })
    }

    pub(crate) fn exact_decade(&self, format: &ResolvedFormat<'_>) -> Result<Option<BigInt>> {
        let product = Product::new(self, format)?;
        if product.twos != product.fives
            || !product.kernel_equals(&BigUint::from(1_u8), &BigUint::from(1_u8))
        {
            return Ok(None);
        }
        Ok(Some(product.twos))
    }

    pub(in crate::numeric) fn equals(
        &self,
        format: &ResolvedFormat<'_>,
        threshold: &ExactDecimal,
    ) -> Result<bool> {
        let factor = scaled_factor(format)?;
        if factor.coefficient.bits() == 0 {
            return Ok(threshold.coefficient.bits() == 0);
        }
        if threshold.coefficient.bits() == 0 {
            return Ok(false);
        }
        let product = Product::new(self, format)?;
        let other = Component::decimal(threshold)?;
        Ok(product.twos == other.twos
            && product.fives == other.fives
            && product.kernel_equals(&other.numerator, &other.denominator))
    }

    pub(crate) fn coordinate(&self) -> Result<f64> {
        let neutral = ResolvedFormat::default();
        if let Some(decade) = self.exact_decade(&neutral)? {
            return super::binary::signed_ratio(decade, BigUint::from(1_u8));
        }
        let mut precision = 128_usize;
        loop {
            let bounds = self.logs(&neutral, precision)?;
            let unit = BigUint::from(1_u8) << precision;
            let lower = super::binary::signed_ratio(bounds.lower, unit.clone())?;
            let upper = super::binary::signed_ratio(bounds.upper, unit)?;
            if lower.to_bits() == upper.to_bits() {
                return Ok(lower);
            }
            precision = refine(precision)?;
        }
    }
}

pub(in crate::numeric) fn scaled_factor(format: &ResolvedFormat<'_>) -> Result<ExactDecimal> {
    let mut factor = ExactDecimal::effective_factor(format)?;
    factor.exponent += format.decimal_scale;
    factor.decade += format.decimal_scale;
    Ok(factor)
}

pub(crate) fn refine(precision: usize) -> Result<usize> {
    precision
        .checked_mul(2)
        .ok_or_else(|| Error::Render("multiplicative certification precision overflow".into()))
}

struct Component {
    numerator: BigUint,
    denominator: BigUint,
    twos: BigInt,
    fives: BigInt,
}
impl Component {
    fn decimal(value: &ExactDecimal) -> Result<Self> {
        let (numerator, top_twos, top_fives) = strip(value.coefficient.clone())?;
        let (denominator, bottom_twos, bottom_fives) = strip(value.denominator.clone())?;
        Ok(Self {
            numerator,
            denominator,
            twos: &value.exponent + top_twos - bottom_twos,
            fives: &value.exponent + top_fives - bottom_fives,
        })
    }
    fn represented(value: f64) -> Result<Self> {
        Self::decimal(&ExactDecimal::decode(value, 0)?)
    }
}

struct Product {
    numerator: BigUint,
    denominator: BigUint,
    kernel: BigUint,
    index: BigInt,
    twos: BigInt,
    fives: BigInt,
}
impl Product {
    fn new(value: &LatticeOperand, format: &ResolvedFormat<'_>) -> Result<Self> {
        let anchor = Component::represented(f64::from_bits(value.anchor_bits))?;
        let ratio = Component::represented(f64::from_bits(value.ratio_bits))?;
        let factor = Component::decimal(&scaled_factor(format)?)?;
        if ratio.denominator != BigUint::from(1_u8) {
            return Err(Error::Render(
                "represented geometric ratio lost its dyadic denominator".into(),
            ));
        }
        Ok(Self {
            numerator: anchor.numerator * factor.numerator,
            denominator: anchor.denominator * factor.denominator,
            kernel: ratio.numerator,
            index: value.index.clone(),
            twos: anchor.twos + &value.index * ratio.twos + factor.twos,
            fives: anchor.fives + &value.index * ratio.fives + factor.fives,
        })
    }
    fn kernel_equals(&self, numerator: &BigUint, denominator: &BigUint) -> bool {
        let left = &self.numerator * denominator;
        let right = &self.denominator * numerator;
        let (left, right) = if self.index.sign() == Sign::Minus {
            (right, left)
        } else {
            (left, right)
        };
        if left.bits() == 0 || &right % &left != BigUint::ZERO {
            return false;
        }
        let target = right / left;
        if self.kernel == BigUint::from(1_u8) {
            return target == BigUint::from(1_u8);
        }
        if self.index.magnitude() > &BigUint::from(target.bits()) {
            return false;
        }
        let mut exponent = self.index.magnitude().clone();
        let mut power = self.kernel.clone();
        let mut product = BigUint::from(1_u8);
        while exponent.bits() != 0 {
            if exponent.bit(0) {
                if product > &target / &power {
                    return false;
                }
                product *= &power;
            }
            exponent >>= 1_usize;
            if exponent.bits() == 0 {
                break;
            }
            if power > &target / &power {
                return false;
            }
            power = &power * &power;
        }
        product == target
    }
}

fn strip(mut value: BigUint) -> Result<(BigUint, BigInt, BigInt)> {
    if value.bits() == 0 {
        return Err(Error::Render(
            "zero rational has no multiplicative factorization".into(),
        ));
    }
    let twos = value.trailing_zeros().unwrap_or(0);
    value >>= usize::try_from(twos)
        .map_err(|_| Error::Render("rational factorization shift overflow".into()))?;
    let mut fives = BigInt::ZERO;
    while &value % 5_u8 == BigUint::ZERO {
        value /= 5_u8;
        fives += 1;
    }
    Ok((value, BigInt::from(twos), fives))
}
