//! Explicit ndarray extraction for `lichtung-plot` native values.

#![deny(missing_docs)]

use lichtung_plot::{BooleanValues, CategoryValues, Error, NumericValues, SourceRows};
use ndarray::{ArrayView1, ArrayView2, Axis};

/// Copy one numeric vector into immutable native values.
#[must_use]
pub fn numeric(name: impl Into<String>, values: ArrayView1<'_, f64>) -> NumericValues {
    NumericValues::new(name, values.iter().copied())
}

/// Copy one Boolean vector into immutable native values.
#[must_use]
pub fn boolean(name: impl Into<String>, values: ArrayView1<'_, bool>) -> BooleanValues {
    BooleanValues::new(name, values.iter().copied())
}

/// Copy one string vector into stable first-observed categorical values.
pub fn category(
    name: impl Into<String>,
    values: ArrayView1<'_, String>,
) -> Result<CategoryValues, Error> {
    CategoryValues::new(name, values.iter().cloned())
}

/// Copy one unsigned vector into unique source-row identities.
pub fn source_rows(values: ArrayView1<'_, usize>) -> Result<SourceRows, Error> {
    SourceRows::new(values.iter().copied())
}

/// Copy matrix columns into ordered named numeric values.
pub fn numeric_columns(
    names: &[impl AsRef<str>],
    values: ArrayView2<'_, f64>,
) -> Result<Vec<NumericValues>, Error> {
    if names.len() != values.ncols() {
        return Err(Error::Data(format!(
            "ndarray matrix has {} columns but {} names",
            values.ncols(),
            names.len()
        )));
    }
    Ok(values
        .axis_iter(Axis(1))
        .zip(names)
        .map(|(column, name)| NumericValues::new(name.as_ref(), column.iter().copied()))
        .collect())
}
