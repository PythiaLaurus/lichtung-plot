use lichtung_plot::{
    AxisScale, AxisTick, Bounds, CategoryOrder, CoefficientInput, FacetScales, FacetSpec,
    NumericLabelFormat, Plot, Significance, SignificanceMark, ValueSort,
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
