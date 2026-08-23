use lichtung_plot::{BarInput, BarSpacing, CategoryOrder, Labels, Plot, ValueSort};

#[path = "../support/data.rs"]
mod data;
use data::{df, DataFrame};

#[path = "../support/visual.rs"]
mod support;

use support::*;

mod dumbbell;
mod grouping;
mod plots;
