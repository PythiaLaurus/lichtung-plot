use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::layout::Layout;
use crate::plots::categorical_distribution::DistributionPayload;
use crate::render::backend::Point2D;
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::theme::{CategoryColors, ResolvedTheme};
use crate::{Plot, RaincloudGroup};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub(crate) struct PreparedRaincloud {
    pub(crate) range: (f64, f64),
    pub(crate) categories: Vec<String>,
    groups: Vec<PreparedRaincloudGroup>,
    hues: Vec<String>,
}

pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let crate::plots::categorical_distribution::PreparedDistribution::Supplied(prepared) =
        crate::plots::categorical_distribution::retained(chart)?
    else {
        return Err(Error::Render(
            "supplied Raincloud intrinsic requires retained groups".into(),
        ));
    };
    if prepared.groups.is_empty() || prepared.groups.len() != prepared.categories.len() {
        return Err(Error::Render(
            "supplied Raincloud groups and category slots differ".into(),
        ));
    }

    let width = 0.8_f64;
    let cloud_style = theme.marks.violin_body.backend_style((0, 0, 0));
    let cloud = crate::render::geometry::rectangular_fill_ink_bounds(&cloud_style)?;
    let (_, cloud_numeric) = crate::render::geometry::rectangular_fill_ink_outset(&cloud_style)?;
    let box_style = theme.marks.raincloud_box.backend_style();
    let box_ink = crate::render::geometry::rectangular_fill_ink_bounds(&box_style)?;
    let (_, box_numeric) = crate::render::geometry::rectangular_fill_ink_outset(&box_style)?;
    let median_ink =
        crate::render::geometry::line_ink_bounds(&theme.marks.raincloud_median.backend_style())?;
    let median = median_ink.map_or(0.0, |ink| ink.top - ink.bottom);
    let point = theme.marks.raw_overlay_point.backend_style(
        (0, 0, 0),
        theme.marks.raw_overlay_radius,
        true,
    );
    let (_, point_numeric) = crate::render::geometry::point_ink_dimensions(&point)?;
    let mut pitch = crate::plots::violin::raincloud_strip_pitch(theme, width)?
        .max((1.0 / 1024.0) / (0.4 * width))
        .max((1.0 / 1024.0) / (0.16 * width));
    if let Some(ink) = cloud {
        pitch = pitch.max((-ink.left).max(0.0) / (0.5 - 0.5 * width));
        pitch = pitch.max(ink.right.max(0.0) / (0.5 + 0.1 * width));
    }
    if let Some(ink) = box_ink {
        pitch = pitch.max((-ink.left).max(0.0) / (0.5 - 0.06 * width));
        pitch = pitch.max(ink.right.max(0.0) / (0.5 - 0.10 * width));
    }
    if let Some(ink) = median_ink {
        pitch = pitch.max((-ink.left).max(0.0) / (0.5 - 0.06 * width));
        pitch = pitch.max(ink.right.max(0.0) / (0.5 - 0.10 * width));
    }
    let category_count = u32::try_from(prepared.categories.len()).map_err(|_| {
        Error::Render("supplied Raincloud category count exceeds spatial range".into())
    })?;
    let category_span = pitch * f64::from(category_count);
    if !category_span.is_finite() {
        return Err(Error::Render(
            "supplied Raincloud hard pitch overflowed".into(),
        ));
    }
    let numeric = cloud_numeric
        .max(box_numeric)
        .max(median)
        .max(point_numeric);
    Ok((
        category_span,
        if chart.user_y_limits().is_none() {
            numeric
        } else {
            0.0
        },
    ))
}

impl PreparedRaincloud {
    pub(crate) fn primary_mark_count(&self) -> u128 {
        self.groups.len() as u128
    }

    pub(crate) fn hue_levels(&self) -> &[String] {
        &self.hues
    }

    pub(crate) fn guide_entries(
        &self,
        chart: &Plot,
        theme: &ResolvedTheme,

        colors: &CategoryColors,
    ) -> Result<Vec<LegendEntry>, Error> {
        crate::data::source::collect_source(self.hues.iter().map(|hue| {
            let color = colors.color(hue)?;
            Ok(LegendEntry::point_fill(
                chart.point_legend_identity(hue)?,
                crate::plots::distribution_sample::overlay_style(color, theme),
                theme.marks.violin_body.backend_style(color),
            ))
        }))
    }
}

#[derive(Debug)]
struct PreparedRaincloudGroup {
    geometry: RaincloudGroup,
}

#[cfg(test)]
thread_local! {
    static HUE_LOOKUP_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn prepare(chart: &Plot) -> Result<PreparedRaincloud, Error> {
    let input = chart
        .raincloud_geometry_input()
        .ok_or_else(|| Error::Render("raincloud requires supplied geometry".into()))?;
    let range = input.validate()?;
    let categories = crate::data::source::collect_source(
        input
            .groups
            .iter()
            .map(|group| crate::data::source::copy_source_string(&group.category)),
    )?;
    let mut hues = Vec::new();
    let groups = crate::data::source::collect_source(input.groups.iter().map(|group| {
        if let Some(hue) = &group.hue {
            #[cfg(test)]
            HUE_LOOKUP_VISITS.with(|count| count.set(count.get() + hues.len()));
            if !hues.iter().any(|level| level == hue) {
                crate::data::source::reserve_source(&mut hues, 1)?;
                hues.push(crate::data::source::copy_source_string(hue)?);
            }
        }
        Ok(PreparedRaincloudGroup {
            geometry: RaincloudGroup {
                category: crate::data::source::copy_source_string(&group.category)?,
                hue: group
                    .hue
                    .as_ref()
                    .map(|hue| crate::data::source::copy_source_string(hue))
                    .transpose()?,
                density: crate::data::source::collect_source(
                    group.density.iter().copied().map(Ok),
                )?,
                q1: group.q1,
                median: group.median,
                q3: group.q3,
                points: crate::data::source::collect_source(group.points.iter().copied().map(Ok))?,
            },
        })
    }))?;
    Ok(PreparedRaincloud {
        range,
        categories,
        groups,
        hues,
    })
}

pub(super) fn measure_marks(
    chart: &Plot,
    input: &PreparedRaincloud,
    context: PlotRenderContext<'_>,
    layout: Layout,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let theme = context.theme;
    let resources = theme.prepared_font_environment()?.operation();
    let data = context.data.with_operation(&resources);
    let colors = crate::plots::categorical_distribution::retained(chart)?
        .resolved_colors(chart, theme, data)?;
    let legend_entries = input.guide_entries(chart, theme, &colors)?;

    let mut payload = DistributionPayload::new(layout.projection.panel_box(), chart.kind());
    for (index, group) in input.groups.iter().enumerate() {
        let color = colors.color(
            group
                .geometry
                .hue
                .as_deref()
                .unwrap_or(&group.geometry.category),
        )?;
        project_cloud_and_summary(&layout, &group.geometry, color, index, theme, &mut payload)?;
    }
    for (index, group) in input.groups.iter().enumerate() {
        let color = colors.color(
            group
                .geometry
                .hue
                .as_deref()
                .unwrap_or(&group.geometry.category),
        )?;
        project_strip(
            &layout,
            &group.geometry,
            color,
            index,
            theme,
            &mut payload,
            context.fit_scope(),
        )?;
    }
    payload.retain(
        PlotRenderResult::new(layout, legend_entries),
        context,
        chart,
    )
}

fn project_cloud_and_summary(
    layout: &Layout,
    group: &RaincloudGroup,
    color: (u8, u8, u8),
    index: usize,
    theme: &ResolvedTheme,
    payload: &mut DistributionPayload,
) -> Result<(), Error> {
    let center = index as f64 + 0.5;
    let width = 0.8;
    let baseline = center - width * 0.10;
    let mut cloud = Vec::new();
    let mut left = baseline;
    let mut projected_left = f64::INFINITY;
    crate::data::source::reserve_source(&mut cloud, group.density.len() + 2)?;
    for point in &group.density {
        let offset = point.normalized_width * width * 0.40;
        let (x, y) = layout.projection.project(baseline - offset, point.y);
        left = left.min(baseline - offset);
        projected_left = projected_left.min(x);
        cloud.push(Point2D::new(x, y));
    }
    let support_low = group.density[0].y;
    let support_high = group.density[group.density.len() - 1].y;
    let (baseline_high_x, baseline_high_y) = layout.projection.project(baseline, support_high);
    let (baseline_low_x, baseline_low_y) = layout.projection.project(baseline, support_low);
    layout.projection.validate_span(
        crate::layout::AxisSide::Bottom,
        (left, baseline),
        (projected_left, baseline_low_x),
        true,
        "raincloud polygon width",
    )?;
    layout.projection.validate_span(
        crate::layout::AxisSide::Left,
        (support_low, support_high),
        (baseline_low_y, baseline_high_y),
        support_low < support_high,
        "raincloud polygon support",
    )?;
    cloud.push(Point2D::new(baseline_high_x, baseline_high_y));
    cloud.push(Point2D::new(baseline_low_x, baseline_low_y));
    payload.polygon(cloud, theme.marks.violin_body.backend_style(color))?;

    let box_center = center + width * 0.02;
    let box_half_width = width * 0.08;
    let (box_left, _) = layout.projection.project(box_center - box_half_width, 0.0);
    let (box_right, _) = layout.projection.project(box_center + box_half_width, 0.0);
    let (_, q1) = layout.projection.project(box_center, group.q1);
    let (_, q3) = layout.projection.project(box_center, group.q3);
    let (_, median) = layout.projection.project(box_center, group.median);
    layout.projection.validate_span(
        crate::layout::AxisSide::Left,
        (group.q1, group.q3),
        (q1, q3),
        group.q1 < group.q3,
        "raincloud box polygon",
    )?;
    payload.polygon(
        crate::data::source::collect_source(
            [
                Point2D::new(box_left, q1),
                Point2D::new(box_right, q1),
                Point2D::new(box_right, q3),
                Point2D::new(box_left, q3),
            ]
            .into_iter()
            .map(Ok),
        )?,
        theme.marks.raincloud_box.backend_style(),
    )?;
    payload.line(
        &[
            Point2D::new(box_left, median),
            Point2D::new(box_right, median),
        ],
        &theme.marks.raincloud_median.backend_style(),
    )
}

fn project_strip(
    layout: &Layout,
    group: &RaincloudGroup,
    color: (u8, u8, u8),
    index: usize,
    theme: &ResolvedTheme,
    payload: &mut DistributionPayload,

    scope: &crate::layout::fit::FitScope,
) -> Result<(), crate::layout::fit::PlanError> {
    let center = index as f64 + 0.5;
    let width = 0.8;
    let (box_right, _) = layout.projection.project(center + width * 0.10, 0.0);
    let (strip_left, _) = layout.projection.project(center + width * 0.24, 0.0);
    let (strip_right, _) = layout.projection.project(center + width * 0.46, 0.0);
    let style = crate::plots::distribution_sample::overlay_style(color, theme);
    let point_ink = crate::render::geometry::point_ink_bounds(&style)?;
    let point_left = point_ink.map_or(0.0, |ink| (-ink.left).max(0.0));
    let point_right = point_ink.map_or(0.0, |ink| ink.right.max(0.0));
    let box_right_ink = crate::render::geometry::rectangular_fill_ink_bounds(
        &theme.marks.raincloud_box.backend_style(),
    )?
    .map_or(0.0, |ink| ink.right.max(0.0));
    let strip_min = strip_left + point_left;
    let strip_max = strip_right - point_right;
    if strip_min - point_left <= box_right + box_right_ink || strip_max < strip_min {
        return Err(scope.child(crate::layout::AddressRole::CategoricalSeriesSlot, index)?.failure(
            crate::layout::fit::FitKind::SpecializedGeometry,
            "raincloud: panel is too narrow to separate the complete observation strip from the IQR box; increase width",
        ).into());
    }
    let points = crate::data::source::collect_source(group.points.iter().map(|point| {
        let x = strip_min + point.normalized_strip * (strip_max - strip_min);
        let (_, y) = layout.projection.project(center, point.y);
        Ok(Point2D::new(x, y))
    }))?;
    Ok(payload.scatter(points, style)?)
}
