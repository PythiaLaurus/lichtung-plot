//! `lichtung-plot::plots::mass_spectrum` — mass-spec spectrum plot.
//!
//! Renders a `(m/z, intensity)` spectrum as a thin line series
//! with filled-circle peak markers at positions supplied via
//! `Chart::add_peaks`, plus an m/z text label next to each
//! marker. Peak detection remains caller-owned.
//!
//! This implementation is pure display: callers pass already
//! detected peaks, which are mapped to pixel coordinates and
//! emitted as draw calls.

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
use crate::layout::Layout;
use crate::render::backend::{with_clip, Backend, LineStyle, Point2D, TextAnchor, TextStyle};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::Chart;
use crate::Error;

/// Render a mass-spec spectrum plot: line series of
/// `(x_col, y_col)` plus filled-circle peak markers at
/// the positions in `chart.peaks()` and an m/z text
/// label next to each marker.
pub(crate) fn render(
    chart: &Chart<'_>,
    context: PlotRenderContext<'_>,
    backend: &mut dyn Backend,
) -> Result<PlotRenderResult, Error> {
    let palette = chart.palette();

    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("mass_spectrum: x column not set".into()))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("mass_spectrum: y column not set".into()))?;
    let x = crate::data::strict_finite_f64(context.data, x_col, "mass_spectrum")?;
    let y = crate::data::strict_finite_f64(context.data, y_col, "mass_spectrum")?;
    if x.len() != y.len() {
        return Err(Error::Data("mass_spectrum: x/y length mismatch".into()));
    }
    if x.len() < 2 {
        return Err(Error::Data(format!(
            "mass_spectrum: need ≥ 2 points, got {}",
            x.len()
        )));
    }
    validate_peaks(chart.peaks())?;

    let (x_min, x_max) = crate::data::min_max(x.iter().copied(), "mass_spectrum")?;
    let y_spectrum_min = y.iter().fold(f64::INFINITY, |a, &v| a.min(v));
    let y_spectrum_max = y.iter().fold(f64::NEG_INFINITY, |a, &v| a.max(v));
    let y_min_data = chart
        .peaks()
        .iter()
        .map(|&(_, yi)| yi)
        .fold(y_spectrum_min, f64::min);
    let y_max_data = chart
        .peaks()
        .iter()
        .map(|&(_, yi)| yi)
        .fold(y_spectrum_max, f64::max);

    let x_range = crate::layout::continuous_auto_range((x_min, x_max), chart.x_axis_spec());
    let y_range =
        crate::layout::continuous_auto_range((y_min_data, y_max_data), chart.y_axis_spec());
    let x_axis_spec = crate::layout::display_axis_spec(chart.x_axis_spec(), x_range);
    let y_axis_spec = crate::layout::display_axis_spec(chart.y_axis_spec(), y_range);

    let layout = Layout::new_with_axis_specs_in_panel_box(
        x_range,
        y_range,
        context.panel,
        chart.x_label_text().unwrap_or("").to_string(),
        chart.y_label_text().unwrap_or("").to_string(),
        &x_axis_spec,
        &y_axis_spec,
    );
    let proj = &layout.projection;

    let line_points: Vec<Point2D> = x
        .iter()
        .zip(y.iter())
        .map(|(&xi, &yi)| {
            let (px, py) = proj.project(xi, yi);
            Point2D::new(px, py)
        })
        .collect();
    let line_style = LineStyle {
        color: palette.colors[0],
        width: palette.line_width,
        dash: String::new(),
    };
    // Draw peak markers + m/z labels.
    let marker_style = chart.point_style(palette.colors[1], 4);
    let text_style = TextStyle {
        color: palette.foreground,
        font_family: palette.font_family.clone(),
        font_size: 10,
        weight: crate::TextWeight::Normal,
        anchor: TextAnchor::Start,
        rotation: 0.0,
    };
    with_clip(backend, layout.projection.panel_box(), |backend| {
        backend.draw_line(&line_points, &line_style)?;
        for &(mz, intensity) in chart.peaks() {
            let (px, py) = proj.project(mz, intensity);
            backend.draw_scatter(&[Point2D::new(px, py)], &marker_style)?;
            if proj.contains_data(mz, intensity) {
                let label_x = (px + 6.0) as u32;
                let label_y = (py + 6.0) as u32;
                backend.draw_text(label_x, label_y, &format!("{mz:.1}"), &text_style)?;
            }
        }
        Ok(())
    })?;

    let mut legend_entries = vec![LegendEntry::point(
        "Spectrum".to_string(),
        chart.point_style(line_style.color, 3),
    )];
    if !chart.peaks().is_empty() {
        legend_entries.push(LegendEntry::point("Peaks".to_string(), marker_style));
    }
    Ok(PlotRenderResult::new(layout, legend_entries))
}

fn validate_peaks(peaks: &[(f64, f64)]) -> Result<(), Error> {
    for (index, (mz, intensity)) in peaks.iter().copied().enumerate() {
        if !mz.is_finite() || !intensity.is_finite() {
            return Err(Error::Data(format!(
                "mass_spectrum: supplemental peak {index} must contain finite m/z and intensity values, got ({mz}, {intensity})"
            )));
        }
    }
    Ok(())
}
