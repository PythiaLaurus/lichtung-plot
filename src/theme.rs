//! Built-in publication themes and private appearance resolution.

pub(crate) mod axis;
mod color;
pub(crate) mod composition;
mod contrast;
pub(crate) mod guide;
mod mark_paint;
mod paint;
mod resolved;
mod scale;
pub(crate) mod scale_values;
pub(crate) mod spacing;
mod typography;

pub(crate) use axis::{AxisLengths, AxisSpacing, AxisTheme, FrameSides, FrameTheme, TickTheme};
pub(crate) use composition::CompositionTheme;
pub(crate) use guide::LegendTheme;
#[cfg(test)]
pub(crate) use guide::{GradientLengths, LegendFrameTheme, LegendSpacing};
pub(crate) use typography::{
    AnnotationTextRoles, AxisTextRoles, CompositionTextRoles, ContrastTextRoles, GuideTextRoles,
    RootTextRoles, TypographyTheme,
};

pub(crate) use scale::{AccentColors, CategoryColors, ContinuousColors, PaletteCache};
pub(crate) use scale_values::{
    AccentPalette, CategoricalColorScale, ColorDomain, ColorScales, ColorStop,
    ContinuousColorScale, ContinuousScaleKind, DivergingGradient, PaletteCapacity,
    QualitativePalette, SequentialGradient, BEESWARM_SEQUENTIAL_STOPS, HEATMAP_DIVERGING_STOPS,
    HEATMAP_SEQUENTIAL_STOPS,
};

pub(crate) use mark_paint::{
    DataFillTheme, DataLineTheme, DataOutlineTheme, FixedFillTheme, OutlineTheme,
    PointOutlineTheme, PointTheme, SolidStrokeTheme, StrokeTheme,
};
pub use resolved::Theme;
#[cfg(test)]
pub(crate) use resolved::THEME_RESOLUTION_CALLS;
pub(crate) use resolved::{ResolvedTheme, SurfaceTheme, TextTheme, TitleAlignment};

pub(crate) use paint::{Color, OpaqueColor};
pub(crate) use spacing::{LocalMetric, RelativeSpacing};

/// Convert the current RGB tuple inputs at their private paint boundary.
pub(crate) fn color_from_rgb((red, green, blue): (u8, u8, u8)) -> Color {
    OpaqueColor::rgb(red, green, blue).into()
}
