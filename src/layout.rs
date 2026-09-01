//! Private plot and canvas layout modules.

pub(crate) mod canvas;
mod kernel;
mod panel;
mod text;

pub(crate) use canvas::{CanvasLayout, CanvasLayoutSpec, PanelBox};
#[cfg(test)]
pub(crate) use kernel::force_reservation_failure;
pub(crate) use kernel::{plan_exact_root, SelectedRoot};
pub use panel::AxisTick;
pub(crate) use panel::{
    anchored_auto_range, automatic_numeric_ticks, continuous_auto_range, display_axis_spec,
    estimated_text_width, fit_automatic_tick_candidates, format_tick,
    retain_automatic_ticks_inside_panel, y_axis_left_inset_for_tick_labels,
};
pub(crate) use panel::{deterministic_text_width, Axis, AxisSide, AxisSpec, Layout, Projection};
#[cfg(test)]
pub(crate) use panel::{nice_ticks, AUTOMATIC_TICK_TARGET};
pub(crate) use text::TextBlock;
