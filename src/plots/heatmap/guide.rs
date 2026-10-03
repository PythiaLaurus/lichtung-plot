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
    data: crate::data::PlotData<'_>,
    content_plot: Option<&crate::Plot>,
) -> Result<(PanelBox, MeasuredContinuousGuide), PlanError> {
    let semantics = marks.guide_semantics(theme, data)?;
    let indices: &[usize] = if crate::decorations::legend::descriptor::same_value(
        semantics.domain.0,
        semantics.domain.1,
    ) {
        &[0]
    } else if semantics
        .center
        .is_some_and(|center| center > semantics.domain.0 && center < semantics.domain.1)
    {
        &[0, 2, 1]
    } else {
        &[0, 1]
    };
    let descriptor = crate::decorations::legend::descriptor::ContinuousDescriptor {
        field: data.copy_source_string(&marks.color_label)?,
        title: data.copy_source_string(&marks.color_label)?,
        stops: data.collect_source(marks.colorbar_stops.iter().copied().map(Ok))?,
        center: semantics.center,
        missing: semantics.missing,
        domain: semantics.domain,
        exact: true,
        labels: data.collect_source(indices.iter().map(|&index| {
            data.claim_source_visits(1)?;
            data.copy_source_string(&marks.colorbar_values[index])
        }))?,
    };
    crate::layout::interior::guide::continuous_right(
        &descriptor,
        panel,
        minimum,
        theme,
        scope,
        data,
        content_plot,
    )
}

pub(super) fn select(
    guide: &MeasuredContinuousGuide,
    body: PanelBox,
    theme: &ResolvedTheme,
    data: crate::data::PlotData<'_>,
) -> Result<LegendDrawPlan, Error> {
    guide.select(
        body.x1 + theme.legend.outside_gap,
        body.y0 + (body.height() - guide.height) / 2.0,
        data,
    )
}
