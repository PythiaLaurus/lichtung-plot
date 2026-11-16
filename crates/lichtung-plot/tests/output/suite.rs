use lichtung_plot::{NumericValues, Plot, PngCompression, PngOptions, PngSizing};

#[path = "../support/visual.rs"]
mod support;

use support::{fixture_values, png_dimensions, png_phys, tempdir, unique_dir};

mod multiformat;
mod paths;
mod png;
