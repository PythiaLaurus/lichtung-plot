//! Exact rational-to-binary64 conversion, including signed affine coordinates.

use crate::{Error, Result};
use num_bigint::{BigInt, BigUint, Sign};

/// Integer units of the least positive binary64 subnormal, with no float arithmetic.
pub(crate) fn units(value: f64) -> Result<BigInt> {
    if !value.is_finite() {
        return Err(Error::Data("exact binary operand must be finite".into()));
    }
    let bits = value.to_bits();
    let exponent = usize::try_from((bits >> 52) & 0x7ff).expect("11 exponent bits");
    let fraction = bits & ((1_u64 << 52) - 1);
    let magnitude = if exponent == 0 {
        BigUint::from(fraction)
    } else {
        BigUint::from(fraction | (1_u64 << 52)) << (exponent - 1)
    };
    Ok(BigInt::from_biguint(
        if value.is_sign_negative() {
            Sign::Minus
        } else {
            Sign::Plus
        },
        magnitude,
    ))
}

pub(crate) fn affine(lower: f64, upper: f64, index: usize, denominator: usize) -> Result<f64> {
    if index == 0 {
        return Ok(lower);
    }
    if index == denominator {
        return Ok(upper);
    }
    let numerator = units(lower)? * (denominator - index) + units(upper)? * index;
    signed_ratio(numerator, BigUint::from(denominator) << 1074)
}

pub(crate) fn signed_ratio(numerator: BigInt, denominator: BigUint) -> Result<f64> {
    let (sign, magnitude) = numerator.into_parts();
    let negative = sign == Sign::Minus;
    let value = rounded_ratio(magnitude, denominator)?;
    Ok(if negative { -value } else { value })
}

/// One nearest-even conversion, with exact subnormal cells and binade carries.
pub(super) fn rounded_ratio(mut numerator: BigUint, mut denominator: BigUint) -> Result<f64> {
    if denominator.bits() == 0 {
        return Err(arithmetic_error());
    }
    if numerator.bits() == 0 {
        return Ok(0.0);
    }
    let mut exponent = i64::try_from(numerator.bits()).map_err(|_| arithmetic_error())?
        - i64::try_from(denominator.bits()).map_err(|_| arithmetic_error())?;
    let below = if exponent >= 0 {
        numerator < (&denominator << usize::try_from(exponent).map_err(|_| arithmetic_error())?)
    } else {
        (&numerator << usize::try_from(-exponent).map_err(|_| arithmetic_error())?) < denominator
    };
    exponent -= i64::from(below);
    if exponent > 1023 {
        return Err(Error::Render(
            "exact numeric position exceeds finite binary64".into(),
        ));
    }
    let shift = 52 - exponent.max(-1022);
    if shift >= 0 {
        numerator <<= usize::try_from(shift).map_err(|_| arithmetic_error())?;
    } else {
        denominator <<= usize::try_from(-shift).map_err(|_| arithmetic_error())?;
    }
    let mut significand =
        u64::try_from(&numerator / &denominator).map_err(|_| arithmetic_error())?;
    let remainder = (&numerator % &denominator) << 1;
    if remainder > denominator || (remainder == denominator && !significand.is_multiple_of(2)) {
        significand += 1;
    }
    let bits = if exponent < -1022 {
        significand
    } else {
        (u64::try_from(exponent + 1023).map_err(|_| arithmetic_error())? << 52) + significand
            - (1_u64 << 52)
    };
    let value = f64::from_bits(bits);
    if !value.is_finite() {
        return Err(Error::Render(
            "exact numeric position exceeds finite binary64".into(),
        ));
    }
    Ok(value)
}

fn arithmetic_error() -> Error {
    Error::Render("binary rational scratch arithmetic overflow".into())
}
