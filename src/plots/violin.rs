//! `lichtung-plot::plots::violin` — violin plot (KDE + box overlay).
//!
//! Groups the data by `x_col` (categorical) and draws a violin
//! per group: a kernel density estimate (KDE) curve mirrored
//! around the category's vertical axis, with the existing
//! boxplot's Q1/median/Q3 box overlaid on top.
//!
//! KDE and quantiles are display geometry; the plot performs no inferential
//! test.
//! The KDE is computed via [`crate::stats::distribution::kde`]; the quantiles
//! (Q1, median, Q3, whiskers) are computed via the same module's
//! [`crate::stats::distribution::quantile`].
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

pub(crate) mod precomputed;

use crate::Error;

use crate::data::PlotData;
use crate::decorations::legend::LegendEntry;
use crate::plots::categorical_distribution::{self, DistributionPayload, PreparedDistribution};
use crate::render::backend::{FillStyle, LineStyle, Point2D};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::stats::distribution::{kde_support_range, kde_with_source};
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

#[derive(Debug)]
struct ViolinDensity {
    summary: ViolinSummary,
    densities: Vec<f64>,
    maximum: f64,
}

#[derive(Debug)]
pub(crate) struct PreparedViolin {
    pub(crate) sample: crate::plots::distribution_sample::DistributionSample,
    pub(crate) support_range: (f64, f64),
    grid: Vec<f64>,
    densities: Vec<ViolinDensity>,
}

pub(crate) fn prepare(
    chart: &Plot,
    data: PlotData<'_>,
    facet: Option<crate::data::FacetAxisSources<'_>>,
) -> Result<PreparedViolin, Error> {
    let family = if chart.kind() == crate::PlotKind::Violin {
        "violin"
    } else {
        "raincloud"
    };
    let sample = crate::plots::distribution_sample::prepare_scoped(chart, data, family, facet)?;
    if chart.kind() == crate::PlotKind::Raincloud {
        let mut source_rows = Vec::new();
        for group in &sample.groups {
            data.reserve_source(&mut source_rows, group.observations.len())?;
            let start = source_rows.len();
            source_rows.extend(
                group
                    .observations
                    .iter()
                    .enumerate()
                    .map(|(index, observation)| (observation.source_row, start + index)),
            );
        }
        crate::data::source::sort_source_by(&mut source_rows, Some(data), Ord::cmp)?;
        data.claim_source_visits(source_rows.len())?;
        // Keep the former traversal's first duplicate diagnostic without a hash
        // table allocation or quadratic identity scans.
        if let Some((row, _)) = source_rows
            .windows(2)
            .filter(|pair| pair[0].0 == pair[1].0)
            .map(|pair| pair[1])
            .min_by_key(|(_, index)| *index)
        {
            return Err(Error::Data(format!(
                "raincloud: duplicate source-row identity {row}"
            )));
        }
    }

    for group in &sample.groups {
        data.claim_source_visits(group.observations.len())?;
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

    let extent_groups = data.collect_source(
        sample
            .groups
            .iter()
            .map(|group| data.collect_source(group.observations.iter().map(|o| Ok(o.value)))),
    )?;
    for values in &extent_groups {
        data.claim_source_visits(values.len().saturating_mul(3))?;
    }
    let support_range = kde_support_range(extent_groups.iter().map(Vec::as_slice), 2.0)
        .ok_or_else(|| Error::Data(format!("{family}: no finite KDE support")))?;
    let grid = data.collect_source((0..64).map(|k| {
        data.claim_source_visits(1)?;
        // The zero-density seam must share the exact retained support end,
        // including when the parent assigns that end as a display limit.
        Ok(if k == 63 {
            support_range.1
        } else {
            support_range.0 + (support_range.1 - support_range.0) * (k as f64) / 63.0
        })
    }))?;
    let densities = data.collect_source(extent_groups.iter().map(|values| {
        let summary = violin_summary_with_source(values, Some(data))?;
        let mut densities = kde_with_source(values, &grid, Some(data))?;
        densities[0] = 0.0;
        densities[63] = 0.0;
        data.claim_source_visits(densities.len())?;
        let maximum = densities.iter().copied().fold(0.0_f64, f64::max);
        Ok(ViolinDensity {
            summary,
            densities,
            maximum: if maximum <= 0.0 { 1.0 } else { maximum },
        })
    }))?;
    Ok(PreparedViolin {
        sample,
        support_range,
        grid,
        densities,
    })
}

pub(crate) fn measure_marks(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    layout: crate::layout::Layout,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    categorical_distribution::count(chart.kind(), 1);
    let retained = categorical_distribution::retained(chart)?;
    if let PreparedDistribution::Supplied(input) = retained {
        return precomputed::measure_marks(chart, input, context, layout);
    }
    let PreparedDistribution::Violin(prepared) = retained else {
        return Err(Error::Render("violin requires retained density samples".into()).into());
    };
    let distribution_layout = if chart.kind() == crate::PlotKind::Violin {
        DistributionLayout::Mirrored
    } else {
        DistributionLayout::Raincloud
    };
    let family = if distribution_layout == DistributionLayout::Mirrored {
        "violin"
    } else {
        "raincloud"
    };
    let theme = context.theme;
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = context.data.with_axis_resources(&resources);
    let sample = &prepared.sample;
    let x_levels = &sample.x_levels;
    let group_levels = &sample.group_levels;
    let hue_levels = &sample.hue_levels;
    let support_range = prepared.support_range;
    let base_display_range = layout.projection.y_range();
    let rendered_maxima = data.collect_source((0..x_levels.len()).map(|_| Ok(support_range.1)))?;
    let overlay = chart.raw_overlay_spec();
    let panel = context.panel;
    let legend_entries =
        data.collect_source(hue_levels.iter().enumerate().map(|(idx, label)| {
            let color = chart.categorical_color(family, label, idx, theme)?;
            let identity = chart.point_legend_identity(label, data)?;
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
        }))?;
    let significance = if group_levels.is_empty() {
        crate::significance::plan(
            chart,
            x_levels,
            &rendered_maxima,
            base_display_range,
            panel,
            theme,
            context.fit_scope(),
        )?
    } else {
        let mut grouped_maxima = data.collect_source((0..x_levels.len()).map(|_| {
            data.collect_source((0..group_levels.len()).map(|_| Ok(f64::NEG_INFINITY)))
        }))?;
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
            context.fit_scope(),
        )?
    };
    let group_active = !group_levels.is_empty();
    let proj = &layout.projection;

    let n_grid: usize = 64;
    let (support_low, support_high) = support_range;

    let mut payload = DistributionPayload::new(layout.projection.panel_box(), chart.kind());
    for (i, group) in sample.groups.iter().enumerate() {
        let density = &prepared.densities[i];
        let summary = &density.summary;
        let grid = &prepared.grid;
        let densities = &density.densities;
        let max_d = density.maximum;

        let center_x_data = crate::plots::distribution_sample::center(group, group_levels.len());
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
                let mut pts = Vec::new();
                data.reserve_source(&mut pts, 2 * n_grid + 1)?;
                for (gy, d) in grid.iter().zip(densities.iter()) {
                    let x_off = (*d / max_d) * (violin_w_data / 2.0);
                    let (px, py) = proj.project(center_x_data + x_off, *gy);
                    pts.push(Point2D::new(px, py));
                }
                proj.validate_span(
                    crate::layout::AxisSide::Left,
                    (support_low, support_high),
                    (pts[0].y, pts[pts.len() - 1].y),
                    support_low < support_high,
                    "violin polygon support",
                )?;
                for (gy, d) in grid.iter().zip(densities.iter()).rev() {
                    let x_off = (*d / max_d) * (violin_w_data / 2.0);
                    let (px, py) = proj.project(center_x_data - x_off, *gy);
                    pts.push(Point2D::new(px, py));
                }
                payload.polygon(pts, fill, data)?;

                // Inner box-and-whisker mark, matching seaborn's default inner="box" cue.
                let (cx_px, _) = proj.project(center_x_data, 0.0);
                let box_w_px =
                    (proj.project(violin_w_data * 0.08, 0.0).0 - proj.project(0.0, 0.0).0).abs();
                let cap_w_px = box_w_px;
                let (_, lo_whisker_px) = proj.project(center_x_data, summary.lo_whisker);
                let (_, q1_px) = proj.project(center_x_data, summary.q1);
                let (_, q3_px) = proj.project(center_x_data, summary.q3);
                let (_, hi_whisker_px) = proj.project(center_x_data, summary.hi_whisker);
                let (_, median_px) = proj.project(center_x_data, summary.median);
                for (source, projected, context) in [
                    (
                        (summary.q1, summary.q3),
                        (q1_px, q3_px),
                        "violin box polygon",
                    ),
                    (
                        (summary.lo_whisker, summary.hi_whisker),
                        (lo_whisker_px, hi_whisker_px),
                        "violin whisker",
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
                let whisker_line = LineStyle::solid(theme.foreground, theme.interval_width);
                payload.line(
                    &[
                        Point2D::new(cx_px, lo_whisker_px),
                        Point2D::new(cx_px, hi_whisker_px),
                    ],
                    &whisker_line,
                    data,
                )?;
                payload.line(
                    &[
                        Point2D::new(cx_px - cap_w_px, lo_whisker_px),
                        Point2D::new(cx_px + cap_w_px, lo_whisker_px),
                    ],
                    &whisker_line,
                    data,
                )?;
                payload.line(
                    &[
                        Point2D::new(cx_px - cap_w_px, hi_whisker_px),
                        Point2D::new(cx_px + cap_w_px, hi_whisker_px),
                    ],
                    &whisker_line,
                    data,
                )?;
                let box_pts = data.collect_source(
                    [
                        Point2D::new(cx_px - box_w_px, q1_px),
                        Point2D::new(cx_px + box_w_px, q1_px),
                        Point2D::new(cx_px + box_w_px, q3_px),
                        Point2D::new(cx_px - box_w_px, q3_px),
                    ]
                    .into_iter()
                    .map(Ok),
                )?;
                let box_fill = FillStyle {
                    color: (255, 255, 255),
                    stroke: theme.foreground,
                    stroke_width: 1.0,
                    opacity: 1.0,
                };
                payload.polygon(box_pts, box_fill, data)?;

                let median_line = LineStyle::solid(theme.foreground, 1.5);
                payload.line(
                    &[
                        Point2D::new(cx_px - box_w_px, median_px),
                        Point2D::new(cx_px + box_w_px, median_px),
                    ],
                    &median_line,
                    data,
                )?;
            }
            DistributionLayout::Raincloud => {
                let baseline = center_x_data - violin_w_data * 0.10;
                let mut cloud = Vec::new();
                data.reserve_source(&mut cloud, n_grid + 2)?;
                for (gy, density) in grid.iter().zip(densities) {
                    let offset = (*density / max_d) * violin_w_data * 0.40;
                    let (x, y) = proj.project(baseline - offset, *gy);
                    cloud.push(Point2D::new(x, y));
                }
                let (baseline_high_x, baseline_high_y) = proj.project(baseline, support_high);
                let (baseline_low_x, baseline_low_y) = proj.project(baseline, support_low);
                proj.validate_span(
                    crate::layout::AxisSide::Left,
                    (support_low, support_high),
                    (baseline_low_y, baseline_high_y),
                    support_low < support_high,
                    "raincloud polygon support",
                )?;
                cloud.push(Point2D::new(baseline_high_x, baseline_high_y));
                cloud.push(Point2D::new(baseline_low_x, baseline_low_y));
                payload.polygon(cloud, fill, data)?;

                let box_center = center_x_data + violin_w_data * 0.02;
                let box_half_width = violin_w_data * 0.08;
                let (box_left, _) = proj.project(box_center - box_half_width, 0.0);
                let (box_right, _) = proj.project(box_center + box_half_width, 0.0);
                let (_, q1) = proj.project(box_center, summary.q1);
                let (_, q3) = proj.project(box_center, summary.q3);
                let (_, median) = proj.project(box_center, summary.median);
                proj.validate_span(
                    crate::layout::AxisSide::Left,
                    (summary.q1, summary.q3),
                    (q1, q3),
                    summary.q1 < summary.q3,
                    "raincloud box polygon",
                )?;
                payload.polygon(
                    data.collect_source(
                        [
                            Point2D::new(box_left, q1),
                            Point2D::new(box_right, q1),
                            Point2D::new(box_right, q3),
                            Point2D::new(box_left, q3),
                        ]
                        .into_iter()
                        .map(Ok),
                    )?,
                    FillStyle {
                        color: (255, 255, 255),
                        stroke: theme.foreground,
                        stroke_width: 1.0,
                        opacity: 1.0,
                    },
                    data,
                )?;
                payload.line(
                    &[
                        Point2D::new(box_left, median),
                        Point2D::new(box_right, median),
                    ],
                    &LineStyle::solid(theme.foreground, 1.5),
                    data,
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
                return Err(context.fit_scope().child(crate::layout::AddressRole::CategoricalSeriesSlot, index)?.failure(crate::layout::fit::FitKind::SpecializedGeometry,
                        "raincloud: panel is too narrow to separate the complete observation strip from the IQR box; increase width or reduce series"
                    ).into());
            }
            let color = group_color(chart, group, index, group_active, family, theme)?;
            let points = data.collect_source(group.observations.iter().map(|observation| {
                data.claim_source_visits(1)?;
                let unit =
                    crate::plots::distribution_sample::source_row_unit(observation.source_row);
                let x = strip_min + unit * (strip_max - strip_min);
                let (_, y) = proj.project(center, observation.value);
                Ok(Point2D::new(x, y))
            }))?;
            payload.scatter(
                points,
                crate::plots::distribution_sample::overlay_style(color),
                data,
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
            let points = data.collect_source(group.observations.iter().map(|observation| {
                data.claim_source_visits(1)?;
                let x = crate::plots::distribution_sample::overlay_x(
                    overlay,
                    center,
                    violin_w_data,
                    observation.source_row,
                );
                let (x, y) = proj.project(x, observation.value);
                Ok(Point2D::new(x, y))
            }))?;
            payload.scatter(
                points,
                crate::plots::distribution_sample::overlay_style(color),
                data,
            )?;
        }
    }
    payload
        .retain(
            PlotRenderResult::new(layout, legend_entries).with_significance_plan(significance),
            context,
        )
        .map_err(Into::into)
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

#[cfg(test)]
fn violin_summary(values: &[f64]) -> Result<ViolinSummary, Error> {
    violin_summary_with_source(values, None)
}

fn violin_summary_with_source(
    values: &[f64],
    data: Option<PlotData<'_>>,
) -> Result<ViolinSummary, Error> {
    use crate::data::source::SourceAdmission;
    data.source_visits(values.len())?;
    if values.is_empty() || values.iter().any(|v| !v.is_finite()) {
        return Err(Error::Data(
            "violin: samples must be finite and nonempty".into(),
        ));
    }
    let mut sorted = Vec::new();
    data.source_reserve(&mut sorted, values.len())?;
    sorted.extend_from_slice(values);
    crate::data::source::sort_numeric_source(&mut sorted, data)?;
    let q1 = crate::stats::distribution::sorted_quantile(&sorted, 0.25);
    let median = crate::stats::distribution::sorted_quantile(&sorted, 0.5);
    let q3 = crate::stats::distribution::sorted_quantile(&sorted, 0.75);
    data.source_visits(values.len().saturating_mul(2))?;
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
    fn admitted_identity_sort_preserves_first_group_traversal_duplicate() {
        let chart = crate::Plot::raincloud(crate::RaincloudInput::samples(
            crate::CategoryValues::new("category", ["a", "b", "a", "b"]).unwrap(),
            crate::NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]),
        ))
        .erase();
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let data = chart
            .data()
            .map_source_rows(&[5, 4, 5, 4])
            .with_axis_resources(&resources);
        let error = super::prepare(&chart, data, None).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("duplicate source-row identity 5"),
            "{error}"
        );
    }

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
