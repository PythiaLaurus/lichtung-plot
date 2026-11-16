//! Standalone Gaussian kernel density plots.

#![allow(clippy::cast_precision_loss)]

use crate::alpha::{
    marks::{fill_style, line_style},
    AlphaTarget, AlphaUnitKind, ResolvedAlpha,
};
use crate::data::PlotData;
use crate::Error;

use crate::decorations::legend::source::{FillRole, LineRole};
use crate::decorations::legend::GuideSourceKey;
use crate::layout::interior::MeasuredCoordinateMarks;
use crate::layout::Layout;
use crate::render::backend::{with_clip, Backend, FillStyle, LineStyle, Point2D};
use crate::spec::PlotMeasureContext;
use crate::stats::distribution::strict_scott_bandwidth;
use crate::theme::{AccentColors, CategoryColors};
#[cfg(test)]
#[cfg(test)]
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
    line_alpha: Option<ResolvedAlpha>,
    fill_alpha: Option<ResolvedAlpha>,
}

#[derive(Debug)]
pub(crate) struct PreparedDensity {
    x_range: (f64, f64),
    maximum: f64,
    curves: Vec<DensityCurve>,
}

/// Group curves, rather than their generated KDE vertices, are the semantic
/// primary marks. The resolved stroke is measured before parent assignment.
pub(crate) fn intrinsic_body_minimum(
    registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
    theme: &crate::text::TextRequestContext<'_, '_>,
) -> Result<(f64, f64), Error> {
    let prepared = registered
        .map(|registered| registered.family.as_ref())
        .and_then(|family| match family {
            crate::orchestrator::registration::FamilyRef::Density(marks) => Some(marks),
            _ => None,
        })
        .ok_or_else(|| Error::Render("Density intrinsic requires root-prepared curves".into()))?;
    if prepared.curves.is_empty() {
        return Err(Error::Render("Density has no retained group curves".into()));
    }
    let line = theme
        .marks
        .get_data_lines()
        .get_density_line()
        .backend_style((0, 0, 0));
    let mut ink = 0.0_f64;
    let mut area = (0.0_f64, 0.0_f64);
    for curve in &prepared.curves {
        ink = ink.max(crate::render::geometry::line_ink_width(&line_style(
            curve.line_alpha,
            line.clone(),
        ))?);
        let next = crate::render::geometry::rectangular_fill_ink_outset(&fill_style(
            curve.fill_alpha,
            theme
                .marks
                .get_fills()
                .get_densities()
                .get_density_area()
                .backend_style((0, 0, 0)),
        ))?;
        area = (area.0.max(next.0), area.1.max(next.1));
    }
    let raw_windows = registered.expect("prepared occurrence").raw_windows();
    Ok((
        if raw_windows[0].is_none() {
            ink.max(area.0)
        } else {
            0.0
        },
        if raw_windows[1].is_none() {
            ink.max(area.1)
        } else {
            0.0
        },
    ))
}

impl PreparedDensity {
    pub(crate) fn alpha_guide(
        &self,
        chart: &crate::builder::PlotDescription,
        theme: &crate::theme::ResolvedTheme,
        guide: &mut crate::decorations::legend::alpha::AlphaGuideBuilder<'_>,
    ) -> crate::Result<()> {
        use crate::decorations::legend::alpha::{AlphaGlyphRole, AlphaUnit};
        let fill = theme.marks.get_fills().get_densities().get_density_area();
        for curve in &self.curves {
            let unit = AlphaUnit::series(chart, Some(&curve.identity))?;
            guide.line(
                AlphaGlyphRole::Line,
                AlphaUnitKind::Series,
                unit.clone(),
                curve.line_alpha,
                theme
                    .marks
                    .get_data_lines()
                    .get_density_line()
                    .backend_style((0, 0, 0)),
                false,
            )?;
            guide.fill(
                AlphaGlyphRole::Fill,
                AlphaUnitKind::Series,
                unit,
                curve.fill_alpha,
                fill.backend_style((0, 0, 0)),
                fill.get_outline()
                    .is_some_and(|outline| outline.mapped_width().is_some()),
            )?;
        }
        Ok(())
    }

    fn resolved_colors(
        &self,
        binding: &crate::orchestrator::registration::CategoricalBinding,
        theme: &crate::theme::ResolvedTheme,
        resources: &crate::font::canonical::OperationContext,
    ) -> Result<(CategoryColors, AccentColors), Error> {
        let keys = if let Some(global) = binding.parent.as_deref() {
            crate::data::source::collect_source(
                global.values().map(crate::data::source::copy_source_string),
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
        let map = binding.resolve(keys.iter().map(String::as_str), theme, resources)?;
        let accent = AccentColors::resolve(
            theme.marks.get_accents(),
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

    pub(crate) fn group_order(
        &self,
        collectives: Option<&[crate::data::CollectivePair]>,
    ) -> Result<Vec<(usize, usize)>, Error> {
        let mut order = crate::data::source::collect_source(self.curves.iter().enumerate().map(
            |(index, curve)| {
                let pairs = collectives.unwrap_or(&[]);

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
        chart: &crate::builder::PlotDescription,
        binding: &crate::orchestrator::registration::CategoricalBinding,
        theme: &crate::theme::ResolvedTheme,
        operation: &crate::font::canonical::OperationContext,
        order: &[(usize, usize)],
    ) -> Result<Vec<GuideSourceKey>, Error> {
        let (colors, accent) = self.resolved_colors(binding, theme, operation)?;
        self.guide_entries_with_colors(chart, theme, order, &colors, &accent)
    }

    fn guide_entries_with_colors(
        &self,
        chart: &crate::builder::PlotDescription,
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
            entries.push(
                GuideSourceKey::collective(
                    chart.collective_legend_identity(&curve.identity)?,
                    chart,
                    false,
                )?
                .with_line(
                    theme
                        .marks
                        .get_data_lines()
                        .get_density_line()
                        .backend_style(color),
                    LineRole::DensityLine,
                )
                .with_fill(
                    theme.marks.get_fills().get_densities().get_density_area(),
                    color,
                    FillRole::DensityArea,
                ),
            );
        }
        Ok(entries)
    }

    pub(crate) fn new(
        chart: &crate::builder::PlotDescription,
        data: PlotData<'_, '_>,
        collectives: Option<&[crate::data::CollectivePair]>,
    ) -> Result<Self, Error> {
        data.validate_row_source()?;
        let column = chart
            .x_col()
            .ok_or_else(|| Error::Data("density: value column not set".into()))?;
        let mut prepared = prepare_density_for_chart(chart, data, column, collectives)?;
        let alpha = chart.prepare_collective_alpha(
            AlphaTarget::Line,
            AlphaUnitKind::Series,
            true,
            data.facet_selector()?,
        )?;
        let fill_alpha = chart.prepare_collective_alpha(
            AlphaTarget::Fill,
            AlphaUnitKind::Series,
            true,
            data.facet_selector()?,
        )?;
        for curve in &mut prepared.curves {
            curve.line_alpha = alpha.collective(chart, &curve.identity)?;
            curve.fill_alpha = fill_alpha.collective(chart, &curve.identity)?;
        }
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

    pub(crate) fn ranges(
        &self,
        chart: &crate::builder::PlotDescription,
    ) -> Result<[(f64, f64); 2], Error> {
        Ok([
            chart
                .x_axis_spec()
                .window
                .map_or(self.x_range, crate::layout::RawWindow::pair),
            crate::layout::anchored_auto_range(
                (0.0, self.maximum),
                chart
                    .y_axis_spec()
                    .window
                    .map(crate::layout::RawWindow::pair),
                0.0,
            )?,
        ])
    }
}

#[derive(Debug)]
pub(crate) struct DensityPayload {
    clip: crate::layout::PixelRect,
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
    context: &PlotMeasureContext<'_, '_>,
    request: &crate::text::TextRequestContext<'_, '_>,
    layout: &Layout,
) -> Result<MeasuredCoordinateMarks, crate::layout::fit::PlanError> {
    let prepared = (match context.family()? {
        crate::orchestrator::registration::FamilyRef::Density(marks) => Some(marks),
        _ => None,
    })
    .ok_or_else(|| {
        Error::Render(
            "Density measurement requires root-prepared semantics for its component".into(),
        )
    })?;
    let theme = request;
    theme.text.prepared_font_environment()?;
    let projection = layout.projection.clone();
    let mut result = project(prepared, layout, context, request)?;
    let raw_windows = context.registered.raw_windows();
    let automatic = [raw_windows[0].is_none(), raw_windows[1].is_none()];
    let mut requests = [None; 2];
    let crate::layout::interior::MarkPayload::Density(payload) = &result.marks.payload else {
        unreachable!("Density selected its own mark payload")
    };
    let mut area = crate::render::geometry::GeometrySnapshot::new();
    for curve in &payload.curves {
        if curve.fill.len() >= 3 {
            area.polygon(&curve.fill, &curve.fill_style, None)?;
        }
        for segment in &curve.segments {
            area.line(segment, &curve.line_style, None)?;
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
    let marks = std::sync::Arc::get_mut(&mut result.marks)
        .expect("newly selected Density marks have one owner");
    let crate::layout::interior::MarkPayload::Density(payload) = &mut marks.payload else {
        unreachable!("Density selected its own mark payload")
    };
    payload.domain_expansions = requests;
    Ok(result)
}

pub(crate) fn project(
    prepared: &PreparedDensity,
    layout: &Layout,
    context: &PlotMeasureContext<'_, '_>,
    request: &crate::text::TextRequestContext<'_, '_>,
) -> Result<MeasuredCoordinateMarks, Error> {
    let theme = request;
    let data = context.data()?;
    let mut payload = DensityPayload {
        clip: layout.projection.panel_box(),
        curves: Vec::new(),
        domain_expansions: [None; 2],
    };
    crate::data::source::reserve_source(&mut payload.curves, prepared.curves.len())?;
    let order = prepared.group_order(crate::orchestrator::registration::collective_pairs(Some(
        context.registered,
    )))?;
    let (colors, accent) = prepared.resolved_colors(
        context
            .registered
            .categorical
            .as_ref()
            .expect("prepared color binding"),
        theme,
        data.source_resources(),
    )?;

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
                project_points(&fill, layout, true)?
            } else {
                Vec::new()
            },
            segments: Vec::new(),
            fill_style: fill_style(
                curve.fill_alpha,
                theme
                    .marks
                    .get_fills()
                    .get_densities()
                    .get_density_area()
                    .backend_style(color),
            ),
            line_style: line_style(
                curve.line_alpha,
                theme
                    .marks
                    .get_data_lines()
                    .get_density_line()
                    .backend_style(color),
            ),
        };
        crate::data::source::reserve_source(&mut projected.segments, visible.len() - 1)?;
        for pair in visible.windows(2) {
            if let Some(segment) = clip_line_y(pair[0], pair[1], layout.projection.y_range()) {
                projected
                    .segments
                    .push(project_points(&segment, layout, false)?);
            }
        }
        payload.curves.push(projected);
    }

    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    let clip = Some(crate::render::backend::validate_clip_region(payload.clip)?.panel());
    for curve in &payload.curves {
        geometry.polygon(&curve.fill, &curve.fill_style, clip)?;
        for segment in &curve.segments {
            geometry.line(segment, &curve.line_style, clip)?;
        }
    }
    Ok(MeasuredCoordinateMarks::new(
        crate::data::source::source_arc(crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Density(payload),
            geometry,
        })?,
    ))
}

#[cfg(test)]
fn prepare_density(
    data: crate::data::PlotData<'_, '_>,
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

fn prepare_density_for_chart(
    chart: &crate::builder::PlotDescription,
    data: crate::data::PlotData<'_, '_>,
    value_col: &str,
    collectives: Option<&[crate::data::CollectivePair]>,
) -> Result<PreparedDensity, Error> {
    prepare_density_ordered(
        data,
        value_col,
        chart.group_col(),
        chart.group_name(),
        chart.hue_col(),
        chart.hue_name(),
        collectives,
    )
}

fn prepare_density_ordered(
    data: crate::data::PlotData<'_, '_>,
    value_col: &str,
    group_col: Option<&str>,
    group_name: Option<&str>,
    hue_col: Option<&str>,
    hue_name: Option<&str>,
    identity_order: Option<&[crate::data::CollectivePair]>,
) -> Result<PreparedDensity, Error> {
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
            line_alpha: None,
            fill_alpha: None,
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

    use crate::{CategoryValues, NumericValues};

    struct DensityValues {
        values: NumericValues,
        group: Option<CategoryValues>,
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
        crate::data::with_source_data_for_test(
            &plot,
            &crate::font::canonical::OperationContext::new(),
            crate::orchestrator::registration::RowSelection::Full,
            |data| {
                prepare_density(
                    data,
                    crate::data::X,
                    grouped.then_some(crate::data::GROUP),
                    grouped.then_some("group"),
                    hued.then_some(crate::data::HUE),
                    hued.then_some("group"),
                )
            },
        )?
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

    fn curve_integral(curve: &DensityCurve) -> f64 {
        curve
            .grid
            .windows(2)
            .zip(curve.densities.windows(2))
            .map(|(x, y)| (x[1] - x[0]) * (y[0] + y[1]) / 2.0)
            .sum()
    }
}
