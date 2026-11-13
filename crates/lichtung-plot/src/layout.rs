//! Private plot and canvas layout modules.

pub(crate) mod canvas;
pub(crate) mod fit;
pub(crate) mod interior;
pub(crate) mod kernel;
mod panel;
pub(crate) mod text;
mod unicode16;

pub(crate) use canvas::{PixelRect, RootFramePlan};
#[cfg(test)]
pub(crate) use kernel::{force_reservation_failure, ReservationStage};
pub(crate) use kernel::{plan_root, FrozenRoot, MeasuredLayout, RootRequest, RootSize};
pub(crate) use kernel::{AddressRole, ComponentAddress};
pub(crate) use panel::category_label_span;
pub(crate) use panel::fixed_interval_is_empty;
pub(crate) use panel::format_guide_label;
pub(crate) use panel::layout_axis_ticks;
#[cfg(test)]
pub(crate) use panel::numeric_tick_plan;
pub use panel::AxisTick;
pub(crate) use panel::NumericTickPlan;
pub(crate) use panel::TickSource;
pub(crate) use panel::LABEL_CLEARANCE;
pub(crate) use panel::{anchored_auto_range, continuous_auto_range, display_axis_spec};
pub(crate) use panel::{
    range_policies, AxisSemanticInputs, CoordinateLayoutInputs, RangePolicy, RangeProvenance,
};
pub(crate) use panel::{Axis, AxisSide, AxisSpec, AxisSystemPlan, Layout, Projection};
#[allow(unused_imports)]
pub(crate) use text::{
    foundational_plot_requirements, maximum_width_quanta, DeferredAnnotationTextPlan,
    PhysicalTextAlignment, PlannedGlyph, PreparedFontPairRequirement, PreparedTextCensus,
    PreparedTextRequirement, PreparedTextRole, ResolvedTextPlan, ShapedTextRun, TextBlock,
    TextFitMiss, TextLineHeight, TextLinePlan, TextPlanError, TextPlanInput, TextPlanProvenance,
    TextSourceSpan, CSS_PIXEL_QUANTA,
};
