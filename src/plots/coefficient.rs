//! Model coefficient estimates with optional genuine intervals.

use crate::Error;

use crate::render::backend::Backend;
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::Chart;

pub(crate) fn render(
    chart: &Chart<'_>,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    crate::plots::interval_estimate::render(chart, context, backend, "coefficient", false, false)
}
