//! Single-series numeric histogram rendering and deterministic bin preparation.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use crate::data::PlotData;
use crate::{Error, HistogramKdePoint};

use crate::render::backend::{with_clip, Backend, FillStyle, LineStyle, Point2D};
use crate::spec::{
    HistogramBins, HistogramComposite, HistogramKde, HistogramStat, PlotRenderContext,
    PlotRenderResult,
};
use crate::Plot;

const MAX_AUTO_BINS: usize = 512;
const MAX_EXPLICIT_BINS: usize = 1_000_000;
const KDE_GRID_POINTS: usize = 128;

#[derive(Debug)]
pub(crate) struct PreparedHistogramInputs {
    pub(crate) panels: Vec<(Option<String>, HistogramSemantics)>,
}

#[derive(Debug)]
pub(crate) struct HistogramSemantics {
    pub(crate) ranges: [(f64, f64); 2],
    source_ranges: [(f64, f64); 2],
    bins: Vec<crate::HistogramBin>,
    curve: Option<Vec<HistogramKdePoint>>,
    rugs: Vec<crate::HistogramRug>,
}

pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let prepared = chart
        .prepared_histogram
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, prepared)| prepared)
        .ok_or_else(|| Error::Render("Histogram intrinsic requires root-prepared bins".into()))?;
    if prepared.bins.is_empty() {
        return Err(Error::Render(
            "Histogram has no prepared primary bins".into(),
        ));
    }
    let style = theme.marks.histogram_bin.backend_style((0, 0, 0));
    let fill = crate::render::geometry::rectangular_fill_ink_outset(&style)?;
    let curve = if prepared
        .curve
        .as_ref()
        .is_some_and(|points| points.len() >= 2)
    {
        crate::render::geometry::line_ink_width(&theme.marks.histogram_curve.backend_style())?
    } else {
        0.0
    };
    let rug = if prepared.rugs.is_empty() {
        0.0
    } else {
        crate::render::geometry::line_ink_width(&theme.marks.histogram_rug.backend_style())?
    };
    let span = (fill.0.max(fill.1) + 1.0 / 1024.0).max(curve).max(rug);
    Ok((
        if chart.user_x_limits().is_none() {
            span
        } else {
            0.0
        },
        if chart.user_y_limits().is_none() {
            span
        } else {
            0.0
        },
    ))
}

impl HistogramSemantics {
    pub(crate) fn primary_mark_count(&self) -> u128 {
        self.bins.len() as u128
    }

    pub(crate) fn new(chart: &Plot, data: PlotData<'_>) -> Result<Self, Error> {
        data.validate_row_source()?;
        #[cfg(test)]
        crate::layout::interior::distributions::count(0, 0);
        let (range, bins, curve, rugs) = if let Some(input) =
            chart.histogram_composite_geometry_input()
        {
            let range = input.validate()?;
            (
                range,
                crate::data::source::collect_source(input.bins.iter().copied().map(Ok))?,
                Some(crate::data::source::collect_source(
                    input.kde.iter().copied().map(Ok),
                )?),
                crate::data::source::collect_source(input.rugs.iter().copied().map(Ok))?,
            )
        } else {
            if let Some(hue) = chart.hue_col() {
                return Err(Error::Data(format!("histogram: hue column {hue:?} is not supported yet; render separate single-series histograms")));
            }
            let column = chart
                .x_col()
                .ok_or_else(|| Error::Data("histogram: value column not set".into()))?;
            let values = crate::data::strict_finite_f64(data, column, "histogram")?;
            let prepared = prepare_histogram(
                &values,
                chart.histogram_bins_spec(),
                chart.histogram_stat_spec(),
            )?;
            let range = (prepared.edges[0], prepared.edges[prepared.edges.len() - 1]);
            let curve = prepare_density_curve(chart.histogram_composite_spec(), &values, range)?;
            let bins = crate::data::source::collect_source(
                prepared.heights.iter().enumerate().map(|(i, height)| {
                    Ok(crate::HistogramBin {
                        lower: prepared.edges[i],
                        upper: prepared.edges[i + 1],
                        density: *height,
                    })
                }),
            )?;

            let mut maximum = prepared.heights.iter().copied().fold(0.0_f64, f64::max);
            if let Some(curve) = &curve {
                maximum = curve.iter().map(|p| p.y).fold(maximum, f64::max);
            }
            let rugs = if curve.is_some() {
                crate::data::source::collect_source(values.iter().map(|x| {
                    Ok(crate::HistogramRug {
                        x: *x,
                        baseline: 0.0,
                        top: maximum * 0.035,
                    })
                }))?
            } else {
                Vec::new()
            };
            (range, bins, curve, rugs)
        };

        let maximum = bins
            .iter()
            .map(|b| b.density)
            .chain(curve.iter().flatten().map(|p| p.y))
            .chain(rugs.iter().map(|r| r.top))
            .fold(0.0_f64, f64::max);
        Ok(Self {
            ranges: [
                chart.x_axis_spec().limits.unwrap_or(range),
                crate::layout::anchored_auto_range((0.0, maximum), chart.y_axis_spec(), 0.0)?,
            ],
            source_ranges: [range, (0.0, maximum)],
            bins,
            curve,
            rugs,
        })
    }
}

#[derive(Debug)]
pub(crate) struct HistogramPayload {
    clip: crate::layout::PanelBox,
    pub(crate) bins: Vec<[Point2D; 4]>,
    curve: Option<Vec<Point2D>>,
    rugs: Vec<[Point2D; 2]>,
    pub(crate) fill: FillStyle,
    curve_style: LineStyle,
    rug_style: LineStyle,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

impl HistogramPayload {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        #[cfg(test)]
        crate::layout::interior::distributions::count(0, 3);
        with_clip(backend, self.clip, |backend| {
            for bin in &self.bins {
                backend.draw_polygon(bin, &self.fill)?;
            }
            if let Some(curve) = &self.curve {
                backend.draw_line(curve, &self.curve_style)?;
            }
            for rug in &self.rugs {
                backend.draw_line(rug, &self.rug_style)?;
            }
            Ok(())
        })
    }
}

pub(crate) fn measure_marks(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    layout: crate::layout::Layout,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    crate::layout::interior::distributions::count(0, 1);
    let prepared = chart
        .prepared_histogram
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
                "Histogram measurement requires root-prepared semantics for its component".into(),
            )
        })?;
    let theme = context.theme;

    let projection = layout.projection.clone();
    let mut result = project(prepared, layout, theme)?;
    let fill = theme.marks.histogram_bin.backend_style((0, 0, 0));
    let mut ink = crate::render::geometry::rectangular_fill_ink_bounds(&fill)?;
    if prepared
        .curve
        .as_ref()
        .is_some_and(|curve| curve.len() >= 2)
    {
        let curve =
            crate::render::geometry::line_ink_bounds(&theme.marks.histogram_curve.backend_style())?;
        ink = match (ink, curve) {
            (Some(fill), Some(curve)) => Some(fill.union(curve)),
            (fill, curve) => fill.or(curve),
        };
    }
    if !prepared.rugs.is_empty() {
        let rug =
            crate::render::geometry::line_ink_bounds(&theme.marks.histogram_rug.backend_style())?;
        ink = match (ink, rug) {
            (Some(ink), Some(rug)) => Some(ink.union(rug)),
            (ink, rug) => ink.or(rug),
        };
    }
    if let Some(ink) = ink {
        let requests = crate::layout::interior::automatic_mark_ink_expansions(
            prepared.source_ranges,
            &projection,
            ink,
            [
                chart.user_x_limits().is_none(),
                chart.user_y_limits().is_none(),
            ],
            context.fit_scope(),
        )?;
        let marks =
            std::sync::Arc::get_mut(result.marks.as_mut().expect("selected Histogram marks"))
                .expect("newly selected Histogram marks have one owner");
        let crate::layout::interior::MarkPayload::Histogram(payload) = &mut marks.payload else {
            unreachable!("Histogram selected its own mark payload")
        };
        payload.domain_expansions = requests;
    }
    Ok(result)
}

pub(crate) fn project(
    prepared: &HistogramSemantics,
    layout: crate::layout::Layout,
    theme: &crate::theme::ResolvedTheme,
) -> Result<PlotRenderResult, Error> {
    let project = |x, y| {
        let (x, y) = layout.projection.project(x, y);
        Point2D::new(x, y)
    };
    let payload = HistogramPayload {
        clip: layout.projection.panel_box(),
        bins: crate::data::source::collect_source(prepared.bins.iter().map(|bin| {
            let left = project(bin.lower, 0.0);
            let right = project(bin.upper, bin.density);
            layout.projection.validate_span(
                crate::layout::AxisSide::Bottom,
                (bin.lower, bin.upper),
                (left.x, right.x),
                bin.lower < bin.upper,
                "histogram polygon width",
            )?;
            layout.projection.validate_span(
                crate::layout::AxisSide::Left,
                (0.0, bin.density),
                (left.y, right.y),
                bin.density != 0.0,
                "histogram polygon height",
            )?;
            Ok([
                left,
                Point2D::new(right.x, left.y),
                right,
                Point2D::new(left.x, right.y),
            ])
        }))?,
        curve: prepared
            .curve
            .as_ref()
            .map(|curve| {
                crate::data::source::collect_source(curve.iter().map(|p| Ok(project(p.x, p.y))))
            })
            .transpose()?,
        rugs: crate::data::source::collect_source(prepared.rugs.iter().map(|r| {
            let points = [project(r.x, r.baseline), project(r.x, r.top)];
            layout.projection.validate_span(
                crate::layout::AxisSide::Left,
                (r.baseline, r.top),
                (points[0].y, points[1].y),
                r.baseline < r.top,
                "histogram rug interval",
            )?;
            Ok(points)
        }))?,
        fill: theme
            .marks
            .histogram_bin
            .backend_style(crate::theme::AccentColors::resolve(&theme.marks.accents, 1)?.color(0)?),
        curve_style: theme.marks.histogram_curve.backend_style(),
        rug_style: theme.marks.histogram_rug.backend_style(),
        domain_expansions: [None; 2],
    };
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    #[cfg(test)]
    crate::layout::interior::distributions::count(0, 2);
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    for bin in &payload.bins {
        geometry.polygon(bin, &payload.fill, Some(clip))?;
    }
    if let Some(curve) = &payload.curve {
        geometry.line(curve, &payload.curve_style, Some(clip))?;
    }
    for rug in &payload.rugs {
        geometry.line(rug, &payload.rug_style, Some(clip))?;
    }
    let mut result = PlotRenderResult::new(layout, Vec::new());
    result.marks = Some(crate::data::source::source_arc(
        crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Histogram(payload),
            geometry,
        },
    )?);
    Ok(result)
}

#[cfg(test)]
pub(crate) fn axis_ranges(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<[(f64, f64); 2], Error> {
    let (range, maximum) = if let Some(input) = chart.histogram_composite_geometry_input() {
        let range = input.validate()?;

        (
            range,
            input
                .bins
                .iter()
                .map(|bin| bin.density)
                .chain(input.kde.iter().map(|point| point.y))
                .chain(input.rugs.iter().map(|rug| rug.top))
                .fold(0.0_f64, f64::max),
        )
    } else {
        let column = chart
            .x_col()
            .ok_or_else(|| Error::Data("histogram: value column not set".into()))?;
        let values = crate::data::strict_finite_f64(data, column, "histogram")?;
        let prepared = prepare_histogram(
            &values,
            chart.histogram_bins_spec(),
            chart.histogram_stat_spec(),
        )?;
        let range = (prepared.edges[0], prepared.edges[prepared.edges.len() - 1]);
        let curve_maximum = density_maximum(chart.histogram_composite_spec(), &values, range)?;

        let maximum = prepared
            .heights
            .iter()
            .copied()
            .fold(curve_maximum, f64::max);
        (range, maximum)
    };
    Ok([
        chart.x_axis_spec().limits.unwrap_or(range),
        crate::layout::anchored_auto_range((0.0, maximum), chart.y_axis_spec(), 0.0)?,
    ])
}

pub(crate) fn shared_x_domain(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<(f64, f64), Error> {
    if let Some(input) = chart.histogram_composite_geometry_input() {
        return input.validate();
    }
    let column = chart
        .x_col()
        .ok_or_else(|| Error::Data("histogram: value column not set".into()))?;
    let values = crate::data::strict_finite_f64(data, column, "histogram")?;
    let edges = prepare_edges(&values, chart.histogram_bins_spec())?;
    Ok((edges[0], edges[edges.len() - 1]))
}

#[derive(Debug)]
struct PreparedHistogram {
    edges: Vec<f64>,
    heights: Vec<f64>,
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

            let grid: [f64; KDE_GRID_POINTS] =
                std::array::from_fn(|index| histogram_grid_coordinate(x_range, index));
            let densities = crate::stats::distribution::gaussian_kde(values, &grid, bandwidth)
                .map_err(|error| match error {
                    Error::Data(_) => Error::Data(format!("histogram KDE: {error}")),
                    error => error,
                })?;
            let mut curve = Vec::new();
            crate::data::source::reserve_source(&mut curve, KDE_GRID_POINTS)?;

            curve.extend(
                grid.into_iter()
                    .zip(densities)
                    .map(|(x, y)| HistogramKdePoint { x, y }),
            );
            curve
        }
        HistogramKde::Precomputed(points) => {
            validate_precomputed_kde(points, x_range)?;
            let mut curve = Vec::new();
            crate::data::source::reserve_source(&mut curve, points.len())?;

            curve.extend_from_slice(points);
            curve
        }
    };
    Ok(Some(curve))
}

#[cfg(test)]
fn density_maximum(
    composite: &HistogramComposite,
    values: &[f64],
    range: (f64, f64),
) -> Result<f64, Error> {
    let HistogramComposite::KdeAndRug(kde) = composite else {
        return Ok(0.0);
    };
    match kde {
        HistogramKde::Precomputed(points) => {
            validate_precomputed_kde(points, range)?;

            Ok(points.iter().map(|point| point.y).fold(0.0, f64::max))
        }
        HistogramKde::Internal => {
            let bandwidth = crate::stats::distribution::strict_scott_bandwidth(values)
                .map_err(|reason| Error::Data(format!("histogram KDE: {reason}")))?;

            let grid: [f64; KDE_GRID_POINTS] =
                std::array::from_fn(|index| histogram_grid_coordinate(range, index));
            let densities = crate::stats::distribution::gaussian_kde(values, &grid, bandwidth)?;

            Ok(densities.into_iter().fold(0.0, f64::max))
        }
    }
}

fn histogram_grid_coordinate(range: (f64, f64), index: usize) -> f64 {
    range.0 + (range.1 - range.0) * index as f64 / (KDE_GRID_POINTS - 1) as f64
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
    finish_histogram(values, prepare_edges(values, bins)?, stat)
}

fn prepare_edges(values: &[f64], bins: HistogramBins) -> Result<Vec<f64>, Error> {
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
        return constant_edges(minimum, bins);
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
            equal_edges(minimum, maximum, count)
        }
        HistogramBins::Count(count) => equal_edges(minimum, maximum, count),
        HistogramBins::Width(width) => requested_width_edges(minimum, maximum, width),
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
    let mut sorted = Vec::new();
    crate::data::source::reserve_source(&mut sorted, values.len())?;
    sorted.extend_from_slice(values);
    crate::data::source::sort_numeric_source(&mut sorted);
    let q1 = crate::stats::distribution::sorted_quantile(&sorted, 0.25);
    let q3 = crate::stats::distribution::sorted_quantile(&sorted, 0.75);
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
    let mut edges = Vec::new();
    crate::data::source::reserve_source(&mut edges, 2)?;
    let preferred = [value - 0.5, value + 0.5];
    if valid_edges(&preferred) {
        edges.extend_from_slice(&preferred);
        return Ok(edges);
    }
    let adjacent = [value.next_down(), value.next_up()];
    if valid_edges(&adjacent) {
        edges.extend_from_slice(&adjacent);
        return Ok(edges);
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
        if let Some(edges) = try_equal_edges(lower, upper, count)? {
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
        let mut retained = Vec::new();
        crate::data::source::reserve_source(&mut retained, edges.len())?;
        retained.extend_from_slice(&edges);
        return Ok(retained);
    }
    Err(Error::Data(format!(
        "histogram: Width({width}) cannot produce distinct finite centered edges around constant value {value}; use a wider bin"
    )))
}

fn equal_edges(lower: f64, upper: f64, count: usize) -> Result<Vec<f64>, Error> {
    try_equal_edges(lower, upper, count)?.ok_or_else(|| {
        Error::Data(
            "histogram: equal-width bin edges are not distinct and finite at this numeric scale; use fewer bins or a wider range"
                .into(),
        )
    })
}

fn try_equal_edges(lower: f64, upper: f64, count: usize) -> Result<Option<Vec<f64>>, Error> {
    if count == 0 || count > MAX_EXPLICIT_BINS || !lower.is_finite() || !upper.is_finite() {
        return Ok(None);
    }
    let range = upper - lower;
    let width = range / count as f64;
    if !range.is_finite() || !width.is_finite() || width <= 0.0 {
        return Ok(None);
    }
    let mut edges = Vec::new();
    crate::data::source::reserve_source(&mut edges, count + 1)?;
    for index in 0..=count {
        edges.push(lower + width * index as f64);
    }
    edges[count] = upper;

    Ok(valid_edges(&edges).then_some(edges))
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

    let mut edges = Vec::new();
    crate::data::source::reserve_source(&mut edges, count + 1)?;
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

    let mut counts = Vec::new();
    crate::data::source::reserve_source(&mut counts, count)?;
    counts.resize(count, 0_usize);

    for value in values {
        if *value < lower || *value > upper {
            return Err(Error::Data(format!(
                "histogram: value {value} falls outside computed edges {lower}..{upper}"
            )));
        }
        let mut left = 0;
        let mut right = edges.len();
        while left < right {
            let middle = left + (right - left) / 2;
            if edges[middle] <= *value {
                left = middle + 1;
            } else {
                right = middle;
            }
        }
        let insertion = left;
        let index = insertion.saturating_sub(1).min(count - 1);
        counts[index] += 1;
    }

    let sample_count = values.len() as f64;
    let mut heights = Vec::new();
    crate::data::source::reserve_source(&mut heights, count)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram_bin_and_present_overlay_ink_are_independent_hard_inputs() {
        let sample = || {
            Plot::histogram(crate::HistogramInput::samples(crate::NumericValues::new(
                "sample",
                [1.0, 2.0, 3.0, 4.0],
            )))
            .bins(crate::HistogramBins::Count(2))
            .stat(HistogramStat::Density)
            .x_limits(0.0, 5.0)
        };
        let source = sample().erase();
        let resources = crate::font::canonical::OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
        let chart = inputs.attach(source).unwrap();
        let mut theme = crate::Theme::Publication.resolve().unwrap();
        let baseline = chart.intrinsic_body_minimum(&theme).unwrap();
        theme.marks.histogram_curve.width = 140.0;
        theme.marks.histogram_rug.width = 140.0;
        assert_eq!(chart.intrinsic_body_minimum(&theme).unwrap(), baseline);
        theme.marks.histogram_bin.outline = Some(crate::theme::DataOutlineTheme::fixed(
            theme
                .marks
                .histogram_bin
                .outline()
                .unwrap()
                .fixed_stroke()
                .unwrap()
                .clone()
                .with_width(140.0)
                .unwrap(),
        ));
        let styled = chart.intrinsic_body_minimum(&theme).unwrap();
        assert_eq!(styled.0.to_bits(), baseline.0.to_bits());
        assert!(styled.1 > 140.0);
        theme.marks.histogram_bin.outline = Some(crate::theme::DataOutlineTheme::fixed(
            crate::theme::OutlineTheme::new(crate::theme::Color::rgba(1, 2, 3, 0), 140.0).unwrap(),
        ));
        assert_eq!(chart.intrinsic_body_minimum(&theme).unwrap(), baseline);

        let source = sample()
            .composite(crate::HistogramComposite::KdeAndRug(
                crate::HistogramKde::Internal,
            ))
            .erase();
        let resources = crate::font::canonical::OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
        let chart = inputs.attach(source).unwrap();
        let overlay = chart.intrinsic_body_minimum(&theme).unwrap();
        assert_eq!(overlay.0.to_bits(), baseline.0.to_bits());
        assert_eq!(overlay.1.to_bits(), 140.0_f64.to_bits());
    }

    #[test]
    fn histogram_domains_and_kde_maxima_match_mark_preparation() {
        for values in [&[1.0, 2.0, 3.0][..], &[5.0, 5.0], &[-10.0, 0.0, 1.0, 10.0]] {
            for bins in [
                HistogramBins::Auto,
                HistogramBins::Count(3),
                HistogramBins::Width(2.0),
            ] {
                for stat in [
                    HistogramStat::Count,
                    HistogramStat::Probability,
                    HistogramStat::Density,
                ] {
                    let expected = super::prepare_histogram(values, bins, stat).unwrap();
                    let actual = super::prepare_histogram(values, bins, stat).unwrap();
                    assert_eq!(actual.edges, expected.edges);
                    assert_eq!(actual.heights, expected.heights);
                }
            }
        }
        let values = [1.0, 2.0, 3.0];
        let composite = HistogramComposite::KdeAndRug(HistogramKde::Internal);
        let curve = prepare_density_curve(&composite, &values, (1.0, 3.0))
            .unwrap()
            .unwrap();
        let expected = curve.iter().map(|point| point.y).fold(0.0, f64::max);
        assert_eq!(
            density_maximum(&composite, &values, (1.0, 3.0))
                .unwrap()
                .to_bits(),
            expected.to_bits()
        );
        let supplied = HistogramComposite::KdeAndRug(HistogramKde::Precomputed(curve));

        assert_eq!(
            density_maximum(&supplied, &values, (1.0, 3.0))
                .unwrap()
                .to_bits(),
            expected.to_bits()
        );
    }

    fn prepare_histogram(
        values: &[f64],
        bins: HistogramBins,
        stat: HistogramStat,
    ) -> Result<PreparedHistogram, Error> {
        super::prepare_histogram(values, bins, stat)
    }

    fn finish_histogram(
        values: &[f64],
        edges: Vec<f64>,
        stat: HistogramStat,
    ) -> Result<PreparedHistogram, Error> {
        super::finish_histogram(values, edges, stat)
    }

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
