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

use crate::layout::PanelBox;
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

/// Visual style for a scatter point.
#[derive(Debug, Clone, PartialEq)]
pub struct PointStyle {
    /// Fill color (R, G, B).
    pub color: (u8, u8, u8),
    /// Radius in pixels.
    pub radius: u32,
    /// Optional outline stroke.
    pub outline: Option<PointOutline>,
    /// Marker geometry.
    pub shape: PointShape,
    /// Fill opacity in the inclusive range `[0, 1]`.
    pub opacity: f64,
}

impl PointStyle {
    /// Construct a filled point with the default publication outline.
    #[must_use]
    pub fn new(color: (u8, u8, u8), radius: u32) -> Self {
        Self {
            color,
            radius,
            outline: Some(PointOutline::default()),
            shape: PointShape::Circle,
            opacity: 0.82,
        }
    }

    /// Construct a filled point without an outline.
    #[must_use]
    pub fn plain(color: (u8, u8, u8), radius: u32) -> Self {
        Self {
            color,
            radius,
            outline: None,
            shape: PointShape::Circle,
            opacity: 0.82,
        }
    }

    /// Construct a filled point with an explicit outline setting.
    #[must_use]
    pub fn with_outline(color: (u8, u8, u8), radius: u32, outline: Option<PointOutline>) -> Self {
        Self {
            color,
            radius,
            outline,
            shape: PointShape::Circle,
            opacity: 0.82,
        }
    }

    /// Set the marker geometry.
    #[must_use]
    pub const fn with_shape(mut self, shape: PointShape) -> Self {
        self.shape = shape;
        self
    }

    /// Set the point fill opacity.
    #[must_use]
    pub const fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }
}

/// Outline stroke for scatter-like point marks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointOutline {
    /// Stroke color (R, G, B).
    pub color: (u8, u8, u8),
    /// Stroke width as a proportion of the point radius.
    pub width_ratio: f64,
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
            color: (255, 255, 255),
            width_ratio: 0.1,
        }
    }
}

/// Visual style for a line (used by regression fits, axis lines).
#[derive(Debug, Clone, PartialEq)]
pub struct LineStyle {
    /// Stroke color (R, G, B).
    pub color: (u8, u8, u8),
    /// Stroke width in pixels (supports sub-pixel, e.g. 0.8).
    pub width: f32,
    /// Closed stroke pattern.
    pub pattern: StrokePattern,
}

impl LineStyle {
    /// Create a solid line style.
    #[must_use]
    pub fn solid(color: (u8, u8, u8), width: f32) -> Self {
        Self {
            color,
            width,
            pattern: StrokePattern::Solid,
        }
    }

    /// Create a line using the fixed `3 2` dash pattern.
    #[must_use]
    pub(crate) fn short_dashed(color: (u8, u8, u8), width: f32) -> Self {
        Self {
            color,
            width,
            pattern: StrokePattern::ShortDash,
        }
    }

    /// Create a line using the fixed `4 3` dash pattern.
    #[must_use]
    pub(crate) fn medium_dashed(color: (u8, u8, u8), width: f32) -> Self {
        Self {
            color,
            width,
            pattern: StrokePattern::MediumDash,
        }
    }

    /// Create a line using the fixed `5 3` dash pattern.
    #[must_use]
    pub(crate) fn long_dashed(color: (u8, u8, u8), width: f32) -> Self {
        Self {
            color,
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

/// Visual style for a filled polygon (e.g. bar bodies, violin body).
#[derive(Debug, Clone)]
pub struct FillStyle {
    /// Fill color (R, G, B).
    pub color: (u8, u8, u8),
    /// Stroke color (R, G, B); some backends draw an outline.
    pub stroke: (u8, u8, u8),
    /// Stroke width in pixels (0.0 for no outline).
    pub stroke_width: f32,
    /// Fill opacity (0.0 = fully transparent, 1.0 = fully opaque).
    pub opacity: f64,
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
#[derive(Debug, Clone)]
pub struct TextStyle {
    /// Color (R, G, B).
    pub color: (u8, u8, u8),
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
pub(crate) struct BracketMetrics {
    pub(crate) tick_height: f64,
    pub(crate) label_gap: f64,
    pub(crate) label_font_size: u32,
    pub(crate) data_gap: f64,
    pub(crate) level_step: f64,
    pub(crate) panel_padding: f64,
}

pub(crate) const SIGNIFICANCE_BRACKET_METRICS: BracketMetrics = BracketMetrics {
    tick_height: 7.0,
    label_gap: 5.0,
    label_font_size: 12,
    data_gap: 6.0,
    level_step: 26.0,
    panel_padding: 2.0,
};

/// Visual style for a significance bracket.
#[derive(Debug, Clone)]
pub struct BracketStyle {
    /// Color (R, G, B).
    pub color: (u8, u8, u8),
    /// Stroke width in pixels.
    pub width: f32,
    /// Label font family.
    pub font_family: String,
    /// Label font size in pixels.
    pub font_size: u32,
    /// Label font weight.
    pub weight: crate::TextWeight,
    /// Gap between the bracket line and its label.
    pub label_gap: f64,
}

/// The `Backend` trait. Every concrete renderer (Plotters,
/// future Skia, future Vello) implements this.
///
/// All draw methods take a mutable `&mut self` and return
/// `Result<(), Error>`. Backends may buffer internally and
/// only emit output on `end()` (or `finalize()`).
///
/// Geometry passed to draw methods is already projected into math-style
/// canvas coordinates: X increases rightward and Y increases upward. Data
/// scaling and panel placement are renderer responsibilities, not backend
/// responsibilities.
pub trait Backend {
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

    /// Draw a text label at integer math-style canvas coordinates with
    /// alignment and optional rotation.
    fn draw_text(
        &mut self,
        x_pixel: u32,
        y_pixel: u32,
        text: &str,
        style: &TextStyle,
    ) -> Result<(), Error>;

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

    /// End the chart. Flushes any buffered output. After `end()`,
    /// no more draw methods may be called until the next `begin()`.
    fn end(&mut self) -> Result<(), Error>;
}

pub(crate) fn validate_clip_region(region: PanelBox) -> Result<(), Error> {
    let values = [region.x0, region.x1, region.y0, region.y1];
    if values.iter().any(|value| !value.is_finite()) {
        return Err(Error::Render(format!(
            "clip region must contain only finite coordinates; got {region:?}"
        )));
    }
    let width = region.width();
    let height = region.height();
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err(Error::Render(format!(
            "clip region must have finite positive width and height; got {region:?}"
        )));
    }
    Ok(())
}

pub(crate) fn validate_points(points: &[Point2D], context: &str) -> Result<(), Error> {
    for (index, point) in points.iter().enumerate() {
        if !point.x.is_finite() || !point.y.is_finite() {
            return Err(Error::Render(format!(
                "{context} point {index} must contain finite coordinates; got ({}, {})",
                point.x, point.y
            )));
        }
    }
    Ok(())
}

pub(crate) fn validate_point_style(style: &PointStyle) -> Result<(), Error> {
    if !style.opacity.is_finite() || !(0.0..=1.0).contains(&style.opacity) {
        return Err(Error::Render(format!(
            "point opacity must be finite and in [0, 1]; got {}",
            style.opacity
        )));
    }
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
    if !style.opacity.is_finite() || !(0.0..=1.0).contains(&style.opacity) {
        return Err(Error::Render(format!(
            "fill opacity must be finite and in [0, 1]; got {}",
            style.opacity
        )));
    }
    Ok(())
}

pub(crate) fn validate_linear_gradient(
    bounds: PanelBox,
    start: Point2D,
    end: Point2D,
    stops: &[GradientStop],
) -> Result<(), Error> {
    let bounds_values = [bounds.x0, bounds.x1, bounds.y0, bounds.y1];
    if bounds_values.iter().any(|value| !value.is_finite())
        || !bounds.width().is_finite()
        || !bounds.height().is_finite()
        || bounds.width() <= 0.0
        || bounds.height() <= 0.0
    {
        return Err(Error::Render(format!(
            "linear gradient bounds must have finite coordinates and positive width and height; got {bounds:?}"
        )));
    }
    validate_points(&[start, end], "linear gradient vector")?;
    if same_finite_coordinate(start.x, end.x) && same_finite_coordinate(start.y, end.y) {
        return Err(Error::Render(
            "linear gradient vector endpoints must be distinct".into(),
        ));
    }
    if stops.len() < 2 {
        return Err(Error::Render(format!(
            "linear gradient requires at least two stops; got {}",
            stops.len()
        )));
    }
    for (index, stop) in stops.iter().enumerate() {
        if !stop.offset.is_finite() || !(0.0..=1.0).contains(&stop.offset) {
            return Err(Error::Render(format!(
                "linear gradient stop {index} offset must be finite and in [0, 1]; got {}",
                stop.offset
            )));
        }
    }
    if stops[0].offset.to_bits() != 0.0_f64.to_bits()
        || stops[stops.len() - 1].offset.to_bits() != 1.0_f64.to_bits()
    {
        return Err(Error::Render(format!(
            "linear gradient stops must begin at 0 and end at 1; got {}..{}",
            stops[0].offset,
            stops[stops.len() - 1].offset
        )));
    }
    for (index, pair) in stops.windows(2).enumerate() {
        if pair[0].offset >= pair[1].offset {
            return Err(Error::Render(format!(
                "linear gradient stop offsets must be strictly increasing; stops {index} and {} are {} and {}",
                index + 1,
                pair[0].offset,
                pair[1].offset
            )));
        }
    }
    Ok(())
}

fn same_finite_coordinate(left: f64, right: f64) -> bool {
    const MAGNITUDE_BITS: u64 = 0x7fff_ffff_ffff_ffff;
    left.to_bits() == right.to_bits()
        || (left.to_bits() & MAGNITUDE_BITS == 0 && right.to_bits() & MAGNITUDE_BITS == 0)
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

pub(crate) fn validate_rounded_rect(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
) -> Result<(), Error> {
    if [x, y, width, height, radius]
        .iter()
        .any(|value| !value.is_finite())
    {
        return Err(Error::Render(format!(
            "rounded rectangle must contain finite geometry; got ({x}, {y}, {width}, {height}, {radius})"
        )));
    }
    if width < 0.0 || height < 0.0 || radius < 0.0 {
        return Err(Error::Render(format!(
            "rounded rectangle dimensions and radius must be non-negative; got ({width}, {height}, {radius})"
        )));
    }
    if !(x + width).is_finite() || !(y + height).is_finite() {
        return Err(Error::Render(format!(
            "rounded rectangle far edges must be finite; got ({x}, {y}, {width}, {height})"
        )));
    }
    if radius > width / 2.0 || radius > height / 2.0 {
        return Err(Error::Render(format!(
            "rounded rectangle radius must not exceed half either extent; got radius {radius} for {width}x{height}"
        )));
    }
    Ok(())
}

pub(crate) fn validate_bracket(
    bracket: &SignificanceBracket,
    style: &BracketStyle,
) -> Result<(), Error> {
    if [bracket.x_left, bracket.x_right, bracket.y]
        .iter()
        .any(|value| !value.is_finite())
    {
        return Err(Error::Render(format!(
            "significance bracket must contain finite coordinates; got ({}, {}, {})",
            bracket.x_left, bracket.x_right, bracket.y
        )));
    }
    if same_finite_coordinate(bracket.x_left, bracket.x_right) {
        return Err(Error::Render(
            "significance bracket must span two distinct rendered anchors".into(),
        ));
    }
    if !style.width.is_finite() || style.width < 0.0 {
        return Err(Error::Render(format!(
            "bracket width must be finite and non-negative; got {}",
            style.width
        )));
    }
    if style.font_size == 0 || !style.label_gap.is_finite() || style.label_gap < 0.0 {
        return Err(Error::Render(
            "bracket label style requires positive font size and finite nonnegative gap".into(),
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
                color: (0, 0, 0),
                width: 1.0,
                font_family: "sans-serif".into(),
                font_size: 12,
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
    fn primitive_point_validation_rejects_nonfinite_coordinates() {
        assert!(validate_points(&[Point2D::new(1.0, 2.0)], "test").is_ok());
        assert!(validate_points(&[Point2D::new(f64::NAN, 2.0)], "test").is_err());
        assert!(validate_points(&[Point2D::new(1.0, f64::INFINITY)], "test").is_err());
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
        invalid_style.label_gap = f64::NAN;
        assert!(validate_bracket(&bracket, &invalid_style).is_err());

        let mut invalid_style = style;
        invalid_style.label_gap = -1.0;
        assert!(validate_bracket(&bracket, &invalid_style).is_err());
    }
}
