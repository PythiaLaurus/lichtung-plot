//! Model coefficient estimates with optional drawable bounds.

use crate::Error;

use crate::render::backend::Backend;
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::Plot;

pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    crate::plots::interval_estimate::render(
        chart,
        context,
        backend,
        "coefficient",
        false,
        chart.coefficient_reference_line_value().unwrap_or(0.0),
    )
}
