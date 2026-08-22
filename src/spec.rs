//! Shared plot specification and render-result types.

use crate::decorations::legend::LegendEntry;
use crate::layout::{Layout, PanelBox};
use crate::render::backend::SignificanceBracket;

mod categorical;
mod estimate;
mod inference;
mod output;

pub use categorical::{CategoricalColors, CategoryOrder, ValueSort};
pub use estimate::{
    AxisScale, CoefficientSignificanceMark, EstimateSource, EstimateStatus,
    GroupedFactorAnnotation, GroupedFactorEndpoint, LineMode, LinePathStyle, LinePattern,
    PointShape, QqLine, RegressionBand, SignificanceMarkStyle, TextWeight,
};
pub use inference::{
    EffectBandCoverage, EffectBandSemantics, InferentialScale, IntervalKind, IntervalSemantics,
};
#[cfg(feature = "png")]
pub(crate) use output::SVG_CSS_DPI;
pub use output::{PngCompression, PngOptions, PngSizing};

/// Spacing preset for bars within a grouped bar category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarSpacing {
    /// Keep visible space between hue bars inside each category.
    #[default]
    Comfort,
    /// Make hue bars within each category touch side-by-side.
    Compact,
}

/// Value construction for ordinary and grouped bar cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarAggregation {
    /// Render the sample arithmetic mean with an exact two-sided Student-t 95% CI.
    #[default]
    Mean,
    /// Render one already-computed finite value per cell without an interval.
    Identity,
}

/// Maximum star count used to display pairwise significance marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignificanceStarLimit {
    /// Display at most three stars, including for p-values below 0.0001.
    Three,
    /// Display the existing four-star scale.
    #[default]
    Four,
}

impl SignificanceStarLimit {
    pub(crate) fn cap(self, glyph: &str) -> &str {
        match (self, glyph) {
            (Self::Three, "****") => "***",
            _ => glyph,
        }
    }
}

impl BarAggregation {
    /// Fixed plot-owned provenance included in reports.
    #[must_use]
    pub const fn provenance(self) -> &'static str {
        match self {
            Self::Mean => {
                "sample arithmetic mean with exact two-sided Student-t 95% confidence interval on the response scale"
            }
            Self::Identity => {
                "identity/precomputed value with exactly one finite row per rendered cell and no confidence interval"
            }
        }
    }
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

/// Density-curve source for an opt-in histogram composite.
#[derive(Debug, Clone, PartialEq)]
pub enum HistogramKde {
    /// Compute a scale-robust Scott-bandwidth Gaussian KDE from the histogram sample.
    Internal,
    /// Draw caller-computed `(x, density)` points in their supplied order.
    Precomputed(Vec<(f64, f64)>),
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
pub enum PanelCorner {
    /// Top-left corner.
    TopLeft,
    /// Top-right corner.
    TopRight,
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom-right corner.
    BottomRight,
}

/// Caller-owned single-line text anchored inside a chart panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelTextAnnotation {
    corner: PanelCorner,
    text: String,
}

impl PanelTextAnnotation {
    /// Construct one validated panel-corner annotation.
    pub fn new(corner: PanelCorner, text: impl Into<String>) -> Result<Self, crate::Error> {
        let text = text.into();
        if text.trim().is_empty()
            || text.trim() != text
            || text.chars().any(|character| {
                character.is_control() || matches!(character, '\u{2028}' | '\u{2029}')
            })
        {
            return Err(crate::Error::Data(
                "panel text must be nonempty, single-line, trimmed, and contain no control characters"
                    .into(),
            ));
        }
        Ok(Self { corner, text })
    }

    /// Panel corner that owns the annotation.
    #[must_use]
    pub const fn corner(&self) -> PanelCorner {
        self.corner
    }

    /// Verbatim caller-owned annotation text.
    #[must_use]
    pub fn text(&self) -> &str {
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
    /// original zero-based source-row identity.
    Jittered,
}

impl RawOverlay {
    /// Whether the overlay draws original observations.
    #[must_use]
    pub const fn is_visible(self) -> bool {
        !matches!(self, Self::Hidden)
    }

    /// Stable provenance included in reports.
    #[must_use]
    pub const fn provenance(self) -> &'static str {
        match self {
            Self::Hidden => "",
            Self::Points => {
                "raw overlay: original strict-sample observations; centered points; same sample; summaries unchanged; no inference or model adjustment"
            }
            Self::Jittered => {
                "raw overlay: original strict-sample observations; deterministic source-row displacement; same sample; summaries unchanged; no inference or model adjustment"
            }
        }
    }
}

/// Body and observation layout for violin-family plots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViolinLayout {
    /// Draw the existing mirrored KDE body and centered inner summary.
    #[default]
    Mirrored,
    /// Draw a one-sided KDE cloud, separated median/IQR box, and complete
    /// deterministic source-row-offset observation strip.
    Raincloud,
}

/// Continuous color-scale policy for heatmap cells.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum HeatmapScale {
    /// Interpolate from a light low-value color to a dark high-value color.
    #[default]
    Sequential,
    /// Interpolate through a neutral midpoint at `center`.
    Diverging {
        /// Finite value represented by the neutral midpoint color.
        center: f64,
    },
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

/// Display policy for categorical heatmap axis labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeatmapLabelFit {
    /// Preserve the dense-grid default by eliding labels that exceed one cell.
    #[default]
    Elide,
    /// Preserve complete labels and wrap x-axis text within each cell.
    Wrap,
}

/// Fixed decimal formatting shared by optional numeric value labels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericLabelFormat {
    precision: usize,
    suffix: Option<String>,
}

impl NumericLabelFormat {
    /// Construct fixed decimal formatting without a suffix.
    pub fn new(precision: usize) -> Result<Self, crate::Error> {
        Self::build(precision, None)
    }

    /// Construct fixed decimal formatting followed by a non-empty suffix.
    pub fn with_suffix(precision: usize, suffix: impl Into<String>) -> Result<Self, crate::Error> {
        Self::build(precision, Some(suffix.into()))
    }

    fn build(precision: usize, suffix: Option<String>) -> Result<Self, crate::Error> {
        if precision > 12 {
            return Err(crate::Error::Data(format!(
                "numeric label precision {precision} exceeds the supported maximum of 12"
            )));
        }
        if suffix
            .as_ref()
            .is_some_and(|value| value.is_empty() || value.chars().any(char::is_control))
        {
            return Err(crate::Error::Data(
                "numeric label suffix must be non-empty and contain no control characters".into(),
            ));
        }
        Ok(Self { precision, suffix })
    }

    /// Digits rendered after the decimal point.
    #[must_use]
    pub const fn precision(&self) -> usize {
        self.precision
    }

    /// Optional verbatim text rendered immediately after the number.
    #[must_use]
    pub fn suffix(&self) -> Option<&str> {
        self.suffix.as_deref()
    }

    pub(crate) fn format(&self, value: f64) -> String {
        let mut label = format!("{value:.precision$}", precision = self.precision);
        if let Some(suffix) = &self.suffix {
            label.push_str(suffix);
        }
        label
    }
}

/// In-cell value-label policy for heatmaps.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum HeatmapAnnotations {
    /// Draw no values inside cells.
    #[default]
    Hidden,
    /// Draw finite raw values with validated fixed decimal formatting.
    Values(NumericLabelFormat),
}

/// Caller-owned annotation for one fully-null Cartesian heatmap row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeatmapNullRowAnnotation {
    row_label: String,
    text: String,
}

impl HeatmapNullRowAnnotation {
    /// Construct an annotation targeting one normalized heatmap y-category.
    pub fn new(
        row_label: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<Self, crate::Error> {
        let annotation = Self {
            row_label: row_label.into(),
            text: text.into(),
        };
        validate_heatmap_annotation_label(&annotation.row_label, "row label")?;
        validate_heatmap_annotation_label(&annotation.text, "text")?;
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DumbbellEndpoints {
    left_label: String,
    left_color: (u8, u8, u8),
    right_label: String,
    right_color: (u8, u8, u8),
}

impl DumbbellEndpoints {
    /// Construct two explicitly labeled and colored endpoint identities.
    pub fn new(
        left_label: impl Into<String>,
        left_color: (u8, u8, u8),
        right_label: impl Into<String>,
        right_color: (u8, u8, u8),
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

    /// RGB color for the left-value endpoint.
    #[must_use]
    pub const fn left_color(&self) -> (u8, u8, u8) {
        self.left_color
    }

    /// Verbatim label for the right-value endpoint.
    #[must_use]
    pub fn right_label(&self) -> &str {
        &self.right_label
    }

    /// RGB color for the right-value endpoint.
    #[must_use]
    pub const fn right_color(&self) -> (u8, u8, u8) {
        self.right_color
    }
}

/// Caller-owned labels for waterfall base and terminal annotations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaterfallLabels {
    base: String,
    terminal: String,
}

impl WaterfallLabels {
    /// Construct non-empty verbatim labels for the base and terminal values.
    pub fn new(base: impl Into<String>, terminal: impl Into<String>) -> Result<Self, crate::Error> {
        let labels = Self {
            base: base.into(),
            terminal: terminal.into(),
        };
        if labels.base.trim().is_empty() || labels.terminal.trim().is_empty() {
            return Err(crate::Error::Data(
                "waterfall annotation labels must not be empty".into(),
            ));
        }
        if labels.base.trim() != labels.base
            || labels.terminal.trim() != labels.terminal
            || labels.base.chars().any(char::is_control)
            || labels.terminal.chars().any(char::is_control)
        {
            return Err(crate::Error::Data(
                "waterfall annotation labels must contain no surrounding whitespace or control characters"
                    .into(),
            ));
        }
        Ok(labels)
    }

    /// Verbatim label for the base or expected output.
    #[must_use]
    pub fn base(&self) -> &str {
        &self.base
    }

    /// Verbatim label for the terminal value or final prediction.
    #[must_use]
    pub fn terminal(&self) -> &str {
        &self.terminal
    }
}

/// Axis-sharing policy for chart facets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FacetScales {
    /// Share both numeric axes where applicable.
    #[default]
    Fixed,
    /// Resolve the x axis independently per panel.
    FreeX,
    /// Resolve the y axis independently per panel.
    FreeY,
    /// Resolve both axes independently per panel.
    Free,
}

/// Legend policy for a heterogeneous composed figure.
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

/// Requested legend placement.
///
/// Inside positions are limited to plot edges so layout can reserve data space
/// without placing the legend in the middle of the main graphic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LegendPosition {
    /// Use the plot-type default placement.
    #[default]
    Auto,
    /// Do not draw a legend.
    Hidden,
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

macro_rules! define_plot_kinds {
    ($($variant:ident => ($documentation:literal, $label:literal, $key:literal)),+ $(,)?) => {
        /// Which plot type a [`crate::Plot`] will produce.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum PlotKind {
            $(
                #[doc = $documentation]
                $variant,
            )+
        }

        impl PlotKind {
            /// All currently supported plot kinds in stable declaration order.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// Rust variant name used by capability inventories.
            #[must_use]
            pub const fn variant_name(self) -> &'static str {
                match self { $(Self::$variant => stringify!($variant)),+ }
            }

            /// Human-facing plot-kind label used in reports.
            #[must_use]
            pub const fn label(self) -> &'static str {
                match self { $(Self::$variant => $label),+ }
            }

            /// Stable kebab-case segment used in generated figure keys.
            #[must_use]
            pub const fn key_segment(self) -> &'static str {
                match self { $(Self::$variant => $key),+ }
            }
        }
    };
}

define_plot_kinds! {
    Scatter => ("Scatter plot.", "scatter plot", "scatter-plot"),
    Histogram => ("Single-series numeric histogram with an optional KDE-and-rug density composite.", "histogram", "histogram"),
    Density => ("Gaussian kernel density estimate.", "density plot", "density-plot"),
    Qq => ("Quantile-quantile plot against a standard normal distribution.", "Q-Q plot", "qq-plot"),
    PointInterval => ("Point estimates with explicit lower and upper interval bounds.", "point-interval plot", "point-interval-plot"),
    Line => ("Grouped ordered numeric series with optional precomputed ribbons.", "line plot", "line-plot"),
    Interaction => ("Precomputed interaction profiles with explicit estimate provenance and optional closed trace styles.", "interaction plot", "interaction-plot"),
    Coefficient => ("Named model coefficients with optional genuine intervals.", "coefficient plot", "coefficient-plot"),
    Forest => ("Complete interval estimates with an explicit scientific null.", "forest plot", "forest-plot"),
    Dumbbell => ("Horizontal paired endpoint estimates with optional replicate identity.", "dumbbell plot", "dumbbell-plot"),
    Heatmap => ("Categorical grid with a continuous numeric fill.", "heatmap", "heatmap"),
    Bar => ("Bar plot with CI error bars.", "bar plot", "bar-plot"),
    HorizontalBar => ("Horizontal bar plot.", "horizontal bar plot", "horizontal-bar-plot"),
    StackedBar => ("Stacked vertical bar plot using graphical group as the stack series.", "stacked bar plot", "stacked-bar-plot"),
    StackedHorizontalBar => ("Stacked horizontal bar plot using graphical group as the stack series.", "stacked horizontal bar plot", "stacked-horizontal-bar-plot"),
    Boxplot => ("Box-and-whisker plot.", "boxplot", "boxplot"),
    Violin => ("Violin plot with mirrored or raincloud KDE, summary, and observation geometry.", "violin plot", "violin-plot"),
    Regression => ("Regression plot (scatter + fit line).", "regression plot", "regression-plot"),
    Marginal => ("Marginal plot (marginal histograms + central scatter).", "marginal plot", "marginal-plot"),
    Matrix => ("Scatter matrix (small multiples).", "scatter matrix", "scatter-matrix"),
    Beeswarm => ("Categorical rows of collision-separated points with continuous x and color values.", "beeswarm plot", "beeswarm-plot"),
    Waterfall => ("Cumulative signed deltas between a base and terminal value.", "waterfall plot", "waterfall-plot"),
}

impl PlotKind {
    pub(crate) const fn supports_significance(self) -> bool {
        matches!(
            self,
            Self::Bar
                | Self::HorizontalBar
                | Self::StackedBar
                | Self::StackedHorizontalBar
                | Self::Boxplot
                | Self::Violin
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
            | Self::Interaction
            | Self::Boxplot
            | Self::Violin
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

/// Shared immutable inputs for one plot renderer.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PlotRenderContext<'a> {
    pub(crate) data: crate::data::PlotData<'a>,
    pub(crate) panel: PanelBox,
}

impl<'a> PlotRenderContext<'a> {
    pub(crate) const fn new(data: crate::data::PlotData<'a>, panel: PanelBox) -> Self {
        Self { data, panel }
    }

    pub(crate) fn with_source_rows(
        data: crate::data::PlotData<'a>,
        source_rows: &'a [usize],
        panel: PanelBox,
    ) -> Self {
        Self {
            data: data.map_source_rows(source_rows),
            panel,
        }
    }
}

pub(crate) struct PlotRenderResult {
    pub(crate) layout: Layout,
    pub(crate) x_domain: AxisDomain,
    pub(crate) y_domain: AxisDomain,
    pub(crate) significance_brackets: Vec<SignificanceBracket>,
    pub(crate) legend_entries: Vec<LegendEntry>,
    pub(crate) decoration_mode: DecorationMode,
    pub(crate) x_tick_keys: Option<Vec<String>>,
    pub(crate) y_tick_keys: Option<Vec<String>>,
    pub(crate) legend_keys: Vec<String>,
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
    pub(crate) fn new(layout: Layout, legend_entries: Vec<LegendEntry>) -> Self {
        Self {
            layout,
            x_domain: AxisDomain::Continuous,
            y_domain: AxisDomain::Continuous,
            significance_brackets: Vec::new(),
            legend_entries,
            decoration_mode: DecorationMode::Standard,
            x_tick_keys: None,
            y_tick_keys: None,
            legend_keys: Vec::new(),
        }
    }

    pub(crate) fn with_axis_domains(mut self, x: AxisDomain, y: AxisDomain) -> Self {
        self.x_domain = x;
        self.y_domain = y;
        self
    }

    pub(crate) fn with_significance_brackets(mut self, brackets: Vec<SignificanceBracket>) -> Self {
        self.significance_brackets = brackets;
        self
    }

    pub(crate) fn with_legend_keys(mut self, keys: Vec<String>) -> Self {
        self.legend_keys = keys;
        self
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

#[cfg(test)]
mod tests;
