//! A strongly typed scientific plotting and SVG rendering library.

#![deny(missing_docs)]

mod builder;
mod comparison;
mod data;
/// Non-data chart decorations: legends, titles, and captions.
pub mod decorations;
mod error;
mod figure;
mod font;
mod geometry;
mod labels;
mod layers;
/// Shared plot layout core: scales, projection, axes, frames.
pub mod layout;
mod orchestrator;
#[cfg(any(feature = "png", test))]
mod output_path;
mod plots;
/// `Backend` trait and the native SVG backend impl.
pub mod render;
mod scene;
mod significance;
mod spec;
/// Display-only statistics (KDE, CI, OLS for fit lines).
pub mod stats;
/// Publication themes (`Theme::Nature`, `Theme::Science`, ...).
pub mod theme;

pub use builder::Chart;
pub use comparison::{ComparisonAnnotation, ComparisonEndpoint, ComparisonFacet, SignificanceMark};
pub use error::{Error, Result};
pub use figure::{Figure, FigureCategoryLabels, TextPanel};
pub use font::{DEFAULT_FONT_FAMILY, DEFAULT_FONT_FAMILY_STACK, SYSTEM_FONT_FAMILIES};
pub use geometry::{
    HistogramBinInput, HistogramCompositeGeometryInput, HistogramDimensionsInput,
    HistogramKdePoint, HistogramRugInput, RaincloudDensityPoint, RaincloudDimensionsInput,
    RaincloudGeometryInput, RaincloudGroupInput, RaincloudPoint,
};
pub use labels::Labels;
pub use layers::{BarPointLayer, GuideSpec, GuideStyle, PointLayer};
pub use layout::AxisTick;
pub use scene::RenderedSvgScene;
pub use spec::{
    AxisScale, BarAggregation, BarSpacing, CategoricalColors, CoefficientSignificanceMark,
    DumbbellEndpoints, EffectBandCoverage, EffectBandSemantics, EstimateSource, EstimateStatus,
    FacetScales, FigureLegendPolicy, GroupedFactorAnnotation, GroupedFactorEndpoint,
    HeatmapAnnotations, HeatmapLabelFit, HeatmapLayout, HeatmapNullRowAnnotation, HeatmapScale,
    HistogramBins, HistogramComposite, HistogramKde, HistogramStat, InferentialScale, IntervalKind,
    IntervalSemantics, LegendPosition, LineMode, LinePathStyle, LinePattern, NumericLabelFormat,
    PanelCorner, PanelTextAnnotation, PlotKind, PngCompression, PngOptions, PngSizing, PointShape,
    QqLine, RawOverlay, RegressionBand, SignificanceMarkStyle, SignificanceStarLimit, TextWeight,
    ValueSort, ViolinLayout, WaterfallLabels,
};
pub use theme::Theme;
