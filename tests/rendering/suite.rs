use lichtung_plot::{BarAggregation, ComparisonAnnotation, Plot, ValueSort};

#[path = "../support/data.rs"]
mod data;
use data::{df, DataFrame};

#[path = "../support/visual.rs"]
mod support;

use support::*;

mod contract;
mod labels;
mod public_contracts;
