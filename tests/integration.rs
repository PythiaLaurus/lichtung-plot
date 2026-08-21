use lichtung_plot::layout::{deterministic_text_width, AxisSpec, Layout, PanelBox};
use lichtung_plot::render::backend::{
    Backend, BracketStyle, FillStyle, GradientStop, LayerRole, LineStyle, Point2D, PointStyle,
    SignificanceBracket, TextStyle,
};
use lichtung_plot::theme::Theme;
use lichtung_plot::Error;
use lichtung_plot::{
    AxisScale, BarAggregation, BarSpacing, Chart, CoefficientSignificanceMark, EstimateSource,
    FacetScales, Figure, FigureCategoryLabels, FigureLegendPolicy, HistogramBins, HistogramStat,
    InferentialScale, IntervalKind, IntervalSemantics, Labels, LegendPosition, NumericLabelFormat,
    QqLine, ValueSort,
};
use lichtung_plot::{ComparisonAnnotation, ComparisonEndpoint};
#[cfg(feature = "png")]
use lichtung_plot::{PngOptions, PngSizing};
use polars::prelude::*;

#[path = "integration/support/composition.rs"]
mod composition_support;
#[path = "integration/support/visual.rs"]
mod visual_support;

#[path = "integration/composition.rs"]
mod composition;
#[path = "integration/plots.rs"]
mod plots;
#[path = "integration/rendering.rs"]
mod rendering;
#[path = "integration/statistics.rs"]
mod statistics;
#[path = "integration/visual_regression.rs"]
mod visual_regression;
