#[cfg(feature = "text-as-path")]
use lichtung_plot::SvgText;
use lichtung_plot::{Canvas, NumericValues, Plot};
#[cfg(feature = "png")]
use lichtung_plot::{HistogramBins, HistogramStat, PngCompression, PngOptions, PngSizing};
#[cfg(feature = "png")]
#[path = "../support/visual.rs"]
mod support;

#[cfg(feature = "png")]
use support::*;

mod multiformat;
#[cfg(feature = "png")]
mod png;
