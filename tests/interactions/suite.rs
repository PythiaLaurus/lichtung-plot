use lichtung_plot::{
    Bounds, Canvas, CanvasLegendPolicy, CategoricalColors, CategoryValues, FacetSpec,
    InteractionStyle, Labels, LegendPolicy, LinePattern, NumericValues, Plot, PointShape,
};

#[path = "../support/composition.rs"]
mod support;
#[path = "../support/visual.rs"]
mod visual;
use support::*;

mod connectors;
mod styles;
