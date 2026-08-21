use lichtung_plot::{Chart, HistogramBins, HistogramStat, QqLine, Theme};
use polars::prelude::*;

#[path = "../support/visual.rs"]
mod support;

use support::*;

mod composites;
mod raw_overlays;
mod sample_plots;
