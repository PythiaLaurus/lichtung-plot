//! Quantile-quantile plots against the standard normal distribution.

#![allow(clippy::cast_precision_loss)]

use crate::Error;

use crate::layout::Layout;
use crate::render::backend::{with_clip, Backend, LineStyle, Point2D};
use crate::spec::{PlotRenderContext, PlotRenderResult, QqLine};
use crate::stats::distribution::{quantile, standard_normal_quantile};
use crate::Plot;

#[derive(Debug, Clone, Copy, PartialEq)]
struct LineGeometry {
    slope: f64,
    intercept: f64,
}

#[derive(Debug)]
struct PreparedQq {
    theoretical: Vec<f64>,
    observed: Vec<f64>,
    line: Option<LineGeometry>,
}

pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    if let Some(hue) = chart.hue_col() {
        return Err(Error::Data(format!(
            "Q-Q plot: hue column {hue:?} is not supported; render separate Q-Q plots"
        )));
    }
    let value_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("Q-Q plot: value column not set".into()))?;
    let values = crate::data::strict_finite_f64(context.data, value_col, "Q-Q plot")?;
    let prepared = prepare_qq(&values, chart.qq_line_spec())?;
    let theoretical_range = crate::data::min_max(prepared.theoretical.iter().copied(), "Q-Q plot")?;
    let observed_range = crate::data::min_max(prepared.observed.iter().copied(), "Q-Q plot")?;
    let x_range = qq_display_range(theoretical_range, chart.x_axis_spec(), "x-axis")?;
    let y_range = qq_display_range(observed_range, chart.y_axis_spec(), "y-axis")?;
    let x_axis_spec = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
    let y_axis_spec = crate::layout::display_axis_spec(chart.y_axis_spec(), y_range);
    let layout = Layout::new_with_axis_specs_in_panel_box(
        x_range,
        y_range,
        context.panel,
        chart
            .x_label_text()
            .unwrap_or("Theoretical normal quantiles")
            .to_string(),
        chart.y_label_text().unwrap_or(value_col).to_string(),
        &x_axis_spec,
        &y_axis_spec,
    );
    let palette = chart.palette();

    with_clip(backend, layout.projection.panel_box(), |backend| {
        if let Some(line) = prepared.line {
            if let Some(((line_x0, line_y0), (line_x1, line_y1))) = visible_line_segment(
                line,
                layout.projection.x_range(),
                layout.projection.y_range(),
            )? {
                let (x0, y0) = layout.projection.project(line_x0, line_y0);
                let (x1, y1) = layout.projection.project(line_x1, line_y1);
                backend.draw_line(
                    &[Point2D::new(x0, y0), Point2D::new(x1, y1)],
                    &LineStyle::solid(palette.foreground, 1.2),
                )?;
            }
        }
        let point_style = chart.point_style(palette.colors[0], 3);
        let points = prepared
            .theoretical
            .iter()
            .zip(&prepared.observed)
            .filter_map(|(theoretical, observed)| {
                if chart
                    .x_axis_spec()
                    .limits
                    .is_some_and(|(minimum, maximum)| {
                        *theoretical < minimum || *theoretical > maximum
                    })
                    || chart
                        .y_axis_spec()
                        .limits
                        .is_some_and(|(minimum, maximum)| {
                            *observed < minimum || *observed > maximum
                        })
                {
                    return None;
                }
                let (x, y) = layout.projection.project(*theoretical, *observed);
                (x.is_finite() && y.is_finite()).then_some(Point2D::new(x, y))
            })
            .collect::<Vec<_>>();
        backend.draw_scatter(&points, &point_style)?;
        Ok(())
    })?;

    Ok(PlotRenderResult::new(layout, Vec::new()))
}

fn qq_display_range(
    data_range: (f64, f64),
    spec: &crate::layout::AxisSpec,
    axis: &str,
) -> Result<(f64, f64), Error> {
    if let Some(limits) = spec.limits {
        return Ok(limits);
    }
    if data_range.0.total_cmp(&data_range.1).is_eq() {
        let value = data_range.0;
        let lower = value.next_down();
        let upper = value.next_up();
        if lower.is_finite() && lower < value {
            if upper.is_finite() && value < upper {
                return Ok((lower, upper));
            }
            return Ok((lower, value));
        }
        if upper.is_finite() && value < upper {
            return Ok((value, upper));
        }
        return Err(Error::Data(format!(
            "Q-Q plot: constant value {} cannot produce a finite representable automatic {axis} range; set explicit limits",
            data_range.0
        )));
    }
    let display = crate::layout::continuous_auto_range(data_range, spec);
    if display.0.is_finite() && display.1.is_finite() && display.0 < display.1 {
        Ok(display)
    } else if data_range.0.is_finite() && data_range.1.is_finite() && data_range.0 < data_range.1 {
        Ok(data_range)
    } else {
        Err(Error::Data(format!(
            "Q-Q plot: automatic {axis} range is not finite and ascending; set explicit limits"
        )))
    }
}

type LineSegment = ((f64, f64), (f64, f64));

fn visible_line_segment(
    line: LineGeometry,
    x_range: (f64, f64),
    y_range: (f64, f64),
) -> Result<Option<LineSegment>, Error> {
    if line.slope.abs() <= 1.0 {
        let first = (x_range.0, line.slope.mul_add(x_range.0, line.intercept));
        let second = (x_range.1, line.slope.mul_add(x_range.1, line.intercept));
        if !first.1.is_finite() || !second.1.is_finite() {
            return Err(reference_clip_error());
        }
        return Ok(clip_segment_by_coordinate(
            first,
            second,
            y_range,
            |point| point.1,
        ));
    }

    let first = (solve_line_x(line, y_range.0)?, y_range.0);
    let second = (solve_line_x(line, y_range.1)?, y_range.1);
    Ok(clip_segment_by_coordinate(
        first,
        second,
        x_range,
        |point| point.0,
    ))
}

fn solve_line_x(line: LineGeometry, y: f64) -> Result<f64, Error> {
    let direct = (y - line.intercept) / line.slope;
    if direct.is_finite() {
        return Ok(direct);
    }
    let scale = y.abs().max(line.intercept.abs()).max(f64::MIN_POSITIVE);
    let scaled = (y / scale - line.intercept / scale) * (scale / line.slope);
    if scaled.is_finite() {
        Ok(scaled)
    } else {
        Err(reference_clip_error())
    }
}

fn clip_segment_by_coordinate(
    first: (f64, f64),
    second: (f64, f64),
    range: (f64, f64),
    coordinate: impl Fn((f64, f64)) -> f64,
) -> Option<LineSegment> {
    let first_coordinate = coordinate(first);
    let second_coordinate = coordinate(second);
    if first_coordinate.total_cmp(&second_coordinate).is_eq() {
        return (first_coordinate >= range.0 && first_coordinate <= range.1)
            .then_some((first, second));
    }
    let lower_t = scaled_fraction(range.0, first_coordinate, second_coordinate);
    let upper_t = scaled_fraction(range.1, first_coordinate, second_coordinate);
    let start = lower_t.min(upper_t).clamp(0.0, 1.0);
    let end = lower_t.max(upper_t).clamp(0.0, 1.0);
    (start <= end
        && coordinate(interpolate_point(first, second, end)) >= range.0
        && coordinate(interpolate_point(first, second, start)) <= range.1)
        .then(|| {
            (
                interpolate_point(first, second, start),
                interpolate_point(first, second, end),
            )
        })
}

fn scaled_fraction(value: f64, first: f64, second: f64) -> f64 {
    let scale = value
        .abs()
        .max(first.abs())
        .max(second.abs())
        .max(f64::MIN_POSITIVE);
    (value / scale - first / scale) / (second / scale - first / scale)
}

fn interpolate_point(first: (f64, f64), second: (f64, f64), fraction: f64) -> (f64, f64) {
    (
        scaled_lerp(first.0, second.0, fraction),
        scaled_lerp(first.1, second.1, fraction),
    )
}

fn scaled_lerp(first: f64, second: f64, fraction: f64) -> f64 {
    let scale = first.abs().max(second.abs()).max(f64::MIN_POSITIVE);
    ((first / scale) * (1.0 - fraction) + (second / scale) * fraction) * scale
}

fn reference_clip_error() -> Error {
    Error::Data("Q-Q plot: reference line cannot be clipped to the displayed finite range".into())
}

fn prepare_qq(values: &[f64], line_mode: QqLine) -> Result<PreparedQq, Error> {
    if values.len() < 2 {
        return Err(Error::Data(format!(
            "Q-Q plot: at least 2 finite observations are required, got {}",
            values.len()
        )));
    }
    if values.iter().any(|value| !value.is_finite()) {
        return Err(Error::Data("Q-Q plot: observations must be finite".into()));
    }
    let mut observed = values.to_vec();
    observed.sort_by(f64::total_cmp);
    let sample_count = observed.len() as f64;
    let theoretical = (0..observed.len())
        .map(|index| standard_normal_quantile((index as f64 + 0.5) / sample_count))
        .collect::<Result<Vec<_>, _>>()?;
    let line = match line_mode {
        QqLine::Quartiles => {
            let theoretical_q1 = quantile(&theoretical, 0.25)?;
            let theoretical_q3 = quantile(&theoretical, 0.75)?;
            let observed_q1 = quantile(&observed, 0.25)?;
            let observed_q3 = quantile(&observed, 0.75)?;
            let denominator = finite_difference(theoretical_q3, theoretical_q1)
                .filter(|difference| *difference > 0.0)
                .ok_or_else(quartile_geometry_error)?;
            let slope = scaled_difference_ratio(observed_q3, observed_q1, denominator)
                .filter(|slope| slope.is_finite())
                .ok_or_else(quartile_geometry_error)?;
            let intercept = scaled_linear_combination(observed_q1, slope, -theoretical_q1)
                .ok_or_else(quartile_geometry_error)?;
            Some(LineGeometry { slope, intercept })
        }
        QqLine::Identity => Some(LineGeometry {
            slope: 1.0,
            intercept: 0.0,
        }),
        QqLine::Hidden => None,
    };
    Ok(PreparedQq {
        theoretical,
        observed,
        line,
    })
}

fn finite_difference(high: f64, low: f64) -> Option<f64> {
    let direct = high - low;
    if direct.is_finite() {
        return Some(direct);
    }
    let scale = high.abs().max(low.abs()).max(f64::MIN_POSITIVE);
    let difference = (high / scale - low / scale) * scale;
    difference.is_finite().then_some(difference)
}

fn scaled_difference_ratio(high: f64, low: f64, denominator: f64) -> Option<f64> {
    if !denominator.is_finite() || denominator <= 0.0 {
        return None;
    }
    let scale = high.abs().max(low.abs()).max(f64::MIN_POSITIVE);
    let ratio = ((high / scale - low / scale) / denominator) * scale;
    ratio.is_finite().then_some(ratio)
}

fn scaled_linear_combination(first: f64, second: f64, factor: f64) -> Option<f64> {
    let scale = first.abs().max(second.abs()).max(f64::MIN_POSITIVE);
    let result = (first / scale + (second / scale) * factor) * scale;
    result.is_finite().then_some(result)
}

fn quartile_geometry_error() -> Error {
    Error::Data(
        "Q-Q plot: quartile reference line geometry is not representable at this numeric scale; use QqLine::Hidden or QqLine::Identity, or explicitly preprocess/rescale the observations"
            .into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qq_sorts_observations_and_theoretical_quantiles() {
        let prepared = prepare_qq(&[3.0, 1.0, 2.0], QqLine::Hidden).unwrap();

        assert_eq!(prepared.observed, [1.0, 2.0, 3.0]);
        assert!(prepared
            .theoretical
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn qq_supports_two_observations() {
        let prepared = prepare_qq(&[2.0, 1.0], QqLine::Quartiles).unwrap();

        assert_eq!(prepared.theoretical.len(), 2);
        assert!(prepared.line.is_some());
    }

    #[test]
    fn qq_quartile_line_is_horizontal_for_constant_sample() {
        let prepared = prepare_qq(&[5.0, 5.0, 5.0], QqLine::Quartiles).unwrap();

        assert!(prepared.line.unwrap().slope.abs() < f64::EPSILON);
        assert!((prepared.line.unwrap().intercept - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn qq_identity_and_hidden_modes_are_explicit() {
        let identity = prepare_qq(&[1.0, 2.0], QqLine::Identity).unwrap();
        let hidden = prepare_qq(&[1.0, 2.0], QqLine::Hidden).unwrap();

        assert_eq!(
            identity.line,
            Some(LineGeometry {
                slope: 1.0,
                intercept: 0.0
            })
        );
        assert_eq!(hidden.line, None);
    }

    #[test]
    fn qq_rejects_too_few_observations() {
        let error = prepare_qq(&[1.0], QqLine::Quartiles).unwrap_err();

        assert!(error.to_string().contains("at least 2"));
    }

    #[test]
    fn qq_handles_large_finite_observations() {
        let prepared = prepare_qq(&[1e300, 1.1e300, 1.2e300], QqLine::Quartiles).unwrap();

        assert!(prepared.line.unwrap().slope.is_finite());
    }

    #[test]
    fn qq_constant_ranges_use_finite_neighbors_at_zero_and_extremes() {
        let spec = crate::layout::AxisSpec::default();

        assert_eq!(
            qq_display_range((0.0, 0.0), &spec, "y-axis").unwrap(),
            (0.0_f64.next_down(), 0.0_f64.next_up())
        );
        assert_eq!(
            qq_display_range((f64::MAX, f64::MAX), &spec, "y-axis").unwrap(),
            (f64::MAX.next_down(), f64::MAX)
        );
        assert_eq!(
            qq_display_range((-f64::MAX, -f64::MAX), &spec, "y-axis").unwrap(),
            (-f64::MAX, (-f64::MAX).next_up())
        );
    }

    #[test]
    fn qq_explicit_range_is_resolved_exactly() {
        let upper = f64::from_bits(1);
        let spec = crate::layout::AxisSpec::default().limits(0.0, upper);

        assert_eq!(
            qq_display_range((-10.0, 10.0), &spec, "y-axis").unwrap(),
            (0.0, upper)
        );
    }

    #[test]
    fn qq_opposite_extreme_auto_range_falls_back_to_exact_finite_endpoints() {
        let spec = crate::layout::AxisSpec::default();

        let range = qq_display_range((-f64::MAX, f64::MAX), &spec, "y-axis").unwrap();

        assert_eq!(range, (-f64::MAX, f64::MAX));
        assert_eq!(
            crate::layout::nice_ticks(range.0, range.1, 5),
            [range.0, range.1]
        );
    }

    #[test]
    fn reference_clipping_handles_horizontal_inside_and_outside_lines() {
        let inside = visible_line_segment(
            LineGeometry {
                slope: 0.0,
                intercept: 0.5,
            },
            (0.0, 1.0),
            (0.0, 1.0),
        )
        .unwrap();
        let outside = visible_line_segment(
            LineGeometry {
                slope: 0.0,
                intercept: 2.0,
            },
            (0.0, 1.0),
            (0.0, 1.0),
        )
        .unwrap();

        assert_eq!(inside, Some(((0.0, 0.5), (1.0, 0.5))));
        assert_eq!(outside, None);
    }

    #[test]
    fn reference_clipping_does_not_divide_by_subnormal_slope() {
        let segment = visible_line_segment(
            LineGeometry {
                slope: f64::from_bits(1),
                intercept: 0.5,
            },
            (-1.0, 1.0),
            (0.0, 1.0),
        )
        .unwrap();

        assert_eq!(segment, Some(((-1.0, 0.5), (1.0, 0.5))));
    }

    #[test]
    fn reference_clipping_handles_steep_and_partially_visible_lines() {
        let steep = visible_line_segment(
            LineGeometry {
                slope: 10.0,
                intercept: 0.0,
            },
            (-1.0, 1.0),
            (-1.0, 1.0),
        )
        .unwrap();
        let partial = visible_line_segment(
            LineGeometry {
                slope: 1.0,
                intercept: -0.5,
            },
            (0.0, 1.0),
            (0.0, 1.0),
        )
        .unwrap();

        assert_eq!(steep, Some(((-0.1, -1.0), (0.1, 1.0))));
        assert_eq!(partial, Some(((0.5, 0.0), (1.0, 0.5))));
    }

    #[test]
    fn reference_clipping_omits_nonhorizontal_line_outside_rectangle() {
        let segment = visible_line_segment(
            LineGeometry {
                slope: 1.0,
                intercept: 2.0,
            },
            (0.0, 1.0),
            (0.0, 1.0),
        )
        .unwrap();

        assert_eq!(segment, None);
    }

    #[test]
    fn qq_quartile_geometry_reports_unrepresentable_extreme_span() {
        let error = prepare_qq(
            &[-f64::MAX, -f64::MAX, f64::MAX, f64::MAX],
            QqLine::Quartiles,
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("quartile reference line geometry is not representable"));
        assert!(error.contains("QqLine::Hidden or QqLine::Identity"));
    }

    #[test]
    fn qq_quartile_geometry_scales_representable_opposite_sign_extremes() {
        let extreme = f64::MAX * 0.6;
        let mut values = vec![-extreme; 30];
        values.extend([0.0; 40]);
        values.extend([extreme; 30]);

        let observed_q1 = quantile(&values, 0.25).unwrap();
        let observed_q3 = quantile(&values, 0.75).unwrap();
        assert!((observed_q3 - observed_q1).is_infinite());

        let prepared = prepare_qq(&values, QqLine::Quartiles).unwrap();
        let line = prepared.line.unwrap();

        assert!(line.slope.is_finite() && line.intercept.is_finite());
    }
}
