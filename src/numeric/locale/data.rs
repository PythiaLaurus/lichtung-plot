//! Transcribed from d3-format 3.1.0, commit
//! 08806c62aa8f7b3fe64925f3989f1491c950a71c, locale/*.json.
//! https://github.com/d3/d3-format/tree/08806c62aa8f7b3fe64925f3989f1491c950a71c/locale
//!
//! Copyright 2010-2021 Mike Bostock
//!
//! Permission to use, copy, modify, and/or distribute this software for any purpose
//! with or without fee is hereby granted, provided that the above copyright notice
//! and this permission notice appear in all copies.
//!
//! THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
//! REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND
//! FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
//! INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS
//! OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER
//! TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF
//! THIS SOFTWARE.

use super::{NumericCurrency, NumericGroupRepeat, NumericGrouping, NumericLocale, NumericSymbols};
use crate::{Error, Result};

// (ID, decimal, group separator, sizes, currency prefix/suffix, Arabic numerals, percent).
type Definition = (
    &'static str,
    &'static str,
    &'static str,
    &'static [usize],
    &'static str,
    &'static str,
    bool,
    &'static str,
);

macro_rules! catalog {
    ($(($id:literal, $decimal:literal, $separator:literal, $sizes:expr, $prefix:literal, $suffix:literal, $arabic:literal, $percent:literal)),* $(,)?) => {
        pub(super) const IDS: &[&str] = &[$($id),*];
        const DATA: &[Definition] = &[$(($id, $decimal, $separator, $sizes, $prefix, $suffix, $arabic, $percent)),*];
    };
}

catalog![
    ("ar-001", "٫", "٬", &[3], "", "", true, "%"),
    ("ar-AE", "٫", "٬", &[3], "", " د.إ.", true, "%"),
    ("ar-BH", "٫", "٬", &[3], "", " د.ب.", true, "%"),
    ("ar-DJ", "٫", "٬", &[3], "\u{200f}Fdj ", "", true, "%"),
    ("ar-DZ", ",", ".", &[3], "د.ج. ", "", false, "%"),
    ("ar-EG", "٫", "٬", &[3], "", " ج.م.", true, "%"),
    ("ar-EH", ".", ",", &[3], "د.م. ", "", false, "%"),
    ("ar-ER", "٫", "٬", &[3], "Nfk ", "", true, "%"),
    ("ar-IL", "٫", "٬", &[3], "₪ ", "", true, "%"),
    ("ar-IQ", "٫", "٬", &[3], "", " د.ع.", true, "%"),
    ("ar-JO", "٫", "٬", &[3], "", " د.أ.", true, "%"),
    ("ar-KM", "٫", "٬", &[3], "", " ف.ج.ق.", true, "%"),
    ("ar-KW", "٫", "٬", &[3], "", " د.ك.", true, "%"),
    ("ar-LB", "٫", "٬", &[3], "", " ل.ل.", true, "%"),
    ("ar-LY", ",", ".", &[3], "د.ل. ", "", false, "%"),
    ("ar-MA", ",", ".", &[3], "د.م. ", "", false, "%"),
    ("ar-MR", "٫", "٬", &[3], "", " أ.م.", true, "%"),
    ("ar-OM", "٫", "٬", &[3], "", " ر.ع.", true, "%"),
    ("ar-PS", "٫", "٬", &[3], "₪ ", "", true, "%"),
    ("ar-QA", "٫", "٬", &[3], "", " ر.ق.", true, "%"),
    ("ar-SA", "٫", "٬", &[3], "", " ر.س.", true, "%"),
    ("ar-SD", "٫", "٬", &[3], "", " ج.س.", true, "%"),
    ("ar-SO", "٫", "٬", &[3], "\u{200f}S ", "", true, "%"),
    ("ar-SS", "٫", "٬", &[3], "£ ", "", true, "%"),
    ("ar-SY", "٫", "٬", &[3], "", " ل.س.", true, "%"),
    ("ar-TD", "٫", "٬", &[3], "\u{200f}FCFA ", "", true, "%"),
    ("ar-TN", ",", ".", &[3], "د.ت. ", "", false, "%"),
    ("ar-YE", "٫", "٬", &[3], "", " ر.ى.", true, "%"),
    ("ca-ES", ",", ".", &[3], "", "\u{a0}€", false, "%"),
    ("cs-CZ", ",", "\u{a0}", &[3], "", "\u{a0}Kč", false, "%"),
    ("da-DK", ",", ".", &[3], "", " kr", false, "%"),
    ("de-CH", ",", "'", &[3], "", "\u{a0}CHF", false, "%"),
    ("de-DE", ",", ".", &[3], "", "\u{a0}€", false, "%"),
    ("en-CA", ".", ",", &[3], "$", "", false, "%"),
    ("en-GB", ".", ",", &[3], "£", "", false, "%"),
    ("en-IE", ".", ",", &[3], "€", "", false, "%"),
    (
        "en-IN",
        ".",
        ",",
        &[3, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        "₹",
        "",
        false,
        "%"
    ),
    ("en-US", ".", ",", &[3], "$", "", false, "%"),
    ("es-BO", ",", ".", &[3], "Bs\u{a0}", "", false, "\u{202f}%"),
    ("es-ES", ",", ".", &[3], "", "\u{a0}€", false, "%"),
    ("es-MX", ".", ",", &[3], "$", "", false, "%"),
    ("fi-FI", ",", "\u{a0}", &[3], "", "\u{a0}€", false, "%"),
    ("fr-CA", ",", "\u{a0}", &[3], "", "$", false, "%"),
    (
        "fr-FR",
        ",",
        "\u{a0}",
        &[3],
        "",
        "\u{a0}€",
        false,
        "\u{202f}%"
    ),
    ("he-IL", ".", ",", &[3], "₪", "", false, "%"),
    ("hu-HU", ",", "\u{a0}", &[3], "", "\u{a0}Ft", false, "%"),
    ("it-IT", ",", ".", &[3], "€", "", false, "%"),
    ("ja-JP", ".", ",", &[3], "", "円", false, "%"),
    ("ko-KR", ".", ",", &[3], "₩", "", false, "%"),
    ("mk-MK", ",", ".", &[3], "", "\u{a0}ден.", false, "%"),
    ("nl-NL", ",", ".", &[3], "€\u{a0}", "", false, "%"),
    ("pl-PL", ",", ".", &[3], "", "zł", false, "%"),
    ("pt-BR", ",", ".", &[3], "R$", "", false, "%"),
    ("pt-PT", ",", "\u{a0}", &[3], "", "\u{a0}€", false, "%"),
    ("ru-RU", ",", "\u{a0}", &[3], "", "\u{a0}₽", false, "%"),
    ("sl-SI", ",", ".", &[3], "", "\u{a0}€", false, "%"),
    ("sv-SE", ",", "\u{a0}", &[3], "", " kr", false, "%"),
    ("uk-UA", ",", "\u{a0}", &[3], "", "\u{a0}₴.", false, "%"),
    ("zh-CN", ".", ",", &[3], "¥", "", false, "%"),
];

pub(super) fn named(id: &str) -> Result<NumericLocale> {
    let index = IDS
        .binary_search(&id)
        .map_err(|_| Error::Data(format!("unknown numeric locale '{id}'")))?;
    let (_, decimal, separator, sizes, prefix, suffix, arabic, percent) = DATA[index];
    let digits = if arabic {
        "٠١٢٣٤٥٦٧٨٩"
    } else {
        "0123456789"
    };
    let mut iter = digits.chars();
    NumericLocale::custom(
        decimal,
        std::array::from_fn(|_| iter.next().expect("ten catalog digits").to_string()),
        Some(NumericGrouping::new(
            separator,
            crate::data::source::collect_source(sizes.iter().copied().map(Ok))?,
            NumericGroupRepeat::Cycle,
        )?),
        NumericSymbols::new("+", "−", percent, "‰")?,
        NumericCurrency::new(prefix, suffix)?,
    )
}
