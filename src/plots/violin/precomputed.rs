use crate::Error;

use crate::data::PlotData;
use crate::decorations::legend::LegendEntry;
use crate::layout::Layout;
use crate::plots::categorical_distribution::DistributionPayload;
use crate::render::backend::Point2D;
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::theme::ResolvedTheme;
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

impl PreparedRaincloud {
    pub(crate) fn hue_levels(&self) -> &[String] {
        &self.hues
    }

    pub(crate) fn guide_entries(
        &self,
        chart: &Plot,
        theme: &ResolvedTheme,
        data: PlotData<'_>,
    ) -> Result<Vec<LegendEntry>, Error> {
        data.collect_source(self.hues.iter().enumerate().map(|(index, hue)| {
            let color = chart.categorical_color(
                "raincloud",
                hue,
                chart.hue_color_index(hue, index),
                theme,
            )?;
            Ok(LegendEntry::point_fill(
                chart.point_legend_identity(hue, data)?,
                crate::plots::distribution_sample::overlay_style(color, theme),
                theme.marks.violin_body.backend_style(color),
            ))
        }))
    }
}

#[derive(Debug)]
struct PreparedRaincloudGroup {
    geometry: RaincloudGroup,
    local_color_index: usize,
}

#[cfg(test)]
thread_local! {
    static HUE_LOOKUP_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn prepare(chart: &Plot, data: PlotData<'_>) -> Result<PreparedRaincloud, Error> {
    let input = chart
        .raincloud_geometry_input()
        .ok_or_else(|| Error::Render("raincloud requires supplied geometry".into()))?;
    let range = input.validate(Some(data))?;
    let categories = data.collect_source(
        input
            .groups
            .iter()
            .map(|group| data.copy_source_string(&group.category)),
    )?;
    let mut hues = Vec::new();
    let groups = data.collect_source(input.groups.iter().enumerate().map(|(index, group)| {
        let local_color_index = if let Some(hue) = &group.hue {
            data.claim_source_visits(hues.len())?;
            #[cfg(test)]
            HUE_LOOKUP_VISITS.with(|count| count.set(count.get() + hues.len()));
            if let Some(index) = hues.iter().position(|level| level == hue) {
                index
            } else {
                let index = hues.len();
                data.reserve_source(&mut hues, 1)?;
                hues.push(data.copy_source_string(hue)?);
                index
            }
        } else {
            index
        };
        Ok(PreparedRaincloudGroup {
            local_color_index,
            geometry: RaincloudGroup {
                category: data.copy_source_string(&group.category)?,
                hue: group
                    .hue
                    .as_ref()
                    .map(|hue| data.copy_source_string(hue))
                    .transpose()?,
                density: data.collect_source(group.density.iter().copied().map(Ok))?,
                q1: group.q1,
                median: group.median,
                q3: group.q3,
                points: data.collect_source(group.points.iter().copied().map(Ok))?,
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
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = context.data.with_axis_resources(&resources);
    let legend_entries = input.guide_entries(chart, theme, data)?;

    let mut payload = DistributionPayload::new(layout.projection.panel_box(), chart.kind());
    for (index, group) in input.groups.iter().enumerate() {
        project_cloud_and_summary(
            chart,
            &layout,
            &group.geometry,
            group.local_color_index,
            index,
            theme,
            &mut payload,
            data,
        )?;
    }
    for (index, group) in input.groups.iter().enumerate() {
        project_strip(
            chart,
            &layout,
            &group.geometry,
            group.local_color_index,
            index,
            theme,
            &mut payload,
            data,
        )?;
    }
    payload
        .retain(PlotRenderResult::new(layout, legend_entries), context)
        .map_err(Into::into)
}

#[expect(
    clippy::too_many_arguments,
    reason = "borrowed selected projection and styles"
)]
fn project_cloud_and_summary(
    chart: &Plot,
    layout: &Layout,
    group: &RaincloudGroup,
    local_color_index: usize,
    index: usize,
    theme: &ResolvedTheme,
    payload: &mut DistributionPayload,
    data: PlotData<'_>,
) -> Result<(), Error> {
    let center = index as f64 + 0.5;
    let width = 0.8;
    let baseline = center - width * 0.10;
    let mut cloud = Vec::new();
    let mut left = baseline;
    let mut projected_left = f64::INFINITY;
    data.reserve_source(&mut cloud, group.density.len() + 2)?;
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
    payload.polygon(
        cloud,
        theme
            .marks
            .violin_body
            .backend_style(group_color(chart, group, local_color_index, theme)?),
        data,
    )?;

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
        data.collect_source(
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
        data,
    )?;
    payload.line(
        &[
            Point2D::new(box_left, median),
            Point2D::new(box_right, median),
        ],
        &theme.marks.raincloud_median.backend_style(),
        data,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "borrowed selected projection and styles"
)]
fn project_strip(
    chart: &Plot,
    layout: &Layout,
    group: &RaincloudGroup,
    local_color_index: usize,
    index: usize,
    theme: &ResolvedTheme,
    payload: &mut DistributionPayload,
    data: PlotData<'_>,
) -> Result<(), Error> {
    let center = index as f64 + 0.5;
    let width = 0.8;
    let (box_right, _) = layout.projection.project(center + width * 0.10, 0.0);
    let (strip_left, _) = layout.projection.project(center + width * 0.24, 0.0);
    let (strip_right, _) = layout.projection.project(center + width * 0.46, 0.0);
    let point_radius = 2.0;
    let strip_min = strip_left + point_radius;
    let strip_max = strip_right - point_radius;
    if strip_min - point_radius <= box_right || strip_max < strip_min {
        return Err(Error::Render(
            "raincloud: panel is too narrow to separate the complete observation strip from the IQR box; increase width"
                .into(),
        ));
    }
    let points = data.collect_source(group.points.iter().map(|point| {
        data.claim_source_visits(1)?;
        let x = strip_min + point.normalized_strip * (strip_max - strip_min);
        let (_, y) = layout.projection.project(center, point.y);
        Ok(Point2D::new(x, y))
    }))?;
    payload.scatter(
        points,
        crate::plots::distribution_sample::overlay_style(
            group_color(chart, group, local_color_index, theme)?,
            theme,
        ),
        data,
    )
}

fn group_color(
    chart: &Plot,
    group: &RaincloudGroup,
    local_color_index: usize,
    theme: &ResolvedTheme,
) -> Result<(u8, u8, u8), Error> {
    if let Some(hue) = &group.hue {
        let color_index = chart.hue_color_index(hue, local_color_index);
        chart.categorical_color("raincloud", hue, color_index, theme)
    } else {
        Ok(theme.colors[local_color_index % theme.colors.len()])
    }
}
