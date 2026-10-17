//! Exact finite-binary64 decimal conversion and one-round decimal emission.
//!
//! A finite dyadic m*2^e is an integer when e>=0, otherwise
//! (m*5^-e)*10^e. Decimal scaling shifts that exponent exactly. Quantization
//! divides by a power of ten and uses the exact remainder and quotient parity.
//! Notation changes only the quantum and final decimal-point/exponent placement;
//! carry normalization never rounds again. No floating arithmetic is used here.

use num_bigint::{BigInt, BigUint, Sign};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NumericNotation {
    Decimal,
    Scientific,
    Engineering,
}

/// One precision already selected by its semantic owner, never by geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolvedPrecision {
    Fractional(u8),
    Significant(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NegativeZero {
    Preserve,
    Normalize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrailingZeros {
    Keep,
    Trim,
}

/// Borrowed effective inputs to the one emitter; not a second public policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResolvedFormat<'a> {
    pub(crate) notation: NumericNotation,
    pub(crate) precision: ResolvedPrecision,
    pub(crate) decimal_scale: i16,
    pub(crate) prefix: Option<&'a str>,
    pub(crate) suffix: Option<&'a str>,
    pub(crate) negative_zero: NegativeZero,
    pub(crate) trailing_zeros: TrailingZeros,
}

impl Default for ResolvedFormat<'_> {
    fn default() -> Self {
        Self {
            notation: NumericNotation::Decimal,
            precision: ResolvedPrecision::Fractional(2),
            decimal_scale: 0,
            prefix: None,
            suffix: None,
            negative_zero: NegativeZero::Preserve,
            trailing_zeros: TrailingZeros::Keep,
        }
    }
}

impl ResolvedFormat<'_> {
    pub(crate) fn validate(&self) -> Result<()> {
        let valid = match self.precision {
            ResolvedPrecision::Fractional(digits) => digits <= 17,
            ResolvedPrecision::Significant(digits) => (1..=17).contains(&digits),
        };
        if !valid || !(-324..=324).contains(&self.decimal_scale) {
            return Err(Error::Data(
                "numeric presentation precision or decimal scale is outside its admitted range"
                    .into(),
            ));
        }
        if [self.prefix, self.suffix]
            .into_iter()
            .flatten()
            .any(|text| text.is_empty() || text.chars().any(char::is_control))
        {
            return Err(Error::Data(
                "numeric presentation affixes must be nonempty and contain no control characters"
                    .into(),
            ));
        }
        Ok(())
    }
}

/// A rounded magnitude at one exact decimal quantum, plus the source sign.
/// Arbitrary-size exponents also admit the later symbolic Forest operand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RoundedDecimal {
    pub(crate) negative: bool,
    pub(crate) coefficient: BigUint,
    pub(crate) exponent: BigInt,
}

struct FiniteDecimal {
    negative: bool,
    coefficient: BigUint,
    exponent: i32,
    decade: i32,
}

impl FiniteDecimal {
    fn decode(value: f64, scale: i16) -> Result<Self> {
        if !value.is_finite() {
            return Err(Error::Data(
                "numeric presentation operand must be finite".into(),
            ));
        }
        let bits = value.to_bits();
        let stored = i32::try_from((bits >> 52) & 0x7ff).expect("11 exponent bits");
        let mut significand = bits & ((1_u64 << 52) - 1);
        let mut exponent = if stored == 0 {
            -1074
        } else {
            significand |= 1_u64 << 52;
            stored - 1023 - 52
        };
        if significand == 0 {
            return Ok(Self {
                negative: value.is_sign_negative(),
                coefficient: BigUint::ZERO,
                exponent: 0,
                decade: 0,
            });
        }
        let common = significand.trailing_zeros();
        significand >>= common;
        exponent += i32::try_from(common).expect("at most 52 common bits");

        // Binary64 bounds these powers/shifts by 1074/1023 and the exact
        // decimal coefficient by 767 digits. No scratch quota is involved.
        let (coefficient, decimal_exponent) = if exponent >= 0 {
            (
                BigUint::from(significand) << exponent.unsigned_abs() as usize,
                0,
            )
        } else {
            (
                BigUint::from(significand) * BigUint::from(5_u8).pow(exponent.unsigned_abs()),
                exponent,
            )
        };
        let exponent = decimal_exponent + i32::from(scale);
        let digits =
            i32::try_from(coefficient.to_radix_be(10).len()).map_err(|_| arithmetic_error())?;
        Ok(Self {
            negative: value.is_sign_negative(),
            coefficient,
            exponent,
            decade: exponent + digits - 1,
        })
    }

    fn round(self, format: &ResolvedFormat<'_>) -> RoundedDecimal {
        let quantum = match format.precision {
            ResolvedPrecision::Significant(digits) => self.decade - i32::from(digits) + 1,
            ResolvedPrecision::Fractional(digits) => {
                let origin = match format.notation {
                    NumericNotation::Decimal => 0,
                    NumericNotation::Scientific => self.decade,
                    NumericNotation::Engineering => self.decade.div_euclid(3) * 3,
                };
                origin - i32::from(digits)
            }
        };
        let shift = quantum - self.exponent;
        // The admitted finite operand/scale/precision bounds this power by
        // 1398. A fine quantum pads exactly; a coarse one rounds only here.
        let coefficient = if shift <= 0 {
            self.coefficient * BigUint::from(10_u8).pow(shift.unsigned_abs())
        } else {
            let denominator = BigUint::from(10_u8).pow(shift.unsigned_abs());
            let mut quotient = &self.coefficient / &denominator;
            let twice_remainder = (&self.coefficient % &denominator) << 1_usize;
            if twice_remainder > denominator || (twice_remainder == denominator && quotient.bit(0))
            {
                quotient += 1_u8;
            }
            quotient
        };
        RoundedDecimal {
            negative: self.negative,
            coefficient,
            exponent: quantum.into(),
        }
    }
}

pub(crate) fn format_finite(value: f64, format: &ResolvedFormat<'_>) -> Result<String> {
    format.validate()?;
    FiniteDecimal::decode(value, format.decimal_scale)?
        .round(format)
        .emit(format)
}

impl RoundedDecimal {
    /// Serialize an already rounded number. Scale has already been consumed by
    /// its mathematical operand, and must not be applied again at this boundary.
    pub(crate) fn emit(&self, format: &ResolvedFormat<'_>) -> Result<String> {
        self.emit_reserved(format, String::new())
    }

    /// A power operand reserves its symbolic output bound before any refinement
    /// or physical coefficient construction. Reuse that same empty buffer here.
    pub(crate) fn emit_reserved(
        &self,
        format: &ResolvedFormat<'_>,
        mut text: String,
    ) -> Result<String> {
        format.validate()?;
        if !text.is_empty() {
            return Err(Error::Render(
                "numeric output reservation must be empty".into(),
            ));
        }
        let mut digits = self.coefficient.to_radix_be(10);
        for digit in &mut digits {
            *digit += b'0';
        }
        let zero = self.coefficient.bits() == 0;
        let negative = self.negative && (!zero || format.negative_zero == NegativeZero::Preserve);
        let decade = if zero {
            BigInt::ZERO
        } else {
            &self.exponent + BigInt::from(digits.len() - 1)
        };
        let display_exponent = match format.notation {
            NumericNotation::Decimal => BigInt::ZERO,
            NumericNotation::Scientific => decade.clone(),
            NumericNotation::Engineering => {
                let mut remainder = &decade % 3_u8;
                if remainder.sign() == Sign::Minus {
                    remainder += 3_u8;
                }
                &decade - remainder
            }
        };
        let point = if zero {
            BigInt::from(1_u8)
        } else {
            BigInt::from(digits.len()) + &self.exponent - &display_exponent
        };
        let fractional = match format.precision {
            ResolvedPrecision::Fractional(count) => usize::from(count),
            ResolvedPrecision::Significant(count) => {
                let places = if zero {
                    BigInt::from(count - 1)
                } else {
                    BigInt::from(count) - &point
                };
                if places.sign() == Sign::Minus {
                    0
                } else {
                    usize::try_from(places).map_err(|_| arithmetic_error())?
                }
            }
        };
        let point = isize::try_from(point).map_err(|_| arithmetic_error())?;
        let integral = usize::try_from(point.max(1)).map_err(|_| arithmetic_error())?;
        let exponent = (format.notation != NumericNotation::Decimal)
            .then(|| display_exponent.to_str_radix(10));
        let prefix = format.prefix.unwrap_or_default();
        let suffix = format.suffix.unwrap_or_default();
        let length = [
            prefix.len(),
            usize::from(negative),
            integral,
            usize::from(fractional > 0),
            fractional,
            exponent.as_ref().map_or(0, |text| 1 + text.len()),
            suffix.len(),
        ]
        .into_iter()
        .try_fold(0_usize, usize::checked_add)
        .filter(|length| isize::try_from(*length).is_ok())
        .ok_or_else(arithmetic_error)?;
        text.try_reserve_exact(length)
            .map_err(|error| Error::Render(format!("numeric presentation output: {error}")))?;
        text.push_str(prefix);
        if negative {
            text.push('-');
        }
        emit_magnitude(&mut text, &digits, point, fractional)?;
        if fractional > 0 && format.trailing_zeros == TrailingZeros::Trim {
            while text.ends_with('0') {
                text.pop();
            }
            if text.ends_with('.') {
                text.pop();
            }
        }
        if let Some(exponent) = exponent {
            text.push('e');
            text.push_str(&exponent);
        }
        text.push_str(suffix);
        Ok(text)
    }
}

fn emit_magnitude(text: &mut String, digits: &[u8], point: isize, fractional: usize) -> Result<()> {
    let integral_digits = usize::try_from(point.max(0)).map_err(|_| arithmetic_error())?;
    let taken = integral_digits.min(digits.len());
    if integral_digits == 0 {
        text.push('0');
    } else {
        text.push_str(std::str::from_utf8(&digits[..taken]).expect("ASCII decimal digits"));
        push_zeroes(text, integral_digits - taken);
    }
    let leading = if point < 0 { point.unsigned_abs() } else { 0 };
    let leading = fractional.min(leading);
    let copied = (fractional - leading).min(digits.len() - taken);
    if fractional > 0 {
        text.push('.');
        push_zeroes(text, leading);
        text.push_str(
            std::str::from_utf8(&digits[taken..taken + copied]).expect("ASCII decimal digits"),
        );
        push_zeroes(text, fractional - leading - copied);
    }
    if digits[taken + copied..].iter().any(|digit| *digit != b'0') {
        return Err(Error::Render(
            "numeric decimal emission would discard nonzero digits".into(),
        ));
    }
    Ok(())
}

fn push_zeroes(text: &mut String, mut count: usize) {
    const ZEROES: &str = "0000000000000000000000000000000000000000000000000000000000000000";
    while count > 0 {
        let part = count.min(ZEROES.len());
        text.push_str(&ZEROES[..part]);
        count -= part;
    }
}

fn arithmetic_error() -> Error {
    Error::Render("numeric presentation output length or decimal arithmetic overflow".into())
}

#[cfg(test)]
mod tests;
