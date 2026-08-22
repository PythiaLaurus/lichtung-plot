//! Line, interaction, and estimate-family display policies.

use crate::Error;

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
    /// Construct a validated mark style.
    pub fn new(font_size: u32, weight: TextWeight, gap: f64) -> Result<Self, Error> {
        if font_size == 0 || !gap.is_finite() || gap < 0.0 {
            return Err(Error::Data(
                "significance mark style requires positive font size and finite nonnegative gap"
                    .into(),
            ));
        }
        Ok(Self {
            font_size,
            weight,
            gap,
        })
    }

    #[must_use]
    /// Return the mark font size.
    pub const fn font_size(self) -> u32 {
        self.font_size
    }

    #[must_use]
    /// Return the mark font weight.
    pub const fn weight(self) -> TextWeight {
        self.weight
    }

    #[must_use]
    /// Return the mark-to-data gap in pixels.
    pub const fn gap(self) -> f64 {
        self.gap
    }
}

impl Default for SignificanceMarkStyle {
    fn default() -> Self {
        Self {
            font_size: 16,
            weight: TextWeight::Normal,
            gap: 8.0,
        }
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
    /// Fixed plot-owned provenance represented in legends and included in reports.
    #[must_use]
    pub const fn provenance(self) -> &'static str {
        match self {
            Self::PointwiseMean95 => {
                "ordinary least-squares fit with classical pointwise mean-response Student-t 95% confidence band"
            }
            Self::Hidden => {
                "descriptive ordinary least-squares fit with confidence band hidden and no interval claim"
            }
        }
    }

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

/// Semantic mode for line plots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineMode {
    /// Draw ordinary ordered numeric series, partitioned only by explicit group.
    #[default]
    Series,
    /// Draw explicit-group repeated trajectories with neutral connectors.
    Trajectory,
    /// Draw explicit-group numeric paths in source-row order, including repeated x values.
    Path,
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

/// Closed point shapes accepted by interaction style columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PointShape {
    /// Filled circular marker.
    #[default]
    Circle,
    /// Filled upward-pointing triangular marker.
    TriangleUp,
}

impl PointShape {
    /// Exact categorical value accepted in an interaction point-shape column.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Circle => "circle",
            Self::TriangleUp => "triangle-up",
        }
    }

    pub(crate) fn parse_exact(value: &str) -> Option<Self> {
        match value {
            "circle" => Some(Self::Circle),
            "triangle-up" => Some(Self::TriangleUp),
            _ => None,
        }
    }
}

/// Closed connector patterns accepted by interaction style columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinePattern {
    /// Continuous connector.
    #[default]
    Solid,
    /// Connector using the fixed publication dash array `5 3`.
    Dashed,
}

impl LinePattern {
    /// Exact categorical value accepted in an interaction line-pattern column.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Dashed => "dashed",
        }
    }

    pub(crate) fn parse_exact(value: &str) -> Option<Self> {
        match value {
            "solid" => Some(Self::Solid),
            "dashed" => Some(Self::Dashed),
            _ => None,
        }
    }

    pub(crate) fn line_style(
        self,
        color: (u8, u8, u8),
        width: f32,
    ) -> crate::render::backend::LineStyle {
        match self {
            Self::Solid => crate::render::backend::LineStyle::solid(color, width),
            Self::Dashed => crate::render::backend::LineStyle::dashed(color, width, "5 3"),
        }
    }
}

/// Exact availability values accepted by estimate-status columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstimateStatus {
    /// The row owns a complete finite estimate tuple.
    Estimated,
    /// The row owns no numeric estimate tuple and is displayed explicitly.
    NotEstimable,
}

impl EstimateStatus {
    /// Exact value accepted in an estimate-status column.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Estimated => "estimated",
            Self::NotEstimable => "not_estimable",
        }
    }

    pub(crate) fn parse_exact(value: &str) -> Option<Self> {
        match value {
            "estimated" => Some(Self::Estimated),
            "not_estimable" => Some(Self::NotEstimable),
            _ => None,
        }
    }
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

/// Provenance of estimates displayed by an interaction plot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstimateSource {
    /// Unadjusted summaries computed for observed cells.
    ObservedCellSummary,
    /// Model-adjusted estimated marginal means.
    EstimatedMarginalMean,
}

impl EstimateSource {
    /// Stable report-visible description of the estimand source.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ObservedCellSummary => "observed cell summary",
            Self::EstimatedMarginalMean => "estimated marginal mean",
        }
    }
}
