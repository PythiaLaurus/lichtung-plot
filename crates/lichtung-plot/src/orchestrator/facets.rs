//! Faceted chart coordination.

use crate::Error;
use std::fmt::Write;

use super::plan::{MeasuredOwnedPlan, MeasuredRootContent};
use crate::builder::Plot;
use crate::decorations::plan::ResolvedLegend;
use crate::layout::fit::{self, FitFailure, FitOutcome, FitScope};
use crate::layout::kernel::{FitKind, TrialOutcome};
use crate::render::backend::Backend;
use crate::spec::{FacetScales, LegendPosition, PlotKind};
use crate::text::TextRequestContext;

mod data;
mod legend;
#[cfg(test)]
mod parent_tests;
mod planning;
mod rendering;
mod sizing;
mod tracks;
pub(super) use sizing::natural_size;

pub(crate) fn text_census_labels(
    source: &Plot,
    mut visit: impl FnMut(crate::font::canonical::CensusSource<'_>) -> Result<(), Error>,
) -> Result<(), Error> {
    let selectors = data::facet_selectors(source.description())?;
    for panel in data::extract_panels(source.description(), &selectors, None)? {
        if source
            .description()
            .text_layouts()
            .facet
            .iter()
            .any(Option::is_some)
        {
            for (index, selector) in panel.cell.iter().enumerate() {
                let role = if panel.cell.as_slice().len() == 1 {
                    1
                } else {
                    index
                };
                if source.description().text_layouts().facet[role]
                    .as_ref()
                    .is_some_and(crate::TextLayout::hidden)
                {
                    continue;
                }
                visit(
                    format_args!(
                        "{} = {}",
                        source.description().resolve_label(&selector.variable)?,
                        source.description().resolve_label(&selector.level)?
                    )
                    .into(),
                )?;
            }
            continue;
        }
        match panel.cell.as_slice() {
            [] => visit("".into())?,
            [selector] => visit(
                format_args!(
                    "{} = {}",
                    source.description().resolve_label(&selector.variable)?,
                    source.description().resolve_label(&selector.level)?
                )
                .into(),
            )?,
            [row, column] => visit(
                format_args!(
                    "{} = {} | {} = {}",
                    source.description().resolve_label(&row.variable)?,
                    source.description().resolve_label(&row.level)?,
                    source.description().resolve_label(&column.variable)?,
                    source.description().resolve_label(&column.level)?
                )
                .into(),
            )?,
            _ => {
                return Err(Error::Render(
                    "facet text census has more than two selectors".into(),
                ))
            }
        }
    }
    Ok(())
}

pub(crate) fn prepare_facet_scopes<'src>(
    plot: &'src Plot,
    builder: &mut super::registration::PlotRegistryBuilder<'src>,
    operation: &crate::font::canonical::OperationContext,
) -> Result<(), Error> {
    let source = plot.description();
    if !source.significance_annotations().is_empty()
        && !source.kind().supports_significance_annotations()
    {
        return Err(Error::Data(format!(
            "significance annotations are not supported for {:?}",
            source.kind()
        )));
    }
    let selectors = data::facet_selectors(source)?;
    let panels = data::extract_panels(source, &selectors, None)?;
    data::validate_significance_facet_anchors(source, &panels)?;
    for (slot, panel) in panels.into_iter().enumerate() {
        let cell = match panel.cell.as_slice() {
            [one] => Some(crate::FacetCell::single(
                crate::data::source::copy_source_string(&one.level)?,
            )),
            [row, column] => Some(crate::FacetCell::grid(
                crate::data::source::copy_source_string(&row.level)?,
                crate::data::source::copy_source_string(&column.level)?,
            )),
            [] => None,
            _ => {
                return Err(Error::Render(
                    "numeric facet identity requires at most two selectors".into(),
                ))
            }
        };
        let render_name = panel.render_scope()?;
        let scope = super::registration::Scope::new(
            plot,
            operation,
            slot,
            crate::layout::ComponentAddress::root().child(
                crate::layout::AddressRole::FacetCell,
                u32::try_from(slot)
                    .map_err(|_| Error::Render("facet address overflowed".into()))?,
            )?,
            panel.rows,
            cell.as_ref(),
            Some(render_name),
        )?;
        builder.reserve(source, scope, operation)?;
    }
    Ok(())
}

pub(super) struct PreparedFacetGuide {
    pub(super) descriptor:
        Option<std::sync::Arc<crate::decorations::legend::discrete::PreparedGuide>>,
    pub(super) hue_levels: Option<Vec<String>>,
}

pub(super) fn prepare_discrete_guide(
    source: &Plot,
    kind: PlotKind,
    theme: &TextRequestContext<'_, '_>,
    inputs: &mut crate::layout::interior::PreparedInteriorInputs,
    resources: &crate::font::canonical::OperationContext,
) -> Result<PreparedFacetGuide, Error> {
    legend::prepare_discrete_guide(source, kind, theme, inputs, resources)
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
    slot: super::root::GridSlot,
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
        source: &Plot,
        operation: &crate::font::canonical::OperationContext,
        kind: PlotKind,
        theme: &TextRequestContext<'_, '_>,
        round: &mut crate::layout::kernel::FeedbackRound,
        children: &mut crate::decorations::plan::children::ChildPlans,
        prepared_guide: &PreparedFacetGuide,
        content_sizing: bool,
        inputs: &crate::layout::interior::PreparedInteriorInputs<'_>,
    ) -> Result<MeasuredOwnedPlan, crate::layout::fit::PlanError> {
        if !source.description().significance_annotations().is_empty()
            && !kind.supports_significance_annotations()
        {
            return Err(Error::Data(format!(
                "significance annotations are not supported for {kind:?}"
            ))
            .into());
        }

        let selectors = data::facet_selectors(source.description())?;
        let [horizontal_axis_label, vertical_axis_label] =
            source.description().prepare_axis_labels(None)?;
        for registered in &inputs.plots {
            registered.axes.retain_axis_labels([
                horizontal_axis_label.as_ref(),
                vertical_axis_label.as_ref(),
            ])?;
        }
        let mut canvas_layout = planning::canvas_layout(
            source,
            theme,
            horizontal_axis_label.as_ref(),
            vertical_axis_label.as_ref(),
            round,
        )?;
        let mut outer = canvas_layout.panel_box;
        round.observe_canvas(&canvas_layout)?;
        theme.text.prepared_font_environment()?;
        let (mut base_plot, _, _) = data::base_plot(source, kind)?;
        if let Some(levels) = &prepared_guide.hue_levels {
            base_plot = base_plot.with_global_hue_levels(crate::data::source::collect_source(
                levels
                    .iter()
                    .map(|level| crate::data::source::copy_source_string(level)),
            )?);
        }
        let mut panels = data::extract_panels(source.description(), &selectors, None)?;
        data::validate_significance_facet_anchors(source.description(), &panels)?;
        let mut minima = Vec::new();
        crate::data::source::reserve_source(&mut minima, panels.len())?;
        for index in 0..panels.len() {
            let scoped = base_plot.clone();
            minima.push(scoped.intrinsic_body_minimum(
                inputs.plots.get(index),
                inputs.intrinsic(&inputs.plots[index]),
                operation,
                theme,
            )?);
        }
        let provisional_content = content_sizing.then_some(tracks::ContentSizing {
            operation,
            plot: &base_plot,
            inputs,
            round,
            natural_width: false,
            natural_height: false,
        });
        let provisional = fit::measured(tracks::allocate(
            outer,
            (selectors.rows, selectors.columns),
            &panels,
            &minima,
            &[],
            theme,
            &FitScope::root(),
            provisional_content.as_ref(),
        ))?
        .cells;
        for facet in &mut panels {
            facet.panel = provisional[facet.row * selectors.columns + facet.column];
        }
        let facet_scales = source.description().facet_scales_spec();

        let (base_plot, preflight) = legend::preflight_coordinated_panels(
            source, base_plot, kind, &panels, inputs, theme, children, round,
        )?;
        let fix_x = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeY);
        let fix_y = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeX);
        let horizontal_domain = legend::resolve_shared_facet_axis(&preflight, fix_x, true)?;
        let vertical_domain = legend::resolve_shared_facet_axis(&preflight, fix_y, false)?;
        let shared_plot =
            base_plot.with_resolved_facet_ranges(kind, horizontal_domain, vertical_domain);
        let shared_plot = planning::feedback_plot(shared_plot, &panels, facet_scales, round)?;
        let discrete_guide = prepared_guide.descriptor.as_ref();
        let position = source.description().legend_position_spec();
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
            crate::data::source::collect_source((0..selectors.columns).map(|_| Ok(positive_span)))?;
        let mut rows =
            crate::data::source::collect_source((0..selectors.rows).map(|_| Ok(positive_span)))?;

        for (facet, hard) in panels.iter().zip(&minima) {
            columns[facet.column] = columns[facet.column].max(hard.0);
            rows[facet.row] = rows[facet.row].max(hard.1);
        }
        let mut minimum = (
            columns.iter().sum::<f64>()
                + selectors.columns.saturating_sub(1) as f64 * theme.composition_facet_gap()?,
            rows.iter().sum::<f64>()
                + selectors.rows.saturating_sub(1) as f64 * theme.composition_facet_gap()?,
        );
        if !minimum.0.is_finite() || !minimum.1.is_finite() {
            return Err(Error::Render("facet intrinsic grid span overflowed".into()).into());
        }
        // The guide and grid share one containing frame. Reserve the consumed
        // column/row gutters before measuring flow, not after fixing its width.
        let grid = crate::layout::ComponentAddress::root()
            .child(crate::layout::AddressRole::FacetGrid, 0)?;
        let mut columns = Vec::new();
        crate::data::source::reserve_source(&mut columns, selectors.columns)?;
        columns.resize(selectors.columns, [0.0_f64; 2]);
        let mut rows = Vec::new();
        crate::data::source::reserve_source(&mut rows, selectors.rows)?;
        rows.resize(selectors.rows, [0.0_f64; 2]);
        for (index, panel) in panels.iter().enumerate() {
            let address = grid.child(
                crate::layout::AddressRole::FacetCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("facet address overflowed".into()))?,
            )?;
            let [left, right, bottom, top] =
                round.insets(&address, crate::layout::kernel::FeedbackKind::AxisGutter);
            columns[panel.column][0] = columns[panel.column][0].max(left);
            columns[panel.column][1] = columns[panel.column][1].max(right);
            rows[panel.row][0] = rows[panel.row][0].max(bottom);
            rows[panel.row][1] = rows[panel.row][1].max(top);
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
                #[cfg(test)]
                let mut guide_trial = None;
                let (reserved_outer, mut outside_legend) = if outside {
                    round.claim_guide_trial(&owner, if automatic { 8 } else { 2 }, ordinal)?;
                    #[cfg(test)]
                    {
                        guide_trial =
                            Some(super::measurements::GuideTrial::new(&owner, edge, compact));
                    }
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
                            #[cfg(test)]
                            if let Some(trial) = &guide_trial {
                                trial.reject();
                            }
                            FitFailure::retain_least(&mut first_miss, reason);
                            continue;
                        }
                    };
                    #[cfg(test)]
                    if let Some(trial) = &guide_trial {
                        trial.body();
                    }
                    if content_sizing {
                        let grid_width = canvas_layout.panel_box.width()
                            - if horizontal {
                                0.0
                            } else {
                                guide.compute_box_width() + gap
                            };
                        let height = match sizing::grid_height(
                            &shared_plot,
                            operation,
                            &panels,
                            (selectors.rows, selectors.columns),
                            &minima,
                            theme,
                            round,
                            grid_width,
                            inputs,
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
                        let owner = source
                            .clone()
                            .with_canvas_size(source.canvas_dimensions()?.0, height);
                        canvas_layout = planning::canvas_layout(
                            &owner,
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
                            #[cfg(test)]
                            if let Some(trial) = &guide_trial {
                                trial.reject();
                            }
                            FitFailure::retain_least(&mut first_miss, reason);
                            continue;
                        }
                    }
                } else {
                    if content_sizing {
                        let height = fit::measured(sizing::grid_height(
                            &shared_plot,
                            operation,
                            &panels,
                            (selectors.rows, selectors.columns),
                            &minima,
                            theme,
                            round,
                            canvas_layout.panel_box.width(),
                            inputs,
                        ))?;
                        let height = crate::layout::kernel::root_dimension(
                            canvas_layout.height_for_body(height, theme)?,
                        )?;
                        let owner = source
                            .clone()
                            .with_canvas_size(source.canvas_dimensions()?.0, height);
                        canvas_layout = planning::canvas_layout(
                            &owner,
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
                            #[cfg(test)]
                            if let Some(trial) = &guide_trial {
                                trial.reject();
                            }
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
                        #[cfg(test)]
                        if let Some(trial) = &guide_trial {
                            trial.reject();
                        }
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
                        #[cfg(test)]
                        if let Some(trial) = &guide_trial {
                            trial.reject();
                        }
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
                    operation,
                    &shared_plot,
                    &mut panels,
                    outer,
                    (selectors.rows, selectors.columns),
                    facet_scales,
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
                    inputs,
                )?;
                let (planned_plot, mut children, grid) = match measured {
                    FitOutcome::Measured(value) => value,
                    FitOutcome::Miss(failure) => {
                        #[cfg(test)]
                        if let Some(trial) = &guide_trial {
                            trial.reject();
                        }
                        round.record_failure(failure)?;
                        if let TrialOutcome::Miss(failure) = round.finish_trial(checkpoint) {
                            FitFailure::retain_least(&mut first_miss, failure);
                        }
                        continue;
                    }
                };
                #[cfg(test)]
                let guide_outcome;
                match round.finish_trial(checkpoint) {
                    TrialOutcome::Miss(reason) => {
                        #[cfg(test)]
                        if let Some(trial) = &guide_trial {
                            trial.reject();
                        }
                        FitFailure::retain_least(&mut first_miss, reason);
                        continue;
                    }
                    _outcome @ (TrialOutcome::Fit | TrialOutcome::Provisional) => {
                        #[cfg(test)]
                        #[expect(
                            clippy::used_underscore_binding,
                            reason = "outcome is observed only in tests"
                        )]
                        {
                            guide_outcome = _outcome;
                        }
                    }
                }
                let mut strip_text = Vec::new();
                crate::render::backend::reserve_text_records(&mut strip_text, panels.len())?;
                for facet in &panels {
                    strip_text.push(rendering::plan_facet_strip(&planned_plot, facet, theme)?);
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
                for (index, (facet, child)) in panels.iter().zip(&children).enumerate() {
                    let panel = &child.plot;
                    let scope = facet.render_scope()?;
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
                    write!(layer, "facet-strip-{}", facet.panel_index).expect("reserved index");
                    strip_layers.push(layer);
                }
                let dimensions = (
                    source.canvas_dimensions()?.0,
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
                #[cfg(test)]
                if let Some(trial) = &guide_trial {
                    trial.complete(&guide_outcome);
                }
                return Ok(MeasuredOwnedPlan {
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
                legend.observe_feedback(round)?;
            }
        }
        rendering::observe_arranged_chrome(self, round)
    }

    pub(super) fn render(
        &self,
        source: &Plot,
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
        use crate::decorations::plan::children::{
            ChildPlans, CHILD_MEASUREMENTS, DISABLE_CHILD_REUSE,
        };
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
                let theme = crate::Theme::publication().resolve().unwrap();
                let text = crate::text::TextService::default();
                let theme = crate::text::TextRequestContext::new(&theme, &text);
                let mut children = ChildPlans::facets();
                let mut calls = 0;
                let mut retained_neighbor = None;
                let root = crate::layout::plan_root(
                    source.description().sizing_request(),
                    |stage| {
                        let mut inputs = crate::plots::shared_x::prepare_plot(
                            &source,
                            &stage.operation(),
                            Some(&crate::theme::Theme::publication()),
                        )?;
                        let guide = prepare_discrete_guide(
                            &source,
                            PlotKind::Scatter,
                            &theme,
                            &mut inputs,
                            &stage.operation(),
                        )?;
                        let requirements = crate::layout::foundational_plot_requirements(
                            Some(&inputs),
                            &source,
                            PlotKind::Scatter,
                            &theme,
                            &ComponentAddress::root(),
                            true,
                            true,
                            true,
                            Some(stage),
                        )?;
                        let mut presence = stage.reserve_scalar_presence_map()?;
                        let mut census = crate::layout::PreparedTextCensus::materialize(
                            requirements,
                            Some(stage),
                            Some(&mut presence),
                        )
                        .map_err(|error| Error::Render(error.to_string()))?;
                        theme.text.prepare_font_environment_from_census(
                            &mut census,
                            presence,
                            stage,
                        )?;
                        Ok((inputs, guide, stage.operation()))
                    },
                    |(inputs, guide, operation), assigned, round| {
                        let source = inputs.attach(source.clone())?;
                        calls += 1;
                        let plan = OwnedFacetPlan::measure(
                            &source,
                            operation,
                            PlotKind::Scatter,
                            &theme,
                            round,
                            &mut children,
                            guide,
                            false,
                            inputs,
                        )
                        .map_err(crate::layout::fit::PlanError::into_error)?;
                        let MeasuredRootContent::Facets(content) = &plan.content else {
                            panic!("measured facet content")
                        };
                        for (index, (child, registered)) in
                            content.children.iter().zip(&inputs.plots).enumerate()
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
                    |_, assigned, plan, stage| {
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
                        let replay = crate::orchestrator::capture_text_replay_plan(
                            width,
                            height,
                            "facet child reuse test",
                            &theme,
                            stage,
                            |backend| {
                                content.render(&source, PlotKind::Scatter, &plan.frame, backend)
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

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "complete parent capture retains every producer and checks stage counters"
    )]
    fn facet_producer_plans_survive_first_capture_without_shaping() {
        let text = format!(
            "{}AA\u{2029}A \u{202b}\u{5d0} A \u{5d0} \u{202c}A",
            "A\u{ad}".repeat(60)
        );
        let source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0, 0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0, 1.0, 0.0]),
        )
        .facet(crate::FacetSpec::column(
            crate::CategoryValues::new("facet", [&text, &text, "second", "second"]).unwrap(),
        ))
        .facet_column_text_layout(
            crate::TextLayout::default()
                .anchor(crate::TextAnchor::top_center())
                .rotation(crate::TextRotation::degrees(30.05).unwrap())
                .rotation_mode(crate::TextRotationMode::AlignAfterRotation),
        )
        .x_label(&text)
        .y_label(&text)
        .title("Outer title\nShort")
        .subtitle("Independent subtitle")
        .caption("Common caption")
        .hue(crate::CategoryValues::new("hue", ["a", "b", "a", "b"]).unwrap())
        .size(900, 900)
        .erase();
        let theme = source.description().resolve_theme().unwrap();
        let text_service = crate::text::TextService::default();
        let theme = crate::text::TextRequestContext::new(&theme, &text_service);
        crate::layout::plan_root(
            source.description().sizing_request(),
            |stage| {
                let mut inputs = crate::plots::shared_x::prepare_plot(
                    &source,
                    &stage.operation(),
                    Some(&crate::theme::Theme::publication()),
                )?;
                assert!(inputs.plots.iter().all(|registered| registered
                    .axes
                    .sources()
                    .iter()
                    .all(|source| source.label.get().is_none())));
                let guide = prepare_discrete_guide(
                    &source,
                    PlotKind::Scatter,
                    &theme,
                    &mut inputs,
                    &stage.operation(),
                )?;
                let requirements = crate::layout::foundational_plot_requirements(
                    Some(&inputs),
                    &source,
                    PlotKind::Scatter,
                    &theme,
                    &crate::layout::ComponentAddress::root(),
                    true,
                    true,
                    true,
                    Some(stage),
                )?;
                let mut presence = stage.reserve_scalar_presence_map()?;
                let mut census = crate::layout::PreparedTextCensus::materialize(
                    requirements,
                    Some(stage),
                    Some(&mut presence),
                )
                .map_err(|error| Error::Render(error.to_string()))?;
                theme
                    .text
                    .prepare_font_environment_from_census(&mut census, presence, stage)?;
                Ok((inputs, guide, stage.operation()))
            },
            |(inputs, guide, operation), assigned, round| {
                let source = inputs.attach(source.clone())?;
                assert!(inputs
                    .plots
                    .iter()
                    .all(
                        |registered| (0..2).all(|side| registered.axes.sources()[side]
                            .label
                            .get()
                            .is_some()
                            && registered.axes.label(registered, side).is_none())
                    ));
                Ok(crate::layout::MeasuredLayout::new(
                    assigned.assigned_size()?,
                    OwnedFacetPlan::measure(
                        &source,
                        operation,
                        PlotKind::Scatter,
                        &theme,
                        round,
                        &mut crate::decorations::plan::children::ChildPlans::facets(),
                        guide,
                        false,
                        inputs,
                    )
                    .map_err(crate::layout::fit::PlanError::into_error)?,
                ))
            },
            |_, assigned, plan, stage| {
                let MeasuredRootContent::Facets(content) = &plan.content else {
                    panic!("measured facet content")
                };
                let expected = content
                    .children
                    .iter()
                    .flat_map(|child| &child.strips)
                    .chain(&content.shared_titles)
                    .collect::<Vec<_>>();
                assert!(expected.iter().any(|text| text
                    .plan
                    .lines
                    .iter()
                    .any(|line| line.selected_soft_hyphen.is_some())));
                assert!(content
                    .shared_titles
                    .iter()
                    .all(|planned| planned.plan.source == text));
                assert!(content
                    .children
                    .iter()
                    .flat_map(|child| &child.strips)
                    .all(|text| text
                        .plan
                        .request
                        .as_ref()
                        .is_some_and(|request| request.instructions().anchor
                            == Some(crate::TextAnchor::top_center()))));
                crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
                let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
                let projections = crate::layout::Projection::test_projection_calls();
                let (width, height) = assigned.dimensions()?;
                let replay = crate::orchestrator::capture_text_replay_plan(
                    width,
                    height,
                    "facet producer closure",
                    &theme,
                    stage,
                    |backend| content.render(&source, PlotKind::Scatter, &plan.frame, backend),
                )?;
                crate::font::canonical::FONT_SHAPE_CALLS
                    .with(|calls| assert_eq!(calls.get(), 0, "first capture must not shape"));
                assert_eq!(
                    crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                    reads
                );
                assert_eq!(
                    crate::layout::Projection::test_projection_calls(),
                    projections
                );
                let captured = replay.retained_text();
                for text in expected {
                    let canonical = crate::render::primitives::CanonicalPlannedText::new(
                        text.placement,
                        &text.plan,
                        height,
                        None,
                    )?;
                    assert!(captured
                        .iter()
                        .any(|(placement, actual)| *placement == canonical
                            && actual.shares_content(&text.plan)
                            && **actual == text.plan.freeze_selected()));
                }
                Ok(())
            },
        )
        .unwrap();
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one actual containing-frame trial verifies the selected parent guide and shared title after the complete grid remeasurement"
    )]
    fn automatic_shared_facet_guide_remeasures_the_whole_grid() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0, 1.0, 2.0]),
            crate::NumericValues::new("y", [1.0, 2.0, 2.0, 3.0]),
        )
        .hue(
            crate::CategoryValues::new(
                "hue",
                [
                    "Complete explanatory first condition",
                    "Complete explanatory second condition",
                    "Complete explanatory first condition",
                    "Complete explanatory second condition",
                ],
            )
            .unwrap(),
        )
        .facet(crate::FacetSpec::column(
            crate::CategoryValues::new("facet", ["first", "first", "second", "second"]).unwrap(),
        ))
        .x_label("MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM")
        .size(960, 640)
        .erase();
        let theme = crate::Theme::publication().resolve().unwrap();
        let text = crate::text::TextService::default();
        let theme = crate::text::TextRequestContext::new(&theme, &text);
        let root = crate::layout::plan_root(
            source.description().sizing_request(),
            |stage| {
                let mut inputs = crate::plots::shared_x::prepare_plot(
                    &source,
                    &stage.operation(),
                    Some(&crate::theme::Theme::publication()),
                )?;
                let guide = prepare_discrete_guide(
                    &source,
                    PlotKind::Scatter,
                    &theme,
                    &mut inputs,
                    &stage.operation(),
                )?;
                let requirements = crate::layout::foundational_plot_requirements(
                    Some(&inputs),
                    &source,
                    PlotKind::Scatter,
                    &theme,
                    &crate::layout::ComponentAddress::root(),
                    true,
                    true,
                    true,
                    Some(stage),
                )?;
                let mut presence = stage.reserve_scalar_presence_map()?;
                let mut census = crate::layout::PreparedTextCensus::materialize(
                    requirements,
                    Some(stage),
                    Some(&mut presence),
                )
                .map_err(|error| Error::Render(format!("facet D1 census: {error}")))?;
                theme
                    .text
                    .prepare_font_environment_from_census(&mut census, presence, stage)?;
                Ok((inputs, guide, stage.operation()))
            },
            |(inputs, guide, operation), assigned, round| {
                let source = inputs.attach(source.clone())?;
                Ok(crate::layout::MeasuredLayout::new(
                    assigned.assigned_size()?,
                    OwnedFacetPlan::measure(
                        &source,
                        operation,
                        PlotKind::Scatter,
                        &theme,
                        round,
                        &mut crate::decorations::plan::children::ChildPlans::facets(),
                        guide,
                        false,
                        inputs,
                    )
                    .map_err(crate::layout::fit::PlanError::into_error)?,
                ))
            },
            |_, _, plan, _| Ok(plan),
        )
        .unwrap();
        let MeasuredRootContent::Facets(plan) = &root.plan().content else {
            panic!("measured facet content")
        };
        let mut geometry = crate::render::geometry::GeometrySnapshot::new();
        plan.parent_guide
            .as_ref()
            .unwrap()
            .1
            .geometry(&mut geometry)
            .unwrap();
        let guide_bottom = geometry
            .bounds()
            .iter()
            .map(|ink| ink.bottom)
            .reduce(f64::min)
            .unwrap();
        assert!(plan
            .children
            .iter()
            .all(|child| guide_bottom > plan.cells[child.slot.0].y1));
        assert_eq!(
            plan.shared_titles[0].plan.source,
            source.description().prepare_axis_labels(None).unwrap()[0]
                .as_ref()
                .unwrap()
                .as_str()
        );
    }

    #[test]
    fn fatal_scatter_facet_source_rejects_before_fonts_or_layout() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0, f64::INFINITY, 3.0]),
            crate::NumericValues::new("y", [1.0, 2.0, 2.0, 3.0]),
        )
        .facet(crate::FacetSpec::column(
            crate::CategoryValues::new("facet", ["first", "first", "second", "second"]).unwrap(),
        ))
        .legend(crate::LegendPolicy::Hidden)
        .size(960, 640)
        .erase();
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let result = crate::layout::plan_root(
            source.description().sizing_request(),
            |stage| {
                crate::plots::shared_x::prepare_plot(
                    &source,
                    &stage.operation(),
                    Some(&crate::theme::Theme::publication()),
                )
            },
            |_, _, _| -> Result<crate::layout::MeasuredLayout<()>, Error> {
                panic!("invalid semantics must reject before layout")
            },
            |_, _, (), _| -> Result<(), Error> {
                panic!("a failed sibling set must never finalize")
            },
        );
        let error = result.err().expect("fatal second facet");
        assert!(matches!(error, Error::Data(_)), "{error}");
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}
