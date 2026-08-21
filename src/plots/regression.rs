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
use crate::layout::Layout;
use crate::render::backend::{with_clip, Backend, FillStyle, LineStyle, Point2D};
use crate::spec::{PlotRenderContext, PlotRenderResult, RegressionBand};
use crate::stats::regression::OlsFit;
use crate::Chart;

const BAND_POINTS: usize = 40;

#[expect(
    clippy::too_many_lines,
    reason = "the renderer keeps fit, band, projection, marks, and provenance in one ordered lifecycle"
)]
pub(crate) fn render(
    chart: &Chart<'_>,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    let palette = chart.palette();
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
        )));
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
    let layout = Layout::new_with_axis_specs_in_panel_box(
        x_range,
        y_range,
        panel,
        chart.x_label_text().unwrap_or("").to_string(),
        chart.y_label_text().unwrap_or("").to_string(),
        &x_axis_spec,
        &y_axis_spec,
    );
    let projection = &layout.projection;
    let point_style = chart.point_style(palette.colors[0], 3);
    let scatter_points = x
        .iter()
        .zip(&y)
        .map(|(x_value, y_value)| {
            let (x, y) = projection.project(*x_value, *y_value);
            Point2D::new(x, y)
        })
        .collect::<Vec<_>>();
    let line_style = LineStyle::solid(palette.foreground, 1.5);
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
        color: palette.colors[0],
        stroke: palette.colors[0],
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
    let legend_entries = if chart.legend_visibility() == Some(true) {
        vec![
            LegendEntry::point("Observations".to_string(), point_style),
            LegendEntry::point(
                band.legend_label().to_string(),
                chart.point_style(line_style.color, 3),
            ),
        ]
    } else {
        Vec::new()
    };
    Ok(PlotRenderResult::new(layout, legend_entries))
}
