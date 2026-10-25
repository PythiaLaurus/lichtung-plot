use std::collections::TryReserveError;

use polars_core::prelude::{Column, DataType, PolarsError};

/// Adapter conversion failure, separate from scientific/render validation.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("column {name:?}: dtype {dtype:?} is unsupported for {channel}")]
    UnsupportedDtype {
        name: String,
        dtype: DataType,
        channel: &'static str,
    },
    #[error("column {name:?}: source-row ID {value} at logical slot {slot} must be present, nonnegative and fit usize")]
    SourceRow {
        name: String,
        slot: usize,
        value: String,
    },
    #[error("column {name:?}: cannot reserve {channel} conversion storage")]
    Allocation {
        name: String,
        channel: &'static str,
        #[source]
        source: TryReserveError,
    },
    #[error("column {name:?}: {source}")]
    Polars {
        name: String,
        #[source]
        source: PolarsError,
    },
    #[error("column {name:?}, {channel}: {source}")]
    Native {
        name: String,
        channel: &'static str,
        #[source]
        source: lichtung_plot::Error,
    },
}

/// Adapter conversion result.
pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn unsupported(column: &Column, channel: &'static str) -> Error {
    Error::UnsupportedDtype {
        name: column.name().to_string(),
        dtype: column.dtype().clone(),
        channel,
    }
}

pub(crate) fn polars(column: &Column, source: PolarsError) -> Error {
    Error::Polars {
        name: column.name().to_string(),
        source,
    }
}

pub(crate) fn native(
    column: &Column,
    channel: &'static str,
    source: lichtung_plot::Error,
) -> Error {
    Error::Native {
        name: column.name().to_string(),
        channel,
        source,
    }
}
