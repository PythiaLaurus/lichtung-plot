use lichtung_plot::{
    BarInput, Canvas, CategoricalColorMap, CategoricalColorScale, CategoryValues, FacetCell,
    FacetSpec, Labels, LegendPolicy, NumericValues, OpaqueColor, Plot, Significance,
    SignificanceMark, SignificanceTarget, Theme,
};

#[path = "../support/composition.rs"]
mod support;

use support::{assert_unique_svg_ids, attribute, render_canvas, render_svg};

mod canvases;
mod facets;
mod foundations;
