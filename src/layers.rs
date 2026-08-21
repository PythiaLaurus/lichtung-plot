//! Typed supplemental geometry for numeric charts.

use crate::Error;
use polars::prelude::DataFrame;

use crate::LinePattern;

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

/// A complete strict point layer borrowed independently from the chart data.
///
/// Every source row contributes exactly one unconnected point. The layer has
/// one fixed RGB color, radius, and transparent opacity and creates no legend.
#[derive(Debug, Clone)]
pub struct PointLayer<'a> {
    data: &'a DataFrame,
    x_column: &'a str,
    y_column: &'a str,
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
pub struct BarPointLayer<'a> {
    data: &'a DataFrame,
    category_column: &'a str,
    group_column: Option<&'a str>,
    value_column: &'a str,
    color: (u8, u8, u8),
    radius: u32,
    opacity: f64,
}

impl<'a> BarPointLayer<'a> {
    /// Construct a borrowed grouped or ungrouped bar point layer.
    ///
    /// Pass `None` for an ungrouped bar and `Some(column)` for a chart whose
    /// ordinary bars use an explicit graphical group.
    pub fn new(
        data: &'a DataFrame,
        category_column: &'a str,
        group_column: Option<&'a str>,
        value_column: &'a str,
        color: (u8, u8, u8),
        radius: u32,
        opacity: f64,
    ) -> Result<Self, Error> {
        validate_point_style("bar-point-layer", radius, opacity)?;
        Ok(Self {
            data,
            category_column,
            group_column,
            value_column,
            color,
            radius,
            opacity,
        })
    }

    /// Borrow the layer's source frame.
    #[must_use]
    pub const fn data(&self) -> &'a DataFrame {
        self.data
    }

    /// Category column resolved against prepared bar categories.
    #[must_use]
    pub const fn category_column(&self) -> &'a str {
        self.category_column
    }

    /// Optional graphical-group column resolved against prepared bar groups.
    #[must_use]
    pub const fn group_column(&self) -> Option<&'a str> {
        self.group_column
    }

    /// Strict finite numeric point-value column.
    #[must_use]
    pub const fn value_column(&self) -> &'a str {
        self.value_column
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

impl<'a> PointLayer<'a> {
    /// Construct a borrowed point layer with fixed styling.
    ///
    /// Column existence, numeric dtypes, and every strict finite row are
    /// validated during chart preflight before output publication.
    ///
    /// # Errors
    ///
    /// Returns an error when `radius` is zero or `opacity` is not finite and
    /// strictly between zero and one.
    pub fn new(
        data: &'a DataFrame,
        x_column: &'a str,
        y_column: &'a str,
        color: (u8, u8, u8),
        radius: u32,
        opacity: f64,
    ) -> Result<Self, Error> {
        validate_point_style("point-layer", radius, opacity)?;
        Ok(Self {
            data,
            x_column,
            y_column,
            color,
            radius,
            opacity,
        })
    }

    pub(crate) const fn data(&self) -> &'a DataFrame {
        self.data
    }

    pub(crate) const fn x_column(&self) -> &'a str {
        self.x_column
    }

    pub(crate) const fn y_column(&self) -> &'a str {
        self.y_column
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
