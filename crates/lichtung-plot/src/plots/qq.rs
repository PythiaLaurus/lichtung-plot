//! Quantile-quantile plots against the standard normal distribution.

#![allow(clippy::cast_precision_loss)]

use crate::alpha::{
    marks::{line_style, point_ink, push_point, ObservationAlpha},
    AlphaTarget, AlphaUnitKind, ResolvedAlpha,
};
use crate::data::PlotData;
use crate::Error;

use crate::builder::PlotDescription;
use crate::layout::interior::MeasuredCoordinateMarks;
use crate::render::backend::{with_clip, Backend, LineStyle, Point2D, PointStyle};
use crate::spec::{PlotMeasureContext, QqLine};
use crate::stats::distribution::{sorted_quantile, standard_normal_quantile};
#[cfg(test)]
use crate::Plot;

#[derive(Debug, Clone, Copy, PartialEq)]
struct LineGeometry {
    slope: f64,
    intercept: f64,
}

#[derive(Debug)]
pub(crate) struct PreparedQq {
    theoretical: Vec<f64>,
    observed: Vec<f64>,
    alphas: Vec<ObservationAlpha>,
    line: Option<LineGeometry>,
    line_alpha: Option<ResolvedAlpha>,
}

pub(crate) fn intrinsic_body_minimum(
    chart: &PlotDescription,
    registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
    theme: &crate::text::TextRequestContext<'_, '_>,
) -> Result<(f64, f64), Error> {
    let prepared = registered
        .map(|registered| registered.family.as_ref())
        .and_then(|family| match family {
            crate::orchestrator::registration::FamilyRef::Qq(marks) => Some(marks),
            _ => None,
        })
        .ok_or_else(|| Error::Render("Qq intrinsic requires root-prepared marks".into()))?;
    if prepared.theoretical.len() != prepared.observed.len() || prepared.theoretical.is_empty() {
        return Err(Error::Render(
            "Qq primary quantile pairs are incomplete".into(),
        ));
    }
    let style = chart.point_style(
        theme,
        (0, 0, 0),
        theme.marks.get_points().get_radii().get_qq(),
    );
    let (width, height) = point_ink(
        &style,
        prepared
            .theoretical
            .iter()
            .zip(&prepared.observed)
            .zip(&prepared.alphas)
            .map(|((&x, &y), &alpha)| (x, y, alpha)),
    )?
    .map_or((0.0, 0.0), crate::alpha::marks::PointInk::dimensions);
    let reference = if prepared.line.is_some() {
        crate::render::geometry::line_ink_width(&line_style(
            prepared.line_alpha,
            theme
                .marks
                .get_strokes()
                .get_references()
                .get_qq_reference()
                .backend_style(),
        ))?
    } else {
        0.0
    };
    let raw_windows = registered.expect("prepared occurrence").raw_windows();
    Ok((
        if raw_windows[0].is_none() {
            width.max(reference)
        } else {
            0.0
        },
        if raw_windows[1].is_none() {
            height.max(reference)
        } else {
            0.0
        },
    ))
}

impl PreparedQq {
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
            theme.marks.get_points().get_radii().get_qq(),
        );
        for &alpha in &self.alphas {
            guide.point(AlphaGlyphRole::Primary, alpha, point.clone(), false)?;
        }
        if self.line.is_some() {
            guide.line(
                AlphaGlyphRole::Line,
                AlphaUnitKind::Single,
                AlphaUnit::Single,
                self.line_alpha,
                theme
                    .marks
                    .get_strokes()
                    .get_references()
                    .get_qq_reference()
                    .backend_style(),
                false,
            )?;
        }
        Ok(())
    }

    pub(crate) fn primary_mark_count(&self) -> u128 {
        self.theoretical.len() as u128
    }

    pub(crate) fn validate_ranges(&self, ranges: [(f64, f64); 2]) -> Result<(), Error> {
        if let Some(line) = self.line {
            visible_line_segment(line, ranges[0], ranges[1])?;
        }
        Ok(())
    }

    pub(crate) fn new(
        chart: &crate::builder::PlotDescription,
        data: PlotData<'_, '_>,
    ) -> Result<Self, Error> {
        data.validate_row_source()?;
        let alpha = chart.prepare_alpha(AlphaTarget::Point, AlphaUnitKind::Observation, &[])?;
        let line_alpha = chart
            .prepare_alpha(AlphaTarget::Line, AlphaUnitKind::Single, &[])?
            .single()?;
        if let Some(hue) = chart.hue_col() {
            return Err(Error::Data(format!(
                "Q-Q plot: hue column {hue:?} is not supported; render separate Q-Q plots"
            )));
        }
        let column = chart
            .x_col()
            .ok_or_else(|| Error::Data("Q-Q plot: value column not set".into()))?;
        let values = crate::data::strict_finite_f64(data, column, "Q-Q plot")?;
        let mut prepared = prepare_qq(&values, chart.qq_line_spec())?;
        prepared.line_alpha = line_alpha;
        for source in &mut prepared.alphas {
            *source = ObservationAlpha::resolve(data, &alpha, source.slot)?;
        }
        Ok(prepared)
    }

    pub(crate) fn ranges(
        &self,
        chart: &crate::builder::PlotDescription,
    ) -> Result<[(f64, f64); 2], Error> {
        Ok([
            qq_display_range(
                (
                    self.theoretical[0],
                    self.theoretical[self.theoretical.len() - 1],
                ),
                chart.x_axis_spec(),
                "x-axis",
            )?,
            qq_display_range(
                (self.observed[0], self.observed[self.observed.len() - 1]),
                chart.y_axis_spec(),
                "y-axis",
            )?,
        ])
    }
}

#[derive(Debug)]
pub(crate) struct QqPayload {
    clip: crate::layout::PixelRect,
    line: Option<[Point2D; 2]>,
    pub(crate) point_batches: Vec<(Vec<Point2D>, PointStyle)>,
    line_style: LineStyle,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

impl QqPayload {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        with_clip(backend, self.clip, |backend| {
            if let Some(line) = &self.line {
                backend.draw_line(line, &self.line_style)?;
            }
            for (points, style) in &self.point_batches {
                backend.draw_scatter(points, style)?;
            }
            Ok(())
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
        crate::orchestrator::registration::FamilyRef::Qq(marks) => Some(marks),
        _ => None,
    })
    .ok_or_else(|| {
        Error::Render("Qq measurement requires root-prepared semantics for its component".into())
    })?;
    let theme = request;

    let projection = layout.projection.clone();
    let mut result = project(chart, prepared, layout, context, request)?;
    let raw_windows = context.registered.raw_windows();
    if let Some(ink) = point_ink(
        &chart.point_style(
            theme,
            (0, 0, 0),
            theme.marks.get_points().get_radii().get_qq(),
        ),
        prepared
            .theoretical
            .iter()
            .zip(&prepared.observed)
            .zip(&prepared.alphas)
            .map(|((&x, &y), &alpha)| (x, y, alpha)),
    )? {
        let requests = crate::layout::interior::automatic_mark_ink_expansions(
            ink.ranges,
            &projection,
            ink.bounds,
            [raw_windows[0].is_none(), raw_windows[1].is_none()],
            context.fit_scope(),
        )?;
        let marks = std::sync::Arc::get_mut(&mut result.marks)
            .expect("newly selected Qq marks have one owner");
        let crate::layout::interior::MarkPayload::Qq(payload) = &mut marks.payload else {
            unreachable!("Qq selected its own mark payload")
        };
        payload.domain_expansions = requests;
    }
    Ok(result)
}

pub(crate) fn project(
    chart: &PlotDescription,
    prepared: &PreparedQq,
    layout: &crate::layout::Layout,
    context: &PlotMeasureContext<'_, '_>,
    request: &crate::text::TextRequestContext<'_, '_>,
) -> Result<MeasuredCoordinateMarks, Error> {
    let theme = request;
    let mut payload = QqPayload {
        clip: layout.projection.panel_box(),
        line: None,
        point_batches: Vec::new(),
        line_style: line_style(
            prepared.line_alpha,
            theme
                .marks
                .get_strokes()
                .get_references()
                .get_qq_reference()
                .backend_style(),
        ),
        domain_expansions: [None; 2],
    };
    let point_style = chart.point_style(
        theme,
        crate::theme::AccentColors::resolve(theme.marks.get_accents(), 1)?.color(0)?,
        theme.marks.get_points().get_radii().get_qq(),
    );

    if let Some(line) = prepared.line {
        if let Some(((line_x0, line_y0), (line_x1, line_y1))) = visible_line_segment(
            line,
            layout.projection.x_range(),
            layout.projection.y_range(),
        )? {
            let (x0, y0) = layout.projection.project(line_x0, line_y0);
            let (x1, y1) = layout.projection.project(line_x1, line_y1);
            payload.line = Some([Point2D::new(x0, y0), Point2D::new(x1, y1)]);
        }
    }
    crate::alpha::marks::require_aligned_points(prepared.theoretical.len(), prepared.alphas.len())?;
    for ((theoretical, observed), alpha) in prepared
        .theoretical
        .iter()
        .zip(&prepared.observed)
        .zip(&prepared.alphas)
    {
        let ranges = context.assignment.local.ranges().expect("Qq assigned pair");
        if *theoretical < ranges[0].0
            || *theoretical > ranges[0].1
            || *observed < ranges[1].0
            || *observed > ranges[1].1
        {
            continue;
        }
        let (x, y) = layout.projection.project(*theoretical, *observed);
        if x.is_finite() && y.is_finite() {
            push_point(
                &mut payload.point_batches,
                Point2D::new(x, y),
                alpha.point(point_style.clone()),
            )?;
        }
    }
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    if let Some(line) = &payload.line {
        geometry.line(line, &payload.line_style, Some(clip))?;
    }
    for (points, style) in &payload.point_batches {
        geometry.scatter(points, style, clip)?;
    }
    Ok(MeasuredCoordinateMarks::new(
        crate::data::source::source_arc(crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Qq(payload),
            geometry,
        })?,
    ))
}

pub(super) fn qq_display_range(
    data_range: (f64, f64),
    spec: &crate::layout::AuthoredAxisSpec,
    axis: &str,
) -> Result<(f64, f64), Error> {
    if let Some(limits) = spec.window.map(crate::layout::RawWindow::pair) {
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
    let display = crate::layout::continuous_auto_range(
        data_range,
        spec.window.map(crate::layout::RawWindow::pair),
    )?;
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
    let mut order = crate::data::source::collect_source((0..values.len()).map(Ok))?;
    crate::data::source::sort_source_by(&mut order, |a, b| {
        values[*a].total_cmp(&values[*b]).then(a.cmp(b))
    });
    let observed = crate::data::source::collect_source(order.iter().map(|&row| Ok(values[row])))?;
    let alphas = crate::data::source::collect_source(order.iter().map(|&slot| {
        Ok(ObservationAlpha {
            slot,
            source_row: slot,
            value: None,
        })
    }))?;
    let mut theoretical = Vec::new();
    crate::data::source::reserve_source(&mut theoretical, observed.len())?;

    for index in 0..observed.len() {
        theoretical.push(theoretical_quantile(index, observed.len())?);
    }
    let line = match line_mode {
        QqLine::Quartiles => {
            let theoretical_q1 = sorted_quantile(&theoretical, 0.25);
            let theoretical_q3 = sorted_quantile(&theoretical, 0.75);
            let observed_q1 = sorted_quantile(&observed, 0.25);
            let observed_q3 = sorted_quantile(&observed, 0.75);
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
        alphas,
        line,
        line_alpha: None,
    })
}

fn theoretical_quantile(index: usize, count: usize) -> Result<f64, Error> {
    standard_normal_quantile((index as f64 + 0.5) / count as f64)
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
        let source = Plot::qq(crate::NumericValues::new("value", [3.0, 1.0, 1.0, 2.0]))
            .reference_line(QqLine::Hidden)
            .with_alpha(
                AlphaTarget::Point,
                crate::alpha::AlphaEncoding::observations(
                    crate::alpha::AlphaValues::new("confidence", [230, 17, 90, 128]).unwrap(),
                ),
            )
            .x_limits(-2.0, 2.0)
            .y_limits(0.0, 4.0)
            .erase();
        let operation = crate::font::canonical::OperationContext::new();
        let prepared = crate::data::with_source_data_for_test(
            &source,
            &operation,
            crate::orchestrator::registration::RowSelection::Full,
            |data| PreparedQq::new(source.description(), data).unwrap(),
        )
        .unwrap();
        assert_eq!(prepared.observed, [1.0, 1.0, 2.0, 3.0]);
        assert_eq!(
            prepared
                .alphas
                .iter()
                .map(|alpha| alpha.slot)
                .collect::<Vec<_>>(),
            [1, 2, 3, 0]
        );
        assert!(prepared
            .theoretical
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
        let theme = crate::Theme::publication().resolve().unwrap();
        let text = crate::text::TextService::default();
        let theme = crate::text::TextRequestContext::new(&theme, &text);
        let selected = source
            .preflight_panel(
                source.kind(),
                crate::layout::PixelRect::new(10.0, 600.0, 20.0, 420.0),
                &theme,
            )
            .unwrap();
        let crate::layout::interior::MarkPayload::Qq(payload) =
            &selected.marks.as_ref().unwrap().payload
        else {
            panic!("Qq payload")
        };
        assert_eq!(
            payload
                .point_batches
                .iter()
                .map(|(_, style)| style.color.get_alpha())
                .collect::<Vec<_>>(),
            [17, 90, 128, 230]
        );
    }
}
