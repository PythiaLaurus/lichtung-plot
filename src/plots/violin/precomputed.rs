use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::layout::{AxisTick, Layout};
use crate::render::backend::{with_clip, Backend, FillStyle, LineStyle, Point2D};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::theme::ResolvedTheme;
use crate::{Plot, RaincloudGeometry, RaincloudGroup};

pub(super) fn render(
    chart: &Plot,
    input: &RaincloudGeometry,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let theme = context.theme;
    let data_y_range = input.validate(None)?;
    let hue_levels = input
        .groups
        .iter()
        .filter_map(|group| group.hue.as_ref())
        .fold(Vec::new(), |mut levels, hue| {
            if !levels.contains(hue) {
                levels.push(hue.clone());
            }
            levels
        });
    let panel = context.panel;
    let layout = context.axis_layout(|| {
        let display_range = crate::layout::continuous_auto_range(data_y_range, chart.y_axis_spec());
        let y_axis_spec = crate::layout::display_axis_spec(chart.y_axis_spec(), display_range);
        let x_ticks = input
            .groups
            .iter()
            .enumerate()
            .map(|(index, group)| AxisTick::new(index as f64 + 0.5, group.category.clone()))
            .collect();
        context.layout_with_ticks(
            (0.0, input.groups.len() as f64),
            display_range,
            panel,
            chart.x_label_text().unwrap_or("").to_string(),
            chart.y_label_text().unwrap_or("").to_string(),
            x_ticks,
            Vec::new(),
            chart.x_axis_spec(),
            &y_axis_spec,
            theme,
        )
    })?;
    let legend_entries = hue_levels
        .iter()
        .enumerate()
        .map(|(index, hue)| {
            let color = chart.categorical_color(
                "raincloud",
                hue,
                chart.hue_color_index(hue, index),
                theme,
            )?;
            Ok(LegendEntry::point_fill(
                chart.legend_identity(hue.clone()),
                crate::plots::distribution_sample::overlay_style(color),
                FillStyle {
                    color,
                    stroke: theme.foreground,
                    stroke_width: 1.0,
                    opacity: 0.70,
                },
            ))
        })
        .collect::<Result<Vec<_>, Error>>()?;

    with_clip(backend, layout.projection.panel_box(), |backend| {
        for (index, group) in input.groups.iter().enumerate() {
            draw_cloud_and_summary(chart, &layout, group, &hue_levels, index, theme, backend)?;
        }
        for (index, group) in input.groups.iter().enumerate() {
            draw_strip(chart, &layout, group, &hue_levels, index, theme, backend)?;
        }
        Ok(())
    })?;

    Ok(PlotRenderResult::new(layout, legend_entries))
}

fn draw_cloud_and_summary(
    chart: &Plot,
    layout: &Layout,
    group: &RaincloudGroup,
    hue_levels: &[String],
    index: usize,
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let center = index as f64 + 0.5;
    let width = 0.8;
    let baseline = center - width * 0.10;
    let mut cloud = Vec::with_capacity(group.density.len() + 2);
    for point in &group.density {
        let offset = point.normalized_width * width * 0.40;
        let (x, y) = layout.projection.project(baseline - offset, point.y);
        cloud.push(Point2D::new(x, y));
    }
    let support_low = group.density[0].y;
    let support_high = group.density[group.density.len() - 1].y;
    let (baseline_high_x, baseline_high_y) = layout.projection.project(baseline, support_high);
    let (baseline_low_x, baseline_low_y) = layout.projection.project(baseline, support_low);
    cloud.push(Point2D::new(baseline_high_x, baseline_high_y));
    cloud.push(Point2D::new(baseline_low_x, baseline_low_y));
    backend.draw_polygon(
        &cloud,
        &FillStyle {
            color: group_color(chart, group, hue_levels, index, theme)?,
            stroke: theme.foreground,
            stroke_width: 1.0,
            opacity: 0.70,
        },
    )?;

    let box_center = center + width * 0.02;
    let box_half_width = width * 0.08;
    let (box_left, _) = layout.projection.project(box_center - box_half_width, 0.0);
    let (box_right, _) = layout.projection.project(box_center + box_half_width, 0.0);
    let (_, q1) = layout.projection.project(box_center, group.q1);
    let (_, q3) = layout.projection.project(box_center, group.q3);
    let (_, median) = layout.projection.project(box_center, group.median);
    backend.draw_polygon(
        &[
            Point2D::new(box_left, q1),
            Point2D::new(box_right, q1),
            Point2D::new(box_right, q3),
            Point2D::new(box_left, q3),
        ],
        &FillStyle {
            color: (255, 255, 255),
            stroke: theme.foreground,
            stroke_width: 1.0,
            opacity: 1.0,
        },
    )?;
    backend.draw_line(
        &[
            Point2D::new(box_left, median),
            Point2D::new(box_right, median),
        ],
        &LineStyle::solid(theme.foreground, 1.5),
    )
}

fn draw_strip(
    chart: &Plot,
    layout: &Layout,
    group: &RaincloudGroup,
    hue_levels: &[String],
    index: usize,
    theme: &ResolvedTheme,
    backend: &mut dyn Backend,
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
    let points = group
        .points
        .iter()
        .map(|point| {
            let x = strip_min + point.normalized_strip * (strip_max - strip_min);
            let (_, y) = layout.projection.project(center, point.y);
            Point2D::new(x, y)
        })
        .collect::<Vec<_>>();
    backend.draw_scatter(
        &points,
        &crate::plots::distribution_sample::overlay_style(group_color(
            chart, group, hue_levels, index, theme,
        )?),
    )
}

fn group_color(
    chart: &Plot,
    group: &RaincloudGroup,
    hue_levels: &[String],
    index: usize,
    theme: &ResolvedTheme,
) -> Result<(u8, u8, u8), Error> {
    if let Some(hue) = &group.hue {
        let local_index = hue_levels
            .iter()
            .position(|level| level == hue)
            .unwrap_or(index);
        let color_index = chart.hue_color_index(hue, local_index);
        chart.categorical_color("raincloud", hue, color_index, theme)
    } else {
        Ok(theme.colors[index % theme.colors.len()])
    }
}
