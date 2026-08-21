#[cfg(feature = "png")]
use lichtung_plot::{Chart, HistogramBins, HistogramStat, PngOptions, PngSizing};
#[cfg(feature = "png")]
use polars::prelude::*;

#[cfg(feature = "png")]
#[path = "../support/visual.rs"]
mod support;

#[cfg(feature = "png")]
use support::*;

mod multiformat;
#[cfg(feature = "png")]
mod png;
