use lichtung_plot::{
    BarAggregation, Canvas, CanvasCategoryLabels, CanvasLegendPolicy, CategoryOrder,
    EstimateSource, FacetScales, InferentialScale, LegendPosition, Plot, Significance,
    SignificanceMark, SignificanceTarget,
};

#[path = "../support/data.rs"]
mod data;
use data::{df, DataFrame};

#[path = "../support/composition.rs"]
mod support;

use support::*;

mod facets;
mod figures;
