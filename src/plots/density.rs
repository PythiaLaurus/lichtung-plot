//! Standalone Gaussian kernel density plots.

#![allow(clippy::cast_precision_loss)]

use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::layout::Layout;
use crate::render::backend::{with_clip, Backend, FillStyle, LineStyle, Point2D};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::stats::distribution::{gaussian_kde_with_bandwidth, strict_scott_bandwidth};
use crate::Plot;

const MIN_GRID_POINTS: usize = 128;
const MAX_GRID_POINTS: usize = 2_048;
const POINTS_PER_BANDWIDTH: f64 = 12.0;
const SUPPORT_BANDWIDTHS: f64 = 3.0;

#[derive(Debug)]
struct DensityCurve {
    hue: Option<String>,
    hue_index: Option<usize>,
    grid: Vec<f64>,
    densities: Vec<f64>,
}

#[derive(Debug)]
struct PreparedDensity {
    x_range: (f64, f64),
    curves: Vec<DensityCurve>,
}

pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    let value_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("density: value column not set".into()))?;
    let prepared = prepare_density_for_chart(chart, context.data, value_col)?;
    let x_range = prepared.x_range;
    let y_max = prepared
        .curves
        .iter()
        .flat_map(|curve| curve.densities.iter())
        .copied()
        .fold(0.0_f64, f64::max);
    let y_range = crate::layout::anchored_auto_range((0.0, y_max), chart.y_axis_spec(), 0.0);
    let x_axis_spec = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
    let y_axis_spec = crate::layout::display_axis_spec(chart.y_axis_spec(), y_range);
    let layout = Layout::new_with_axis_specs_in_panel_box(
        x_range,
        y_range,
        context.panel,
        chart.x_label_text().unwrap_or(value_col).to_string(),
        chart.y_label_text().unwrap_or("Density").to_string(),
        &x_axis_spec,
        &y_axis_spec,
    );
    let palette = chart.palette();

    with_clip(backend, layout.projection.panel_box(), |backend| {
        for curve in &prepared.curves {
            let visible = visible_curve(curve, layout.projection.x_range());
            if visible.len() < 2 {
                continue;
            }
            let color_index = curve.hue_index.unwrap_or(0);
            let color = palette.colors[color_index % palette.colors.len()];
            let mut fill = Vec::with_capacity(visible.len() + 2);
            fill.push((visible[0].0, 0.0));
            fill.extend(visible.iter().copied());
            fill.push((visible[visible.len() - 1].0, 0.0));
            let fill = clip_polygon_y(fill, layout.projection.y_range());
            if fill.len() >= 3 {
                let projected = project_points(&fill, &layout)?;
                backend.draw_polygon(
                    &projected,
                    &FillStyle {
                        color,
                        stroke: color,
                        stroke_width: 0.0,
                        opacity: 0.22,
                    },
                )?;
            }
            for pair in visible.windows(2) {
                if let Some(segment) = clip_line_y(pair[0], pair[1], layout.projection.y_range()) {
                    let projected = project_points(&segment, &layout)?;
                    backend.draw_line(&projected, &LineStyle::solid(color, 1.6))?;
                }
            }
        }
        Ok(())
    })?;

    let mut legend_hues = prepared
        .curves
        .iter()
        .filter_map(|curve| Some((curve.hue_index?, curve.hue.clone()?)))
        .collect::<Vec<_>>();
    legend_hues.sort_by_key(|(index, _)| *index);
    legend_hues.dedup_by(|left, right| left.1 == right.1);
    let legend_entries = legend_hues
        .into_iter()
        .map(|(index, label)| {
            LegendEntry::point(
                label,
                chart.point_style(palette.colors[index % palette.colors.len()], 3),
            )
        })
        .collect();
    Ok(PlotRenderResult::new(layout, legend_entries))
}

#[cfg(test)]
fn prepare_density<'a>(
    data: impl Into<crate::data::PlotData<'a>>,
    value_col: &str,
    group_col: Option<&str>,
    hue_col: Option<&str>,
) -> Result<PreparedDensity, Error> {
    prepare_density_ordered(data, value_col, group_col, hue_col, None, None)
}

fn prepare_density_for_chart<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
    value_col: &str,
) -> Result<PreparedDensity, Error> {
    let data = data.into();
    let observed_groups = chart
        .group_col()
        .map(|column| crate::data::strict_strings(data, column, "density"))
        .transpose()?
        .map_or_else(Vec::new, |values| crate::data::stable_levels(&values));
    let group_order = chart.resolved_group_levels(observed_groups);
    let hue_order = chart
        .hue_col()
        .map(|column| crate::data::strict_strings(data, column, "density"))
        .transpose()?
        .map_or_else(Vec::new, |values| crate::data::stable_levels(&values));
    let hue_order = chart.resolved_hue_levels(hue_order);
    prepare_density_ordered(
        data,
        value_col,
        chart.group_col(),
        chart.hue_col(),
        Some(&group_order),
        Some(&hue_order),
    )
}

#[expect(
    clippy::too_many_lines,
    reason = "density grouping, strict KDE validation, and support construction remain one preparation pass"
)]
fn prepare_density_ordered<'a>(
    data: impl Into<crate::data::PlotData<'a>>,
    value_col: &str,
    group_col: Option<&str>,
    hue_col: Option<&str>,
    group_order: Option<&[String]>,
    hue_order: Option<&[String]>,
) -> Result<PreparedDensity, Error> {
    let data = data.into();
    let values = crate::data::strict_finite_f64(data, value_col, "density")?;
    let channels = crate::data::collective_channels(data, group_col, hue_col, None, "density")?;
    if channels
        .groups
        .as_ref()
        .is_some_and(|groups| groups.len() != values.len())
    {
        return Err(Error::Data(
            "density: value, graphical group, and hue columns must have equal lengths".into(),
        ));
    }

    let mut groups: Vec<(Option<String>, Option<String>, Vec<f64>)> = Vec::new();
    for (index, value) in values.into_iter().enumerate() {
        let group = channels.groups.as_ref().map(|groups| groups[index].clone());
        let hue = channels.hues.as_ref().map(|hues| hues[index].clone());
        if let Some((_, _, group_values)) = groups.iter_mut().find(|(seen, _, _)| seen == &group) {
            group_values.push(value);
        } else {
            groups.push((group, hue, vec![value]));
        }
    }
    if let Some(order) = group_order {
        groups.sort_by_key(|(group, _, _)| {
            group
                .as_ref()
                .and_then(|group| order.iter().position(|level| level == group))
                .unwrap_or(usize::MAX)
        });
    }
    if groups.is_empty() {
        return Err(Error::Data(
            "density: value column contains no observations".into(),
        ));
    }

    let mut support_low = f64::INFINITY;
    let mut support_high = f64::NEG_INFINITY;
    let mut curves = Vec::with_capacity(groups.len());
    for (group, hue, samples) in &groups {
        let description = group.as_ref().map_or_else(
            || "ungrouped series".to_string(),
            |label| format!("graphical group {label:?}"),
        );
        if samples.len() < 2 {
            return Err(Error::Data(format!(
                "density: {description} requires at least 2 observations, got {}",
                samples.len()
            )));
        }
        let bandwidth = strict_scott_bandwidth(samples)
            .map_err(|reason| Error::Data(format!("density: {description} {reason}")))?;
        let (minimum, maximum) = crate::data::min_max(samples.iter().copied(), "density")?;
        let low = minimum - SUPPORT_BANDWIDTHS * bandwidth;
        let high = maximum + SUPPORT_BANDWIDTHS * bandwidth;
        if !low.is_finite() || !high.is_finite() || low >= high {
            return Err(Error::Data(format!(
                "density: {description} cannot produce finite KDE support"
            )));
        }
        support_low = support_low.min(low);
        support_high = support_high.max(high);
        let span_in_bandwidths = scaled_difference_ratio(high, low, bandwidth);
        if !span_in_bandwidths.is_finite() && !span_in_bandwidths.is_infinite() {
            return Err(Error::Data(format!(
                "density: {description} grid not representable at this numeric scale"
            )));
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let interval_count = (span_in_bandwidths * POINTS_PER_BANDWIDTH)
            .ceil()
            .clamp((MIN_GRID_POINTS - 1) as f64, (MAX_GRID_POINTS - 1) as f64)
            as usize;
        let step = scaled_difference_ratio(high, low, interval_count as f64);
        if !step.is_finite() || step <= 0.0 {
            return Err(Error::Data(format!(
                "density: {description} grid not representable at this numeric scale"
            )));
        }
        let grid = (0..=interval_count)
            .map(|index| scaled_lerp(low, high, index as f64 / interval_count as f64))
            .collect::<Vec<_>>();
        if grid.iter().any(|coordinate| !coordinate.is_finite())
            || grid.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(Error::Data(format!(
                "density: {description} grid not representable at this numeric scale"
            )));
        }
        let densities = gaussian_kde_with_bandwidth(samples, &grid, bandwidth)
            .map_err(|error| Error::Data(format!("density: {description}: {error}")))?;
        curves.push(DensityCurve {
            hue: hue.clone(),
            hue_index: hue.as_ref().and_then(|hue| {
                hue_order
                    .unwrap_or(&channels.hue_levels)
                    .iter()
                    .position(|level| level == hue)
            }),
            grid,
            densities,
        });
    }
    Ok(PreparedDensity {
        x_range: (support_low, support_high),
        curves,
    })
}

fn visible_curve(curve: &DensityCurve, x_range: (f64, f64)) -> Vec<(f64, f64)> {
    let first = curve.grid[0];
    let last = curve.grid[curve.grid.len() - 1];
    if x_range.1 < first || x_range.0 > last {
        return Vec::new();
    }
    let low = x_range.0.max(first);
    let high = x_range.1.min(last);
    let mut visible = Vec::new();
    visible.push((low, density_at(curve, low)));
    visible.extend(
        curve
            .grid
            .iter()
            .copied()
            .zip(curve.densities.iter().copied())
            .filter(|(x, _)| *x > low && *x < high),
    );
    if high > low {
        visible.push((high, density_at(curve, high)));
    }
    visible
}

fn density_at(curve: &DensityCurve, x: f64) -> f64 {
    match curve
        .grid
        .binary_search_by(|coordinate| coordinate.total_cmp(&x))
    {
        Ok(index) => curve.densities[index],
        Err(index) => {
            let left = index.saturating_sub(1);
            let right = index.min(curve.grid.len() - 1);
            let fraction = scaled_fraction(x, curve.grid[left], curve.grid[right]);
            scaled_lerp(curve.densities[left], curve.densities[right], fraction)
        }
    }
}

fn clip_line_y(
    first: (f64, f64),
    second: (f64, f64),
    y_range: (f64, f64),
) -> Option<[(f64, f64); 2]> {
    if (first.1 < y_range.0 && second.1 < y_range.0)
        || (first.1 > y_range.1 && second.1 > y_range.1)
    {
        return None;
    }
    let mut clipped = [first, second];
    if clipped[0].1 < y_range.0 || clipped[0].1 > y_range.1 {
        let boundary = clipped[0].1.clamp(y_range.0, y_range.1);
        let fraction = scaled_fraction(boundary, first.1, second.1);
        clipped[0] = (scaled_lerp(first.0, second.0, fraction), boundary);
    }
    if clipped[1].1 < y_range.0 || clipped[1].1 > y_range.1 {
        let boundary = clipped[1].1.clamp(y_range.0, y_range.1);
        let fraction = scaled_fraction(boundary, first.1, second.1);
        clipped[1] = (scaled_lerp(first.0, second.0, fraction), boundary);
    }
    Some(clipped)
}

fn clip_polygon_y(mut polygon: Vec<(f64, f64)>, y_range: (f64, f64)) -> Vec<(f64, f64)> {
    polygon = clip_polygon_boundary(polygon, y_range.0, true);
    clip_polygon_boundary(polygon, y_range.1, false)
}

fn clip_polygon_boundary(
    polygon: Vec<(f64, f64)>,
    boundary: f64,
    keep_above: bool,
) -> Vec<(f64, f64)> {
    let Some(mut previous) = polygon.last().copied() else {
        return polygon;
    };
    let mut output = Vec::new();
    for current in polygon {
        let previous_inside = if keep_above {
            previous.1 >= boundary
        } else {
            previous.1 <= boundary
        };
        let current_inside = if keep_above {
            current.1 >= boundary
        } else {
            current.1 <= boundary
        };
        if previous_inside != current_inside {
            let fraction = scaled_fraction(boundary, previous.1, current.1);
            output.push((scaled_lerp(previous.0, current.0, fraction), boundary));
        }
        if current_inside {
            output.push(current);
        }
        previous = current;
    }
    output
}

fn project_points(points: &[(f64, f64)], layout: &Layout) -> Result<Vec<Point2D>, Error> {
    points
        .iter()
        .map(|(x, y)| {
            let (x, y) = layout.projection.project(*x, *y);
            if x.is_finite() && y.is_finite() {
                Ok(Point2D::new(x, y))
            } else {
                Err(Error::Data(
                    "density: visible curve cannot be projected at this numeric scale".into(),
                ))
            }
        })
        .collect()
}

fn scaled_fraction(value: f64, first: f64, second: f64) -> f64 {
    let scale = value
        .abs()
        .max(first.abs())
        .max(second.abs())
        .max(f64::MIN_POSITIVE);
    (value / scale - first / scale) / (second / scale - first / scale)
}

fn scaled_lerp(first: f64, second: f64, fraction: f64) -> f64 {
    let scale = first.abs().max(second.abs()).max(f64::MIN_POSITIVE);
    ((first / scale) * (1.0 - fraction) + (second / scale) * fraction) * scale
}

fn scaled_difference_ratio(high: f64, low: f64, denominator: f64) -> f64 {
    let scale = high.abs().max(low.abs()).max(f64::MIN_POSITIVE);
    ((high / scale - low / scale) / denominator) * scale
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{into_column, Fixture};

    fn frame(values: &[f64], labels: Option<&[&str]>) -> Fixture {
        let mut columns = vec![("value", into_column(values.to_vec()))];
        if let Some(labels) = labels {
            columns.push(("group", into_column(labels.to_vec())));
        }
        Fixture::from_columns(columns).unwrap()
    }

    fn prepare_frame(data: &Fixture, grouped: bool, hued: bool) -> Result<PreparedDensity, Error> {
        let mut plot = Plot::density(crate::test_support::numeric(data, "value"));
        if grouped {
            plot = plot.group(crate::test_support::category(data, "group"));
        }
        if hued {
            plot = plot.hue(crate::test_support::category(data, "group"));
        }
        prepare_density(
            &plot,
            crate::data::X,
            grouped.then_some(crate::data::GROUP),
            hued.then_some(crate::data::HUE),
        )
    }

    #[test]
    fn grouped_density_preserves_hue_order_and_shared_axis() {
        let data = frame(
            &[0.0, 2.0, 1.0, 3.0, 4.0, 6.0],
            Some(&["b", "a", "b", "a", "b", "a"]),
        );

        let prepared = prepare_frame(&data, true, true).unwrap();

        assert_eq!(prepared.curves[0].hue.as_deref(), Some("b"));
        assert_eq!(prepared.curves[1].hue.as_deref(), Some("a"));
        assert_eq!(
            prepared.curves[0].densities.len(),
            prepared.curves[0].grid.len()
        );
        assert_eq!(
            prepared.curves[1].densities.len(),
            prepared.curves[1].grid.len()
        );
        assert!(prepared.x_range.0 <= prepared.curves[0].grid[0]);
        assert!(prepared.x_range.1 >= prepared.curves[1].grid[prepared.curves[1].grid.len() - 1]);
    }

    #[test]
    fn each_hue_density_integrates_independently() {
        let data = frame(
            &[0.0, 1.0, 2.0, 10.0, 11.0, 12.0],
            Some(&["a", "a", "a", "b", "b", "b"]),
        );
        let prepared = prepare_frame(&data, true, true).unwrap();
        let integrals = prepared
            .curves
            .iter()
            .map(curve_integral)
            .collect::<Vec<_>>();

        assert!((integrals[0] - 1.0).abs() < 0.01, "{integrals:?}");
        assert!((integrals[1] - 1.0).abs() < 0.01, "{integrals:?}");
    }

    #[test]
    fn density_support_uses_deterministic_three_bandwidth_tails() {
        let samples = [0.0, 1.0, 2.0];
        let data = frame(&samples, None);
        let bandwidth = strict_scott_bandwidth(&samples).unwrap();

        let prepared = prepare_frame(&data, false, false).unwrap();

        assert!((prepared.curves[0].grid[0] - (samples[0] - 3.0 * bandwidth)).abs() < 1e-12);
        assert!(
            (prepared.curves[0].grid[prepared.curves[0].grid.len() - 1]
                - (samples[2] + 3.0 * bandwidth))
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn separated_groups_use_local_bandwidth_resolving_grids_without_bridge() {
        let data = frame(
            &[0.0, 0.01, 0.02, 100.0, 110.0, 120.0],
            Some(&["sharp", "sharp", "sharp", "broad", "broad", "broad"]),
        );

        let prepared = prepare_frame(&data, true, true).unwrap();
        let sharp = &prepared.curves[0];
        let broad = &prepared.curves[1];

        assert!(sharp.grid[sharp.grid.len() - 1] < broad.grid[0]);
        assert!((curve_integral(sharp) - 1.0).abs() < 0.01);
        assert!((curve_integral(broad) - 1.0).abs() < 0.01);
        assert!(!(sharp.grid[1] - sharp.grid[0])
            .total_cmp(&(broad.grid[1] - broad.grid[0]))
            .is_eq());
    }

    #[test]
    fn density_rejects_small_group_with_label() {
        let data = frame(&[0.0, 1.0, 2.0], Some(&["small", "ok", "ok"]));

        let error = prepare_frame(&data, true, true).unwrap_err();

        assert!(error
            .to_string()
            .contains("graphical group \"small\" requires at least 2"));
    }

    #[test]
    fn density_rejects_constant_group_with_label() {
        let data = frame(&[1.0, 1.0, 2.0, 3.0], Some(&["flat", "flat", "ok", "ok"]));

        let error = prepare_frame(&data, true, true).unwrap_err();

        assert!(error
            .to_string()
            .contains("graphical group \"flat\" has constant"));
    }

    #[test]
    fn density_rejects_large_offset_grid_when_coordinates_cannot_be_distinct() {
        let base = 1e300_f64;
        let data = frame(&[base, base.next_up(), base.next_up().next_up()], None);

        let error = prepare_frame(&data, false, false).unwrap_err();

        assert!(error
            .to_string()
            .contains("series grid not representable at this numeric scale"));
    }

    #[test]
    fn density_large_offset_grid_is_finite_and_strictly_increasing_when_representable() {
        let base = 1e300_f64;
        let data = frame(&[base, base + 1e287, base + 2e287], None);

        let prepared = prepare_frame(&data, false, false).unwrap();

        assert!(prepared.curves[0]
            .grid
            .iter()
            .all(|value| value.is_finite()));
        assert!(prepared.curves[0]
            .grid
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn density_builds_broad_opposite_sign_grid_when_direct_support_span_overflows() {
        let extreme = f64::MAX * 0.2;
        let data = frame(&[-extreme, extreme], None);

        let prepared = prepare_frame(&data, false, false).unwrap();
        let curve = &prepared.curves[0];

        assert!((prepared.x_range.1 - prepared.x_range.0).is_infinite());
        assert!(curve.grid.iter().all(|value| value.is_finite()));
        assert!(curve.grid.windows(2).all(|pair| pair[0] < pair[1]));
        let maximum_density = curve.densities.iter().copied().fold(0.0, f64::max);
        assert!(maximum_density > 0.0 && maximum_density < 1e-294);
    }

    fn curve_integral(curve: &DensityCurve) -> f64 {
        curve
            .grid
            .windows(2)
            .zip(curve.densities.windows(2))
            .map(|(x, y)| (x[1] - x[0]) * (y[0] + y[1]) / 2.0)
            .sum()
    }
}
