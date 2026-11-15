use lichtung_plot::{
    Bounds, Canvas, CanvasLegendPolicy, CategoricalColorMap, CategoricalColorScale, CategoryValues,
    FacetSpec, InteractionStyle, Labels, LegendPolicy, LinePattern, NumericValues, OpaqueColor,
    Plot, PointShape,
};

#[path = "../support/composition.rs"]
mod support;
use support::*;

mod connectors;
mod styles;
