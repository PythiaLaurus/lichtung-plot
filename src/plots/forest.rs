//! Complete interval estimates with an explicit scientific null.

use crate::Error;

use crate::render::backend::Backend;
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::Chart;

pub(crate) fn render(
    chart: &Chart<'_>,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    crate::plots::interval_estimate::render(chart, context, backend, "forest", true, true)
}
