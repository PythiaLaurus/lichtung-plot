//! Exact origin-aligned binary64 lattices. Membership uses representable endpoint
//! rounding cells, not a tolerance or lossy conversion of a lattice index.

use crate::numeric::binary::{signed_ratio, units};
use crate::{Error, Result, TickRequirement, TickSpace};

#[cfg(test)]
mod tests;
use num_bigint::{BigInt, BigUint, Sign};

pub(super) struct Positions {
    pub(super) coordinates: Vec<f64>,
    pub(super) data: Option<Vec<f64>>,
}

pub(crate) fn fixed_is_empty(
    range: (f64, f64),
    raw: Option<(f64, f64)>,
    log: bool,
    parameters: (f64, f64, TickSpace, TickRequirement),
    maximum: usize,
) -> Result<bool> {
    let (step, origin, space, requirement) = parameters;
    if requirement != TickRequirement::Fixed {
        return Ok(false);
    }
    let view = if log && space == TickSpace::Data {
        data_range(range, raw)?
    } else {
        range
    };
    match bounds(view, step, origin)? {
        Some((_, count)) if count <= maximum => Ok(count == 0),
        _ => Err(Error::Render(
            "fixed interval count exceeds its configured maximum or storage representation".into(),
        )),
    }
}

pub(super) fn data_range(range: (f64, f64), raw: Option<(f64, f64)>) -> Result<(f64, f64)> {
    if let Some(raw) = raw {
        return Ok(raw);
    }
    let mut formatter = crate::numeric::power::certified::PowerFormatter::default();
    let lower = if range.0 < -324.0 {
        f64::from_bits(1)
    } else {
        formatter
            .affine_binary64(range, 0, 1)?
            .max(f64::from_bits(1))
    };
    let upper = formatter.affine_binary64(range, 1, 1)?;
    Ok((lower, upper))
}

pub(super) fn positions(
    range: (f64, f64),
    raw: Option<(f64, f64)>,
    log: bool,
    parameters: (f64, f64, TickSpace, TickRequirement),
    maximum: usize,
) -> Result<Option<Positions>> {
    let (step, origin, space, requirement) = parameters;
    let raw_mode = log && space == TickSpace::Data;
    let view = if raw_mode {
        data_range(range, raw)?
    } else {
        range
    };
    let (first, count) = match bounds(view, step, origin)? {
        Some(bounds) if bounds.1 <= maximum => bounds,
        _ => return Ok(None),
    };
    let mut coordinates = Vec::new();
    super::reserve(&mut coordinates, count)?;
    let mut data = raw_mode.then(Vec::new);
    if let Some(data) = &mut data {
        super::reserve(data, count)?;
    }
    let origin = units(origin)?;
    let step = units(step)?;
    let divisor = BigUint::from(1_u8) << 1074_usize;
    for offset in 0..count {
        let index = &first + offset;
        let value = signed_ratio(&origin + &step * index, divisor.clone())?;
        let coordinate = if raw_mode {
            if value <= 0.0 {
                return Err(Error::Render(
                    "data interval produced a nonpositive logarithmic position".into(),
                ));
            }
            if value.to_bits() == view.0.to_bits() {
                range.0
            } else if value.to_bits() == view.1.to_bits() {
                range.1
            } else {
                crate::numeric::log10(value)?
            }
        } else {
            value
        };
        if !coordinate.is_finite()
            || coordinate < range.0
            || coordinate > range.1
            || coordinates
                .last()
                .is_some_and(|previous| *previous >= coordinate)
        {
            return if requirement == TickRequirement::Preferred {
                Ok(None)
            } else {
                Err(Error::Render(format!(
                    "interval occurrence {offset} collapses or leaves its represented domain"
                )))
            };
        }
        coordinates.push(coordinate);
        if let Some(data) = &mut data {
            data.push(value);
        }
    }
    Ok(Some(Positions { coordinates, data }))
}

fn bounds(range: (f64, f64), step: f64, origin: f64) -> Result<Option<(BigInt, usize)>> {
    if range.0 > range.1 {
        return Ok(Some((BigInt::ZERO, 0)));
    }
    let lower = units(range.0)?;
    let upper = units(range.1)?;
    let before = if range.0.next_down().is_finite() {
        units(range.0.next_down())?
    } else {
        &lower - (units(range.0.next_up())? - &lower)
    };
    let after = if range.1.next_up().is_finite() {
        units(range.1.next_up())?
    } else {
        &upper + (&upper - units(range.1.next_down())?)
    };
    let origin = units(origin)? << 1;
    let divisor = units(step)? << 1;
    let low = before + lower - &origin;
    let high = upper + after - origin;
    let first = if range.0.to_bits() & 1 == 0 {
        ceil(&low, &divisor)
    } else {
        floor(&low, &divisor) + 1
    };
    let last = if range.1.to_bits() & 1 == 0 {
        floor(&high, &divisor)
    } else {
        ceil(&high, &divisor) - 1
    };
    if last < first {
        return Ok(Some((first, 0)));
    }
    Ok(usize::try_from(last - &first + 1_u8)
        .ok()
        .map(|count| (first, count)))
}

fn floor(numerator: &BigInt, denominator: &BigInt) -> BigInt {
    let quotient = numerator / denominator;
    if (numerator % denominator).sign() == Sign::Minus {
        quotient - 1
    } else {
        quotient
    }
}
fn ceil(numerator: &BigInt, denominator: &BigInt) -> BigInt {
    let quotient = numerator / denominator;
    if (numerator % denominator).sign() == Sign::Plus {
        quotient + 1
    } else {
        quotient
    }
}
