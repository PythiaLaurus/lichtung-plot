use lichtung_plot::{
    AxisScale, Chart, CoefficientSignificanceMark, EstimateSource, InferentialScale, IntervalKind,
    IntervalSemantics, NumericLabelFormat,
};
use polars::prelude::*;

#[path = "../support/composition.rs"]
mod support;

use support::*;

mod estimate_status;
mod model_displays;
mod point_interval;
