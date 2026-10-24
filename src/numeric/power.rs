//! Exact mathematical 10^(binary64 exponent + decimal scale) operands.
//!
//! Keep E=n+F/2^d with a signed arbitrary-size floor n and 0<=F<2^d.
//! Integral powers are symbolic and use the common decimal emitter. Fractional
//! operands retain their complete dyadic remainder for separate certification.

use num_bigint::{BigInt, BigUint, Sign};

use super::decimal::{
    DecimalBuffer, NumericNotation, NumericRounding, ResolvedFormat, ResolvedPrecision,
    RoundedDecimal,
};
use crate::{Error, Result};

pub(crate) mod certified;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Log10Operand {
    pub(crate) exponent_bits: u64,
    pub(crate) decimal_scale: i16,
    pub(crate) decade: BigInt,
    pub(crate) fraction: BigUint,
    pub(crate) denominator_bits: u32,
}

/// Quantum/operand context is semantic; the empty output buffer is transient
/// allocation state, never an adaptive-set/cache identity or numerical result.
#[derive(Debug)]
pub(crate) struct PowerPlan {
    pub(crate) quantum: BigInt,
    pub(crate) multiplier: BigInt,
    pub(crate) tiny: bool,
    pub(crate) output: DecimalBuffer,
}

impl Log10Operand {
    pub(crate) fn new(exponent: f64, decimal_scale: i16) -> Result<Self> {
        if !exponent.is_finite() || !(-324..=324).contains(&decimal_scale) {
            return Err(Error::Data(
                "Log10 presentation requires a finite exponent and decimal scale in -324..=324"
                    .into(),
            ));
        }
        let bits = exponent.to_bits();
        let stored = i32::try_from((bits >> 52) & 0x7ff).expect("11 exponent bits");
        let mut significand = bits & ((1_u64 << 52) - 1);
        let mut shift = if stored == 0 {
            -1074
        } else {
            significand |= 1_u64 << 52;
            stored - 1023 - 52
        };
        if significand == 0 {
            return Ok(Self {
                exponent_bits: 0,
                decimal_scale,
                decade: decimal_scale.into(),
                fraction: BigUint::ZERO,
                denominator_bits: 0,
            });
        }
        let common = significand.trailing_zeros();
        significand >>= common;
        shift += i32::try_from(common).expect("at most 52 common bits");
        let sign = if exponent.is_sign_negative() {
            Sign::Minus
        } else {
            Sign::Plus
        };
        let numerator = BigInt::from_biguint(sign, significand.into());
        let (decade, fraction, denominator_bits) = if shift >= 0 {
            (
                (numerator << shift.unsigned_abs() as usize) + decimal_scale,
                BigUint::ZERO,
                0,
            )
        } else {
            let denominator_bits = shift.unsigned_abs();
            let denominator = BigInt::from(1_u8) << denominator_bits as usize;
            let scaled = numerator + &denominator * decimal_scale;
            let mut decade = &scaled / &denominator;
            let mut remainder = scaled % &denominator;
            if remainder.sign() == Sign::Minus {
                decade -= 1_u8;
                remainder += denominator;
            }
            (
                decade,
                BigUint::try_from(remainder).expect("Euclidean nonnegative remainder"),
                denominator_bits,
            )
        };
        Ok(Self {
            exponent_bits: bits,
            decimal_scale,
            decade,
            fraction,
            denominator_bits,
        })
    }

    pub(crate) fn plan(&self, format: &ResolvedFormat<'_>) -> Result<PowerPlan> {
        format.validate()?;
        if format.decimal_scale != self.decimal_scale {
            return Err(Error::Render(
                "Log10 operand decimal scale changed after reduction".into(),
            ));
        }
        let quantum = match format.precision {
            ResolvedPrecision::Significant(digits) => &self.decade - (digits - 1),
            ResolvedPrecision::Fractional(digits) => {
                let origin = match format.notation {
                    NumericNotation::Decimal => BigInt::ZERO,
                    NumericNotation::Scientific => self.decade.clone(),
                    NumericNotation::Engineering => {
                        let mut remainder = &self.decade % 3_u8;
                        if remainder.sign() == Sign::Minus {
                            remainder += 3_u8;
                        }
                        &self.decade - remainder
                    }
                };
                origin - digits
            }
        };
        let multiplier = &self.decade - &quantum;
        let tiny = format.notation == NumericNotation::Decimal
            && matches!(format.precision, ResolvedPrecision::Fractional(_))
            && multiplier < BigInt::from(if self.fraction.bits() == 0 { 0_i8 } else { -1 });
        let body = if format.notation == NumericNotation::Decimal {
            let integral = std::cmp::max(BigInt::from(1_u8), &self.decade + 2_u8);
            match format.precision {
                ResolvedPrecision::Fractional(digits) => {
                    let integral = if tiny { BigInt::from(1_u8) } else { integral };
                    integral
                        + if digits == 0 {
                            0
                        } else {
                            u16::from(digits) + 1
                        }
                }
                ResolvedPrecision::Significant(digits) => {
                    if self.decade < BigInt::from(digits - 1) {
                        integral + 1_u8 + (BigInt::from(digits - 1) - &self.decade)
                    } else {
                        integral
                    }
                }
            }
        } else {
            // Spec07: <=3 integral, point, selected fractional digits, e, optional minus,
            // <=ceil(1025/3) exponent digits. A capacity bound, not a length cap.
            let fractional = match format.precision {
                ResolvedPrecision::Fractional(digits) => digits,
                ResolvedPrecision::Significant(digits) => digits - 1,
            };
            BigInt::from(348_u16 + u16::from(fractional))
        };
        let length = body + format.prefix.map_or(0, str::len) + format.suffix.map_or(0, str::len);
        let capacity = usize::try_from(length)
            .ok()
            .filter(|length| isize::try_from(*length).is_ok())
            .ok_or_else(|| {
                Error::Render("Log10 presentation output length is not representable".into())
            })?;
        let integer_bits = if self.fraction.bits() == 0 || tiny || multiplier.sign() == Sign::Minus
        {
            1025
        } else {
            usize::try_from(&multiplier)
                .ok()
                .and_then(|power| power.checked_mul(4))
                .and_then(|bits| bits.checked_add(5))
                .map(|bits| bits.max(1025))
                .ok_or_else(|| {
                    Error::Render("Log10 coefficient bit length is not representable".into())
                })?
        };
        let output = DecimalBuffer::reserve(capacity, integer_bits)?;
        Ok(PowerPlan {
            quantum,
            multiplier,
            tiny,
            output,
        })
    }

    pub(super) fn symbolic(
        &self,
        plan: &PowerPlan,
        rounding: NumericRounding,
    ) -> Option<RoundedDecimal> {
        (self.fraction.bits() == 0 || plan.tiny).then(|| RoundedDecimal {
            negative: false,
            coefficient: BigUint::from(u8::from(
                !plan.tiny
                    || matches!(
                        rounding,
                        NumericRounding::AwayFromZero | NumericRounding::Ceiling
                    ),
            )),
            exponent: if plan.tiny {
                plan.quantum.clone()
            } else {
                self.decade.clone()
            },
        })
    }
}

#[cfg(test)]
mod tests;
