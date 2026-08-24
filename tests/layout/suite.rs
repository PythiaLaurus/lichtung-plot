use lichtung_plot::{AxisTick, BarInput, Labels, LegendPolicy, LegendPosition, Plot, PointLayer};

#[path = "../support/data.rs"]
mod data;
use data::df;

#[path = "../support/visual.rs"]
mod support;

use support::*;

mod layout_and_legends;
