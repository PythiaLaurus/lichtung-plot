//! Facet preflight, shared scales, and shared legend coordination.

use crate::Error;

use super::data::FacetPanel;
use crate::builder::Plot;
use crate::decorations::plan::children::MeasurementPhase;
use crate::spec::{AxisDomain, PlotKind, PlotRenderContext, PlotRenderResult};
use crate::theme::ResolvedTheme;

/// Prepare the real shared discrete identity and glyph set at the root's
/// semantic boundary, before any font census or panel assignment.
pub(super) fn prepare_discrete_guide(
    source: &Plot,
    kind: PlotKind,
    theme: &ResolvedTheme,
    inputs: &crate::layout::interior::PreparedInteriorInputs,
    resources: &crate::font::canonical::OperationContext,
) -> Result<super::PreparedFacetGuide, Error> {
    let source_data = source.data().with_operation(resources);
    let mut base = super::data::base_plot(source, kind, false, false, resources)?;
    if let Some(column) = source.hue_col() {
        let observed = inputs.observed_color_levels(&base)?;
        let values = crate::data::strict_strings(source_data, column, "facet hue")?;
        let levels = crate::data::source::source_levels(&values)?;
        let mut present = Vec::new();
        crate::data::source::reserve_source(&mut present, levels.len())?;
        for level in levels {
            if observed.contains(&level) {
                present.push(level);
            }
        }
        if !present.is_empty() {
            base = base.with_global_hue_levels(present);
        }
    }
    inputs.prepare_discrete_guides(
        &base,
        theme,
        resources,
        &crate::layout::ComponentAddress::root(),
        true,
    )?;
    let descriptor = inputs
        .discrete_guides
        .get()
        .expect("prepared facet guide")
        .root
        .clone();
    let hue_levels = base
        .global_hue_levels()
        .map(|levels| {
            crate::data::source::collect_source(
                levels
                    .iter()
                    .map(|level| crate::data::source::copy_source_string(level)),
            )
        })
        .transpose()?;

    Ok(super::PreparedFacetGuide {
        descriptor,
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
    let resources = theme.prepared_font_environment()?.operation();

    let mut preflight = Vec::new();
    crate::data::source::reserve_source(&mut preflight, panels.len())?;
    for (index, facet) in panels.iter().enumerate() {
        let panel_plot = base_plot
            .clone()
            .with_render_scope(facet.render_scope()?)
            .with_significance_subset(facet.significance_selectors());
        let context = PlotRenderContext::new(
            crate::data::PlotData::with_rows(&panel_plot, &facet.rows, &facet.source_rows)
                .with_operation(&resources),
            facet.panel,
            theme,
        )
        .with_facet_scales(facet_scales);
        let owner = crate::layout::ComponentAddress::root().child(
            crate::layout::AddressRole::FacetCell,
            u32::try_from(index).map_err(|_| Error::Render("facet address overflowed".into()))?,
        )?;
        let result = children.measure(&panel_plot, context, &owner, phase, round, || {
            crate::layout::fit::outcome(panel_plot.measure_panel(kind, context))
        })?;
        preflight.push(crate::layout::fit::measured(Ok(result))?);
    }
    Ok(preflight)
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
            let mut expected = facet_axis_signature(&results[0], x_axis);
            let mut actual = facet_axis_signature(result, x_axis);
            loop {
                match (expected.next(), actual.next()) {
                    (None, None) => break,
                    (Some(a), Some(b)) if crate::decorations::legend::legend_text_eq(a, b) => {
                        continue
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
