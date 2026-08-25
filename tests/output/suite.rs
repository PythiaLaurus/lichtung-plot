#[cfg(feature = "text-as-path")]
use lichtung_plot::SvgText;
use lichtung_plot::{Canvas, Plot};
#[cfg(feature = "png")]
use lichtung_plot::{HistogramBins, HistogramStat, PngCompression, PngOptions, PngSizing};
#[path = "../support/data.rs"]
mod data;
use data::{df, DataFrame};

#[cfg(feature = "png")]
#[path = "../support/visual.rs"]
mod support;

#[cfg(feature = "png")]
use support::*;

mod multiformat;
#[cfg(feature = "png")]
mod png;
