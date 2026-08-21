use lichtung_plot::{BarAggregation, BarSpacing, Chart, Labels};
use polars::prelude::*;

#[path = "../support/visual.rs"]
mod support;

use support::*;

mod dumbbell;
mod grouping;
mod plots;
