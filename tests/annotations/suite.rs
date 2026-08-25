use lichtung_plot::{
    BarInput, CategoryValues, NumericValues, Plot, Significance, SignificanceMark,
    SignificanceTarget,
};

#[path = "../support/composition.rs"]
mod support;

use support::*;

mod pairwise_significance;
mod panel_text;
mod significance_layout;
