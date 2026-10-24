//! The independent cell/estimate/total common-text block outside scientific marks.

use super::slot::ExplanationPlan;
use crate::layout::fit::{FitKind, FitScope, PlanError};
use crate::layout::interior::preference::panel;
use crate::render::backend::PlacedText;
use crate::spec::PlotRenderContext;
use crate::theme::ResolvedTheme;
use crate::{Error, Plot, PlotKind, Result, Text, TextTheme};

fn role<'a>(plot: &Plot, theme: &'a ResolvedTheme) -> Result<&'a TextTheme> {
    match plot.kind() {
        PlotKind::Heatmap => theme.contrasting_text(theme.surfaces.get_plot_panel()),
        PlotKind::Waterfall => Ok(&theme.typography.annotations.waterfall_total),
        _ => Ok(&theme.typography.annotations.estimate_detail),
    }
}

fn sources(plot: &Plot, mut visit: impl FnMut(&Text) -> Result<()>) -> Result<()> {
    match plot.kind() {
        PlotKind::Heatmap => {
            if let Some(inputs) = &plot.prepared_heatmap {
                if let Some(text) =
                    panel(&inputs.panels, plot.axis_scope(), "Heatmap")?.numeric_explanation()
                {
                    visit(text)?;
                }
            }
        }
        PlotKind::Coefficient => {
            if let Some(inputs) = &plot.prepared_estimate {
                if let Some(text) =
                    panel(&inputs.panels, plot.axis_scope(), "Coefficient")?.numeric_explanation()
                {
                    visit(text)?;
                }
            }
        }
        PlotKind::Waterfall => {
            if let Some(inputs) = &plot.prepared_waterfall {
                for text in
                    panel(&inputs.panels, plot.axis_scope(), "Waterfall")?.total_explanations()
                {
                    visit(text)?;
                }
            }
        }
        _ => (),
    }
    Ok(())
}

fn source<'a>(plot: &'a Plot, context: PlotRenderContext<'_>) -> Result<Option<&'a Text>> {
    match plot.kind() {
        PlotKind::Heatmap => plot
            .prepared_heatmap
            .as_ref()
            .map(|inputs| {
                Ok(panel(&inputs.panels, plot.axis_scope(), "Heatmap")?.numeric_explanation())
            })
            .transpose()
            .map(Option::flatten),
        PlotKind::Coefficient => plot
            .prepared_estimate
            .as_ref()
            .map(|inputs| {
                Ok(panel(&inputs.panels, plot.axis_scope(), "Coefficient")?.numeric_explanation())
            })
            .transpose()
            .map(Option::flatten),
        PlotKind::Waterfall => {
            let Some(inputs) = &plot.prepared_waterfall else {
                return Ok(None);
            };
            let axes = plot
                .prepared_axis_inputs()
                .ok_or_else(|| Error::Render("total explanation lacks prepared axes".into()))?;
            let x = context.axis_range(crate::layout::AxisSide::Bottom, || Ok(axes.ranges[0]))?;
            let y = context.axis_range(crate::layout::AxisSide::Left, || Ok(axes.ranges[1]))?;
            panel(&inputs.panels, plot.axis_scope(), "Waterfall")?.total_explanation(
                &crate::layout::Projection::new(x, y, context.panel.as_tuple()),
            )
        }
        _ => Ok(None),
    }
}

fn gap(theme: &ResolvedTheme) -> Result<f64> {
    theme
        .panel_edge_length()?
        .pixels_rounded(crate::layout::kernel::SpatialRounding::Ceil)
}

pub(crate) fn preferred_width(plot: &Plot, theme: &ResolvedTheme) -> Result<f64> {
    let mut width = 0.0_f64;
    sources(plot, |text| {
        width = width.max(super::slot::preferred_width(
            text,
            role(plot, theme)?,
            theme,
        )?);
        Ok(())
    })?;
    Ok(width.max(axis_preferences(plot, theme)?.0))
}

pub(crate) fn axis_preferences(plot: &Plot, theme: &ResolvedTheme) -> Result<(f64, f64)> {
    let mut preferred = [0.0_f64; 2];
    // The axis owner later measures the actual selected set. Intrinsic probes
    // include common explanations without inventing scientific tick positions.
    if let Some(axes) = plot.prepared_axis_inputs() {
        for (side, preferred) in preferred.iter_mut().enumerate() {
            if axes.specs[side].exact_ticks.is_some() || axes.categories[side].is_some() {
                continue;
            }
            if let Some(set) = axes.numeric_probe(side)? {
                if let Some(text) = set.explanation {
                    let preference = super::slot::preferred_width(
                        &text,
                        &theme.typography.axes.tick_label,
                        theme,
                    )?;
                    *preferred = preferred.max(preference);
                }
            }
        }
    }
    Ok((preferred[0], preferred[1]))
}

pub(crate) fn natural_height(plot: &Plot, theme: &ResolvedTheme, width: f64) -> Result<f64> {
    let mut height = 0.0_f64;
    sources(plot, |text| {
        let plan = ExplanationPlan::measure(
            text,
            role(plot, theme)?,
            width,
            crate::TextAlignment::Start,
            theme,
            &FitScope::root(),
        )
        .map_err(PlanError::into_error)?;
        height = height.max(plan.height + gap(theme)?);
        Ok(())
    })?;
    Ok(height)
}

pub(crate) fn reserve<'a>(
    plot: &Plot,
    context: PlotRenderContext<'a>,
) -> std::result::Result<(PlotRenderContext<'a>, Option<PlacedText>), PlanError> {
    let Some(text) = source(plot, context)? else {
        return Ok((context, None));
    };
    let plan = ExplanationPlan::measure(
        text,
        role(plot, context.theme)?,
        context.panel.width(),
        crate::TextAlignment::Start,
        context.theme,
        context.fit_scope(),
    )?;
    let mut body = context.panel;
    body.y1 -= plan.height + gap(context.theme)?;
    if body.height() <= 0.0 {
        return Err(context
            .fit_scope()
            .failure(
                FitKind::Text,
                "required numeric explanation leaves no mark body",
            )
            .into());
    }
    let text = plan.at(context.panel.x0, context.panel.y1);
    Ok((context.with_panel(body), Some(text)))
}
