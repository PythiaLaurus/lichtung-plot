//! Categorical beeswarm plot with a continuous color scale.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]

use std::collections::BTreeMap;

use crate::layout::{AxisTick, Layout, PanelBox, PhysicalTextAlignment, TextBlock};
use crate::render::backend::{
    with_clip, Backend, GradientStop, LineStyle, PlacedText, Point2D, TextAnchor,
};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::Error;
use crate::Plot;

pub(crate) fn render(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let theme = context.theme;
    let mut text = context.producer_text()?;
    let category_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("beeswarm: categorical row column not set".into()))?;
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("beeswarm: x column not set".into()))?;
    let color_col = chart
        .color_col()
        .ok_or_else(|| Error::Data("beeswarm: continuous color column not set".into()))?;
    let categories = crate::data::strict_strings(context.data, category_col, "beeswarm")?;
    let xs = crate::data::strict_finite_f64(context.data, x_col, "beeswarm")?;
    let colors = crate::data::strict_finite_f64(context.data, color_col, "beeswarm")?;
    if categories.len() != xs.len() || xs.len() != colors.len() || xs.is_empty() {
        return Err(Error::Data(
            "beeswarm: category/x/color columns must have equal non-zero lengths".into(),
        )
        .into());
    }
    let mut levels = Vec::new();
    for category in &categories {
        if !levels.contains(category) {
            levels.push(category.clone());
        }
    }
    let x_data = crate::data::min_max(xs.iter().copied(), "beeswarm")?;
    let color_range = match chart.beeswarm_color_limits_spec() {
        Some((minimum, maximum)) => {
            if !minimum.is_finite() || !maximum.is_finite() || minimum >= maximum {
                return Err(Error::Data(format!(
                    "beeswarm: color limits must be finite and strictly ascending; got {minimum}..{maximum}"
                )).into());
            }
            (minimum, maximum)
        }
        None => crate::data::min_max(colors.iter().copied(), "beeswarm")?,
    };
    let detail_font_size = theme.detail_font_size;
    let high_label = format_colorbar_value(color_range.1);
    let low_label = format_colorbar_value(color_range.0);
    let color_label = chart.resolved_color_label();
    let guide_scope = context
        .fit_scope()
        .child(crate::layout::AddressRole::OutsideGuideRight, 0)?;
    let color_label_block =
        TextBlock::from_resolved(text.plan(&color_label, "beeswarm colorbar title", || {
            crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                &color_label,
                &theme.legacy_text_appearance(detail_font_size, crate::TextWeight::Normal),
                context.panel.height().min(150.0),
                PhysicalTextAlignment::Center,
                "beeswarm colorbar title",
                &guide_scope.child(crate::layout::AddressRole::GuideTitle, 0)?,
            ))
        })?);
    let values = [&high_label, &low_label]
        .into_iter()
        .map(|label| {
            text.plan(label, "beeswarm colorbar value", || {
                Ok(theme.measure_legacy_text(
                    label,
                    detail_font_size,
                    PhysicalTextAlignment::Start,
                    "beeswarm colorbar value",
                )?)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let value_width = values
        .iter()
        .map(crate::layout::ResolvedTextPlan::width)
        .fold(0.0_f64, f64::max);
    let colorbar_reserve = (34.0 + value_width).max(
        65.0 + color_label_block.lines().len().saturating_sub(1) as f64
            * color_label_block.line_height()
            + f64::from(detail_font_size) / 2.0,
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
        let x_range = crate::layout::continuous_auto_range(x_data, chart.x_axis_spec());
        let x_axis_spec = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
        let y_ticks = levels
            .iter()
            .enumerate()
            .map(|(index, label)| AxisTick::new(index as f64 + 0.5, label.clone()))
            .collect();
        context.layout_with_ticks(
            x_range,
            (0.0, levels.len() as f64),
            panel,
            chart.x_label_text().unwrap_or("").to_string(),
            chart.y_label_text().unwrap_or("").to_string(),
            Vec::new(),
            y_ticks,
            &x_axis_spec,
            chart.y_axis_spec(),
            theme,
        )
    })?;
    let x_range = layout.projection.x_range();
    with_clip(backend, layout.projection.panel_box(), |backend| {
        if chart.zero_line_visible() && x_range.0 <= 0.0 && x_range.1 >= 0.0 {
            let (x, y0) = layout.projection.project(0.0, 0.0);
            let (_, y1) = layout.projection.project(0.0, levels.len() as f64);
            backend.draw_line(
                &[Point2D::new(x, y0), Point2D::new(x, y1)],
                &LineStyle::solid(theme.foreground, 0.8),
            )?;
        }

        let mut rows: Vec<(usize, f64, f64, usize)> = categories
            .iter()
            .zip(&xs)
            .zip(&colors)
            .enumerate()
            .map(|(row, ((category, x), color))| {
                let category_index = levels
                    .iter()
                    .position(|level| level == category)
                    .unwrap_or(0);
                (category_index, *x, *color, row)
            })
            .collect();
        rows.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.total_cmp(&b.1))
                .then_with(|| a.2.total_cmp(&b.2))
                .then_with(|| a.3.cmp(&b.3))
        });
        let mut occupied: BTreeMap<(usize, i64), usize> = BTreeMap::new();
        for (category, x, color, _) in rows {
            let bin = (((x - x_range.0) / (x_range.1 - x_range.0)) * 48.0).floor() as i64;
            let count = occupied.entry((category, bin)).or_default();
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
            let (px, py) = layout.projection.project(x, category as f64 + 0.5 + jitter);
            backend.draw_scatter(
                &[Point2D::new(px, py)],
                &chart.point_style(continuous_color(color, color_range), 3),
            )?;
        }
        Ok(())
    })?;
    draw_colorbar(
        backend,
        &layout,
        &color_label_block,
        color_range,
        &values,
        &mut text,
    )?;
    let mut result = PlotRenderResult::new(layout, Vec::new());
    result.producer_text = text.finish()?;
    Ok(result)
}

pub(crate) fn text_census_colorbar_values<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
    mut visit: impl FnMut(crate::font::canonical::CensusSource<'_>) -> Result<(), Error>,
) -> Result<(), Error> {
    let color_col = chart
        .color_col()
        .ok_or_else(|| Error::Data("beeswarm: continuous color column not set".into()))?;
    let colors = crate::data::strict_finite_f64(data, color_col, "beeswarm")?;
    let range = match chart.beeswarm_color_limits_spec() {
        Some((minimum, maximum)) => (minimum, maximum),
        None => crate::data::min_max(colors, "beeswarm")?,
    };
    for value in [range.0, range.1] {
        if value.abs() >= 1_000_000.0 || (value.abs().to_bits() != 0 && value.abs() < 0.0001) {
            visit(format_args!("{value:.3e}").into())?;
        } else {
            visit(format_args!("{value:.3}").into())?;
        }
    }
    Ok(())
}

fn draw_colorbar(
    backend: &mut dyn Backend,
    layout: &Layout,
    label: &crate::layout::TextBlock,
    range: (f64, f64),
    values: &[crate::layout::ResolvedTextPlan],
    text: &mut crate::spec::ProducerText<'_>,
) -> Result<(), Error> {
    let (_, panel_right, panel_bottom, panel_top) = layout.projection.pixel_box();
    let left = panel_right + 18.0;
    let height = (panel_top - panel_bottom).min(150.0);
    let bottom = panel_bottom + (panel_top - panel_bottom - height) / 2.0;
    let stops = [
        GradientStop::new(0.0, continuous_color(range.0, range)),
        GradientStop::new(1.0, continuous_color(range.1, range)),
    ];
    backend.draw_linear_gradient(
        PanelBox::new(left, left + 10.0, bottom, bottom + height),
        Point2D::new(left + 5.0, bottom),
        Point2D::new(left + 5.0, bottom + height),
        &stops,
    )?;
    text.draw(
        PlacedText::from_deferred_first_line(
            values[0].clone(),
            Point2D::new(
                (left + 16.0).trunc(),
                (bottom + height - 4.0).max(0.0).trunc(),
            ),
            TextAnchor::Start,
            0.0,
        ),
        backend,
    )?;
    text.draw(
        PlacedText::from_deferred_first_line(
            values[1].clone(),
            Point2D::new((left + 16.0).trunc(), (bottom - 13.0).max(0.0).trunc()),
            TextAnchor::Start,
            0.0,
        ),
        backend,
    )?;
    text.draw(
        PlacedText::from_deferred_first_line(
            label.plan().clone(),
            Point2D::new((left + 47.0).trunc(), (bottom + height / 2.0).trunc()),
            TextAnchor::Middle,
            -90.0,
        ),
        backend,
    )?;
    Ok(())
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

fn format_colorbar_value(value: f64) -> String {
    if value.abs() >= 1_000_000.0 || (value.abs().to_bits() != 0 && value.abs() < 0.0001) {
        format!("{value:.3e}")
    } else {
        format!("{value:.3}")
    }
}

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
        invalid.detail_font_size = 0;
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
