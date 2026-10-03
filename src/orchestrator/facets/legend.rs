//! Facet preflight, shared scales, and shared legend coordination.

use crate::Error;

use super::data::FacetPanel;
use crate::builder::Plot;
use crate::decorations::plan::children::MeasurementPhase;
use crate::spec::{AxisDomain, LegendPolicy, PlotKind, PlotRenderContext, PlotRenderResult};
use crate::theme::ResolvedTheme;

/// Prepare the real shared discrete identity and glyph set at the root's
/// semantic boundary, before any font census or panel assignment.
pub(super) fn prepare_discrete_guide(
    source: &Plot,
    kind: PlotKind,
    theme: &ResolvedTheme,
    inputs: &crate::layout::interior::PreparedInteriorInputs,
    resources: &crate::font::canonical::OperationGenerationResources,
) -> Result<super::PreparedFacetGuide, Error> {
    let source_data = source.data().with_axis_resources(resources);
    let base = super::data::base_plot(source, kind, false, false, resources)?;
    let entries_for = |base: &Plot| -> Result<Vec<crate::decorations::legend::LegendEntry>, Error> {
        if matches!(kind, PlotKind::Heatmap | PlotKind::Beeswarm) {
            // Continuous contributors retain their specialized guide until
            // the kind-discriminated shared descriptor lands in T5b1c.
            return Ok(Vec::new());
        }
        let mut entries = Vec::new();
        for (scope, _) in inputs.axes.iter() {
            let scope = scope.as_deref().ok_or_else(|| {
                Error::Render("facet shared guide has no prepared cell scope".into())
            })?;
            let scoped = base
                .admitted_clone(source_data)?
                .with_render_scope(source_data.copy_source_string(scope)?);
            let mut contribution =
                inputs.guide_entries(&scoped, theme, source_data, Some(scope))?;
            crate::decorations::legend::resolve_entries(&scoped, &mut contribution, resources)?;
            source_data.reserve_source(&mut entries, contribution.len())?;
            entries.extend(contribution);
        }
        Ok(entries)
    };
    let mut base = base;
    if let (Some(field), Some(column)) = (source.hue_dimension_key(), source.hue_col()) {
        let observed = entries_for(&base)?;
        let values = crate::data::strict_strings(source_data, column, "facet hue")?;
        let levels = source_data.source_levels(&values)?;
        let mut present = Vec::new();
        source_data.reserve_source(&mut present, levels.len())?;
        for level in levels {
            let mut found = false;
            for entry in &observed {
                source_data.claim_source_visits(1)?;
                if entry.identity().hue_value_for(field) == Some(level.as_str()) {
                    found = true;
                    break;
                }
            }
            if found {
                present.push(level);
            }
        }
        if !present.is_empty() {
            base = base.with_global_hue_levels(present);
        }
    }
    let entries = if source.legend_policy_spec() == LegendPolicy::Hidden {
        Vec::new()
    } else {
        let entries = entries_for(&base)?;
        crate::decorations::legend::merge_exact_entries(entries, "shared facet legend", resources)?
    };
    let mut entries = entries;
    order_entries(&mut entries, &base, source_data)?;
    let title = crate::decorations::legend::resolved_title(&entries, resources)?;
    let hue_levels = base
        .global_hue_levels()
        .map(|levels| {
            source_data.collect_source(
                levels
                    .iter()
                    .map(|level| source_data.copy_source_string(level)),
            )
        })
        .transpose()?;
    let layout =
        std::alloc::Layout::array::<crate::decorations::legend::LegendEntry>(entries.len())
            .and_then(|entries| std::alloc::Layout::new::<[usize; 2]>().extend(entries))
            .map_err(|_| Error::Render("facet prepared guide storage overflow".into()))?
            .0
            .pad_to_align();
    source_data.claim_source_bytes(layout.size())?;
    Ok(super::PreparedFacetGuide {
        entries: entries.into(),
        title,
        hue_levels,
    })
}

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
        if source.hue_dimension_key().is_some() {
            MeasurementPhase::Hue
        } else {
            MeasurementPhase::Probe
        },
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
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = base_plot.data().with_axis_resources(&resources);
    let mut preflight = Vec::new();
    data.reserve_source(&mut preflight, panels.len())?;
    for (index, facet) in panels.iter().enumerate() {
        let panel_plot = base_plot
            .admitted_clone(data)?
            .with_render_scope(facet.render_scope(data)?)
            .with_significance_subset(facet.significance_selectors());
        let context = PlotRenderContext::new(
            crate::data::PlotData::with_rows(&panel_plot, &facet.rows, &facet.source_rows)
                .with_axis_resources(&resources),
            facet.panel,
            theme,
        )
        .with_facet_scales(facet_scales);
        let owner = crate::layout::ComponentAddress::root().child(
            crate::layout::AddressRole::FacetCell,
            u32::try_from(index).map_err(|_| Error::Render("facet address overflowed".into()))?,
            &resources,
        )?;
        let result = children.measure(&panel_plot, context, &owner, phase, round, |call| {
            crate::layout::fit::outcome(panel_plot.measure_panel_in_call(kind, context, call))
        })?;
        preflight.push(crate::layout::fit::measured(Ok(result))?);
    }
    Ok(preflight)
}

fn order_entries(
    entries: &mut [crate::decorations::legend::LegendEntry],
    shared_plot: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<(), Error> {
    data.source_resources()?;
    if !entries.is_empty() {
        let identity_order = if let Some(pairs) = shared_plot.global_collective_pairs() {
            let mut identities = Vec::new();
            data.reserve_source(&mut identities, pairs.len())?;
            for pair in pairs
                .iter()
                .filter(|pair| pair.group.is_some() || pair.hue.is_some())
            {
                identities.push(shared_plot.collective_legend_identity(pair, data)?);
            }
            identities
        } else {
            shared_plot.global_hue_levels().map_or_else(
                || Ok(Vec::new()),
                |levels| {
                    data.collect_source(
                        levels
                            .iter()
                            .map(|level| shared_plot.point_legend_identity(level, data)),
                    )
                },
            )?
        };
        let mut positions = data.collect_source(entries.iter().map(|entry| {
            for (index, identity) in identity_order.iter().enumerate() {
                data.claim_source_visits(1)?;
                if identity.same_identity(entry.identity(), data.source_resources()?)? {
                    return Ok(index);
                }
            }
            Ok(usize::MAX)
        }))?;
        data.claim_source_visits(entries.len())?;
        // Stable in-place sorting moves admitted storage; key searches above
        // are the actual variable semantic work, not comparator estimates.
        for index in 1..entries.len() {
            let mut current = index;
            while current > 0 && positions[current - 1] > positions[current] {
                positions.swap(current - 1, current);
                entries.swap(current - 1, current);
                current -= 1;
            }
        }
    }
    Ok(())
}

pub(super) fn resolve_shared_facet_axis(
    results: &[std::sync::Arc<PlotRenderResult>],
    shared: bool,
    x_axis: bool,
    data: crate::data::PlotData<'_>,
) -> Result<Option<(f64, f64)>, Error> {
    if !shared || results.is_empty() {
        return Ok(None);
    }
    data.source_resources()?;
    let domain = if x_axis {
        results[0].x_domain
    } else {
        results[0].y_domain
    };
    for result in results {
        data.claim_source_visits(1)?;
        let candidate = if x_axis {
            result.x_domain
        } else {
            result.y_domain
        };
        if candidate != domain {
            return Err(Error::Render(
                "fixed facet scales cannot combine categorical and continuous domains".into(),
            ));
        }
    }
    if domain == AxisDomain::Categorical {
        for result in &results[1..] {
            data.claim_source_visits(1)?;
            let mut expected = facet_axis_signature(&results[0], x_axis);
            let mut actual = facet_axis_signature(result, x_axis);
            loop {
                match (expected.next(), actual.next()) {
                    (None, None) => break,
                    (Some(a), Some(b)) => {
                        data.claim_source_visits(1)?;
                        if crate::decorations::legend::legend_text_eq(
                            a,
                            b,
                            data.source_resources()?,
                        )? {
                            continue;
                        }
                    }
                    _ => {}
                }
                return Err(Error::Render(format!(
                "fixed facet {} axis requires identical categorical levels and ordering in every panel; use a free scale or provide complete panel data",
                if x_axis { "x" } else { "y" }
            )));
            }
        }
        return Ok(None);
    }
    let mut minimum = f64::INFINITY;
    let mut maximum = f64::NEG_INFINITY;
    for result in results {
        data.claim_source_visits(1)?;
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

fn facet_axis_signature(result: &PlotRenderResult, x_axis: bool) -> impl Iterator<Item = &str> {
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
    ticks.iter().enumerate().map(move |(index, tick)| {
        keys.and_then(|keys| keys.get(index))
            .map_or(tick.label.as_str(), String::as_str)
    })
}
