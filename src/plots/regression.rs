//! Scatter with a generated simple OLS fit and optional classical confidence band.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::similar_names
)]

use crate::data::PlotData;
use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::render::backend::{with_clip, Backend, FillStyle, LineStyle, Point2D, PointStyle};
use crate::spec::{PlotRenderContext, PlotRenderResult, RegressionBand};
use crate::stats::regression::OlsFit;
use crate::Plot;

const BAND_POINTS: usize = 40;

#[cfg(test)]
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
        crate::layout::continuous_auto_range(xr, chart.x_axis_spec(), data.source_resources()?)?,
        crate::layout::continuous_auto_range(yr, chart.y_axis_spec(), data.source_resources()?)?,
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
    let baseline_resources = OperationGenerationResources::new();
    let expected = axis_ranges(
        &chart,
        chart.data().with_axis_resources(&baseline_resources),
    )
    .unwrap();
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

#[derive(Debug)]
pub(crate) struct PreparedRegressionInputs {
    pub(crate) panels: Vec<(Option<String>, PreparedRegression)>,
}

#[derive(Debug)]
pub(crate) struct PreparedRegression {
    pub(crate) ranges: [(f64, f64); 2],
    points: Vec<(f64, f64)>,
    fit_line: [(f64, f64); 2],
    band_values: Option<Vec<(f64, f64, f64)>>,
}

pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let prepared = chart
        .prepared_regression
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, prepared)| prepared)
        .ok_or_else(|| Error::Render("Regression intrinsic requires root-prepared marks".into()))?;
    if prepared.points.is_empty() {
        return Err(Error::Render(
            "Regression has no retained observations".into(),
        ));
    }
    let style = chart.point_style(theme, (0, 0, 0), theme.marks.point_sizes.regression);
    let (width, height) = crate::render::geometry::point_ink_dimensions(theme, &style)?;
    let fit = crate::render::geometry::line_ink_width(
        theme,
        &theme.marks.regression_fit.backend_style(),
    )?;
    Ok((
        if chart.user_x_limits().is_none() {
            width.max(fit)
        } else {
            0.0
        },
        if chart.user_y_limits().is_none() {
            height.max(fit)
        } else {
            0.0
        },
    ))
}

impl PreparedRegression {
    /// Exact marker, fitted-line and optional band legend glyphs. The prepared
    /// fit confirms this producer's semantic input; a panel is not needed.
    pub(crate) fn guide_entries(
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        data: PlotData<'_>,
    ) -> Result<Vec<LegendEntry>, Error> {
        if !matches!(chart.legend_policy_spec(), crate::LegendPolicy::Visible(_)) {
            return Ok(Vec::new());
        }
        let mut entries = Vec::new();
        data.reserve_source(&mut entries, 2)?;
        let color = theme.colors[0];
        let band = chart.regression_band_spec();
        let fit_identity = chart.point_legend_identity(band.legend_label(), data)?;
        let line = theme.marks.regression_fit.backend_style();
        let fit_entry = if band == RegressionBand::PointwiseMean95 {
            LegendEntry::line_fill(
                fit_identity,
                line,
                theme.marks.regression_band.backend_style(color),
            )
        } else {
            LegendEntry::line(fit_identity, line)
        };
        entries.push(LegendEntry::point(
            chart.point_legend_identity("Observations", data)?,
            chart.point_style(theme, color, theme.marks.point_sizes.regression),
        ));
        entries.push(fit_entry);
        Ok(entries)
    }

    pub(crate) fn new(chart: &Plot, data: PlotData<'_>) -> Result<Self, Error> {
        data.claim_mark_rows()?;
        #[cfg(test)]
        crate::layout::interior::distributions::count(3, 0);
        let x_col = chart
            .x_col()
            .ok_or_else(|| Error::Data("regression: x column not set".into()))?;
        let y_col = chart
            .y_col()
            .ok_or_else(|| Error::Data("regression: y column not set".into()))?;
        let points = crate::data::aligned_finite_f64(data, x_col, y_col, "regression")?;
        let x = data.collect_source(points.iter().map(|p| Ok(p.0)))?;
        let y = data.collect_source(points.iter().map(|p| Ok(p.1)))?;
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

        for _ in 0..10 {
            data.claim_source_visits(x.len())?;
        }
        let fit = OlsFit::new(&x, &y)?;
        data.claim_source_visits(x.len())?;
        let (x_min, x_max) = crate::data::min_max(x.iter().copied(), "regression")?;
        data.claim_source_visits(2)?;
        let y_fit_lo = fit.predict(x_min)?;
        let y_fit_hi = fit.predict(x_max)?;

        let band_values = if band == RegressionBand::PointwiseMean95 {
            let mut values = Vec::new();
            data.reserve_source(&mut values, BAND_POINTS)?;
            data.claim_source_visits(BAND_POINTS)?;
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

        data.claim_source_visits(y.len())?;
        let (mut y_min, mut y_max) = crate::data::min_max(y.iter().copied(), "regression")?;
        y_min = y_min.min(y_fit_lo).min(y_fit_hi);
        y_max = y_max.max(y_fit_lo).max(y_fit_hi);
        if let Some(values) = &band_values {
            data.claim_source_visits(values.len())?;
            for (_, lower, upper) in values {
                y_min = y_min.min(*lower);
                y_max = y_max.max(*upper);
            }
        }

        let x_range = crate::layout::continuous_auto_range(
            (x_min, x_max),
            chart.x_axis_spec(),
            data.source_resources()?,
        )?;
        let y_range = crate::layout::continuous_auto_range(
            (y_min, y_max),
            chart.y_axis_spec(),
            data.source_resources()?,
        )?;
        Ok(Self {
            ranges: [x_range, y_range],
            points,
            fit_line: [(x_min, y_fit_lo), (x_max, y_fit_hi)],
            band_values,
        })
    }
}

#[derive(Debug)]
pub(crate) struct RegressionPayload {
    clip: crate::layout::PanelBox,
    points: Vec<Point2D>,
    fit_line: [Point2D; 2],
    band: Option<Vec<Point2D>>,
    point_style: PointStyle,
    line_style: LineStyle,
    band_fill: FillStyle,
}

impl RegressionPayload {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        #[cfg(test)]
        crate::layout::interior::distributions::count(3, 3);
        with_clip(backend, self.clip, |backend| {
            if let Some(band) = &self.band {
                backend.draw_polygon(band, &self.band_fill)?;
            }
            backend.draw_scatter(&self.points, &self.point_style)?;
            backend.draw_line(&self.fit_line, &self.line_style)
        })
    }
}

pub(crate) fn measure_marks(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    layout: crate::layout::Layout,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    crate::layout::interior::distributions::count(3, 1);
    let prepared = chart
        .prepared_regression
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, prepared)| prepared)
        .ok_or_else(|| {
            Error::Render(
                "Regression measurement requires root-prepared semantics for its component".into(),
            )
        })?;
    let theme = context.theme;
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = context.data.with_axis_resources(&resources);
    project(chart, prepared, layout, theme, data).map_err(Into::into)
}

pub(crate) fn project(
    chart: &Plot,
    prepared: &PreparedRegression,
    layout: crate::layout::Layout,
    theme: &crate::theme::ResolvedTheme,
    data: PlotData<'_>,
) -> Result<PlotRenderResult, Error> {
    let projection = &layout.projection;
    let point_style = chart.point_style(theme, theme.colors[0], theme.marks.point_sizes.regression);
    let scatter_points =
        data.collect_source(prepared.points.iter().map(|(x_value, y_value)| {
            data.claim_source_visits(1)?;
            let (x, y) = projection.project(*x_value, *y_value);
            Ok(Point2D::new(x, y))
        }))?;
    let line_style = theme.marks.regression_fit.backend_style();
    data.claim_source_visits(2)?;
    let (fit_x0, fit_y0) = projection.project(prepared.fit_line[0].0, prepared.fit_line[0].1);
    let (fit_x1, fit_y1) = projection.project(prepared.fit_line[1].0, prepared.fit_line[1].1);
    let fit_line = [Point2D::new(fit_x0, fit_y0), Point2D::new(fit_x1, fit_y1)];
    let band_polygon = prepared
        .band_values
        .as_ref()
        .map(|values| -> Result<_, Error> {
            let mut upper = Vec::new();
            let mut lower = Vec::new();
            data.reserve_source(&mut upper, BAND_POINTS * 2)?;
            data.reserve_source(&mut lower, BAND_POINTS)?;
            data.claim_source_visits(values.len())?;
            let mut has_area = false;
            for (x_value, lower_value, upper_value) in values {
                has_area |= lower_value < upper_value;
                let (upper_x, upper_y) = projection.project(*x_value, *upper_value);
                let (lower_x, lower_y) = projection.project(*x_value, *lower_value);
                projection.validate_span(
                    crate::layout::AxisSide::Left,
                    (*lower_value, *upper_value),
                    (lower_y, upper_y),
                    lower_value < upper_value,
                    "regression band polygon",
                )?;
                upper.push(Point2D::new(upper_x, upper_y));
                lower.push(Point2D::new(lower_x, lower_y));
            }
            if let Some((first, last)) = values.first().zip(values.last()) {
                projection.validate_span(
                    crate::layout::AxisSide::Bottom,
                    (first.0, last.0),
                    (upper[0].x, upper[upper.len() - 1].x),
                    has_area && first.0 < last.0,
                    "regression band polygon width",
                )?;
            }
            data.claim_source_visits(lower.len())?;
            lower.reverse();
            data.claim_source_visits(lower.len())?;
            upper.extend(lower);
            Ok(upper)
        })
        .transpose()?;
    let band_fill = theme.marks.regression_band.backend_style(theme.colors[0]);

    let legend_entries = PreparedRegression::guide_entries(chart, theme, data)?;
    let payload = RegressionPayload {
        clip: projection.panel_box(),
        points: scatter_points,
        fit_line,
        band: band_polygon,
        point_style,
        line_style,
        band_fill,
    };
    let mut geometry = crate::render::geometry::GeometrySnapshot::new(theme)?;
    #[cfg(test)]
    crate::layout::interior::distributions::count(3, 2);
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    if let Some(band) = &payload.band {
        geometry.polygon(band, &payload.band_fill, Some(clip))?;
    }
    geometry.scatter(&payload.points, &payload.point_style, clip)?;
    geometry.line(&payload.fit_line, &payload.line_style, Some(clip))?;
    let mut result = PlotRenderResult::new(layout, legend_entries);
    result.marks = Some(data.source_arc(crate::layout::interior::DeferredMarkPlan {
        payload: crate::layout::interior::MarkPayload::Regression(payload),
        geometry,
    })?);
    Ok(result)
}

#[cfg(test)]
mod style_tests {
    use super::*;
    use crate::font::canonical::OperationGenerationResources;

    #[test]
    fn regression_observation_and_fit_ink_remeasure_with_custom_styles() {
        let source = Plot::regression(
            crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            crate::NumericValues::new("y", [1.0, 2.5, 2.0, 4.0]),
        )
        .erase();
        let resources = OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        let mut theme = crate::Theme::Publication.resolve();
        let before = chart.intrinsic_body_minimum(&theme).unwrap();
        theme.marks.point_sizes.regression = 54;
        let sized = chart.intrinsic_body_minimum(&theme).unwrap();
        assert_eq!(sized.0.to_bits(), (108.0_f64 + 5530.0 / 1024.0).to_bits());
        assert_eq!(sized.1.to_bits(), (108.0_f64 + 5530.0 / 1024.0).to_bits());
        theme.marks.primary_point.fill_alpha = 0;
        theme.marks.primary_point.outline.color =
            crate::render::backend::Rgba::new(255, 255, 255, 0);
        assert_eq!(chart.intrinsic_body_minimum(&theme).unwrap(), before);
        theme.marks.regression_fit.width = 140.0;
        assert_eq!(
            chart.intrinsic_body_minimum(&theme).unwrap(),
            (140.0, 140.0)
        );
        theme.marks.regression_fit.paint = crate::render::backend::Rgba::new(1, 2, 3, 0);
        assert_eq!(chart.intrinsic_body_minimum(&theme).unwrap(), before);
    }

    #[test]
    fn custom_regression_band_preserves_scientific_polygon_and_changes_guide_glyph() {
        let source = Plot::regression(
            crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            crate::NumericValues::new("y", [1.0, 2.5, 2.0, 4.0]),
        )
        .legend(crate::LegendPolicy::Visible(
            crate::LegendPosition::OutsideRight,
        ))
        .erase();
        let resources = OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        let mut theme = crate::Theme::Publication.resolve();
        let panel = crate::layout::PanelBox::new(10.0, 620.0, 20.0, 420.0);
        let old = chart
            .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
            .unwrap();
        theme.marks.regression_band.fill_alpha = 128;
        theme.marks.regression_band.outline_width = 2.0;
        let changed = chart
            .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
            .unwrap();
        let payload = |result: &PlotRenderResult| {
            let crate::layout::interior::MarkPayload::Regression(payload) =
                &result.marks.as_ref().unwrap().payload
            else {
                panic!("Regression mark payload")
            };
            (payload.band.clone(), payload.band_fill.clone())
        };
        let (old_polygon, old_fill) = payload(&old);
        let (new_polygon, new_fill) = payload(&changed);
        assert_eq!(old_polygon, new_polygon);
        assert_eq!(old_fill.color.alpha(), 64);
        assert_eq!(new_fill.color.alpha(), 128);
        assert_eq!(new_fill.stroke_width.to_bits(), 2.0_f32.to_bits());
        assert!(crate::decorations::legend::merge_exact_entries(
            [
                old.legend_entries[1].clone(),
                changed.legend_entries[1].clone()
            ],
            "custom regression band",
            &OperationGenerationResources::new(),
        )
        .unwrap_err()
        .to_string()
        .contains("glyphs must match exactly"));
    }

    #[test]
    fn custom_fit_stroke_preserves_scientific_line_and_changes_exact_guide_glyph() {
        let source = Plot::regression(
            crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            crate::NumericValues::new("y", [1.0, 2.5, 2.0, 4.0]),
        )
        .legend(crate::LegendPolicy::Visible(
            crate::LegendPosition::OutsideRight,
        ))
        .erase();
        let resources = OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        let mut theme = crate::Theme::Publication.resolve();
        let panel = crate::layout::PanelBox::new(10.0, 620.0, 20.0, 420.0);
        let old = chart
            .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
            .unwrap();
        theme.marks.regression_fit.paint = crate::render::backend::Rgba::rgb(17, 41, 67);
        theme.marks.regression_fit.width = 2.5;
        let changed = chart
            .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
            .unwrap();
        let payload = |result: &PlotRenderResult| {
            let crate::layout::interior::MarkPayload::Regression(payload) =
                &result.marks.as_ref().unwrap().payload
            else {
                panic!("Regression mark payload")
            };
            (payload.fit_line, payload.line_style.clone())
        };
        let (old_line, old_style) = payload(&old);
        let (new_line, new_style) = payload(&changed);
        assert_eq!(old_line, new_line);
        assert_ne!(old_style, new_style);
        assert_eq!(
            new_style.color,
            crate::render::backend::Rgba::rgb(17, 41, 67)
        );
        assert_eq!(new_style.width.to_bits(), 2.5_f32.to_bits());
        assert!(crate::decorations::legend::merge_exact_entries(
            [
                old.legend_entries[1].clone(),
                changed.legend_entries[1].clone()
            ],
            "custom regression fit",
            &OperationGenerationResources::new(),
        )
        .unwrap_err()
        .to_string()
        .contains("glyphs must match exactly"));
    }
}
