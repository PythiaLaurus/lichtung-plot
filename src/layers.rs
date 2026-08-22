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

/// A complete strict point layer owned independently from the plot input.
///
/// Every source row contributes exactly one unconnected point. The layer has
/// one fixed RGB color, radius, and transparent opacity and creates no legend.
#[derive(Debug, Clone)]
pub struct PointLayer {
    x: NumericValues,
    y: NumericValues,
    labels: Option<CategoryValues>,
    legend_label: Option<String>,
    color: (u8, u8, u8),
    radius: u32,
    opacity: f64,
}

/// A complete strict point layer positioned at prepared ordinary-bar centers.
///
/// Category and optional graphical-group values resolve against the bar cells;
/// hue is never used as dodge identity. Every row creates exactly one point and
/// the layer creates no legend.
#[derive(Debug, Clone)]
pub struct BarPointLayer {
    category: CategoryValues,
    group: Option<CategoryValues>,
    value: NumericValues,
    color: (u8, u8, u8),
    radius: u32,
    opacity: f64,
}

impl BarPointLayer {
    /// Construct an owned grouped or ungrouped bar point layer.
    ///
    /// Pass `None` for an ungrouped bar and `Some(column)` for a chart whose
    /// ordinary bars use an explicit graphical group.
    pub fn new(
        category: CategoryValues,
        group: Option<CategoryValues>,
        value: NumericValues,
        color: (u8, u8, u8),
        radius: u32,
        opacity: f64,
    ) -> Result<Self, Error> {
        validate_point_style("bar-point-layer", radius, opacity)?;
        Ok(Self {
            category,
            group,
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

    /// Optional graphical-group values resolved against prepared bar groups.
    #[must_use]
    pub const fn group_values(&self) -> Option<&CategoryValues> {
        self.group.as_ref()
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

    pub(crate) fn point_style(&self) -> crate::render::backend::PointStyle {
        crate::render::backend::PointStyle::plain(self.color, self.radius)
            .with_opacity(self.opacity)
    }
}

impl PointLayer {
    /// Construct an owned point layer with fixed styling.
    ///
    /// Column existence, numeric dtypes, and every strict finite row are
    /// validated during chart preflight before output publication.
    ///
    /// # Errors
    ///
    /// Returns an error when `radius` is zero or `opacity` is not finite and
    /// strictly between zero and one.
    pub fn new(
        x: NumericValues,
        y: NumericValues,
        color: (u8, u8, u8),
        radius: u32,
        opacity: f64,
    ) -> Result<Self, Error> {
        validate_point_style("point-layer", radius, opacity)?;
        Ok(Self {
            x,
            y,
            labels: None,
            legend_label: None,
            color,
            radius,
            opacity,
        })
    }

    /// Attach one caller-owned label to every point.
    #[must_use]
    pub fn labels(mut self, labels: CategoryValues) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Give the layer one legend identity.
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

    pub(crate) fn point_style(&self) -> crate::render::backend::PointStyle {
        crate::render::backend::PointStyle::plain(self.color, self.radius)
            .with_opacity(self.opacity)
    }
}

fn validate_point_style(role: &str, radius: u32, opacity: f64) -> Result<(), Error> {
    if radius == 0 {
        return Err(Error::Data(format!(
            "{role} radius must be greater than zero"
        )));
    }
    if !opacity.is_finite() || !(0.0..1.0).contains(&opacity) {
        return Err(Error::Data(format!(
            "{role} opacity must be finite and strictly between zero and one; got {opacity}"
        )));
    }
    Ok(())
}
