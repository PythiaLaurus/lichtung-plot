use std::sync::Arc;

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
    entries: Arc<Vec<(String, crate::theme::OpaqueColor)>>,
}

impl CategoricalColors {
    #[expect(
        clippy::unused_self,
        reason = "the source map's copy-admission contract prices one immutable handle before the enclosing Plot clone"
    )]
    pub(crate) fn admit_clone(&self, data: crate::data::PlotData<'_>) -> Result<(), Error> {
        data.require_source_resources()?;
        data.claim_source_visits(1)
    }

    /// Constructs a non-empty color map with unique, exact category labels.
    ///
    /// # Errors
    ///
    /// Returns an error when a label is empty, has surrounding whitespace or
    /// control characters, or appears more than once.
    pub fn new(entries: impl IntoIterator<Item = (String, (u8, u8, u8))>) -> Result<Self, Error> {
        Self::from_opaque(
            entries
                .into_iter()
                .map(|(key, (r, g, b))| (key, crate::theme::OpaqueColor::rgb(r, g, b))),
        )
    }

    pub(crate) fn from_opaque(
        input: impl IntoIterator<Item = (String, crate::theme::OpaqueColor)>,
    ) -> Result<Self, Error> {
        let mut entries: Vec<(String, crate::theme::OpaqueColor)> = Vec::new();
        for (key, color) in input {
            if key.is_empty() || key.trim() != key || key.chars().any(char::is_control) {
                return Err(Error::Data("categorical color labels must be non-empty and contain no surrounding whitespace or control characters".into()));
            }
            if entries.iter().any(|(previous, _)| previous == &key) {
                return Err(Error::Data(format!(
                    "categorical color label {key:?} is duplicated"
                )));
            }
            entries
                .len()
                .checked_add(1)
                .ok_or_else(|| Error::Data("categorical map length overflow".into()))?;
            entries
                .try_reserve_exact(1)
                .map_err(|_| Error::Data("cannot reserve categorical color map".into()))?;
            entries.push((key, color));
        }
        if entries.is_empty() {
            return Err(Error::Data(
                "categorical colors require at least one category label".into(),
            ));
        }
        Ok(Self {
            entries: Arc::new(entries),
        })
    }

    pub(crate) fn entries(
        &self,
    ) -> impl ExactSizeIterator<Item = (&str, crate::theme::OpaqueColor)> + '_ {
        self.entries
            .iter()
            .map(|(key, color)| (key.as_str(), *color))
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "readonly opaque map inspection awaits the categorical public cutover"
        )
    )]
    pub(crate) fn color(&self, label: &str) -> Option<crate::theme::OpaqueColor> {
        self.entries
            .iter()
            .find(|(key, _)| key == label)
            .map(|(_, color)| *color)
    }

    pub(crate) fn color_in(
        &self,
        label: &str,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<Option<crate::theme::OpaqueColor>, Error> {
        for (candidate, color) in self.entries() {
            if crate::font::canonical::same_bytes(candidate, label, resources)
                .map_err(|e| Error::Render(e.to_string()))?
            {
                return Ok(Some(color));
            }
        }
        Ok(None)
    }
}
