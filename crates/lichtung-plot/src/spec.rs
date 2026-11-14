//! Shared plot specification and render-result types.

use crate::layout::Layout;
use crate::significance::SignificancePlan;
use crate::{NumericFormat, NumericValues};

mod bar;
mod categorical;
mod estimate;
mod facet;
mod output;
pub(crate) mod ticks;

pub use bar::{
    BarInput, BarSamples, Bootstrap, ConfidenceInterval, ErrorBar, PercentileInterval,
    StandardDeviation, StandardError,
};
pub(crate) use bar::{BarSource, ConfidenceIntervalAlgorithm, ErrorBarKind};
pub use categorical::{CategoricalColorMap, CategoryOrder, ValueSort};
pub use estimate::{
    AxisScale, Bounds, CoefficientInput, EstimateStatus, InteractionStyle, LinePathStyle,
    LinePattern, PointShape, QqLine, RegressionBand, SignificanceMarkStyle, TextWeight,
};
pub use facet::{FacetScales, FacetSpec};
pub(crate) use output::SVG_CSS_DPI;
pub use output::{PngCompression, PngOptions, PngSizing};
pub use ticks::{
    TickContext, TickGeneration, TickPositions, TickRequirement, TickSingleton, TickSpace,
};
mod category_labels;
pub use category_labels::{CategoryLabelSelection, LabelEndpoints};

/// Spacing preset for multiple series within one bar category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarSpacing {
    /// Keep visible space between series bars inside each category.
    #[default]
    Comfort,
    /// Make series bars within each category touch side-by-side.
    Compact,
}

/// Histogram bin-selection strategy.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum HistogramBins {
    /// Select a width with the Freedman-Diaconis rule, falling back to Sturges.
    /// Automatic selection is capped at 512 bins; constant data uses one
    /// centered bin.
    #[default]
    Auto,
    /// Render exactly this many centered equal-width bins, including for
    /// constant data, up to the safety limit of 1,000,000 bins.
    Count(usize),
    /// Render bins with this data-space width, provided the resulting count
    /// does not exceed the 1,000,000-bin safety limit. Constant data uses one
    /// bin centered on the value and rejects widths that are not representable.
    Width(f64),
}

/// Histogram vertical-axis normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HistogramStat {
    /// Raw observations per bin.
    #[default]
    Count,
    /// Fraction of all observations per bin.
    Probability,
    /// Probability density; bin area integrates to one.
    Density,
}

/// Density-curve source for an opt-in composite over sample-owned histogram input.
#[derive(Debug, Clone)]
pub enum HistogramKde {
    /// Compute a scale-robust Scott-bandwidth Gaussian KDE from the histogram sample.
    Internal,
    /// Draw caller-computed aligned x/y columns in supplied order.
    ///
    /// ```
    /// use lichtung_plot::{HistogramKde, NumericValues};
    ///
    /// let kde = HistogramKde::Precomputed {
    ///     x: NumericValues::new("x", [0.0, 1.0]),
    ///     y: NumericValues::new("density", [0.1, 0.2]),
    /// };
    /// # let _ = kde;
    /// ```
    Precomputed {
        /// Strictly increasing x coordinates within the sample-owned bin support.
        x: NumericValues,
        /// Nonnegative densities aligned with x.
        y: NumericValues,
    },
}

impl PartialEq for HistogramKde {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Internal, Self::Internal) => true,
            (
                Self::Precomputed { x, y },
                Self::Precomputed {
                    x: other_x,
                    y: other_y,
                },
            ) => {
                crate::geometry::numeric_equal(x, other_x)
                    && crate::geometry::numeric_equal(y, other_y)
            }
            _ => false,
        }
    }
}

/// Optional layers composed on a histogram density axis.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum HistogramComposite {
    /// Draw only histogram bars. This is the byte-inert default.
    #[default]
    HistogramOnly,
    /// Draw a thin KDE density curve and one rug mark per source observation.
    KdeAndRug(HistogramKde),
}

impl HistogramComposite {
    pub(crate) const fn is_enabled(&self) -> bool {
        matches!(self, Self::KdeAndRug(_))
    }
}

/// A corner inside one chart's resolved mark panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataAreaCorner {
    /// Top-left corner.
    TopLeft,
    /// Top-right corner.
    TopRight,
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom-right corner.
    BottomRight,
}

/// Caller-owned text block anchored inside a chart panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlotTextAnnotation {
    position: crate::TextPosition,
    text: crate::Text,
}

impl PlotTextAnnotation {
    /// Construct one validated panel-corner annotation from plain or styled text.
    ///
    /// Local appearance replaces the complete panel-text role. Alignment applies
    /// within the text block; the selected corner continues to place that block.
    pub fn new(corner: DataAreaCorner, text: impl Into<crate::Text>) -> Result<Self, crate::Error> {
        Self::at(crate::TextPosition::corner(corner), text)
    }

    /// Attach authored text to independent typed data, panel or figure references.
    pub fn at(
        position: crate::TextPosition,
        text: impl Into<crate::Text>,
    ) -> Result<Self, crate::Error> {
        let text = text.into();
        text.required_source("panel text")?;
        Ok(Self { position, text })
    }

    /// Semantic reference that owns this annotation's placement.
    #[must_use]
    pub const fn position(&self) -> &crate::TextPosition {
        &self.position
    }

    /// Verbatim caller-owned annotation text.
    #[must_use]
    pub fn text(&self) -> &str {
        self.text.as_str()
    }

    pub(crate) const fn content(&self) -> &crate::Text {
        &self.text
    }
}

/// Original-observation overlay for boxplots and violin plots.
///
/// Visible modes draw one fixed-style point for every observation in the same
/// strict sample used by the summary. They do not filter, transform, aggregate,
/// resample, or create a legend identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RawOverlay {
    /// Draw no raw observations. This is the inert default.
    #[default]
    Hidden,
    /// Draw every observation at its exact category or dodge center.
    Points,
    /// Deterministically displace every observation from its center using its
    /// plot-level source-row identity.
    Jittered,
}

impl RawOverlay {
    /// Whether the overlay draws original observations.
    #[must_use]
    pub const fn is_visible(self) -> bool {
        !matches!(self, Self::Hidden)
    }
}

/// Coordinate layout for a categorical heatmap grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeatmapLayout {
    /// Draw every cell in the Cartesian product of observed x and y levels.
    #[default]
    Cartesian,
    /// Draw the lower triangle of one shared family order.
    LowerTriangle {
        /// Whether diagonal coordinates are active cells.
        diagonal: bool,
    },
    /// Draw the upper triangle of one shared family order.
    UpperTriangle {
        /// Whether diagonal coordinates are active cells.
        diagonal: bool,
    },
}

/// In-cell value-label policy for heatmaps.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum HeatmapAnnotations {
    /// Draw no values inside cells.
    #[default]
    Hidden,
    /// Draw finite raw values using one complete panel-local numeric format set.
    Values(NumericFormat),
}

/// Caller-owned annotation for one fully-null Cartesian heatmap row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeatmapNullRowAnnotation {
    row_label: String,
    text: crate::Text,
}

impl HeatmapNullRowAnnotation {
    /// Construct an annotation targeting one normalized heatmap y-category.
    /// Text can replace the complete selected contrast style and line alignment;
    /// row identity remains a semantic string and is validated independently.
    pub fn new(
        row_label: impl Into<String>,
        text: impl Into<crate::Text>,
    ) -> Result<Self, crate::Error> {
        let annotation = Self {
            row_label: row_label.into(),
            text: text.into(),
        };
        validate_heatmap_annotation_label(&annotation.row_label, "row label")?;
        annotation
            .text
            .required_source("heatmap fully-null row annotation text")?;
        Ok(annotation)
    }

    /// Normalized y-category targeted by this annotation.
    #[must_use]
    pub fn row_label(&self) -> &str {
        &self.row_label
    }

    /// Verbatim text wrapped inside the merged row.
    #[must_use]
    pub fn text(&self) -> &str {
        self.text.as_str()
    }

    pub(crate) const fn content(&self) -> &crate::Text {
        &self.text
    }
}

fn validate_heatmap_annotation_label(value: &str, role: &str) -> Result<(), crate::Error> {
    if value.is_empty() || value.trim() != value || value.chars().any(char::is_control) {
        return Err(crate::Error::Data(format!(
            "heatmap fully-null row annotation {role} must be nonempty, trimmed, and contain no control characters"
        )));
    }
    Ok(())
}

/// Explicit endpoint identities and colors for a dumbbell plot.
///
/// Each exact Color replaces the entire endpoint fill and its literal guide
/// paint. Theme marker geometry and the independent outline still apply.
/// ```
/// use lichtung_plot::{Color, DumbbellEndpoints};
/// let endpoints = DumbbellEndpoints::new(
///     "Before", Color::rgba(20, 80, 140, 128), "After", Color::rgb(180, 60, 40),
/// )?;
/// assert_eq!(endpoints.left_color().get_alpha(), 128);
/// assert_eq!(endpoints.right_color().get_alpha(), 255);
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
///
/// ```compile_fail
/// use lichtung_plot::DumbbellEndpoints;
/// let _ = DumbbellEndpoints::new("Before", (20, 80, 140), "After", (180, 60, 40));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DumbbellEndpoints {
    left_label: String,
    left_color: crate::Color,
    right_label: String,
    right_color: crate::Color,
}

impl DumbbellEndpoints {
    /// Construct two explicitly labeled and colored endpoint identities.
    pub fn new(
        left_label: impl Into<String>,
        left_color: crate::Color,
        right_label: impl Into<String>,
        right_color: crate::Color,
    ) -> Result<Self, crate::Error> {
        let endpoints = Self {
            left_label: left_label.into(),
            left_color,
            right_label: right_label.into(),
            right_color,
        };
        if endpoints.left_label.trim().is_empty() || endpoints.right_label.trim().is_empty() {
            return Err(crate::Error::Data(
                "dumbbell endpoint labels must not be empty".into(),
            ));
        }
        if endpoints.left_label.trim() != endpoints.left_label
            || endpoints.right_label.trim() != endpoints.right_label
            || endpoints.left_label.chars().any(char::is_control)
            || endpoints.right_label.chars().any(char::is_control)
        {
            return Err(crate::Error::Data(
                "dumbbell endpoint labels must contain no surrounding whitespace or control characters"
                    .into(),
            ));
        }
        if endpoints.left_label == endpoints.right_label {
            return Err(crate::Error::Data(
                "dumbbell endpoint labels must be distinct".into(),
            ));
        }
        Ok(endpoints)
    }

    /// Verbatim label for the left-value endpoint.
    #[must_use]
    pub fn left_label(&self) -> &str {
        &self.left_label
    }

    /// Complete exact RGBA paint for the left-value endpoint and its guide key.
    #[must_use]
    pub const fn left_color(&self) -> crate::Color {
        self.left_color
    }

    /// Verbatim label for the right-value endpoint.
    #[must_use]
    pub fn right_label(&self) -> &str {
        &self.right_label
    }

    /// Complete exact RGBA paint for the right-value endpoint and its guide key.
    #[must_use]
    pub const fn right_color(&self) -> crate::Color {
        self.right_color
    }
}

/// Caller-owned labels for waterfall base and terminal annotations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaterfallLabels {
    base: crate::Text,
    terminal: crate::Text,
}

impl WaterfallLabels {
    /// Construct verbatim display labels for the base and terminal values.
    /// Each Text retains independent complete appearance and line alignment for
    /// the displayed name plus formatted numeric value. Readers borrow the name.
    pub fn new(
        base: impl Into<crate::Text>,
        terminal: impl Into<crate::Text>,
    ) -> Result<Self, crate::Error> {
        let labels = Self {
            base: base.into(),
            terminal: terminal.into(),
        };
        labels.base.required_source("waterfall base label")?;
        labels
            .terminal
            .required_source("waterfall terminal label")?;
        Ok(labels)
    }

    /// Verbatim label for the base or expected output.
    #[must_use]
    pub fn base(&self) -> &str {
        self.base.as_str()
    }

    /// Verbatim label for the terminal value or final prediction.
    #[must_use]
    pub fn terminal(&self) -> &str {
        self.terminal.as_str()
    }

    pub(crate) const fn contents(&self) -> [&crate::Text; 2] {
        [&self.base, &self.terminal]
    }
}

/// Legend policy for a heterogeneous composed canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasLegendPolicy {
    /// Render each panel's own legend.
    #[default]
    PerPlot,
    /// Render one legend after validating that panel legends are compatible.
    Shared,
    /// Suppress all panel legends.
    Hidden,
}

/// Semantic composition of one discrete legend, independently of its appearance.
///
/// Standalone plots and facets use their Plot's selection. Canvas PerPlot uses
/// each Plot's selection; Shared uses the Canvas selection for every contributor
/// without changing the stored Plot values. Hidden skips guide-only composition.
/// Continuous color guides keep their own representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LegendComposition {
    /// Preserve observed identities and complete mark keys in one section.
    #[default]
    Combined,
    /// Explain independent mapped channels in ordered sections within one frame.
    ///
    /// Independent group/hue pairs require an explicitly configured group-style
    /// channel, currently Interaction styles. Unsupported decompositions refuse
    /// before fonts. Same-field channels remain one complete joint section.
    Separate,
}

/// Requested legend placement.
///
/// Inside positions are limited to plot edges so layout can reserve data space
/// without placing the legend in the middle of the main graphic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LegendPosition {
    /// Use the plot-type default placement.
    #[default]
    Auto,
    /// Inside the axes at the top-left edge.
    InsideTopLeft,
    /// Inside the axes at the top edge.
    InsideTop,
    /// Inside the axes at the top-right edge.
    InsideTopRight,
    /// Inside the axes at the right-top edge.
    InsideRightTop,
    /// Inside the axes at the right edge.
    InsideRight,
    /// Inside the axes at the right-bottom edge.
    InsideRightBottom,
    /// Inside the axes at the bottom-left edge.
    InsideBottomLeft,
    /// Inside the axes at the bottom edge.
    InsideBottom,
    /// Inside the axes at the bottom-right edge.
    InsideBottomRight,
    /// Inside the axes at the left-top edge.
    InsideLeftTop,
    /// Inside the axes at the left edge.
    InsideLeft,
    /// Inside the axes at the left-bottom edge.
    InsideLeftBottom,
    /// Outside the axes in right-side whitespace, aligned to panel top.
    OutsideRightTop,
    /// Outside the axes in right-side whitespace, aligned to panel center.
    OutsideRight,
    /// Outside the axes in right-side whitespace, aligned to panel bottom.
    OutsideRightBottom,
    /// Outside the axes in top whitespace, aligned to panel left.
    OutsideTopLeft,
    /// Outside the axes in top whitespace, aligned to panel center.
    OutsideTop,
    /// Outside the axes in top whitespace, aligned to panel right.
    OutsideTopRight,
    /// Outside the axes in bottom whitespace, aligned to panel left.
    OutsideBottomLeft,
    /// Outside the axes in bottom whitespace, aligned to panel center.
    OutsideBottom,
    /// Outside the axes in bottom whitespace, aligned to panel right.
    OutsideBottomRight,
    /// Outside the axes in left-side whitespace, aligned to panel top.
    OutsideLeftTop,
    /// Outside the axes in left-side whitespace, aligned to panel center.
    OutsideLeft,
    /// Outside the axes in left-side whitespace, aligned to panel bottom.
    OutsideLeftBottom,
}

/// Visibility and placement policy for a plot legend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LegendPolicy {
    /// Use the plot-type default visibility and automatic placement.
    #[default]
    Automatic,
    /// Do not draw a legend.
    Hidden,
    /// Draw the legend at the requested placement.
    Visible(LegendPosition),
}

impl LegendPolicy {
    pub(crate) const fn position(self) -> LegendPosition {
        match self {
            Self::Visible(position) => position,
            Self::Automatic | Self::Hidden => LegendPosition::Auto,
        }
    }
}

macro_rules! define_plot_kinds {
    ($($variant:ident => $documentation:literal),+ $(,)?) => {
        /// Which plot type a [`crate::Plot`] will produce.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum PlotKind {
            $(
                #[doc = $documentation]
                $variant,
            )+
        }
    };
}

define_plot_kinds! {
    Scatter => "Scatter plot.",
    Histogram => "Single-series numeric histogram with an optional KDE-and-rug density composite.",
    Density => "Gaussian kernel density estimate.",
    Qq => "Quantile-quantile plot against a standard normal distribution.",
    PointInterval => "Point estimates with explicit lower and upper interval bounds.",
    Line => "Grouped ordered numeric series with optional precomputed ribbons.",
    Trajectory => "Explicit-group repeated trajectories with neutral connectors.",
    LinePath => "Explicit-group numeric paths in source-row order.",
    Interaction => "Precomputed interaction profiles with explicit bounds and optional closed trace styles.",
    Coefficient => "Named model coefficients with optional drawable bounds.",
    Forest => "Complete interval estimates with an explicit scientific null.",
    Dumbbell => "Horizontal paired endpoint estimates with optional replicate identity.",
    Heatmap => "Categorical grid with a continuous numeric fill.",
    Bar => "Bar plot with CI error bars.",
    HorizontalBar => "Horizontal bar plot.",
    StackedBar => "Stacked vertical bar plot using one categorical series.",
    StackedHorizontalBar => "Stacked horizontal bar plot using one categorical series.",
    Boxplot => "Box-and-whisker plot.",
    Violin => "Mirrored KDE violins with centered inner summaries.",
    Raincloud => "One-sided KDE clouds with separated summaries and complete observation strips.",
    Regression => "Regression plot (scatter + fit line).",
    Marginal => "Marginal plot (marginal histograms + central scatter).",
    Matrix => "Scatter matrix (small multiples).",
    Beeswarm => "Categorical rows of collision-separated points with continuous x and color values.",
    Waterfall => "Cumulative signed deltas between a base and terminal value.",
}

impl PlotKind {
    pub(crate) const fn uses_unified_series(self) -> bool {
        matches!(
            self,
            Self::Bar
                | Self::HorizontalBar
                | Self::StackedBar
                | Self::StackedHorizontalBar
                | Self::Boxplot
                | Self::Violin
                | Self::Raincloud
        )
    }

    pub(crate) const fn supports_significance_annotations(self) -> bool {
        matches!(
            self,
            Self::Bar
                | Self::HorizontalBar
                | Self::StackedBar
                | Self::StackedHorizontalBar
                | Self::Boxplot
                | Self::Violin
                | Self::Raincloud
                | Self::Coefficient
        )
    }

    pub(crate) const fn legend_default(self, explicit_y_limits: bool) -> LegendDefault {
        match self {
            Self::HorizontalBar | Self::StackedHorizontalBar | Self::Dumbbell => {
                LegendDefault::TopRight(LegendClearance::HorizontalMaximum)
            }
            Self::Bar
            | Self::StackedBar
            | Self::Line
            | Self::Trajectory
            | Self::LinePath
            | Self::Interaction
            | Self::Boxplot
            | Self::Violin
            | Self::Raincloud
            | Self::Regression => LegendDefault::TopRight(LegendClearance::VerticalMaximum),
            Self::Scatter | Self::Marginal if !explicit_y_limits => {
                LegendDefault::TopRight(LegendClearance::VerticalMaximum)
            }
            Self::Matrix => LegendDefault::SelfManaged,
            _ => LegendDefault::LowestOverlap,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LegendDefault {
    LowestOverlap,
    TopRight(LegendClearance),
    SelfManaged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LegendClearance {
    HorizontalMaximum,
    VerticalMaximum,
}

use crate::Error;

/// A short borrow of one root-issued allocation and its canonical occurrence.
/// Geometry, domains and dimensions have exactly one owner: the allocation.
pub(crate) struct PlotMeasureContext<'v, 'src> {
    pub(crate) registered: &'v crate::orchestrator::registration::PlotRegistration<'src>,
    pub(crate) assignment: crate::orchestrator::assignment::RootPlotAssignmentRef<'v>,
    pub(crate) canvas: Option<&'src crate::Canvas>,
    pub(crate) text: &'v crate::text::TextService,
    pub(crate) operation: &'v crate::font::canonical::OperationContext,
    pub(crate) intrinsic: &'v crate::orchestrator::root::PlotIntrinsics,
    pub(crate) guides: crate::decorations::legend::discrete::scoped::GuideConsumption<'v>,
    pub(crate) fit_scope: &'v crate::layout::fit::FitScope,
}

impl<'v, 'src> PlotMeasureContext<'v, 'src> {
    pub(crate) const fn fit_scope(&self) -> &'v crate::layout::fit::FitScope {
        self.fit_scope
    }

    pub(crate) const fn intrinsic(&self) -> &'v crate::orchestrator::root::PlotIntrinsics {
        self.intrinsic
    }

    pub(crate) fn family(&self) -> Result<crate::orchestrator::registration::FamilyRef<'v>, Error> {
        crate::layout::interior::InteriorRoute::classify(self.registered.family.kind())
            .validate(self.registered.source)?;
        self.registered.family()
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "the fixed fallible axis-pair interface projects prepared operands without adding validation"
    )]
    pub(crate) fn axis_pair(
        &self,
        scope: crate::orchestrator::assignment::AxisPairScope,
    ) -> Result<crate::orchestrator::assignment::AxisPairRef<'v>, Error> {
        use crate::layout::AxisSide::{Bottom, Left};
        use crate::orchestrator::assignment::{AssignedAxisRef, AxisDomainAssignments as D};
        use crate::orchestrator::assignment::{AxisPairRef, AxisPairScope as S};
        use crate::orchestrator::registration::{AxisSlot as A, MarginalStrip};
        let axes = &self.registered.axes;
        let source = self.registered.source;
        let pair = match (scope, &self.assignment.local.domains) {
            (S::Pair, D::Pair(domains)) => {
                [(A::Pair(Bottom), &domains[0]), (A::Pair(Left), &domains[1])]
            }
            (S::MarginalCentral, D::Marginal { central, .. }) => [
                (A::MarginalCentral(Bottom), &central[0]),
                (A::MarginalCentral(Left), &central[1]),
            ],
            (
                S::MarginalTop,
                D::Marginal {
                    central,
                    strip_density,
                },
            ) => [
                (A::MarginalCentral(Bottom), &central[0]),
                (
                    A::MarginalStripDensity(MarginalStrip::Top),
                    &strip_density[0],
                ),
            ],
            (
                S::MarginalRight,
                D::Marginal {
                    central,
                    strip_density,
                },
            ) => [
                (
                    A::MarginalStripDensity(MarginalStrip::Right),
                    &strip_density[1],
                ),
                (A::MarginalCentral(Left), &central[1]),
            ],
            (S::MatrixCell { row, column }, D::Matrix { variables, .. }) => [
                (A::MatrixVariable(column), &variables[column]),
                (A::MatrixVariable(row), &variables[row]),
            ],
            (
                S::MatrixDiagonalDensity { variable },
                D::Matrix {
                    variables,
                    diagonal_density,
                },
            ) => [
                (A::MatrixVariable(variable), &variables[variable]),
                (
                    A::MatrixDiagonalDensity(variable),
                    &diagonal_density[variable],
                ),
            ],
            _ => unreachable!("axis pair belongs to the selected interior route"),
        };
        Ok(AxisPairRef {
            axes: pair.map(|(slot, domain)| AssignedAxisRef {
                base: axes.axis_at(source, slot),
                domain,
            }),
        })
    }

    pub(crate) fn data(&self) -> Result<crate::data::PlotData<'src, 'v>, Error> {
        self.registered.data(self.operation)
    }

    pub(crate) fn request(&self) -> crate::text::TextRequestContext<'v, 'src> {
        crate::text::TextRequestContext::new(&self.registered.appearance, self.text)
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "native measurements share the existing fallible range-reader interface"
    )]
    pub(crate) fn axis_range(&self, side: crate::layout::AxisSide) -> Result<(f64, f64), Error> {
        use crate::orchestrator::assignment::AxisDomainAssignments;
        let index = usize::from(side == crate::layout::AxisSide::Left);
        let pair = match &self.assignment.local.domains {
            AxisDomainAssignments::Pair(pair) => pair,
            AxisDomainAssignments::Marginal { central, .. } => central,
            AxisDomainAssignments::Matrix { .. } => {
                unreachable!("Matrix selects an indexed axis pair")
            }
        };
        Ok(pair[index].extent.pair())
    }

    pub(crate) fn prepared_category_labels(
        &self,
        side: usize,
    ) -> Option<&'v [crate::layout::AxisTick]> {
        Some(&self.registered.axes.coordinate()?.categories(side)?.ticks)
    }

    pub(crate) fn semantic_inputs(
        &self,
        scope: crate::orchestrator::assignment::AxisPairScope,
    ) -> Result<[crate::layout::AxisSemanticInputs; 2], Error> {
        use crate::orchestrator::assignment::AssignedAxisRef;

        let mut inputs = self
            .axis_pair(scope)?
            .axes
            .map(AssignedAxisRef::semantic_inputs);
        for (side, input) in inputs.iter_mut().enumerate() {
            input
                .text_layout
                .clone_from(&self.registered.appearance.text_layouts.ticks[side]);
        }
        Ok(inputs)
    }
}

/// Current constrained interior result. FitOutcome/PlanError distinguish measured,
/// terminal fit and fatal outcomes; provisional misses belong to FeedbackRound.
/// Point/path families retain selected marks and annotations; other marks remain combined.
#[derive(Debug)]
pub(crate) struct PlotRenderResult {
    pub(crate) layout: Layout,
    pub(crate) axes: Option<crate::layout::AxisSystemPlan>,
    pub(crate) grid_axis_gutters: Option<[f64; 4]>,
    pub(crate) x_domain: AxisDomain,
    pub(crate) y_domain: AxisDomain,
    pub(crate) significance: Option<SignificancePlan>,
    pub(crate) discrete_guide:
        Option<std::sync::Arc<crate::decorations::legend::discrete::PreparedGuide>>,
    pub(crate) decoration_mode: DecorationMode,
    pub(crate) x_tick_keys: Option<Vec<String>>,
    pub(crate) y_tick_keys: Option<Vec<String>>,
    pub(crate) deferred_annotation_text: Vec<crate::layout::DeferredAnnotationTextPlan>,
    pub(crate) producer_text: Vec<crate::render::backend::PlacedText>,
    pub(crate) panel_text: Vec<crate::decorations::panel_text::PlotTextAnnotationPlan>,
    pub(crate) self_managed_legend: Option<std::sync::Arc<crate::decorations::legend::Legend>>,
    pub(crate) matrix_axes: Option<std::sync::Arc<crate::plots::matrix::MatrixAxisPlan>>,
    pub(crate) marks: Option<std::sync::Arc<crate::layout::interior::SelectedMarkPlan>>,
    pub(crate) annotations: Option<std::sync::Arc<crate::layout::interior::SelectedAnnotationPlan>>,
    pub(crate) numeric_presentation:
        Option<std::sync::Arc<crate::numeric::set::NumericPresentation>>,
}

/// Semantic kind of one renderer-resolved axis domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AxisDomain {
    Continuous,
    Categorical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecorationMode {
    Standard,
    SelfManaged,
}

impl PlotRenderResult {
    pub(crate) fn draw_body(
        &self,
        backend: &mut dyn crate::render::backend::Backend,
    ) -> Result<DecorationMode, Error> {
        self.marks
            .as_ref()
            .ok_or_else(|| Error::Render("selected mark payload is missing".into()))?
            .draw(backend)?;
        self.annotations
            .as_ref()
            .ok_or_else(|| Error::Render("selected annotation payload is missing".into()))?
            .draw(backend)?;
        Ok(self.decoration_mode)
    }

    pub(crate) fn new(
        layout: Layout,
        discrete_guide: Option<std::sync::Arc<crate::decorations::legend::discrete::PreparedGuide>>,
    ) -> Self {
        Self {
            layout,
            axes: None,
            grid_axis_gutters: None,
            x_domain: AxisDomain::Continuous,
            y_domain: AxisDomain::Continuous,
            significance: None,
            discrete_guide,
            decoration_mode: DecorationMode::Standard,
            x_tick_keys: None,
            y_tick_keys: None,
            deferred_annotation_text: Vec::new(),
            producer_text: Vec::new(),
            panel_text: Vec::new(),
            self_managed_legend: None,
            matrix_axes: None,
            marks: None,
            annotations: None,
            numeric_presentation: None,
        }
    }

    pub(crate) fn with_self_managed_decorations(mut self) -> Self {
        self.decoration_mode = DecorationMode::SelfManaged;
        self
    }

    pub(crate) fn with_resolved_axis_keys(
        mut self,
        x_keys: Vec<String>,
        y_keys: Vec<String>,
    ) -> Self {
        self.x_tick_keys = Some(x_keys);
        self.y_tick_keys = Some(y_keys);
        self
    }
}
