use std::collections::BTreeSet;

use crate::Error;

/// Direction used to order categories by their prepared rendered value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueSort {
    /// Sort from the smallest rendered value to the largest.
    Ascending,
    /// Sort from the largest rendered value to the smallest.
    Descending,
}

/// Display ordering for one categorical plot channel.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CategoryOrder {
    /// Preserve deterministic first-observed category order.
    #[default]
    Observed,
    /// Render category identities in this exact caller-supplied order.
    Explicit(Vec<String>),
    /// Sort categories by the scalar value prepared for each rendered category.
    Value(ValueSort),
}

impl CategoryOrder {
    /// Construct an exact caller-supplied category order.
    #[must_use]
    pub fn explicit<S>(levels: impl IntoIterator<Item = S>) -> Self
    where
        S: Into<String>,
    {
        Self::Explicit(levels.into_iter().map(Into::into).collect())
    }
}

impl From<ValueSort> for CategoryOrder {
    fn from(sort: ValueSort) -> Self {
        Self::Value(sort)
    }
}

/// Exact caller-owned colors for categorical plot levels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoricalColors {
    entries: Vec<(String, (u8, u8, u8))>,
}

impl CategoricalColors {
    /// Constructs a non-empty color map with unique, exact category labels.
    ///
    /// # Errors
    ///
    /// Returns an error when a label is empty, has surrounding whitespace or
    /// control characters, or appears more than once.
    pub fn new(entries: impl IntoIterator<Item = (String, (u8, u8, u8))>) -> Result<Self, Error> {
        let entries = entries.into_iter().collect::<Vec<_>>();
        if entries.is_empty() {
            return Err(Error::Data(
                "categorical colors require at least one category label".into(),
            ));
        }
        let mut labels = BTreeSet::new();
        for (label, _) in &entries {
            if label.trim().is_empty()
                || label.trim() != label
                || label.chars().any(char::is_control)
            {
                return Err(Error::Data(
                    "categorical color labels must be non-empty and contain no surrounding whitespace or control characters"
                        .into(),
                ));
            }
            if !labels.insert(label.as_str()) {
                return Err(Error::Data(format!(
                    "categorical color label {label:?} is duplicated"
                )));
            }
        }
        Ok(Self { entries })
    }

    pub(crate) fn color(&self, label: &str) -> Option<(u8, u8, u8)> {
        self.entries
            .iter()
            .find_map(|(candidate, color)| (candidate == label).then_some(*color))
    }
}
