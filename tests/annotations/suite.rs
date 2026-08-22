use lichtung_plot::{BarAggregation, ComparisonAnnotation, ComparisonEndpoint, Plot};

#[path = "../support/data.rs"]
mod data;
use data::{df, DataFrame};

#[path = "../support/composition.rs"]
mod support;

use support::*;

mod pairwise_significance;
mod panel_text;
mod significance_layout;
