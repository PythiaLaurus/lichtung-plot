//! Mandatory Heatmap guide measured by the common continuous-guide owner.

use super::HeatmapData;
use crate::decorations::legend::{LegendDrawPlan, MeasuredContinuousGuide};
use crate::layout::fit::{FitScope, PlanError};
use crate::layout::PanelBox;
use crate::theme::ResolvedTheme;
use crate::Error;

pub(super) fn measure(
    marks: &HeatmapData,
    panel: PanelBox,
    minimum: f64,
    theme: &ResolvedTheme,
    scope: &FitScope,

    content_plot: Option<&crate::Plot>,
) -> Result<(PanelBox, MeasuredContinuousGuide), PlanError> {
    let semantics = marks.guide_semantics(theme)?;
    let descriptor = crate::decorations::legend::descriptor::ContinuousDescriptor {
        field: crate::data::source::copy_source_string(&marks.color_label)?,
        title: crate::data::source::copy_source_string(&marks.color_label)?,
        stops: crate::data::source::collect_source(marks.colorbar_stops.iter().copied().map(Ok))?,
        center: semantics.selection.center(),
        selection: semantics.selection,
        missing: semantics.missing,
        domain: semantics.domain,
        exact: true,
        labels: crate::data::source::collect_source(
            marks
                .guide_labels()
                .map(crate::data::source::copy_source_string),
        )?,
    };
    crate::layout::interior::guide::continuous_right(
        &descriptor,
        panel,
        minimum,
        theme,
        scope,
        content_plot,
    )
}

pub(super) fn select(
    guide: &MeasuredContinuousGuide,
    body: PanelBox,
    theme: &ResolvedTheme,
) -> Result<LegendDrawPlan, Error> {
    guide.select(
        body.x1 + theme.legend_outside_gap()?,
        body.y0 + (body.height() - guide.height) / 2.0,
    )
}
