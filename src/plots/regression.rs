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

#[derive(Debug)]
pub(crate) struct PreparedRegressionInputs {
    pub(crate) panels: Vec<(Option<String>, PreparedRegression)>,
}

#[derive(Debug)]
pub(crate) struct PreparedRegression {
    pub(crate) ranges: [(f64, f64); 2],
    observation_ranges: [(f64, f64); 2],
    mark_ranges: [(f64, f64); 2],
    points: Vec<(f64, f64)>,
    pub(crate) fit_line: [(f64, f64); 2],
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
    let style = chart.point_style(theme, (0, 0, 0), theme.marks.points().radii().regression());
    let (width, height) = crate::render::geometry::point_ink_dimensions(&style)?;
    let fit = crate::render::geometry::line_ink_width(
        &theme
            .marks
            .strokes()
            .curves()
            .regression_fit()
            .backend_style(),
    )?;
    let band = if prepared.band_values.is_some() {
        crate::render::geometry::rectangular_fill_ink_outset(
            &theme
                .marks
                .fills()
                .bands()
                .regression_band()
                .backend_style((0, 0, 0)),
        )?
    } else {
        (0.0, 0.0)
    };
    Ok((
        if chart.user_x_limits().is_none() {
            width.max(fit).max(band.0)
        } else {
            0.0
        },
        if chart.user_y_limits().is_none() {
            height.max(fit).max(band.1)
        } else {
            0.0
        },
    ))
}

impl PreparedRegression {
    pub(crate) fn primary_mark_count(&self) -> u128 {
        self.points.len() as u128
    }

    /// Exact marker, fitted-line and optional band legend glyphs. The prepared
    /// fit confirms this producer's semantic input; a panel is not needed.
    pub(crate) fn guide_entries(
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,

        color: (u8, u8, u8),
    ) -> Result<Vec<LegendEntry>, Error> {
        if !matches!(chart.legend_policy_spec(), crate::LegendPolicy::Visible(_)) {
            return Ok(Vec::new());
        }
        let mut entries = Vec::new();
        crate::data::source::reserve_source(&mut entries, 2)?;
        let band = chart.regression_band_spec();
        let fit_identity = chart.point_legend_identity(band.legend_label())?;
        let line = theme
            .marks
            .strokes()
            .curves()
            .regression_fit()
            .backend_style();
        let fit_entry = if band == RegressionBand::PointwiseMean95 {
            LegendEntry::line_fill(
                fit_identity,
                line,
                theme
                    .marks
                    .fills()
                    .bands()
                    .regression_band()
                    .backend_style(color),
            )
        } else {
            LegendEntry::line(fit_identity, line)
        };
        entries.push(LegendEntry::point(
            chart.point_legend_identity("Observations")?,
            chart.point_style(theme, color, theme.marks.points().radii().regression()),
        ));
        entries.push(fit_entry);
        Ok(entries)
    }

    pub(crate) fn new(chart: &Plot, data: PlotData<'_>) -> Result<Self, Error> {
        data.validate_row_source()?;
        #[cfg(test)]
        crate::layout::interior::distributions::count(3, 0);
        let x_col = chart
            .x_col()
            .ok_or_else(|| Error::Data("regression: x column not set".into()))?;
        let y_col = chart
            .y_col()
            .ok_or_else(|| Error::Data("regression: y column not set".into()))?;
        let points = crate::data::aligned_finite_f64(data, x_col, y_col, "regression")?;
        let x = crate::data::source::collect_source(points.iter().map(|p| Ok(p.0)))?;
        let y = crate::data::source::collect_source(points.iter().map(|p| Ok(p.1)))?;
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

        let band_values = if band == RegressionBand::PointwiseMean95 {
            let mut values = Vec::new();
            crate::data::source::reserve_source(&mut values, BAND_POINTS)?;

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
        let observation_ranges = [(x_min, x_max), (y_min, y_max)];
        y_min = y_min.min(y_fit_lo).min(y_fit_hi);
        y_max = y_max.max(y_fit_lo).max(y_fit_hi);
        if let Some(values) = &band_values {
            for (_, lower, upper) in values {
                y_min = y_min.min(*lower);
                y_max = y_max.max(*upper);
            }
        }
        let mark_ranges = [(x_min, x_max), (y_min, y_max)];

        let x_range = crate::layout::continuous_auto_range((x_min, x_max), chart.x_axis_spec())?;
        let y_range = crate::layout::continuous_auto_range((y_min, y_max), chart.y_axis_spec())?;
        Ok(Self {
            ranges: [x_range, y_range],
            observation_ranges,
            mark_ranges,
            points,
            fit_line: [(x_min, y_fit_lo), (x_max, y_fit_hi)],
            band_values,
        })
    }
}

#[derive(Debug)]
pub(crate) struct RegressionPayload {
    clip: crate::layout::PanelBox,
    pub(crate) points: Vec<Point2D>,
    pub(crate) fit_line: [Point2D; 2],
    pub(crate) band: Option<Vec<Point2D>>,
    point_style: PointStyle,
    line_style: LineStyle,
    pub(crate) band_fill: FillStyle,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
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

    let projection = layout.projection.clone();
    let mut result = project(chart, prepared, layout, theme)?;
    let automatic = [
        chart.user_x_limits().is_none(),
        chart.user_y_limits().is_none(),
    ];
    let mut requests = [None; 2];
    if let Some(ink) = crate::render::geometry::point_ink_bounds(&chart.point_style(
        theme,
        (0, 0, 0),
        theme.marks.points().radii().regression(),
    ))? {
        requests = crate::layout::interior::automatic_mark_ink_expansions(
            prepared.observation_ranges,
            &projection,
            ink,
            automatic,
            context.fit_scope(),
        )?;
    }
    let crate::layout::interior::MarkPayload::Regression(payload) = &result
        .marks
        .as_ref()
        .expect("selected Regression marks")
        .payload
    else {
        unreachable!("Regression selected its own mark payload")
    };
    let mut scientific_ink = crate::render::geometry::GeometrySnapshot::new();
    if let Some(band) = &payload.band {
        scientific_ink.polygon(band, &payload.band_fill, None)?;
    }
    scientific_ink.line(&payload.fit_line, &payload.line_style, None)?;
    if let Some(bounds) = scientific_ink
        .bounds()
        .iter()
        .copied()
        .reduce(crate::render::geometry::GeometryBox::union)
    {
        let additional = crate::layout::interior::automatic_absolute_ink_expansions(
            prepared.mark_ranges,
            &projection,
            bounds,
            automatic,
            context.fit_scope(),
        )?;
        requests = crate::layout::interior::merge_mark_domain_expansions(requests, additional);
    }
    let marks = std::sync::Arc::get_mut(result.marks.as_mut().expect("selected Regression marks"))
        .expect("newly selected Regression marks have one owner");
    let crate::layout::interior::MarkPayload::Regression(payload) = &mut marks.payload else {
        unreachable!("Regression selected its own mark payload")
    };
    payload.domain_expansions = requests;
    Ok(result)
}

pub(crate) fn project(
    chart: &Plot,
    prepared: &PreparedRegression,
    layout: crate::layout::Layout,
    theme: &crate::theme::ResolvedTheme,
) -> Result<PlotRenderResult, Error> {
    let projection = &layout.projection;
    let color = crate::theme::AccentColors::resolve(theme.marks.accents(), 1)?.color(0)?;
    let point_style = chart.point_style(theme, color, theme.marks.points().radii().regression());
    let scatter_points =
        crate::data::source::collect_source(prepared.points.iter().map(|(x_value, y_value)| {
            let (x, y) = projection.project(*x_value, *y_value);
            Ok(Point2D::new(x, y))
        }))?;
    let line_style = theme
        .marks
        .strokes()
        .curves()
        .regression_fit()
        .backend_style();

    let (fit_x0, fit_y0) = projection.project(prepared.fit_line[0].0, prepared.fit_line[0].1);
    let (fit_x1, fit_y1) = projection.project(prepared.fit_line[1].0, prepared.fit_line[1].1);
    let fit_line = [Point2D::new(fit_x0, fit_y0), Point2D::new(fit_x1, fit_y1)];
    let band_polygon = prepared
        .band_values
        .as_ref()
        .map(|values| -> Result<_, Error> {
            let mut upper = Vec::new();
            let mut lower = Vec::new();
            crate::data::source::reserve_source(&mut upper, BAND_POINTS * 2)?;
            crate::data::source::reserve_source(&mut lower, BAND_POINTS)?;

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

            lower.reverse();

            upper.extend(lower);
            Ok(upper)
        })
        .transpose()?;
    let band_fill = theme
        .marks
        .fills()
        .bands()
        .regression_band()
        .backend_style(color);

    let legend_entries = PreparedRegression::guide_entries(chart, theme, color)?;
    let payload = RegressionPayload {
        clip: projection.panel_box(),
        points: scatter_points,
        fit_line,
        band: band_polygon,
        point_style,
        line_style,
        band_fill,
        domain_expansions: [None; 2],
    };
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    #[cfg(test)]
    crate::layout::interior::distributions::count(3, 2);
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    if let Some(band) = &payload.band {
        geometry.polygon(band, &payload.band_fill, Some(clip))?;
    }
    geometry.scatter(&payload.points, &payload.point_style, clip)?;
    geometry.line(&payload.fit_line, &payload.line_style, Some(clip))?;
    let mut result = PlotRenderResult::new(layout, legend_entries);
    result.marks = Some(crate::data::source::source_arc(
        crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Regression(payload),
            geometry,
        },
    )?);
    Ok(result)
}

#[cfg(test)]
mod style_tests {
    use super::*;
    use crate::font::canonical::OperationContext;

    #[test]
    fn regression_observation_and_fit_ink_remeasure_with_custom_styles() {
        let source = Plot::regression(
            crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            crate::NumericValues::new("y", [1.0, 2.5, 2.0, 4.0]),
        )
        .erase();
        let resources = OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
        let chart = inputs.attach(source).unwrap();
        let mut theme = crate::Theme::Publication.resolve().unwrap();
        let before = chart.intrinsic_body_minimum(&theme).unwrap();
        let radii = theme
            .marks
            .points()
            .radii()
            .clone()
            .with_regression(54)
            .unwrap();
        let points = theme.marks.points().clone().with_radii(radii).unwrap();
        theme.marks = theme.marks.clone().with_points(points).unwrap();
        let sized = chart.intrinsic_body_minimum(&theme).unwrap();
        assert_eq!(sized.0.to_bits(), (108.0_f64 + 5530.0 / 1024.0).to_bits());
        assert_eq!(sized.1.to_bits(), (108.0_f64 + 5530.0 / 1024.0).to_bits());
        let mut primary = theme.marks.points().primary().clone();
        primary.fill_alpha = 0;
        primary.outline.as_mut().unwrap().color = crate::theme::Color::rgba(255, 255, 255, 0);
        let points = theme.marks.points().clone().with_primary(primary).unwrap();
        theme.marks = theme.marks.clone().with_points(points).unwrap();
        assert_eq!(chart.intrinsic_body_minimum(&theme).unwrap(), before);
        let fit = theme
            .marks
            .strokes()
            .curves()
            .regression_fit()
            .clone()
            .with_width(140.0)
            .unwrap();
        let curves = theme
            .marks
            .strokes()
            .curves()
            .clone()
            .with_regression_fit(fit)
            .unwrap();
        let strokes = theme.marks.strokes().clone().with_curves(curves).unwrap();
        theme.marks = theme.marks.clone().with_strokes(strokes).unwrap();
        assert_eq!(
            chart.intrinsic_body_minimum(&theme).unwrap(),
            (140.0, 140.0)
        );
        let fit = theme
            .marks
            .strokes()
            .curves()
            .regression_fit()
            .clone()
            .with_color(crate::theme::Color::rgba(1, 2, 3, 0))
            .unwrap();
        let curves = theme
            .marks
            .strokes()
            .curves()
            .clone()
            .with_regression_fit(fit)
            .unwrap();
        let strokes = theme.marks.strokes().clone().with_curves(curves).unwrap();
        theme.marks = theme.marks.clone().with_strokes(strokes).unwrap();
        assert_eq!(chart.intrinsic_body_minimum(&theme).unwrap(), before);
        let band = theme
            .marks
            .fills()
            .bands()
            .regression_band()
            .clone()
            .with_outline(Some(
                crate::theme::DataOutlineTheme::mapped(140.0, 255).unwrap(),
            ))
            .unwrap();
        let bands = theme
            .marks
            .fills()
            .bands()
            .clone()
            .with_regression_band(band)
            .unwrap();
        let fills = theme.marks.fills().clone().with_bands(bands).unwrap();
        theme.marks = theme.marks.clone().with_fills(fills).unwrap();
        let band = chart.intrinsic_body_minimum(&theme).unwrap();
        assert!(band.0 >= 140.0 && band.1 >= 140.0);
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
        let resources = OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
        let chart = inputs.attach(source).unwrap();
        let mut theme = crate::Theme::Publication.resolve().unwrap();
        let panel = crate::layout::PanelBox::new(10.0, 620.0, 20.0, 420.0);
        let old = chart
            .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
            .unwrap();
        let band = theme
            .marks
            .fills()
            .bands()
            .regression_band()
            .clone()
            .with_fill_alpha(128)
            .unwrap()
            .with_outline(Some(
                crate::theme::DataOutlineTheme::mapped(2.0, 255).unwrap(),
            ))
            .unwrap();
        let bands = theme
            .marks
            .fills()
            .bands()
            .clone()
            .with_regression_band(band)
            .unwrap();
        let fills = theme.marks.fills().clone().with_bands(bands).unwrap();
        theme.marks = theme.marks.clone().with_fills(fills).unwrap();
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
        let resources = OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
        let chart = inputs.attach(source).unwrap();
        let mut theme = crate::Theme::Publication.resolve().unwrap();
        let panel = crate::layout::PanelBox::new(10.0, 620.0, 20.0, 420.0);
        let old = chart
            .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
            .unwrap();
        let fit = theme
            .marks
            .strokes()
            .curves()
            .regression_fit()
            .clone()
            .with_color(crate::theme::Color::rgb(17, 41, 67))
            .unwrap()
            .with_width(2.5)
            .unwrap();
        let curves = theme
            .marks
            .strokes()
            .curves()
            .clone()
            .with_regression_fit(fit)
            .unwrap();
        let strokes = theme.marks.strokes().clone().with_curves(curves).unwrap();
        theme.marks = theme.marks.clone().with_strokes(strokes).unwrap();
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
        assert_eq!(new_style.color, crate::theme::Color::rgb(17, 41, 67));
        assert_eq!(new_style.width.to_bits(), 2.5_f32.to_bits());
        assert!(crate::decorations::legend::merge_exact_entries(
            [
                old.legend_entries[1].clone(),
                changed.legend_entries[1].clone()
            ],
            "custom regression fit",
        )
        .unwrap_err()
        .to_string()
        .contains("glyphs must match exactly"));
    }
}
