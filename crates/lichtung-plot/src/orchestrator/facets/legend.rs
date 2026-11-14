//! Facet preflight, shared scales, and shared legend coordination.

use crate::Error;

use crate::builder::PlotDescription;
use crate::decorations::plan::children::MeasurementPhase;
use crate::orchestrator::assignment::{
    GuideTrialAssignment, ParentInsets, PlotAllocationInputs, PlotMeasureDimensions, RootAssignment,
};
use crate::orchestrator::registration::RowSelection;
use crate::orchestrator::root::{GridSlot, PreparedFacetTopology, Root};
use crate::spec::{AxisDomain, PlotKind, PlotRenderResult};

/// Prepare the real shared discrete identity and glyph set at the root's
/// semantic boundary, before any font census or panel assignment.
pub(super) fn prepare_discrete_guide(
    source: &PlotDescription,
    kind: PlotKind,
    inputs: &mut crate::orchestrator::registration::PlotRegistryBuilder,
    resources: &crate::font::canonical::OperationContext,
) -> Result<super::PreparedFacetGuide, Error> {
    let (groups, collectives, mut hues) = super::data::parent_orders(source, kind)?;
    let groups = groups.map(crate::data::source::source_arc).transpose()?;
    let collectives = collectives
        .map(crate::data::source::source_arc)
        .transpose()?;
    for registered in &mut inputs.panels {
        if groups.is_some() {
            registered.parent_orders.groups.clone_from(&groups);
        }
        registered
            .parent_orders
            .collectives
            .clone_from(&collectives);
    }
    if let Some(column) = source.hue_col() {
        let observed =
            crate::layout::interior::guide::observed_color_levels(source, &inputs.panels)?;
        let raw = source
            .data_column(column)
            .ok_or_else(|| Error::Data(format!("unknown semantic value {column:?}")))?;
        let crate::data::DataColumn::Category(values) = raw else {
            return Err(Error::Data(format!(
                "facet hue: value {:?} must be categorical",
                raw.name()
            )));
        };
        let values = crate::data::strict_source_strings(values, None, "facet hue")?;
        let levels = crate::data::source::source_levels(&values)?;
        let mut present = Vec::new();
        crate::data::source::reserve_source(&mut present, levels.len())?;
        for level in levels {
            if observed.contains(&level) {
                present.push(level);
            }
        }
        if !present.is_empty() {
            hues = Some(present);
        }
    }
    let hues = hues
        .map(|values| {
            crate::data::source::source_arc(crate::data::category::CategoryDomain::from_values(
                source.hue_field().as_ref(),
                values,
            )?)
        })
        .transpose()?;
    let group_colors = if kind == PlotKind::Interaction && source.hue_col().is_none() {
        groups
            .as_deref()
            .map(|values| {
                let values = crate::data::source::collect_source(
                    values
                        .iter()
                        .map(|value| crate::data::source::copy_source_string(value)),
                )?;
                crate::data::source::source_arc(crate::data::category::CategoryDomain::from_values(
                    source.categorical_field().as_ref(),
                    values,
                )?)
            })
            .transpose()?
    } else {
        None
    };
    for registered in &mut inputs.panels {
        registered.parent_orders.hues.clone_from(&hues);
        if let Some(binding) = &mut registered.categorical {
            binding.parent.clone_from(
                if kind == PlotKind::Interaction && source.hue_col().is_none() {
                    &group_colors
                } else {
                    &hues
                },
            );
        }
    }
    let descriptor = crate::layout::interior::guide::prepare_discrete_guides(
        source,
        &mut inputs.panels,
        resources,
        true,
    )?;
    Ok(super::PreparedFacetGuide { descriptor })
}

#[expect(
    clippy::too_many_arguments,
    reason = "the same prepared occurrence inventory accompanies the existing facet probe inputs"
)]
pub(super) fn preflight_coordinated_panels(
    source: &PlotDescription,
    root: &Root<'_>,
    topology: &PreparedFacetTopology,
    measurement: &RootAssignment,
    panels: &[(GridSlot, crate::FacetCell, RowSelection)],
    cells: &[crate::layout::PixelRect],
    theme: &crate::text::TextRequestContext<'_, '_>,
    children: &mut crate::decorations::plan::children::ChildPlans,
    round: &crate::layout::kernel::FeedbackRound,
) -> Result<Vec<std::sync::Arc<PlotRenderResult>>, crate::layout::fit::PlanError> {
    preflight_panels(
        root,
        topology,
        measurement,
        panels,
        cells,
        theme,
        children,
        round,
        if source.hue_dimension_key().is_some() {
            MeasurementPhase::Hue
        } else {
            MeasurementPhase::Probe
        },
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "root-local child reuse keeps computed hue inputs explicit"
)]
fn preflight_panels(
    root: &Root<'_>,
    topology: &PreparedFacetTopology,
    measurement: &RootAssignment,
    panels: &[(GridSlot, crate::FacetCell, RowSelection)],
    cells: &[crate::layout::PixelRect],
    theme: &crate::text::TextRequestContext<'_, '_>,
    children: &mut crate::decorations::plan::children::ChildPlans,
    round: &crate::layout::kernel::FeedbackRound,
    phase: MeasurementPhase,
) -> Result<Vec<std::sync::Arc<PlotRenderResult>>, crate::layout::fit::PlanError> {
    theme.text.prepared_font_environment()?;

    let mut preflight = Vec::new();
    crate::data::source::reserve_source(&mut preflight, panels.len())?;
    for (slot, _, _) in panels {
        let id = topology.slots[slot.0]
            .plot_id
            .as_ref()
            .expect("facet producer");
        let factory = root.plot_assignment_factory(measurement, id)?;
        let registered = factory.registered;
        let panel = cells[slot.0];
        let owner = &registered.scope.address;
        let assignment = factory.assign(
            PlotAllocationInputs {
                dimensions: PlotMeasureDimensions {
                    canvas: measurement.measurement_size,
                    output_override: None,
                },
                cell: panel,
                frame: panel,
                body: panel,
                axis_label_width: panel.width(),
                content_height: false,
                fill_content_height: false,
                content_preference: None,
                insets: ParentInsets::consumed(round, owner),
                guide: GuideTrialAssignment {
                    edge: registered.source.legend_position_spec(),
                    compact: false,
                    allocation: None,
                },
            },
            round,
            Some(phase),
        )?;
        let scope = crate::layout::fit::FitScope::root();
        #[cfg(test)]
        super::observe_factory_phase(id, Some(phase), &assignment);
        let context = factory.context(
            &assignment,
            crate::decorations::legend::discrete::scoped::SUPPRESSED_GUIDES,
            &scope,
        )?;
        let result = children.measure(&context, owner, phase, round, || {
            crate::layout::fit::outcome(
                crate::orchestrator::measure_probe(
                    &factory,
                    assignment.clone(),
                    crate::decorations::legend::discrete::scoped::SUPPRESSED_GUIDES,
                    &scope,
                )
                .map(|(_, result)| result),
            )
        })?;
        preflight.push(crate::layout::fit::measured(Ok(result))?);
    }
    Ok(preflight)
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
                        continue;
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
