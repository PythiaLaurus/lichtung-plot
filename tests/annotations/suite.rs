use lichtung_plot::{BarAggregation, Chart, ComparisonAnnotation, ComparisonEndpoint};
use polars::prelude::*;

#[path = "../support/composition.rs"]
mod support;

use support::*;

mod pairwise_significance;
mod panel_text;
mod significance_layout;
