use lichtung_plot::{
    AxisScale, CategoryOrder, CoefficientSignificanceMark, EstimateSource, InferentialScale,
    IntervalKind, IntervalSemantics, NumericLabelFormat, Plot, ValueSort,
};

#[path = "../support/data.rs"]
mod data;
use data::{df, DataFrame};

#[path = "../support/composition.rs"]
mod support;

use support::*;

mod estimate_status;
mod model_displays;
mod point_interval;
