//! `lichtung-plot::render::backend` — the `Backend` trait.
//!
//! A `Backend` implementation draws a chart onto some 2D surface
//! (SVG file, PNG bitmap, in-memory image). The sole Backend impl
//! lives in `svg_backend.rs` and emits SVG directly. PNG output is
//! produced by rasterizing the SVG through `resvg` (see the `png`
//! Cargo feature). Future backends (skia, vello) can drop in behind
//! the same trait.

#![deny(missing_docs)]

use crate::Error;

use super::primitives::{
    CanonicalFillStyle, CanonicalGradientFill, CanonicalLineStyle, CanonicalPlannedBracket,
    CanonicalPlannedText, CanonicalScatter,
};
use crate::layout::{
    DeferredAnnotationTextPlan, PanelBox, PhysicalTextAlignment, ResolvedTextPlan,
};
use crate::render::coordinates::{
    self, CanonicalBracket, CanonicalGradient, CanonicalPoint, CanonicalRect, CanonicalRoundedRect,
    CanonicalText,
};
use crate::PointShape;

/// A 2D point in math-style canvas coordinates after data projection.
///
/// X increases rightward and Y increases upward from the canvas bottom. A
/// backend converts this canonical canvas space to its native coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
}

impl Point2D {
    /// Construct a point.
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Complete RGBA paint at the private backend boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Rgba {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl Rgba {
    /// Construct an opaque RGB paint.
    #[must_use]
    pub(crate) const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::new(red, green, blue, 255)
    }

    /// Construct a complete RGBA paint.
    #[must_use]
    pub(crate) const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub(crate) const fn with_alpha(self, alpha: u8) -> Self {
        Self { alpha, ..self }
    }

    /// Resolve one validated legacy unit opacity to the canonical 8-bit alpha.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "finite validated unit opacity rounds to an integral value in 0..=255"
    )]
    pub(crate) fn quantized_alpha(opacity: f64) -> Result<u8, Error> {
        if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
            return Err(Error::Render(format!(
                "paint opacity must be finite and in [0, 1]; got {opacity}"
            )));
        }
        Ok((opacity * 255.0).round_ties_even() as u8)
    }

    pub(crate) const fn red(self) -> u8 {
        self.red
    }

    pub(crate) const fn green(self) -> u8 {
        self.green
    }

    pub(crate) const fn blue(self) -> u8 {
        self.blue
    }

    pub(crate) const fn alpha(self) -> u8 {
        self.alpha
    }
}

impl From<(u8, u8, u8)> for Rgba {
    fn from((red, green, blue): (u8, u8, u8)) -> Self {
        Self::rgb(red, green, blue)
    }
}

impl PartialEq<(u8, u8, u8)> for Rgba {
    fn eq(&self, other: &(u8, u8, u8)) -> bool {
        (self.red, self.green, self.blue) == *other && self.alpha == 255
    }
}

/// Visual style for a scatter point.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PointStyle {
    /// Complete point fill paint.
    pub(crate) color: Rgba,
    /// Radius in pixels.
    pub(crate) radius: u32,
    /// Optional outline stroke.
    pub(crate) outline: Option<PointOutline>,
    /// Marker geometry.
    pub(crate) shape: PointShape,
}

impl PointStyle {
    /// Construct a filled point with the default publication outline.
    #[must_use]
    pub(crate) fn new(color: (u8, u8, u8), radius: u32) -> Self {
        Self {
            color: Rgba::from(color).with_alpha(209),
            radius,
            outline: Some(PointOutline::default()),
            shape: PointShape::Circle,
        }
    }

    /// Construct a filled point without an outline.
    #[must_use]
    pub(crate) fn plain(color: (u8, u8, u8), radius: u32) -> Self {
        Self {
            color: Rgba::from(color).with_alpha(209),
            radius,
            outline: None,
            shape: PointShape::Circle,
        }
    }

    /// Set the marker geometry.
    #[must_use]
    pub const fn with_shape(mut self, shape: PointShape) -> Self {
        self.shape = shape;
        self
    }

    /// Set the point fill opacity.
    pub(crate) fn with_opacity(mut self, opacity: f64) -> Result<Self, Error> {
        self.color = self.color.with_alpha(Rgba::quantized_alpha(opacity)?);
        Ok(self)
    }
}

/// Outline stroke for scatter-like point marks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PointOutline {
    /// Independent complete outline paint.
    pub(crate) color: Rgba,
    /// Stroke width as a proportion of the point radius.
    pub(crate) width_ratio: f64,
}

impl PointOutline {
    /// Resolve the outline stroke width for the given point radius.
    #[must_use]
    pub fn width_for_radius(self, radius: u32) -> f64 {
        f64::from(radius) * self.width_ratio
    }
}

impl Default for PointOutline {
    fn default() -> Self {
        Self {
            color: Rgba::rgb(255, 255, 255),
            width_ratio: 0.1,
        }
    }
}

/// Visual style for a line (used by regression fits, axis lines).
#[derive(Debug, Clone, PartialEq)]
pub struct LineStyle {
    /// Complete stroke paint.
    pub(crate) color: Rgba,
    /// Stroke width in pixels (supports sub-pixel, e.g. 0.8).
    pub width: f32,
    /// Closed stroke pattern.
    pub pattern: StrokePattern,
}

impl LineStyle {
    /// Create a solid line style.
    #[must_use]
    pub fn solid(color: impl Into<Rgba>, width: f32) -> Self {
        Self {
            color: color.into(),
            width,
            pattern: StrokePattern::Solid,
        }
    }

    /// Create a line using the fixed `3 2` dash pattern.
    #[must_use]
    pub(crate) fn short_dashed(color: impl Into<Rgba>, width: f32) -> Self {
        Self {
            color: color.into(),
            width,
            pattern: StrokePattern::ShortDash,
        }
    }

    /// Create a line using the fixed `4 3` dash pattern.
    #[must_use]
    pub(crate) fn medium_dashed(color: impl Into<Rgba>, width: f32) -> Self {
        Self {
            color: color.into(),
            width,
            pattern: StrokePattern::MediumDash,
        }
    }

    /// Create a line using the fixed `5 3` dash pattern.
    #[must_use]
    pub(crate) fn long_dashed(color: impl Into<Rgba>, width: f32) -> Self {
        Self {
            color: color.into(),
            width,
            pattern: StrokePattern::LongDash,
        }
    }
}

/// Closed line-stroke patterns accepted by the private backend protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokePattern {
    /// Continuous stroke.
    Solid,
    /// Fixed `3 2` dash and gap lengths.
    ShortDash,
    /// Fixed `4 3` dash and gap lengths.
    MediumDash,
    /// Fixed `5 3` dash and gap lengths.
    LongDash,
}

impl StrokePattern {
    pub(crate) const fn svg_dasharray(self) -> Option<&'static str> {
        match self {
            Self::Solid => None,
            Self::ShortDash => Some("3 2"),
            Self::MediumDash => Some("4 3"),
            Self::LongDash => Some("5 3"),
        }
    }
}

/// Independent complete fill and stroke paints for a polygon or rectangle.
#[derive(Debug, Clone)]
pub(crate) struct FillStyle {
    pub(crate) color: Rgba,
    pub(crate) stroke: Rgba,
    /// Stroke width in pixels (0.0 for no outline).
    pub(crate) stroke_width: f32,
}

impl FillStyle {
    pub(crate) const fn new(color: Rgba, stroke: Rgba, stroke_width: f32) -> Self {
        Self {
            color,
            stroke,
            stroke_width,
        }
    }
}

/// One color in a linear gradient at a normalized offset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    /// Position along the gradient vector, in the inclusive range `[0, 1]`.
    pub offset: f64,
    /// Color at this position (R, G, B).
    pub color: (u8, u8, u8),
}

impl GradientStop {
    /// Construct a gradient stop.
    #[must_use]
    pub const fn new(offset: f64, color: (u8, u8, u8)) -> Self {
        Self { offset, color }
    }
}

/// Text anchor alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAnchor {
    /// Left-aligned (SVG default).
    #[default]
    Start,
    /// Center-aligned.
    Middle,
    /// Right-aligned.
    End,
}

/// Visual style for a text label.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// Complete text paint.
    pub(crate) color: Rgba,
    /// Font family.
    pub font_family: String,
    /// Font size in points.
    pub font_size: u32,
    /// Closed font-weight policy.
    pub weight: crate::TextWeight,
    /// Horizontal alignment anchor. Default: `Start`.
    pub anchor: TextAnchor,
    /// Rotation angle in degrees (clockwise). Default: `0.0`.
    pub rotation: f64,
}

/// A significance bracket drawn inside the plot panel: a horizontal
/// line spanning two x-positions, with a vertical tick at each end
/// and a glyph (e.g. "***") above the midpoint.
#[derive(Debug, Clone)]
pub struct SignificanceBracket {
    /// X position of the left tick.
    pub x_left: f64,
    /// X position of the right tick.
    pub x_right: f64,
    /// Y position of the bracket.
    pub y: f64,
    /// Glyph string ("ns", "*", "**", "***", "****").
    pub label: String,
    /// Whether the span runs along x above marks or along y to their right.
    pub orientation: BracketOrientation,
}

/// Semantic role of a named backend layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayerRole {
    /// Ordinary layer emitted in call order.
    #[default]
    Generic,
    /// Grid layer that must render behind data marks.
    Grid,
}

/// Ownership role for an ordered painted surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SurfaceRole {
    /// Canvas-owned root surface.
    CanvasRoot,
    /// Plot-owned panel surface.
    PlotPanel,
}

/// Orientation of a pairwise significance bracket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BracketOrientation {
    /// Span two x anchors at a fixed y position.
    Vertical,
    /// Span two y anchors at a fixed x position. In this orientation,
    /// `x_left`/`x_right` store bottom/top y anchors and `y` stores x.
    Horizontal,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BracketGeometryMetrics {
    pub(crate) tick_height: f64,
    pub(crate) data_gap: f64,
    pub(crate) level_step: f64,
    pub(crate) panel_padding: f64,
}

pub(crate) const SIGNIFICANCE_BRACKET_GEOMETRY: BracketGeometryMetrics = BracketGeometryMetrics {
    tick_height: 7.0,
    data_gap: 6.0,
    level_step: 26.0,
    panel_padding: 2.0,
};

pub(crate) const POLYGON_MITER_LIMIT: f64 = 4.0;

/// Visual style for a significance bracket.
#[derive(Debug, Clone)]
pub struct BracketStyle {
    /// Complete bracket-path stroke paint.
    pub(crate) color: Rgba,
    /// Complete bracket-label text paint.
    pub(crate) label_color: Rgba,
    /// Stroke width in pixels.
    pub width: f32,
    /// Label font family.
    pub font_family: String,
    /// Label font size in pixels.
    pub font_size: u32,
    /// Label line height in pixels.
    pub(crate) line_height: f64,
    /// Label font weight.
    pub weight: crate::TextWeight,
    /// Gap between the bracket line and its label.
    pub label_gap: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlannedTextOrigin {
    Middle,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlannedTextPlacement {
    pub(crate) position: Point2D,
    pub(crate) anchor: TextAnchor,
    pub(crate) rotation: f64,
    pub(crate) origin: PlannedTextOrigin,
    /// Physical column order for existing outward-stacked rotated axis titles.
    pub(crate) reverse_lines: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct PlacedText {
    pub(crate) plan: ResolvedTextPlan,
    pub(crate) placement: PlannedTextPlacement,
}

pub(crate) fn reserve_text_records<T>(
    records: &mut Vec<T>,
    additional: usize,
    environment: &crate::font::canonical::PreparedFontEnvironment,
) -> Result<(), Error> {
    environment
        .claim_text_generation_slots(additional as u128)
        .map_err(|error| Error::Render(format!("producer text slots: {error:?}")))?;
    if additional > records.capacity() - records.len() {
        let capacity = records
            .len()
            .checked_add(additional)
            .zip(records.capacity().checked_mul(2))
            .map(|(required, doubled)| required.max(doubled).max(4))
            .ok_or_else(|| Error::Render("producer text capacity overflow".into()))?;
        let bytes = capacity
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| Error::Render("producer text storage overflow".into()))?;
        environment
            .claim_unicode_bytes(bytes)
            .map_err(|error| Error::Render(format!("producer text bytes: {error}")))?;
        records
            .try_reserve_exact(capacity - records.len())
            .map_err(|_| Error::Render("producer text allocation failed".into()))?;
    }
    Ok(())
}

impl PlacedText {
    pub(crate) fn middle(
        plan: ResolvedTextPlan,
        position: Point2D,
        anchor: TextAnchor,
        rotation: f64,
    ) -> Self {
        Self {
            plan,
            placement: PlannedTextPlacement::middle(position, anchor, rotation),
        }
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "line counts are bounded by text admission"
    )]
    /// Deferred producers retain their whole-pixel block placement until migration.
    /// Snap after the multiline offset, using the former canonical nearest-even rule.
    pub(crate) fn from_deferred_first_line(
        plan: ResolvedTextPlan,
        first: Point2D,
        anchor: TextAnchor,
        rotation: f64,
    ) -> Self {
        let offset = plan.lines.len().saturating_sub(1) as f64 * plan.line_height() / 2.0;
        let angle = rotation.to_radians();
        Self {
            plan,
            placement: PlannedTextPlacement::middle(
                Point2D::new(
                    (first.x - offset * angle.sin()).round_ties_even(),
                    (first.y - offset * angle.cos()).round_ties_even(),
                ),
                anchor,
                rotation,
            ),
        }
    }

    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        backend.draw_planned_text(self.placement, &self.plan)
    }

    pub(crate) fn with_reversed_lines(mut self) -> Self {
        self.placement.reverse_lines = true;
        self
    }
}

impl PlannedTextPlacement {
    pub(crate) fn line_y(self, plan: &ResolvedTextPlan, y: i64) -> Result<i64, Error> {
        if !self.reverse_lines {
            return Ok(y);
        }
        plan.last_baseline_quanta
            .unwrap_or(0)
            .checked_sub(y)
            .and_then(|offset| plan.first_baseline_quanta.unwrap_or(0).checked_add(offset))
            .ok_or_else(|| Error::Render("reversed text line placement overflowed".into()))
    }
    pub(crate) const fn middle(position: Point2D, anchor: TextAnchor, rotation: f64) -> Self {
        Self {
            position,
            anchor,
            rotation,
            origin: PlannedTextOrigin::Middle,
            reverse_lines: false,
        }
    }

    pub(crate) const fn baseline(position: Point2D, anchor: TextAnchor, rotation: f64) -> Self {
        Self {
            position,
            anchor,
            rotation,
            origin: PlannedTextOrigin::Baseline,
            reverse_lines: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PlannedSignificanceCall<'a> {
    pub(crate) bracket: &'a SignificanceBracket,
    pub(crate) style: &'a BracketStyle,
    pub(crate) label: &'a DeferredAnnotationTextPlan,
}

/// The `Backend` trait. Every concrete renderer (Plotters,
/// future Skia, future Vello) implements this.
///
/// All draw methods take a mutable `&mut self` and return
/// `Result<(), Error>`. Backends may buffer internally and
/// only emit output on `end()` (or `finalize()`).
///
/// Geometry passed to raw draw methods is already projected into math-style
/// canvas coordinates: X increases rightward and Y increases upward. Data
/// scaling and panel placement are renderer responsibilities, not backend
/// responsibilities. Canonical methods consume retained top-down operands for
/// the selected canvas and must not delegate back through raw draw methods.
pub trait Backend {
    /// Draw retained line, run and glyph placement without layout or conversion.
    fn draw_canonical_text(&mut self, _text: &CanonicalPlannedText) -> Result<(), Error> {
        Err(Error::Render(
            "backend does not support canonical text".into(),
        ))
    }

    /// Draw a retained bracket and its canonical label placement.
    fn draw_canonical_bracket(&mut self, _bracket: &CanonicalPlannedBracket) -> Result<(), Error> {
        Err(Error::Render(
            "backend does not support canonical bracket".into(),
        ))
    }

    /// Draw retained top-down markers without raw geometry conversion.
    fn draw_canonical_scatter(&mut self, _scatter: &CanonicalScatter) -> Result<(), Error> {
        Err(Error::Render(
            "backend does not support canonical scatter".into(),
        ))
    }

    /// Draw retained top-down line points and stroke operands.
    fn draw_canonical_line(
        &mut self,
        _points: &[CanonicalPoint],
        _style: &CanonicalLineStyle,
    ) -> Result<(), Error> {
        Err(Error::Render(
            "backend does not support canonical lines".into(),
        ))
    }

    /// Draw retained top-down polygon points and paint operands.
    fn draw_canonical_polygon(
        &mut self,
        _points: &[CanonicalPoint],
        _style: &CanonicalFillStyle,
    ) -> Result<(), Error> {
        Err(Error::Render(
            "backend does not support canonical polygons".into(),
        ))
    }

    /// Draw retained top-down gradient geometry and normalized stops.
    fn draw_canonical_gradient(&mut self, _gradient: &CanonicalGradientFill) -> Result<(), Error> {
        Err(Error::Render(
            "backend does not support canonical gradients".into(),
        ))
    }

    /// Begin a clip from retained, checked top-down spatial operands.
    fn begin_canonical_clip(&mut self, _region: coordinates::CanonicalBox) -> Result<(), Error> {
        Err(Error::Render(
            "backend does not support canonical clips".into(),
        ))
    }

    /// Paint a surface from retained, checked top-down spatial operands.
    fn draw_canonical_surface(
        &mut self,
        _bounds: coordinates::CanonicalBox,
        _paint: Rgba,
        _role: SurfaceRole,
    ) -> Result<(), Error> {
        Err(Error::Render(
            "backend does not support canonical surfaces".into(),
        ))
    }

    /// Draw a rectangle from retained top-down geometry, stroke, and paint.
    fn draw_canonical_rounded_rect(
        &mut self,
        _rounded: coordinates::CanonicalRoundedBox,
    ) -> Result<(), Error> {
        Err(Error::Render(
            "backend does not support canonical rectangles".into(),
        ))
    }

    /// Begin a chart of the given pixel size. Must be called before
    /// any draw method.
    fn begin(&mut self, width: u32, height: u32) -> Result<(), Error>;

    /// Begin a named SVG layer (`<g id="...">`). Draw calls between
    /// `begin_layer` and `end_layer` are grouped. Layers control
    /// z-order: earlier layers render behind later ones.
    fn begin_layer(&mut self, _name: &str) -> Result<(), Error> {
        Ok(())
    }

    /// Begin a named layer with an explicit semantic role.
    ///
    /// Backends that do not reorder layers may use the default delegation to
    /// [`Backend::begin_layer`].
    fn begin_layer_with_role(&mut self, name: &str, _role: LayerRole) -> Result<(), Error> {
        self.begin_layer(name)
    }

    /// End the current SVG layer.
    fn end_layer(&mut self) -> Result<(), Error> {
        Ok(())
    }

    /// Begin clipping subsequent draw calls to a math-style canvas rectangle.
    /// Clip scopes may nest with layers and must be closed in last-in-first-out
    /// order with [`Backend::end_clip`].
    fn begin_clip(&mut self, region: PanelBox) -> Result<(), Error>;

    /// End the current clip scope.
    fn end_clip(&mut self) -> Result<(), Error>;

    /// Paint a validated math-style rectangular surface in semantic surface
    /// order rather than ordinary draw-call order.
    fn draw_surface(
        &mut self,
        bounds: PanelBox,
        paint: Rgba,
        role: SurfaceRole,
    ) -> Result<(), Error>;

    /// Draw a scatter cloud.
    fn draw_scatter(&mut self, points: &[Point2D], style: &PointStyle) -> Result<(), Error>;

    /// Draw a line strip (e.g. a regression fit).
    fn draw_line(&mut self, points: &[Point2D], style: &LineStyle) -> Result<(), Error>;

    /// Draw a filled polygon (bar body, violin body).
    fn draw_polygon(&mut self, points: &[Point2D], style: &FillStyle) -> Result<(), Error>;

    /// Fill a rectangle with a native linear gradient.
    ///
    /// `bounds` and the gradient vector use math-style canvas coordinates.
    /// Stops must begin at 0, end at 1, and have strictly increasing offsets.
    fn draw_linear_gradient(
        &mut self,
        bounds: PanelBox,
        start: Point2D,
        end: Point2D,
        stops: &[GradientStop],
    ) -> Result<(), Error>;

    /// Draw a text label at finite signed math-style canvas coordinates with
    /// alignment and optional rotation.
    fn draw_text(&mut self, position: Point2D, text: &str, style: &TextStyle) -> Result<(), Error>;

    /// Draw one immutable operation-owned text plan.
    #[doc(hidden)]
    #[allow(
        private_interfaces,
        reason = "immutable text plans are a crate-private renderer transition"
    )]
    fn draw_planned_text(
        &mut self,
        placement: PlannedTextPlacement,
        plan: &ResolvedTextPlan,
    ) -> Result<(), Error> {
        self.draw_text(
            placement.position,
            &plan.source,
            &planned_text_style(placement, plan),
        )
    }

    /// Draw a rounded rectangle. Coordinates are in math-coordinate space
    /// (y increases upward). `x` and `y` are the bottom-left corner,
    /// `w` and `h` are width and height, `r` is the corner radius.
    ///
    fn draw_rounded_rect(
        &mut self,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        r: f64,
        style: &FillStyle,
    ) -> Result<(), Error>;

    /// Draw a significance bracket.
    fn draw_bracket(
        &mut self,
        bracket: &SignificanceBracket,
        style: &BracketStyle,
    ) -> Result<(), Error>;

    /// Draw one significance bracket with its immutable planned label.
    #[doc(hidden)]
    #[allow(
        private_interfaces,
        reason = "immutable significance plans are a crate-private renderer transition"
    )]
    fn draw_planned_bracket(&mut self, call: PlannedSignificanceCall<'_>) -> Result<(), Error> {
        self.draw_bracket(call.bracket, call.style)
    }

    /// End the chart. Flushes any buffered output. After `end()`,
    /// no more draw methods may be called until the next `begin()`.
    fn end(&mut self) -> Result<(), Error>;
}

pub(crate) fn validate_clip_region(region: PanelBox) -> Result<CanonicalRect, Error> {
    coordinates::canonical_clip(region)
}

pub(crate) fn validate_surface_bounds(bounds: PanelBox) -> Result<CanonicalRect, Error> {
    coordinates::canonical_clip(bounds)
}

pub(crate) fn validate_points(
    points: &[Point2D],
    context: &str,
) -> Result<Vec<CanonicalPoint>, Error> {
    coordinates::canonical_points(points, context)
}

pub(crate) fn points_share_canonical_position(
    first: Point2D,
    second: Point2D,
    context: &str,
) -> Result<bool, Error> {
    let points = coordinates::canonical_points(&[first, second], context)?;
    Ok(points[0].same_position(points[1]))
}

pub(crate) fn validate_line_points(points: &[Point2D]) -> Result<Vec<CanonicalPoint>, Error> {
    coordinates::canonical_line_points(points)
}

pub(crate) fn validate_polygon_points(points: &[Point2D]) -> Result<Vec<CanonicalPoint>, Error> {
    coordinates::canonical_polygon_points(points)
}

pub(crate) fn validate_triangle_points(
    points: &[Point2D],
    radius: u32,
) -> Result<Vec<[CanonicalPoint; 3]>, Error> {
    coordinates::canonical_triangles(points, radius)
}

pub(crate) fn validate_point_style(style: &PointStyle) -> Result<(), Error> {
    if let Some(outline) = style.outline {
        let width = outline.width_for_radius(style.radius);
        if !outline.width_ratio.is_finite() || outline.width_ratio < 0.0 || !width.is_finite() {
            return Err(Error::Render(format!(
                "point outline width must be finite and non-negative; got {}",
                outline.width_ratio
            )));
        }
    }
    Ok(())
}

pub(crate) fn validate_line_style(style: &LineStyle) -> Result<(), Error> {
    if style.width.is_finite() && style.width >= 0.0 {
        Ok(())
    } else {
        Err(Error::Render(format!(
            "line width must be finite and non-negative; got {}",
            style.width
        )))
    }
}

pub(crate) fn validate_fill_style(style: &FillStyle) -> Result<(), Error> {
    if !style.stroke_width.is_finite() || style.stroke_width < 0.0 {
        return Err(Error::Render(format!(
            "fill stroke width must be finite and non-negative; got {}",
            style.stroke_width
        )));
    }
    Ok(())
}

pub(crate) fn validate_linear_gradient(
    bounds: PanelBox,
    start: Point2D,
    end: Point2D,
    stops: &[GradientStop],
) -> Result<CanonicalGradient, Error> {
    coordinates::canonical_gradient(bounds, start, end, stops)
}

pub(crate) fn validate_text_style(style: &TextStyle) -> Result<(), Error> {
    if style.rotation.is_finite() {
        Ok(())
    } else {
        Err(Error::Render(format!(
            "text rotation must be finite; got {}",
            style.rotation
        )))
    }
}

pub(crate) fn validate_text(position: Point2D, style: &TextStyle) -> Result<CanonicalText, Error> {
    validate_text_style(style)?;
    coordinates::canonical_text(
        position,
        style.rotation,
        coordinates::CoordinatePrecision::CssQuantum,
    )
}

pub(crate) fn planned_text_style(
    placement: PlannedTextPlacement,
    plan: &ResolvedTextPlan,
) -> TextStyle {
    TextStyle {
        color: plan.style.paint,
        font_family: plan.style.stack_source.clone(),
        font_size: plan.style.font_size,
        weight: plan.style.weight,
        anchor: placement.anchor,
        rotation: placement.rotation,
    }
}

pub(crate) const fn nearest_even_text_half(value: i64) -> Option<i64> {
    if value < 0 {
        return None;
    }
    let quotient = value / 2;
    if value % 2 == 0 || quotient % 2 == 0 {
        Some(quotient)
    } else {
        quotient.checked_add(1)
    }
}

pub(crate) fn validate_planned_text(
    placement: PlannedTextPlacement,
    plan: &ResolvedTextPlan,
) -> Result<CanonicalText, Error> {
    let canonical = coordinates::canonical_text(
        placement.position,
        placement.rotation,
        coordinates::CoordinatePrecision::CssQuantum,
    )?;
    validate_text_plan(placement, plan)?;
    Ok(canonical)
}

#[expect(
    clippy::too_many_lines,
    reason = "one protocol validator checks linked immutable style, line, discretionary-break, and glyph spans"
)]
pub(crate) fn validate_text_plan(
    placement: PlannedTextPlacement,
    plan: &ResolvedTextPlan,
) -> Result<(), Error> {
    let expected_alignment = match placement.anchor {
        TextAnchor::Start => PhysicalTextAlignment::Start,
        TextAnchor::Middle => PhysicalTextAlignment::Center,
        TextAnchor::End => PhysicalTextAlignment::End,
    };
    if plan.style.alignment != expected_alignment {
        return Err(Error::Render(
            "planned text alignment does not match its backend placement".into(),
        ));
    }
    if plan.style.font_size == 0
        || plan.style.stack_source.is_empty()
        || plan.style.stack.is_empty()
        || plan.style.line_height_quanta <= 0
        || plan.line_pitch_quanta <= 0
        || plan.width_quanta < 0
        || plan.height_quanta < 0
    {
        return Err(Error::Render(
            "planned text has invalid resolved style or dimensions".into(),
        ));
    }
    let valid_span = |start: u32, end: u32| {
        let start = start as usize;
        let end = end as usize;
        start <= end
            && end <= plan.source.len()
            && plan.source.is_char_boundary(start)
            && plan.source.is_char_boundary(end)
    };
    let valid_ink = |left: i64, bottom: i64, right: i64, top: i64| left <= right && bottom <= top;
    match (plan.lines.first(), plan.lines.last()) {
        (Some(first), Some(last)) => {
            if plan.first_baseline_quanta != Some(first.translation_y_quanta)
                || plan.last_baseline_quanta != Some(last.translation_y_quanta)
                || plan.height_quanta == 0
            {
                return Err(Error::Render(
                    "planned text baselines do not match its stored lines".into(),
                ));
            }
        }
        (None, None) => {
            if plan.first_baseline_quanta.is_some()
                || plan.last_baseline_quanta.is_some()
                || plan.width_quanta != 0
                || plan.height_quanta != 0
            {
                return Err(Error::Render(
                    "empty planned text has nonempty metrics".into(),
                ));
            }
        }
        _ => unreachable!("a slice cannot have only one of first/last"),
    }
    for (index, line) in plan.lines.iter().enumerate() {
        placement.line_y(plan, line.translation_y_quanta)?;
        if let Some(soft) = line.selected_soft_hyphen {
            if !valid_span(soft.start_byte, soft.end_byte)
                || soft.start_byte < line.source_span.start_byte
                || soft.end_byte != line.source_span.end_byte
                || &plan.source[soft.start_byte as usize..soft.end_byte as usize] != "\u{ad}"
                || !line.runs.iter().any(|run| {
                    run.source_span.start_byte <= soft.start_byte
                        && run.source_span.end_byte >= soft.end_byte
                })
            {
                return Err(Error::Render(
                    "planned text has an invalid selected soft-hyphen span or run".into(),
                ));
            }
        }
        let expected_y = plan
            .first_baseline_quanta
            .and_then(|first| {
                i64::try_from(index)
                    .ok()
                    .and_then(|index| plan.line_pitch_quanta.checked_mul(index))
                    .and_then(|offset| first.checked_add(offset))
            })
            .ok_or_else(|| Error::Render("planned text line offset overflow".into()))?;
        if line.translation_y_quanta != expected_y
            || !valid_span(line.source_span.start_byte, line.source_span.end_byte)
            || line
                .ink_quanta
                .is_some_and(|ink| !valid_ink(ink.left, ink.bottom, ink.right, ink.top))
        {
            return Err(Error::Render(
                "planned text has an invalid line span, translation, or ink box".into(),
            ));
        }
        for run in &line.runs {
            if !valid_span(run.source_span.start_byte, run.source_span.end_byte)
                || run.source_span.start_byte < line.source_span.start_byte
                || run.source_span.end_byte > line.source_span.end_byte
                || run.units_per_em == 0
                || run
                    .ink_quanta
                    .is_some_and(|ink| !valid_ink(ink.left, ink.bottom, ink.right, ink.top))
                || run.glyphs.iter().any(|glyph| {
                    glyph
                        .ink_quanta
                        .is_some_and(|ink| !valid_ink(ink.left, ink.bottom, ink.right, ink.top))
                })
            {
                return Err(Error::Render(
                    "planned text has an invalid shaped run or glyph ink box".into(),
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_rounded_rect(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
) -> Result<CanonicalRoundedRect, Error> {
    coordinates::canonical_rounded_rect(x, y, width, height, radius)
}

pub(crate) fn validate_bracket(
    bracket: &SignificanceBracket,
    style: &BracketStyle,
) -> Result<CanonicalBracket, Error> {
    if !style.width.is_finite() || style.width < 0.0 {
        return Err(Error::Render(format!(
            "bracket width must be finite and non-negative; got {}",
            style.width
        )));
    }
    if style.font_size == 0
        || !style.line_height.is_finite()
        || style.line_height <= 0.0
        || !style.label_gap.is_finite()
        || style.label_gap < 0.0
    {
        return Err(Error::Render(
            "bracket label style requires positive font size and line height and finite nonnegative gap"
                .into(),
        ));
    }
    coordinates::canonical_bracket(
        bracket,
        SIGNIFICANCE_BRACKET_GEOMETRY.tick_height,
        style.label_gap,
    )
}

pub(crate) fn planned_bracket_placement(canonical: CanonicalBracket) -> PlannedTextPlacement {
    let position = canonical.label_position.value();
    match canonical.orientation {
        BracketOrientation::Vertical => {
            PlannedTextPlacement::baseline(position, TextAnchor::Middle, 0.0)
        }
        BracketOrientation::Horizontal => {
            PlannedTextPlacement::middle(position, TextAnchor::Start, 0.0)
        }
    }
}

pub(crate) fn validate_planned_bracket(
    call: PlannedSignificanceCall<'_>,
) -> Result<CanonicalBracket, Error> {
    let canonical = validate_bracket(call.bracket, call.style)?;
    validate_bracket_label(canonical, call)?;
    Ok(canonical)
}

pub(crate) fn validate_bracket_label(
    canonical: CanonicalBracket,
    call: PlannedSignificanceCall<'_>,
) -> Result<(), Error> {
    let plan = &call.label.plan;
    validate_text_plan(planned_bracket_placement(canonical), plan)?;
    if call.label.exact_override != plan.style.exact_override
        || call.bracket.label != plan.source
        || call.style.label_color != plan.style.paint
        || call.style.font_family != plan.style.stack_source
        || call.style.font_size != plan.style.font_size
        || call.style.weight != plan.style.weight
    {
        return Err(Error::Render(
            "planned significance label does not match its frozen source and style".into(),
        ));
    }
    Ok(())
}

pub(crate) fn with_clip<T>(
    backend: &mut dyn Backend,
    region: PanelBox,
    draw: impl FnOnce(&mut dyn Backend) -> Result<T, Error>,
) -> Result<T, Error> {
    backend.begin_clip(region)?;
    finish_scope(draw(backend), backend.end_clip(), "clip scope")
}

pub(crate) fn with_layer<T>(
    backend: &mut dyn Backend,
    name: &str,
    draw: impl FnOnce(&mut dyn Backend) -> Result<T, Error>,
) -> Result<T, Error> {
    backend.begin_layer(name)?;
    finish_scope(
        draw(backend),
        backend.end_layer(),
        &format!("layer {name:?}"),
    )
}

pub(crate) fn with_layer_role<T>(
    backend: &mut dyn Backend,
    name: &str,
    role: LayerRole,
    draw: impl FnOnce(&mut dyn Backend) -> Result<T, Error>,
) -> Result<T, Error> {
    backend.begin_layer_with_role(name, role)?;
    finish_scope(
        draw(backend),
        backend.end_layer(),
        &format!("layer {name:?}"),
    )
}

pub(crate) fn finish_scope<T>(
    primary: Result<T, Error>,
    cleanup: Result<(), Error>,
    scope: &str,
) -> Result<T, Error> {
    match (primary, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(primary), Ok(())) => Err(primary),
        (Ok(_), Err(cleanup)) => Err(Error::Render(format!("failed to close {scope}: {cleanup}"))),
        (Err(primary), Err(cleanup)) => Err(Error::Render(format!(
            "{primary}; additionally failed to close {scope}: {cleanup}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn producer_record_storage_refuses_before_allocating() {
        use crate::font::canonical::PreparedFontEnvironment;
        for resource in ["slots", "bytes"] {
            let environment = PreparedFontEnvironment::synthetic();
            if resource == "slots" {
                environment
                    .claim_text_generation_slots(
                        1_000_000 - environment.generation_resources().usage().unwrap().0,
                    )
                    .unwrap();
            } else {
                let used = environment.resource_usage().unwrap().font_bytes;
                environment
                    .claim_unicode_bytes(u64::try_from(536_870_912 - used).unwrap())
                    .unwrap();
            }
            let mut records = Vec::<super::PlacedText>::new();
            assert!(
                super::reserve_text_records(&mut records, 1, &environment).is_err(),
                "{resource}"
            );
            assert_eq!(records.capacity(), 0);
        }
        let environment = PreparedFontEnvironment::synthetic();
        environment
            .claim_processing_visits(
                1_000_000 - environment.resource_usage().unwrap().processing_visits,
            )
            .unwrap();
        let mut records = Vec::<super::PlacedText>::new();
        super::reserve_text_records(&mut records, 1, &environment).unwrap();
        assert!(
            records.capacity() >= 1,
            "record retention has no processing price"
        );
    }
    use super::*;

    fn valid_bracket() -> (SignificanceBracket, BracketStyle) {
        (
            SignificanceBracket {
                x_left: 2.0,
                x_right: 8.0,
                y: 10.0,
                label: "*".into(),
                orientation: BracketOrientation::Vertical,
            },
            BracketStyle {
                color: (0, 0, 0).into(),
                width: 1.0,
                label_color: (0, 0, 0).into(),
                font_family: "sans-serif".into(),
                font_size: 12,
                line_height: 14.4,
                weight: crate::TextWeight::Normal,
                label_gap: 5.0,
            },
        )
    }

    fn valid_gradient() -> (PanelBox, Point2D, Point2D, [GradientStop; 2]) {
        (
            PanelBox::new(1.0, 5.0, 2.0, 8.0),
            Point2D::new(3.0, 2.0),
            Point2D::new(3.0, 8.0),
            [
                GradientStop::new(0.0, (0, 0, 0)),
                GradientStop::new(1.0, (255, 255, 255)),
            ],
        )
    }

    fn valid_text_style() -> TextStyle {
        TextStyle {
            color: (0, 0, 0).into(),
            font_family: "sans-serif".into(),
            font_size: 12,
            weight: crate::TextWeight::Normal,
            anchor: TextAnchor::Start,
            rotation: 0.0,
        }
    }

    #[test]
    fn line_patterns_have_one_exact_closed_serialization() {
        let styles = [
            LineStyle::solid((1, 2, 3), 1.0),
            LineStyle::short_dashed((1, 2, 3), 1.0),
            LineStyle::medium_dashed((1, 2, 3), 1.0),
            LineStyle::long_dashed((1, 2, 3), 1.0),
        ];

        assert_eq!(
            styles.map(|style| (style.pattern, style.pattern.svg_dasharray())),
            [
                (StrokePattern::Solid, None),
                (StrokePattern::ShortDash, Some("3 2")),
                (StrokePattern::MediumDash, Some("4 3")),
                (StrokePattern::LongDash, Some("5 3")),
            ]
        );
    }

    #[test]
    fn planned_text_half_quanta_use_nearest_even() {
        assert_eq!(nearest_even_text_half(-1), None);
        assert_eq!(nearest_even_text_half(0), Some(0));
        assert_eq!(nearest_even_text_half(1), Some(0));
        assert_eq!(nearest_even_text_half(2), Some(1));
        assert_eq!(nearest_even_text_half(3), Some(2));
        assert_eq!(nearest_even_text_half(5), Some(2));
        assert_eq!(nearest_even_text_half(7), Some(4));
    }

    #[test]
    fn deferred_first_line_snaps_final_half_pixel_offsets_like_old_canonical_text() {
        let environment = crate::font::canonical::PreparedFontEnvironment::synthetic();
        let plan = environment
            .plan_text(crate::layout::TextPlanInput {
                source: "A\nAA",
                stack: "Test Sans",
                weight: crate::TextWeight::Normal,
                font_size: 10,
                line_height: crate::layout::TextLineHeight::ResolvedQuanta(17 * 1024),
                maximum_width_quanta: i64::MAX,
                alignment: PhysicalTextAlignment::Center,
                paint: Rgba::rgb(0, 0, 0),
                provenance: crate::layout::TextPlanProvenance {
                    pair_ordinal: 0,
                    owner: "deferred placement".into(),
                },
                exact_override: false,
            })
            .unwrap();
        for first in [
            Point2D::new(34.0, 50.0),
            Point2D::new(35.0, 49.0),
            Point2D::new(-34.0, -50.0),
        ] {
            for rotation in [0.0_f64, -90.0] {
                let angle = rotation.to_radians();
                let old = coordinates::canonical_text(
                    Point2D::new(first.x - 8.5 * angle.sin(), first.y - 8.5 * angle.cos()),
                    rotation,
                    coordinates::CoordinatePrecision::Integer,
                )
                .unwrap();
                let placed = PlacedText::from_deferred_first_line(
                    plan.clone(),
                    first,
                    TextAnchor::Middle,
                    rotation,
                );
                assert_eq!(placed.placement.position, old.position.value());
                assert_eq!(
                    validate_planned_text(placed.placement, &plan)
                        .unwrap()
                        .position
                        .value(),
                    old.position.value()
                );
            }
        }
    }

    #[test]
    fn deferred_bracket_labels_keep_canonical_subpixel_placement() {
        let (mut bracket, style) = valid_bracket();
        bracket.x_left = 10.25;
        bracket.x_right = 20.75;
        bracket.y = 30.5;
        for orientation in [BracketOrientation::Vertical, BracketOrientation::Horizontal] {
            bracket.orientation = orientation;
            let canonical = validate_bracket(&bracket, &style).unwrap();
            let old = coordinates::canonical_text(
                canonical.label_position.value(),
                0.0,
                coordinates::CoordinatePrecision::Integer,
            )
            .unwrap();
            assert_ne!(canonical.label_position.value(), old.position.value());
            assert_eq!(
                planned_bracket_placement(canonical).position,
                canonical.label_position.value()
            );
        }
    }

    #[test]
    fn rgba_construction_and_rgb_conversion_preserve_channels() {
        let opaque = Rgba::rgb(1, 2, 3);
        let converted = Rgba::from((1, 2, 3));
        let transparent = Rgba::new(4, 5, 6, 0);

        assert_eq!(opaque, converted);
        assert_eq!(
            (opaque.red(), opaque.green(), opaque.blue(), opaque.alpha()),
            (1, 2, 3, 255)
        );
        assert_eq!(
            (
                transparent.red(),
                transparent.green(),
                transparent.blue(),
                transparent.alpha()
            ),
            (4, 5, 6, 0)
        );
    }

    #[test]
    fn legacy_opacity_rounds_nearest_even_once_before_complete_paint() {
        for (opacity, alpha) in [
            (0.0, 0),
            (1.0 / 510.0, 0),
            (3.0 / 510.0, 2),
            (0.25, 64),
            (0.70, 178),
            (0.82, 209),
            (1.0, 255),
        ] {
            assert_eq!(Rgba::quantized_alpha(opacity).unwrap(), alpha);
        }
        for opacity in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
            assert!(Rgba::quantized_alpha(opacity).is_err());
        }
        let fill = FillStyle::new(
            Rgba::rgb(1, 2, 3).with_alpha(Rgba::quantized_alpha(0.25).unwrap()),
            Rgba::new(4, 5, 6, 128),
            1.0,
        );
        assert_eq!(fill.color.alpha(), 64);
        assert_eq!(fill.stroke.alpha(), 128);
    }

    #[test]
    fn surface_bounds_require_positive_canonical_geometry() {
        assert!(validate_surface_bounds(PanelBox::new(1.0, 2.0, 3.0, 4.0)).is_ok());
        assert!(validate_surface_bounds(PanelBox::new(f64::NAN, 2.0, 3.0, 4.0)).is_err());
        assert!(validate_surface_bounds(PanelBox::new(1.0, 1.0, 3.0, 4.0)).is_err());

        let error = validate_surface_bounds(PanelBox::new(0.0, 0.0004, 0.0, 1.0)).unwrap_err();
        assert!(error.to_string().contains("positive extent collapses"));
    }

    #[test]
    fn primitive_point_validation_rejects_nonfinite_coordinates() {
        assert!(validate_points(&[Point2D::new(1.0, 2.0)], "test").is_ok());
        assert!(validate_points(&[Point2D::new(f64::NAN, 2.0)], "test").is_err());
        assert!(validate_points(&[Point2D::new(1.0, f64::INFINITY)], "test").is_err());
    }

    #[test]
    fn coordinate_conversion_rejects_values_outside_exact_signed_range() {
        let error = validate_points(&[Point2D::new(f64::MAX, 0.0)], "test").unwrap_err();

        assert!(error.to_string().contains("quantum conversion overflowed"));
    }

    #[test]
    fn text_conversion_rejects_nonfinite_and_unrepresentable_signed_positions() {
        let nonfinite =
            validate_text(Point2D::new(f64::NAN, 0.0), &valid_text_style()).unwrap_err();
        let unrepresentable =
            validate_text(Point2D::new(f64::MAX, 0.0), &valid_text_style()).unwrap_err();

        assert!(nonfinite
            .to_string()
            .contains("layout spatial input must be finite"));
        assert!(unrepresentable
            .to_string()
            .contains("quantum conversion overflowed"));
    }

    #[test]
    fn primitive_conversion_retains_representable_subprecision_points() {
        let points =
            validate_line_points(&[Point2D::new(0.0, 0.0), Point2D::new(0.0006, 0.0006)]).unwrap();

        assert_eq!(points[1].value(), Point2D::new(1.0 / 1024.0, 1.0 / 1024.0));
    }

    #[test]
    fn primitive_conversion_rejects_collapsed_required_distinction() {
        let error = validate_line_points(&[Point2D::new(0.0, 0.0), Point2D::new(0.0004, 0.0004)])
            .unwrap_err();

        assert!(error.to_string().contains("distinct points"));
        assert!(error.to_string().contains("collapse"));
    }

    #[test]
    fn primitive_conversion_rejects_partial_point_collapse() {
        let error = validate_line_points(&[
            Point2D::new(0.0, 0.0),
            Point2D::new(0.0004, 0.0004),
            Point2D::new(1.0, 1.0),
        ])
        .unwrap_err();

        assert!(error.to_string().contains("distinct points"));
    }

    #[test]
    fn polygon_conversion_rejects_partial_point_collapse() {
        let error = validate_polygon_points(&[
            Point2D::new(0.0, 0.0),
            Point2D::new(0.0004, 0.0004),
            Point2D::new(1.0, 1.0),
            Point2D::new(2.0, 0.0),
        ])
        .unwrap_err();

        assert!(error.to_string().contains("distinct adjacent points"));
    }

    #[test]
    fn polygon_conversion_retains_nonadjacent_canonical_self_touch() {
        let points = validate_polygon_points(&[
            Point2D::new(0.0, 0.0),
            Point2D::new(2.0, 0.0),
            Point2D::new(0.0004, 0.0004),
            Point2D::new(0.0, 2.0),
        ])
        .unwrap();

        assert_eq!(points[0], points[2]);
    }

    #[test]
    fn polygon_conversion_rejects_adjacent_signed_zero_collapse() {
        let error = validate_polygon_points(&[
            Point2D::new(0.0, -0.0004),
            Point2D::new(0.0, 0.0004),
            Point2D::new(2.0, 0.0),
            Point2D::new(0.0, 2.0),
        ])
        .unwrap_err();

        assert!(error.to_string().contains("distinct adjacent points"));
    }

    #[test]
    fn clip_conversion_retains_positive_subprecision_extents() {
        let rect = validate_clip_region(PanelBox::new(0.0, 0.0006, 0.0, 0.0006)).unwrap();

        assert_eq!(
            rect.panel(),
            PanelBox::new(0.0, 1.0 / 1024.0, 0.0, 1.0 / 1024.0)
        );
    }

    #[test]
    fn clip_conversion_rejects_collapsed_positive_extents() {
        let error = validate_clip_region(PanelBox::new(0.0, 0.0004, 0.0, 0.0004)).unwrap_err();

        assert!(error.to_string().contains("positive extent collapses"));
    }

    #[test]
    fn line_style_validation_accepts_nonnegative_finite_widths_only() {
        assert!(validate_line_style(&LineStyle::solid((0, 0, 0), 0.0)).is_ok());
        assert!(validate_line_style(&LineStyle::solid((0, 0, 0), 1.0)).is_ok());
        for width in [-1.0, f32::NAN, f32::INFINITY] {
            assert!(validate_line_style(&LineStyle::solid((0, 0, 0), width)).is_err());
        }
    }

    #[test]
    fn linear_gradient_validation_accepts_strict_complete_stops() {
        let (bounds, start, end, stops) = valid_gradient();

        assert!(validate_linear_gradient(bounds, start, end, &stops).is_ok());
    }

    #[test]
    fn linear_gradient_validation_rejects_invalid_geometry() {
        let (bounds, start, end, stops) = valid_gradient();

        assert!(validate_linear_gradient(
            PanelBox::new(f64::NAN, bounds.x1, bounds.y0, bounds.y1),
            start,
            end,
            &stops
        )
        .is_err());
        assert!(validate_linear_gradient(
            PanelBox::new(bounds.x0, bounds.x0, bounds.y0, bounds.y1),
            start,
            end,
            &stops
        )
        .is_err());
        assert!(validate_linear_gradient(
            PanelBox::new(-f64::MAX, f64::MAX, bounds.y0, bounds.y1),
            start,
            end,
            &stops
        )
        .is_err());
        assert!(validate_linear_gradient(bounds, start, start, &stops).is_err());
        assert!(
            validate_linear_gradient(bounds, Point2D::new(f64::NAN, start.y), end, &stops).is_err()
        );
    }

    #[test]
    fn linear_gradient_validation_rejects_incomplete_or_unordered_stops() {
        let (bounds, start, end, _) = valid_gradient();
        let cases = [
            vec![GradientStop::new(0.0, (0, 0, 0))],
            vec![
                GradientStop::new(0.1, (0, 0, 0)),
                GradientStop::new(1.0, (255, 255, 255)),
            ],
            vec![
                GradientStop::new(0.0, (0, 0, 0)),
                GradientStop::new(0.5, (127, 127, 127)),
            ],
            vec![
                GradientStop::new(0.0, (0, 0, 0)),
                GradientStop::new(0.5, (127, 127, 127)),
                GradientStop::new(0.5, (128, 128, 128)),
                GradientStop::new(1.0, (255, 255, 255)),
            ],
            vec![
                GradientStop::new(0.0, (0, 0, 0)),
                GradientStop::new(f64::NAN, (127, 127, 127)),
                GradientStop::new(1.0, (255, 255, 255)),
            ],
            vec![
                GradientStop::new(0.0, (0, 0, 0)),
                GradientStop::new(1.1, (127, 127, 127)),
                GradientStop::new(1.0, (255, 255, 255)),
            ],
        ];

        assert!(cases
            .iter()
            .all(|stops| validate_linear_gradient(bounds, start, end, stops).is_err()));
    }

    #[test]
    fn linear_gradient_conversion_rejects_collapsed_endpoints_and_stops() {
        let (bounds, _, _, stops) = valid_gradient();
        let endpoint_error = validate_linear_gradient(
            bounds,
            Point2D::new(0.0, 0.0),
            Point2D::new(0.0004, 0.0004),
            &stops,
        )
        .unwrap_err();
        let stop_error = validate_linear_gradient(
            bounds,
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 1.0),
            &[
                GradientStop::new(0.0, (0, 0, 0)),
                GradientStop::new(0.000_000_4, (127, 127, 127)),
                GradientStop::new(1.0, (255, 255, 255)),
            ],
        )
        .unwrap_err();

        assert!(endpoint_error.to_string().contains("endpoints collapse"));
        assert!(stop_error.to_string().contains("stop offsets collapse"));
    }

    #[test]
    fn rounded_rectangle_validation_covers_geometry_and_radius_bounds() {
        assert!(validate_rounded_rect(1.0, 2.0, 8.0, 6.0, 3.0).is_ok());
        assert!(validate_rounded_rect(1.0, 2.0, 0.0, 0.0, 0.0).is_ok());

        assert!(validate_rounded_rect(f64::NAN, 2.0, 8.0, 6.0, 2.0).is_err());
        assert!(validate_rounded_rect(1.0, 2.0, -8.0, 6.0, 2.0).is_err());
        assert!(validate_rounded_rect(1.0, 2.0, 8.0, -6.0, 2.0).is_err());
        assert!(validate_rounded_rect(1.0, 2.0, 8.0, 6.0, -2.0).is_err());
        assert!(validate_rounded_rect(f64::MAX, 2.0, f64::MAX, 6.0, 2.0).is_err());
        assert!(validate_rounded_rect(1.0, f64::MAX, 8.0, f64::MAX, 2.0).is_err());
        assert!(validate_rounded_rect(1.0, 2.0, 8.0, 6.0, 4.0).is_err());
        assert!(validate_rounded_rect(1.0, 2.0, 8.0, 6.0, 3.1).is_err());
    }

    #[test]
    fn rounded_rectangle_conversion_rejects_collapsed_positive_extent() {
        let error = validate_rounded_rect(0.0, 0.0, 0.0004, 1.0, 0.0).unwrap_err();

        assert!(error.to_string().contains("positive extent collapses"));
    }

    #[test]
    fn significance_bracket_validation_rejects_invalid_anchors_and_styles() {
        let (bracket, style) = valid_bracket();
        assert!(validate_bracket(&bracket, &style).is_ok());

        let mut invalid_bracket = bracket.clone();
        invalid_bracket.y = f64::NAN;
        assert!(validate_bracket(&invalid_bracket, &style).is_err());

        let mut invalid_bracket = bracket.clone();
        invalid_bracket.x_right = invalid_bracket.x_left;
        assert!(validate_bracket(&invalid_bracket, &style).is_err());

        let mut invalid_style = style.clone();
        invalid_style.width = f32::NAN;
        assert!(validate_bracket(&bracket, &invalid_style).is_err());

        let mut invalid_style = style.clone();
        invalid_style.width = -1.0;
        assert!(validate_bracket(&bracket, &invalid_style).is_err());

        let mut invalid_style = style.clone();
        invalid_style.font_size = 0;
        assert!(validate_bracket(&bracket, &invalid_style).is_err());

        let mut invalid_style = style.clone();
        invalid_style.line_height = 0.0;
        assert!(validate_bracket(&bracket, &invalid_style).is_err());

        let mut invalid_style = style.clone();
        invalid_style.line_height = f64::NAN;
        assert!(validate_bracket(&bracket, &invalid_style).is_err());

        let mut invalid_style = style.clone();
        invalid_style.label_gap = f64::NAN;
        assert!(validate_bracket(&bracket, &invalid_style).is_err());

        let mut invalid_style = style;
        invalid_style.label_gap = -1.0;
        assert!(validate_bracket(&bracket, &invalid_style).is_err());
    }

    #[test]
    fn significance_bracket_conversion_rejects_collapsed_anchors() {
        let (mut bracket, style) = valid_bracket();
        bracket.x_left = 0.0;
        bracket.x_right = 0.0004;

        let error = validate_bracket(&bracket, &style).unwrap_err();

        assert!(error.to_string().contains("anchors collapse"));
    }

    #[test]
    fn significance_bracket_conversion_retains_subprecision_anchor_span() {
        let (mut bracket, style) = valid_bracket();
        bracket.x_left = 0.0;
        bracket.x_right = 0.0006;

        let canonical = validate_bracket(&bracket, &style).unwrap();

        assert_eq!(canonical.line_end.x.units(), 1);
    }

    #[test]
    fn scope_cleanup_errors_preserve_primary_context() {
        let cleanup_only = finish_scope::<()>(
            Ok(()),
            Err(Error::Render("cleanup failed".into())),
            "test scope",
        )
        .unwrap_err()
        .to_string();
        assert!(cleanup_only.contains("failed to close test scope"));
        assert!(cleanup_only.contains("cleanup failed"));

        let combined = finish_scope::<()>(
            Err(Error::Render("primary failed".into())),
            Err(Error::Render("cleanup failed".into())),
            "test scope",
        )
        .unwrap_err()
        .to_string();
        assert!(combined.contains("primary failed"));
        assert!(combined.contains("additionally failed to close test scope"));
        assert!(combined.contains("cleanup failed"));
    }
}
