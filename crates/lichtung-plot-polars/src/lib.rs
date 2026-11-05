//! Plot eager Polars fields or borrowed Series/Column variables directly.
//!
//! Family constructors bind immutable native snapshots. `from_data` additionally
//! borrows a DataFrame for subsequent field-based setters; `new` releases direct
//! input borrows at each binding. All native lichtung-plot paths are re-exported
//! unchanged, including [`Plot`], [`Error`] and [`Result`]. An explicit `.into()`
//! moves a configured [`PlotAdapter`] into its exact-family native Plot and ends
//! source-bound chaining. Single-image output needs no explicit conversion.
//!
//! Null, present NaN, logical slot order, chunks and scalar-column lengths remain
//! distinct. There are no implicit joins, query execution or new estimators.
//! The standalone renderer never depends on Polars or this adapter.

mod alpha;
mod boolean;
mod category;
mod error;
mod frontend;
mod input;
mod layers;
mod numeric;
mod provenance;
mod significance;
mod snapshot;

pub use alpha::{AlphaInput, AlphaValuesInput};
pub use error::AdapterError;
pub use frontend::{
    Bar, Beeswarm, Boxplot, Coefficient, Density, Dumbbell, Forest, Heatmap, Histogram,
    HorizontalBar, Interaction, Line, LinePath, Marginal, Matrix, PlotAdapter, PointInterval, Qq,
    Raincloud, Regression, Scatter, StackedBar, StackedHorizontalBar, Trajectory, Violin,
    Waterfall,
};
pub use input::{
    AxisInput, BooleanInput, CategoryInput, ColumnInput, ColumnsInput, NumericInput,
    NumericSeriesInput, SourceRowsInput,
};
pub use layers::{BarPointLayerAdapter, PointLayerAdapter};
pub use lichtung_plot::*;
pub use significance::*;

#[cfg(test)]
mod tests;
