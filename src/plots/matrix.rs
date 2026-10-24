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

use crate::alpha::{
    marks::{
        fill_ink_outset, fill_style, point_ink, push_point, require_aligned_points,
        ObservationAlpha,
    },
    AlphaTarget, AlphaUnitKind, ResolvedAlpha,
};
use crate::decorations::legend::source::{FillRole, PointRole};
use crate::decorations::legend::{GuideSourceKey, Legend};
use crate::layout::{self, Layout, PanelBox};
use crate::render::backend::{
    points_share_canonical_position, with_clip, with_layer, Backend, FillStyle, LineStyle, Point2D,
};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::stats::distribution::{gaussian_kde, strict_scott_bandwidth};
use crate::theme::{AccentColors, CategoryColors, ResolvedTheme};
use crate::Error;
use crate::{LegendPosition, Plot};

/// One graphical group for the matrix plot.
#[derive(Debug)]
struct HueGroup {
    identity: crate::data::CollectivePair,
    collapsed: bool,
    /// Per-column data vectors (same order as `cols`).
    columns: Vec<Vec<f64>>,
    alphas: Vec<ObservationAlpha>,
    fill_alpha: Option<ResolvedAlpha>,
    /// Strict Scott bandwidth per matrix column.
    bandwidths: Vec<f64>,
}

#[derive(Debug)]
pub(crate) struct PreparedMatrixAxes {
    ranges: Vec<(f64, f64)>,
    labels: Vec<String>,
    groups: Vec<HueGroup>,
    hues: Vec<String>,
    diagonals: Vec<MatrixDensity>,
}

impl PreparedMatrixAxes {
    pub(crate) fn alpha_guide(
        &self,
        chart: &Plot,
        theme: &ResolvedTheme,
        guide: &mut crate::decorations::legend::alpha::AlphaGuideBuilder<'_>,
    ) -> crate::Result<()> {
        use crate::decorations::legend::alpha::{AlphaGlyphRole, AlphaUnit};
        let point = chart.point_style(
            theme,
            (0, 0, 0),
            theme.marks.get_points().get_radii().get_matrix(),
        );
        let fill = theme.marks.get_fills().get_densities().get_matrix_density();
        for group in &self.groups {
            if self.labels.len() > 1 {
                for &alpha in &group.alphas {
                    guide.point(AlphaGlyphRole::Primary, alpha, point.clone(), false)?;
                }
            }
            guide.fill(
                AlphaGlyphRole::Fill,
                AlphaUnitKind::Series,
                AlphaUnit::series(chart, Some(&group.identity))?,
                group.fill_alpha,
                fill.backend_style((0, 0, 0)),
                fill.get_outline()
                    .is_some_and(|outline| outline.mapped_width().is_some()),
            )?;
        }
        Ok(())
    }

    fn density_ink(&self, theme: &ResolvedTheme) -> Result<(f64, f64), Error> {
        fill_ink_outset(
            &theme
                .marks
                .get_fills()
                .get_densities()
                .get_matrix_density()
                .backend_style((0, 0, 0)),
            self.groups.iter().map(|group| group.fill_alpha),
        )
    }

    fn point_dimensions(&self, chart: &Plot, theme: &ResolvedTheme) -> Result<(f64, f64), Error> {
        let style = chart.point_style(
            theme,
            (0, 0, 0),
            theme.marks.get_points().get_radii().get_matrix(),
        );
        Ok(point_ink(
            &style,
            self.groups
                .iter()
                .flat_map(|group| group.alphas.iter().map(|&alpha| (0.0, 0.0, alpha))),
        )?
        .map_or((0.0, 0.0), |ink| ink.dimensions()))
    }

    pub(crate) fn hue_levels(&self) -> &[String] {
        &self.hues
    }

    fn resolved_colors(
        &self,
        chart: &Plot,
        theme: &ResolvedTheme,
        data: crate::data::PlotData<'_>,
    ) -> Result<(CategoryColors, AccentColors), Error> {
        let resources = data.source_resources()?;
        let colors = CategoryColors::resolve(
            chart.global_hue_levels().unwrap_or(&self.hues),
            theme.color_scales.get_qualitative(),
            chart.categorical_color_scale_spec(),
            resources,
        )?;
        let accents = AccentColors::resolve(
            theme.marks.get_accents(),
            usize::from(self.groups.iter().any(|group| group.identity.hue.is_none())),
        )?;
        Ok((colors, accents))
    }

    fn group_color(
        group: &HueGroup,
        colors: &CategoryColors,
        accents: &AccentColors,
    ) -> Result<(u8, u8, u8), Error> {
        if let Some(hue) = &group.identity.hue {
            colors.color(hue)
        } else {
            accents.color(0)
        }
    }
}

/// Prepared variable count determines the number of cells, including empty
/// off-diagonal cells. Painted primary points and diagonal density outlines
/// determine the hard area of each variable's native tracks.
pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let prepared = chart.prepared_matrix_axes.as_ref().ok_or_else(|| {
        Error::Render("matrix intrinsic requires prepared variable domains".into())
    })?;
    let n = prepared.labels.len();
    let (out_x, out_y) = prepared.density_ink(theme)?;
    let mut cell = (
        (2.0 * out_x).max(1.0 / 1024.0),
        (2.0 * out_y).max(1.0 / 1024.0),
    );

    let point = prepared.point_dimensions(chart, theme)?;
    cell.0 = cell.0.max(point.0);
    cell.1 = cell.1.max(point.1);
    let mut width = 0.0;
    let mut height = 0.0;
    for label in &prepared.labels {
        let column = crate::layout::fit::intrinsic_widths(
            theme,
            label,
            crate::text::TextAppearance::from(&theme.typography.axes.axis_title)
                .layout(theme.text_layouts.matrix[1].as_ref()),
            "matrix variable label",
        )?
        .0;
        let row = crate::layout::fit::intrinsic_widths(
            theme,
            label,
            crate::text::TextAppearance::from(&theme.typography.axes.axis_title)
                .layout(theme.text_layouts.matrix[0].as_ref()),
            "matrix row variable label",
        )?
        .0;
        width += cell.0.max(column);
        height += cell.1.max(row);
    }
    let gaps = theme.composition_grid_gap()? * n.saturating_sub(1) as f64;
    Ok((width + gaps, height + gaps))
}

pub(crate) fn natural_body_preference(
    chart: &Plot,
    theme: &ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let marks = chart
        .prepared_matrix_axes
        .as_ref()
        .ok_or_else(|| Error::Render("natural Matrix variables missing".into()))?;
    let hard = intrinsic_body_minimum(chart, theme)?;

    let point = marks.point_dimensions(chart, theme)?;
    let ink = marks.density_ink(theme)?;
    let mut side = theme.composition_grid_gap()? * marks.labels.len().saturating_sub(1) as f64;
    for label in &marks.labels {
        let width = layout::fit::intrinsic_widths(
            theme,
            label,
            crate::text::TextAppearance::from(&theme.typography.axes.axis_title)
                .layout(theme.text_layouts.matrix[1].as_ref()),
            "Matrix natural variable",
        )?
        .0;
        side += width
            .max(point.0)
            .max(point.1)
            .max(2.0 * ink.0)
            .max(2.0 * ink.1)
            .max(12.0 * theme.typography.axes.tick_label.font_size);
    }
    Ok((side.max(hard.0), side.max(hard.1)))
}

pub(crate) fn natural_guide_preference(
    chart: &Plot,
    theme: &ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let Some(descriptor) = chart.prepared_discrete_guide()? else {
        return Ok((0.0, 0.0));
    };
    Legend::natural_preference(&descriptor, theme)
}

impl PreparedMatrixAxes {
    pub(crate) fn guide_sources(
        &self,
        chart: &Plot,
        theme: &ResolvedTheme,
        data: crate::data::PlotData<'_>,
    ) -> Result<Vec<GuideSourceKey>, Error> {
        let (colors, accents) = self.resolved_colors(chart, theme, data)?;
        let mapped = crate::data::source::collect_source(
            self.groups
                .iter()
                .map(|group| Self::group_color(group, &colors, &accents)),
        )?;
        matrix_guide_entries(chart, &self.groups, &mapped, theme)
    }
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
    Scatter(
        Vec<Point2D>,
        Vec<ObservationAlpha>,
        crate::render::backend::PointStyle,
    ),
    Polygon(Vec<Point2D>, FillStyle),
}

impl MatrixPrimitive {
    fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        match self {
            Self::Scatter(points, _, style) => backend.draw_scatter(points, style),
            Self::Polygon(points, style) => backend.draw_polygon(points, style),
        }
    }

    fn geometry(
        &self,
        geometry: &mut crate::render::geometry::GeometrySnapshot,
        clip: PanelBox,
    ) -> Result<(), Error> {
        match self {
            Self::Scatter(points, _, style) => geometry.scatter(points, style, clip),
            Self::Polygon(points, style) => geometry.polygon(points, style, Some(clip)),
        }
    }
}

#[derive(Debug)]
struct MatrixCellPayload {
    address: layout::ComponentAddress,
    clip: PanelBox,
    frame: [Option<[Point2D; 2]>; 4],
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
    pub(crate) fn observe_axes(
        &self,
        round: &mut layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        for cell in &self.cells {
            round.begin_owner(&cell.address)?;
            cell.axes.observe_feedback(round)?;
            for mark in &cell.marks {
                if let MatrixPrimitive::Scatter(_, rows, _) = mark {
                    for row in rows {
                        round.observe_count(row.source_row)?;
                    }
                }
            }
        }
        Ok(())
    }
    pub(crate) fn axis_geometry(&self) -> Result<crate::render::geometry::GeometrySnapshot, Error> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new();

        for cell in &self.cells {
            if let Some(style) = &self.frame_style {
                for line in cell.frame.iter().flatten() {
                    geometry.line(line, style, None)?;
                }
            }
            geometry.append(&cell.axes.geometry_snapshot()?)?;
        }
        Ok(geometry)
    }

    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        for cell in &self.cells {
            if let Some(style) = &self.frame_style {
                for line in cell.frame.iter().flatten() {
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

    fn geometry(&self) -> Result<crate::render::geometry::GeometrySnapshot, Error> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new();
        for cell in &self.cells {
            if let Some(style) = &self.frame_style {
                for line in cell.frame.iter().flatten() {
                    geometry.line(line, style, None)?;
                }
            }
            let clip = crate::render::backend::validate_clip_region(cell.clip)?.panel();
            for mark in &cell.marks {
                mark.geometry(&mut geometry, clip)?;
            }
            geometry.append(&cell.axes.geometry_snapshot()?)?;
        }
        if let Some(legend) = &self.legend {
            legend.geometry(&mut geometry)?;
        }
        Ok(geometry)
    }
}

pub(crate) fn prepare_axes(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<PreparedMatrixAxes, Error> {
    #[cfg(test)]
    crate::plots::shared_x::AXIS_PREPARATIONS.with(|count| count.set(count.get() + 1));
    let mut cols = Vec::new();

    crate::data::source::reserve_source(&mut cols, chart.matrix_variable_names().len())?;
    cols.extend(chart.matrix_variable_names().iter().map(String::as_str));
    let labels = chart.resolve_axis_label_domain("matrix variables", cols.iter().copied())?;
    let (mut groups, hues) = build_groups(chart, &cols, data)?;
    let fill_alpha =
        chart.prepare_collective_alpha(AlphaTarget::Fill, AlphaUnitKind::Series, true, data)?;
    for group in &mut groups {
        group.fill_alpha = fill_alpha.collective(chart, &group.identity)?;
    }
    if let Some(exact) = chart.categorical_map_spec() {
        for hue in &hues {
            if exact.color_in(hue).is_none() {
                return Err(Error::Data(format!(
                    "matrix: category {hue:?} has no caller-owned color"
                )));
            }
        }
    }
    let mut ranges = Vec::new();
    crate::data::source::reserve_source(&mut ranges, cols.len())?;
    for (index, column) in cols.iter().enumerate() {
        ranges.push(matrix_support_range(
            &groups,
            index,
            column,
            chart.group_name(),
            chart.hue_name(),
        )?);
    }
    let diagonals = crate::data::source::collect_source(
        cols.iter()
            .enumerate()
            .map(|(index, column)| prepare_density(chart, &groups, index, column, ranges[index])),
    )?;
    Ok(PreparedMatrixAxes {
        ranges,
        labels,
        groups,
        hues,
        diagonals,
    })
}

pub(crate) fn measure(
    chart: &Plot,
    context: PlotRenderContext<'_>,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let theme = context.theme;
    let resources = theme.prepared_font_environment()?.operation();
    let data = context.data.with_operation(&resources);
    let prepared = chart
        .prepared_matrix_axes
        .as_ref()
        .ok_or_else(|| Error::Render("matrix requires root-prepared semantics".into()))?;
    let n = prepared.labels.len();
    let display_cols = &prepared.labels;

    let groups = &prepared.groups;
    let (colors, accents) = prepared.resolved_colors(chart, theme, data)?;
    let group_colors = crate::data::source::collect_source(
        groups
            .iter()
            .map(|group| PreparedMatrixAxes::group_color(group, &colors, &accents)),
    )?;
    let legend = match matrix_legend(
        chart,
        (
            context.panel.width(),
            (!context.content_height).then_some(context.panel.height()),
        ),
        theme,
        context.fit_scope(),
    )? {
        crate::layout::fit::FitOutcome::Measured(legend) => {
            legend.map(crate::data::source::source_arc).transpose()?
        }
        crate::layout::fit::FitOutcome::Miss(reason) => {
            return Err(crate::layout::fit::PlanError::Fit(reason));
        }
    };

    let mut outer = context.panel;
    let legend_reserve = if let Some(legend) = &legend {
        legend.compute_box_width() + theme.legend_outside_gap()?
    } else {
        0.0
    };
    if context.content_height {
        let hard = intrinsic_body_minimum(chart, theme)?;
        let height = (outer.width() - legend_reserve).max(hard.1).max(
            legend
                .as_ref()
                .map_or(0.0, |legend| legend.compute_box_height()),
        );
        outer.y1 = outer.y0 + height;
    }
    let gap = theme.composition_grid_gap()?;
    let ink = prepared.density_ink(theme)?;
    let point = prepared.point_dimensions(chart, theme)?;
    let specs = crate::data::source::collect_source(display_cols.iter().map(|label| {
        let hard = layout::fit::intrinsic_widths(
            theme,
            label,
            crate::text::TextAppearance::from(&theme.typography.axes.axis_title)
                .layout(theme.text_layouts.matrix[1].as_ref()),
            "matrix variable label",
        )?
        .0;
        let x = layout::kernel::ceil_spatial_quanta(
            hard.max(point.0).max(2.0 * ink.0).max(1.0 / 1024.0),
        )?;
        let y = layout::kernel::ceil_spatial_quanta(
            layout::fit::intrinsic_widths(
                theme,
                label,
                crate::text::TextAppearance::from(&theme.typography.axes.axis_title)
                    .layout(theme.text_layouts.matrix[0].as_ref()),
                "matrix row variable label",
            )?
            .0
            .max(point.1)
            .max(2.0 * ink.1)
            .max(1.0 / 1024.0),
        )?;
        let preferred =
            layout::kernel::ceil_spatial_quanta(12.0 * theme.typography.axes.tick_label.font_size)?
                .max(x)
                .max(y);
        let track = |min| layout::kernel::TrackSpec {
            min,
            preferred,
            max: None,
            flexible: true,
        };
        Ok((track(x), track(y)))
    }))?;
    let x_specs = crate::data::source::collect_source(specs.iter().map(|(x, _)| Ok(*x)))?;
    let y_specs = crate::data::source::collect_source(specs.iter().map(|(_, y)| Ok(*y)))?;
    let columns = layout::interior::tracks::assign(
        &x_specs,
        (outer.x0, outer.x1 - legend_reserve),
        gap,
        false,
        context.fit_scope(),
    )?;
    let rows = layout::interior::tracks::assign(
        &y_specs,
        (outer.y0, outer.y1),
        gap,
        true,
        context.fit_scope(),
    )?;
    let (grid_bottom, grid_top) = (outer.y0, outer.y1);
    #[cfg(test)]
    crate::orchestrator::measurements::count(
        crate::orchestrator::measurements::Counter::MatrixLayoutPass,
    );

    // --- Per-variable axis ranges ---
    // A pairplot is a coordinated matrix: each variable should use one
    // canonical range wherever it appears, not one range as x and another
    // as y. The range includes the diagonal KDE evaluation extent so the
    // diagonal and off-diagonal cells stay visually aligned.
    let prepared_ranges =
        crate::data::source::collect_source(prepared.ranges.iter().copied().map(Ok))?;

    // --- Styles ---
    let frame_style = theme
        .axes
        .get_cell_frame()
        .map(|frame| crate::theme::axis::stroke_style(frame.get_stroke()))
        .transpose()?;
    // Each cell freezes the selected sides once. Common plans own ticks and
    // outer text without painting a duplicate frame or grid.
    let common_chrome = theme.axes.cell_axis_chrome();
    let mut axis_theme = theme.clone();
    axis_theme.axes = axis_theme
        .axes
        .clone()
        .frame(common_chrome.frame.cloned())?
        .cell_frame(None)?
        .grid(common_chrome.grid.cloned())?;

    // X selection depends only on a column's range and X coordinates; Y
    // selection depends only on a row's range and Y coordinates. Retain the
    // complete sources across capture; outer cells alone paint their labels.

    let cache_count = n;

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
        columns,
        rows,
        x_ticks,
        y_ticks,
    };
    let variable_ranges = &axes.ranges;
    let mut gutters = [0.0_f64; 4];

    let mut cells = Vec::new();
    crate::data::source::reserve_source(
        &mut cells,
        n.checked_mul(n)
            .ok_or_else(|| Error::Render("matrix cell count overflow".into()))?,
    )?;
    // Retain every declared cell in row-major semantic order.
    for row in 0..n {
        for col in 0..n {
            let (x0, x1) = axes.columns[col];
            let (y0, y1) = axes.rows[row];
            let cell_panel = PanelBox::new(x0, x1, y0, y1);
            // The native cell owns the polygon outline reserve. All cells in
            // a variable track use the same inset, so pairwise coordinates and
            // diagonal sources remain aligned with their frozen axes.
            let mark_panel = PanelBox::new(
                x0 + ink.0 + 2.0 / 1024.0,
                x1 - ink.0 - 2.0 / 1024.0,
                y0 + ink.1 + 2.0 / 1024.0,
                y1 - ink.1 - 2.0 / 1024.0,
            );
            let scope = context
                .fit_scope()
                .child(layout::AddressRole::MatrixCell, row * n + col)?;
            if mark_panel.width() <= 0.0 || mark_panel.height() <= 0.0 {
                return Err(scope
                    .failure(
                        layout::fit::FitKind::SpecializedGeometry,
                        "Matrix density outline reserve leaves no representable cell domain",
                    )
                    .into());
            }
            let projection = layout::Projection::new(
                variable_ranges[col],
                variable_ranges[row],
                mark_panel.as_tuple(),
            );
            let axis = |ticks: &layout::NumericTickPlan, label: Option<&str>, role: usize| {
                Ok::<_, Error>(layout::Axis {
                    source: ticks.source,
                    numeric: ticks.numeric.clone(),
                    ticks: crate::data::source::collect_source(ticks.ticks.iter().map(|tick| {
                        Ok(layout::AxisTick {
                            position: tick.position,
                            label: crate::data::source::copy_source_string(&tick.label)?,
                        })
                    }))?,
                    label: label
                        .map(crate::data::source::copy_source_string)
                        .transpose()?
                        .map(|source| {
                            let text = crate::Text::from(source);
                            if let Some(layout) = &theme.text_layouts.matrix[role] {
                                text.layout(layout.clone())
                            } else {
                                text
                            }
                        }),
                    tick_labels_visible: label.is_some(),
                    category_labels: crate::CategoryLabelSelection::all(),
                    explicit_ticks: false,
                    assigned_tick_width: None,
                    assigned_cells: None,
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
                1,
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
                0,
            )?;
            let cell_layout = Layout {
                projection,
                x_axis,
                y_axis,
            };
            let cell_axes = cell_layout.measure_axes(
                &axis_theme,
                &scope,
                chart.axis_semantic_inputs([crate::spec::AxisDomain::Continuous; 2]),
            )?;
            let mut cell = MatrixCellPayload {
                address: scope.address,
                clip: cell_panel,
                frame: {
                    let sides = theme.axes.get_cell_frame().map(|frame| frame.get_sides());
                    [
                        sides
                            .is_some_and(crate::theme::FrameSides::get_bottom)
                            .then_some([
                                Point2D::new(mark_panel.x0, mark_panel.y0),
                                Point2D::new(mark_panel.x1, mark_panel.y0),
                            ]),
                        sides
                            .is_some_and(crate::theme::FrameSides::get_left)
                            .then_some([
                                Point2D::new(mark_panel.x0, mark_panel.y0),
                                Point2D::new(mark_panel.x0, mark_panel.y1),
                            ]),
                        sides
                            .is_some_and(crate::theme::FrameSides::get_top)
                            .then_some([
                                Point2D::new(mark_panel.x0, mark_panel.y1),
                                Point2D::new(mark_panel.x1, mark_panel.y1),
                            ]),
                        sides
                            .is_some_and(crate::theme::FrameSides::get_right)
                            .then_some([
                                Point2D::new(mark_panel.x1, mark_panel.y0),
                                Point2D::new(mark_panel.x1, mark_panel.y1),
                            ]),
                    ]
                },
                marks: Vec::new(),
                axes: cell_axes,
            };
            let required = cell.axes.requirements();
            if col == 0 {
                gutters[0] = gutters[0].max(required.left + 4.0);
            }
            if col == n - 1 {
                gutters[1] = gutters[1].max(required.right + 4.0);
            }
            if row == n - 1 {
                gutters[2] = gutters[2].max(required.bottom + 4.0);
            }
            if row == 0 {
                gutters[3] = gutters[3].max(required.top + 4.0);
            }
            crate::data::source::reserve_source(&mut cell.marks, groups.len())?;

            if row == col {
                select_diagonal_kde(
                    theme,
                    &mut cell.marks,
                    &group_colors,
                    groups,
                    &prepared.diagonals[row],
                    &cell_layout.projection,
                )?;
            } else {
                for (group, &color) in groups.iter().zip(&group_colors) {
                    let xs = &group.columns[col];
                    let ys = &group.columns[row];
                    require_aligned_points(xs.len(), group.alphas.len())?;
                    let ps = chart.point_style(
                        theme,
                        color,
                        theme.marks.get_points().get_radii().get_matrix(),
                    );
                    let mut batches = Vec::new();
                    for ((&xi, &yi), &alpha) in xs.iter().zip(ys).zip(&group.alphas) {
                        let (x, y) = cell_layout.projection.project(xi, yi);
                        push_point(&mut batches, Point2D::new(x, y), alpha.point(ps.clone()))?;
                    }
                    let mut offset = 0_usize;
                    crate::data::source::reserve_source(&mut cell.marks, batches.len())?;
                    for (points, style) in batches {
                        let end = offset.checked_add(points.len()).ok_or_else(|| {
                            Error::Render("Matrix point membership offset overflow".into())
                        })?;
                        let members = group.alphas.get(offset..end).ok_or_else(|| {
                            Error::Render("Matrix point membership is not aligned".into())
                        })?;
                        let members =
                            crate::data::source::collect_source(members.iter().copied().map(Ok))?;
                        cell.marks
                            .push(MatrixPrimitive::Scatter(points, members, style));
                        offset = end;
                    }
                }
            }

            // Inspect independent, unclipped ink before the selected geometry
            // applies the explicit cell clip. Matrix has no parent per-variable
            // domain feedback owner; an uncontainable automatic mark is a FIT.
            let mut full = crate::render::geometry::GeometrySnapshot::new();
            for mark in &cell.marks {
                match mark {
                    MatrixPrimitive::Scatter(points, _, style) => {
                        full.scatter_with_clip(points, style, None)?;
                    }
                    MatrixPrimitive::Polygon(points, style) => full.polygon(points, style, None)?,
                }
            }
            if full.bounds().iter().any(|bounds| {
                (chart.user_x_limits().is_none()
                    && (bounds.left < cell_panel.x0 || bounds.right > cell_panel.x1))
                    || (chart.user_y_limits().is_none()
                        && (bounds.bottom < cell_panel.y0 || bounds.top > cell_panel.y1))
            }) {
                return Err(layout::fit::FitScope::new(&cell.address, 0)
                    .failure(
                        layout::fit::FitKind::SpecializedGeometry,
                        "Matrix automatic point or density ink exceeds its assigned cell",
                    )
                    .into());
            }

            cells.push(cell);
        }
    }

    // --- Legend (when hue is active) ---
    let legend_payload = if let Some(legend) = &legend {
        let right_edge = outer.x1;
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
        Some(legend.select(lx, ly)?)
    } else {
        None
    };

    let layout = Layout::envelope(outer);

    let mut result = PlotRenderResult::new(layout, chart.prepared_discrete_guide()?)
        .with_self_managed_decorations();
    result.grid_axis_gutters = Some(gutters.map(f64::ceil));
    let payload = MatrixPayload {
        cells,
        frame_style,
        legend: legend_payload,
    };
    let geometry = payload.geometry()?;
    result.marks = Some(crate::data::source::source_arc(
        crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Matrix(payload),
            geometry,
        },
    )?);
    result.self_managed_legend = legend;
    result.matrix_axes = Some(crate::data::source::source_arc(axes)?);
    Ok(result)
}

#[derive(Debug)]
pub(crate) struct MatrixAxisPlan {
    ranges: Vec<(f64, f64)>,
    columns: Vec<(f64, f64)>,
    rows: Vec<(f64, f64)>,
    x_ticks: Vec<Option<MatrixTickCache>>,
    y_ticks: Vec<Option<MatrixTickCache>>,
}

#[derive(Debug)]
struct MatrixTickCache {
    plan: layout::NumericTickPlan,
    standard: layout::StandardPanelInputs,
    range: [u64; 2],
    span: [u64; 2],
    side: layout::AxisSide,
    operation: crate::font::canonical::OperationContext,
}

fn matrix_ticks<'a>(
    cached: &'a mut Option<MatrixTickCache>,
    range: (f64, f64),
    projection: &layout::Projection,
    side: layout::AxisSide,
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<&'a layout::NumericTickPlan, crate::layout::fit::PlanError> {
    let environment = theme.prepared_font_environment()?;

    let range_bits = [range.0.to_bits(), range.1.to_bits()];
    let panel = projection.panel_box();
    let span = match side {
        layout::AxisSide::Bottom => [panel.x0.to_bits(), panel.x1.to_bits()],
        layout::AxisSide::Left => [panel.y0.to_bits(), panel.y1.to_bits()],
    };
    let matches = if let Some(cached) = cached.as_ref() {
        cached.operation.same_operation(&environment.operation())
            && cached.range == range_bits
            && cached.span == span
            && cached.side == side
            && cached.standard.matches(theme)?
    } else {
        false
    };
    if !matches {
        let plan = layout::NumericTickPlan::measure_selected_source(
            range,
            crate::TickGeneration::automatic(),
            false,
            projection,
            side,
            theme,
            None,
            scope,
            None,
        )?;

        *cached = Some(MatrixTickCache {
            plan,
            standard: layout::StandardPanelInputs::new(theme)?,
            range: range_bits,
            span,
            side,
            operation: environment.operation(),
        });
    }
    cached
        .as_ref()
        .map(|cached| &cached.plan)
        .ok_or_else(|| Error::Render("matrix tick selection is missing".into()).into())
}

fn matrix_legend(
    chart: &Plot,
    available: (f64, Option<f64>),
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<crate::layout::fit::FitOutcome<Option<Legend>>, Error> {
    use crate::layout::fit::FitOutcome;
    let Some(descriptor) = chart.prepared_discrete_guide()? else {
        return Ok(FitOutcome::Measured(None));
    };
    let scope = scope.child(crate::layout::AddressRole::InsideGuide, 0)?;
    match Legend::measure_assigned(
        descriptor, available, false, false, theme, &mut false, &scope,
    )? {
        FitOutcome::Measured(legend) => Ok(FitOutcome::Measured(Some(legend))),
        FitOutcome::Miss(reason) => Ok(FitOutcome::Miss(reason)),
    }
}

fn matrix_guide_entries(
    chart: &Plot,
    groups: &[HueGroup],
    group_colors: &[(u8, u8, u8)],
    theme: &ResolvedTheme,
) -> Result<Vec<GuideSourceKey>, Error> {
    if !groups.iter().any(|group| group.identity.hue.is_some())
        || chart.legend_policy_spec() == crate::LegendPolicy::Hidden
    {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    crate::data::source::reserve_source(&mut entries, groups.len())?;
    for (group, &color) in groups
        .iter()
        .zip(group_colors)
        .filter(|(group, _)| group.identity.hue.is_some())
    {
        entries.push(
            GuideSourceKey::collective(
                chart.collective_legend_identity(&group.identity)?,
                chart,
                false,
            )?
            .with_point(
                chart.point_style(
                    theme,
                    color,
                    theme.marks.get_points().get_radii().get_matrix(),
                ),
                PointRole::Primary,
            )
            .with_fill(
                theme.marks.get_fills().get_densities().get_matrix_density(),
                color,
                FillRole::MatrixDensity,
            ),
        );
    }
    Ok(entries)
}

fn prepare_density(
    chart: &Plot,
    groups: &[HueGroup],
    col_idx: usize,
    column: &str,
    range: (f64, f64),
) -> Result<MatrixDensity, Error> {
    let (d_lo, d_hi) = range;
    let grid_pts = crate::data::source::collect_source(
        (0..KDE_POINTS).map(|i| Ok(d_lo + (d_hi - d_lo) * (i as f64 / (KDE_POINTS - 1) as f64))),
    )?;

    // Total sample count across all groups for common_norm scaling
    // (matches seaborn pairplot default: area proportional to group size).

    let n_total: f64 = groups.iter().map(|g| g.columns[col_idx].len() as f64).sum();

    // Compute KDE for each group, scaled by group proportion (common_norm).
    let densities_per_group = crate::data::source::collect_source(groups.iter().map(|g| {
        let mut weighted = gaussian_kde(&g.columns[col_idx], &grid_pts, g.bandwidths[col_idx])
            .map_err(|error| {
                let context = matrix_group_context(column, chart.group_name(), chart.hue_name(), g);
                Error::Data(format!("matrix: {context}: {error}"))
            })?;
        #[allow(clippy::cast_precision_loss)]
        let weight = g.columns[col_idx].len() as f64 / n_total;

        for density in &mut weighted {
            *density *= weight;
        }

        if weighted.iter().any(|density| !density.is_finite()) {
            let context = matrix_group_context(column, chart.group_name(), chart.hue_name(), g);
            return Err(Error::Data(format!(
                "matrix: {context} produced non-finite weighted KDE density"
            )));
        }
        Ok(weighted)
    }))?;

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
    group_colors: &[(u8, u8, u8)],
    groups: &[HueGroup],
    density: &MatrixDensity,
    projection: &layout::Projection,
) -> Result<(), Error> {
    let panel = projection.panel_box();
    let projection = layout::Projection::new(
        projection.x_range(),
        (0.0, density.maximum),
        (
            panel.x0,
            panel.x1,
            panel.y0,
            panel.y0
                + panel.height()
                    * theme
                        .marks
                        .get_extents()
                        .get_matrix_density_fraction()
                        .as_f64(),
        ),
    );
    let (x0, y0) = projection.project(projection.x_range().0, 0.0);
    let (x1, _) = projection.project(projection.x_range().1, 0.0);

    if density.densities.len() != group_colors.len() || groups.len() != group_colors.len() {
        return Err(Error::Render(
            "matrix density groups differ from mapped colors".into(),
        ));
    }
    for ((densities, &color), group) in density.densities.iter().zip(group_colors).zip(groups) {
        let mut poly = Vec::new();
        crate::data::source::reserve_source(&mut poly, KDE_POINTS + 2)?;
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
        let fill = fill_style(
            group.fill_alpha,
            theme
                .marks
                .get_fills()
                .get_densities()
                .get_matrix_density()
                .backend_style(color),
        );
        marks.push(MatrixPrimitive::Polygon(poly, fill));
    }
    Ok(())
}

/// Build graphical groups while preserving hue as color-only metadata.
fn build_groups(
    chart: &Plot,
    cols: &[&str],
    data: crate::data::PlotData<'_>,
) -> Result<(Vec<HueGroup>, Vec<String>), Error> {
    let alpha = chart.prepare_alpha(AlphaTarget::Point, AlphaUnitKind::Observation, &[])?;
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
    let hue_levels = crate::data::source::collect_source(
        chart
            .global_hue_levels()
            .unwrap_or(&channels.hue_levels)
            .iter()
            .map(|level| crate::data::source::copy_source_string(level)),
    )?;
    let identities = chart
        .global_collective_pairs()
        .unwrap_or(&channels.identities);

    let mut groups = Vec::new();
    crate::data::source::reserve_source(&mut groups, identities.len())?;
    let mut rows = Vec::new();
    crate::data::source::reserve_source(&mut rows, channels.rows.len())?;
    for identity in identities {
        rows.clear();

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

        if let Some(hue) = &identity.hue {
            if !hue_levels.contains(hue) {
                return Err(Error::Data(format!(
                    "matrix: hue {hue:?} is absent from the prepared domain"
                )));
            }
        }
        groups.push(HueGroup {
            identity: crate::data::source::copy_source_pair(identity)?,
            collapsed: channels.collapsed,
            fill_alpha: None,
            alphas: crate::data::source::collect_source(
                rows.iter()
                    .map(|row| ObservationAlpha::resolve(data, &alpha, *row)),
            )?,
            columns: crate::data::source::collect_source(columns.iter().map(|column| {
                crate::data::source::collect_source(rows.iter().map(|row| Ok(column[*row])))
            }))?,
            bandwidths: Vec::new(),
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
        group.bandwidths = crate::data::source::collect_source(
            group.columns.iter().zip(columns).map(|(samples, column)| {
                strict_scott_bandwidth(samples).map_err(|reason| {
                    Error::Data(format!(
                        "matrix: {} {reason}",
                        matrix_group_context(column, group_column, hue_column, group)
                    ))
                })
            }),
        )?;
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
    let columns = crate::data::source::collect_source(
        cols.iter()
            .map(|column| crate::data::strict_finite_f64(data, column, "matrix")),
    )?;

    if columns.iter().any(|column| column.len() != data.height()) {
        return Err(Error::Data("matrix: column length mismatch".into()));
    }
    Ok(columns)
}

#[cfg(test)]
mod alpha_tests;
