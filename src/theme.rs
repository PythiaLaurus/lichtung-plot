//! Built-in publication themes and private appearance resolution.

pub(crate) mod annotation;
pub(crate) mod axis;
mod color;
mod component;
pub(crate) mod composition;
mod contrast;
pub(crate) mod guide;
pub(crate) mod mark;
mod mark_paint;
mod paint;
mod resolved;
mod scale;
pub(crate) mod scale_values;
pub(crate) mod spacing;
mod typography;
mod value;

pub(crate) use annotation::AnnotationTheme;
pub(crate) use axis::{AxisLengths, AxisSpacing, AxisTheme, FrameSides, FrameTheme, TickTheme};
pub(crate) use composition::CompositionTheme;
#[cfg(test)]
pub(crate) use guide::LegendFrameTheme;
pub(crate) use guide::LegendTheme;
pub(crate) use mark::MarkTheme;
pub(crate) use typography::{
    AnnotationTextRoles, AxisTextRoles, CompositionTextRoles, ContrastTextRoles, GuideTextRoles,
    RootTextRoles, TypographyTheme,
};

pub(crate) use scale::{AccentColors, CategoryColors, ContinuousColors, PaletteCache};
pub(crate) use scale_values::{
    AccentPalette, CategoricalColorScale, ColorDomain, ColorScales, ColorStop,
    ContinuousColorScale, DivergingGradient, PaletteCapacity, QualitativePalette,
    SequentialGradient, HEATMAP_DIVERGING_STOPS, HEATMAP_SEQUENTIAL_STOPS,
};

pub(crate) use mark_paint::{
    DataFillTheme, DataLineTheme, DataOutlineTheme, FixedFillTheme, OutlineTheme,
    PointOutlineTheme, PointTheme, SolidStrokeTheme, StrokeTheme,
};
pub use resolved::Theme;
pub(crate) use resolved::{ResolvedTheme, SurfaceTheme, TextTheme, TitleAlignment};

pub use paint::Color;
pub(crate) use paint::OpaqueColor;
pub(crate) use spacing::{LocalMetric, RelativeSpacing};
pub(crate) use value::ThemeValue;

/// Convert the current RGB tuple inputs at their private paint boundary.
pub(crate) fn color_from_rgb((red, green, blue): (u8, u8, u8)) -> Color {
    OpaqueColor::rgb(red, green, blue).into()
}
