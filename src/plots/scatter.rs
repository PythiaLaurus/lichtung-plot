//! `lichtung-plot::plots::scatter` — scatter plot implementation.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]

use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::render::backend::{with_clip, Backend, Point2D};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::Plot;

/// Render a scatter plot of the chart's `x_col` vs `y_col`.
///
/// Returns the `Layout` that was used to project the points so
/// the caller can apply decorations (frame, grid, axes) and the
/// collected `LegendEntry`s for the hue groups (empty when no
/// hue was set).
///
/// # Errors
/// `Error::Data` if either column is missing or
/// non-castable to f64.
pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let theme = context.theme;
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("scatter: x column not set".into()))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("scatter: y column not set".into()))?;
    let df = context.data;

    // Compute global x/y ranges from the *whole* dataset so the
    // Layout covers every hue group on a common axis scale.
    let points_all = crate::data::aligned_finite_f64(df, x_col, y_col, "scatter")?;
    if points_all.is_empty() {
        return Err(Error::Data("scatter: no rows".into()).into());
    }
    let (xmin, xmax) = crate::data::min_max(points_all.iter().map(|(x, _)| *x), "scatter")?;
    let (ymin, ymax) = crate::data::min_max(points_all.iter().map(|(_, y)| *y), "scatter")?;
    let mut hue_keys = if let Some(hue_col) = chart.hue_col() {
        crate::data::observed_levels_for_finite_pairs(df, x_col, y_col, hue_col, "scatter")?
    } else {
        Vec::new()
    };
    hue_keys.sort_by_key(|key| chart.hue_color_index(key, usize::MAX));
    let layout = context.axis_layout(|| {
        let x_range = crate::layout::continuous_auto_range((xmin, xmax), chart.x_axis_spec());
        let y_range = crate::layout::continuous_auto_range((ymin, ymax), chart.y_axis_spec());
        let x_axis_spec = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
        let y_axis_spec = crate::layout::display_axis_spec(chart.y_axis_spec(), y_range);

        // Single shared layout for the whole plot panel.
        context.layout_with_axis_specs(
            x_range,
            y_range,
            context.panel,
            chart.x_label_text().unwrap_or("").to_string(),
            chart.y_label_text().unwrap_or("").to_string(),
            &x_axis_spec,
            &y_axis_spec,
            theme,
        )
    })?;
    let proj = &layout.projection;

    let mut legend_entries = Vec::new();
    with_clip(backend, layout.projection.panel_box(), |backend| {
        if let Some(hue_col) = chart.hue_col() {
            let hues = crate::data::strict_strings(df, hue_col, "scatter")?;
            let aligned = crate::data::aligned_finite_rows(df, x_col, y_col, "scatter")?;
            for (i, label) in hue_keys.into_iter().enumerate() {
                let color = theme.colors[chart.hue_color_index(&label, i) % theme.colors.len()];
                let style = chart.point_style(color, 3);
                let pts: Vec<Point2D> = aligned
                    .iter()
                    .filter(|(row, _, _)| hues[*row] == label)
                    .map(|(_, x, y)| {
                        let (px, py) = proj.project(*x, *y);
                        Point2D::new(px, py)
                    })
                    .collect();
                backend.draw_scatter(&pts, &style)?;

                legend_entries.push(LegendEntry::point(chart.legend_identity(label), style));
            }
        } else {
            let color = theme.colors.first().copied().unwrap_or((0, 0, 0));
            let style = chart.point_style(color, 3);
            let pts: Vec<Point2D> = points_all
                .iter()
                .map(|(x, y)| {
                    let (px, py) = proj.project(*x, *y);
                    Point2D::new(px, py)
                })
                .collect();
            backend.draw_scatter(&pts, &style)?;
        }
        Ok(())
    })?;

    Ok(PlotRenderResult::new(layout, legend_entries))
}
