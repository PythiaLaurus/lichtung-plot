//! Canvas measurement, panel placement, and final panel planning.

use crate::Error;

use super::data::FacetPanel;
use super::rendering::facet_label;
use crate::builder::Plot;
use crate::decorations::plan::children::MeasurementPhase;
use crate::decorations::plan::{LegendRole, PanelPlan};
use crate::layout::canvas::PanelRequirements;
use crate::layout::kernel::{FeedbackKind, FeedbackRound, FitKind, FrameRequirements};
use crate::layout::{AddressRole, ComponentAddress};
use crate::layout::{CanvasLayout, CanvasLayoutSpec, PanelBox, PhysicalTextAlignment};
use crate::spec::{DecorationMode, FacetScales, PlotKind, PlotRenderContext};
use crate::theme::ResolvedTheme;

pub(super) fn canvas_layout(
    plot: &Plot,
    theme: &ResolvedTheme,
    horizontal_axis_label: Option<&str>,
    vertical_axis_label: Option<&str>,
    round: &mut FeedbackRound,
) -> Result<CanvasLayout, crate::layout::fit::PlanError> {
    let mut canvas = CanvasLayout::measured_plot_root(
        plot.width(),
        plot.height(),
        theme,
        CanvasLayoutSpec {
            has_title: plot.title_text().is_some_and(|value| !value.is_empty()),
            has_subtitle: plot.subtitle_text().is_some_and(|value| !value.is_empty()),
            has_caption: plot.caption_text().is_some_and(|value| !value.is_empty()),
            has_x_label: horizontal_axis_label.is_some(),
            has_y_label: vertical_axis_label.is_some(),
        },
        (plot.title_text(), plot.subtitle_text(), plot.caption_text()),
        crate::decorations::title_block::plan::RootAlignments::resolve(
            theme,
            [plot.title_alignment, plot.subtitle_alignment],
            [None; 2],
        ),
        false,
    )?;
    let baseline = (
        usize::from(horizontal_axis_label.is_some()),
        usize::from(vertical_axis_label.is_some()),
    );
    let (panel, extra_left, extra_bottom) = adjusted_shared_axis_panel(
        horizontal_axis_label,
        vertical_axis_label,
        theme,
        baseline,
        canvas.panel_box,
        round,
        &ComponentAddress::root().child(AddressRole::RootBody, 0)?,
    )?;
    canvas.panel_box = panel;
    canvas.insets.left += extra_left;
    canvas.insets.bottom += extra_bottom;
    Ok(canvas)
}

pub(super) fn reserve_rewrapped_shared_axis_titles(
    horizontal_axis_label: Option<&str>,
    vertical_axis_label: Option<&str>,
    theme: &ResolvedTheme,
    reserved_panel: PanelBox,
    candidate_panel: PanelBox,
    round: &mut FeedbackRound,
) -> Result<PanelBox, crate::layout::fit::PlanError> {
    let baseline = shared_axis_line_counts(
        horizontal_axis_label,
        vertical_axis_label,
        theme,
        reserved_panel,
    )?;
    adjusted_shared_axis_panel(
        horizontal_axis_label,
        vertical_axis_label,
        theme,
        baseline,
        candidate_panel,
        round,
        &ComponentAddress::root().child(AddressRole::FacetGrid, 0)?,
    )
    .map(|(panel, _, _)| panel)
}

pub(super) fn shared_axis_line_counts(
    horizontal_axis_label: Option<&str>,
    vertical_axis_label: Option<&str>,
    theme: &ResolvedTheme,
    panel: PanelBox,
) -> Result<(usize, usize), crate::layout::fit::PlanError> {
    let mut lines = [0; 2];
    for (index, (label, span, role)) in [
        (
            horizontal_axis_label,
            panel.width(),
            AddressRole::AxisBottom,
        ),
        (vertical_axis_label, panel.height(), AddressRole::AxisLeft),
    ]
    .into_iter()
    .enumerate()
    {
        if let Some(label) = label {
            lines[index] = crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                label,
                &theme.typography.axis_title,
                span,
                PhysicalTextAlignment::Center,
                "shared facet axis title",
                &crate::layout::fit::FitScope::root().child(role, 0)?,
            ))?
            .lines
            .len();
        }
    }
    Ok((lines[0], lines[1]))
}

#[expect(
    clippy::cast_precision_loss,
    reason = "wrapped line counts are bounded by finite u32 canvas dimensions"
)]
fn adjusted_shared_axis_panel(
    horizontal_axis_label: Option<&str>,
    vertical_axis_label: Option<&str>,
    theme: &ResolvedTheme,
    baseline: (usize, usize),
    candidate: PanelBox,
    round: &mut FeedbackRound,
    address: &ComponentAddress,
) -> Result<(PanelBox, f64, f64), crate::layout::fit::PlanError> {
    let line_height = theme.typography.axis_title.line_height();
    let consumed = round.insets(address, FeedbackKind::AxisGutter);
    let panel = crate::layout::fit::measured(
        FrameRequirements {
            minimum_body: (80.0, 60.0),
            natural_insets: [0.0; 4],
        }
        .assign_fit(
            candidate,
            consumed,
            &crate::layout::fit::FitScope::new(address, 0),
        ),
    )?;
    let lines = shared_axis_line_counts(horizontal_axis_label, vertical_axis_label, theme, panel)?;
    let extra_bottom = lines.0.saturating_sub(baseline.0) as f64 * line_height;
    let extra_left = lines.1.saturating_sub(baseline.1) as f64 * line_height;
    if candidate.width() - extra_left < 80.0 || candidate.height() - extra_bottom < 60.0 {
        return Err(crate::layout::fit::FitScope::new(address, 0).failure(FitKind::TrackOrResidual,
                "multiline shared facet axis titles do not fit while preserving the minimum 80x60 plot region; increase chart size"
            ).into());
    }
    round.begin_owner(address)?;
    round.observe_box(panel)?;
    if let Some(request) = round.request_insets(
        address,
        address,
        FeedbackKind::AxisGutter,
        [extra_left, 0.0, extra_bottom, 0.0],
    )? {
        round.miss(
            address,
            FitKind::Tick,
            &[Some(request)],
            "shared facet axis title requires a changed gutter",
        )?;
    }
    Ok((panel, consumed[0], consumed[2]))
}

#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "facet measurement keeps immutable grid, scale, theme, and child-plan inputs explicit"
)]
pub(super) fn legacy_parent_adapter(
    plot: &Plot,
    kind: PlotKind,
    panels: &mut [FacetPanel],
    outer: PanelBox,
    shape: (usize, usize),
    facet_scales: FacetScales,
    theme: &ResolvedTheme,
    round: &mut FeedbackRound,
    probes: &[std::sync::Arc<crate::spec::PlotRenderResult>],
    probe_cells: &[PanelBox],
    containing_trial: bool,
    scope: &crate::layout::fit::FitScope,
    children: &mut crate::decorations::plan::children::ChildPlans,
) -> Result<crate::layout::fit::FitOutcome<()>, Error> {
    use crate::layout::fit::{self, FitOutcome};
    let (rows, columns) = shape;
    let grid = ComponentAddress::root().child(AddressRole::FacetGrid, 0)?;
    let mut requirements = Vec::new();
    plot.data()
        .reserve_source(&mut requirements, panels.len())?;
    for (index, facet) in panels.iter().enumerate() {
        let address = grid.child(
            AddressRole::FacetCell,
            u32::try_from(index).map_err(|_| Error::Render("facet address overflowed".into()))?,
        )?;
        let [left, right, bottom, top] = round.insets(&address, FeedbackKind::AxisGutter);
        requirements.push((
            facet.row,
            facet.column,
            PanelRequirements {
                left,
                right,
                bottom,
                top,
            },
        ));
    }
    let cells = match allocate_legacy_grid(
        plot.data(),
        outer,
        rows,
        columns,
        &requirements,
        &scope.child(AddressRole::FacetGrid, 0)?,
    )? {
        FitOutcome::Measured(cells) => cells,
        FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
    };
    let mut child_miss = None;
    for (index, facet) in panels.iter().enumerate() {
        let mut panel_plot = plot
            .clone()
            .with_render_scope(facet.render_scope(plot.data())?)
            .with_significance_subset(facet.significance_selectors());
        let domain_address = ComponentAddress::root().child(
            AddressRole::FacetCell,
            u32::try_from(index).map_err(|_| Error::Render("facet address overflowed".into()))?,
        )?;
        if matches!(facet_scales, FacetScales::Free | FacetScales::FreeX) {
            if let Some(range) = round.domain(&domain_address, FeedbackKind::AutomaticDomainX) {
                panel_plot = panel_plot.with_planned_x_limits(range);
            }
        }
        if matches!(facet_scales, FacetScales::Free | FacetScales::FreeY) {
            if let Some(range) = round.domain(&domain_address, FeedbackKind::AutomaticDomainY) {
                panel_plot = panel_plot.with_planned_y_limits(range);
            }
        }
        let cell = cells[facet.row * columns + facet.column];
        if containing_trial {
            if let FitOutcome::Miss(reason) = crate::decorations::plan::check_assigned_body(
                &panel_plot,
                &probes[index],
                probe_cells[facet.row * columns + facet.column],
                cell,
                theme,
                &scope.child(AddressRole::FacetCell, index)?,
            )? {
                fit::FitFailure::retain_least(&mut child_miss, reason);
            }
            let label = facet_label(&panel_plot, &facet.cell)?;
            let appearance = theme
                .legacy_text_appearance(theme.facet_header_font_size, crate::TextWeight::Normal);
            if let FitOutcome::Miss(reason) = fit::text(
                theme,
                &label,
                &appearance,
                (cell.width() - 16.0).max(0.0),
                PhysicalTextAlignment::Center,
                "facet strip",
                &scope
                    .child(AddressRole::FacetCell, index)?
                    .child(AddressRole::FacetHeader, 0)?,
            )? {
                fit::FitFailure::retain_least(&mut child_miss, reason);
            }
        }
        let cell_scope = scope.child(AddressRole::FacetCell, index)?;
        let context = PlotRenderContext::new(
            crate::data::PlotData::with_rows(&panel_plot, &facet.rows, &facet.source_rows),
            cell,
            theme,
        )
        .with_facet_scales(facet_scales)
        .with_fit_scope(&cell_scope);
        let result = match children.measure(
            &panel_plot,
            context,
            &domain_address,
            MeasurementPhase::Layout,
            round,
            || {
                fit::outcome(
                    panel_plot.measure_panel(
                        kind,
                        PlotRenderContext::new(
                            crate::data::PlotData::with_rows(
                                &panel_plot,
                                &facet.rows,
                                &facet.source_rows,
                            ),
                            cells[facet.row * columns + facet.column],
                            theme,
                        )
                        .with_facet_scales(facet_scales)
                        .with_fit_scope(&cell_scope),
                    ),
                )
            },
        )? {
            FitOutcome::Measured(result) => result,
            FitOutcome::Miss(failure) => {
                fit::FitFailure::retain_least(&mut child_miss, failure);
                continue;
            }
        };
        let label = facet_label(&panel_plot, &facet.cell)?;
        let cell = cells[facet.row * columns + facet.column];
        let requirement = match fit::outcome(PanelRequirements::measure(
            result
                .axes
                .as_ref()
                .ok_or_else(|| Error::Render("facet panel is missing its measured axes".into()))?,
            theme,
            Some(&label),
            cell,
            &cell_scope,
        ))? {
            FitOutcome::Measured(requirement) => requirement,
            FitOutcome::Miss(failure) => {
                fit::FitFailure::retain_least(&mut child_miss, failure);
                continue;
            }
        };
        let address = grid.child(
            AddressRole::FacetCell,
            u32::try_from(index).map_err(|_| Error::Render("facet address overflowed".into()))?,
        )?;
        round.begin_owner(&address)?;
        round.observe_box(cell)?;
        round.observe_layout(&result.layout)?;
        if let Some(request) = round.request_insets(
            &address,
            &address,
            FeedbackKind::AxisGutter,
            [
                requirement.left,
                requirement.right,
                requirement.bottom,
                requirement.top,
            ],
        )? {
            round.miss(
                &address,
                FitKind::Tick,
                &[Some(request)],
                "facet cell requires measured decoration gutters",
            )?;
        }
    }
    if let Some(failure) = child_miss {
        return Ok(FitOutcome::Miss(failure));
    }
    for facet in panels {
        facet.panel = cells[facet.row * columns + facet.column];
    }
    Ok(FitOutcome::Measured(()))
}

/// Admission boundary for the unchanged equal-panel allocator, not measured tracks.
pub(super) fn allocate_legacy_grid(
    data: crate::data::PlotData<'_>,
    outer: PanelBox,
    rows: usize,
    columns: usize,
    requirements: &[(usize, usize, PanelRequirements)],
    scope: &crate::layout::fit::FitScope,
) -> Result<crate::layout::fit::FitOutcome<Vec<PanelBox>>, Error> {
    let cells = rows
        .checked_mul(columns)
        .ok_or_else(|| Error::Render("facet topology overflow".into()))?;
    let tracks = rows
        .checked_add(columns)
        .and_then(|count| count.checked_mul(3))
        .ok_or_else(|| Error::Render("facet track storage overflow".into()))?;
    let bytes = cells
        .checked_mul(std::mem::size_of::<PanelBox>())
        .and_then(|bytes| {
            tracks
                .checked_mul(std::mem::size_of::<f64>())
                .and_then(|extra| bytes.checked_add(extra))
        })
        .ok_or_else(|| Error::Render("facet allocation storage overflow".into()))?;
    data.claim_source_visits(cells)?;
    data.claim_source_visits(tracks)?;
    data.claim_source_bytes(bytes)?;
    crate::layout::canvas::allocate_composed_grid_fit(
        outer,
        rows,
        columns,
        requirements,
        "facet layout",
        scope,
    )
}

pub(super) fn feedback_plot(
    mut plot: Plot,
    panels: &[FacetPanel],
    facet_scales: FacetScales,
    round: &FeedbackRound,
) -> Result<Plot, Error> {
    let fix_x = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeY);
    let fix_y = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeX);
    for (fixed, kind) in [
        (fix_x, FeedbackKind::AutomaticDomainX),
        (fix_y, FeedbackKind::AutomaticDomainY),
    ] {
        if !fixed {
            continue;
        }
        let mut shared: Option<(f64, f64)> = None;
        for index in 0..panels.len() {
            let address = ComponentAddress::root().child(
                AddressRole::FacetCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("facet address overflowed".into()))?,
            )?;
            if let Some(range) = round.domain(&address, kind) {
                shared = Some(
                    shared.map_or(range, |prior| (prior.0.min(range.0), prior.1.max(range.1))),
                );
            }
        }
        if let Some(range) = shared {
            plot = if kind == FeedbackKind::AutomaticDomainX {
                plot.with_planned_x_limits(range)
            } else {
                plot.with_planned_y_limits(range)
            };
        }
    }
    Ok(plot)
}

#[expect(
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "one facet round carries shared-domain and parent feedback explicitly"
)]
pub(super) fn plan_panels(
    plot: Plot,
    kind: PlotKind,
    panels: &[FacetPanel],
    outside_legend: bool,
    legend_entries: &std::sync::Arc<[crate::decorations::legend::LegendEntry]>,
    facet_scales: FacetScales,
    theme: &ResolvedTheme,
    round: &mut FeedbackRound,
    children: &mut crate::decorations::plan::children::ChildPlans,
) -> Result<crate::layout::fit::FitOutcome<(Plot, Vec<std::sync::Arc<PanelPlan>>)>, Error> {
    let mut panel_plans = Vec::new();
    plot.data().reserve_source(&mut panel_plans, panels.len())?;
    let mut legend_owner = !outside_legend;
    let mut child_miss = None;
    for (index, facet) in panels.iter().enumerate() {
        let panel_plot = plot
            .clone()
            .with_render_scope(facet.render_scope(plot.data())?)
            .with_significance_subset(facet.significance_selectors());
        let role = if legend_owner {
            LegendRole::Panel
        } else {
            LegendRole::Suppressed
        };
        let context = PlotRenderContext::new(
            crate::data::PlotData::with_rows(&panel_plot, &facet.rows, &facet.source_rows),
            facet.panel,
            theme,
        )
        .with_facet_scales(facet_scales);
        let address = ComponentAddress::root().child(
            AddressRole::FacetCell,
            u32::try_from(index).map_err(|_| Error::Render("facet address overflowed".into()))?,
        )?;
        let outcome = children.plan(
            &panel_plot,
            role,
            context,
            (role == LegendRole::Panel).then_some(legend_entries),
            round,
            &address,
        )?;
        let plan = match outcome {
            crate::layout::fit::FitOutcome::Measured(plan) => plan,
            crate::layout::fit::FitOutcome::Miss(failure) => {
                crate::layout::fit::FitFailure::retain_least(&mut child_miss, failure);
                continue;
            }
        };
        if plan.result.decoration_mode == DecorationMode::SelfManaged {
            return Err(Error::Render(format!(
                "{kind:?} cannot be nested in generic faceting"
            )));
        }
        if plan.legend.is_some() {
            legend_owner = false;
        }
        panel_plans.push(plan);
    }
    if let Some(failure) = child_miss {
        return Ok(crate::layout::fit::FitOutcome::Miss(failure));
    }
    Ok(crate::layout::fit::FitOutcome::Measured((
        plot,
        panel_plans,
    )))
}
