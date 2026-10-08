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
    let indices: &[usize] = if crate::decorations::legend::descriptor::same_value(
        semantics.domain.0,
        semantics.domain.1,
    ) {
        &[0]
    } else if semantics
        .selection
        .center()
        .is_some_and(|center| center > semantics.domain.0 && center < semantics.domain.1)
    {
        &[0, 2, 1]
    } else {
        &[0, 1]
    };
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
            indices.iter().map(|&index| {
                crate::data::source::copy_source_string(&marks.colorbar_values[index])
            }),
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
