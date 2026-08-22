//! Immutable Rust-native values accepted by plot constructors.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::Error;

/// Original source-row identities shared by aligned plot values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRows(Arc<[usize]>);

impl SourceRows {
    /// Construct unique source-row identities.
    pub fn new(rows: impl IntoIterator<Item = usize>) -> Result<Self, Error> {
        let rows = rows.into_iter().collect::<Vec<_>>();
        let mut seen = HashSet::with_capacity(rows.len());
        if let Some(duplicate) = rows.iter().copied().find(|row| !seen.insert(*row)) {
            return Err(Error::Data(format!(
                "source-row identities must be unique; found duplicate {duplicate}"
            )));
        }
        Ok(Self(rows.into()))
    }

    /// Number of represented rows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether no rows are represented.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn get(&self, row: usize) -> Option<usize> {
        self.0.get(row).copied()
    }

    pub(crate) fn as_slice(&self) -> &[usize] {
        &self.0
    }
}

#[derive(Debug, Clone)]
enum Validity {
    All,
    Bitmap(Arc<[u64]>),
}

impl Validity {
    fn from_flags(flags: &[bool]) -> Self {
        if flags.iter().all(|valid| *valid) {
            return Self::All;
        }
        let mut words = vec![0_u64; flags.len().div_ceil(64)];
        for (index, valid) in flags.iter().copied().enumerate() {
            if valid {
                words[index / 64] |= 1_u64 << (index % 64);
            }
        }
        Self::Bitmap(words.into())
    }

    fn is_valid(&self, index: usize) -> bool {
        match self {
            Self::All => true,
            Self::Bitmap(words) => words
                .get(index / 64)
                .is_some_and(|word| word & (1_u64 << (index % 64)) != 0),
        }
    }
}

/// Named nullable numeric values stored as immutable native `f64` data.
#[derive(Debug, Clone)]
pub struct NumericValues {
    name: Arc<str>,
    values: Arc<[f64]>,
    validity: Validity,
    source_rows: Option<SourceRows>,
}

impl NumericValues {
    /// Construct complete numeric values.
    #[must_use]
    pub fn new(name: impl Into<String>, values: impl IntoIterator<Item = f64>) -> Self {
        let values = values.into_iter().collect::<Vec<_>>();
        Self {
            name: Arc::from(name.into()),
            values: values.into(),
            validity: Validity::All,
            source_rows: None,
        }
    }

    /// Construct nullable numeric values without conflating nulls with `NaN`.
    #[must_use]
    pub fn nullable(
        name: impl Into<String>,
        values: impl IntoIterator<Item = Option<f64>>,
    ) -> Self {
        let values = values.into_iter().collect::<Vec<_>>();
        let validity =
            Validity::from_flags(&values.iter().map(Option::is_some).collect::<Vec<_>>());
        let values = values
            .into_iter()
            .map(|value| value.unwrap_or(0.0))
            .collect::<Vec<_>>();
        Self {
            name: Arc::from(name.into()),
            values: values.into(),
            validity,
            source_rows: None,
        }
    }

    /// Attach source-row identities aligned one-to-one with these values.
    pub fn source_rows(mut self, rows: SourceRows) -> Result<Self, Error> {
        validate_source_rows(self.values.len(), &rows, &self.name)?;
        self.source_rows = Some(rows);
        Ok(self)
    }

    /// Semantic value name used for inferred labels and diagnostics.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Number of values, including nulls.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether no values are present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    #[expect(
        clippy::option_option,
        reason = "the outer option is row bounds and the inner option is value validity"
    )]
    pub(crate) fn get(&self, row: usize) -> Option<Option<f64>> {
        self.values
            .get(row)
            .map(|value| self.validity.is_valid(row).then_some(*value))
    }

    pub(crate) fn source_row(&self, row: usize) -> usize {
        self.source_rows
            .as_ref()
            .and_then(|rows| rows.get(row))
            .unwrap_or(row)
    }
}

/// Named nullable categorical values stored with stable dictionary encoding.
#[derive(Debug, Clone)]
pub struct CategoryValues {
    name: Arc<str>,
    levels: Arc<[String]>,
    codes: Arc<[u32]>,
    validity: Validity,
    source_rows: Option<SourceRows>,
}

impl CategoryValues {
    /// Construct complete categorical values in first-observed level order.
    pub fn new<S>(
        name: impl Into<String>,
        values: impl IntoIterator<Item = S>,
    ) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Self::from_optional(
            name.into(),
            values.into_iter().map(|value| Some(value.into())),
        )
    }

    /// Construct nullable categorical values in first-observed level order.
    pub fn nullable<S>(
        name: impl Into<String>,
        values: impl IntoIterator<Item = Option<S>>,
    ) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Self::from_optional(
            name.into(),
            values.into_iter().map(|value| value.map(Into::into)),
        )
    }

    fn from_optional(
        name: String,
        values: impl IntoIterator<Item = Option<String>>,
    ) -> Result<Self, Error> {
        let values = values.into_iter().collect::<Vec<_>>();
        let mut levels = Vec::new();
        let mut indices = HashMap::new();
        let mut codes = Vec::with_capacity(values.len());
        let mut valid = Vec::with_capacity(values.len());
        for value in values {
            let Some(value) = value else {
                valid.push(false);
                codes.push(0);
                continue;
            };
            valid.push(true);
            let code = if let Some(code) = indices.get(&value).copied() {
                code
            } else {
                let code = u32::try_from(levels.len()).map_err(|_| {
                    Error::Data(format!(
                        "categorical values {name:?} exceed the supported distinct-level count"
                    ))
                })?;
                indices.insert(value.clone(), code);
                levels.push(value);
                code
            };
            codes.push(code);
        }
        Ok(Self {
            name: Arc::from(name),
            levels: levels.into(),
            codes: codes.into(),
            validity: Validity::from_flags(&valid),
            source_rows: None,
        })
    }

    /// Attach source-row identities aligned one-to-one with these values.
    pub fn source_rows(mut self, rows: SourceRows) -> Result<Self, Error> {
        validate_source_rows(self.codes.len(), &rows, &self.name)?;
        self.source_rows = Some(rows);
        Ok(self)
    }

    /// Semantic value name used for inferred labels and diagnostics.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Number of values, including nulls.
    #[must_use]
    pub fn len(&self) -> usize {
        self.codes.len()
    }

    /// Whether no values are present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.codes.is_empty()
    }

    #[expect(
        clippy::option_option,
        reason = "the outer option is row bounds and the inner option is value validity"
    )]
    pub(crate) fn get(&self, row: usize) -> Option<Option<&str>> {
        self.codes.get(row).map(|code| {
            if !self.validity.is_valid(row) {
                return None;
            }
            usize::try_from(*code)
                .ok()
                .and_then(|index| self.levels.get(index))
                .map(String::as_str)
        })
    }

    pub(crate) fn source_row(&self, row: usize) -> usize {
        self.source_rows
            .as_ref()
            .and_then(|rows| rows.get(row))
            .unwrap_or(row)
    }
}

/// Named nullable Boolean values.
#[derive(Debug, Clone)]
pub struct BooleanValues {
    name: Arc<str>,
    values: Arc<[bool]>,
    validity: Validity,
    source_rows: Option<SourceRows>,
}

impl BooleanValues {
    /// Construct complete Boolean values.
    #[must_use]
    pub fn new(name: impl Into<String>, values: impl IntoIterator<Item = bool>) -> Self {
        let values = values.into_iter().collect::<Vec<_>>();
        Self {
            name: Arc::from(name.into()),
            values: values.into(),
            validity: Validity::All,
            source_rows: None,
        }
    }

    /// Construct nullable Boolean values.
    #[must_use]
    pub fn nullable(
        name: impl Into<String>,
        values: impl IntoIterator<Item = Option<bool>>,
    ) -> Self {
        let values = values.into_iter().collect::<Vec<_>>();
        let validity =
            Validity::from_flags(&values.iter().map(Option::is_some).collect::<Vec<_>>());
        let values = values
            .into_iter()
            .map(|value| value.unwrap_or(false))
            .collect::<Vec<_>>();
        Self {
            name: Arc::from(name.into()),
            values: values.into(),
            validity,
            source_rows: None,
        }
    }

    /// Attach source-row identities aligned one-to-one with these values.
    pub fn source_rows(mut self, rows: SourceRows) -> Result<Self, Error> {
        validate_source_rows(self.values.len(), &rows, &self.name)?;
        self.source_rows = Some(rows);
        Ok(self)
    }

    /// Semantic value name used for diagnostics.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Number of values, including nulls.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether no values are present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    #[expect(
        clippy::option_option,
        reason = "the outer option is row bounds and the inner option is value validity"
    )]
    pub(crate) fn get(&self, row: usize) -> Option<Option<bool>> {
        self.values
            .get(row)
            .map(|value| self.validity.is_valid(row).then_some(*value))
    }
}

fn validate_source_rows(length: usize, rows: &SourceRows, name: &str) -> Result<(), Error> {
    if rows.len() != length {
        return Err(Error::Data(format!(
            "source-row identities for {name:?} have length {}, expected {length}",
            rows.len()
        )));
    }
    Ok(())
}
