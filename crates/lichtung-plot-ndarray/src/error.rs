use std::collections::TryReserveError;

/// A structural ndarray binding failure, distinct from native scientific validation.
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("{operation}: expected shape {expected:?}, got {actual:?}")]
    Shape {
        operation: &'static str,
        expected: Vec<usize>,
        actual: Vec<usize>,
    },
    #[error("{role}: column {index} requires an array source")]
    MissingSource { role: &'static str, index: usize },
    #[error("{role}: column {index} is outside {columns} logical columns")]
    Column {
        role: &'static str,
        index: usize,
        columns: usize,
    },
    #[error("{role}: element type {element} is not supported{context}", context = .column.map_or_else(String::new, |index| format!(" for logical column {index}")))]
    UnsupportedElement {
        role: &'static str,
        element: &'static str,
        column: Option<usize>,
    },
    #[error("{role}: {detail}")]
    Binding { role: &'static str, detail: String },
    #[error(
        "source-row ID {value} at logical slot {slot}{context} must be present, nonnegative and fit usize", context = .column.map_or_else(String::new, |index| format!(" in logical column {index}"))
    )]
    SourceRow {
        slot: usize,
        value: String,
        column: Option<usize>,
    },
    #[error("{operation}: cannot reserve conversion storage")]
    Allocation {
        operation: &'static str,
        #[source]
        source: TryReserveError,
    },
    #[error("{0}")]
    Native(#[from] crate::Error),
}

pub(crate) type BindingResult<T> = std::result::Result<T, AdapterError>;

pub(crate) fn binding(role: &'static str, detail: impl Into<String>) -> AdapterError {
    AdapterError::Binding {
        role,
        detail: detail.into(),
    }
}
pub(crate) fn length(
    role: &'static str,
    name: &str,
    expected: usize,
    actual: usize,
) -> BindingResult<()> {
    if actual != expected {
        return Err(binding(
            role,
            format!("{name:?}: expected {expected} slots, got {actual}"),
        ));
    }
    Ok(())
}
pub(crate) fn unsupported<T>(role: &'static str) -> AdapterError {
    AdapterError::UnsupportedElement {
        role,
        element: std::any::type_name::<T>(),
        column: None,
    }
}

pub(crate) fn unsupported_column<T>(role: &'static str, index: usize) -> AdapterError {
    AdapterError::UnsupportedElement {
        role,
        element: std::any::type_name::<T>(),
        column: Some(index),
    }
}
