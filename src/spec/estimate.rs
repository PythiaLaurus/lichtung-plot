//! Line, interaction, and estimate-family display policies.

use crate::{Error, NumericValues};

/// Explicit drawable lower and upper bounds.
#[derive(Debug, Clone)]
pub struct Bounds {
    lower: NumericValues,
    upper: NumericValues,
}

impl Bounds {
    /// Construct aligned drawable bounds.
    #[must_use]
    pub fn new(lower: NumericValues, upper: NumericValues) -> Self {
        Self { lower, upper }
    }

    /// Borrow the lower endpoints.
    #[must_use]
    pub const fn lower(&self) -> &NumericValues {
        &self.lower
    }

    /// Borrow the upper endpoints.
    #[must_use]
    pub const fn upper(&self) -> &NumericValues {
        &self.upper
    }
}

/// Closed owned input for a model-coefficient plot.
#[derive(Debug, Clone)]
pub struct CoefficientInput {
    values: NumericValues,
    bounds: Option<Bounds>,
}

impl CoefficientInput {
    /// Use one precomputed estimate per term without bounds.
    #[must_use]
    pub fn estimates(values: NumericValues) -> Self {
        Self {
            values,
            bounds: None,
        }
    }

    /// Use one precomputed estimate and explicit drawable bounds per term.
    #[must_use]
    pub fn estimates_with_intervals(values: NumericValues, bounds: Bounds) -> Self {
        Self {
            values,
            bounds: Some(bounds),
        }
    }

    pub(crate) const fn values(&self) -> &NumericValues {
        &self.values
    }

    pub(crate) const fn bounds(&self) -> Option<&Bounds> {
        self.bounds.as_ref()
    }
}

/// Closed text weights used by renderer-owned emphasis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextWeight {
    /// Ordinary body and axis text.
    #[default]
    Normal,
    /// Moderate emphasis.
    Semibold,
    /// Strong emphasis.
    Bold,
}

impl TextWeight {
    pub(crate) const fn svg_value(self) -> u16 {
        match self {
            Self::Normal => 400,
            Self::Semibold => 600,
            Self::Bold => 700,
        }
    }
}

/// Independent typography and clearance for exact significance marks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignificanceMarkStyle {
    font_size: u32,
    weight: TextWeight,
    gap: f64,
}

impl SignificanceMarkStyle {
    /// Default significance mark style used without an exact caller override.
    pub(crate) const DEFAULT: Self = Self {
        font_size: 16,
        weight: TextWeight::Normal,
        gap: 8.0,
    };

    /// Construct a validated mark style.
    pub fn new(font_size: u32, weight: TextWeight, gap: f64) -> Result<Self, Error> {
        Self {
            font_size,
            weight,
            gap,
        }
        .validate()
    }

    /// Set the positive mark font size.
    pub fn font_size(mut self, font_size: u32) -> Result<Self, Error> {
        self.font_size = font_size;
        self.validate()
    }

    /// Set the mark font weight.
    #[must_use]
    pub const fn weight(mut self, weight: TextWeight) -> Self {
        self.weight = weight;
        self
    }

    /// Set the finite nonnegative mark-to-data gap in pixels.
    pub fn gap(mut self, gap: f64) -> Result<Self, Error> {
        self.gap = gap;
        self.validate()
    }

    fn validate(self) -> Result<Self, Error> {
        if self.font_size == 0 || !self.gap.is_finite() || self.gap < 0.0 {
            return Err(Error::Data(
                "significance mark style requires positive font size and finite nonnegative gap"
                    .into(),
            ));
        }
        Ok(self)
    }

    pub(crate) const fn font_size_spec(self) -> u32 {
        self.font_size
    }

    pub(crate) const fn weight_spec(self) -> TextWeight {
        self.weight
    }

    pub(crate) const fn gap_spec(self) -> f64 {
        self.gap
    }
}

impl Default for SignificanceMarkStyle {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Confidence-band policy for a generated simple OLS regression plot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RegressionBand {
    /// Draw the classical pointwise mean-response Student-t 95% confidence band.
    #[default]
    PointwiseMean95,
    /// Draw only the descriptive OLS fit and make no interval claim.
    Hidden,
}

impl RegressionBand {
    pub(crate) const fn legend_label(self) -> &'static str {
        match self {
            Self::PointwiseMean95 => "Pointwise 95% Student-t mean band",
            Self::Hidden => "Descriptive OLS fit (no band)",
        }
    }
}

/// Reference-line policy for a normal Q-Q plot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QqLine {
    /// Draw the line through the theoretical and observed first and third quartiles.
    #[default]
    Quartiles,
    /// Draw the identity line `y = x`.
    Identity,
    /// Draw no reference line.
    Hidden,
}

/// Color and legend policy for source-ordered line paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinePathStyle {
    /// Retain neutral connectors and the existing observation-point behavior.
    #[default]
    Neutral,
    /// Color each complete path by its one categorical hue and emit line legend glyphs.
    Hue,
}

/// Closed point shapes accepted by interaction styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PointShape {
    /// Filled circular marker.
    #[default]
    Circle,
    /// Filled upward-pointing triangular marker.
    TriangleUp,
}

/// Closed connector patterns accepted by interaction styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinePattern {
    /// Continuous connector.
    #[default]
    Solid,
    /// Connector using the fixed publication dash array `5 3`.
    Dashed,
}

impl LinePattern {
    /// Exact caller connector grammar overrides the theme's default pattern.
    pub(crate) const fn stroke_pattern(self) -> crate::render::backend::StrokePattern {
        match self {
            Self::Solid => crate::render::backend::StrokePattern::Solid,
            Self::Dashed => crate::render::backend::StrokePattern::LongDash,
        }
    }

    pub(crate) fn line_style(
        self,
        color: (u8, u8, u8),
        width: f32,
    ) -> crate::render::backend::LineStyle {
        crate::render::backend::LineStyle {
            color: crate::theme::color_from_rgb(color),
            width,
            pattern: self.stroke_pattern(),
        }
    }
}

/// One exact point and connector style owned by an interaction group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionStyle {
    group: String,
    shape: PointShape,
    pattern: LinePattern,
}

impl InteractionStyle {
    /// Construct a circle-and-solid style for one exact graphical group label.
    #[must_use]
    pub fn new(group: impl Into<String>) -> Self {
        Self {
            group: group.into(),
            shape: PointShape::default(),
            pattern: LinePattern::default(),
        }
    }

    /// Select the point shape for this group.
    #[must_use]
    pub const fn shape(mut self, shape: PointShape) -> Self {
        self.shape = shape;
        self
    }

    /// Select the connector pattern for this group.
    #[must_use]
    pub const fn pattern(mut self, pattern: LinePattern) -> Self {
        self.pattern = pattern;
        self
    }

    /// Exact graphical group label that owns this style.
    #[must_use]
    pub fn group(&self) -> &str {
        &self.group
    }

    /// Configured point shape.
    #[must_use]
    pub const fn point_shape(&self) -> PointShape {
        self.shape
    }

    /// Configured connector pattern.
    #[must_use]
    pub const fn line_pattern(&self) -> LinePattern {
        self.pattern
    }
}

/// Typed availability of one aligned estimate row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstimateStatus {
    /// The row owns a complete finite estimate tuple.
    Estimated,
    /// The row owns no numeric estimate tuple and is displayed explicitly.
    NotEstimable,
}

/// Numeric scale used by interval-estimate plot families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AxisScale {
    /// Untransformed additive scale.
    #[default]
    Linear,
    /// Base-10 logarithmic scale for strictly positive multiplicative estimates.
    Log10,
}
