//! Scatter with a generated simple OLS fit and optional classical confidence band.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::similar_names
)]

use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::render::backend::{with_clip, Backend, FillStyle, LineStyle, Point2D};
use crate::spec::{PlotRenderContext, PlotRenderResult, RegressionBand};
use crate::stats::regression::OlsFit;
use crate::Plot;

const BAND_POINTS: usize = 40;

#[expect(
    clippy::cast_precision_loss,
    reason = "fixed regression band sample count"
)]
pub(crate) fn axis_ranges(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<[(f64, f64); 2], Error> {
    let points = crate::data::aligned_finite_f64(
        data,
        chart
            .x_col()
            .ok_or_else(|| Error::Data("regression: x column not set".into()))?,
        chart
            .y_col()
            .ok_or_else(|| Error::Data("regression: y column not set".into()))?,
        "regression",
    )?;
    let mut x = Vec::new();
    let mut y = Vec::new();
    data.reserve_source(&mut x, points.len())?;
    data.reserve_source(&mut y, points.len())?;
    for point in points {
        x.push(point.0);
        y.push(point.1);
    }
    // OlsFit has no dynamic scratch: finite validation, scale, mean, centered
    // products and residuals each visit both channels once (ten scalar scans).
    for _ in 0..10 {
        data.claim_source_visits(x.len())?;
    }
    let fit = OlsFit::new(&x, &y)?;
    data.claim_source_visits(x.len())?;
    let xr = crate::data::min_max(x.iter().copied(), "regression")?;
    data.claim_source_visits(y.len())?;
    let mut yr = crate::data::min_max(y.iter().copied(), "regression")?;
    for x in [xr.0, xr.1] {
        data.claim_source_visits(1)?;
        let value = fit.predict(x)?;
        yr = (yr.0.min(value), yr.1.max(value));
    }
    if chart.regression_band_spec() == RegressionBand::PointwiseMean95 {
        for index in 0..BAND_POINTS {
            data.claim_source_visits(1)?;
            let fraction = index as f64 / (BAND_POINTS - 1) as f64;
            let interval =
                fit.pointwise_mean_response_95(xr.0 * (1.0 - fraction) + xr.1 * fraction)?;
            yr = (yr.0.min(interval.lower), yr.1.max(interval.upper));
        }
    }
    Ok([
        crate::layout::continuous_auto_range(xr, chart.x_axis_spec()),
        crate::layout::continuous_auto_range(yr, chart.y_axis_spec()),
    ])
}

#[cfg(test)]
#[test]
fn regression_axis_channel_and_fit_work_is_admitted_before_preparation() {
    use crate::font::canonical::OperationGenerationResources;
    let chart = Plot::regression(
        crate::NumericValues::new("x", [1.0, 2.0, 3.0]),
        crate::NumericValues::new("y", [1.0, 4.0, 2.0]),
    )
    .erase();
    let expected = axis_ranges(&chart, chart.data()).unwrap();
    let resources = OperationGenerationResources::new();
    let data = chart.data().with_axis_resources(&resources);
    let points =
        crate::data::aligned_finite_f64(data, crate::data::X, crate::data::Y, "regression")
            .unwrap();
    let pair_bytes = resources.byte_usage().unwrap();
    let pair_visits = resources.usage().unwrap().1;
    let mut x: Vec<f64> = Vec::new();
    let mut y: Vec<f64> = Vec::new();
    data.reserve_source(&mut x, points.len()).unwrap();
    data.reserve_source(&mut y, points.len()).unwrap();
    let channel_visits = resources.usage().unwrap().1;
    for (visits, bytes, message) in [
        (u128::MAX, pair_bytes, "storage admission"),
        (pair_visits, u64::MAX, "work admission"),
        (channel_visits, u64::MAX, "work admission"),
    ] {
        let resources = OperationGenerationResources::with_test_limits(visits, bytes);
        let error = axis_ranges(&chart, chart.data().with_axis_resources(&resources)).unwrap_err();
        assert!(error.to_string().contains(message), "{error}");
    }
    let resources = OperationGenerationResources::new();
    assert_eq!(
        axis_ranges(&chart, chart.data().with_axis_resources(&resources)).unwrap(),
        expected
    );
}

#[expect(
    clippy::too_many_lines,
    reason = "the renderer keeps fit, band, projection, marks, and provenance in one ordered lifecycle"
)]
pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let theme = context.theme;
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("regression: x column not set".into()))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("regression: y column not set".into()))?;
    let points = crate::data::aligned_finite_f64(context.data, x_col, y_col, "regression")?;
    let (x, y): (Vec<f64>, Vec<f64>) = points.into_iter().unzip();
    let band = chart.regression_band_spec();
    let minimum = match band {
        RegressionBand::PointwiseMean95 => 3,
        RegressionBand::Hidden => 2,
    };
    if x.len() < minimum {
        return Err(Error::Data(format!(
            "regression: {band:?} requires at least {minimum} points, got {}",
            x.len()
        ))
        .into());
    }

    let fit = OlsFit::new(&x, &y)?;
    let (x_min, x_max) = crate::data::min_max(x.iter().copied(), "regression")?;
    let y_fit_lo = fit.predict(x_min)?;
    let y_fit_hi = fit.predict(x_max)?;
    let panel = context.panel;

    let band_values = if band == RegressionBand::PointwiseMean95 {
        let mut values = Vec::with_capacity(BAND_POINTS);
        for index in 0..BAND_POINTS {
            let fraction = index as f64 / (BAND_POINTS - 1) as f64;
            let x_value = x_min * (1.0 - fraction) + x_max * fraction;
            let interval = fit.pointwise_mean_response_95(x_value)?;
            values.push((x_value, interval.lower, interval.upper));
        }
        Some(values)
    } else {
        None
    };

    let layout = context.axis_layout(|| {
        let (mut y_min, mut y_max) = crate::data::min_max(y.iter().copied(), "regression")?;
        y_min = y_min.min(y_fit_lo).min(y_fit_hi);
        y_max = y_max.max(y_fit_lo).max(y_fit_hi);
        if let Some(values) = &band_values {
            for (_, lower, upper) in values {
                y_min = y_min.min(*lower);
                y_max = y_max.max(*upper);
            }
        }

        let x_range = crate::layout::continuous_auto_range((x_min, x_max), chart.x_axis_spec());
        let y_range = crate::layout::continuous_auto_range((y_min, y_max), chart.y_axis_spec());
        let x_axis_spec = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
        let y_axis_spec = crate::layout::display_axis_spec(chart.y_axis_spec(), y_range);
        context.layout_with_axis_specs(
            x_range,
            y_range,
            panel,
            chart.x_label_text().unwrap_or("").to_string(),
            chart.y_label_text().unwrap_or("").to_string(),
            &x_axis_spec,
            &y_axis_spec,
            theme,
        )
    })?;
    let projection = &layout.projection;
    let point_style = chart.point_style(theme.colors[0], 3);
    let scatter_points = x
        .iter()
        .zip(&y)
        .map(|(x_value, y_value)| {
            let (x, y) = projection.project(*x_value, *y_value);
            Point2D::new(x, y)
        })
        .collect::<Vec<_>>();
    let line_style = LineStyle::solid(theme.foreground, 1.5);
    let (fit_x0, fit_y0) = projection.project(x_min, y_fit_lo);
    let (fit_x1, fit_y1) = projection.project(x_max, y_fit_hi);
    let fit_line = [Point2D::new(fit_x0, fit_y0), Point2D::new(fit_x1, fit_y1)];
    let band_polygon = band_values.map(|values| {
        let mut upper = Vec::with_capacity(BAND_POINTS);
        let mut lower = Vec::with_capacity(BAND_POINTS);
        for (x_value, lower_value, upper_value) in values {
            let (upper_x, upper_y) = projection.project(x_value, upper_value);
            let (lower_x, lower_y) = projection.project(x_value, lower_value);
            upper.push(Point2D::new(upper_x, upper_y));
            lower.push(Point2D::new(lower_x, lower_y));
        }
        lower.reverse();
        upper.extend(lower);
        upper
    });
    let band_fill = FillStyle {
        color: theme.colors[0],
        stroke: theme.colors[0],
        stroke_width: 0.0,
        opacity: 0.25,
    };

    with_clip(backend, projection.panel_box(), |backend| {
        if let Some(polygon) = &band_polygon {
            backend.draw_polygon(polygon, &band_fill)?;
        }
        backend.draw_scatter(&scatter_points, &point_style)?;
        backend.draw_line(&fit_line, &line_style)
    })?;
    let legend_entries = if matches!(chart.legend_policy_spec(), crate::LegendPolicy::Visible(_)) {
        let fit_identity = chart.legend_identity(band.legend_label());
        let fit_entry = if band == RegressionBand::PointwiseMean95 {
            LegendEntry::line_fill(fit_identity, line_style, band_fill)
        } else {
            LegendEntry::line(fit_identity, line_style)
        };
        vec![
            LegendEntry::point(chart.legend_identity("Observations"), point_style),
            fit_entry,
        ]
    } else {
        Vec::new()
    };
    Ok(PlotRenderResult::new(layout, legend_entries))
}
