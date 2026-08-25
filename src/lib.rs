//! A strongly typed scientific plotting and SVG rendering library.
//!
//! The built-in theme type is available from the crate root:
//! ```
//! let _ = lichtung_plot::Theme::Publication;
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

#![cfg_attr(
    not(feature = "png"),
    doc = r#"
## SVG-only builds

PNG option types and terminal methods are absent without the `png` feature:

```compile_fail
use lichtung_plot::PngOptions;
```

```compile_fail
use lichtung_plot::{NumericValues, Plot};
let plot = Plot::scatter(
    NumericValues::new("x", [1.0]),
    NumericValues::new("y", [2.0]),
);
let _ = plot.png_options(());
```

```compile_fail
use lichtung_plot::{NumericValues, Plot};
let plot = Plot::scatter(
    NumericValues::new("x", [1.0]),
    NumericValues::new("y", [2.0]),
);
let _ = plot.png("plot.png");
```

```compile_fail
use lichtung_plot::{Canvas, NumericValues, Plot};
let plot = Plot::scatter(
    NumericValues::new("x", [1.0]),
    NumericValues::new("y", [2.0]),
);
let canvas = Canvas::grid(1, 1).unwrap().panel(0, 0, plot).unwrap();
let _ = canvas.png("canvas.png");
```

```compile_fail
use lichtung_plot::{NumericValues, Plot};
let output = Plot::scatter(
    NumericValues::new("x", [1.0]),
    NumericValues::new("y", [2.0]),
).svg("plot.svg").unwrap();
let _ = output.png("plot.png");
```
"#
)]
#![deny(missing_docs)]

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
mod orchestrator;
mod output;
mod plots;
mod render;
mod scene;
mod significance;
mod spec;
/// Display-only statistics (KDE, CI, OLS for fit lines).
pub mod stats;
#[cfg(test)]
mod test_support;
mod theme;
mod values;

pub use builder::Plot;
pub use canvas::{Canvas, CanvasCategoryLabels, TextPanel};
pub use error::{Error, Result};
pub use font::{DEFAULT_FONT_FAMILY, DEFAULT_FONT_FAMILY_STACK, SYSTEM_FONT_FAMILIES};
pub use geometry::{
    HistogramBin, HistogramDimensions, HistogramGeometry, HistogramKdePoint, HistogramRug,
    RaincloudDensityPoint, RaincloudDimensions, RaincloudGeometry, RaincloudGroup, RaincloudPoint,
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
    PlotKind, PointShape, QqLine, RawOverlay, RegressionBand, SignificanceMarkStyle,
    StandardDeviation, StandardError, TextWeight, ValueSort, WaterfallLabels,
};
#[cfg(feature = "png")]
pub use spec::{PngCompression, PngOptions, PngSizing};
pub use theme::Theme;
pub use values::{BooleanValues, CategoryValues, NumericValues, SourceRows};
