//! Explicit immutable numeric tokens; no host locale or runtime data loading.

use std::sync::Arc;

use crate::{Error, Result};

mod data;

pub(super) fn validate_token(value: &str, empty: bool) -> Result<()> {
    if (!empty && value.is_empty())
        || value
            .chars()
            .any(|c| c.is_control() || crate::text::is_nonrendering_mandatory_break(c))
    {
        return Err(Error::Data(
            "numeric token must have valid single-line content".into(),
        ));
    }
    Ok(())
}

/// How a positive grouping-size sequence continues after its last entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericGroupRepeat {
    /// Restart the entire sequence.
    Cycle,
    /// Repeat the last size.
    Last,
    /// Leave the remaining leading digits as one group.
    Stop,
}

/// Right-to-left integer grouping with literal separators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericGrouping {
    separator: String,
    sizes: Vec<usize>,
    repeat: NumericGroupRepeat,
}

impl NumericGrouping {
    /// Construct positive group sizes and a nonempty single-line separator.
    pub fn new(
        separator: impl Into<String>,
        sizes: Vec<usize>,
        repeat: NumericGroupRepeat,
    ) -> Result<Self> {
        let separator = separator.into();
        validate_token(&separator, false)?;
        if sizes.is_empty() || sizes.contains(&0) {
            return Err(Error::Data(
                "numeric grouping requires positive sizes".into(),
            ));
        }
        Ok(Self {
            separator,
            sizes,
            repeat,
        })
    }
    /// Literal separator, without whitespace normalization.
    pub fn separator(&self) -> &str {
        &self.separator
    }
    /// Rightmost group first.
    pub fn sizes(&self) -> &[usize] {
        &self.sizes
    }
    /// Continuation after the last declared size.
    pub const fn repeat(&self) -> NumericGroupRepeat {
        self.repeat
    }
    pub(super) fn size(&self, index: usize) -> Option<usize> {
        self.sizes
            .get(index)
            .copied()
            .or_else(|| match self.repeat {
                NumericGroupRepeat::Cycle => Some(self.sizes[index % self.sizes.len()]),
                NumericGroupRepeat::Last => self.sizes.last().copied(),
                NumericGroupRepeat::Stop => None,
            })
    }
}

/// Literal signs and rate suffixes for one numeric locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericSymbols {
    positive: String,
    negative: String,
    percent: String,
    per_mille: String,
}

impl NumericSymbols {
    /// Construct four nonempty single-line tokens.
    pub fn new(
        positive: impl Into<String>,
        negative: impl Into<String>,
        percent: impl Into<String>,
        per_mille: impl Into<String>,
    ) -> Result<Self> {
        let value = Self {
            positive: positive.into(),
            negative: negative.into(),
            percent: percent.into(),
            per_mille: per_mille.into(),
        };
        for token in [
            &value.positive,
            &value.negative,
            &value.percent,
            &value.per_mille,
        ] {
            validate_token(token, false)?;
        }
        Ok(value)
    }
    /// Positive sign.
    pub fn positive(&self) -> &str {
        &self.positive
    }
    /// Negative sign.
    pub fn negative(&self) -> &str {
        &self.negative
    }
    /// Percent suffix.
    pub fn percent(&self) -> &str {
        &self.percent
    }
    /// Per-mille suffix.
    pub fn per_mille(&self) -> &str {
        &self.per_mille
    }
}

/// Literal currency affixes, independent of any exchange or unit conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericCurrency {
    prefix: String,
    suffix: String,
}

impl NumericCurrency {
    /// Construct single-line affixes; either may be empty.
    pub fn new(prefix: impl Into<String>, suffix: impl Into<String>) -> Result<Self> {
        let value = Self {
            prefix: prefix.into(),
            suffix: suffix.into(),
        };
        validate_token(&value.prefix, true)?;
        validate_token(&value.suffix, true)?;
        Ok(value)
    }
    /// Prefix after the sign.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
    /// Suffix before accounting closure.
    pub fn suffix(&self) -> &str {
        &self.suffix
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LocaleData {
    decimal: String,
    digits: [String; 10],
    grouping: Option<NumericGrouping>,
    symbols: NumericSymbols,
    currency: NumericCurrency,
}

/// A complete immutable numeric locale, selected explicitly per format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericLocale(Arc<LocaleData>);

impl NumericLocale {
    /// ASCII digits and signs, period decimal, western grouping and dollar currency.
    pub fn invariant() -> Self {
        Self::custom(
            ".",
            std::array::from_fn(|i| {
                char::from(b'0' + u8::try_from(i).expect("ten digits")).to_string()
            }),
            Some(
                NumericGrouping::new(",", vec![3], NumericGroupRepeat::Cycle)
                    .expect("invariant grouping"),
            ),
            NumericSymbols::new("+", "-", "%", "‰").expect("invariant symbols"),
            NumericCurrency::new("$", "").expect("invariant currency"),
        )
        .expect("invariant locale")
    }
    /// Select an exact d3-format 3.1.0 locale ID, never a host fallback.
    pub fn named(name: &str) -> Result<Self> {
        data::named(name)
    }
    /// The complete pinned catalog, in canonical ID order.
    pub fn named_ids() -> &'static [&'static str] {
        data::IDS
    }
    /// Construct complete tokens. Digits must be ten nonempty distinct strings.
    pub fn custom(
        decimal: impl Into<String>,
        digits: [String; 10],
        grouping: Option<NumericGrouping>,
        symbols: NumericSymbols,
        currency: NumericCurrency,
    ) -> Result<Self> {
        let decimal = decimal.into();
        validate_token(&decimal, false)?;
        for (index, digit) in digits.iter().enumerate() {
            validate_token(digit, false)?;
            if digits[..index].contains(digit) {
                return Err(Error::Data("numeric digit tokens must be distinct".into()));
            }
        }
        Ok(Self(Arc::new(LocaleData {
            decimal,
            digits,
            grouping,
            symbols,
            currency,
        })))
    }
    /// Decimal separator.
    pub fn decimal(&self) -> &str {
        &self.0.decimal
    }
    /// Ten digit tokens in value order.
    pub fn digits(&self) -> &[String; 10] {
        &self.0.digits
    }
    /// Optional grouping pattern; a missing pattern disables grouping.
    pub fn grouping(&self) -> Option<&NumericGrouping> {
        self.0.grouping.as_ref()
    }
    /// Sign and rate tokens.
    pub fn symbols(&self) -> &NumericSymbols {
        &self.0.symbols
    }
    /// Currency affixes.
    pub fn currency(&self) -> &NumericCurrency {
        &self.0.currency
    }
    pub(super) fn observe(&self, round: &mut crate::layout::kernel::FeedbackRound) -> Result<()> {
        round.observe_string(self.decimal())?;
        for token in self.digits() {
            round.observe_string(token)?;
        }
        round.observe_count(usize::from(self.grouping().is_some()))?;
        if let Some(grouping) = self.grouping() {
            round.observe_string(grouping.separator())?;
            round.observe_count(grouping.sizes().len())?;
            for size in grouping.sizes() {
                round.observe_count(*size)?;
            }
            round.observe_count(grouping.repeat() as usize)?;
        }
        for token in [
            self.symbols().positive(),
            self.symbols().negative(),
            self.symbols().percent(),
            self.symbols().per_mille(),
            self.currency().prefix(),
            self.currency().suffix(),
        ] {
            round.observe_string(token)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn named_catalog_matches_the_pinned_upstream_token_transcription() {
        // Digest independently collected from all 59 upstream JSON definitions at
        // the pinned commit, with documented defaults applied. Each UTF-8 token
        // is NUL-terminated; sizes are positive ASCII integers joined by commas.
        let mut hash = Sha256::new();
        assert_eq!(NumericLocale::named_ids().len(), 59);
        for &id in NumericLocale::named_ids() {
            let locale = NumericLocale::named(id).unwrap();
            let grouping = locale.grouping().unwrap();
            let sizes = grouping
                .sizes()
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(",");
            for token in std::iter::once(id)
                .chain(std::iter::once(locale.decimal()))
                .chain(locale.digits().iter().map(String::as_str))
                .chain([
                    grouping.separator(),
                    &sizes,
                    "Cycle",
                    locale.symbols().positive(),
                    locale.symbols().negative(),
                    locale.symbols().percent(),
                    locale.symbols().per_mille(),
                    locale.currency().prefix(),
                    locale.currency().suffix(),
                ])
            {
                hash.update(token.as_bytes());
                hash.update([0]);
            }
        }
        assert_eq!(
            format!("{:x}", hash.finalize()),
            "a520ae09788584621b6b11520d142cad4512e2c6a2830da2e471a3ac3287c5d3"
        );
        assert!(NumericLocale::named("en-us").is_err());
        assert!(NumericLocale::named("host").is_err());
    }
}
