//! Facet preflight, shared scales, and shared legend coordination.

use crate::Error;

use super::data::{stable_levels, FacetPanel};
use crate::builder::Plot;
use crate::decorations::plan::ResolvedLegend;
use crate::layout::PanelBox;
use crate::render::backend::Backend;
use crate::spec::{
    AxisDomain, LegendPolicy, LegendPosition, PlotKind, PlotRenderContext, PlotRenderResult,
};
use crate::theme::ResolvedTheme;

pub(super) fn preflight_coordinated_panels(
    source: &Plot,
    base_plot: Plot,
    kind: PlotKind,
    panels: &[FacetPanel],
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(Plot, Vec<PlotRenderResult>), Error> {
    let facet_scales = source.facet_scales_spec();
    let preflight = preflight_panels(&base_plot, kind, panels, facet_scales, theme, backend)?;
    if source.hue_dimension_key().is_none() {
        return Ok((base_plot, preflight));
    }
    let base_plot = coordinate_hue_levels(source, base_plot, &preflight)?;
    let preflight = preflight_panels(&base_plot, kind, panels, facet_scales, theme, backend)?;
    Ok((base_plot, preflight))
}

fn preflight_panels(
    base_plot: &Plot,
    kind: PlotKind,
    panels: &[FacetPanel],
    facet_scales: crate::FacetScales,
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<Vec<PlotRenderResult>, Error> {
    let mut preflight = Vec::with_capacity(panels.len());
    for facet in panels {
        let panel_plot = base_plot
            .clone()
            .with_render_scope(facet.render_scope())
            .with_significance_subset(facet.significance_selectors());
        preflight.push(
            panel_plot.render_panel(
                kind,
                PlotRenderContext::new(
                    crate::data::PlotData::with_rows(&panel_plot, &facet.rows, &facet.source_rows),
                    facet.panel,
                    theme,
                )
                .with_facet_scales(facet_scales),
                backend,
            )?,
        );
    }
    Ok(preflight)
}

fn coordinate_hue_levels(
    source: &Plot,
    mut base_plot: Plot,
    preflight: &[PlotRenderResult],
) -> Result<Plot, Error> {
    if let Some(hue_field) = source.hue_dimension_key() {
        let rendered_levels = stable_levels(
            preflight
                .iter()
                .flat_map(|result| result.legend_entries.iter())
                .filter_map(|entry| entry.identity().hue_value_for(hue_field))
                .map(str::to_owned)
                .collect(),
        );
        let hue_column = source
            .hue_col()
            .ok_or_else(|| Error::Render("facet hue dimension has no data column".into()))?;
        let levels = stable_levels(crate::data::strict_strings(
            source.data(),
            hue_column,
            "facet hue",
        )?)
        .into_iter()
        .filter(|level| rendered_levels.contains(level))
        .collect::<Vec<_>>();
        if !levels.is_empty() {
            base_plot = base_plot.with_global_hue_levels(levels);
        }
    }
    Ok(base_plot)
}

pub(super) fn reserve_shared_legend(
    source: &Plot,
    shared_plot: &Plot,
    preflight: &[PlotRenderResult],
    mut outer: PanelBox,
) -> Result<
    (
        PanelBox,
        Option<ResolvedLegend>,
        Vec<crate::decorations::legend::LegendEntry>,
    ),
    Error,
> {
    let mut outside_legend = None;
    let legend_position = source.legend_position_spec();
    let facet_legend_position = if legend_position == LegendPosition::Auto {
        Some(LegendPosition::OutsideRightTop)
    } else if crate::decorations::plan::is_outside(legend_position) {
        Some(legend_position)
    } else {
        None
    };
    let mut entries = if source.legend_policy_spec() == LegendPolicy::Hidden {
        Vec::new()
    } else {
        crate::decorations::legend::merge_exact_entries(
            preflight
                .iter()
                .flat_map(|result| result.legend_entries.iter().cloned()),
            "shared facet legend",
        )?
    };
    if !entries.is_empty() {
        let identity_order = if let Some(pairs) = shared_plot.global_collective_pairs() {
            pairs
                .iter()
                .filter(|pair| pair.group.is_some() || pair.hue.is_some())
                .map(|pair| shared_plot.collective_legend_identity(pair))
                .collect::<Result<Vec<_>, Error>>()?
        } else {
            shared_plot
                .global_hue_levels()
                .map_or_else(Vec::new, |levels| {
                    levels
                        .iter()
                        .map(|level| shared_plot.legend_identity(level.clone()))
                        .collect()
                })
        };
        entries.sort_by_key(|entry| {
            identity_order
                .iter()
                .position(|identity| identity == entry.identity())
                .unwrap_or(usize::MAX)
        });
        if let Some(facet_legend_position) = facet_legend_position {
            let legend_plot = shared_plot
                .clone()
                .with_legend_position(facet_legend_position);
            let (reserved, legend) =
                crate::decorations::plan::plan_outside_legend(&legend_plot, &entries, outer)?;
            outer = reserved;
            outside_legend = Some(legend);
        }
    }
    Ok((outer, outside_legend, entries))
}

pub(super) fn resolve_shared_facet_axis(
    results: &[PlotRenderResult],
    shared: bool,
    x_axis: bool,
) -> Result<Option<(f64, f64)>, Error> {
    if !shared || results.is_empty() {
        return Ok(None);
    }
    let domain = if x_axis {
        results[0].x_domain
    } else {
        results[0].y_domain
    };
    if results.iter().any(|result| {
        let candidate = if x_axis {
            result.x_domain
        } else {
            result.y_domain
        };
        candidate != domain
    }) {
        return Err(Error::Render(
            "fixed facet scales cannot combine categorical and continuous domains".into(),
        ));
    }
    if domain == AxisDomain::Categorical {
        let expected = facet_axis_signature(&results[0], x_axis);
        if results
            .iter()
            .skip(1)
            .any(|result| facet_axis_signature(result, x_axis) != expected)
        {
            return Err(Error::Render(format!(
                "fixed facet {} axis requires identical categorical levels and ordering in every panel; use a free scale or provide complete panel data",
                if x_axis { "x" } else { "y" }
            )));
        }
        return Ok(None);
    }
    let mut minimum = f64::INFINITY;
    let mut maximum = f64::NEG_INFINITY;
    for result in results {
        let range = if x_axis {
            result.layout.projection.x_range()
        } else {
            result.layout.projection.y_range()
        };
        minimum = minimum.min(range.0);
        maximum = maximum.max(range.1);
    }
    if minimum.is_finite() && maximum.is_finite() && minimum < maximum {
        Ok(Some((minimum, maximum)))
    } else {
        Err(Error::Render(
            "renderer preflight did not produce a finite shared facet domain".into(),
        ))
    }
}

fn facet_axis_signature(result: &PlotRenderResult, x_axis: bool) -> Vec<(u64, String)> {
    let ticks = if x_axis {
        &result.layout.x_axis.ticks
    } else {
        &result.layout.y_axis.ticks
    };
    let keys = if x_axis {
        result.x_tick_keys.as_deref()
    } else {
        result.y_tick_keys.as_deref()
    };
    ticks
        .iter()
        .enumerate()
        .map(|(index, tick)| {
            (
                tick.position.to_bits(),
                keys.and_then(|keys| keys.get(index))
                    .cloned()
                    .unwrap_or_else(|| tick.label.clone()),
            )
        })
        .collect()
}
