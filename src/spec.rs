//! Shared plot specification and render-result types.

use crate::layout::{Layout, PanelBox};
use crate::significance::SignificancePlan;
use crate::{HistogramKdePoint, NumericFormat};

mod bar;
mod categorical;
mod estimate;
mod facet;
mod output;
mod ticks;

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
pub use ticks::TickGeneration;

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

/// Caller-owned text block anchored inside a chart panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelTextAnnotation {
    corner: PanelCorner,
    text: crate::Text,
}

impl PanelTextAnnotation {
    /// Construct one validated panel-corner annotation from plain or styled text.
    ///
    /// Local appearance replaces the complete panel-text role. Alignment applies
    /// within the text block; the selected corner continues to place that block.
    pub fn new(corner: PanelCorner, text: impl Into<crate::Text>) -> Result<Self, crate::Error> {
        let text = text.into();
        crate::text::validate_for(text.as_str(), "panel text")?;
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

/// Display policy for categorical heatmap axis labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeatmapLabelFit {
    /// Preserve the dense-grid default by eliding labels that exceed one cell.
    #[default]
    Elide,
    /// Preserve complete labels and wrap x-axis text within each cell.
    Wrap,
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
        crate::text::validate_for(
            annotation.text.as_str(),
            "heatmap fully-null row annotation text",
        )?;
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
    base: String,
    terminal: String,
}

impl WaterfallLabels {
    /// Construct verbatim display labels for the base and terminal values.
    pub fn new(base: impl Into<String>, terminal: impl Into<String>) -> Result<Self, crate::Error> {
        let labels = Self {
            base: base.into(),
            terminal: terminal.into(),
        };
        crate::text::validate_for(&labels.base, "waterfall base label")?;
        crate::text::validate_for(&labels.terminal, "waterfall terminal label")?;
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

/// Shared immutable inputs for one plot renderer.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PlotRenderContext<'a> {
    pub(crate) data: crate::data::PlotData<'a>,
    pub(crate) panel: PanelBox,
    pub(crate) content_height: bool,
    // An embedded content-sized child still fills its parent's arranged row.
    // Its own preferred height is reported separately to that same root driver.
    pub(crate) fill_content_height: bool,
    // Derived inside the containing-frame measure, never supplied by callers.
    pub(crate) content_preference: Option<f64>,
    // Parent-owned perpendicular capacity; axis feedback must not shrink it a
    // second time merely because the assigned mark panel became narrower.
    pub(crate) axis_label_width: f64,
    pub(crate) retained_axes: Option<&'a crate::layout::PanelAxisPlan>,
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
    pub(crate) guides: &'a crate::decorations::legend::discrete::scoped::GuideConsumption<'a>,
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
            content_height: false,
            fill_content_height: false,
            content_preference: None,
            axis_label_width: panel.width().max(0.0),
            retained_axes: None,
            selected_interior: None,
            prepared_axes: None,
            prepared_limits: [None; 2],
            prepared_kind: None,
            theme,
            facet_scales: None,
            producer_text: None,
            panel_text: None,
            self_managed_legend: None,
            guides: &crate::decorations::legend::discrete::scoped::SUPPRESSED_GUIDES,
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
            content_height: false,
            fill_content_height: false,
            content_preference: None,
            axis_label_width: panel.width().max(0.0),
            retained_axes: None,
            selected_interior: None,
            prepared_axes: None,
            prepared_limits: [None; 2],
            prepared_kind: None,
            theme,
            facet_scales: None,
            producer_text: None,
            panel_text: None,
            self_managed_legend: None,
            guides: &crate::decorations::legend::discrete::scoped::SUPPRESSED_GUIDES,
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

    pub(crate) const fn with_guides(
        mut self,
        guides: &'a crate::decorations::legend::discrete::scoped::GuideConsumption<'a>,
    ) -> Self {
        self.guides = guides;
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
            // Four bucket records per entry also covers the hash table's control bytes.

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

    pub(crate) fn with_retained_body(mut self, result: &'a PlotRenderResult) -> Self {
        self.selected_interior = Some(result);
        self.retained_axes = result.axes.as_ref();
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

    pub(crate) fn with_prepared_axes(mut self, plot: &'a crate::Plot) -> Result<Self, Error> {
        if self.selected_interior.is_some() {
            return Ok(self);
        }
        self.prepared_axes = plot.prepared_axis_inputs();
        self.prepared_kind = Some(plot.kind());
        self.prepared_limits = [plot.x_axis_spec().limits, plot.y_axis_spec().limits];
        if plot.kind() == PlotKind::Forest && plot.axis_scale_spec() == AxisScale::Log10 {
            if let Some((lo, hi)) = self.prepared_limits[0] {
                self.prepared_limits[0] =
                    Some((crate::numeric::log10(lo)?, crate::numeric::log10(hi)?));
            }
        }
        if let Some(shared) = plot.resolved_facet_x_range() {
            self.prepared_limits[0] = Some(shared);
        }
        Ok(self)
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
        x_label: Option<crate::Text>,
        y_label: Option<crate::Text>,
        x_spec: &crate::layout::AxisSpec,
        y_spec: &crate::layout::AxisSpec,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<Layout, crate::layout::fit::PlanError> {
        if self.prepared_kind.is_some_and(|kind| {
            crate::layout::interior::InteriorRoute::classify(kind).is_coordinate()
        }) {
            return Err(Error::Render("native axes require the coordinate producer".into()).into());
        }
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
        x_label: Option<crate::Text>,
        y_label: Option<crate::Text>,
        mut x_ticks: Vec<crate::layout::AxisTick>,
        mut y_ticks: Vec<crate::layout::AxisTick>,
        x_spec: &crate::layout::AxisSpec,
        y_spec: &crate::layout::AxisSpec,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<Layout, crate::layout::fit::PlanError> {
        let mut specs = [
            crate::layout::AxisSpec::default(),
            crate::layout::AxisSpec::default(),
        ];
        for (target, source) in specs.iter_mut().zip([x_spec, y_spec]) {
            *target = crate::layout::AxisSpec {
                provenance: source.provenance,
                limits: source.limits,
                generation: source.generation,
                format: source.format.clone(),
                exact_ticks: source
                    .exact_ticks
                    .as_ref()
                    .map(|ticks| {
                        crate::data::source::collect_source(ticks.iter().map(|tick| {
                            Ok(crate::layout::AxisTick::new(
                                tick.position,
                                crate::data::source::copy_source_string(&tick.label)?,
                            ))
                        }))
                    })
                    .transpose()?,
            };
        }
        if let Some(axes) = self.prepared_axes {
            for (side, (range, ticks)) in [(&mut x, &mut x_ticks), (&mut y, &mut y_ticks)]
                .into_iter()
                .enumerate()
            {
                if let Some(categories) = &axes.categories[side] {
                    *ticks = crate::data::source::collect_source(categories.iter().map(|tick| {
                        Ok(crate::layout::AxisTick::new(
                            tick.position,
                            crate::data::source::copy_source_string(&tick.label)?,
                        ))
                    }))?;
                    *range = axes.ranges[side];
                } else {
                    *range = self.prepared_limits[side].unwrap_or(axes.ranges[side]);
                }
                specs[side].limits = Some(*range);
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
    pub(crate) panel_text: Vec<crate::decorations::panel_text::PanelTextPlan>,
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
