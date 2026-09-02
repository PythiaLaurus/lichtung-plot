use super::range_ticks::resolve_range;
use super::{
    automatic_numeric_ticks, Axis, AxisSide, AxisSpec, AxisTick, Grid, Layout, Projection,
};
use crate::layout::PanelBox;
use crate::render::backend::Backend;
use crate::theme::ResolvedTheme;
use crate::Error;

impl Layout {
    /// Build the non-rendered carrier returned by a self-decorating plot.
    pub(crate) fn self_managed(canvas_width: u32, canvas_height: u32) -> Self {
        Self {
            x_axis: Axis {
                label: String::new(),
                ticks: Vec::new(),
            },
            y_axis: Axis {
                label: String::new(),
                ticks: Vec::new(),
            },
            projection: Projection {
                x_range: (0.0, 1.0),
                y_range: (0.0, 1.0),
                pixel_box: default_panel_box(canvas_width, canvas_height).as_tuple(),
            },
        }
    }

    /// Create a new layout using explicit axis specs for limits.
    #[cfg(test)]
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
        theme: &ResolvedTheme,
    ) -> Self {
        // Hardcoded academic-style margins for now.
        let pixel_box = default_panel_box(canvas_width, canvas_height);

        Self::new_with_axis_specs_in_panel_box(
            x_range, y_range, pixel_box, x_label, y_label, x_spec, y_spec, theme,
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
        theme: &ResolvedTheme,
    ) -> Self {
        let pixel_box = panel_box.as_tuple();

        let x_range = resolve_range(x_range, x_spec);
        let y_range = resolve_range(y_range, y_spec);
        let projection = Projection {
            x_range,
            y_range,
            pixel_box,
        };

        Self {
            x_axis: Axis {
                label: x_label,
                ticks: automatic_numeric_ticks(x_range, &projection, AxisSide::Bottom, theme),
            },
            y_axis: Axis {
                label: y_label,
                ticks: automatic_numeric_ticks(y_range, &projection, AxisSide::Left, theme),
            },
            projection,
        }
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
        theme: &ResolvedTheme,
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
        let projection = Projection {
            x_range,
            y_range,
            pixel_box,
        };

        // Auto-generate numeric ticks when none were provided.
        let x_ticks = if x_ticks.is_empty() {
            automatic_numeric_ticks(x_range, &projection, AxisSide::Bottom, theme)
        } else {
            x_ticks
        };
        let y_ticks = if y_ticks.is_empty() {
            automatic_numeric_ticks(y_range, &projection, AxisSide::Left, theme)
        } else {
            y_ticks
        };

        Self {
            projection,
            x_axis: Axis {
                label: x_label,
                ticks: x_ticks,
            },
            y_axis: Axis {
                label: y_label,
                ticks: y_ticks,
            },
        }
    }

    /// Draw frame and axes ON TOP of data (called after data rendering).
    /// Grid is drawn separately via `draw_grid_below()` BEFORE data.
    #[cfg(test)]
    pub fn draw_decorations(
        &self,
        theme: &ResolvedTheme,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        let text = self.plan_axis_text(theme)?;
        self.draw_decorations_with_text(theme, &text, backend)
    }

    /// Draw the grid lines BELOW data. Call this before rendering
    /// data marks so the grid appears behind the data.
    pub fn draw_grid_below(
        &self,
        theme: &ResolvedTheme,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        Grid::draw(&self.projection, &self.x_axis, &self.y_axis, theme, backend)
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

/// Estimate the rendered width of text with the deterministic layout model.
///
/// Wide glyphs and combining marks are handled consistently with canvas,
/// legend, geometry, and axis-inset planning.
#[must_use]
pub(super) fn y_axis_label_half_width(font_size: u32) -> f64 {
    f64::from(font_size) / 2.0
}

pub(crate) fn y_axis_left_inset_for_tick_labels(
    tick_label_width: f64,
    tick_length: u32,
    label_font_size: u32,
    spacing: &crate::theme::AxisSpacing,
) -> f64 {
    spacing.y_title_edge_padding
        + f64::from(label_font_size)
        + spacing.y_title_tick_label_gap
        + tick_label_width
        + f64::from(tick_length)
        + spacing.y_tick_label_gap
}
