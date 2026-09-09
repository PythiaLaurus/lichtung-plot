//! Complete interval estimates with an explicit scientific null.

use crate::render::backend::Backend;
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::Plot;

pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    crate::plots::interval_estimate::render(
        chart,
        context,
        backend,
        "forest",
        true,
        chart.forest_reference_value(),
    )
}
