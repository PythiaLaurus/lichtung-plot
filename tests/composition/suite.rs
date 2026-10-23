use lichtung_plot::{
    BarInput, Bounds, Canvas, CanvasCategoryLabels, CanvasLegendPolicy, CategoricalColorMap,
    CategoricalColorScale, CategoryOrder, CategoryValues, FacetCell, FacetScales, FacetSpec,
    Labels, LegendPolicy, LegendPosition, NumericValues, OpaqueColor, Plot, Significance,
    SignificanceMark, SignificanceTarget, Theme,
};

#[path = "../support/composition.rs"]
mod support;

use support::*;

mod canvases;
mod facets;
mod foundations;
mod themes;
