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

use crate::alpha::{
    marks::{
        fill_ink_outset, fill_style, point_ink, push_point, require_aligned_points,
        ObservationAlpha,
    },
    AlphaTarget, AlphaUnitKind, ResolvedAlpha,
};
use crate::decorations::legend::source::{FillRole, PointRole};
use crate::decorations::legend::GuideSourceKey;
use crate::layout::{self, AxisSpec, PanelBox};
use crate::render::backend::{with_clip, Backend, FillStyle, Point2D};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::stats::distribution::{gaussian_kde, kde_support_range, strict_scott_bandwidth};
use crate::theme::{AccentColors, CategoryColors};
use crate::Error;
use crate::Plot;

const MARGINAL_KDE_CUT: f64 = 3.0;
const MARGINAL_KDE_GRID_POINTS: usize = 256;
const MARGINAL_KDE_VISIBLE_FRACTION: f64 = 0.02;
const MARGINAL_DATA_MARGIN_FRACTION: f64 = 0.06;

/// One graphical group's data and optional color identity.
#[derive(Debug)]
struct HueGroup {
    identity: crate::data::CollectivePair,
    collapsed: bool,
    x: Vec<f64>,
    y: Vec<f64>,
    alphas: Vec<ObservationAlpha>,
    fill_alpha: Option<ResolvedAlpha>,
    x_bandwidth: f64,
    y_bandwidth: f64,
}

#[derive(Debug)]
pub(crate) struct PreparedMarginal {
    groups: Vec<HueGroup>,
    hues: Vec<String>,
    pub(crate) ranges: [(f64, f64); 2],
    curves: MarginalCurves,
}

impl PreparedMarginal {
    pub(crate) fn alpha_guide(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        guide: &mut crate::decorations::legend::alpha::AlphaGuideBuilder<'_>,
    ) -> crate::Result<()> {
        use crate::decorations::legend::alpha::{AlphaGlyphRole, AlphaUnit};
        let point = chart.point_style(theme, (0, 0, 0), theme.marks.points().radii().marginal());
        let fill = theme.marks.fills().densities().marginal_density();
        for group in &self.groups {
            for &alpha in &group.alphas {
                guide.point(AlphaGlyphRole::Primary, alpha, point.clone(), false)?;
            }
            guide.fill(
                AlphaGlyphRole::Fill,
                AlphaUnitKind::Series,
                AlphaUnit::series(chart, Some(&group.identity))?,
                group.fill_alpha,
                fill.backend_style((0, 0, 0)),
                fill.outline()
                    .is_some_and(|outline| outline.mapped_width().is_some()),
            )?;
        }
        Ok(())
    }

    pub(crate) fn checked_primary_mark_count(&self) -> Result<u128, Error> {
        let mut count = 0_u128;
        for group in &self.groups {
            count = count
                .checked_add(group.x.len() as u128)
                .ok_or_else(|| Error::Render("Marginal mark count overflow".into()))?;
        }
        Ok(count)
    }

    fn resolved_colors(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        data: crate::data::PlotData<'_>,
    ) -> Result<(CategoryColors, AccentColors), Error> {
        let resources = data.source_resources()?;
        let keys = chart.global_hue_levels().unwrap_or(&self.hues);
        let colors = CategoryColors::resolve(
            keys,
            theme.color_scales.qualitative(),
            chart.categorical_color_scale_spec(),
            resources,
        )?;
        let accents = AccentColors::resolve(
            theme.marks.accents(),
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

    pub(crate) fn hue_levels(&self) -> impl Iterator<Item = &str> {
        self.groups
            .iter()
            .filter_map(|group| group.identity.hue.as_deref())
    }

    pub(crate) fn guide_entries(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        data: crate::data::PlotData<'_>,
    ) -> Result<Vec<GuideSourceKey>, Error> {
        let (colors, accents) = self.resolved_colors(chart, theme, data)?;
        self.guide_entries_with_colors(chart, theme, &colors, &accents)
    }

    fn guide_entries_with_colors(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,

        colors: &CategoryColors,
        accents: &AccentColors,
    ) -> Result<Vec<GuideSourceKey>, Error> {
        let mut entries = Vec::new();
        crate::data::source::reserve_source(&mut entries, self.groups.len())?;
        for group in &self.groups {
            if group.identity.hue.is_none() {
                continue;
            }
            let color = Self::group_color(group, colors, accents)?;
            entries.push(
                GuideSourceKey::collective(
                    chart.collective_legend_identity(&group.identity)?,
                    chart,
                    false,
                )?
                .with_point(
                    chart.point_style(theme, color, theme.marks.points().radii().marginal()),
                    PointRole::Primary,
                )
                .with_fill(
                    theme.marks.fills().densities().marginal_density(),
                    color,
                    FillRole::MarginalDensity,
                ),
            );
        }
        Ok(entries)
    }
}

/// Retained central point and marginal-density ink supply the linked-panel
/// parent's hard input; the fixed strip/gap packing remains local until T5e3.
pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let prepared = chart
        .prepared_marginal
        .as_ref()
        .ok_or_else(|| Error::Render("marginal intrinsic requires prepared groups".into()))?;
    let mut central =
        crate::layout::interior::coordinate::Intrinsics::prepared(chart, theme)?.minimum(chart);

    for group in &prepared.groups {
        let style = chart.point_style(theme, (0, 0, 0), theme.marks.points().radii().marginal());
        require_aligned_points(group.x.len(), group.alphas.len())?;
        if let Some(ink) = point_ink(
            &style,
            group
                .x
                .iter()
                .zip(&group.y)
                .zip(&group.alphas)
                .map(|((&x, &y), &alpha)| (x, y, alpha)),
        )? {
            let (x, y) = ink.dimensions();
            central.0 = central.0.max(x);
            central.1 = central.1.max(y);
        }
    }
    let (out_x, out_y) = density_ink(chart, theme)?;
    // Each filled KDE strip has a positive extent even when its paint is
    // transparent. Its stroke is measured by the same canonical geometry as
    // selection, not inferred from the nominal line width.
    Ok((
        central.0 + theme.composition_grid_gap()? + (1.0 / 1024.0_f64).max(2.0 * out_x),
        central.1 + theme.composition_grid_gap()? + (1.0 / 1024.0_f64).max(2.0 * out_y),
    ))
}

fn density_ink(chart: &Plot, theme: &crate::theme::ResolvedTheme) -> Result<(f64, f64), Error> {
    let prepared = chart
        .prepared_marginal
        .as_ref()
        .ok_or_else(|| Error::Render("Marginal density ink requires prepared groups".into()))?;
    fill_ink_outset(
        &theme
            .marks
            .fills()
            .densities()
            .marginal_density()
            .backend_style((0, 0, 0)),
        prepared.groups.iter().map(|group| group.fill_alpha),
    )
}

#[derive(Debug, Default)]
struct MarginalCurves {
    x_grid: Vec<f64>,
    y_grid: Vec<f64>,
    x_densities: Vec<Vec<f64>>,
    y_densities: Vec<Vec<f64>>,
}

#[derive(Debug)]
struct MarginalGroupPayload {
    point_batches: Vec<(Vec<Point2D>, crate::render::backend::PointStyle)>,
    source_rows: Vec<usize>,
    top: Vec<Point2D>,
    right: Vec<Point2D>,
    fill: FillStyle,
}

#[derive(Debug)]
struct LinkedPanelPlan {
    address: crate::layout::ComponentAddress,
    projection: layout::Projection,
}

fn measure_linked_panel(
    panel: PanelBox,
    ranges: [(f64, f64); 2],
    role: layout::AddressRole,
    context: PlotRenderContext<'_>,
) -> Result<LinkedPanelPlan, crate::layout::fit::PlanError> {
    if ![panel.x0, panel.x1, panel.y0, panel.y1]
        .into_iter()
        .all(f64::is_finite)
        || ranges
            .iter()
            .any(|range| !range.0.is_finite() || !range.1.is_finite() || range.0 >= range.1)
    {
        return Err(Error::Render(
            "Marginal linked subpanel assignment/domain is not finite and ordered".into(),
        )
        .into());
    }

    let address = context.fit_scope().address.child(role, 0)?;
    if panel.width() <= 0.0 || panel.height() <= 0.0 {
        return Err(crate::layout::fit::FitScope::new(&address, 0)
            .failure(
                crate::layout::fit::FitKind::AssignedChild,
                "Marginal linked subpanel has no positive assigned extent",
            )
            .into());
    }
    Ok(LinkedPanelPlan {
        address,
        projection: layout::Projection::new(ranges[0], ranges[1], panel.as_tuple()),
    })
}

#[derive(Debug)]
pub(crate) struct MarginalPayload {
    panels: [LinkedPanelPlan; 3],
    ticks: Vec<[Point2D; 2]>,
    tick_style: Option<crate::render::backend::LineStyle>,
    groups: Vec<MarginalGroupPayload>,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

impl MarginalPayload {
    fn clips(&self) -> [PanelBox; 3] {
        self.panels
            .each_ref()
            .map(|panel| panel.projection.panel_box())
    }

    pub(crate) fn observe_subpanels(
        &self,
        round: &mut layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        for panel in &self.panels {
            round.begin_owner(&panel.address)?;
            round.observe_box(panel.projection.panel_box())?;
            let ranges = [panel.projection.x_range(), panel.projection.y_range()];
            for (lower, upper) in ranges {
                round.observe_number(lower)?;
                round.observe_number(upper)?;
            }
        }
        round.begin_owner(&self.panels[0].address)?;
        for group in &self.groups {
            for row in &group.source_rows {
                round.observe_count(*row)?;
            }
        }
        Ok(())
    }

    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        if let Some(style) = &self.tick_style {
            for tick in &self.ticks {
                backend.draw_line(tick, style)?;
            }
        }
        for group in &self.groups {
            with_clip(backend, self.clips()[0], |backend| {
                for (points, style) in &group.point_batches {
                    backend.draw_scatter(points, style)?;
                }
                Ok(())
            })?;
            with_clip(backend, self.clips()[1], |backend| {
                backend.draw_polygon(&group.top, &group.fill)
            })?;
            with_clip(backend, self.clips()[2], |backend| {
                backend.draw_polygon(&group.right, &group.fill)
            })?;
        }
        Ok(())
    }
}

pub(crate) fn natural_body_preference(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let hard = intrinsic_body_minimum(chart, theme)?;
    let gap = theme.composition_grid_gap()?;
    let ink = density_ink(chart, theme)?;
    let axes = layout::interior::coordinate::Intrinsics::prepared(chart, theme)?.minimum(chart);
    let central = (
        (hard.0 - gap - (2.0 * ink.0).max(1.0 / 1024.0)).max(axes.0),
        (hard.1 - gap - (2.0 * ink.1).max(1.0 / 1024.0)).max(axes.1),
    );
    let preference = layout::interior::preference::mark_preference(chart, theme, central)?;
    let width = preference.body_quanta.0 as f64 / 1024.0;
    let height = preference.body_quanta.1 as f64 / 1024.0;
    let quarter = |value: i64| -> Result<f64, Error> {
        let quanta = i64::try_from((i128::from(value) + 3) / 4)
            .map_err(|_| Error::Render("Marginal quarter preference overflowed".into()))?;
        Ok(quanta as f64 / 1024.0)
    };
    Ok((
        width + quarter(preference.body_quanta.0)?.max(2.0 * ink.0) + gap,
        height + quarter(preference.body_quanta.1)?.max(2.0 * ink.1) + gap,
    ))
}

pub(crate) fn natural_body_height(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
    width: f64,
) -> Result<f64, Error> {
    let gap = theme.composition_grid_gap()?;
    let hard = intrinsic_body_minimum(chart, theme)?;
    let ink = density_ink(chart, theme)?;
    let right_hard = (2.0 * ink.0).max(1.0 / 1024.0);
    let top_hard = (2.0 * ink.1).max(1.0 / 1024.0);
    let axes = layout::interior::coordinate::Intrinsics::prepared(chart, theme)?.minimum(chart);
    let central_hard = (
        (hard.0 - gap - right_hard).max(axes.0),
        (hard.1 - gap - top_hard).max(axes.1),
    );
    let preference = layout::interior::preference::mark_preference(chart, theme, central_hard)?;
    let track = |min: f64, preferred: i64| -> Result<_, Error> {
        let min = layout::kernel::ceil_spatial_quanta(min)?;
        Ok(layout::kernel::TrackSpec {
            min,
            preferred: preferred.max(min),
            max: None,
            flexible: true,
        })
    };
    let quarter = i64::try_from((i128::from(preference.body_quanta.0) + 3) / 4)
        .map_err(|_| Error::Render("Marginal quarter width overflowed".into()))?;

    let columns = layout::interior::tracks::assign(
        &[
            track(central_hard.0, preference.body_quanta.0)?,
            track(right_hard, quarter)?,
        ],
        (0.0, width),
        gap,
        false,
        &layout::fit::FitScope::root(),
    )
    .map_err(layout::fit::PlanError::into_error)?;
    let central_width = columns[0].1 - columns[0].0;
    let quarter_height = i64::try_from((i128::from(preference.body_quanta.1) + 3) / 4)
        .map_err(|_| Error::Render("Marginal quarter height overflowed".into()))?;
    let top = (quarter_height as f64 / 1024.0).max(top_hard);
    Ok(central_hard
        .1
        .max(3.0 * central_width / 4.0)
        .max(preference.density_side)
        + top
        + gap)
}

pub(crate) fn measure(
    chart: &Plot,
    context: PlotRenderContext<'_>,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let theme = context.theme;
    let prepared = chart
        .prepared_marginal
        .as_ref()
        .ok_or_else(|| Error::Render("marginal requires root-prepared semantics".into()))?;
    let groups = &prepared.groups;
    let resources = theme.prepared_font_environment()?.operation();
    let data = context.data.with_operation(&resources);

    // --- Layout geometry ---
    let outer = context.panel;
    let gap = theme.composition_grid_gap()?;
    let hard = intrinsic_body_minimum(chart, theme)?;
    if outer.width() < hard.0 || outer.height() < hard.1 {
        return Err(context.fit_scope().failure(
            crate::layout::fit::FitKind::SpecializedGeometry,
            "marginal linked strips leave less than the prepared central mark and density ink minimum",
        ).into());
    }
    let ink = density_ink(chart, theme)?;
    let right_hard = (2.0 * ink.0).max(1.0 / 1024.0);
    let top_hard = (2.0 * ink.1).max(1.0 / 1024.0);
    let axes_hard =
        layout::interior::coordinate::Intrinsics::prepared(chart, theme)?.minimum(chart);
    let central_hard = (
        (hard.0 - gap - right_hard).max(axes_hard.0),
        (hard.1 - gap - top_hard).max(axes_hard.1),
    );
    let preference = layout::interior::preference::mark_preference(chart, theme, central_hard)?;
    let track = |min: f64, preferred: i64| -> Result<_, Error> {
        let min = layout::kernel::ceil_spatial_quanta(min)?;
        Ok(layout::kernel::TrackSpec {
            min,
            preferred: preferred.max(min),
            max: None,
            flexible: true,
        })
    };
    let quarter = |value: i64| -> Result<i64, Error> {
        i64::try_from((i128::from(value) + 3) / 4)
            .map_err(|_| Error::Render("Marginal quarter preference overflow".into()))
    };
    let columns = layout::interior::tracks::assign(
        &[
            track(central_hard.0, preference.body_quanta.0)?,
            track(right_hard, quarter(preference.body_quanta.0)?)?,
        ],
        (outer.x0, outer.x1),
        gap,
        false,
        context.fit_scope(),
    )?;
    let rows = layout::interior::tracks::assign(
        &[
            track(top_hard, quarter(preference.body_quanta.1)?)?,
            track(central_hard.1, preference.body_quanta.1)?,
        ],
        (outer.y0, outer.y1),
        gap,
        true,
        context.fit_scope(),
    )?;
    let (central_x0, central_x1) = columns[0];
    let (central_y0, central_y1) = rows[1];
    let (top_y0, top_y1) = rows[0];
    let (right_x0, right_x1) = columns[1];

    // --- Data ranges ---
    let (x_low, x_high) =
        context.axis_range(layout::AxisSide::Bottom, || Ok(prepared.ranges[0]))?;
    let (y_low, y_high) = context.axis_range(layout::AxisSide::Left, || Ok(prepared.ranges[1]))?;
    let ranges = [(x_low, x_high), (y_low, y_high)];
    // Automatic clearance expands the projection around the retained density
    // source. Regenerating endpoints on each expanded boundary would recreate
    // the same clipped outline and could never discharge that feedback.
    let curves = &prepared.curves;

    // --- Build Layout for the central scatter ---
    let layout = (|| {
        context.layout_with_axis_specs(
            (x_low, x_high),
            (y_low, y_high),
            crate::layout::PanelBox::new(central_x0, central_x1, central_y0, central_y1),
            crate::data::source::copy_source_string(
                &chart
                    .prepared_axis_inputs()
                    .ok_or_else(|| Error::Render("Marginal axes missing".into()))?
                    .labels[0],
            )?,
            crate::data::source::copy_source_string(
                &chart
                    .prepared_axis_inputs()
                    .ok_or_else(|| Error::Render("Marginal axes missing".into()))?
                    .labels[1],
            )?,
            &layout::display_axis_spec(chart.x_axis_spec(), (x_low, x_high)),
            &layout::display_axis_spec(chart.y_axis_spec(), (y_low, y_high)),
            theme,
        )
    })()?;

    let proj = &layout.projection;

    // --- Tick marks on marginal panels (no labels, matching seaborn) ---
    let tick_len = theme
        .axes
        .ticks
        .as_ref()
        .map_or(0.0, |appearance| f64::from(appearance.length));
    let tick_style = theme
        .axes
        .ticks
        .as_ref()
        .map(|appearance| appearance.stroke.backend_style());

    // Top KDE panel: x-axis ticks along the bottom edge (y = top_y0).
    // Positions match the central scatter's x-axis ticks.
    let mut ticks = Vec::new();
    crate::data::source::reserve_source(
        &mut ticks,
        layout.x_axis.ticks.len() + layout.y_axis.ticks.len(),
    )?;
    if tick_style.is_some() {
        for tick in &layout.x_axis.ticks {
            let (x_px, _) = proj.project(tick.position, y_low);
            if x_px >= central_x0 && x_px <= central_x1 {
                ticks.push([
                    Point2D::new(x_px, top_y0),
                    Point2D::new(x_px, top_y0 - tick_len),
                ]);
            }
        }

        // Right KDE panel: y-axis ticks along the left edge (x = right_x0).
        // Positions match the central scatter's y-axis ticks.
        for tick in &layout.y_axis.ticks {
            let (_, y_px) = proj.project(x_low, tick.position);
            if y_px >= central_y0 && y_px <= central_y1 {
                ticks.push([
                    Point2D::new(right_x0, y_px),
                    Point2D::new(right_x0 - tick_len, y_px),
                ]);
            }
        }
    }

    let x_densities = &curves.x_densities;
    let y_densities = &curves.y_densities;
    let y_grid = &curves.y_grid;
    let n_grid = curves.x_grid.len();

    let x_dens_max = x_densities
        .iter()
        .flatten()
        .copied()
        .fold(0.0_f64, f64::max);
    let y_dens_max = y_densities
        .iter()
        .flatten()
        .copied()
        .fold(0.0_f64, f64::max);
    if !x_dens_max.is_finite() || !y_dens_max.is_finite() || x_dens_max <= 0.0 || y_dens_max <= 0.0
    {
        return Err(context
            .fit_scope()
            .failure(
                crate::layout::fit::FitKind::SpecializedGeometry,
                "marginal KDE has no positive representable density in a required linked strip",
            )
            .into());
    }

    let central_panel = PanelBox::new(central_x0, central_x1, central_y0, central_y1);
    let top_panel = PanelBox::new(central_x0, central_x1, top_y0, top_y1);
    let right_panel = PanelBox::new(right_x0, right_x1, central_y0, central_y1);
    let panels = [
        measure_linked_panel(
            central_panel,
            ranges,
            layout::AddressRole::MarginalCentral,
            context,
        )?,
        measure_linked_panel(
            top_panel,
            [ranges[0], (0.0, 1.0)],
            layout::AddressRole::MarginalTop,
            context,
        )?,
        measure_linked_panel(
            right_panel,
            [(0.0, 1.0), ranges[1]],
            layout::AddressRole::MarginalRight,
            context,
        )?,
    ];
    let mut selected = Vec::new();
    crate::data::source::reserve_source(&mut selected, groups.len())?;
    let (colors, accents) = prepared.resolved_colors(chart, theme, data)?;
    let mut source = [(f64::INFINITY, f64::NEG_INFINITY); 2];
    let mut point_bounds: Option<crate::render::geometry::GeometryBox> = None;
    for (gi, group) in groups.iter().enumerate() {
        let color = PreparedMarginal::group_color(group, &colors, &accents)?;
        let point_style = chart.point_style(theme, color, theme.marks.points().radii().marginal());
        require_aligned_points(group.x.len(), group.alphas.len())?;
        if let Some(ink) = point_ink(
            &point_style,
            group
                .x
                .iter()
                .zip(&group.y)
                .zip(&group.alphas)
                .map(|((&x, &y), &alpha)| (x, y, alpha)),
        )? {
            point_bounds =
                Some(point_bounds.map_or(ink.bounds, |previous| previous.union(ink.bounds)));
            for (source, range) in source.iter_mut().zip(ink.ranges) {
                source.0 = source.0.min(range.0);
                source.1 = source.1.max(range.1);
            }
        }
        let mut point_batches = Vec::new();
        for ((&x, &y), &alpha) in group.x.iter().zip(&group.y).zip(&group.alphas) {
            let (x, y) = panels[0].projection.project(x, y);
            push_point(
                &mut point_batches,
                Point2D::new(x, y),
                alpha.point(point_style.clone()),
            )?;
        }
        let mut top_poly = Vec::new();
        let mut right_poly = Vec::new();
        crate::data::source::reserve_source(&mut top_poly, n_grid + 2)?;
        crate::data::source::reserve_source(&mut right_poly, n_grid + 2)?;
        let (top_start, _) = panels[1].projection.project(curves.x_grid[0], 0.0);
        let (_, right_start) = panels[2].projection.project(0.0, curves.y_grid[0]);
        let baseline_y = top_y0 + ink.1 + 2.0 / 1024.0;
        let baseline_x = right_x0 + ink.0 + 2.0 / 1024.0;
        let top_span = top_panel.height() - 2.0 * ink.1 - 4.0 / 1024.0;
        let right_span = right_panel.width() - 2.0 * ink.0 - 4.0 / 1024.0;
        if top_span <= 0.0 || right_span <= 0.0 {
            return Err(context
                .fit_scope()
                .failure(
                    layout::fit::FitKind::SpecializedGeometry,
                    "Marginal density outline reserve leaves no representable strip",
                )
                .into());
        }
        top_poly.push(Point2D::new(top_start, baseline_y));
        right_poly.push(Point2D::new(baseline_x, right_start));
        for i in 0..n_grid {
            let (x, _) = panels[1].projection.project(curves.x_grid[i], 0.0);
            let y = baseline_y
                + x_densities[gi][i] / x_dens_max
                    * theme.marks.extents().marginal_density_fraction().as_f64()
                    * top_span;
            top_poly.push(Point2D::new(x, y));
            let (_, y) = panels[2].projection.project(0.0, y_grid[i]);
            let x = baseline_x
                + y_densities[gi][i] / y_dens_max
                    * theme.marks.extents().marginal_density_fraction().as_f64()
                    * right_span;
            right_poly.push(Point2D::new(x, y));
        }
        let (top_end, _) = panels[1].projection.project(curves.x_grid[n_grid - 1], 0.0);
        let (_, right_end) = panels[2].projection.project(0.0, curves.y_grid[n_grid - 1]);
        top_poly.push(Point2D::new(top_end, baseline_y));
        right_poly.push(Point2D::new(baseline_x, right_end));
        let fill = fill_style(
            group.fill_alpha,
            theme
                .marks
                .fills()
                .densities()
                .marginal_density()
                .backend_style(color),
        );
        selected.push(MarginalGroupPayload {
            point_batches,
            source_rows: crate::data::source::collect_source(
                group.alphas.iter().map(|alpha| Ok(alpha.source_row)),
            )?,
            top: top_poly,
            right: right_poly,
            fill,
        });
    }
    let mut domain_expansions = if let Some(ink) = point_bounds {
        crate::layout::interior::automatic_mark_ink_expansions(
            source,
            proj,
            ink,
            [
                chart.user_x_limits().is_none(),
                chart.user_y_limits().is_none(),
            ],
            context.fit_scope(),
        )?
    } else {
        [None; 2]
    };
    for (side, panel) in [(0, &panels[1]), (1, &panels[2])] {
        let mut full = crate::render::geometry::GeometrySnapshot::new();
        for group in &selected {
            full.polygon(
                if side == 0 { &group.top } else { &group.right },
                &group.fill,
                None,
            )?;
        }
        if let Some(bounds) = full
            .bounds()
            .iter()
            .copied()
            .reduce(crate::render::geometry::GeometryBox::union)
        {
            let assigned = panel.projection.panel_box();
            if (side == 0 && (bounds.bottom < assigned.y0 || bounds.top > assigned.y1))
                || (side == 1 && (bounds.left < assigned.x0 || bounds.right > assigned.x1))
            {
                return Err(layout::fit::FitScope::new(&panel.address, 0)
                    .failure(
                        layout::fit::FitKind::SpecializedGeometry,
                        "Marginal density outline exceeds its assigned amplitude reserve",
                    )
                    .into());
            }
            let requests = crate::layout::interior::automatic_absolute_ink_expansions(
                if side == 0 {
                    [prepared.ranges[0], (0.0, 1.0)]
                } else {
                    [(0.0, 1.0), prepared.ranges[1]]
                },
                &panel.projection,
                bounds,
                [
                    side == 0 && chart.user_x_limits().is_none(),
                    side == 1 && chart.user_y_limits().is_none(),
                ],
                context.fit_scope(),
            )?;
            domain_expansions =
                crate::layout::interior::merge_mark_domain_expansions(domain_expansions, requests);
        }
    }
    let payload = MarginalPayload {
        panels,
        ticks,
        tick_style,
        groups: selected,
        domain_expansions,
    };
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    if let Some(style) = &payload.tick_style {
        for tick in &payload.ticks {
            geometry.line(tick, style, None)?;
        }
    }
    for group in &payload.groups {
        for (points, style) in &group.point_batches {
            geometry.scatter(points, style, central_panel)?;
        }
        geometry.polygon(&group.top, &group.fill, Some(top_panel))?;
        geometry.polygon(&group.right, &group.fill, Some(right_panel))?;
    }
    let mut result = PlotRenderResult::new(layout, chart.prepared_discrete_guide()?);
    result.axes = Some(result.layout.measure_axes_with_inputs(
        theme,
        context.fit_scope(),
        chart.axis_semantic_inputs([crate::spec::AxisDomain::Continuous; 2]),
        || layout::StandardPanelInputs::new(theme),
    )?);
    result.marks = Some(crate::data::source::source_arc(
        crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Marginal(payload),
            geometry,
        },
    )?);
    Ok(result)
}

pub(crate) fn prepare(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<PreparedMarginal, Error> {
    let (mut groups, hues) = build_groups(chart, data)?;
    let fill_alpha =
        chart.prepare_collective_alpha(AlphaTarget::Fill, AlphaUnitKind::Series, true, data)?;
    for group in &mut groups {
        group.fill_alpha = fill_alpha.collective(chart, &group.identity)?;
    }
    if let Some(exact) = chart.categorical_map_spec() {
        for hue in &hues {
            if exact.color_in(hue).is_none() {
                return Err(Error::Data(format!(
                    "marginal: category {hue:?} has no caller-owned color"
                )));
            }
        }
    }
    let ranges = ranges_from_groups(chart, &groups)?;
    Ok(PreparedMarginal {
        groups,
        hues,
        ranges,
        curves: MarginalCurves::default(),
    })
}

pub(crate) fn finish(
    mut prepared: PreparedMarginal,
    x_limits: Option<(f64, f64)>,
) -> Result<PreparedMarginal, Error> {
    if let Some(limits) = x_limits {
        prepared.ranges[0] = limits;
    }
    prepared.curves = render_curves(&prepared.groups, prepared.ranges)?;
    Ok(prepared)
}

fn render_curves(groups: &[HueGroup], ranges: [(f64, f64); 2]) -> Result<MarginalCurves, Error> {
    let [(x_low, x_high), (y_low, y_high)] = ranges;
    // --- KDE grid (shared across groups) ---
    let n_grid = 80;
    let x_grid = crate::data::source::collect_source(
        (0..n_grid).map(|i| Ok(x_low + (x_high - x_low) * (i as f64 / (n_grid - 1) as f64))),
    )?;
    let y_grid = crate::data::source::collect_source(
        (0..n_grid).map(|i| Ok(y_low + (y_high - y_low) * (i as f64 / (n_grid - 1) as f64))),
    )?;

    // Compute per-group KDE densities to find the shared max.
    let mut x_densities = crate::data::source::collect_source(groups.iter().map(|group| {
        gaussian_kde(&group.x, &x_grid, group.x_bandwidth).map_err(|error| {
            if !matches!(error, Error::Data(_)) {
                return error;
            }
            Error::Data(format!(
                "marginal: x distribution for {}: {error}",
                group.identity.description(group.collapsed)
            ))
        })
    }))?;
    let mut y_densities = crate::data::source::collect_source(groups.iter().map(|group| {
        gaussian_kde(&group.y, &y_grid, group.y_bandwidth).map_err(|error| {
            if !matches!(error, Error::Data(_)) {
                return error;
            }
            Error::Data(format!(
                "marginal: y distribution for {}: {error}",
                group.identity.description(group.collapsed)
            ))
        })
    }))?;

    for density in x_densities.iter_mut().chain(y_densities.iter_mut()) {
        if let Some(first) = density.first_mut() {
            *first = 0.0;
        }
        if let Some(last) = density.last_mut() {
            *last = 0.0;
        }
    }

    Ok(MarginalCurves {
        x_grid,
        y_grid,
        x_densities,
        y_densities,
    })
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
    let mut invalid = None;
    for group in groups {
        let samples = values(group);

        let wide = crate::data::min_max(samples.iter().copied(), "marginal")
            .is_ok_and(|(minimum, maximum)| !(maximum - minimum).is_finite());

        if wide || kde_support_range(std::iter::once(samples), MARGINAL_KDE_CUT).is_none() {
            invalid = Some(group);
            break;
        }
    }
    let group = invalid.map_or_else(
        || "ungrouped series".to_owned(),
        |group| group.identity.description(group.collapsed),
    );
    Err(Error::Data(format!(
        "marginal: {channel} distribution for {group}: support/grid is not representable at this numeric scale"
    )))
}

fn ranges_from_groups(chart: &Plot, groups: &[HueGroup]) -> Result<[(f64, f64); 2], Error> {
    let mut ranges = [(0.0, 1.0); 2];
    for (index, spec) in [chart.x_axis_spec(), chart.y_axis_spec()]
        .into_iter()
        .enumerate()
    {
        let values: fn(&HueGroup) -> &[f64] = if index == 0 {
            |group| &group.x
        } else {
            |group| &group.y
        };

        let range = crate::data::min_max(
            groups
                .iter()
                .flat_map(|group| values(group).iter().copied()),
            "marginal",
        )?;
        ranges[index] = marginal_display_range(groups.iter().map(values), range, spec)?;
        validate_marginal_range(
            groups,
            ranges[index],
            values,
            if index == 0 { "x" } else { "y" },
        )?;
    }
    Ok(ranges)
}

pub(crate) fn shared_x_domain(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<(f64, f64), Error> {
    let (groups, _) = build_groups(chart, data)?;

    let range = crate::data::min_max(
        groups.iter().flat_map(|group| group.x.iter().copied()),
        "marginal",
    )?;
    let range = marginal_display_range(
        groups.iter().map(|group| group.x.as_slice()),
        range,
        chart.x_axis_spec(),
    )?;
    validate_marginal_range(&groups, range, |group| &group.x, "x")?;
    Ok(range)
}

fn build_groups<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
) -> Result<(Vec<HueGroup>, Vec<String>), Error> {
    let data = data.into();
    let alpha = chart.prepare_alpha(AlphaTarget::Point, AlphaUnitKind::Observation, &[])?;

    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("marginal: x column not set".into()))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("marginal: y column not set".into()))?;
    let rows = crate::data::aligned_finite_rows(data, x_col, y_col, "marginal")?;
    data.validate_row_source()?;
    let retained_rows =
        crate::data::source::collect_source(rows.iter().map(|(row, _, _)| Ok(*row)))?;
    let channels = crate::data::observed_collectives(
        data,
        chart.group_col(),
        chart.group_name(),
        chart.hue_col(),
        chart.hue_name(),
        Some(&retained_rows),
        "marginal",
    )?;
    let hue_levels = crate::data::source::collect_source(
        chart
            .global_hue_levels()
            .unwrap_or(&channels.hue_levels)
            .iter()
            .map(|level| crate::data::source::copy_source_string(level)),
    )?;
    let mut groups = Vec::new();

    crate::data::source::reserve_source(&mut groups, channels.identities.len())?;
    for identity in &channels.identities {
        let count = channels.rows.iter().filter(|row| *row == identity).count();
        let selected = rows
            .iter()
            .zip(&channels.rows)
            .filter(|(_, row_identity)| *row_identity == identity);
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut alphas = Vec::new();
        crate::data::source::reserve_source(&mut x, count)?;
        crate::data::source::reserve_source(&mut y, count)?;
        crate::data::source::reserve_source(&mut alphas, count)?;

        for ((row, x_value, y_value), _) in selected {
            x.push(*x_value);
            y.push(*y_value);
            alphas.push(ObservationAlpha::resolve(data, &alpha, *row)?);
        }
        if x.is_empty() {
            continue;
        }

        let x_bandwidth = strict_scott_bandwidth(&x).map_err(|reason| {
            Error::Data(format!(
                "marginal: x distribution for {} {reason}",
                identity.description(channels.collapsed)
            ))
        })?;
        let y_bandwidth = strict_scott_bandwidth(&y).map_err(|reason| {
            Error::Data(format!(
                "marginal: y distribution for {} {reason}",
                identity.description(channels.collapsed)
            ))
        })?;
        if let Some(hue) = &identity.hue {
            if !hue_levels.contains(hue) {
                return Err(Error::Data(format!(
                    "marginal: hue {hue:?} is absent from the prepared domain"
                )));
            }
        }
        groups.push(HueGroup {
            identity: crate::data::source::copy_source_pair(identity)?,
            collapsed: channels.collapsed,
            x,
            y,
            alphas,
            fill_alpha: None,
            x_bandwidth,
            y_bandwidth,
        });
    }
    Ok((groups, hue_levels))
}

fn marginal_display_range<'a>(
    groups: impl Iterator<Item = &'a [f64]> + Clone,
    data_range: (f64, f64),
    spec: &AxisSpec,
) -> Result<(f64, f64), Error> {
    let data_display_range = marginal_data_display_range(data_range, spec);
    if spec.limits.is_some() {
        return Ok(data_display_range);
    }

    let Some(visible_kde_range) =
        kde_visible_support_range(groups, MARGINAL_KDE_CUT, MARGINAL_KDE_VISIBLE_FRACTION)?
    else {
        return Ok(data_display_range);
    };

    Ok((
        data_display_range.0.min(visible_kde_range.0),
        data_display_range.1.max(visible_kde_range.1),
    ))
}

fn marginal_data_display_range(data_range: (f64, f64), spec: &AxisSpec) -> (f64, f64) {
    #[cfg(test)]
    crate::orchestrator::measurements::count(
        crate::orchestrator::measurements::Counter::AxisRangePreparation,
    );
    if let Some(limits) = spec.limits {
        return limits;
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
    groups: impl Iterator<Item = &'a [f64]> + Clone,
    cut: f64,
    visible_fraction: f64,
) -> Result<Option<(f64, f64)>, Error> {
    let Some((support_low, support_high)) = kde_support_range(groups.clone(), cut) else {
        return Ok(None);
    };

    let grid: [f64; MARGINAL_KDE_GRID_POINTS] = std::array::from_fn(|i| {
        support_low
            + (support_high - support_low) * (i as f64 / (MARGINAL_KDE_GRID_POINTS - 1) as f64)
    });

    let mut low = f64::INFINITY;
    let mut high = f64::NEG_INFINITY;
    for samples in groups {
        let densities = match crate::stats::distribution::kde(samples, &grid) {
            Ok(densities) => densities,
            Err(Error::Data(_)) => continue,
            Err(error) => return Err(error),
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

    Ok((low.is_finite() && high.is_finite() && low < high).then_some((low, high)))
}

#[cfg(test)]
mod alpha_tests;
