//! Native Heatmap guide measured by the single sectioned-guide owner.

use crate::decorations::legend::{Legend, LegendDrawPlan};
use crate::layout::fit::{FitScope, PlanError};
use crate::layout::PixelRect;
use crate::Error;

pub(super) fn measure(
    chart: &crate::Plot,
    panel: PixelRect,
    minimum: f64,
    theme: &crate::text::TextRequestContext<'_, '_>,
    scope: &FitScope,

    content_plot: Option<&crate::Plot>,
) -> Result<(PixelRect, Legend), PlanError> {
    let descriptor = chart
        .prepared_discrete_guide()?
        .ok_or_else(|| Error::Render("native Heatmap guide was not prepared".into()))?;
    crate::layout::interior::guide::native_right(
        descriptor,
        panel,
        minimum,
        theme,
        scope,
        content_plot,
    )
}

pub(super) fn select(
    guide: &Legend,
    body: PixelRect,
    theme: &crate::text::TextRequestContext<'_, '_>,
) -> Result<LegendDrawPlan, Error> {
    let bounds = guide.box_at(
        body.x1 + theme.legend_outside_gap()?,
        body.y0 + (body.height() - guide.compute_box_height()) / 2.0,
    );
    let (x, y) = bounds.anchor();
    guide.select(x, y)
}
