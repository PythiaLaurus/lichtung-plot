//! Exact finite-binary64 decimal conversion and one-round decimal emission.
//!
//! A finite dyadic m*2^e is an integer when e>=0, otherwise
//! (m*5^-e)*10^e. Decimal scaling shifts that exponent exactly. Quantization
//! divides by a power of ten and uses the exact remainder and quotient parity.
//! Notation changes only the quantum and final decimal-point/exponent placement;
//! carry normalization never rounds again. No floating arithmetic is used here.

use num_bigint::{BigInt, BigUint, Sign};

use crate::{Error, Result};

pub(super) const MAX_FRACTIONAL_DIGITS: u8 = 20;
pub(super) const MAX_SIGNIFICANT_DIGITS: u8 = 21;

/// Decimal-point and exponent spelling for numeric presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericNotation {
    /// Plain decimal notation, without an exponent.
    Decimal,
    /// One leading integral digit and a decimal exponent.
    Scientific,
    /// One to three integral digits and an exponent divisible by three.
    Engineering,
}

/// Exact rounding at the selected decimal quantum, before numeric emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericRounding {
    /// Round to the nearest value; an exact tie chooses the even coefficient.
    NearestEven,
    /// Round to the nearest value; an exact tie rounds away from zero.
    NearestAway,
    /// Round to the nearest value; an exact tie rounds toward zero.
    NearestTowardZero,
    /// Discard the fractional remainder toward zero.
    TowardZero,
    /// Round every nonzero fractional remainder away from zero.
    AwayFromZero,
    /// Round toward negative infinity.
    Floor,
    /// Round toward positive infinity.
    Ceiling,
}

/// One precision already selected by its semantic owner, never by geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolvedPrecision {
    Fractional(u8),
    Significant(u8),
}

/// Whether actual or rounded negative zero retains its minus sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NegativeZero {
    /// Preserve the source sign, including when rounding produces zero.
    Preserve,
    /// Display every zero without a minus sign.
    Normalize,
}

/// Fractional trailing-zero handling after numeric rounding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrailingZeros {
    /// Retain the digits required by the selected precision.
    Keep,
    /// Remove fractional trailing zeros and an empty decimal point.
    /// Integral place-value zeros are never removed.
    Trim,
}

/// Borrowed effective inputs to the one emitter; not a second public policy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ResolvedFormat<'a> {
    pub(crate) notation: NumericNotation,
    pub(crate) precision: ResolvedPrecision,
    pub(crate) decimal_scale: i16,
    pub(crate) prefix: Option<&'a str>,
    pub(crate) suffix: Option<&'a str>,
    pub(crate) negative_zero: NegativeZero,
    pub(crate) trailing_zeros: TrailingZeros,
    pub(crate) rounding: NumericRounding,
    pub(crate) multiplier: f64,
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
            rounding: NumericRounding::NearestEven,
            multiplier: 1.0,
        }
    }
}

impl ResolvedFormat<'_> {
    pub(crate) fn validate(&self) -> Result<()> {
        let valid = match self.precision {
            ResolvedPrecision::Fractional(digits) => digits <= MAX_FRACTIONAL_DIGITS,
            ResolvedPrecision::Significant(digits) => {
                (1..=MAX_SIGNIFICANT_DIGITS).contains(&digits)
            }
        };
        if !valid || !(-324..=324).contains(&self.decimal_scale) || !self.multiplier.is_finite() {
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

/// One output allocation and a reusable, fallible base-10^9 conversion buffer.
#[derive(Debug, Default)]
pub(crate) struct DecimalBuffer {
    text: String,
    chunks: Vec<u32>,
}

impl DecimalBuffer {
    pub(crate) fn reserve(output_bytes: usize, integer_bits: usize) -> Result<Self> {
        let mut buffer = Self::default();
        buffer.reserve_chunks(u64::try_from(integer_bits).map_err(|_| arithmetic_error())?)?;
        buffer
            .text
            .try_reserve_exact(output_bytes)
            .map_err(|error| Error::Render(format!("numeric presentation output: {error}")))?;
        Ok(buffer)
    }

    fn reserve_chunks(&mut self, bits: u64) -> Result<()> {
        // 2^29 < 10^9, so ceil(bits/29) bounds the exact division count.
        let count = usize::try_from(bits.div_ceil(29).max(1)).map_err(|_| arithmetic_error())?;
        let additional = count.saturating_sub(self.chunks.len());
        crate::data::source::reserve_source(&mut self.chunks, additional)
    }

    fn convert(&mut self, value: &BigUint) -> Result<()> {
        self.chunks.clear();
        self.reserve_chunks(value.bits())?;
        let mut remaining = value.clone();
        loop {
            let quotient = &remaining / 1_000_000_000_u32;
            let remainder =
                u32::try_from(&remaining % 1_000_000_000_u32).expect("decimal chunk below 10^9");
            if self.chunks.len() == self.chunks.capacity() {
                return Err(arithmetic_error());
            }
            self.chunks.push(remainder);
            if quotient.bits() == 0 {
                break;
            }
            remaining = quotient;
        }
        Ok(())
    }

    fn digit_count(&self) -> Result<usize> {
        let leading = *self.chunks.last().ok_or_else(arithmetic_error)?;
        (self.chunks.len() - 1)
            .checked_mul(9)
            .and_then(|length| length.checked_add(chunk_width(leading)))
            .ok_or_else(arithmetic_error)
    }

    fn append_exponent(&mut self, exponent: &BigInt) -> Result<()> {
        self.convert(exponent.magnitude())?;
        self.text.push('e');
        if exponent.sign() == Sign::Minus {
            self.text.push('-');
        }
        visit_digits(&self.chunks, 0..self.digit_count()?, |digits| {
            self.text
                .push_str(std::str::from_utf8(digits).expect("ASCII decimal digits"));
        });
        Ok(())
    }
}

pub(super) struct ExactDecimal {
    pub(super) negative: bool,
    pub(super) coefficient: BigUint,
    pub(super) exponent: BigInt,
    pub(super) decade: BigInt,
}

impl ExactDecimal {
    pub(super) fn new(negative: bool, coefficient: BigUint, exponent: BigInt) -> Result<Self> {
        let decade = if coefficient.bits() == 0 {
            BigInt::ZERO
        } else {
            let mut digits = DecimalBuffer::default();
            digits.convert(&coefficient)?;
            &exponent + BigInt::from(digits.digit_count()? - 1)
        };
        Ok(Self {
            negative,
            coefficient,
            exponent,
            decade,
        })
    }

    pub(super) fn decode(value: f64, scale: i16) -> Result<Self> {
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
                exponent: BigInt::ZERO,
                decade: BigInt::ZERO,
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
        Self::new(value.is_sign_negative(), coefficient, exponent.into())
    }

    fn multiply(self, multiplier: f64) -> Result<Self> {
        let other = Self::decode(multiplier, 0)?;
        Self::new(
            self.negative ^ other.negative,
            self.coefficient * other.coefficient,
            self.exponent + other.exponent,
        )
    }

    pub(super) fn round(self, format: &ResolvedFormat<'_>) -> Result<RoundedDecimal> {
        let quantum = decimal_quantum(&self.decade, format);
        let shift = &quantum - &self.exponent;
        if shift.sign() != Sign::Plus || self.coefficient.bits() == 0 {
            return Ok(RoundedDecimal {
                negative: self.negative,
                coefficient: self.coefficient,
                exponent: self.exponent,
            });
        }
        let coefficient = if &self.decade - &quantum < BigInt::from(-1) {
            tiny_coefficient(self.negative, format.rounding)
        } else {
            let shift = u32::try_from(shift).map_err(|_| arithmetic_error())?;
            let denominator = BigUint::from(10_u8).pow(shift);
            round_integer(
                &self.coefficient,
                &denominator,
                self.negative,
                format.rounding,
            )
        };
        Ok(RoundedDecimal {
            negative: self.negative,
            coefficient,
            exponent: quantum,
        })
    }
}

pub(super) fn tiny_coefficient(negative: bool, rounding: NumericRounding) -> BigUint {
    BigUint::from(u8::from(match rounding {
        NumericRounding::AwayFromZero => true,
        NumericRounding::Floor => negative,
        NumericRounding::Ceiling => !negative,
        _ => false,
    }))
}

pub(super) fn decimal_quantum(decade: &BigInt, format: &ResolvedFormat<'_>) -> BigInt {
    match format.precision {
        ResolvedPrecision::Significant(digits) => decade - (digits - 1),
        ResolvedPrecision::Fractional(digits) => {
            let origin = match format.notation {
                NumericNotation::Decimal => BigInt::ZERO,
                NumericNotation::Scientific => decade.clone(),
                NumericNotation::Engineering => engineering_exponent(decade),
            };
            origin - digits
        }
    }
}

fn engineering_exponent(decade: &BigInt) -> BigInt {
    let mut remainder = decade % 3_u8;
    if remainder.sign() == Sign::Minus {
        remainder += 3_u8;
    }
    decade - remainder
}

/// One exact signed-magnitude quantizer for ordinary rationals and power endpoints.
pub(super) fn round_integer(
    numerator: &BigUint,
    denominator: &BigUint,
    negative: bool,
    rounding: NumericRounding,
) -> BigUint {
    let mut quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let increment = match rounding {
        NumericRounding::TowardZero => false,
        NumericRounding::AwayFromZero => remainder.bits() != 0,
        NumericRounding::Floor => negative && remainder.bits() != 0,
        NumericRounding::Ceiling => !negative && remainder.bits() != 0,
        NumericRounding::NearestEven
        | NumericRounding::NearestAway
        | NumericRounding::NearestTowardZero => {
            let twice = remainder << 1_usize;
            twice > *denominator
                || (twice == *denominator
                    && match rounding {
                        NumericRounding::NearestEven => quotient.bit(0),
                        NumericRounding::NearestAway => true,
                        _ => false,
                    })
        }
    };
    if increment {
        quotient += 1_u8;
    }
    quotient
}

pub(crate) fn format_finite(value: f64, format: &ResolvedFormat<'_>) -> Result<String> {
    format.validate()?;
    ExactDecimal::decode(value, format.decimal_scale)?
        .multiply(format.multiplier)?
        .round(format)?
        .emit(format)
}

impl RoundedDecimal {
    /// Serialize an already rounded number. Scale has already been consumed by
    /// its mathematical operand, and must not be applied again at this boundary.
    pub(crate) fn emit(&self, format: &ResolvedFormat<'_>) -> Result<String> {
        self.emit_reserved(format, DecimalBuffer::default())
    }

    /// A power operand reserves its symbolic output bound before any refinement
    /// or physical coefficient construction. Reuse that same empty buffer here.
    pub(crate) fn emit_reserved(
        &self,
        format: &ResolvedFormat<'_>,
        mut buffer: DecimalBuffer,
    ) -> Result<String> {
        format.validate()?;
        if !buffer.text.is_empty() {
            return Err(Error::Render(
                "numeric output reservation must be empty".into(),
            ));
        }
        buffer.convert(&self.coefficient)?;
        let digits = buffer.digit_count()?;
        let zero = self.coefficient.bits() == 0;
        let negative = self.negative && (!zero || format.negative_zero == NegativeZero::Preserve);
        let decade = if zero {
            BigInt::ZERO
        } else {
            &self.exponent + BigInt::from(digits - 1)
        };
        let display_exponent = match format.notation {
            NumericNotation::Decimal => BigInt::ZERO,
            NumericNotation::Scientific => decade.clone(),
            NumericNotation::Engineering => engineering_exponent(&decade),
        };
        let point = if zero {
            BigInt::from(1_u8)
        } else {
            BigInt::from(digits) + &self.exponent - &display_exponent
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
        let exponent = format.notation != NumericNotation::Decimal;
        let exponent_length = if exponent {
            // 2^3 < 10; this is a checked capacity bound, not spelling.
            let digits = usize::try_from(display_exponent.bits().div_ceil(3).max(1))
                .map_err(|_| arithmetic_error())?;
            digits
                .checked_add(1 + usize::from(display_exponent.sign() == Sign::Minus))
                .ok_or_else(arithmetic_error)?
        } else {
            0
        };
        buffer.reserve_chunks(display_exponent.bits())?;
        let prefix = format.prefix.unwrap_or_default();
        let suffix = format.suffix.unwrap_or_default();
        let length = [
            prefix.len(),
            usize::from(negative),
            integral,
            usize::from(fractional > 0),
            fractional,
            exponent_length,
            suffix.len(),
        ]
        .into_iter()
        .try_fold(0_usize, usize::checked_add)
        .filter(|length| isize::try_from(*length).is_ok())
        .ok_or_else(arithmetic_error)?;
        buffer
            .text
            .try_reserve_exact(length)
            .map_err(|error| Error::Render(format!("numeric presentation output: {error}")))?;
        let text = &mut buffer.text;
        text.push_str(prefix);
        if negative {
            text.push('-');
        }
        emit_magnitude(text, &buffer.chunks, digits, point, fractional)?;
        if fractional > 0 && format.trailing_zeros == TrailingZeros::Trim {
            while text.ends_with('0') {
                text.pop();
            }
            if text.ends_with('.') {
                text.pop();
            }
        }
        if exponent {
            buffer.append_exponent(&display_exponent)?;
        }
        buffer.text.push_str(suffix);
        Ok(buffer.text)
    }
}

fn emit_magnitude(
    text: &mut String,
    chunks: &[u32],
    digits: usize,
    point: isize,
    fractional: usize,
) -> Result<()> {
    let integral_digits = usize::try_from(point.max(0)).map_err(|_| arithmetic_error())?;
    let taken = integral_digits.min(digits);
    if integral_digits == 0 {
        text.push('0');
    } else {
        visit_digits(chunks, 0..taken, |digits| {
            text.push_str(std::str::from_utf8(digits).expect("ASCII decimal digits"));
        });
        push_zeroes(text, integral_digits - taken);
    }
    let leading = if point < 0 { point.unsigned_abs() } else { 0 };
    let leading = fractional.min(leading);
    let copied = (fractional - leading).min(digits - taken);
    if fractional > 0 {
        text.push('.');
        push_zeroes(text, leading);
        visit_digits(chunks, taken..taken + copied, |digits| {
            text.push_str(std::str::from_utf8(digits).expect("ASCII decimal digits"));
        });
        push_zeroes(text, fractional - leading - copied);
    }
    let mut discarded_nonzero = false;
    visit_digits(chunks, taken + copied..digits, |digits| {
        discarded_nonzero |= digits.iter().any(|digit| *digit != b'0');
    });
    if discarded_nonzero {
        return Err(Error::Render(
            "numeric decimal emission would discard nonzero digits".into(),
        ));
    }
    Ok(())
}

fn chunk_width(value: u32) -> usize {
    if value == 0 {
        1
    } else {
        value.ilog10() as usize + 1
    }
}

/// View ordinary decimal digit spans without allocating a reversed digit string.
fn visit_digits(chunks: &[u32], range: std::ops::Range<usize>, mut visit: impl FnMut(&[u8])) {
    if range.is_empty() {
        return;
    }
    let mut offset = 0;
    for (index, &chunk) in chunks.iter().rev().enumerate() {
        let width = if index == 0 { chunk_width(chunk) } else { 9 };
        let end = offset + width;
        if range.start < end && range.end > offset {
            let mut bytes = [b'0'; 9];
            let mut value = chunk;
            for byte in bytes.iter_mut().rev() {
                *byte += u8::try_from(value % 10).expect("decimal digit");
                value /= 10;
            }
            let start = range.start.max(offset) - offset;
            let finish = range.end.min(end) - offset;
            visit(&bytes[9 - width + start..9 - width + finish]);
        }
        offset = end;
        if offset >= range.end {
            break;
        }
    }
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
