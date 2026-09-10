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
use crate::layout::{
    self, CanvasLayout, CanvasLayoutSpec, Layout, PanelBox, PhysicalTextAlignment, TextBlock,
};
use crate::render::backend::{
    points_share_canonical_position, with_clip, with_layer, Backend, FillStyle, LineStyle,
    PlacedText, Point2D, TextAnchor,
};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::stats::distribution::{gaussian_kde_with_bandwidth, strict_scott_bandwidth};
use crate::theme::ResolvedTheme;
use crate::Error;
use crate::{LegendPosition, Plot};

/// One graphical group for the matrix plot.
struct HueGroup {
    identity: crate::data::CollectivePair,
    collapsed: bool,
    /// Per-column data vectors (same order as `cols`).
    columns: Vec<Vec<f64>>,
    /// Strict Scott bandwidth per matrix column.
    bandwidths: Vec<f64>,
    color: (u8, u8, u8),
}

#[cfg(test)]
#[test]
fn tick_planner_failure_is_not_reported_as_small_matrix_cells() {
    let plot = Plot::matrix(vec![
        crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        crate::NumericValues::new("y", [4.0, 3.0, 2.0, 1.0]),
    ])
    .erase();
    let mut theme = crate::Theme::Publication.resolve();
    theme.prepare_font_environment().unwrap();
    theme.typography.tick_label.font_size = 0;
    let mut backend = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
    backend.begin(plot.width(), plot.height()).unwrap();
    let context =
        PlotRenderContext::new(plot.data(), PanelBox::new(80.0, 600.0, 60.0, 440.0), &theme);
    let error = render(&plot, context, &mut backend).err().unwrap();
    backend.end().unwrap();
    assert!(
        error.to_string().contains("font size must be positive"),
        "{error}"
    );
    assert!(!error.to_string().contains("within their cells"));
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
    let theme = context.theme;
    let mut text = context.producer_text()?;
    let df = context.data;

    let cols = chart
        .matrix_variable_names()
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let n = cols.len();
    let display_cols = chart.resolve_label_domain("matrix variables", cols.iter().copied())?;

    // --- Build graphical groups ---
    let (groups, _hue_levels) = build_groups(chart, &cols, df, theme)?;
    let legend = match context.self_managed_legend {
        Some(legend) => legend.clone(),
        None => matrix_legend(chart, &groups, theme)?
            .map(|legend| {
                let environment = theme.prepared_font_environment()?;
                environment
                    .claim_text_generation_slots(1)
                    .map_err(|error| Error::Render(format!("matrix legend slot: {error:?}")))?;
                environment
                    .claim_unicode_bytes(
                        (std::mem::size_of::<Legend>() + 2 * std::mem::size_of::<usize>()) as u64,
                    )
                    .map_err(|error| Error::Render(format!("matrix legend bytes: {error}")))?;
                Ok::<_, Error>(std::sync::Arc::new(legend))
            })
            .transpose()?,
    };

    // --- Layout geometry ---
    let w = chart.width() as f64;
    let canvas = matrix_canvas_layout(chart, theme)?;
    let outer = context.panel;
    let legend_reserve = legend.as_ref().map_or(0.0, |legend| {
        (legend_total_width(legend) + 32.0 - canvas.insets.right).max(0.0)
    });
    let gap = 8.0;
    let provisional_grid_width =
        (outer.x1 - outer.x0 - legend_reserve - gap * n.saturating_sub(1) as f64).max(1.0);
    let provisional_grid_height = (outer.y1 - outer.y0 - gap * n.saturating_sub(1) as f64).max(1.0);
    let mut candidate_cell_width = provisional_grid_width / n as f64;
    let mut candidate_cell_height = provisional_grid_height / n as f64;
    let label_line_height = theme.typography.axis_title.line_height();
    let mut resolved_layout = None;
    for _ in 0..8 {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::MatrixLayoutPass,
        );
        let column_blocks = display_cols
            .iter()
            .map(|label| {
                text.plan(label, "matrix column label", || {
                    Ok(theme.plan_legacy_appearance_text(
                        label,
                        &theme.typography.axis_title,
                        candidate_cell_width.max(1.0),
                        PhysicalTextAlignment::Center,
                        "matrix column label",
                        false,
                    )?)
                })
                .map(TextBlock::from_resolved)
                .map_err(crate::layout::fit::PlanError::into_error)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let row_blocks = display_cols
            .iter()
            .map(|label| {
                text.plan(label, "matrix row label", || {
                    Ok(theme.plan_legacy_appearance_text(
                        label,
                        &theme.typography.axis_title,
                        candidate_cell_height.max(1.0),
                        PhysicalTextAlignment::Center,
                        "matrix row label",
                        false,
                    )?)
                })
                .map(TextBlock::from_resolved)
                .map_err(crate::layout::fit::PlanError::into_error)
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
    let frame_style = theme
        .axes
        .frame
        .as_ref()
        .map(|appearance| appearance.stroke.backend_style());
    let tick_style = theme
        .axes
        .ticks
        .as_ref()
        .map(|appearance| appearance.stroke.backend_style());
    let tick_len = theme
        .axes
        .ticks
        .as_ref()
        .map_or(0.0, |appearance| f64::from(appearance.length));

    // X selection depends only on a column's range and X coordinates; Y
    // selection depends only on a row's range and Y coordinates. Keep the
    // unfiltered selections separate from outer-cell label filtering.
    let environment = theme.prepared_font_environment()?;
    let cache_records = n
        .checked_mul(2)
        .ok_or_else(|| Error::Render("matrix tick cache count overflow".into()))?;
    let cache_bytes = cache_records
        .checked_mul(size_of::<Option<Vec<layout::AxisTick>>>())
        .ok_or_else(|| Error::Render("matrix tick cache bytes overflow".into()))?;
    environment
        .claim_text_generation_slots(cache_records as u128)
        .map_err(|error| Error::Render(format!("matrix tick cache: {error:?}")))?;
    environment
        .claim_unicode_bytes(
            u64::try_from(cache_bytes)
                .map_err(|_| Error::Render("matrix tick cache bytes overflow".into()))?,
        )
        .map_err(|error| Error::Render(format!("matrix tick cache: {error}")))?;
    let mut x_ticks = Vec::new();
    let mut y_ticks = Vec::new();
    x_ticks
        .try_reserve_exact(n)
        .map_err(|_| Error::Render("cannot reserve matrix X tick cache".into()))?;
    y_ticks
        .try_reserve_exact(n)
        .map_err(|_| Error::Render("cannot reserve matrix Y tick cache".into()))?;
    x_ticks.resize_with(n, || None);
    y_ticks.resize_with(n, || None);

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
                if let Some(frame_style) = &frame_style {
                    draw_frame(backend, x0, x1, y0, y1, frame_style)?;
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
                if let Some(frame_style) = &frame_style {
                    draw_frame(backend, x0, x1, y0, y1, frame_style)?;
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
                let projection = crate::layout::Projection::new(
                    (x_lo, x_hi),
                    variable_ranges[row],
                    (x0, x1, y0, y1),
                );
                let ticks = matrix_ticks(
                    &mut x_ticks[col],
                    (x_lo, x_hi),
                    &projection,
                    crate::layout::AxisSide::Bottom,
                    theme,
                )?;
                let filtered;
                let ticks = if row == n - 1 {
                    filtered = layout::retain_automatic_ticks_inside_panel_legacy(
                        copy_ticks(ticks, theme)?,
                        &projection,
                        crate::layout::AxisSide::Bottom,
                        theme,
                    )?;
                    filtered.as_slice()
                } else {
                    ticks
                };
                if row == n - 1 && ticks.is_empty() {
                    return Err(Error::Render(
                        "matrix x-axis tick labels do not fit within their cells; increase chart width or reduce the variable count"
                            .into(),
                    ));
                }
                for tick in ticks {
                    let tv = tick.position;
                    let t = ((tv - x_lo) / (x_hi - x_lo)).clamp(0.0, 1.0);
                    let px = x0 + t * cell_w;
                    // Tick mark on every cell
                    if let Some(tick_style) = &tick_style {
                        backend.draw_line(
                            &[Point2D::new(px, y0), Point2D::new(px, y0 - tick_len)],
                            tick_style,
                        )?;
                    }
                    // Tick label only on bottom row
                    if row == n - 1 {
                        let plan = text
                            .plan(&tick.label, "matrix x tick", || {
                                Ok(theme.measure_legacy_appearance_text(
                                    &tick.label,
                                    &theme.typography.tick_label,
                                    PhysicalTextAlignment::Center,
                                    "matrix x tick",
                                    false,
                                )?)
                            })
                            .map_err(crate::layout::fit::PlanError::into_error)?;
                        text.draw(
                            PlacedText::from_first_line(
                                plan,
                                Point2D::new(px.round(), (y0 - tick_len - 8.0).round()),
                                TextAnchor::Middle,
                                0.0,
                            ),
                            backend,
                        )?;
                    }
                }
                // Column label below ticks (bottom row only)
                if row == n - 1 {
                    let cx = f64::midpoint(x0, x1);
                    let first_y = y0 - tick_len - 28.0;
                    text.draw(
                        PlacedText::from_first_line(
                            column_blocks[col].plan().clone(),
                            Point2D::new(cx.round(), first_y.round()),
                            TextAnchor::Middle,
                            0.0,
                        ),
                        backend,
                    )?;
                }
            }
            // --- Y-axis ticks ---
            {
                let (y_lo, y_hi) = variable_ranges[row];
                let projection = crate::layout::Projection::new(
                    variable_ranges[col],
                    (y_lo, y_hi),
                    (x0, x1, y0, y1),
                );
                let ticks = matrix_ticks(
                    &mut y_ticks[row],
                    (y_lo, y_hi),
                    &projection,
                    crate::layout::AxisSide::Left,
                    theme,
                )?;
                let filtered;
                let ticks = if col == 0 {
                    filtered = layout::retain_automatic_ticks_inside_panel_legacy(
                        copy_ticks(ticks, theme)?,
                        &projection,
                        crate::layout::AxisSide::Left,
                        theme,
                    )?;
                    filtered.as_slice()
                } else {
                    ticks
                };
                if col == 0 && ticks.is_empty() {
                    return Err(Error::Render(
                        "matrix y-axis tick labels do not fit within their cells; increase chart height or reduce the variable count"
                            .into(),
                    ));
                }
                for tick in ticks {
                    let tv = tick.position;
                    let t = ((tv - y_lo) / (y_hi - y_lo)).clamp(0.0, 1.0);
                    let py = y0 + t * cell_h;
                    // Tick mark on every cell
                    if let Some(tick_style) = &tick_style {
                        backend.draw_line(
                            &[Point2D::new(x0, py), Point2D::new(x0 - tick_len, py)],
                            tick_style,
                        )?;
                    }
                    // Tick label only on left column
                    if col == 0 {
                        let plan = text
                            .plan(&tick.label, "matrix y tick", || {
                                Ok(theme.measure_legacy_appearance_text(
                                    &tick.label,
                                    &theme.typography.tick_label,
                                    PhysicalTextAlignment::End,
                                    "matrix y tick",
                                    false,
                                )?)
                            })
                            .map_err(crate::layout::fit::PlanError::into_error)?;
                        text.draw(
                            PlacedText::from_first_line(
                                plan,
                                Point2D::new((x0 - tick_len - 4.0).max(0.0).round(), py.round()),
                                TextAnchor::End,
                                0.0,
                            ),
                            backend,
                        )?;
                    }
                }
                // Row label to the left of ticks (left column only)
                if col == 0 {
                    let cy = f64::midpoint(y0, y1);
                    let first_x = (x0 - tick_len - 40.0).max(4.0);
                    let first_x = (first_x
                        - row_blocks[row].plan().lines.len().saturating_sub(1) as f64
                            * row_blocks[row].line_height())
                    .max(4.0);
                    text.draw(
                        PlacedText::from_first_line(
                            row_blocks[row].plan().clone(),
                            Point2D::new(first_x.round(), cy.round()),
                            TextAnchor::Middle,
                            -90.0,
                        )
                        .with_reversed_lines(),
                        backend,
                    )?;
                }
            }
        }
    }

    // --- Legend (when hue is active) ---
    if let Some(legend) = &legend {
        let right_edge = (w - 16.0).max(legend_total_width(legend));
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
    let layout = Layout::self_managed(chart.width(), chart.height());

    let mut result = PlotRenderResult::new(layout, Vec::new()).with_self_managed_decorations();
    result.producer_text = text.finish()?;
    result.self_managed_legend = legend;
    Ok(result)
}

fn matrix_ticks<'a>(
    cached: &'a mut Option<Vec<layout::AxisTick>>,
    range: (f64, f64),
    projection: &layout::Projection,
    side: layout::AxisSide,
    theme: &ResolvedTheme,
) -> Result<&'a [layout::AxisTick], Error> {
    let environment = theme.prepared_font_environment()?;
    environment
        .claim_processing_visits(1)
        .map_err(|error| Error::Render(format!("matrix tick lookup: {error:?}")))?;
    if cached.is_none() {
        let ticks = layout::automatic_numeric_ticks_legacy(range, projection, side, theme)?;
        environment
            .claim_text_generation_slots(ticks.len() as u128)
            .map_err(|error| Error::Render(format!("matrix tick retention: {error:?}")))?;
        let bytes = ticks.iter().try_fold(
            ticks
                .capacity()
                .checked_mul(size_of::<layout::AxisTick>())
                .ok_or_else(|| Error::Render("matrix tick bytes overflow".into()))?,
            |bytes, tick| {
                bytes
                    .checked_add(tick.label.capacity())
                    .ok_or_else(|| Error::Render("matrix tick bytes overflow".into()))
            },
        )?;
        environment
            .claim_unicode_bytes(
                u64::try_from(bytes)
                    .map_err(|_| Error::Render("matrix tick bytes overflow".into()))?,
            )
            .map_err(|error| Error::Render(format!("matrix tick retention: {error}")))?;
        *cached = Some(ticks);
    }
    cached
        .as_deref()
        .ok_or_else(|| Error::Render("matrix tick selection is missing".into()))
}

fn copy_ticks(
    ticks: &[layout::AxisTick],
    theme: &ResolvedTheme,
) -> Result<Vec<layout::AxisTick>, Error> {
    let environment = theme.prepared_font_environment()?;
    environment
        .claim_text_generation_slots(ticks.len() as u128)
        .map_err(|error| Error::Render(format!("matrix outer ticks: {error:?}")))?;
    let text_bytes = ticks.iter().try_fold(0_usize, |bytes, tick| {
        bytes
            .checked_add(tick.label.len())
            .ok_or_else(|| Error::Render("matrix outer tick text bytes overflow".into()))
    })?;
    let bytes = ticks
        .len()
        .checked_mul(size_of::<layout::AxisTick>())
        .and_then(|bytes| bytes.checked_add(text_bytes))
        .ok_or_else(|| Error::Render("matrix outer tick bytes overflow".into()))?;
    environment
        .claim_unicode_bytes(
            u64::try_from(bytes)
                .map_err(|_| Error::Render("matrix outer tick bytes overflow".into()))?,
        )
        .map_err(|error| Error::Render(format!("matrix outer ticks: {error}")))?;
    let mut copied = Vec::new();
    copied
        .try_reserve_exact(ticks.len())
        .map_err(|_| Error::Render("cannot reserve matrix outer ticks".into()))?;
    for tick in ticks {
        let mut label = String::new();
        label
            .try_reserve_exact(tick.label.len())
            .map_err(|_| Error::Render("cannot reserve matrix tick text".into()))?;
        label.push_str(&tick.label);
        copied.push(layout::AxisTick::new(tick.position, label));
    }
    Ok(copied)
}

#[test]
fn matrix_tick_selection_is_shared_only_along_the_orthogonal_axis() {
    let theme = crate::Theme::Publication.resolve();
    let environment = theme.prepared_font_environment().unwrap();
    let resources = environment.generation_resources();
    environment.share_generation_resources(&resources).unwrap();
    let range = (0.0, 10.0);
    let original = layout::Projection::new(range, range, (10.25, 210.75, 20.25, 220.75));
    for (side, translated) in [
        (
            layout::AxisSide::Bottom,
            layout::Projection::new(range, (-100.0, 100.0), (10.25, 210.75, 500.0, 800.0)),
        ),
        (
            layout::AxisSide::Left,
            layout::Projection::new((-100.0, 100.0), range, (500.0, 800.0, 20.25, 220.75)),
        ),
    ] {
        let mut cached = None;
        let first = matrix_ticks(&mut cached, range, &original, side, &theme).unwrap();
        let expected =
            layout::automatic_numeric_ticks_legacy(range, &translated, side, &theme).unwrap();
        assert_eq!(first, expected);
        let before = resources.usage().unwrap();
        let bytes = environment.resource_usage().unwrap().font_bytes;
        let reused = matrix_ticks(&mut cached, range, &translated, side, &theme).unwrap();
        assert_eq!(reused, expected);
        assert_eq!(resources.usage().unwrap(), (before.0, before.1 + 1));
        assert_eq!(environment.resource_usage().unwrap().font_bytes, bytes);
        let expected_outer =
            layout::retain_automatic_ticks_inside_panel_legacy(expected, &translated, side, &theme)
                .unwrap();
        let actual_outer = layout::retain_automatic_ticks_inside_panel_legacy(
            copy_ticks(reused, &theme).unwrap(),
            &translated,
            side,
            &theme,
        )
        .unwrap();
        assert_eq!(actual_outer, expected_outer);
    }
}

#[test]
fn matrix_outer_tick_copy_admits_bytes_before_copying() {
    let theme = crate::Theme::Publication.resolve();
    let environment = theme.prepared_font_environment().unwrap();
    let used = environment.resource_usage().unwrap().font_bytes;
    environment
        .claim_unicode_bytes(u64::try_from(536_870_912 - used).unwrap())
        .unwrap();
    let error = copy_ticks(&[layout::AxisTick::new(1.0, "1")], &theme).unwrap_err();
    assert!(error.to_string().contains("system-font bytes"));
}

pub(crate) fn panel_box(
    chart: &Plot,
    theme: &ResolvedTheme,
) -> Result<crate::layout::PanelBox, Error> {
    Ok(matrix_canvas_layout(chart, theme)?.panel_box)
}

fn matrix_legend(
    chart: &Plot,
    groups: &[HueGroup],
    theme: &ResolvedTheme,
) -> Result<Option<Legend>, Error> {
    if !groups.iter().any(|group| group.identity.hue.is_some())
        || chart.legend_policy_spec() == crate::LegendPolicy::Hidden
    {
        return Ok(None);
    }
    let mut entries = groups
        .iter()
        .filter(|group| group.identity.hue.is_some())
        .map(|group| {
            Ok(LegendEntry::point_fill(
                chart.collective_legend_identity(&group.identity)?,
                chart.point_style(group.color, 2),
                FillStyle {
                    color: group.color,
                    stroke: group.color,
                    stroke_width: 1.0,
                    opacity: 0.40,
                },
            ))
        })
        .collect::<Result<Vec<_>, Error>>()?;
    crate::decorations::legend::resolve_entries(chart, &mut entries)?;
    Ok(Some(Legend::new(
        &crate::decorations::legend::resolved_title(&entries)?,
        entries,
        f64::from(chart.width()),
        f64::from(chart.height()),
        theme,
    )?))
}

fn legend_total_width(legend: &Legend) -> f64 {
    legend.compute_box_width()
}

fn matrix_canvas_layout(chart: &Plot, theme: &ResolvedTheme) -> Result<CanvasLayout, Error> {
    CanvasLayout::measured_single_panel(
        chart.width(),
        chart.height(),
        theme,
        CanvasLayoutSpec {
            has_title: chart.title_text().is_some_and(|s| !s.is_empty()),
            has_subtitle: chart.subtitle_text().is_some_and(|s| !s.is_empty()),
            has_caption: chart.caption_text().is_some_and(|s| !s.is_empty()),
            has_x_label: true,
            has_y_label: true,
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
            let point = Point2D::new(px, py);
            let closure = if i == 0 {
                Some(Point2D::new(x0, y0))
            } else if i + 1 == densities.len() {
                Some(Point2D::new(x1, y0))
            } else {
                None
            };
            if let Some(closure) = closure {
                if points_share_canonical_position(point, closure, "matrix KDE closure")? {
                    poly.push(closure);
                    continue;
                }
            }
            poly.push(point);
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
    theme: &ResolvedTheme,
) -> Result<(Vec<HueGroup>, Vec<String>), Error> {
    let columns = extract_columns_f64(data, cols)?;
    let channels = crate::data::observed_collectives(
        data,
        chart.group_col(),
        chart.group_name(),
        chart.hue_col(),
        chart.hue_name(),
        None,
        "matrix",
    )?;
    let hue_levels = chart.resolved_hue_levels(channels.hue_levels.clone());
    let identities = chart.resolved_collective_pairs(channels.identities.clone());
    let mut groups = Vec::new();
    for identity in identities {
        let rows = channels
            .rows
            .iter()
            .enumerate()
            .filter_map(|(row, row_identity)| (row_identity == &identity).then_some(row))
            .collect::<Vec<_>>();
        if rows.is_empty() {
            continue;
        }
        let color_index = identity
            .hue
            .as_deref()
            .and_then(|hue| hue_levels.iter().position(|level| level == hue))
            .unwrap_or(0);
        groups.push(HueGroup {
            identity,
            collapsed: channels.collapsed,
            columns: columns
                .iter()
                .map(|column| rows.iter().map(|row| column[*row]).collect())
                .collect(),
            bandwidths: Vec::new(),
            color: theme.colors[color_index % theme.colors.len()],
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
    match (
        group_column,
        hue_column,
        group.identity.group.as_deref(),
        group.identity.hue.as_deref(),
        group.collapsed,
    ) {
        (Some(group_column), _, Some(value), _, true)
        | (Some(group_column), None, Some(value), None, false) => {
            format!("column {column:?} in graphical group {group_column:?}={value:?}")
        }
        (Some(group_column), Some(hue_column), Some(group_value), Some(hue_value), false) => format!(
            "column {column:?} in graphical group {group_column:?}={group_value:?} with hue {hue_column:?}={hue_value:?}"
        ),
        (None, Some(hue_column), None, Some(value), _) => {
            format!("column {column:?} in hue {hue_column:?}={value:?}")
        }
        _ => format!("column {column:?} in the ungrouped data"),
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
