use std::collections::TryReserveError;

/// A structural or conversion failure, separate from later plot validation.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{operation}: expected shape {expected:?}, got {actual:?}")]
    Shape {
        operation: &'static str,
        expected: Vec<usize>,
        actual: Vec<usize>,
    },
    #[error("numeric columns: expected {expected} names, got {actual}")]
    ColumnNames { expected: usize, actual: usize },
    #[error("heatmap {axis}: duplicate raw key {key:?}")]
    DuplicateAxisKey { axis: &'static str, key: String },
    #[error(
        "source-row ID {value} at logical slot {slot} must be present, nonnegative and fit usize"
    )]
    SourceRow { slot: usize, value: String },
    #[error("heatmap cell count exceeds usize")]
    SizeOverflow,
    #[error("{operation}: cannot reserve conversion storage")]
    Allocation {
        operation: &'static str,
        #[source]
        source: TryReserveError,
    },
    #[error("{operation}: {source}")]
    Native {
        operation: &'static str,
        #[source]
        source: lichtung_plot::Error,
    },
}

/// Adapter conversion result.
pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn native(operation: &'static str, source: lichtung_plot::Error) -> Error {
    Error::Native { operation, source }
}
