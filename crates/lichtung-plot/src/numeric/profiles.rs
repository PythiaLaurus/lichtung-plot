//! Prescribed omitted-policy grammars of the one decimal engine. Compact
//! operands retain their historical rounded binary64 division before conversion.

use super::decimal::{
    format_finite, NegativeZero, NumericNotation, ResolvedFormat, ResolvedPrecision, TrailingZeros,
};
use crate::Result;

pub(crate) fn format_operand(
    value: &super::operand::Operand,
    integral: bool,
    precision: usize,
    formatter: &mut super::power::certified::PowerFormatter,
) -> Result<String> {
    let digits =
        (precision != 0).then(|| u8::try_from(precision).expect("bounded source precision"));
    match value {
        super::operand::Operand::Binary64(bits) => {
            BuiltinProfile::Automatic(digits).format(f64::from_bits(*bits))
        }
        super::operand::Operand::PowerOfTen(bits) => if integral {
            BuiltinProfile::ForestIntegral
        } else {
            BuiltinProfile::ForestExpression(digits)
        }
        .format(f64::from_bits(*bits)),
        super::operand::Operand::Multiplicative(_) => value.format(
            &ResolvedFormat {
                notation: NumericNotation::General,
                precision: ResolvedPrecision::Significant(digits.map_or(3, |digits| digits + 1)),
                trailing_zeros: TrailingZeros::Trim,
                negative_zero: NegativeZero::Normalize,
                ..ResolvedFormat::default()
            },
            formatter,
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuiltinProfile {
    Automatic(Option<u8>),
    ForestExpression(Option<u8>),
    ForestIntegral,
    WaterfallTotal,
}

impl BuiltinProfile {
    pub(crate) fn format(self, value: f64) -> Result<String> {
        match self {
            Self::Automatic(precision) => automatic(value, precision, None),
            Self::ForestExpression(precision) => automatic(value, precision, Some("10^")),
            Self::WaterfallTotal => format_finite(
                value,
                &ResolvedFormat {
                    precision: ResolvedPrecision::Fractional(3),
                    ..ResolvedFormat::default()
                },
            ),
            Self::ForestIntegral => {
                let value = if value == 0.0 { 0.0 } else { value };
                for (exponent, literal) in [
                    (-2, "0.01"),
                    (-1, "0.1"),
                    (0, "1"),
                    (1, "10"),
                    (2, "100"),
                    (3, "1000"),
                    (4, "10k"),
                    (5, "100k"),
                    (6, "1M"),
                    (7, "10M"),
                    (8, "100M"),
                    (9, "1000M"),
                    (10, "10000M"),
                    (11, "100000M"),
                ] {
                    if value.to_bits() == f64::from(exponent).to_bits() {
                        return crate::data::source::copy_source_string(literal);
                    }
                }
                format_finite(
                    value,
                    &ResolvedFormat {
                        precision: ResolvedPrecision::Fractional(0),
                        prefix: Some("1.00e"),
                        negative_zero: NegativeZero::Normalize,
                        ..ResolvedFormat::default()
                    },
                )
            }
        }
    }
}

fn automatic(value: f64, scientific: Option<u8>, prefix: Option<&str>) -> Result<String> {
    let mut format = ResolvedFormat {
        prefix,
        negative_zero: NegativeZero::Normalize,
        ..ResolvedFormat::default()
    };
    let absolute = value.abs();
    let operand = if let Some(precision) = scientific {
        format.notation = NumericNotation::Scientific;
        format.precision = ResolvedPrecision::Fractional(precision);
        value
    } else if absolute >= 1e12 || (absolute > 0.0 && absolute < 0.01) {
        format.notation = NumericNotation::Scientific;
        value
    } else {
        format.trailing_zeros = TrailingZeros::Trim;
        if absolute >= 1_000_000.0 {
            format.precision = ResolvedPrecision::Fractional(1);
            format.suffix = Some("M");
            value / 1_000_000.0
        } else if absolute >= 10_000.0 {
            format.precision = ResolvedPrecision::Fractional(1);
            format.suffix = Some("k");
            value / 1000.0
        } else if absolute >= 100.0 {
            format.precision = ResolvedPrecision::Fractional(0);
            format.trailing_zeros = TrailingZeros::Keep;
            value
        } else if absolute >= 1.0 {
            format.precision = ResolvedPrecision::Fractional(1);
            value
        } else {
            value
        }
    };
    format_finite(operand, &format)
}
