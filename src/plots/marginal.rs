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
use crate::layout::{self, AxisSpec, PanelBox};
use crate::render::backend::{with_clip, Backend, FillStyle, Point2D};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::stats::distribution::{
    gaussian_kde_with_source, kde_support_range, strict_scott_bandwidth,
};
use crate::theme::{AccentColors, CategoryColors};
use crate::Error;
use crate::Plot;

const MARGINAL_KDE_CUT: f64 = 3.0;
const MARGINAL_KDE_GRID_POINTS: usize = 256;
const MARGINAL_KDE_VISIBLE_FRACTION: f64 = 0.02;
const MARGINAL_DATA_MARGIN_FRACTION: f64 = 0.06;

#[cfg(test)]
thread_local! {
    static STAGES: std::cell::Cell<[usize; 3]> = const { std::cell::Cell::new([0; 3]) };
}

#[cfg(test)]
fn count(stage: usize) {
    STAGES.with(|counts| {
        let mut next = counts.get();
        next[stage] += 1;
        counts.set(next);
    });
}

/// One graphical group's data and optional color identity.
#[derive(Debug)]
struct HueGroup {
    identity: crate::data::CollectivePair,
    collapsed: bool,
    x: Vec<f64>,
    y: Vec<f64>,
    source_rows: Vec<usize>,
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
    pub(crate) fn primary_mark_count(
        &self,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<u128, Error> {
        let mut count = 0_u128;
        for group in &self.groups {
            resources
                .claim_visits(1)
                .map_err(|error| Error::Render(format!("Marginal mark count: {error:?}")))?;
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
            chart.categorical_colors_spec(),
            resources,
        )?;
        let accents = AccentColors::resolve(
            &theme.marks.accents,
            usize::from(self.groups.iter().any(|group| group.identity.hue.is_none())),
            resources,
        )?;
        Ok((colors, accents))
    }

    fn group_color(
        group: &HueGroup,
        colors: &CategoryColors,
        accents: &AccentColors,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<(u8, u8, u8), Error> {
        if let Some(hue) = &group.identity.hue {
            colors.color(hue, resources)
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
    ) -> Result<Vec<LegendEntry>, Error> {
        let (colors, accents) = self.resolved_colors(chart, theme, data)?;
        self.guide_entries_with_colors(chart, theme, data, &colors, &accents)
    }

    fn guide_entries_with_colors(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        data: crate::data::PlotData<'_>,
        colors: &CategoryColors,
        accents: &AccentColors,
    ) -> Result<Vec<LegendEntry>, Error> {
        let resources = data.source_resources()?;
        let mut entries = Vec::new();
        data.reserve_source(&mut entries, self.groups.len())?;
        for group in &self.groups {
            if group.identity.hue.is_none() {
                continue;
            }
            let color = Self::group_color(group, colors, accents, resources)?;
            entries.push(LegendEntry::point_fill(
                chart.collective_legend_identity(&group.identity, data)?,
                chart.point_style(theme, color, theme.marks.point_sizes.marginal),
                theme.marks.marginal_density.backend_style(color),
            ));
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
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = chart.data().with_axis_resources(&resources);
    for _ in &prepared.groups {
        data.claim_source_visits(1)?;
        let style = chart.point_style(theme, (0, 0, 0), theme.marks.point_sizes.marginal);
        let (x, y) = crate::render::geometry::point_ink_dimensions(theme, &style)?;
        central.0 = central.0.max(x);
        central.1 = central.1.max(y);
    }
    let density = theme.marks.marginal_density.backend_style((0, 0, 0));
    let (out_x, out_y) = crate::render::geometry::rectangular_fill_ink_outset(theme, &density)?;
    // Each filled KDE strip has a positive extent even when its paint is
    // transparent. Its stroke is measured by the same canonical geometry as
    // selection, not inferred from the nominal line width.
    Ok((
        central.0 + theme.composition_grid_gap()? + (1.0 / 1024.0_f64).max(2.0 * out_x),
        central.1 + theme.composition_grid_gap()? + (1.0 / 1024.0_f64).max(2.0 * out_y),
    ))
}

#[test]
fn prepared_marginal_points_and_density_strokes_enlarge_linked_tracks() {
    let source = Plot::marginal(
        crate::NumericValues::new("x", [1.0, 1.2, 1.4, 3.0, 3.2, 3.4]),
        crate::NumericValues::new("y", [2.0, 2.1, 2.3, 4.0, 4.2, 4.3]),
    )
    .erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
    let chart = inputs.attach(source).unwrap();
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let baseline = intrinsic_body_minimum(&chart, &theme).unwrap();
    assert!(
        matches!(chart.measure_selected_interior(PlotRenderContext::new(
        chart.data(), PanelBox::new(0.0, baseline.0 - 1.0 / 1024.0,
            0.0, baseline.1), &theme,
    )), Err(crate::layout::fit::PlanError::Fit(failure))
        if failure.reason.kind == crate::layout::fit::FitKind::SpecializedGeometry)
    );
    let mut styled = theme.clone();
    styled.marks.point_sizes.marginal = 24;
    styled.marks.marginal_density.outline =
        Some(crate::theme::DataOutlineTheme::mapped(120.0, 255).unwrap());
    let expanded = intrinsic_body_minimum(&chart, &styled).unwrap();
    assert!(expanded.0 > baseline.0);
    assert!(expanded.1 > baseline.1);
    let mut point_only = theme;
    point_only
        .marks
        .primary_point
        .outline
        .as_mut()
        .unwrap()
        .width_ratio = crate::theme::RelativeSpacing::ratio(15, 1).unwrap();
    let shown = intrinsic_body_minimum(&chart, &point_only).unwrap().0;
    point_only
        .marks
        .primary_point
        .outline
        .as_mut()
        .unwrap()
        .color = crate::theme::Color::rgba(0, 0, 0, 0);
    assert!(shown > intrinsic_body_minimum(&chart, &point_only).unwrap().0);
}

#[test]
fn wide_finite_scales_normalize_each_linked_kde_to_its_actual_peak() {
    let chart = Plot::marginal(
        crate::NumericValues::new("x", [1e14, 1.2e14, 1.4e14, 3e14, 3.2e14, 3.4e14]),
        crate::NumericValues::new("y", [2.0, 2.1, 2.3, 4.0, 4.2, 4.3]),
    )
    .erase();
    let inputs = crate::plots::shared_x::prepare_plot(
        &chart,
        &crate::font::canonical::OperationGenerationResources::new(),
        None,
    )
    .unwrap();
    let chart = inputs.attach(chart).unwrap();
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            PanelBox::new(70.0, 670.0, 60.0, 450.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Marginal(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("selected Marginal")
    };
    let strip = payload.clips()[1];
    let peak = payload
        .groups
        .iter()
        .flat_map(|group| group.top.iter())
        .map(|point| point.y)
        .fold(f64::NEG_INFINITY, f64::max);
    let density = theme.marks.marginal_density.backend_style((0, 0, 0));
    let ink = crate::render::geometry::rectangular_fill_ink_outset(&theme, &density).unwrap();
    let expected =
        strip.y0 + ink.1 + 2.0 / 1024.0 + 0.90 * (strip.height() - 2.0 * ink.1 - 4.0 / 1024.0);
    assert!((peak - expected).abs() < 1.0 / 1024.0);
}

#[test]
fn unrepresentable_linked_kde_rejects_instead_of_substituting_a_density_floor() {
    let chart = Plot::marginal(
        crate::NumericValues::new("x", [1.0, 1.2, 1.4, 3.0, 3.2, 3.4]),
        crate::NumericValues::new("y", [2.0, 2.1, 2.3, 4.0, 4.2, 4.3]),
    )
    .x_limits(1e9, 1e9 + 1.0)
    .erase();
    let inputs = crate::plots::shared_x::prepare_plot(
        &chart,
        &crate::font::canonical::OperationGenerationResources::new(),
        None,
    )
    .unwrap();
    let chart = inputs.attach(chart).unwrap();
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    assert!(
        matches!(chart.measure_selected_interior(PlotRenderContext::new(
        chart.data(), PanelBox::new(70.0, 670.0, 60.0, 450.0), &theme,
    )), Err(crate::layout::fit::PlanError::Fit(failure))
        if failure.reason.kind == crate::layout::fit::FitKind::SpecializedGeometry)
    );
}

#[test]
fn marginal_central_points_request_only_automatic_numeric_edges() {
    let source = |exact| {
        let plot = Plot::marginal(
            crate::NumericValues::new("x", [1.0, 1.2, 1.4, 3.0, 3.2, 3.4]),
            crate::NumericValues::new("y", [2.0, 2.1, 2.3, 4.0, 4.2, 4.3]),
        )
        .size(1400, 1400);
        if exact {
            plot.x_limits(1.0, 3.4).erase()
        } else {
            plot.erase()
        }
    };
    let mut theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    theme.marks.point_sizes.marginal = 350;
    for exact in [false, true] {
        let plot = source(exact);
        let inputs = crate::plots::shared_x::prepare_plot(
            &plot,
            &crate::font::canonical::OperationGenerationResources::new(),
            None,
        )
        .unwrap();
        let chart = inputs.attach(plot).unwrap();
        let selected = chart
            .measure_selected_interior(PlotRenderContext::new(
                chart.data(),
                PanelBox::new(70.0, 1000.0, 60.0, 1000.0),
                &theme,
            ))
            .unwrap();
        let crate::layout::interior::MarkPayload::Marginal(payload) =
            &selected.marks.as_ref().unwrap().payload
        else {
            panic!("selected Marginal")
        };
        assert_eq!(payload.domain_expansions[0].is_none(), exact);
        assert!(payload.domain_expansions[1].is_some());
        if !exact {
            let final_plan = crate::decorations::plan::plan_panel_for_test(
                &chart,
                crate::PlotKind::Marginal,
                PanelBox::new(70.0, 1300.0, 60.0, 1250.0),
                crate::decorations::plan::LegendRole::Panel,
                &theme,
            )
            .unwrap();
            let crate::layout::interior::MarkPayload::Marginal(final_payload) =
                &final_plan.result.marks.as_ref().unwrap().payload
            else {
                panic!("selected Marginal after feedback")
            };
            assert!(final_payload.domain_expansions.iter().all(Option::is_none));
        }
    }
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
    points: Vec<Point2D>,
    source_rows: Vec<usize>,
    point_style: crate::render::backend::PointStyle,
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
    data: crate::data::PlotData<'_>,
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
    let resources = data.source_resources()?;
    resources
        .claim_slots(1)
        .map_err(|error| Error::Render(format!("Marginal subpanel record: {error:?}")))?;
    data.claim_source_bytes(std::mem::size_of::<LinkedPanelPlan>())?;
    context
        .theme
        .prepared_font_environment()?
        .claim_measurement_call()
        .map_err(|error| Error::Render(format!("Marginal subpanel call: {error:?}")))?;
    let address = context.fit_scope().address.child(role, 0, resources)?;
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
                backend.draw_scatter(&group.points, &group.point_style)
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
    let density = theme.marks.marginal_density.backend_style((0, 0, 0));
    let ink = crate::render::geometry::rectangular_fill_ink_outset(theme, &density)?;
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
    let density = theme.marks.marginal_density.backend_style((0, 0, 0));
    let ink = crate::render::geometry::rectangular_fill_ink_outset(theme, &density)?;
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
    let resources = theme.prepared_font_environment()?.generation_resources();
    let columns = layout::interior::tracks::assign(
        &[
            track(central_hard.0, preference.body_quanta.0)?,
            track(right_hard, quarter)?,
        ],
        (0.0, width),
        gap,
        false,
        chart.data().with_axis_resources(&resources),
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
    #[cfg(test)]
    count(2);
    let theme = context.theme;
    let prepared = chart
        .prepared_marginal
        .as_ref()
        .ok_or_else(|| Error::Render("marginal requires root-prepared semantics".into()))?;
    let groups = &prepared.groups;
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = context.data.with_axis_resources(&resources);

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
    let density = theme.marks.marginal_density.backend_style((0, 0, 0));
    let ink = crate::render::geometry::rectangular_fill_ink_outset(theme, &density)?;
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
        data,
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
        data,
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
            data.copy_source_string(
                &chart
                    .prepared_axis_inputs()
                    .ok_or_else(|| Error::Render("Marginal axes missing".into()))?
                    .labels[0],
            )?,
            data.copy_source_string(
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
    data.reserve_source(
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
    for density in x_densities.iter().chain(y_densities) {
        data.claim_source_visits(density.len())?;
    }
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
            data,
        )?,
        measure_linked_panel(
            top_panel,
            [ranges[0], (0.0, 1.0)],
            layout::AddressRole::MarginalTop,
            context,
            data,
        )?,
        measure_linked_panel(
            right_panel,
            [(0.0, 1.0), ranges[1]],
            layout::AddressRole::MarginalRight,
            context,
            data,
        )?,
    ];
    let mut selected = Vec::new();
    data.reserve_source(&mut selected, groups.len())?;
    let (colors, accents) = prepared.resolved_colors(chart, theme, data)?;
    let legend_entries =
        prepared.guide_entries_with_colors(chart, theme, data, &colors, &accents)?;
    let mut source = [(f64::INFINITY, f64::NEG_INFINITY); 2];
    let mut point_ink: Option<crate::render::geometry::GeometryBox> = None;
    for (gi, group) in groups.iter().enumerate() {
        let color = PreparedMarginal::group_color(group, &colors, &accents, &resources)?;
        let point_style = chart.point_style(theme, color, theme.marks.point_sizes.marginal);
        if let Some(ink) = crate::render::geometry::point_ink_bounds(theme, &point_style)? {
            point_ink = Some(point_ink.map_or(ink, |previous| previous.union(ink)));
        }
        let pts = data.collect_source(group.x.iter().zip(&group.y).map(|(&x, &y)| {
            data.claim_source_visits(1)?;
            source[0].0 = source[0].0.min(x);
            source[0].1 = source[0].1.max(x);
            source[1].0 = source[1].0.min(y);
            source[1].1 = source[1].1.max(y);
            let (x, y) = panels[0].projection.project(x, y);
            Ok(Point2D::new(x, y))
        }))?;
        let mut top_poly = Vec::new();
        let mut right_poly = Vec::new();
        data.reserve_source(&mut top_poly, n_grid + 2)?;
        data.reserve_source(&mut right_poly, n_grid + 2)?;
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
            data.claim_source_visits(2)?;
            let (x, _) = panels[1].projection.project(curves.x_grid[i], 0.0);
            let y = baseline_y + x_densities[gi][i] / x_dens_max * 0.90 * top_span;
            top_poly.push(Point2D::new(x, y));
            let (_, y) = panels[2].projection.project(0.0, y_grid[i]);
            let x = baseline_x + y_densities[gi][i] / y_dens_max * 0.90 * right_span;
            right_poly.push(Point2D::new(x, y));
        }
        let (top_end, _) = panels[1].projection.project(curves.x_grid[n_grid - 1], 0.0);
        let (_, right_end) = panels[2].projection.project(0.0, curves.y_grid[n_grid - 1]);
        top_poly.push(Point2D::new(top_end, baseline_y));
        right_poly.push(Point2D::new(baseline_x, right_end));
        let fill = theme.marks.marginal_density.backend_style(color);
        selected.push(MarginalGroupPayload {
            points: pts,
            source_rows: data.collect_source(group.source_rows.iter().copied().map(Ok))?,
            point_style,
            top: top_poly,
            right: right_poly,
            fill,
        });
    }
    let mut domain_expansions = if let Some(ink) = point_ink {
        crate::layout::interior::automatic_mark_ink_expansions(
            source,
            proj,
            ink,
            [
                chart.user_x_limits().is_none(),
                chart.user_y_limits().is_none(),
            ],
            data,
            context.fit_scope(),
        )?
    } else {
        [None; 2]
    };
    for (side, panel) in [(0, &panels[1]), (1, &panels[2])] {
        let mut full = crate::render::geometry::GeometrySnapshot::new(theme)?;
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
                data,
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
    let mut geometry = crate::render::geometry::GeometrySnapshot::new(theme)?;
    if let Some(style) = &payload.tick_style {
        for tick in &payload.ticks {
            geometry.line(tick, style, None)?;
        }
    }
    for group in &payload.groups {
        geometry.scatter(&group.points, &group.point_style, central_panel)?;
        geometry.polygon(&group.top, &group.fill, Some(top_panel))?;
        geometry.polygon(&group.right, &group.fill, Some(right_panel))?;
    }
    let mut result = PlotRenderResult::new(layout, legend_entries);
    result.axes = Some(result.layout.measure_axes_with_inputs(
        theme,
        context.fit_scope(),
        chart.axis_semantic_inputs([crate::spec::AxisDomain::Continuous; 2]),
        || layout::StandardPanelInputs::new(theme),
    )?);
    result.marks = Some(data.source_arc(crate::layout::interior::SelectedMarkPlan {
        payload: crate::layout::interior::MarkPayload::Marginal(payload),
        geometry,
    })?);
    Ok(result)
}

pub(crate) fn prepare(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<PreparedMarginal, Error> {
    #[cfg(test)]
    count(0);
    data.source_resources()?
        .claim_topology_slots(3)
        .map_err(|error| {
            Error::Render(format!("Marginal declared subpanel topology: {error:?}"))
        })?;
    let (groups, hues) = build_groups(chart, data)?;
    if let Some(exact) = chart.categorical_colors_spec() {
        for hue in &hues {
            if exact.color_in(hue, data.source_resources()?)?.is_none() {
                return Err(Error::Data(format!(
                    "marginal: category {hue:?} has no caller-owned color"
                )));
            }
        }
    }
    let ranges = ranges_from_groups(chart, data, &groups)?;
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
    data: crate::data::PlotData<'_>,
) -> Result<PreparedMarginal, Error> {
    #[cfg(test)]
    count(1);
    if let Some(limits) = x_limits {
        prepared.ranges[0] = limits;
    }
    prepared.curves = render_curves(&prepared.groups, prepared.ranges, data)?;
    Ok(prepared)
}

fn render_curves(
    groups: &[HueGroup],
    ranges: [(f64, f64); 2],
    data: crate::data::PlotData<'_>,
) -> Result<MarginalCurves, Error> {
    let [(x_low, x_high), (y_low, y_high)] = ranges;
    // --- KDE grid (shared across groups) ---
    let n_grid = 80;
    let x_grid = data.collect_source((0..n_grid).map(|i| {
        data.claim_source_visits(1)?;
        Ok(x_low + (x_high - x_low) * (i as f64 / (n_grid - 1) as f64))
    }))?;
    let y_grid = data.collect_source((0..n_grid).map(|i| {
        data.claim_source_visits(1)?;
        Ok(y_low + (y_high - y_low) * (i as f64 / (n_grid - 1) as f64))
    }))?;

    // Compute per-group KDE densities to find the shared max.
    let mut x_densities = data.collect_source(groups.iter().map(|group| {
        gaussian_kde_with_source(&group.x, &x_grid, group.x_bandwidth, Some(data)).map_err(
            |error| {
                if !matches!(error, Error::Data(_)) {
                    return error;
                }
                Error::Data(format!(
                    "marginal: x distribution for {}: {error}",
                    group.identity.description(group.collapsed)
                ))
            },
        )
    }))?;
    let mut y_densities = data.collect_source(groups.iter().map(|group| {
        gaussian_kde_with_source(&group.y, &y_grid, group.y_bandwidth, Some(data)).map_err(
            |error| {
                if !matches!(error, Error::Data(_)) {
                    return error;
                }
                Error::Data(format!(
                    "marginal: y distribution for {}: {error}",
                    group.identity.description(group.collapsed)
                ))
            },
        )
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
    data: Option<crate::data::PlotData<'_>>,
) -> Result<(), Error> {
    use crate::data::source::SourceAdmission;
    if range.0.is_finite() && range.1.is_finite() && range.0 < range.1 {
        return Ok(());
    }
    let mut invalid = None;
    for group in groups {
        let samples = values(group);
        data.source_visits(samples.len())?;
        let wide = crate::data::min_max(samples.iter().copied(), "marginal")
            .is_ok_and(|(minimum, maximum)| !(maximum - minimum).is_finite());
        if !wide {
            data.source_visits(samples.len())?;
            if samples.len() >= 2 {
                data.source_visits(samples.len())?;
                data.source_visits(samples.len())?;
            }
        }
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

fn ranges_from_groups(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
    groups: &[HueGroup],
) -> Result<[(f64, f64); 2], Error> {
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
        for group in groups {
            data.claim_source_visits(values(group).len())?;
        }
        let range = crate::data::min_max(
            groups
                .iter()
                .flat_map(|group| values(group).iter().copied()),
            "marginal",
        )?;
        ranges[index] = marginal_display_range(groups.iter().map(values), range, spec, Some(data))?;
        validate_marginal_range(
            groups,
            ranges[index],
            values,
            if index == 0 { "x" } else { "y" },
            Some(data),
        )?;
    }
    Ok(ranges)
}

pub(crate) fn shared_x_domain(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<(f64, f64), Error> {
    let (groups, _) = build_groups(chart, data)?;
    for group in &groups {
        data.claim_source_visits(group.x.len())?;
    }
    let range = crate::data::min_max(
        groups.iter().flat_map(|group| group.x.iter().copied()),
        "marginal",
    )?;
    let range = marginal_display_range(
        groups.iter().map(|group| group.x.as_slice()),
        range,
        chart.x_axis_spec(),
        Some(data),
    )?;
    validate_marginal_range(&groups, range, |group| &group.x, "x", Some(data))?;
    Ok(range)
}

fn build_groups<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
) -> Result<(Vec<HueGroup>, Vec<String>), Error> {
    let data = data.into();

    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("marginal: x column not set".into()))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("marginal: y column not set".into()))?;
    let rows = crate::data::aligned_finite_rows(data, x_col, y_col, "marginal")?;
    data.claim_mark_rows()?;
    let retained_rows = data.collect_source(rows.iter().map(|(row, _, _)| Ok(*row)))?;
    let channels = crate::data::observed_collectives(
        data,
        chart.group_col(),
        chart.group_name(),
        chart.hue_col(),
        chart.hue_name(),
        Some(&retained_rows),
        "marginal",
    )?;
    let hue_levels = data.collect_source(
        chart
            .global_hue_levels()
            .unwrap_or(&channels.hue_levels)
            .iter()
            .map(|level| data.copy_source_string(level)),
    )?;
    let mut groups = Vec::new();
    data.claim_topology_slots(
        channels
            .identities
            .len()
            .checked_mul(2)
            .ok_or_else(|| Error::Render("Marginal density topology overflow".into()))?,
    )?;
    data.reserve_source(&mut groups, channels.identities.len())?;
    for identity in &channels.identities {
        data.claim_source_visits(rows.len())?;
        let count = channels.rows.iter().filter(|row| *row == identity).count();
        let selected = rows
            .iter()
            .zip(&channels.rows)
            .filter(|(_, row_identity)| *row_identity == identity);
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut source_rows = Vec::new();
        data.reserve_source(&mut x, count)?;
        data.reserve_source(&mut y, count)?;
        data.reserve_source(&mut source_rows, count)?;
        data.claim_source_visits(rows.len())?;
        for ((row, x_value, y_value), _) in selected {
            x.push(*x_value);
            y.push(*y_value);
            source_rows.push(data.source_row(*row));
        }
        if x.is_empty() {
            continue;
        }
        for _ in 0..5 {
            data.claim_source_visits(x.len())?;
            data.claim_source_visits(y.len())?;
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
            data.claim_source_visits(hue_levels.len())?;
            if !hue_levels.contains(hue) {
                return Err(Error::Data(format!(
                    "marginal: hue {hue:?} is absent from the prepared domain"
                )));
            }
        }
        groups.push(HueGroup {
            identity: data.copy_source_pair(identity)?,
            collapsed: channels.collapsed,
            x,
            y,
            source_rows,
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
    data: Option<crate::data::PlotData<'_>>,
) -> Result<(f64, f64), Error> {
    let data_display_range = marginal_data_display_range(data_range, spec);
    if spec.limits.is_some() {
        return Ok(data_display_range);
    }

    let Some(visible_kde_range) = kde_visible_support_range(
        groups,
        MARGINAL_KDE_CUT,
        MARGINAL_KDE_VISIBLE_FRACTION,
        data,
    )?
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
    data: Option<crate::data::PlotData<'_>>,
) -> Result<Option<(f64, f64)>, Error> {
    use crate::data::source::SourceAdmission;
    for samples in groups.clone() {
        data.source_visits(samples.len())?;
        if samples.len() >= 2 {
            data.source_visits(samples.len())?;
            data.source_visits(samples.len())?;
        }
    }
    let Some((support_low, support_high)) = kde_support_range(groups.clone(), cut) else {
        return Ok(None);
    };
    data.source_visits(MARGINAL_KDE_GRID_POINTS)?;
    let grid: [f64; MARGINAL_KDE_GRID_POINTS] = std::array::from_fn(|i| {
        support_low
            + (support_high - support_low) * (i as f64 / (MARGINAL_KDE_GRID_POINTS - 1) as f64)
    });

    let mut low = f64::INFINITY;
    let mut high = f64::NEG_INFINITY;
    for samples in groups {
        let densities = match crate::stats::distribution::kde_with_source(samples, &grid, data) {
            Ok(densities) => densities,
            Err(Error::Data(_)) => continue,
            Err(error) => return Err(error),
        };
        data.source_visits(densities.len())?;
        let max_density = densities.iter().fold(0.0_f64, |acc, value| acc.max(*value));
        if max_density <= 0.0 {
            continue;
        }
        let threshold = max_density * visible_fraction;
        data.source_visits(grid.len())?;
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
mod extraction;

#[cfg(test)]
#[test]
fn visible_support_propagates_resource_refusal_instead_of_skipping_the_group() {
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [1.0]),
        crate::NumericValues::new("y", [1.0]),
    )
    .erase();
    for (visits, bytes, expected) in [
        (0, 1000, "work admission"),
        (100_000, 0, "storage admission"),
    ] {
        let resources =
            crate::font::canonical::OperationGenerationResources::with_test_limits(visits, bytes);
        let error = kde_visible_support_range(
            std::iter::once(&[1.0, 2.0, 3.0][..]),
            MARGINAL_KDE_CUT,
            MARGINAL_KDE_VISIBLE_FRACTION,
            Some(plot.data().with_axis_resources(&resources)),
        )
        .unwrap_err();
        assert!(matches!(error, Error::Render(_)), "{error}");
        assert!(error.to_string().contains(expected), "{error}");
    }
    let samples = [1.0, 2.0, 3.0];
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let calculate = |data| {
        kde_visible_support_range(
            std::iter::once(samples.as_slice()),
            MARGINAL_KDE_CUT,
            MARGINAL_KDE_VISIBLE_FRACTION,
            data,
        )
        .unwrap()
    };
    assert_eq!(
        calculate(None),
        calculate(Some(plot.data().with_axis_resources(&resources)))
    );
}
