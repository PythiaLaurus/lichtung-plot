//! Publication themes: hardcoded palettes per `Theme` variant.

/// Hardcoded publication palettes.
pub mod palette;

pub(crate) use palette::TICK_FONT_SIZE;
pub use palette::{rgb_to_hex, Theme, ThemePalette};
