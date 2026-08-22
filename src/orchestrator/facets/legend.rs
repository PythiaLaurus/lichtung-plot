//! Facet preflight, shared scales, and shared legend coordination.

use crate::Error;

use super::data::{stable_levels, FacetPanel};
use crate::builder::Plot;
use crate::decorations::plan::ResolvedLegend;
use crate::layout::PanelBox;
use crate::render::backend::Backend;
use crate::spec::{AxisDomain, LegendPosition, PlotKind, PlotRenderContext, PlotRenderResult};

pub(super) fn preflight_panels(
    chart: &Plot,
    kind: PlotKind,
    panels: &[FacetPanel],
    backend: &mut dyn Backend,
) -> Result<Vec<PlotRenderResult>, Error> {
    let mut preflight = Vec::with_capacity(panels.len());
    for facet in panels {
        let panel_chart = chart
            .clone()
            .with_render_scope(facet.render_scope())
            .with_significance_subset(facet.significance_selectors())?;
        preflight.push(panel_chart.render_panel(
            kind,
            PlotRenderContext::new(
                crate::data::PlotData::with_rows(&panel_chart, &facet.rows, &facet.source_rows),
                facet.panel,
            ),
            backend,
        )?);
    }
    Ok(preflight)
}

pub(super) fn coordinate_hue_levels(
    source: &Plot,
    mut chart: Plot,
    preflight: &[PlotRenderResult],
) -> Result<Plot, Error> {
    if let Some(hue_column) = source.hue_col() {
        let rendered_levels = stable_levels(
            preflight
                .iter()
                .flat_map(|result| result.legend_keys.iter())
                .cloned()
                .collect(),
        );
        let levels = stable_levels(crate::data::strict_strings(
            source.data(),
            hue_column,
            "facet hue",
        )?)
        .into_iter()
        .filter(|level| rendered_levels.contains(level))
        .collect::<Vec<_>>();
        if !levels.is_empty() {
            chart = chart.with_global_hue_levels(levels);
        }
    }
    Ok(chart)
}

pub(super) fn reserve_shared_legend(
    source: &Plot,
    shared: &Plot,
    preflight: &[PlotRenderResult],
    mut outer: PanelBox,
) -> Result<(PanelBox, Option<ResolvedLegend>), Error> {
    let mut outside_legend = None;
    let legend_position = source.legend_position_spec();
    let facet_legend_position = if legend_position == LegendPosition::Auto {
        Some(LegendPosition::OutsideRightTop)
    } else if crate::decorations::plan::is_outside(legend_position) {
        Some(legend_position)
    } else {
        None
    };
    if source.legend_visibility() != Some(false) && legend_position != LegendPosition::Hidden {
        if let Some(facet_legend_position) = facet_legend_position {
            let global_entries = shared
                .global_hue_levels()
                .map(|levels| {
                    let palette = shared.palette();
                    let display = shared.resolve_label_domain(
                        "shared facet legend entries",
                        levels.iter().map(String::as_str),
                    )?;
                    Ok::<_, Error>(
                        levels
                            .iter()
                            .zip(display)
                            .enumerate()
                            .map(|(index, (_, label))| {
                                crate::decorations::legend::LegendEntry::point(
                                    label,
                                    shared.point_style(
                                        palette.colors[index % palette.colors.len()],
                                        3,
                                    ),
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .transpose()?;
            let entries = global_entries.as_deref().or_else(|| {
                preflight
                    .iter()
                    .find(|result| !result.legend_entries.is_empty())
                    .map(|result| result.legend_entries.as_slice())
            });
            if let Some(entries) = entries {
                let legend_chart = shared.clone().legend_position(facet_legend_position);
                let (reserved, legend) =
                    crate::decorations::plan::plan_outside_legend(&legend_chart, entries, outer)?;
                outer = reserved;
                outside_legend = Some(legend);
            }
        }
    }
    Ok((outer, outside_legend))
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
