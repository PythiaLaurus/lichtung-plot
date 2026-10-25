//! Convert borrowed ndarray views into owned lichtung-plot values.
//!
//! Iteration follows logical indices, including transposed, reversed, strided and
//! broadcast views. Conversion never sorts or drops slots. Numeric values round
//! normally to `f64`; null remains distinct from present NaN. Explicit validity
//! uses **true = present**, not NumPy masked-array true-means-missing semantics.
//! The returned snapshot does not retain the source array.
//!
//! Use `.view()` on an owned array and the ordinary native Plot constructors on
//! the result. Two-dimensional tables use observations in rows and variables in
//! columns; labeled heatmap expansion is a separate explicit operation.

#![allow(
    clippy::needless_pass_by_value,
    reason = "ndarray views are lightweight borrowed descriptors conventionally accepted by value"
)]

mod boolean;
mod category;
mod columns;
mod element;
mod error;
mod grid;
mod numeric;
mod provenance;
mod snapshot;

pub use boolean::{booleans, booleans_with_validity};
pub use category::{categories, categories_with_validity};
pub use columns::{numeric_columns, numeric_columns_with_validity};
pub use element::{BooleanElement, CategoryElement, NumericElement, SourceRowElement};
pub use error::{Error, Result};
pub use grid::{heatmap_values, heatmap_values_with_validity, HeatmapValues};
pub use numeric::{numeric, numeric_with_validity};
pub use provenance::source_rows;

#[cfg(test)]
mod tests;
