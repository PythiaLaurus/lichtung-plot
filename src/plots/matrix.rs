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
use crate::layout::{self, Layout, PanelBox, PhysicalTextAlignment, TextBlock};
use crate::render::backend::{
    points_share_canonical_position, with_clip, with_layer, Backend, FillStyle, LineStyle, Point2D,
};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::stats::distribution::{gaussian_kde_with_source, strict_scott_bandwidth};
use crate::theme::ResolvedTheme;
use crate::Error;
use crate::{LegendPosition, Plot};

#[cfg(test)]
mod axis_tests;

/// One graphical group for the matrix plot.
#[derive(Debug)]
struct HueGroup {
    identity: crate::data::CollectivePair,
    collapsed: bool,
    /// Per-column data vectors (same order as `cols`).
    columns: Vec<Vec<f64>>,
    /// Strict Scott bandwidth per matrix column.
    bandwidths: Vec<f64>,
    color_index: usize,
}

#[derive(Debug)]
pub(crate) struct PreparedMatrixAxes {
    ranges: Vec<(f64, f64)>,
    labels: Vec<String>,
    groups: Vec<HueGroup>,
    diagonals: Vec<MatrixDensity>,
}

#[derive(Debug)]
struct MatrixDensity {
    positions: Vec<f64>,
    densities: Vec<Vec<f64>>,
    maximum: f64,
}

const KDE_POINTS: usize = 60;

#[derive(Debug)]
enum MatrixPrimitive {
    Scatter(Vec<Point2D>, crate::render::backend::PointStyle),
    Polygon(Vec<Point2D>, FillStyle),
}

impl MatrixPrimitive {
    fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        match self {
            Self::Scatter(points, style) => backend.draw_scatter(points, style),
            Self::Polygon(points, style) => backend.draw_polygon(points, style),
        }
    }

    fn geometry(
        &self,
        geometry: &mut crate::render::geometry::GeometrySnapshot,
        clip: PanelBox,
    ) -> Result<(), Error> {
        match self {
            Self::Scatter(points, style) => geometry.scatter(points, style, clip),
            Self::Polygon(points, style) => geometry.polygon(points, style, Some(clip)),
        }
    }
}

#[derive(Debug)]
struct MatrixCellPayload {
    clip: PanelBox,
    frame: [[Point2D; 2]; 2],
    marks: Vec<MatrixPrimitive>,
    axes: layout::PanelAxisPlan,
}

#[derive(Debug)]
pub(crate) struct MatrixPayload {
    cells: Vec<MatrixCellPayload>,
    frame_style: Option<LineStyle>,
    legend: Option<crate::decorations::legend::LegendDrawPlan>,
}

impl MatrixPayload {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        for cell in &self.cells {
            if let Some(style) = &self.frame_style {
                for line in &cell.frame {
                    backend.draw_line(line, style)?;
                }
            }
            with_clip(backend, cell.clip, |backend| {
                for mark in &cell.marks {
                    mark.draw(backend)?;
                }
                Ok(())
            })?;
            cell.axes.draw(backend)?;
        }
        if let Some(legend) = &self.legend {
            with_layer(backend, "legend", |backend| legend.draw(backend))?;
        }
        Ok(())
    }

    fn geometry(
        &self,
        theme: &ResolvedTheme,
    ) -> Result<crate::render::geometry::GeometrySnapshot, Error> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new(theme)?;
        for cell in &self.cells {
            if let Some(style) = &self.frame_style {
                for line in &cell.frame {
                    geometry.line(line, style, None)?;
                }
            }
            let clip = crate::render::backend::validate_clip_region(cell.clip)?.panel();
            for mark in &cell.marks {
                mark.geometry(&mut geometry, clip)?;
            }
            geometry.append(&cell.axes.geometry_snapshot(theme)?)?;
        }
        if let Some(legend) = &self.legend {
            legend.geometry(&mut geometry)?;
        }
        Ok(geometry)
    }
}

#[cfg(test)]
thread_local! {
    static SEMANTIC_STAGES: std::cell::Cell<[usize; 2]> = const { std::cell::Cell::new([0; 2]) };
    static SELECTIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static DIAGNOSTIC_CONTEXTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn prepare_axes(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<PreparedMatrixAxes, Error> {
    #[cfg(test)]
    crate::plots::shared_x::AXIS_PREPARATIONS.with(|count| count.set(count.get() + 1));
    let mut cols = Vec::new();
    data.reserve_source(&mut cols, chart.matrix_variable_names().len())?;
    cols.extend(chart.matrix_variable_names().iter().map(String::as_str));
    let labels = chart.resolve_axis_label_domain("matrix variables", cols.iter().copied(), data)?;
    let groups = build_groups(chart, &cols, data)?;
    let mut ranges = Vec::new();
    data.reserve_source(&mut ranges, cols.len())?;
    for (index, column) in cols.iter().enumerate() {
        for group in &groups {
            data.claim_source_visits(group.columns[index].len())?;
            data.claim_source_visits(group.columns[index].len())?;
        }
        ranges.push(matrix_support_range(
            &groups,
            index,
            column,
            chart.group_name(),
            chart.hue_name(),
        )?);
    }
    let diagonals = data.collect_source(cols.iter().enumerate().map(|(index, column)| {
        prepare_density(chart, &groups, index, column, ranges[index], data)
    }))?;
    Ok(PreparedMatrixAxes {
        ranges,
        labels,
        groups,
        diagonals,
    })
}

#[cfg(test)]
#[test]
fn prefont_variable_domains_and_labels_are_consumed_by_matrix_measurement() {
    let plot = Plot::matrix(vec![
        crate::NumericValues::new("first_value", [1.0, 2.0, 3.0, 4.0]),
        crate::NumericValues::new("second_value", [100.0, 300.0, 200.0, 500.0]),
    ])
    .erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let prepared = crate::plots::shared_x::prepare_plot(
        &plot,
        &crate::font::canonical::OperationGenerationResources::new(),
    )
    .unwrap()
    .attach(plot.clone())
    .unwrap();
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
    let inputs = prepared.prepared_matrix_axes.as_ref().unwrap();
    let preparations = crate::plots::shared_x::AXIS_PREPARATIONS.with(std::cell::Cell::get);
    let theme = crate::Theme::Publication.resolve();
    let panel = panel_box(&plot, &theme).unwrap();
    let measured = prepared
        .preflight_panel(crate::PlotKind::Matrix, panel, &theme)
        .unwrap();
    assert_eq!(inputs.labels, ["First value", "Second value"]);
    assert_eq!(inputs.ranges, measured.matrix_axes.as_ref().unwrap().ranges);
    assert_eq!(
        crate::plots::shared_x::AXIS_PREPARATIONS.with(std::cell::Cell::get),
        preparations
    );
    prepared.preflight().unwrap();
}

#[cfg(test)]
#[test]
fn matrix_dispatch_preserves_scoped_tick_fit_miss() {
    use crate::layout::fit::{FitKind, FitOutcome, FitScope};
    use crate::layout::AddressRole;
    let plot = Plot::matrix(vec![
        crate::NumericValues::new("x", [1e100, 2e100, 3e100]),
        crate::NumericValues::new("y", [0.0, 1.0, 2.0]),
    ])
    .erase();
    let mut theme = crate::Theme::Publication.resolve();
    theme.typography.tick_label.font_size = 48;
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let prepared = crate::plots::shared_x::prepare_plot(&plot, &resources).unwrap();
    let plot = prepared.attach(plot).unwrap();
    let scope = FitScope::root().child(AddressRole::CanvasCell, 2).unwrap();
    let outcome = plot
        .preflight_measurement(
            plot.kind(),
            PanelBox::new(80.0, 180.0, 60.0, 360.0),
            &theme,
            &scope,
        )
        .unwrap();
    let FitOutcome::Miss(failure) = outcome else {
        panic!("narrow matrix numeric source must miss")
    };
    assert_eq!(failure.reason.kind, FitKind::Tick);
    assert_eq!(
        failure.reason.address,
        scope
            .child(AddressRole::MatrixCell, 0)
            .unwrap()
            .child(AddressRole::AxisBottom, 0)
            .unwrap()
            .address
    );
    assert!(failure.message.contains("terminal midpoint"));
}

#[cfg(test)]
#[test]
fn tick_planner_failure_is_not_reported_as_small_matrix_cells() {
    let plot = Plot::matrix(vec![
        crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        crate::NumericValues::new("y", [4.0, 3.0, 2.0, 1.0]),
    ])
    .erase();
    let plot = crate::plots::shared_x::prepare_plot(
        &plot,
        &crate::font::canonical::OperationGenerationResources::new(),
    )
    .unwrap()
    .attach(plot)
    .unwrap();
    let mut theme = crate::Theme::Publication.resolve();
    theme.prepare_font_environment().unwrap();
    theme.typography.tick_label.font_size = 0;
    let context =
        PlotRenderContext::new(plot.data(), PanelBox::new(80.0, 600.0, 60.0, 440.0), &theme);
    let error = plot.measure_panel(plot.kind(), context).err().unwrap();
    assert!(matches!(error, crate::layout::fit::PlanError::Fatal(_)));
    let error = error.into_error();
    assert!(
        error.to_string().contains("font size must be positive"),
        "{error}"
    );
    assert!(!error.to_string().contains("within their cells"));
}

pub(crate) fn measure(
    chart: &Plot,
    context: PlotRenderContext<'_>,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    SELECTIONS.with(|count| count.set(count.get() + 1));
    let theme = context.theme;
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = context.data.with_axis_resources(&resources);
    let prepared = chart
        .prepared_matrix_axes
        .as_ref()
        .ok_or_else(|| Error::Render("matrix requires root-prepared semantics".into()))?;
    let n = prepared.labels.len();
    let display_cols = &prepared.labels;

    let groups = &prepared.groups;
    let legend = match matrix_legend(
        chart,
        groups,
        context.panel,
        theme,
        context.fit_scope(),
        data,
    )? {
        crate::layout::fit::FitOutcome::Measured(legend) => {
            legend.map(|legend| data.source_arc(legend)).transpose()?
        }
        crate::layout::fit::FitOutcome::Miss(reason) => {
            return Err(crate::layout::fit::PlanError::Fit(reason));
        }
    };

    // --- Layout geometry ---
    let w = chart.width() as f64;
    let insets = chart.legacy_root_axes().insets(theme, true);
    let outer = context.panel;
    let legend_reserve = legend.as_ref().map_or(0.0, |legend| {
        (legend_total_width(legend) + 32.0 - insets.right).max(0.0)
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
        let column_blocks = data.collect_source(display_cols.iter().map(|label| {
            layout::fit::measured(layout::fit::text(
                theme,
                label,
                &theme.typography.axis_title,
                candidate_cell_width.max(1.0),
                PhysicalTextAlignment::Center,
                "matrix column label",
                context.fit_scope(),
            ))
            .map(TextBlock::from_resolved)
            .map_err(crate::layout::fit::PlanError::into_error)
        }))?;
        let row_blocks = data.collect_source(display_cols.iter().map(|label| {
            layout::fit::measured(layout::fit::text(
                theme,
                label,
                &theme.typography.axis_title,
                candidate_cell_height.max(1.0),
                PhysicalTextAlignment::Center,
                "matrix row label",
                context.fit_scope(),
            ))
            .map(TextBlock::from_resolved)
            .map_err(crate::layout::fit::PlanError::into_error)
        }))?;
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
            )).into());
        }
        if (cell_w - candidate_cell_width).abs() <= f64::EPSILON
            && (cell_h - candidate_cell_height).abs() <= f64::EPSILON
        {
            resolved_layout = Some((grid_left, grid_bottom, grid_top, cell_w, cell_h));
            break;
        }
        candidate_cell_width = cell_w;
        candidate_cell_height = cell_h;
    }
    let Some((grid_left, grid_bottom, grid_top, cell_w, cell_h)) = resolved_layout else {
        return Err(Error::Render(
            "matrix label layout did not stabilize; increase chart size or shorten variable labels"
                .into(),
        )
        .into());
    };

    // --- Per-variable axis ranges ---
    // A pairplot is a coordinated matrix: each variable should use one
    // canonical range wherever it appears, not one range as x and another
    // as y. The range includes the diagonal KDE evaluation extent so the
    // diagonal and off-diagonal cells stay visually aligned.
    let prepared_ranges = data.collect_source(prepared.ranges.iter().copied().map(Ok))?;

    // --- Styles ---
    let frame_style = theme
        .axes
        .frame
        .as_ref()
        .map(|appearance| appearance.stroke.backend_style());
    // Matrix owns its two-spine cell topology until T5e. Common plans own
    // ticks and outer text, without introducing a four-sided frame or grid.
    let mut axis_theme = theme.clone();
    axis_theme.axes.frame = None;
    axis_theme.axes.grid = None;

    // X selection depends only on a column's range and X coordinates; Y
    // selection depends only on a row's range and Y coordinates. Retain the
    // complete sources across capture; outer cells alone paint their labels.
    let environment = theme.prepared_font_environment()?;
    let cache_count = n;
    let cache_records = cache_count
        .checked_mul(2)
        .ok_or_else(|| Error::Render("matrix tick cache count overflow".into()))?;
    let cache_bytes = cache_records
        .checked_mul(size_of::<Option<layout::NumericTickPlan>>())
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
        .try_reserve_exact(cache_count)
        .map_err(|_| Error::Render("cannot reserve matrix X tick cache".into()))?;
    y_ticks
        .try_reserve_exact(cache_count)
        .map_err(|_| Error::Render("cannot reserve matrix Y tick cache".into()))?;
    x_ticks.resize_with(cache_count, || None);
    y_ticks.resize_with(cache_count, || None);
    let mut axes = MatrixAxisPlan {
        ranges: prepared_ranges,
        cell_span: (cell_w, cell_h),
        x_ticks,
        y_ticks,
    };
    let variable_ranges = &axes.ranges;
    let (cell_w, cell_h) = axes.cell_span;

    let mut cells = Vec::new();
    data.reserve_source(
        &mut cells,
        n.checked_mul(n)
            .ok_or_else(|| Error::Render("matrix cell count overflow".into()))?,
    )?;
    // Preserve the legacy cell order and local packing until T5e.
    for row in 0..n {
        for col in 0..n {
            let x0 = grid_left + col as f64 * (cell_w + gap);
            let x1 = x0 + cell_w;
            // row 0 = top visually → highest y in math coords
            let y0 = grid_bottom + (n - 1 - row) as f64 * (cell_h + gap);
            let y1 = y0 + cell_h;
            let cell_panel = PanelBox::new(x0, x1, y0, y1);
            let projection = layout::Projection::new(
                variable_ranges[col],
                variable_ranges[row],
                cell_panel.as_tuple(),
            );
            let scope = context
                .fit_scope()
                .child(layout::AddressRole::MatrixCell, row * n + col)?;
            let axis = |ticks: &layout::NumericTickPlan, label: Option<&str>| {
                Ok::<_, Error>(layout::Axis {
                    source: ticks.source,
                    ticks: data.collect_source(ticks.ticks.iter().map(|tick| {
                        Ok(layout::AxisTick {
                            position: tick.position,
                            label: if label.is_some() {
                                data.copy_source_string(&tick.label)?
                            } else {
                                String::new()
                            },
                        })
                    }))?,
                    label: data.copy_source_string(label.unwrap_or_default())?,
                    explicit_ticks: false,
                    assigned_tick_width: None,
                })
            };
            let x_axis = axis(
                matrix_ticks(
                    &mut axes.x_ticks[col],
                    variable_ranges[col],
                    &projection,
                    layout::AxisSide::Bottom,
                    &axis_theme,
                    &scope,
                )?,
                (row == n - 1).then_some(display_cols[col].as_str()),
            )?;
            let y_axis = axis(
                matrix_ticks(
                    &mut axes.y_ticks[row],
                    variable_ranges[row],
                    &projection,
                    layout::AxisSide::Left,
                    &axis_theme,
                    &scope,
                )?,
                (col == 0).then_some(display_cols[row].as_str()),
            )?;
            let cell_layout = Layout {
                projection,
                x_axis,
                y_axis,
            };
            let mut cell = MatrixCellPayload {
                clip: cell_panel,
                frame: [
                    [Point2D::new(x0, y0), Point2D::new(x1, y0)],
                    [Point2D::new(x0, y0), Point2D::new(x0, y1)],
                ],
                marks: Vec::new(),
                axes: cell_layout.measure_axes(
                    &axis_theme,
                    &scope,
                    chart.axis_semantic_inputs([crate::spec::AxisDomain::Continuous; 2]),
                )?,
            };
            data.reserve_source(&mut cell.marks, groups.len())?;

            if row == col {
                select_diagonal_kde(
                    theme,
                    &mut cell.marks,
                    groups,
                    &prepared.diagonals[row],
                    &cell_layout.projection,
                    data,
                )?;
            } else {
                for group in groups {
                    let xs = &group.columns[col];
                    let ys = &group.columns[row];
                    let ps =
                        chart.point_style(theme.colors[group.color_index % theme.colors.len()], 2);
                    let pts = data.collect_source(xs.iter().zip(ys.iter()).map(|(&xi, &yi)| {
                        data.claim_source_visits(1)?;
                        let (px, py) = cell_layout.projection.project(xi, yi);
                        Ok(Point2D::new(px, py))
                    }))?;
                    cell.marks.push(MatrixPrimitive::Scatter(pts, ps));
                }
            }

            cells.push(cell);
        }
    }

    // --- Legend (when hue is active) ---
    let legend_payload = if let Some(legend) = &legend {
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
        Some(legend.select(lx, ly, data)?)
    } else {
        None
    };

    // Return a dummy layout — matrix is self-decorated.
    let layout = Layout::self_managed(chart.width(), chart.height());

    let mut result = PlotRenderResult::new(layout, Vec::new()).with_self_managed_decorations();
    let payload = MatrixPayload {
        cells,
        frame_style,
        legend: legend_payload,
    };
    let geometry = payload.geometry(theme)?;
    result.marks = Some(data.source_arc(crate::layout::interior::DeferredMarkPlan {
        payload: crate::layout::interior::MarkPayload::Matrix(payload),
        geometry,
    })?);
    result.self_managed_legend = legend;
    result.matrix_axes = Some(data.source_arc(axes)?);
    Ok(result)
}

#[derive(Debug)]
pub(crate) struct MatrixAxisPlan {
    ranges: Vec<(f64, f64)>,
    cell_span: (f64, f64),
    x_ticks: Vec<Option<layout::NumericTickPlan>>,
    y_ticks: Vec<Option<layout::NumericTickPlan>>,
}

fn matrix_ticks<'a>(
    cached: &'a mut Option<layout::NumericTickPlan>,
    range: (f64, f64),
    projection: &layout::Projection,
    side: layout::AxisSide,
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<&'a layout::NumericTickPlan, crate::layout::fit::PlanError> {
    let environment = theme.prepared_font_environment()?;
    let admission = environment
        .claim_measurement_call()
        .map_err(|error| Error::Render(format!("matrix tick call admission: {error:?}")))?;
    environment
        .claim_processing_visits(1)
        .map_err(|error| Error::Render(format!("matrix tick lookup: {error:?}")))?;
    if cached.is_none() {
        let plan = layout::NumericTickPlan::measure_in_call(
            range, None, false, projection, side, theme, false, scope, &admission,
        )?;
        environment
            .claim_cache_entries(1)
            .map_err(|error| Error::Render(format!("matrix tick cache admission: {error:?}")))?;
        *cached = Some(plan);
    }
    cached
        .as_ref()
        .ok_or_else(|| Error::Render("matrix tick selection is missing".into()).into())
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
        let first = matrix_ticks(
            &mut cached,
            range,
            &original,
            side,
            &theme,
            &crate::layout::fit::ROOT_SCOPE,
        )
        .unwrap();
        let expected = layout::numeric_tick_plan(
            range,
            None,
            false,
            &translated,
            side,
            &theme,
            false,
            &crate::layout::fit::ROOT_SCOPE,
        )
        .unwrap()
        .ticks;
        assert_eq!(first.ticks, expected);
        let before = resources.usage().unwrap();
        let bytes = environment.resource_usage().unwrap().font_bytes;
        let reused = matrix_ticks(
            &mut cached,
            range,
            &translated,
            side,
            &theme,
            &crate::layout::fit::ROOT_SCOPE,
        )
        .unwrap();
        assert_eq!(reused.ticks, expected);
        assert_eq!(resources.usage().unwrap(), (before.0, before.1 + 1));
        assert_eq!(environment.resource_usage().unwrap().font_bytes, bytes);
        assert!(cached.as_ref().unwrap().source.cap > 1);
    }
}

#[test]
fn matrix_tick_source_refusal_leaves_no_retained_cache_entry() {
    let theme = crate::Theme::Publication.resolve();
    let environment = theme.prepared_font_environment().unwrap();
    let used = environment.resource_usage().unwrap().font_bytes;
    environment
        .claim_unicode_bytes(u64::try_from(536_870_912 - used).unwrap())
        .unwrap();
    let projection = layout::Projection::new((0.0, 1.0), (0.0, 1.0), (0.0, 100.0, 0.0, 100.0));
    let mut cached = None;
    let error = matrix_ticks(
        &mut cached,
        (0.0, 1.0),
        &projection,
        layout::AxisSide::Bottom,
        &theme,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap_err();
    assert!(matches!(error, crate::layout::fit::PlanError::Fatal(_)));
    assert!(error.into_error().to_string().contains("system-font bytes"));
    assert!(cached.is_none());
}

#[cfg(test)]
fn panel_box(chart: &Plot, theme: &ResolvedTheme) -> Result<crate::layout::PanelBox, Error> {
    Ok(chart.measured_canvas_layout(theme)?.panel_box)
}

fn matrix_legend(
    chart: &Plot,
    groups: &[HueGroup],
    available: crate::layout::PanelBox,
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
    data: crate::data::PlotData<'_>,
) -> Result<crate::layout::fit::FitOutcome<Option<Legend>>, Error> {
    use crate::layout::fit::FitOutcome;
    if !groups.iter().any(|group| group.identity.hue.is_some())
        || chart.legend_policy_spec() == crate::LegendPolicy::Hidden
    {
        return Ok(FitOutcome::Measured(None));
    }
    let mut entries = Vec::new();
    data.reserve_source(&mut entries, groups.len())?;
    for group in groups.iter().filter(|group| group.identity.hue.is_some()) {
        entries.push(LegendEntry::point_fill(
            chart.collective_legend_identity(&group.identity, data)?,
            chart.point_style(theme.colors[group.color_index % theme.colors.len()], 2),
            FillStyle {
                color: theme.colors[group.color_index % theme.colors.len()],
                stroke: theme.colors[group.color_index % theme.colors.len()],
                stroke_width: 1.0,
                opacity: 0.40,
            },
        ));
    }
    crate::decorations::legend::resolve_entries(
        chart,
        &mut entries,
        &theme.prepared_font_environment()?.generation_resources(),
    )?;
    let title = crate::decorations::legend::resolved_title(
        &entries,
        &theme.prepared_font_environment()?.generation_resources(),
    )?;
    let entries = Legend::admit_entries(entries, theme)?;
    let scope = scope.child(crate::layout::AddressRole::InsideGuide, 0)?;
    match Legend::measure_assigned(
        &title,
        entries,
        (available.width(), available.height()),
        false,
        false,
        theme,
        &mut false,
        &scope,
    )? {
        FitOutcome::Measured(legend) => Ok(FitOutcome::Measured(Some(legend))),
        FitOutcome::Miss(reason) => Ok(FitOutcome::Miss(reason)),
    }
}

fn legend_total_width(legend: &Legend) -> f64 {
    legend.compute_box_width()
}

fn prepare_density(
    chart: &Plot,
    groups: &[HueGroup],
    col_idx: usize,
    column: &str,
    range: (f64, f64),
    data: crate::data::PlotData<'_>,
) -> Result<MatrixDensity, Error> {
    #[cfg(test)]
    SEMANTIC_STAGES.with(|counts| {
        let [groups, densities] = counts.get();
        counts.set([groups, densities + 1]);
    });
    let (d_lo, d_hi) = range;
    let grid_pts = data.collect_source((0..KDE_POINTS).map(|i| {
        data.claim_source_visits(1)?;
        Ok(d_lo + (d_hi - d_lo) * (i as f64 / (KDE_POINTS - 1) as f64))
    }))?;

    // Total sample count across all groups for common_norm scaling
    // (matches seaborn pairplot default: area proportional to group size).
    data.claim_source_visits(groups.len())?;
    let n_total: f64 = groups.iter().map(|g| g.columns[col_idx].len() as f64).sum();

    // Compute KDE for each group, scaled by group proportion (common_norm).
    let densities_per_group = data.collect_source(groups.iter().map(|g| {
        let mut weighted = gaussian_kde_with_source(
            &g.columns[col_idx],
            &grid_pts,
            g.bandwidths[col_idx],
            Some(data),
        )
        .map_err(|error| {
            let context = matrix_group_context(column, chart.group_name(), chart.hue_name(), g);
            Error::Data(format!("matrix: {context}: {error}"))
        })?;
        #[allow(clippy::cast_precision_loss)]
        let weight = g.columns[col_idx].len() as f64 / n_total;
        data.claim_source_visits(weighted.len())?;
        for density in &mut weighted {
            *density *= weight;
        }
        data.claim_source_visits(weighted.len())?;
        if weighted.iter().any(|density| !density.is_finite()) {
            let context = matrix_group_context(column, chart.group_name(), chart.hue_name(), g);
            return Err(Error::Data(format!(
                "matrix: {context} produced non-finite weighted KDE density"
            )));
        }
        Ok(weighted)
    }))?;

    // Shared max across all weighted densities.
    for densities in &densities_per_group {
        data.claim_source_visits(densities.len())?;
    }
    let d_max = densities_per_group
        .iter()
        .flat_map(|d| d.iter())
        .fold(0.0_f64, |a, &v| a.max(v));
    if !d_max.is_finite() || d_max <= 0.0 {
        return Err(Error::Data(format!(
            "matrix: column {column:?} produced no positive finite KDE density"
        )));
    }

    Ok(MatrixDensity {
        positions: grid_pts,
        densities: densities_per_group,
        maximum: d_max,
    })
}

/// Project root-prepared KDE curves into admitted cell operands.
fn select_diagonal_kde(
    theme: &ResolvedTheme,
    marks: &mut Vec<MatrixPrimitive>,
    groups: &[HueGroup],
    density: &MatrixDensity,
    projection: &layout::Projection,
    data: crate::data::PlotData<'_>,
) -> Result<(), Error> {
    let panel = projection.panel_box();
    let projection = layout::Projection::new(
        projection.x_range(),
        (0.0, density.maximum),
        (
            panel.x0,
            panel.x1,
            panel.y0,
            panel.y0 + panel.height() * 0.85,
        ),
    );
    let (x0, y0) = projection.project(projection.x_range().0, 0.0);
    let (x1, _) = projection.project(projection.x_range().1, 0.0);

    for (gi, densities) in density.densities.iter().enumerate() {
        let color = theme.colors[groups[gi].color_index % theme.colors.len()];
        let mut poly = Vec::new();
        data.reserve_source(&mut poly, KDE_POINTS + 2)?;
        poly.push(Point2D::new(x0, y0));
        for (i, (&x, &d)) in density.positions.iter().zip(densities).enumerate() {
            let (px, py) = projection.project(x, d);
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
        marks.push(MatrixPrimitive::Polygon(poly, fill));
    }
    Ok(())
}

/// Build graphical groups while preserving hue as color-only metadata.
fn build_groups(
    chart: &Plot,
    cols: &[&str],
    data: crate::data::PlotData<'_>,
) -> Result<Vec<HueGroup>, Error> {
    #[cfg(test)]
    SEMANTIC_STAGES.with(|counts| {
        let [groups, densities] = counts.get();
        counts.set([groups + 1, densities]);
    });
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
    let hue_levels = data.collect_source(
        chart
            .global_hue_levels()
            .unwrap_or(&channels.hue_levels)
            .iter()
            .map(|level| data.copy_source_string(level)),
    )?;
    let identities = chart
        .global_collective_pairs()
        .unwrap_or(&channels.identities);
    let mut groups = Vec::new();
    data.reserve_source(&mut groups, identities.len())?;
    let mut rows = Vec::new();
    data.reserve_source(&mut rows, channels.rows.len())?;
    for identity in identities {
        rows.clear();
        data.claim_source_visits(channels.rows.len())?;
        rows.extend(
            channels
                .rows
                .iter()
                .enumerate()
                .filter_map(|(row, row_identity)| (row_identity == identity).then_some(row)),
        );
        if rows.is_empty() {
            continue;
        }
        data.claim_source_visits(hue_levels.len())?;
        let color_index = identity
            .hue
            .as_deref()
            .and_then(|hue| hue_levels.iter().position(|level| level == hue))
            .unwrap_or(0);
        groups.push(HueGroup {
            identity: data.copy_source_pair(identity)?,
            collapsed: channels.collapsed,
            columns: data.collect_source(
                columns
                    .iter()
                    .map(|column| data.collect_source(rows.iter().map(|row| Ok(column[*row])))),
            )?,
            bandwidths: Vec::new(),
            color_index,
        });
    }
    if groups.is_empty() {
        return Err(Error::Data("matrix: no valid finite rows".into()));
    }
    validate_group_kdes(
        &mut groups,
        cols,
        chart.group_name(),
        chart.hue_name(),
        data,
    )?;
    Ok(groups)
}

fn validate_group_kdes(
    groups: &mut [HueGroup],
    columns: &[&str],
    group_column: Option<&str>,
    hue_column: Option<&str>,
    data: crate::data::PlotData<'_>,
) -> Result<(), Error> {
    for group in groups {
        group.bandwidths =
            data.collect_source(group.columns.iter().zip(columns).map(|(samples, column)| {
                for _ in 0..5 {
                    data.claim_source_visits(samples.len())?;
                }
                strict_scott_bandwidth(samples).map_err(|reason| {
                    Error::Data(format!(
                        "matrix: {} {reason}",
                        matrix_group_context(column, group_column, hue_column, group)
                    ))
                })
            }))?;
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
    #[cfg(test)]
    DIAGNOSTIC_CONTEXTS.with(|calls| calls.set(calls.get() + 1));
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
    let columns = data.collect_source(
        cols.iter()
            .map(|column| crate::data::strict_finite_f64(data, column, "matrix")),
    )?;

    data.claim_source_visits(columns.len())?;
    if columns.iter().any(|column| column.len() != data.height()) {
        return Err(Error::Data("matrix: column length mismatch".into()));
    }
    Ok(columns)
}

#[cfg(test)]
mod semantic_tests {
    use super::*;
    use crate::font::canonical::OperationGenerationResources;
    use crate::{CategoryValues, NumericValues};

    #[test]
    fn successful_kde_budget_does_not_depend_on_diagnostic_names() {
        let source = source();
        DIAGNOSTIC_CONTEXTS.with(|calls| calls.set(0));
        let mut prepared = prepare_axes(&source, source.data()).unwrap();
        assert_eq!(DIAGNOSTIC_CONTEXTS.with(std::cell::Cell::get), 0);
        let run = |name: &str, resources: &OperationGenerationResources| {
            prepare_density(
                &source,
                &prepared.groups,
                0,
                name,
                prepared.ranges[0],
                source.data().with_axis_resources(resources),
            )
        };
        let resources = OperationGenerationResources::new();
        run("first_value", &resources).unwrap();
        let visits = resources.usage().unwrap().1;
        let bytes = resources.byte_usage().unwrap();
        let exact = OperationGenerationResources::with_test_limits(visits, bytes);
        run(&"long diagnostic name".repeat(1000), &exact).unwrap();
        assert_eq!(exact.usage().unwrap().1, visits);
        assert_eq!(exact.byte_usage().unwrap(), bytes);
        assert_eq!(DIAGNOSTIC_CONTEXTS.with(std::cell::Cell::get), 0);
        prepared.groups[0].bandwidths[0] = 0.0;
        let error = prepare_density(
            &source,
            &prepared.groups,
            0,
            "first_value",
            prepared.ranges[0],
            source.data(),
        )
        .unwrap_err();
        assert_eq!(DIAGNOSTIC_CONTEXTS.with(std::cell::Cell::get), 1);
        assert!(error
            .to_string()
            .contains("column \"first_value\" in hue \"hue\"=\"b\""));
        assert!(error.to_string().contains("positive finite bandwidth"));
    }

    #[test]
    fn many_hue_matrix_legend_uses_the_operation_ledger_end_to_end() {
        let source = Plot::matrix(vec![
            NumericValues::new("x", (0..24).map(f64::from)),
            NumericValues::new("y", (0..24).map(|index| f64::from(index * index))),
        ])
        .hue(
            CategoryValues::new(
                "measurement_condition",
                (0..24).map(|index| format!("long_named_measurement_condition_{}", index / 2)),
            )
            .unwrap(),
        )
        .size(1600, 1000)
        .erase();
        let prepared = prepare_axes(&source, source.data()).unwrap();
        let run = |limits: Option<(u128, u64)>| {
            let theme = crate::Theme::Publication.resolve();
            let resources = theme
                .prepared_font_environment()
                .unwrap()
                .generation_resources();
            if let Some((visits, bytes)) = limits {
                resources.set_test_limits(visits, bytes);
            }
            let result = matrix_legend(
                &source,
                &prepared.groups,
                PanelBox::new(0.0, 1600.0, 0.0, 1000.0),
                &theme,
                &crate::layout::fit::FitScope::root(),
                source.data().with_axis_resources(&resources),
            );
            (resources, result)
        };
        let (resources, legend) = run(None);
        let crate::layout::fit::FitOutcome::Measured(Some(legend)) = legend.unwrap() else {
            panic!("prepared matrix guide should fit its assigned panel")
        };
        assert_eq!(legend.text_plans().count(), 13);
        let visits = resources.usage().unwrap().1;
        let bytes = resources.byte_usage().unwrap();
        for limits in [
            (visits - 1, bytes),
            (visits, bytes - 1),
            (0, bytes),
            (visits, 0),
        ] {
            assert!(run(Some(limits)).1.is_err());
        }
        run(Some((visits, bytes))).1.unwrap();
    }

    fn source() -> Plot {
        Plot::matrix(vec![
            NumericValues::new("first_value", [1.0, 1.2, 1.4, 3.0, 3.2, 3.4]),
            NumericValues::new("second_value", [2.0, 2.1, 2.3, 4.0, 4.2, 4.3]),
        ])
        .hue(CategoryValues::new("hue", ["b", "b", "b", "a", "a", "a"]).unwrap())
        .size(800, 600)
        .erase()
    }

    #[test]
    fn root_semantics_and_density_storage_have_exact_pre_font_admission() {
        let source = source();
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        SEMANTIC_STAGES.with(|counts| counts.set([0; 2]));
        let resources = OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        assert_eq!(SEMANTIC_STAGES.with(std::cell::Cell::get), [1, 2]);
        let prepared = inputs.matrix.as_ref().unwrap();
        assert_eq!(prepared.groups.len(), 2);
        assert_eq!(prepared.labels, ["First value", "Second value"]);
        assert_eq!(prepared.groups[0].identity.hue.as_deref(), Some("b"));
        assert_eq!(prepared.groups[1].identity.hue.as_deref(), Some("a"));
        assert_eq!(prepared.groups[0].columns[0], [1.0, 1.2, 1.4]);
        assert_eq!(prepared.groups[1].columns[1], [4.0, 4.2, 4.3]);
        let visits = resources.usage().unwrap().1;
        let bytes = resources.byte_usage().unwrap();
        for (visits, bytes) in [(visits - 1, bytes), (visits, bytes - 1)] {
            assert!(crate::plots::shared_x::prepare_plot(
                &source,
                &OperationGenerationResources::with_test_limits(visits, bytes),
            )
            .is_err());
        }
        crate::plots::shared_x::prepare_plot(
            &source,
            &OperationGenerationResources::with_test_limits(visits, bytes),
        )
        .unwrap();
        let attached = inputs.attach(source).unwrap();
        assert!(std::sync::Arc::ptr_eq(
            attached.prepared_matrix_axes.as_ref().unwrap(),
            prepared,
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn retained_densities_match_legacy_grid_and_common_normalization() {
        let source = source();
        let prepared = prepare_axes(&source, source.data()).unwrap();
        for (index, &(low, high)) in prepared.ranges.iter().enumerate() {
            let grid = (0..60)
                .map(|i| low + (high - low) * (i as f64 / 59.0))
                .collect::<Vec<_>>();
            let total = prepared
                .groups
                .iter()
                .map(|group| group.columns[index].len() as f64)
                .sum::<f64>();
            let expected = prepared
                .groups
                .iter()
                .map(|group| {
                    let weight = group.columns[index].len() as f64 / total;
                    gaussian_kde_with_source(
                        &group.columns[index],
                        &grid,
                        group.bandwidths[index],
                        None,
                    )
                    .unwrap()
                    .into_iter()
                    .map(|density| density * weight)
                    .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            assert_eq!(prepared.diagonals[index].densities, expected);
            assert_eq!(
                prepared.diagonals[index].maximum.to_bits(),
                expected
                    .iter()
                    .flatten()
                    .copied()
                    .fold(0.0_f64, f64::max)
                    .to_bits()
            );
        }
    }

    #[test]
    fn repeated_matrix_measurements_do_not_rebuild_groups_or_statistics() {
        let source = source();
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let source = inputs.attach(source).unwrap();
        let stages = SEMANTIC_STAGES.with(std::cell::Cell::get);
        let theme = crate::Theme::Publication.resolve();
        for panel in [
            PanelBox::new(80.0, 700.0, 60.0, 540.0),
            PanelBox::new(80.25, 700.75, 60.5, 500.25),
        ] {
            source
                .preflight_panel(crate::PlotKind::Matrix, panel, &theme)
                .unwrap();
        }
        assert_eq!(SEMANTIC_STAGES.with(std::cell::Cell::get), stages);
    }

    #[test]
    fn selected_matrix_cells_preserve_geometry_and_poisoned_capture_does_no_layout_work() {
        for preset in [crate::Theme::Publication, crate::Theme::Science] {
            let source = source();
            let inputs =
                crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                    .unwrap();
            let source = inputs.attach(source).unwrap();
            let theme = preset.resolve();
            let panel = PanelBox::new(80.25, 700.75, 60.5, 500.25);
            let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
            let begins =
                crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get);
            let stages = SEMANTIC_STAGES.with(std::cell::Cell::get);
            let selected = source
                .measure_selected_interior(PlotRenderContext::new(source.data(), panel, &theme))
                .unwrap();
            assert_eq!(
                reads,
                crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get)
            );
            assert_eq!(
                begins,
                crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get)
            );
            assert_eq!(stages, SEMANTIC_STAGES.with(std::cell::Cell::get));
            let marks = selected.marks.as_ref().unwrap();
            let crate::layout::interior::MarkPayload::Matrix(payload) = &marks.payload else {
                panic!("wrong payload")
            };
            assert_eq!(payload.cells.len(), 4);
            assert!(payload.legend.is_some());
            let axes = selected.matrix_axes.as_ref().unwrap();
            for (index, cell) in payload.cells.iter().enumerate() {
                assert_eq!(cell.marks.len(), 2);
                assert!((cell.clip.width() - axes.cell_span.0).abs() < 1e-10);
                assert!((cell.clip.height() - axes.cell_span.1).abs() < 1e-10);
                for mark in &cell.marks {
                    if index == 0 || index == 3 {
                        assert!(
                            matches!(mark, MatrixPrimitive::Polygon(points, _) if points.len() == 62)
                        );
                    } else {
                        assert!(
                            matches!(mark, MatrixPrimitive::Scatter(points, _) if points.len() == 3)
                        );
                    }
                }
            }
            let mut expected = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
            expected.begin(800, 600).unwrap();
            marks.draw(&mut expected).unwrap();
            expected.end().unwrap();
            assert_eq!(marks.geometry.bounds(), expected.bounds());
            let poison = Plot::matrix(vec![NumericValues::new("poison", [f64::NAN])]).erase();
            let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
            let projections = crate::layout::Projection::test_projection_calls();
            let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
            let selections = SELECTIONS.with(std::cell::Cell::get);
            crate::orchestrator::measurements::start();
            let mut captured = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
            captured.begin(800, 600).unwrap();
            poison
                .render_panel(
                    crate::PlotKind::Matrix,
                    PlotRenderContext::new(
                        poison.data(),
                        PanelBox::new(-10.0, -1.0, -10.0, -1.0),
                        &theme,
                    )
                    .with_retained_body(&selected),
                    &mut captured,
                )
                .unwrap();
            captured.end().unwrap();
            let work = crate::orchestrator::measurements::finish();
            for counter in [
                crate::orchestrator::measurements::Counter::TickSource,
                crate::orchestrator::measurements::Counter::TickDistinctList,
                crate::orchestrator::measurements::Counter::TickMetricLookup,
                crate::orchestrator::measurements::Counter::MatrixLayoutPass,
            ] {
                assert_eq!(work.counters[counter as usize], 0);
            }
            assert_eq!(captured.bounds(), expected.bounds());
            assert_eq!(
                reads,
                crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get)
            );
            assert_eq!(
                projections,
                crate::layout::Projection::test_projection_calls()
            );
            assert_eq!(
                shapes,
                crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get)
            );
            assert_eq!(stages, SEMANTIC_STAGES.with(std::cell::Cell::get));
            assert_eq!(selections, SELECTIONS.with(std::cell::Cell::get));
        }
    }

    #[test]
    fn selected_matrix_storage_refuses_at_exact_operation_limits() {
        let source = source();
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let source = inputs.attach(source).unwrap();
        let run = |limits: Option<(u128, u64)>| {
            let theme = crate::Theme::Publication.resolve();
            let resources = theme
                .prepared_font_environment()
                .unwrap()
                .generation_resources();
            if let Some((visits, bytes)) = limits {
                resources.set_test_limits(visits, bytes);
            }
            let result = source.measure_selected_interior(PlotRenderContext::new(
                source.data(),
                PanelBox::new(80.0, 700.0, 60.0, 500.0),
                &theme,
            ));
            (resources, result)
        };
        let (resources, selected) = run(None);
        selected.unwrap();
        let visits = resources.usage().unwrap().1;
        let bytes = resources.byte_usage().unwrap();
        for (visits, bytes) in [
            (visits - 1, bytes),
            (visits, bytes - 1),
            (0, bytes),
            (visits, 0),
        ] {
            assert!(run(Some((visits, bytes))).1.is_err());
        }
        run(Some((visits, bytes))).1.unwrap();
    }

    #[test]
    fn matrix_output_modes_reuse_one_selected_scene_and_identical_clips() {
        let directory = tempfile::tempdir().unwrap();
        let first = directory.path().join("first.svg");
        let second = directory.path().join("second.svg");
        let outline = directory.path().join("outline.svg");
        let png = directory.path().join("matrix.png");
        SEMANTIC_STAGES.with(|counts| counts.set([0; 2]));
        crate::scene::SCENE_CONSTRUCTION_CALLS.with(|calls| calls.set(0));
        let begins = crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get);
        let output = source().svg_preserve_text(&first).unwrap();
        assert_eq!(
            begins,
            crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get)
        );
        assert_eq!(SEMANTIC_STAGES.with(std::cell::Cell::get), [1, 2]);
        let selections = SELECTIONS.with(std::cell::Cell::get);
        let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
        let projections = crate::layout::Projection::test_projection_calls();
        let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
        output
            .svg(&outline)
            .unwrap()
            .png_preview(&png)
            .unwrap()
            .svg_preserve_text(&second)
            .unwrap();
        assert_eq!(SEMANTIC_STAGES.with(std::cell::Cell::get), [1, 2]);
        assert_eq!(SELECTIONS.with(std::cell::Cell::get), selections);
        assert_eq!(
            crate::scene::SCENE_CONSTRUCTION_CALLS.with(std::cell::Cell::get),
            1
        );
        assert_eq!(
            reads,
            crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get)
        );
        assert_eq!(
            projections,
            crate::layout::Projection::test_projection_calls()
        );
        assert_eq!(
            shapes,
            crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get)
        );
        let first = std::fs::read_to_string(first).unwrap();
        assert_eq!(first, std::fs::read_to_string(second).unwrap());
        let outline = std::fs::read_to_string(outline).unwrap();
        let preserved = roxmltree::Document::parse(&first).unwrap();
        let outlined = roxmltree::Document::parse(&outline).unwrap();
        assert_eq!(
            preserved.root_element().attribute("viewBox"),
            outlined.root_element().attribute("viewBox")
        );
        let clips = |document: &roxmltree::Document<'_>| {
            document
                .descendants()
                .filter(|node| {
                    node.has_tag_name("rect")
                        && node
                            .parent()
                            .is_some_and(|parent| parent.has_tag_name("clipPath"))
                })
                .map(|node| {
                    ["x", "y", "width", "height"]
                        .map(|name| node.attribute(name).unwrap().to_owned())
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(clips(&preserved).len(), 4);
        assert_eq!(clips(&preserved), clips(&outlined));
        assert!(!outline.contains("<text"));
        assert!(std::fs::read(png)
            .unwrap()
            .starts_with(b"\x89PNG\r\n\x1a\n"));
    }
}
