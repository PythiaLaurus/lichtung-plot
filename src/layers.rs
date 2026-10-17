//! Typed supplemental geometry for numeric charts.

use crate::Error;

use crate::LinePattern;
use crate::{CategoryValues, Color, NumericValues};

/// Closed visual style for a numeric guide line.
///
/// The pattern is selected through [`LinePattern`]; arbitrary backend dash
/// strings are not accepted.
///
/// ```
/// use lichtung_plot::{Color, GuideStyle, LinePattern};
/// let guide = GuideStyle::new(Color::rgba(20, 80, 140, 128), 1.5, LinePattern::Solid)?
///     .with_pattern(LinePattern::Dashed)?;
/// assert_eq!(guide.color().alpha(), 128);
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
///
/// Exact paint is typed; RGB tuples are not style inputs:
/// ```compile_fail
/// use lichtung_plot::{GuideStyle, LinePattern};
/// let _ = GuideStyle::new((20, 80, 140), 1.5, LinePattern::Solid);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GuideStyle {
    color: Color,
    width: f32,
    pattern: LinePattern,
}

impl GuideStyle {
    /// Construct a guide style with a complete RGBA paint, positive width, and
    /// closed line pattern.
    ///
    /// # Errors
    ///
    /// Returns an error when `width` is not finite and strictly positive.
    pub fn new(color: Color, width: f32, pattern: LinePattern) -> Result<Self, Error> {
        if !width.is_finite() || width <= 0.0 {
            return Err(Error::Data(format!(
                "guide width must be finite and strictly positive; got {width}"
            )));
        }
        Ok(Self {
            color,
            width,
            pattern,
        })
    }

    /// Inspect the complete exact guide paint.
    pub const fn color(&self) -> Color {
        self.color
    }

    /// Inspect the positive guide width in CSS pixels.
    pub const fn width(&self) -> f32 {
        self.width
    }

    /// Inspect the closed guide pattern.
    pub const fn pattern(&self) -> LinePattern {
        self.pattern
    }

    /// Replace the complete exact guide paint.
    pub fn with_color(self, color: Color) -> Result<Self, Error> {
        Self::new(color, self.width, self.pattern)
    }

    /// Replace the positive guide width in CSS pixels.
    pub fn with_width(self, width: f32) -> Result<Self, Error> {
        Self::new(self.color, width, self.pattern)
    }

    /// Replace the closed guide pattern.
    pub fn with_pattern(self, pattern: LinePattern) -> Result<Self, Error> {
        Self::new(self.color, self.width, pattern)
    }

    pub(crate) fn line_style(&self) -> crate::render::backend::LineStyle {
        crate::render::backend::LineStyle {
            color: self.color,
            width: self.width,
            pattern: self.pattern.stroke_pattern(),
        }
    }
}

/// Supplemental numeric guide geometry.
///
/// Horizontal and vertical guides span the resolved panel range. A segment is
/// finite and may be diagonal. Every coordinate must lie in the corresponding
/// resolved numeric axis range.
#[derive(Debug, Clone, PartialEq)]
pub enum GuideSpec {
    /// A horizontal guide spanning the numeric x range.
    Horizontal {
        /// Numeric y position.
        y: f64,
        /// Guide visual style.
        style: GuideStyle,
    },
    /// A vertical guide spanning the numeric y range.
    Vertical {
        /// Numeric x position.
        x: f64,
        /// Guide visual style.
        style: GuideStyle,
    },
    /// A finite segment between two numeric data-space endpoints.
    Segment {
        /// First `(x, y)` endpoint.
        start: (f64, f64),
        /// Second `(x, y)` endpoint.
        end: (f64, f64),
        /// Guide visual style.
        style: GuideStyle,
    },
}

impl GuideSpec {
    /// Construct a horizontal guide.
    #[must_use]
    pub const fn horizontal(y: f64, style: GuideStyle) -> Self {
        Self::Horizontal { y, style }
    }

    /// Construct a vertical guide.
    #[must_use]
    pub const fn vertical(x: f64, style: GuideStyle) -> Self {
        Self::Vertical { x, style }
    }

    /// Construct a finite segment, including a diagonal guide when both
    /// coordinates differ between endpoints.
    #[must_use]
    pub const fn segment(
        x_start: f64,
        y_start: f64,
        x_end: f64,
        y_end: f64,
        style: GuideStyle,
    ) -> Self {
        Self::Segment {
            start: (x_start, y_start),
            end: (x_end, y_end),
            style,
        }
    }
}

/// A strict supplemental point layer owned independently from the primary line.
///
/// The aligned x/y values are arbitrary caller-selected coordinates. Optional
/// labels annotate individual points, while an optional legend label identifies
/// the complete layer once.
///
/// ```
/// use lichtung_plot::{Color, NumericValues, PointLayer};
/// let layer = PointLayer::new(
///     NumericValues::new("x", [1.0]), NumericValues::new("y", [2.0]),
/// ).with_alpha(128).with_color(Color::rgb(20, 80, 140)).with_radius(5)?;
/// assert_eq!(layer.alpha(), Some(128));
/// assert_eq!(layer.color(), Some(Color::rgb(20, 80, 140)));
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
///
/// Alpha instructions are exact bytes rather than floating opacity:
/// ```compile_fail
/// use lichtung_plot::{NumericValues, PointLayer};
/// let _ = PointLayer::new(
///     NumericValues::new("x", [1.0]), NumericValues::new("y", [2.0]),
/// ).opacity(0.5);
/// ```
#[derive(Debug, Clone)]
pub struct PointLayer {
    x: NumericValues,
    y: NumericValues,
    labels: Option<CategoryValues>,
    legend_label: Option<String>,
    color: Option<Color>,
    radius: Option<u32>,
    alpha: Option<u8>,
}

/// A complete strict point layer positioned at prepared ordinary-bar centers.
///
/// Category and optional series values resolve against the bar cells. Every row
/// creates exactly one point and the layer creates no legend.
///
/// The constructor's Color owns the complete paint, with no opacity argument:
/// ```
/// use lichtung_plot::{BarPointLayer, CategoryValues, Color, NumericValues};
/// let layer = BarPointLayer::new(
///     CategoryValues::new("category", ["A"])?, None,
///     NumericValues::new("value", [2.0]), Color::rgba(20, 80, 140, 0), 4,
/// )?;
/// assert_eq!(layer.color().alpha(), 0);
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{BarPointLayer, CategoryValues, Color, NumericValues};
/// let _ = BarPointLayer::new(
///     CategoryValues::new("category", ["A"]).unwrap(), None,
///     NumericValues::new("value", [2.0]), Color::rgb(20, 80, 140), 4, 0.5,
/// );
/// ```
#[derive(Debug, Clone)]
pub struct BarPointLayer {
    category: CategoryValues,
    series: Option<CategoryValues>,
    value: NumericValues,
    color: Color,
    radius: u32,
}

impl BarPointLayer {
    /// Construct an owned bar point layer with optional positioned series.
    ///
    /// Pass `None` for a single-series bar and `Some(column)` for a chart whose
    /// ordinary bars use an explicit positioned series.
    pub fn new(
        category: CategoryValues,
        series: Option<CategoryValues>,
        value: NumericValues,
        color: Color,
        radius: u32,
    ) -> Result<Self, Error> {
        validate_point_radius("bar-point-layer", radius)?;
        Ok(Self {
            category,
            series,
            value,
            color,
            radius,
        })
    }

    /// Categories resolved against prepared bar categories.
    #[must_use]
    pub const fn category_values(&self) -> &CategoryValues {
        &self.category
    }

    /// Optional series values resolved against prepared bar series.
    #[must_use]
    pub const fn series_values(&self) -> Option<&CategoryValues> {
        self.series.as_ref()
    }

    /// Strict finite numeric point values.
    #[must_use]
    pub const fn values(&self) -> &NumericValues {
        &self.value
    }

    /// Complete exact point paint, including alpha.
    #[must_use]
    pub const fn color(&self) -> Color {
        self.color
    }

    /// Fixed positive point radius.
    #[must_use]
    pub const fn radius(&self) -> u32 {
        self.radius
    }

    pub(crate) fn point_style(&self) -> crate::render::backend::PointStyle {
        crate::render::backend::PointStyle {
            color: self.color,
            radius: self.radius,
            outline: None,
            shape: crate::PointShape::Circle,
        }
    }
}

impl PointLayer {
    /// Construct an owned point layer with owner-theme styling.
    ///
    /// The x/y values may describe raw observations or any other supplemental
    /// coordinates and need not align with the primary line input. They default
    /// to the next theme-coordinated color and the theme's supplemental radius
    /// and alpha (built-ins use radius 4 and alpha 204). Nonempty,
    /// aligned, finite values are validated during chart preflight.
    #[must_use]
    pub fn new(x: NumericValues, y: NumericValues) -> Self {
        Self {
            x,
            y,
            labels: None,
            legend_label: None,
            color: None,
            radius: None,
            alpha: None,
        }
    }

    /// Replace the automatic mark paint and label paint with complete RGBA.
    /// An independent explicit alpha instruction takes precedence over this alpha.
    #[must_use]
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Override the default point radius.
    ///
    /// # Errors
    ///
    /// Returns an error when `radius` is zero.
    pub fn with_radius(mut self, radius: u32) -> Result<Self, Error> {
        validate_point_radius("point-layer", radius)?;
        self.radius = Some(radius);
        Ok(self)
    }

    /// Replace mark and label alpha, including transparent and opaque endpoints.
    ///
    /// This replaces alpha after resolving color, regardless of edit order.
    /// Without an exact color, label RGB stays owned by its theme text role.
    #[must_use]
    pub const fn with_alpha(mut self, alpha: u8) -> Self {
        self.alpha = Some(alpha);
        self
    }

    /// Inspect the optional caller paint instruction, before theme resolution.
    pub const fn color(&self) -> Option<Color> {
        self.color
    }

    /// Inspect the optional caller alpha instruction, before theme resolution.
    pub const fn alpha(&self) -> Option<u8> {
        self.alpha
    }

    /// Inspect the optional caller radius instruction, before theme resolution.
    pub const fn radius(&self) -> Option<u32> {
        self.radius
    }

    /// Attach one caller-owned annotation label to every point.
    ///
    /// Point labels do not create legend entries. They must align one-to-one
    /// with the layer's x/y coordinates.
    #[must_use]
    pub fn labels(mut self, labels: CategoryValues) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Give the complete layer one legend identity.
    ///
    /// This creates at most one legend entry and does not label individual
    /// points.
    #[must_use]
    pub fn legend_label(mut self, label: impl Into<String>) -> Self {
        self.legend_label = Some(label.into());
        self
    }

    pub(crate) const fn x_values(&self) -> &NumericValues {
        &self.x
    }

    pub(crate) const fn y_values(&self) -> &NumericValues {
        &self.y
    }

    pub(crate) const fn label_values(&self) -> Option<&CategoryValues> {
        self.labels.as_ref()
    }

    pub(crate) fn legend_label_text(&self) -> Option<&str> {
        self.legend_label.as_deref()
    }

    pub(crate) fn point_style(
        &self,
        automatic_color: (u8, u8, u8),
        defaults: &crate::theme::mark::SupplementalPointTheme,
    ) -> crate::render::backend::PointStyle {
        let color = self.color.unwrap_or_else(|| {
            crate::theme::color_from_rgb(automatic_color).with_alpha(defaults.fill_alpha())
        });
        crate::render::backend::PointStyle {
            color: self.alpha.map_or(color, |alpha| color.with_alpha(alpha)),
            radius: self.radius.unwrap_or(defaults.radius()),
            outline: None,
            shape: crate::PointShape::Circle,
        }
    }

    pub(crate) fn label_appearance(
        &self,
        role: &crate::theme::TextTheme,
    ) -> Result<crate::theme::TextTheme, Error> {
        let paint = self.color.unwrap_or(role.color());
        role.try_copy()?
            .with_color(self.alpha.map_or(paint, |alpha| paint.with_alpha(alpha)))
    }
}

fn validate_point_radius(role: &str, radius: u32) -> Result<(), Error> {
    if radius == 0 {
        return Err(Error::Data(format!(
            "{role} radius must be greater than zero"
        )));
    }
    Ok(())
}
