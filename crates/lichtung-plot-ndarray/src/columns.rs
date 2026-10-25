use lichtung_plot::NumericValues;
use ndarray::ArrayView2;

use crate::{snapshot, Error, NumericElement, Result};

#[derive(Debug)]
pub(crate) struct ColumnValues<'a> {
    pub name: &'a str,
    pub values: Vec<Option<f64>>,
}

pub(crate) fn columns<'a, T: NumericElement>(
    names: &'a [&str],
    view: ArrayView2<'_, T>,
    validity: Option<ArrayView2<'_, bool>>,
) -> Result<Vec<ColumnValues<'a>>> {
    if names.len() != view.ncols() {
        return Err(Error::ColumnNames {
            expected: view.ncols(),
            actual: names.len(),
        });
    }
    if let Some(validity) = &validity {
        if validity.shape() != view.shape() {
            return Err(Error::Shape {
                operation: "numeric column validity",
                expected: view.shape().to_vec(),
                actual: validity.shape().to_vec(),
            });
        }
    }
    let mut columns = snapshot::buffer(names.len(), "numeric columns")?;
    for (index, name) in names.iter().enumerate() {
        let validity = validity.as_ref().map(|validity| validity.column(index));
        let values = snapshot::values(
            view.column(index),
            validity,
            NumericElement::to_f64,
            "numeric columns",
        )?;
        columns.push(ColumnValues { name, values });
    }
    Ok(columns)
}

/// Convert an observation-by-variable table into ordered native numeric columns.
///
/// Supply exactly one name per column. Names are neither deduplicated nor renamed;
/// the eventual native consumer owns its own name/minimum-variable constraints.
pub fn numeric_columns<T: NumericElement>(
    names: &[&str],
    view: ArrayView2<'_, T>,
) -> Result<Vec<NumericValues>> {
    native(columns(names, view, None)?)
}

/// Convert a table with exact-shape validity; no broadcasting or row removal.
pub fn numeric_columns_with_validity<T: NumericElement>(
    names: &[&str],
    view: ArrayView2<'_, T>,
    validity: ArrayView2<'_, bool>,
) -> Result<Vec<NumericValues>> {
    native(columns(names, view, Some(validity))?)
}

fn native(columns: Vec<ColumnValues<'_>>) -> Result<Vec<NumericValues>> {
    let mut native = snapshot::buffer(columns.len(), "native numeric columns")?;
    native.extend(
        columns
            .into_iter()
            .map(|column| NumericValues::nullable(column.name, column.values)),
    );
    Ok(native)
}
