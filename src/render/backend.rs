//! `lichtung-plot::render::backend` — the `Backend` trait.
//!
//! A `Backend` implementation draws a chart onto some 2D surface
//! (SVG file, PNG bitmap, in-memory image). The sole Backend impl
//! lives in `svg_backend.rs` and emits SVG directly. PNG output is
//! produced by rasterizing the SVG through `resvg` (see the `png`
//! Cargo feature). Future backends (skia, vello) can drop in behind
//! the same trait.

#![deny(missing_docs)]

use crate::theme::{color_from_rgb, Color};
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

/// Quantize a validated floating-opacity input to one canonical byte alpha.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "finite validated unit opacity rounds to an integral value in 0..=255"
)]
pub(crate) fn quantize_legacy_opacity(opacity: f64) -> Result<u8, Error> {
    if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
        return Err(Error::Render(format!(
            "paint opacity must be finite and in [0, 1]; got {opacity}"
        )));
    }
    Ok((opacity * 255.0).round_ties_even() as u8)
}

/// Visual style for a scatter point.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PointStyle {
    /// Complete point fill paint.
    pub(crate) color: Color,
    /// Radius in pixels.
    pub(crate) radius: u32,
    /// Optional outline stroke.
    pub(crate) outline: Option<PointOutline>,
    /// Marker geometry.
    pub(crate) shape: PointShape,
}

impl PointStyle {
    /// Construct a backend fixture with the default publication point outline.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn new(color: (u8, u8, u8), radius: u32) -> Self {
        Self {
            color: color_from_rgb(color).with_alpha(209),
            radius,
            outline: Some(PointOutline::publication().unwrap()),
            shape: PointShape::Circle,
        }
    }

    /// Construct a filled point without an outline.
    #[must_use]
    pub(crate) fn plain(color: (u8, u8, u8), radius: u32) -> Self {
        Self {
            color: color_from_rgb(color).with_alpha(209),
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
        self.color = self.color.with_alpha(quantize_legacy_opacity(opacity)?);
        Ok(self)
    }
}

/// Outline stroke for scatter-like point marks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PointOutline {
    /// Independent complete outline paint.
    pub(crate) color: Color,
    /// Stroke width as a proportion of the point radius.
    pub(crate) width_ratio: crate::theme::RelativeSpacing,
}

impl PointOutline {
    /// Construct the validated publication outline.
    #[cfg(test)]
    pub(crate) fn publication() -> Result<Self, Error> {
        Ok(crate::theme::PointOutlineTheme::publication()?.backend_style())
    }

    /// Resolve the outline stroke width for the given point radius.
    pub(crate) fn width_for_radius(
        &self,
        radius: u32,
    ) -> Result<coordinates::CanonicalCoordinate, Error> {
        let width = self
            .width_ratio
            .product(crate::theme::LocalMetric::css_pixels(radius)?)
            .round(crate::layout::kernel::SpatialRounding::NearestEven)?
            .quanta();
        if width > (1_i64 << 53) - 1 || (width == 0 && !self.width_ratio.is_zero()) {
            return Err(Error::Render("point outline width exceeds canonical coordinates or its positive extent collapses".into()));
        }
        Ok(coordinates::CanonicalCoordinate::from_quanta(width))
    }
}

/// Visual style for a line (used by regression fits, axis lines).
#[derive(Debug, Clone, PartialEq)]
pub struct LineStyle {
    /// Complete stroke paint.
    pub(crate) color: Color,
    /// Stroke width in pixels (supports sub-pixel, e.g. 0.8).
    pub width: f32,
    /// Closed stroke pattern.
    pub pattern: StrokePattern,
}

impl LineStyle {
    /// Create a solid line style.
    #[must_use]
    #[cfg(test)]
    pub(crate) fn solid(color: Color, width: f32) -> Self {
        Self {
            color,
            width,
            pattern: StrokePattern::Solid,
        }
    }

    /// Create a line using the fixed `3 2` dash pattern.
    #[must_use]
    #[cfg(test)]
    pub(crate) fn short_dashed(color: Color, width: f32) -> Self {
        Self {
            color,
            width,
            pattern: StrokePattern::ShortDash,
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
    pub(crate) color: Color,
    pub(crate) stroke: Color,
    /// Stroke width in pixels (0.0 for no outline).
    pub(crate) stroke_width: f32,
}

impl FillStyle {
    pub(crate) const fn new(color: Color, stroke: Color, stroke_width: f32) -> Self {
        // Width zero is the operational encoding of structural absence. Its
        // unused paint cannot become guide/cache/feedback identity.
        if stroke_width == 0.0 {
            return Self {
                color,
                stroke: Color::rgba(0, 0, 0, 0),
                stroke_width: 0.0,
            };
        }
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
    pub(crate) color: Color,
    /// Font family.
    pub(crate) font_family: crate::font::canonical::FontStack,
    /// Font size in actual CSS pixels.
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

pub(crate) const POLYGON_MITER_LIMIT: f64 = 4.0;

/// Visual style for a significance bracket.
#[derive(Debug, Clone)]
pub struct BracketStyle {
    /// Complete bracket-path stroke paint.
    pub(crate) color: Color,
    /// Complete bracket-label text paint.
    pub(crate) label_color: Color,
    /// Stroke width in pixels.
    pub width: f32,
    /// Label font family.
    pub(crate) font_family: crate::font::canonical::FontStack,
    /// Label font size in pixels.
    pub font_size: u32,
    /// Label line height in pixels.
    pub(crate) line_height: f64,
    /// Label font weight.
    pub weight: crate::TextWeight,
    /// Gap between the bracket line and its label.
    pub label_gap: f64,
    /// Retained selected tick height; canonical validation has no default.
    pub(crate) tick_height: f64,
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
) -> Result<(), Error> {
    if additional > records.capacity() - records.len() {
        let capacity = records
            .len()
            .checked_add(additional)
            .zip(records.capacity().checked_mul(2))
            .map(|(required, doubled)| required.max(doubled).max(4))
            .ok_or_else(|| Error::Render("producer text capacity overflow".into()))?;

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
        _paint: Color,
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
        paint: Color,
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
            &planned_text_style(placement, plan)?,
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
    if style.outline.is_some() && style.radius == 0 {
        return Err(Error::Render(
            "outlined point radius must be positive".into(),
        ));
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
) -> Result<TextStyle, Error> {
    Ok(TextStyle {
        color: plan.style.paint,
        font_family: plan
            .copy_stack()
            .map_err(|error| Error::Render(error.to_string()))?,
        font_size: plan.style.font_size,
        weight: plan.style.weight,
        anchor: placement.anchor,
        rotation: placement.rotation,
    })
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
        || !style.tick_height.is_finite()
        || style.tick_height < 0.0
    {
        return Err(Error::Render(
            "bracket style requires positive font size and line height and finite nonnegative gap and tick height"
                .into(),
        ));
    }
    coordinates::canonical_bracket(bracket, style.tick_height, style.label_gap)
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
        || !plan.same_stack(&call.style.font_family)
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
