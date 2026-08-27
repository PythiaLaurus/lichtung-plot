//! Title, subtitle, and caption rendering.
//!
//! A `TitleBlock` sits outside the data plot area:
//! - Title is drawn at the top, centered horizontally on the canvas.
//! - Subtitle is drawn below the title (smaller font), also centered.
//! - Caption is drawn at the bottom of the canvas, centered horizontally.
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
use crate::render::backend::{Backend, Point2D, TextAnchor, TextStyle};
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
    /// Font family for all three text lines.
    font_family: String,
}

impl TitleBlock {
    /// Create an empty `TitleBlock` (no text rendered).
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            font_family: crate::DEFAULT_FONT_FAMILY_STACK.to_string(),
            ..Self::default()
        }
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

    /// Set the font family used for title/subtitle/caption.
    #[must_use]
    pub(crate) fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }

    /// Whether any of the title / subtitle / caption fields is
    /// non-empty. The orchestrator uses this to skip drawing
    /// the title block when nothing is set.
    #[must_use]
    pub(crate) fn has_content(&self) -> bool {
        self.title.as_ref().is_some_and(|s| !s.is_empty())
            || self.subtitle.as_ref().is_some_and(|s| !s.is_empty())
            || self.caption.as_ref().is_some_and(|s| !s.is_empty())
    }

    /// Draw the title block at the given canvas anchor positions.
    ///
    /// - `canvas_width` is used to center each line horizontally.
    /// - `top_y` is the pixel y-coordinate of the title's baseline
    ///   (top of the canvas).
    /// - `bottom_y` is the pixel y-coordinate of the caption's
    ///   baseline (bottom of the canvas).
    ///
    /// All three lines are drawn centered. Missing lines are skipped.
    pub(crate) fn draw(
        &self,
        backend: &mut dyn Backend,
        canvas_width: u32,
        canvas: &CanvasLayout,
        palette: &ResolvedTheme,
    ) -> Result<(), Error> {
        let cx = f64::from(canvas_width) / 2.0;

        if let Some(title) = &self.title {
            let style = TextStyle {
                color: palette.foreground,
                font_family: self.font_family.clone(),
                font_size: palette.title_font_size,
                weight: crate::TextWeight::Normal,
                anchor: TextAnchor::Middle,
                rotation: 0.0,
            };
            for (line, y) in crate::layout::canvas::try_wrap_text(
                title,
                palette.title_font_size,
                (f64::from(canvas_width) - 32.0).max(1.0),
                "canvas title",
            )?
            .iter()
            .zip(&canvas.title_lines_y)
            {
                backend.draw_text(Point2D::new(cx.round(), y.round()), line, &style)?;
            }
        }

        if let Some(subtitle) = &self.subtitle {
            let style = TextStyle {
                color: (80, 80, 80),
                font_family: self.font_family.clone(),
                font_size: palette.subtitle_font_size,
                weight: crate::TextWeight::Normal,
                anchor: TextAnchor::Middle,
                rotation: 0.0,
            };
            for (line, y) in crate::layout::canvas::try_wrap_text(
                subtitle,
                palette.subtitle_font_size,
                (f64::from(canvas_width) - 32.0).max(1.0),
                "canvas subtitle",
            )?
            .iter()
            .zip(&canvas.subtitle_lines_y)
            {
                backend.draw_text(Point2D::new(cx.round(), y.round()), line, &style)?;
            }
        }

        if let Some(caption) = &self.caption {
            let style = TextStyle {
                color: (80, 80, 80),
                font_family: self.font_family.clone(),
                font_size: palette.caption_font_size,
                weight: crate::TextWeight::Normal,
                anchor: TextAnchor::Middle,
                rotation: 0.0,
            };
            for (line, y) in crate::layout::canvas::try_wrap_text(
                caption,
                palette.caption_font_size,
                (f64::from(canvas_width) - 32.0).max(1.0),
                "canvas caption",
            )?
            .iter()
            .zip(&canvas.caption_lines_y)
            {
                backend.draw_text(Point2D::new(cx.round(), y.round()), line, &style)?;
            }
        }

        Ok(())
    }
}
