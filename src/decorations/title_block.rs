//! Title, subtitle, and caption rendering.
//!
//! Standalone Plot roots consume the immutable assigned-width producer in
//! `plan`. Composed roots retain their deferred centered placement until their
//! T5a6a adapters connect that same producer. Coordinates are math-style pixels.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use crate::layout::{CanvasLayout, ResolvedTextPlan};
use crate::render::backend::{Backend, PlannedTextPlacement, Point2D, TextAnchor};
use crate::Error;

pub(crate) mod plan;

/// A canvas-level title / subtitle / caption block.
#[derive(Debug, Clone, Default)]
pub(crate) struct TitleBlock {
    /// Main title text (top of the canvas).
    title: Option<String>,
    /// Subtitle text (below the title, smaller).
    subtitle: Option<String>,
    /// Caption text (bottom of the canvas).
    caption: Option<String>,
}

impl TitleBlock {
    /// Create an empty `TitleBlock` (no text rendered).
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Set the title text.
    #[must_use]
    pub(crate) fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the subtitle text.
    #[must_use]
    pub(crate) fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set the caption text.
    #[must_use]
    pub(crate) fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Whether any of the title / subtitle / caption fields is
    /// non-empty. The orchestrator uses this to skip drawing
    /// the title block when nothing is set.
    #[must_use]
    pub(crate) fn has_content(&self) -> bool {
        self.title.as_ref().is_some_and(|value| !value.is_empty())
            || self
                .subtitle
                .as_ref()
                .is_some_and(|value| !value.is_empty())
            || self.caption.as_ref().is_some_and(|value| !value.is_empty())
    }

    /// Draw the title block at the given canvas anchor positions.
    ///
    /// - `canvas_width` is used to center each line horizontally.
    /// - `top_y` is the pixel y-coordinate of the title's baseline
    ///   (top of the canvas).
    /// - `bottom_y` is the pixel y-coordinate of the caption's
    ///   baseline (bottom of the canvas).
    ///
    /// Standalone roots consume frozen role placements. Only deferred composition
    /// still uses the centered anchors below. Missing roles are skipped.
    pub(crate) fn draw(
        &self,
        backend: &mut dyn Backend,
        canvas_width: u32,
        canvas: &CanvasLayout,
    ) -> Result<(), Error> {
        let (plans, baselines) = match &canvas.root_text {
            crate::layout::canvas::CanvasRootText::Planned(plan) => {
                return plan.draw(
                    [
                        self.title.as_deref(),
                        self.subtitle.as_deref(),
                        self.caption.as_deref(),
                    ],
                    backend,
                )
            }
            crate::layout::canvas::CanvasRootText::Deferred { plans, baselines } => {
                (plans, baselines)
            }
        };
        let x = f64::from(canvas_width) / 2.0;

        draw_planned_role(
            backend,
            x,
            self.title.as_deref().filter(|value| !value.is_empty()),
            plans[0].as_ref(),
            &baselines[0],
            "title",
        )?;
        draw_planned_role(
            backend,
            x,
            self.subtitle.as_deref().filter(|value| !value.is_empty()),
            plans[1].as_ref(),
            &baselines[1],
            "subtitle",
        )?;
        draw_planned_role(
            backend,
            x,
            self.caption.as_deref().filter(|value| !value.is_empty()),
            plans[2].as_ref(),
            &baselines[2],
            "caption",
        )?;

        Ok(())
    }
}

fn draw_planned_role(
    backend: &mut dyn Backend,
    x: f64,
    source: Option<&str>,
    plan: Option<&ResolvedTextPlan>,
    line_positions: &[f64],
    role: &str,
) -> Result<(), Error> {
    let plan = match (source, plan) {
        (None, None) => return Ok(()),
        (Some(source), Some(plan)) if plan.source == source => plan,
        _ => {
            return Err(Error::Render(format!(
                "planned canvas {role} does not match its configured source"
            )))
        }
    };
    if plan.lines.len() != line_positions.len() {
        return Err(Error::Render(format!(
            "planned canvas {role} line count does not match its layout"
        )));
    }
    let Some((&first, &last)) = line_positions.first().zip(line_positions.last()) else {
        return Ok(());
    };
    backend.draw_planned_text(
        PlannedTextPlacement::middle(
            Point2D::new(x.round(), f64::midpoint(first, last).round()),
            TextAnchor::Middle,
            0.0,
        ),
        plan,
    )
}
