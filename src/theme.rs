//! Built-in publication themes and private appearance resolution.

mod resolved;

pub use resolved::Theme;
pub(crate) use resolved::{rgb_to_hex, ResolvedTheme, TICK_FONT_SIZE};
