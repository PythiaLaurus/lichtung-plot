//! Private plot and canvas layout modules.

pub(crate) mod canvas;
pub(crate) mod fit;
pub(crate) mod interior;
pub(crate) mod kernel;
mod panel;
mod text;
mod unicode16;

pub(crate) use canvas::{LegacyRootAxes, PanelBox, RootFramePlan};
#[cfg(test)]
pub(crate) use kernel::{force_reservation_failure, ReservationStage};
pub(crate) use kernel::{plan_exact_root, FrozenRoot, MeasuredLayout, RootSize};
pub(crate) use kernel::{AddressRole, ComponentAddress};
#[cfg(test)]
pub(crate) use panel::nice_ticks;
pub use panel::AxisTick;
#[cfg(test)]
pub(crate) use panel::TickSource;
pub(crate) use panel::NUMERIC_ALPHABET;
pub(crate) use panel::{
    anchored_auto_range, continuous_auto_range, display_axis_spec, numeric_tick_plan,
    y_axis_left_inset_for_tick_labels,
};
pub(crate) use panel::{Axis, AxisSide, AxisSpec, Layout, PanelAxisPlan, Projection};
pub(crate) use panel::{NumericTickPlan, TickInterval};
pub(crate) use panel::{RangeProvenance, StandardPanelInputs};
#[allow(unused_imports)]
pub(crate) use text::{
    foundational_plot_requirements, maximum_width_quanta, DeferredAnnotationTextPlan,
    PhysicalTextAlignment, PlannedGlyph, PreparedFontPairRequirement, PreparedTextCensus,
    PreparedTextRequirement, PreparedTextRole, ResolvedTextPlan, ShapedTextRun, TextBlock,
    TextFitMiss, TextLineHeight, TextLinePlan, TextPlanError, TextPlanInput, TextPlanProvenance,
    TextSourceSpan, CSS_PIXEL_QUANTA,
};
