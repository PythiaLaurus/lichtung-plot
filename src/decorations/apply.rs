//! Decoration orchestration for single-panel plots.

use crate::Error;

use crate::builder::Plot;
use crate::decorations::plan::ResolvedLegend as PlannedLegend;
use crate::decorations::title_block::TitleBlock;
use crate::render::backend::{with_layer, with_layer_role, Backend, LayerRole};
use crate::spec::PlotRenderResult;
use crate::theme::ResolvedTheme;

/// Draw one panel's grid before its marks.
pub(crate) fn apply_grid(
    chart: &Plot,
    result: &PlotRenderResult,
    palette: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    with_layer_role(
        backend,
        &chart.layer_name("grid"),
        LayerRole::Grid,
        |backend| result.layout.draw_grid_below(palette, backend),
    )
}

/// Draw one panel's axes after its marks.
pub(crate) fn apply_axes(
    chart: &Plot,
    result: &PlotRenderResult,
    palette: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    with_layer(backend, &chart.layer_name("axes"), |backend| {
        result.layout.draw_decorations(palette, backend)
    })
}

pub(crate) fn apply_canvas_text(
    chart: &Plot,
    palette: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let mut title_block = TitleBlock::new().font_family(palette.font_family.clone());
    if palette.show_title {
        if let Some(title) = chart.title_text() {
            title_block = title_block.title(title.to_owned());
        }
    }
    if palette.show_subtitle {
        if let Some(subtitle) = chart.subtitle_text() {
            title_block = title_block.subtitle(subtitle.to_owned());
        }
    }
    if palette.show_caption {
        if let Some(caption) = chart.caption_text() {
            title_block = title_block.caption(caption.to_owned());
        }
    }
    if title_block.has_content() {
        title_block.draw(
            backend,
            chart.width(),
            &chart.measured_canvas_layout()?,
            palette,
        )?;
    }
    Ok(())
}

/// Draw a legend whose dimensions and position were frozen during preflight.
pub(crate) fn apply_planned_legend(
    chart: &Plot,
    legend: Option<&PlannedLegend>,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let Some(resolved) = legend else {
        return Ok(());
    };
    let (x, y) = resolved.bounds.anchor();
    with_layer(backend, &chart.layer_name("legend"), |backend| {
        resolved.legend.draw(backend, x, y)
    })
}
