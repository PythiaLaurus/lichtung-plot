use lichtung_plot::{
    BarAggregation, Chart, EstimateSource, FacetScales, Figure, FigureCategoryLabels,
    FigureLegendPolicy, InferentialScale, LegendPosition,
};
use lichtung_plot::{ComparisonAnnotation, ComparisonEndpoint};
use polars::prelude::*;

#[path = "../support/composition.rs"]
mod support;

use support::*;

mod facets;
mod figures;
