//! Built-in publication themes and private appearance resolution.

mod color;
mod contrast;
mod paint;
mod resolved;
mod scale;
mod spacing;

pub(crate) use scale::{
    AccentColors, CategoryColors, ContinuousColors, PaletteCache, PaletteSource,
    BEESWARM_SEQUENTIAL_STOPS, HEATMAP_DIVERGING_STOPS, HEATMAP_SEQUENTIAL_STOPS,
};

#[cfg(test)]
pub(crate) use resolved::FillOutlinePaint;
pub use resolved::Theme;
#[cfg(test)]
pub(crate) use resolved::THEME_RESOLUTION_CALLS;
pub(crate) use resolved::{
    FillOutlineAppearance, ResolvedTheme, SurfaceTheme, TextTheme, TitleAlignment,
};

pub(crate) use paint::{Color, OpaqueColor};
pub(crate) use spacing::{LocalMetric, RelativeSpacing};

/// Convert the current RGB tuple inputs at their private paint boundary.
pub(crate) fn color_from_rgb((red, green, blue): (u8, u8, u8)) -> Color {
    OpaqueColor::rgb(red, green, blue).into()
}
