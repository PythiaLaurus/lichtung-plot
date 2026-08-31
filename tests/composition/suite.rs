use lichtung_plot::{
    BarInput, Bounds, Canvas, CanvasCategoryLabels, CanvasLegendPolicy, CategoryOrder,
    CategoryValues, CoefficientInput, EstimateStatus, FacetCell, FacetScales, FacetSpec, GuideSpec,
    GuideStyle, LegendPolicy, LegendPosition, LinePattern, NumericValues, Plot, Significance,
    SignificanceMark, SignificanceTarget, Theme,
};

#[path = "../support/composition.rs"]
mod support;

use support::*;

mod canvases;
mod facets;
mod themes;
