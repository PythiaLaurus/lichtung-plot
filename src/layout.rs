//! Shared plot and figure layout modules.

/// Canvas-level layout reservations and panel placement.
pub mod figure;
/// Single-panel layout primitives: projection, axes, frame, grid, and ticks.
mod panel;
mod text;

pub use figure::{FigureInsets, FigureLayout, FigureLayoutSpec, PanelBox};
pub(crate) use panel::{
    anchored_auto_range, continuous_auto_range, display_axis_spec, estimated_text_width,
    format_tick, nice_ticks, y_axis_left_inset_for_tick_labels,
};
pub use panel::{
    deterministic_text_width, Axis, AxisSide, AxisSpec, AxisTick, Frame, Grid, Layout, Projection,
    Scale,
};
pub(crate) use text::TextBlock;
