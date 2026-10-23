//! A strongly typed scientific plotting and SVG rendering library.
//!
//! Complete immutable theme values and their built-in constructors are available
//! from the crate root:
//! ```
//! let _ = lichtung_plot::Theme::publication();
//! ```
//!
//! Canvas owns one optional-to-call complete root theme. Unthemed child Plots inherit
//! it, while an explicit Plot theme replaces it locally:
//! ```
//! use lichtung_plot::{Canvas, NumericValues, Plot, Theme};
//!
//! let canvas = Canvas::grid(1, 2)?
//!     .theme(Theme::nature())
//!     .panel(0, 0, Plot::scatter(
//!         NumericValues::new("x", [1.0, 2.0]),
//!         NumericValues::new("y", [1.0, 2.0]),
//!     ))?
//!     .panel(0, 1, Plot::scatter(
//!         NumericValues::new("x", [1.0, 2.0]),
//!         NumericValues::new("y", [2.0, 1.0]),
//!     ).theme(Theme::science()))?;
//! # let _ = canvas;
//! # Ok::<(), lichtung_plot::Error>(())
//! ```
//!
//! Its implementation module is private:
//! ```compile_fail
//! use lichtung_plot::theme::Theme;
//! ```
//!
//! Decoration rendering is also private:
//! ```compile_fail
//! use lichtung_plot::decorations::title_block::TitleBlock;
//! ```
//!
//! ```compile_fail
//! use lichtung_plot::layout::Layout;
//! ```
//!
//! Alpha inputs are exported from the root; internal targets and preparation stay private:
//! ```compile_fail
//! use lichtung_plot::alpha::AlphaTarget;
//! ```

#![deny(missing_docs)]

mod alpha;
mod builder;
mod canvas;
mod data;
mod decorations;
mod error;
mod font;
mod geometry;
mod input;
/// Plot-family markers and sealed configuration capabilities.
pub mod kind;
mod labels;
mod layers;
mod layout;
mod numeric;
mod orchestrator;
mod output;
mod plots;
mod render;
mod scene;
mod significance;
mod spec;
/// Display-only statistics (KDE, CI, OLS for fit lines).
pub mod stats;
mod text;
mod theme;
mod values;

pub use alpha::{AlphaEncoding, AlphaSemantics, AlphaValues};
pub use builder::Plot;
pub use canvas::{Canvas, CanvasCategoryLabels, TextPanel};
pub use error::{Error, Result};
pub use font::canonical::{FontFamily, FontStack, GenericFontFamily};
pub use geometry::{
    HistogramBin, HistogramDimensions, HistogramGeometry, HistogramKdePoint, HistogramRug,
    RaincloudDensityPoint, RaincloudDimensions, RaincloudGeometry, RaincloudGroup, RaincloudPoint,
};
pub use input::{AxisValues, HistogramInput, RaincloudInput};
pub use labels::Labels;
pub use layers::{BarPointLayer, GuideSpec, GuideStyle, PointLayer};
pub use layout::AxisTick;
pub use numeric::decimal::{NegativeZero, NumericNotation, TrailingZeros};
pub use numeric::format::{NumericFormat, NumericPrecision};
pub use output::Output;
pub use render::backend::StrokePattern;
pub use significance::{
    Attached, FacetCell, Pairwise, Significance, SignificanceMark, SignificanceTarget,
};
pub use spec::{
    AxisScale, BarInput, BarSamples, BarSpacing, Bootstrap, Bounds, CanvasLegendPolicy,
    CategoricalColorMap, CategoryOrder, CoefficientInput, ConfidenceInterval, DumbbellEndpoints,
    ErrorBar, EstimateStatus, FacetScales, FacetSpec, HeatmapAnnotations, HeatmapLabelFit,
    HeatmapLayout, HeatmapNullRowAnnotation, HistogramBins, HistogramComposite, HistogramKde,
    HistogramStat, InteractionStyle, LegendComposition, LegendPolicy, LegendPosition,
    LinePathStyle, LinePattern, PanelCorner, PanelTextAnnotation, PercentileInterval, PlotKind,
    PointShape, QqLine, RawOverlay, RegressionBand, SignificanceMarkStyle, StandardDeviation,
    StandardError, TextWeight, TickGeneration, ValueSort, WaterfallLabels,
};
pub use spec::{PngCompression, PngOptions, PngSizing};
pub use theme::{
    AccentPalette, AnnotationTextRoles, AnnotationTheme, AxisSpacing, AxisTextRoles, AxisTheme,
    BandFillRoles, BodyFillRoles, CategoricalColorScale, CategoryMarkGeometry,
    CellAnnotationGeometry, CellMarkRoles, Color, ColorDomain, ColorScales, ColorStop,
    CompositionTextRoles, CompositionTheme, ConnectorStrokeRoles, ContinuousColorScale,
    ContinuousScaleKind, ContrastTextRoles, CurveStrokeRoles, DataFillTheme, DataLineRoles,
    DataLineTheme, DataOutlineTheme, DensityFillRoles, DivergingGradient,
    EstimateAnnotationGeometry, FacetStripSpacing, FillRoles, FixedFillTheme, FrameSides,
    FrameTheme, GradientLengths, GuideTextRoles, HeaderSpacing, LegendFrameTheme, LegendSpacing,
    LegendTheme, MarkExtentTheme, MarkTheme, NeutralPaintRoles, OpaqueColor, OutlineTheme,
    PaletteCapacity, PointOutlineTheme, PointRadii, PointRoles, PointTheme, QualitativePalette,
    ReferenceStrokeRoles, RelativeSpacing, RootPadding, RootTextRoles, RootTitleSpacing,
    SequentialGradient, SignificanceGeometry, SizedPointTheme, SolidStrokeTheme, StrokeRoles,
    StrokeTheme, SummaryStrokeRoles, SupplementalPointTheme, SurfaceTheme, TextPanelSpacing,
    TextTheme, Theme, ThemeBuilder, TickTheme, TitleAlignment, TypographyTheme, ValueShelfSpacing,
};
pub use values::{BooleanValues, CategoryValues, NumericValues, SourceRows};
