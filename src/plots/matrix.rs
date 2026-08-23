//! `lichtung-plot::plots::matrix` — scatter matrix (pairplot).
//!
//! Renders an N×N grid of pairwise scatter plots, matching
//! seaborn's `sns.pairplot()`. Diagonal cells show per-column
//! KDE density curves. Off-diagonal cells show scatter plots.
//! When `hue_col` is set, points and KDE curves are colored
//! per group with a legend.
//!
//! The matrix manages its own decorations (cell frames, ticks,
//! labels, legend) — the global `apply_decorations` is skipped.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::redundant_closure_for_method_calls,
    clippy::if_not_else
)]

use crate::decorations::legend::{Legend, LegendEntry};
use crate::decorations::title_block::TitleBlock;
use crate::layout::{self, FigureLayout, FigureLayoutSpec, Layout, PanelBox};
use crate::render::backend::{
    with_clip, with_layer, Backend, FillStyle, LineStyle, Point2D, TextAnchor, TextStyle,
};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::stats::distribution::{gaussian_kde_with_bandwidth, strict_scott_bandwidth};
use crate::Error;
use crate::{LegendPosition, Plot};

/// One graphical group for the matrix plot.
struct HueGroup {
    label: String,
    hue: Option<String>,
    /// Per-column data vectors (same order as `cols`).
    columns: Vec<Vec<f64>>,
    /// Strict Scott bandwidth per matrix column.
    bandwidths: Vec<f64>,
    color: (u8, u8, u8),
}

/// Render a scatter matrix (pairplot).
///
/// Returns a dummy `Layout` (the matrix is self-decorated) and
/// legend entries (empty — legend is drawn internally).
pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    let palette = chart.palette();
    let df = context.data;

    let cols = chart
        .matrix_variable_names()
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let n = cols.len();
    let display_cols = chart.resolve_label_domain("matrix variables", cols.iter().copied())?;

    // --- Build graphical groups ---
    let (groups, hue_levels) = build_groups(chart, &cols, df)?;
    let legend = matrix_legend(chart, &hue_levels)?;

    // --- Layout geometry ---
    let w = chart.width() as f64;
    let figure = matrix_figure_layout(chart)?;
    let outer = context.panel;
    let legend_reserve = legend.as_ref().map_or(0.0, |legend| {
        (legend_total_width(legend) + 32.0 - figure.insets.right).max(0.0)
    });
    let gap = 8.0;
    let provisional_grid_width =
        (outer.x1 - outer.x0 - legend_reserve - gap * n.saturating_sub(1) as f64).max(1.0);
    let provisional_grid_height = (outer.y1 - outer.y0 - gap * n.saturating_sub(1) as f64).max(1.0);
    let mut candidate_cell_width = provisional_grid_width / n as f64;
    let mut candidate_cell_height = provisional_grid_height / n as f64;
    let label_line_height = f64::from(palette.label_font_size) * 1.2;
    let mut resolved_layout = None;
    for _ in 0..8 {
        let column_blocks = display_cols
            .iter()
            .map(|label| {
                crate::layout::TextBlock::layout(
                    label,
                    palette.label_font_size,
                    candidate_cell_width.max(1.0),
                    "matrix column label",
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let row_blocks = display_cols
            .iter()
            .map(|label| {
                crate::layout::TextBlock::layout(
                    label,
                    palette.label_font_size,
                    candidate_cell_height.max(1.0),
                    "matrix row label",
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let extra_bottom = column_blocks
            .iter()
            .map(|block| block.lines().len().saturating_sub(1) as f64 * label_line_height)
            .fold(0.0_f64, f64::max);
        let extra_left = row_blocks
            .iter()
            .map(|block| block.lines().len().saturating_sub(1) as f64 * label_line_height)
            .fold(0.0_f64, f64::max);
        let grid_left = outer.x0 + extra_left;
        let grid_right = (outer.x1 - legend_reserve).max(grid_left + 1.0);
        let grid_bottom = outer.y0 + extra_bottom;
        let grid_top = outer.y1;
        let grid_w = (grid_right - grid_left - gap * (n.saturating_sub(1) as f64)).max(1.0);
        let grid_h = (grid_top - grid_bottom - gap * (n.saturating_sub(1) as f64)).max(1.0);
        let cell_w = grid_w / n as f64;
        let cell_h = grid_h / n as f64;
        if cell_w < 40.0 || cell_h < 40.0 {
            return Err(Error::Render(format!(
                "matrix labels and guides leave {cell_w:.1}x{cell_h:.1}px cells; increase chart size to preserve minimum 40x40 matrix panels"
            )));
        }
        if (cell_w - candidate_cell_width).abs() <= f64::EPSILON
            && (cell_h - candidate_cell_height).abs() <= f64::EPSILON
        {
            resolved_layout = Some((
                column_blocks,
                row_blocks,
                grid_left,
                grid_bottom,
                grid_top,
                cell_w,
                cell_h,
            ));
            break;
        }
        candidate_cell_width = cell_w;
        candidate_cell_height = cell_h;
    }
    let Some((column_blocks, row_blocks, grid_left, grid_bottom, grid_top, cell_w, cell_h)) =
        resolved_layout
    else {
        return Err(Error::Render(
            "matrix label layout did not stabilize; increase chart size or shorten variable labels"
                .into(),
        ));
    };

    // --- Per-variable axis ranges ---
    // A pairplot is a coordinated matrix: each variable should use one
    // canonical range wherever it appears, not one range as x and another
    // as y. The range includes the diagonal KDE evaluation extent so the
    // diagonal and off-diagonal cells stay visually aligned.
    let variable_ranges: Vec<(f64, f64)> = (0..n)
        .map(|column_index| {
            matrix_support_range(
                &groups,
                column_index,
                cols[column_index],
                chart.group_name(),
                chart.hue_name(),
            )
        })
        .collect::<Result<_, _>>()?;

    // --- Styles ---
    let frame_style = LineStyle {
        color: palette.foreground,
        width: palette.axis_width,
        dash: String::new(),
    };
    let tick_style = LineStyle {
        color: palette.foreground,
        width: palette.axis_width,
        dash: String::new(),
    };
    let tick_text_style = TextStyle {
        color: palette.foreground,
        font_family: palette.font_family.clone(),
        font_size: palette.tick_font_size,
        weight: crate::TextWeight::Normal,
        anchor: TextAnchor::Middle,
        rotation: 0.0,
    };
    let label_style = TextStyle {
        color: palette.foreground,
        font_family: palette.font_family.clone(),
        font_size: palette.label_font_size,
        weight: crate::TextWeight::Normal,
        anchor: TextAnchor::Middle,
        rotation: 0.0,
    };
    let tick_len = f64::from(palette.tick_length);

    draw_title_block(chart, backend)?;

    // --- Render cells ---
    for row in 0..n {
        for col in 0..n {
            let x0 = grid_left + col as f64 * (cell_w + gap);
            let x1 = x0 + cell_w;
            // row 0 = top visually → highest y in math coords
            let y0 = grid_bottom + (n - 1 - row) as f64 * (cell_h + gap);
            let y1 = y0 + cell_h;
            let cell_panel = PanelBox::new(x0, x1, y0, y1);

            if row == col {
                // --- Diagonal: KDE density curves + open frame ---
                if palette.show_frame {
                    draw_frame(backend, x0, x1, y0, y1, &frame_style)?;
                }
                with_clip(backend, cell_panel, |backend| {
                    draw_diagonal_kde(
                        backend,
                        &groups,
                        row,
                        cols[row],
                        chart.hue_col(),
                        chart.group_col(),
                        x0,
                        x1,
                        y0,
                        cell_h,
                        &variable_ranges[row],
                    )
                })?;
            } else {
                // --- Off-diagonal: scatter + open frame ---
                if palette.show_frame {
                    draw_frame(backend, x0, x1, y0, y1, &frame_style)?;
                }
                let (x_lo, x_hi) = variable_ranges[col];
                let (y_lo, y_hi) = variable_ranges[row];
                with_clip(backend, cell_panel, |backend| {
                    for group in &groups {
                        let xs = &group.columns[col];
                        let ys = &group.columns[row];
                        let ps = chart.point_style(group.color, 2);
                        let pts: Vec<Point2D> = xs
                            .iter()
                            .zip(ys.iter())
                            .map(|(&xi, &yi)| {
                                let px = x0 + ((xi - x_lo) / (x_hi - x_lo)) * cell_w;
                                let py = y0 + ((yi - y_lo) / (y_hi - y_lo)) * cell_h;
                                Point2D::new(px, py)
                            })
                            .collect();
                        backend.draw_scatter(&pts, &ps)?;
                    }
                    Ok(())
                })?;
            }

            // --- X-axis ticks ---
            {
                let (x_lo, x_hi) = variable_ranges[col];
                let ticks = layout::nice_ticks(x_lo, x_hi, 4);
                for &tv in &ticks {
                    let t = ((tv - x_lo) / (x_hi - x_lo)).clamp(0.0, 1.0);
                    let px = x0 + t * cell_w;
                    // Tick mark on every cell
                    if palette.show_ticks {
                        backend.draw_line(
                            &[Point2D::new(px, y0), Point2D::new(px, y0 - tick_len)],
                            &tick_style,
                        )?;
                    }
                    // Tick label only on bottom row
                    if row == n - 1 && palette.show_tick_labels {
                        backend.draw_text(
                            px.round() as u32,
                            (y0 - tick_len - 8.0).round() as u32,
                            &layout::format_tick(tv),
                            &tick_text_style,
                        )?;
                    }
                }
                // Column label below ticks (bottom row only)
                if row == n - 1 && palette.show_x_label {
                    let cx = f64::midpoint(x0, x1);
                    let first_y = y0 - tick_len - 28.0;
                    for (index, line) in column_blocks[col].lines().iter().enumerate() {
                        backend.draw_text(
                            cx.round() as u32,
                            (first_y - index as f64 * column_blocks[col].line_height()).round()
                                as u32,
                            line,
                            &label_style,
                        )?;
                    }
                }
            }
            // --- Y-axis ticks ---
            {
                let (y_lo, y_hi) = variable_ranges[row];
                let ticks = layout::nice_ticks(y_lo, y_hi, 6);
                let y_tick_text = TextStyle {
                    anchor: TextAnchor::End,
                    ..tick_text_style.clone()
                };
                for &tv in &ticks {
                    let t = ((tv - y_lo) / (y_hi - y_lo)).clamp(0.0, 1.0);
                    let py = y0 + t * cell_h;
                    // Tick mark on every cell
                    if palette.show_ticks {
                        backend.draw_line(
                            &[Point2D::new(x0, py), Point2D::new(x0 - tick_len, py)],
                            &tick_style,
                        )?;
                    }
                    // Tick label only on left column
                    if col == 0 && palette.show_tick_labels {
                        backend.draw_text(
                            (x0 - tick_len - 4.0).max(0.0).round() as u32,
                            py.round() as u32,
                            &layout::format_tick(tv),
                            &y_tick_text,
                        )?;
                    }
                }
                // Row label to the left of ticks (left column only)
                if col == 0 && palette.show_y_label {
                    let cy = f64::midpoint(y0, y1);
                    let row_label_style = TextStyle {
                        anchor: TextAnchor::Middle,
                        rotation: -90.0,
                        ..label_style.clone()
                    };
                    let first_x = (x0 - tick_len - 40.0).max(4.0);
                    for (index, line) in row_blocks[row].lines().iter().enumerate() {
                        backend.draw_text(
                            (first_x - index as f64 * row_blocks[row].line_height())
                                .max(4.0)
                                .round() as u32,
                            cy.round() as u32,
                            line,
                            &row_label_style,
                        )?;
                    }
                }
            }
        }
    }

    // --- Legend (when hue is active) ---
    if let Some(legend) = legend {
        let right_edge = (w - 16.0).max(legend_total_width(&legend));
        let top_edge = grid_top - 4.0;
        let legend_w = legend.compute_box_width();
        let legend_h = legend.compute_box_height();
        let left = right_edge - legend_w;
        let bottom = match chart.legend_position_spec() {
            LegendPosition::OutsideRight => f64::midpoint(grid_bottom, top_edge) - legend_h / 2.0,
            LegendPosition::OutsideRightBottom => grid_bottom,
            _ => top_edge - legend_h,
        }
        .max(0.0);
        let (lx, ly) = legend.box_at(left, bottom).anchor();
        with_layer(backend, "legend", |backend| legend.draw(backend, lx, ly))?;
    }

    // Return a dummy layout — matrix is self-decorated.
    let layout = Layout::new(
        (0.0, 1.0),
        (0.0, 1.0),
        chart.width(),
        chart.height(),
        String::new(),
        String::new(),
    );

    Ok(PlotRenderResult::new(layout, Vec::new()).with_self_managed_decorations())
}

pub(crate) fn panel_box(chart: &Plot) -> Result<crate::layout::PanelBox, Error> {
    Ok(matrix_figure_layout(chart)?.panel_box)
}

fn matrix_legend(chart: &Plot, hue_levels: &[String]) -> Result<Option<Legend>, Error> {
    if hue_levels.len() <= 1 || chart.legend_policy_spec() == crate::LegendPolicy::Hidden {
        return Ok(None);
    }
    let hue_title = chart.resolved_legend_title();
    let display_levels = chart.resolve_label_domain(
        "matrix legend entries",
        hue_levels.iter().map(String::as_str),
    )?;
    let entries: Vec<LegendEntry> = hue_levels
        .iter()
        .zip(display_levels)
        .enumerate()
        .map(|(index, (hue, display))| {
            LegendEntry::point(
                display,
                chart.point_style(
                    chart.palette().colors
                        [chart.hue_color_index(hue, index) % chart.palette().colors.len()],
                    3,
                ),
            )
        })
        .collect();
    Ok(Some(Legend::new(
        hue_title,
        entries,
        f64::from(chart.width()),
        f64::from(chart.height()),
    )))
}

fn legend_total_width(legend: &Legend) -> f64 {
    legend.compute_box_width()
}

fn draw_title_block(chart: &Plot, backend: &mut dyn Backend) -> Result<(), Error> {
    let palette = chart.palette();
    let mut title_block = TitleBlock::new().font_family(palette.font_family.clone());
    if palette.show_title {
        if let Some(title) = chart.title_text() {
            title_block = title_block.title(title.to_string());
        }
    }
    if palette.show_subtitle {
        if let Some(subtitle) = chart.subtitle_text() {
            title_block = title_block.subtitle(subtitle.to_string());
        }
    }
    if palette.show_caption {
        if let Some(caption) = chart.caption_text() {
            title_block = title_block.caption(caption.to_string());
        }
    }
    if title_block.has_content() {
        let figure = matrix_figure_layout(chart)?;
        title_block.draw(backend, chart.width(), &figure, &palette)?;
    }
    Ok(())
}

fn matrix_figure_layout(chart: &Plot) -> Result<FigureLayout, Error> {
    let palette = chart.palette();
    FigureLayout::measured_single_panel(
        chart.width(),
        chart.height(),
        &palette,
        FigureLayoutSpec {
            has_title: palette.show_title && chart.title_text().is_some_and(|s| !s.is_empty()),
            has_subtitle: palette.show_subtitle
                && chart.subtitle_text().is_some_and(|s| !s.is_empty()),
            has_caption: palette.show_caption
                && chart.caption_text().is_some_and(|s| !s.is_empty()),
            has_x_label: palette.show_x_label,
            has_y_label: palette.show_y_label,
        },
        (
            chart.title_text(),
            chart.subtitle_text(),
            chart.caption_text(),
        ),
    )
}

/// Draw KDE curves on a diagonal cell (one per graphical group).
#[allow(clippy::too_many_arguments)]
fn draw_diagonal_kde(
    backend: &mut dyn Backend,
    groups: &[HueGroup],
    col_idx: usize,
    column: &str,
    hue_column: Option<&str>,
    group_column: Option<&str>,
    x0: f64,
    x1: f64,
    y0: f64,
    cell_h: f64,
    range: &(f64, f64),
) -> Result<(), Error> {
    let cell_w = x1 - x0;
    let (d_lo, d_hi) = *range;
    let n_kde = 60;
    let grid_pts: Vec<f64> = (0..n_kde)
        .map(|i| d_lo + (d_hi - d_lo) * (i as f64 / (n_kde - 1) as f64))
        .collect();

    // Total sample count across all groups for common_norm scaling
    // (matches seaborn pairplot default: area proportional to group size).
    #[allow(clippy::cast_precision_loss)]
    let n_total: f64 = groups.iter().map(|g| g.columns[col_idx].len() as f64).sum();

    // Compute KDE for each group, scaled by group proportion (common_norm).
    let densities_per_group: Vec<Vec<f64>> = groups
        .iter()
        .map(|g| {
            let context = matrix_group_context(column, group_column, hue_column, g);
            let raw =
                gaussian_kde_with_bandwidth(&g.columns[col_idx], &grid_pts, g.bandwidths[col_idx])
                    .map_err(|error| Error::Data(format!("matrix: {context}: {error}")))?;
            #[allow(clippy::cast_precision_loss)]
            let weight = g.columns[col_idx].len() as f64 / n_total;
            let weighted = raw
                .into_iter()
                .map(|density| density * weight)
                .collect::<Vec<_>>();
            if weighted.iter().any(|density| !density.is_finite()) {
                return Err(Error::Data(format!(
                    "matrix: {context} produced non-finite weighted KDE density"
                )));
            }
            Ok(weighted)
        })
        .collect::<Result<_, Error>>()?;

    // Shared max across all weighted densities.
    let d_max = densities_per_group
        .iter()
        .flat_map(|d| d.iter())
        .fold(0.0_f64, |a, &v| a.max(v));
    if !d_max.is_finite() || d_max <= 0.0 {
        return Err(Error::Data(format!(
            "matrix: column {column:?} produced no positive finite KDE density"
        )));
    }

    let usable_h = cell_h * 0.85;

    for (gi, densities) in densities_per_group.iter().enumerate() {
        let color = groups[gi].color;
        let mut poly = Vec::with_capacity(n_kde + 2);
        poly.push(Point2D::new(x0, y0));
        for (i, &d) in densities.iter().enumerate() {
            let px = x0 + (i as f64 / (n_kde - 1) as f64) * cell_w;
            let py = y0 + (d / d_max) * usable_h;
            poly.push(Point2D::new(px, py));
        }
        poly.push(Point2D::new(x1, y0));
        let fill = FillStyle {
            color,
            stroke: color,
            stroke_width: 1.0,
            opacity: 0.40,
        };
        backend.draw_polygon(&poly, &fill)?;
    }
    Ok(())
}

/// Build graphical groups while preserving hue as color-only metadata.
fn build_groups(
    chart: &Plot,
    cols: &[&str],
    data: crate::data::PlotData<'_>,
) -> Result<(Vec<HueGroup>, Vec<String>), Error> {
    let palette = chart.palette();
    let columns = extract_columns_f64(data, cols)?;
    let channels =
        crate::data::collective_channels(data, chart.group_col(), chart.hue_col(), None, "matrix")?;
    let group_levels = chart.resolved_group_levels(channels.group_levels.clone());
    let hue_levels = chart.resolved_hue_levels(channels.hue_levels.clone());
    let identities = channels.groups.as_ref().map_or_else(
        || vec![None],
        |_| group_levels.into_iter().map(Some).collect(),
    );
    let mut groups = Vec::new();
    for identity in identities {
        let rows = (0..data.height())
            .filter(|row| {
                channels
                    .groups
                    .as_ref()
                    .is_none_or(|values| identity.as_ref() == Some(&values[*row]))
            })
            .collect::<Vec<_>>();
        if rows.is_empty() {
            continue;
        }
        let hue = channels.hues.as_ref().map(|values| values[rows[0]].clone());
        let color_index = hue
            .as_deref()
            .and_then(|hue| hue_levels.iter().position(|level| level == hue))
            .unwrap_or(0);
        groups.push(HueGroup {
            label: identity.unwrap_or_default(),
            hue,
            columns: columns
                .iter()
                .map(|column| rows.iter().map(|row| column[*row]).collect())
                .collect(),
            bandwidths: Vec::new(),
            color: palette.colors[color_index % palette.colors.len()],
        });
    }
    if groups.is_empty() {
        return Err(Error::Data("matrix: no valid finite rows".into()));
    }
    validate_group_kdes(&mut groups, cols, chart.group_name(), chart.hue_name())?;
    Ok((groups, hue_levels))
}

fn validate_group_kdes(
    groups: &mut [HueGroup],
    columns: &[&str],
    group_column: Option<&str>,
    hue_column: Option<&str>,
) -> Result<(), Error> {
    for group in groups {
        group.bandwidths = group
            .columns
            .iter()
            .zip(columns)
            .map(|(samples, column)| {
                let context = matrix_group_context(column, group_column, hue_column, group);
                strict_scott_bandwidth(samples)
                    .map_err(|reason| Error::Data(format!("matrix: {context} {reason}")))
            })
            .collect::<Result<_, _>>()?;
    }
    Ok(())
}

fn matrix_support_range(
    groups: &[HueGroup],
    column_index: usize,
    column: &str,
    group_column: Option<&str>,
    hue_column: Option<&str>,
) -> Result<(f64, f64), Error> {
    let mut support_low = f64::INFINITY;
    let mut support_high = f64::NEG_INFINITY;
    for group in groups {
        let samples = &group.columns[column_index];
        let minimum = samples.iter().copied().fold(f64::INFINITY, f64::min);
        let maximum = samples.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let bandwidth = group.bandwidths[column_index];
        let low = minimum - 3.0 * bandwidth;
        let high = maximum + 3.0 * bandwidth;
        if !low.is_finite() || !high.is_finite() || low >= high {
            let context = matrix_group_context(column, group_column, hue_column, group);
            return Err(Error::Data(format!(
                "matrix: {context} cannot produce finite KDE support"
            )));
        }
        support_low = support_low.min(low);
        support_high = support_high.max(high);
    }

    let span = support_high - support_low;
    let margin = span * 0.05;
    let range = (support_low - margin, support_high + margin);
    if !span.is_finite()
        || span <= 0.0
        || !range.0.is_finite()
        || !range.1.is_finite()
        || range.0 >= range.1
    {
        let context = match (group_column, hue_column) {
            (Some(group), Some(hue)) => format!(
                "column {column:?} across graphical group {group:?} with hue column {hue:?}"
            ),
            (Some(group), None) => {
                format!("column {column:?} across graphical group column {group:?}")
            }
            (None, Some(hue)) => {
                format!("column {column:?} across hue column {hue:?} groups")
            }
            (None, None) => format!("column {column:?}"),
        };
        return Err(Error::Data(format!(
            "matrix: {context} cannot produce a finite shared KDE support range"
        )));
    }
    Ok(range)
}

fn matrix_group_context(
    column: &str,
    group_column: Option<&str>,
    hue_column: Option<&str>,
    group: &HueGroup,
) -> String {
    match (group_column, hue_column, group.hue.as_deref()) {
        (Some(group_column), Some(hue_column), Some(hue)) => format!(
            "column {column:?} in graphical group {group_column:?}={:?} with hue {hue_column:?}={hue:?}",
            group.label
        ),
        (Some(group_column), _, _) => format!(
            "column {column:?} in graphical group {group_column:?}={:?}",
            group.label
        ),
        (None, Some(hue_column), _) => format!(
            "column {column:?} in hue {hue_column:?} group {:?}",
            group.label
        ),
        (None, None, _) => format!("column {column:?} in the ungrouped data"),
    }
}

fn extract_columns_f64(
    data: crate::data::PlotData<'_>,
    cols: &[&str],
) -> Result<Vec<Vec<f64>>, Error> {
    let columns: Vec<_> = cols
        .iter()
        .map(|column| crate::data::strict_finite_f64(data, column, "matrix"))
        .collect::<Result<Vec<_>, Error>>()?;

    if columns.iter().any(|column| column.len() != data.height()) {
        return Err(Error::Data("matrix: column length mismatch".into()));
    }
    Ok(columns)
}

/// Draw an open frame (despine): only bottom and left spines.
fn draw_frame(
    backend: &mut dyn Backend,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    style: &LineStyle,
) -> Result<(), Error> {
    // Bottom spine
    backend.draw_line(&[Point2D::new(x0, y0), Point2D::new(x1, y0)], style)?;
    // Left spine
    backend.draw_line(&[Point2D::new(x0, y0), Point2D::new(x0, y1)], style)
}
