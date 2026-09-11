//! Built-in publication themes and private appearance resolution.

mod resolved;

pub use resolved::Theme;
#[cfg(test)]
pub(crate) use resolved::THEME_RESOLUTION_CALLS;
pub(crate) use resolved::{AxisSpacing, ResolvedTheme, TextAppearance, TitleAlignment};
