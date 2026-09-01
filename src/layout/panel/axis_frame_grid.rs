use crate::Error;

use super::geometry::y_axis_label_half_width;
use super::{Axis, AxisSide, AxisSpec, AxisTick, Frame, Grid, Projection};
use crate::layout::TextBlock;
use crate::render::backend::{Backend, Point2D, TextAnchor, TextStyle};
use crate::theme::ResolvedTheme;

impl AxisSpec {
    /// Return a copy of this spec with explicit limits.
    ///
    /// Rendering requires finite, strictly ascending endpoints.
    #[must_use]
    pub fn limits(mut self, min: f64, max: f64) -> Self {
        self.limits = Some((min, max));
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
    pub(crate) fn validate_explicit_tick_collisions(
        &self,
        projection: &Projection,
        side: AxisSide,
        theme: &ResolvedTheme,
        axis: &str,
    ) -> Result<(), Error> {
        let maximum_width = self.tick_label_maximum_width(projection, side);
        let role = format!("{axis} explicit tick label");
        let mut previous = None;
        for (index, tick) in self
            .ticks
            .iter()
            .enumerate()
            .filter(|(_, tick)| !tick.label.is_empty())
        {
            let block = TextBlock::layout(
                &tick.label,
                theme.typography.tick_label.font_size,
                maximum_width,
                &role,
            )?;
            let center = match side {
                AxisSide::Bottom => projection.project(tick.position, projection.y_range().0).0,
                AxisSide::Left => projection.project(projection.x_range().0, tick.position).1,
            };
            let (start, end) = match side {
                AxisSide::Bottom => {
                    let center = center.round();
                    (center - block.width() / 2.0, center + block.width() / 2.0)
                }
                AxisSide::Left => {
                    let line_offset = (block.height() - block.line_height()) / 2.0;
                    (
                        (center - line_offset).round() - block.line_height() / 2.0,
                        (center + line_offset).round() + block.line_height() / 2.0,
                    )
                }
            };
            if let Some((previous_index, previous_end)) = previous {
                if start < previous_end {
                    return Err(Error::Render(format!(
                        "{axis} explicit tick labels at indices {previous_index} and {index} overlap after projection; increase chart size, shorten the labels, or provide non-overlapping exact ticks"
                    )));
                }
            }
            previous = Some((index, end));
        }
        Ok(())
    }

    fn tick_label_maximum_width(&self, projection: &Projection, side: AxisSide) -> f64 {
        let (x0, x1, _, _) = projection.pixel_box();
        match side {
            AxisSide::Bottom => (x1 - x0) / self.ticks.len().max(1) as f64 * 0.9,
            AxisSide::Left => ((x1 - x0) * 0.4).max(1.0),
        }
    }

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
        theme: &ResolvedTheme,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        let (x0, x1, y0, y1) = projection.pixel_box();
        // y0 = bottom of plot area (math), y1 = top (math).

        let frame = theme.axes.frame.as_ref();
        let ticks = theme.axes.ticks.as_ref();
        let frame_style = frame.map(|appearance| appearance.stroke.backend_style());
        let tick_style = ticks.map(|appearance| appearance.stroke.backend_style());
        let tick_length = ticks.map_or(0.0, |appearance| f64::from(appearance.length));
        let label_style = theme
            .typography
            .axis_title
            .backend_style(TextAnchor::Middle, 0.0);
        let tick_text_style = theme
            .typography
            .tick_label
            .backend_style(TextAnchor::Middle, 0.0);
        let tick_width = self.tick_label_maximum_width(projection, side);

        // Spine (only the two visible edges for open-frame
        // styles; the Frame renderer decides whether the full
        // box is drawn).
        if frame.is_some_and(|appearance| appearance.source_show_top_right_spines) {
            if let Some(frame_style) = &frame_style {
                let spine = match side {
                    AxisSide::Bottom => vec![Point2D::new(x0, y0), Point2D::new(x1, y0)],
                    AxisSide::Left => vec![Point2D::new(x0, y0), Point2D::new(x0, y1)],
                };
                backend.draw_line(&spine, frame_style)?;
            }
        }

        // Tick marks + tick labels.
        for tick in &self.ticks {
            match side {
                AxisSide::Bottom => {
                    let (x_px, _) = projection.project(tick.position, projection.y_range().0);
                    if let Some(tick_style) = &tick_style {
                        backend.draw_line(
                            &[Point2D::new(x_px, y0), Point2D::new(x_px, y0 - tick_length)],
                            tick_style,
                        )?;
                    }
                    if !tick.label.is_empty() {
                        let block = TextBlock::layout(
                            &tick.label,
                            theme.typography.tick_label.font_size,
                            tick_width,
                            "x-axis tick label",
                        )?;
                        let first_y = y0
                            - tick_length
                            - theme.axes.spacing.x_tick_label_gap
                            - block.line_height() / 2.0;
                        for (index, line) in block.lines().iter().enumerate() {
                            backend.draw_text(
                                Point2D::new(
                                    x_px.round(),
                                    (first_y - index as f64 * block.line_height()).round(),
                                ),
                                line,
                                &tick_text_style,
                            )?;
                        }
                    }
                }
                AxisSide::Left => {
                    let (_, y_px) = projection.project(projection.x_range().0, tick.position);
                    if let Some(tick_style) = &tick_style {
                        backend.draw_line(
                            &[Point2D::new(x0, y_px), Point2D::new(x0 - tick_length, y_px)],
                            tick_style,
                        )?;
                    }
                    if !tick.label.is_empty() {
                        let y_tick_style = TextStyle {
                            anchor: TextAnchor::End,
                            ..tick_text_style.clone()
                        };
                        let block = TextBlock::layout(
                            &tick.label,
                            theme.typography.tick_label.font_size,
                            tick_width,
                            "y-axis tick label",
                        )?;
                        let first_y = y_px
                            + block.lines().len().saturating_sub(1) as f64 * block.line_height()
                                / 2.0;
                        for (index, line) in block.lines().iter().enumerate() {
                            backend.draw_text(
                                Point2D::new(
                                    (x0 - tick_length - theme.axes.spacing.y_tick_label_gap)
                                        .max(0.0)
                                        .round(),
                                    (first_y - index as f64 * block.line_height()).round(),
                                ),
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
                    let tick_label_height = self
                        .ticks
                        .iter()
                        .filter(|tick| !tick.label.is_empty())
                        .map(|tick| {
                            TextBlock::layout(
                                &tick.label,
                                theme.typography.tick_label.font_size,
                                tick_width,
                                "x-axis tick label",
                            )
                            .map(|block| block.height())
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .into_iter()
                        .fold(0.0_f64, f64::max);
                    let block = TextBlock::layout(
                        &self.label,
                        theme.typography.axis_title.font_size,
                        (x1 - x0).max(1.0),
                        "x-axis title",
                    )?;
                    let tick_offset = if tick_label_height > 0.0 {
                        theme.axes.spacing.x_tick_label_gap
                            + tick_label_height
                            + theme.axes.spacing.x_title_tick_label_gap
                    } else {
                        theme.axes.spacing.x_title_without_tick_labels_gap
                    };
                    let first_y = y0 - tick_length - tick_offset - block.line_height() / 2.0;
                    for (index, line) in block.lines().iter().enumerate() {
                        backend.draw_text(
                            Point2D::new(
                                cx.round(),
                                (first_y - index as f64 * block.line_height()).round(),
                            ),
                            line,
                            &label_style,
                        )?;
                    }
                }
                AxisSide::Left => {
                    let cy = f64::midpoint(y0, y1);
                    let tick_label_width = self
                        .ticks
                        .iter()
                        .filter(|tick| !tick.label.is_empty())
                        .map(|tick| {
                            TextBlock::layout(
                                &tick.label,
                                theme.typography.tick_label.font_size,
                                tick_width,
                                "y-axis tick label",
                            )
                            .map(|block| block.width())
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .into_iter()
                        .fold(0.0_f64, f64::max);
                    let block = TextBlock::layout(
                        &self.label,
                        theme.typography.axis_title.font_size,
                        (y1 - y0).max(1.0),
                        "y-axis title",
                    )?;
                    let tick_label_anchor_x =
                        x0 - tick_length - theme.axes.spacing.y_tick_label_gap;
                    let label_half_width =
                        y_axis_label_half_width(theme.typography.axis_title.font_size);
                    let min_lx = theme.axes.spacing.y_title_edge_padding
                        + label_half_width
                        + block.lines().len().saturating_sub(1) as f64 * block.line_height();
                    let lx = if tick_label_width > 0.0 {
                        tick_label_anchor_x
                            - tick_label_width
                            - theme.axes.spacing.y_title_tick_label_gap
                            - label_half_width
                    } else {
                        x0 - tick_length - 48.0
                    }
                    .max(min_lx);
                    let y_label_style = TextStyle {
                        anchor: TextAnchor::Middle,
                        rotation: -90.0,
                        ..label_style.clone()
                    };
                    for (index, line) in block.lines().iter().enumerate() {
                        backend.draw_text(
                            Point2D::new(
                                (lx - index as f64 * block.line_height()).max(0.0).round(),
                                cy.round(),
                            ),
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
    /// Draw the current structurally present four-spine frame.
    pub fn draw(
        projection: &Projection,
        theme: &ResolvedTheme,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        let Some(frame) = &theme.axes.frame else {
            return Ok(());
        };
        let (x0, x1, y0, y1) = projection.pixel_box();
        let line_style = frame.stroke.backend_style();
        backend.draw_line(&[Point2D::new(x0, y0), Point2D::new(x1, y0)], &line_style)?;
        backend.draw_line(&[Point2D::new(x1, y0), Point2D::new(x1, y1)], &line_style)?;
        backend.draw_line(&[Point2D::new(x1, y1), Point2D::new(x0, y1)], &line_style)?;
        backend.draw_line(&[Point2D::new(x0, y1), Point2D::new(x0, y0)], &line_style)
    }
}

impl Grid {
    /// Draw the structurally present grid style.
    pub fn draw(
        projection: &Projection,
        x_axis: &Axis,
        y_axis: &Axis,
        theme: &ResolvedTheme,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        let Some(grid) = &theme.axes.grid else {
            return Ok(());
        };
        let line_style = grid.backend_style();
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
