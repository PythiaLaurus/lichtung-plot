//! Faceted chart coordination.

use crate::Error;
use std::fmt::Write;

use crate::builder::Plot;
use crate::decorations::plan::ResolvedLegend;
use crate::layout::fit::{self, FitFailure, FitOutcome, FitScope};
use crate::layout::kernel::{FitKind, TrialOutcome};
use crate::render::backend::Backend;
use crate::spec::{FacetScales, LegendPosition, PlotKind};
use crate::theme::ResolvedTheme;

mod data;
mod legend;
#[cfg(test)]
mod parent_tests;
mod planning;
mod rendering;

pub(crate) fn text_census_labels(
    source: &Plot,
    mut visit: impl FnMut(crate::font::canonical::CensusSource<'_>) -> Result<(), Error>,
) -> Result<(), Error> {
    let selectors = data::facet_selectors(source, source.data())?;
    for panel in data::extract_panels(source, &selectors, None, source.data())? {
        match panel.cell.as_slice() {
            [] => visit("".into())?,
            [selector] => visit(
                format_args!(
                    "{} = {}",
                    source.resolve_source_label(&selector.variable, source.data())?,
                    source.resolve_source_label(&selector.level, source.data())?
                )
                .into(),
            )?,
            [row, column] => visit(
                format_args!(
                    "{} = {} | {} = {}",
                    source.resolve_source_label(&row.variable, source.data())?,
                    source.resolve_source_label(&row.level, source.data())?,
                    source.resolve_source_label(&column.variable, source.data())?,
                    source.resolve_source_label(&column.level, source.data())?
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

pub(crate) struct FacetTextCensusPanel<'a> {
    plot: &'a Plot,
    panel: data::FacetPanel,
}

impl FacetTextCensusPanel<'_> {
    pub(crate) fn plot(&self) -> &Plot {
        self.plot
    }

    pub(crate) fn data(&self) -> crate::data::PlotData<'_> {
        crate::data::PlotData::with_rows(self.plot, &self.panel.rows, &self.panel.source_rows)
    }

    pub(crate) fn scope(&self, data: crate::data::PlotData<'_>) -> Result<String, Error> {
        self.panel.render_scope(data)
    }

    pub(crate) fn significance_annotations(
        &self,
        data: crate::data::PlotData<'_>,
    ) -> Result<impl Iterator<Item = &crate::significance::SignificanceAnnotation>, Error> {
        for _ in self.plot.significance_annotations() {
            data.claim_source_visits(1)?;
            for selector in &self.panel.cell {
                data.claim_source_visits(selector.level.len())?;
            }
        }
        Ok(self
            .plot
            .significance_annotations()
            .iter()
            .filter(|annotation| {
                annotation.facet_cell().is_some_and(|cell| {
                    cell.matches(
                        self.panel
                            .cell
                            .iter()
                            .map(|selector| selector.level.as_str()),
                    )
                })
            }))
    }
}

pub(crate) fn text_census_panels<'a>(
    source: &'a Plot,
    source_data: crate::data::PlotData<'_>,
) -> Result<Vec<FacetTextCensusPanel<'a>>, Error> {
    if !source.significance_annotations().is_empty()
        && !source.kind().supports_significance_annotations()
    {
        return Err(Error::Data(format!(
            "significance annotations are not supported for {:?}",
            source.kind()
        )));
    }
    let selectors = data::facet_selectors(source, source_data)?;
    let panels = data::extract_panels(source, &selectors, None, source_data)?;
    data::validate_significance_facet_anchors(source, &panels, source_data)?;
    source_data.collect_source(panels.into_iter().map(|panel| {
        Ok(FacetTextCensusPanel {
            plot: source,
            panel,
        })
    }))
}

pub(super) struct PreparedFacetGuide {
    pub(super) entries: std::sync::Arc<[crate::decorations::legend::LegendEntry]>,
    pub(super) title: String,
    pub(super) hue_levels: Option<Vec<String>>,
}

pub(super) fn prepare_discrete_guide(
    source: &Plot,
    kind: PlotKind,
    theme: &ResolvedTheme,
    inputs: &crate::layout::interior::PreparedInteriorInputs,
    resources: &crate::font::canonical::OperationGenerationResources,
) -> Result<PreparedFacetGuide, Error> {
    legend::prepare_discrete_guide(source, kind, theme, inputs, resources)
}

pub(super) struct FacetPlan {
    panels: Vec<data::FacetPanel>,
    panel_plans: Vec<std::sync::Arc<crate::decorations::plan::PanelPlan>>,
    outside_legend: Option<ResolvedLegend>,
    horizontal_axis_label: Option<String>,
    vertical_axis_label: Option<String>,
    canvas_layout: crate::layout::RootFramePlan,
    outer: crate::layout::PanelBox,
    shared_axis_origin: (f64, f64),
    strip_text: Vec<crate::render::backend::PlacedText>,
    shared_axis_text: Vec<crate::render::backend::PlacedText>,
    panel_plots: Vec<Plot>,
    legends: Vec<Option<crate::decorations::legend::LegendDrawPlan>>,
    strip_layers: Vec<String>,
    panel_layers: Vec<[String; 3]>,
}

impl FacetPlan {
    #[expect(
        clippy::too_many_lines,
        clippy::cast_precision_loss,
        reason = "the legacy parent adapter keeps complete-frame guide trials explicit"
    )]
    pub(super) fn new(
        source: &Plot,
        kind: PlotKind,
        theme: &ResolvedTheme,
        round: &mut crate::layout::kernel::FeedbackRound,
        children: &mut crate::decorations::plan::children::ChildPlans,
        prepared_guide: &PreparedFacetGuide,
    ) -> Result<Self, crate::layout::fit::PlanError> {
        if !source.significance_annotations().is_empty()
            && !kind.supports_significance_annotations()
        {
            return Err(Error::Data(format!(
                "significance annotations are not supported for {kind:?}"
            ))
            .into());
        }

        let selectors = data::facet_selectors(source, source.data())?;
        let [horizontal_axis_label, vertical_axis_label] =
            source.prepare_axis_labels(source.data(), None)?;
        let horizontal_axis_label =
            (!horizontal_axis_label.is_empty()).then_some(horizontal_axis_label);
        let vertical_axis_label = (!vertical_axis_label.is_empty()).then_some(vertical_axis_label);
        let canvas_layout = planning::canvas_layout(
            source,
            theme,
            horizontal_axis_label.as_deref(),
            vertical_axis_label.as_deref(),
            round,
        )?;
        let mut outer = canvas_layout.panel_box;
        round.observe_canvas(&canvas_layout)?;
        let provisional = fit::measured(planning::allocate_legacy_grid(
            source.data(),
            outer,
            selectors.rows,
            selectors.columns,
            &[],
            &FitScope::root(),
        ))?;
        let mut base_plot = data::base_plot(
            source,
            kind,
            horizontal_axis_label.is_some(),
            vertical_axis_label.is_some(),
            &theme.prepared_font_environment()?.generation_resources(),
        )?;
        if let Some(levels) = &prepared_guide.hue_levels {
            let resources = theme.prepared_font_environment()?.generation_resources();
            let data = source.data().with_axis_resources(&resources);
            base_plot = base_plot.with_global_hue_levels(
                data.collect_source(levels.iter().map(|level| data.copy_source_string(level)))?,
            );
        }
        let mut panels =
            data::extract_panels(source, &selectors, Some(&provisional), source.data())?;
        data::validate_significance_facet_anchors(source, &panels, source.data())?;
        let facet_scales = source.facet_scales_spec();

        let (base_plot, preflight) = legend::preflight_coordinated_panels(
            source, base_plot, kind, &panels, theme, children, round,
        )?;
        let fix_x = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeY);
        let fix_y = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeX);
        let horizontal_domain = legend::resolve_shared_facet_axis(&preflight, fix_x, true)?;
        let vertical_domain = legend::resolve_shared_facet_axis(&preflight, fix_y, false)?;
        let shared_plot = base_plot
            .with_resolved_facet_ranges(kind, horizontal_domain, vertical_domain)
            .with_parent_axis_gutters();
        let shared_plot = planning::feedback_plot(shared_plot, &panels, facet_scales, round)?;
        let legend_entries = std::sync::Arc::clone(&prepared_guide.entries);
        let title = prepared_guide.title.as_str();
        let position = source.legend_position_spec();
        let automatic = position == LegendPosition::Auto && !legend_entries.is_empty();
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
        let mut minimum = (
            selectors.columns as f64 * crate::layout::interior::DeferredLegacyPanelMinimum::SIZE.0
                + selectors.columns.saturating_sub(1) as f64 * 12.0,
            selectors.rows as f64 * crate::layout::interior::DeferredLegacyPanelMinimum::SIZE.1
                + selectors.rows.saturating_sub(1) as f64 * 12.0,
        );
        // The guide and grid share one containing frame. Reserve the consumed
        // column/row gutters before measuring flow, not after fixing its width.
        let grid = crate::layout::ComponentAddress::root()
            .child(crate::layout::AddressRole::FacetGrid, 0)?;
        let mut columns = Vec::new();
        source
            .data()
            .reserve_source(&mut columns, selectors.columns)?;
        columns.resize(selectors.columns, [0.0_f64; 2]);
        let mut rows = Vec::new();
        source.data().reserve_source(&mut rows, selectors.rows)?;
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
            || (!legend_entries.is_empty() && crate::decorations::plan::is_outside(position))
        {
            planning::shared_axis_line_counts(
                horizontal_axis_label.as_deref(),
                vertical_axis_label.as_deref(),
                theme,
                canvas_layout.panel_box,
            )?
        } else {
            (0, 0)
        };
        let mut first_miss = None;
        for (edge_index, &edge) in edges.iter().enumerate() {
            let outside = !legend_entries.is_empty() && crate::decorations::plan::is_outside(edge);
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
                    let metrics = crate::decorations::legend::LegendMetrics::from_theme(theme);
                    let frame = crate::decorations::plan::AssignedGuideFrame::new(
                        edge,
                        minimum,
                        metrics.outside_gap,
                    );
                    let available = frame.available(canvas_layout.panel_box, [0.0; 4]);
                    let guide = match crate::decorations::legend::Legend::measure_assigned(
                        title,
                        std::sync::Arc::clone(&legend_entries),
                        available,
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
                    (canvas_layout.panel_box, None)
                };
                if outside {
                    let mut lines = [0_usize; 2];
                    for (index, (label, span)) in [
                        (horizontal_axis_label.as_deref(), reserved_outer.width()),
                        (vertical_axis_label.as_deref(), reserved_outer.height()),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        if let Some(label) = label {
                            match fit::text(
                                theme,
                                label,
                                &theme.typography.axis_title,
                                span,
                                crate::layout::PhysicalTextAlignment::Center,
                                "shared facet axis title",
                                &scope.child(
                                    if index == 0 {
                                        crate::layout::AddressRole::AxisBottom
                                    } else {
                                        crate::layout::AddressRole::AxisLeft
                                    },
                                    0,
                                )?,
                            )? {
                                FitOutcome::Measured(plan) => lines[index] = plan.lines.len(),
                                FitOutcome::Miss(reason) => {
                                    #[cfg(test)]
                                    if let Some(trial) = &guide_trial {
                                        trial.reject();
                                    }
                                    FitFailure::retain_least(&mut first_miss, reason);
                                    continue 'trial;
                                }
                            }
                        }
                    }
                    if reserved_outer.width()
                        - lines[1].saturating_sub(baseline.1) as f64
                            * theme.typography.axis_title.line_height()
                        < minimum.0
                        || reserved_outer.height()
                            - lines[0].saturating_sub(baseline.0) as f64
                                * theme.typography.axis_title.line_height()
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
                    horizontal_axis_label.as_deref(),
                    vertical_axis_label.as_deref(),
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
                let measured = planning::legacy_parent_adapter(
                    &shared_plot,
                    kind,
                    &mut panels,
                    outer,
                    (selectors.rows, selectors.columns),
                    facet_scales,
                    theme,
                    round,
                    &preflight,
                    &provisional,
                    outside,
                    &scope,
                    children,
                )?;
                if let FitOutcome::Miss(reason) = measured {
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
                let (planned_plot, panel_plans) = match planning::plan_panels(
                    shared_plot.clone(),
                    kind,
                    &panels,
                    outside_legend.is_some(),
                    &legend_entries,
                    facet_scales,
                    theme,
                    round,
                    children,
                )? {
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
                crate::render::backend::reserve_text_records(
                    &mut strip_text,
                    panels.len(),
                    theme.prepared_font_environment()?,
                )?;
                for facet in &panels {
                    strip_text.push(rendering::plan_facet_strip(&planned_plot, facet, theme)?);
                }
                let mut panel_plots = Vec::new();
                let mut legends = Vec::new();
                let mut strip_layers = Vec::new();
                let mut panel_layers = Vec::new();
                let resources = theme.prepared_font_environment()?.generation_resources();
                let data = source.data().with_axis_resources(&resources);
                data.reserve_source(&mut panel_layers, panels.len())?;
                data.reserve_source(&mut panel_plots, panels.len())?;
                data.reserve_source(&mut legends, panels.len())?;
                data.reserve_source(&mut strip_layers, panels.len())?;
                for (index, (facet, panel)) in panels.iter().zip(&panel_plans).enumerate() {
                    let scope = facet.render_scope(data)?;
                    let mut layers = [String::new(), String::new(), String::new()];
                    for (layer, suffix) in layers.iter_mut().zip(["grid", "axes", "legend"]) {
                        let length = scope
                            .len()
                            .checked_add(suffix.len() + 1)
                            .ok_or_else(|| Error::Render("facet layer length overflow".into()))?;
                        data.reserve_source_string(layer, length)?;
                        layer.push_str(&scope);
                        layer.push('-');
                        layer.push_str(suffix);
                    }
                    panel_layers.push(layers);
                    let plot = panel
                        .plot_with_planned_limits(&planned_plot, data)?
                        .with_render_scope(scope)
                        .with_significance_subset(facet.significance_selectors());
                    panel_plots.push(plot);
                    let legend = if index == 0 {
                        outside_legend.as_ref().or(panel.legend.as_ref())
                    } else {
                        panel.legend.as_ref()
                    };
                    legends.push(
                        legend
                            .map(|legend| {
                                let (x, y) = legend.bounds.anchor();
                                legend.legend.select(x, y, source.data())
                            })
                            .transpose()?,
                    );
                    let mut layer = String::new();
                    source.data().reserve_source_string(&mut layer, 32)?;
                    write!(layer, "facet-strip-{}", facet.panel_index).expect("reserved index");
                    strip_layers.push(layer);
                }
                let plan = Self {
                    panel_layers,
                    panel_plots,
                    legends,
                    strip_layers,
                    strip_text,
                    shared_axis_text: rendering::plan_shared_facet_axis_labels(
                        horizontal_axis_label.as_deref(),
                        vertical_axis_label.as_deref(),
                        outer,
                        shared_axis_origin,
                        theme,
                    )?,
                    panels,
                    panel_plans,
                    outside_legend,
                    horizontal_axis_label,
                    vertical_axis_label,
                    canvas_layout,
                    outer,
                    shared_axis_origin,
                };
                plan.observe_arranged(theme, round)?;
                #[cfg(test)]
                if let Some(trial) = &guide_trial {
                    trial.complete(&guide_outcome);
                }
                return Ok(plan);
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
        theme: &ResolvedTheme,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        round.begin_owner(
            &crate::layout::ComponentAddress::root()
                .child(crate::layout::AddressRole::FacetGrid, 0)?,
        )?;
        round.observe_box(self.outer)?;
        for label in [&self.horizontal_axis_label, &self.vertical_axis_label]
            .into_iter()
            .flatten()
        {
            round.observe_string(label)?;
        }
        round.observe_number(self.shared_axis_origin.0)?;
        round.observe_number(self.shared_axis_origin.1)?;
        round.observe_count(self.panel_plans.len())?;
        for (index, plan) in self.panel_plans.iter().enumerate() {
            round.observe_box(plan.panel_box)?;
            let legend = if index == 0 {
                self.outside_legend.as_ref().or(plan.legend.as_ref())
            } else {
                plan.legend.as_ref()
            };
            round.observe_count(usize::from(legend.is_some()))?;
            if let Some(legend) = legend {
                legend.observe_feedback(round)?;
            }
        }
        rendering::observe_arranged_chrome(self, theme, round)
    }

    pub(super) fn render(
        &self,
        source: &Plot,
        kind: PlotKind,
        theme: &ResolvedTheme,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        rendering::render_scene(
            source,
            &self.panel_plots,
            kind,
            &self.panels,
            &self.panel_plans,
            &self.legends,
            &self.strip_layers,
            &self.panel_layers,
            &self.strip_text,
            &self.shared_axis_text,
            &self.canvas_layout,
            theme,
            backend,
        )
    }
}

fn position_shared_axis_guides(
    position: LegendPosition,
    canvas_panel: crate::layout::PanelBox,
    reserved_outer: crate::layout::PanelBox,
    outer: crate::layout::PanelBox,
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
            .panel_text_annotation(
                crate::PanelTextAnnotation::new(crate::PanelCorner::TopLeft, "annotation").unwrap(),
            )
            .legend(crate::LegendPolicy::Hidden)
            .size(1200, 800)
            .erase();
            let mut expected = None;
            for disabled in [true, false] {
                DISABLE_CHILD_REUSE.with(|value| value.set(disabled));
                CHILD_MEASUREMENTS.with(|events| events.borrow_mut().clear());
                let theme = crate::Theme::Publication.resolve();
                let mut children = ChildPlans::facets(&source);
                let mut calls = 0;
                let mut retained_neighbor = None;
                let root = crate::layout::plan_exact_root(
                    source.width(),
                    source.height(),
                    |stage| {
                        let inputs = crate::plots::shared_x::prepare_plot(
                            &source,
                            &stage.generation_resources(),
                        )?;
                        let guide = prepare_discrete_guide(
                            &source,
                            PlotKind::Scatter,
                            &theme,
                            &inputs,
                            &stage.generation_resources(),
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
                        theme.prepare_font_environment_from_census(&mut census, presence, stage)?;
                        Ok((inputs, guide))
                    },
                    |(inputs, guide), assigned, round| {
                        let source = inputs.attach(source.clone())?;
                        calls += 1;
                        let plan = FacetPlan::new(
                            &source,
                            PlotKind::Scatter,
                            &theme,
                            round,
                            &mut children,
                            guide,
                        )
                        .map_err(crate::layout::fit::PlanError::into_error)?;
                        if calls == 2 {
                            retained_neighbor = Some(std::sync::Arc::clone(&plan.panel_plans[1]));
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
                                    &plan.panel_plans[1]
                                ),
                                !disabled && scales == FacetScales::Free
                            );
                        }
                        children.finish_round(round);
                        Ok(crate::layout::MeasuredLayout::new(assigned, plan))
                    },
                    |_, assigned, plan, stage| {
                        let (width, height) = assigned.dimensions();
                        let panels = plan
                            .panel_plans
                            .iter()
                            .map(|panel| {
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
                            |backend| plan.render(&source, PlotKind::Scatter, &theme, backend),
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
                    for phase in ["probe", "layout", "panel"] {
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
        .x_label(&text)
        .y_label(&text)
        .title("Outer title\nShort")
        .subtitle("Independent subtitle")
        .caption("Common caption")
        .hue(crate::CategoryValues::new("hue", ["a", "b", "a", "b"]).unwrap())
        .size(900, 900)
        .erase();
        let theme = crate::Theme::Publication.resolve();
        crate::layout::plan_exact_root(
            source.width(),
            source.height(),
            |stage| {
                let inputs =
                    crate::plots::shared_x::prepare_plot(&source, &stage.generation_resources())?;
                let guide = prepare_discrete_guide(
                    &source,
                    PlotKind::Scatter,
                    &theme,
                    &inputs,
                    &stage.generation_resources(),
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
                theme.prepare_font_environment_from_census(&mut census, presence, stage)?;
                Ok((inputs, guide))
            },
            |(inputs, guide), assigned, round| {
                let source = inputs.attach(source.clone())?;
                Ok(crate::layout::MeasuredLayout::new(
                    assigned,
                    FacetPlan::new(
                        &source,
                        PlotKind::Scatter,
                        &theme,
                        round,
                        &mut crate::decorations::plan::children::ChildPlans::facets(&source),
                        guide,
                    )
                    .map_err(crate::layout::fit::PlanError::into_error)?,
                ))
            },
            |_, assigned, plan, stage| {
                let expected = plan
                    .strip_text
                    .iter()
                    .chain(&plan.shared_axis_text)
                    .collect::<Vec<_>>();
                assert!(expected.iter().any(|text| text
                    .plan
                    .lines
                    .iter()
                    .any(|line| line.selected_soft_hyphen.is_some())));
                assert!(plan
                    .shared_axis_text
                    .iter()
                    .all(|planned| planned.plan.source == text));
                crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
                let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
                let projections = crate::layout::Projection::test_projection_calls();
                let (width, height) = assigned.dimensions();
                let replay = crate::orchestrator::capture_text_replay_plan(
                    width,
                    height,
                    "facet producer closure",
                    &theme,
                    stage,
                    |backend| plan.render(&source, PlotKind::Scatter, &theme, backend),
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
                    assert!(captured
                        .iter()
                        .any(|(placement, actual)| *placement == text.placement
                            && actual.shares_payload(&text.plan)
                            && **actual == text.plan));
                }
                Ok(())
            },
        )
        .unwrap();
    }

    #[test]
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
        let theme = crate::Theme::Publication.resolve();
        let root = crate::layout::plan_exact_root(
            source.width(),
            source.height(),
            |stage| {
                let inputs =
                    crate::plots::shared_x::prepare_plot(&source, &stage.generation_resources())?;
                let guide = prepare_discrete_guide(
                    &source,
                    PlotKind::Scatter,
                    &theme,
                    &inputs,
                    &stage.generation_resources(),
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
                    None,
                )?;
                let mut presence = stage.reserve_scalar_presence_map()?;
                let mut census = crate::layout::PreparedTextCensus::materialize(
                    requirements,
                    Some(stage),
                    Some(&mut presence),
                )
                .map_err(|error| Error::Render(format!("facet D1 census: {error}")))?;
                theme.prepare_font_environment_from_census(&mut census, presence, stage)?;
                Ok((inputs, guide))
            },
            |(inputs, guide), assigned, round| {
                let source = inputs.attach(source.clone())?;
                Ok(crate::layout::MeasuredLayout::new(
                    assigned,
                    FacetPlan::new(
                        &source,
                        PlotKind::Scatter,
                        &theme,
                        round,
                        &mut crate::decorations::plan::children::ChildPlans::facets(&source),
                        guide,
                    )
                    .map_err(crate::layout::fit::PlanError::into_error)?,
                ))
            },
            |_, _, plan, _| Ok(plan),
        )
        .unwrap();
        let plan = root.plan();
        let guide = plan.outside_legend.as_ref().unwrap();
        assert!(plan
            .panels
            .iter()
            .all(|panel| guide.bounds.bottom > panel.panel.y1));
        assert_eq!(plan.horizontal_axis_label.as_deref(), source.x_label_text());
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
        let result = crate::layout::plan_exact_root(
            source.width(),
            source.height(),
            |stage| crate::plots::shared_x::prepare_plot(&source, &stage.generation_resources()),
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

    #[test]
    fn final_facet_legend_and_shared_axis_origin_invalidate_observations() {
        let source = Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0, 1.0, 2.0]),
            crate::NumericValues::new("y", [1.0, 2.0, 2.0, 3.0]),
        )
        .hue(crate::CategoryValues::new("hue", ["a", "b", "a", "b"]).unwrap())
        .facet(crate::FacetSpec::column(
            crate::CategoryValues::new("facet", ["first", "first", "second", "second"]).unwrap(),
        ))
        .legend(crate::LegendPolicy::Visible(LegendPosition::OutsideRight))
        .size(960, 640)
        .erase();
        let theme = crate::Theme::Publication.resolve();
        theme.prepare_font_environment().unwrap();
        let mut arranged = None;
        crate::layout::plan_exact_root(
            source.width(),
            source.height(),
            |stage| {
                let inputs =
                    crate::plots::shared_x::prepare_plot(&source, &stage.generation_resources())?;
                let guide = prepare_discrete_guide(
                    &source,
                    PlotKind::Scatter,
                    &theme,
                    &inputs,
                    &stage.generation_resources(),
                )?;
                Ok((inputs, guide))
            },
            |(inputs, guide), assigned, round| {
                let source = inputs.attach(source.clone())?;
                Ok(crate::layout::MeasuredLayout::new(
                    assigned,
                    FacetPlan::new(
                        &source,
                        PlotKind::Scatter,
                        &theme,
                        round,
                        &mut crate::decorations::plan::children::ChildPlans::facets(&source),
                        guide,
                    )
                    .map_err(crate::layout::fit::PlanError::into_error)?,
                ))
            },
            |_, _, plan, _| {
                arranged = Some(plan);
                Ok(())
            },
        )
        .unwrap();
        let mut plan = arranged.unwrap();
        assert!(plan.outside_legend.is_some());
        for change_legend in [true, false] {
            let mut calls = 0;
            crate::layout::plan_exact_root(
                source.width(),
                source.height(),
                |_| Ok(()),
                |(), assigned, round| {
                    calls += 1;
                    let owner = crate::layout::ComponentAddress::root();
                    round.request_insets(
                        &owner,
                        &owner,
                        crate::layout::kernel::FeedbackKind::OwnedInk,
                        [1.0; 4],
                    )?;
                    if calls == 2 {
                        if change_legend {
                            let legend = plan.outside_legend.as_mut().unwrap();
                            legend.bounds = legend.bounds.translated(1.0, 0.0);
                        } else {
                            plan.shared_axis_origin.0 += 1.0;
                        }
                    }
                    plan.observe_arranged(&theme, round)?;
                    Ok(crate::layout::MeasuredLayout::new(assigned, ()))
                },
                |(), _, (), _| Ok(()),
            )
            .unwrap();
            assert_eq!(
                calls, 2,
                "current post-install parent state is validated without a duplicate round"
            );
        }
    }
}
