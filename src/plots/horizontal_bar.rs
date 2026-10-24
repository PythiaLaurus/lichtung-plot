//! Constrained horizontal bar payloads from retained scientific preparation.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]

use crate::Error;

use crate::layout::{Layout, PanelBox, PhysicalTextAlignment, Projection, ResolvedTextPlan};
use crate::plots::bar_data::{self, BarMode, PreparedBar, PreparedBarSignificance, PreparedBars};
use crate::render::backend::{PlacedText, Point2D, TextAnchor};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::Plot;

pub(crate) fn measure_marks(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    layout: Layout,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let theme = context.theme;

    #[cfg(test)]
    bar_data::count(chart.kind(), 1);
    let mode = bar_data::mode(chart);
    let prepared = bar_data::retained(chart)?;
    let resources = theme.prepared_font_environment()?.operation();
    let data = context.data.with_operation(&resources);
    let (colors, accents) = prepared.resolved_colors(chart, theme, data)?;
    let y_count = prepared.categories.len() as f64;
    let panel_box = context.panel;
    let attached = bar_data::BarSignificance::new(chart, theme, true)?;
    let mut attached_label_plans = bar_data::attached_significance_text_plans(
        context,
        &prepared.significance,
        &attached.appearance.label,
        |mark| {
            if mark.positive {
                PhysicalTextAlignment::Start
            } else {
                PhysicalTextAlignment::End
            }
        },
        "horizontal bar attached significance label",
        attached.appearance.exact_override,
    )?;
    let base_x_range = layout.projection.x_range();
    let mark_range = range_for_bar_marks(
        chart,
        prepared,
        panel_box,
        base_x_range,
        &attached_label_plans,
        &attached,
    )?;
    let anchor_count = prepared.series.len().max(1);
    let mut rendered_maxima =
        crate::data::source::collect_source((0..prepared.categories.len()).map(|_| {
            crate::data::source::collect_source((0..anchor_count).map(|_| Ok(f64::NEG_INFINITY)))
        }))?;
    for bar in &prepared.bars {
        let anchor = bar.series_index.unwrap_or(0);
        rendered_maxima[bar.category_index][anchor] = rendered_maxima[bar.category_index][anchor]
            .max(
                bar.interval
                    .map_or(bar.start.max(bar.end), |(_, upper)| bar.start.max(upper)),
            );
    }
    for point in &prepared.points {
        rendered_maxima[point.category_index][point.series_index.unwrap_or(0)] =
            rendered_maxima[point.category_index][point.series_index.unwrap_or(0)].max(point.value);
    }
    let value_per_pixel = (base_x_range.1 - base_x_range.0) / panel_box.width();
    for point in &prepared.points {
        let clearance = point.value + f64::from(point.style.radius) * value_per_pixel;
        rendered_maxima[point.category_index][point.series_index.unwrap_or(0)] =
            rendered_maxima[point.category_index][point.series_index.unwrap_or(0)].max(clearance);
    }
    for (mark, text_plan) in prepared
        .significance
        .iter()
        .zip(&attached_label_plans)
        .filter(|(mark, _)| mark.positive)
    {
        let width = text_plan.width();
        let clearance = mark.edge + attached.appearance.attached_reserve(width)? * value_per_pixel;
        if let Some(series_index) = mark.series_index {
            rendered_maxima[mark.category_index][series_index] =
                rendered_maxima[mark.category_index][series_index].max(clearance);
        } else {
            for maximum in &mut rendered_maxima[mark.category_index] {
                if maximum.is_finite() {
                    *maximum = maximum.max(clearance);
                }
            }
        }
    }
    if mode == BarMode::Stacked {
        for maxima in &mut rendered_maxima {
            let maximum = maxima.iter().copied().fold(0.0_f64, f64::max);
            maxima.fill(maximum);
        }
    }
    let significance = crate::significance::plan_horizontal(
        chart,
        &prepared.categories,
        &prepared.series,
        &rendered_maxima,
        theme.marks.get_extents().get_categories().group_span(true),
        base_x_range,
        panel_box,
        mode == BarMode::Stacked,
        theme,
        context.fit_scope(),
    )?;
    let domain_expansion = bar_data::domain_expansion(
        base_x_range,
        mark_range,
        significance.y_range,
        crate::layout::interior::ClearanceAxis::X,
    )?;
    bar_data::validate_attached_text(
        &prepared.significance,
        &mut attached_label_plans,
        layout.projection.panel_box(),
        context.fit_scope(),
        true,
        domain_expansion.is_some(),
        |mark, plan| place_bar_significance(&attached, mark, plan, prepared, &layout),
    )?;

    let mut payload = bar_data::BarPayload {
        clip: layout.projection.panel_box(),
        domain_expansion,
        zero: None,
        zero_style: theme
            .marks
            .get_strokes()
            .get_references()
            .get_bar_zero()
            .backend_style(),
        bars: Vec::new(),
        text: Vec::new(),
        points: Vec::new(),
        #[cfg(test)]
        kind: chart.kind(),
    };
    crate::data::source::reserve_source(&mut payload.bars, prepared.bars.len())?;
    crate::data::source::reserve_source(&mut payload.text, prepared.significance.len())?;
    crate::data::source::reserve_source(&mut payload.points, prepared.points.len())?;
    if chart.zero_line_visible() && base_x_range.0 <= 0.0 && base_x_range.1 >= 0.0 {
        let (x, y0) = layout.projection.project(0.0, 0.0);
        let (_, y1) = layout.projection.project(0.0, y_count);
        payload.zero = Some([Point2D::new(x, y0), Point2D::new(x, y1)]);
    }
    for bar in &prepared.bars {
        let (y_center, height) = bar_data::category_bar_geometry(
            bar,
            prepared,
            mode,
            chart.bar_spacing_spec(),
            true,
            theme.marks.get_extents().get_categories(),
        );
        let color = prepared.bar_color(chart, bar, &colors, &accents)?;
        let polygon = project_bar(&layout.projection, bar, y_center, height)?;
        payload.bars.push(bar_data::BarShape {
            polygon,
            interval_style: crate::alpha::marks::line_style(
                bar.line_alpha,
                theme
                    .marks
                    .get_strokes()
                    .get_summaries()
                    .get_uncertainty()
                    .backend_style(),
            ),
            fill: crate::alpha::marks::fill_style(
                bar.fill_alpha,
                theme
                    .marks
                    .get_fills()
                    .get_bodies()
                    .get_bar_body()
                    .backend_style(color),
            ),
            interval: bar
                .interval
                .map(|(lower, upper)| {
                    project_error_bar(
                        &layout.projection,
                        lower,
                        upper,
                        y_center,
                        height,
                        theme
                            .marks
                            .get_extents()
                            .get_categories()
                            .get_uncertainty_cap_ratio()
                            .as_f64(),
                    )
                })
                .transpose()?,
        });
    }
    for (mark, text_plan) in prepared.significance.iter().zip(&attached_label_plans) {
        payload.text.push(place_bar_significance(
            &attached, mark, text_plan, prepared, &layout,
        )?);
    }
    for point in &prepared.points {
        let y_center = bar_data::category_center(
            point.category_index,
            point.series_index,
            prepared,
            mode,
            theme.marks.get_extents().get_categories().group_span(true),
        );
        let projected = layout.projection.project(point.value, y_center);
        payload
            .points
            .push((Point2D::new(projected.0, projected.1), point.style.clone()));
    }
    payload.domain_expansion = bar_data::automatic_numeric_ink_expansion(
        chart,
        prepared,
        &payload,
        &layout.projection,
        context.fit_scope(),
    )?;
    let mut result = PlotRenderResult::new(layout, chart.prepared_discrete_guide()?)
        .with_significance_plan(significance)
        .with_deferred_annotation_text(crate::data::source::collect_source(
            attached_label_plans.into_iter().map(|plan| {
                Ok(crate::layout::DeferredAnnotationTextPlan::from_resolved(
                    plan,
                ))
            }),
        )?);
    result.y_domain = crate::spec::AxisDomain::Categorical;
    payload.retain(result, context, chart, prepared)
}

fn range_for_bar_marks(
    chart: &Plot,
    prepared: &PreparedBars,
    panel: PanelBox,
    range: (f64, f64),
    text_plans: &[ResolvedTextPlan],
    attached: &bar_data::BarSignificance,
) -> Result<(f64, f64), Error> {
    if prepared.significance.is_empty() || chart.user_x_limits().is_some() {
        return Ok(range);
    }
    let units_per_pixel = (range.1 - range.0) / panel.width();
    let mut extent = prepared.value_range;
    for (mark, text_plan) in prepared.significance.iter().zip(text_plans) {
        let width = text_plan.width();
        let clearance = attached.appearance.attached_reserve(width)? * units_per_pixel;
        if mark.positive {
            extent.1 = extent.1.max(mark.edge + clearance);
        } else {
            extent.0 = extent.0.min(mark.edge - clearance);
        }
    }
    if extent.0 >= range.0 && extent.1 <= range.1 {
        return Ok(range);
    }
    Ok((range.0.min(extent.0), range.1.max(extent.1)))
}

fn place_bar_significance(
    attached: &bar_data::BarSignificance,
    mark: &PreparedBarSignificance,
    text_plan: &ResolvedTextPlan,
    prepared: &PreparedBars,
    layout: &Layout,
) -> Result<PlacedText, Error> {
    let gap = attached.appearance.gap_pixels()?;
    let y = attached.center(mark, prepared);
    let (edge_x, y) = layout.projection.project(mark.edge, y);
    let (x, anchor) = if mark.positive {
        let x = edge_x + gap;
        (x, TextAnchor::Start)
    } else {
        let x = edge_x - gap;
        (x, TextAnchor::End)
    };
    PlacedText::middle(
        text_plan.clone(),
        Point2D::new(x.round(), y.round()),
        anchor,
        0.0,
    )
}

fn project_bar(
    proj: &Projection,
    bar: &PreparedBar,
    y_center: f64,
    height: f64,
) -> Result<[Point2D; 4], Error> {
    let y = (y_center - height / 2.0, y_center + height / 2.0);
    let (left, bottom) = proj.project(bar.start, y.0);
    let (right, top) = proj.project(bar.end, y.1);
    proj.validate_span(
        crate::layout::AxisSide::Bottom,
        (bar.start, bar.end),
        (left, right),
        bar.nonzero,
        "horizontal bar polygon width",
    )?;
    proj.validate_span(
        crate::layout::AxisSide::Left,
        y,
        (bottom, top),
        height > 0.0,
        "horizontal bar polygon height",
    )?;
    Ok([
        Point2D::new(left, bottom),
        Point2D::new(right, bottom),
        Point2D::new(right, top),
        Point2D::new(left, top),
    ])
}

fn project_error_bar(
    proj: &Projection,
    lower: f64,
    upper: f64,
    y_center: f64,
    height: f64,
    cap_ratio: f64,
) -> Result<[[Point2D; 2]; 3], Error> {
    let (x_left, y_px) = proj.project(lower, y_center);
    let (x_right, _) = proj.project(upper, y_center);
    proj.validate_span(
        crate::layout::AxisSide::Bottom,
        (lower, upper),
        (x_left, x_right),
        lower < upper,
        "horizontal bar interval",
    )?;
    let cap_px = (proj.project(0.0, height).1 - proj.project(0.0, 0.0).1).abs() * cap_ratio;
    Ok([
        [Point2D::new(x_left, y_px), Point2D::new(x_right, y_px)],
        [
            Point2D::new(x_left, y_px - cap_px),
            Point2D::new(x_left, y_px + cap_px),
        ],
        [
            Point2D::new(x_right, y_px - cap_px),
            Point2D::new(x_right, y_px + cap_px),
        ],
    ])
}
