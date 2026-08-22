use lichtung_plot::{
    BarAggregation, Canvas, CanvasCategoryLabels, CanvasLegendPolicy, EstimateSource, FacetScales,
    InferentialScale, LegendPosition, Plot,
};
use lichtung_plot::{ComparisonAnnotation, ComparisonEndpoint};

#[path = "../support/data.rs"]
mod data;
use data::{df, DataFrame};

#[path = "../support/composition.rs"]
mod support;

use support::*;

mod facets;
mod figures;
