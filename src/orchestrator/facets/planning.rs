//! Canvas measurement, panel placement, and final panel planning.

use crate::Error;

use super::data::FacetPanel;
use super::rendering::facet_label;
use crate::builder::Plot;
use crate::decorations::plan::{LegendRole, PanelPlan, ResolvedLegend};
use crate::layout::canvas::{allocate_composed_grid, PanelRequirements};
use crate::layout::{CanvasLayout, CanvasLayoutSpec, PanelBox, TextBlock};
use crate::render::backend::Backend;
use crate::spec::{DecorationMode, FacetScales, PlotKind, PlotRenderContext};
use crate::theme::ResolvedTheme;

pub(super) fn canvas_layout(
    chart: &Plot,
    palette: &ResolvedTheme,
    horizontal_axis_label: Option<&str>,
    vertical_axis_label: Option<&str>,
) -> Result<CanvasLayout, Error> {
    let mut canvas = CanvasLayout::measured_composed_canvas(
        chart.width(),
        chart.height(),
        palette,
        CanvasLayoutSpec {
            has_title: palette.show_title
                && chart.title_text().is_some_and(|value| !value.is_empty()),
            has_subtitle: palette.show_subtitle
                && chart.subtitle_text().is_some_and(|value| !value.is_empty()),
            has_caption: palette.show_caption
                && chart.caption_text().is_some_and(|value| !value.is_empty()),
            has_x_label: horizontal_axis_label.is_some(),
            has_y_label: vertical_axis_label.is_some(),
        },
        (
            chart.title_text(),
            chart.subtitle_text(),
            chart.caption_text(),
        ),
    )?;
    let baseline = (
        usize::from(horizontal_axis_label.is_some()),
        usize::from(vertical_axis_label.is_some()),
    );
    let (panel, extra_left, extra_bottom) = adjusted_shared_axis_panel(
        horizontal_axis_label,
        vertical_axis_label,
        palette,
        baseline,
        canvas.panel_box,
    )?;
    canvas.panel_box = panel;
    canvas.insets.left += extra_left;
    canvas.insets.bottom += extra_bottom;
    Ok(canvas)
}

pub(super) fn reserve_rewrapped_shared_axis_titles(
    horizontal_axis_label: Option<&str>,
    vertical_axis_label: Option<&str>,
    palette: &ResolvedTheme,
    reserved_panel: PanelBox,
    candidate_panel: PanelBox,
) -> Result<PanelBox, Error> {
    let baseline = shared_axis_line_counts(
        horizontal_axis_label,
        vertical_axis_label,
        palette,
        reserved_panel,
    )?;
    adjusted_shared_axis_panel(
        horizontal_axis_label,
        vertical_axis_label,
        palette,
        baseline,
        candidate_panel,
    )
    .map(|(panel, _, _)| panel)
}

fn shared_axis_line_counts(
    horizontal_axis_label: Option<&str>,
    vertical_axis_label: Option<&str>,
    palette: &ResolvedTheme,
    panel: PanelBox,
) -> Result<(usize, usize), Error> {
    let horizontal_lines = horizontal_axis_label
        .map(|label| {
            TextBlock::layout(
                label,
                palette.label_font_size,
                panel.width().max(1.0),
                "shared facet x-axis title",
            )
            .map(|block| block.lines().len())
        })
        .transpose()?
        .unwrap_or(0);
    let vertical_lines = vertical_axis_label
        .map(|label| {
            TextBlock::layout(
                label,
                palette.label_font_size,
                panel.height().max(1.0),
                "shared facet y-axis title",
            )
            .map(|block| block.lines().len())
        })
        .transpose()?
        .unwrap_or(0);
    Ok((horizontal_lines, vertical_lines))
}

#[expect(
    clippy::cast_precision_loss,
    reason = "wrapped line counts are bounded by finite u32 canvas dimensions"
)]
fn adjusted_shared_axis_panel(
    horizontal_axis_label: Option<&str>,
    vertical_axis_label: Option<&str>,
    palette: &ResolvedTheme,
    baseline: (usize, usize),
    candidate: PanelBox,
) -> Result<(PanelBox, f64, f64), Error> {
    let line_height = f64::from(palette.label_font_size) * 1.2;
    let mut panel = candidate;
    for _ in 0..4 {
        let lines =
            shared_axis_line_counts(horizontal_axis_label, vertical_axis_label, palette, panel)?;
        let extra_bottom = lines.0.saturating_sub(baseline.0) as f64 * line_height;
        let extra_left = lines.1.saturating_sub(baseline.1) as f64 * line_height;
        if candidate.width() - extra_left < 80.0 || candidate.height() - extra_bottom < 60.0 {
            return Err(Error::Render(
                "multiline shared facet axis titles do not fit while preserving the minimum 80x60 plot region; increase chart size"
                    .into(),
            ));
        }
        let next = PanelBox::new(
            candidate.x0 + extra_left,
            candidate.x1,
            candidate.y0 + extra_bottom,
            candidate.y1,
        );
        if next == panel {
            return Ok((next, extra_left, extra_bottom));
        }
        panel = next;
    }
    Err(Error::Render(
        "multiline shared facet axis-title layout did not stabilize; increase chart size or shorten labels"
            .into(),
    ))
}

#[expect(
    clippy::too_many_arguments,
    reason = "facet measurement keeps the immutable grid, scale, theme, and backend inputs explicit"
)]
pub(super) fn layout_panels(
    chart: &Plot,
    kind: PlotKind,
    panels: &mut [FacetPanel],
    outer: PanelBox,
    shape: (usize, usize),
    facet_scales: FacetScales,
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let (rows, columns) = shape;
    let provisional = allocate_composed_grid(outer, rows, columns, &[], "facet layout")?;
    let mut measured = Vec::with_capacity(panels.len());
    for facet in panels.iter() {
        let panel_chart = chart
            .clone()
            .with_render_scope(facet.render_scope())
            .with_significance_subset(facet.significance_selectors());
        let result = panel_chart.render_panel(
            kind,
            PlotRenderContext::new(
                crate::data::PlotData::with_rows(&panel_chart, &facet.rows, &facet.source_rows),
                provisional[facet.row * columns + facet.column],
                theme,
            )
            .with_facet_scales(facet_scales),
            backend,
        )?;
        let label = facet_label(&panel_chart, &facet.cell);
        let cell = provisional[facet.row * columns + facet.column];
        let requirement = PanelRequirements::measure(&result.layout, theme, Some(&label), cell)?;
        measured.push((facet.row, facet.column, result, label, requirement));
    }
    let requirements = measured
        .iter()
        .map(|(row, column, _, _, requirement)| (*row, *column, *requirement))
        .collect::<Vec<_>>();
    let first_cells = allocate_composed_grid(outer, rows, columns, &requirements, "facet layout")?;
    let requirements = measured
        .iter()
        .map(|(row, column, result, label, _)| {
            Ok::<_, Error>((
                *row,
                *column,
                PanelRequirements::measure(
                    &result.layout,
                    theme,
                    Some(label),
                    first_cells[*row * columns + *column],
                )?,
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let cells = allocate_composed_grid(outer, rows, columns, &requirements, "facet layout")?;
    for facet in panels {
        facet.panel = cells[facet.row * columns + facet.column];
    }
    Ok(())
}

pub(super) fn plan_panels(
    mut chart: Plot,
    kind: PlotKind,
    panels: &[FacetPanel],
    outside_legend: Option<ResolvedLegend>,
    legend_entries: &[crate::decorations::legend::LegendEntry],
    facet_scales: FacetScales,
    theme: &ResolvedTheme,
) -> Result<(Plot, Vec<PanelPlan>), Error> {
    let fix_x = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeY);
    let fix_y = matches!(facet_scales, FacetScales::Fixed | FacetScales::FreeX);
    let mut final_plans = Vec::new();
    for _ in 0..2 {
        final_plans = Vec::with_capacity(panels.len());
        let mut legend_owner = outside_legend.is_none();
        for facet in panels {
            let panel_chart = chart
                .clone()
                .with_render_scope(facet.render_scope())
                .with_significance_subset(facet.significance_selectors());
            let role = if legend_owner {
                LegendRole::Panel
            } else {
                LegendRole::Suppressed
            };
            let context = PlotRenderContext::new(
                crate::data::PlotData::with_rows(&panel_chart, &facet.rows, &facet.source_rows),
                facet.panel,
                theme,
            )
            .with_facet_scales(facet_scales);
            let plan = crate::decorations::plan::plan_panel_with_context(
                &panel_chart,
                kind,
                role,
                context,
                (role == LegendRole::Panel).then_some(legend_entries),
            )?;
            if plan.result.decoration_mode == DecorationMode::SelfManaged {
                return Err(Error::Render(format!(
                    "{kind:?} cannot be nested in generic faceting"
                )));
            }
            if plan.legend.is_some() {
                legend_owner = false;
            }
            final_plans.push(plan);
        }
        let planned_x = fix_x
            .then(|| final_plans.iter().find_map(PanelPlan::planned_x_limits))
            .flatten();
        let planned_y = fix_y
            .then(|| final_plans.iter().find_map(PanelPlan::planned_y_limits))
            .flatten();
        if planned_x.is_none() && planned_y.is_none() {
            break;
        }
        if let Some(limits) = planned_x {
            chart = chart.with_planned_x_limits(limits);
        }
        if let Some(limits) = planned_y {
            chart = chart.with_planned_y_limits(limits);
        }
    }
    if let (Some(legend), Some(plan)) = (outside_legend, final_plans.first_mut()) {
        plan.legend = Some(legend);
    }
    Ok((chart, final_plans))
}
