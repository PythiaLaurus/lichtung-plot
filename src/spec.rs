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

    pub(crate) fn format(&self, value: f64) -> String {
        self.display(value).to_string()
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
    pub(crate) theme: &'a crate::theme::ResolvedTheme,
    facet_scales: Option<FacetScales>,
    deferred_annotation_text: Option<&'a [crate::layout::DeferredAnnotationTextPlan]>,
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
            theme,
            facet_scales: None,
            deferred_annotation_text: None,
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
            theme,
            facet_scales: None,
            deferred_annotation_text: None,
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

    pub(crate) const fn with_deferred_annotation_text(
        mut self,
        plans: &'a [crate::layout::DeferredAnnotationTextPlan],
    ) -> Self {
        self.deferred_annotation_text = Some(plans);
        self
    }

    pub(crate) const fn deferred_annotation_text(
        self,
    ) -> Option<&'a [crate::layout::DeferredAnnotationTextPlan]> {
        self.deferred_annotation_text
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
            retained: Vec::new(),
            environment: self.theme,
        })
    }

    pub(crate) fn with_retained_body(mut self, result: &'a PlotRenderResult) -> Self {
        self.producer_text = Some(&result.producer_text);
        self.panel_text = Some(&result.panel_text);
        self.self_managed_legend = Some(&result.self_managed_legend);
        self
    }
}

/// Typography measured by body producers, retained across the final capture boundary.
pub(crate) struct ProducerText<'a> {
    frozen: Option<&'a [crate::render::backend::PlacedText]>,
    lookup: std::collections::HashMap<(&'a str, &'a str), &'a crate::layout::ResolvedTextPlan>,
    retained: Vec<crate::render::backend::PlacedText>,
    environment: &'a crate::theme::ResolvedTheme,
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

    pub(crate) fn draw(
        &mut self,
        text: crate::render::backend::PlacedText,
        backend: &mut dyn crate::render::backend::Backend,
    ) -> Result<(), Error> {
        let text = if let Some(frozen) = self.frozen {
            let retained = frozen
                .get(self.retained.len())
                .ok_or_else(|| Error::Render("extra frozen producer text call".into()))?;
            if retained.plan != text.plan || retained.placement != text.placement {
                return Err(Error::Render(
                    "frozen producer text identity changed".into(),
                ));
            }
            retained.clone()
        } else {
            text
        };
        let environment = self.environment.prepared_font_environment()?;
        crate::render::backend::reserve_text_records(&mut self.retained, 1, environment)?;
        text.draw(backend)?;
        self.retained.push(text);
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<Vec<crate::render::backend::PlacedText>, Error> {
        if self
            .frozen
            .is_some_and(|text| text.len() != self.retained.len())
        {
            return Err(Error::Render("missing frozen producer text calls".into()));
        }
        Ok(self.retained)
    }
}

pub(crate) struct PlotRenderResult {
    pub(crate) layout: Layout,
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
            significance: None,
            legend_entries,
            decoration_mode: DecorationMode::Standard,
            x_tick_keys: None,
            y_tick_keys: None,
            deferred_annotation_text: Vec::new(),
            producer_text: Vec::new(),
            panel_text: Vec::new(),
            self_managed_legend: None,
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
