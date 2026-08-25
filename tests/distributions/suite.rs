use lichtung_plot::{
    CategoryValues, HistogramBins, HistogramInput, HistogramStat, NumericValues, Plot, PointLayer,
    QqLine, Theme,
};

#[path = "../support/visual.rs"]
mod support;

use support::*;

mod composites;
mod raw_overlays;
mod sample_plots;
