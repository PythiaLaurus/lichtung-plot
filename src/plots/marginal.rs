//! `lichtung-plot::plots::marginal` — marginal plot.
//!
//! Visual composite: a central scatter of `(x, y)` with
//! marginal KDE curves on the top (x) and right (y).
//! This is the standard "marginal plot" idiom in
//! seaborn (`sns.jointplot`) and ggplot2 (`ggmarginal`).
//!
//! Supports explicit graphical grouping with independent hue color and legend identity.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::redundant_closure_for_method_calls,
    clippy::unnecessary_cast
)]

use crate::decorations::legend::LegendEntry;
use crate::layout::{self, AxisSpec, Layout, PanelBox};
use crate::render::backend::{with_clip, Backend, FillStyle, Point2D};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::stats::distribution::{
    gaussian_kde_with_bandwidth, kde, kde_support_range, strict_scott_bandwidth,
};
use crate::Error;
use crate::Plot;

const MARGINAL_KDE_CUT: f64 = 3.0;
const MARGINAL_KDE_GRID_POINTS: usize = 256;
const MARGINAL_KDE_VISIBLE_FRACTION: f64 = 0.02;
const MARGINAL_DATA_MARGIN_FRACTION: f64 = 0.06;

/// One graphical group's data and optional color identity.
struct HueGroup {
    identity: crate::data::CollectivePair,
    description: String,
    x: Vec<f64>,
    y: Vec<f64>,
    x_bandwidth: f64,
    y_bandwidth: f64,
    color: (u8, u8, u8),
}

/// Render a marginal plot: central scatter with
/// top-and-right marginal KDE curves.
///
/// Returns the `Layout` covering the **central** scatter panel
/// so the orchestrator can apply decorations (frame / grid /
/// axes / legend) to it.
///
/// Explicit graphical groups are rendered separately; hue supplies their color
/// and legend entries.
#[allow(clippy::if_not_else)]
pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    let palette = chart.palette();

    // --- Extract data (grouped or ungrouped) ---
    let (groups, _hue_levels) = build_groups(chart, context.data)?;

    // Collect all x/y for range computation.
    let all_x: Vec<f64> = groups.iter().flat_map(|g| g.x.iter().copied()).collect();
    let all_y: Vec<f64> = groups.iter().flat_map(|g| g.y.iter().copied()).collect();

    if all_x.is_empty() {
        return Err(Error::Data("marginal: no rows".into()));
    }

    // --- Layout geometry ---
    let outer = context.panel;
    let gap = 12.0;
    let top_h = 82.0;
    let right_w = 96.0;
    let central_x0 = outer.x0;
    let central_y0 = outer.y0;
    let central_x1 = (outer.x1 - right_w - gap).max(central_x0 + 1.0);
    let central_y1 = (outer.y1 - top_h - gap).max(central_y0 + 1.0);
    let top_y0 = central_y1 + gap;
    let top_y1 = (top_y0 + top_h).min(outer.y1);
    let right_x0 = central_x1 + gap;
    let right_x1 = (right_x0 + right_w).min(outer.x1);

    // --- Data ranges ---
    let (x_min, x_max) = all_x
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });
    let (y_min, y_max) = all_y
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });
    let (x_low, x_high) = marginal_display_range(
        groups.iter().map(|group| group.x.as_slice()),
        (x_min, x_max),
        chart.x_axis_spec(),
    );
    let y_display_range = marginal_display_range(
        groups.iter().map(|group| group.y.as_slice()),
        (y_min, y_max),
        chart.y_axis_spec(),
    );
    validate_marginal_range(&groups, (x_low, x_high), |group| &group.x, "x")?;
    validate_marginal_range(&groups, y_display_range, |group| &group.y, "y")?;
    let (y_low, y_high) = y_display_range;

    // --- Build Layout for the central scatter ---
    let projection = crate::layout::Projection::new(
        (x_low, x_high),
        (y_low, y_high),
        (central_x0, central_x1, central_y0, central_y1),
    );
    let x_ticks = chart.explicit_x_ticks().map_or_else(
        || {
            layout::automatic_numeric_ticks(
                (x_low, x_high),
                &projection,
                crate::layout::AxisSide::Bottom,
                palette.tick_font_size,
            )
        },
        <[crate::layout::AxisTick]>::to_vec,
    );
    let y_ticks = chart.explicit_y_ticks().map_or_else(
        || {
            layout::automatic_numeric_ticks(
                (y_low, y_high),
                &projection,
                crate::layout::AxisSide::Left,
                palette.tick_font_size,
            )
        },
        <[crate::layout::AxisTick]>::to_vec,
    );
    let layout = Layout {
        projection,
        x_axis: crate::layout::Axis {
            label: chart.x_label_text().unwrap_or("").to_string(),
            ticks: x_ticks,
        },
        y_axis: crate::layout::Axis {
            label: chart.y_label_text().unwrap_or("").to_string(),
            ticks: y_ticks,
        },
    };

    let proj = &layout.projection;

    // --- Tick marks on marginal panels (no labels, matching seaborn) ---
    let tick_len = f64::from(palette.tick_length);
    let tick_style =
        crate::render::backend::LineStyle::solid(palette.foreground, palette.axis_width);

    // Top KDE panel: x-axis ticks along the bottom edge (y = top_y0).
    // Positions match the central scatter's x-axis ticks.
    for tick in &layout.x_axis.ticks {
        let (x_px, _) = proj.project(tick.position, y_low);
        if x_px >= central_x0 && x_px <= central_x1 {
            backend.draw_line(
                &[
                    Point2D::new(x_px, top_y0),
                    Point2D::new(x_px, top_y0 - tick_len),
                ],
                &tick_style,
            )?;
        }
    }

    // Right KDE panel: y-axis ticks along the left edge (x = right_x0).
    // Positions match the central scatter's y-axis ticks.
    for tick in &layout.y_axis.ticks {
        let (_, y_px) = proj.project(x_low, tick.position);
        if y_px >= central_y0 && y_px <= central_y1 {
            backend.draw_line(
                &[
                    Point2D::new(right_x0, y_px),
                    Point2D::new(right_x0 - tick_len, y_px),
                ],
                &tick_style,
            )?;
        }
    }

    // --- KDE grid (shared across groups) ---
    let n_grid = 80;
    let x_grid: Vec<f64> = (0..n_grid)
        .map(|i| x_low + (x_high - x_low) * (i as f64 / (n_grid - 1) as f64))
        .collect();
    let y_grid: Vec<f64> = (0..n_grid)
        .map(|i| y_low + (y_high - y_low) * (i as f64 / (n_grid - 1) as f64))
        .collect();

    // Compute per-group KDE densities to find the shared max.
    let mut x_densities: Vec<Vec<f64>> = groups
        .iter()
        .map(|group| {
            gaussian_kde_with_bandwidth(&group.x, &x_grid, group.x_bandwidth).map_err(|error| {
                Error::Data(format!(
                    "marginal: x distribution for {}: {error}",
                    group.description
                ))
            })
        })
        .collect::<Result<_, _>>()?;
    let mut y_densities: Vec<Vec<f64>> = groups
        .iter()
        .map(|group| {
            gaussian_kde_with_bandwidth(&group.y, &y_grid, group.y_bandwidth).map_err(|error| {
                Error::Data(format!(
                    "marginal: y distribution for {}: {error}",
                    group.description
                ))
            })
        })
        .collect::<Result<_, _>>()?;

    for density in x_densities.iter_mut().chain(y_densities.iter_mut()) {
        if let Some(first) = density.first_mut() {
            *first = 0.0;
        }
        if let Some(last) = density.last_mut() {
            *last = 0.0;
        }
    }

    let x_dens_max = x_densities
        .iter()
        .flat_map(|d| d.iter())
        .fold(0.0_f64, |a, &v| a.max(v))
        .max(1e-12);
    let y_dens_max = y_densities
        .iter()
        .flat_map(|d| d.iter())
        .fold(0.0_f64, |a, &v| a.max(v))
        .max(1e-12);

    let usable_h = (top_y1 - top_y0) * 0.90;
    let usable_w = (right_x1 - right_x0) * 0.90;
    let central_panel = PanelBox::new(central_x0, central_x1, central_y0, central_y1);
    let top_panel = PanelBox::new(central_x0, central_x1, top_y0, top_y1);
    let right_panel = PanelBox::new(right_x0, right_x1, central_y0, central_y1);

    // --- Render each group ---
    let mut legend_entries = Vec::new();

    for (gi, group) in groups.iter().enumerate() {
        let color = group.color;

        // Scatter points in central panel.
        let point_style = chart.point_style(color, 3);
        let pts: Vec<Point2D> = group
            .x
            .iter()
            .zip(group.y.iter())
            .map(|(xi, yi)| {
                let (px, py) = proj.project(*xi, *yi);
                Point2D::new(px, py)
            })
            .collect();
        with_clip(backend, central_panel, |backend| {
            backend.draw_scatter(&pts, &point_style)
        })?;

        // Top marginal KDE (filled polygon).
        let x_density = &x_densities[gi];
        let mut top_poly = Vec::with_capacity(n_grid + 2);
        top_poly.push(Point2D::new(central_x0, top_y0));
        for (i, &d) in x_density.iter().enumerate() {
            let px = central_x0 + (i as f64 / (n_grid - 1) as f64) * (central_x1 - central_x0);
            let py = top_y0 + (d / x_dens_max) * usable_h;
            top_poly.push(Point2D::new(px, py));
        }
        top_poly.push(Point2D::new(central_x1, top_y0));
        let fill_top = FillStyle {
            color,
            stroke: color,
            stroke_width: 1.0,
            opacity: 0.45,
        };
        with_clip(backend, top_panel, |backend| {
            backend.draw_polygon(&top_poly, &fill_top)
        })?;

        // Right marginal KDE (filled polygon, rotated 90°).
        let y_density = &y_densities[gi];
        let mut right_poly = Vec::with_capacity(n_grid + 2);
        right_poly.push(Point2D::new(right_x0, central_y0));
        for (i, &d) in y_density.iter().enumerate() {
            let gy = y_grid[i];
            let py = proj.project(x_low, gy).1;
            let px = right_x0 + (d / y_dens_max) * usable_w;
            right_poly.push(Point2D::new(px, py));
        }
        right_poly.push(Point2D::new(right_x0, central_y1));
        let fill_right = FillStyle {
            color,
            stroke: color,
            stroke_width: 1.0,
            opacity: 0.45,
        };
        with_clip(backend, right_panel, |backend| {
            backend.draw_polygon(&right_poly, &fill_right)
        })?;

        if group.identity.hue.is_some() {
            legend_entries.push(LegendEntry::point_fill(
                chart.collective_legend_identity(&group.identity)?,
                point_style,
                fill_top,
            ));
        }
    }

    Ok(PlotRenderResult::new(layout, legend_entries))
}

fn validate_marginal_range<'a>(
    groups: &'a [HueGroup],
    range: (f64, f64),
    values: impl Fn(&'a HueGroup) -> &'a [f64],
    channel: &str,
) -> Result<(), Error> {
    if range.0.is_finite() && range.1.is_finite() && range.0 < range.1 {
        return Ok(());
    }
    let group = groups
        .iter()
        .find(|group| {
            let samples = values(group);
            crate::data::min_max(samples.iter().copied(), "marginal")
                .is_ok_and(|(minimum, maximum)| !(maximum - minimum).is_finite())
                || kde_support_range(std::iter::once(samples), MARGINAL_KDE_CUT).is_none()
        })
        .map_or("ungrouped series", |group| group.description.as_str());
    Err(Error::Data(format!(
        "marginal: {channel} distribution for {group}: support/grid is not representable at this numeric scale"
    )))
}

/// Build graphical groups while keeping hue as color-only metadata.
fn build_groups<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
) -> Result<(Vec<HueGroup>, Vec<String>), Error> {
    let data = data.into();
    let palette = chart.palette();

    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("marginal: x column not set".into()))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("marginal: y column not set".into()))?;
    let rows = crate::data::aligned_finite_rows(data, x_col, y_col, "marginal")?;
    let retained_rows = rows.iter().map(|(row, _, _)| *row).collect::<Vec<_>>();
    let channels = crate::data::observed_collectives(
        data,
        chart.group_col(),
        chart.group_name(),
        chart.hue_col(),
        chart.hue_name(),
        Some(&retained_rows),
        "marginal",
    )?;
    let hue_levels = chart.resolved_hue_levels(channels.hue_levels.clone());
    let mut groups = Vec::new();
    for identity in &channels.identities {
        let selected = rows
            .iter()
            .zip(&channels.rows)
            .filter(|(_, row_identity)| *row_identity == identity);
        let mut x = Vec::new();
        let mut y = Vec::new();
        for ((_, x_value, y_value), _) in selected {
            x.push(*x_value);
            y.push(*y_value);
        }
        if x.is_empty() {
            continue;
        }
        let description = identity.description(channels.collapsed);
        let x_bandwidth = strict_scott_bandwidth(&x).map_err(|reason| {
            Error::Data(format!(
                "marginal: x distribution for {description} {reason}"
            ))
        })?;
        let y_bandwidth = strict_scott_bandwidth(&y).map_err(|reason| {
            Error::Data(format!(
                "marginal: y distribution for {description} {reason}"
            ))
        })?;
        let color_index = identity
            .hue
            .as_deref()
            .and_then(|label| hue_levels.iter().position(|level| level == label))
            .unwrap_or(0);
        groups.push(HueGroup {
            identity: identity.clone(),
            description,
            x,
            y,
            x_bandwidth,
            y_bandwidth,
            color: palette.colors[color_index % palette.colors.len()],
        });
    }
    Ok((groups, hue_levels))
}

fn marginal_display_range<'a>(
    groups: impl IntoIterator<Item = &'a [f64]>,
    data_range: (f64, f64),
    spec: &AxisSpec,
) -> (f64, f64) {
    let data_display_range = marginal_data_display_range(data_range, spec);
    if spec.limits.is_some() {
        return data_display_range;
    }

    let Some(visible_kde_range) =
        kde_visible_support_range(groups, MARGINAL_KDE_CUT, MARGINAL_KDE_VISIBLE_FRACTION)
    else {
        return data_display_range;
    };

    (
        data_display_range.0.min(visible_kde_range.0),
        data_display_range.1.max(visible_kde_range.1),
    )
}

fn marginal_data_display_range(data_range: (f64, f64), spec: &AxisSpec) -> (f64, f64) {
    if spec.limits.is_some() {
        return layout::continuous_auto_range(data_range, spec);
    }
    let lo = data_range.0.min(data_range.1);
    let hi = data_range.0.max(data_range.1);
    if !(lo.is_finite() && hi.is_finite()) {
        return data_range;
    }
    if (hi - lo).abs() < f64::EPSILON {
        return (lo - 1.0, hi + 1.0);
    }
    let pad = (hi - lo) * MARGINAL_DATA_MARGIN_FRACTION;
    (lo - pad, hi + pad)
}

fn kde_visible_support_range<'a>(
    groups: impl IntoIterator<Item = &'a [f64]>,
    cut: f64,
    visible_fraction: f64,
) -> Option<(f64, f64)> {
    let groups: Vec<&[f64]> = groups.into_iter().collect();
    let (support_low, support_high) = kde_support_range(groups.iter().copied(), cut)?;
    let grid: Vec<f64> = (0..MARGINAL_KDE_GRID_POINTS)
        .map(|i| {
            support_low
                + (support_high - support_low) * (i as f64 / (MARGINAL_KDE_GRID_POINTS - 1) as f64)
        })
        .collect();

    let mut low = f64::INFINITY;
    let mut high = f64::NEG_INFINITY;
    for samples in groups {
        let Ok(densities) = kde(samples, &grid) else {
            continue;
        };
        let max_density = densities.iter().fold(0.0_f64, |acc, value| acc.max(*value));
        if max_density <= 0.0 {
            continue;
        }
        let threshold = max_density * visible_fraction;
        for (&x, &density) in grid.iter().zip(densities.iter()) {
            if density >= threshold {
                low = low.min(x);
                high = high.max(x);
            }
        }
    }

    (low.is_finite() && high.is_finite() && low < high).then_some((low, high))
}
