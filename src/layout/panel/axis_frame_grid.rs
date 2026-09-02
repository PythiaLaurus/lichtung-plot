use crate::Error;

use super::geometry::y_axis_label_half_width;
use super::{Axis, AxisSide, AxisSpec, AxisTick, Frame, Grid, Layout, Projection};
use crate::layout::{PhysicalTextAlignment, ResolvedTextPlan, TextBlock};
use crate::render::backend::{Backend, PlannedTextPlacement, Point2D, TextAnchor};
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

#[derive(Debug, Clone)]
pub(crate) struct PanelAxisTextPlan {
    x_axis: AxisTextPlan,
    y_axis: AxisTextPlan,
}

#[derive(Debug, Clone)]
struct AxisTextPlan {
    tick_labels: Vec<Option<ResolvedTextPlan>>,
    title: Option<ResolvedTextPlan>,
}

impl Layout {
    pub(crate) fn plan_axis_text(&self, theme: &ResolvedTheme) -> Result<PanelAxisTextPlan, Error> {
        Ok(PanelAxisTextPlan {
            x_axis: self
                .x_axis
                .plan_text(&self.projection, AxisSide::Bottom, theme)?,
            y_axis: self
                .y_axis
                .plan_text(&self.projection, AxisSide::Left, theme)?,
        })
    }

    pub(crate) fn draw_decorations_with_text(
        &self,
        theme: &ResolvedTheme,
        text: &PanelAxisTextPlan,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        Frame::draw(&self.projection, theme, backend)?;
        self.x_axis.draw_planned(
            &self.projection,
            AxisSide::Bottom,
            theme,
            &text.x_axis,
            backend,
        )?;
        self.y_axis.draw_planned(
            &self.projection,
            AxisSide::Left,
            theme,
            &text.y_axis,
            backend,
        )
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
            let block = text_block(
                theme,
                &tick.label,
                &theme.typography.tick_label,
                maximum_width,
                match side {
                    AxisSide::Bottom => PhysicalTextAlignment::Center,
                    AxisSide::Left => PhysicalTextAlignment::End,
                },
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
            AxisSide::Bottom => (x1 - x0) / self.ticks.len().max(1) as f64,
            AxisSide::Left => ((x1 - x0) * 0.4).max(1.0),
        }
    }

    fn plan_text(
        &self,
        projection: &Projection,
        side: AxisSide,
        theme: &ResolvedTheme,
    ) -> Result<AxisTextPlan, Error> {
        let maximum_width = self.tick_label_maximum_width(projection, side);
        let (alignment, tick_role, title_role, title_width) = match side {
            AxisSide::Bottom => (
                PhysicalTextAlignment::Center,
                "x-axis tick label",
                "x-axis title",
                (projection.pixel_box().1 - projection.pixel_box().0).max(1.0),
            ),
            AxisSide::Left => (
                PhysicalTextAlignment::End,
                "y-axis tick label",
                "y-axis title",
                (projection.pixel_box().3 - projection.pixel_box().2).max(1.0),
            ),
        };
        let tick_labels = self
            .ticks
            .iter()
            .map(|tick| {
                (!tick.label.is_empty())
                    .then(|| {
                        theme.plan_text(
                            &tick.label,
                            &theme.typography.tick_label,
                            maximum_width,
                            alignment,
                            tick_role,
                            false,
                        )
                    })
                    .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;
        let title = (!self.label.is_empty())
            .then(|| {
                theme.plan_text(
                    &self.label,
                    &theme.typography.axis_title,
                    title_width,
                    PhysicalTextAlignment::Center,
                    title_role,
                    false,
                )
            })
            .transpose()?;
        Ok(AxisTextPlan { tick_labels, title })
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn draw_planned(
        &self,
        projection: &Projection,
        side: AxisSide,
        theme: &ResolvedTheme,
        text: &AxisTextPlan,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        let (x0, x1, y0, y1) = projection.pixel_box();
        // y0 = bottom of plot area (math), y1 = top (math).

        let frame = theme.axes.frame.as_ref();
        let ticks = theme.axes.ticks.as_ref();
        let frame_style = frame.map(|appearance| appearance.stroke.backend_style());
        let tick_style = ticks.map(|appearance| appearance.stroke.backend_style());
        let tick_length = ticks.map_or(0.0, |appearance| f64::from(appearance.length));
        if text.tick_labels.len() != self.ticks.len() {
            return Err(Error::Render(
                "planned axis tick-label count does not match its axis".into(),
            ));
        }

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
        for (tick, tick_plan) in self.ticks.iter().zip(&text.tick_labels) {
            match side {
                AxisSide::Bottom => {
                    let (x_px, _) = projection.project(tick.position, projection.y_range().0);
                    if let Some(tick_style) = &tick_style {
                        backend.draw_line(
                            &[Point2D::new(x_px, y0), Point2D::new(x_px, y0 - tick_length)],
                            tick_style,
                        )?;
                    }
                    if let Some(plan) = tick_plan {
                        let first_y = y0
                            - tick_length
                            - theme.axes.spacing.x_tick_label_gap
                            - plan.line_height() / 2.0;
                        let center_y = first_y
                            - plan.lines.len().saturating_sub(1) as f64 * plan.line_height() / 2.0;
                        backend.draw_planned_text(
                            PlannedTextPlacement::middle(
                                Point2D::new(x_px.round(), center_y.round()),
                                TextAnchor::Middle,
                                0.0,
                            ),
                            plan,
                        )?;
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
                    if let Some(plan) = tick_plan {
                        let first_y = y_px
                            + plan.lines.len().saturating_sub(1) as f64 * plan.line_height() / 2.0;
                        let center_y = first_y
                            - plan.lines.len().saturating_sub(1) as f64 * plan.line_height() / 2.0;
                        backend.draw_planned_text(
                            PlannedTextPlacement::middle(
                                Point2D::new(
                                    (x0 - tick_length - theme.axes.spacing.y_tick_label_gap)
                                        .max(0.0)
                                        .round(),
                                    center_y.round(),
                                ),
                                TextAnchor::End,
                                0.0,
                            ),
                            plan,
                        )?;
                    }
                }
            }
        }

        // Axis label.
        if let Some(block) = &text.title {
            match side {
                AxisSide::Bottom => {
                    let cx = f64::midpoint(x0, x1);
                    let tick_label_height = self
                        .ticks
                        .iter()
                        .enumerate()
                        .filter_map(|(index, _)| {
                            text.tick_labels
                                .get(index)
                                .and_then(Option::as_ref)
                                .map(ResolvedTextPlan::height)
                        })
                        .fold(0.0_f64, f64::max);
                    let tick_offset = if tick_label_height > 0.0 {
                        theme.axes.spacing.x_tick_label_gap
                            + tick_label_height
                            + theme.axes.spacing.x_title_tick_label_gap
                    } else {
                        theme.axes.spacing.x_title_without_tick_labels_gap
                    };
                    let first_y = y0 - tick_length - tick_offset - block.line_height() / 2.0;
                    let center_y = first_y
                        - block.lines.len().saturating_sub(1) as f64 * block.line_height() / 2.0;
                    backend.draw_planned_text(
                        PlannedTextPlacement::middle(
                            Point2D::new(cx.round(), center_y.round()),
                            TextAnchor::Middle,
                            0.0,
                        ),
                        block,
                    )?;
                }
                AxisSide::Left => {
                    let cy = f64::midpoint(y0, y1);
                    let tick_label_width = self
                        .ticks
                        .iter()
                        .enumerate()
                        .filter_map(|(index, _)| {
                            text.tick_labels
                                .get(index)
                                .and_then(Option::as_ref)
                                .map(ResolvedTextPlan::width)
                        })
                        .fold(0.0_f64, f64::max);
                    let tick_label_anchor_x =
                        x0 - tick_length - theme.axes.spacing.y_tick_label_gap;
                    let label_half_width =
                        y_axis_label_half_width(theme.typography.axis_title.font_size);
                    let min_lx = theme.axes.spacing.y_title_edge_padding
                        + label_half_width
                        + block.lines.len().saturating_sub(1) as f64 * block.line_height();
                    let lx = if tick_label_width > 0.0 {
                        tick_label_anchor_x
                            - tick_label_width
                            - theme.axes.spacing.y_title_tick_label_gap
                            - label_half_width
                    } else {
                        x0 - tick_length - 48.0
                    }
                    .max(min_lx);
                    let center_x =
                        lx - block.lines.len().saturating_sub(1) as f64 * block.line_height() / 2.0;
                    backend.draw_planned_text(
                        PlannedTextPlacement::middle(
                            Point2D::new(center_x.max(0.0).round(), cy.round()),
                            TextAnchor::Middle,
                            -90.0,
                        ),
                        block,
                    )?;
                }
            }
        }

        Ok(())
    }
}

fn text_block(
    theme: &ResolvedTheme,
    source: &str,
    appearance: &crate::theme::TextAppearance,
    maximum_width: f64,
    alignment: PhysicalTextAlignment,
    owner: &str,
) -> Result<TextBlock, Error> {
    theme
        .plan_text(source, appearance, maximum_width, alignment, owner, false)
        .map(TextBlock::from_resolved)
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
