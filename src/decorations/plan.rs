//! Immutable panel and legend planning shared by every renderer.

use crate::Error;

use crate::decorations::legend::{Legend, LegendBox};
use crate::layout::PanelBox;
use crate::render::geometry::GeometryBackend;
use crate::spec::{
    LegendClearance, LegendDefault, LegendPosition, PlotKind, PlotRenderContext, PlotRenderResult,
};
use crate::Chart;

// Data-space expansion normally clears a legend in one pass. The larger bound
// permits coupled axis/decorations to settle while making non-convergence fatal.
const MAX_LEGEND_CLEARANCE_PASSES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LegendRole {
    Panel,
    Suppressed,
}

pub(crate) struct ResolvedLegend {
    pub(crate) legend: Legend,
    pub(crate) bounds: LegendBox,
}

pub(crate) struct PanelPlan {
    pub(crate) panel: PanelBox,
    pub(crate) result: PlotRenderResult,
    pub(crate) legend: Option<ResolvedLegend>,
    planned_x_limits: Option<(f64, f64)>,
    planned_y_limits: Option<(f64, f64)>,
}

pub(crate) fn plan_outside_legend(
    chart: &Chart<'_>,
    entries: &[crate::decorations::legend::LegendEntry],
    cell: PanelBox,
) -> Result<(PanelBox, ResolvedLegend), Error> {
    let legend = Legend::new(
        chart.resolved_legend_title(),
        entries.to_vec(),
        f64::from(chart.width()),
        f64::from(chart.height()),
    );
    let panel = reserve_outside_panel(cell, &legend, chart.legend_position_spec())?;
    let bounds = resolve_box(
        &legend,
        chart.legend_position_spec(),
        LegendDefault::LowestOverlap,
        panel,
        panel,
        cell,
        &[],
    );
    Ok((panel, ResolvedLegend { legend, bounds }))
}

pub(crate) const fn is_outside(position: LegendPosition) -> bool {
    matches!(
        position,
        LegendPosition::OutsideRightTop
            | LegendPosition::OutsideRight
            | LegendPosition::OutsideRightBottom
            | LegendPosition::OutsideLeftTop
            | LegendPosition::OutsideLeft
            | LegendPosition::OutsideLeftBottom
            | LegendPosition::OutsideTopLeft
            | LegendPosition::OutsideTop
            | LegendPosition::OutsideTopRight
            | LegendPosition::OutsideBottomLeft
            | LegendPosition::OutsideBottom
            | LegendPosition::OutsideBottomRight
    )
}

const fn is_inside(position: LegendPosition) -> bool {
    !matches!(position, LegendPosition::Hidden) && !is_outside(position)
}

impl PanelPlan {
    pub(crate) fn chart_for_render<'a>(&self, chart: &Chart<'a>) -> Chart<'a> {
        let chart = self.planned_x_limits.map_or_else(
            || chart.clone(),
            |limits| chart.clone().with_planned_x_limits(limits),
        );
        self.planned_y_limits
            .map_or(chart.clone(), |limits| chart.with_planned_y_limits(limits))
    }

    pub(crate) const fn planned_x_limits(&self) -> Option<(f64, f64)> {
        self.planned_x_limits
    }

    pub(crate) const fn planned_y_limits(&self) -> Option<(f64, f64)> {
        self.planned_y_limits
    }
}

pub(crate) fn plan_panel(
    chart: &Chart<'_>,
    kind: PlotKind,
    cell: PanelBox,
    role: LegendRole,
) -> Result<PanelPlan, Error> {
    let source_rows = chart.configured_source_rows()?;
    plan_panel_with_data(
        chart,
        kind,
        cell,
        role,
        chart.data(),
        source_rows.as_deref(),
    )
}

#[expect(
    clippy::too_many_lines,
    clippy::similar_names,
    reason = "the two-pass panel lifecycle keeps discovery, reservation, collision resolution, and frozen output together"
)]
pub(crate) fn plan_panel_with_data(
    chart: &Chart<'_>,
    kind: PlotKind,
    cell: PanelBox,
    role: LegendRole,
    data: &polars::prelude::DataFrame,
    source_rows: Option<&[usize]>,
) -> Result<PanelPlan, Error> {
    let legend_default = kind.legend_default(chart.user_y_limits().is_some());
    let mut planning_chart = match role {
        LegendRole::Panel => chart.clone(),
        LegendRole::Suppressed => chart.clone().with_suppressed_legend(),
    };
    let mut discovery = GeometryBackend::default();
    let discovered = render(
        &planning_chart,
        kind,
        cell,
        data,
        source_rows,
        &mut discovery,
    )?;
    let visible = role == LegendRole::Panel
        && planning_chart.legend_visibility() != Some(false)
        && planning_chart.legend_position_spec() != LegendPosition::Hidden
        && !discovered.legend_entries.is_empty();
    let legend = visible.then(|| {
        Legend::new(
            planning_chart.resolved_legend_title(),
            discovered.legend_entries.clone(),
            f64::from(planning_chart.width()),
            f64::from(planning_chart.height()),
        )
    });
    let panel = legend.as_ref().map_or(Ok(cell), |legend| {
        reserve_outside_panel(cell, legend, planning_chart.legend_position_spec())
    })?;
    let panel =
        planning_chart.reserve_rewrapped_axis_titles(kind, cell, panel, &discovered.layout)?;

    let mut geometry = GeometryBackend::default();
    let mut result = if panel == cell {
        discovered
    } else {
        render(
            &planning_chart,
            kind,
            panel,
            data,
            source_rows,
            &mut geometry,
        )?
    };
    if panel == cell {
        geometry = discovery;
    }
    record_brackets(&planning_chart, &result, &mut geometry)?;
    let mut planned_x_limits = None;
    let mut planned_y_limits = None;
    let position = planning_chart.legend_position_spec();
    let mut auto_candidate_index = None;
    let mut resolved = legend
        .map(|legend| {
            let bounds = if position == LegendPosition::Auto {
                let preferred = initial_auto_candidate(
                    &legend,
                    result.layout.projection.panel_box(),
                    geometry.bounds(),
                    legend_default,
                );
                let index = auto_candidate_avoiding_panel_text(
                    &planning_chart,
                    &result,
                    &legend,
                    preferred,
                )?;
                auto_candidate_index = Some(index);
                legend.inside_candidates(result.layout.projection.pixel_box())[index]
            } else {
                resolve_box(
                    &legend,
                    position,
                    legend_default,
                    result.layout.projection.panel_box(),
                    panel,
                    cell,
                    geometry.bounds(),
                )
            };
            Ok::<_, Error>(ResolvedLegend { legend, bounds })
        })
        .transpose()?;
    if is_inside(position) {
        if let Some(legend) = &resolved {
            if let Some(corner) = crate::decorations::panel_text::conflicting_legend_corner(
                &planning_chart,
                &result.layout,
                legend.bounds,
            )? {
                return Err(Error::Render(format!(
                    "panel text annotation at {corner:?} is incompatible with the resolved inside legend; hide the legend or choose an outside legend position"
                )));
            }
        }
    }
    for _ in 0..MAX_LEGEND_CLEARANCE_PASSES {
        let Some(legend) = &resolved else { break };
        let collision = overlap_score_within_panel(
            legend.bounds,
            geometry.bounds(),
            legend.legend.metrics.font_size.max(6.0),
            result.layout.projection.panel_box(),
        );
        if collision <= 0.0 {
            break;
        }
        let mut replacement_bounds = None;
        let expansion = if position == LegendPosition::Auto {
            let Some(clearance) = automatic_clearance(
                kind,
                chart,
                &result,
                &legend.legend,
                geometry.bounds(),
                legend_default,
                auto_candidate_index.unwrap_or(0),
            ) else {
                break;
            };
            auto_candidate_index = Some(clearance.candidate_index);
            replacement_bounds = Some(
                legend
                    .legend
                    .inside_candidates(result.layout.projection.pixel_box())
                    [clearance.candidate_index],
            );
            clearance.expansion
        } else {
            let preferred_edge = clearance_edge(
                planning_chart.legend_position_spec(),
                legend.bounds,
                result.layout.projection.panel_box(),
            );
            explicit_clearance(
                kind,
                chart,
                &result,
                &legend.legend,
                legend.bounds,
                preferred_edge,
            )
        };
        let changed_candidate = replacement_bounds.is_some();
        if let Some(bounds) = replacement_bounds {
            resolved.as_mut().expect("resolved legend exists").bounds = bounds;
        }
        let Some((axis, limits)) = expansion else {
            if changed_candidate {
                continue;
            }
            break;
        };
        match axis {
            ClearanceAxis::X => {
                planning_chart = planning_chart.with_planned_x_limits(limits);
                planned_x_limits = Some(limits);
            }
            ClearanceAxis::Y => {
                planning_chart = planning_chart.with_planned_y_limits(limits);
                planned_y_limits = Some(limits);
            }
        }
        geometry = GeometryBackend::default();
        result = render(
            &planning_chart,
            kind,
            panel,
            data,
            source_rows,
            &mut geometry,
        )?;
        record_brackets(&planning_chart, &result, &mut geometry)?;
        let legend = resolved.take().expect("resolved legend exists").legend;
        let bounds = if position == LegendPosition::Auto {
            legend.inside_candidates(result.layout.projection.pixel_box())
                [auto_candidate_index.unwrap_or(0)]
        } else {
            resolve_box(
                &legend,
                position,
                legend_default,
                result.layout.projection.panel_box(),
                panel,
                cell,
                geometry.bounds(),
            )
        };
        resolved = Some(ResolvedLegend { legend, bounds });
    }
    if is_inside(position) {
        if let Some(legend) = &resolved {
            let axes = result.layout.projection.panel_box();
            let margin = legend.legend.metrics.font_size.max(6.0);
            let fits = legend_fits_panel(legend.bounds, axes);
            let overlap =
                overlap_score_within_panel(legend.bounds, geometry.bounds(), margin, axes);
            if !fits || overlap > 0.0 {
                let placement = if position == LegendPosition::Auto {
                    "automatic legend".to_string()
                } else {
                    format!("explicit inside legend {position:?}")
                };
                return Err(Error::Render(format!(
                    "{placement} cannot fit inside the plot without overlapping recorded geometry (in_bounds={fits}, overlap_score={overlap:.3}); increase the canvas, shorten legend labels, choose an outside position, or provide more automatic axis range"
                )));
            }
        }
    }
    Ok(PanelPlan {
        panel,
        result,
        legend: resolved,
        planned_x_limits,
        planned_y_limits,
    })
}

fn record_brackets(
    chart: &Chart<'_>,
    result: &PlotRenderResult,
    geometry: &mut GeometryBackend,
) -> Result<(), Error> {
    if !result.significance_brackets.is_empty() {
        crate::significance::render(chart, &result.significance_brackets, geometry)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClearanceEdge {
    Left,
    Right,
    Bottom,
    Top,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClearanceAxis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy)]
struct AutomaticClearance {
    candidate_index: usize,
    expansion: Option<(ClearanceAxis, (f64, f64))>,
}

fn clearance_edge(position: LegendPosition, legend: LegendBox, panel: PanelBox) -> ClearanceEdge {
    match position {
        LegendPosition::InsideRightTop
        | LegendPosition::InsideRight
        | LegendPosition::InsideRightBottom => ClearanceEdge::Right,
        LegendPosition::InsideLeftTop
        | LegendPosition::InsideLeft
        | LegendPosition::InsideLeftBottom => ClearanceEdge::Left,
        LegendPosition::InsideBottomLeft
        | LegendPosition::InsideBottom
        | LegendPosition::InsideBottomRight => ClearanceEdge::Bottom,
        LegendPosition::Auto
        | LegendPosition::InsideTopLeft
        | LegendPosition::InsideTop
        | LegendPosition::InsideTopRight => ClearanceEdge::Top,
        _ => {
            let distances = [
                (legend.left - panel.x0, ClearanceEdge::Left),
                (panel.x1 - legend.right, ClearanceEdge::Right),
                (legend.bottom - panel.y0, ClearanceEdge::Bottom),
                (panel.y1 - legend.top, ClearanceEdge::Top),
            ];
            distances
                .into_iter()
                .min_by(|left, right| left.0.total_cmp(&right.0))
                .map_or(ClearanceEdge::Top, |(_, edge)| edge)
        }
    }
}

fn automatic_clearance(
    kind: PlotKind,
    chart: &Chart<'_>,
    result: &PlotRenderResult,
    legend: &Legend,
    geometry: &[crate::render::geometry::GeometryBox],
    default: LegendDefault,
    current_candidate: usize,
) -> Option<AutomaticClearance> {
    match default {
        LegendDefault::TopRight(clearance) => top_right_clearance(
            kind,
            chart,
            result,
            legend,
            geometry,
            clearance,
            current_candidate,
        ),
        LegendDefault::LowestOverlap | LegendDefault::SelfManaged => {
            lowest_overlap_clearance(kind, chart, result, legend, geometry)
        }
    }
}

fn top_right_clearance(
    kind: PlotKind,
    chart: &Chart<'_>,
    result: &PlotRenderResult,
    legend: &Legend,
    geometry: &[crate::render::geometry::GeometryBox],
    clearance: LegendClearance,
    current_candidate: usize,
) -> Option<AutomaticClearance> {
    let margin = legend.metrics.font_size.max(6.0);
    let candidates = legend.inside_candidates(result.layout.projection.pixel_box());
    for (candidate_index, bounds) in candidates
        .iter()
        .copied()
        .enumerate()
        .skip(current_candidate)
    {
        if crate::decorations::panel_text::conflicting_legend_corner(chart, &result.layout, bounds)
            .map_or(true, |corner| corner.is_some())
        {
            continue;
        }
        if !legend_fits_panel(bounds, result.layout.projection.panel_box()) {
            continue;
        }
        if overlap_score_within_panel(
            bounds,
            geometry,
            margin,
            result.layout.projection.panel_box(),
        ) <= 0.0
        {
            return Some(AutomaticClearance {
                candidate_index,
                expansion: None,
            });
        }
        for edge in candidate_edges(candidate_index, clearance) {
            if let Some(expansion) = expanded_axis_limits(kind, chart, result, legend, bounds, edge)
            {
                return Some(AutomaticClearance {
                    candidate_index,
                    expansion: Some(expansion),
                });
            }
        }
    }
    None
}

fn candidate_edges(index: usize, clearance: LegendClearance) -> [ClearanceEdge; 2] {
    let horizontal = match index {
        0 | 2 => ClearanceEdge::Right,
        _ => ClearanceEdge::Left,
    };
    let vertical = match index {
        0 | 1 => ClearanceEdge::Top,
        _ => ClearanceEdge::Bottom,
    };
    match clearance {
        LegendClearance::HorizontalMaximum => [horizontal, vertical],
        LegendClearance::VerticalMaximum => [vertical, horizontal],
    }
}

fn lowest_overlap_clearance(
    kind: PlotKind,
    chart: &Chart<'_>,
    result: &PlotRenderResult,
    legend: &Legend,
    geometry: &[crate::render::geometry::GeometryBox],
) -> Option<AutomaticClearance> {
    let margin = legend.metrics.font_size.max(6.0);
    let candidates = legend.inside_candidates(result.layout.projection.pixel_box());
    let edges = [
        [ClearanceEdge::Right, ClearanceEdge::Top],
        [ClearanceEdge::Left, ClearanceEdge::Top],
        [ClearanceEdge::Right, ClearanceEdge::Bottom],
        [ClearanceEdge::Left, ClearanceEdge::Bottom],
    ];

    let mut best = None;
    for (candidate_index, bounds) in candidates.into_iter().enumerate() {
        if !legend_fits_panel(bounds, result.layout.projection.panel_box())
            || crate::decorations::panel_text::conflicting_legend_corner(
                chart,
                &result.layout,
                bounds,
            )
            .map_or(true, |corner| corner.is_some())
        {
            continue;
        }
        for (edge_index, edge) in edges[candidate_index].into_iter().enumerate() {
            let Some((axis, limits)) =
                expanded_axis_limits(kind, chart, result, legend, bounds, edge)
            else {
                continue;
            };
            let range = match axis {
                ClearanceAxis::X => result.layout.projection.x_range(),
                ClearanceAxis::Y => result.layout.projection.y_range(),
            };
            let current_span = range.1 - range.0;
            let candidate = (
                overlap_score_within_panel(
                    bounds,
                    geometry,
                    margin,
                    result.layout.projection.panel_box(),
                ),
                (limits.1 - limits.0) / current_span,
                candidate_index,
                edge_index,
                axis,
                limits,
            );
            if best.as_ref().is_none_or(
                |current: &(f64, f64, usize, usize, ClearanceAxis, (f64, f64))| {
                    candidate
                        .0
                        .total_cmp(&current.0)
                        .then_with(|| candidate.1.total_cmp(&current.1))
                        .then_with(|| candidate.2.cmp(&current.2))
                        .then_with(|| candidate.3.cmp(&current.3))
                        .is_lt()
                },
            ) {
                best = Some(candidate);
            }
        }
    }
    best.map(
        |(_, _, candidate_index, _, axis, limits)| AutomaticClearance {
            candidate_index,
            expansion: Some((axis, limits)),
        },
    )
}

fn explicit_clearance(
    kind: PlotKind,
    chart: &Chart<'_>,
    result: &PlotRenderResult,
    legend: &Legend,
    bounds: LegendBox,
    preferred_edge: ClearanceEdge,
) -> Option<(ClearanceAxis, (f64, f64))> {
    [
        preferred_edge,
        ClearanceEdge::Left,
        ClearanceEdge::Right,
        ClearanceEdge::Bottom,
        ClearanceEdge::Top,
    ]
    .into_iter()
    .enumerate()
    .filter_map(|(index, edge)| {
        let (axis, limits) = expanded_axis_limits(kind, chart, result, legend, bounds, edge)?;
        let range = match axis {
            ClearanceAxis::X => result.layout.projection.x_range(),
            ClearanceAxis::Y => result.layout.projection.y_range(),
        };
        let expansion = (limits.1 - limits.0) / (range.1 - range.0);
        Some((expansion, index, axis, limits))
    })
    .min_by(|left, right| {
        left.0
            .total_cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
    })
    .map(|(_, _, axis, limits)| (axis, limits))
}

fn legend_fits_panel(legend: LegendBox, panel: PanelBox) -> bool {
    const EPSILON: f64 = 1e-6;
    legend.left >= panel.x0 - EPSILON
        && legend.right <= panel.x1 + EPSILON
        && legend.bottom >= panel.y0 - EPSILON
        && legend.top <= panel.y1 + EPSILON
}

fn overlap_score_within_panel(
    legend: LegendBox,
    geometry: &[crate::render::geometry::GeometryBox],
    margin: f64,
    panel: PanelBox,
) -> f64 {
    let legend = crate::render::geometry::GeometryBox {
        left: legend.left,
        right: legend.right,
        bottom: legend.bottom,
        top: legend.top,
        weight: 1.0,
    };
    geometry
        .iter()
        .filter_map(|bounds| {
            let clipped = crate::render::geometry::GeometryBox {
                left: bounds.left.max(panel.x0),
                right: bounds.right.min(panel.x1),
                bottom: bounds.bottom.max(panel.y0),
                top: bounds.top.min(panel.y1),
                weight: bounds.weight,
            };
            (clipped.left <= clipped.right && clipped.bottom <= clipped.top).then_some(clipped)
        })
        .map(|bounds| bounds.overlap_area(legend, margin))
        .sum()
}

fn expanded_axis_limits(
    kind: PlotKind,
    chart: &Chart<'_>,
    result: &PlotRenderResult,
    legend: &Legend,
    bounds: LegendBox,
    edge: ClearanceEdge,
) -> Option<(ClearanceAxis, (f64, f64))> {
    let panel = result.layout.projection.panel_box();
    let margin = legend.metrics.font_size.max(6.0);
    let (axis, range, fraction, expand_minimum) = match edge {
        ClearanceEdge::Left if result.x_domain == crate::spec::AxisDomain::Continuous => (
            ClearanceAxis::X,
            result.layout.projection.x_range(),
            (panel.x1 - bounds.right - margin) / panel.width(),
            true,
        ),
        ClearanceEdge::Right if result.x_domain == crate::spec::AxisDomain::Continuous => (
            ClearanceAxis::X,
            result.layout.projection.x_range(),
            (bounds.left - margin - panel.x0) / panel.width(),
            false,
        ),
        ClearanceEdge::Bottom if result.y_domain == crate::spec::AxisDomain::Continuous => (
            ClearanceAxis::Y,
            result.layout.projection.y_range(),
            (panel.y1 - bounds.top - margin) / panel.height(),
            true,
        ),
        ClearanceEdge::Top if result.y_domain == crate::spec::AxisDomain::Continuous => (
            ClearanceAxis::Y,
            result.layout.projection.y_range(),
            (bounds.bottom - margin - panel.y0) / panel.height(),
            false,
        ),
        _ => return None,
    };
    if matches!(axis, ClearanceAxis::X) && chart.user_x_limits().is_some()
        || matches!(axis, ClearanceAxis::Y) && chart.user_y_limits().is_some()
    {
        return None;
    }
    let (minimum, maximum) = range;
    if bar_expansion_moves_zero_anchor(kind, axis, range, expand_minimum) {
        return None;
    }
    let span = maximum - minimum;
    if !span.is_finite() || span <= 0.0 || !fraction.is_finite() || fraction <= 0.05 {
        return None;
    }
    let expanded_span = span / fraction.clamp(0.05, 0.95) * 1.05;
    let limits = if expand_minimum {
        (maximum - expanded_span, maximum)
    } else {
        (minimum, minimum + expanded_span)
    };
    (limits.0.is_finite() && limits.1.is_finite() && limits.0 < limits.1).then_some((axis, limits))
}

fn bar_expansion_moves_zero_anchor(
    kind: PlotKind,
    axis: ClearanceAxis,
    range: (f64, f64),
    expand_minimum: bool,
) -> bool {
    let numeric_bar_axis = matches!(
        (kind, axis),
        (PlotKind::Bar | PlotKind::StackedBar, ClearanceAxis::Y)
            | (
                PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar,
                ClearanceAxis::X
            )
    );
    numeric_bar_axis && ((expand_minimum && range.0 == 0.0) || (!expand_minimum && range.1 == 0.0))
}

fn render(
    chart: &Chart<'_>,
    kind: PlotKind,
    panel: PanelBox,
    data: &polars::prelude::DataFrame,
    source_rows: Option<&[usize]>,
    backend: &mut GeometryBackend,
) -> Result<PlotRenderResult, Error> {
    let context = source_rows.map_or_else(
        || PlotRenderContext::new(data, panel),
        |rows| PlotRenderContext::with_source_rows(data, rows, panel),
    );
    chart.render_panel(kind, context, backend)
}

fn auto_candidate_avoiding_panel_text(
    chart: &Chart<'_>,
    result: &PlotRenderResult,
    legend: &Legend,
    preferred: usize,
) -> Result<usize, Error> {
    let candidates = legend.inside_candidates(result.layout.projection.pixel_box());
    for offset in 0..candidates.len() {
        let index = (preferred + offset) % candidates.len();
        if crate::decorations::panel_text::conflicting_legend_corner(
            chart,
            &result.layout,
            candidates[index],
        )?
        .is_none()
        {
            return Ok(index);
        }
    }
    Err(Error::Render(
        "automatic legend cannot avoid the configured panel text annotations; hide the legend or choose an outside legend position"
            .into(),
    ))
}

fn reserve_outside_panel(
    mut panel: PanelBox,
    legend: &Legend,
    position: LegendPosition,
) -> Result<PanelBox, Error> {
    let gap = legend.metrics.pad.max(8.0);
    let vertical_band = matches!(
        position,
        LegendPosition::OutsideRightTop
            | LegendPosition::OutsideRight
            | LegendPosition::OutsideRightBottom
            | LegendPosition::OutsideLeftTop
            | LegendPosition::OutsideLeft
            | LegendPosition::OutsideLeftBottom
    );
    let horizontal_band = matches!(
        position,
        LegendPosition::OutsideTopLeft
            | LegendPosition::OutsideTop
            | LegendPosition::OutsideTopRight
            | LegendPosition::OutsideBottomLeft
            | LegendPosition::OutsideBottom
            | LegendPosition::OutsideBottomRight
    );
    if (vertical_band && legend.compute_box_height() > panel.height())
        || (horizontal_band && legend.compute_box_width() > panel.width())
    {
        return Err(Error::Render(
            "measured outside legend does not fit within its panel cell".into(),
        ));
    }
    match position {
        LegendPosition::OutsideRightTop
        | LegendPosition::OutsideRight
        | LegendPosition::OutsideRightBottom => panel.x1 -= legend.compute_box_width() + gap,
        LegendPosition::OutsideLeftTop
        | LegendPosition::OutsideLeft
        | LegendPosition::OutsideLeftBottom => panel.x0 += legend.compute_box_width() + gap,
        LegendPosition::OutsideTopLeft
        | LegendPosition::OutsideTop
        | LegendPosition::OutsideTopRight => panel.y1 -= legend.compute_box_height() + gap,
        LegendPosition::OutsideBottomLeft
        | LegendPosition::OutsideBottom
        | LegendPosition::OutsideBottomRight => panel.y0 += legend.compute_box_height() + gap,
        _ => {}
    }
    if panel.width() < 80.0 || panel.height() < 60.0 {
        return Err(Error::Render(
            "measured outside legend leaves less than the minimum 80x60 plot panel; increase the canvas or shorten legend labels"
                .into(),
        ));
    }
    Ok(panel)
}

fn initial_auto_candidate(
    legend: &Legend,
    panel: PanelBox,
    geometry: &[crate::render::geometry::GeometryBox],
    default: LegendDefault,
) -> usize {
    let pixel_box = panel.as_tuple();
    match default {
        LegendDefault::TopRight(_) | LegendDefault::SelfManaged => 0,
        LegendDefault::LowestOverlap => {
            let margin = legend.metrics.font_size.max(6.0);
            legend
                .inside_candidates(pixel_box)
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    overlap_score_within_panel(**left, geometry, margin, panel).total_cmp(
                        &overlap_score_within_panel(**right, geometry, margin, panel),
                    )
                })
                .map_or(0, |(index, _)| index)
        }
    }
}

fn resolve_box(
    legend: &Legend,
    position: LegendPosition,
    default: LegendDefault,
    axes: PanelBox,
    reserved_panel: PanelBox,
    cell: PanelBox,
    geometry: &[crate::render::geometry::GeometryBox],
) -> LegendBox {
    let pixel_box = axes.as_tuple();
    match position {
        LegendPosition::Auto => legend.inside_candidates(pixel_box)
            [initial_auto_candidate(legend, axes, geometry, default)],
        LegendPosition::Hidden => legend.top_right_box(pixel_box),
        LegendPosition::InsideTopLeft | LegendPosition::InsideLeftTop => {
            legend.aligned_inside_box(pixel_box, 0.0, 1.0)
        }
        LegendPosition::InsideTop => legend.aligned_inside_box(pixel_box, 0.5, 1.0),
        LegendPosition::InsideTopRight | LegendPosition::InsideRightTop => {
            legend.aligned_inside_box(pixel_box, 1.0, 1.0)
        }
        LegendPosition::InsideRight => legend.aligned_inside_box(pixel_box, 1.0, 0.5),
        LegendPosition::InsideRightBottom | LegendPosition::InsideBottomRight => {
            legend.aligned_inside_box(pixel_box, 1.0, 0.0)
        }
        LegendPosition::InsideBottomLeft | LegendPosition::InsideLeftBottom => {
            legend.aligned_inside_box(pixel_box, 0.0, 0.0)
        }
        LegendPosition::InsideBottom => legend.aligned_inside_box(pixel_box, 0.5, 0.0),
        LegendPosition::InsideLeft => legend.aligned_inside_box(pixel_box, 0.0, 0.5),
        LegendPosition::OutsideRightTop => outside_right(legend, reserved_panel, cell, 1.0),
        LegendPosition::OutsideRight => outside_right(legend, reserved_panel, cell, 0.5),
        LegendPosition::OutsideRightBottom => outside_right(legend, reserved_panel, cell, 0.0),
        LegendPosition::OutsideLeftTop => outside_left(legend, reserved_panel, cell, 1.0),
        LegendPosition::OutsideLeft => outside_left(legend, reserved_panel, cell, 0.5),
        LegendPosition::OutsideLeftBottom => outside_left(legend, reserved_panel, cell, 0.0),
        LegendPosition::OutsideTopLeft => outside_top(legend, reserved_panel, cell, 0.0),
        LegendPosition::OutsideTop => outside_top(legend, reserved_panel, cell, 0.5),
        LegendPosition::OutsideTopRight => outside_top(legend, reserved_panel, cell, 1.0),
        LegendPosition::OutsideBottomLeft => outside_bottom(legend, reserved_panel, cell, 0.0),
        LegendPosition::OutsideBottom => outside_bottom(legend, reserved_panel, cell, 0.5),
        LegendPosition::OutsideBottomRight => outside_bottom(legend, reserved_panel, cell, 1.0),
    }
}

fn outside_right(legend: &Legend, panel: PanelBox, _cell: PanelBox, align: f64) -> LegendBox {
    let gap = legend.metrics.pad.max(8.0);
    let height = legend.compute_box_height();
    legend.box_at(
        panel.x1 + gap,
        panel.y0 + (panel.height() - height).max(0.0) * align,
    )
}

fn outside_left(legend: &Legend, panel: PanelBox, _cell: PanelBox, align: f64) -> LegendBox {
    let gap = legend.metrics.pad.max(8.0);
    let height = legend.compute_box_height();
    legend.box_at(
        panel.x0 - gap - legend.compute_box_width(),
        panel.y0 + (panel.height() - height).max(0.0) * align,
    )
}

fn outside_top(legend: &Legend, panel: PanelBox, _cell: PanelBox, align: f64) -> LegendBox {
    let gap = legend.metrics.pad.max(8.0);
    legend.box_at(
        panel.x0 + (panel.width() - legend.compute_box_width()).max(0.0) * align,
        panel.y1 + gap,
    )
}

fn outside_bottom(legend: &Legend, panel: PanelBox, _cell: PanelBox, align: f64) -> LegendBox {
    let gap = legend.metrics.pad.max(8.0);
    legend.box_at(
        panel.x0 + (panel.width() - legend.compute_box_width()).max(0.0) * align,
        panel.y0 - gap - legend.compute_box_height(),
    )
}

#[cfg(test)]
mod tests;
