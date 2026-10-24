//! Certified decimal rounding of fractional powers: atanh constants, a positive
//! small exponential series and four outward squares. Spec07 proves enclosure,
//! accumulated error and eventual agreement at one fixed decimal quantum.

use num_bigint::{BigInt, BigUint, Sign};

use super::{Log10Operand, PowerPlan};
use crate::numeric::decimal::{
    round_integer, tiny_coefficient, ExactDecimal, ResolvedFormat, RoundedDecimal,
};
use crate::{Error, Result};

pub(crate) struct CertifiedPower {
    #[cfg(test)]
    pub(crate) rounded: RoundedDecimal,
    pub(crate) text: String,
}

struct ConstantBounds {
    precision: usize,
    lower: BigUint,
    upper: BigUint,
}

/// Created by the operation's numeric owner; no global or font-dependent cache.
#[derive(Default)]
pub(crate) struct PowerFormatter {
    constants: Vec<ConstantBounds>,
    #[cfg(test)]
    pub(super) attempts: Vec<usize>,
    #[cfg(test)]
    pub(super) stages: Vec<(BigUint, BigUint)>,
    #[cfg(test)]
    pub(super) constant_hits: usize,
    #[cfg(test)]
    pub(super) constant_misses: usize,
}

impl PowerFormatter {
    #[cfg(test)]
    pub(crate) fn certification_work(&self) -> (&[usize], usize, usize) {
        (&self.attempts, self.constant_hits, self.constant_misses)
    }

    pub(crate) fn format(
        &mut self,
        operand: &Log10Operand,
        format: &ResolvedFormat<'_>,
    ) -> Result<CertifiedPower> {
        #[cfg(test)]
        {
            self.attempts.clear();
            self.stages.clear();
        }
        format.validate()?;
        if format.decimal_scale != operand.decimal_scale {
            return Err(Error::Render(
                "Log10 operand decimal scale changed after reduction".into(),
            ));
        }
        let factor = ExactDecimal::effective_factor(format)?;
        let negative = factor.negative;
        let zero = factor.coefficient.bits() == 0;
        let exponent = &operand.decade + &factor.exponent;
        let decade = if zero {
            BigInt::ZERO
        } else {
            self.factored_decade(operand, &factor)?
        };
        let plan = PowerPlan::new(
            &decade,
            &exponent,
            factor.coefficient.bits(),
            zero || operand.fraction.bits() == 0,
            negative,
            format,
        )?;
        let rounded = if zero || operand.fraction.bits() == 0 {
            ExactDecimal::ratio(negative, factor.coefficient, factor.denominator, exponent)?
                .round(format)?
        } else if plan.tiny {
            RoundedDecimal {
                negative,
                coefficient: tiny_coefficient(negative, format.rounding),
                exponent: plan.quantum.clone(),
            }
        } else {
            let power =
                usize::try_from(plan.multiplier.magnitude()).map_err(|_| arithmetic_error())?;
            let divisor = plan.multiplier.sign() == Sign::Minus;
            let mut precision = 128_usize;
            loop {
                scratch_bits(precision, power)?;
                #[cfg(test)]
                {
                    crate::data::source::reserve_source(&mut self.attempts, 1)?;
                    self.attempts.push(precision);
                }
                let (lower, upper) = self.mantissa(operand, precision)?;
                let unit = &factor.denominator << precision;
                let multiplier = power_of_ten(power);
                let lower = lower * &factor.coefficient;
                let upper = upper * &factor.coefficient;
                let (denominator, lower, upper) = if divisor {
                    (unit * multiplier, lower, upper)
                } else {
                    (unit, lower * &multiplier, upper * multiplier)
                };
                let lower = round_integer(&lower, &denominator, negative, format.rounding);
                let upper = round_integer(&upper, &denominator, negative, format.rounding);
                if lower == upper {
                    break RoundedDecimal {
                        negative,
                        coefficient: lower,
                        exponent: plan.quantum.clone(),
                    };
                }
                precision = precision.checked_mul(2).ok_or_else(arithmetic_error)?;
            }
        };
        let text = rounded.emit_reserved(format, plan.output)?;
        Ok(CertifiedPower {
            #[cfg(test)]
            rounded,
            text,
        })
    }

    pub(super) fn factored_decade(
        &mut self,
        operand: &Log10Operand,
        factor: &ExactDecimal,
    ) -> Result<BigInt> {
        let decade = &operand.decade + &factor.decade;
        if operand.fraction.bits() == 0 {
            return Ok(decade);
        }
        let neutral = ExactDecimal::new(false, BigUint::from(1_u8), factor.decade.clone())?;
        if factor.compare_abs(&neutral)? == std::cmp::Ordering::Equal {
            return Ok(decade);
        }
        let threshold = ExactDecimal::new(false, BigUint::from(1_u8), &decade + 1_u8)?;
        Ok(
            if self.compare_power_factor(operand, factor, &threshold)? == std::cmp::Ordering::Less {
                decade
            } else {
                decade + 1_u8
            },
        )
    }

    fn constants(&mut self, precision: usize) -> Result<&ConstantBounds> {
        scratch_bits(precision, 0)?;
        let ratio = precision / 128;
        let index = ratio.trailing_zeros() as usize;
        if index < self.constants.len() {
            if self.constants[index].precision != precision {
                return Err(arithmetic_error());
            }
            #[cfg(test)]
            {
                self.constant_hits += 1;
            }
            return Ok(&self.constants[index]);
        }
        while self.constants.len() <= index {
            let precision = 128_usize
                .checked_shl(u32::try_from(self.constants.len()).map_err(|_| arithmetic_error())?)
                .ok_or_else(arithmetic_error)?;
            if self.constants.len() == self.constants.capacity() {
                let additional = self.constants.capacity().max(1);
                crate::data::source::reserve_source(&mut self.constants, additional)?;
            }
            let (two_lower, two_upper) = super::super::ln_bounds(1_u8, 3_u8, precision)?;
            let (quarter_lower, quarter_upper) = super::super::ln_bounds(1_u8, 9_u8, precision)?;
            self.constants.push(ConstantBounds {
                precision,
                lower: two_lower * 3_u8 + quarter_lower,
                upper: two_upper * 3_u8 + quarter_upper,
            });
            #[cfg(test)]
            {
                self.constant_misses += 1;
            }
        }
        Ok(&self.constants[index])
    }

    pub(super) fn mantissa(
        &mut self,
        operand: &Log10Operand,
        precision: usize,
    ) -> Result<(BigUint, BigUint)> {
        scratch_bits(precision, 0)?;
        #[cfg(test)]
        self.stages.clear();
        let unit = BigUint::from(1_u8) << precision;
        let bounds = self.constants(precision)?;
        let shift = usize::try_from(operand.denominator_bits)
            .ok()
            .and_then(|bits| bits.checked_add(4))
            .ok_or_else(arithmetic_error)?;
        let ceiling = (BigUint::from(1_u8) << shift) - 1_u8;
        let lower_x = (&operand.fraction * &bounds.lower) >> shift;
        let upper_x = (&operand.fraction * &bounds.upper + ceiling) >> shift;
        if lower_x > upper_x || upper_x > (&unit >> 2_usize) {
            return Err(arithmetic_error());
        }
        let mut lower_term = unit.clone();
        let mut upper_term = unit.clone();
        let mut lower = unit.clone();
        let mut upper = unit.clone();
        for index in 1..=precision {
            let denominator = &unit * index;
            lower_term = lower_term * &lower_x / &denominator;
            upper_term = ceiling_ratio(upper_term * &upper_x, &denominator);
            lower += &lower_term;
            upper += &upper_term;
        }
        let omitted = ceiling_ratio(upper_term * &upper_x, &(&unit * (precision + 1)));
        upper += omitted * 2_u8;
        check_interval(&lower, &upper, &unit)?;
        #[cfg(test)]
        self.record_stage(&lower, &upper)?;
        for _ in 0..4 {
            lower = (&lower * &lower) / &unit;
            upper = ceiling_ratio(&upper * &upper, &unit);
            check_interval(&lower, &upper, &unit)?;
            #[cfg(test)]
            self.record_stage(&lower, &upper)?;
        }
        Ok((lower, upper))
    }

    #[cfg(test)]
    fn record_stage(&mut self, lower: &BigUint, upper: &BigUint) -> Result<()> {
        crate::data::source::reserve_source(&mut self.stages, 1)?;
        self.stages.push((lower.clone(), upper.clone()));
        Ok(())
    }
}

fn check_interval(lower: &BigUint, upper: &BigUint, unit: &BigUint) -> Result<()> {
    if lower < unit || lower > upper || *upper >= unit * 11_u8 {
        return Err(Error::Render(
            "Log10 mantissa enclosure invariant failed".into(),
        ));
    }
    Ok(())
}

fn ceiling_ratio(numerator: BigUint, denominator: &BigUint) -> BigUint {
    (numerator + denominator - 1_u8) / denominator
}

/// All full products, before division/shifts, fit the spec07 materialization
/// envelope. Check real index/allocation-layout arithmetic, without prices.
pub(super) fn scratch_bits(precision: usize, power: usize) -> Result<usize> {
    if precision < 128 || !precision.is_multiple_of(128) || !(precision / 128).is_power_of_two() {
        return Err(arithmetic_error());
    }
    let ell = usize::BITS as usize
        - precision
            .checked_add(1)
            .ok_or_else(arithmetic_error)?
            .leading_zeros() as usize;
    let bounds = [
        Some(1084),
        precision.checked_mul(2).and_then(|n| n.checked_add(9)),
        precision.checked_add(1077),
        precision.checked_add(1).and_then(|n| n.checked_add(ell)),
        power
            .checked_mul(4)
            .and_then(|n| n.checked_add(5))
            .and_then(|n| n.checked_add(precision)),
    ];
    let mut bits = 1084;
    for bound in bounds {
        bits = bits.max(bound.ok_or_else(arithmetic_error)?);
    }
    let limbs = bits.checked_add(63).ok_or_else(arithmetic_error)? / 64;
    std::alloc::Layout::array::<u64>(limbs).map_err(|_| arithmetic_error())?;
    Ok(bits)
}

fn power_of_ten(exponent: usize) -> BigUint {
    let mut value = BigUint::from(1_u8);
    for bit in (0..usize::BITS - exponent.leading_zeros()).rev() {
        value = &value * &value;
        if exponent & (1_usize << bit) != 0 {
            value *= 10_u8;
        }
    }
    value
}

fn arithmetic_error() -> Error {
    Error::Render("Log10 presentation precision or scratch arithmetic overflow".into())
}
