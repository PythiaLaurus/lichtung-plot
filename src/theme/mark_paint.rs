//! Validated compound paints, independent of backend geometry and operation state.

use crate::render::backend::{FillStyle, LineStyle, PointOutline, PointStyle, StrokePattern};
use crate::{Error, PointShape, Result};

use super::{color_from_rgb, Color, RelativeSpacing};

fn validate_width(width: f32) -> Result<()> {
    if width.is_finite() && width > 0.0 {
        Ok(())
    } else {
        Err(Error::Data(
            "present stroke width must be finite and positive CSS pixels".into(),
        ))
    }
}

// Solid outlines and solid bracket strokes share scalar validation, not roles.
macro_rules! solid_paint {
    ($name:ident, $documentation:literal) => {
        #[doc = $documentation]
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            pub(crate) paint: Color,
            pub(crate) width: f32,
        }

        impl $name {
            /// Construct a complete paint and positive finite CSS stroke width.
            pub fn new(color: Color, width: f32) -> Result<Self> {
                let value = Self {
                    paint: color,
                    width,
                };
                value.validate()?;
                Ok(value)
            }

            pub(crate) fn validate(&self) -> Result<()> {
                validate_width(self.width())
            }

            /// Inspect the complete stroke paint.
            pub fn color(&self) -> Color {
                self.paint
            }
            /// Inspect the positive width in CSS pixels.
            pub fn width(&self) -> f32 {
                self.width
            }

            /// Replace only the complete stroke paint.
            pub fn with_color(self, color: Color) -> Result<Self> {
                Self::new(color, self.width)
            }

            /// Replace only the positive CSS width.
            pub fn with_width(self, width: f32) -> Result<Self> {
                Self::new(self.paint, width)
            }
        }
    };
}

solid_paint!(
    SolidStrokeTheme,
    "Complete solid bracket stroke; its topology has no pattern edit."
);
solid_paint!(
    OutlineTheme,
    "Complete independent outline paint and positive CSS width."
);

/// Complete stroke with the renderer's one closed dash vocabulary.
#[derive(Debug, Clone, PartialEq)]
pub struct StrokeTheme {
    pub(crate) paint: Color,
    pub(crate) width: f32,
    pub(crate) pattern: StrokePattern,
}

impl StrokeTheme {
    pub(crate) fn validate(&self) -> Result<()> {
        validate_width(self.width())
    }
    /// Construct a complete stroke with positive finite CSS width and a closed pattern.
    pub fn new(color: Color, width: f32, pattern: StrokePattern) -> Result<Self> {
        validate_width(width)?;
        Ok(Self {
            paint: color,
            width,
            pattern,
        })
    }

    /// Inspect the complete stroke paint.
    pub fn color(&self) -> Color {
        self.paint
    }
    /// Inspect the positive CSS width.
    pub fn width(&self) -> f32 {
        self.width
    }
    /// Inspect the closed stroke pattern.
    pub fn pattern(&self) -> StrokePattern {
        self.pattern
    }

    /// Replace only the complete stroke paint.
    pub fn with_color(self, color: Color) -> Result<Self> {
        Self::new(color, self.width, self.pattern)
    }
    /// Replace only the positive CSS width.
    pub fn with_width(self, width: f32) -> Result<Self> {
        Self::new(self.paint, width, self.pattern)
    }
    /// Replace only the closed stroke pattern.
    pub fn with_pattern(self, pattern: StrokePattern) -> Result<Self> {
        Self::new(self.paint, self.width, pattern)
    }

    pub(crate) fn backend_style(&self) -> LineStyle {
        LineStyle {
            color: self.color(),
            width: self.width(),
            pattern: self.pattern(),
        }
    }
}

/// A point outline owns one exact relationship to the selected point radius.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointOutlineTheme {
    pub(crate) color: Color,
    pub(crate) width_ratio: RelativeSpacing,
}

impl PointOutlineTheme {
    /// Construct independent outline paint and a positive radius-relative width.
    pub fn new(color: Color, width_ratio: RelativeSpacing) -> Result<Self> {
        if width_ratio.is_zero() {
            return Err(Error::Data(
                "point outline width ratio must be positive".into(),
            ));
        }
        Ok(Self { color, width_ratio })
    }

    fn validate(&self) -> Result<()> {
        if self.width_ratio.is_zero() {
            Err(Error::Data(
                "point outline width ratio must be positive".into(),
            ))
        } else {
            Ok(())
        }
    }

    /// Inspect the independent outline paint.
    pub fn color(&self) -> Color {
        self.color
    }
    /// Borrow the exact positive width-to-radius ratio.
    pub fn width_ratio(&self) -> &RelativeSpacing {
        &self.width_ratio
    }
    /// Replace only the independent outline paint.
    pub fn with_color(self, color: Color) -> Result<Self> {
        Self::new(color, self.width_ratio)
    }
    /// Replace only the positive radius-relative width.
    pub fn with_width_ratio(self, width_ratio: RelativeSpacing) -> Result<Self> {
        Self::new(self.color, width_ratio)
    }

    pub(crate) fn publication() -> Result<Self> {
        Self::new(Color::rgb(255, 255, 255), RelativeSpacing::new(0.1)?)
    }

    /// Materialize the operational outline by sharing the same immutable ratio.
    pub(crate) fn backend_style(&self) -> PointOutline {
        PointOutline {
            color: self.color(),
            width_ratio: self.width_ratio().clone(),
        }
    }
}

/// Fixed paint or mapped RGB with an independent outline alpha.
#[derive(Debug, Clone, PartialEq)]
pub struct DataOutlineTheme {
    choice: DataOutline,
}

#[derive(Debug, Clone, PartialEq)]
enum DataOutline {
    Fixed(OutlineTheme),
    Mapped { width: f32, alpha: u8 },
}

impl DataOutlineTheme {
    /// Select a complete independent fixed outline.
    pub fn fixed(stroke: OutlineTheme) -> Self {
        Self {
            choice: DataOutline::Fixed(stroke),
        }
    }

    /// Select mapped RGB with positive CSS width and an independent byte alpha.
    pub fn mapped(width: f32, alpha: u8) -> Result<Self> {
        validate_width(width)?;
        Ok(Self {
            choice: DataOutline::Mapped { width, alpha },
        })
    }

    /// Borrow the fixed outline, or None for mapped RGB.
    pub fn fixed_stroke(&self) -> Option<&OutlineTheme> {
        match &self.choice {
            DataOutline::Fixed(stroke) => Some(stroke),
            DataOutline::Mapped { .. } => None,
        }
    }
    /// Inspect the mapped width, or None for a fixed outline.
    pub fn mapped_width(&self) -> Option<f32> {
        match self.choice {
            DataOutline::Fixed(_) => None,
            DataOutline::Mapped { width, .. } => Some(width),
        }
    }
    /// Inspect mapped alpha, or None for a fixed outline.
    pub fn mapped_alpha(&self) -> Option<u8> {
        match self.choice {
            DataOutline::Fixed(_) => None,
            DataOutline::Mapped { alpha, .. } => Some(alpha),
        }
    }

    fn validate(&self) -> Result<()> {
        match &self.choice {
            DataOutline::Fixed(stroke) => stroke.validate(),
            DataOutline::Mapped { width, .. } => validate_width(*width),
        }
    }

    fn operands(&self, mapped: Color) -> (Color, f32) {
        if let Some(stroke) = self.fixed_stroke() {
            (stroke.color(), stroke.width())
        } else {
            (
                mapped.alpha(self.mapped_alpha().expect("validated mapped outline")),
                self.mapped_width().expect("validated mapped outline"),
            )
        }
    }
}

/// Mapped fill alpha and a structural optional outline.
#[derive(Debug, Clone, PartialEq)]
pub struct DataFillTheme {
    pub(crate) fill_alpha: u8,
    pub(crate) outline: Option<DataOutlineTheme>,
}

impl DataFillTheme {
    pub(crate) fn validate(&self) -> Result<()> {
        if let Some(outline) = self.outline() {
            outline.validate()?;
        }
        Ok(())
    }
    /// Construct mapped fill alpha and its independently owned optional outline.
    pub fn new(fill_alpha: u8, outline: Option<DataOutlineTheme>) -> Result<Self> {
        let value = Self {
            fill_alpha,
            outline,
        };
        value.validate()?;
        Ok(value)
    }
    /// Inspect fill alpha, including transparent and opaque endpoints.
    pub fn fill_alpha(&self) -> u8 {
        self.fill_alpha
    }
    /// Borrow the optional independent outline.
    pub fn outline(&self) -> Option<&DataOutlineTheme> {
        self.outline.as_ref()
    }
    /// Replace only fill alpha.
    pub fn with_fill_alpha(self, fill_alpha: u8) -> Result<Self> {
        Self::new(fill_alpha, self.outline)
    }
    /// Replace the complete optional outline without changing fill alpha.
    pub fn with_outline(self, outline: Option<DataOutlineTheme>) -> Result<Self> {
        Self::new(self.fill_alpha, outline)
    }
    pub(crate) fn backend_style(&self, rgb: (u8, u8, u8)) -> FillStyle {
        let color = color_from_rgb(rgb);
        let (stroke, width) = self
            .outline()
            .map_or((Color::rgba(0, 0, 0, 0), 0.0), |outline| {
                outline.operands(color)
            });
        FillStyle::new(color.alpha(self.fill_alpha()), stroke, width)
    }
}

/// Complete fixed fill paint and an independent structural optional outline.
#[derive(Debug, Clone, PartialEq)]
pub struct FixedFillTheme {
    pub(crate) paint: Color,
    pub(crate) outline: Option<OutlineTheme>,
}

impl FixedFillTheme {
    pub(crate) fn validate(&self) -> Result<()> {
        if let Some(outline) = self.outline() {
            outline.validate()?;
        }
        Ok(())
    }
    /// Construct complete fixed fill and independent optional outline paints.
    pub fn new(color: Color, outline: Option<OutlineTheme>) -> Result<Self> {
        let value = Self {
            paint: color,
            outline,
        };
        value.validate()?;
        Ok(value)
    }
    /// Inspect the complete fill paint.
    pub fn color(&self) -> Color {
        self.paint
    }
    /// Borrow the optional independent outline.
    pub fn outline(&self) -> Option<&OutlineTheme> {
        self.outline.as_ref()
    }
    /// Replace only the complete fill paint.
    pub fn with_color(self, color: Color) -> Result<Self> {
        Self::new(color, self.outline)
    }
    /// Replace the complete optional outline without changing fill paint.
    pub fn with_outline(self, outline: Option<OutlineTheme>) -> Result<Self> {
        Self::new(self.paint, outline)
    }
    pub(crate) fn backend_style(&self) -> FillStyle {
        self.outline().map_or_else(
            || FillStyle::new(self.color(), Color::rgba(0, 0, 0, 0), 0.0),
            |outline| outline.backend_style(self.color()),
        )
    }
}

impl OutlineTheme {
    pub(crate) fn backend_style(&self, fill: Color) -> FillStyle {
        FillStyle::new(fill, self.color(), self.width())
    }
}

/// Shared point appearance before selecting its final integral radius.
#[derive(Debug, Clone, PartialEq)]
pub struct PointTheme {
    pub(crate) fill_alpha: u8,
    pub(crate) shape: PointShape,
    pub(crate) outline: Option<PointOutlineTheme>,
}

impl PointTheme {
    pub(crate) fn validate(&self) -> Result<()> {
        if let Some(outline) = self.outline() {
            outline.validate()?;
        }
        Ok(())
    }
    /// Construct mapped fill alpha, closed shape and an independent optional outline.
    pub fn new(
        fill_alpha: u8,
        shape: PointShape,
        outline: Option<PointOutlineTheme>,
    ) -> Result<Self> {
        let value = Self {
            fill_alpha,
            shape,
            outline,
        };
        value.validate()?;
        Ok(value)
    }
    /// Inspect the mapped point fill alpha.
    pub fn fill_alpha(&self) -> u8 {
        self.fill_alpha
    }
    /// Inspect the closed point shape.
    pub fn shape(&self) -> PointShape {
        self.shape
    }
    /// Borrow the optional independent outline.
    pub fn outline(&self) -> Option<&PointOutlineTheme> {
        self.outline.as_ref()
    }
    /// Replace only mapped fill alpha.
    pub fn with_fill_alpha(self, fill_alpha: u8) -> Result<Self> {
        Self::new(fill_alpha, self.shape, self.outline)
    }
    /// Replace only the closed point shape.
    pub fn with_shape(self, shape: PointShape) -> Result<Self> {
        Self::new(self.fill_alpha, shape, self.outline)
    }
    /// Replace the optional independent outline without changing fill or shape.
    pub fn with_outline(self, outline: Option<PointOutlineTheme>) -> Result<Self> {
        Self::new(self.fill_alpha, self.shape, outline)
    }
    pub(crate) fn backend_style(
        &self,
        rgb: (u8, u8, u8),
        radius: u32,
        outline_visible: bool,
    ) -> PointStyle {
        PointStyle {
            color: color_from_rgb(rgb).alpha(self.fill_alpha()),
            radius,
            shape: self.shape(),
            outline: self
                .outline()
                .filter(|_| outline_visible)
                .map(PointOutlineTheme::backend_style),
        }
    }
}

/// Data-colored line with one independent alpha and the closed pattern input.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLineTheme {
    pub(crate) width: f32,
    pub(crate) alpha: u8,
    pub(crate) pattern: StrokePattern,
}

impl DataLineTheme {
    pub(crate) fn validate(&self) -> Result<()> {
        validate_width(self.get_width())
    }
    /// Construct positive finite CSS width, mapped byte alpha and a closed pattern.
    pub fn new(width: f32, alpha: u8, pattern: StrokePattern) -> Result<Self> {
        let value = Self {
            width,
            alpha,
            pattern,
        };
        value.validate()?;
        Ok(value)
    }
    /// Inspect the positive CSS width.
    pub fn get_width(&self) -> f32 {
        self.width
    }
    /// Inspect mapped stroke alpha.
    pub fn get_alpha(&self) -> u8 {
        self.alpha
    }
    /// Inspect the closed stroke pattern.
    pub fn get_pattern(&self) -> StrokePattern {
        self.pattern
    }
    /// Replace only the positive CSS width.
    pub fn width(self, width: f32) -> Result<Self> {
        Self::new(width, self.alpha, self.pattern)
    }
    /// Replace only mapped stroke alpha.
    pub fn alpha(self, alpha: u8) -> Result<Self> {
        Self::new(self.width, alpha, self.pattern)
    }
    /// Replace only the closed stroke pattern.
    pub fn pattern(self, pattern: StrokePattern) -> Result<Self> {
        Self::new(self.width, self.alpha, pattern)
    }
    pub(crate) fn backend_style(&self, rgb: (u8, u8, u8)) -> LineStyle {
        LineStyle {
            color: color_from_rgb(rgb).alpha(self.get_alpha()),
            width: self.get_width(),
            pattern: self.get_pattern(),
        }
    }
}

#[cfg(test)]
mod tests;
