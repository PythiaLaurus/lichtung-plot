//! Certified affine interpolation of physical endpoints without inverting a
//! rounded log or first rounding either physical power to binary64.

use super::{certified::PowerFormatter, Log10Operand};
use crate::{Error, Result};
use num_bigint::BigUint;

impl PowerFormatter {
    pub(crate) fn affine_binary64(
        &mut self,
        range: (f64, f64),
        index: usize,
        denominator: usize,
    ) -> Result<f64> {
        let lower = Log10Operand::new(range.0, 0)?;
        let upper = Log10Operand::new(range.1, 0)?;
        let mut precision = 128_usize;
        loop {
            let (lower_min, lower_max, lower_unit) = self.binary_bounds(&lower, precision)?;
            let (upper_min, upper_max, upper_unit) = self.binary_bounds(&upper, precision)?;
            let low =
                &lower_min * &upper_unit * (denominator - index) + &upper_min * &lower_unit * index;
            let high =
                lower_max * &upper_unit * (denominator - index) + upper_max * &lower_unit * index;
            let divisor = lower_unit * upper_unit * denominator;
            let low = crate::numeric::binary::rounded_ratio(low, divisor.clone())?;
            let high = crate::numeric::binary::rounded_ratio(high, divisor);
            if high.is_ok_and(|high| low.to_bits() == high.to_bits()) {
                return Ok(low);
            }
            precision = precision
                .checked_mul(2)
                .ok_or_else(|| Error::Render("exact-count power scratch overflow".into()))?;
        }
    }

    fn binary_bounds(
        &mut self,
        operand: &Log10Operand,
        precision: usize,
    ) -> Result<(BigUint, BigUint, BigUint)> {
        let decade = i32::try_from(&operand.decade).map_err(|_| {
            Error::Render("data-space power endpoint exceeds finite binary64 range".into())
        })?;
        if !(-324..=308).contains(&decade) {
            return Err(Error::Render(
                "data-space power endpoint exceeds finite binary64 range".into(),
            ));
        }
        let unit = BigUint::from(1_u8) << precision;
        let (mut lower, mut upper) = if operand.fraction.bits() == 0 {
            (unit.clone(), unit.clone())
        } else {
            self.mantissa(operand, precision)?
        };
        let power = BigUint::from(10_u8).pow(decade.unsigned_abs());
        let divisor = if decade < 0 {
            unit * power
        } else {
            lower *= &power;
            upper *= power;
            unit
        };
        Ok((lower, upper, divisor))
    }
}
