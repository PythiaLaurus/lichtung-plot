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
#[cfg(test)]
use crate::stats::distribution::quantile;
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
    let fill = theme.marks.box_body.backend_style((0, 0, 0));
    let body = crate::render::geometry::rectangular_fill_ink_bounds(theme, &fill)?;
    let (_, out_y) = crate::render::geometry::rectangular_fill_ink_outset(theme, &fill)?;
    let summary = crate::render::geometry::line_ink_width(
        theme,
        &theme.marks.summary_stroke.backend_style(),
    )?;
    let overlay = chart.raw_overlay_spec();
    let point_style = if overlay.is_visible() {
        theme
            .marks
            .raw_overlay_point
            .backend_style((0, 0, 0), theme.marks.raw_overlay_radius, true)
    } else {
        chart.point_style(theme, theme.foreground, theme.marks.point_sizes.box_flier)
    };
    let point = crate::render::geometry::point_ink_bounds(theme, &point_style)?;
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = chart.data().with_axis_resources(&resources);
    data.claim_source_visits(prepared.stats.len())?;
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
                data.claim_source_visits(group.observations.len())?;
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
    let stats = data.collect_source(sample.groups.iter().map(
        |group| -> Result<BoxStats, Error> {
            let mut ys = data.collect_source(group.observations.iter().map(|o| Ok(o.value)))?;
            let summary = box_summary_with_source(&mut ys, Some(data))?;
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

    for stat in &stats {
        data.claim_source_visits(stat.fliers.len().saturating_add(1).saturating_mul(2))?;
    }
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
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = context.data.with_axis_resources(&resources);
    let sample = &prepared.sample;
    let stats = &prepared.stats;
    let x_levels = &sample.x_levels;
    let group_levels = &sample.group_levels;
    let group_active = !group_levels.is_empty();
    let base_y_range = layout.projection.y_range();
    let mut rendered_maxima =
        data.collect_source((0..x_levels.len()).map(|_| Ok(f64::NEG_INFINITY)))?;
    let mut grouped_maxima = data.collect_source((0..x_levels.len()).map(|_| {
        data.collect_source((0..group_levels.len().max(1)).map(|_| Ok(f64::NEG_INFINITY)))
    }))?;
    for stat in stats {
        let maximum = stat.fliers.iter().copied().fold(stat.hi_whisker, f64::max);
        rendered_maxima[stat.x_index] = rendered_maxima[stat.x_index].max(maximum);
        grouped_maxima[stat.x_index][stat.group_index] = maximum;
    }
    let overlay = chart.raw_overlay_spec();
    let panel = context.panel;
    let (colors, legend_entries) = {
        let retained = categorical_distribution::retained(chart)?;
        let colors = retained.resolved_colors(chart, theme, data)?;
        let entries = retained.guide_entries_with_colors(chart, theme, data, &colors)?;
        (colors, entries)
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
        let color = colors.color(
            group.hue_label.as_deref().unwrap_or(&group.label),
            &resources,
        )?;
        let fill = theme.marks.box_body.backend_style(color);
        let style = theme.marks.summary_stroke.backend_style();

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
        let pts = data.collect_source(
            [
                Point2D::new(xc - box_w_px / 2.0, y_q1),
                Point2D::new(xc - box_w_px / 2.0, y_q3),
                Point2D::new(xc + box_w_px / 2.0, y_q3),
                Point2D::new(xc + box_w_px / 2.0, y_q1),
            ]
            .into_iter()
            .map(Ok),
        )?;
        payload.polygon(pts, fill, data)?;

        // Median line.
        payload.line(
            &[
                Point2D::new(xc - box_w_px / 2.0, y_med),
                Point2D::new(xc + box_w_px / 2.0, y_med),
            ],
            &style,
            data,
        )?;

        // Whiskers.
        payload.line(
            &[Point2D::new(xc, y_q3), Point2D::new(xc, y_hi)],
            &style,
            data,
        )?;
        payload.line(
            &[Point2D::new(xc, y_q1), Point2D::new(xc, y_lo)],
            &style,
            data,
        )?;
        let cap = box_w_px * 0.3;
        payload.line(
            &[Point2D::new(xc - cap, y_hi), Point2D::new(xc + cap, y_hi)],
            &style,
            data,
        )?;
        payload.line(
            &[Point2D::new(xc - cap, y_lo), Point2D::new(xc + cap, y_lo)],
            &style,
            data,
        )?;

        if overlay == RawOverlay::Hidden {
            let flier_style =
                chart.point_style(theme, theme.foreground, theme.marks.point_sizes.box_flier);
            let flier_points = data.collect_source(stat.fliers.iter().map(|y| {
                data.claim_source_visits(1)?;
                let (x_px, y_px) = proj.project(x_center_data, *y);
                Ok(Point2D::new(x_px, y_px))
            }))?;
            if !flier_points.is_empty() {
                payload.scatter(flier_points, flier_style, data)?;
            }
        }
    }
    if overlay.is_visible() {
        for group in &sample.groups {
            let center = crate::plots::distribution_sample::center(group, group_levels.len());
            let points = data.collect_source(group.observations.iter().map(|observation| {
                data.claim_source_visits(1)?;
                let x = crate::plots::distribution_sample::overlay_x(
                    overlay,
                    center,
                    box_w_data,
                    observation.source_row,
                );
                let (x, y) = proj.project(x, observation.value);
                Ok(Point2D::new(x, y))
            }))?;
            payload.scatter(
                points,
                crate::plots::distribution_sample::overlay_style(
                    colors.color(
                        group.hue_label.as_deref().unwrap_or(&group.label),
                        &resources,
                    )?,
                    theme,
                ),
                data,
            )?;
        }
    }
    payload.retain(
        PlotRenderResult::new(layout, legend_entries).with_significance_plan(significance),
        context,
        chart,
    )
}

#[cfg(test)]
fn box_summary(values: &mut [f64]) -> Result<BoxSummary, Error> {
    box_summary_with_source(values, None)
}

fn box_summary_with_source(
    values: &mut [f64],
    data: Option<PlotData<'_>>,
) -> Result<BoxSummary, Error> {
    use crate::data::source::SourceAdmission;
    data.source_visits(values.len())?;
    if values.is_empty() || values.iter().any(|v| !v.is_finite()) {
        return Err(Error::Data(
            "boxplot: samples must be finite and nonempty".into(),
        ));
    }
    crate::data::source::sort_numeric_source(values, data)?;
    let q1 = crate::stats::distribution::sorted_quantile(values, 0.25);
    let median = crate::stats::distribution::sorted_quantile(values, 0.5);
    let q3 = crate::stats::distribution::sorted_quantile(values, 0.75);
    let iqr = q3 - q1;
    let lo_fence = q1 - 1.5 * iqr;
    let hi_fence = q3 + 1.5 * iqr;
    data.source_visits(values.len().saturating_mul(3))?;
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
    data.source_reserve(&mut fliers, values.len())?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_grouped_boxplot_bodies_and_fliers_have_separate_hard_pitch() {
        let categories =
            crate::CategoryValues::new("category", ["a", "a", "a", "a", "b", "b", "b", "b"])
                .unwrap();
        let values = crate::NumericValues::new("value", [1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 100.0]);
        let grouped = Plot::boxplot(categories.clone(), values.clone())
            .series(
                crate::CategoryValues::new(
                    "group",
                    ["one", "one", "one", "one", "two", "two", "two", "two"],
                )
                .unwrap(),
            )
            .erase();
        let single = Plot::boxplot(categories, values).erase();
        let intrinsic = |source: Plot, theme: &crate::theme::ResolvedTheme| {
            let resources = crate::font::canonical::OperationGenerationResources::new();
            let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
            inputs
                .attach(source)
                .unwrap()
                .intrinsic_body_minimum(theme)
                .unwrap()
        };
        let mut theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        theme.marks.box_body.outline = Some(crate::theme::DataOutlineTheme::fixed(
            theme
                .marks
                .box_body
                .outline()
                .unwrap()
                .fixed_stroke()
                .unwrap()
                .clone()
                .with_width(100.0)
                .unwrap(),
        ));
        let grouped_body = intrinsic(grouped.clone(), &theme);
        let single_body = intrinsic(single.clone(), &theme);
        assert!(grouped_body.0 > single_body.0);
        theme.marks.box_body.outline = Some(crate::theme::DataOutlineTheme::fixed(
            theme
                .marks
                .box_body
                .outline()
                .unwrap()
                .fixed_stroke()
                .unwrap()
                .clone()
                .with_width(1.0)
                .unwrap(),
        ));
        theme.marks.point_sizes.box_flier = 80;
        let flier = intrinsic(grouped.clone(), &theme);
        assert!(
            flier.0
                > intrinsic(
                    grouped,
                    &crate::Theme::Publication
                        .resolve(&crate::font::canonical::OperationGenerationResources::new())
                        .unwrap()
                )
                .0
        );
        let exact = Plot::boxplot(
            crate::CategoryValues::new("category", ["a", "a", "a", "a", "b", "b", "b", "b"])
                .unwrap(),
            crate::NumericValues::new("value", [1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 100.0]),
        )
        .y_limits(0.0, 101.0)
        .erase();
        let styled = intrinsic(exact, &theme);
        let plain = intrinsic(
            single,
            &crate::Theme::Publication
                .resolve(&crate::font::canonical::OperationGenerationResources::new())
                .unwrap(),
        );
        assert_eq!(styled.1.to_bits(), plain.1.to_bits());
    }

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
