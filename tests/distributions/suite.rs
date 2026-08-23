use lichtung_plot::{
    HistogramBins, HistogramInput, HistogramStat, Plot, PointLayer, QqLine, Theme,
};

#[path = "../support/data.rs"]
mod data;
use data::{df, DataFrame, Series};

#[path = "../support/visual.rs"]
mod support;

use support::*;

mod composites;
mod raw_overlays;
mod sample_plots;
