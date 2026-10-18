//! Standalone Gaussian kernel density plots.

#![allow(clippy::cast_precision_loss)]

use crate::data::PlotData;
use crate::Error;

use crate::decorations::legend::GuideSourceKey;
use crate::layout::Layout;
use crate::render::backend::{with_clip, Backend, FillStyle, LineStyle, Point2D};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::stats::distribution::strict_scott_bandwidth;
use crate::theme::{AccentColors, CategoryColors};
use crate::Plot;

const MIN_GRID_POINTS: usize = 128;
const MAX_GRID_POINTS: usize = 2_048;
const POINTS_PER_BANDWIDTH: f64 = 12.0;
const SUPPORT_BANDWIDTHS: f64 = 3.0;

#[derive(Debug)]
struct DensityCurve {
    identity: crate::data::CollectivePair,
    grid: Vec<f64>,
    densities: Vec<f64>,
}

#[derive(Debug)]
pub(crate) struct PreparedDensity {
    x_range: (f64, f64),
    maximum: f64,
    curves: Vec<DensityCurve>,
}

#[derive(Debug)]
pub(crate) struct PreparedDensityInputs {
    pub(crate) panels: Vec<(Option<String>, PreparedDensity)>,
}

/// Group curves, rather than their generated KDE vertices, are the semantic
/// primary marks. The resolved stroke is measured before parent assignment.
pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let prepared = chart
        .prepared_density
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, prepared)| prepared)
        .ok_or_else(|| Error::Render("Density intrinsic requires root-prepared curves".into()))?;
    if prepared.curves.is_empty() {
        return Err(Error::Render("Density has no retained group curves".into()));
    }
    let line = theme
        .marks
        .data_lines()
        .density_line()
        .backend_style((0, 0, 0));
    let ink = crate::render::geometry::line_ink_width(&line)?;
    let area = crate::render::geometry::rectangular_fill_ink_outset(
        &theme
            .marks
            .fills()
            .densities()
            .density_area()
            .backend_style((0, 0, 0)),
    )?;
    Ok((
        if chart.user_x_limits().is_none() {
            ink.max(area.0)
        } else {
            0.0
        },
        if chart.user_y_limits().is_none() {
            ink.max(area.1)
        } else {
            0.0
        },
    ))
}

impl PreparedDensity {
    fn resolved_colors(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        data: PlotData<'_>,
    ) -> Result<(CategoryColors, AccentColors), Error> {
        let resources = data.source_resources()?;
        let keys = if let Some(global) = chart.global_hue_levels() {
            crate::data::source::collect_source(
                global
                    .iter()
                    .map(|key| crate::data::source::copy_source_string(key)),
            )?
        } else {
            let mut observed = Vec::new();
            crate::data::source::reserve_source(&mut observed, self.curves.len())?;

            for curve in &self.curves {
                if let Some(hue) = &curve.identity.hue {
                    observed.push(crate::data::source::copy_source_string(hue)?);
                }
            }
            crate::data::source::source_levels(&observed)?
        };
        let map = CategoryColors::resolve(
            &keys,
            theme.color_scales.qualitative(),
            chart.categorical_color_scale_spec(),
            resources,
        )?;
        let accent = AccentColors::resolve(
            theme.marks.accents(),
            usize::from(self.curves.iter().any(|curve| curve.identity.hue.is_none())),
        )?;
        Ok((map, accent))
    }

    fn curve_color(
        curve: &DensityCurve,
        colors: &CategoryColors,
        accent: &AccentColors,
    ) -> Result<(u8, u8, u8), Error> {
        if let Some(hue) = &curve.identity.hue {
            colors.color(hue)
        } else {
            accent.color(0)
        }
    }
    pub(crate) fn primary_mark_count(&self) -> u128 {
        self.curves.len() as u128
    }

    pub(crate) fn hue_levels(&self) -> impl Iterator<Item = &str> {
        self.curves
            .iter()
            .filter_map(|curve| curve.identity.hue.as_deref())
    }

    pub(crate) fn group_order(&self, chart: &Plot) -> Result<Vec<(usize, usize)>, Error> {
        let mut order = crate::data::source::collect_source(self.curves.iter().enumerate().map(
            |(index, curve)| {
                let pairs = chart.global_collective_pairs().unwrap_or(&[]);

                let key = pairs
                    .iter()
                    .position(|pair| pair == &curve.identity)
                    .unwrap_or(index);
                Ok((key, index))
            },
        ))?;
        crate::data::source::sort_source_by(&mut order, |a, b| (a.0, a.1).cmp(&(b.0, b.1)));
        Ok(order)
    }

    pub(crate) fn guide_entries_in_order(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        data: PlotData<'_>,
        order: &[(usize, usize)],
    ) -> Result<Vec<GuideSourceKey>, Error> {
        let (colors, accent) = self.resolved_colors(chart, theme, data)?;
        self.guide_entries_with_colors(chart, theme, order, &colors, &accent)
    }

    fn guide_entries_with_colors(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,

        order: &[(usize, usize)],
        colors: &CategoryColors,
        accent: &AccentColors,
    ) -> Result<Vec<GuideSourceKey>, Error> {
        let mut entries = Vec::new();
        crate::data::source::reserve_source(&mut entries, order.len())?;
        for &(_, index) in order {
            let curve = &self.curves[index];
            if curve.identity.hue.is_none() {
                continue;
            }
            let color = Self::curve_color(curve, colors, accent)?;
            entries.push(GuideSourceKey::line_fill(
                chart.collective_legend_identity(&curve.identity)?,
                theme.marks.data_lines().density_line().backend_style(color),
                theme
                    .marks
                    .fills()
                    .densities()
                    .density_area()
                    .backend_style(color),
            ));
        }
        Ok(entries)
    }

    pub(crate) fn new(chart: &Plot, data: PlotData<'_>) -> Result<Self, Error> {
        data.validate_row_source()?;
        #[cfg(test)]
        crate::layout::interior::distributions::count(1, 0);
        let column = chart
            .x_col()
            .ok_or_else(|| Error::Data("density: value column not set".into()))?;
        let prepared = prepare_density_for_chart(chart, data, column)?;
        if let Some(exact) = chart.categorical_map_spec() {
            for hue in prepared.hue_levels() {
                if exact.color_in(hue).is_none() {
                    return Err(Error::Data(format!(
                        "density: category {hue:?} has no caller-owned color"
                    )));
                }
            }
        }
        Ok(prepared)
    }

    pub(crate) fn ranges(&self, chart: &Plot) -> Result<[(f64, f64); 2], Error> {
        Ok([
            chart.x_axis_spec().limits.unwrap_or(self.x_range),
            crate::layout::anchored_auto_range((0.0, self.maximum), chart.y_axis_spec(), 0.0)?,
        ])
    }
}

#[derive(Debug)]
pub(crate) struct DensityPayload {
    clip: crate::layout::PanelBox,
    pub(crate) curves: Vec<DensityCurvePayload>,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

#[derive(Debug)]
pub(crate) struct DensityCurvePayload {
    pub(crate) fill: Vec<Point2D>,
    segments: Vec<Vec<Point2D>>,
    pub(crate) fill_style: FillStyle,
    line_style: LineStyle,
}

impl DensityPayload {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        #[cfg(test)]
        crate::layout::interior::distributions::count(1, 3);
        with_clip(backend, self.clip, |backend| {
            for curve in &self.curves {
                if curve.fill.len() >= 3 {
                    backend.draw_polygon(&curve.fill, &curve.fill_style)?;
                }
                for segment in &curve.segments {
                    backend.draw_line(segment, &curve.line_style)?;
                }
            }
            Ok(())
        })
    }
}

pub(crate) fn measure_marks(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    layout: Layout,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    crate::layout::interior::distributions::count(1, 1);
    let prepared = chart
        .prepared_density
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
                "Density measurement requires root-prepared semantics for its component".into(),
            )
        })?;
    let theme = context.theme;
    let resources = theme.prepared_font_environment()?.operation();
    let data = context.data.with_operation(&resources);
    let projection = layout.projection.clone();
    let mut result = project(chart, prepared, layout, theme, data)?;
    let automatic = [
        chart.user_x_limits().is_none(),
        chart.user_y_limits().is_none(),
    ];
    let mut requests = [None; 2];
    if let Some(ink) = crate::render::geometry::line_ink_bounds(
        &theme
            .marks
            .data_lines()
            .density_line()
            .backend_style((0, 0, 0)),
    )? {
        requests = crate::layout::interior::automatic_mark_ink_expansions(
            [prepared.x_range, (0.0, prepared.maximum)],
            &projection,
            ink,
            automatic,
            context.fit_scope(),
        )?;
    }
    let crate::layout::interior::MarkPayload::Density(payload) = &result
        .marks
        .as_ref()
        .expect("selected Density marks")
        .payload
    else {
        unreachable!("Density selected its own mark payload")
    };
    let mut area = crate::render::geometry::GeometrySnapshot::new();
    for curve in &payload.curves {
        if curve.fill.len() >= 3 {
            area.polygon(&curve.fill, &curve.fill_style, None)?;
        }
    }
    if let Some(bounds) = area
        .bounds()
        .iter()
        .copied()
        .reduce(crate::render::geometry::GeometryBox::union)
    {
        let additional = crate::layout::interior::automatic_absolute_ink_expansions(
            [prepared.x_range, (0.0, prepared.maximum)],
            &projection,
            bounds,
            automatic,
            context.fit_scope(),
        )?;
        requests = crate::layout::interior::merge_mark_domain_expansions(requests, additional);
    }
    let marks = std::sync::Arc::get_mut(result.marks.as_mut().expect("selected Density marks"))
        .expect("newly selected Density marks have one owner");
    let crate::layout::interior::MarkPayload::Density(payload) = &mut marks.payload else {
        unreachable!("Density selected its own mark payload")
    };
    payload.domain_expansions = requests;
    Ok(result)
}

pub(crate) fn project(
    chart: &Plot,
    prepared: &PreparedDensity,
    layout: Layout,
    theme: &crate::theme::ResolvedTheme,
    data: PlotData<'_>,
) -> Result<PlotRenderResult, Error> {
    let mut payload = DensityPayload {
        clip: layout.projection.panel_box(),
        curves: Vec::new(),
        domain_expansions: [None; 2],
    };
    crate::data::source::reserve_source(&mut payload.curves, prepared.curves.len())?;
    let order = prepared.group_order(chart)?;
    let (colors, accent) = prepared.resolved_colors(chart, theme, data)?;

    for &(_, index) in &order {
        let curve = &prepared.curves[index];
        let visible = visible_curve(curve, layout.projection.x_range())?;
        if visible.len() < 2 {
            continue;
        }
        let color = PreparedDensity::curve_color(curve, &colors, &accent)?;
        let mut fill = Vec::new();
        crate::data::source::reserve_source(&mut fill, visible.len() + 2)?;
        fill.push((visible[0].0, 0.0));
        fill.extend(visible.iter().copied());
        fill.push((visible[visible.len() - 1].0, 0.0));
        let fill = clip_polygon_y(fill, layout.projection.y_range())?;
        let mut projected = DensityCurvePayload {
            fill: if fill.len() >= 3 {
                project_points(&fill, &layout, true)?
            } else {
                Vec::new()
            },
            segments: Vec::new(),
            fill_style: theme
                .marks
                .fills()
                .densities()
                .density_area()
                .backend_style(color),
            line_style: theme.marks.data_lines().density_line().backend_style(color),
        };
        crate::data::source::reserve_source(&mut projected.segments, visible.len() - 1)?;
        for pair in visible.windows(2) {
            if let Some(segment) = clip_line_y(pair[0], pair[1], layout.projection.y_range()) {
                projected
                    .segments
                    .push(project_points(&segment, &layout, false)?);
            }
        }
        payload.curves.push(projected);
    }

    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    #[cfg(test)]
    crate::layout::interior::distributions::count(1, 2);
    let clip = Some(crate::render::backend::validate_clip_region(payload.clip)?.panel());
    for curve in &payload.curves {
        geometry.polygon(&curve.fill, &curve.fill_style, clip)?;
        for segment in &curve.segments {
            geometry.line(segment, &curve.line_style, clip)?;
        }
    }
    let mut result = PlotRenderResult::new(layout, chart.prepared_discrete_guide()?);
    result.marks = Some(crate::data::source::source_arc(
        crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Density(payload),
            geometry,
        },
    )?);
    Ok(result)
}

#[cfg(test)]
fn prepare_density<'a>(
    data: impl Into<crate::data::PlotData<'a>>,
    value_col: &str,
    group_col: Option<&str>,
    group_name: Option<&str>,
    hue_col: Option<&str>,
    hue_name: Option<&str>,
) -> Result<PreparedDensity, Error> {
    prepare_density_ordered(
        data, value_col, group_col, group_name, hue_col, hue_name, None,
    )
}

pub(crate) fn shared_x_domain(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<(f64, f64), Error> {
    let column = chart
        .x_col()
        .ok_or_else(|| Error::Data("density: value column not set".into()))?;
    let values = crate::data::strict_finite_f64(data, column, "density")?;
    let channels = crate::data::observed_collectives(
        data,
        chart.group_col(),
        chart.group_name(),
        chart.hue_col(),
        chart.hue_name(),
        None,
        "density",
    )?;
    if channels.rows.len() != values.len() {
        return Err(Error::Data(
            "density: value, graphical group, and hue columns must have equal lengths".into(),
        ));
    }
    let mut range = (f64::INFINITY, f64::NEG_INFINITY);
    let mut samples = Vec::new();
    crate::data::source::reserve_source(&mut samples, values.len())?;
    for identity in &channels.identities {
        samples.clear();

        samples.extend(
            channels
                .rows
                .iter()
                .zip(&values)
                .filter_map(|(row, value)| (row == identity).then_some(*value)),
        );
        // Scott bandwidth scans five times; support adds one min/max pass.

        let (low, high, _) =
            density_support(&samples, || identity.description(channels.collapsed))?;
        range = (range.0.min(low), range.1.max(high));
    }
    Ok(range)
}

fn density_support(
    samples: &[f64],
    description: impl Fn() -> String,
) -> Result<(f64, f64, f64), Error> {
    if samples.len() < 2 {
        return Err(Error::Data(format!(
            "density: {} requires at least 2 observations, got {}",
            description(),
            samples.len()
        )));
    }
    let bandwidth = strict_scott_bandwidth(samples)
        .map_err(|reason| Error::Data(format!("density: {} {reason}", description())))?;
    let (minimum, maximum) = crate::data::min_max(samples.iter().copied(), "density")?;
    let low = minimum - SUPPORT_BANDWIDTHS * bandwidth;
    let high = maximum + SUPPORT_BANDWIDTHS * bandwidth;
    if !low.is_finite() || !high.is_finite() || low >= high {
        return Err(Error::Data(format!(
            "density: {} cannot produce finite KDE support",
            description()
        )));
    }
    Ok((low, high, bandwidth))
}

fn prepare_density_for_chart<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
    value_col: &str,
) -> Result<PreparedDensity, Error> {
    prepare_density_ordered(
        data,
        value_col,
        chart.group_col(),
        chart.group_name(),
        chart.hue_col(),
        chart.hue_name(),
        chart.global_collective_pairs(),
    )
}

fn prepare_density_ordered<'a>(
    data: impl Into<crate::data::PlotData<'a>>,
    value_col: &str,
    group_col: Option<&str>,
    group_name: Option<&str>,
    hue_col: Option<&str>,
    hue_name: Option<&str>,
    identity_order: Option<&[crate::data::CollectivePair]>,
) -> Result<PreparedDensity, Error> {
    let data = data.into();
    let values = crate::data::strict_finite_f64(data, value_col, "density")?;
    let channels = crate::data::observed_collectives(
        data, group_col, group_name, hue_col, hue_name, None, "density",
    )?;
    if channels.rows.len() != values.len() {
        return Err(Error::Data(
            "density: value, graphical group, and hue columns must have equal lengths".into(),
        ));
    }

    let ordered_identities = identity_order.unwrap_or(&channels.identities);
    let mut support_low = f64::INFINITY;
    let mut support_high = f64::NEG_INFINITY;
    let mut curves = Vec::new();
    let mut maximum = 0.0_f64;
    crate::data::source::reserve_source(&mut curves, ordered_identities.len())?;
    let mut samples = Vec::new();
    crate::data::source::reserve_source(&mut samples, values.len())?;
    for identity in ordered_identities {
        samples.clear();

        samples.extend(
            channels
                .rows
                .iter()
                .zip(&values)
                .filter_map(|(row, value)| (row == identity).then_some(*value)),
        );
        if samples.is_empty() {
            continue;
        }

        let description = || identity.description(channels.collapsed);
        let (low, high, bandwidth) = density_support(&samples, description)?;
        support_low = support_low.min(low);
        support_high = support_high.max(high);
        let span_in_bandwidths = scaled_difference_ratio(high, low, bandwidth);
        if !span_in_bandwidths.is_finite() && !span_in_bandwidths.is_infinite() {
            return Err(Error::Data(format!(
                "density: {} grid not representable at this numeric scale",
                description()
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
                "density: {} grid not representable at this numeric scale",
                description()
            )));
        }
        let mut grid = Vec::new();
        crate::data::source::reserve_source(&mut grid, interval_count + 1)?;
        grid.extend(
            (0..=interval_count)
                .map(|index| scaled_lerp(low, high, index as f64 / interval_count as f64)),
        );

        if grid.iter().any(|coordinate| !coordinate.is_finite())
            || grid.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(Error::Data(format!(
                "density: {} grid not representable at this numeric scale",
                description()
            )));
        }
        let densities = crate::stats::distribution::gaussian_kde(&samples, &grid, bandwidth)
            .map_err(|error| match error {
                Error::Data(_) => Error::Data(format!("density: {}: {error}", description())),
                error => error,
            })?;

        maximum = densities.iter().copied().fold(maximum, f64::max);
        curves.push(DensityCurve {
            identity: crate::data::source::copy_source_pair(identity)?,
            grid,
            densities,
        });
    }
    if curves.is_empty() {
        return Err(Error::Data(
            "density: value column contains no observations".into(),
        ));
    }
    Ok(PreparedDensity {
        x_range: (support_low, support_high),
        maximum,
        curves,
    })
}

fn visible_curve(curve: &DensityCurve, x_range: (f64, f64)) -> Result<Vec<(f64, f64)>, Error> {
    let first = curve.grid[0];
    let last = curve.grid[curve.grid.len() - 1];
    if x_range.1 < first || x_range.0 > last {
        return Ok(Vec::new());
    }
    let low = x_range.0.max(first);
    let high = x_range.1.min(last);
    let mut visible = Vec::new();
    crate::data::source::reserve_source(&mut visible, curve.grid.len() + 2)?;

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
    Ok(visible)
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

fn clip_polygon_y(
    mut polygon: Vec<(f64, f64)>,
    y_range: (f64, f64),
) -> Result<Vec<(f64, f64)>, Error> {
    polygon = clip_polygon_boundary(polygon, y_range.0, true)?;
    clip_polygon_boundary(polygon, y_range.1, false)
}

fn clip_polygon_boundary(
    polygon: Vec<(f64, f64)>,
    boundary: f64,
    keep_above: bool,
) -> Result<Vec<(f64, f64)>, Error> {
    let Some(mut previous) = polygon.last().copied() else {
        return Ok(polygon);
    };
    let mut output = Vec::new();
    let count = polygon
        .len()
        .checked_mul(2)
        .ok_or_else(|| Error::Render("density clip storage overflow".into()))?;
    crate::data::source::reserve_source(&mut output, count)?;

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
    Ok(output)
}

fn project_points(
    points: &[(f64, f64)],
    layout: &Layout,

    polygon: bool,
) -> Result<Vec<Point2D>, Error> {
    let mut source_extents = [(f64::INFINITY, f64::NEG_INFINITY); 2];
    let mut projected_extents = source_extents;
    let points =
        crate::data::source::collect_source(points.iter().map(|&(source_x, source_y)| {
            let (x, y) = layout.projection.project(source_x, source_y);
            for (index, (source, projected)) in
                [(source_x, x), (source_y, y)].into_iter().enumerate()
            {
                source_extents[index].0 = source_extents[index].0.min(source);
                source_extents[index].1 = source_extents[index].1.max(source);
                projected_extents[index].0 = projected_extents[index].0.min(projected);
                projected_extents[index].1 = projected_extents[index].1.max(projected);
            }
            if x.is_finite() && y.is_finite() {
                Ok(Point2D::new(x, y))
            } else {
                Err(Error::Data(
                    "density: visible curve cannot be projected at this numeric scale".into(),
                ))
            }
        }))?;
    // A filled density needs both extents. A line only requires distinct points,
    // not a positive extent on each axis (horizontal/vertical lines are valid).
    if polygon || points.first() == points.last() {
        for (index, axis) in [
            crate::layout::AxisSide::Bottom,
            crate::layout::AxisSide::Left,
        ]
        .into_iter()
        .enumerate()
        {
            layout.projection.validate_span(
                axis,
                source_extents[index],
                projected_extents[index],
                source_extents[index].0 < source_extents[index].1,
                if polygon {
                    "density polygon"
                } else {
                    "density line"
                },
            )?;
        }
    }
    Ok(points)
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

    #[test]
    fn exact_density_color_map_uses_raw_hues_and_refuses_missing_keys_before_fonts() {
        let source = Plot::density(crate::NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]))
            .hue(crate::CategoryValues::new("hue", ["a", "a", "b", "b"]).unwrap())
            .categorical_color_scale(crate::CategoricalColorScale::from_map(
                crate::CategoricalColorMap::new([("a".into(), crate::OpaqueColor::rgb(5, 7, 11))])
                    .unwrap(),
            ))
            .erase();
        let resources = crate::font::canonical::OperationContext::new();
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let error =
            PreparedDensity::new(&source, source.data().with_operation(&resources)).unwrap_err();
        assert!(matches!(error, Error::Data(message) if message.contains("category \"b\"")));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
    use crate::{CategoryValues, NumericValues};

    #[test]
    fn density_group_curve_stroke_enters_automatic_hard_size_without_counting_kde_vertices() {
        let source = Plot::density(NumericValues::new("sample", [1.0, 2.0, 3.0, 4.0]))
            .x_limits(0.0, 5.0)
            .erase();
        let resources = crate::font::canonical::OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
        let chart = inputs.attach(source).unwrap();
        let mut theme = crate::Theme::publication().resolve().unwrap();
        let original = chart.intrinsic_body_minimum(&theme).unwrap();
        let line = theme
            .marks
            .data_lines()
            .density_line()
            .clone()
            .with_width(140.0)
            .unwrap();
        let lines = theme
            .marks
            .data_lines()
            .clone()
            .with_density_line(line)
            .unwrap();
        theme.marks = theme.marks.clone().with_data_lines(lines).unwrap();
        let thick = chart.intrinsic_body_minimum(&theme).unwrap();
        assert_eq!(thick.0.to_bits(), original.0.to_bits());
        assert_eq!(thick.1.to_bits(), 140.0_f64.to_bits());
        let line = theme
            .marks
            .data_lines()
            .density_line()
            .clone()
            .with_alpha(0)
            .unwrap();
        let lines = theme
            .marks
            .data_lines()
            .clone()
            .with_density_line(line)
            .unwrap();
        theme.marks = theme.marks.clone().with_data_lines(lines).unwrap();
        assert_eq!(chart.intrinsic_body_minimum(&theme).unwrap(), original);
        let area = theme
            .marks
            .fills()
            .densities()
            .density_area()
            .clone()
            .with_outline(Some(
                crate::theme::DataOutlineTheme::mapped(140.0, 255).unwrap(),
            ))
            .unwrap();
        let densities = theme
            .marks
            .fills()
            .densities()
            .clone()
            .with_density_area(area)
            .unwrap();
        let fills = theme
            .marks
            .fills()
            .clone()
            .with_densities(densities)
            .unwrap();
        theme.marks = theme.marks.clone().with_fills(fills).unwrap();
        let area = chart.intrinsic_body_minimum(&theme).unwrap();
        assert_eq!(area.0.to_bits(), original.0.to_bits());
        assert!(area.1 >= 140.0);
    }

    #[test]
    fn selected_curves_use_parent_collective_order_and_palette_without_rereading_columns() {
        let source = Plot::density(NumericValues::new(
            "sample",
            [1.0, 2.0, 3.0, 101.0, 102.0, 103.0],
        ))
        .hue(CategoryValues::new("series", ["a", "a", "a", "b", "b", "b"]).unwrap())
        .erase();
        let chart = source
            .with_global_hue_levels(vec!["b".into(), "a".into()])
            .with_global_collective_pairs(vec![
                crate::data::CollectivePair {
                    group: None,
                    hue: Some("b".into()),
                },
                crate::data::CollectivePair {
                    group: None,
                    hue: Some("a".into()),
                },
            ]);
        let mut theme = crate::Theme::publication().resolve().unwrap();
        let chart = crate::plots::shared_x::prepare_panel_for_test(&chart, &theme).unwrap();
        let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
        let selected = chart
            .measure_selected_interior(PlotRenderContext::new(
                chart.data(),
                crate::layout::PanelBox::new(10.0, 600.0, 20.0, 400.0),
                &theme,
            ))
            .unwrap();
        assert_eq!(
            crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
            reads
        );
        let crate::layout::interior::MarkPayload::Density(payload) =
            &selected.marks.as_ref().unwrap().payload
        else {
            panic!("Density payload")
        };
        assert_eq!(
            payload.curves[0].fill_style.color,
            crate::theme::Color::from(theme.color_scales.qualitative().colors()[0]).with_alpha(56)
        );
        assert_eq!(
            payload.curves[1].fill_style.color,
            crate::theme::Color::from(theme.color_scales.qualitative().colors()[1]).with_alpha(56)
        );
        assert!(payload.curves[0].fill[0].x > payload.curves[1].fill[0].x);
        let line = theme
            .marks
            .data_lines()
            .density_line()
            .clone()
            .with_width(2.5)
            .unwrap()
            .with_alpha(128)
            .unwrap();
        let lines = theme
            .marks
            .data_lines()
            .clone()
            .with_density_line(line)
            .unwrap();
        theme.marks = theme.marks.clone().with_data_lines(lines).unwrap();
        let chart = crate::plots::shared_x::prepare_panel_for_test(&chart, &theme).unwrap();
        let changed = chart
            .measure_selected_interior(PlotRenderContext::new(
                chart.data(),
                crate::layout::PanelBox::new(10.0, 600.0, 20.0, 400.0),
                &theme,
            ))
            .unwrap();
        let crate::layout::interior::MarkPayload::Density(changed_payload) =
            &changed.marks.as_ref().unwrap().payload
        else {
            panic!("changed Density payload")
        };
        assert_eq!(
            payload.curves[0].segments,
            changed_payload.curves[0].segments
        );
        assert_eq!(
            changed_payload.curves[0].line_style.width.to_bits(),
            2.5_f32.to_bits()
        );
        assert_eq!(changed_payload.curves[0].line_style.color.alpha(), 128);
        assert!(
            crate::decorations::legend::discrete::PreparedDiscreteGuide::merge(
                [
                    selected.discrete_guide.as_ref().unwrap(),
                    changed.discrete_guide.as_ref().unwrap()
                ],
                &[],
                "custom Density line",
            )
            .unwrap_err()
            .to_string()
            .contains("glyphs must match exactly")
        );
    }

    struct DensityValues {
        values: NumericValues,
        group: Option<CategoryValues>,
    }

    #[test]
    fn admitted_grouped_source_support_matches_the_authoritative_preparer() {
        let values = NumericValues::new("value", [0.0, 1.0, 2.0, 100.0, 110.0, 120.0]);
        let groups = CategoryValues::new("group", ["a", "a", "a", "b", "b", "b"]).unwrap();
        for collapsed in [false, true] {
            let mut plot = Plot::density(values.clone()).group(groups.clone());
            if collapsed {
                plot = plot.hue(groups.clone());
            } else {
                plot =
                    plot.hue(CategoryValues::new("color", ["u", "u", "u", "v", "v", "v"]).unwrap());
            }
            let plot = plot.erase();
            let resources = crate::font::canonical::OperationContext::new();
            let data = plot.data().with_operation(&resources);
            let range = shared_x_domain(&plot, data).unwrap();
            let prepared = prepare_density_for_chart(&plot, plot.data(), crate::data::X).unwrap();
            assert_eq!(range, prepared.x_range);
        }
    }

    fn density_values(values: &[f64], labels: Option<&[&str]>) -> DensityValues {
        DensityValues {
            values: NumericValues::new("value", values.iter().copied()),
            group: labels
                .map(|labels| CategoryValues::new("group", labels.iter().copied()).unwrap()),
        }
    }

    fn prepare_values(
        data: &DensityValues,
        grouped: bool,
        hued: bool,
    ) -> Result<PreparedDensity, Error> {
        let mut plot = Plot::density(data.values.clone());
        if grouped {
            plot = plot.group(data.group.as_ref().unwrap().clone());
        }
        if hued {
            plot = plot.hue(data.group.as_ref().unwrap().clone());
        }
        let plot = plot.erase();
        prepare_density(
            &plot,
            crate::data::X,
            grouped.then_some(crate::data::GROUP),
            grouped.then_some("group"),
            hued.then_some(crate::data::HUE),
            hued.then_some("group"),
        )
    }

    #[test]
    fn grouped_density_preserves_hue_order_and_shared_axis() {
        let data = density_values(
            &[0.0, 2.0, 1.0, 3.0, 4.0, 6.0],
            Some(&["b", "a", "b", "a", "b", "a"]),
        );

        let prepared = prepare_values(&data, true, true).unwrap();

        assert_eq!(prepared.curves[0].identity.hue.as_deref(), Some("b"));
        assert_eq!(prepared.curves[1].identity.hue.as_deref(), Some("a"));
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
        let data = density_values(
            &[0.0, 1.0, 2.0, 10.0, 11.0, 12.0],
            Some(&["a", "a", "a", "b", "b", "b"]),
        );
        let prepared = prepare_values(&data, true, true).unwrap();
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
        let data = density_values(&samples, None);
        let bandwidth = strict_scott_bandwidth(&samples).unwrap();

        let prepared = prepare_values(&data, false, false).unwrap();

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
        let data = density_values(
            &[0.0, 0.01, 0.02, 100.0, 110.0, 120.0],
            Some(&["sharp", "sharp", "sharp", "broad", "broad", "broad"]),
        );

        let prepared = prepare_values(&data, true, true).unwrap();
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
        let data = density_values(&[0.0, 1.0, 2.0], Some(&["small", "ok", "ok"]));

        let error = prepare_values(&data, true, true).unwrap_err();

        assert!(error
            .to_string()
            .contains("graphical group \"small\" requires at least 2"));
    }

    #[test]
    fn density_rejects_constant_group_with_label() {
        let data = density_values(&[1.0, 1.0, 2.0, 3.0], Some(&["flat", "flat", "ok", "ok"]));

        let error = prepare_values(&data, true, true).unwrap_err();

        assert!(error
            .to_string()
            .contains("graphical group \"flat\" has constant"));
    }

    #[test]
    fn density_rejects_large_offset_grid_when_coordinates_cannot_be_distinct() {
        let base = 1e300_f64;
        let data = density_values(&[base, base.next_up(), base.next_up().next_up()], None);

        let error = prepare_values(&data, false, false).unwrap_err();

        assert!(error
            .to_string()
            .contains("series grid not representable at this numeric scale"));
    }

    #[test]
    fn density_large_offset_grid_is_finite_and_strictly_increasing_when_representable() {
        let base = 1e300_f64;
        let data = density_values(&[base, base + 1e287, base + 2e287], None);

        let prepared = prepare_values(&data, false, false).unwrap();

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
        let data = density_values(&[-extreme, extreme], None);

        let prepared = prepare_values(&data, false, false).unwrap();
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
