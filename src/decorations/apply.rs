//! Decoration orchestration for single-panel plots.

use crate::Error;

use crate::builder::Plot;
use crate::decorations::plan::{PanelPlan, ResolvedLegend as PlannedLegend};
use crate::layout::CanvasLayout;
use crate::render::backend::{with_layer, with_layer_role, Backend, LayerRole};

/// Draw one panel's grid before its marks.
pub(crate) fn apply_grid(
    plot: &Plot,
    plan: &PanelPlan,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    with_layer_role(
        backend,
        &plot.layer_name("grid"),
        LayerRole::Grid,
        |backend| {
            plan.result
                .axes
                .as_ref()
                .ok_or_else(|| {
                    Error::Render("standard panel plan is missing immutable axes".into())
                })?
                .draw_grid(backend)
        },
    )
}

/// Draw one panel's axes after its marks.
pub(crate) fn apply_axes(
    plot: &Plot,
    plan: &PanelPlan,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    with_layer(backend, &plot.layer_name("axes"), |backend| {
        let axes =
            plan.result.axes.as_ref().ok_or_else(|| {
                Error::Render("standard panel plan is missing immutable axes".into())
            })?;
        axes.draw(backend)
    })
}

pub(crate) fn apply_canvas_text(
    plot: &Plot,
    canvas: &CanvasLayout,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    if let crate::layout::canvas::CanvasRootText::Planned(plan) = &canvas.root_text {
        return plan.draw(
            [plot.title_text(), plot.subtitle_text(), plot.caption_text()],
            backend,
        );
    }
    Err(Error::Render("root title block was not measured".into()))
}

/// Draw a legend whose dimensions and position were frozen during preflight.
pub(crate) fn apply_planned_legend(
    plot: &Plot,
    legend: Option<&PlannedLegend>,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let Some(resolved) = legend else {
        return Ok(());
    };
    let (x, y) = resolved.bounds.anchor();
    with_layer(backend, &plot.layer_name("legend"), |backend| {
        resolved.legend.draw(backend, x, y)
    })
}
