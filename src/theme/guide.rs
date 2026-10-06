//! Immutable guide chrome and operation-admitted local relationships.

use std::cmp::Ordering;

use crate::font::canonical::OperationGenerationResources;
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
    ($($field:ident => $edit:ident: $metric:literal),+ $(,)?) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub(crate) struct LegendSpacing { $($field: RelativeSpacing),+ }
        impl LegendSpacing {
            #[expect(clippy::too_many_arguments, reason = "the authoritative complete spacing constructor names all nine independent relationships")]
            pub(crate) fn new($($field: RelativeSpacing),+) -> Result<Self> {
                let value = Self { $($field),+ }; value.validate()?; Ok(value)
            }
            fn validate(&self) -> Result<()> { positive(self.key_width(), "key width") }
            $(
                pub(crate) fn $field(&self) -> &RelativeSpacing { &self.$field }
                #[cfg_attr(not(test), allow(dead_code, reason = "private guide construction edits await complete Theme exposure"))]
                pub(crate) fn $edit(mut self, value: RelativeSpacing) -> Result<Self> { self.$field = value; self.validate()?; Ok(self) }
            )+
            fn relationships(&self) -> [&RelativeSpacing; 9] { [$(&self.$field),+] }
        }
        #[derive(Debug)]
        pub(crate) struct LegendLengths { $(pub(crate) $field: ExactLength),+,
            pub(crate) minimum: ExactLength, pub(crate) preferred: ExactLength,
            pub(crate) radius: Option<ExactLength>,
        }
        impl LegendTheme {
            pub(crate) fn lengths(&self, label_size: u32, title_size: u32, resources: &OperationGenerationResources) -> Result<LegendLengths> {
                let metrics = [LocalMetric::css_pixels(label_size)?, LocalMetric::css_pixels(title_size)?];
                Ok(LegendLengths {
                    $($field: self.spacing().$field().product(metrics[$metric], resources)?),+,
                    minimum: self.gradient_lengths().minimum().product(metrics[0], resources)?,
                    preferred: self.gradient_lengths().preferred().product(metrics[0], resources)?,
                    radius: self.frame().map(|frame| frame.radius().product(metrics[0], resources)).transpose()?,
                })
            }
        }
    };
}

legend_spacing! {
    padding => with_padding: 0, key_width => with_key_width: 0,
    key_label_gap => with_key_label_gap: 0, outside_gap => with_outside_gap: 0,
    shared_gap => with_shared_gap: 0, row_gap => with_row_gap: 0,
    column_gap => with_column_gap: 0, title_gap => with_title_gap: 1,
    glyph_edge_padding => with_glyph_edge_padding: 0,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GradientLengths {
    minimum: RelativeSpacing,
    preferred: RelativeSpacing,
}

impl GradientLengths {
    pub(crate) fn new(minimum: RelativeSpacing, preferred: RelativeSpacing) -> Result<Self> {
        Self::new_in(minimum, preferred, &OperationGenerationResources::new())
    }
    pub(crate) fn new_in(
        minimum: RelativeSpacing,
        preferred: RelativeSpacing,
        resources: &OperationGenerationResources,
    ) -> Result<Self> {
        let value = Self { minimum, preferred };
        value.validate_in(resources)?;
        Ok(value)
    }
    fn validate_in(&self, resources: &OperationGenerationResources) -> Result<()> {
        positive(self.minimum(), "gradient minimum")?;
        positive(self.preferred(), "gradient preference")?;
        if self.minimum().compare(self.preferred(), resources)? == Ordering::Greater {
            return Err(Error::Data(
                "guide gradient preference must be at least its minimum".into(),
            ));
        }
        Ok(())
    }
    pub(crate) fn minimum(&self) -> &RelativeSpacing {
        &self.minimum
    }
    pub(crate) fn preferred(&self) -> &RelativeSpacing {
        &self.preferred
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private gradient edits await complete Theme exposure"
        )
    )]
    pub(crate) fn with_minimum(self, minimum: RelativeSpacing) -> Result<Self> {
        Self::new(minimum, self.preferred)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private gradient edits await complete Theme exposure"
        )
    )]
    pub(crate) fn with_preferred(self, preferred: RelativeSpacing) -> Result<Self> {
        Self::new(self.minimum, preferred)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LegendFrameTheme {
    fill: Color,
    outline: Option<OutlineTheme>,
    radius: RelativeSpacing,
}

impl LegendFrameTheme {
    pub(crate) fn new(
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
        if let Some(outline) = self.outline() {
            outline.validate()?;
        }
        Ok(())
    }
    pub(crate) fn fill(&self) -> Color {
        self.fill
    }
    pub(crate) fn outline(&self) -> Option<&OutlineTheme> {
        self.outline.as_ref()
    }
    pub(crate) fn radius(&self) -> &RelativeSpacing {
        &self.radius
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private frame edits await complete Theme exposure"
        )
    )]
    pub(crate) fn with_fill(self, fill: Color) -> Result<Self> {
        Self::new(fill, self.outline, self.radius)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private frame edits await complete Theme exposure"
        )
    )]
    pub(crate) fn with_outline(self, outline: Option<OutlineTheme>) -> Result<Self> {
        Self::new(self.fill, outline, self.radius)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private frame edits await complete Theme exposure"
        )
    )]
    pub(crate) fn with_radius(self, radius: RelativeSpacing) -> Result<Self> {
        Self::new(self.fill, self.outline, radius)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LegendTheme {
    spacing: LegendSpacing,
    gradient_lengths: GradientLengths,
    frame: Option<LegendFrameTheme>,
}

impl LegendTheme {
    pub(crate) fn builtin(resources: &OperationGenerationResources) -> Result<Self> {
        let spacing = LegendSpacing::new(
            RelativeSpacing::from_pixels_in(8.8, 11, resources)?,
            RelativeSpacing::ratio_in(1, 1, resources)?,
            RelativeSpacing::from_pixels_in(8.8, 11, resources)?,
            RelativeSpacing::from_pixels_in(8.8, 11, resources)?,
            RelativeSpacing::from_pixels_in(17.6, 11, resources)?,
            RelativeSpacing::from_pixels_in(3.3, 11, resources)?,
            RelativeSpacing::from_pixels_in(13.2, 11, resources)?,
            RelativeSpacing::from_pixels_in(7.8, 12, resources)?,
            RelativeSpacing::new_in(0.2, resources)?,
        )?;
        let lengths = GradientLengths::new_in(
            RelativeSpacing::ratio_in(40, 11, resources)?,
            RelativeSpacing::ratio_in(160, 11, resources)?,
            resources,
        )?;
        let frame = LegendFrameTheme::new(
            Color::rgba(255, 255, 255, 204),
            Some(OutlineTheme::new(Color::rgb(200, 200, 200), 1.0)?),
            RelativeSpacing::ratio_in(2, 11, resources)?,
        )?;
        Self::new_in(spacing, lengths, Some(frame), resources)
    }
    pub(crate) fn new(
        spacing: LegendSpacing,
        gradient_lengths: GradientLengths,
        frame: Option<LegendFrameTheme>,
    ) -> Result<Self> {
        Self::new_in(
            spacing,
            gradient_lengths,
            frame,
            &OperationGenerationResources::new(),
        )
    }
    pub(crate) fn new_in(
        spacing: LegendSpacing,
        gradient_lengths: GradientLengths,
        frame: Option<LegendFrameTheme>,
        resources: &OperationGenerationResources,
    ) -> Result<Self> {
        let value = Self {
            spacing,
            gradient_lengths,
            frame,
        };
        value.validate_in(resources)?;
        Ok(value)
    }
    fn validate_in(&self, resources: &OperationGenerationResources) -> Result<()> {
        self.spacing().validate()?;
        self.gradient_lengths().validate_in(resources)?;
        if let Some(frame) = self.frame() {
            frame.validate()?;
        }
        Ok(())
    }
    pub(crate) fn spacing(&self) -> &LegendSpacing {
        &self.spacing
    }
    pub(crate) fn gradient_lengths(&self) -> &GradientLengths {
        &self.gradient_lengths
    }
    pub(crate) fn frame(&self) -> Option<&LegendFrameTheme> {
        self.frame.as_ref()
    }
    pub(crate) fn shared_gap(&self) -> &RelativeSpacing {
        self.spacing().shared_gap()
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private guide edits await complete Theme exposure"
        )
    )]
    pub(crate) fn with_spacing(self, spacing: LegendSpacing) -> Result<Self> {
        Self::new(spacing, self.gradient_lengths, self.frame)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private guide edits await complete Theme exposure"
        )
    )]
    pub(crate) fn with_gradient_lengths(self, value: GradientLengths) -> Result<Self> {
        Self::new(self.spacing, value, self.frame)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private guide edits await complete Theme exposure"
        )
    )]
    pub(crate) fn with_frame(self, frame: Option<LegendFrameTheme>) -> Result<Self> {
        Self::new(self.spacing, self.gradient_lengths, frame)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private guide edits await complete Theme exposure"
        )
    )]
    pub(crate) fn with_shared_gap(self, value: RelativeSpacing) -> Result<Self> {
        let spacing = self.spacing.with_shared_gap(value)?;
        Self::new(spacing, self.gradient_lengths, self.frame)
    }
    pub(crate) fn relationship_count(&self) -> usize {
        11 + usize::from(self.frame().is_some())
    }
    #[cfg(test)]
    pub(crate) fn retained_relationships(&self) -> impl Iterator<Item = &RelativeSpacing> {
        self.spacing()
            .relationships()
            .into_iter()
            .chain([
                self.gradient_lengths().minimum(),
                self.gradient_lengths().preferred(),
            ])
            .chain(self.frame().map(LegendFrameTheme::radius))
    }
    pub(crate) fn share(&self, resources: &OperationGenerationResources) -> Result<Self> {
        resources
            .claim_visits(self.relationship_count() as u128)
            .map_err(|error| Error::Render(format!("guide sharing: {error:?}")))?;
        Ok(self.clone())
    }
    pub(crate) fn same(
        &self,
        other: &Self,
        resources: &OperationGenerationResources,
    ) -> Result<bool> {
        resources
            .claim_visits(1)
            .map_err(|error| Error::Render(format!("guide identity: {error:?}")))?;
        match (self.frame(), other.frame()) {
            (None, None) => {}
            (Some(left), Some(right))
                if left.fill() == right.fill() && left.outline() == right.outline() => {}
            _ => return Ok(false),
        }
        for (left, right) in self
            .spacing()
            .relationships()
            .into_iter()
            .zip(other.spacing().relationships())
        {
            if !left.same(right, resources)? {
                return Ok(false);
            }
        }
        for (left, right) in [
            (
                self.gradient_lengths().minimum(),
                other.gradient_lengths().minimum(),
            ),
            (
                self.gradient_lengths().preferred(),
                other.gradient_lengths().preferred(),
            ),
        ] {
            if !left.same(right, resources)? {
                return Ok(false);
            }
        }
        if let (Some(left), Some(right)) = (self.frame(), other.frame()) {
            if !left.radius().same(right.radius(), resources)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    pub(crate) fn observe_feedback(&self, round: &mut FeedbackRound) -> Result<()> {
        round.observe_count(usize::from(self.frame().is_some()))?;
        for value in self.spacing().relationships() {
            value.observe_feedback(round)?;
        }
        self.gradient_lengths().minimum().observe_feedback(round)?;
        self.gradient_lengths()
            .preferred()
            .observe_feedback(round)?;
        if let Some(frame) = self.frame() {
            for color in [Some(frame.fill()), frame.outline().map(OutlineTheme::color)]
                .into_iter()
                .flatten()
            {
                for byte in [color.red(), color.green(), color.blue(), color.alpha()] {
                    round.observe_count(usize::from(byte))?;
                }
            }
            round.observe_count(usize::from(frame.outline().is_some()))?;
            if let Some(outline) = frame.outline() {
                round.observe_number(f64::from(outline.width()))?;
            }
            frame.radius().observe_feedback(round)?;
        }
        Ok(())
    }
    pub(crate) fn outside_gap_pixels(
        &self,
        label_size: u32,
        resources: &OperationGenerationResources,
    ) -> Result<f64> {
        self.spacing()
            .outside_gap()
            .product(LocalMetric::css_pixels(label_size)?, resources)?
            .pixels_rounded(SpatialRounding::NearestEven, resources)
    }
    pub(crate) fn shared_gap_pixels(
        &self,
        label_size: u32,
        resources: &OperationGenerationResources,
    ) -> Result<f64> {
        self.shared_gap()
            .product(LocalMetric::css_pixels(label_size)?, resources)?
            .pixels_rounded(SpatialRounding::NearestEven, resources)
    }
}

#[cfg(test)]
mod tests;

/// Independent native-integer certificate for the builtin construction charges.
#[cfg(test)]
pub(crate) fn builtin_resource_oracle() -> (u128, u64) {
    // Exact dyadic numerator and pre-reduction denominator*metric, independently
    // encoded from the documented source IEEE bits. All operands fit one limb.
    let sources = [
        (2_476_979_795_053_773_u64, (1_u64 << 48) * 11),
        (2_476_979_795_053_773, (1_u64 << 48) * 11),
        (2_476_979_795_053_773, (1_u64 << 48) * 11),
        (2_476_979_795_053_773, (1_u64 << 47) * 11),
        (3_715_469_692_580_659, (1_u64 << 50) * 11),
        (3_715_469_692_580_659, (1_u64 << 48) * 11),
        (8_782_019_273_372_467, (1_u64 << 50) * 12),
    ];
    let mut gcd_steps = 0_u64;
    for (mut left, mut right) in sources {
        while right != 0 {
            (left, right) = (right, left % right);
            gcd_steps += 1;
        }
    }
    // Seven P values: header+dyadic+division+gcd setup+reduction = 15 visits,
    // 640 bytes, plus 6 visits/240 bytes per gcd step. Four native ratios cost
    // 4+8+7+6 visits and 160 bytes each; edge dyadic costs 5/240; the two actual
    // complete gradient comparisons cost 2/80 each.
    let header = (size_of::<RelativeSpacing>()
        + 2 * size_of::<num_bigint::BigUint>()
        + size_of::<f64>()
        + 2 * size_of::<usize>()) as u64;
    let operand = 16 + size_of::<num_bigint::BigUint>() as u64;
    let bytes = 7 * (header + 14 * operand)
        + 6 * gcd_steps * operand
        + 4 * (header + 2 * operand)
        + header
        + 4 * operand
        + 4 * operand;
    (139 + 6 * u128::from(gcd_steps), bytes)
}
