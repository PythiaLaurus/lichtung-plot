//! Exact rational thresholds for automatic factors and signed rounding cells.

use std::cmp::Ordering;

use num_bigint::{BigInt, BigUint, Sign};

use super::{certified::PowerFormatter, Log10Operand};
use crate::numeric::decimal::{finite_operand, ExactDecimal, ResolvedFormat};
use crate::{Error, Result};

impl PowerFormatter {
    pub(crate) fn operand_decade(
        &mut self,
        value: f64,
        power: bool,
        format: &ResolvedFormat<'_>,
    ) -> Result<Option<BigInt>> {
        let factor = ExactDecimal::effective_factor(format)?;
        if factor.coefficient.bits() == 0 || (!power && value == 0.0) {
            return Ok(None);
        }
        if power {
            let operand = Log10Operand::new(value, format.decimal_scale)?;
            self.factored_decade(&operand, &factor).map(Some)
        } else {
            Ok(Some(finite_operand(value, format)?.decade))
        }
    }

    pub(in crate::numeric) fn compare_magnitude(
        &mut self,
        value: f64,
        power: bool,
        format: &ResolvedFormat<'_>,
        threshold: &ExactDecimal,
    ) -> Result<Ordering> {
        if !power {
            return finite_operand(value, format)?.compare_abs(threshold);
        }
        let factor = ExactDecimal::effective_factor(format)?;
        if factor.coefficient.bits() == 0 {
            return Ok(if threshold.coefficient.bits() == 0 {
                Ordering::Equal
            } else {
                Ordering::Less
            });
        }
        let operand = Log10Operand::new(value, format.decimal_scale)?;
        if operand.fraction.bits() == 0 {
            return ExactDecimal::ratio(
                false,
                factor.coefficient,
                factor.denominator,
                &operand.decade + factor.exponent,
            )?
            .compare_abs(threshold);
        }
        if threshold.coefficient.bits() == 0 {
            return Ok(Ordering::Greater);
        }
        let decade = self.factored_decade(&operand, &factor)?;
        if decade != threshold.decade {
            return Ok(decade.cmp(&threshold.decade));
        }
        self.compare_power_factor(&operand, &factor, threshold)
    }

    pub(super) fn compare_power_factor(
        &mut self,
        operand: &Log10Operand,
        factor: &ExactDecimal,
        threshold: &ExactDecimal,
    ) -> Result<Ordering> {
        // Callers first bound decades. The remaining exponent difference is
        // bounded by the stored rational coefficients, not the symbolic power.
        let shift = &operand.decade + &factor.exponent - &threshold.exponent;
        let mut top = &factor.coefficient * &threshold.denominator;
        let mut bottom = &factor.denominator * &threshold.coefficient;
        let power = BigUint::from(10_u8)
            .pow(u32::try_from(shift.magnitude()).map_err(|_| arithmetic_error())?);
        if shift.sign() == Sign::Minus {
            bottom *= power;
        } else {
            top *= power;
        }
        let mut precision = 128_usize;
        loop {
            let (lower, upper) = self.mantissa(operand, precision)?;
            let boundary = &bottom << precision;
            // A nonintegral dyadic exponent makes the nonzero rational product
            // irrational, so equality with the rational boundary is impossible.
            if &upper * &top <= boundary {
                return Ok(Ordering::Less);
            }
            if lower * &top >= boundary {
                return Ok(Ordering::Greater);
            }
            precision = precision.checked_mul(2).ok_or_else(arithmetic_error)?;
        }
    }
}

fn arithmetic_error() -> Error {
    Error::Render("numeric threshold comparison scratch arithmetic overflow".into())
}
