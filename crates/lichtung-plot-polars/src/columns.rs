use lichtung_plot::NumericValues;
use polars_core::prelude::DataFrame;

use crate::error::{AdapterError as Error, BindingResult as Result};

/// Convert exactly the requested numeric columns in caller order.
///
/// Selectors are literal names, not expressions, regexes or automatic discovery.
/// Missing/wrong-type columns fail; duplicates are not renamed or silently removed.
pub(crate) fn numeric_columns(frame: &DataFrame, names: &[&str]) -> Result<Vec<NumericValues>> {
    let mut columns = Vec::new();
    columns
        .try_reserve_exact(names.len())
        .map_err(|source| Error::Allocation {
            name: "selected frame columns".into(),
            channel: "numeric columns",
            source,
        })?;
    for name in names {
        let column = frame.column(name).map_err(|source| Error::Polars {
            name: (*name).into(),
            source,
        })?;
        columns.push(crate::numeric::numeric(column)?);
    }
    Ok(columns)
}
