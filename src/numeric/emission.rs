//! Presentation of the exact engine's private ASCII digit runs, never caller text.

use super::decimal::{NumericNotation, NumericRadix};
use super::locale::{validate_token, NumericGroupRepeat, NumericGrouping, NumericLocale};
use crate::{Error, Result};

/// Positive and negative sign presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericSign {
    /// Only negative values have a sign.
    NegativeOnly,
    /// Emit a positive or negative sign for every value.
    Always,
    /// Positive values have one ordinary space.
    Space,
    /// Negative values are enclosed in parentheses.
    Accounting,
}

/// A locale currency or integer-radix affix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericSymbol {
    /// No symbol.
    None,
    /// Use the locale's currency prefix and suffix.
    Currency,
    /// Emit 0b, 0o or 0x for a nondecimal integer notation.
    RadixPrefix,
}

/// Placement of field fill around a complete numeric presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericAlignment {
    /// Fill after the number.
    Left,
    /// Fill before the number.
    Right,
    /// Fill on both sides, with an odd extra scalar on the right.
    Center,
    /// Fill after sign and currency/radix prefix.
    AfterSign,
}

/// Minimum scalar width, independent of CSS geometry and numeric precision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericField {
    pub(super) width: usize,
    pub(super) fill: char,
    pub(super) alignment: NumericAlignment,
}

impl NumericField {
    /// Construct a field with one single-line fill scalar; zero width adds no fill.
    pub fn new(width: usize, fill: char, alignment: NumericAlignment) -> Result<Self> {
        let mut bytes = [0; 4];
        validate_token(fill.encode_utf8(&mut bytes), false)?;
        Ok(Self {
            width,
            fill,
            alignment,
        })
    }
}

/// Exact rate scaling, independent of a literal suffix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericRate {
    /// No rate scaling.
    Value,
    /// Multiply exactly by 100 and use the locale percent suffix.
    Percent,
    /// Multiply exactly by 1000 and use the locale per-mille suffix.
    PerMille,
}

impl NumericRate {
    pub(super) const fn shift(self) -> i16 {
        match self {
            Self::Value => 0,
            Self::Percent => 2,
            Self::PerMille => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Presentation {
    pub(super) locale: NumericLocale,
    pub(super) grouping: bool,
    pub(super) sign: NumericSign,
    pub(super) symbol: NumericSymbol,
    pub(super) field: Option<NumericField>,
}

#[derive(Clone, Copy)]
pub(super) struct Affixes<'a> {
    pub(super) outer_prefix: &'a str,
    pub(super) outer_suffix: &'a str,
    pub(super) prefix: &'a str,
    pub(super) suffix: &'a str,
}

impl Default for Presentation {
    fn default() -> Self {
        Self {
            locale: NumericLocale::invariant(),
            grouping: false,
            sign: NumericSign::NegativeOnly,
            symbol: NumericSymbol::None,
            field: None,
        }
    }
}

pub(super) fn append(out: &mut String, text: &str) -> Result<()> {
    out.try_reserve(text.len())
        .map_err(|e| Error::Render(format!("numeric text allocation: {e}")))?;
    out.push_str(text);
    Ok(())
}

fn fill(out: &mut String, scalar: char, count: usize) -> Result<()> {
    let bytes = count
        .checked_mul(scalar.len_utf8())
        .ok_or_else(length_error)?;
    out.try_reserve_exact(bytes)
        .map_err(|e| Error::Render(format!("numeric field allocation: {e}")))?;
    out.extend(std::iter::repeat_n(scalar, count));
    Ok(())
}

fn length_error() -> Error {
    Error::Render("numeric field or token length overflow".into())
}

fn digit(out: &mut String, byte: u8, locale: &NumericLocale) -> Result<()> {
    if byte.is_ascii_digit() {
        append(out, &locale.digits()[usize::from(byte - b'0')])
    } else {
        let mut bytes = [0; 4];
        append(out, char::from(byte).encode_utf8(&mut bytes))
    }
}

fn separators(count: usize, grouping: &NumericGrouping) -> Result<usize> {
    if count == 0 {
        return Ok(0);
    }
    let mut remaining = count;
    let mut groups = 0_usize;
    for &size in grouping.sizes() {
        if remaining <= size {
            return Ok(groups);
        }
        remaining -= size;
        groups = groups.checked_add(1).ok_or_else(length_error)?;
    }
    match grouping.repeat() {
        NumericGroupRepeat::Stop => Ok(groups),
        NumericGroupRepeat::Last => groups
            .checked_add((remaining - 1) / grouping.sizes()[grouping.sizes().len() - 1])
            .ok_or_else(length_error),
        NumericGroupRepeat::Cycle => {
            let cycle = grouping
                .sizes()
                .iter()
                .try_fold(0_usize, |n, &s| n.checked_add(s))
                .ok_or_else(length_error)?;
            let full = (remaining - 1) / cycle;
            groups = groups
                .checked_add(
                    full.checked_mul(grouping.sizes().len())
                        .ok_or_else(length_error)?,
                )
                .ok_or_else(length_error)?;
            remaining -= full * cycle;
            for &size in grouping.sizes() {
                if remaining <= size {
                    break;
                }
                remaining -= size;
                groups = groups.checked_add(1).ok_or_else(length_error)?;
            }
            Ok(groups)
        }
    }
}

fn integer_scalars(
    integer: &str,
    zeros: usize,
    locale: &NumericLocale,
    grouping: Option<&NumericGrouping>,
) -> Result<usize> {
    let mut count = zeros
        .checked_mul(locale.digits()[0].chars().count())
        .ok_or_else(length_error)?;
    for byte in integer.bytes() {
        count = count
            .checked_add(if byte.is_ascii_digit() {
                locale.digits()[usize::from(byte - b'0')].chars().count()
            } else {
                1
            })
            .ok_or_else(length_error)?;
    }
    if let Some(grouping) = grouping {
        count = count
            .checked_add(
                separators(
                    integer.len().checked_add(zeros).ok_or_else(length_error)?,
                    grouping,
                )?
                .checked_mul(grouping.separator().chars().count())
                .ok_or_else(length_error)?,
            )
            .ok_or_else(length_error)?;
    }
    Ok(count)
}

fn integer_text(
    integer: &str,
    zeros: usize,
    locale: &NumericLocale,
    grouping: Option<&NumericGrouping>,
) -> Result<String> {
    let total = integer.len().checked_add(zeros).ok_or_else(length_error)?;
    let mut cuts = Vec::new();
    let count = grouping.map_or(Ok(0), |g| separators(total, g))?;
    crate::data::source::reserve_source(&mut cuts, count)?;
    let mut remaining = total;
    if let Some(grouping) = grouping {
        for index in 0..count {
            remaining -= grouping.size(index).ok_or_else(length_error)?;
            cuts.push(remaining);
        }
    }
    let mut output = String::new();
    // Reserve before visiting a possibly large zero field, including multibyte tokens.
    let max_digit = locale.digits().iter().map(String::len).max().unwrap_or(1);
    let bytes = total
        .checked_mul(max_digit)
        .and_then(|n| {
            n.checked_add(count.checked_mul(grouping.map_or(0, |g| g.separator().len()))?)
        })
        .ok_or_else(length_error)?;
    output
        .try_reserve_exact(bytes)
        .map_err(|e| Error::Render(format!("numeric integer tokens: {e}")))?;
    for index in 0..total {
        if cuts.last() == Some(&index) {
            append(&mut output, grouping.expect("grouped cut").separator())?;
            cuts.pop();
        }
        digit(
            &mut output,
            if index < zeros {
                b'0'
            } else {
                integer.as_bytes()[index - zeros]
            },
            locale,
        )?;
    }
    Ok(output)
}

impl Presentation {
    pub(super) fn observe(&self, round: &mut crate::layout::kernel::FeedbackRound) -> Result<()> {
        self.locale.observe(round)?;
        round.observe_count(usize::from(self.grouping))?;
        round.observe_count(self.sign as usize)?;
        round.observe_count(self.symbol as usize)?;
        round.observe_count(usize::from(self.field.is_some()))?;
        if let Some(field) = &self.field {
            round.observe_count(field.width)?;
            round.observe_count(field.fill as usize)?;
            round.observe_count(field.alignment as usize)?;
        }
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one normalized emission order owns sign, grouping, locale tokens and final scalar padding"
    )]
    pub(super) fn emit(
        &self,
        ascii: &str,
        notation: NumericNotation,
        rate: NumericRate,
        affixes: Affixes<'_>,
    ) -> Result<String> {
        let negative = ascii.starts_with('-');
        let magnitude = ascii.strip_prefix('-').unwrap_or(ascii);
        let (mantissa, exponent) = if matches!(notation, NumericNotation::Integer(_)) {
            (magnitude, None)
        } else {
            magnitude
                .split_once('e')
                .map_or((magnitude, None), |(m, e)| (m, Some(e)))
        };
        let (integer, fraction) = mantissa
            .split_once('.')
            .map_or((mantissa, None), |(i, f)| (i, Some(f)));
        let mut leading = String::new();
        append(&mut leading, affixes.outer_prefix)?;
        append(&mut leading, affixes.prefix)?;
        append(
            &mut leading,
            match (negative, self.sign) {
                (true, NumericSign::Accounting) => "(",
                (true, _) => self.locale.symbols().negative(),
                (false, NumericSign::Always) => self.locale.symbols().positive(),
                (false, NumericSign::Space) => " ",
                _ => "",
            },
        )?;
        let radix = match (self.symbol, notation) {
            (NumericSymbol::RadixPrefix, NumericNotation::Integer(NumericRadix::Binary)) => "0b",
            (NumericSymbol::RadixPrefix, NumericNotation::Integer(NumericRadix::Octal)) => "0o",
            (
                NumericSymbol::RadixPrefix,
                NumericNotation::Integer(NumericRadix::HexLower | NumericRadix::HexUpper),
            ) => "0x",
            _ => "",
        };
        append(
            &mut leading,
            if self.symbol == NumericSymbol::Currency {
                self.locale.currency().prefix()
            } else {
                radix
            },
        )?;
        let mut trailing = String::new();
        if let Some(fraction) = fraction {
            append(&mut trailing, self.locale.decimal())?;
            for byte in fraction.bytes() {
                digit(&mut trailing, byte, &self.locale)?;
            }
        }
        if let Some(exponent) = exponent {
            append(&mut trailing, "e")?;
            for byte in exponent.bytes() {
                if byte == b'-' {
                    append(&mut trailing, self.locale.symbols().negative())?;
                } else {
                    digit(&mut trailing, byte, &self.locale)?;
                }
            }
        }
        append(
            &mut trailing,
            if self.symbol == NumericSymbol::Currency {
                self.locale.currency().suffix()
            } else {
                match rate {
                    NumericRate::Value => "",
                    NumericRate::Percent => self.locale.symbols().percent(),
                    NumericRate::PerMille => self.locale.symbols().per_mille(),
                }
            },
        )?;
        if negative && self.sign == NumericSign::Accounting {
            append(&mut trailing, ")")?;
        }
        append(&mut trailing, affixes.suffix)?;
        append(&mut trailing, affixes.outer_suffix)?;
        let grouping = self.grouping.then(|| self.locale.grouping()).flatten();
        let fixed = leading
            .chars()
            .count()
            .checked_add(trailing.chars().count())
            .ok_or_else(length_error)?;
        let mut zeros = 0;
        if let Some(field) = &self.field {
            if field.fill == '0' && field.alignment == NumericAlignment::AfterSign {
                let (mut low, mut high) = (0_usize, field.width);
                while low < high {
                    let middle = low + (high - low) / 2;
                    if fixed
                        .checked_add(integer_scalars(integer, middle, &self.locale, grouping)?)
                        .ok_or_else(length_error)?
                        < field.width
                    {
                        low = middle + 1;
                    } else {
                        high = middle;
                    }
                }
                zeros = low;
            }
        }
        let numeric = integer_text(integer, zeros, &self.locale, grouping)?;
        let length = fixed
            .checked_add(numeric.chars().count())
            .ok_or_else(length_error)?;
        let mut output = String::new();
        let padding = self
            .field
            .as_ref()
            .map_or(0, |field| field.width.saturating_sub(length));
        let (before, after, inside, scalar) =
            self.field
                .as_ref()
                .map_or((0, 0, 0, ' '), |field| match field.alignment {
                    NumericAlignment::Left => (0, padding, 0, field.fill),
                    NumericAlignment::Right => (padding, 0, 0, field.fill),
                    NumericAlignment::Center => (padding / 2, padding - padding / 2, 0, field.fill),
                    NumericAlignment::AfterSign => (0, 0, padding, field.fill),
                });
        fill(&mut output, scalar, before)?;
        append(&mut output, &leading)?;
        fill(&mut output, scalar, inside)?;
        append(&mut output, &numeric)?;
        append(&mut output, &trailing)?;
        fill(&mut output, scalar, after)?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        NumericCurrency, NumericFormat, NumericGroupRepeat, NumericPrecision, NumericRounding,
        NumericSymbols,
    };

    fn custom(grouping: Option<NumericGrouping>) -> NumericLocale {
        NumericLocale::custom(
            ".",
            std::array::from_fn(|i| i.to_string()),
            grouping,
            NumericSymbols::new("+", "-", "%", "‰").unwrap(),
            NumericCurrency::new("$", "").unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn grouping_continuations_keep_their_own_right_to_left_relationships() {
        for (repeat, expected) in [
            (NumericGroupRepeat::Cycle, "12,345,67,890"),
            (NumericGroupRepeat::Last, "1,23,45,67,890"),
            (NumericGroupRepeat::Stop, "12345,67,890"),
        ] {
            let locale = custom(Some(NumericGrouping::new(",", vec![3, 2], repeat).unwrap()));
            let format =
                NumericFormat::new(NumericNotation::Decimal, NumericPrecision::Fractional(0))
                    .unwrap()
                    .locale(locale)
                    .unwrap()
                    .grouping(true)
                    .unwrap();
            assert_eq!(
                format.format_singleton(1_234_567_890.0, false).unwrap(),
                expected
            );
        }
        let format = NumericFormat::default()
            .locale(custom(None))
            .unwrap()
            .grouping(true)
            .unwrap();
        assert_eq!(format.format_singleton(1234.0, false).unwrap(), "1234.00");
        assert!(NumericGrouping::new(",", vec![3, 0], NumericGroupRepeat::Last).is_err());
    }

    #[test]
    fn locale_signs_rates_currency_and_literal_affixes_follow_one_emission_order() {
        let format = NumericFormat::new(NumericNotation::Decimal, NumericPrecision::Fractional(1))
            .unwrap()
            .locale(NumericLocale::named("fr-FR").unwrap())
            .unwrap()
            .grouping(true)
            .unwrap()
            .rate(NumericRate::Percent)
            .unwrap()
            .symbol(NumericSymbol::Currency)
            .unwrap()
            .sign(NumericSign::Accounting)
            .unwrap()
            .prefix("[")
            .unwrap()
            .suffix(" 00]")
            .unwrap();
        assert_eq!(
            format.format_singleton(-12.5, false).unwrap(),
            "[(1\u{a0}250,0\u{a0}€) 00]"
        );
        let rate = format
            .clone()
            .symbol(NumericSymbol::None)
            .unwrap()
            .sign(NumericSign::Always)
            .unwrap();
        assert_eq!(
            rate.format_singleton(0.125, false).unwrap(),
            "[+12,5\u{202f}% 00]"
        );
        let per_mille = NumericFormat::default()
            .rate(NumericRate::PerMille)
            .unwrap();
        assert_eq!(per_mille.format_singleton(0.125, false).unwrap(), "125.00‰");
        assert_eq!(per_mille.format_singleton(0.0, true).unwrap(), "1000.00‰");
        let arabic = NumericFormat::default()
            .locale(NumericLocale::named("ar-001").unwrap())
            .unwrap()
            .notation(NumericNotation::Scientific)
            .unwrap()
            .prefix("ID 012: ")
            .unwrap();
        assert_eq!(
            arabic.format_singleton(-0.125, false).unwrap(),
            "ID 012: −١٫٢٥e−١"
        );
    }

    #[test]
    fn numeric_fields_count_final_scalars_and_do_not_split_numeral_tokens() {
        let field = NumericField::new(9, '0', NumericAlignment::AfterSign).unwrap();
        let digits = std::array::from_fn(|i| format!("d{i}"));
        let locale = NumericLocale::custom(
            ".",
            digits,
            None,
            NumericSymbols::new("+", "-", "%", "‰").unwrap(),
            NumericCurrency::new("", "").unwrap(),
        )
        .unwrap();
        let format = NumericFormat::new(NumericNotation::Decimal, NumericPrecision::Fractional(0))
            .unwrap()
            .locale(locale)
            .unwrap()
            .field(Some(field))
            .unwrap();
        assert_eq!(format.format_singleton(12.0, false).unwrap(), "d0d0d0d1d2");
        let centered = NumericFormat::default()
            .field(Some(
                NumericField::new(8, '·', NumericAlignment::Center).unwrap(),
            ))
            .unwrap();
        assert_eq!(centered.format_singleton(1.0, false).unwrap(), "··1.00··");
        let grouped = NumericFormat::new(NumericNotation::Decimal, NumericPrecision::Fractional(0))
            .unwrap()
            .grouping(true)
            .unwrap()
            .field(Some(
                NumericField::new(7, '0', NumericAlignment::AfterSign).unwrap(),
            ))
            .unwrap();
        assert_eq!(grouped.format_singleton(12.0, false).unwrap(), "000,012");
        let enormous = centered
            .field(Some(
                NumericField::new(usize::MAX, 'λ', NumericAlignment::Right).unwrap(),
            ))
            .unwrap();
        assert!(enormous.format_singleton(1.0, false).is_err());
        assert!(NumericField::new(1, '\n', NumericAlignment::Left).is_err());
    }

    #[test]
    fn general_and_integer_notations_use_the_once_rounded_mathematical_operand() {
        let general =
            NumericFormat::new(NumericNotation::General, NumericPrecision::Significant(3)).unwrap();
        assert_eq!(general.format_singleton(999.5, false).unwrap(), "1.00e3");
        assert_eq!(
            general.format_singleton(0.000_125, false).unwrap(),
            "0.000125"
        );
        assert_eq!(
            general.format_singleton(0.000_012_5, false).unwrap(),
            "1.25e-5"
        );
        let integer = NumericFormat::new(
            NumericNotation::Integer(NumericRadix::HexUpper),
            NumericPrecision::Fractional(0),
        )
        .unwrap()
        .symbol(NumericSymbol::RadixPrefix)
        .unwrap();
        assert_eq!(integer.format_singleton(255.5, false).unwrap(), "0x100");
        assert_eq!(
            integer
                .format_singleton(f64::from_bits((1023_u64 + 80) << 52), false)
                .unwrap(),
            "0x100000000000000000000"
        );
        let binary = NumericFormat::new(
            NumericNotation::Integer(NumericRadix::Binary),
            NumericPrecision::Fractional(0),
        )
        .unwrap()
        .rounding(NumericRounding::Floor)
        .unwrap();
        assert_eq!(binary.format_singleton(0.5, true).unwrap(), "11");
        assert!(binary.format_singleton(f64::MAX, true).is_err());
        assert!(
            NumericFormat::new(NumericNotation::General, NumericPrecision::Fractional(2)).is_err()
        );
        assert!(NumericFormat::default()
            .symbol(NumericSymbol::RadixPrefix)
            .is_err());
    }
}
