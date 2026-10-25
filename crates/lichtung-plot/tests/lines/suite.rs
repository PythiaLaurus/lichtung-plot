use lichtung_plot::{
    AxisTick, AxisValues, Bounds, Canvas, CanvasLegendPolicy, CategoryValues, FacetSpec, GuideSpec,
    GuideStyle, LinePattern, NumericValues, Plot, PointLayer,
};

#[path = "../support/composition.rs"]
mod support;
use support::*;

mod effect_bands;
mod numeric_layers;
mod paths;
