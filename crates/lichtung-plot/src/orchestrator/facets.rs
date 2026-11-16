//! Faceted chart coordination.

use crate::Error;
use std::fmt::Write;

use super::plan::{MeasuredOwnedPlan, MeasuredRootContent};
#[cfg(test)]
use super::registration::PlotOccurrenceId;
use super::root::{GridSlot, PreparedFacetTopology, Root};
#[cfg(test)]
use crate::builder::Plot;
use crate::builder::PlotDescription;
use crate::decorations::plan::ResolvedLegend;
use crate::layout::fit::{self, FitFailure, FitOutcome, FitScope};
use crate::layout::kernel::{FitKind, TrialOutcome};
use crate::render::backend::Backend;
use crate::spec::{FacetScales, LegendPosition, PlotKind};
use crate::text::TextRequestContext;

mod data;
mod legend;
mod planning;
mod rendering;
mod sizing;
mod tracks;
pub(super) use sizing::natural_size;

#[cfg(test)]
thread_local! {
    #[expect(
        clippy::type_complexity,
        reason = "one test observation retains occurrence, phase, extents and provenance without another owner record"
    )]
    static FACTORY_PHASES: std::cell::RefCell<Vec<(
        PlotOccurrenceId,
        Option<crate::decorations::plan::children::MeasurementPhase>,
        [(f64, f64); 2],
        [crate::layout::RangeProvenance; 2],
    )>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
fn observe_factory_phase(
    id: &PlotOccurrenceId,
    phase: Option<crate::decorations::plan::children::MeasurementPhase>,
    assignment: &super::assignment::PlotMeasureAssignment,
) {
    if let super::assignment::AxisDomainAssignments::Pair(axes) = &assignment.domains {
        FACTORY_PHASES.with(|events| {
            events.borrow_mut().push((
                id.clone(),
                phase,
                axes.each_ref().map(|axis| axis.extent.pair()),
                axes.each_ref().map(|axis| axis.provenance),
            ));
        });
    }
}

pub(crate) fn text_census_labels(
    source: &crate::builder::PlotDescription,
    plots: Option<&[super::registration::PlotRegistration<'_>]>,
    mut visit: impl FnMut(crate::font::canonical::CensusSource<'_>) -> Result<(), Error>,
) -> Result<(), Error> {
    let selectors = data::facet_selectors(source)?;
    for (index, (_, extracted_cell, _)) in data::extract_facet_rows(source, selectors)?
        .1
        .into_iter()
        .enumerate()
    {
        let (source, cell) = match plots {
            // Source-only census still visits authored facet labels without
            // requiring an occurrence inventory.
            None | Some([]) => (source, &extracted_cell),
            Some(plots) => {
                let registered = &plots[index];
                (
                    registered.source,
                    registered
                        .scope
                        .numeric_owner
                        .facet
                        .as_ref()
                        .expect("prepared facet occurrence"),
                )
            }
        };
        let fields = data::fields(source, cell);
        if source.text_layouts().facet.iter().any(Option::is_some) {
            for (index, (_, values, level)) in fields.iter().enumerate() {
                let role = if cell.dimensions() == 1 { 1 } else { index };
                if source.text_layouts().facet[role]
                    .as_ref()
                    .is_some_and(crate::TextLayout::hidden)
                {
                    continue;
                }
                visit(
                    format_args!(
                        "{} = {}",
                        source.resolve_label(values.name())?,
                        source.resolve_label(level)?
                    )
                    .into(),
                )?;
            }
            continue;
        }
        match fields.as_slice() {
            [] => visit("".into())?,
            [(_, values, level)] => visit(
                format_args!(
                    "{} = {}",
                    source.resolve_label(values.name())?,
                    source.resolve_label(level)?
                )
                .into(),
            )?,
            [(_, row, row_level), (_, column, column_level)] => visit(
                format_args!(
                    "{} = {} | {} = {}",
                    source.resolve_label(row.name())?,
                    source.resolve_label(row_level)?,
                    source.resolve_label(column.name())?,
                    source.resolve_label(column_level)?
                )
                .into(),
            )?,
            _ => {
                return Err(Error::Render(
                    "facet text census has more than two selectors".into(),
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn prepare_facet_scopes<'src>(
    source: &'src crate::builder::PlotDescription,
    builder: &mut super::registration::PlotRegistryBuilder<'src>,
    appearance: &crate::theme::ResolvedTheme,
    operation: &crate::font::canonical::OperationContext,
) -> Result<PreparedFacetTopology, Error> {
    if !source.significance_annotations().is_empty()
        && !source.kind().supports_significance_annotations()
    {
        return Err(Error::Data(format!(
            "significance annotations are not supported for {:?}",
            source.kind()
        )));
    }
    let selectors = data::facet_selectors(source)?;
    let (mut shell, panels) = data::extract_facet_rows(source, selectors)?;
    data::validate_significance_facet_anchors(source, &panels)?;
    let mut producers = Vec::new();
    crate::data::source::reserve_source(&mut producers, panels.len())?;
    let mut shared_axis_members = [Vec::new(), Vec::new()];
    let mut shared_titles_from = None;
    for (slot, (grid_slot, cell, rows)) in panels.into_iter().enumerate() {
        let render_name = data::render_scope(source, &cell)?;
        let scope = super::registration::Scope::new(
            source,
            operation,
            slot,
            crate::layout::ComponentAddress::root().child(
                crate::layout::AddressRole::FacetCell,
                u32::try_from(slot)
                    .map_err(|_| Error::Render("facet address overflowed".into()))?,
            )?,
            rows,
            Some(&cell),
            Some(render_name),
        )?;
        let policy = super::registration::PlotScopePolicy::for_source(source, Some(&cell));
        let id = builder.reserve(
            source,
            scope.address,
            scope.numeric_owner,
            scope.axis_numeric_owner,
            scope.rows,
            scope.render_name,
            policy,
            appearance.clone(),
            operation,
        )?;
        shared_titles_from.get_or_insert_with(|| id.clone());
        for (side, fixed) in [
            matches!(
                source.facet_scales_spec(),
                FacetScales::Fixed | FacetScales::FreeY
            ),
            matches!(
                source.facet_scales_spec(),
                FacetScales::Fixed | FacetScales::FreeX
            ),
        ]
        .into_iter()
        .enumerate()
        {
            if fixed {
                crate::data::source::reserve_source(&mut shared_axis_members[side], 1)?;
                shared_axis_members[side].push(id.clone());
            }
        }
        shell.slots[grid_slot.0].plot_id = Some(id);
        producers.push(grid_slot);
    }
    Ok(PreparedFacetTopology {
        rows: shell.rows,
        columns: shell.columns,
        slots: shell.slots,
        producers,
        shared_axis_members,
        shared_titles_from: shared_titles_from.expect("observed facet producer"),
        parent_guide: None,
    })
}

pub(crate) struct PreparedFacetGuide {
    pub(crate) descriptor:
        Option<std::sync::Arc<crate::decorations::legend::discrete::PreparedGuide>>,
}

pub(crate) fn prepare_discrete_guide(
    source: &PlotDescription,
    kind: PlotKind,
    inputs: &mut super::registration::PlotRegistryBuilder,
    resources: &crate::font::canonical::OperationContext,
) -> Result<PreparedFacetGuide, Error> {
    legend::prepare_discrete_guide(source, kind, inputs, resources)
}

pub(crate) struct OwnedFacetPlan {
    grid: crate::layout::PixelRect,
    cells: Vec<crate::layout::PixelRect>,
    shared_axis_origin: (f64, f64),
    children: Vec<OwnedFacetChild>,
    shared_titles: Vec<crate::render::backend::PlacedText>,
    parent_guide: Option<(String, crate::decorations::legend::LegendDrawPlan)>,
}

pub(super) struct OwnedFacetChild {
    slot: GridSlot,
    plot: std::sync::Arc<crate::decorations::plan::MeasuredPlotPlan>,
    strips: Vec<crate::render::backend::PlacedText>,
    strip_layer: String,
    legend: Option<(String, crate::decorations::legend::LegendDrawPlan)>,
}

impl OwnedFacetPlan {
    #[expect(
        clippy::too_many_lines,
        clippy::too_many_arguments,
        clippy::cast_precision_loss,
        reason = "the legacy parent adapter keeps complete-frame guide trials explicit"
    )]
    pub(super) fn measure(
        source: &PlotDescription,
        root: &Root<'_>,
        topology: &PreparedFacetTopology,
        measurement: super::assignment::RootAssignment,
        theme: &TextRequestContext<'_, '_>,
        round: &mut crate::layout::kernel::FeedbackRound,
        children: &mut crate::decorations::plan::children::ChildPlans,
        content_sizing: bool,
    ) -> Result<MeasuredOwnedPlan, crate::layout::fit::PlanError> {
        let kind = source.kind();
        let operation = &root.operation;
        if !source.significance_annotations().is_empty()
            && !kind.supports_significance_annotations()
        {
            return Err(Error::Data(format!(
                "significance annotations are not supported for {kind:?}"
            ))
            .into());
        }

        let selectors = data::facet_selectors(source)?;
        let [horizontal_axis_label, vertical_axis_label] =
            super::registration::prepare_axis_labels(root.plot(&topology.shared_titles_from)?)?;
        for registered in &root.plots {
            registered.axes.retain_axis_labels([
                horizontal_axis_label.as_ref(),
                vertical_axis_label.as_ref(),
            ])?;
        }
        let mut canvas_layout = planning::canvas_layout(
            source,
            measurement.measurement_size,
            theme,
            horizontal_axis_label.as_ref(),
            vertical_axis_label.as_ref(),
            round,
        )?;
        let mut outer = canvas_layout.panel_box;
        round.observe_canvas(&canvas_layout)?;
        theme.text.prepared_font_environment()?;
        data::parent_orders(source, kind)?;
        let (_shell, panels) = data::extract_facet_rows(source, selectors)?;
        data::validate_significance_facet_anchors(source, &panels)?;
        let mut minima = Vec::new();
        crate::data::source::reserve_source(&mut minima, panels.len())?;
        for index in 0..panels.len() {
            let registered = root.plot(
                topology.slots[topology.producers[index].0]
                    .plot_id
                    .as_ref()
                    .expect("facet occurrence"),
            )?;
            minima.push(crate::layout::interior::intrinsic_body_minimum(
                registered,
                crate::builder::RootFrameDomainInput::Prepared,
                root.plot_intrinsic(&registered.scope.id)?,
                operation,
                theme,
            )?);
        }
        let provisional_content = content_sizing.then_some(tracks::ContentSizing {
            root,
            topology,
            windows: [source.x_axis_spec(), source.y_axis_spec()]
                .map(|axis| axis.window.map(crate::layout::RawWindow::pair)),
            round,
            natural_width: false,
            natural_height: false,
        });
        let provisional = fit::measured(tracks::allocate(
            outer,
            (topology.rows, topology.columns),
            &panels,
            &minima,
            &[],
            theme,
            &FitScope::root(),
            provisional_content.as_ref(),
        ))?
        .cells;
        let facet_scales = source.facet_scales_spec();

        let preflight = legend::preflight_coordinated_panels(
            source,
            root,
            topology,
            &measurement,
            &panels,
            &provisional,
            theme,
            children,
            round,
        )?;
        let fix_x = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeY);
        let fix_y = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeX);
        let horizontal = legend::resolve_shared_facet_axis(&preflight, fix_x, true)?;
        let vertical = legend::resolve_shared_facet_axis(&preflight, fix_y, false)?;
        let windows = planning::feedback_domains(
            source,
            &panels,
            [horizontal, vertical],
            facet_scales,
            round,
        )?;
        let discrete_guide = topology
            .parent_guide
            .as_ref()
            .map(|guide| &guide.descriptor);
        let position = source.legend_position_spec();
        let automatic = position == LegendPosition::Auto && discrete_guide.is_some();
        let automatic_edges = [
            LegendPosition::OutsideRight,
            LegendPosition::OutsideTop,
            LegendPosition::OutsideBottom,
            LegendPosition::OutsideLeft,
        ];
        let edges = if automatic {
            &automatic_edges[..]
        } else {
            std::slice::from_ref(&position)
        };
        let owner = crate::layout::ComponentAddress::root()
            .child(crate::decorations::legend::address_role(position), 0)?;
        let positive_span = crate::layout::canvas::MINIMUM_PANEL_SPAN;
        let mut columns =
            crate::data::source::collect_source((0..topology.columns).map(|_| Ok(positive_span)))?;
        let mut rows =
            crate::data::source::collect_source((0..topology.rows).map(|_| Ok(positive_span)))?;

        for ((slot, _, _), hard) in panels.iter().zip(&minima) {
            columns[slot.0 % topology.columns] = columns[slot.0 % topology.columns].max(hard.0);
            rows[slot.0 / topology.columns] = rows[slot.0 / topology.columns].max(hard.1);
        }
        let mut minimum = (
            columns.iter().sum::<f64>()
                + topology.columns.saturating_sub(1) as f64 * theme.composition_facet_gap()?,
            rows.iter().sum::<f64>()
                + topology.rows.saturating_sub(1) as f64 * theme.composition_facet_gap()?,
        );
        if !minimum.0.is_finite() || !minimum.1.is_finite() {
            return Err(Error::Render("facet intrinsic grid span overflowed".into()).into());
        }
        // The guide and grid share one containing frame. Reserve the consumed
        // column/row gutters before measuring flow, not after fixing its width.
        let grid = crate::layout::ComponentAddress::root()
            .child(crate::layout::AddressRole::FacetGrid, 0)?;
        let mut columns = Vec::new();
        crate::data::source::reserve_source(&mut columns, topology.columns)?;
        columns.resize(topology.columns, [0.0_f64; 2]);
        let mut rows = Vec::new();
        crate::data::source::reserve_source(&mut rows, topology.rows)?;
        rows.resize(topology.rows, [0.0_f64; 2]);
        for (index, (slot, _, _)) in panels.iter().enumerate() {
            let address = grid.child(
                crate::layout::AddressRole::FacetCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("facet address overflowed".into()))?,
            )?;
            let [left, right, bottom, top] =
                round.insets(&address, crate::layout::kernel::FeedbackKind::AxisGutter);
            columns[slot.0 % topology.columns][0] = columns[slot.0 % topology.columns][0].max(left);
            columns[slot.0 % topology.columns][1] =
                columns[slot.0 % topology.columns][1].max(right);
            rows[slot.0 / topology.columns][0] = rows[slot.0 / topology.columns][0].max(bottom);
            rows[slot.0 / topology.columns][1] = rows[slot.0 / topology.columns][1].max(top);
        }
        minimum.0 += columns.iter().flatten().sum::<f64>();
        minimum.1 += rows.iter().flatten().sum::<f64>();
        let baseline = if automatic
            || (discrete_guide.is_some() && crate::decorations::plan::is_outside(position))
        {
            planning::shared_axis_requirements(
                horizontal_axis_label.as_ref(),
                vertical_axis_label.as_ref(),
                theme,
                canvas_layout.panel_box,
            )?
        } else {
            [0.0; 4]
        };
        let mut first_miss = None;
        for (edge_index, &edge) in edges.iter().enumerate() {
            let outside = discrete_guide.is_some() && crate::decorations::plan::is_outside(edge);
            let mut compact_changes = true;
            'trial: for compact in [false, true] {
                if compact && (!outside || !compact_changes) {
                    continue;
                }
                let ordinal = u32::try_from(edge_index * 2 + usize::from(compact))
                    .map_err(|_| Error::Render("facet guide ordinal overflowed".into()))?;
                let scope = FitScope::new(&crate::layout::ComponentAddress::root(), ordinal);
                let guide_scope = FitScope::new(&owner, ordinal);
                let (reserved_outer, mut outside_legend) = if outside {
                    round.claim_guide_trial(&owner, if automatic { 8 } else { 2 }, ordinal)?;
                    let horizontal = matches!(
                        edge,
                        LegendPosition::OutsideTopLeft
                            | LegendPosition::OutsideTop
                            | LegendPosition::OutsideTopRight
                            | LegendPosition::OutsideBottomLeft
                            | LegendPosition::OutsideBottom
                            | LegendPosition::OutsideBottomRight
                    );
                    let gap = theme.legend_outside_gap()?;
                    let frame =
                        crate::decorations::plan::AssignedGuideFrame::new(edge, minimum, gap);
                    let available = frame.available(canvas_layout.panel_box, [0.0; 4]);
                    let guide = match crate::decorations::legend::Legend::measure_assigned(
                        std::sync::Arc::clone(discrete_guide.expect("contributing facet guide")),
                        (available.0, (!content_sizing).then_some(available.1)),
                        horizontal,
                        compact,
                        theme,
                        &mut compact_changes,
                        &guide_scope,
                    )? {
                        FitOutcome::Measured(guide) => guide,
                        FitOutcome::Miss(reason) => {
                            FitFailure::retain_least(&mut first_miss, reason);
                            continue;
                        }
                    };
                    if content_sizing {
                        let grid_width = canvas_layout.panel_box.width()
                            - if horizontal {
                                0.0
                            } else {
                                guide.compute_box_width() + gap
                            };
                        let height = match sizing::grid_height(
                            root,
                            topology,
                            &panels,
                            windows,
                            (topology.rows, topology.columns),
                            &minima,
                            theme,
                            round,
                            grid_width,
                        )? {
                            FitOutcome::Measured(height) => height,
                            FitOutcome::Miss(reason) => {
                                FitFailure::retain_least(&mut first_miss, reason);
                                continue;
                            }
                        };
                        let height = if horizontal {
                            height + guide.compute_box_height() + gap
                        } else {
                            height.max(guide.compute_box_height())
                        };
                        let height = crate::layout::kernel::root_dimension(
                            canvas_layout.height_for_body(height, theme)?,
                        )?;
                        canvas_layout = planning::canvas_layout(
                            source,
                            crate::layout::RootSize::new(
                                measurement.measurement_size.dimensions()?.0,
                                height,
                            ),
                            theme,
                            horizontal_axis_label.as_ref(),
                            vertical_axis_label.as_ref(),
                            round,
                        )?;
                    }
                    match crate::decorations::plan::place_outside_legend(
                        guide,
                        edge,
                        canvas_layout.panel_box,
                        minimum,
                        &scope,
                    )? {
                        FitOutcome::Measured((panel, guide)) => (panel, Some(guide)),
                        FitOutcome::Miss(reason) => {
                            FitFailure::retain_least(&mut first_miss, reason);
                            continue;
                        }
                    }
                } else {
                    if content_sizing {
                        let height = fit::measured(sizing::grid_height(
                            root,
                            topology,
                            &panels,
                            windows,
                            (topology.rows, topology.columns),
                            &minima,
                            theme,
                            round,
                            canvas_layout.panel_box.width(),
                        ))?;
                        let height = crate::layout::kernel::root_dimension(
                            canvas_layout.height_for_body(height, theme)?,
                        )?;
                        canvas_layout = planning::canvas_layout(
                            source,
                            crate::layout::RootSize::new(
                                measurement.measurement_size.dimensions()?.0,
                                height,
                            ),
                            theme,
                            horizontal_axis_label.as_ref(),
                            vertical_axis_label.as_ref(),
                            round,
                        )?;
                    }
                    (canvas_layout.panel_box, None)
                };
                if outside {
                    let measured = match fit::outcome(planning::shared_axis_requirements(
                        horizontal_axis_label.as_ref(),
                        vertical_axis_label.as_ref(),
                        theme,
                        reserved_outer,
                    ))? {
                        FitOutcome::Measured(insets) => insets,
                        FitOutcome::Miss(reason) => {
                            FitFailure::retain_least(&mut first_miss, reason);
                            continue 'trial;
                        }
                    };
                    if reserved_outer.width() - (measured[0] - baseline[0]).max(0.0) < minimum.0
                        || reserved_outer.height() - (measured[2] - baseline[2]).max(0.0)
                            < minimum.1
                    {
                        FitFailure::retain_least(
                            &mut first_miss,
                            scope
                                .child(crate::layout::AddressRole::FacetGrid, 0)?
                                .failure(
                                    FitKind::TrackOrResidual,
                                    "shared facet titles leave insufficient assigned grid space",
                                ),
                        );
                        continue;
                    }
                }
                let checkpoint = round.begin_trial()?;
                round.observe_count(edge as usize)?;
                round.observe_count(usize::from(compact))?;
                outer = match fit::outcome(planning::reserve_rewrapped_shared_axis_titles(
                    horizontal_axis_label.as_ref(),
                    vertical_axis_label.as_ref(),
                    theme,
                    canvas_layout.panel_box,
                    reserved_outer,
                    round,
                ))? {
                    FitOutcome::Measured(outer) => outer,
                    FitOutcome::Miss(reason) => {
                        round.record_failure(reason)?;
                        if let TrialOutcome::Miss(reason) = round.finish_trial(checkpoint) {
                            FitFailure::retain_least(&mut first_miss, reason);
                        }
                        continue;
                    }
                };
                let shared_axis_origin = position_shared_axis_guides(
                    edge,
                    canvas_layout.panel_box,
                    reserved_outer,
                    outer,
                    outside_legend.as_mut(),
                );
                let measured = planning::native_parent(
                    source,
                    root,
                    topology,
                    &measurement,
                    &panels,
                    windows,
                    outer,
                    (topology.rows, topology.columns),
                    theme,
                    round,
                    &preflight,
                    &provisional,
                    &minima,
                    outside,
                    discrete_guide,
                    &scope,
                    children,
                    content_sizing,
                )?;
                let (mut children, grid) = match measured {
                    FitOutcome::Measured(value) => value,
                    FitOutcome::Miss(failure) => {
                        round.record_failure(failure)?;
                        if let TrialOutcome::Miss(failure) = round.finish_trial(checkpoint) {
                            FitFailure::retain_least(&mut first_miss, failure);
                        }
                        continue;
                    }
                };
                match round.finish_trial(checkpoint) {
                    TrialOutcome::Miss(reason) => {
                        FitFailure::retain_least(&mut first_miss, reason);
                        continue;
                    }
                    _outcome @ (TrialOutcome::Fit | TrialOutcome::Provisional) => {}
                }
                let mut strip_text = Vec::new();
                crate::render::backend::reserve_text_records(&mut strip_text, panels.len())?;
                for (index, (slot, _, _)) in panels.iter().enumerate() {
                    let registered = &root.plots[children[index].plot.plot_id.slot];
                    strip_text.push(rendering::plan_facet_strip(
                        registered.source,
                        registered
                            .scope
                            .numeric_owner
                            .facet
                            .as_ref()
                            .expect("prepared facet occurrence"),
                        *slot,
                        grid.cells[slot.0],
                        theme,
                    )?);
                }
                for role in 0..2 {
                    let members = crate::data::source::collect_source(
                        strip_text
                            .iter()
                            .enumerate()
                            .flat_map(|(panel, parts)| {
                                parts
                                    .iter()
                                    .enumerate()
                                    .filter(move |(part, _)| {
                                        if parts.len() == 1 {
                                            role == 1
                                        } else {
                                            *part == role
                                        }
                                    })
                                    .map(move |(part, _)| (panel, part))
                            })
                            .map(Ok),
                    )?;
                    let mut text = crate::data::source::collect_source(
                        members
                            .iter()
                            .map(|&(panel, part)| Ok(strip_text[panel][part].clone())),
                    )?;
                    crate::layout::text::font_fit::uniform_placed(
                        theme,
                        &mut text,
                        &crate::layout::fit::ROOT_SCOPE
                            .child(crate::layout::AddressRole::FacetHeader, role)?,
                    )?;
                    for (&(panel, part), text) in members.iter().zip(text) {
                        strip_text[panel][part] = text;
                    }
                }
                let mut legends = Vec::new();
                let mut strip_layers = Vec::new();
                theme.text.prepared_font_environment()?;

                crate::data::source::reserve_source(&mut legends, panels.len())?;
                crate::data::source::reserve_source(&mut strip_layers, panels.len())?;
                for (index, ((slot, _, _), child)) in panels.iter().zip(&children).enumerate() {
                    let panel = &child.plot;
                    let registered = &root.plots[child.plot.plot_id.slot];
                    let scope = data::render_scope(
                        registered.source,
                        registered
                            .scope
                            .numeric_owner
                            .facet
                            .as_ref()
                            .expect("prepared facet occurrence"),
                    )?;
                    let mut layers = [String::new()];
                    for (layer, suffix) in layers.iter_mut().zip(["legend"]) {
                        let length = scope
                            .len()
                            .checked_add(suffix.len() + 1)
                            .ok_or_else(|| Error::Render("facet layer length overflow".into()))?;
                        crate::data::source::reserve_source_string(layer, length)?;
                        layer.push_str(&scope);
                        layer.push('-');
                        layer.push_str(suffix);
                    }
                    let [legend_layer] = layers;
                    let legend = if index == 0 {
                        outside_legend.as_ref().or(panel.legend.as_ref())
                    } else {
                        panel.legend.as_ref()
                    };
                    legends.push(
                        legend
                            .map(|legend| {
                                let (x, y) = legend.bounds.anchor();
                                legend.legend.select(x, y)
                            })
                            .transpose()?
                            .map(|legend| (legend_layer, legend)),
                    );
                    let mut layer = String::new();
                    crate::data::source::reserve_source_string(&mut layer, 32)?;
                    write!(layer, "facet-strip-{}", slot.0).expect("reserved index");
                    strip_layers.push(layer);
                }
                let dimensions = (
                    measurement.measurement_size.dimensions()?.0,
                    crate::layout::kernel::root_dimension(
                        canvas_layout.panel_box.y1 + canvas_layout.insets.top,
                    )?,
                );
                let shared_axis_text = rendering::plan_shared_facet_axis_labels(
                    horizontal_axis_label.as_ref(),
                    vertical_axis_label.as_ref(),
                    outer,
                    shared_axis_origin,
                    theme,
                )?;
                let parent_guide = if outside_legend.is_some() {
                    legends.first_mut().and_then(Option::take)
                } else {
                    None
                };
                for (((child, strips), strip_layer), legend) in children
                    .iter_mut()
                    .zip(strip_text)
                    .zip(strip_layers)
                    .zip(legends)
                {
                    child.strips = strips;
                    child.strip_layer = strip_layer;
                    child.legend = legend;
                }
                let plan = Self {
                    grid: outer,
                    cells: grid.cells,
                    shared_axis_origin,
                    children,
                    shared_titles: shared_axis_text,
                    parent_guide,
                };
                plan.observe_arranged(
                    [horizontal_axis_label.as_ref(), vertical_axis_label.as_ref()],
                    outside_legend.as_ref(),
                    round,
                )?;
                return Ok(MeasuredOwnedPlan {
                    measurement,
                    size: crate::layout::RootSize::new(dimensions.0, dimensions.1),
                    frame: canvas_layout.into_shared(),
                    root_paint: theme.surfaces.canvas_root,
                    content: MeasuredRootContent::Facets(plan),
                });
            }
        }
        Err(first_miss
            .unwrap_or_else(|| {
                FitScope::new(&owner, 0).failure(
                    FitKind::LegendOrGuide,
                    "facet containing frame cannot fit its guide",
                )
            })
            .into())
    }

    fn observe_arranged(
        &self,
        labels: [Option<&crate::Text>; 2],
        outside_legend: Option<&ResolvedLegend>,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        use crate::decorations::legend::discrete::scoped::GuideAllocation;

        round.begin_owner(
            &crate::layout::ComponentAddress::root()
                .child(crate::layout::AddressRole::FacetGrid, 0)?,
        )?;
        round.observe_box(self.grid)?;
        for label in labels.into_iter().flatten() {
            round.observe_authored_text(label)?;
        }
        round.observe_number(self.shared_axis_origin.0)?;
        round.observe_number(self.shared_axis_origin.1)?;
        round.observe_count(self.children.len())?;
        for (index, child) in self.children.iter().enumerate() {
            let plan = &child.plot;
            round.observe_box(plan.panel_box)?;
            let legend = if index == 0 {
                outside_legend.or(plan.legend.as_ref())
            } else {
                plan.legend.as_ref()
            };
            round.observe_count(usize::from(legend.is_some()))?;
            if let Some(legend) = legend {
                let available_width = match (
                    index,
                    outside_legend,
                    plan.selected.guide.allocation.as_ref(),
                ) {
                    (0, Some(parent), _) => parent.legend.available_width(),
                    (_, _, Some(GuideAllocation::Width(width)))
                        if crate::decorations::plan::is_outside(plan.selected.guide.edge) =>
                    {
                        *width
                    }
                    _ => legend.legend.available_width(),
                };
                legend.observe_feedback(round, available_width)?;
            }
        }
        rendering::observe_arranged_chrome(self, round)
    }

    pub(super) fn render(
        &self,
        source: &PlotDescription,
        kind: PlotKind,
        frame: &crate::layout::RootFramePlan,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        rendering::render_scene(
            source,
            kind,
            &self.children,
            self.parent_guide.as_ref(),
            &self.shared_titles,
            frame,
            backend,
        )
    }
}

fn position_shared_axis_guides(
    position: LegendPosition,
    canvas_panel: crate::layout::PixelRect,
    reserved_outer: crate::layout::PixelRect,
    outer: crate::layout::PixelRect,
    legend: Option<&mut ResolvedLegend>,
) -> (f64, f64) {
    let mut origin = (outer.x0, outer.y0);
    let Some(legend) = legend else {
        return origin;
    };
    let extra_left = outer.x0 - reserved_outer.x0;
    let extra_bottom = outer.y0 - reserved_outer.y0;
    match position {
        LegendPosition::OutsideLeftTop
        | LegendPosition::OutsideLeft
        | LegendPosition::OutsideLeftBottom => {
            origin.0 = canvas_panel.x0 + extra_left;
            legend.bounds = legend.bounds.translated(extra_left, 0.0);
        }
        LegendPosition::OutsideBottomLeft
        | LegendPosition::OutsideBottom
        | LegendPosition::OutsideBottomRight => {
            origin.1 = canvas_panel.y0 + extra_bottom;
            legend.bounds = legend.bounds.translated(0.0, extra_bottom);
        }
        _ => {}
    }
    origin
}

#[cfg(test)]
mod feedback_tests {
    use super::*;

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "the production regression compares free and fixed scales through complete cached and uncached roots"
    )]
    fn dirty_facet_child_reuses_only_independent_neighbors_with_exact_output() {
        use crate::decorations::plan::children::{CHILD_MEASUREMENTS, DISABLE_CHILD_REUSE};
        use crate::layout::{AddressRole, ComponentAddress};
        let owner = ComponentAddress::root()
            .child(AddressRole::FacetCell, 0)
            .unwrap();
        let neighbor = ComponentAddress::root()
            .child(AddressRole::FacetCell, 1)
            .unwrap();
        for scales in [FacetScales::Free, FacetScales::Fixed] {
            let source = Plot::scatter(
                crate::NumericValues::new("x", [0.0, 1.0, 0.0, 1.0]),
                crate::NumericValues::new("y", [0.0, 1.0, 1.0, 0.0]),
            )
            .facet(
                crate::FacetSpec::column(
                    crate::CategoryValues::new("facet", ["first", "first", "second", "second"])
                        .unwrap(),
                )
                .scales(scales),
            )
            .x_ticks(vec![
                crate::layout::AxisTick::new(0.0, "zero"),
                crate::layout::AxisTick::new(1.0, "one"),
            ])
            .text_annotation(
                crate::PlotTextAnnotation::new(crate::DataAreaCorner::TopLeft, "annotation")
                    .unwrap(),
            )
            .legend(crate::LegendPolicy::Hidden)
            .size(1200, 800)
            .erase();
            let mut expected = None;
            for disabled in [true, false] {
                DISABLE_CHILD_REUSE.with(|value| value.set(disabled));
                CHILD_MEASUREMENTS.with(|events| events.borrow_mut().clear());
                let appearance = crate::Theme::publication().resolve().unwrap();
                let mut calls = 0;
                let mut retained_neighbor = None;
                let root = crate::layout::plan_root(
                    source.description().sizing_request(),
                    |stage| {
                        let (registry, composition) = crate::plots::shared_x::prepare_plot(
                            source.description(),
                            &stage.operation(),
                            &appearance,
                        )?;
                        super::super::complete_prepared_plot_for_test(
                            source.description(),
                            appearance,
                            registry,
                            composition,
                            stage,
                        )
                    },
                    |prepared, assigned, round| {
                        calls += 1;
                        let super::super::root::PreparedComposition::Facets(topology) =
                            &prepared.composition
                        else {
                            panic!("facet topology")
                        };
                        let theme = prepared.request();
                        let mut caches = prepared.caches.borrow_mut();
                        let children = &mut caches.children;
                        let plan = OwnedFacetPlan::measure(
                            source.description(),
                            prepared,
                            topology,
                            prepared
                                .assignment_factory()
                                .assign(assigned.assigned_size()?),
                            &theme,
                            round,
                            children,
                            false,
                        )
                        .map_err(crate::layout::fit::PlanError::into_error)?;
                        let MeasuredRootContent::Facets(content) = &plan.content else {
                            panic!("measured facet content")
                        };
                        for (index, (child, registered)) in
                            content.children.iter().zip(&prepared.plots).enumerate()
                        {
                            assert_eq!(child.plot.plot_id, registered.scope.id);
                            assert_eq!(child.plot.selected.plot_id, child.plot.plot_id);
                            assert_eq!(
                                registered.scope.address,
                                crate::layout::ComponentAddress::root()
                                    .child(
                                        crate::layout::AddressRole::FacetCell,
                                        u32::try_from(index).unwrap()
                                    )
                                    .unwrap()
                            );
                        }
                        if calls == 2 {
                            retained_neighbor =
                                Some(std::sync::Arc::clone(&content.children[1].plot));
                            round.request_domain(
                                &owner,
                                &owner,
                                crate::layout::kernel::FeedbackKind::AutomaticDomainX,
                                (-1.0, 2.0),
                            )?;
                        } else if calls == 3 {
                            assert_eq!(
                                std::sync::Arc::ptr_eq(
                                    retained_neighbor.as_ref().unwrap(),
                                    &content.children[1].plot
                                ),
                                !disabled && scales == FacetScales::Free
                            );
                        }
                        children.finish_round(round);
                        Ok(crate::layout::MeasuredLayout::new(
                            assigned.assigned_size()?,
                            plan,
                        ))
                    },
                    |prepared, assigned, plan, stage| {
                        let MeasuredRootContent::Facets(content) = &plan.content else {
                            panic!("measured facet content")
                        };
                        let (width, height) = assigned.dimensions()?;
                        let panels = content
                            .children
                            .iter()
                            .map(|child| {
                                let panel = &child.plot;
                                let layout = &panel.result.layout;
                                let bounds = layout.projection.panel_box();
                                (
                                    panel.panel_box,
                                    layout.projection.x_range(),
                                    layout.projection.y_range(),
                                    layout.x_axis.ticks.clone(),
                                    layout.y_axis.ticks.clone(),
                                    panel
                                        .result
                                        .panel_text
                                        .iter()
                                        .map(|annotation| {
                                            (
                                                annotation.bounds_in(bounds),
                                                annotation.placement_in(bounds),
                                                annotation.text_plan.clone(),
                                            )
                                        })
                                        .collect::<Vec<_>>(),
                                )
                            })
                            .collect::<Vec<_>>();
                        let verifier = prepared.sealed_verifier()?;
                        let replay = crate::orchestrator::capture_calls(
                            assigned,
                            &verifier,
                            stage,
                            |backend| {
                                content.render(
                                    source.description(),
                                    PlotKind::Scatter,
                                    &plan.frame,
                                    backend,
                                )
                            },
                        )?;
                        crate::orchestrator::validate_text_replay_plan(
                            width,
                            height,
                            &replay,
                            "facet child reuse test",
                        )?;
                        let text = replay
                            .retained_text()
                            .into_iter()
                            .map(|(placement, text)| (placement, text.clone()))
                            .collect::<Vec<_>>();
                        for source in ["annotation", "zero", "one"] {
                            assert!(text.iter().any(|(_, plan)| plan.source == source));
                        }
                        let mut svg = replay.take_svg_backend()?;
                        replay.replay(width, height, &mut svg, "facet child reuse test")?;
                        Ok((svg.into_bytes()?, text, panels))
                    },
                )
                .unwrap();
                DISABLE_CHILD_REUSE.with(|value| value.set(false));
                assert_eq!(calls, 3);
                CHILD_MEASUREMENTS.with(|events| {
                    let events = events.borrow();
                    assert!(events.iter().all(|(_, phase)| *phase != "layout"));
                    for phase in ["probe", "panel"] {
                        assert_eq!(
                            events
                                .iter()
                                .filter(|(address, stage)| address == &owner && *stage == phase)
                                .count(),
                            3,
                            "dirty {phase}"
                        );
                        assert_eq!(
                            events
                                .iter()
                                .filter(|(address, stage)| address == &neighbor && *stage == phase)
                                .count(),
                            if disabled || (scales == FacetScales::Fixed && phase != "probe") {
                                3
                            } else {
                                2
                            },
                            "neighbor {phase}"
                        );
                    }
                });
                if let Some(expected) = &expected {
                    assert_eq!(root.plan(), expected);
                } else {
                    expected = Some(root.plan().clone());
                }
            }
        }
    }
}
