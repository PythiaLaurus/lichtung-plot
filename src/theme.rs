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

pub use annotation::{
    AnnotationTheme, CellAnnotationGeometry, EstimateAnnotationGeometry, SignificanceGeometry,
    ValueShelfSpacing,
};
pub(crate) use axis::AxisLengths;
pub use axis::{AxisSpacing, AxisTheme, FrameSides, FrameTheme, TickTheme};
pub use composition::{
    CompositionTheme, FacetStripSpacing, HeaderSpacing, RootPadding, RootTitleSpacing,
    TextPanelSpacing,
};
pub use guide::{GradientLengths, LegendFrameTheme, LegendSpacing, LegendTheme};
pub use mark::{
    BandFillRoles, BodyFillRoles, CategoryMarkGeometry, CellMarkRoles, ConnectorStrokeRoles,
    CurveStrokeRoles, DataLineRoles, DensityFillRoles, FillRoles, MarkExtentTheme, MarkTheme,
    NeutralPaintRoles, PointRadii, PointRoles, ReferenceStrokeRoles, SizedPointTheme, StrokeRoles,
    SummaryStrokeRoles, SupplementalPointTheme,
};
pub use typography::{
    AnnotationTextRoles, AxisTextRoles, CompositionTextRoles, ContrastTextRoles, GuideTextRoles,
    RootTextRoles, TypographyTheme,
};

pub(crate) use scale::{AccentColors, CategoryColors, ContinuousColors, PaletteCache};
pub use scale_values::{
    AccentPalette, CategoricalColorScale, ColorDomain, ColorScales, ColorStop,
    ContinuousColorScale, ContinuousScaleKind, DivergingGradient, PaletteCapacity,
    QualitativePalette, SequentialGradient,
};
pub(crate) use scale_values::{HEATMAP_DIVERGING_STOPS, HEATMAP_SEQUENTIAL_STOPS};

pub use mark_paint::{
    DataFillTheme, DataLineTheme, DataOutlineTheme, FixedFillTheme, OutlineTheme,
    PointOutlineTheme, PointTheme, SolidStrokeTheme, StrokeTheme,
};
pub(crate) use resolved::ResolvedTheme;
pub use resolved::{SurfaceTheme, TextTheme};

pub use paint::Color;
pub use paint::OpaqueColor;
pub(crate) use spacing::LocalMetric;
pub use spacing::RelativeSpacing;
pub use value::{Theme, ThemeBuilder};

/// Convert the current RGB tuple inputs at their private paint boundary.
pub(crate) fn color_from_rgb((red, green, blue): (u8, u8, u8)) -> Color {
    OpaqueColor::rgb(red, green, blue).into()
}
