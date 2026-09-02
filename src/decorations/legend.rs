//! `lichtung-plot::decorations::legend` — plot legend rendering.
//!
//! Coordinates use the **math coordinate system** (y increases upward).
//! The SVG backend flips y internally via `y_mark()`. Therefore, to lay
//! out elements "top-to-bottom" visually, we *decrement* y_offset.
//!
//! ## Sizing philosophy
//!
//! All dimensions are derived from a computed `font_size` which itself
//! scales with the canvas size:
//!   font_size = BASE_FONT × (min(canvas_w, canvas_h) / REF_SIZE)
//! Spacings are multiples of font_size. This ensures the legend looks
//! proportional on any canvas size while remaining overridable.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss
)]

use crate::layout::{PhysicalTextAlignment, TextBlock};
use crate::render::backend::{
    Backend, FillStyle, LineStyle, Point2D, PointStyle, TextAnchor, TextStyle,
};
use crate::Error;

/// Raw semantic identity for one renderer-owned legend entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LegendIdentity {
    Single {
        field: String,
        value: String,
    },
    Pair {
        group_field: String,
        group_value: String,
        hue_field: String,
        hue_value: String,
    },
    Literal {
        title: String,
        value: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LegendTitleIdentity {
    Single(String),
    Pair(String, String),
    Literal(String),
}

impl LegendIdentity {
    pub(crate) fn single(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Single {
            field: field.into(),
            value: value.into(),
        }
    }

    pub(crate) fn pair(
        group_field: impl Into<String>,
        group_value: impl Into<String>,
        hue_field: impl Into<String>,
        hue_value: impl Into<String>,
    ) -> Self {
        Self::Pair {
            group_field: group_field.into(),
            group_value: group_value.into(),
            hue_field: hue_field.into(),
            hue_value: hue_value.into(),
        }
    }

    pub(crate) fn literal(title: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Literal {
            title: title.into(),
            value: value.into(),
        }
    }

    fn title_identity(&self) -> LegendTitleIdentity {
        match self {
            Self::Single { field, .. } => LegendTitleIdentity::Single(field.clone()),
            Self::Pair {
                group_field,
                hue_field,
                ..
            } => LegendTitleIdentity::Pair(group_field.clone(), hue_field.clone()),
            Self::Literal { title, .. } => LegendTitleIdentity::Literal(title.clone()),
        }
    }

    pub(crate) fn resolved_text(&self, plot: &crate::Plot) -> (String, String) {
        match self {
            Self::Single { field, value } => (plot.resolve_label(field), plot.resolve_label(value)),
            Self::Pair {
                group_field,
                group_value,
                hue_field,
                hue_value,
            } => (
                format!(
                    "{} × {}",
                    plot.resolve_label(group_field),
                    plot.resolve_label(hue_field)
                ),
                format!(
                    "{}, {}",
                    plot.resolve_label(group_value),
                    plot.resolve_label(hue_value)
                ),
            ),
            Self::Literal { title, value } => (title.clone(), value.clone()),
        }
    }

    pub(crate) fn hue_value_for(&self, field: &str) -> Option<&str> {
        match self {
            Self::Single {
                field: entry_field,
                value,
            } if entry_field == field => Some(value),
            Self::Pair {
                hue_field,
                hue_value,
                ..
            } if hue_field == field => Some(hue_value),
            _ => None,
        }
    }
}

/// Exact visible components of one legend glyph.
#[derive(Debug, Clone, Default)]
struct LegendGlyph {
    point: Option<PointStyle>,
    line: Option<LineStyle>,
    fill: Option<FillStyle>,
}

impl LegendGlyph {
    fn exact_eq(&self, other: &Self) -> bool {
        optional_point_style_eq(self.point.as_ref(), other.point.as_ref())
            && optional_line_style_eq(self.line.as_ref(), other.line.as_ref())
            && optional_fill_style_eq(self.fill.as_ref(), other.fill.as_ref())
    }
}

/// One private renderer-owned legend identity and its exact visible glyph.
#[derive(Debug, Clone)]
pub(crate) struct LegendEntry {
    identity: LegendIdentity,
    title: String,
    label: String,
    glyph: LegendGlyph,
    contributes_hue_level: bool,
}

impl LegendEntry {
    fn new(identity: LegendIdentity, glyph: LegendGlyph) -> Self {
        Self {
            identity,
            title: String::new(),
            label: String::new(),
            glyph,
            contributes_hue_level: true,
        }
    }

    pub(crate) fn point(identity: LegendIdentity, point: PointStyle) -> Self {
        Self::new(
            identity,
            LegendGlyph {
                point: Some(point),
                ..LegendGlyph::default()
            },
        )
    }

    pub(crate) fn line(identity: LegendIdentity, line: LineStyle) -> Self {
        Self::new(
            identity,
            LegendGlyph {
                line: Some(line),
                ..LegendGlyph::default()
            },
        )
    }

    pub(crate) fn fill(identity: LegendIdentity, fill: FillStyle) -> Self {
        Self::new(
            identity,
            LegendGlyph {
                fill: Some(fill),
                ..LegendGlyph::default()
            },
        )
    }

    pub(crate) fn point_line(identity: LegendIdentity, point: PointStyle, line: LineStyle) -> Self {
        Self::new(
            identity,
            LegendGlyph {
                point: Some(point),
                line: Some(line),
                fill: None,
            },
        )
    }

    pub(crate) fn point_fill(identity: LegendIdentity, point: PointStyle, fill: FillStyle) -> Self {
        Self::new(
            identity,
            LegendGlyph {
                point: Some(point),
                line: None,
                fill: Some(fill),
            },
        )
    }

    pub(crate) fn line_fill(identity: LegendIdentity, line: LineStyle, fill: FillStyle) -> Self {
        Self::new(
            identity,
            LegendGlyph {
                point: None,
                line: Some(line),
                fill: Some(fill),
            },
        )
    }

    pub(crate) fn point_line_fill(
        identity: LegendIdentity,
        point: PointStyle,
        line: LineStyle,
        fill: FillStyle,
    ) -> Self {
        Self::new(
            identity,
            LegendGlyph {
                point: Some(point),
                line: Some(line),
                fill: Some(fill),
            },
        )
    }

    pub(crate) fn resolve(&mut self, plot: &crate::Plot) {
        (self.title, self.label) = self.identity.resolved_text(plot);
    }

    pub(crate) fn identity(&self) -> &LegendIdentity {
        &self.identity
    }

    pub(crate) fn without_hue_level(mut self) -> Self {
        self.contributes_hue_level = false;
        self
    }

    pub(crate) fn coordinated_hue_value_for(&self, field: &str) -> Option<&str> {
        self.contributes_hue_level
            .then(|| self.identity.hue_value_for(field))
            .flatten()
    }

    fn exact_eq(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.title == other.title
            && self.label == other.label
            && self.glyph.exact_eq(&other.glyph)
    }

    fn glyph_extent(&self) -> f64 {
        self.glyph
            .point
            .as_ref()
            .map_or(0.0, |point| f64::from(point.radius))
    }
}

pub(crate) fn resolve_entries(
    plot: &crate::Plot,
    entries: &mut Vec<LegendEntry>,
) -> Result<(), Error> {
    for entry in entries.iter_mut() {
        entry.resolve(plot);
    }
    *entries = merge_exact_entries(std::mem::take(entries), "plot legend")?;
    Ok(())
}

pub(crate) fn merge_exact_entries(
    entries: impl IntoIterator<Item = LegendEntry>,
    context: &str,
) -> Result<Vec<LegendEntry>, Error> {
    let mut merged: Vec<LegendEntry> = Vec::new();
    for entry in entries {
        if let Some(first) = merged.first() {
            if first.identity.title_identity() != entry.identity.title_identity()
                || first.title != entry.title
            {
                return Err(Error::Render(format!(
                    "{context} requires compatible semantic titles, labels, and styles; glyphs must match exactly"
                )));
            }
        }
        if merged
            .iter()
            .any(|existing| existing.identity != entry.identity && existing.label == entry.label)
        {
            return Err(Error::Render(format!(
                "{context} has distinct raw identities resolving to display label {:?}",
                entry.label
            )));
        }
        if let Some(existing) = merged
            .iter()
            .find(|existing| existing.identity == entry.identity)
        {
            if !existing.exact_eq(&entry) {
                return Err(Error::Render(format!(
                    "{context} requires compatible semantic titles, labels, and styles; glyphs must match exactly"
                )));
            }
        } else {
            merged.push(entry);
        }
    }
    Ok(merged)
}

pub(crate) fn resolved_title(entries: &[LegendEntry]) -> Result<String, Error> {
    let Some(first) = entries.first() else {
        return Ok(String::new());
    };
    if entries.iter().any(|entry| {
        entry.identity.title_identity() != first.identity.title_identity()
            || entry.title != first.title
    }) {
        return Err(Error::Render(
            "legend entries require one compatible semantic title".into(),
        ));
    }
    Ok(first.title.clone())
}

fn optional_point_style_eq(left: Option<&PointStyle>, right: Option<&PointStyle>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            left.color == right.color
                && left.radius == right.radius
                && left.shape == right.shape
                && left.opacity.to_bits() == right.opacity.to_bits()
                && match (left.outline, right.outline) {
                    (Some(left), Some(right)) => {
                        left.color == right.color
                            && left.width_ratio.to_bits() == right.width_ratio.to_bits()
                    }
                    (None, None) => true,
                    _ => false,
                }
        }
        (None, None) => true,
        _ => false,
    }
}

fn optional_line_style_eq(left: Option<&LineStyle>, right: Option<&LineStyle>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            left.color == right.color
                && left.width.to_bits() == right.width.to_bits()
                && left.pattern == right.pattern
        }
        (None, None) => true,
        _ => false,
    }
}

fn optional_fill_style_eq(left: Option<&FillStyle>, right: Option<&FillStyle>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            left.color == right.color
                && left.stroke == right.stroke
                && left.stroke_width.to_bits() == right.stroke_width.to_bits()
                && left.opacity.to_bits() == right.opacity.to_bits()
        }
        (None, None) => true,
        _ => false,
    }
}

// ─── Reference constants ────────────────────────────────────────────────────

/// Reference canvas dimension (px) at which BASE_FONT applies 1:1.
const REF_SIZE: f64 = 640.0;
/// Base font size at REF_SIZE canvas.
const BASE_FONT: f64 = 14.0;
/// Minimum font size to ensure readability on very small canvases.
const MIN_FONT: f64 = 11.0;
/// Maximum font size to avoid oversized legends on huge canvases.
const MAX_FONT: f64 = 18.0;

// ─── Fontsize-relative multipliers (following matplotlib conventions) ────────

/// Padding between legend border and content (× font_size).
const BORDER_PAD_FACTOR: f64 = 0.8;
/// Vertical spacing between entries (× font_size).
const LABEL_SPACING_FACTOR: f64 = 0.5;
/// Horizontal pad between marker and label text (× font_size).
const HANDLE_TEXT_PAD_FACTOR: f64 = 0.8;
/// Width of the marker area (× font_size).
const HANDLE_LENGTH_FACTOR: f64 = 1.0;
/// Conservative half-height around the backend's middle-aligned text anchor.
const GLYPH_HALF_HEIGHT_FACTOR: f64 = 0.6;
/// Extra edge comfort retained from the previous bottom padding (x font_size).
const GLYPH_EDGE_PAD_FACTOR: f64 = 0.2;
/// Horizontal separation between adjacent legend columns (x font_size).
const COLUMN_SPACING_FACTOR: f64 = 1.2;
/// Maximum number of entries retained in one automatic legend column.
const MAX_SINGLE_COLUMN_ENTRIES: usize = 6;
/// Title line height relative to font_size.
const TITLE_HEIGHT_FACTOR: f64 = 1.8;
/// Background frame opacity (matplotlib default: 0.8).
const FRAME_ALPHA: f64 = 0.8;
/// Corner radius for the background box.
const CORNER_RADIUS: f64 = 2.0;

/// Computed metrics for legend layout, derived from canvas size.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LegendMetrics {
    /// Effective font size (px).
    pub font_size: f64,
    /// Border padding (px).
    pub pad: f64,
    /// Entry height (px) = font_size + label_spacing.
    pub entry_h: f64,
    /// Title height (px).
    pub title_h: f64,
    /// Handle (marker) width (px).
    pub handle_w: f64,
    /// Handle-to-text gap (px).
    pub handle_text: f64,
}

impl LegendMetrics {
    /// Compute metrics from canvas dimensions.
    /// Uses `min(canvas_w, canvas_h)` as the reference for scaling.
    pub fn from_canvas(canvas_w: f64, canvas_h: f64) -> Self {
        let scale = canvas_w.min(canvas_h) / REF_SIZE;
        let font_size = (BASE_FONT * scale).clamp(MIN_FONT, MAX_FONT);
        let pad = BORDER_PAD_FACTOR * font_size;
        let entry_h = font_size + LABEL_SPACING_FACTOR * font_size;
        let title_h = TITLE_HEIGHT_FACTOR * font_size;
        let handle_w = HANDLE_LENGTH_FACTOR * font_size;
        let handle_text = HANDLE_TEXT_PAD_FACTOR * font_size;
        Self {
            font_size,
            pad,
            entry_h,
            title_h,
            handle_w,
            handle_text,
        }
    }
}

/// A plot legend.
pub(crate) struct Legend {
    /// The entries in the legend.
    entries: Vec<LegendEntry>,
    /// Layout metrics (derived from canvas size or overridden).
    pub(crate) metrics: LegendMetrics,
    available_width: f64,
    available_height: f64,
    title_block: TextBlock,
    entry_blocks: Vec<TextBlock>,
    font_family: String,
}

/// A resolved legend rectangle in math-style pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LegendBox {
    /// Left edge.
    pub left: f64,
    /// Right edge.
    pub right: f64,
    /// Bottom edge.
    pub bottom: f64,
    /// Top edge.
    pub top: f64,
    anchor_x: f64,
    anchor_y: f64,
}

impl LegendBox {
    /// Return the anchor expected by [`Legend::draw`].
    #[must_use]
    pub fn anchor(self) -> (f64, f64) {
        (self.anchor_x, self.anchor_y)
    }

    pub(crate) fn translated(self, x: f64, y: f64) -> Self {
        Self {
            left: self.left + x,
            right: self.right + x,
            bottom: self.bottom + y,
            top: self.top + y,
            anchor_x: self.anchor_x + x,
            anchor_y: self.anchor_y + y,
        }
    }
}

impl Legend {
    /// Create a new legend with metrics derived from canvas size.
    pub fn new(
        title: &str,
        entries: Vec<LegendEntry>,
        canvas_w: f64,
        canvas_h: f64,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<Self, Error> {
        let metrics = LegendMetrics::from_canvas(canvas_w, canvas_h);
        let title_maximum_width = (canvas_w * 0.5).max(metrics.font_size);
        let entry_maximum_width = (canvas_w * 0.35).max(metrics.font_size);
        let title_font_size = (metrics.font_size * 1.1).round() as u32;
        let entry_font_size = metrics.font_size.round() as u32;
        let title_block = TextBlock::from_resolved(theme.plan_legacy_text(
            title,
            title_font_size,
            title_maximum_width,
            PhysicalTextAlignment::Start,
            "legend title",
        )?);
        let entry_blocks = entries
            .iter()
            .map(|entry| {
                theme
                    .plan_legacy_text(
                        &entry.label,
                        entry_font_size,
                        entry_maximum_width,
                        PhysicalTextAlignment::Start,
                        "legend entry",
                    )
                    .map(TextBlock::from_resolved)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            entries,
            metrics,
            available_width: canvas_w * 0.8,
            available_height: canvas_h * 0.8,
            title_block,
            entry_blocks,
            font_family: theme.font_family.clone(),
        })
    }

    /// Compute the total bounding box width based on entry labels.
    pub fn compute_box_width(&self) -> f64 {
        self.box_size_for_columns(self.column_count()).0
    }

    fn box_size_for_columns(&self, columns: usize) -> (f64, f64) {
        let m = &self.metrics;
        let columns = columns.clamp(1, self.entries.len().max(1));
        let title_block = &self.title_block;
        let entry_blocks = &self.entry_blocks;
        let mut column_widths = vec![m.handle_w + m.handle_text; columns];
        for (index, block) in entry_blocks.iter().enumerate() {
            column_widths[index % columns] =
                column_widths[index % columns].max(m.handle_w + m.handle_text + block.width());
        }
        let entries_width = column_widths.iter().sum::<f64>()
            + (columns.saturating_sub(1) as f64) * self.column_spacing();
        let title_width = title_block.width();
        let width = entries_width.max(title_width) + m.pad * 2.0;
        let content_height = if self.entries.is_empty() {
            title_block.height()
        } else {
            let centers = self.row_centers(columns, title_block, entry_blocks);
            self.title_half_height()
                + -centers.last().copied().unwrap_or(0.0)
                + self.last_row_half_height(columns, entry_blocks)
        };
        (width, content_height + self.glyph_edge_pad() * 2.0)
    }

    /// Compute the total bounding box height.
    pub fn compute_box_height(&self) -> f64 {
        self.box_size_for_columns(self.column_count()).1
    }

    /// Pick the preferred right-inside-axes legend box.
    #[must_use]
    #[cfg(test)]
    pub fn right_box(&self, pixel_box: (f64, f64, f64, f64)) -> LegendBox {
        let (x0, x1, y0, y1) = pixel_box;
        let plot_short = (x1 - x0).min(y1 - y0);
        let inset = plot_short * 0.04;
        let width = self.compute_box_width();
        let height = self.compute_box_height();
        let bottom = f64::midpoint(y0, y1) - height / 2.0;
        self.box_from_rect(x1 - inset - width, bottom, width, height)
    }

    /// Pick an inside-axes legend box by normalized x/y alignment.
    #[must_use]
    pub fn aligned_inside_box(
        &self,
        pixel_box: (f64, f64, f64, f64),
        x_align: f64,
        y_align: f64,
    ) -> LegendBox {
        let (x0, x1, y0, y1) = pixel_box;
        let plot_short = (x1 - x0).min(y1 - y0);
        let inset = plot_short * 0.04;
        let width = self.compute_box_width();
        let height = self.compute_box_height();
        let available_w = (x1 - x0 - inset * 2.0 - width).max(0.0);
        let available_h = (y1 - y0 - inset * 2.0 - height).max(0.0);
        self.box_from_rect(
            x0 + inset + available_w * x_align.clamp(0.0, 1.0),
            y0 + inset + available_h * y_align.clamp(0.0, 1.0),
            width,
            height,
        )
    }

    /// Build a legend box from its lower-left corner and total size.
    #[must_use]
    pub fn box_at(&self, left: f64, bottom: f64) -> LegendBox {
        self.box_from_rect(
            left,
            bottom,
            self.compute_box_width(),
            self.compute_box_height(),
        )
    }

    pub(crate) fn inside_candidates(&self, pixel_box: (f64, f64, f64, f64)) -> [LegendBox; 4] {
        let (x0, x1, y0, y1) = pixel_box;
        let plot_short = (x1 - x0).min(y1 - y0);
        let inset = plot_short * 0.04;
        let width = self.compute_box_width();
        let height = self.compute_box_height();
        // Tie-breaker order favors top placements when overlap scores are equal.
        [
            self.box_from_rect(x1 - inset - width, y1 - inset - height, width, height),
            self.box_from_rect(x0 + inset, y1 - inset - height, width, height),
            self.box_from_rect(x1 - inset - width, y0 + inset, width, height),
            self.box_from_rect(x0 + inset, y0 + inset, width, height),
        ]
    }

    fn box_from_rect(&self, left: f64, bottom: f64, width: f64, height: f64) -> LegendBox {
        let left = left.max(0.0);
        let bottom = bottom.max(0.0);
        LegendBox {
            left,
            right: left + width,
            bottom,
            top: bottom + height,
            anchor_x: left + self.metrics.pad,
            anchor_y: bottom + height - self.glyph_edge_pad() - self.title_half_height(),
        }
    }

    /// Draw the legend with the **top-left corner** at `(x, y)` in math coords.
    ///
    /// The legend extends *downward* (decreasing y in math coords) from this
    /// anchor point. The SVG backend's y-flip ensures correct visual output.
    pub fn draw(&self, backend: &mut dyn Backend, x: f64, y: f64) -> Result<(), Error> {
        let m = &self.metrics;
        let columns = self.column_count();
        let title_block = &self.title_block;
        let entry_blocks = &self.entry_blocks;

        let box_w = self.compute_box_width();
        let box_h = self.compute_box_height();

        // ─── Background box ───
        // In math coords: top-left content is (x, y), box extends pad outward.
        let rx = x - m.pad;
        let ry_bot = y + self.glyph_edge_pad() + self.title_half_height() - box_h;

        let bg_fill = FillStyle {
            color: (255, 255, 255),
            stroke: (200, 200, 200),
            stroke_width: 1.0,
            opacity: FRAME_ALPHA,
        };
        backend.draw_rounded_rect(rx, ry_bot, box_w, box_h, CORNER_RADIUS, &bg_fill)?;

        // ─── Title ───
        let title_font_size = self.title_font_size() as u32;
        let title_style = TextStyle {
            color: (60, 60, 60).into(),
            font_family: self.font_family.clone(),
            font_size: title_font_size,
            weight: crate::TextWeight::Normal,
            anchor: TextAnchor::Start,
            rotation: 0.0,
        };
        for (index, line) in title_block.lines().iter().enumerate() {
            backend.draw_text(
                Point2D::new(
                    x.round(),
                    (y - index as f64 * title_block.line_height()).round(),
                ),
                line,
                &title_style,
            )?;
        }
        // ─── Entries (top to bottom = decreasing y) ───
        let entry_style = TextStyle {
            color: (40, 40, 40).into(),
            font_family: self.font_family.clone(),
            font_size: self.entry_font_size() as u32,
            weight: crate::TextWeight::Normal,
            anchor: TextAnchor::Start,
            rotation: 0.0,
        };

        let column_widths = self.column_widths(columns, entry_blocks);
        let row_centers = self.row_centers(columns, title_block, entry_blocks);
        for (index, entry) in self.entries.iter().enumerate() {
            let column = index % columns;
            let row = index / columns;
            let column_x = x
                + column_widths[..column].iter().sum::<f64>()
                + column as f64 * self.column_spacing();
            let row_y = y + row_centers[row];
            // Renderer-owned fill, line, and point components share one handle.
            let marker_x = column_x + m.handle_w * 0.5;
            if let Some(fill_style) = &entry.glyph.fill {
                let half_height = self.entry_font_size() * 0.32;
                backend.draw_polygon(
                    &[
                        Point2D::new(column_x, row_y - half_height),
                        Point2D::new(column_x + m.handle_w, row_y - half_height),
                        Point2D::new(column_x + m.handle_w, row_y + half_height),
                        Point2D::new(column_x, row_y + half_height),
                    ],
                    fill_style,
                )?;
            }
            if let Some(line_style) = &entry.glyph.line {
                backend.draw_line(
                    &[
                        Point2D::new(column_x, row_y),
                        Point2D::new(column_x + m.handle_w, row_y),
                    ],
                    line_style,
                )?;
            }
            if let Some(point_style) = &entry.glyph.point {
                backend.draw_scatter(&[Point2D::new(marker_x, row_y)], point_style)?;
            }
            // Label text
            let text_x = column_x + m.handle_w + m.handle_text;
            let block = &entry_blocks[index];
            let first_y =
                row_y + block.lines().len().saturating_sub(1) as f64 * block.line_height() / 2.0;
            for (line_index, line) in block.lines().iter().enumerate() {
                backend.draw_text(
                    Point2D::new(
                        text_x.round(),
                        (first_y - line_index as f64 * block.line_height()).round(),
                    ),
                    line,
                    &entry_style,
                )?;
            }
        }

        Ok(())
    }

    fn title_font_size(&self) -> f64 {
        (self.metrics.font_size * 1.1).round()
    }

    fn entry_font_size(&self) -> f64 {
        self.metrics.font_size.round()
    }

    fn title_half_height(&self) -> f64 {
        GLYPH_HALF_HEIGHT_FACTOR * self.title_font_size()
    }

    fn entry_half_height(&self) -> f64 {
        GLYPH_HALF_HEIGHT_FACTOR * self.entry_font_size()
    }

    fn last_row_half_height(&self, columns: usize, entry_blocks: &[TextBlock]) -> f64 {
        let rows = self.entries.len().div_ceil(columns);
        let marker_extent = self
            .entries
            .iter()
            .enumerate()
            .filter(|(index, _)| index / columns == rows.saturating_sub(1))
            .map(|(_, entry)| entry.glyph_extent())
            .fold(0.0_f64, f64::max);
        let text_extent = entry_blocks
            .iter()
            .enumerate()
            .filter(|(index, _)| index / columns == rows.saturating_sub(1))
            .map(|(_, block)| block.height() / 2.0)
            .fold(self.entry_half_height(), f64::max);
        text_extent.max(marker_extent)
    }

    fn glyph_edge_pad(&self) -> f64 {
        self.metrics.pad + GLYPH_EDGE_PAD_FACTOR * self.entry_font_size()
    }

    fn column_spacing(&self) -> f64 {
        COLUMN_SPACING_FACTOR * self.metrics.font_size
    }

    fn column_widths(&self, columns: usize, entry_blocks: &[TextBlock]) -> Vec<f64> {
        let mut widths = vec![self.metrics.handle_w + self.metrics.handle_text; columns];
        for (index, block) in entry_blocks.iter().enumerate() {
            widths[index % columns] = widths[index % columns]
                .max(self.metrics.handle_w + self.metrics.handle_text + block.width());
        }
        widths
    }

    fn row_centers(
        &self,
        columns: usize,
        title_block: &TextBlock,
        entry_blocks: &[TextBlock],
    ) -> Vec<f64> {
        let rows = self.entries.len().div_ceil(columns);
        if rows == 0 {
            return Vec::new();
        }
        let title_extra =
            title_block.lines().len().saturating_sub(1) as f64 * title_block.line_height();
        let mut centers = vec![-self.metrics.title_h - title_extra];
        let row_gap = self.metrics.entry_h - self.entry_font_size() * 1.2;
        for row in 1..rows {
            let previous_half = self.row_half_height(row - 1, columns, entry_blocks);
            let current_half = self.row_half_height(row, columns, entry_blocks);
            centers.push(centers[row - 1] - previous_half - row_gap - current_half);
        }
        centers
    }

    fn row_half_height(&self, row: usize, columns: usize, entry_blocks: &[TextBlock]) -> f64 {
        entry_blocks
            .iter()
            .enumerate()
            .filter(|(index, _)| index / columns == row)
            .map(|(index, block)| (block.height() / 2.0).max(self.entries[index].glyph_extent()))
            .fold(self.entry_half_height(), f64::max)
    }

    fn column_count(&self) -> usize {
        self.balanced_column_count(self.available_width, self.available_height)
    }

    fn balanced_column_count(&self, available_width: f64, available_height: f64) -> usize {
        if self.entries.len() <= MAX_SINGLE_COLUMN_ENTRIES {
            return 1;
        }
        let minimum_columns = self.entries.len().div_ceil(MAX_SINGLE_COLUMN_ENTRIES);
        (minimum_columns..=self.entries.len())
            .filter_map(|columns| {
                let (width, height) = self.box_size_for_columns(columns);
                (width <= available_width && height <= available_height).then_some((
                    (width / available_width).max(height / available_height),
                    columns,
                ))
            })
            .min_by(|left, right| {
                left.0
                    .total_cmp(&right.0)
                    .then_with(|| left.1.cmp(&right.1))
            })
            .map_or(1, |(_, columns)| columns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(label: impl Into<String>) -> LegendEntry {
        let mut entry = LegendEntry::point(
            LegendIdentity::literal("group", "entry"),
            PointStyle::new((0, 0, 0), 3),
        );
        entry.title = "group".into();
        entry.label = label.into();
        entry
    }

    fn legend() -> Legend {
        let theme = crate::Theme::Publication.resolve();
        Legend::new(
            "group",
            vec![entry("first"), entry("second")],
            640.0,
            480.0,
            &theme,
        )
        .unwrap()
    }

    #[test]
    fn glyph_edge_padding_is_symmetric_and_retains_bottom_comfort() {
        let legend = legend();
        let bounds = legend.box_at(20.0, 30.0);
        let (_, title_center) = bounds.anchor();
        let title_top = title_center + legend.title_half_height();
        let columns = legend.column_count();
        let title_block = &legend.title_block;
        let entry_blocks = &legend.entry_blocks;
        let last_center = title_center
            + legend
                .row_centers(columns, title_block, entry_blocks)
                .last()
                .copied()
                .unwrap();
        let last_bottom = last_center - legend.last_row_half_height(columns, entry_blocks);
        let top_padding = bounds.top - title_top;
        let bottom_padding = last_bottom - bounds.bottom;
        let previous_bottom_comfort =
            legend.metrics.pad + GLYPH_EDGE_PAD_FACTOR * legend.entry_font_size();

        assert!((top_padding - bottom_padding).abs() <= 1e-9);
        assert!(top_padding >= previous_bottom_comfort);
    }

    #[test]
    fn eight_short_entries_use_multiple_balanced_columns() {
        let entries = (1..=8)
            .map(|index| entry(format!("item {index}")))
            .collect();
        let theme = crate::Theme::Publication.resolve();
        let legend = Legend::new("group", entries, 640.0, 480.0, &theme).unwrap();

        assert_eq!(legend.column_count(), 2);
    }

    #[test]
    fn overwide_legend_text_returns_the_planning_error() {
        let theme = crate::Theme::Publication.resolve();
        let error = Legend::new(
            &"W".repeat(10_000),
            vec![entry("entry")],
            160.0,
            120.0,
            &theme,
        )
        .err()
        .expect("overwide legend title must fail");

        assert!(error.to_string().contains("legend title text planning"));
    }

    #[test]
    fn right_box_does_not_add_border_padding_twice() {
        let legend = legend();
        let bounds = legend.right_box((10.0, 410.0, 20.0, 300.0));

        assert!((bounds.right - bounds.left - legend.compute_box_width()).abs() <= 1.0e-9);
    }
}
