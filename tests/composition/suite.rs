use lichtung_plot::{
    BarInput, Bounds, Canvas, CanvasCategoryLabels, CanvasLegendPolicy, CategoryOrder,
    CoefficientInput, EstimateStatus, FacetCell, FacetScales, FacetSpec, GuideSpec, GuideStyle,
    LegendPolicy, LegendPosition, LinePattern, Plot, Significance, SignificanceMark,
    SignificanceTarget,
};

#[path = "../support/data.rs"]
mod data;
use data::{df, DataFrame};

#[path = "../support/composition.rs"]
mod support;

use support::*;

mod facets;
mod figures;
