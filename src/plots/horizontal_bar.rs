//! Horizontal bar plot implementation.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]

use crate::Error;

use crate::layout::{
    AxisTick, Layout, PanelBox, PhysicalTextAlignment, Projection, ResolvedTextPlan, TextBlock,
};
use crate::plots::bar_data::{self, BarMode, PreparedBar, PreparedBarSignificance, PreparedBars};
use crate::render::backend::{
    with_clip, Backend, FillStyle, LineStyle, PlannedTextPlacement, Point2D, TextAnchor,
};
use crate::spec::{BarSpacing, PlotKind, PlotRenderContext, PlotRenderResult};
use crate::theme::ResolvedTheme;
use crate::Plot;

pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let theme = context.theme;

    let mode = match chart.kind() {
        PlotKind::StackedHorizontalBar => BarMode::Stacked,
        _ if chart.group_col().is_some() => BarMode::Grouped,
        _ => BarMode::Single,
    };
    let prepared = bar_data::prepare(chart, context.data, mode)?;
    let display_categories = chart.resolve_label_domain(
        "y-axis categories",
        prepared.categories.iter().map(String::as_str),
    )?;
    let legend_entries = bar_data::legend_entries(&prepared, chart, theme);
    let mut base_x_range =
        crate::layout::anchored_auto_range(prepared.value_range, chart.x_axis_spec(), 0.0);
    let y_count = prepared.categories.len() as f64;
    let y_ticks = prepared
        .categories
        .iter()
        .enumerate()
        .map(|(i, label)| {
            AxisTick::new(
                i as f64 + 0.5,
                if chart.y_tick_labels_visible() {
                    label.clone()
                } else {
                    String::new()
                },
            )
        })
        .collect();
    let semantic_panel = context.panel;
    let (panel_box, tick_width) = if chart.y_tick_labels_visible() {
        panel_box_for_y_tick_labels(
            chart,
            semantic_panel,
            &display_categories,
            theme,
            context.fit_scope(),
        )?
    } else {
        (semantic_panel, 1.0 / 1024.0)
    };
    let mark_style = chart.significance_mark_style_spec();
    let significance_label = theme
        .typography
        .effective_significance_label(chart.significance_mark_style_override());
    let attached_label_plans = bar_data::attached_significance_text_plans(
        context,
        &prepared.significance,
        &significance_label,
        |mark| {
            if mark.positive {
                PhysicalTextAlignment::Start
            } else {
                PhysicalTextAlignment::End
            }
        },
        "horizontal bar attached significance label",
        chart.significance_mark_style_override().is_some(),
    )?;
    base_x_range = range_for_bar_marks(
        chart,
        &prepared,
        panel_box,
        base_x_range,
        &attached_label_plans,
    );
    let anchor_count = prepared.series.len().max(1);
    let mut rendered_maxima =
        vec![vec![f64::NEG_INFINITY; anchor_count]; prepared.categories.len()];
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
        let clearance = mark.edge + (mark_style.gap_spec() + width + 2.0) * value_per_pixel;
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
        0.74,
        base_x_range,
        panel_box,
        mode == BarMode::Stacked,
        theme,
        context.fit_scope(),
    )?;
    let x_range = significance.y_range;
    let x_axis_spec = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
    let mut layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
        x_range,
        (0.0, y_count),
        panel_box,
        chart.x_label_text().unwrap_or("").to_string(),
        chart.y_label_text().unwrap_or("").to_string(),
        Vec::new(),
        y_ticks,
        &x_axis_spec,
        chart.y_axis_spec(),
        theme,
    )?;
    layout.y_axis.assigned_tick_width = Some(tick_width);
    validate_bar_significance_collisions(
        chart,
        &prepared,
        mode,
        &layout,
        &attached_label_plans,
        context.fit_scope(),
    )?;

    with_clip(backend, layout.projection.panel_box(), |backend| {
        if chart.zero_line_visible() && x_range.0 <= 0.0 && x_range.1 >= 0.0 {
            let (x, y0) = layout.projection.project(0.0, 0.0);
            let (_, y1) = layout.projection.project(0.0, y_count);
            backend.draw_line(
                &[Point2D::new(x, y0), Point2D::new(x, y1)],
                &LineStyle::solid(theme.foreground, 0.8),
            )?;
        }
        for bar in &prepared.bars {
            let (y_center, height) = bar_geometry(bar, &prepared, mode, chart.bar_spacing_spec());
            let color_index = bar.hue_index.unwrap_or_else(|| {
                if chart.group_col().is_some() {
                    0
                } else {
                    bar.category_index
                }
            });
            let color = if chart.colors_by_sign() {
                if bar.end >= bar.start {
                    theme.colors[0]
                } else {
                    theme.colors[1 % theme.colors.len()]
                }
            } else {
                theme.colors[color_index % theme.colors.len()]
            };
            draw_bar(
                backend,
                &layout.projection,
                bar.start,
                bar.end,
                y_center,
                height,
                &FillStyle {
                    color,
                    stroke: theme.foreground,
                    stroke_width: 1.0,
                    opacity: 1.0,
                },
            )?;
            if let Some((lower, upper)) = bar.interval {
                let style = LineStyle::solid(theme.foreground, theme.interval_width);
                draw_error_bar(
                    backend,
                    &layout.projection,
                    lower,
                    upper,
                    y_center,
                    height,
                    &style,
                )?;
            }
        }
        for (mark, text_plan) in prepared.significance.iter().zip(&attached_label_plans) {
            draw_bar_significance(chart, mark, text_plan, &prepared, mode, &layout, backend)?;
        }
        for point in &prepared.points {
            let y_center = point_center(point.category_index, point.series_index, &prepared, mode);
            let projected = layout.projection.project(point.value, y_center);
            backend.draw_scatter(&[Point2D::new(projected.0, projected.1)], &point.style)?;
        }
        Ok(())
    })?;
    if legend_entries.is_empty() {
        Ok(PlotRenderResult::new(layout, Vec::new())
            .with_significance_plan(significance)
            .with_deferred_annotation_text(deferred_annotation_text(&attached_label_plans)))
    } else {
        Ok(PlotRenderResult::new(layout, legend_entries)
            .with_significance_plan(significance)
            .with_deferred_annotation_text(deferred_annotation_text(&attached_label_plans)))
    }
}

fn deferred_annotation_text(
    plans: &[ResolvedTextPlan],
) -> Vec<crate::layout::DeferredAnnotationTextPlan> {
    plans
        .iter()
        .cloned()
        .map(crate::layout::DeferredAnnotationTextPlan::from_resolved)
        .collect()
}

fn range_for_bar_marks(
    chart: &Plot,
    prepared: &PreparedBars,
    panel: PanelBox,
    range: (f64, f64),
    text_plans: &[ResolvedTextPlan],
) -> (f64, f64) {
    if prepared.significance.is_empty() || chart.user_x_limits().is_some() {
        return range;
    }
    let style = chart.significance_mark_style_spec();
    let units_per_pixel = (range.1 - range.0) / panel.width();
    let mut extent = prepared.value_range;
    for (mark, text_plan) in prepared.significance.iter().zip(text_plans) {
        let width = text_plan.width();
        let clearance = (style.gap_spec() + width + 2.0) * units_per_pixel;
        if mark.positive {
            extent.1 = extent.1.max(mark.edge + clearance);
        } else {
            extent.0 = extent.0.min(mark.edge - clearance);
        }
    }
    crate::layout::anchored_auto_range(extent, chart.x_axis_spec(), 0.0)
}

fn draw_bar_significance(
    chart: &Plot,
    mark: &PreparedBarSignificance,
    text_plan: &ResolvedTextPlan,
    prepared: &PreparedBars,
    mode: BarMode,
    layout: &Layout,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let style = chart.significance_mark_style_spec();
    let y = significance_center(mark.category_index, mark.series_index, prepared, mode);
    let (edge_x, y) = layout.projection.project(mark.edge, y);
    let width = text_plan.width();
    let panel = layout.projection.panel_box();
    let (x, anchor, fits) = if mark.positive {
        let x = edge_x + style.gap_spec();
        (x, TextAnchor::Start, x + width <= panel.x1)
    } else {
        let x = edge_x - style.gap_spec();
        (x, TextAnchor::End, x - width >= panel.x0)
    };
    if !fits {
        return Err(Error::Render(format!(
            "horizontal bar significance mark {:?} does not clear its confidence interval and panel boundary; widen the x-axis range",
            mark.label
        )));
    }
    backend.draw_planned_text(
        PlannedTextPlacement::middle(Point2D::new(x.round(), y.round()), anchor, 0.0),
        text_plan,
    )
}

fn validate_bar_significance_collisions(
    chart: &Plot,
    prepared: &PreparedBars,
    mode: BarMode,
    layout: &Layout,
    text_plans: &[ResolvedTextPlan],
    scope: &crate::layout::fit::FitScope,
) -> Result<(), crate::layout::fit::PlanError> {
    let style = chart.significance_mark_style_spec();
    let mut bounds = Vec::new();
    for (index, (mark, text_plan)) in prepared.significance.iter().zip(text_plans).enumerate() {
        let scope = scope.child(crate::layout::AddressRole::Annotation, index)?;
        let y = significance_center(mark.category_index, mark.series_index, prepared, mode);
        let (edge_x, y) = layout.projection.project(mark.edge, y);
        let width = text_plan.width();
        let height = text_plan.height();
        let (x0, x1) = if mark.positive {
            (edge_x + style.gap_spec(), edge_x + style.gap_spec() + width)
        } else {
            (edge_x - style.gap_spec() - width, edge_x - style.gap_spec())
        };
        let candidate = (x0, x1, y - height / 2.0, y + height / 2.0);
        let panel = layout.projection.panel_box();
        if x0 < panel.x0 || x1 > panel.x1 {
            return Err(scope.failure(crate::layout::fit::FitKind::OverlayCollision, format!("horizontal bar significance mark {:?} does not clear its confidence interval and panel boundary; widen the x-axis range", mark.label)).into());
        }
        if bounds.iter().any(|&(x0, x1, y0, y1)| {
            candidate.0 < x1 && x0 < candidate.1 && candidate.2 < y1 && y0 < candidate.3
        }) {
            return Err(scope.failure(crate::layout::fit::FitKind::OverlayCollision,
                "horizontal bar significance marks overlap; enlarge the chart or reduce mark typography"
            ).into());
        }
        bounds.push(candidate);
    }
    Ok(())
}

fn significance_center(
    category_index: usize,
    series_index: Option<usize>,
    prepared: &PreparedBars,
    mode: BarMode,
) -> f64 {
    series_index.map_or(category_index as f64 + 0.5, |series_index| {
        point_center(category_index, Some(series_index), prepared, mode)
    })
}

fn point_center(
    category_index: usize,
    series_index: Option<usize>,
    prepared: &PreparedBars,
    mode: BarMode,
) -> f64 {
    let category_center = category_index as f64 + 0.5;
    if mode != BarMode::Grouped {
        return category_center;
    }
    let series_count = prepared.series.len().max(1) as f64;
    let slot = 0.74 / series_count;
    category_center - 0.74 / 2.0 + slot * (series_index.unwrap_or(0) as f64 + 0.5)
}

fn bar_geometry(
    bar: &PreparedBar,
    prepared: &PreparedBars,
    mode: BarMode,
    spacing: BarSpacing,
) -> (f64, f64) {
    let category_center = bar.category_index as f64 + 0.5;
    match mode {
        BarMode::Single | BarMode::Stacked => (category_center, 0.55),
        BarMode::Grouped => {
            let series_count = prepared.series.len().max(1) as f64;
            let slot_width = 0.74;
            let slot = slot_width / series_count;
            let series_index = bar.series_index.unwrap_or(0) as f64;
            let center = category_center - slot_width / 2.0 + slot * (series_index + 0.5);
            let height = match spacing {
                BarSpacing::Comfort => slot * 0.82,
                BarSpacing::Compact => slot,
            };
            (center, height)
        }
    }
}

fn draw_bar(
    backend: &mut dyn Backend,
    proj: &Projection,
    x0: f64,
    x1: f64,
    y_center: f64,
    height: f64,
    fill: &FillStyle,
) -> Result<(), Error> {
    let (left, bottom) = proj.project(x0, y_center - height / 2.0);
    let (right, top) = proj.project(x1, y_center + height / 2.0);
    backend.draw_polygon(
        &[
            Point2D::new(left, bottom),
            Point2D::new(right, bottom),
            Point2D::new(right, top),
            Point2D::new(left, top),
        ],
        fill,
    )
}

fn draw_error_bar(
    backend: &mut dyn Backend,
    proj: &Projection,
    lower: f64,
    upper: f64,
    y_center: f64,
    height: f64,
    style: &LineStyle,
) -> Result<(), Error> {
    let (x_left, y_px) = proj.project(lower, y_center);
    let (x_right, _) = proj.project(upper, y_center);
    let cap_px = (proj.project(0.0, height).1 - proj.project(0.0, 0.0).1).abs() * 0.2;
    backend.draw_line(
        &[Point2D::new(x_left, y_px), Point2D::new(x_right, y_px)],
        style,
    )?;
    backend.draw_line(
        &[
            Point2D::new(x_left, y_px - cap_px),
            Point2D::new(x_left, y_px + cap_px),
        ],
        style,
    )?;
    backend.draw_line(
        &[
            Point2D::new(x_right, y_px - cap_px),
            Point2D::new(x_right, y_px + cap_px),
        ],
        style,
    )
}

pub(crate) fn panel_box_for_y_tick_labels(
    chart: &Plot,
    panel: PanelBox,
    labels: &[String],
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<(PanelBox, f64), crate::layout::fit::PlanError> {
    panel_box_for_y_tick_labels_impl(chart, panel, labels, true, theme, scope)
}

pub(crate) fn panel_box_for_elided_y_tick_labels(
    chart: &Plot,
    panel: PanelBox,
    labels: &[String],
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<(PanelBox, f64), crate::layout::fit::PlanError> {
    panel_box_for_y_tick_labels_impl(chart, panel, labels, false, theme, scope)
}

fn panel_box_for_y_tick_labels_impl(
    chart: &Plot,
    mut panel: PanelBox,
    labels: &[String],
    validate_row_height: bool,
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<(PanelBox, f64), crate::layout::fit::PlanError> {
    use crate::layout::fit::FitKind;
    if labels.is_empty() {
        return Ok((panel, 1.0 / 1024.0));
    }
    let axis_scope = scope.child(crate::layout::AddressRole::AxisLeft, 0)?;
    if chart.composed_axis_text_is_measured() && validate_row_height {
        // The composed grid owns this gutter. Preserve its intrinsic preference
        // rather than treating the residual mark width as a new label allocation.
        let width =
            theme.intrinsic_tick_label_width(labels.iter().map(String::as_str), panel.height())?;
        validate_y_tick_label_rows(panel, labels, theme, width, &axis_scope)?;
        return Ok((panel, width));
    }
    let chrome = crate::layout::y_axis_left_inset_for_tick_labels(
        0.0,
        theme
            .axes
            .ticks
            .as_ref()
            .map_or(0, |appearance| appearance.length),
        theme.typography.axis_title.font_size,
        &theme.axes.spacing,
    );
    let base_left = chart.base_canvas_layout(theme).panel_box.x0;
    let minimum_body = match chart.kind() {
        PlotKind::Beeswarm => 100.0,
        PlotKind::Heatmap => 40.0,
        _ => 80.0,
    };
    let available = panel.width() - minimum_body + base_left.min(panel.x0) - chrome;
    let mut natural = 1.0 / 1024.0;
    for label in labels {
        let plan = theme.measure_text(
            label,
            &theme.typography.tick_label,
            PhysicalTextAlignment::End,
            "y-axis category label",
            false,
        )?;
        natural = f64::max(natural, plan.width().max(plan.fit_width()));
    }
    if available <= 0.0 && (!chart.axis_text_is_measured() || !validate_row_height) {
        return Err(axis_scope.failure(FitKind::TrackOrResidual, format!("y-axis labels leave no width while preserving the minimum {minimum_body}px plot panel")).into());
    }
    let width = if chart.axis_text_is_measured() && validate_row_height {
        theme.intrinsic_tick_label_width(labels.iter().map(String::as_str), panel.height())?
    } else if natural <= available || !validate_row_height {
        natural.min(available)
    } else {
        theme
            .intrinsic_tick_label_width(labels.iter().map(String::as_str), panel.height())?
            .min(available)
    };
    if validate_row_height {
        let blocks = y_tick_label_blocks(labels, theme, width, &axis_scope)?;
        validate_y_tick_label_row_height(panel, labels.len(), &blocks, &axis_scope)?;
    }
    if chart.composed_axis_text_is_measured() {
        return Ok((panel, width));
    }
    let required_left = width + chrome;
    let additional_inset = (required_left - base_left.min(panel.x0)).max(0.0);
    if additional_inset > 0.0 {
        panel.x0 = (panel.x0 + additional_inset).min(panel.x1 - minimum_body);
    }
    Ok((panel, width))
}

pub(crate) fn validate_y_tick_label_rows(
    panel: PanelBox,
    labels: &[String],
    theme: &ResolvedTheme,
    width: f64,
    scope: &crate::layout::fit::FitScope,
) -> Result<(), crate::layout::fit::PlanError> {
    if labels.is_empty() {
        return Ok(());
    }
    let blocks = y_tick_label_blocks(labels, theme, width, scope)?;
    validate_y_tick_label_row_height(panel, labels.len(), &blocks, scope)
}

fn y_tick_label_blocks(
    labels: &[String],
    theme: &ResolvedTheme,
    maximum_width: f64,
    scope: &crate::layout::fit::FitScope,
) -> Result<Vec<crate::layout::TextBlock>, crate::layout::fit::PlanError> {
    crate::layout::fit::collect(
        theme,
        "y-axis category labels",
        labels.iter().map(|label| {
            crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                label,
                &theme.typography.tick_label,
                maximum_width,
                PhysicalTextAlignment::End,
                "y-axis category label",
                scope,
            ))
            .map(TextBlock::from_resolved)
        }),
    )
}

fn validate_y_tick_label_row_height(
    panel: PanelBox,
    label_count: usize,
    blocks: &[crate::layout::TextBlock],
    scope: &crate::layout::fit::FitScope,
) -> Result<(), crate::layout::fit::PlanError> {
    let row_height = panel.height() / label_count.max(1) as f64;
    if blocks.iter().any(|block| block.height() > row_height) {
        return Err(scope.failure(crate::layout::fit::FitKind::Tick,
            "multiline y-axis category labels do not fit their rows; increase chart height or shorten labels"
        ).into());
    }
    Ok(())
}

pub(crate) fn panel_box_for_x_tick_labels(
    chart: &Plot,
    mut panel: PanelBox,
    labels: &[String],
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<PanelBox, crate::layout::fit::PlanError> {
    if labels.is_empty() || chart.axis_text_is_measured() {
        return Ok(panel);
    }
    let display =
        chart.resolve_label_domain("x-axis categories", labels.iter().map(String::as_str))?;
    let maximum_width = panel.width() / labels.len() as f64;
    let axis_scope = scope.child(crate::layout::AddressRole::AxisBottom, 0)?;
    let maximum_height = crate::layout::fit::collect(
        theme,
        "x-axis category labels",
        display.iter().map(|label| {
            crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                label,
                &theme.typography.tick_label,
                maximum_width.max(1.0),
                PhysicalTextAlignment::Center,
                "x-axis category label",
                &axis_scope,
            ))
            .map(TextBlock::from_resolved)
            .map(|block| block.height())
        }),
    )?
    .into_iter()
    .fold(0.0_f64, f64::max);
    let extra = (maximum_height - theme.typography.tick_label.line_height()).max(0.0);
    if panel.height() - extra < 60.0 {
        return Err(axis_scope.failure(crate::layout::fit::FitKind::TrackOrResidual,
            "multiline x-axis category labels do not fit while preserving the minimum 60px plot panel; increase chart height"
        ).into());
    }
    panel.y0 += extra;
    Ok(panel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferred_y_width_uses_available_room_and_retains_compact_wrapping() {
        let plot = Plot::horizontal_bar(
            crate::CategoryValues::new("category", ["Control group"]).unwrap(),
            crate::BarInput::estimates(crate::NumericValues::new("value", [1.0])),
        )
        .erase();
        let theme = crate::Theme::Publication.resolve();
        theme.prepare_font_environment().unwrap();
        let labels = ["Control group".to_owned()];
        let natural = theme
            .measure_text(
                &labels[0],
                &theme.typography.tick_label,
                PhysicalTextAlignment::End,
                "y-axis category label",
                false,
            )
            .unwrap();
        let scope = crate::layout::fit::FitScope::root();
        let (_, wide) = panel_box_for_y_tick_labels(
            &plot,
            PanelBox::new(80.0, 600.0, 80.0, 400.0),
            &labels,
            &theme,
            &scope,
        )
        .unwrap();
        assert_eq!(
            wide.to_bits(),
            natural.width().max(natural.fit_width()).to_bits()
        );
        let (panel, compact) = panel_box_for_y_tick_labels(
            &plot,
            PanelBox::new(80.0, 230.0, 80.0, 400.0),
            &labels,
            &theme,
            &scope,
        )
        .unwrap();
        assert!(compact < wide);
        assert!(panel.width() >= 80.0);
        let blocks = y_tick_label_blocks(&labels, &theme, compact, &scope).unwrap();
        assert!(blocks[0].lines().len() > 1);
        assert_eq!(blocks[0].plan().source, labels[0]);
    }
}
