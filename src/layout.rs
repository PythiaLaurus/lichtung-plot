//! Shared plot and figure layout modules.

/// Canvas-level layout reservations and panel placement.
pub mod figure;
/// Single-panel layout primitives: projection, axes, frame, grid, and ticks.
mod panel;
mod text;

pub use figure::{FigureInsets, FigureLayout, FigureLayoutSpec, PanelBox};
pub(crate) use panel::{
    anchored_auto_range, automatic_numeric_ticks, continuous_auto_range, display_axis_spec,
    estimated_text_width, fit_automatic_tick_candidates, format_tick,
    retain_automatic_ticks_inside_panel, y_axis_left_inset_for_tick_labels,
};
pub use panel::{
    deterministic_text_width, Axis, AxisSide, AxisSpec, AxisTick, Frame, Grid, Layout, Projection,
    Scale,
};
#[cfg(test)]
pub(crate) use panel::{nice_ticks, AUTOMATIC_TICK_TARGET};
pub(crate) use text::TextBlock;
