//! Facet preflight, shared scales, and shared legend coordination.

use crate::Error;

use super::data::FacetPanel;
use crate::builder::Plot;
use crate::decorations::plan::children::MeasurementPhase;
use crate::spec::{AxisDomain, LegendPolicy, PlotKind, PlotRenderContext, PlotRenderResult};
use crate::theme::ResolvedTheme;

pub(super) fn preflight_coordinated_panels(
    source: &Plot,
    base_plot: Plot,
    kind: PlotKind,
    panels: &[FacetPanel],
    theme: &ResolvedTheme,
    children: &mut crate::decorations::plan::children::ChildPlans,
    round: &crate::layout::kernel::FeedbackRound,
) -> Result<(Plot, Vec<std::sync::Arc<PlotRenderResult>>), crate::layout::fit::PlanError> {
    let facet_scales = source.facet_scales_spec();
    let preflight = preflight_panels(
        &base_plot,
        kind,
        panels,
        facet_scales,
        theme,
        children,
        round,
        MeasurementPhase::Probe,
    )?;
    if source.hue_dimension_key().is_none() {
        return Ok((base_plot, preflight));
    }
    let base_plot = coordinate_hue_levels(source, base_plot, &preflight)?;
    let preflight = preflight_panels(
        &base_plot,
        kind,
        panels,
        facet_scales,
        theme,
        children,
        round,
        MeasurementPhase::Hue,
    )?;
    Ok((base_plot, preflight))
}

#[expect(
    clippy::too_many_arguments,
    reason = "root-local child reuse keeps computed hue inputs explicit"
)]
fn preflight_panels(
    base_plot: &Plot,
    kind: PlotKind,
    panels: &[FacetPanel],
    facet_scales: crate::FacetScales,
    theme: &ResolvedTheme,
    children: &mut crate::decorations::plan::children::ChildPlans,
    round: &crate::layout::kernel::FeedbackRound,
    phase: MeasurementPhase,
) -> Result<Vec<std::sync::Arc<PlotRenderResult>>, crate::layout::fit::PlanError> {
    let mut preflight = Vec::new();
    base_plot
        .data()
        .reserve_source(&mut preflight, panels.len())?;
    for (index, facet) in panels.iter().enumerate() {
        let panel_plot = base_plot
            .clone()
            .with_render_scope(facet.render_scope(base_plot.data())?)
            .with_significance_subset(facet.significance_selectors());
        let context = PlotRenderContext::new(
            crate::data::PlotData::with_rows(&panel_plot, &facet.rows, &facet.source_rows),
            facet.panel,
            theme,
        )
        .with_facet_scales(facet_scales);
        let owner = crate::layout::ComponentAddress::root().child(
            crate::layout::AddressRole::FacetCell,
            u32::try_from(index).map_err(|_| Error::Render("facet address overflowed".into()))?,
        )?;
        let result = children.measure(&panel_plot, context, &owner, phase, round, |call| {
            crate::layout::fit::outcome(panel_plot.measure_panel_in_call(kind, context, call))
        })?;
        preflight.push(crate::layout::fit::measured(Ok(result))?);
    }
    Ok(preflight)
}

fn coordinate_hue_levels(
    source: &Plot,
    mut base_plot: Plot,
    preflight: &[std::sync::Arc<PlotRenderResult>],
) -> Result<Plot, Error> {
    if let Some(hue_field) = source.hue_dimension_key() {
        let hue_column = source
            .hue_col()
            .ok_or_else(|| Error::Render("facet hue dimension has no data column".into()))?;
        let mut levels = source.data().source_levels(&crate::data::strict_strings(
            source.data(),
            hue_column,
            "facet hue",
        )?)?;
        let entries = preflight
            .iter()
            .try_fold(0_usize, |count, panel| {
                count.checked_add(panel.legend_entries.len())
            })
            .and_then(|count| count.checked_mul(levels.len()))
            .ok_or_else(|| Error::Render("facet hue coordination work overflow".into()))?;
        source.data().claim_source_visits(entries)?;
        levels.retain(|level| {
            preflight
                .iter()
                .flat_map(|result| result.legend_entries.iter())
                .any(|entry| entry.identity().hue_value_for(hue_field) == Some(level.as_str()))
        });
        if !levels.is_empty() {
            base_plot = base_plot.with_global_hue_levels(levels);
        }
    }
    Ok(base_plot)
}

pub(super) fn shared_legend_entries(
    source: &Plot,
    shared_plot: &Plot,
    preflight: &[std::sync::Arc<PlotRenderResult>],
    theme: &crate::theme::ResolvedTheme,
) -> Result<Vec<crate::decorations::legend::LegendEntry>, Error> {
    let mut entries = if source.legend_policy_spec() == LegendPolicy::Hidden {
        Vec::new()
    } else {
        crate::decorations::legend::merge_exact_entries(
            preflight
                .iter()
                .flat_map(|result| result.legend_entries.iter().cloned()),
            "shared facet legend",
            &theme.prepared_font_environment()?.generation_resources(),
        )?
    };
    if !entries.is_empty() {
        let identity_order = if let Some(pairs) = shared_plot.global_collective_pairs() {
            let mut identities = Vec::new();
            shared_plot
                .data()
                .reserve_source(&mut identities, pairs.len())?;
            for pair in pairs
                .iter()
                .filter(|pair| pair.group.is_some() || pair.hue.is_some())
            {
                identities.push(shared_plot.collective_legend_identity(pair, shared_plot.data())?);
            }
            identities
        } else {
            shared_plot.global_hue_levels().map_or_else(
                || Ok(Vec::new()),
                |levels| {
                    shared_plot.data().collect_source(
                        levels.iter().map(|level| {
                            shared_plot.point_legend_identity(level, shared_plot.data())
                        }),
                    )
                },
            )?
        };
        entries.sort_by_key(|entry| {
            identity_order
                .iter()
                .position(|identity| identity == entry.identity())
                .unwrap_or(usize::MAX)
        });
    }
    Ok(entries)
}

pub(super) fn resolve_shared_facet_axis(
    results: &[std::sync::Arc<PlotRenderResult>],
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
