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

use crate::font::canonical::OperationGenerationResources;
use crate::Error;
use num_bigint::BigUint;

#[cfg(test)]
thread_local! {
    static SERIES_WORK: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn log10(value: f64, resources: &OperationGenerationResources) -> Result<f64, Error> {
    admit(resources, 0, 0)?;
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
        admit(resources, 0, 0)?;
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
        // Admit orchestration products/sums before any first-party BigUint algebra.
        admit(resources, precision, 8)?;
        let (two_lower, two_upper) = ln_bounds(1, 3, precision, resources)?;
        let (five_fourths_lower, five_fourths_upper) = ln_bounds(1, 9, precision, resources)?;
        let ten_lower = &two_lower * 3_u32 + five_fourths_lower;
        let ten_upper = &two_upper * 3_u32 + five_fourths_upper;
        let (mantissa_lower, mantissa_upper) = ln_bounds(
            significand - (1_u64 << 52),
            significand + (1_u64 << 52),
            precision,
            resources,
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
        let lower = rounded_ratio(lower, ten_upper, resources)?;
        let upper = rounded_ratio(upper, ten_lower, resources)?;
        if lower.to_bits() == upper.to_bits() {
            return Ok(if exponent < 0 { -lower } else { lower });
        }
        precision = precision.checked_mul(2).ok_or_else(arithmetic_error)?;
    }
}

/// Bounds in units of 2^-precision, for 2*atanh(numerator/denominator), z <= 1/3.
fn ln_bounds(
    numerator: u64,
    denominator: u64,
    precision: usize,
    resources: &OperationGenerationResources,
) -> Result<(BigUint, BigUint), Error> {
    admit(resources, precision, if numerator == 0 { 0 } else { 7 })?;
    if numerator == 0 {
        return Ok((BigUint::ZERO, BigUint::ZERO));
    }
    let unit = BigUint::from(1_u32) << precision;
    let ceiling = &unit - 1_u32;
    let scaled = BigUint::from(numerator) << precision;
    let mut lower_power = &scaled / denominator;
    let mut upper_power = (&scaled + (denominator - 1)) / denominator;
    let lower_square = (&lower_power * &lower_power) >> precision;
    let upper_square = (&upper_power * &upper_power + &ceiling) >> precision;
    let mut lower = BigUint::ZERO;
    let mut upper = BigUint::ZERO;
    // With N = floor(precision/3)+1, 2*sum(k>=N) z^(2k+1)/(2k+1)
    // < (3/4)*9^-N < 2^-precision. Add one unit to the upper bound.
    for index in 0..=precision / 3 {
        // Two quotient temporaries, two accumulating sums, and two products.
        // Internal limb arithmetic in num-bigint is not counted as logical visits.
        admit(resources, precision, 6)?;
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
    admit(resources, precision, 3)?;
    Ok((lower << 1, (upper << 1) + 1_u32))
}

/// Exact nearest-even rational conversion. Nonzero binary64 log10 magnitudes
/// are normal and lie between 2^-55 and 324, including their certified bounds.
fn rounded_ratio(
    mut numerator: BigUint,
    mut denominator: BigUint,
    resources: &OperationGenerationResources,
) -> Result<f64, Error> {
    let bits = numerator.bits().max(denominator.bits());
    admit(
        resources,
        usize::try_from(bits).map_err(|_| arithmetic_error())?,
        4,
    )?;
    let mut exponent = i32::try_from(numerator.bits()).map_err(|_| arithmetic_error())?
        - i32::try_from(denominator.bits()).map_err(|_| arithmetic_error())?;
    let below = if exponent >= 0 {
        numerator < (&denominator << exponent.unsigned_abs() as usize)
    } else {
        (&numerator << exponent.unsigned_abs() as usize) < denominator
    };
    exponent -= i32::from(below);
    let shift = 52_i32.checked_sub(exponent).ok_or_else(arithmetic_error)?;
    if shift >= 0 {
        numerator <<= shift.unsigned_abs() as usize;
    } else {
        denominator <<= shift.unsigned_abs() as usize;
    }
    let mut significand =
        u64::try_from(&numerator / &denominator).map_err(|_| arithmetic_error())?;
    let remainder = (&numerator % &denominator) << 1;
    if remainder > denominator || (remainder == denominator && !significand.is_multiple_of(2)) {
        significand += 1;
    }
    // A carry into bit 53 naturally increments the encoded exponent.
    Ok(f64::from_bits(
        (u64::try_from(exponent + 1023).map_err(|_| arithmetic_error())? << 52) + significand
            - (1_u64 << 52),
    ))
}

fn arithmetic_error() -> Error {
    Error::Render("log10 precision or scratch arithmetic overflow".into())
}

/// Every first-party temporary has at most 2*p+128 significant bits: p-bit
/// powers, their products, the 54-bit input mantissa, and at most 107 bits of
/// final normalization. Round to 64-bit limbs and allow doubled Vec capacity.
/// Charge cumulative scratch, not peak live storage, against the existing root
/// 512 MiB ledger. That same ledger's 1M visits bounds refinement and series work.
fn admit(
    resources: &OperationGenerationResources,
    precision: usize,
    temporaries: u64,
) -> Result<(), Error> {
    resources
        .claim_visits(1)
        .map_err(|error| Error::Render(format!("log10 work admission: {error:?}")))?;
    if temporaries == 0 {
        return Ok(());
    }
    let bits = (precision as u128)
        .checked_mul(2)
        .and_then(|bits| bits.checked_add(128))
        .ok_or_else(arithmetic_error)?;
    let bytes = bits
        .div_ceil(64)
        .checked_mul(16)
        .and_then(|bytes| bytes.checked_mul(u128::from(temporaries)))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(arithmetic_error)?;
    resources
        .claim_bytes(bytes)
        .map_err(|error| Error::Render(format!("log10 scratch admission: {error}")))
}

#[cfg(test)]
mod tests;
