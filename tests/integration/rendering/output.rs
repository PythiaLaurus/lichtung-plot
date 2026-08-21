#[cfg(feature = "png")]
use super::*;

#[path = "output/multiformat.rs"]
mod multiformat;
#[cfg(feature = "png")]
#[path = "output/png.rs"]
mod png;
