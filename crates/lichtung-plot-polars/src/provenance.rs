use lichtung_plot::SourceRows;
use polars_core::prelude::{Column, DataType};

use crate::{error, snapshot, Error, Result};

pub(crate) fn ids(column: &Column) -> Result<Vec<usize>> {
    macro_rules! integer {
        ($accessor:ident) => {
            collect_ids(
                column,
                column
                    .as_materialized_series()
                    .$accessor()
                    .map_err(|source| error::polars(column, source))?
                    .iter(),
            )
        };
    }
    match column.dtype() {
        DataType::Int8 => integer!(i8),
        DataType::Int16 => integer!(i16),
        DataType::Int32 => integer!(i32),
        DataType::Int64 => integer!(i64),
        DataType::Int128 => integer!(i128),
        DataType::UInt8 => integer!(u8),
        DataType::UInt16 => integer!(u16),
        DataType::UInt32 => integer!(u32),
        DataType::UInt64 => integer!(u64),
        DataType::UInt128 => integer!(u128),
        _ => Err(error::unsupported(column, "source rows")),
    }
}

fn collect_ids<T: Copy + TryInto<usize> + std::fmt::Debug>(
    column: &Column,
    input: impl IntoIterator<Item = Option<T>>,
) -> Result<Vec<usize>> {
    let mut ids = snapshot::buffer(column, "source rows")?;
    for (slot, value) in input.into_iter().enumerate() {
        ids.push(
            value
                .and_then(|value| value.try_into().ok())
                .ok_or_else(|| Error::SourceRow {
                    name: column.name().to_string(),
                    slot,
                    value: format!("{value:?}"),
                })?,
        );
    }
    Ok(ids)
}

/// Snapshot exact unique non-null nonnegative integer IDs fitting usize.
///
/// Floating, string, categorical and untyped Null columns are rejected rather
/// than coerced. Keep IDs aligned explicitly while filtering/reordering upstream.
pub fn source_rows(column: &Column) -> Result<SourceRows> {
    SourceRows::new(ids(column)?).map_err(|source| error::native(column, "source rows", source))
}
