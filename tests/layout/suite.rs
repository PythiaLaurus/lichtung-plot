use lichtung_plot::layout::{deterministic_text_width, AxisSpec, AxisTick, Layout};
use lichtung_plot::{BarAggregation, Labels, LegendPosition, Plot, PointLayer, Theme};

#[path = "../support/data.rs"]
mod data;
use data::df;

#[path = "../support/visual.rs"]
mod support;

use support::*;

mod layout_and_legends;
