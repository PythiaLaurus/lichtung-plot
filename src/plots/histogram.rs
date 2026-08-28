//! Single-series numeric histogram rendering and deterministic bin preparation.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use crate::{Error, HistogramKdePoint};

use crate::layout::{Layout, PanelBox};
use crate::render::backend::{with_clip, Backend, FillStyle, LineStyle, Point2D};
use crate::spec::{
    HistogramBins, HistogramComposite, HistogramKde, HistogramStat, PlotRenderContext,
    PlotRenderResult,
};
use crate::theme::ResolvedTheme;
use crate::Plot;

const MAX_AUTO_BINS: usize = 512;
const MAX_EXPLICIT_BINS: usize = 1_000_000;
const KDE_GRID_POINTS: usize = 128;

#[derive(Debug)]
struct PreparedHistogram {
    edges: Vec<f64>,
    heights: Vec<f64>,
}

pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    if let Some(input) = chart.histogram_composite_geometry_input() {
        return render_precomputed_composite(chart, input, context.panel, context.theme, backend);
    }
    if let Some(hue) = chart.hue_col() {
        return Err(Error::Data(format!(
            "histogram: hue column {hue:?} is not supported yet; render separate single-series histograms"
        )));
    }
    let value_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("histogram: value column not set".into()))?;
    let values = crate::data::strict_finite_f64(context.data, value_col, "histogram")?;
    let prepared = prepare_histogram(
        &values,
        chart.histogram_bins_spec(),
        chart.histogram_stat_spec(),
    )?;
    let x_range = (prepared.edges[0], prepared.edges[prepared.edges.len() - 1]);
    let density_curve = prepare_density_curve(chart.histogram_composite_spec(), &values, x_range)?;
    let mut y_max = prepared.heights.iter().copied().fold(0.0_f64, f64::max);
    if let Some(curve) = &density_curve {
        y_max = curve.iter().map(|point| point.y).fold(y_max, f64::max);
    }
    let y_range = crate::layout::anchored_auto_range((0.0, y_max), chart.y_axis_spec(), 0.0);
    let x_axis_spec = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
    let y_axis_spec = crate::layout::display_axis_spec(chart.y_axis_spec(), y_range);
    let layout = Layout::new_with_axis_specs_in_panel_box(
        x_range,
        y_range,
        context.panel,
        chart.x_label_text().unwrap_or(value_col).to_string(),
        chart
            .y_label_text()
            .unwrap_or(histogram_axis_label(chart.histogram_stat_spec()))
            .to_string(),
        &x_axis_spec,
        &y_axis_spec,
    );
    let theme = context.theme;
    let fill = FillStyle {
        color: theme.colors[0],
        stroke: theme.foreground,
        stroke_width: 0.8,
        opacity: 0.82,
    };

    with_clip(backend, layout.projection.panel_box(), |backend| {
        for (index, height) in prepared.heights.iter().enumerate() {
            let (left, bottom) = layout.projection.project(prepared.edges[index], 0.0);
            let (right, top) = layout
                .projection
                .project(prepared.edges[index + 1], *height);
            backend.draw_polygon(
                &[
                    Point2D::new(left, bottom),
                    Point2D::new(right, bottom),
                    Point2D::new(right, top),
                    Point2D::new(left, top),
                ],
                &fill,
            )?;
        }
        if let Some(curve) = &density_curve {
            let points = curve
                .iter()
                .map(|point| {
                    let (x, y) = layout.projection.project(point.x, point.y);
                    Point2D::new(x, y)
                })
                .collect::<Vec<_>>();
            backend.draw_line(&points, &LineStyle::solid(theme.foreground, 1.0))?;

            let rug_height = y_max * 0.035;
            let rug_style = LineStyle::solid(theme.foreground, 0.7);
            for value in &values {
                let (x, bottom) = layout.projection.project(*value, 0.0);
                let (_, top) = layout.projection.project(*value, rug_height);
                backend.draw_line(&[Point2D::new(x, bottom), Point2D::new(x, top)], &rug_style)?;
            }
        }
        Ok(())
    })?;

    Ok(PlotRenderResult::new(layout, Vec::new()))
}

fn render_precomputed_composite(
    chart: &Plot,
    input: &crate::HistogramGeometry,
    panel: PanelBox,
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    let x_range = input.validate()?;
    let y_max = input
        .bins
        .iter()
        .map(|bin| bin.density)
        .chain(input.kde.iter().map(|point| point.y))
        .chain(input.rugs.iter().map(|rug| rug.top))
        .fold(0.0_f64, f64::max);
    let y_range = crate::layout::anchored_auto_range((0.0, y_max), chart.y_axis_spec(), 0.0);
    let x_axis_spec = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
    let y_axis_spec = crate::layout::display_axis_spec(chart.y_axis_spec(), y_range);
    let layout = Layout::new_with_axis_specs_in_panel_box(
        x_range,
        y_range,
        panel,
        chart.x_label_text().unwrap_or("").to_string(),
        chart.y_label_text().unwrap_or("Density").to_string(),
        &x_axis_spec,
        &y_axis_spec,
    );
    let fill = FillStyle {
        color: theme.colors[0],
        stroke: theme.foreground,
        stroke_width: 0.8,
        opacity: 0.82,
    };

    with_clip(backend, layout.projection.panel_box(), |backend| {
        for bin in &input.bins {
            let (left, bottom) = layout.projection.project(bin.lower, 0.0);
            let (right, top) = layout.projection.project(bin.upper, bin.density);
            backend.draw_polygon(
                &[
                    Point2D::new(left, bottom),
                    Point2D::new(right, bottom),
                    Point2D::new(right, top),
                    Point2D::new(left, top),
                ],
                &fill,
            )?;
        }
        let curve = input
            .kde
            .iter()
            .map(|point| {
                let (x, y) = layout.projection.project(point.x, point.y);
                Point2D::new(x, y)
            })
            .collect::<Vec<_>>();
        backend.draw_line(&curve, &LineStyle::solid(theme.foreground, 1.0))?;

        let rug_style = LineStyle::solid(theme.foreground, 0.7);
        for rug in &input.rugs {
            let (x, baseline) = layout.projection.project(rug.x, rug.baseline);
            let (_, top) = layout.projection.project(rug.x, rug.top);
            backend.draw_line(
                &[Point2D::new(x, baseline), Point2D::new(x, top)],
                &rug_style,
            )?;
        }
        Ok(())
    })?;

    Ok(PlotRenderResult::new(layout, Vec::new()))
}

fn prepare_density_curve(
    composite: &HistogramComposite,
    values: &[f64],
    x_range: (f64, f64),
) -> Result<Option<Vec<HistogramKdePoint>>, Error> {
    let HistogramComposite::KdeAndRug(kde) = composite else {
        return Ok(None);
    };
    let curve = match kde {
        HistogramKde::Internal => {
            let bandwidth = crate::stats::distribution::strict_scott_bandwidth(values)
                .map_err(|reason| Error::Data(format!("histogram KDE: {reason}")))?;
            let grid = (0..KDE_GRID_POINTS)
                .map(|index| {
                    x_range.0
                        + (x_range.1 - x_range.0) * index as f64 / (KDE_GRID_POINTS - 1) as f64
                })
                .collect::<Vec<_>>();
            let densities =
                crate::stats::distribution::gaussian_kde_with_bandwidth(values, &grid, bandwidth)
                    .map_err(|error| Error::Data(format!("histogram KDE: {error}")))?;
            grid.into_iter()
                .zip(densities)
                .map(|(x, y)| HistogramKdePoint { x, y })
                .collect()
        }
        HistogramKde::Precomputed(points) => {
            validate_precomputed_kde(points, x_range)?;
            points.clone()
        }
    };
    Ok(Some(curve))
}

fn validate_precomputed_kde(
    points: &[HistogramKdePoint],
    x_range: (f64, f64),
) -> Result<(), Error> {
    if points.len() < 2 {
        return Err(Error::Data(
            "histogram KDE: a precomputed curve requires at least two points".into(),
        ));
    }
    for (index, point) in points.iter().enumerate() {
        if !point.x.is_finite() || !point.y.is_finite() || point.y < 0.0 {
            return Err(Error::Data(format!(
                "histogram KDE: precomputed point {index} must contain finite x and nonnegative finite density"
            )));
        }
        if !(x_range.0..=x_range.1).contains(&point.x) {
            return Err(Error::Data(format!(
                "histogram KDE: precomputed x {} at point {index} lies outside histogram bin range {}..{}",
                point.x,
                x_range.0, x_range.1
            )));
        }
        if index > 0 && point.x <= points[index - 1].x {
            return Err(Error::Data(format!(
                "histogram KDE: precomputed x values must be strictly increasing; point {index} is {}",
                point.x
            )));
        }
    }
    if !points.iter().any(|point| point.y > 0.0) {
        return Err(Error::Data(
            "histogram KDE: a precomputed density curve must contain a positive density".into(),
        ));
    }
    Ok(())
}

fn prepare_histogram(
    values: &[f64],
    bins: HistogramBins,
    stat: HistogramStat,
) -> Result<PreparedHistogram, Error> {
    if values.is_empty() {
        return Err(Error::Data(
            "histogram: value column contains no observations".into(),
        ));
    }
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(Error::Data(format!(
                "histogram: non-finite value {value} at row {index}"
            )));
        }
    }
    validate_bins(bins)?;
    let (minimum, maximum) = crate::data::min_max(values.iter().copied(), "histogram")?;
    if minimum.total_cmp(&maximum).is_eq() {
        let edges = constant_edges(minimum, bins)?;
        return finish_histogram(values, edges, stat);
    }
    let range = maximum - minimum;
    if !range.is_finite() || range <= 0.0 {
        return Err(Error::Data(format!(
            "histogram: numeric range {minimum}..{maximum} is too large to bin safely"
        )));
    }

    match bins {
        HistogramBins::Auto => {
            let count = automatic_bin_count(values, range)?;
            let edges = equal_edges(minimum, maximum, count)?;
            finish_histogram(values, edges, stat)
        }
        HistogramBins::Count(count) => {
            let edges = equal_edges(minimum, maximum, count)?;
            finish_histogram(values, edges, stat)
        }
        HistogramBins::Width(width) => {
            let edges = requested_width_edges(minimum, maximum, width)?;
            finish_histogram(values, edges, stat)
        }
    }
}

fn validate_bins(bins: HistogramBins) -> Result<(), Error> {
    match bins {
        HistogramBins::Count(0) => Err(Error::Data(
            "histogram: bin count must be greater than zero".into(),
        )),
        HistogramBins::Count(count) if count > MAX_EXPLICIT_BINS => Err(Error::Data(format!(
            "histogram: bin count {count} exceeds the safety limit of {MAX_EXPLICIT_BINS}"
        ))),
        HistogramBins::Width(width) if !width.is_finite() || width <= 0.0 => Err(Error::Data(
            format!("histogram: bin width must be positive and finite; got {width}"),
        )),
        HistogramBins::Auto | HistogramBins::Count(_) | HistogramBins::Width(_) => Ok(()),
    }
}

fn automatic_bin_count(values: &[f64], range: f64) -> Result<usize, Error> {
    let q1 = crate::stats::distribution::quantile(values, 0.25)?;
    let q3 = crate::stats::distribution::quantile(values, 0.75)?;
    let iqr = q3 - q1;
    let width = 2.0 * iqr * (values.len() as f64).powf(-1.0 / 3.0);
    if width.is_finite() && width > 0.0 {
        let raw_count = range / width;
        let count = if raw_count.is_finite() {
            raw_count.ceil().max(1.0) as usize
        } else {
            MAX_AUTO_BINS
        };
        return Ok(count.clamp(1, MAX_AUTO_BINS));
    }
    Ok(sturges_bin_count(values.len()).min(MAX_AUTO_BINS))
}

fn sturges_bin_count(sample_count: usize) -> usize {
    ((sample_count as f64).log2() + 1.0).ceil().max(1.0) as usize
}

fn constant_edges(value: f64, bins: HistogramBins) -> Result<Vec<f64>, Error> {
    match bins {
        HistogramBins::Auto => constant_auto_edges(value),
        HistogramBins::Count(count) => constant_count_edges(value, count),
        HistogramBins::Width(width) => constant_width_edges(value, width),
    }
}

fn constant_auto_edges(value: f64) -> Result<Vec<f64>, Error> {
    let preferred = [value - 0.5, value + 0.5];
    if valid_edges(&preferred) {
        return Ok(preferred.to_vec());
    }
    let adjacent = [value.next_down(), value.next_up()];
    if valid_edges(&adjacent) {
        return Ok(adjacent.to_vec());
    }
    Err(Error::Data(format!(
        "histogram: constant value {value} is too large to construct a finite centered automatic bin"
    )))
}

fn constant_count_edges(value: f64, count: usize) -> Result<Vec<f64>, Error> {
    let lower_ulp = value - value.next_down();
    let upper_ulp = value.next_up() - value;
    let required_padding = lower_ulp.max(upper_ulp) * count as f64 / 2.0;
    let mut padding = 0.5_f64.max(required_padding);
    for _ in 0..8 {
        let lower = value - padding;
        let upper = value + padding;
        if let Some(edges) = try_equal_edges(lower, upper, count) {
            return Ok(edges);
        }
        padding *= 2.0;
    }
    Err(Error::Data(format!(
        "histogram: Count({count}) cannot produce distinct finite centered bins around constant value {value}; use fewer bins"
    )))
}

fn constant_width_edges(value: f64, width: f64) -> Result<Vec<f64>, Error> {
    let half_width = width / 2.0;
    let edges = [value - half_width, value + half_width];
    if half_width > 0.0 && edges[0] < value && edges[1] > value && valid_edges(&edges) {
        return Ok(edges.to_vec());
    }
    Err(Error::Data(format!(
        "histogram: Width({width}) cannot produce distinct finite centered edges around constant value {value}; use a wider bin"
    )))
}

fn equal_edges(lower: f64, upper: f64, count: usize) -> Result<Vec<f64>, Error> {
    try_equal_edges(lower, upper, count).ok_or_else(|| {
        Error::Data(
            "histogram: equal-width bin edges are not distinct and finite at this numeric scale; use fewer bins or a wider range"
                .into(),
        )
    })
}

fn try_equal_edges(lower: f64, upper: f64, count: usize) -> Option<Vec<f64>> {
    if count == 0 || count > MAX_EXPLICIT_BINS || !lower.is_finite() || !upper.is_finite() {
        return None;
    }
    let range = upper - lower;
    let width = range / count as f64;
    if !range.is_finite() || !width.is_finite() || width <= 0.0 {
        return None;
    }
    let mut edges = Vec::with_capacity(count + 1);
    for index in 0..=count {
        edges.push(lower + width * index as f64);
    }
    edges[count] = upper;
    valid_edges(&edges).then_some(edges)
}

fn requested_width_edges(minimum: f64, maximum: f64, width: f64) -> Result<Vec<f64>, Error> {
    let range = maximum - minimum;
    let first_upper = minimum + width;
    if !first_upper.is_finite() || first_upper <= minimum {
        return Err(Error::Data(format!(
            "histogram: width {width} cannot produce a distinct finite edge above minimum {minimum}; use a wider bin or Count"
        )));
    }
    let raw_count = range / width;
    if !raw_count.is_finite() || raw_count.ceil() > MAX_EXPLICIT_BINS as f64 + 1.0 {
        return Err(Error::Data(
            format!(
                "histogram: width {width} would require too many bins for range {minimum}..{maximum}; use a larger width or Count"
            ),
        ));
    }
    let mut count = raw_count.ceil().max(1.0) as usize;

    while count > 1 {
        let prior = minimum + width * (count - 1) as f64;
        if prior.is_finite() && prior >= maximum {
            count -= 1;
        } else {
            break;
        }
    }
    loop {
        let upper = minimum + width * count as f64;
        if !upper.is_finite() {
            return Err(Error::Data(format!(
                "histogram: width {width} cannot produce a finite upper edge from minimum {minimum}; use a smaller width or Count"
            )));
        }
        if upper >= maximum {
            break;
        }
        count = count.checked_add(1).ok_or_else(|| {
            Error::Data("histogram: bin count overflow for requested width".into())
        })?;
        if count > MAX_EXPLICIT_BINS {
            return Err(Error::Data(format!(
                "histogram: width {width} requires more than {MAX_EXPLICIT_BINS} bins; use a larger width or Count"
            )));
        }
    }
    if count > MAX_EXPLICIT_BINS {
        return Err(Error::Data(format!(
            "histogram: width {width} requires {count} bins, exceeding the safety limit of {MAX_EXPLICIT_BINS}"
        )));
    }

    let mut edges = Vec::with_capacity(count + 1);
    for index in 0..=count {
        edges.push(minimum + width * index as f64);
    }
    if !valid_edges(&edges) {
        return Err(Error::Data(format!(
            "histogram: width {width} cannot produce distinct finite edges at this numeric scale; use a wider bin or Count"
        )));
    }
    Ok(edges)
}

fn valid_edges(edges: &[f64]) -> bool {
    edges.len() >= 2
        && edges.iter().all(|edge| edge.is_finite())
        && edges.windows(2).all(|window| window[1] > window[0])
}

fn finish_histogram(
    values: &[f64],
    edges: Vec<f64>,
    stat: HistogramStat,
) -> Result<PreparedHistogram, Error> {
    if !valid_edges(&edges) {
        return Err(Error::Data(
            "histogram: computed bin edges are not distinct and finite".into(),
        ));
    }
    let count = edges.len() - 1;
    let lower = edges[0];
    let upper = edges[count];

    let mut counts = vec![0_usize; count];
    for value in values {
        if *value < lower || *value > upper {
            return Err(Error::Data(format!(
                "histogram: value {value} falls outside computed edges {lower}..{upper}"
            )));
        }
        let insertion = edges.partition_point(|edge| edge <= value);
        let index = insertion.saturating_sub(1).min(count - 1);
        counts[index] += 1;
    }

    let sample_count = values.len() as f64;
    let mut heights = Vec::with_capacity(count);
    for (index, count) in counts.into_iter().enumerate() {
        let actual_width = edges[index + 1] - edges[index];
        let height = match stat {
            HistogramStat::Count => count as f64,
            HistogramStat::Probability => count as f64 / sample_count,
            HistogramStat::Density => (count as f64 / sample_count) / actual_width,
        };
        if !height.is_finite() {
            let message = match stat {
                HistogramStat::Density => format!(
                    "histogram: density overflow for representable bin width {actual_width}; use wider or fewer bins"
                ),
                HistogramStat::Count | HistogramStat::Probability => {
                    "histogram: normalized bin height is not finite".to_string()
                }
            };
            return Err(Error::Data(message));
        }
        heights.push(height);
    }
    Ok(PreparedHistogram { edges, heights })
}

const fn histogram_axis_label(stat: HistogramStat) -> &'static str {
    match stat {
        HistogramStat::Count => "Count",
        HistogramStat::Probability => "Probability",
        HistogramStat::Density => "Density",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_uses_freedman_diaconis_for_nonzero_iqr() {
        let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];

        let histogram =
            prepare_histogram(&values, HistogramBins::Auto, HistogramStat::Count).unwrap();

        assert_eq!(histogram.heights.len(), 2);
    }

    #[test]
    fn auto_falls_back_to_sturges_when_iqr_is_zero() {
        let values = [0.0, 0.0, 0.0, 0.0, 1.0];

        let histogram =
            prepare_histogram(&values, HistogramBins::Auto, HistogramStat::Count).unwrap();

        assert_eq!(histogram.heights.len(), 4);
    }

    #[test]
    fn auto_caps_excessive_freedman_diaconis_bins() {
        let mut values = (0..100)
            .map(|index| f64::from(index) / 100.0)
            .collect::<Vec<_>>();
        values.push(1e12);

        let histogram =
            prepare_histogram(&values, HistogramBins::Auto, HistogramStat::Count).unwrap();

        assert_eq!(histogram.heights.len(), MAX_AUTO_BINS);
    }

    #[test]
    fn constant_auto_uses_numpy_like_centered_bin() {
        let histogram =
            prepare_histogram(&[5.0, 5.0, 5.0], HistogramBins::Auto, HistogramStat::Count).unwrap();

        assert_eq!(histogram.heights, [3.0]);
        assert_eq!(histogram.edges, [4.5, 5.5]);
    }

    #[test]
    fn constant_count_produces_requested_centered_bins() {
        let histogram = prepare_histogram(
            &[5.0, 5.0, 5.0],
            HistogramBins::Count(8),
            HistogramStat::Count,
        )
        .unwrap();

        assert_eq!(histogram.edges.len(), 9);
        assert_eq!(histogram.heights, [0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn constant_count_expands_to_representable_edges_at_large_scale() {
        let histogram =
            prepare_histogram(&[1e16, 1e16], HistogramBins::Count(4), HistogramStat::Count)
                .unwrap();

        assert_eq!(histogram.edges.len(), 5);
        assert!(histogram
            .edges
            .windows(2)
            .all(|window| window[1] > window[0]));
    }

    #[test]
    fn constant_width_uses_requested_centered_geometry() {
        let histogram =
            prepare_histogram(&[5.0, 5.0], HistogramBins::Width(2.0), HistogramStat::Count)
                .unwrap();

        assert_eq!(histogram.edges, [4.0, 6.0]);
        assert_eq!(histogram.heights, [2.0]);
    }

    #[test]
    fn constant_width_rejects_unrepresentable_geometry() {
        let error = prepare_histogram(
            &[1e16, 1e16],
            HistogramBins::Width(1.0),
            HistogramStat::Count,
        )
        .unwrap_err();

        assert!(error.to_string().contains("use a wider bin"));
    }

    #[test]
    fn internal_edges_are_half_open_and_maximum_uses_final_bin() {
        let histogram = prepare_histogram(
            &[0.0, 1.0, 2.0],
            HistogramBins::Count(2),
            HistogramStat::Count,
        )
        .unwrap();

        assert_eq!(histogram.heights, [1.0, 2.0]);
    }

    #[test]
    fn negative_offset_internal_edges_use_authoritative_edge_order() {
        let histogram = finish_histogram(
            &[-0.3, -0.2, -0.1, 0.0],
            vec![-0.3, -0.2, -0.1, 0.0],
            HistogramStat::Count,
        )
        .unwrap();

        assert_eq!(histogram.heights, [1.0, 1.0, 2.0]);
    }

    #[test]
    fn values_around_internal_edge_follow_half_open_ulp_boundaries() {
        let edge = 1.0_f64;
        let histogram = finish_histogram(
            &[0.0, edge.next_down(), edge, edge.next_up(), 2.0],
            vec![0.0, edge, 2.0],
            HistogramStat::Count,
        )
        .unwrap();

        assert_eq!(histogram.heights, [2.0, 3.0]);
    }

    #[test]
    fn probability_normalization_sums_to_one() {
        let histogram = prepare_histogram(
            &[0.0, 1.0, 2.0, 3.0],
            HistogramBins::Count(2),
            HistogramStat::Probability,
        )
        .unwrap();

        assert!((histogram.heights.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn density_normalization_integrates_to_one() {
        let histogram = prepare_histogram(
            &[0.0, 1.0, 2.0, 3.0],
            HistogramBins::Count(2),
            HistogramStat::Density,
        )
        .unwrap();
        let integral = density_integral(&histogram);

        assert!((integral - 1.0).abs() < 1e-12);
    }

    #[test]
    fn density_uses_each_uneven_representable_width_at_large_scale() {
        let base = 1e16_f64;
        let histogram = prepare_histogram(
            &[base, base + 2.0, base + 4.0, base + 6.0, base + 8.0],
            HistogramBins::Count(3),
            HistogramStat::Density,
        )
        .unwrap();
        let widths = histogram
            .edges
            .windows(2)
            .map(|window| window[1] - window[0])
            .collect::<Vec<_>>();

        assert_eq!(widths, [2.0, 4.0, 2.0]);
        assert!((density_integral(&histogram) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn requested_width_removes_quotient_rounding_extra_bin() {
        let histogram = prepare_histogram(
            &[0.0, 0.009_000_000_000_000_001],
            HistogramBins::Width(0.003),
            HistogramStat::Count,
        )
        .unwrap();

        assert_eq!(histogram.heights.len(), 3);
    }

    #[test]
    fn requested_width_keeps_minimum_bins_just_below_boundary() {
        let boundary = 0.003_f64 * 3.0;
        let histogram = prepare_histogram(
            &[0.0, boundary.next_down()],
            HistogramBins::Width(0.003),
            HistogramStat::Count,
        )
        .unwrap();

        assert_eq!(histogram.heights.len(), 3);
    }

    #[test]
    fn requested_width_adds_bin_just_above_boundary() {
        let boundary = 0.003_f64 * 3.0;
        let histogram = prepare_histogram(
            &[0.0, boundary.next_up()],
            HistogramBins::Width(0.003),
            HistogramStat::Count,
        )
        .unwrap();

        assert_eq!(histogram.heights.len(), 4);
    }

    #[test]
    fn invalid_count_and_width_are_rejected() {
        assert!(
            prepare_histogram(&[0.0, 1.0], HistogramBins::Count(0), HistogramStat::Count).is_err()
        );
        assert!(prepare_histogram(
            &[0.0, 1.0],
            HistogramBins::Width(f64::NAN),
            HistogramStat::Count
        )
        .is_err());
        assert!(
            prepare_histogram(&[0.0, 1.0], HistogramBins::Width(0.0), HistogramStat::Count)
                .is_err()
        );
    }

    #[test]
    fn unsafe_bin_allocations_and_extreme_ranges_are_rejected() {
        assert!(prepare_histogram(
            &[0.0, 1.0],
            HistogramBins::Count(MAX_EXPLICIT_BINS + 1),
            HistogramStat::Count
        )
        .is_err());
        assert!(prepare_histogram(
            &[0.0, 1.0],
            HistogramBins::Width(f64::MIN_POSITIVE),
            HistogramStat::Count
        )
        .is_err());
        assert!(prepare_histogram(
            &[-f64::MAX, f64::MAX],
            HistogramBins::Count(2),
            HistogramStat::Count
        )
        .is_err());
    }

    #[test]
    fn very_wide_density_bin_normalizes_without_denominator_overflow() {
        let histogram = prepare_histogram(
            &[0.0, 1.0],
            HistogramBins::Width(f64::MAX),
            HistogramStat::Density,
        )
        .unwrap();

        assert!((histogram.heights[0] * f64::MAX - 1.0).abs() < 1e-12);
    }

    #[test]
    fn subnormal_density_width_returns_actionable_overflow_error() {
        let smallest = f64::from_bits(1);
        let error = prepare_histogram(
            &[0.0, smallest],
            HistogramBins::Count(1),
            HistogramStat::Density,
        )
        .unwrap_err();

        assert!(error.to_string().contains("use wider or fewer bins"));
    }

    #[test]
    fn empty_and_nonfinite_values_are_rejected() {
        assert!(prepare_histogram(&[], HistogramBins::Auto, HistogramStat::Count).is_err());
        assert!(prepare_histogram(
            &[0.0, f64::INFINITY],
            HistogramBins::Auto,
            HistogramStat::Count
        )
        .is_err());
        assert!(
            prepare_histogram(&[0.0, f64::NAN], HistogramBins::Auto, HistogramStat::Count).is_err()
        );
    }

    fn density_integral(histogram: &PreparedHistogram) -> f64 {
        histogram
            .heights
            .iter()
            .zip(histogram.edges.windows(2))
            .map(|(height, edge)| height * (edge[1] - edge[0]))
            .sum()
    }
}
