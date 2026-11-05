use lichtung_plot::BooleanValues;
use polars_core::prelude::{Column, DataType};

use crate::error::BindingResult as Result;
use crate::{error, snapshot};

pub(crate) fn values(column: &Column) -> Result<Vec<Option<bool>>> {
    match column.dtype() {
        DataType::Boolean => snapshot::collect(
            column,
            "Boolean",
            column
                .as_materialized_series()
                .bool()
                .map_err(|source| error::polars(column, source))?
                .iter(),
        ),
        DataType::Null => {
            snapshot::collect(column, "Boolean", std::iter::repeat_n(None, column.len()))
        }
        _ => Err(error::unsupported(column, "Boolean")),
    }
}

/// Snapshot true/false/null without numeric or string coercion.
pub(crate) fn booleans(column: &Column) -> Result<BooleanValues> {
    Ok(BooleanValues::nullable(
        column.name().as_str(),
        values(column)?,
    ))
}
