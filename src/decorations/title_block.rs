//! Title, subtitle, and caption rendering.
//!
//! A `TitleBlock` sits outside the data plot area:
//! - Title is drawn centered at the top.
//! - Subtitle is drawn centered below the title.
//! - Caption is drawn centered at the bottom.
//!
//! Coordinates use canvas pixel space (top-left origin). The
//! actual placement is the caller's responsibility — the
//! `Layout` knows the plot panel's `(x0, x1, y0, y1)` and can
//! hand the right `(x_center, y_top, y_bottom)` values to
//! `TitleBlock::draw`.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use crate::layout::CanvasLayout;
use crate::render::backend::{Backend, Point2D, TextAnchor};
use crate::theme::ResolvedTheme;
use crate::Error;

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
    /// All three roles remain centered until root title-block planning activates
    /// the resolved alignment. Missing roles are skipped.
    pub(crate) fn draw(
        &self,
        backend: &mut dyn Backend,
        canvas_width: u32,
        canvas: &CanvasLayout,
        theme: &ResolvedTheme,
    ) -> Result<(), Error> {
        let x = f64::from(canvas_width) / 2.0;
        let anchor = TextAnchor::Middle;

        if let Some(title) = self.title.as_deref().filter(|value| !value.is_empty()) {
            let role = &theme.typography.title;
            let style = role.backend_style(anchor, 0.0);
            for (line, y) in crate::layout::canvas::try_wrap_text(
                title,
                role.font_size,
                (f64::from(canvas_width) - 32.0).max(1.0),
                "canvas title",
            )?
            .iter()
            .zip(&canvas.title_lines_y)
            {
                backend.draw_text(Point2D::new(x.round(), y.round()), line, &style)?;
            }
        }

        if let Some(subtitle) = self.subtitle.as_deref().filter(|value| !value.is_empty()) {
            let role = &theme.typography.subtitle;
            let style = role.backend_style(anchor, 0.0);
            for (line, y) in crate::layout::canvas::try_wrap_text(
                subtitle,
                role.font_size,
                (f64::from(canvas_width) - 32.0).max(1.0),
                "canvas subtitle",
            )?
            .iter()
            .zip(&canvas.subtitle_lines_y)
            {
                backend.draw_text(Point2D::new(x.round(), y.round()), line, &style)?;
            }
        }

        if let Some(caption) = self.caption.as_deref().filter(|value| !value.is_empty()) {
            let role = &theme.typography.caption;
            let style = role.backend_style(anchor, 0.0);
            for (line, y) in crate::layout::canvas::try_wrap_text(
                caption,
                role.font_size,
                (f64::from(canvas_width) - 32.0).max(1.0),
                "canvas caption",
            )?
            .iter()
            .zip(&canvas.caption_lines_y)
            {
                backend.draw_text(Point2D::new(x.round(), y.round()), line, &style)?;
            }
        }

        Ok(())
    }
}
