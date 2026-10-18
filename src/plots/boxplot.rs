//! `lichtung-plot::plots::boxplot` — box-and-whisker plot.
//!
//! Groups the data by `x_col` (categorical) and draws a box per
//! group: Q1..Q3 with median line, whiskers at 1.5*IQR.
//! This is display geometry only and performs no inferential logic.
//!
//! Explicit pairwise comparisons are rendered as stacked brackets above their
//! named categorical groups. Omnibus results are not interpreted as pairs.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]

use crate::Error;

use crate::data::PlotData;
use crate::plots::categorical_distribution::{self, DistributionPayload, PreparedDistribution};
use crate::render::backend::Point2D;
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::{Plot, RawOverlay};

#[derive(Debug, PartialEq)]
struct BoxSummary {
    lo_whisker: f64,
    q1: f64,
    median: f64,
    q3: f64,
    hi_whisker: f64,
    fliers: Vec<f64>,
}

#[derive(Debug)]
struct BoxStats {
    x_index: usize,
    group_index: usize,
    lo_whisker: f64,
    q1: f64,
    median: f64,
    q3: f64,
    hi_whisker: f64,
    fliers: Vec<f64>,
}

#[derive(Debug)]
pub(crate) struct PreparedBoxplot {
    pub(crate) sample: crate::plots::distribution_sample::DistributionSample,
    stats: Vec<BoxStats>,
    pub(crate) range: (f64, f64),
}

pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let PreparedDistribution::Boxplot(prepared) = categorical_distribution::retained(chart)? else {
        return Err(Error::Render(
            "Boxplot intrinsic requires prepared summaries".into(),
        ));
    };
    let sample = &prepared.sample;
    if sample.x_levels.is_empty() || prepared.stats.len() != sample.groups.len() {
        return Err(Error::Render(
            "Boxplot summary and populated groups differ".into(),
        ));
    }
    let group_count = sample.group_levels.len();
    let thickness = if group_count > 0 {
        (0.72 / group_count as f64).min(0.32)
    } else {
        0.5
    };
    let fill = theme
        .marks
        .fills()
        .bodies()
        .box_body()
        .backend_style((0, 0, 0));
    let body = crate::render::geometry::rectangular_fill_ink_bounds(&fill)?;
    let (_, out_y) = crate::render::geometry::rectangular_fill_ink_outset(&fill)?;
    let summary = crate::render::geometry::line_ink_width(
        &theme.marks.strokes().summaries().summary().backend_style(),
    )?;
    let overlay = chart.raw_overlay_spec();
    let point_style = if overlay.is_visible() {
        chart.distribution_point_style(theme.marks.points().raw_overlay(), (0, 0, 0))
    } else {
        chart.point_style(
            theme,
            theme.marks.neutral_paints().box_flier().as_rgb(),
            theme.marks.points().radii().box_flier(),
        )
    };
    let point = crate::render::geometry::point_ink_bounds(&point_style)?;

    let mut pitch = 1.0_f64 / 1024.0;
    let mut numeric = out_y.max(summary);
    for (stat, group) in prepared.stats.iter().zip(&sample.groups) {
        if stat.x_index != group.x_index
            || stat.group_index != group.group_index
            || group.x_index >= sample.x_levels.len()
        {
            return Err(Error::Render(
                "Boxplot summary is outside its prepared category slot".into(),
            ));
        }
        let center = crate::plots::distribution_sample::center(group, group_count);
        let offset = center - (group.x_index as f64 + 0.5);
        let half = thickness / 2.0;
        let left_room = 0.5 + offset - half;
        let right_room = 0.5 - offset - half;
        if left_room <= 0.0 || right_room <= 0.0 {
            return Err(Error::Render(
                "Boxplot body exceeds prepared category slot".into(),
            ));
        }
        pitch = pitch.max((1.0 / 1024.0) / thickness);
        if let Some(ink) = body {
            pitch = pitch.max((-ink.left).max(0.0) / left_room);
            pitch = pitch.max(ink.right.max(0.0) / right_room);
        }
        if summary > 0.0 {
            // The 0.3-box-width whisker cap is narrower than the filled box.
            let cap_room = 0.3 * thickness;
            pitch = pitch.max((summary / 2.0) / (0.5 + offset - cap_room));
            pitch = pitch.max((summary / 2.0) / (0.5 - offset - cap_room));
        }
        if let Some(ink) = point {
            if overlay.is_visible() {
                for observation in &group.observations {
                    let x = crate::plots::distribution_sample::overlay_x(
                        overlay,
                        center,
                        thickness,
                        observation.source_row,
                    );
                    let shift = x - (group.x_index as f64 + 0.5);
                    pitch = pitch.max((-ink.left).max(0.0) / (0.5 + shift));
                    pitch = pitch.max(ink.right.max(0.0) / (0.5 - shift));
                }
                numeric = numeric.max(ink.top - ink.bottom);
            } else if !stat.fliers.is_empty() {
                pitch = pitch.max((-ink.left).max(0.0) / (0.5 + offset));
                pitch = pitch.max(ink.right.max(0.0) / (0.5 - offset));
                numeric = numeric.max(ink.top - ink.bottom);
            }
        }
    }
    let count = u32::try_from(sample.x_levels.len())
        .map_err(|_| Error::Render("Boxplot category count exceeds coordinate range".into()))?;
    let category_span = pitch * f64::from(count);
    if !category_span.is_finite() {
        return Err(Error::Render(
            "Boxplot category hard span overflowed".into(),
        ));
    }
    Ok((
        category_span,
        if chart.user_y_limits().is_none() {
            numeric
        } else {
            0.0
        },
    ))
}

pub(crate) fn prepare(
    chart: &Plot,
    data: PlotData<'_>,
    facet: Option<crate::data::FacetAxisSources<'_>>,
) -> Result<PreparedBoxplot, Error> {
    let sample = crate::plots::distribution_sample::prepare_scoped(chart, data, "boxplot", facet)?;

    // Display stats per group: Q1, median, Q3, lower whisker, upper whisker.
    let stats = crate::data::source::collect_source(sample.groups.iter().map(
        |group| -> Result<BoxStats, Error> {
            let mut ys = crate::data::source::collect_source(
                group.observations.iter().map(|o| Ok(o.value)),
            )?;
            let summary = box_summary_with_source(&mut ys)?;
            Ok(BoxStats {
                x_index: group.x_index,
                group_index: group.group_index,
                lo_whisker: summary.lo_whisker,
                q1: summary.q1,
                median: summary.median,
                q3: summary.q3,
                hi_whisker: summary.hi_whisker,
                fliers: summary.fliers,
            })
        },
    ))?;

    let y_min = stats.iter().fold(f64::INFINITY, |min, stat| {
        stat.fliers
            .iter()
            .copied()
            .fold(min.min(stat.lo_whisker), f64::min)
    });
    let y_max = stats.iter().fold(f64::NEG_INFINITY, |max, stat| {
        stat.fliers
            .iter()
            .copied()
            .fold(max.max(stat.hi_whisker), f64::max)
    });

    Ok(PreparedBoxplot {
        sample,
        stats,
        range: (y_min, y_max),
    })
}

pub(crate) fn measure_marks(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    layout: crate::layout::Layout,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    categorical_distribution::count(chart.kind(), 1);
    let PreparedDistribution::Boxplot(prepared) = categorical_distribution::retained(chart)? else {
        return Err(Error::Render("boxplot requires retained box summaries".into()).into());
    };
    let theme = context.theme;
    let resources = theme.prepared_font_environment()?.operation();
    let data = context.data.with_operation(&resources);
    let sample = &prepared.sample;
    let stats = &prepared.stats;
    let x_levels = &sample.x_levels;
    let group_levels = &sample.group_levels;
    let group_active = !group_levels.is_empty();
    let base_y_range = layout.projection.y_range();
    let mut rendered_maxima =
        crate::data::source::collect_source((0..x_levels.len()).map(|_| Ok(f64::NEG_INFINITY)))?;
    let mut grouped_maxima = crate::data::source::collect_source((0..x_levels.len()).map(|_| {
        crate::data::source::collect_source(
            (0..group_levels.len().max(1)).map(|_| Ok(f64::NEG_INFINITY)),
        )
    }))?;
    for stat in stats {
        let maximum = stat.fliers.iter().copied().fold(stat.hi_whisker, f64::max);
        rendered_maxima[stat.x_index] = rendered_maxima[stat.x_index].max(maximum);
        grouped_maxima[stat.x_index][stat.group_index] = maximum;
    }
    let overlay = chart.raw_overlay_spec();
    let panel = context.panel;
    let colors = {
        let retained = categorical_distribution::retained(chart)?;
        retained.resolved_colors(chart, theme, data)?
    };
    let significance = if group_levels.is_empty() {
        crate::significance::plan(
            chart,
            x_levels,
            &rendered_maxima,
            base_y_range,
            panel,
            theme,
            context.fit_scope(),
        )?
    } else {
        crate::significance::plan_grouped(
            chart,
            x_levels,
            group_levels,
            &grouped_maxima,
            0.72,
            base_y_range,
            panel,
            false,
            theme,
            context.fit_scope(),
        )?
    };
    let proj = &layout.projection;
    let box_w_data = if group_active {
        (0.72 / group_levels.len() as f64).min(0.32)
    } else {
        0.5
    };

    let mut payload = DistributionPayload::new(layout.projection.panel_box(), chart.kind());
    for (i, stat) in stats.iter().enumerate() {
        let group = &sample.groups[i];
        let color = colors.color(group.hue_label.as_deref().unwrap_or(&group.label))?;
        let fill = theme.marks.fills().bodies().box_body().backend_style(color);
        let style = theme.marks.strokes().summaries().summary().backend_style();

        let x_center_data =
            crate::plots::distribution_sample::center(&sample.groups[i], group_levels.len());
        let (xc, _) = proj.project(x_center_data, 0.0);
        let box_w_px = (proj.project(box_w_data, 0.0).0 - proj.project(0.0, 0.0).0).abs();
        let (_, y_lo) = proj.project(x_center_data, stat.lo_whisker);
        let (_, y_q1) = proj.project(x_center_data, stat.q1);
        let (_, y_med) = proj.project(x_center_data, stat.median);
        let (_, y_q3) = proj.project(x_center_data, stat.q3);
        let (_, y_hi) = proj.project(x_center_data, stat.hi_whisker);
        for (source, projected, context) in [
            ((stat.q1, stat.q3), (y_q1, y_q3), "boxplot polygon"),
            (
                (stat.lo_whisker, stat.q1),
                (y_lo, y_q1),
                "boxplot lower whisker",
            ),
            (
                (stat.q3, stat.hi_whisker),
                (y_q3, y_hi),
                "boxplot upper whisker",
            ),
        ] {
            proj.validate_span(
                crate::layout::AxisSide::Left,
                source,
                projected,
                source.0 < source.1,
                context,
            )?;
        }

        // Box.
        let pts = crate::data::source::collect_source(
            [
                Point2D::new(xc - box_w_px / 2.0, y_q1),
                Point2D::new(xc - box_w_px / 2.0, y_q3),
                Point2D::new(xc + box_w_px / 2.0, y_q3),
                Point2D::new(xc + box_w_px / 2.0, y_q1),
            ]
            .into_iter()
            .map(Ok),
        )?;
        payload.polygon(pts, fill)?;

        // Median line.
        payload.line(
            &[
                Point2D::new(xc - box_w_px / 2.0, y_med),
                Point2D::new(xc + box_w_px / 2.0, y_med),
            ],
            &style,
        )?;

        // Whiskers.
        payload.line(&[Point2D::new(xc, y_q3), Point2D::new(xc, y_hi)], &style)?;
        payload.line(&[Point2D::new(xc, y_q1), Point2D::new(xc, y_lo)], &style)?;
        let cap = box_w_px * 0.3;
        payload.line(
            &[Point2D::new(xc - cap, y_hi), Point2D::new(xc + cap, y_hi)],
            &style,
        )?;
        payload.line(
            &[Point2D::new(xc - cap, y_lo), Point2D::new(xc + cap, y_lo)],
            &style,
        )?;

        if overlay == RawOverlay::Hidden {
            let flier_style = chart.point_style(
                theme,
                theme.marks.neutral_paints().box_flier().as_rgb(),
                theme.marks.points().radii().box_flier(),
            );
            let flier_points = crate::data::source::collect_source(stat.fliers.iter().map(|y| {
                let (x_px, y_px) = proj.project(x_center_data, *y);
                Ok(Point2D::new(x_px, y_px))
            }))?;
            if !flier_points.is_empty() {
                payload.scatter(flier_points, flier_style)?;
            }
        }
    }
    if overlay.is_visible() {
        for group in &sample.groups {
            let center = crate::plots::distribution_sample::center(group, group_levels.len());
            let points = crate::data::source::collect_source(group.observations.iter().map(
                |observation| {
                    let x = crate::plots::distribution_sample::overlay_x(
                        overlay,
                        center,
                        box_w_data,
                        observation.source_row,
                    );
                    let (x, y) = proj.project(x, observation.value);
                    Ok(Point2D::new(x, y))
                },
            ))?;
            payload.scatter(
                points,
                chart.distribution_point_style(
                    theme.marks.points().raw_overlay(),
                    colors.color(group.hue_label.as_deref().unwrap_or(&group.label))?,
                ),
            )?;
        }
    }
    payload.retain(
        PlotRenderResult::new(layout, chart.prepared_discrete_guide()?)
            .with_significance_plan(significance),
        context,
        chart,
    )
}

fn box_summary_with_source(values: &mut [f64]) -> Result<BoxSummary, Error> {
    if values.is_empty() || values.iter().any(|v| !v.is_finite()) {
        return Err(Error::Data(
            "boxplot: samples must be finite and nonempty".into(),
        ));
    }
    crate::data::source::sort_numeric_source(values);
    let q1 = crate::stats::distribution::sorted_quantile(values, 0.25);
    let median = crate::stats::distribution::sorted_quantile(values, 0.5);
    let q3 = crate::stats::distribution::sorted_quantile(values, 0.75);
    let iqr = q3 - q1;
    let lo_fence = q1 - 1.5 * iqr;
    let hi_fence = q3 + 1.5 * iqr;

    let lo_whisker = values
        .iter()
        .copied()
        .find(|value| *value >= lo_fence)
        .unwrap_or(q1);
    let hi_whisker = values
        .iter()
        .rev()
        .copied()
        .find(|value| *value <= hi_fence)
        .unwrap_or(q3);
    let mut fliers = Vec::new();
    crate::data::source::reserve_source(&mut fliers, values.len())?;
    fliers.extend(
        values
            .iter()
            .copied()
            .filter(|value| *value < lo_fence || *value > hi_fence),
    );
    Ok(BoxSummary {
        lo_whisker,
        q1,
        median,
        q3,
        hi_whisker,
        fliers,
    })
}
