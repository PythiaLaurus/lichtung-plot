use lichtung_plot::NumericValues;
use polars_core::prelude::{Column, DataType};

use crate::{error, snapshot, Result};

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_lossless,
    reason = "all primitive widths deliberately use ordinary f64 normalization"
)]
pub(crate) fn values(column: &Column) -> Result<Vec<Option<f64>>> {
    macro_rules! numeric {
        ($accessor:ident) => {
            snapshot::collect(
                column,
                "numeric",
                column
                    .as_materialized_series()
                    .$accessor()
                    .map_err(|source| error::polars(column, source))?
                    .iter()
                    .map(|value| value.map(|value| value as f64)),
            )
        };
    }
    match column.dtype() {
        DataType::Int8 => numeric!(i8),
        DataType::Int16 => numeric!(i16),
        DataType::Int32 => numeric!(i32),
        DataType::Int64 => numeric!(i64),
        DataType::Int128 => numeric!(i128),
        DataType::UInt8 => numeric!(u8),
        DataType::UInt16 => numeric!(u16),
        DataType::UInt32 => numeric!(u32),
        DataType::UInt64 => numeric!(u64),
        DataType::UInt128 => numeric!(u128),
        DataType::Float32 => numeric!(f32),
        DataType::Float64 => numeric!(f64),
        DataType::Float16 => snapshot::collect(
            column,
            "numeric",
            column
                .as_materialized_series()
                .f16()
                .map_err(|source| error::polars(column, source))?
                .iter()
                .map(|value| value.map(f64::from)),
        ),
        DataType::Null => {
            snapshot::collect(column, "numeric", std::iter::repeat_n(None, column.len()))
        }
        _ => Err(error::unsupported(column, "numeric")),
    }
}

/// Snapshot a named numeric column with normal f64 rounding and null preservation.
///
/// Supports Float16/32/64 and signed/unsigned 8/16/32/64/128-bit integers, plus
/// untyped Null. Boolean/string/temporal/decimal/nested dtypes are not coerced.
pub fn numeric(column: &Column) -> Result<NumericValues> {
    Ok(NumericValues::nullable(
        column.name().as_str(),
        values(column)?,
    ))
}
