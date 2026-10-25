//! Rational geometric operands share decimal quantization with certified powers.
//! Exact factorized equality resolves the rational ties that powers cannot have.

use super::{certified::PowerFormatter, Log10Operand, PowerPlan};
use crate::numeric::decimal::{
    round_integer, tiny_coefficient, ExactDecimal, NumericRounding, ResolvedFormat, RoundedDecimal,
};
use crate::numeric::lattice::{refine, scaled_factor, LatticeOperand};
use crate::numeric::logarithm;
use crate::{Error, Result};
use num_bigint::{BigInt, BigUint, Sign};
use std::cmp::Ordering;

impl PowerFormatter {
    pub(crate) fn lattice_binary(&mut self, value: &LatticeOperand) -> Result<f64> {
        let format = ResolvedFormat::default();
        let decade = Self::lattice_decade(value, &format)?
            .ok_or_else(|| Error::Render("positive lattice lost its decimal decade".into()))?;
        let decade = i32::try_from(decade).map_err(|_| {
            Error::Render("geometric position exceeds finite binary64 range".into())
        })?;
        if !(-324..=308).contains(&decade) {
            return Err(Error::Render(
                "geometric position exceeds finite binary64 range".into(),
            ));
        }
        let decimal_power = BigUint::from(10_u8).pow(decade.unsigned_abs());
        if value.exact_decade(&format)?.is_some() {
            return if decade < 0 {
                crate::numeric::binary::rounded_ratio(BigUint::from(1_u8), decimal_power)
            } else {
                crate::numeric::binary::rounded_ratio(decimal_power, BigUint::from(1_u8))
            };
        }
        let mut precision = 128_usize;
        loop {
            let bounds = value.logs(&format, precision)?;
            let unit = BigInt::from(1_u8) << precision;
            if logarithm::floor(&bounds.lower, &unit) != BigInt::from(decade)
                || logarithm::floor(&bounds.upper, &unit) != BigInt::from(decade)
            {
                precision = refine(precision)?;
                continue;
            }
            let lower_operand = Log10Operand::from_fixed(bounds.lower, precision)?;
            let upper_operand = Log10Operand::from_fixed(bounds.upper, precision)?;
            let (lower, _) = self.mantissa(&lower_operand, precision)?;
            let (_, upper) = self.mantissa(&upper_operand, precision)?;
            let unit = BigUint::from(1_u8) << precision;
            let (lower, upper, divisor) = if decade < 0 {
                (lower, upper, unit * &decimal_power)
            } else {
                (lower * &decimal_power, upper * &decimal_power, unit)
            };
            let lower = crate::numeric::binary::rounded_ratio(lower, divisor.clone())?;
            if let Ok(upper) = crate::numeric::binary::rounded_ratio(upper, divisor) {
                if lower.to_bits() == upper.to_bits() {
                    return Ok(lower);
                }
                if lower.next_up().to_bits() == upper.to_bits() {
                    let numerator = crate::numeric::binary::units(lower)?
                        + crate::numeric::binary::units(upper)?;
                    let boundary = ExactDecimal::ratio(
                        false,
                        numerator.into_parts().1,
                        BigUint::from(1_u8) << 1075_usize,
                        BigInt::ZERO,
                    )?;
                    if value.equals(&format, &boundary)? {
                        return Ok(if lower.to_bits() & 1 == 0 {
                            lower
                        } else {
                            upper
                        });
                    }
                }
            }
            precision = refine(precision)?;
        }
    }

    pub(in crate::numeric) fn lattice_decade(
        value: &LatticeOperand,
        format: &ResolvedFormat<'_>,
    ) -> Result<Option<BigInt>> {
        if scaled_factor(format)?.coefficient.bits() == 0 {
            return Ok(None);
        }
        if let Some(decade) = value.exact_decade(format)? {
            return Ok(Some(decade));
        }
        let mut precision = 128_usize;
        loop {
            let bounds = value.logs(format, precision)?;
            let unit = BigInt::from(1_u8) << precision;
            let lower = logarithm::floor(&bounds.lower, &unit);
            let upper = logarithm::floor(&bounds.upper, &unit);
            if lower == upper {
                return Ok(Some(lower));
            }
            precision = refine(precision)?;
        }
    }

    pub(in crate::numeric) fn compare_lattice(
        value: &LatticeOperand,
        format: &ResolvedFormat<'_>,
        threshold: &ExactDecimal,
    ) -> Result<Ordering> {
        if value.equals(format, threshold)? {
            return Ok(Ordering::Equal);
        }
        if scaled_factor(format)?.coefficient.bits() == 0 {
            return Ok(Ordering::Less);
        }
        if threshold.coefficient.bits() == 0 {
            return Ok(Ordering::Greater);
        }
        let mut precision = 128_usize;
        loop {
            let value = value.logs(format, precision)?;
            let mut other =
                logarithm::rational(&threshold.coefficient, &threshold.denominator, precision)?;
            let shift = &threshold.exponent << precision;
            other.lower += &shift;
            other.upper += shift;
            if value.upper < other.lower {
                return Ok(Ordering::Less);
            }
            if value.lower > other.upper {
                return Ok(Ordering::Greater);
            }
            precision = refine(precision)?;
        }
    }

    pub(in crate::numeric) fn format_lattice(
        &mut self,
        value: &LatticeOperand,
        format: &ResolvedFormat<'_>,
    ) -> Result<String> {
        format.validate()?;
        let factor = scaled_factor(format)?;
        let negative = factor.negative;
        let neutral = ResolvedFormat {
            decimal_scale: 0,
            multiplier: if negative { -1.0 } else { 1.0 },
            rate: crate::NumericRate::Value,
            divisor: None,
            ..*format
        };
        if factor.coefficient.bits() == 0 {
            return crate::numeric::decimal::format_finite(0.0, &neutral);
        }
        if let Some(decade) = value.exact_decade(format)? {
            let operand = Log10Operand::from_fixed(decade, 0)?;
            return Ok(self.format(&operand, &neutral)?.text);
        }
        let decade = Self::lattice_decade(value, format)?
            .ok_or_else(|| Error::Render("nonzero lattice lost its decade".into()))?;
        let plan = PowerPlan::new(&decade, &decade, 0, false, negative, format)?;
        if plan.tiny {
            return RoundedDecimal {
                negative,
                coefficient: tiny_coefficient(negative, format.rounding),
                exponent: plan.quantum,
            }
            .emit_reserved(format, plan.output);
        }
        let power = u32::try_from(plan.multiplier.magnitude())
            .map_err(|_| Error::Render("lattice decimal quantum scratch overflow".into()))?;
        let decimal_power = BigUint::from(10_u8).pow(power);
        let mut precision = 128_usize;
        let coefficient = loop {
            let bounds = value.logs(format, precision)?;
            let unit = BigInt::from(1_u8) << precision;
            if logarithm::floor(&bounds.lower, &unit) != decade
                || logarithm::floor(&bounds.upper, &unit) != decade
            {
                precision = refine(precision)?;
                continue;
            }
            let lower_operand = Log10Operand::from_fixed(bounds.lower, precision)?;
            let upper_operand = Log10Operand::from_fixed(bounds.upper, precision)?;
            let (lower, _) = self.mantissa(&lower_operand, precision)?;
            let (_, upper) = self.mantissa(&upper_operand, precision)?;
            let unit = BigUint::from(1_u8) << precision;
            let (lower, upper, divisor) = if plan.multiplier.sign() == Sign::Minus {
                (lower, upper, unit * &decimal_power)
            } else {
                (lower * &decimal_power, upper * &decimal_power, unit)
            };
            let lower = round_integer(&lower, &divisor, negative, format.rounding);
            let upper = round_integer(&upper, &divisor, negative, format.rounding);
            if lower == upper {
                break lower;
            }
            if upper == &lower + 1_u8 {
                let (numerator, denominator) =
                    rounding_boundary(&lower, &upper, negative, format.rounding);
                let boundary = ExactDecimal::ratio(
                    false,
                    numerator.clone(),
                    denominator.clone(),
                    plan.quantum.clone(),
                )?;
                if value.equals(format, &boundary)? {
                    break round_integer(&numerator, &denominator, negative, format.rounding);
                }
            }
            precision = refine(precision)?;
        };
        RoundedDecimal {
            negative,
            coefficient,
            exponent: plan.quantum,
        }
        .emit_reserved(format, plan.output)
    }
}

fn rounding_boundary(
    lower: &BigUint,
    upper: &BigUint,
    negative: bool,
    rounding: NumericRounding,
) -> (BigUint, BigUint) {
    match rounding {
        NumericRounding::NearestEven
        | NumericRounding::NearestAway
        | NumericRounding::NearestTowardZero => (lower * 2_u8 + 1_u8, BigUint::from(2_u8)),
        NumericRounding::AwayFromZero => (lower.clone(), BigUint::from(1_u8)),
        NumericRounding::Floor if negative => (lower.clone(), BigUint::from(1_u8)),
        NumericRounding::Ceiling if !negative => (lower.clone(), BigUint::from(1_u8)),
        _ => (upper.clone(), BigUint::from(1_u8)),
    }
}
