//! Categorical beeswarm plot with a continuous color scale.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]

use crate::layout::{Layout, PanelBox, PhysicalTextAlignment, TextBlock};
use crate::render::backend::{
    with_clip, Backend, GradientStop, LineStyle, PlacedText, Point2D, TextAnchor,
};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::Error;
use crate::Plot;

#[derive(Debug)]
pub(crate) struct PreparedBeeswarmInputs {
    pub(crate) panels: Vec<(Option<String>, PreparedBeeswarm)>,
}

#[derive(Debug)]
pub(crate) struct PreparedBeeswarm {
    pub(crate) levels: Vec<String>,
    pub(crate) x_data: (f64, f64),
    rows: Vec<(usize, f64, f64, usize)>,
    points: Vec<(f64, f64, (u8, u8, u8))>,
    scale: crate::theme::ContinuousColors,
    colorbar_stops: Vec<GradientStop>,
    values: [String; 2],
    pub(crate) color_label: String,
}

pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let prepared = chart
        .prepared_beeswarm
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, marks)| marks)
        .ok_or_else(|| {
            Error::Render("Beeswarm intrinsic requires root-prepared source rows".into())
        })?;
    if prepared.levels.is_empty()
        || prepared.rows.is_empty()
        || prepared.rows.len() != prepared.points.len()
    {
        return Err(Error::Render(
            "Beeswarm source rows and prepared points differ".into(),
        ));
    }
    let point = chart.point_style(theme, (0, 0, 0), theme.marks.point_sizes.beeswarm);
    let ink = crate::render::geometry::point_ink_bounds(theme, &point)?;
    let resources = theme.prepared_font_environment()?.generation_resources();
    resources
        .claim_visits(prepared.rows.len() as u128)
        .map_err(|error| {
            Error::Render(format!("Beeswarm occupied category admission: {error:?}"))
        })?;
    let mut pitch = 1.0_f64 / 1024.0;
    for (&(category, _, _, _), &(_, y, _)) in prepared.rows.iter().zip(&prepared.points) {
        if category >= prepared.levels.len() {
            return Err(Error::Render(
                "Beeswarm point category exceeds prepared levels".into(),
            ));
        }
        let offset = (y - (category as f64 + 0.5)).abs();
        if offset >= 0.5 {
            return Err(Error::Render(
                "Beeswarm point exceeds its prepared category slot".into(),
            ));
        }
        if let Some(ink) = ink {
            pitch = pitch.max(ink.top.max(-ink.bottom) / (0.5 - offset));
        }
    }
    let count = u32::try_from(prepared.levels.len())
        .map_err(|_| Error::Render("Beeswarm category count exceeds spatial range".into()))?;
    let height = pitch * f64::from(count);
    if !height.is_finite() {
        return Err(Error::Render(
            "Beeswarm category hard span overflowed".into(),
        ));
    }
    let mut numeric = ink.map_or(0.0, |ink| ink.right - ink.left);
    if chart.zero_line_visible() && prepared.x_data.0 <= 0.0 && prepared.x_data.1 >= 0.0 {
        numeric = numeric.max(crate::render::geometry::line_ink_width(
            theme,
            &theme.marks.beeswarm_reference.backend_style(),
        )?);
    }
    Ok((
        if chart.user_x_limits().is_none() {
            numeric
        } else {
            0.0
        },
        height,
    ))
}

#[cfg(test)]
thread_local! {
    pub(crate) static STAGES: std::cell::Cell<[usize; 3]> = const { std::cell::Cell::new([0; 3]) };
}

#[cfg(test)]
fn count(stage: usize) {
    STAGES.with(|counts| {
        let mut next = counts.get();
        next[stage] += 1;
        counts.set(next);
    });
}

pub(crate) fn prepare(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<PreparedBeeswarm, Error> {
    #[cfg(test)]
    count(0);
    let category_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("beeswarm: categorical row column not set".into()))?;
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("beeswarm: x column not set".into()))?;
    let color_col = chart
        .color_col()
        .ok_or_else(|| Error::Data("beeswarm: continuous color column not set".into()))?;
    let categories = crate::data::strict_strings(data, category_col, "beeswarm")?;
    let xs = crate::data::strict_finite_f64(data, x_col, "beeswarm")?;
    let colors = crate::data::strict_finite_f64(data, color_col, "beeswarm")?;
    if categories.len() != xs.len() || xs.len() != colors.len() || xs.is_empty() {
        return Err(Error::Data(
            "beeswarm: category/x/color columns must have equal non-zero lengths".into(),
        ));
    }
    let levels = data.source_levels(&categories)?;
    data.claim_source_visits(xs.len())?;
    let x_data = crate::data::min_max(xs.iter().copied(), "beeswarm")?;
    let color_range = if let Some((minimum, maximum)) = chart.beeswarm_color_limits_spec() {
        if !minimum.is_finite() || !maximum.is_finite() || minimum >= maximum {
            return Err(Error::Data(format!(
                    "beeswarm: color limits must be finite and strictly ascending; got {minimum}..{maximum}"
                )));
        }
        (minimum, maximum)
    } else {
        data.claim_source_visits(colors.len())?;
        crate::data::min_max(colors.iter().copied(), "beeswarm")?
    };
    let mut indices =
        data.collect_source(levels.iter().enumerate().map(|(i, s)| Ok((s.as_str(), i))))?;
    crate::data::source::sort_source_by(&mut indices, Some(data), |a, b| a.0.cmp(b.0))?;
    let mut rows = data.collect_source(categories.iter().zip(&xs).zip(&colors).enumerate().map(
        |(row, ((category, x), color))| {
            data.claim_source_visits(indices.len().ilog2() as usize + 2)?;
            let index = indices
                .binary_search_by_key(&category.as_str(), |entry| entry.0)
                .map_err(|_| Error::Data("beeswarm: category identity missing".into()))?;
            Ok((indices[index].1, *x, *color, row))
        },
    ))?;
    crate::data::source::sort_source_by(&mut rows, Some(data), |a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.total_cmp(&b.1))
            .then_with(|| a.2.total_cmp(&b.2))
            .then_with(|| a.3.cmp(&b.3))
    })?;
    let values = colorbar_values(color_range, data)?;
    let scale = crate::theme::ContinuousColors::resolve(
        color_range,
        &[
            GradientStop::new(0.0, (49, 130, 189)),
            GradientStop::new(1.0, (222, 45, 38)),
        ],
        data.source_resources()?,
    )?;
    let samples = data.collect_source(rows.iter().map(|row| Ok(row.2)))?;
    let colorbar_stops = scale.guide_stops(&samples, data)?;
    Ok(PreparedBeeswarm {
        levels,
        x_data,
        rows,
        points: Vec::new(),
        values,
        scale,
        colorbar_stops,
        color_label: chart.prepare_color_label(data, None)?,
    })
}

impl PreparedBeeswarm {
    pub(crate) fn guide_samples(&self, data: crate::data::PlotData<'_>) -> Result<Vec<f64>, Error> {
        data.collect_source(self.rows.iter().map(|row| Ok(row.2)))
    }

    fn guide_stops(&self, data: crate::data::PlotData<'_>) -> Result<Vec<GradientStop>, Error> {
        data.collect_source(self.colorbar_stops.iter().copied().map(Ok))
    }

    #[cfg(test)]
    pub(crate) fn color_for_source_value(&self, value: f64) -> Option<(u8, u8, u8)> {
        self.rows
            .iter()
            .zip(&self.points)
            .find(|(row, _)| row.2.to_bits() == value.to_bits())
            .map(|(_, point)| point.2)
    }

    pub(crate) fn install_shared_domain(
        &mut self,
        descriptor: &crate::decorations::legend::descriptor::ContinuousDescriptor,
        data: crate::data::PlotData<'_>,
    ) -> Result<(), Error> {
        let values = colorbar_values(descriptor.domain, data)?;
        let scale = self
            .scale
            .with_domain(descriptor.domain, data.source_resources()?)?;
        let colorbar_stops = scale.guide_stops(&self.guide_samples(data)?, data)?;
        self.values = values;
        self.scale = scale;
        self.colorbar_stops = colorbar_stops;
        for ((_, _, color), &(_, _, value, _)) in self.points.iter_mut().zip(&self.rows) {
            *color = self.scale.color(value, data.source_resources()?)?;
        }
        Ok(())
    }

    pub(crate) fn guide_semantics(
        &self,
        data: crate::data::PlotData<'_>,
    ) -> Result<crate::decorations::legend::descriptor::ContinuousSemantics, Error> {
        let stops = self.scale.stops();
        Ok(
            crate::decorations::legend::descriptor::ContinuousSemantics {
                domain: self.scale.domain(),
                center: None,
                stops: data.collect_source(stops.iter().copied().map(Ok))?,
                missing: None,
            },
        )
    }

    pub(crate) fn finish(
        &mut self,
        range: (f64, f64),
        data: crate::data::PlotData<'_>,
    ) -> Result<(), Error> {
        #[cfg(test)]
        count(2);
        let mut occupied: Vec<((usize, i64), usize)> = Vec::new();
        data.reserve_source(&mut occupied, self.rows.len())?;
        data.reserve_source(&mut self.points, self.rows.len())?;
        data.claim_source_visits(self.rows.len())?;
        // Preserve signed 48-bin occupancy and capped data-space offsets until T5e.
        for &(category, x, color, _) in &self.rows {
            let bin = (((x - range.0) / (range.1 - range.0)) * 48.0).floor() as i64;
            data.claim_source_visits(occupied.len().checked_ilog2().unwrap_or(0) as usize + 2)?;
            let index = match occupied.binary_search_by_key(&(category, bin), |entry| entry.0) {
                Ok(index) => index,
                Err(index) => {
                    data.claim_source_visits(occupied.len() - index)?;
                    occupied.insert(index, ((category, bin), 0));
                    index
                }
            };
            let count = &mut occupied[index].1;
            let layer = (*count).div_ceil(2);
            let direction = if (*count).is_multiple_of(2) {
                1.0
            } else {
                -1.0
            };
            let jitter = if *count == 0 {
                0.0
            } else {
                direction * (layer as f64 * 0.09).min(0.36)
            };
            *count += 1;
            self.points.push((
                x,
                category as f64 + 0.5 + jitter,
                self.scale.color(color, data.source_resources()?)?,
            ));
        }
        Ok(())
    }

    pub(crate) fn guide_labels(&self) -> impl Iterator<Item = &str> {
        self.values.iter().map(String::as_str)
    }
}

fn colorbar_values(
    range: (f64, f64),
    data: crate::data::PlotData<'_>,
) -> Result<[String; 2], Error> {
    let mut values = [String::new(), String::new()];
    for (label, value) in values.iter_mut().zip([range.1, range.0]) {
        use std::fmt::Write;
        data.reserve_source_string(label, 32)?;
        if value.abs() >= 1_000_000.0 || (value.abs().to_bits() != 0 && value.abs() < 0.0001) {
            write!(label, "{value:.3e}").expect("reserved colorbar label");
        } else {
            write!(label, "{value:.3}").expect("reserved colorbar label");
        }
    }
    Ok(values)
}

pub(crate) fn measure(
    chart: &Plot,
    context: PlotRenderContext<'_>,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    count(1);
    let prepared = chart
        .prepared_beeswarm
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, marks)| marks)
        .ok_or_else(|| Error::Render("beeswarm requires root-prepared semantics".into()))?;
    let theme = context.theme;
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = context.data.with_axis_resources(&resources);
    let text = context.producer_text()?;
    let colorbar = if chart.continuous_guide_suppressed() {
        None
    } else {
        let [high_label, low_label] = &prepared.values;
        let color_label = &prepared.color_label;
        let guide_scope = context
            .fit_scope()
            .child(crate::layout::AddressRole::OutsideGuideRight, 0)?;
        let title =
            TextBlock::from_resolved(text.plan(color_label, "beeswarm colorbar title", || {
                crate::layout::fit::measured(crate::layout::fit::text(
                    theme,
                    color_label,
                    &theme.typography.legend_title,
                    context.panel.height().min(150.0),
                    PhysicalTextAlignment::Center,
                    "beeswarm colorbar title",
                    &guide_scope.child(crate::layout::AddressRole::GuideTitle, 0)?,
                ))
            })?);
        let mut values = Vec::new();
        data.reserve_source(&mut values, 2)?;
        for label in [high_label, low_label] {
            values.push(text.plan(label, "beeswarm colorbar value", || {
                Ok(theme.measure_legacy_appearance_text(
                    label,
                    &theme.typography.legend_label,
                    PhysicalTextAlignment::Start,
                    "beeswarm colorbar value",
                    false,
                )?)
            })?);
        }
        let value_width = values
            .iter()
            .map(crate::layout::ResolvedTextPlan::width)
            .fold(0.0_f64, f64::max);
        let reserve = (34.0 + value_width).max(
            65.0 + title.lines().len().saturating_sub(1) as f64 * title.line_height()
                + f64::from(theme.typography.legend_label.font_size) / 2.0,
        ) + 4.0;
        Some((title, values, reserve))
    };
    let colorbar_reserve = colorbar.as_ref().map_or(0.0, |(_, _, reserve)| *reserve);
    if context.panel.width() - colorbar_reserve < 100.0 {
        let guide_scope = context
            .fit_scope()
            .child(crate::layout::AddressRole::OutsideGuideRight, 0)?;
        return Err(guide_scope.failure(crate::layout::fit::FitKind::LegendOrGuide,
            "beeswarm: colorbar title does not fit while preserving the minimum 100px plot panel; increase chart width"
        ).into());
    }
    let mut body = context.panel;
    body.x1 -= colorbar_reserve;
    let panel = body;
    let layout = context.axis_layout(|| {
        context.layout_with_axis_specs(
            prepared.x_data,
            (0.0, prepared.levels.len() as f64),
            panel,
            data.copy_source_string(chart.x_label_text().unwrap_or(""))?,
            data.copy_source_string(chart.y_label_text().unwrap_or(""))?,
            chart.x_axis_spec(),
            chart.y_axis_spec(),
            theme,
        )
    })?;
    let x_range = layout.projection.x_range();
    let zero = if chart.zero_line_visible() && x_range.0 <= 0.0 && x_range.1 >= 0.0 {
        let (x, y0) = layout.projection.project(0.0, 0.0);
        let (_, y1) = layout.projection.project(0.0, prepared.levels.len() as f64);
        Some((
            [Point2D::new(x, y0), Point2D::new(x, y1)],
            theme.marks.beeswarm_reference.backend_style(),
        ))
    } else {
        None
    };
    let points = data.collect_source(prepared.points.iter().map(|&(x, y, color)| {
        data.claim_source_visits(1)?;
        let (x, y) = layout.projection.project(x, y);
        Ok((
            Point2D::new(x, y),
            chart.point_style(theme, color, theme.marks.point_sizes.beeswarm),
        ))
    }))?;
    let mut payload = BeeswarmPayload {
        clip: layout.projection.panel_box(),
        zero,
        points,
        domain_expansions: [None; 2],
        colorbar: colorbar
            .as_ref()
            .map(|(title, values, _)| {
                select_colorbar(&layout, title, values, prepared.guide_stops(data)?, data)
            })
            .transpose()?,
    };
    let geometry = payload.geometry(theme)?;
    let mut full = crate::render::geometry::GeometrySnapshot::new(theme)?;
    data.claim_source_visits(payload.points.len())?;
    for (point, style) in &payload.points {
        full.scatter_with_clip(std::slice::from_ref(point), style, None)?;
    }
    let panel = layout.projection.panel_box();
    if let Some(bounds) = full
        .bounds()
        .iter()
        .copied()
        .reduce(crate::render::geometry::GeometryBox::union)
    {
        if bounds.bottom < panel.y0 || bounds.top > panel.y1 {
            return Err(context
                .fit_scope()
                .failure(
                    crate::layout::fit::FitKind::SpecializedGeometry,
                    "Beeswarm painted ink exceeds its assigned category panel",
                )
                .into());
        }
    }
    // A structural zero reference intentionally terminates at the frame;
    // its caps do not impose category-track overflow on data points.
    if let Some((line, style)) = &payload.zero {
        full.line(line, style, None)?;
    }
    if let Some(bounds) = full
        .bounds()
        .iter()
        .copied()
        .reduce(crate::render::geometry::GeometryBox::union)
    {
        if chart.user_x_limits().is_none() {
            let levels = u32::try_from(prepared.levels.len()).map_err(|_| {
                Error::Render("Beeswarm category count exceeds spatial range".into())
            })?;
            payload.domain_expansions = crate::layout::interior::automatic_absolute_ink_expansions(
                [prepared.x_data, (0.0, f64::from(levels))],
                &layout.projection,
                bounds,
                [true, false],
                data,
                context.fit_scope(),
            )?;
        }
    }
    let mut result = PlotRenderResult::new(layout, Vec::new());
    result.y_domain = crate::spec::AxisDomain::Categorical;
    if let Some(bar) = &payload.colorbar {
        result.producer_text = data.collect_source(bar.text.iter().map(|text| Ok(text.clone())))?;
    }
    result.marks = Some(data.source_arc(crate::layout::interior::SelectedMarkPlan {
        payload: crate::layout::interior::MarkPayload::Beeswarm(payload),
        geometry,
    })?);
    Ok(result)
}

#[derive(Debug)]
pub(crate) struct BeeswarmPayload {
    clip: PanelBox,
    zero: Option<([Point2D; 2], LineStyle)>,
    points: Vec<(Point2D, crate::render::backend::PointStyle)>,
    colorbar: Option<ColorbarPayload>,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

#[derive(Debug)]
struct ColorbarPayload {
    bounds: PanelBox,
    start: Point2D,
    end: Point2D,
    stops: Vec<GradientStop>,
    text: [PlacedText; 3],
}

impl BeeswarmPayload {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        with_clip(backend, self.clip, |backend| {
            if let Some((line, style)) = &self.zero {
                backend.draw_line(line, style)?;
            }
            for (point, style) in &self.points {
                backend.draw_scatter(&[*point], style)?;
            }
            Ok(())
        })?;
        if let Some(bar) = &self.colorbar {
            backend.draw_linear_gradient(bar.bounds, bar.start, bar.end, &bar.stops)?;
            for text in &bar.text {
                text.draw(backend)?;
            }
        }
        Ok(())
    }

    fn geometry(
        &self,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<crate::render::geometry::GeometrySnapshot, Error> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new(theme)?;
        let clip = crate::render::backend::validate_clip_region(self.clip)?.panel();
        if let Some((line, style)) = &self.zero {
            geometry.line(line, style, Some(clip))?;
        }
        for (point, style) in &self.points {
            geometry.scatter(&[*point], style, clip)?;
        }
        if let Some(bar) = &self.colorbar {
            geometry.linear_gradient(bar.bounds, bar.start, bar.end, &bar.stops, None)?;
            for text in &bar.text {
                geometry.text(text.placement, &text.plan, None)?;
            }
        }
        Ok(geometry)
    }
}

fn select_colorbar(
    layout: &Layout,
    label: &crate::layout::TextBlock,
    values: &[crate::layout::ResolvedTextPlan],
    stops: Vec<GradientStop>,
    data: crate::data::PlotData<'_>,
) -> Result<ColorbarPayload, Error> {
    data.claim_source_visits(3)?;
    data.claim_source_bytes(std::mem::size_of::<ColorbarPayload>())?;
    let (_, panel_right, panel_bottom, panel_top) = layout.projection.pixel_box();
    let left = panel_right + 18.0;
    let height = (panel_top - panel_bottom).min(150.0);
    let bottom = panel_bottom + (panel_top - panel_bottom - height) / 2.0;
    Ok(ColorbarPayload {
        bounds: PanelBox::new(left, left + 10.0, bottom, bottom + height),
        start: Point2D::new(left + 5.0, bottom),
        end: Point2D::new(left + 5.0, bottom + height),
        stops,
        text: [
            PlacedText::from_deferred_first_line(
                values[0].clone(),
                Point2D::new(
                    (left + 16.0).trunc(),
                    (bottom + height - 4.0).max(0.0).trunc(),
                ),
                TextAnchor::Start,
                0.0,
            ),
            PlacedText::from_deferred_first_line(
                values[1].clone(),
                Point2D::new((left + 16.0).trunc(), (bottom - 13.0).max(0.0).trunc()),
                TextAnchor::Start,
                0.0,
            ),
            PlacedText::from_deferred_first_line(
                label.plan().clone(),
                Point2D::new((left + 47.0).trunc(), (bottom + height / 2.0).trunc()),
                TextAnchor::Middle,
                -90.0,
            ),
        ],
    })
}

#[cfg(test)]
mod extraction;

#[cfg(test)]
mod fit_tests {
    use super::*;
    use crate::layout::fit::{FitKind, FitOutcome, FitScope};
    use crate::layout::{AddressRole, ComponentAddress};
    use crate::theme::ResolvedTheme;

    #[test]
    fn prepared_bin_offsets_and_complete_marker_ink_raise_category_hard_pitch() {
        let source = |exact: bool| {
            let plot = Plot::beeswarm(
                crate::CategoryValues::new(
                    "category",
                    ["a", "a", "a", "a", "a", "a", "a", "a", "b"],
                )
                .unwrap(),
                crate::NumericValues::new("x", [0.0; 9]),
                crate::NumericValues::new("color", [0.5; 9]),
            );
            if exact {
                plot.x_limits(-1.0, 1.0).erase()
            } else {
                plot.erase()
            }
        };
        let intrinsic = |source: Plot, theme: &ResolvedTheme| {
            let resources = crate::font::canonical::OperationGenerationResources::new();
            let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
            inputs
                .attach(source)
                .unwrap()
                .intrinsic_body_minimum(theme)
                .unwrap()
        };
        let theme = crate::Theme::Publication.resolve();
        let baseline = intrinsic(source(false), &theme);
        let mut styled = theme.clone();
        styled.marks.point_sizes.beeswarm = 54;
        let raised = intrinsic(source(false), &styled);
        assert!(raised.0 > baseline.0);
        assert!(raised.1 > 700.0);
        let exact = intrinsic(source(true), &styled);
        assert_eq!(
            exact.0.to_bits(),
            crate::layout::canvas::MINIMUM_PANEL_SPAN.to_bits()
        );
        assert!(exact.0 < baseline.0);
        assert_eq!(exact.1.to_bits(), raised.1.to_bits());
    }

    #[test]
    fn selected_swarm_ink_clears_numeric_edges_without_repacking_points() {
        let source = Plot::beeswarm(
            crate::CategoryValues::new("category", ["a", "a", "a", "a", "a", "a", "a", "a", "b"])
                .unwrap(),
            crate::NumericValues::new("x", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 100.0]),
            crate::NumericValues::new("color", [0.5; 9]),
        )
        .color_limits(0.0, 1.0)
        .size(1800, 6000)
        .erase();
        let mut theme = crate::Theme::Publication.resolve();
        theme.marks.point_sizes.beeswarm = 300;
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        let cell = PanelBox::new(80.0, 1750.0, 80.0, 5900.0);
        let selected = crate::decorations::plan::plan_panel_for_test(
            &chart,
            crate::PlotKind::Beeswarm,
            cell,
            crate::decorations::plan::LegendRole::Suppressed,
            &theme,
        )
        .unwrap();
        assert!(selected.planned_x_limits().is_some());
        let crate::layout::interior::MarkPayload::Beeswarm(payload) =
            &selected.result.marks.as_ref().unwrap().payload
        else {
            panic!("selected Beeswarm")
        };
        assert!(payload.domain_expansions.iter().all(Option::is_none));
        assert_eq!(payload.points.len(), 9);
        let panel = selected.result.layout.projection.panel_box();
        for (point, style) in &payload.points {
            let ink = crate::render::geometry::point_ink_bounds(&theme, style)
                .unwrap()
                .unwrap();
            assert!(point.x + ink.left >= panel.x0 && point.x + ink.right <= panel.x1);
            assert!(point.y + ink.bottom >= panel.y0 && point.y + ink.top <= panel.y1);
        }
    }

    #[test]
    fn colorbar_title_fit_preserves_its_child_address_and_zero_local_attempt() {
        let plot = Plot::beeswarm(
            crate::CategoryValues::new("category", ["one", "two"]).unwrap(),
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("color", [0.0, 1.0]),
        )
        .color_label("W".repeat(200))
        .erase();
        let inputs = crate::plots::shared_x::prepare_plot(
            &plot,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap();
        let plot = inputs.attach(plot).unwrap();
        let owner = ComponentAddress::root()
            .child(AddressRole::CanvasCell, 2)
            .unwrap();
        let scope = FitScope::new(&owner, 7);
        let theme = crate::Theme::Publication.resolve();
        theme.prepare_font_environment().unwrap();
        let result = plot
            .preflight_measurement(
                crate::spec::PlotKind::Beeswarm,
                PanelBox::new(80.0, 500.0, 80.0, 350.0),
                &theme,
                &scope,
            )
            .unwrap();
        let FitOutcome::Miss(failure) = result else {
            panic!("unbreakable colorbar title unexpectedly fitted")
        };
        assert_eq!(failure.reason.kind, FitKind::Text);
        assert_eq!(failure.reason.ordinal, 0);
        assert_eq!(
            failure.reason.address,
            owner
                .child(AddressRole::OutsideGuideRight, 0)
                .unwrap()
                .child(AddressRole::GuideTitle, 0)
                .unwrap()
        );

        let mut invalid = crate::Theme::Publication.resolve();
        invalid.typography.legend_title.font_size = 0;
        invalid.prepare_font_environment().unwrap();
        assert!(plot
            .preflight_measurement(
                crate::spec::PlotKind::Beeswarm,
                PanelBox::new(80.0, 500.0, 80.0, 350.0),
                &invalid,
                &scope
            )
            .is_err());
    }
}
