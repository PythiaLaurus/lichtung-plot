//! Plot logical ndarray vectors, selected table columns and rectangular grids.
//!
//! The complete native renderer API is re-exported unchanged. Additional family
//! constructors bind foreign inputs eagerly; only `from_data` retains a read-only
//! source context for later logical-column lookup. `.into()` hands off an owned
//! native plot and releases that context. Validity pairs use **true = present**.
//!
//! Strided, transposed, reversed, Fortran and broadcast views follow logical index
//! order. Nulls and present NaNs stay distinct; numeric primitives round to f64.
//!
//! Native capability bounds also apply to adapter receivers:
//! ```compile_fail
//! use lichtung_plot_ndarray::{AlphaEncoding, Scatter};
//! fn unsupported(plot: Scatter<'_>) {
//!     plot.line_alpha(AlphaEncoding::theme()); // Scatter has no line target.
//! }
//! ```

#![allow(
    clippy::needless_pass_by_value,
    reason = "ndarray views are lightweight borrowed descriptors conventionally accepted by value"
)]

pub use lichtung_plot::*;

mod alpha;
mod element;
mod error;
mod frontend;
mod input;
mod layers;
mod significance;
mod snapshot;
mod source;

pub use alpha::{AlphaInput, AlphaValuesInput};
pub use element::{
    ArrayElement, BooleanElement, CategoryElement, NumericElement, SourceRowElement,
};
pub use error::AdapterError;
pub use frontend::*;
pub use input::{
    AxisInput, BooleanInput, CategoryInput, ColumnsInput, NumericInput, NumericSeriesInput,
    SourceRowsInput,
};
pub use layers::{BarPointLayerAdapter, PointLayerAdapter};
pub use significance::*;
pub use source::ArraySource;

#[cfg(test)]
mod tests;
