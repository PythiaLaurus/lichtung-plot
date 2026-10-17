use lichtung_plot::{
    BarInput, Bounds, Canvas, CanvasCategoryLabels, CanvasLegendPolicy, CategoricalColorMap,
    CategoricalColorScale, CategoryOrder, CategoryValues, CoefficientInput, EstimateStatus,
    FacetCell, FacetScales, FacetSpec, GuideSpec, GuideStyle, Labels, LegendPolicy, LegendPosition,
    LinePattern, NumericValues, OpaqueColor, Plot, Significance, SignificanceMark,
    SignificanceTarget, Theme,
};

#[path = "../support/composition.rs"]
mod support;

use support::*;

mod canvases;
mod facets;
mod foundations;
mod themes;
