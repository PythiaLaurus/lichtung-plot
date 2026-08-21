use lichtung_plot::layout::{deterministic_text_width, AxisSpec, Layout};
use lichtung_plot::{BarAggregation, Chart, Labels, LegendPosition, Theme};
use polars::prelude::*;

#[path = "../support/visual.rs"]
mod support;

use support::*;

mod layout_and_legends;
