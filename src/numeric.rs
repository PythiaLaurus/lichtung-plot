//! Certified binary64 layout arithmetic, independent of platform libm.
//!
//! This is an exact-integer interval algorithm, not a port of CORE-MATH.
//! For x = m * 2^e, 1 <= m < 2, bound ln(m), ln(2), and ln(5/4) with
//! 2 * atanh(z) = 2 * sum(z^(2k+1)/(2k+1)), z = (m-1)/(m+1).
//! All fixed-point operations round outward. Divide the resulting bounds by
//! ln(10) = 3*ln(2) + ln(5/4), and return only when both exact rational bounds
//! round to the same binary64. Otherwise double the working precision.
//!
//! Termination does not assume a libm error bound or a hard-case table: a rational
//! x can have rational log10(x) only at an integral power of ten (prime
//! factorization of x^q = 10^p). The only such binary64 inputs are 10^0..10^22,
//! handled exactly. Every other result is separated from every rounding boundary.

use crate::Error;
use binary::rounded_ratio;
use num_bigint::BigUint;

pub(crate) mod binary;
pub(crate) mod context;
pub(crate) mod decimal;
pub(crate) mod emission;
pub(crate) mod format;
pub(crate) mod function;
pub(crate) mod inputs;
pub(crate) mod lattice;
pub(crate) mod locale;
pub(crate) mod logarithm;
pub(crate) mod mapping;
pub(crate) mod operand;
pub(crate) mod prefixes;
pub(crate) mod profiles;
pub(crate) mod set;
pub(crate) mod slot;
pub(crate) mod source;
mod specifier;
pub(crate) mod table;
pub(crate) mod unit;
pub(crate) mod values;

pub(crate) mod power;

#[cfg(test)]
thread_local! {
    static SERIES_WORK: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn log10(value: f64) -> Result<f64, Error> {
    if value.is_nan() || value < 0.0 {
        return Ok(f64::NAN);
    }
    if value == 0.0 {
        return Ok(f64::NEG_INFINITY);
    }
    if value == f64::INFINITY {
        return Ok(value);
    }
    let mut power = 1.0_f64;
    for exponent in 0..=22 {
        if value.to_bits() == power.to_bits() {
            return Ok(f64::from(exponent));
        }
        power *= 10.0;
    }

    let bits = value.to_bits();
    let stored_exponent = i32::try_from((bits >> 52) & 0x7ff).expect("11 exponent bits");
    let mut significand = bits & ((1_u64 << 52) - 1);
    let exponent = if stored_exponent == 0 {
        let shift = significand.leading_zeros() - 11;
        significand <<= shift;
        -1022 - i32::try_from(shift).expect("at most 52 subnormal bits")
    } else {
        significand |= 1_u64 << 52;
        stored_exponent - 1023
    };
    let mut precision = 128_usize;
    loop {
        let (two_lower, two_upper) = ln_bounds(1_u8, 3_u8, precision)?;
        let (five_fourths_lower, five_fourths_upper) = ln_bounds(1_u8, 9_u8, precision)?;
        let ten_lower = &two_lower * 3_u32 + five_fourths_lower;
        let ten_upper = &two_upper * 3_u32 + five_fourths_upper;
        let (mantissa_lower, mantissa_upper) = ln_bounds(
            significand - (1_u64 << 52),
            significand + (1_u64 << 52),
            precision,
        )?;
        let (lower, upper) = if exponent >= 0 {
            (
                two_lower * exponent.unsigned_abs() + mantissa_lower,
                two_upper * exponent.unsigned_abs() + mantissa_upper,
            )
        } else {
            (
                two_lower * exponent.unsigned_abs() - mantissa_upper,
                two_upper * exponent.unsigned_abs() - mantissa_lower,
            )
        };
        let lower = rounded_ratio(lower, ten_upper)?;
        let upper = rounded_ratio(upper, ten_lower)?;
        if lower.to_bits() == upper.to_bits() {
            return Ok(if exponent < 0 { -lower } else { lower });
        }
        precision = precision.checked_mul(2).ok_or_else(arithmetic_error)?;
    }
}

/// Bounds in units of 2^-precision, for 2*atanh(numerator/denominator), z <= 1/3.
fn ln_bounds(
    numerator: impl Into<BigUint>,
    denominator: impl Into<BigUint>,
    precision: usize,
) -> Result<(BigUint, BigUint), Error> {
    let numerator = numerator.into();
    let denominator = denominator.into();
    if numerator.bits() == 0 {
        return Ok((BigUint::ZERO, BigUint::ZERO));
    }
    let unit = BigUint::from(1_u32) << precision;
    let ceiling = &unit - 1_u32;
    let scaled = numerator << precision;
    let mut lower_power = &scaled / &denominator;
    let mut upper_power = (&scaled + (&denominator - 1_u8)) / denominator;
    let lower_square = (&lower_power * &lower_power) >> precision;
    let upper_square = (&upper_power * &upper_power + &ceiling) >> precision;
    let mut lower = BigUint::ZERO;
    let mut upper = BigUint::ZERO;
    // With N = floor(precision/3)+1, 2*sum(k>=N) z^(2k+1)/(2k+1)
    // < (3/4)*9^-N < 2^-precision. Add one unit to the upper bound.
    for index in 0..=precision / 3 {
        #[cfg(test)]
        SERIES_WORK.with(|work| work.set(work.get() + 1));
        let divisor = index
            .checked_mul(2)
            .and_then(|value| value.checked_add(1))
            .ok_or_else(arithmetic_error)?;
        lower += &lower_power / divisor;
        upper += (&upper_power + (divisor - 1)) / divisor;
        lower_power = (&lower_power * &lower_square) >> precision;
        upper_power = (&upper_power * &upper_square + &ceiling) >> precision;
    }

    Ok((lower << 1, (upper << 1) + 1_u32))
}

fn arithmetic_error() -> Error {
    Error::Render("log10 precision or scratch arithmetic overflow".into())
}

#[cfg(test)]
mod tests;
