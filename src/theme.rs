//! Built-in publication themes and private appearance resolution.

mod contrast;
mod resolved;

#[cfg(test)]
pub(crate) use resolved::FillOutlinePaint;
pub use resolved::Theme;
#[cfg(test)]
pub(crate) use resolved::THEME_RESOLUTION_CALLS;
pub(crate) use resolved::{AxisSpacing, ResolvedTheme, TextAppearance, TitleAlignment};
