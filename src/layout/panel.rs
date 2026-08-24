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
//! `ThemePalette` layout grammar (frame on/off, grid on/off, etc.)
//! after each plot renderer returns, so callers get a
//! publication-quality figure by default.

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
mod geometry;
mod projection;
mod range_ticks;

pub use geometry::deterministic_text_width;
pub(crate) use geometry::{estimated_text_width, y_axis_left_inset_for_tick_labels};
pub(crate) use range_ticks::{
    anchored_auto_range, automatic_numeric_ticks, continuous_auto_range, display_axis_spec,
    fit_automatic_tick_candidates, format_tick, retain_automatic_ticks_inside_panel,
};
#[cfg(test)]
pub(crate) use range_ticks::{nice_ticks, AUTOMATIC_TICK_TARGET};

#[cfg(test)]
use range_ticks::{distinct_tick_labels, labels_are_unique, nice_range};

/// Axis scale transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    /// Linear data-to-pixel mapping.
    Linear,
}

/// User-configurable axis behavior.
#[derive(Debug, Clone, PartialEq)]
pub struct AxisSpec {
    /// Axis scale transform.
    pub scale: Scale,
    /// Explicit visible data range. When `None`, the renderer computes one.
    pub limits: Option<(f64, f64)>,
}

/// A single tick on an axis: a data-space position and the
/// string label to render next to it.
///
/// Replaces the previous parallel `tick_positions: Vec<f64>` +
/// `tick_labels: Vec<String>` design so categorical labels
/// ("setosa", "versicolor") and numeric labels ("0.5", "10k")
/// share the same type.
#[derive(Debug, Clone)]
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
    /// Axis label (e.g. "petal_length (cm)" or "species").
    pub label: String,
    /// Tick positions + labels in data space.
    pub ticks: Vec<AxisTick>,
    /// Whether this axis is log scale.
    pub log_scale: bool,
}

/// A four-sided frame for a plot area.
#[derive(Debug, Clone, Copy)]
pub struct Frame;

/// Grid lines for a plot area.
#[derive(Debug, Clone, Copy)]
pub struct Grid;

/// A 2D data-to-pixel projection.
#[derive(Debug, Clone)]
pub struct Projection {
    /// Data-space bounding box `(x_min, x_max, y_min, y_max)`.
    x_range: (f64, f64),
    y_range: (f64, f64),
    /// Math-style canvas bounding box `(x0, x1, y0, y1)`: `x0 < x1`,
    /// `y0 < y1`, with `y0` at the panel bottom and `y1` at the top.
    pixel_box: (f64, f64, f64, f64),
}

/// A complete plot layout: scales, projection, frame, grid, axes.
#[derive(Debug, Clone)]
pub struct Layout {
    /// The data-to-pixel projection.
    pub projection: Projection,
    /// The plot frame.
    pub frame: Frame,
    /// The plot grid.
    pub grid: Grid,
    /// The x-axis.
    pub x_axis: Axis,
    /// The y-axis.
    pub y_axis: Axis,
}

#[cfg(test)]
mod tests;
