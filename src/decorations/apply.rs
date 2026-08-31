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
    plot: &Plot,
    result: &PlotRenderResult,
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    with_layer_role(
        backend,
        &plot.layer_name("grid"),
        LayerRole::Grid,
        |backend| result.layout.draw_grid_below(theme, backend),
    )
}

/// Draw one panel's axes after its marks.
pub(crate) fn apply_axes(
    plot: &Plot,
    result: &PlotRenderResult,
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    with_layer(backend, &plot.layer_name("axes"), |backend| {
        result.layout.draw_decorations(theme, backend)
    })
}

pub(crate) fn apply_canvas_text(
    plot: &Plot,
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let mut title_block = TitleBlock::new().font_family(theme.font_family.clone());
    if theme.show_title {
        if let Some(title) = plot.title_text() {
            title_block = title_block.title(title.to_owned());
        }
    }
    if theme.show_subtitle {
        if let Some(subtitle) = plot.subtitle_text() {
            title_block = title_block.subtitle(subtitle.to_owned());
        }
    }
    if theme.show_caption {
        if let Some(caption) = plot.caption_text() {
            title_block = title_block.caption(caption.to_owned());
        }
    }
    if title_block.has_content() {
        title_block.draw(
            backend,
            plot.width(),
            &plot.measured_canvas_layout(theme)?,
            theme,
        )?;
    }
    Ok(())
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
