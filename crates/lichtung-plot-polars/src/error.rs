use std::collections::TryReserveError;

use polars_core::prelude::{Column, DataType, PolarsError};

/// A Polars binding failure or an original error from a native plotting operation.
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("{role}: field {name:?} requires a DataFrame lookup context")]
    MissingSource { name: String, role: &'static str },
    #[error("{role}: field {name:?}: {source}")]
    Lookup {
        name: String,
        role: &'static str,
        #[source]
        source: PolarsError,
    },
    #[error("{role}: {name:?} has {actual} slots, expected {expected}")]
    Length {
        name: String,
        role: &'static str,
        expected: usize,
        actual: usize,
    },
    #[error("{role}: {message}")]
    Binding { role: &'static str, message: String },
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
    #[error("{0}")]
    Plot(#[from] lichtung_plot::Error),
}

pub(crate) type BindingResult<T> = std::result::Result<T, AdapterError>;

pub(crate) fn binding(role: &'static str, message: impl Into<String>) -> AdapterError {
    AdapterError::Binding {
        role,
        message: message.into(),
    }
}

pub(crate) fn length(
    role: &'static str,
    name: &str,
    expected: usize,
    actual: usize,
) -> BindingResult<()> {
    if expected == actual {
        Ok(())
    } else {
        Err(AdapterError::Length {
            name: name.into(),
            role,
            expected,
            actual,
        })
    }
}

pub(crate) fn unsupported(column: &Column, channel: &'static str) -> AdapterError {
    AdapterError::UnsupportedDtype {
        name: column.name().to_string(),
        dtype: column.dtype().clone(),
        channel,
    }
}

pub(crate) fn polars(column: &Column, source: PolarsError) -> AdapterError {
    AdapterError::Polars {
        name: column.name().to_string(),
        source,
    }
}

pub(crate) fn native(
    column: &Column,
    channel: &'static str,
    source: lichtung_plot::Error,
) -> AdapterError {
    AdapterError::Native {
        name: column.name().to_string(),
        channel,
        source,
    }
}
