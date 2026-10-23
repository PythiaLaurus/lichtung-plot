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
use crate::plots::categorical_distribution::{self, DistributionPayload, PreparedDistribution};
use crate::render::backend::Point2D;
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::stats::distribution::{kde, kde_support_range};
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

impl PreparedViolin {
    pub(crate) fn alpha_guide(
        &self,
        chart: &Plot,
        theme: &ResolvedTheme,
        guide: &mut crate::decorations::legend::alpha::AlphaGuideBuilder<'_>,
    ) -> crate::Result<()> {
        use crate::alpha::AlphaUnitKind;
        use crate::decorations::legend::alpha::{AlphaGlyphRole, AlphaUnit};
        let raincloud = chart.kind() == crate::PlotKind::Raincloud;
        let fill = theme.marks.get_fills().get_bodies().get_violin_body();
        let point = chart.distribution_point_style(
            if raincloud {
                theme.marks.get_points().get_mandatory_strip()
            } else {
                theme.marks.get_points().get_raw_overlay()
            },
            (0, 0, 0),
        );
        for group in &self.sample.groups {
            let unit = AlphaUnit::aggregate(chart, &group.label, group.group_label.as_deref())?;
            guide.fill(
                AlphaGlyphRole::Fill,
                AlphaUnitKind::Aggregate,
                unit.clone(),
                group.fill_alpha,
                fill.backend_style((0, 0, 0)),
                fill.get_outline()
                    .is_some_and(|outline| outline.mapped_width().is_some()),
            )?;
            if !raincloud {
                guide.line(
                    AlphaGlyphRole::Summary,
                    AlphaUnitKind::Aggregate,
                    unit.clone(),
                    group.line_alpha,
                    theme
                        .marks
                        .get_strokes()
                        .get_summaries()
                        .get_summary()
                        .backend_style(),
                    false,
                )?;
            }
            guide.line(
                AlphaGlyphRole::Median,
                AlphaUnitKind::Aggregate,
                unit,
                group.line_alpha,
                theme
                    .marks
                    .get_strokes()
                    .get_summaries()
                    .get_median()
                    .backend_style(),
                false,
            )?;
            if raincloud || chart.raw_overlay_spec().is_visible() {
                let role = if raincloud {
                    AlphaGlyphRole::MandatoryStrip
                } else {
                    AlphaGlyphRole::RawOverlay
                };
                for observation in &group.observations {
                    guide.point(role, observation.alpha, point.clone(), false)?;
                }
            }
        }
        Ok(())
    }
}

/// The strip's occupied physical width and the final IQR-box separation use
/// the complete resolved point/box stroke, not the historical 2px point hint.
pub(crate) fn raincloud_strip_pitch(
    theme: &crate::theme::ResolvedTheme,
    width: f64,
    point: Option<crate::render::geometry::GeometryBox>,
) -> Result<f64, Error> {
    let left = point.map_or(0.0, |ink| (-ink.left).max(0.0));
    let right = point.map_or(0.0, |ink| ink.right.max(0.0));
    let box_right = crate::render::geometry::rectangular_fill_ink_bounds(
        &theme
            .marks
            .get_fills()
            .get_bodies()
            .get_inner_summary()
            .backend_style(),
    )?
    .map_or(0.0, |ink| ink.right.max(0.0));
    Ok(((left + right) / (0.22 * width)).max((box_right + 1.0 / 1024.0) / (0.14 * width)))
}

/// Sampled Violin and Raincloud retain the same group centers and box/cloud
/// grammar consumed by their selected polygons, without KDE-vertex counting.
pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let PreparedDistribution::Violin(prepared) = categorical_distribution::retained(chart)? else {
        return Err(Error::Render(
            "sampled distribution intrinsic requires retained KDE groups".into(),
        ));
    };
    let sample = &prepared.sample;
    if sample.x_levels.is_empty()
        || sample.groups.is_empty()
        || sample.groups.len() != prepared.densities.len()
    {
        return Err(Error::Render(
            "sampled distribution groups and prepared KDEs differ".into(),
        ));
    }
    let raincloud = chart.kind() == crate::PlotKind::Raincloud;
    let grouped = !sample.group_levels.is_empty();
    let width = if grouped {
        (0.72 / sample.group_levels.len() as f64).min(0.34)
    } else {
        0.8
    };
    let cloud_style = theme
        .marks
        .get_fills()
        .get_bodies()
        .get_violin_body()
        .backend_style((0, 0, 0));
    let inner_style = theme
        .marks
        .get_fills()
        .get_bodies()
        .get_inner_summary()
        .backend_style();
    let inner = crate::render::geometry::rectangular_fill_ink_bounds(&inner_style)?;
    let (_, inner_y) = crate::render::geometry::rectangular_fill_ink_outset(&inner_style)?;
    let overlay = chart.raw_overlay_spec();
    let points_visible = raincloud || overlay.is_visible();
    let role = if raincloud {
        theme.marks.get_points().get_mandatory_strip()
    } else {
        theme.marks.get_points().get_raw_overlay()
    };
    let style = chart.distribution_point_style(role, (0, 0, 0));
    let mut pitch = 1.0_f64 / 1024.0;
    let mut numeric = inner_y;
    let body_half = if raincloud { 0.4 * width } else { width };
    pitch = pitch.max((1.0 / 1024.0) / body_half);
    let grow = |pitch: &mut f64,
                center: f64,
                left: f64,
                right: f64,
                ink: Option<crate::render::geometry::GeometryBox>|
     -> Result<(), Error> {
        let (left_room, right_room) = (0.5 + center + left, 0.5 - center - right);
        if left_room <= 0.0 || right_room <= 0.0 {
            return Err(Error::Render(
                "sampled distribution body exceeds its category slot".into(),
            ));
        }
        if let Some(ink) = ink {
            *pitch = pitch
                .max((-ink.left).max(0.0) / left_room)
                .max(ink.right.max(0.0) / right_room);
        }
        Ok(())
    };
    for group in &sample.groups {
        let cloud_style = crate::alpha::marks::fill_style(group.fill_alpha, cloud_style.clone());
        let cloud = crate::render::geometry::rectangular_fill_ink_bounds(&cloud_style)?;
        let (_, cloud_y) = crate::render::geometry::rectangular_fill_ink_outset(&cloud_style)?;
        numeric = numeric.max(cloud_y);
        let summary_ink =
            crate::render::geometry::line_ink_bounds(&crate::alpha::marks::line_style(
                group.line_alpha,
                theme
                    .marks
                    .get_strokes()
                    .get_summaries()
                    .get_summary()
                    .backend_style(),
            ))?;
        let median_ink =
            crate::render::geometry::line_ink_bounds(&crate::alpha::marks::line_style(
                group.line_alpha,
                theme
                    .marks
                    .get_strokes()
                    .get_summaries()
                    .get_median()
                    .backend_style(),
            ))?;
        numeric = numeric.max(median_ink.map_or(0.0, |ink| ink.top - ink.bottom));
        if !raincloud {
            numeric = numeric.max(summary_ink.map_or(0.0, |ink| ink.top - ink.bottom));
        }
        if group.x_index >= sample.x_levels.len()
            || (grouped && group.group_index >= sample.group_levels.len())
        {
            return Err(Error::Render(
                "sampled distribution group exceeds prepared category slot".into(),
            ));
        }
        let center = crate::plots::distribution_sample::center(group, sample.group_levels.len());
        let offset = center - (group.x_index as f64 + 0.5);
        let point = if points_visible {
            crate::alpha::marks::point_ink(
                &style,
                group.observations.iter().map(|observation| {
                    let x = crate::plots::distribution_sample::overlay_x(
                        overlay,
                        center,
                        width,
                        observation.alpha.source_row,
                    );
                    (x, observation.value, observation.alpha)
                }),
            )?
        } else {
            None
        };
        if raincloud {
            pitch = pitch.max(raincloud_strip_pitch(
                theme,
                width,
                point.map(|ink| ink.bounds),
            )?);
            grow(&mut pitch, offset, -0.5 * width, -0.1 * width, cloud)?;
            grow(&mut pitch, offset, -0.06 * width, 0.10 * width, inner)?;
            grow(&mut pitch, offset, -0.06 * width, 0.10 * width, median_ink)?;
        } else {
            grow(&mut pitch, offset, -0.5 * width, 0.5 * width, cloud)?;
            grow(&mut pitch, offset, -0.08 * width, 0.08 * width, inner)?;
        }
        if !raincloud {
            grow(&mut pitch, offset, -0.08 * width, 0.08 * width, summary_ink)?;
            grow(&mut pitch, offset, -0.08 * width, 0.08 * width, median_ink)?;
        }
        if let Some(ink) = point {
            numeric = numeric.max(ink.dimensions().1);
            if !raincloud && overlay.is_visible() {
                for x in [ink.ranges[0].0, ink.ranges[0].1] {
                    let offset = x - (group.x_index as f64 + 0.5);
                    grow(&mut pitch, offset, 0.0, 0.0, Some(ink.bounds))?;
                }
            }
        }
    }
    let count = u32::try_from(sample.x_levels.len()).map_err(|_| {
        Error::Render("sampled distribution category count exceeds spatial range".into())
    })?;
    let categorical = pitch * f64::from(count);
    if !categorical.is_finite() {
        return Err(Error::Render(
            "sampled distribution category hard span overflowed".into(),
        ));
    }
    Ok((
        categorical,
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
            crate::data::source::reserve_source(&mut source_rows, group.observations.len())?;
            let start = source_rows.len();
            source_rows.extend(
                group
                    .observations
                    .iter()
                    .enumerate()
                    .map(|(index, observation)| (observation.alpha.source_row, start + index)),
            );
        }
        crate::data::source::sort_source_by(&mut source_rows, Ord::cmp);

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

    let extent_groups = crate::data::source::collect_source(sample.groups.iter().map(|group| {
        crate::data::source::collect_source(group.observations.iter().map(|o| Ok(o.value)))
    }))?;

    let support_range = kde_support_range(extent_groups.iter().map(Vec::as_slice), 2.0)
        .ok_or_else(|| Error::Data(format!("{family}: no finite KDE support")))?;
    let grid = crate::data::source::collect_source((0..64).map(|k| {
        // The zero-density seam must share the exact retained support end,
        // including when the parent assigns that end as a display limit.
        Ok(if k == 63 {
            support_range.1
        } else {
            support_range.0 + (support_range.1 - support_range.0) * (k as f64) / 63.0
        })
    }))?;
    let densities = crate::data::source::collect_source(extent_groups.iter().map(|values| {
        let summary = violin_summary_with_source(values)?;
        let mut densities = kde(values, &grid)?;
        densities[0] = 0.0;
        densities[63] = 0.0;

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
    let theme = context.theme;
    let resources = theme.prepared_font_environment()?.operation();
    let data = context.data.with_operation(&resources);
    let sample = &prepared.sample;
    let x_levels = &sample.x_levels;
    let group_levels = &sample.group_levels;
    let support_range = prepared.support_range;
    let base_display_range = layout.projection.y_range();
    let rendered_maxima =
        crate::data::source::collect_source((0..x_levels.len()).map(|_| Ok(support_range.1)))?;
    let overlay = chart.raw_overlay_spec();
    let panel = context.panel;
    let colors = retained.resolved_colors(chart, theme, data)?;
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
        let mut grouped_maxima =
            crate::data::source::collect_source((0..x_levels.len()).map(|_| {
                crate::data::source::collect_source(
                    (0..group_levels.len()).map(|_| Ok(f64::NEG_INFINITY)),
                )
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

        let fill = crate::alpha::marks::fill_style(
            group.fill_alpha,
            theme
                .marks
                .get_fills()
                .get_bodies()
                .get_violin_body()
                .backend_style(group_color(group, &colors)?),
        );
        match distribution_layout {
            DistributionLayout::Mirrored => {
                // Violin shape
                let mut pts = Vec::new();
                crate::data::source::reserve_source(&mut pts, 2 * n_grid + 1)?;
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
                payload.polygon(pts, fill)?;

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
                let whisker_line = crate::alpha::marks::line_style(
                    group.line_alpha,
                    theme
                        .marks
                        .get_strokes()
                        .get_summaries()
                        .get_summary()
                        .backend_style(),
                );
                payload.line(
                    &[
                        Point2D::new(cx_px, lo_whisker_px),
                        Point2D::new(cx_px, hi_whisker_px),
                    ],
                    &whisker_line,
                )?;
                payload.line(
                    &[
                        Point2D::new(cx_px - cap_w_px, lo_whisker_px),
                        Point2D::new(cx_px + cap_w_px, lo_whisker_px),
                    ],
                    &whisker_line,
                )?;
                payload.line(
                    &[
                        Point2D::new(cx_px - cap_w_px, hi_whisker_px),
                        Point2D::new(cx_px + cap_w_px, hi_whisker_px),
                    ],
                    &whisker_line,
                )?;
                let box_pts = crate::data::source::collect_source(
                    [
                        Point2D::new(cx_px - box_w_px, q1_px),
                        Point2D::new(cx_px + box_w_px, q1_px),
                        Point2D::new(cx_px + box_w_px, q3_px),
                        Point2D::new(cx_px - box_w_px, q3_px),
                    ]
                    .into_iter()
                    .map(Ok),
                )?;
                let box_fill = theme
                    .marks
                    .get_fills()
                    .get_bodies()
                    .get_inner_summary()
                    .backend_style();
                payload.polygon(box_pts, box_fill)?;

                let median_line = crate::alpha::marks::line_style(
                    group.line_alpha,
                    theme
                        .marks
                        .get_strokes()
                        .get_summaries()
                        .get_median()
                        .backend_style(),
                );
                payload.line(
                    &[
                        Point2D::new(cx_px - box_w_px, median_px),
                        Point2D::new(cx_px + box_w_px, median_px),
                    ],
                    &median_line,
                )?;
            }
            DistributionLayout::Raincloud => {
                let baseline = center_x_data - violin_w_data * 0.10;
                let mut cloud = Vec::new();
                crate::data::source::reserve_source(&mut cloud, n_grid + 2)?;
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
                payload.polygon(cloud, fill)?;

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
                    crate::data::source::collect_source(
                        [
                            Point2D::new(box_left, q1),
                            Point2D::new(box_right, q1),
                            Point2D::new(box_right, q3),
                            Point2D::new(box_left, q3),
                        ]
                        .into_iter()
                        .map(Ok),
                    )?,
                    theme
                        .marks
                        .get_fills()
                        .get_bodies()
                        .get_inner_summary()
                        .backend_style(),
                )?;
                payload.line(
                    &[
                        Point2D::new(box_left, median),
                        Point2D::new(box_right, median),
                    ],
                    &crate::alpha::marks::line_style(
                        group.line_alpha,
                        theme
                            .marks
                            .get_strokes()
                            .get_summaries()
                            .get_median()
                            .backend_style(),
                    ),
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
            let color = group_color(group, &colors)?;
            let style = chart
                .distribution_point_style(theme.marks.get_points().get_mandatory_strip(), color);
            let point_ink = crate::alpha::marks::point_ink(
                &style,
                group
                    .observations
                    .iter()
                    .map(|observation| (0.0, observation.value, observation.alpha)),
            )?
            .map(|ink| ink.bounds);
            let point_left = point_ink.map_or(0.0, |ink| (-ink.left).max(0.0));
            let point_right = point_ink.map_or(0.0, |ink| ink.right.max(0.0));
            let box_right_ink = crate::render::geometry::rectangular_fill_ink_bounds(
                &theme
                    .marks
                    .get_fills()
                    .get_bodies()
                    .get_inner_summary()
                    .backend_style(),
            )?
            .map_or(0.0, |ink| ink.right.max(0.0));
            let strip_min = strip_left + point_left;
            let strip_max = strip_right - point_right;
            if strip_min - point_left <= box_right + box_right_ink || strip_max < strip_min {
                return Err(context.fit_scope().child(crate::layout::AddressRole::CategoricalSeriesSlot, index)?.failure(crate::layout::fit::FitKind::SpecializedGeometry,
                        "raincloud: panel is too narrow to separate the complete observation strip from the IQR box; increase width or reduce series"
                    ).into());
            }
            let points = crate::data::source::collect_source(group.observations.iter().map(
                |observation| {
                    let unit = crate::plots::distribution_sample::source_row_unit(
                        observation.alpha.source_row,
                    );
                    let x = strip_min + unit * (strip_max - strip_min);
                    let (_, y) = proj.project(center, observation.value);
                    Ok(Point2D::new(x, y))
                },
            ))?;
            payload.scatter(
                points,
                &style,
                group
                    .observations
                    .iter()
                    .map(|observation| observation.alpha),
            )?;
        }
    }
    if overlay.is_visible() {
        let violin_w_data = if group_active {
            (0.72 / group_levels.len() as f64).min(0.34)
        } else {
            0.8
        };
        for group in &sample.groups {
            let center = crate::plots::distribution_sample::center(group, group_levels.len());
            let color = group_color(group, &colors)?;
            let points = crate::data::source::collect_source(group.observations.iter().map(
                |observation| {
                    let x = crate::plots::distribution_sample::overlay_x(
                        overlay,
                        center,
                        violin_w_data,
                        observation.alpha.source_row,
                    );
                    let (x, y) = proj.project(x, observation.value);
                    Ok(Point2D::new(x, y))
                },
            ))?;
            payload.scatter(
                points,
                &chart.distribution_point_style(theme.marks.get_points().get_raw_overlay(), color),
                group
                    .observations
                    .iter()
                    .map(|observation| observation.alpha),
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

fn group_color(
    group: &crate::plots::distribution_sample::DistributionGroup,
    colors: &crate::theme::CategoryColors,
) -> Result<(u8, u8, u8), Error> {
    colors.color(group.hue_label.as_deref().unwrap_or(&group.label))
}

fn violin_summary_with_source(values: &[f64]) -> Result<ViolinSummary, Error> {
    if values.is_empty() || values.iter().any(|v| !v.is_finite()) {
        return Err(Error::Data(
            "violin: samples must be finite and nonempty".into(),
        ));
    }
    let mut sorted = Vec::new();
    crate::data::source::reserve_source(&mut sorted, values.len())?;
    sorted.extend_from_slice(values);
    crate::data::source::sort_numeric_source(&mut sorted);
    let q1 = crate::stats::distribution::sorted_quantile(&sorted, 0.25);
    let median = crate::stats::distribution::sorted_quantile(&sorted, 0.5);
    let q3 = crate::stats::distribution::sorted_quantile(&sorted, 0.75);

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
mod alpha_tests;
