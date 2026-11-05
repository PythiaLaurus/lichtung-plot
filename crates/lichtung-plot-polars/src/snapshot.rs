use polars_core::prelude::Column;

use crate::error::{AdapterError as Error, BindingResult as Result};

pub(crate) fn allocate<T>(len: usize, role: &'static str) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(len)
        .map_err(|source| Error::Allocation {
            name: role.into(),
            channel: role,
            source,
        })?;
    Ok(values)
}

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
