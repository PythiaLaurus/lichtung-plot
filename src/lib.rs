//! A strongly typed scientific plotting and SVG rendering library.

#![deny(missing_docs)]

mod builder;
mod canvas;
mod data;
/// Non-data chart decorations: legends, titles, and captions.
pub mod decorations;
mod error;
mod font;
mod geometry;
mod input;
/// Plot-family markers and sealed configuration capabilities.
pub mod kind;
mod labels;
mod layers;
/// Shared plot layout core: scales, projection, axes, frames.
pub mod layout;
mod orchestrator;
mod output;
#[cfg(any(feature = "png", test))]
mod output_path;
mod plots;
mod render;
mod scene;
mod significance;
mod spec;
/// Display-only statistics (KDE, CI, OLS for fit lines).
pub mod stats;
#[cfg(test)]
mod test_support;
/// Publication themes (`Theme::Nature`, `Theme::Science`, ...).
pub mod theme;
mod values;

pub use builder::Plot;
pub use canvas::{Canvas, CanvasCategoryLabels, TextPanel};
pub use error::{Error, Result};
pub use font::{DEFAULT_FONT_FAMILY, DEFAULT_FONT_FAMILY_STACK, SYSTEM_FONT_FAMILIES};
pub use geometry::{
    HistogramBinInput, HistogramCompositeGeometryInput, HistogramDimensionsInput,
    HistogramKdePoint, HistogramRugInput, RaincloudDensityPoint, RaincloudDimensionsInput,
    RaincloudGeometryInput, RaincloudGroupInput, RaincloudPoint,
};
pub use input::{AxisValues, HistogramInput, RaincloudInput};
pub use labels::Labels;
pub use layers::{BarPointLayer, GuideSpec, GuideStyle, PointLayer};
pub use layout::AxisTick;
pub use output::{Output, SvgText};
pub use significance::{
    Attached, FacetCell, Pairwise, Significance, SignificanceMark, SignificanceTarget,
};
pub use spec::{
    AxisScale, BarInput, BarSamples, BarSpacing, Bootstrap, Bounds, CanvasLegendPolicy,
    CategoricalColors, CategoryOrder, CoefficientInput, ConfidenceInterval, DumbbellEndpoints,
    ErrorBar, EstimateStatus, FacetScales, FacetSpec, HeatmapAnnotations, HeatmapLabelFit,
    HeatmapLayout, HeatmapNullRowAnnotation, HeatmapScale, HistogramBins, HistogramComposite,
    HistogramKde, HistogramStat, InteractionStyle, LegendPolicy, LegendPosition, LinePathStyle,
    LinePattern, NumericLabelFormat, PanelCorner, PanelTextAnnotation, PercentileInterval,
    PlotKind, PngCompression, PngOptions, PngSizing, PointShape, QqLine, RawOverlay,
    RegressionBand, SignificanceMarkStyle, StandardDeviation, StandardError, TextWeight, ValueSort,
    WaterfallLabels,
};
pub use theme::Theme;
pub use values::{BooleanValues, CategoryValues, NumericValues, SourceRows};
