use lichtung_plot::{
    AxisTick, Canvas, EffectBandSemantics, GuideSpec, GuideStyle, InferentialScale, IntervalKind,
    IntervalSemantics, LineMode, LinePattern, Plot, PointLayer,
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
