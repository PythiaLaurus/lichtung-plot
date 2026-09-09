//! Private plot and canvas layout modules.

pub(crate) mod canvas;
pub(crate) mod fit;
pub(crate) mod kernel;
mod panel;
mod text;
mod unicode16;

pub(crate) use canvas::{CanvasLayout, CanvasLayoutSpec, PanelBox};
#[cfg(test)]
pub(crate) use kernel::{force_reservation_failure, ReservationStage};
pub(crate) use kernel::{plan_exact_root, FrozenRoot, MeasuredLayout, RootSize};
pub(crate) use kernel::{AddressRole, ComponentAddress};
pub use panel::AxisTick;
pub(crate) use panel::{
    anchored_auto_range, automatic_numeric_ticks, automatic_numeric_ticks_legacy,
    continuous_auto_range, display_axis_spec, fit_automatic_tick_candidates, format_tick,
    retain_automatic_ticks_inside_panel_legacy, y_axis_left_inset_for_tick_labels,
};
#[cfg(test)]
pub(crate) use panel::{nice_ticks, AUTOMATIC_TICK_TARGET};
pub(crate) use panel::{Axis, AxisSide, AxisSpec, Layout, PanelAxisTextPlan, Projection};
#[allow(unused_imports)]
pub(crate) use text::{
    foundational_plot_requirements, maximum_width_quanta, DeferredAnnotationTextPlan,
    PhysicalTextAlignment, PlannedGlyph, PreparedFontPairRequirement, PreparedTextCensus,
    PreparedTextRequirement, PreparedTextRole, ResolvedTextPlan, ShapedTextRun, TextBlock,
    TextFitMiss, TextLineHeight, TextLinePlan, TextPlanError, TextPlanInput, TextPlanProvenance,
    TextSourceSpan, CSS_PIXEL_QUANTA,
};
