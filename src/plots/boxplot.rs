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
use crate::render::backend::{FillStyle, Point2D};
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
    hue_index: Option<usize>,
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
                hue_index: group.hue_index,
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
    let legend_entries =
        categorical_distribution::retained(chart)?.guide_entries(chart, theme, data)?;
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
        let color_index = stat.hue_index.unwrap_or(if group_active { 0 } else { i });
        let color = theme.colors[color_index % theme.colors.len()];
        let fill = FillStyle::new(color.into(), theme.foreground.into(), 1.0);
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
            let flier_style = chart.point_style(theme, theme.foreground, 2);
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
        for (index, group) in sample.groups.iter().enumerate() {
            let center = crate::plots::distribution_sample::center(group, group_levels.len());
            let color_index = group
                .hue_index
                .unwrap_or(if group_active { 0 } else { index });
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
                    theme.colors[color_index % theme.colors.len()],
                ),
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
