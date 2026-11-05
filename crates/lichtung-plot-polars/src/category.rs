use lichtung_plot::CategoryValues;
use polars_core::prelude::{Column, DataType};

use crate::error::BindingResult as Result;
use crate::{error, snapshot};

pub(crate) fn values(column: &Column) -> Result<Vec<Option<String>>> {
    match column.dtype() {
        DataType::String => snapshot::collect(
            column,
            "categories",
            column
                .as_materialized_series()
                .str()
                .map_err(|source| error::polars(column, source))?
                .iter()
                .map(|value| value.map(str::to_owned)),
        ),
        DataType::Categorical(..) | DataType::Enum(..) => {
            // Decode dictionary identities, never physical category codes. This
            // dtype-bounded cast has no numeric precision or parsing policy.
            let decoded = column
                .as_materialized_series()
                .strict_cast(&DataType::String)
                .map_err(|source| error::polars(column, source))?;
            snapshot::collect(
                column,
                "categories",
                decoded
                    .str()
                    .map_err(|source| error::polars(column, source))?
                    .iter()
                    .map(|value| value.map(str::to_owned)),
            )
        }
        DataType::Null => snapshot::collect(
            column,
            "categories",
            std::iter::repeat_n(None, column.len()),
        ),
        _ => Err(error::unsupported(column, "categories")),
    }
}

/// Snapshot String/Categorical/Enum/null as exact raw native category keys.
///
/// Rebuild the native first-observed dictionary. Foreign codes, unobserved levels
/// and Enum schema order are not plot policies; apply native CategoryOrder
/// explicitly when needed. No physical integer/stringification fallback exists.
pub(crate) fn categories(column: &Column) -> Result<CategoryValues> {
    CategoryValues::nullable(column.name().as_str(), values(column)?)
        .map_err(|source| error::native(column, "categories", source))
}
