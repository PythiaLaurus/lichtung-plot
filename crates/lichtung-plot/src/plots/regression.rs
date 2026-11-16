//! Scatter with a generated simple OLS fit and optional classical confidence band.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::similar_names
)]

use crate::alpha::{
    marks::{fill_style, line_style, point_ink, push_point, ObservationAlpha},
    AlphaTarget, AlphaUnitKind, ResolvedAlpha,
};
use crate::data::PlotData;
use crate::Error;

use crate::builder::PlotDescription;
use crate::decorations::legend::source::{FillRole, LineRole, LiteralDomain, PointRole};
use crate::decorations::legend::GuideSourceKey;
use crate::layout::interior::MeasuredCoordinateMarks;
use crate::render::backend::{with_clip, Backend, FillStyle, LineStyle, Point2D, PointStyle};
use crate::spec::{PlotMeasureContext, RegressionBand};
use crate::stats::regression::OlsFit;

const BAND_POINTS: usize = 40;

#[derive(Debug)]
pub(crate) struct PreparedRegression {
    mark_ranges: [(f64, f64); 2],
    points: Vec<(f64, f64)>,
    alphas: Vec<ObservationAlpha>,
    line_alpha: Option<ResolvedAlpha>,
    fill_alpha: Option<ResolvedAlpha>,
    pub(crate) fit_line: [(f64, f64); 2],
    band_values: Option<Vec<(f64, f64, f64)>>,
}

pub(crate) fn intrinsic_body_minimum(
    chart: &PlotDescription,
    registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
    theme: &crate::text::TextRequestContext<'_, '_>,
) -> Result<(f64, f64), Error> {
    let prepared = registered
        .map(|registered| registered.family.as_ref())
        .and_then(|family| match family {
            crate::orchestrator::registration::FamilyRef::Regression(marks) => Some(marks),
            _ => None,
        })
        .ok_or_else(|| Error::Render("Regression intrinsic requires root-prepared marks".into()))?;
    if prepared.points.is_empty() {
        return Err(Error::Render(
            "Regression has no retained observations".into(),
        ));
    }
    let style = chart.point_style(
        theme,
        (0, 0, 0),
        theme.marks.get_points().get_radii().get_regression(),
    );
    let (width, height) = point_ink(
        &style,
        prepared
            .points
            .iter()
            .zip(&prepared.alphas)
            .map(|(&(x, y), &alpha)| (x, y, alpha)),
    )?
    .map_or((0.0, 0.0), crate::alpha::marks::PointInk::dimensions);
    let fit = crate::render::geometry::line_ink_width(&line_style(
        prepared.line_alpha,
        theme
            .marks
            .get_strokes()
            .get_curves()
            .get_regression_fit()
            .backend_style(),
    ))?;
    let band = if prepared.band_values.is_some() {
        crate::render::geometry::rectangular_fill_ink_outset(&fill_style(
            prepared.fill_alpha,
            theme
                .marks
                .get_fills()
                .get_bands()
                .get_regression_band()
                .backend_style((0, 0, 0)),
        ))?
    } else {
        (0.0, 0.0)
    };
    let raw_windows = registered.expect("prepared occurrence").raw_windows();
    Ok((
        if raw_windows[0].is_none() {
            width.max(fit).max(band.0)
        } else {
            0.0
        },
        if raw_windows[1].is_none() {
            height.max(fit).max(band.1)
        } else {
            0.0
        },
    ))
}

impl PreparedRegression {
    pub(crate) fn alpha_guide(
        &self,
        chart: &crate::builder::PlotDescription,
        theme: &crate::theme::ResolvedTheme,
        guide: &mut crate::decorations::legend::alpha::AlphaGuideBuilder<'_>,
    ) -> crate::Result<()> {
        use crate::decorations::legend::alpha::{AlphaGlyphRole, AlphaUnit};
        let point = chart.point_style(
            theme,
            (0, 0, 0),
            theme.marks.get_points().get_radii().get_regression(),
        );
        for &alpha in &self.alphas {
            guide.point(AlphaGlyphRole::Primary, alpha, point.clone(), false)?;
        }
        guide.line(
            AlphaGlyphRole::Line,
            AlphaUnitKind::Single,
            AlphaUnit::Single,
            self.line_alpha,
            theme
                .marks
                .get_strokes()
                .get_curves()
                .get_regression_fit()
                .backend_style(),
            false,
        )?;
        if self.band_values.is_some() {
            let fill = theme.marks.get_fills().get_bands().get_regression_band();
            guide.fill(
                AlphaGlyphRole::Fill,
                AlphaUnitKind::Single,
                AlphaUnit::Single,
                self.fill_alpha,
                fill.backend_style((0, 0, 0)),
                fill.get_outline()
                    .is_some_and(|outline| outline.mapped_width().is_some()),
            )?;
        }
        Ok(())
    }

    pub(crate) fn primary_mark_count(&self) -> u128 {
        self.points.len() as u128
    }

    /// Exact marker, fitted-line and optional band legend glyphs. The prepared
    /// fit confirms this producer's semantic input; a panel is not needed.
    pub(crate) fn guide_entries(
        chart: &crate::builder::PlotDescription,
        theme: &crate::theme::ResolvedTheme,

        color: (u8, u8, u8),
    ) -> Result<Vec<GuideSourceKey>, Error> {
        if !matches!(chart.legend_policy_spec(), crate::LegendPolicy::Visible(_)) {
            return Ok(Vec::new());
        }
        let mut entries = Vec::new();
        crate::data::source::reserve_source(&mut entries, 2)?;
        let band = chart.regression_band_spec();
        let fit_identity = crate::decorations::legend::GuideEntryIdentity::RegressionFit { band };
        let line = theme
            .marks
            .get_strokes()
            .get_curves()
            .get_regression_fit()
            .backend_style();
        let mut fit_entry =
            GuideSourceKey::literal(fit_identity, LiteralDomain::RegressionReference)
                .with_role_line(line, LineRole::RegressionFit);
        if band == RegressionBand::PointwiseMean95 {
            fit_entry = fit_entry.with_fill(
                theme.marks.get_fills().get_bands().get_regression_band(),
                color,
                FillRole::RegressionBand,
            );
        }
        entries.push(
            GuideSourceKey::literal(
                crate::decorations::legend::GuideEntryIdentity::RegressionObservation,
                LiteralDomain::RegressionReference,
            )
            .with_point(
                chart.point_style(
                    theme,
                    color,
                    theme.marks.get_points().get_radii().get_regression(),
                ),
                PointRole::Primary,
            ),
        );
        entries.push(fit_entry);
        Ok(entries)
    }

    pub(crate) fn new(
        chart: &crate::builder::PlotDescription,
        data: PlotData<'_, '_>,
    ) -> Result<(Self, [(f64, f64); 2]), Error> {
        data.validate_row_source()?;
        let alpha = chart.prepare_alpha(AlphaTarget::Point, AlphaUnitKind::Observation, &[])?;
        let line_alpha = chart
            .prepare_alpha(AlphaTarget::Line, AlphaUnitKind::Single, &[])?
            .single()?;
        let fill_alpha = chart.prepare_alpha(AlphaTarget::Fill, AlphaUnitKind::Single, &[])?;
        let x_col = chart
            .x_col()
            .ok_or_else(|| Error::Data("regression: x column not set".into()))?;
        let y_col = chart
            .y_col()
            .ok_or_else(|| Error::Data("regression: y column not set".into()))?;
        let rows = crate::data::aligned_finite_rows(data, x_col, y_col, "regression")?;
        let points = crate::data::source::collect_source(rows.iter().map(|&(_, x, y)| Ok((x, y))))?;
        let alphas = crate::data::source::collect_source(
            rows.iter()
                .map(|&(row, _, _)| ObservationAlpha::resolve(data, &alpha, row)),
        )?;
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
        y_min = y_min.min(y_fit_lo).min(y_fit_hi);
        y_max = y_max.max(y_fit_lo).max(y_fit_hi);
        if let Some(values) = &band_values {
            for (_, lower, upper) in values {
                y_min = y_min.min(*lower);
                y_max = y_max.max(*upper);
            }
        }
        let mark_ranges = [(x_min, x_max), (y_min, y_max)];

        let x_range = crate::layout::continuous_auto_range(
            (x_min, x_max),
            chart
                .x_axis_spec()
                .window
                .map(crate::layout::RawWindow::pair),
        )?;
        let y_range = crate::layout::continuous_auto_range(
            (y_min, y_max),
            chart
                .y_axis_spec()
                .window
                .map(crate::layout::RawWindow::pair),
        )?;
        Ok((
            Self {
                mark_ranges,
                points,
                alphas,
                line_alpha,
                fill_alpha: if band_values.is_some() {
                    fill_alpha.single()?
                } else {
                    None
                },
                fit_line: [(x_min, y_fit_lo), (x_max, y_fit_hi)],
                band_values,
            },
            [x_range, y_range],
        ))
    }
}

#[derive(Debug)]
pub(crate) struct RegressionPayload {
    clip: crate::layout::PixelRect,
    pub(crate) point_batches: Vec<(Vec<Point2D>, PointStyle)>,
    pub(crate) fit_line: [Point2D; 2],
    pub(crate) band: Option<Vec<Point2D>>,
    line_style: LineStyle,
    pub(crate) band_fill: FillStyle,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

impl RegressionPayload {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        with_clip(backend, self.clip, |backend| {
            if let Some(band) = &self.band {
                backend.draw_polygon(band, &self.band_fill)?;
            }
            for (points, style) in &self.point_batches {
                backend.draw_scatter(points, style)?;
            }
            backend.draw_line(&self.fit_line, &self.line_style)
        })
    }
}

pub(crate) fn measure_marks(
    chart: &PlotDescription,
    context: &PlotMeasureContext<'_, '_>,
    request: &crate::text::TextRequestContext<'_, '_>,
    layout: &crate::layout::Layout,
) -> Result<MeasuredCoordinateMarks, crate::layout::fit::PlanError> {
    let prepared = (match context.family()? {
        crate::orchestrator::registration::FamilyRef::Regression(marks) => Some(marks),
        _ => None,
    })
    .ok_or_else(|| {
        Error::Render(
            "Regression measurement requires root-prepared semantics for its component".into(),
        )
    })?;
    let theme = request;

    let projection = layout.projection.clone();
    let mut result = project(chart, prepared, layout, request)?;
    let raw_windows = context.registered.raw_windows();
    let automatic = [raw_windows[0].is_none(), raw_windows[1].is_none()];
    let mut requests = [None; 2];
    if let Some(ink) = point_ink(
        &chart.point_style(
            theme,
            (0, 0, 0),
            theme.marks.get_points().get_radii().get_regression(),
        ),
        prepared
            .points
            .iter()
            .zip(&prepared.alphas)
            .map(|(&(x, y), &alpha)| (x, y, alpha)),
    )? {
        requests = crate::layout::interior::automatic_mark_ink_expansions(
            ink.ranges,
            &projection,
            ink.bounds,
            automatic,
            context.fit_scope(),
        )?;
    }
    let crate::layout::interior::MarkPayload::Regression(payload) = &result.marks.payload else {
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
    let marks = std::sync::Arc::get_mut(&mut result.marks)
        .expect("newly selected Regression marks have one owner");
    let crate::layout::interior::MarkPayload::Regression(payload) = &mut marks.payload else {
        unreachable!("Regression selected its own mark payload")
    };
    payload.domain_expansions = requests;
    Ok(result)
}

pub(crate) fn project(
    chart: &PlotDescription,
    prepared: &PreparedRegression,
    layout: &crate::layout::Layout,
    request: &crate::text::TextRequestContext<'_, '_>,
) -> Result<MeasuredCoordinateMarks, Error> {
    let theme = request;
    let projection = &layout.projection;
    let marks = theme.marks();
    let color = crate::theme::AccentColors::resolve(marks.get_accents(), 1)?.color(0)?;
    let point_radius = marks.get_points().get_radii().get_regression();
    let point_style = chart.point_style(theme, color, point_radius);
    crate::alpha::marks::require_aligned_points(prepared.points.len(), prepared.alphas.len())?;
    let mut point_batches = Vec::new();
    for ((x_value, y_value), alpha) in prepared.points.iter().zip(&prepared.alphas) {
        let (x, y) = projection.project(*x_value, *y_value);
        push_point(
            &mut point_batches,
            Point2D::new(x, y),
            alpha.point(point_style.clone()),
        )?;
    }
    let line_style = line_style(
        prepared.line_alpha,
        marks
            .get_strokes()
            .get_curves()
            .get_regression_fit()
            .backend_style(),
    );

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
    let band_fill = fill_style(
        prepared.fill_alpha,
        marks
            .get_fills()
            .get_bands()
            .get_regression_band()
            .backend_style(color),
    );

    let payload = RegressionPayload {
        clip: projection.panel_box(),
        point_batches,
        fit_line,
        band: band_polygon,
        line_style,
        band_fill,
        domain_expansions: [None; 2],
    };
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    if let Some(band) = &payload.band {
        geometry.polygon(band, &payload.band_fill, Some(clip))?;
    }
    for (points, style) in &payload.point_batches {
        geometry.scatter(points, style, clip)?;
    }
    geometry.line(&payload.fit_line, &payload.line_style, Some(clip))?;
    Ok(MeasuredCoordinateMarks::new(
        crate::data::source::source_arc(crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Regression(payload),
            geometry,
        })?,
    ))
}
