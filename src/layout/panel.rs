//! `lichtung-plot::layout` — shared plot layout core.
//!
//! This module centralizes the geometry of a single plot panel:
//! data range, pixel box, projection (data→pixel), the spines
//! (Frame), the gridlines, and the x/y axes (tick positions, tick
//! labels, axis labels).
//!
//! Every plot renderer consumes a `Layout` and only talks to the `Backend`
//! through primitive draw calls. Per the theme-driven defaults
//! policy, the `Plot` render orchestrator applies the theme's
//! resolved-theme layout grammar (frame on/off, grid on/off, etc.)
//! after each plot renderer returns, so callers get a
//! publication-quality canvas by default.

// `Backend::draw_text` takes `u32` pixel coords, so f64 → u32
// rounding is unavoidable when emitting text. The `usize → f64`
// and `usize → u32` casts only occur for indices that are
// bounded by the canvas size (≤ a few thousand), well within
// `f64`/`u32` precision for chart coordinates.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_lossless
)]

mod axis_frame_grid;
mod cells;
mod geometry;
mod projection;
mod range_ticks;
mod semantics;
mod sources;
#[cfg(test)]
use semantics::RangePolicy;
pub(crate) use semantics::{AxisSemanticInputs, RangeProvenance, StandardPanelInputs};
pub(crate) use sources::format_guide_label;
pub(crate) use sources::format_label;
pub(crate) use sources::FOREST_NUMERIC_ALPHABET;
pub(crate) use sources::NUMERIC_ALPHABET;
pub(crate) use sources::{numeric_tick_plan, NumericTickPlan, TickSource};

pub(crate) use axis_frame_grid::PanelAxisPlan;
pub(crate) use cells::LABEL_CLEARANCE;
pub(crate) use range_ticks::{anchored_auto_range, continuous_auto_range, display_axis_spec};
#[cfg(test)]
pub(crate) use range_ticks::{automatic_numeric_ticks, nice_ticks};

#[cfg(test)]
use range_ticks::{distinct_tick_labels, labels_are_unique, nice_range};

/// User-configurable axis behavior.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AxisSpec {
    pub(crate) provenance: RangeProvenance,
    /// Explicit visible data range. When `None`, the renderer computes one.
    pub limits: Option<(f64, f64)>,
    /// Exact numeric tick source, selected before any automatic text is measured.
    pub(crate) exact_ticks: Option<Vec<AxisTick>>,
    pub(crate) generation: crate::TickGeneration,
    pub(crate) format: Option<std::sync::Arc<crate::numeric::format::NumericFormat>>,
}

/// A single tick on an axis: a data-space position and the
/// string label to render next to it.
///
/// Replaces the previous parallel `tick_positions: Vec<f64>` +
/// `tick_labels: Vec<String>` design so categorical labels
/// ("setosa", "versicolor") and numeric labels ("0.5", "10k")
/// share the same type.
#[derive(Debug, Clone, PartialEq)]
pub struct AxisTick {
    /// Data-space position of the tick.
    pub position: f64,
    /// String label to render.
    pub label: String,
}

/// One side of the plot panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisSide {
    /// Bottom (x) axis.
    Bottom,
    /// Left (y) axis.
    Left,
}

/// An axis: a label plus a list of `AxisTick`s to draw on it.
///
/// `ticks` is in data-space positions (which the `Projection`
/// converts to pixel coordinates). Labels are rendered verbatim
/// (no formatting), so callers can put numeric or categorical
/// labels in the same vector.
#[derive(Debug, Clone)]
pub struct Axis {
    /// Present axis title, including an explicitly authored empty line.
    pub label: Option<crate::Text>,
    /// Tick positions + labels in data space.
    pub ticks: Vec<AxisTick>,
    /// Owner-level suppression, separate from an explicitly empty display label.
    pub(crate) tick_labels_visible: bool,
    pub(crate) explicit_ticks: bool,
    pub(crate) assigned_tick_width: Option<f64>,
    /// Selected category track edges in panel coordinates, when a producer owns them.
    pub(crate) assigned_cells: Option<Vec<(f64, f64)>>,
    pub(crate) source: TickSource,
}

/// A 2D data-to-pixel projection.
#[derive(Debug, Clone)]
pub struct Projection {
    /// Data-space bounding box `(x_min, x_max, y_min, y_max)`.
    x_range: (f64, f64),
    y_range: (f64, f64),
    /// Math-style canvas bounding box `(x0, x1, y0, y1)`: `x0 < x1`,
    /// `y0 < y1`, with `y0` at the panel bottom and `y1` at the top.
    pixel_box: (f64, f64, f64, f64),
    category_counts: [Option<usize>; 2],
}

/// A complete plot layout: scales, projection, frame, grid, axes.
#[derive(Debug, Clone)]
pub struct Layout {
    /// The data-to-pixel projection.
    pub projection: Projection,
    /// The x-axis.
    pub x_axis: Axis,
    /// The y-axis.
    pub y_axis: Axis,
}

#[cfg(test)]
mod tests;
