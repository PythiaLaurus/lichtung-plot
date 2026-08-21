//! Plot construction, layout, rendering, and output errors.

/// Error returned by plotting operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Filesystem input or output failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Input data or a data-bound visual option is invalid.
    #[error("data error: {0}")]
    Data(String),
    /// Layout, backend, font, or output rendering failed.
    #[error("render error: {0}")]
    Render(String),
}

impl From<polars::error::PolarsError> for Error {
    fn from(error: polars::error::PolarsError) -> Self {
        Self::Data(error.to_string())
    }
}

/// Result returned by plotting operations.
pub type Result<T> = std::result::Result<T, Error>;
