use lichtung_plot::{
    AxisTick, Bounds, Canvas, CanvasLegendPolicy, FacetSpec, GuideSpec, GuideStyle, LinePattern,
    Plot, PointLayer,
};

#[path = "../support/data.rs"]
mod data;
use data::{df, DataFrame};

#[path = "../support/composition.rs"]
mod support;
use support::*;

mod effect_bands;
mod numeric_layers;
mod paths;
