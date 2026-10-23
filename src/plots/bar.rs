//! Constrained vertical bar payloads from retained scientific preparation.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]

use crate::Error;

use crate::layout::{Layout, PhysicalTextAlignment, Projection, ResolvedTextPlan};
use crate::plots::bar_data::{self, BarMode, PreparedBar, PreparedBarSignificance, PreparedBars};
use crate::render::backend::{PlacedText, Point2D, TextAnchor};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::Plot;

/// Measure the current vertical bar policy without a layout backend.
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
    let mut rendered_maxima =
        crate::data::source::collect_source((0..prepared.categories.len()).map(|_| Ok(0.0_f64)))?;
    let anchor_count = prepared.series.len().max(1);
    let mut grouped_maxima =
        crate::data::source::collect_source((0..prepared.categories.len()).map(|_| {
            crate::data::source::collect_source((0..anchor_count).map(|_| Ok(f64::NEG_INFINITY)))
        }))?;
    for bar in &prepared.bars {
        let maximum = bar
            .interval
            .map_or(bar.start.max(bar.end), |(_, upper)| bar.start.max(upper));
        rendered_maxima[bar.category_index] = rendered_maxima[bar.category_index].max(maximum);
        let anchor = bar.series_index.unwrap_or(0);
        grouped_maxima[bar.category_index][anchor] =
            grouped_maxima[bar.category_index][anchor].max(maximum);
    }
    for point in &prepared.points {
        rendered_maxima[point.category_index] =
            rendered_maxima[point.category_index].max(point.value);
        grouped_maxima[point.category_index][point.series_index.unwrap_or(0)] =
            grouped_maxima[point.category_index][point.series_index.unwrap_or(0)].max(point.value);
    }
    if mode == BarMode::Stacked {
        for (category_index, maximum) in rendered_maxima.iter().enumerate() {
            grouped_maxima[category_index].fill(*maximum);
        }
    }
    let panel = context.panel;
    let attached = bar_data::BarSignificance::new(chart, theme, false)?;
    let attached_label_plans = bar_data::attached_significance_text_plans(
        context,
        &prepared.significance,
        &attached.appearance.label,
        |_| PhysicalTextAlignment::Center,
        "bar attached significance label",
        attached.appearance.exact_override,
    )?;
    let base_y_range = layout.projection.y_range();
    let mark_range = range_for_bar_marks(
        chart,
        prepared,
        panel,
        base_y_range,
        &attached_label_plans,
        &attached,
    )?;
    let value_per_pixel = (base_y_range.1 - base_y_range.0) / panel.height();
    for point in &prepared.points {
        let clearance = point.value + f64::from(point.style.radius) * value_per_pixel;
        rendered_maxima[point.category_index] =
            rendered_maxima[point.category_index].max(clearance);
        grouped_maxima[point.category_index][point.series_index.unwrap_or(0)] =
            grouped_maxima[point.category_index][point.series_index.unwrap_or(0)].max(clearance);
    }
    for (mark, text_plan) in prepared
        .significance
        .iter()
        .zip(&attached_label_plans)
        .filter(|(mark, _)| mark.positive)
    {
        let clearance =
            mark.edge + attached.appearance.attached_reserve(text_plan.height())? * value_per_pixel;
        rendered_maxima[mark.category_index] = rendered_maxima[mark.category_index].max(clearance);
        if let Some(series_index) = mark.series_index {
            grouped_maxima[mark.category_index][series_index] =
                grouped_maxima[mark.category_index][series_index].max(clearance);
        } else {
            for maximum in &mut grouped_maxima[mark.category_index] {
                if maximum.is_finite() {
                    *maximum = maximum.max(clearance);
                }
            }
        }
    }
    let significance = if prepared.series.is_empty() {
        crate::significance::plan(
            chart,
            &prepared.categories,
            &rendered_maxima,
            base_y_range,
            panel,
            theme,
            context.fit_scope(),
        )?
    } else {
        crate::significance::plan_grouped(
            chart,
            &prepared.categories,
            &prepared.series,
            &grouped_maxima,
            theme.marks.get_extents().get_categories().group_span(false),
            base_y_range,
            panel,
            mode == BarMode::Stacked,
            theme,
            context.fit_scope(),
        )?
    };
    let domain_expansion = bar_data::domain_expansion(
        base_y_range,
        mark_range,
        significance.y_range,
        crate::layout::interior::ClearanceAxis::Y,
    )?;
    validate_bar_significance_collisions(
        &attached,
        prepared,
        &layout,
        &attached_label_plans,
        context.fit_scope(),
        domain_expansion.is_some(),
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
    for bar in &prepared.bars {
        let (x_center, width) = bar_data::category_bar_geometry(
            bar,
            prepared,
            mode,
            chart.bar_spacing_spec(),
            false,
            theme.marks.get_extents().get_categories(),
        );
        let polygon = project_bar(&layout.projection, x_center, width, bar)?;
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
                    .backend_style(prepared.bar_color(chart, bar, &colors, &accents)?),
            ),
            interval: bar
                .interval
                .map(|(lower, upper)| {
                    project_error_bar(
                        &layout.projection,
                        x_center,
                        width,
                        lower,
                        upper,
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
            &attached,
            mark,
            text_plan,
            prepared,
            &layout,
            domain_expansion.is_some(),
        )?);
    }
    for point in &prepared.points {
        let x_center = bar_data::category_center(
            point.category_index,
            point.series_index,
            prepared,
            mode,
            theme.marks.get_extents().get_categories().group_span(false),
        );
        let projected = layout.projection.project(x_center, point.value);
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
    result.x_domain = crate::spec::AxisDomain::Categorical;
    payload.retain(result, context, chart, prepared)
}

fn range_for_bar_marks(
    chart: &Plot,
    prepared: &PreparedBars,
    panel: crate::layout::PanelBox,
    range: (f64, f64),
    text_plans: &[ResolvedTextPlan],
    attached: &bar_data::BarSignificance,
) -> Result<(f64, f64), Error> {
    if prepared.significance.is_empty() || chart.user_y_limits().is_some() {
        return Ok(range);
    }
    let units_per_pixel = (range.1 - range.0) / panel.height();
    let mut extent = prepared.value_range;
    for (mark, text_plan) in prepared.significance.iter().zip(text_plans) {
        let clearance = attached.appearance.attached_reserve(text_plan.height())? * units_per_pixel;
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
    pending_domain: bool,
) -> Result<PlacedText, Error> {
    let gap = attached.appearance.gap_pixels()?;
    let x = attached.center(mark, prepared);
    let (x, edge_y) = layout.projection.project(x, mark.edge);
    let y = if mark.positive {
        edge_y + gap
    } else {
        edge_y - gap - f64::from(text_plan.style.font_size)
    };
    let panel = layout.projection.panel_box();
    if !pending_domain && (y < panel.y0 || y + text_plan.height() > panel.y1) {
        return Err(Error::Render(format!(
            "bar significance mark {:?} does not clear its confidence interval and panel boundary; widen the y-axis range",
            mark.label
        )));
    }
    Ok(PlacedText::middle(
        text_plan.clone(),
        Point2D::new(x.round(), y.round()),
        TextAnchor::Middle,
        0.0,
    ))
}

fn validate_bar_significance_collisions(
    attached: &bar_data::BarSignificance,
    prepared: &PreparedBars,
    layout: &Layout,
    text_plans: &[ResolvedTextPlan],
    scope: &crate::layout::fit::FitScope,

    pending_domain: bool,
) -> Result<(), crate::layout::fit::PlanError> {
    if prepared.significance.is_empty() {
        return Ok(());
    }
    let gap = attached.appearance.gap_pixels()?;
    let mut bounds = Vec::new();
    crate::data::source::reserve_source(&mut bounds, prepared.significance.len())?;
    for (index, (mark, text_plan)) in prepared.significance.iter().zip(text_plans).enumerate() {
        let scope = scope.child(crate::layout::AddressRole::Annotation, index)?;
        let center = attached.center(mark, prepared);
        let (x, edge_y) = layout.projection.project(center, mark.edge);
        let width = text_plan.width();
        let height = text_plan.height();
        let y0 = if mark.positive {
            edge_y + gap
        } else {
            edge_y - gap - height
        };
        let candidate = (x - width / 2.0, x + width / 2.0, y0, y0 + height);
        let draw_y = if mark.positive {
            edge_y + gap
        } else {
            edge_y - gap - f64::from(text_plan.style.font_size)
        };
        let panel = layout.projection.panel_box();
        if !pending_domain && (draw_y < panel.y0 || draw_y + height > panel.y1) {
            return Err(scope.failure(crate::layout::fit::FitKind::OverlayCollision, format!("bar significance mark {:?} does not clear its confidence interval and panel boundary; widen the y-axis range", mark.label)).into());
        }

        if bounds.iter().any(|&(x0, x1, y0, y1)| {
            candidate.0 < x1 && x0 < candidate.1 && candidate.2 < y1 && y0 < candidate.3
        }) {
            return Err(scope
                .failure(
                    crate::layout::fit::FitKind::OverlayCollision,
                    "bar significance marks overlap; enlarge the chart or reduce mark typography",
                )
                .into());
        }
        bounds.push(candidate);
    }
    Ok(())
}

fn project_bar(
    proj: &Projection,
    x_center: f64,
    width: f64,
    bar: &PreparedBar,
) -> Result<[Point2D; 4], Error> {
    let x = (x_center - width / 2.0, x_center + width / 2.0);
    let (left, bottom) = proj.project(x.0, bar.start);
    let (right, top) = proj.project(x.1, bar.end);
    proj.validate_span(
        crate::layout::AxisSide::Bottom,
        x,
        (left, right),
        width > 0.0,
        "bar polygon width",
    )?;
    proj.validate_span(
        crate::layout::AxisSide::Left,
        (bar.start, bar.end),
        (bottom, top),
        bar.nonzero,
        "bar polygon height",
    )?;
    Ok([
        Point2D::new(left, bottom),
        Point2D::new(left, top),
        Point2D::new(right, top),
        Point2D::new(right, bottom),
    ])
}

fn project_error_bar(
    proj: &Projection,
    x_center: f64,
    width: f64,
    lower: f64,
    upper: f64,
    cap_ratio: f64,
) -> Result<[[Point2D; 2]; 3], Error> {
    let (x_px, y_top_px) = proj.project(x_center, upper);
    let (_, y_bot_px) = proj.project(x_center, lower);
    proj.validate_span(
        crate::layout::AxisSide::Left,
        (lower, upper),
        (y_bot_px, y_top_px),
        lower < upper,
        "bar interval",
    )?;
    let cap_px = (proj.project(width, 0.0).0 - proj.project(0.0, 0.0).0).abs() * cap_ratio;
    Ok([
        [Point2D::new(x_px, y_top_px), Point2D::new(x_px, y_bot_px)],
        [
            Point2D::new(x_px - cap_px, y_top_px),
            Point2D::new(x_px + cap_px, y_top_px),
        ],
        [
            Point2D::new(x_px - cap_px, y_bot_px),
            Point2D::new(x_px + cap_px, y_bot_px),
        ],
    ])
}
