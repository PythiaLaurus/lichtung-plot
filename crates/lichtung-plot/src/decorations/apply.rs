//! Decoration orchestration for single-panel plots.

use crate::Error;

use crate::decorations::plan::MeasuredPlotPlan;
use crate::render::backend::{with_layer, with_layer_role, Backend, LayerRole};

/// Draw one panel's grid before its marks.
pub(crate) fn apply_grid(
    layer: &str,
    plan: &MeasuredPlotPlan,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    with_layer_role(backend, layer, LayerRole::Grid, |backend| {
        plan.result
            .axes
            .as_ref()
            .ok_or_else(|| Error::Render("standard panel plan is missing immutable axes".into()))?
            .draw_grid(backend)
    })
}

/// Draw one panel's axes after its marks.
pub(crate) fn apply_axes(
    layer: &str,
    plan: &MeasuredPlotPlan,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    with_layer(backend, layer, |backend| {
        let axes =
            plan.result.axes.as_ref().ok_or_else(|| {
                Error::Render("standard panel plan is missing immutable axes".into())
            })?;
        axes.draw(backend)
    })
}
