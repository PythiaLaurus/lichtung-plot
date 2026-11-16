//! D3 numeric syntax compiled into the sole normalized typed policy.

use super::decimal::{NumericNotation, NumericRadix, TrailingZeros};
use super::emission::{NumericAlignment, NumericField, NumericRate, NumericSign, NumericSymbol};
use super::format::{NumericFormat, NumericPrecision};
use super::prefixes::{NumericPrefixes, NumericPromotion, NumericSelection};
use crate::{Error, Result};

impl NumericFormat {
    /// Compile `[[fill]align][sign][symbol][0][width][,][.precision][~][type]`.
    /// Numeric types are f/e/g/r/s/%/p/d/b/o/x/X/n or omitted; unknown types and
    /// text coercion reject. Exact renderer rounding is not JavaScript arithmetic.
    pub fn from_specifier(specifier: &str) -> Result<Self> {
        parse(specifier)
    }
}

fn alignment(value: char) -> Option<NumericAlignment> {
    match value {
        '<' => Some(NumericAlignment::Left),
        '>' => Some(NumericAlignment::Right),
        '^' => Some(NumericAlignment::Center),
        '=' => Some(NumericAlignment::AfterSign),
        _ => None,
    }
}

fn take(source: &mut &str) -> Option<char> {
    let value = source.chars().next()?;
    *source = &source[value.len_utf8()..];
    Some(value)
}

fn digits<'a>(source: &mut &'a str) -> &'a str {
    let count = source.bytes().take_while(u8::is_ascii_digit).count();
    let (number, rest) = source.split_at(count);
    *source = rest;
    number
}

fn flag(source: &mut &str, value: char) -> bool {
    if source.starts_with(value) {
        take(source);
        true
    } else {
        false
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "the pinned grammar and its explicit irrelevant-field normalizations compile once into typed edits"
)]
fn parse(mut source: &str) -> Result<NumericFormat> {
    let (mut fill, mut align) = (' ', NumericAlignment::Right);
    if let Some(value) = source.chars().nth(1).and_then(alignment) {
        fill = take(&mut source).expect("fill before alignment");
        take(&mut source);
        align = value;
    } else if let Some(value) = source.chars().next().and_then(alignment) {
        take(&mut source);
        align = value;
    }
    let sign = match source.chars().next() {
        Some('-') => Some(NumericSign::NegativeOnly),
        Some('+') => Some(NumericSign::Always),
        Some(' ') => Some(NumericSign::Space),
        Some('(') => Some(NumericSign::Accounting),
        _ => None,
    };
    if sign.is_some() {
        take(&mut source);
    }
    let symbol = match source.chars().next() {
        Some('$') => Some(NumericSymbol::Currency),
        Some('#') => Some(NumericSymbol::RadixPrefix),
        _ => None,
    };
    if symbol.is_some() {
        take(&mut source);
    }
    if flag(&mut source, '0') || (fill == '0' && align == NumericAlignment::AfterSign) {
        fill = '0';
        align = NumericAlignment::AfterSign;
    }
    let width = digits(&mut source);
    let width = if width.is_empty() {
        None
    } else {
        Some(width.parse::<usize>().map_err(|_| invalid())?)
    };
    let grouping = flag(&mut source, ',');
    let precision = if flag(&mut source, '.') {
        let precision = digits(&mut source);
        if precision.is_empty() {
            return Err(invalid());
        }
        Some(precision)
    } else {
        None
    };
    let trim = flag(&mut source, '~');
    let kind = take(&mut source);
    if !source.is_empty() {
        return Err(invalid());
    }
    let notation = match kind {
        Some('f' | 'r' | 's' | '%' | 'p') => NumericNotation::Decimal,
        Some('e') => NumericNotation::Scientific,
        Some('g' | 'n') | None => NumericNotation::General,
        Some('d') => NumericNotation::Integer(NumericRadix::Decimal),
        Some('b') => NumericNotation::Integer(NumericRadix::Binary),
        Some('o') => NumericNotation::Integer(NumericRadix::Octal),
        Some('x') => NumericNotation::Integer(NumericRadix::HexLower),
        Some('X') => NumericNotation::Integer(NumericRadix::HexUpper),
        _ => return Err(invalid()),
    };
    let precision = if matches!(notation, NumericNotation::Integer(_)) {
        // The reference integer grammar discards precision, including values
        // larger than u8/usize. Only the decimal syntax above matters here.
        NumericPrecision::Fractional(0)
    } else {
        let count = precision.map_or(Ok(if kind.is_none() { 12 } else { 6 }), |value| {
            value.parse::<u8>().map_err(|_| invalid())
        })?;
        if matches!(kind, Some('f' | 'e' | '%')) {
            NumericPrecision::Fractional(count)
        } else {
            NumericPrecision::Significant(count)
        }
    };
    let symbol = match (symbol, notation) {
        (Some(NumericSymbol::RadixPrefix), NumericNotation::Integer(radix))
            if radix != NumericRadix::Decimal =>
        {
            NumericSymbol::RadixPrefix
        }
        (Some(NumericSymbol::Currency), _) => NumericSymbol::Currency,
        _ => NumericSymbol::None,
    };
    let field = NumericField::new(width.unwrap_or(0), fill, align)?;
    let mut format = NumericFormat::new(notation, precision)?
        .sign(sign.unwrap_or(NumericSign::NegativeOnly))?
        .symbol(symbol)?
        .grouping(grouping || kind == Some('n'))?
        .field(width.map(|_| field))?
        .trailing_zeros(if trim || kind.is_none() {
            TrailingZeros::Trim
        } else {
            TrailingZeros::Keep
        })?;
    if matches!(kind, Some('%' | 'p')) {
        format = format.rate(NumericRate::Percent)?;
    }
    if kind == Some('s') {
        format = format.prefixes(NumericPrefixes::si(
            NumericSelection::PerValue,
            NumericPromotion::PromoteOnRounding,
        )?)?;
    }
    Ok(format)
}

fn invalid() -> Error {
    Error::Data("invalid numeric format specifier".into())
}
