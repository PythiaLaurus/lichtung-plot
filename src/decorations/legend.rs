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
    Backend, FillStyle, LineStyle, PlannedTextPlacement, Point2D, PointStyle, TextAnchor,
};
use crate::Error;

#[derive(Debug)]
enum LegendCall {
    Background([f64; 5], FillStyle),
    Fill([Point2D; 4], FillStyle),
    Line([Point2D; 2], LineStyle),
    Point([Point2D; 1], PointStyle),
    Text(crate::render::backend::PlacedText),
}

impl LegendCall {
    fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        match self {
            Self::Background([x, y, width, height, radius], style) => {
                backend.draw_rounded_rect(*x, *y, *width, *height, *radius, style)
            }
            Self::Fill(points, style) => backend.draw_polygon(points, style),
            Self::Line(points, style) => backend.draw_line(points, style),
            Self::Point(points, style) => backend.draw_scatter(points, style),
            Self::Text(text) => text.draw(backend),
        }
    }
}

/// Assigned legend operands, including glyph and text placement.
#[derive(Debug)]
pub(crate) struct LegendDrawPlan {
    calls: Vec<LegendCall>,
}

impl LegendDrawPlan {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        for call in &self.calls {
            call.draw(backend)?;
        }
        Ok(())
    }

    pub(crate) fn geometry(
        &self,
        geometry: &mut crate::render::geometry::GeometrySnapshot,
    ) -> Result<(), Error> {
        for call in &self.calls {
            match call {
                LegendCall::Background(rect, style) => geometry.rounded_rect(*rect, style, None)?,
                LegendCall::Fill(points, style) => geometry.polygon(points, style, None)?,
                LegendCall::Line(points, style) => geometry.line(points, style, None)?,
                LegendCall::Point(points, style) => {
                    geometry.scatter_with_clip(points, style, None)?;
                }
                LegendCall::Text(text) => geometry.text(text.placement, &text.plan, None)?,
            }
        }
        Ok(())
    }
}

pub(crate) fn address_role(position: crate::LegendPosition) -> crate::layout::AddressRole {
    use crate::layout::AddressRole;
    use crate::LegendPosition::{
        Auto, OutsideBottom, OutsideBottomLeft, OutsideBottomRight, OutsideLeft, OutsideLeftBottom,
        OutsideLeftTop, OutsideRight, OutsideRightBottom, OutsideRightTop, OutsideTop,
        OutsideTopLeft, OutsideTopRight,
    };
    match position {
        Auto => AddressRole::OutsideGuideAuto,
        OutsideTopLeft | OutsideTop | OutsideTopRight => AddressRole::OutsideGuideTop,
        OutsideRightTop | OutsideRight | OutsideRightBottom => AddressRole::OutsideGuideRight,
        OutsideBottomLeft | OutsideBottom | OutsideBottomRight => AddressRole::OutsideGuideBottom,
        OutsideLeftTop | OutsideLeft | OutsideLeftBottom => AddressRole::OutsideGuideLeft,
        _ => AddressRole::InsideGuide,
    }
}

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

    fn same_title(&self, other: &Self, theme: &crate::theme::ResolvedTheme) -> Result<bool, Error> {
        match (self, other) {
            (Self::Single { field: left, .. }, Self::Single { field: right, .. })
            | (Self::Literal { title: left, .. }, Self::Literal { title: right, .. }) => {
                legend_text_eq(left, right, theme)
            }
            (
                Self::Pair {
                    group_field: left_group,
                    hue_field: left_hue,
                    ..
                },
                Self::Pair {
                    group_field: right_group,
                    hue_field: right_hue,
                    ..
                },
            ) => Ok(legend_text_eq(left_group, right_group, theme)?
                && legend_text_eq(left_hue, right_hue, theme)?),
            _ => Ok(false),
        }
    }

    fn same_identity(
        &self,
        other: &Self,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<bool, Error> {
        if !self.same_title(other, theme)? {
            return Ok(false);
        }
        match (self, other) {
            (Self::Single { value: left, .. }, Self::Single { value: right, .. })
            | (Self::Literal { value: left, .. }, Self::Literal { value: right, .. }) => {
                legend_text_eq(left, right, theme)
            }
            (
                Self::Pair {
                    group_value: left_group,
                    hue_value: left_hue,
                    ..
                },
                Self::Pair {
                    group_value: right_group,
                    hue_value: right_hue,
                    ..
                },
            ) => Ok(legend_text_eq(left_group, right_group, theme)?
                && legend_text_eq(left_hue, right_hue, theme)?),
            _ => Ok(false),
        }
    }

    pub(crate) fn resolved_text(
        &self,
        plot: &crate::Plot,
        data: crate::data::PlotData<'_>,
    ) -> Result<(String, String), Error> {
        let join = |mut first: String, separator: &str, second: String| -> Result<String, Error> {
            let additional = separator
                .len()
                .checked_add(second.len())
                .ok_or_else(|| Error::Render("legend label length overflow".into()))?;
            data.reserve_source_string(&mut first, additional)?;
            first.push_str(separator);
            first.push_str(&second);
            Ok(first)
        };
        match self {
            Self::Single { field, value } => Ok((
                plot.resolve_source_label(field, data)?,
                plot.resolve_source_label(value, data)?,
            )),
            Self::Pair {
                group_field,
                group_value,
                hue_field,
                hue_value,
            } => Ok((
                join(
                    plot.resolve_source_label(group_field, data)?,
                    " × ",
                    plot.resolve_source_label(hue_field, data)?,
                )?,
                join(
                    plot.resolve_source_label(group_value, data)?,
                    ", ",
                    plot.resolve_source_label(hue_value, data)?,
                )?,
            )),
            Self::Literal { title, value } => Ok((
                data.copy_source_string(title)?,
                data.copy_source_string(value)?,
            )),
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
    pub(crate) fn admitted_clone(&self, data: crate::data::PlotData<'_>) -> Result<Self, Error> {
        let identity = match &self.identity {
            LegendIdentity::Single { field, value } => [Some(field), Some(value), None, None],
            LegendIdentity::Literal { title, value } => [Some(title), Some(value), None, None],
            LegendIdentity::Pair {
                group_field,
                group_value,
                hue_field,
                hue_value,
            } => [
                Some(group_field),
                Some(group_value),
                Some(hue_field),
                Some(hue_value),
            ],
        };
        for text in identity
            .into_iter()
            .flatten()
            .chain([&self.title, &self.label])
        {
            data.admit_clone_slice(text.as_bytes())?;
        }
        Ok(self.clone())
    }

    pub(crate) fn same_child_input(&self, other: &Self) -> bool {
        self.exact_eq(other) && self.contributes_hue_level == other.contributes_hue_level
    }

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
    theme: &crate::theme::ResolvedTheme,
) -> Result<(), Error> {
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = plot.data().with_axis_resources(&resources);
    data.claim_source_visits(entries.len())?;
    for entry in entries.iter_mut() {
        (entry.title, entry.label) = entry.identity.resolved_text(plot, data)?;
    }
    *entries = merge_exact_entries(std::mem::take(entries), "plot legend", theme)?;
    Ok(())
}

pub(crate) fn merge_exact_entries(
    entries: impl IntoIterator<Item = LegendEntry>,
    context: &str,
    theme: &crate::theme::ResolvedTheme,
) -> Result<Vec<LegendEntry>, Error> {
    let mut merged: Vec<LegendEntry> = Vec::new();
    for entry in entries {
        legend_visits(theme, 1)?;
        if let Some(first) = merged.first() {
            if !first.identity.same_title(&entry.identity, theme)?
                || !legend_text_eq(&first.title, &entry.title, theme)?
            {
                return Err(Error::Render(format!(
                    "{context} requires compatible semantic titles, labels, and styles; glyphs must match exactly"
                )));
            }
        }
        let mut duplicate = None;
        for existing in &merged {
            legend_visits(theme, 1)?;
            if !existing.identity.same_identity(&entry.identity, theme)? {
                if legend_text_eq(&existing.label, &entry.label, theme)? {
                    return Err(Error::Render(format!(
                        "{context} has distinct raw identities resolving to display label {:?}",
                        entry.label
                    )));
                }
            } else if duplicate.is_none() {
                duplicate = Some(existing);
            }
        }
        if let Some(existing) = duplicate {
            if !legend_text_eq(&existing.title, &entry.title, theme)?
                || !legend_text_eq(&existing.label, &entry.label, theme)?
                || !existing.glyph.exact_eq(&entry.glyph)
            {
                return Err(Error::Render(format!(
                    "{context} requires compatible semantic titles, labels, and styles; glyphs must match exactly"
                )));
            }
        } else {
            crate::layout::fit::reserve(theme, &mut merged, 1, "merged legend entries")?;
            merged.push(entry);
        }
    }
    Ok(merged)
}

pub(crate) fn resolved_title(
    entries: &[LegendEntry],
    theme: &crate::theme::ResolvedTheme,
) -> Result<String, Error> {
    let Some(first) = entries.first() else {
        return Ok(String::new());
    };
    legend_visits(theme, entries.len())?;
    for entry in entries {
        if !entry.identity.same_title(&first.identity, theme)?
            || !legend_text_eq(&entry.title, &first.title, theme)?
        {
            return Err(Error::Render(
                "legend entries require one compatible semantic title".into(),
            ));
        }
    }
    copy_legend_string(&first.title, theme)
}

fn legend_visits(theme: &crate::theme::ResolvedTheme, count: usize) -> Result<(), Error> {
    theme
        .prepared_font_environment()?
        .claim_processing_visits(count as u128)
        .map_err(|error| Error::Render(format!("legend work admission: {error:?}")))
}

fn legend_text_eq(
    left: &str,
    right: &str,
    theme: &crate::theme::ResolvedTheme,
) -> Result<bool, Error> {
    legend_visits(theme, 1)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    if std::ptr::eq(left, right) {
        return Ok(true);
    }
    for index in 0..left.len() {
        legend_visits(theme, 1)?;
        if left.as_bytes()[index] != right.as_bytes()[index] {
            return Ok(false);
        }
    }
    Ok(true)
}

fn copy_legend_string(source: &str, theme: &crate::theme::ResolvedTheme) -> Result<String, Error> {
    legend_visits(theme, source.len())?;
    theme
        .prepared_font_environment()?
        .claim_unicode_bytes(
            u64::try_from(source.len())
                .map_err(|_| Error::Render("legend string length overflow".into()))?,
        )
        .map_err(|error| Error::Render(format!("legend string admission: {error}")))?;
    let mut value = String::new();
    value
        .try_reserve_exact(source.len())
        .map_err(|error| Error::Render(format!("legend string reservation: {error}")))?;
    value.push_str(source);
    Ok(value)
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
#[derive(Debug)]
pub(crate) struct Legend {
    /// The entries in the legend.
    entries: std::sync::Arc<[LegendEntry]>,
    /// Layout metrics (derived from canvas size or overridden).
    pub(crate) metrics: LegendMetrics,
    available_width: f64,
    available_height: f64,
    title_block: TextBlock,
    entry_blocks: Vec<TextBlock>,
    font_family: String,
    layout: LegendLayout,
}

#[derive(Debug)]
struct LegendLayout {
    width: f64,
    height: f64,
    columns: usize,
    title: PlannedTextPlacement,
    entries: Vec<LegendSlot>,
}

#[derive(Debug)]
struct LegendSlot {
    key: Point2D,
    label: PlannedTextPlacement,
    width: f64,
    height: f64,
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
    #[cfg(test)]
    pub(crate) fn text_plans(&self) -> impl Iterator<Item = &crate::layout::ResolvedTextPlan> {
        std::iter::once(self.title_block.plan())
            .chain(self.entry_blocks.iter().map(TextBlock::plan))
    }
    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "one assigned edge/flow measurement retains all entry geometry"
    )]
    pub(crate) fn measure_outside(
        title: &str,
        entries: std::sync::Arc<[LegendEntry]>,
        root: (u32, u32),
        available: (f64, f64),
        horizontal: bool,
        compact: bool,
        theme: &crate::theme::ResolvedTheme,
        compact_changes: &mut bool,
        scope: &crate::layout::fit::FitScope,
    ) -> Result<crate::layout::fit::FitOutcome<Self>, Error> {
        use crate::layout::fit::{self, FitOutcome};
        use crate::layout::{kernel::FitKind, AddressRole};
        theme.prepare_font_environment()?;
        if !compact {
            *compact_changes = false;
        }
        if !available.0.is_finite() || !available.1.is_finite() {
            return Err(Error::Render("nonfinite guide assignment".into()));
        }
        let metrics = LegendMetrics::from_canvas(f64::from(root.0), f64::from(root.1));
        let font_size = metrics.font_size.round() as u32;
        let edge_pad = metrics.pad + GLYPH_EDGE_PAD_FACTOR * f64::from(font_size);
        let inner_width = available.0 - 2.0 * metrics.pad;
        let inner_height = available.1 - 2.0 * edge_pad;
        if inner_width <= 0.0 || inner_height <= 0.0 {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "outside guide has no positive content extent",
            )));
        }
        let mut title_appearance = theme.legacy_text_appearance(
            (metrics.font_size * 1.1).round() as u32,
            crate::TextWeight::Normal,
        );
        title_appearance.paint = (60, 60, 60).into();
        let mut appearance = theme.legacy_text_appearance(font_size, crate::TextWeight::Normal);
        appearance.paint = (40, 40, 40).into();
        let title_plan = match fit::text(
            theme,
            title,
            &title_appearance,
            inner_width,
            PhysicalTextAlignment::Start,
            "outside legend title",
            &scope.child(AddressRole::LegendTitle, 0)?,
        )? {
            FitOutcome::Measured(plan) => plan,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let title_gap = (metrics.title_h - f64::from(title_appearance.font_size)).max(0.0);
        let body_height = inner_height - title_plan.height() - title_gap;
        if body_height <= 0.0 {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "outside legend title leaves no entry height",
            )));
        }
        let gap = COLUMN_SPACING_FACTOR * metrics.font_size;
        let row_gap = (metrics.entry_h - f64::from(font_size) * 1.2).max(0.0);
        let zero = theme.measure_legacy_appearance_text(
            "0",
            &appearance,
            PhysicalTextAlignment::Start,
            "legend metric probe",
            false,
        )?;
        let ch = zero
            .lines
            .first()
            .map_or(0.0, |line| line.logical_advance_quanta as f64 / 1024.0);
        if ch <= 0.0 || !ch.is_finite() {
            return Err(Error::Render("invalid legend ch metric".into()));
        }
        let mut widths = Vec::new();
        fit::reserve(
            theme,
            &mut widths,
            entries.len(),
            "guide width requirements",
        )?;
        for entry in entries.iter() {
            let (hard, natural) =
                fit::intrinsic_widths(theme, &entry.label, &appearance, "legend entry intrinsic")?;
            let key = metrics.handle_w.max(2.0 * entry.glyph_extent());
            let chrome = crate::layout::kernel::required_length(key + metrics.handle_text)?;
            widths.push((
                hard + chrome,
                hard.max(natural.min(66.0 * ch)) + chrome,
                key,
                chrome,
            ));
        }
        let preferred_column = widths
            .iter()
            .map(|(_, preferred, _, _)| *preferred)
            .fold(0.0_f64, f64::max)
            .min(inner_width);
        if !compact {
            if let Some(cap) = crate::layout::kernel::compact_half_width(inner_width, gap)? {
                *compact_changes = widths.iter().any(|(_, preferred, _, _)| {
                    if horizontal {
                        preferred.min(inner_width) > cap
                    } else {
                        preferred_column > cap
                    }
                });
            }
        }
        let cap = if compact {
            let Some(width) = crate::layout::kernel::compact_half_width(inner_width, gap)? else {
                return Ok(FitOutcome::Miss(scope.failure(
                    FitKind::LegendOrGuide,
                    "compact guide has no column width",
                )));
            };
            width
        } else {
            inner_width
        };
        let mut entry_blocks = Vec::new();
        let mut slots = Vec::new();
        fit::reserve(
            theme,
            &mut entry_blocks,
            entries.len(),
            "measured guide labels",
        )?;
        fit::reserve(theme, &mut slots, entries.len(), "measured guide slots")?;
        let title_half = GLYPH_HALF_HEIGHT_FACTOR * f64::from(title_appearance.font_size);
        let body_top = title_half - title_plan.height() - title_gap;
        let mut x = 0.0;
        let mut y = 0.0;
        let mut row_height = 0.0_f64;
        let mut used_width = 0.0_f64;
        let mut used_height = 0.0_f64;
        let mut columns = 1;
        let mut row_entries = 0;
        let mut miss = None;
        for (index, (entry, (minimum, preferred, key, chrome))) in
            entries.iter().zip(widths).enumerate()
        {
            let allocated = if horizontal {
                preferred.min(cap)
            } else {
                preferred_column.min(cap)
            };
            if allocated < minimum {
                crate::layout::fit::FitFailure::retain_least(
                    &mut miss,
                    scope.child(AddressRole::LegendEntry, index)?.failure(
                        FitKind::Text,
                        "guide entry allocation is below its hard unbroken requirement",
                    ),
                );
                continue;
            }
            let plan = match fit::text(
                theme,
                &entry.label,
                &appearance,
                allocated - chrome,
                PhysicalTextAlignment::Start,
                "outside legend entry",
                &scope.child(AddressRole::LegendEntry, index)?,
            )? {
                FitOutcome::Measured(plan) => plan,
                FitOutcome::Miss(reason) => {
                    crate::layout::fit::FitFailure::retain_least(&mut miss, reason);
                    continue;
                }
            };
            let baseline = plan.first_baseline_quanta.unwrap_or(0) as f64 / 1024.0;
            let key_half = entry.glyph_extent().max(f64::from(font_size) * 0.32);
            let ascent = baseline.max(key_half);
            let height = ascent + (plan.height() - baseline).max(key_half);
            if horizontal {
                if x > 0.0 && x + allocated > inner_width {
                    x = 0.0;
                    y += row_height + row_gap;
                    row_height = 0.0;
                    row_entries = 0;
                }
                row_height = row_height.max(height);
                row_entries += 1;
                columns = columns.max(row_entries);
            } else if y > 0.0 && y + height > body_height {
                y = 0.0;
                x += allocated + gap;
                columns += 1;
            }
            if x + allocated > inner_width || y + height > body_height {
                crate::layout::fit::FitFailure::retain_least(
                    &mut miss,
                    scope.failure(
                        FitKind::LegendOrGuide,
                        "outside legend flow exceeds its assigned track",
                    ),
                );
            }
            let baseline_y = body_top - y - ascent;
            slots.push(LegendSlot {
                key: Point2D::new(x + (key - metrics.handle_w) / 2.0, baseline_y),
                label: PlannedTextPlacement::baseline(
                    Point2D::new(x + chrome, baseline_y),
                    TextAnchor::Start,
                    0.0,
                ),
                width: allocated,
                height,
            });
            used_width = used_width.max(x + allocated);
            used_height = used_height.max(y + height);
            entry_blocks.push(TextBlock::from_resolved(plan));
            if horizontal {
                x += allocated + gap;
            } else {
                y += height + row_gap;
            }
        }
        if let Some(failure) = miss {
            return Ok(FitOutcome::Miss(failure));
        }
        let title_baseline = title_plan.first_baseline_quanta.unwrap_or(0) as f64 / 1024.0;
        let layout = LegendLayout {
            width: used_width.max(title_plan.width()) + 2.0 * metrics.pad,
            height: title_plan.height() + title_gap + used_height + 2.0 * edge_pad,
            columns,
            title: PlannedTextPlacement::baseline(
                Point2D::new(0.0, title_half - title_baseline),
                TextAnchor::Start,
                0.0,
            ),
            entries: slots,
        };
        Ok(FitOutcome::Measured(Self {
            entries,
            metrics,
            available_width: available.0,
            available_height: available.1,
            title_block: TextBlock::from_resolved(title_plan),
            entry_blocks,
            font_family: theme.font_family.clone(),
            layout,
        }))
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        round.observe_number(self.compute_box_width())?;
        round.observe_number(self.compute_box_height())?;
        round.observe_count(self.column_count())?;
        for metric in [
            self.available_width,
            self.available_height,
            self.metrics.font_size,
            self.metrics.pad,
            self.metrics.entry_h,
            self.metrics.title_h,
            self.metrics.handle_w,
            self.metrics.handle_text,
        ] {
            round.observe_number(metric)?;
        }
        round.observe_count(self.entry_blocks.len())?;
        round.observe_string(&self.font_family)?;
        for entry in self.entries.iter() {
            round.observe_legend_glyph(
                entry.glyph.point.as_ref(),
                entry.glyph.line.as_ref(),
                entry.glyph.fill.as_ref(),
            )?;
        }
        round.observe_text_block(&self.title_block)?;
        for entry in &self.entry_blocks {
            round.observe_text_block(entry)?;
        }
        round.observe_placement(self.layout.title)?;
        for slot in &self.layout.entries {
            round.observe_number(slot.key.x)?;
            round.observe_number(slot.key.y)?;
            round.observe_placement(slot.label)?;
            round.observe_number(slot.width)?;
            round.observe_number(slot.height)?;
        }
        Ok(())
    }
    /// Create a new legend with metrics derived from canvas size.
    #[expect(
        clippy::too_many_lines,
        reason = "admission and unchanged legacy packing share one constructor"
    )]
    pub fn new(
        title: &str,
        entries: Vec<LegendEntry>,
        canvas_w: f64,
        canvas_h: f64,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<Self, Error> {
        let mut entry_blocks = Vec::new();
        crate::layout::fit::reserve(
            theme,
            &mut entry_blocks,
            entries.len(),
            "legend text blocks",
        )?;
        let allocation = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(
                std::alloc::Layout::array::<LegendEntry>(entries.len())
                    .map_err(|_| Error::Render("legend entry storage overflow".into()))?,
            )
            .map_err(|_| Error::Render("legend entry Arc layout overflow".into()))?
            .0
            .pad_to_align();
        legend_visits(theme, entries.len())?;
        theme
            .prepared_font_environment()?
            .claim_unicode_bytes(
                u64::try_from(allocation.size())
                    .map_err(|_| Error::Render("legend entry storage overflow".into()))?,
            )
            .map_err(|error| Error::Render(format!("legend entry Arc admission: {error}")))?;
        // Stable Arc has no fallible slice constructor; admit its exact layout first.
        let entries: std::sync::Arc<[LegendEntry]> = entries.into();
        let metrics = LegendMetrics::from_canvas(canvas_w, canvas_h);
        let title_maximum_width = (canvas_w * 0.5).max(metrics.font_size);
        let entry_maximum_width = (canvas_w * 0.35).max(metrics.font_size);
        let title_font_size = (metrics.font_size * 1.1).round() as u32;
        let entry_font_size = metrics.font_size.round() as u32;
        let title_appearance = crate::theme::TextAppearance {
            stack: copy_legend_string(&theme.font_family, theme)?,
            paint: (60, 60, 60).into(),
            font_size: title_font_size,
            weight: crate::TextWeight::Normal,
            line_height_multiplier: 1.2,
        };
        let entry_appearance = crate::theme::TextAppearance {
            stack: copy_legend_string(&theme.font_family, theme)?,
            paint: (40, 40, 40).into(),
            font_size: entry_font_size,
            weight: crate::TextWeight::Normal,
            line_height_multiplier: 1.2,
        };
        let title_block = TextBlock::from_resolved(theme.plan_legacy_appearance_text(
            title,
            &title_appearance,
            title_maximum_width,
            PhysicalTextAlignment::Start,
            "legend title",
            false,
        )?);
        legend_visits(theme, entries.len())?;
        for entry in entries.iter() {
            entry_blocks.push(
                theme
                    .plan_legacy_appearance_text(
                        &entry.label,
                        &entry_appearance,
                        entry_maximum_width,
                        PhysicalTextAlignment::Start,
                        "legend entry",
                        false,
                    )
                    .map(TextBlock::from_resolved)?,
            );
        }
        let mut legend = Self {
            entries,
            metrics,
            available_width: canvas_w * 0.8,
            available_height: canvas_h * 0.8,
            title_block,
            entry_blocks,
            font_family: copy_legend_string(&theme.font_family, theme)?,
            layout: LegendLayout {
                width: 0.0,
                height: 0.0,
                columns: 1,
                title: PlannedTextPlacement::middle(Point2D::new(0.0, 0.0), TextAnchor::Start, 0.0),
                entries: Vec::new(),
            },
        };
        let columns =
            legend.balanced_column_count(legend.available_width, legend.available_height, theme)?;
        let (width, height) = legend.box_size_for_columns(columns, theme)?;
        let widths = legend.column_widths(columns, &legend.entry_blocks, theme)?;
        let centers =
            legend.row_centers(columns, &legend.title_block, &legend.entry_blocks, theme)?;
        let mut slots = Vec::new();
        crate::layout::fit::reserve(theme, &mut slots, legend.entries.len(), "legend slots")?;
        for (index, block) in legend.entry_blocks.iter().enumerate() {
            let column = index % columns;
            let row = index / columns;
            legend_visits(theme, column + 1)?;
            let x = widths[..column].iter().sum::<f64>() + column as f64 * legend.column_spacing();
            slots.push(LegendSlot {
                key: Point2D::new(x, centers[row]),
                label: PlannedTextPlacement::middle(
                    Point2D::new(x + metrics.handle_w + metrics.handle_text, centers[row]),
                    TextAnchor::Start,
                    0.0,
                ),
                width: widths[column],
                height: block
                    .height()
                    .max(2.0 * legend.entries[index].glyph_extent()),
            });
        }
        legend.layout = LegendLayout {
            width,
            height,
            columns,
            title: PlannedTextPlacement::middle(
                Point2D::new(
                    0.0,
                    -(legend.title_block.lines().len().saturating_sub(1) as f64)
                        * legend.title_block.line_height()
                        / 2.0,
                ),
                TextAnchor::Start,
                0.0,
            ),
            entries: slots,
        };
        Ok(legend)
    }

    /// Compute the total bounding box width based on entry labels.
    pub fn compute_box_width(&self) -> f64 {
        self.layout.width
    }

    fn box_size_for_columns(
        &self,
        columns: usize,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<(f64, f64), Error> {
        let m = &self.metrics;
        let columns = columns.clamp(1, self.entries.len().max(1));
        let title_block = &self.title_block;
        let entry_blocks = &self.entry_blocks;
        let column_widths = self.column_widths(columns, entry_blocks, theme)?;
        legend_visits(theme, column_widths.len())?;
        let entries_width = column_widths.iter().sum::<f64>()
            + (columns.saturating_sub(1) as f64) * self.column_spacing();
        let title_width = title_block.width();
        let width = entries_width.max(title_width) + m.pad * 2.0;
        let content_height = if self.entries.is_empty() {
            title_block.height()
        } else {
            let centers = self.row_centers(columns, title_block, entry_blocks, theme)?;
            legend_visits(theme, self.entries.len())?;
            legend_visits(theme, entry_blocks.len())?;
            self.title_half_height()
                + -centers.last().copied().unwrap_or(0.0)
                + self.last_row_half_height(columns, entry_blocks)
        };
        Ok((width, content_height + self.glyph_edge_pad() * 2.0))
    }

    /// Compute the total bounding box height.
    pub fn compute_box_height(&self) -> f64 {
        self.layout.height
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
        self.visit_calls(x, y, |call| call.draw(backend))
    }

    pub(crate) fn select(
        &self,
        x: f64,
        y: f64,
        data: crate::data::PlotData<'_>,
    ) -> Result<LegendDrawPlan, Error> {
        data.claim_source_visits(self.entries.len())?;
        let count = self.entries.iter().try_fold(2_usize, |count, entry| {
            count
                .checked_add(
                    1 + usize::from(entry.glyph.fill.is_some())
                        + usize::from(entry.glyph.line.is_some())
                        + usize::from(entry.glyph.point.is_some()),
                )
                .ok_or_else(|| Error::Render("legend call count overflow".into()))
        })?;
        let mut calls = Vec::new();
        data.reserve_source(&mut calls, count)?;
        self.visit_calls(x, y, |call| {
            calls.push(call);
            Ok(())
        })?;
        Ok(LegendDrawPlan { calls })
    }

    fn visit_calls(
        &self,
        x: f64,
        y: f64,
        mut emit: impl FnMut(LegendCall) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let m = &self.metrics;
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
        emit(LegendCall::Background(
            [
                rx + 0.5,
                ry_bot + 0.5,
                box_w - 1.0,
                box_h - 1.0,
                CORNER_RADIUS,
            ],
            bg_fill,
        ))?;

        // ─── Title ───
        let translate = |placement: PlannedTextPlacement| PlannedTextPlacement {
            position: Point2D::new(
                (x + placement.position.x).round(),
                (y + placement.position.y).round(),
            ),
            ..placement
        };
        emit(LegendCall::Text(crate::render::backend::PlacedText {
            placement: translate(self.layout.title),
            plan: title_block.plan().clone(),
        }))?;
        // ─── Entries (top to bottom = decreasing y) ───
        for (index, entry) in self.entries.iter().enumerate() {
            let slot = &self.layout.entries[index];
            let column_x = x + slot.key.x;
            let row_y = y + slot.key.y;
            // Renderer-owned fill, line, and point components share one handle.
            let marker_x = column_x + m.handle_w * 0.5;
            if let Some(fill_style) = &entry.glyph.fill {
                let half_height = self.entry_font_size() * 0.32;
                emit(LegendCall::Fill(
                    [
                        Point2D::new(column_x, row_y - half_height),
                        Point2D::new(column_x + m.handle_w, row_y - half_height),
                        Point2D::new(column_x + m.handle_w, row_y + half_height),
                        Point2D::new(column_x, row_y + half_height),
                    ],
                    fill_style.clone(),
                ))?;
            }
            if let Some(line_style) = &entry.glyph.line {
                emit(LegendCall::Line(
                    [
                        Point2D::new(column_x, row_y),
                        Point2D::new(column_x + m.handle_w, row_y),
                    ],
                    line_style.clone(),
                ))?;
            }
            if let Some(point_style) = &entry.glyph.point {
                emit(LegendCall::Point(
                    [Point2D::new(marker_x, row_y)],
                    point_style.clone(),
                ))?;
            }
            // Label text
            let block = &entry_blocks[index];
            emit(LegendCall::Text(crate::render::backend::PlacedText {
                placement: translate(slot.label),
                plan: block.plan().clone(),
            }))?;
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

    fn column_widths(
        &self,
        columns: usize,
        entry_blocks: &[TextBlock],
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<Vec<f64>, Error> {
        let mut widths = Vec::new();
        crate::layout::fit::reserve(theme, &mut widths, columns, "legend column widths")?;
        legend_visits(theme, columns)?;
        widths.resize(columns, self.metrics.handle_w + self.metrics.handle_text);
        legend_visits(theme, entry_blocks.len())?;
        for (index, block) in entry_blocks.iter().enumerate() {
            widths[index % columns] = widths[index % columns]
                .max(self.metrics.handle_w + self.metrics.handle_text + block.width());
        }
        Ok(widths)
    }

    fn row_centers(
        &self,
        columns: usize,
        title_block: &TextBlock,
        entry_blocks: &[TextBlock],
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<Vec<f64>, Error> {
        let rows = self.entries.len().div_ceil(columns);
        if rows == 0 {
            return Ok(Vec::new());
        }
        let title_extra =
            title_block.lines().len().saturating_sub(1) as f64 * title_block.line_height();
        let mut centers = Vec::new();
        crate::layout::fit::reserve(theme, &mut centers, rows, "legend row centers")?;
        legend_visits(theme, rows)?;
        centers.push(-self.metrics.title_h - title_extra);
        let row_gap = self.metrics.entry_h - self.entry_font_size() * 1.2;
        for row in 1..rows {
            legend_visits(theme, entry_blocks.len())?;
            legend_visits(theme, entry_blocks.len())?;
            let previous_half = self.row_half_height(row - 1, columns, entry_blocks);
            let current_half = self.row_half_height(row, columns, entry_blocks);
            centers.push(centers[row - 1] - previous_half - row_gap - current_half);
        }
        Ok(centers)
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
        self.layout.columns
    }

    fn balanced_column_count(
        &self,
        available_width: f64,
        available_height: f64,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<usize, Error> {
        if self.entries.len() <= MAX_SINGLE_COLUMN_ENTRIES {
            return Ok(1);
        }
        let minimum_columns = self.entries.len().div_ceil(MAX_SINGLE_COLUMN_ENTRIES);
        let mut best: Option<(f64, usize)> = None;
        for columns in minimum_columns..=self.entries.len() {
            legend_visits(theme, 1)?;
            let (width, height) = self.box_size_for_columns(columns, theme)?;
            if width <= available_width && height <= available_height {
                let score = (width / available_width).max(height / available_height);
                if best.is_none_or(|(previous, _)| score.total_cmp(&previous).is_lt()) {
                    best = Some((score, columns));
                }
            }
        }
        Ok(best.map_or(1, |(_, columns)| columns))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_admits_long_labels_arc_and_packing_before_work() {
        let run = |limits: Option<(u128, u64)>| {
            let theme = crate::Theme::Publication.resolve();
            let resources = theme
                .prepared_font_environment()
                .unwrap()
                .generation_resources();
            let entries = (0..16)
                .map(|index| {
                    entry(format!(
                        "Long measurement condition number {index} with repeated observations"
                    ))
                })
                .collect();
            if let Some((visits, bytes)) = limits {
                resources.set_test_limits(visits, bytes);
            }
            let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
            let legend = Legend::new(
                "Long measurement conditions",
                entries,
                1600.0,
                1000.0,
                &theme,
            );
            (
                resources,
                legend,
                crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get) - shapes,
            )
        };
        let (resources, legend, _) = run(None);
        let legend = legend.unwrap();
        assert_eq!(legend.entry_blocks.len(), 16);
        assert_eq!(legend.layout.columns, 1);
        let visits = resources.usage().unwrap().1;
        let bytes = resources.byte_usage().unwrap();
        for limits in [(visits - 1, bytes), (visits, bytes - 1)] {
            assert!(run(Some(limits)).1.is_err());
        }
        for limits in [(0, bytes), (visits, 0)] {
            let (_, result, shapes) = run(Some(limits));
            assert!(result.is_err());
            assert_eq!(shapes, 0, "constructor refusal must precede text planning");
        }
        let (_, exact, _) = run(Some((visits, bytes)));
        let exact = exact.unwrap();
        assert_eq!(exact.layout.columns, legend.layout.columns);
        assert_eq!(exact.layout.width.to_bits(), legend.layout.width.to_bits());
        assert_eq!(
            exact.layout.height.to_bits(),
            legend.layout.height.to_bits()
        );
    }

    #[test]
    fn resolved_pair_strings_merge_and_title_obey_exact_limits() {
        let plot = crate::Plot::matrix(vec![crate::NumericValues::new("x", [1.0, 2.0])]).erase();
        let run = |limits: Option<(u128, u64)>| {
            let theme = crate::Theme::Publication.resolve();
            let resources = theme
                .prepared_font_environment()
                .unwrap()
                .generation_resources();
            let mut entries = (0..12)
                .map(|index| {
                    LegendEntry::point(
                        LegendIdentity::pair(
                            "long_group_field",
                            format!("measured_group_{index}"),
                            "long_hue_field",
                            "first_condition",
                        ),
                        PointStyle::new((0, 0, 0), 3),
                    )
                })
                .collect::<Vec<_>>();
            entries.push(entries[0].clone());
            if let Some((visits, bytes)) = limits {
                resources.set_test_limits(visits, bytes);
            }
            let result = resolve_entries(&plot, &mut entries, &theme)
                .and_then(|()| resolved_title(&entries, &theme));
            (resources, entries, result)
        };
        let (resources, entries, title) = run(None);
        assert_eq!(entries.len(), 12);
        assert_eq!(title.unwrap(), "Long group field × Long hue field");
        assert_eq!(entries[0].label, "Measured group 0, First condition");
        let visits = resources.usage().unwrap().1;
        let bytes = resources.byte_usage().unwrap();
        for limits in [
            (visits - 1, bytes),
            (visits, bytes - 1),
            (0, bytes),
            (visits, 0),
        ] {
            assert!(run(Some(limits)).2.is_err());
        }
        run(Some((visits, bytes))).2.unwrap();
    }

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
    fn compact_vertical_flow_remeasures_labels_and_fits_two_columns() {
        use crate::layout::fit::FitOutcome;
        let theme = crate::Theme::Publication.resolve();
        let entries: std::sync::Arc<[LegendEntry]> =
            vec![entry("longer longer"), entry("a"), entry("b"), entry("c")].into();
        let probe = match Legend::measure_outside(
            "group",
            std::sync::Arc::clone(&entries),
            (640, 480),
            (500.0, 1000.0),
            false,
            false,
            &theme,
            &mut true,
            &crate::layout::fit::FitScope::root(),
        )
        .unwrap()
        {
            FitOutcome::Measured(guide) => guide,
            FitOutcome::Miss(reason) => panic!("{reason}"),
        };
        let row_gap = probe.metrics.entry_h - probe.entry_font_size() * 1.2;
        let title_gap = (probe.metrics.title_h - probe.title_font_size()).max(0.0);
        let height = 2.0 * probe.glyph_edge_pad()
            + probe.title_block.height()
            + title_gap
            + 3.0 * probe.layout.entries[0].height
            + 2.0 * row_gap;
        let width =
            2.0 * probe.layout.entries[0].width + probe.column_spacing() + 2.0 * probe.metrics.pad
                - 2.0;
        assert!(matches!(
            Legend::measure_outside(
                "group",
                std::sync::Arc::clone(&entries),
                (640, 480),
                (width, height),
                false,
                false,
                &theme,
                &mut true,
                &crate::layout::fit::FitScope::root(),
            )
            .unwrap(),
            FitOutcome::Miss(_)
        ));
        let compact = match Legend::measure_outside(
            "group",
            entries,
            (640, 480),
            (width, height),
            false,
            true,
            &theme,
            &mut true,
            &crate::layout::fit::FitScope::root(),
        )
        .unwrap()
        {
            FitOutcome::Measured(guide) => guide,
            FitOutcome::Miss(reason) => panic!("{reason}"),
        };
        assert_eq!(compact.layout.columns, 2);
        assert!(compact.entry_blocks[0].lines().len() > probe.entry_blocks[0].lines().len());
        assert!(compact.compute_box_width() <= width && compact.compute_box_height() <= height);
    }

    #[test]
    fn glyph_changes_invalidate_feedback_even_when_legend_extent_is_unchanged() {
        let mut legend = legend();
        let extent = [
            legend.compute_box_width().to_bits(),
            legend.compute_box_height().to_bits(),
        ];
        let mut calls = 0;
        let mut observations = None;
        crate::layout::plan_exact_root(
            640,
            480,
            |_| Ok(()),
            |(), assigned, round| {
                calls += 1;
                let owner = crate::layout::ComponentAddress::root();
                round.request_insets(
                    &owner,
                    &owner,
                    crate::layout::kernel::FeedbackKind::OwnedInk,
                    [1.0; 4],
                )?;
                if calls == 2 {
                    std::sync::Arc::make_mut(&mut legend.entries)[1]
                        .glyph
                        .point
                        .as_mut()
                        .unwrap()
                        .radius = 2;
                }
                assert_eq!(
                    extent,
                    [
                        legend.compute_box_width().to_bits(),
                        legend.compute_box_height().to_bits()
                    ]
                );
                let start = round.child_checkpoint();
                legend.observe_feedback(round)?;
                let current = round.retain_child_feedback(start)?.unwrap();
                if let Some(previous) = &observations {
                    assert_ne!(&current, previous);
                }
                observations = Some(current);
                Ok(crate::layout::MeasuredLayout::new(assigned, ()))
            },
            |(), _, (), _| Ok(()),
        )
        .unwrap();
        assert_eq!(calls, 2);
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
                .row_centers(
                    columns,
                    title_block,
                    entry_blocks,
                    &crate::Theme::Publication.resolve(),
                )
                .unwrap()
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
        .expect_err("overwide legend title must fail");

        assert!(error.to_string().contains("legend title text planning"));
    }

    #[test]
    fn right_box_does_not_add_border_padding_twice() {
        let legend = legend();
        let bounds = legend.right_box((10.0, 410.0, 20.0, 300.0));

        assert!((bounds.right - bounds.left - legend.compute_box_width()).abs() <= 1.0e-9);
    }
}
