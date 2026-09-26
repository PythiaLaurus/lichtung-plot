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
    stops: [GradientStop; 2],
    color_range: (f64, f64),
    values: [String; 2],
    pub(crate) color_label: String,
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
    let mut values = [String::new(), String::new()];
    for (label, value) in values.iter_mut().zip([color_range.1, color_range.0]) {
        use std::fmt::Write;
        data.reserve_source_string(label, 32)?;
        if value.abs() >= 1_000_000.0 || (value.abs().to_bits() != 0 && value.abs() < 0.0001) {
            write!(label, "{value:.3e}").expect("reserved colorbar label");
        } else {
            write!(label, "{value:.3}").expect("reserved colorbar label");
        }
    }
    Ok(PreparedBeeswarm {
        levels,
        x_data,
        rows,
        points: Vec::new(),
        values,
        color_range,
        stops: [
            GradientStop::new(0.0, continuous_color(color_range.0, color_range)),
            GradientStop::new(1.0, continuous_color(color_range.1, color_range)),
        ],
        color_label: chart.prepare_color_label(data, None)?,
    })
}

impl PreparedBeeswarm {
    pub(crate) fn guide_semantics(
        &self,
    ) -> crate::decorations::legend::descriptor::ContinuousSemantics {
        crate::decorations::legend::descriptor::ContinuousSemantics {
            domain: self.color_range,
            center: None,
            palette: crate::decorations::legend::descriptor::Palette {
                low: self.stops[0].color,
                center: None,
                high: self.stops[1].color,
            },
            missing: None,
        }
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
                continuous_color(color, self.color_range),
            ));
        }
        Ok(())
    }

    pub(crate) fn guide_labels(&self) -> impl Iterator<Item = &str> {
        self.values.iter().map(String::as_str)
    }
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
    let [high_label, low_label] = &prepared.values;
    let color_label = &prepared.color_label;
    let guide_scope = context
        .fit_scope()
        .child(crate::layout::AddressRole::OutsideGuideRight, 0)?;
    let color_label_block =
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
    let colorbar_reserve = (34.0 + value_width).max(
        65.0 + color_label_block.lines().len().saturating_sub(1) as f64
            * color_label_block.line_height()
            + f64::from(theme.typography.legend_label.font_size) / 2.0,
    ) + 4.0;
    if context.panel.width() - colorbar_reserve < 100.0 {
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
            LineStyle::solid(theme.foreground, 0.8),
        ))
    } else {
        None
    };
    let points = data.collect_source(prepared.points.iter().map(|&(x, y, color)| {
        data.claim_source_visits(1)?;
        let (x, y) = layout.projection.project(x, y);
        Ok((Point2D::new(x, y), chart.point_style(color, 3)))
    }))?;
    let payload = BeeswarmPayload {
        clip: layout.projection.panel_box(),
        zero,
        points,
        colorbar: select_colorbar(&layout, &color_label_block, &values, prepared.stops, data)?,
    };
    let geometry = payload.geometry(theme)?;
    let mut result = PlotRenderResult::new(layout, Vec::new());
    result.y_domain = crate::spec::AxisDomain::Categorical;
    result.producer_text =
        data.collect_source(payload.colorbar.text.iter().map(|text| Ok(text.clone())))?;
    result.marks = Some(data.source_arc(crate::layout::interior::DeferredMarkPlan {
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
    colorbar: ColorbarPayload,
}

#[derive(Debug)]
struct ColorbarPayload {
    bounds: PanelBox,
    start: Point2D,
    end: Point2D,
    stops: [GradientStop; 2],
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
        let bar = &self.colorbar;
        backend.draw_linear_gradient(bar.bounds, bar.start, bar.end, &bar.stops)?;
        for text in &bar.text {
            text.draw(backend)?;
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
        let bar = &self.colorbar;
        geometry.linear_gradient(bar.bounds, bar.start, bar.end, &bar.stops, None)?;
        for text in &bar.text {
            geometry.text(text.placement, &text.plan, None)?;
        }
        Ok(geometry)
    }
}

fn select_colorbar(
    layout: &Layout,
    label: &crate::layout::TextBlock,
    values: &[crate::layout::ResolvedTextPlan],
    stops: [GradientStop; 2],
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

fn continuous_color(value: f64, range: (f64, f64)) -> (u8, u8, u8) {
    let span = range.1 - range.0;
    let t = if span.to_bits() == 0 {
        0.5
    } else if value <= range.0 {
        0.0
    } else if value >= range.1 {
        1.0
    } else if span.is_finite() {
        (value - range.0) / span
    } else {
        let scale = range.0.abs().max(range.1.abs());
        let minimum = range.0 / scale;
        let maximum = range.1 / scale;
        (value / scale - minimum) / (maximum - minimum)
    };
    let t = t.clamp(0.0, 1.0);
    let low = (49.0, 130.0, 189.0);
    let high = (222.0, 45.0, 38.0);
    (
        (low.0 + (high.0 - low.0) * t).round() as u8,
        (low.1 + (high.1 - low.1) * t).round() as u8,
        (low.2 + (high.2 - low.2) * t).round() as u8,
    )
}

#[cfg(test)]
mod extraction;

#[cfg(test)]
mod fit_tests {
    use super::*;
    use crate::layout::fit::{FitKind, FitOutcome, FitScope};
    use crate::layout::{AddressRole, ComponentAddress};

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
