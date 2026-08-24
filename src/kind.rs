//! Plot-family markers and sealed configuration capabilities.
//!
//! Constructors return `Plot` values carrying one of these zero-sized markers.
//! [`crate::kind::Any`] is the explicit erased marker used by heterogeneous storage.

mod sealed {
    pub trait Sealed {}
}

/// A marker admitted as the family parameter of [`crate::Plot`].
///
/// This trait is sealed; callers can use its bound but cannot add plot families.
pub trait Marker: sealed::Sealed + Clone {}

macro_rules! markers {
    ($($name:ident => $documentation:literal),+ $(,)?) => {
        $(
            #[doc = $documentation]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct $name;

            impl sealed::Sealed for $name {}
            impl Marker for $name {}
        )+
    };
}

markers! {
    Any => "The erased plot-family marker used by heterogeneous storage.",
    Scatter => "Scatter-plot family marker.",
    Histogram => "Histogram family marker for sample-derived or caller-computed geometry.",
    Density => "Density-plot family marker.",
    Qq => "Normal Q-Q plot family marker.",
    PointInterval => "Point-interval family marker.",
    Line => "Ordinary line-series family marker.",
    Trajectory => "Repeated-trajectory family marker.",
    LinePath => "Source-ordered line-path family marker.",
    Interaction => "Interaction-profile family marker.",
    Coefficient => "Coefficient-plot family marker.",
    Forest => "Forest-plot family marker.",
    Dumbbell => "Dumbbell-plot family marker.",
    Heatmap => "Heatmap family marker.",
    Bar => "Vertical ordinary-bar family marker.",
    HorizontalBar => "Horizontal ordinary-bar family marker.",
    StackedBar => "Vertical stacked-bar family marker.",
    StackedHorizontalBar => "Horizontal stacked-bar family marker.",
    Boxplot => "Boxplot family marker.",
    Violin => "Mirrored-violin family marker.",
    Raincloud => "Raincloud family marker for sample-derived or caller-computed geometry.",
    Regression => "Regression-plot family marker.",
    Marginal => "Marginal-plot family marker.",
    Matrix => "Scatter-matrix family marker.",
    Beeswarm => "Beeswarm-plot family marker.",
    Waterfall => "Waterfall-plot family marker.",
}

macro_rules! capability {
    ($name:ident, $documentation:literal => [$($marker:ty),* $(,)?]) => {
        #[doc = $documentation]
        pub trait $name: Marker {}

        $(impl $name for $marker {})*
    };
}

capability!(SourceRows, "Plots that accept aligned source-row identities." => [
    Scatter, Histogram, Density, Qq, PointInterval, Line, Trajectory, LinePath, Interaction, Coefficient, Forest,
    Dumbbell, Heatmap, Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin,
    Raincloud, Regression, Marginal, Matrix, Beeswarm, Waterfall,
]);
capability!(Hue, "Plots that accept categorical color identity." => [
    Scatter, Density, Line, Trajectory, LinePath, Interaction, Bar, HorizontalBar, Boxplot, Violin,
    Raincloud, Marginal, Matrix,
]);
capability!(Group, "Plots that accept graphical grouping identity." => [
    Density, Line, Bar, HorizontalBar, Boxplot, Violin, Raincloud, Marginal, Matrix, Dumbbell,
]);
capability!(ContinuousColor, "Plots that configure a constructor-owned continuous color channel." => [
    Heatmap, Beeswarm,
]);
capability!(NumericX, "Plots that can expose a numeric horizontal axis." => [
    Scatter, Histogram, Density, Qq, PointInterval, Line, Trajectory, LinePath, Coefficient, Forest, Dumbbell,
    HorizontalBar, StackedHorizontalBar, Regression, Marginal, Beeswarm, Waterfall,
]);
capability!(NumericY, "Plots that can expose a numeric vertical axis." => [
    Scatter, Histogram, Density, Qq, Line, Trajectory, LinePath, Interaction, Bar, StackedBar,
    Boxplot, Violin, Raincloud, Regression, Marginal,
]);
capability!(Guides, "Plots with at least one numeric axis that accept explicit guides." => [
    Scatter, Histogram, Density, Qq, PointInterval, Line, Trajectory, LinePath, Interaction, Coefficient, Forest,
    Dumbbell, Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin, Regression,
    Raincloud, Marginal, Beeswarm, Waterfall,
]);
capability!(HistogramOptions, "Histogram-specific configuration." => [Histogram]);
capability!(QqOptions, "Q-Q-specific configuration." => [Qq]);
capability!(HeatmapOptions, "Heatmap-specific configuration." => [Heatmap]);
capability!(BeeswarmOptions, "Beeswarm-specific continuous-color configuration." => [Beeswarm]);
capability!(BarSpacing, "Bar-family spacing configuration." => [
    Bar, HorizontalBar,
]);
capability!(BarPointLayers, "Ordinary bars that accept supplemental point layers." => [
    Bar, HorizontalBar,
]);
capability!(CategoryOrder, "Plots that consume one primary categorical order." => [
    PointInterval, Bar, HorizontalBar, StackedBar, StackedHorizontalBar,
]);
capability!(SignedBars, "Bar plots that consume sign-based coloring." => [HorizontalBar]);
capability!(ZeroLine, "Plots that consume an explicit zero reference line." => [
    PointInterval, HorizontalBar, StackedHorizontalBar, Beeswarm,
]);
capability!(LineOptions, "Ordinary line-series band configuration." => [Line]);
capability!(PointLayers, "Ordinary line series that accept supplemental point layers." => [Line]);
capability!(LinePathOptions, "Source-ordered line-path configuration." => [LinePath]);
capability!(EstimateStatus, "Estimate displays that accept availability status." => [
    PointInterval, Coefficient, Forest,
]);
capability!(Markers, "Line-like plots with configurable estimate markers." => [Line, LinePath, Interaction]);
capability!(RegressionOptions, "Regression-specific band configuration." => [Regression]);
capability!(CoefficientOptions, "Coefficient-specific labels and reference configuration." => [
    Coefficient,
]);
capability!(ForestOptions, "Forest-specific axis-scale configuration." => [Forest]);
capability!(InteractionOptions, "Interaction-specific connector and style configuration." => [
    Interaction,
]);
capability!(CategoricalColors, "Plots that consume caller-owned categorical colors." => [
    LinePath, Interaction, Violin, Raincloud,
]);
capability!(PanelText, "Plots with one standard panel that accept corner text." => [
    Scatter, Histogram, Density, Qq, PointInterval, Line, Trajectory, LinePath, Interaction, Coefficient, Forest,
    Dumbbell, Heatmap, Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin,
    Raincloud, Regression, Marginal, Beeswarm, Waterfall,
]);
capability!(WaterfallOptions, "Waterfall-specific annotation configuration." => [Waterfall]);
capability!(PointOutline, "Point-rendering families with a configurable outline." => [
    Scatter, Density, Qq, PointInterval, Line, Trajectory, LinePath, Interaction, Coefficient, Forest,
    Dumbbell, Boxplot, Violin, Raincloud, Regression, Marginal, Matrix, Beeswarm,
]);
capability!(SignificanceStyle, "Plots with configurable significance-mark typography." => [
    Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin, Raincloud, Coefficient,
]);

/// Plot families that accept significance annotations with placement `P`.
///
/// This trait is sealed; its implementations are the exact supported
/// plot-family and placement combinations.
pub trait Significance<P>: Marker {}

impl Significance<crate::Attached> for Bar {}
impl Significance<crate::Attached> for HorizontalBar {}
impl Significance<crate::Attached> for Coefficient {}

impl Significance<crate::Pairwise> for Bar {}
impl Significance<crate::Pairwise> for HorizontalBar {}
impl Significance<crate::Pairwise> for StackedBar {}
impl Significance<crate::Pairwise> for StackedHorizontalBar {}
impl Significance<crate::Pairwise> for Boxplot {}
impl Significance<crate::Pairwise> for Violin {}
impl Significance<crate::Pairwise> for Raincloud {}
capability!(Legend, "Plot families that can produce a configurable legend." => [
    Scatter, Density, Line, Trajectory, LinePath, Interaction, Dumbbell, Bar, HorizontalBar,
    StackedBar, StackedHorizontalBar, Boxplot, Violin, Raincloud, Regression, Marginal, Matrix,
]);
capability!(RawOverlay, "Distribution plots that accept raw-observation overlays." => [
    Boxplot, Violin,
]);
capability!(Facet, "Plot families supported by generic faceting." => [
    Scatter, Histogram, Density, Qq, PointInterval, Line, Trajectory, LinePath, Interaction, Coefficient, Forest,
    Dumbbell, Heatmap, Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin,
    Raincloud, Regression, Beeswarm, Waterfall,
]);
