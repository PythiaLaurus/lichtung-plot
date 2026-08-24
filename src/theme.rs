//! Publication themes: hardcoded palettes per `Theme` variant.

mod palette;

pub use palette::Theme;
pub(crate) use palette::{rgb_to_hex, ThemePalette, TICK_FONT_SIZE};
