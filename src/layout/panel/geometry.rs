use crate::Error;
use unicode_segmentation::UnicodeSegmentation;

use super::range_ticks::{numeric_ticks, resolve_range};
use super::{Axis, AxisSide, AxisSpec, AxisTick, Frame, Grid, Layout, Projection};
use crate::layout::PanelBox;
use crate::render::backend::Backend;
use crate::theme::ThemePalette;

const Y_AXIS_LABEL_EDGE_PADDING: f64 = 12.0;
const Y_AXIS_LABEL_TICK_GAP: f64 = 16.0;

impl Layout {
    /// Create a new layout for the given numeric data ranges
    /// and canvas area. The x and y axis tick labels are
    /// auto-generated from `nice_ticks(lo, hi, 5)` and formatted
    /// with one decimal place.
    pub fn new(
        x_range: (f64, f64),
        y_range: (f64, f64),
        canvas_width: u32,
        canvas_height: u32,
        x_label: String,
        y_label: String,
    ) -> Self {
        Self::new_with_axis_specs(
            x_range,
            y_range,
            canvas_width,
            canvas_height,
            x_label,
            y_label,
            &AxisSpec::default(),
            &AxisSpec::default(),
        )
    }

    /// Create a new layout using explicit axis specs for limits and tick counts.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new_with_axis_specs(
        x_range: (f64, f64),
        y_range: (f64, f64),
        canvas_width: u32,
        canvas_height: u32,
        x_label: String,
        y_label: String,
        x_spec: &AxisSpec,
        y_spec: &AxisSpec,
    ) -> Self {
        // Hardcoded academic-style margins for now.
        let pixel_box = default_panel_box(canvas_width, canvas_height);

        Self::new_with_axis_specs_in_panel_box(
            x_range, y_range, pixel_box, x_label, y_label, x_spec, y_spec,
        )
    }

    /// Create a new numeric layout inside an explicit panel box.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new_with_axis_specs_in_panel_box(
        x_range: (f64, f64),
        y_range: (f64, f64),
        panel_box: PanelBox,
        x_label: String,
        y_label: String,
        x_spec: &AxisSpec,
        y_spec: &AxisSpec,
    ) -> Self {
        let pixel_box = panel_box.as_tuple();

        let x_range = resolve_range(x_range, x_spec);
        let y_range = resolve_range(y_range, y_spec);

        Self {
            projection: Projection {
                x_range,
                y_range,
                pixel_box,
            },
            frame: Frame,
            grid: Grid,
            x_axis: Axis {
                label: x_label,
                ticks: numeric_ticks(x_range, x_spec),
                log_scale: false,
            },
            y_axis: Axis {
                label: y_label,
                ticks: numeric_ticks(y_range, y_spec),
                log_scale: false,
            },
        }
    }

    /// Create a new layout with explicit (position, label) ticks
    /// on both axes. Use this for categorical x-axes (bar,
    /// boxplot, violin).
    ///
    /// `x_ticks` and `y_ticks` are exactly the ticks to render;
    /// the `x_range` and `y_range` should cover the tick
    /// positions (typically `0..n` for categorical x, and the
    /// data's numeric min/max for y).
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new_with_ticks(
        x_range: (f64, f64),
        y_range: (f64, f64),
        canvas_width: u32,
        canvas_height: u32,
        x_label: String,
        y_label: String,
        x_ticks: Vec<AxisTick>,
        y_ticks: Vec<AxisTick>,
    ) -> Self {
        Self::new_with_ticks_and_axis_specs(
            x_range,
            y_range,
            canvas_width,
            canvas_height,
            x_label,
            y_label,
            x_ticks,
            y_ticks,
            &AxisSpec::default(),
            &AxisSpec::default(),
        )
    }

    /// Create a new layout with explicit ticks and axis specs.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new_with_ticks_and_axis_specs(
        x_range: (f64, f64),
        y_range: (f64, f64),
        canvas_width: u32,
        canvas_height: u32,
        x_label: String,
        y_label: String,
        x_ticks: Vec<AxisTick>,
        y_ticks: Vec<AxisTick>,
        x_spec: &AxisSpec,
        y_spec: &AxisSpec,
    ) -> Self {
        let pixel_box = default_panel_box(canvas_width, canvas_height);

        Self::new_with_ticks_and_axis_specs_in_panel_box(
            x_range, y_range, pixel_box, x_label, y_label, x_ticks, y_ticks, x_spec, y_spec,
        )
    }

    /// Create a new layout with explicit ticks and axis specs inside an explicit panel box.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new_with_ticks_and_axis_specs_in_panel_box(
        x_range: (f64, f64),
        y_range: (f64, f64),
        panel_box: PanelBox,
        x_label: String,
        y_label: String,
        x_ticks: Vec<AxisTick>,
        y_ticks: Vec<AxisTick>,
        x_spec: &AxisSpec,
        y_spec: &AxisSpec,
    ) -> Self {
        let pixel_box = panel_box.as_tuple();

        // Don't expand categorical x ranges — the user picked
        // them deliberately.
        let x_range = if x_ticks.is_empty() {
            resolve_range(x_range, x_spec)
        } else if let Some(limits) = x_spec.limits {
            limits
        } else {
            x_range
        };
        let y_range = if y_ticks.is_empty() {
            resolve_range(y_range, y_spec)
        } else if let Some(limits) = y_spec.limits {
            limits
        } else {
            y_range
        };

        // Auto-generate numeric ticks when none were provided.
        let x_ticks = if x_ticks.is_empty() {
            numeric_ticks(x_range, x_spec)
        } else {
            x_ticks
        };
        let y_ticks = if y_ticks.is_empty() {
            numeric_ticks(y_range, y_spec)
        } else {
            y_ticks
        };

        Self {
            projection: Projection {
                x_range,
                y_range,
                pixel_box,
            },
            frame: Frame,
            grid: Grid,
            x_axis: Axis {
                label: x_label,
                ticks: x_ticks,
                log_scale: false,
            },
            y_axis: Axis {
                label: y_label,
                ticks: y_ticks,
                log_scale: false,
            },
        }
    }

    /// Draw frame and axes ON TOP of data (called after data rendering).
    /// Grid is drawn separately via `draw_grid_below()` BEFORE data.
    pub fn draw_decorations(
        &self,
        palette: &ThemePalette,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        self.frame.draw(&self.projection, palette, backend)?;
        self.x_axis
            .draw(&self.projection, AxisSide::Bottom, palette, backend)?;
        self.y_axis
            .draw(&self.projection, AxisSide::Left, palette, backend)?;
        Ok(())
    }

    /// Draw the grid lines BELOW data. Call this before rendering
    /// data marks so the grid appears behind the data.
    pub fn draw_grid_below(
        &self,
        palette: &ThemePalette,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        self.grid.draw(
            &self.projection,
            &self.x_axis,
            &self.y_axis,
            palette,
            backend,
        )
    }
}

fn default_panel_box(canvas_width: u32, canvas_height: u32) -> PanelBox {
    PanelBox::new(
        80.0,
        f64::from(canvas_width) - 30.0,
        60.0,
        f64::from(canvas_height) - 30.0,
    )
}

pub(crate) fn estimated_text_width(text: &str, font_size: u32) -> f64 {
    deterministic_text_width(text, f64::from(font_size))
}

/// Estimate the rendered width of text with the deterministic layout model.
///
/// Wide glyphs and combining marks are handled consistently with figure,
/// legend, geometry, and axis-inset planning.
#[must_use]
pub fn deterministic_text_width(text: &str, font_size: f64) -> f64 {
    text.graphemes(true)
        .map(|grapheme| {
            grapheme
                .chars()
                .map(character_width_em)
                .fold(0.0_f64, f64::max)
        })
        .sum::<f64>()
        * font_size
}

fn character_width_em(character: char) -> f64 {
    let code = u32::from(character);
    if matches!(character, '\n' | '\r') || code == 0x200d {
        return 0.0;
    }
    if matches!(
        code,
        0x0300..=0x036f
            | 0x1ab0..=0x1aff
            | 0x1dc0..=0x1dff
            | 0x20d0..=0x20ff
            | 0xfe20..=0xfe2f
            | 0xfe00..=0xfe0f
    ) {
        return 0.0;
    }
    if matches!(
        code,
        0x1100..=0x115f
            | 0x2329..=0x232a
            | 0x2e80..=0xa4cf
            | 0xac00..=0xd7a3
            | 0xf900..=0xfaff
            | 0xfe10..=0xfe19
            | 0xfe30..=0xfe6f
            | 0xff00..=0xff60
            | 0xffe0..=0xffe6
            | 0x1f300..=0x1faff
            | 0x20000..=0x3fffd
    ) {
        return 1.0;
    }
    match character {
        '\t' => 2.4,
        ' ' => 0.33,
        'i' | 'l' | 'I' | '!' | '|' | '.' | ',' | ':' | ';' | '\'' | '`' => 0.35,
        'm' | 'w' | 'M' | 'W' | '@' | '%' => 0.85,
        _ => 0.60,
    }
}

pub(super) fn y_axis_label_half_width(font_size: u32) -> f64 {
    f64::from(font_size) / 2.0
}

pub(crate) fn y_axis_left_inset_for_tick_labels(
    tick_label_width: f64,
    tick_length: u32,
    label_font_size: u32,
) -> f64 {
    Y_AXIS_LABEL_EDGE_PADDING
        + f64::from(label_font_size)
        + Y_AXIS_LABEL_TICK_GAP
        + tick_label_width
        + f64::from(tick_length)
        + 6.0
}
