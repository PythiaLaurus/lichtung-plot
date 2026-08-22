use lichtung_plot::{
    Canvas, CanvasLegendPolicy, CategoricalColors, EstimateSource, InferentialScale, IntervalKind,
    IntervalSemantics, Labels, LinePattern, Plot, PointShape,
};

#[path = "../support/data.rs"]
mod data;
use data::{df, DataFrame};

#[path = "../support/composition.rs"]
mod support;
use support::*;

mod connectors;
mod styles;
