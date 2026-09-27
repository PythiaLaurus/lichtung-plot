//! Typed supplemental geometry for numeric charts.

use crate::Error;

use crate::LinePattern;
use crate::{CategoryValues, NumericValues};

/// Closed visual style for a numeric guide line.
///
/// The pattern is selected through [`LinePattern`]; arbitrary backend dash
/// strings are not accepted.
#[derive(Debug, Clone, PartialEq)]
pub struct GuideStyle {
    color: (u8, u8, u8),
    width: f32,
    pattern: LinePattern,
}

impl GuideStyle {
    /// Construct a guide style with a fixed RGB color, positive width, and
    /// closed line pattern.
    ///
    /// # Errors
    ///
    /// Returns an error when `width` is not finite and strictly positive.
    pub fn new(color: (u8, u8, u8), width: f32, pattern: LinePattern) -> Result<Self, Error> {
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

    pub(crate) fn line_style(&self) -> crate::render::backend::LineStyle {
        self.pattern.line_style(self.color, self.width)
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
#[derive(Debug, Clone)]
pub struct PointLayer {
    x: NumericValues,
    y: NumericValues,
    labels: Option<CategoryValues>,
    legend_label: Option<String>,
    color: Option<(u8, u8, u8)>,
    radius: u32,
    opacity: f64,
}

/// A complete strict point layer positioned at prepared ordinary-bar centers.
///
/// Category and optional series values resolve against the bar cells. Every row
/// creates exactly one point and the layer creates no legend.
#[derive(Debug, Clone)]
pub struct BarPointLayer {
    category: CategoryValues,
    series: Option<CategoryValues>,
    value: NumericValues,
    color: (u8, u8, u8),
    radius: u32,
    opacity: f64,
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
        color: (u8, u8, u8),
        radius: u32,
        opacity: f64,
    ) -> Result<Self, Error> {
        validate_point_style("bar-point-layer", radius, opacity)?;
        Ok(Self {
            category,
            series,
            value,
            color,
            radius,
            opacity,
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

    /// Fixed RGB point color.
    #[must_use]
    pub const fn color(&self) -> (u8, u8, u8) {
        self.color
    }

    /// Fixed positive point radius.
    #[must_use]
    pub const fn radius(&self) -> u32 {
        self.radius
    }

    /// Fixed opacity strictly between zero and one.
    #[must_use]
    pub const fn opacity(&self) -> f64 {
        self.opacity
    }

    pub(crate) fn point_style(&self) -> Result<crate::render::backend::PointStyle, Error> {
        crate::render::backend::PointStyle::plain(self.color, self.radius)
            .with_opacity(self.opacity)
    }
}

impl PointLayer {
    /// Construct an owned point layer with renderer-owned styling.
    ///
    /// The x/y values may describe raw observations or any other supplemental
    /// coordinates and need not align with the primary line input. They default
    /// to the next theme-coordinated color, radius 4, and opacity 0.8. Nonempty,
    /// aligned, finite values are validated during chart preflight.
    #[must_use]
    pub fn new(x: NumericValues, y: NumericValues) -> Self {
        Self {
            x,
            y,
            labels: None,
            legend_label: None,
            color: None,
            radius: 4,
            opacity: 0.8,
        }
    }

    /// Override the automatic theme color with one fixed RGB color.
    #[must_use]
    pub fn color(mut self, color: (u8, u8, u8)) -> Self {
        self.color = Some(color);
        self
    }

    /// Override the default point radius.
    ///
    /// # Errors
    ///
    /// Returns an error when `radius` is zero.
    pub fn radius(mut self, radius: u32) -> Result<Self, Error> {
        validate_point_radius("point-layer", radius)?;
        self.radius = radius;
        Ok(self)
    }

    /// Override the default point opacity.
    ///
    /// # Errors
    ///
    /// Returns an error unless `opacity` is finite and in `(0, 1]`.
    pub fn opacity(mut self, opacity: f64) -> Result<Self, Error> {
        if !opacity.is_finite() || opacity <= 0.0 || opacity > 1.0 {
            return Err(Error::Data(format!(
                "point-layer opacity must be finite, greater than zero, and at most one; got {opacity}"
            )));
        }
        self.opacity = opacity;
        Ok(self)
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
    ) -> Result<crate::render::backend::PointStyle, Error> {
        crate::render::backend::PointStyle::plain(
            self.color.unwrap_or(automatic_color),
            self.radius,
        )
        .with_opacity(self.opacity)
    }
}

fn validate_point_style(role: &str, radius: u32, opacity: f64) -> Result<(), Error> {
    validate_point_radius(role, radius)?;
    if !opacity.is_finite() || !(0.0..1.0).contains(&opacity) {
        return Err(Error::Data(format!(
            "{role} opacity must be finite and strictly between zero and one; got {opacity}"
        )));
    }
    Ok(())
}

fn validate_point_radius(role: &str, radius: u32) -> Result<(), Error> {
    if radius == 0 {
        return Err(Error::Data(format!(
            "{role} radius must be greater than zero"
        )));
    }
    Ok(())
}
