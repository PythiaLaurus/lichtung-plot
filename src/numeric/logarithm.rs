//! Signed outward logarithms of positive rational operands in 2^-p units.

use crate::{Error, Result};
use num_bigint::{BigInt, BigUint, Sign};

#[derive(Debug)]
pub(crate) struct LogBounds {
    pub(crate) lower: BigInt,
    pub(crate) upper: BigInt,
}

pub(crate) fn floor(numerator: &BigInt, denominator: &BigInt) -> BigInt {
    let quotient = numerator / denominator;
    if (numerator % denominator).sign() == Sign::Minus {
        quotient - 1
    } else {
        quotient
    }
}
pub(crate) fn ceil(numerator: &BigInt, denominator: &BigInt) -> BigInt {
    let quotient = numerator / denominator;
    if (numerator % denominator).sign() == Sign::Plus {
        quotient + 1
    } else {
        quotient
    }
}

pub(crate) fn represented(value: f64, precision: usize) -> Result<LogBounds> {
    let numerator = super::binary::units(value)?.into_parts().1;
    rational(&numerator, &(BigUint::from(1_u8) << 1074_usize), precision)
}

pub(crate) fn rational(
    numerator: &BigUint,
    denominator: &BigUint,
    precision: usize,
) -> Result<LogBounds> {
    if numerator.bits() == 0 || denominator.bits() == 0 {
        return Err(Error::Render(
            "logarithmic rational operand must be positive".into(),
        ));
    }
    if numerator == denominator {
        return Ok(LogBounds {
            lower: BigInt::ZERO,
            upper: BigInt::ZERO,
        });
    }
    let mut exponent = i64::try_from(numerator.bits()).map_err(|_| arithmetic_error())?
        - i64::try_from(denominator.bits()).map_err(|_| arithmetic_error())?;
    let below = if exponent >= 0 {
        numerator < &(denominator << usize::try_from(exponent).map_err(|_| arithmetic_error())?)
    } else {
        &(numerator << usize::try_from(-exponent).map_err(|_| arithmetic_error())?) < denominator
    };
    exponent -= i64::from(below);
    let (top, bottom) = if exponent >= 0 {
        (
            numerator.clone(),
            denominator << usize::try_from(exponent).map_err(|_| arithmetic_error())?,
        )
    } else {
        (
            numerator << usize::try_from(-exponent).map_err(|_| arithmetic_error())?,
            denominator.clone(),
        )
    };
    let (mantissa_low, mantissa_high) =
        super::ln_bounds(&top - &bottom, &top + &bottom, precision)?;
    let (two_low, two_high) = super::ln_bounds(1_u8, 3_u8, precision)?;
    let (quarter_low, quarter_high) = super::ln_bounds(1_u8, 9_u8, precision)?;
    let ten_low = &two_low * 3_u8 + quarter_low;
    let ten_high = &two_high * 3_u8 + quarter_high;
    let (lower, upper) = if exponent >= 0 {
        (
            BigInt::from(mantissa_low + &two_low * exponent.unsigned_abs()),
            BigInt::from(mantissa_high + &two_high * exponent.unsigned_abs()),
        )
    } else {
        (
            BigInt::from(mantissa_low) - BigInt::from(&two_high * exponent.unsigned_abs()),
            BigInt::from(mantissa_high) - BigInt::from(&two_low * exponent.unsigned_abs()),
        )
    };
    let low_divisor = BigInt::from(if lower.sign() == Sign::Minus {
        ten_low.clone()
    } else {
        ten_high.clone()
    });
    let high_divisor = BigInt::from(if upper.sign() == Sign::Minus {
        ten_high
    } else {
        ten_low
    });
    Ok(LogBounds {
        lower: floor(&(lower << precision), &low_divisor),
        upper: ceil(&(upper << precision), &high_divisor),
    })
}

fn arithmetic_error() -> Error {
    Error::Render("rational logarithm scratch arithmetic overflow".into())
}
