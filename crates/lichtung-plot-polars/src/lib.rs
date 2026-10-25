//! Snapshot eager Polars columns as owned lichtung-plot native values.
//!
//! Logical row order, sliced validity, all chunks and repeated scalar lengths are
//! preserved. Null stays distinct from present NaN. Numeric primitives normalize
//! to f64 with ordinary rounding; semantic/nested dtypes are not reinterpreted.
//! No filtering, aggregation, query execution or foreign buffer retention occurs.
//! A ScalarColumn may populate Polars' internal full-Series materialization cache.
//!
//! Pass a DataFrame's `column(name)?` directly. A standalone Series uses Polars'
//! own `IntoColumn` conversion. The renderer does not depend on this adapter.

mod boolean;
mod category;
mod columns;
mod error;
mod numeric;
mod provenance;
mod snapshot;

pub use boolean::booleans;
pub use category::categories;
pub use columns::numeric_columns;
pub use error::{Error, Result};
pub use numeric::numeric;
pub use provenance::source_rows;

#[cfg(test)]
mod tests;
