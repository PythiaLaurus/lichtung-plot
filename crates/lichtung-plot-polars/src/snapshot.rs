use polars_core::prelude::Column;

use crate::{Error, Result};

pub(crate) fn buffer<T>(column: &Column, channel: &'static str) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(column.len())
        .map_err(|source| Error::Allocation {
            name: column.name().to_string(),
            channel,
            source,
        })?;
    Ok(values)
}

pub(crate) fn collect<T>(
    column: &Column,
    channel: &'static str,
    input: impl IntoIterator<Item = T>,
) -> Result<Vec<T>> {
    let mut values = buffer(column, channel)?;
    values.extend(input);
    Ok(values)
}
