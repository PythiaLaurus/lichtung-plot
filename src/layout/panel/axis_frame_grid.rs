use crate::Error;

use super::geometry::y_axis_label_half_width;
use super::{Axis, AxisSide, AxisSpec, AxisTick, Frame, Grid, Projection, Scale};
use crate::layout::figure::{
    X_AXIS_TICK_LABEL_GAP, X_AXIS_TITLE_GAP, X_AXIS_TITLE_WITHOUT_TICKS_GAP,
};
use crate::layout::TextBlock;
use crate::render::backend::{Backend, LineStyle, Point2D, TextAnchor, TextStyle};
use crate::theme::ThemePalette;

const Y_AXIS_LABEL_EDGE_PADDING: f64 = 12.0;
const Y_AXIS_LABEL_TICK_GAP: f64 = 16.0;

impl Default for AxisSpec {
    fn default() -> Self {
        Self {
            scale: Scale::Linear,
            limits: None,
            tick_count: 5,
        }
    }
}

impl AxisSpec {
    /// Return a copy of this spec with explicit limits.
    ///
    /// Rendering requires finite, strictly ascending endpoints.
    #[must_use]
    pub fn limits(mut self, min: f64, max: f64) -> Self {
        self.limits = Some((min, max));
        self
    }

    /// Return a copy of this spec with an auto tick target.
    #[must_use]
    pub fn tick_count(mut self, count: usize) -> Self {
        self.tick_count = count.max(1);
        self
    }

    pub(crate) fn validate(&self, axis: &str) -> Result<(), Error> {
        if let Some((minimum, maximum)) = self.limits {
            if !minimum.is_finite() || !maximum.is_finite() || minimum >= maximum {
                return Err(Error::Data(format!(
                    "{axis} limits must be finite and strictly ascending; got {minimum}..{maximum}"
                )));
            }
        }
        Ok(())
    }
}

impl AxisTick {
    /// Construct an `AxisTick`.
    #[must_use]
    pub fn new(position: f64, label: impl Into<String>) -> Self {
        Self {
            position,
            label: label.into(),
        }
    }
}

impl Axis {
    /// Draw this axis's spine, ticks, tick labels, and axis label.
    ///
    /// `side` selects which side the ticks are drawn on:
    /// - `AxisSide::Bottom` → x-axis: ticks point down (math
    ///   convention; the SVG backend applies the y-flip), labels
    ///   below, axis label centered below the tick labels.
    /// - `AxisSide::Left` → y-axis: ticks point left, labels to
    ///   the left, axis label centered vertically to the far left.
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    pub fn draw(
        &self,
        projection: &Projection,
        side: AxisSide,
        palette: &ThemePalette,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        let (x0, x1, y0, y1) = projection.pixel_box();
        // y0 = bottom of plot area (math), y1 = top (math).

        let tick_style = LineStyle {
            color: palette.foreground,
            width: palette.axis_width,
            dash: String::new(),
        };
        let label_style = TextStyle {
            color: palette.foreground,
            font_family: palette.font_family.clone(),
            font_size: palette.label_font_size,
            weight: crate::TextWeight::Normal,
            anchor: TextAnchor::Middle,
            rotation: 0.0,
        };
        let tick_text_style = TextStyle {
            color: palette.foreground,
            font_family: palette.font_family.clone(),
            font_size: palette.tick_font_size,
            weight: crate::TextWeight::Normal,
            anchor: TextAnchor::Middle,
            rotation: 0.0,
        };
        let tick_width = match side {
            AxisSide::Bottom => (x1 - x0) / self.ticks.len().max(1) as f64 * 0.9,
            AxisSide::Left => ((x1 - x0) * 0.4).max(1.0),
        };

        // Spine (only the two visible edges for open-frame
        // styles; the Frame renderer decides whether the full
        // box is drawn).
        if palette.show_top_right_spines {
            let spine = match side {
                AxisSide::Bottom => vec![Point2D::new(x0, y0), Point2D::new(x1, y0)],
                AxisSide::Left => vec![Point2D::new(x0, y0), Point2D::new(x0, y1)],
            };
            backend.draw_line(&spine, &tick_style)?;
        }

        // Tick marks + tick labels.
        for tick in &self.ticks {
            match side {
                AxisSide::Bottom => {
                    let (x_px, _) = projection.project(tick.position, projection.y_range().0);
                    if palette.show_ticks {
                        let tick_len = f64::from(palette.tick_length);
                        backend.draw_line(
                            &[Point2D::new(x_px, y0), Point2D::new(x_px, y0 - tick_len)],
                            &tick_style,
                        )?;
                    }
                    if palette.show_tick_labels && !tick.label.is_empty() {
                        let block = TextBlock::layout(
                            &tick.label,
                            palette.tick_font_size,
                            tick_width,
                            "x-axis tick label",
                        )?;
                        let first_y = y0
                            - f64::from(palette.tick_length)
                            - X_AXIS_TICK_LABEL_GAP
                            - block.line_height() / 2.0;
                        for (index, line) in block.lines().iter().enumerate() {
                            backend.draw_text(
                                x_px.round() as u32,
                                (first_y - index as f64 * block.line_height()).round() as u32,
                                line,
                                &tick_text_style,
                            )?;
                        }
                    }
                }
                AxisSide::Left => {
                    let (_, y_px) = projection.project(projection.x_range().0, tick.position);
                    if palette.show_ticks {
                        let tick_len = f64::from(palette.tick_length);
                        backend.draw_line(
                            &[Point2D::new(x0, y_px), Point2D::new(x0 - tick_len, y_px)],
                            &tick_style,
                        )?;
                    }
                    if palette.show_tick_labels && !tick.label.is_empty() {
                        let y_tick_style = TextStyle {
                            anchor: TextAnchor::End,
                            ..tick_text_style.clone()
                        };
                        let block = TextBlock::layout(
                            &tick.label,
                            palette.tick_font_size,
                            tick_width,
                            "y-axis tick label",
                        )?;
                        let first_y = y_px
                            + block.lines().len().saturating_sub(1) as f64 * block.line_height()
                                / 2.0;
                        for (index, line) in block.lines().iter().enumerate() {
                            backend.draw_text(
                                ((x0 - f64::from(palette.tick_length)) - 6.0)
                                    .max(0.0)
                                    .round() as u32,
                                (first_y - index as f64 * block.line_height()).round() as u32,
                                line,
                                &y_tick_style,
                            )?;
                        }
                    }
                }
            }
        }

        // Axis label.
        if !self.label.is_empty() {
            match side {
                AxisSide::Bottom => {
                    let cx = f64::midpoint(x0, x1);
                    let tick_label_height = if palette.show_tick_labels {
                        self.ticks
                            .iter()
                            .filter(|tick| !tick.label.is_empty())
                            .map(|tick| {
                                TextBlock::layout(
                                    &tick.label,
                                    palette.tick_font_size,
                                    tick_width,
                                    "x-axis tick label",
                                )
                                .map(|block| block.height())
                            })
                            .collect::<Result<Vec<_>, _>>()?
                            .into_iter()
                            .fold(0.0_f64, f64::max)
                    } else {
                        0.0
                    };
                    let block = TextBlock::layout(
                        &self.label,
                        palette.label_font_size,
                        (x1 - x0).max(1.0),
                        "x-axis title",
                    )?;
                    let tick_offset = if tick_label_height > 0.0 {
                        X_AXIS_TICK_LABEL_GAP + tick_label_height + X_AXIS_TITLE_GAP
                    } else {
                        X_AXIS_TITLE_WITHOUT_TICKS_GAP
                    };
                    let first_y = y0
                        - f64::from(palette.tick_length)
                        - tick_offset
                        - block.line_height() / 2.0;
                    for (index, line) in block.lines().iter().enumerate() {
                        backend.draw_text(
                            cx.round() as u32,
                            (first_y - index as f64 * block.line_height()).round() as u32,
                            line,
                            &label_style,
                        )?;
                    }
                }
                AxisSide::Left => {
                    let cy = f64::midpoint(y0, y1);
                    let tick_label_width = if palette.show_tick_labels {
                        self.ticks
                            .iter()
                            .filter(|tick| !tick.label.is_empty())
                            .map(|tick| {
                                TextBlock::layout(
                                    &tick.label,
                                    palette.tick_font_size,
                                    tick_width,
                                    "y-axis tick label",
                                )
                                .map(|block| block.width())
                            })
                            .collect::<Result<Vec<_>, _>>()?
                            .into_iter()
                            .fold(0.0_f64, f64::max)
                    } else {
                        0.0
                    };
                    let block = TextBlock::layout(
                        &self.label,
                        palette.label_font_size,
                        (y1 - y0).max(1.0),
                        "y-axis title",
                    )?;
                    let tick_label_anchor_x = x0 - f64::from(palette.tick_length) - 6.0;
                    let label_half_width = y_axis_label_half_width(palette.label_font_size);
                    let min_lx = Y_AXIS_LABEL_EDGE_PADDING
                        + label_half_width
                        + block.lines().len().saturating_sub(1) as f64 * block.line_height();
                    let lx = if tick_label_width > 0.0 {
                        tick_label_anchor_x
                            - tick_label_width
                            - Y_AXIS_LABEL_TICK_GAP
                            - label_half_width
                    } else {
                        x0 - f64::from(palette.tick_length) - 48.0
                    }
                    .max(min_lx);
                    let y_label_style = TextStyle {
                        anchor: TextAnchor::Middle,
                        rotation: -90.0,
                        ..label_style.clone()
                    };
                    for (index, line) in block.lines().iter().enumerate() {
                        backend.draw_text(
                            (lx - index as f64 * block.line_height()).max(0.0).round() as u32,
                            cy.round() as u32,
                            line,
                            &y_label_style,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Frame {
    /// Draw the four-spine frame using `palette` styles. If
    /// `palette.show_frame` is `false`, this is a no-op. The
    /// `palette.show_top_right_spines` flag decides whether the
    /// top and right spines are drawn in addition to the bottom
    /// and left ones.
    pub fn draw(
        &self,
        projection: &Projection,
        palette: &ThemePalette,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        if !palette.show_frame {
            return Ok(());
        }
        let (x0, x1, y0, y1) = projection.pixel_box();
        let line_style = LineStyle {
            color: palette.foreground,
            width: palette.axis_width,
            dash: String::new(),
        };
        backend.draw_line(&[Point2D::new(x0, y0), Point2D::new(x1, y0)], &line_style)?;
        backend.draw_line(&[Point2D::new(x1, y0), Point2D::new(x1, y1)], &line_style)?;
        backend.draw_line(&[Point2D::new(x1, y1), Point2D::new(x0, y1)], &line_style)?;
        backend.draw_line(&[Point2D::new(x0, y1), Point2D::new(x0, y0)], &line_style)
    }
}

impl Grid {
    /// Draw the grid lines using `palette` styles. If either
    /// `palette.show_grid` is `false` or `palette.grid_width == 0`,
    /// this is a no-op.
    pub fn draw(
        &self,
        projection: &Projection,
        x_axis: &Axis,
        y_axis: &Axis,
        palette: &ThemePalette,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        if !palette.show_grid || palette.grid_width == 0.0 {
            return Ok(());
        }
        let line_style = LineStyle {
            color: palette.grid_color,
            width: palette.grid_width,
            dash: "4 3".to_string(),
        };
        for tick in &x_axis.ticks {
            let (x_px, _) = projection.project(tick.position, 0.0);
            let (_, y_min) = projection.project(tick.position, projection.y_range().0);
            let (_, y_max) = projection.project(tick.position, projection.y_range().1);
            backend.draw_line(
                &[Point2D::new(x_px, y_min), Point2D::new(x_px, y_max)],
                &line_style,
            )?;
        }
        for tick in &y_axis.ticks {
            let (_, y_px) = projection.project(0.0, tick.position);
            let (x_min, _) = projection.project(projection.x_range().0, tick.position);
            let (x_max, _) = projection.project(projection.x_range().1, tick.position);
            backend.draw_line(
                &[Point2D::new(x_min, y_px), Point2D::new(x_max, y_px)],
                &line_style,
            )?;
        }
        Ok(())
    }
}
