//! Shared plot specification and render-result types.

use crate::decorations::legend::LegendEntry;
use crate::layout::{Layout, PanelBox};
use crate::significance::SignificancePlan;
use crate::HistogramKdePoint;

mod bar;
mod categorical;
mod estimate;
mod facet;
mod output;

pub use bar::{
    BarInput, BarSamples, Bootstrap, ConfidenceInterval, ErrorBar, PercentileInterval,
    StandardDeviation, StandardError,
};
pub(crate) use bar::{BarSource, ConfidenceIntervalAlgorithm, ErrorBarKind};
pub use categorical::{CategoricalColors, CategoryOrder, ValueSort};
pub use estimate::{
    AxisScale, Bounds, CoefficientInput, EstimateStatus, InteractionStyle, LinePathStyle,
    LinePattern, PointShape, QqLine, RegressionBand, SignificanceMarkStyle, TextWeight,
};
pub use facet::{FacetScales, FacetSpec};
pub(crate) use output::SVG_CSS_DPI;
pub use output::{PngCompression, PngOptions, PngSizing};

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
#[derive(Debug, Clone, PartialEq)]
pub enum HistogramKde {
    /// Compute a scale-robust Scott-bandwidth Gaussian KDE from the histogram sample.
    Internal,
    /// Draw caller-computed points with named `x` and `y` coordinates in supplied order.
    ///
    /// ```
    /// use lichtung_plot::{HistogramKde, HistogramKdePoint};
    ///
    /// let kde = HistogramKde::Precomputed(vec![
    ///     HistogramKdePoint { x: 0.0, y: 0.1 },
    ///     HistogramKdePoint { x: 1.0, y: 0.2 },
    /// ]);
    /// # let _ = kde;
    /// ```
    Precomputed(Vec<HistogramKdePoint>),
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

/// Fixed-decimal formatting for opt-in heatmap cell values and point-only
/// coefficient estimate labels.
///
/// An optional suffix is appended literally and does not scale or otherwise
/// transform the value. This format does not control axis ticks, colorbars,
/// waterfall totals, point-layer labels, or legends.
///
/// ```compile_fail
/// use lichtung_plot::NumericLabelFormat;
/// let _ = NumericLabelFormat::with_suffix(2, "%");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericLabelFormat {
    precision: usize,
    suffix: Option<String>,
}

impl NumericLabelFormat {
    pub(crate) fn admit_clone(&self, data: crate::data::PlotData<'_>) -> Result<(), crate::Error> {
        if let Some(suffix) = &self.suffix {
            data.admit_clone_slice(suffix.as_bytes())?;
        }
        Ok(())
    }

    /// Construct fixed decimal formatting without a suffix.
    pub fn new(precision: usize) -> Result<Self, crate::Error> {
        Self::default().precision(precision)
    }

    /// Set the number of digits rendered after the decimal point.
    pub fn precision(mut self, precision: usize) -> Result<Self, crate::Error> {
        if precision > 12 {
            return Err(crate::Error::Data(format!(
                "numeric label precision {precision} exceeds the supported maximum of 12"
            )));
        }
        self.precision = precision;
        Ok(self)
    }

    /// Set non-empty verbatim text rendered immediately after the number.
    ///
    /// The suffix does not scale or otherwise transform the numeric value.
    pub fn suffix(mut self, suffix: impl Into<String>) -> Result<Self, crate::Error> {
        let suffix = suffix.into();
        if suffix.is_empty() || suffix.chars().any(char::is_control) {
            return Err(crate::Error::Data(
                "numeric label suffix must be non-empty and contain no control characters".into(),
            ));
        }
        self.suffix = Some(suffix);
        Ok(self)
    }

    pub(crate) const fn configured_precision(&self) -> usize {
        self.precision
    }

    #[cfg(test)]
    pub(crate) fn format(&self, value: f64) -> String {
        self.display(value).to_string()
    }

    pub(crate) fn format_source(
        &self,
        value: f64,
        data: crate::data::PlotData<'_>,
    ) -> Result<String, crate::Error> {
        use std::fmt::Write;
        struct Count(usize);
        impl Write for Count {
            fn write_str(&mut self, value: &str) -> std::fmt::Result {
                self.0 = self.0.checked_add(value.len()).ok_or(std::fmt::Error)?;
                Ok(())
            }
        }
        // A finite f64 needs at most 309 integral digits and 12 fractional digits.
        data.claim_source_visits(324)?;
        data.claim_source_visits(self.suffix.as_ref().map_or(0, String::len))?;
        let mut count = Count(0);
        write!(count, "{}", self.display(value))
            .map_err(|error| crate::Error::Render(format!("estimate label count: {error}")))?;
        let mut text = String::new();
        data.reserve_source_string(&mut text, count.0)?;
        write!(text, "{}", self.display(value))
            .map_err(|error| crate::Error::Render(format!("estimate label: {error}")))?;
        Ok(text)
    }

    pub(crate) fn display(&self, value: f64) -> impl std::fmt::Display + '_ {
        NumericLabelDisplay {
            format: self,
            value,
        }
    }
}

struct NumericLabelDisplay<'a> {
    format: &'a NumericLabelFormat,
    value: f64,
}

impl std::fmt::Display for NumericLabelDisplay<'_> {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            output,
            "{:.precision$}",
            self.value,
            precision = self.format.precision
        )?;
        output.write_str(self.format.suffix.as_deref().unwrap_or_default())
    }
}

impl Default for NumericLabelFormat {
    fn default() -> Self {
        Self {
            precision: 2,
            suffix: None,
        }
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

/// Shared immutable inputs for one plot renderer.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PlotRenderContext<'a> {
    pub(crate) data: crate::data::PlotData<'a>,
    pub(crate) panel: PanelBox,
    // Parent-owned perpendicular capacity; axis feedback must not shrink it a
    // second time merely because the assigned mark panel became narrower.
    pub(crate) axis_label_width: f64,
    pub(crate) retained_axes: Option<&'a crate::layout::PanelAxisPlan>,
    retained_layout: Option<&'a Layout>,
    pub(crate) selected_interior: Option<&'a PlotRenderResult>,
    prepared_axes: Option<&'a crate::plots::shared_x::PreparedAxes>,
    prepared_limits: [Option<(f64, f64)>; 2],
    prepared_kind: Option<PlotKind>,
    pub(crate) theme: &'a crate::theme::ResolvedTheme,
    facet_scales: Option<FacetScales>,
    producer_text: Option<&'a [crate::render::backend::PlacedText]>,
    pub(crate) panel_text: Option<&'a [crate::decorations::panel_text::PanelTextPlan]>,
    pub(crate) self_managed_legend:
        Option<&'a Option<std::sync::Arc<crate::decorations::legend::Legend>>>,
    fit_scope: &'a crate::layout::fit::FitScope,
}

impl<'a> PlotRenderContext<'a> {
    pub(crate) fn new(
        data: crate::data::PlotData<'a>,
        panel: PanelBox,
        theme: &'a crate::theme::ResolvedTheme,
    ) -> Self {
        Self {
            data,
            panel,
            axis_label_width: (panel.width()
                - crate::layout::interior::DeferredLegacyPanelMinimum::SIZE.0)
                .max(0.0),
            retained_axes: None,
            retained_layout: None,
            selected_interior: None,
            prepared_axes: None,
            prepared_limits: [None; 2],
            prepared_kind: None,
            theme,
            facet_scales: None,
            producer_text: None,
            panel_text: None,
            self_managed_legend: None,
            fit_scope: &crate::layout::fit::ROOT_SCOPE,
        }
    }

    pub(crate) fn with_source_rows(
        data: crate::data::PlotData<'a>,
        source_rows: &'a [usize],
        panel: PanelBox,
        theme: &'a crate::theme::ResolvedTheme,
    ) -> Self {
        Self {
            data: data.map_source_rows(source_rows),
            panel,
            axis_label_width: (panel.width()
                - crate::layout::interior::DeferredLegacyPanelMinimum::SIZE.0)
                .max(0.0),
            retained_axes: None,
            retained_layout: None,
            selected_interior: None,
            prepared_axes: None,
            prepared_limits: [None; 2],
            prepared_kind: None,
            theme,
            facet_scales: None,
            producer_text: None,
            panel_text: None,
            self_managed_legend: None,
            fit_scope: &crate::layout::fit::ROOT_SCOPE,
        }
    }

    pub(crate) const fn with_facet_scales(mut self, facet_scales: FacetScales) -> Self {
        self.facet_scales = Some(facet_scales);
        self
    }

    pub(crate) const fn with_panel(mut self, panel: PanelBox) -> Self {
        self.panel = panel;
        self
    }

    pub(crate) const fn with_axis_label_width(mut self, width: f64) -> Self {
        self.axis_label_width = width;
        self
    }

    pub(crate) const fn with_fit_scope(mut self, scope: &'a crate::layout::fit::FitScope) -> Self {
        self.fit_scope = scope;
        self
    }
    pub(crate) const fn fit_scope(self) -> &'a crate::layout::fit::FitScope {
        self.fit_scope
    }

    pub(crate) const fn facet_scales(self) -> Option<FacetScales> {
        self.facet_scales
    }

    pub(crate) fn producer_text(self) -> Result<ProducerText<'a>, Error> {
        let mut lookup = std::collections::HashMap::new();
        if let Some(frozen) = self.producer_text {
            let environment = self.theme.prepared_font_environment()?;
            environment
                .claim_text_generation_slots(frozen.len() as u128)
                .map_err(|error| Error::Render(format!("producer lookup slots: {error:?}")))?;
            // Four bucket records per entry also covers the hash table's control bytes.
            let bucket_bytes = 4
                * (std::mem::size_of::<((&str, &str), &crate::layout::ResolvedTextPlan)>() + 1)
                + 16;
            environment
                .claim_unicode_bytes((frozen.len() * bucket_bytes) as u64)
                .map_err(|error| Error::Render(format!("producer lookup bytes: {error}")))?;
            lookup
                .try_reserve(frozen.len())
                .map_err(|_| Error::Render("producer lookup allocation failed".into()))?;
            for text in frozen {
                let owner = text.plan.style.provenance.owner.as_str();
                lookup
                    .entry((owner, text.plan.source.as_str()))
                    .or_insert(&text.plan);
            }
        }
        Ok(ProducerText {
            frozen: self.producer_text,
            lookup,
        })
    }

    pub(crate) fn with_retained_body(
        mut self,
        payload: crate::layout::interior::RetainedInteriorPayload<'a>,
    ) -> Self {
        let result = match payload {
            crate::layout::interior::RetainedInteriorPayload::LegacyCombined(result) => result,
            crate::layout::interior::RetainedInteriorPayload::Selected(result) => {
                self.selected_interior = Some(result);
                result
            }
        };
        self.retained_axes = result.axes.as_ref();
        self.retained_layout = Some(&result.layout);
        self.producer_text = Some(&result.producer_text);
        self.panel_text = Some(&result.panel_text);
        self.self_managed_legend = Some(&result.self_managed_legend);
        self
    }

    /// Annotation geometry may read the selected range but must not resolve it again.
    pub(crate) fn axis_range(
        self,
        side: crate::layout::AxisSide,
        prepare: impl FnOnce() -> Result<(f64, f64), Error>,
    ) -> Result<(f64, f64), Error> {
        if let Some(axes) = self.retained_axes {
            return Ok(axes.range(side));
        }
        let index = usize::from(side == crate::layout::AxisSide::Left);
        self.prepared_axes.map_or_else(prepare, |axes| {
            Ok(self.prepared_limits[index].unwrap_or(axes.ranges[index]))
        })
    }

    pub(crate) fn with_prepared_axes(mut self, plot: &'a crate::Plot) -> Self {
        if self.retained_layout.is_some() {
            return self;
        }
        self.prepared_axes = plot.prepared_axis_inputs();
        self.prepared_kind = Some(plot.kind());
        self.prepared_limits = [plot.x_axis_spec().limits, plot.y_axis_spec().limits];
        if plot.kind() == PlotKind::Forest && plot.axis_scale_spec() == AxisScale::Log10 {
            self.prepared_limits[0] =
                self.prepared_limits[0].map(|(lo, hi)| (lo.log10(), hi.log10()));
        }
        if let Some(shared) = plot.resolved_facet_x_range() {
            self.prepared_limits[0] = Some(shared);
        }
        self
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "same closed axis construction inputs as Layout"
    )]
    pub(crate) fn layout_with_axis_specs(
        self,
        x: (f64, f64),
        y: (f64, f64),
        panel: PanelBox,
        x_label: String,
        y_label: String,
        x_spec: &crate::layout::AxisSpec,
        y_spec: &crate::layout::AxisSpec,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<Layout, crate::layout::fit::PlanError> {
        self.layout_with_ticks(
            x,
            y,
            panel,
            x_label,
            y_label,
            Vec::new(),
            Vec::new(),
            x_spec,
            y_spec,
            theme,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "same closed axis construction inputs as Layout"
    )]
    pub(crate) fn layout_with_ticks(
        self,
        mut x: (f64, f64),
        mut y: (f64, f64),
        panel: PanelBox,
        x_label: String,
        y_label: String,
        mut x_ticks: Vec<crate::layout::AxisTick>,
        mut y_ticks: Vec<crate::layout::AxisTick>,
        x_spec: &crate::layout::AxisSpec,
        y_spec: &crate::layout::AxisSpec,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<Layout, crate::layout::fit::PlanError> {
        let resources = theme.prepared_font_environment()?.generation_resources();
        let data = self.data.with_axis_resources(&resources);
        let mut specs = [
            crate::layout::AxisSpec::default(),
            crate::layout::AxisSpec::default(),
        ];
        for (target, source) in specs.iter_mut().zip([x_spec, y_spec]) {
            *target = crate::layout::AxisSpec {
                provenance: source.provenance,
                limits: source.limits,
                interval: source.interval,
                exact_ticks: source
                    .exact_ticks
                    .as_ref()
                    .map(|ticks| {
                        data.collect_source(ticks.iter().map(|tick| {
                            Ok(crate::layout::AxisTick::new(
                                tick.position,
                                data.copy_source_string(&tick.label)?,
                            ))
                        }))
                    })
                    .transpose()?,
            };
        }
        if let Some(axes) = self.prepared_axes {
            // Heatmap keeps its text-dependent rotated category adapter. Its raw
            // labels are borrowed separately from this same preparation.
            if self.prepared_kind != Some(PlotKind::Heatmap) {
                for (side, (range, ticks)) in [(&mut x, &mut x_ticks), (&mut y, &mut y_ticks)]
                    .into_iter()
                    .enumerate()
                {
                    let geometry_expansion = matches!(
                        self.prepared_kind,
                        Some(
                            PlotKind::Bar
                                | PlotKind::StackedBar
                                | PlotKind::HorizontalBar
                                | PlotKind::StackedHorizontalBar
                                | PlotKind::Boxplot
                                | PlotKind::Violin
                                | PlotKind::Raincloud
                        )
                    );
                    if let Some(categories) = &axes.categories[side] {
                        *ticks = data.collect_source(categories.iter().map(|tick| {
                            Ok(crate::layout::AxisTick::new(
                                tick.position,
                                data.copy_source_string(&tick.label)?,
                            ))
                        }))?;
                        *range = axes.ranges[side];
                    } else if !geometry_expansion {
                        *range = self.prepared_limits[side].unwrap_or(axes.ranges[side]);
                    }
                    specs[side].limits = Some(*range);
                }
            }
        }
        Layout::new_with_ticks_and_axis_specs_in_panel_box(
            x,
            y,
            panel,
            x_label,
            y_label,
            x_ticks,
            y_ticks,
            &specs[0],
            &specs[1],
            theme,
            self.fit_scope,
        )
    }

    pub(crate) fn prepared_category_labels(self, side: usize) -> Option<&'a [String]> {
        self.prepared_axes?.display_categories[side].as_deref()
    }

    /// Capture consumes the selected layout; only measurement may select sources.
    #[expect(
        clippy::too_many_lines,
        reason = "closed native preparation consumption and admitted retained-layout replay"
    )]
    pub(crate) fn axis_layout(
        self,
        measure: impl FnOnce() -> Result<Layout, crate::layout::fit::PlanError>,
    ) -> Result<Layout, crate::layout::fit::PlanError> {
        if self.retained_layout.is_none() {
            if let (Some(axes), Some(kind)) = (self.prepared_axes, self.prepared_kind) {
                let standard = match kind {
                    PlotKind::Scatter
                    | PlotKind::Line
                    | PlotKind::Trajectory
                    | PlotKind::LinePath
                    | PlotKind::Histogram
                    | PlotKind::Density
                    | PlotKind::Qq
                    | PlotKind::Regression
                    | PlotKind::PointInterval
                    | PlotKind::Coefficient
                    | PlotKind::Forest
                    | PlotKind::Dumbbell => {
                        return Err(Error::Render(
                            "native axes require the coordinate producer".into(),
                        )
                        .into())
                    }
                    PlotKind::Interaction => true,
                    PlotKind::Bar
                    | PlotKind::StackedBar
                    | PlotKind::HorizontalBar
                    | PlotKind::StackedHorizontalBar
                    | PlotKind::Boxplot
                    | PlotKind::Violin
                    | PlotKind::Raincloud
                    | PlotKind::Heatmap
                    | PlotKind::Marginal
                    | PlotKind::Matrix
                    | PlotKind::Beeswarm
                    | PlotKind::Waterfall => false,
                };
                if standard {
                    let resources = self
                        .theme
                        .prepared_font_environment()?
                        .generation_resources();
                    let data = self.data.with_axis_resources(&resources);
                    let x_spec = &axes.specs[0];
                    let layout = self.layout_with_ticks(
                        axes.ranges[0],
                        axes.ranges[1],
                        self.panel,
                        data.copy_source_string(&axes.labels[0])?,
                        data.copy_source_string(&axes.labels[1])?,
                        Vec::new(),
                        Vec::new(),
                        x_spec,
                        &axes.specs[1],
                        self.theme,
                    )?;
                    return Ok(layout);
                }
            }
        }
        match self.retained_layout {
            Some(layout) => {
                let environment = self.theme.prepared_font_environment()?;
                let retain_string = |source: &str| -> Result<String, Error> {
                    environment
                        .claim_unicode_bytes(source.len() as u64)
                        .map_err(|error| Error::Render(error.to_string()))?;
                    let mut retained = String::new();
                    retained
                        .try_reserve_exact(source.len())
                        .map_err(|error| Error::Render(error.to_string()))?;
                    retained.push_str(source);
                    Ok(retained)
                };
                let retain_axis =
                    |axis: &crate::layout::Axis| -> Result<crate::layout::Axis, Error> {
                        let mut ticks = Vec::new();
                        crate::render::backend::reserve_text_records(
                            &mut ticks,
                            axis.ticks.len(),
                            environment,
                        )?;
                        for tick in &axis.ticks {
                            ticks.push(crate::layout::AxisTick {
                                position: tick.position,
                                label: retain_string(&tick.label)?,
                            });
                        }
                        Ok(crate::layout::Axis {
                            label: retain_string(&axis.label)?,
                            ticks,
                            explicit_ticks: axis.explicit_ticks,
                            assigned_tick_width: axis.assigned_tick_width,
                            source: axis.source,
                        })
                    };
                Ok(Layout {
                    projection: layout.projection.clone(),
                    x_axis: retain_axis(&layout.x_axis)?,
                    y_axis: retain_axis(&layout.y_axis)?,
                })
            }
            None => measure(),
        }
    }
}

#[cfg(test)]
#[test]
fn deferred_axis_consumption_admits_every_owned_source_copy() {
    use crate::layout::{AxisSpec, AxisTick};
    let plot = crate::Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .erase();
    let axes = crate::plots::shared_x::PreparedAxes {
        ranges: [(0.0, 2.0), (0.0, 1.0)],
        categories: [
            Some(vec![
                AxisTick::new(0.5, "first"),
                AxisTick::new(1.5, "second"),
            ]),
            None,
        ],
        display_categories: [None, None],
        labels: ["x title".into(), "y title".into()],
        specs: [
            AxisSpec::default(),
            AxisSpec {
                exact_ticks: Some(vec![AxisTick::new(0.5, "exact")]),
                ..AxisSpec::default()
            },
        ],
        logarithmic: false,
    };
    let consume = |theme: &crate::theme::ResolvedTheme| {
        let mut context =
            PlotRenderContext::new(plot.data(), PanelBox::new(0.0, 400.0, 0.0, 300.0), theme);
        context.prepared_axes = Some(&axes);
        context.prepared_kind = Some(PlotKind::Interaction);
        context.axis_layout(|| panic!("prepared axes must be consumed"))
    };
    let theme = crate::Theme::Publication.resolve();
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let before = (
        resources.usage().unwrap().1,
        resources.byte_usage().unwrap(),
    );
    let layout = consume(&theme).unwrap();
    assert_eq!(layout.x_axis.ticks, *axes.categories[0].as_ref().unwrap());
    assert_eq!(
        layout.y_axis.ticks,
        *axes.specs[1].exact_ticks.as_ref().unwrap()
    );
    let visits = resources.usage().unwrap().1 - before.0;
    let bytes = resources.byte_usage().unwrap() - before.1;
    // Two titles, two category records/strings, and two copies of the exact source
    // (temporary spec and final axis, whose reserve has a four-record minimum).
    // No font or final text-plan storage is involved.
    assert_eq!(
        bytes,
        (7 * std::mem::size_of::<AxisTick>() + 14 + 11 + 10) as u64
    );
    assert!(visits >= 14 + 2 + 11 + 1 + 5);
    for (remaining_visits, remaining_bytes, succeeds) in [
        (visits, bytes, true),
        (visits - 1, bytes, false),
        (visits, bytes - 1, false),
    ] {
        let theme = crate::Theme::Publication.resolve();
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        resources
            .claim_visits(1_000_000 - resources.usage().unwrap().1 - remaining_visits)
            .unwrap();
        resources
            .claim_bytes(536_870_912 - resources.byte_usage().unwrap() - remaining_bytes)
            .unwrap();
        let result = consume(&theme);
        if succeeds {
            result.unwrap();
        } else {
            assert!(matches!(
                result,
                Err(crate::layout::fit::PlanError::Fatal(Error::Render(_)))
            ));
        }
    }
}

/// Typography measured by body producers, retained across the final capture boundary.
pub(crate) struct ProducerText<'a> {
    frozen: Option<&'a [crate::render::backend::PlacedText]>,
    lookup: std::collections::HashMap<(&'a str, &'a str), &'a crate::layout::ResolvedTextPlan>,
}

impl ProducerText<'_> {
    pub(crate) fn plan(
        &self,
        source: &str,
        owner: &str,
        measure: impl FnOnce() -> Result<crate::layout::ResolvedTextPlan, crate::layout::fit::PlanError>,
    ) -> Result<crate::layout::ResolvedTextPlan, crate::layout::fit::PlanError> {
        if self.frozen.is_some() {
            return self
                .lookup
                .get(&(owner, source))
                .map(|plan| (*plan).clone())
                .ok_or_else(|| {
                    Error::Render(format!(
                        "frozen producer text is missing {owner}: {source:?}"
                    ))
                    .into()
                });
        }
        measure()
    }
}

/// Current constrained interior result. FitOutcome/PlanError distinguish measured,
/// terminal fit and fatal outcomes; provisional misses belong to FeedbackRound.
/// Point/path families retain selected marks and annotations; other marks remain combined.
#[derive(Debug)]
pub(crate) struct PlotRenderResult {
    pub(crate) layout: Layout,
    pub(crate) axes: Option<crate::layout::PanelAxisPlan>,
    pub(crate) x_domain: AxisDomain,
    pub(crate) y_domain: AxisDomain,
    pub(crate) significance: Option<SignificancePlan>,
    pub(crate) legend_entries: Vec<LegendEntry>,
    pub(crate) decoration_mode: DecorationMode,
    pub(crate) x_tick_keys: Option<Vec<String>>,
    pub(crate) y_tick_keys: Option<Vec<String>>,
    pub(crate) deferred_annotation_text: Vec<crate::layout::DeferredAnnotationTextPlan>,
    pub(crate) producer_text: Vec<crate::render::backend::PlacedText>,
    pub(crate) panel_text: Vec<crate::decorations::panel_text::PanelTextPlan>,
    pub(crate) self_managed_legend: Option<std::sync::Arc<crate::decorations::legend::Legend>>,
    pub(crate) matrix_axes: Option<std::sync::Arc<crate::plots::matrix::MatrixAxisPlan>>,
    pub(crate) marks: Option<std::sync::Arc<crate::layout::interior::DeferredMarkPlan>>,
    pub(crate) annotations: Option<std::sync::Arc<crate::layout::interior::DeferredAnnotationPlan>>,
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
    pub(crate) fn retained_payload(&self) -> crate::layout::interior::RetainedInteriorPayload<'_> {
        if self.marks.is_some() {
            crate::layout::interior::RetainedInteriorPayload::Selected(self)
        } else {
            crate::layout::interior::RetainedInteriorPayload::LegacyCombined(self)
        }
    }

    pub(crate) fn new(layout: Layout, legend_entries: Vec<LegendEntry>) -> Self {
        Self {
            layout,
            axes: None,
            x_domain: AxisDomain::Continuous,
            y_domain: AxisDomain::Continuous,
            significance: None,
            legend_entries,
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
        }
    }

    pub(crate) fn with_axis_domains(mut self, x: AxisDomain, y: AxisDomain) -> Self {
        self.x_domain = x;
        self.y_domain = y;
        self
    }

    pub(crate) fn with_significance_plan(mut self, significance: SignificancePlan) -> Self {
        self.significance = Some(significance);
        self
    }

    pub(crate) fn with_deferred_annotation_text(
        mut self,
        plans: Vec<crate::layout::DeferredAnnotationTextPlan>,
    ) -> Self {
        self.deferred_annotation_text = plans;
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
