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

use crate::decorations::legend::LegendEntry;
use crate::layout::{AxisTick, Layout};
use crate::render::backend::{with_clip, Backend, FillStyle, LineStyle, Point2D};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::stats::distribution::quantile;
use crate::{Chart, RawOverlay};

#[derive(Debug, PartialEq)]
struct BoxSummary {
    lo_whisker: f64,
    q1: f64,
    median: f64,
    q3: f64,
    hi_whisker: f64,
    fliers: Vec<f64>,
}

struct BoxStats {
    x_index: usize,
    group_index: usize,
    hue_index: Option<usize>,
    lo_whisker: f64,
    q1: f64,
    median: f64,
    q3: f64,
    hi_whisker: f64,
    fliers: Vec<f64>,
}

/// Render a boxplot: per-group quartiles with whiskers, grouped
/// by `x_col`. Returns the `Layout` used to project the boxes
/// so the orchestrator can apply decorations (frame / grid /
/// axes / legend). Categorical x-ticks carry the group labels.
pub(crate) fn render(
    chart: &Chart<'_>,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    let palette = chart.palette();

    let sample = crate::plots::distribution_sample::prepare(chart, context.data, "boxplot")?;
    let x_levels = &sample.x_levels;
    let group_levels = &sample.group_levels;
    let hue_levels = &sample.hue_levels;

    // Display stats per group: Q1, median, Q3, lower whisker, upper whisker.
    let stats: Vec<BoxStats> = sample
        .groups
        .iter()
        .map(|group| -> Result<BoxStats, Error> {
            let mut ys = group.values();
            let summary = box_summary(&mut ys)?;
            Ok(BoxStats {
                x_index: group.x_index,
                group_index: group.group_index,
                hue_index: group.hue_index,
                lo_whisker: summary.lo_whisker,
                q1: summary.q1,
                median: summary.median,
                q3: summary.q3,
                hi_whisker: summary.hi_whisker,
                fliers: summary.fliers,
            })
        })
        .collect::<Result<_, _>>()?;

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

    let group_active = !group_levels.is_empty();
    let n_boxes = x_levels.len() as f64;
    let x_ticks: Vec<AxisTick> = x_levels
        .iter()
        .enumerate()
        .map(|(i, label)| AxisTick::new(i as f64 + 0.5, label.clone()))
        .collect();
    let base_y_range = crate::layout::continuous_auto_range((y_min, y_max), chart.y_axis_spec());
    let mut rendered_maxima = vec![f64::NEG_INFINITY; x_levels.len()];
    let mut grouped_maxima =
        vec![vec![f64::NEG_INFINITY; group_levels.len().max(1)]; x_levels.len()];
    let mut mark_hues = vec![vec![None; group_levels.len().max(1)]; x_levels.len()];
    for stat in &stats {
        let maximum = stat.fliers.iter().copied().fold(stat.hi_whisker, f64::max);
        rendered_maxima[stat.x_index] = rendered_maxima[stat.x_index].max(maximum);
        grouped_maxima[stat.x_index][stat.group_index] = maximum;
        mark_hues[stat.x_index][stat.group_index] =
            stat.hue_index.map(|index| hue_levels[index].clone());
    }
    let overlay = chart.raw_overlay_spec();
    let panel = context.panel;
    let panel = crate::plots::horizontal_bar::panel_box_for_x_tick_labels(chart, panel, x_levels)?;
    let legend_entries: Vec<LegendEntry> = hue_levels
        .iter()
        .enumerate()
        .map(|(idx, label)| {
            LegendEntry::point(
                label.clone(),
                chart.point_style(palette.colors[idx % palette.colors.len()], 3),
            )
        })
        .collect();
    let significance = if group_levels.is_empty() {
        crate::significance::plan(chart, x_levels, &rendered_maxima, base_y_range, panel)?
    } else {
        crate::significance::plan_grouped(
            chart,
            x_levels,
            group_levels,
            &mark_hues,
            &grouped_maxima,
            0.72,
            base_y_range,
            panel,
            false,
        )?
    };
    let y_range = significance.y_range;
    let y_axis_spec = crate::layout::display_axis_spec(chart.y_axis_spec(), y_range);
    let layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
        (0.0, n_boxes),
        y_range,
        panel,
        chart.x_label_text().unwrap_or("").to_string(),
        chart.y_label_text().unwrap_or("").to_string(),
        x_ticks,
        Vec::new(),
        chart.x_axis_spec(),
        &y_axis_spec,
    );
    let proj = &layout.projection;
    let box_w_data = if group_active {
        (0.72 / group_levels.len() as f64).min(0.32)
    } else {
        0.5
    };

    with_clip(backend, layout.projection.panel_box(), |backend| {
        for (i, stat) in stats.iter().enumerate() {
            let color_index = stat.hue_index.unwrap_or(if group_active { 0 } else { i });
            let color = palette.colors[color_index % palette.colors.len()];
            let fill = FillStyle {
                color,
                stroke: palette.foreground,
                stroke_width: 1.0,
                opacity: 1.0,
            };
            let style = LineStyle {
                color: palette.foreground,
                width: palette.axis_width,
                dash: String::new(),
            };

            let x_center_data =
                crate::plots::distribution_sample::center(&sample.groups[i], group_levels.len());
            let (xc, _) = proj.project(x_center_data, 0.0);
            let box_w_px = (proj.project(box_w_data, 0.0).0 - proj.project(0.0, 0.0).0).abs();
            let (_, y_lo) = proj.project(x_center_data, stat.lo_whisker);
            let (_, y_q1) = proj.project(x_center_data, stat.q1);
            let (_, y_med) = proj.project(x_center_data, stat.median);
            let (_, y_q3) = proj.project(x_center_data, stat.q3);
            let (_, y_hi) = proj.project(x_center_data, stat.hi_whisker);

            // Box.
            let pts = vec![
                Point2D::new(xc - box_w_px / 2.0, y_q1),
                Point2D::new(xc - box_w_px / 2.0, y_q3),
                Point2D::new(xc + box_w_px / 2.0, y_q3),
                Point2D::new(xc + box_w_px / 2.0, y_q1),
            ];
            backend.draw_polygon(&pts, &fill)?;

            // Median line.
            backend.draw_line(
                &[
                    Point2D::new(xc - box_w_px / 2.0, y_med),
                    Point2D::new(xc + box_w_px / 2.0, y_med),
                ],
                &style,
            )?;

            // Whiskers.
            backend.draw_line(&[Point2D::new(xc, y_q3), Point2D::new(xc, y_hi)], &style)?;
            backend.draw_line(&[Point2D::new(xc, y_q1), Point2D::new(xc, y_lo)], &style)?;
            let cap = box_w_px * 0.3;
            backend.draw_line(
                &[Point2D::new(xc - cap, y_hi), Point2D::new(xc + cap, y_hi)],
                &style,
            )?;
            backend.draw_line(
                &[Point2D::new(xc - cap, y_lo), Point2D::new(xc + cap, y_lo)],
                &style,
            )?;

            if overlay == RawOverlay::Hidden {
                let flier_style = chart.point_style(palette.foreground, 2);
                let flier_points: Vec<Point2D> = stat
                    .fliers
                    .iter()
                    .map(|y| {
                        let (x_px, y_px) = proj.project(x_center_data, *y);
                        Point2D::new(x_px, y_px)
                    })
                    .collect();
                if !flier_points.is_empty() {
                    backend.draw_scatter(&flier_points, &flier_style)?;
                }
            }
        }
        if overlay.is_visible() {
            for (index, group) in sample.groups.iter().enumerate() {
                let center = crate::plots::distribution_sample::center(group, group_levels.len());
                let color_index = group
                    .hue_index
                    .unwrap_or(if group_active { 0 } else { index });
                let points = group
                    .observations
                    .iter()
                    .map(|observation| {
                        let x = crate::plots::distribution_sample::overlay_x(
                            overlay,
                            center,
                            box_w_data,
                            observation.source_row,
                        );
                        let (x, y) = proj.project(x, observation.value);
                        Point2D::new(x, y)
                    })
                    .collect::<Vec<_>>();
                backend.draw_scatter(
                    &points,
                    &crate::plots::distribution_sample::overlay_style(
                        palette.colors[color_index % palette.colors.len()],
                    ),
                )?;
            }
        }
        Ok(())
    })?;

    if hue_levels.is_empty() {
        Ok(PlotRenderResult::new(layout, Vec::new())
            .with_significance_brackets(significance.brackets))
    } else {
        Ok(PlotRenderResult::new(layout, legend_entries)
            .with_significance_brackets(significance.brackets))
    }
}

fn box_summary(values: &mut [f64]) -> Result<BoxSummary, Error> {
    let q1 = quantile(values, 0.25)?;
    let median = quantile(values, 0.5)?;
    let q3 = quantile(values, 0.75)?;
    values.sort_by(f64::total_cmp);
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
    let fliers = values
        .iter()
        .copied()
        .filter(|value| *value < lo_fence || *value > hi_fence)
        .collect();
    Ok(BoxSummary {
        lo_whisker,
        q1,
        median,
        q3,
        hi_whisker,
        fliers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_point_box_summary_uses_type_7_quartile_geometry() {
        let mut values = [0.0, 4.0];

        assert_eq!(
            box_summary(&mut values).unwrap(),
            BoxSummary {
                lo_whisker: 0.0,
                q1: 1.0,
                median: 2.0,
                q3: 3.0,
                hi_whisker: 4.0,
                fliers: vec![],
            }
        );
    }

    #[test]
    fn singleton_box_summary_preserves_type_7_geometry() {
        let mut values = [4.0];

        assert_eq!(
            box_summary(&mut values).unwrap(),
            BoxSummary {
                lo_whisker: 4.0,
                q1: 4.0,
                median: 4.0,
                q3: 4.0,
                hi_whisker: 4.0,
                fliers: vec![],
            }
        );
    }

    #[test]
    fn repeated_values_preserve_inclusive_whiskers_and_outlier() {
        let mut values = [1.0, 1.0, 1.0, 9.0];

        assert_eq!(
            box_summary(&mut values).unwrap(),
            BoxSummary {
                lo_whisker: 1.0,
                q1: 1.0,
                median: 1.0,
                q3: 3.0,
                hi_whisker: 1.0,
                fliers: vec![9.0],
            }
        );
    }

    #[test]
    fn observations_equal_to_fences_remain_inclusive_whiskers() {
        let mut values = [-2.0, 1.0, 2.0, 3.0, 6.0];

        let summary = box_summary(&mut values).unwrap();

        assert_eq!(summary.lo_whisker.to_bits(), (-2.0_f64).to_bits());
        assert_eq!(summary.hi_whisker.to_bits(), 6.0_f64.to_bits());
        assert!(summary.fliers.is_empty());
    }

    #[test]
    fn box_summary_quartiles_agree_with_shared_quantile() {
        let mut values = [9.0, 1.0, 5.0, 3.0, 7.0, 11.0];
        let original = values;

        let summary = box_summary(&mut values).unwrap();

        assert_eq!(
            [summary.q1, summary.median, summary.q3].map(f64::to_bits),
            [
                quantile(&original, 0.25).unwrap(),
                quantile(&original, 0.5).unwrap(),
                quantile(&original, 0.75).unwrap(),
            ]
            .map(f64::to_bits)
        );
    }

    #[test]
    fn box_summary_propagates_shared_quantile_errors() {
        let mut values = [1.0, f64::NAN];

        let error = box_summary(&mut values).unwrap_err();

        assert!(error.to_string().contains("must be finite"));
    }
}
