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
///
/// An explicit default is 16 CSS pixels, Normal weight and an 8px gap. Omitting
/// the style lets the owning theme supply those properties instead.
/// ```
/// use lichtung_plot::{SignificanceMarkStyle, TextWeight};
/// let style = SignificanceMarkStyle::default()
///     .font_size(18)?.weight(TextWeight::Semibold)?.gap(9.0)?;
/// assert_eq!(style.get_font_size(), 18);
/// assert_eq!(style.get_weight(), TextWeight::Semibold);
/// assert_eq!(style.get_gap(), 9.0);
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
///
/// Private fields prevent bypassing complete construction validation:
/// ```compile_fail
/// use lichtung_plot::SignificanceMarkStyle;
/// let mut style = SignificanceMarkStyle::default();
/// style.gap = f64::NAN;
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignificanceMarkStyle {
    font_size: u32,
    weight: TextWeight,
    gap: f64,
}

impl SignificanceMarkStyle {
    /// Built-in values for an explicitly constructed default exact style.
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

    /// Replace the positive mark font size in CSS pixels.
    pub fn font_size(mut self, font_size: u32) -> Result<Self, Error> {
        self.font_size = font_size;
        self.validate()
    }

    /// Replace the mark font weight.
    pub fn weight(mut self, weight: TextWeight) -> Result<Self, Error> {
        self.weight = weight;
        self.validate()
    }

    /// Replace the finite nonnegative mark-to-data gap in CSS pixels.
    pub fn gap(mut self, gap: f64) -> Result<Self, Error> {
        self.gap = gap;
        self.validate()
    }

    fn validate(mut self) -> Result<Self, Error> {
        if self.font_size == 0 || !self.gap.is_finite() || self.gap < 0.0 {
            return Err(Error::Data(
                "significance mark style requires positive font size and finite nonnegative gap"
                    .into(),
            ));
        }
        if self.gap == 0.0 {
            self.gap = 0.0;
        }
        Ok(self)
    }

    /// Inspect the exact positive font size in CSS pixels.
    pub const fn get_font_size(&self) -> u32 {
        self.font_size
    }

    /// Inspect the exact font weight.
    pub const fn get_weight(&self) -> TextWeight {
        self.weight
    }

    /// Inspect the exact finite nonnegative gap in CSS pixels.
    pub const fn get_gap(&self) -> f64 {
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
}

/// One exact point and connector style owned by an interaction group.
///
/// ```
/// use lichtung_plot::{InteractionStyle, LinePattern, PointShape};
/// let style = InteractionStyle::new("Control")?
///     .point_shape(PointShape::TriangleUp)?
///     .line_pattern(LinePattern::Dashed)?;
/// assert_eq!(style.group(), "Control");
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
///
/// ```compile_fail
/// use lichtung_plot::{InteractionStyle, PointShape};
/// let _ = InteractionStyle::new("Control").unwrap().shape(PointShape::TriangleUp);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionStyle {
    group: String,
    shape: PointShape,
    pattern: LinePattern,
}

impl InteractionStyle {
    /// Construct a circle-and-solid style for one exact graphical group label.
    ///
    /// The raw selector must be nonempty, already trimmed and control-free.
    /// Its bytes, including interior whitespace, are retained for exact lookup.
    pub fn new(group: impl Into<String>) -> Result<Self, Error> {
        Self {
            group: group.into(),
            shape: PointShape::default(),
            pattern: LinePattern::default(),
        }
        .validate()
    }

    /// Select the point shape for this group.
    pub fn point_shape(mut self, shape: PointShape) -> Result<Self, Error> {
        self.shape = shape;
        self.validate()
    }

    /// Select the connector pattern for this group.
    pub fn line_pattern(mut self, pattern: LinePattern) -> Result<Self, Error> {
        self.pattern = pattern;
        self.validate()
    }

    fn validate(self) -> Result<Self, Error> {
        if self.group.is_empty()
            || self.group.trim() != self.group
            || self.group.chars().any(char::is_control)
        {
            return Err(Error::Data(
                "interaction style group must be nonempty, already trimmed, and contain no control characters".into(),
            ));
        }
        Ok(self)
    }

    /// Exact graphical group label that owns this style.
    #[must_use]
    pub fn group(&self) -> &str {
        &self.group
    }

    /// Configured point shape.
    #[must_use]
    pub const fn get_point_shape(&self) -> PointShape {
        self.shape
    }

    /// Configured connector pattern.
    #[must_use]
    pub const fn get_line_pattern(&self) -> LinePattern {
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
