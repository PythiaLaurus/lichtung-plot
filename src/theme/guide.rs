//! Immutable guide chrome and exact local relationships.

use std::cmp::Ordering;

use crate::layout::kernel::{FeedbackRound, SpatialRounding};
use crate::{Error, Result};

use super::spacing::ExactLength;
use super::{Color, LocalMetric, OutlineTheme, RelativeSpacing};

fn positive(value: &RelativeSpacing, field: &str) -> Result<()> {
    if value.is_zero() {
        Err(Error::Data(format!("guide {field} must be positive")))
    } else {
        Ok(())
    }
}

macro_rules! legend_spacing {
    ($($field:ident => $get:ident: $metric:literal),+ $(,)?) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        /// Complete independent exact guide relationships, resolved against selected guide text em.
        pub struct LegendSpacing { $($field: RelativeSpacing),+ }
        impl LegendSpacing {
            #[expect(clippy::too_many_arguments, reason = "the authoritative complete spacing constructor names all nine independent relationships")]
            /// Construct complete nonnegative relationships with a strictly positive key width.
            pub fn new($($field: RelativeSpacing),+) -> Result<Self> {
                let value = Self { $($field),+ }; value.validate()?; Ok(value)
            }
            fn validate(&self) -> Result<()> { positive(self.get_key_width(), "key width") }
            $(
                #[doc = concat!("Borrow the exact `", stringify!($field), "` relationship.")]
                pub fn $get(&self) -> &RelativeSpacing { &self.$field }
                #[doc = concat!("Replace only `", stringify!($field), "`, validating the complete spacing value.")]
                pub fn $field(mut self, value: RelativeSpacing) -> Result<Self> { self.$field = value; self.validate()?; Ok(self) }
            )+
            fn relationships(&self) -> [&RelativeSpacing; 9] { [$(&self.$field),+] }
        }
        #[derive(Debug, Clone)]
        pub(crate) struct LegendLengths { $(pub(crate) $field: ExactLength),+,
            pub(crate) minimum: ExactLength, pub(crate) preferred: ExactLength,
            pub(crate) radius: Option<ExactLength>,
        }
        impl LegendTheme {
            pub(crate) fn lengths(&self, label_size: f64, title_size: f64) -> Result<LegendLengths> {
                let metrics = [LocalMetric::css_pixels(label_size)?, LocalMetric::css_pixels(title_size)?];
                Ok(LegendLengths {
                    $($field: self.get_spacing().$get().product(metrics[$metric])),+,
                    minimum: self.get_gradient_lengths().get_minimum().product(metrics[0]),
                    preferred: self.get_gradient_lengths().get_preferred().product(metrics[0]),
                    radius: self.get_frame().map(|frame| frame.get_radius().product(metrics[0])),
                })
            }
        }
    };
}

legend_spacing! {
    padding => get_padding: 0, key_width => get_key_width: 0,
    key_label_gap => get_key_label_gap: 0, outside_gap => get_outside_gap: 0,
    shared_gap => get_shared_gap: 0, row_gap => get_row_gap: 0,
    column_gap => get_column_gap: 0, title_gap => get_title_gap: 1,
    glyph_edge_padding => get_glyph_edge_padding: 0,
}

/// Positive minimum and at-least-minimum preferred continuous-guide lengths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GradientLengths {
    minimum: RelativeSpacing,
    preferred: RelativeSpacing,
}

impl GradientLengths {
    /// Construct positive lengths with preferred at least minimum under exact comparison.
    pub fn new(minimum: RelativeSpacing, preferred: RelativeSpacing) -> Result<Self> {
        let value = Self { minimum, preferred };
        value.validate_in()?;
        Ok(value)
    }
    fn validate_in(&self) -> Result<()> {
        positive(self.get_minimum(), "gradient minimum")?;
        positive(self.get_preferred(), "gradient preference")?;
        if self.get_minimum().compare(self.get_preferred()) == Ordering::Greater {
            return Err(Error::Data(
                "guide gradient preference must be at least its minimum".into(),
            ));
        }
        Ok(())
    }
    /// Borrow the exact minimum relationship.
    pub fn get_minimum(&self) -> &RelativeSpacing {
        &self.minimum
    }
    /// Borrow the exact preferred relationship.
    pub fn get_preferred(&self) -> &RelativeSpacing {
        &self.preferred
    }
    /// Replace minimum, preserving the positive ordered length invariant.
    pub fn minimum(self, minimum: RelativeSpacing) -> Result<Self> {
        Self::new(minimum, self.preferred)
    }
    /// Replace preference, preserving the positive ordered length invariant.
    pub fn preferred(self, preferred: RelativeSpacing) -> Result<Self> {
        Self::new(self.minimum, preferred)
    }
}

/// Complete guide-container paint, optional independent outline and local corner radius.
#[derive(Clone, Debug, PartialEq)]
pub struct LegendFrameTheme {
    fill: Color,
    outline: Option<OutlineTheme>,
    radius: RelativeSpacing,
}

impl LegendFrameTheme {
    /// Construct a complete container with a nonnegative local corner radius.
    pub fn new(
        fill: Color,
        outline: Option<OutlineTheme>,
        radius: RelativeSpacing,
    ) -> Result<Self> {
        let value = Self {
            fill,
            outline,
            radius,
        };
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<()> {
        if let Some(outline) = self.get_outline() {
            outline.validate()?;
        }
        Ok(())
    }
    /// Inspect the complete container fill.
    pub fn get_fill(&self) -> Color {
        self.fill
    }
    /// Borrow the optional independent outline.
    pub fn get_outline(&self) -> Option<&OutlineTheme> {
        self.outline.as_ref()
    }
    /// Borrow the nonnegative legend-label-em corner-radius relationship.
    pub fn get_radius(&self) -> &RelativeSpacing {
        &self.radius
    }
    /// Replace only the container fill.
    pub fn fill(self, fill: Color) -> Result<Self> {
        Self::new(fill, self.outline, self.radius)
    }
    /// Replace only the optional independent outline.
    pub fn outline(self, outline: Option<OutlineTheme>) -> Result<Self> {
        Self::new(self.fill, outline, self.radius)
    }
    /// Replace only the nonnegative local corner-radius relationship.
    pub fn radius(self, radius: RelativeSpacing) -> Result<Self> {
        Self::new(self.fill, self.outline, radius)
    }
}

/// Complete measured guide chrome, independent of guide content or placement policy.
#[derive(Clone, Debug, PartialEq)]
pub struct LegendTheme {
    spacing: LegendSpacing,
    gradient_lengths: GradientLengths,
    frame: Option<LegendFrameTheme>,
}

impl LegendTheme {
    pub(crate) fn builtin() -> Result<Self> {
        let spacing = LegendSpacing::new(
            RelativeSpacing::from_pixels(8.8, 11)?,
            RelativeSpacing::ratio(1, 1)?,
            RelativeSpacing::from_pixels(8.8, 11)?,
            RelativeSpacing::from_pixels(8.8, 11)?,
            RelativeSpacing::from_pixels(17.6, 11)?,
            RelativeSpacing::from_pixels(3.3, 11)?,
            RelativeSpacing::from_pixels(13.2, 11)?,
            RelativeSpacing::from_pixels(7.8, 12)?,
            RelativeSpacing::new(0.2)?,
        )?;
        let lengths = GradientLengths::new(
            RelativeSpacing::ratio(40, 11)?,
            RelativeSpacing::ratio(160, 11)?,
        )?;
        let frame = LegendFrameTheme::new(
            Color::rgba(255, 255, 255, 204),
            Some(OutlineTheme::new(Color::rgb(200, 200, 200), 1.0)?),
            RelativeSpacing::ratio(2, 11)?,
        )?;
        Self::new(spacing, lengths, Some(frame))
    }
    /// Construct complete spacing, continuous lengths and an optional container frame.
    pub fn new(
        spacing: LegendSpacing,
        gradient_lengths: GradientLengths,
        frame: Option<LegendFrameTheme>,
    ) -> Result<Self> {
        let value = Self {
            spacing,
            gradient_lengths,
            frame,
        };
        value.validate_in()?;
        Ok(value)
    }
    pub(crate) fn validate_in(&self) -> Result<()> {
        self.get_spacing().validate()?;
        self.get_gradient_lengths().validate_in()?;
        if let Some(frame) = self.get_frame() {
            frame.validate()?;
        }
        Ok(())
    }
    /// Borrow all complete exact guide relationships.
    pub fn get_spacing(&self) -> &LegendSpacing {
        &self.spacing
    }
    /// Borrow the continuous-guide minimum and preference.
    pub fn get_gradient_lengths(&self) -> &GradientLengths {
        &self.gradient_lengths
    }
    /// Borrow the optional guide-container appearance.
    pub fn get_frame(&self) -> Option<&LegendFrameTheme> {
        self.frame.as_ref()
    }
    /// Borrow the Canvas-owned shared-guide gap relationship.
    pub fn get_shared_gap(&self) -> &RelativeSpacing {
        self.get_spacing().get_shared_gap()
    }
    /// Replace the complete guide-spacing value.
    pub fn spacing(self, spacing: LegendSpacing) -> Result<Self> {
        Self::new(spacing, self.gradient_lengths, self.frame)
    }
    /// Replace the complete continuous-guide length value.
    pub fn gradient_lengths(self, value: GradientLengths) -> Result<Self> {
        Self::new(self.spacing, value, self.frame)
    }
    /// Replace only the optional guide-container appearance.
    pub fn frame(self, frame: Option<LegendFrameTheme>) -> Result<Self> {
        Self::new(self.spacing, self.gradient_lengths, frame)
    }
    /// Replace only the Canvas-owned shared-guide gap relationship.
    pub fn shared_gap(self, value: RelativeSpacing) -> Result<Self> {
        let spacing = self.spacing.shared_gap(value)?;
        Self::new(spacing, self.gradient_lengths, self.frame)
    }

    pub(crate) fn same(&self, other: &Self) -> bool {
        match (self.get_frame(), other.get_frame()) {
            (None, None) => {}
            (Some(left), Some(right))
                if left.get_fill() == right.get_fill()
                    && left.get_outline() == right.get_outline() => {}
            _ => return false,
        }
        for (left, right) in self
            .get_spacing()
            .relationships()
            .into_iter()
            .zip(other.get_spacing().relationships())
        {
            if !left.same(right) {
                return false;
            }
        }
        for (left, right) in [
            (
                self.get_gradient_lengths().get_minimum(),
                other.get_gradient_lengths().get_minimum(),
            ),
            (
                self.get_gradient_lengths().get_preferred(),
                other.get_gradient_lengths().get_preferred(),
            ),
        ] {
            if !left.same(right) {
                return false;
            }
        }
        if let (Some(left), Some(right)) = (self.get_frame(), other.get_frame()) {
            if !left.get_radius().same(right.get_radius()) {
                return false;
            }
        }
        true
    }
    pub(crate) fn observe_feedback(&self, round: &mut FeedbackRound) -> Result<()> {
        round.observe_count(usize::from(self.get_frame().is_some()))?;
        for value in self.get_spacing().relationships() {
            value.observe_feedback(round)?;
        }
        self.get_gradient_lengths()
            .get_minimum()
            .observe_feedback(round)?;
        self.get_gradient_lengths()
            .get_preferred()
            .observe_feedback(round)?;
        if let Some(frame) = self.get_frame() {
            for color in [
                Some(frame.get_fill()),
                frame.get_outline().map(OutlineTheme::get_color),
            ]
            .into_iter()
            .flatten()
            {
                for byte in [color.red(), color.green(), color.blue(), color.get_alpha()] {
                    round.observe_count(usize::from(byte))?;
                }
            }
            round.observe_count(usize::from(frame.get_outline().is_some()))?;
            if let Some(outline) = frame.get_outline() {
                round.observe_number(f64::from(outline.get_width()))?;
            }
            frame.get_radius().observe_feedback(round)?;
        }
        Ok(())
    }
    pub(crate) fn outside_gap_pixels(&self, label_size: f64) -> Result<f64> {
        self.get_spacing()
            .get_outside_gap()
            .product(LocalMetric::css_pixels(label_size)?)
            .pixels_rounded(SpatialRounding::NearestEven)
    }
    pub(crate) fn shared_gap_pixels(&self, label_size: f64) -> Result<f64> {
        self.get_shared_gap()
            .product(LocalMetric::css_pixels(label_size)?)
            .pixels_rounded(SpatialRounding::NearestEven)
    }
}

#[cfg(test)]
mod tests;
