//! `lichtung-plot::plots::violin` — violin plot (KDE + box overlay).
//!
//! Groups the data by `x_col` (categorical) and draws a violin
//! per group: a kernel density estimate (KDE) curve mirrored
//! around the category's vertical axis, with the existing
//! boxplot's Q1/median/Q3 box overlaid on top.
//!
//! KDE and quantiles are display geometry; the plot performs no inferential
//! test.
//! The KDE is computed via [`kde`]; the quantiles
//! (Q1, median, Q3, whiskers) are computed via the same module's
//! [`quantile`].
//!
//! Explicit pairwise comparisons are rendered as stacked brackets above their
//! named categorical groups. Omnibus results are not interpreted as pairs.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::redundant_closure_for_method_calls
)]

mod precomputed;

use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::layout::{AxisTick, Layout};
use crate::render::backend::{with_clip, Backend, FillStyle, LineStyle, Point2D};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::stats::distribution::{kde, kde_support_range, quantile};
use crate::theme::ResolvedTheme;
use crate::Plot;

#[derive(Debug, PartialEq)]
struct ViolinSummary {
    q1: f64,
    median: f64,
    q3: f64,
    lo_whisker: f64,
    hi_whisker: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DistributionLayout {
    Mirrored,
    Raincloud,
}

/// Render a violin plot. Returns the `Layout` used to project
/// the violins so the orchestrator can apply decorations (frame
/// / grid / axes / legend). Categorical x-ticks carry the
/// group labels.
pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    let theme = context.theme;
    render_samples(chart, context, theme, backend, DistributionLayout::Mirrored)
}

pub(crate) fn render_raincloud(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    let theme = context.theme;
    if let Some(input) = chart.raincloud_geometry_input() {
        return precomputed::render(chart, input, context, backend);
    }
    render_samples(
        chart,
        context,
        theme,
        backend,
        DistributionLayout::Raincloud,
    )
}

fn render_samples(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
    distribution_layout: DistributionLayout,
) -> Result<PlotRenderResult, Error> {
    let family = match distribution_layout {
        DistributionLayout::Mirrored => "violin",
        DistributionLayout::Raincloud => "raincloud",
    };
    let sample = crate::plots::distribution_sample::prepare(chart, context.data, family)?;
    let x_levels = &sample.x_levels;
    let group_levels = &sample.group_levels;
    let hue_levels = &sample.hue_levels;
    if distribution_layout == DistributionLayout::Raincloud {
        let mut source_rows = std::collections::HashSet::new();
        for observation in sample.groups.iter().flat_map(|group| &group.observations) {
            if !source_rows.insert(observation.source_row) {
                return Err(Error::Data(format!(
                    "raincloud: duplicate source-row identity {}",
                    observation.source_row
                )));
            }
        }
    }

    let (data_y_min, data_y_max) = sample
        .groups
        .iter()
        .flat_map(|group| {
            group
                .observations
                .iter()
                .map(|observation| observation.value)
        })
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
            (lo.min(v), hi.max(v))
        });
    if data_y_min >= data_y_max {
        return Err(Error::Data(format!(
            "{family}: data has zero variance (all y values are equal)"
        )));
    }

    let extent_groups: Vec<(String, Vec<f64>)> = sample
        .groups
        .iter()
        .map(|group| {
            (
                group.hue_label.as_ref().map_or_else(
                    || group.label.clone(),
                    |hue| format!("{}/{hue}", group.label),
                ),
                group.values(),
            )
        })
        .collect();
    let support_range = kde_support_range(
        extent_groups.iter().map(|(_, values)| values.as_slice()),
        2.0,
    )
    .ok_or_else(|| Error::Data(format!("{family}: no finite KDE support")))?;
    let base_display_range =
        crate::layout::continuous_auto_range(support_range, chart.y_axis_spec());
    let rendered_maxima = vec![support_range.1; x_levels.len()];
    let overlay = chart.raw_overlay_spec();
    let panel = context.panel;
    let panel =
        crate::plots::horizontal_bar::panel_box_for_x_tick_labels(chart, panel, x_levels, theme)?;
    let legend_entries = hue_levels
        .iter()
        .enumerate()
        .map(|(idx, label)| {
            let color = chart.categorical_color(family, label, idx, theme)?;
            let identity = chart.legend_identity(label.clone());
            let fill = FillStyle {
                color,
                stroke: theme.foreground,
                stroke_width: 1.0,
                opacity: 0.70,
            };
            Ok(if distribution_layout == DistributionLayout::Raincloud {
                LegendEntry::point_fill(
                    identity,
                    crate::plots::distribution_sample::overlay_style(color),
                    fill,
                )
            } else {
                LegendEntry::fill(identity, fill)
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let significance = if group_levels.is_empty() {
        crate::significance::plan(
            chart,
            x_levels,
            &rendered_maxima,
            base_display_range,
            panel,
            theme,
        )?
    } else {
        let mut grouped_maxima = vec![vec![f64::NEG_INFINITY; group_levels.len()]; x_levels.len()];
        for group in &sample.groups {
            grouped_maxima[group.x_index][group.group_index] = support_range.1;
        }
        crate::significance::plan_grouped(
            chart,
            x_levels,
            group_levels,
            &grouped_maxima,
            0.72,
            base_display_range,
            panel,
            false,
            theme,
        )?
    };
    let display_range = significance.y_range;
    let y_axis_spec = crate::layout::display_axis_spec(chart.y_axis_spec(), display_range);

    let group_active = !group_levels.is_empty();
    let n_groups = x_levels.len();
    let x_ticks: Vec<AxisTick> = x_levels
        .iter()
        .enumerate()
        .map(|(i, label)| AxisTick::new(i as f64 + 0.5, label.clone()))
        .collect();
    let layout = Layout::new_with_ticks_and_axis_specs_in_panel_box(
        (0.0, n_groups as f64),
        display_range,
        panel,
        chart.x_label_text().unwrap_or("").to_string(),
        chart.y_label_text().unwrap_or("").to_string(),
        x_ticks,
        Vec::new(),
        chart.x_axis_spec(),
        &y_axis_spec,
        theme,
    );
    let proj = &layout.projection;

    let n_grid: usize = 64;
    let (support_low, support_high) = support_range;

    with_clip(backend, layout.projection.panel_box(), |backend| {
        for (i, group) in sample.groups.iter().enumerate() {
            let ys = group.values();
            let summary = violin_summary(&ys)?;

            let grid: Vec<f64> = (0..n_grid)
                .map(|k| {
                    support_low + (support_high - support_low) * (k as f64) / ((n_grid - 1) as f64)
                })
                .collect();
            let mut densities = kde(&ys, &grid)?;
            if let Some(first) = densities.first_mut() {
                *first = 0.0;
            }
            if let Some(last) = densities.last_mut() {
                *last = 0.0;
            }
            let max_d = densities.iter().fold(0.0_f64, |a, b| a.max(*b));
            let max_d = if max_d <= 0.0 { 1.0 } else { max_d };

            let center_x_data =
                crate::plots::distribution_sample::center(group, group_levels.len());
            let violin_w_data = if group_active {
                (0.72 / group_levels.len() as f64).min(0.34)
            } else {
                0.8
            };

            let fill = FillStyle {
                color: group_color(chart, group, i, group_active, family, theme)?,
                stroke: theme.foreground,
                stroke_width: 1.0,
                opacity: 0.70,
            };
            match distribution_layout {
                DistributionLayout::Mirrored => {
                    // Violin shape
                    let mut pts: Vec<Point2D> = Vec::with_capacity(2 * n_grid + 1);
                    for (gy, d) in grid.iter().zip(densities.iter()) {
                        let x_off = (*d / max_d) * (violin_w_data / 2.0);
                        let (px, py) = proj.project(center_x_data + x_off, *gy);
                        pts.push(Point2D::new(px, py));
                    }
                    for (gy, d) in grid.iter().zip(densities.iter()).rev() {
                        let x_off = (*d / max_d) * (violin_w_data / 2.0);
                        let (px, py) = proj.project(center_x_data - x_off, *gy);
                        pts.push(Point2D::new(px, py));
                    }
                    backend.draw_polygon(&pts, &fill)?;

                    // Inner box-and-whisker mark, matching seaborn's default inner="box" cue.
                    let (cx_px, _) = proj.project(center_x_data, 0.0);
                    let box_w_px = (proj.project(violin_w_data * 0.08, 0.0).0
                        - proj.project(0.0, 0.0).0)
                        .abs();
                    let cap_w_px = box_w_px;
                    let (_, lo_whisker_px) = proj.project(center_x_data, summary.lo_whisker);
                    let (_, q1_px) = proj.project(center_x_data, summary.q1);
                    let (_, q3_px) = proj.project(center_x_data, summary.q3);
                    let (_, hi_whisker_px) = proj.project(center_x_data, summary.hi_whisker);
                    let (_, median_px) = proj.project(center_x_data, summary.median);
                    let whisker_line = LineStyle::solid(theme.foreground, theme.interval_width);
                    backend.draw_line(
                        &[
                            Point2D::new(cx_px, lo_whisker_px),
                            Point2D::new(cx_px, hi_whisker_px),
                        ],
                        &whisker_line,
                    )?;
                    backend.draw_line(
                        &[
                            Point2D::new(cx_px - cap_w_px, lo_whisker_px),
                            Point2D::new(cx_px + cap_w_px, lo_whisker_px),
                        ],
                        &whisker_line,
                    )?;
                    backend.draw_line(
                        &[
                            Point2D::new(cx_px - cap_w_px, hi_whisker_px),
                            Point2D::new(cx_px + cap_w_px, hi_whisker_px),
                        ],
                        &whisker_line,
                    )?;
                    let box_pts = vec![
                        Point2D::new(cx_px - box_w_px, q1_px),
                        Point2D::new(cx_px + box_w_px, q1_px),
                        Point2D::new(cx_px + box_w_px, q3_px),
                        Point2D::new(cx_px - box_w_px, q3_px),
                    ];
                    let box_fill = FillStyle {
                        color: (255, 255, 255),
                        stroke: theme.foreground,
                        stroke_width: 1.0,
                        opacity: 1.0,
                    };
                    backend.draw_polygon(&box_pts, &box_fill)?;

                    let median_line = LineStyle::solid(theme.foreground, 1.5);
                    backend.draw_line(
                        &[
                            Point2D::new(cx_px - box_w_px, median_px),
                            Point2D::new(cx_px + box_w_px, median_px),
                        ],
                        &median_line,
                    )?;
                }
                DistributionLayout::Raincloud => {
                    let baseline = center_x_data - violin_w_data * 0.10;
                    let mut cloud = Vec::with_capacity(n_grid + 2);
                    for (gy, density) in grid.iter().zip(&densities) {
                        let offset = (*density / max_d) * violin_w_data * 0.40;
                        let (x, y) = proj.project(baseline - offset, *gy);
                        cloud.push(Point2D::new(x, y));
                    }
                    let (baseline_high_x, baseline_high_y) = proj.project(baseline, support_high);
                    let (baseline_low_x, baseline_low_y) = proj.project(baseline, support_low);
                    cloud.push(Point2D::new(baseline_high_x, baseline_high_y));
                    cloud.push(Point2D::new(baseline_low_x, baseline_low_y));
                    backend.draw_polygon(&cloud, &fill)?;

                    let box_center = center_x_data + violin_w_data * 0.02;
                    let box_half_width = violin_w_data * 0.08;
                    let (box_left, _) = proj.project(box_center - box_half_width, 0.0);
                    let (box_right, _) = proj.project(box_center + box_half_width, 0.0);
                    let (_, q1) = proj.project(box_center, summary.q1);
                    let (_, q3) = proj.project(box_center, summary.q3);
                    let (_, median) = proj.project(box_center, summary.median);
                    backend.draw_polygon(
                        &[
                            Point2D::new(box_left, q1),
                            Point2D::new(box_right, q1),
                            Point2D::new(box_right, q3),
                            Point2D::new(box_left, q3),
                        ],
                        &FillStyle {
                            color: (255, 255, 255),
                            stroke: theme.foreground,
                            stroke_width: 1.0,
                            opacity: 1.0,
                        },
                    )?;
                    backend.draw_line(
                        &[
                            Point2D::new(box_left, median),
                            Point2D::new(box_right, median),
                        ],
                        &LineStyle::solid(theme.foreground, 1.5),
                    )?;
                }
            }
        }
        if distribution_layout == DistributionLayout::Raincloud {
            let violin_w_data = if group_active {
                (0.72 / group_levels.len() as f64).min(0.34)
            } else {
                0.8
            };
            for (index, group) in sample.groups.iter().enumerate() {
                let center = crate::plots::distribution_sample::center(group, group_levels.len());
                let (box_right, _) = proj.project(center + violin_w_data * 0.10, 0.0);
                let (strip_left, _) = proj.project(center + violin_w_data * 0.24, 0.0);
                let (strip_right, _) = proj.project(center + violin_w_data * 0.46, 0.0);
                let point_radius = 2.0;
                let strip_min = strip_left + point_radius;
                let strip_max = strip_right - point_radius;
                if strip_min - point_radius <= box_right || strip_max < strip_min {
                    return Err(Error::Render(
                        "raincloud: panel is too narrow to separate the complete observation strip from the IQR box; increase width or reduce series"
                            .into(),
                    ));
                }
                let color = group_color(chart, group, index, group_active, family, theme)?;
                let points = group
                    .observations
                    .iter()
                    .map(|observation| {
                        let unit = crate::plots::distribution_sample::source_row_unit(
                            observation.source_row,
                        );
                        let x = strip_min + unit * (strip_max - strip_min);
                        let (_, y) = proj.project(center, observation.value);
                        Point2D::new(x, y)
                    })
                    .collect::<Vec<_>>();
                backend.draw_scatter(
                    &points,
                    &crate::plots::distribution_sample::overlay_style(color),
                )?;
            }
        }
        if overlay.is_visible() {
            let violin_w_data = if group_active {
                (0.72 / group_levels.len() as f64).min(0.34)
            } else {
                0.8
            };
            for (index, group) in sample.groups.iter().enumerate() {
                let center = crate::plots::distribution_sample::center(group, group_levels.len());
                let color = group_color(chart, group, index, group_active, family, theme)?;
                let points = group
                    .observations
                    .iter()
                    .map(|observation| {
                        let x = crate::plots::distribution_sample::overlay_x(
                            overlay,
                            center,
                            violin_w_data,
                            observation.source_row,
                        );
                        let (x, y) = proj.project(x, observation.value);
                        Point2D::new(x, y)
                    })
                    .collect::<Vec<_>>();
                backend.draw_scatter(
                    &points,
                    &crate::plots::distribution_sample::overlay_style(color),
                )?;
            }
        }
        Ok(())
    })?;

    if hue_levels.is_empty() {
        Ok(PlotRenderResult::new(layout, Vec::new()).with_significance_plan(significance))
    } else {
        Ok(PlotRenderResult::new(layout, legend_entries).with_significance_plan(significance))
    }
}

fn group_color(
    chart: &Plot,
    group: &crate::plots::distribution_sample::DistributionGroup,
    default_index: usize,
    group_active: bool,
    family: &'static str,
    theme: &ResolvedTheme,
) -> Result<(u8, u8, u8), Error> {
    let color_index = group
        .hue_index
        .unwrap_or(if group_active { 0 } else { default_index });
    if let Some(label) = group.hue_label.as_deref() {
        chart.categorical_color(family, label, color_index, theme)
    } else {
        Ok(theme.colors[color_index % theme.colors.len()])
    }
}

fn violin_summary(values: &[f64]) -> Result<ViolinSummary, Error> {
    let q1 = quantile(values, 0.25)?;
    let median = quantile(values, 0.5)?;
    let q3 = quantile(values, 0.75)?;
    let iqr = q3 - q1;
    let lo_whisker = values
        .iter()
        .copied()
        .filter(|value| *value >= q1 - 1.5 * iqr)
        .fold(f64::INFINITY, f64::min)
        .min(q1);
    let hi_whisker = values
        .iter()
        .copied()
        .filter(|value| *value <= q3 + 1.5 * iqr)
        .fold(f64::NEG_INFINITY, f64::max)
        .max(q3);
    Ok(ViolinSummary {
        q1,
        median,
        q3,
        lo_whisker,
        hi_whisker,
    })
}

#[cfg(test)]
mod tests {
    use super::{violin_summary, ViolinSummary};

    #[test]
    fn violin_summary_preserves_type_7_singleton_geometry() {
        assert_eq!(
            violin_summary(&[4.0]).unwrap(),
            ViolinSummary {
                q1: 4.0,
                median: 4.0,
                q3: 4.0,
                lo_whisker: 4.0,
                hi_whisker: 4.0,
            }
        );
    }

    #[test]
    fn violin_summary_preserves_type_7_two_point_geometry() {
        assert_eq!(
            violin_summary(&[0.0, 4.0]).unwrap(),
            ViolinSummary {
                q1: 1.0,
                median: 2.0,
                q3: 3.0,
                lo_whisker: 0.0,
                hi_whisker: 4.0,
            }
        );
    }

    #[test]
    fn violin_summary_preserves_repeated_values_and_inclusive_fences() {
        assert_eq!(
            violin_summary(&[1.0, 1.0, 1.0, 9.0]).unwrap(),
            ViolinSummary {
                q1: 1.0,
                median: 1.0,
                q3: 3.0,
                lo_whisker: 1.0,
                hi_whisker: 3.0,
            }
        );
    }
}
