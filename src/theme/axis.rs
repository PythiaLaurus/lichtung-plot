//! Complete axis chrome and exact role-local spacing construction.

use crate::font::canonical::OperationGenerationResources;
use crate::layout::kernel::{FeedbackRound, SpatialRounding};
use crate::{Error, Result};

use super::spacing::ExactLength;
use super::{LocalMetric, RelativeSpacing, StrokeTheme};

pub(crate) fn stroke_style(stroke: &StrokeTheme) -> Result<crate::render::backend::LineStyle> {
    stroke.validate()?;
    let width = crate::layout::kernel::LayoutScalar::from_css_pixels(
        f64::from(stroke.width()),
        SpatialRounding::NearestEven,
    )?;
    if width.quanta() == 0 {
        return Err(Error::Render("axis stroke collapses in CSS quanta".into()));
    }
    Ok(stroke.backend_style())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FrameSides {
    bits: u8,
}

macro_rules! side_field {
    ($field:ident, $edit:ident, $bit:literal) => {
        pub(crate) fn $field(&self) -> bool {
            self.bits & $bit != 0
        }
        #[cfg_attr(
            not(test),
            expect(dead_code, reason = "private side edits await complete Theme exposure")
        )]
        pub(crate) fn $edit(mut self, value: bool) -> Result<Self> {
            self.bits = if value {
                self.bits | $bit
            } else {
                self.bits & !$bit
            };
            self.validate()?;
            Ok(self)
        }
    };
}

impl FrameSides {
    #[expect(
        clippy::fn_params_excessive_bools,
        reason = "the complete constructor explicitly names the four independent frame sides"
    )]
    pub(crate) fn new(top: bool, right: bool, bottom: bool, left: bool) -> Result<Self> {
        let value = Self {
            bits: u8::from(top)
                | (u8::from(right) << 1)
                | (u8::from(bottom) << 2)
                | (u8::from(left) << 3),
        };
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<()> {
        if self.bits != 0 && self.bits & !15 == 0 {
            Ok(())
        } else {
            Err(Error::Data(
                "a present frame requires at least one side".into(),
            ))
        }
    }

    side_field!(top, with_top, 1);
    side_field!(right, with_right, 2);
    side_field!(bottom, with_bottom, 4);
    side_field!(left, with_left, 8);

    /// Existing standard drawing order: bottom, right, top, left.
    pub(crate) fn drawing_order(&self) -> [bool; 4] {
        [self.bottom(), self.right(), self.top(), self.left()]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FrameTheme {
    pub(crate) sides: FrameSides,
    pub(crate) stroke: StrokeTheme,
}

impl FrameTheme {
    pub(crate) fn new(sides: FrameSides, stroke: StrokeTheme) -> Result<Self> {
        let value = Self { sides, stroke };
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<()> {
        self.sides.validate()?;
        self.stroke.validate()
    }
    pub(crate) fn sides(&self) -> &FrameSides {
        &self.sides
    }
    pub(crate) fn stroke(&self) -> &StrokeTheme {
        &self.stroke
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private frame edits await complete Theme exposure"
        )
    )]
    pub(crate) fn with_sides(self, sides: FrameSides) -> Result<Self> {
        Self::new(sides, self.stroke)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private frame edits await complete Theme exposure"
        )
    )]
    pub(crate) fn with_stroke(self, stroke: StrokeTheme) -> Result<Self> {
        Self::new(self.sides, stroke)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TickTheme {
    pub(crate) stroke: StrokeTheme,
    pub(crate) length: u32,
}

impl TickTheme {
    pub(crate) fn new(stroke: StrokeTheme, length: u32) -> Result<Self> {
        let value = Self { stroke, length };
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<()> {
        self.stroke.validate()?;
        if self.length == 0 {
            return Err(Error::Data(
                "tick length must be positive CSS pixels".into(),
            ));
        }
        Ok(())
    }
    pub(crate) fn stroke(&self) -> &StrokeTheme {
        &self.stroke
    }
    pub(crate) fn length(&self) -> u32 {
        self.length
    }
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "private tick edits await complete Theme exposure")
    )]
    pub(crate) fn with_stroke(self, stroke: StrokeTheme) -> Result<Self> {
        Self::new(stroke, self.length)
    }
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "private tick edits await complete Theme exposure")
    )]
    pub(crate) fn with_length(self, length: u32) -> Result<Self> {
        Self::new(self.stroke, length)
    }
}

macro_rules! axis_spacing {
    ($($field:ident => $edit:ident : $metric:literal),+ $(,)?) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub(crate) struct AxisSpacing { $(pub(crate) $field: RelativeSpacing),+ }

        impl AxisSpacing {
            pub(crate) fn new($($field: RelativeSpacing),+) -> Result<Self> { Ok(Self { $($field),+ }) }
            $(
                pub(crate) fn $field(&self) -> &RelativeSpacing { &self.$field }
                #[cfg_attr(not(test), expect(dead_code, reason = "private spacing edits await complete Theme exposure"))]
                pub(crate) fn $edit(mut self, value: RelativeSpacing) -> Result<Self> {
                    self.$field = value;
                    self.revalidated()
                }
            )+
            fn revalidated(self) -> Result<Self> { Self::new($(self.$field),+) }
            pub(crate) fn relationships(&self) -> [&RelativeSpacing; 6] { [$(&self.$field),+] }
            pub(crate) fn lengths(&self, tick: u32, title: u32, resources: &OperationGenerationResources) -> Result<AxisLengths> {
                let metrics = [LocalMetric::css_pixels(tick)?, LocalMetric::css_pixels(title)?];
                Ok(AxisLengths { $($field: self.$field().product(metrics[$metric], resources)?),+ })
            }
        }

        /// Post-selection exact products; never a second stored appearance value.
        #[derive(Debug)]
        pub(crate) struct AxisLengths { $(pub(crate) $field: ExactLength),+ }
    };
}

axis_spacing! {
    x_tick_label_gap => with_x_tick_label_gap: 0,
    y_tick_label_gap => with_y_tick_label_gap: 0,
    x_title_tick_label_gap => with_x_title_tick_label_gap: 1,
    x_title_without_tick_labels_gap => with_x_title_without_tick_labels_gap: 1,
    y_title_tick_label_gap => with_y_title_tick_label_gap: 1,
    y_title_edge_padding => with_y_title_edge_padding: 1,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AxisTheme {
    pub(crate) frame: Option<FrameTheme>,
    pub(crate) cell_frame: Option<FrameTheme>,
    pub(crate) ticks: Option<TickTheme>,
    pub(crate) grid: Option<StrokeTheme>,
    pub(crate) spacing: AxisSpacing,
}

/// Borrowed chrome selected for a coordinate-axis consumer, not another theme.
pub(crate) struct AxisChrome<'a> {
    pub(crate) frame: Option<&'a FrameTheme>,
    pub(crate) grid: Option<&'a StrokeTheme>,
    ticks: Option<&'a TickTheme>,
}

impl AxisChrome<'_> {
    pub(crate) fn outward(
        &self,
        side: crate::layout::AxisSide,
        present_ticks: bool,
        resources: &OperationGenerationResources,
    ) -> Result<ExactLength> {
        let frame = self.frame.filter(|frame| match side {
            crate::layout::AxisSide::Bottom => frame.sides().bottom(),
            crate::layout::AxisSide::Left => frame.sides().left(),
        });
        let mut length = ExactLength::pixels(
            frame.map_or(0.0, |frame| f64::from(frame.stroke().width()) / 2.0),
            resources,
        )?;
        if let Some(ticks) = self.ticks.filter(|_| present_ticks) {
            length = length
                .add_pixels(f64::from(ticks.length()), resources)?
                .add_pixels(f64::from(ticks.stroke().width()) / 2.0, resources)?;
        }
        Ok(length)
    }
}

impl AxisTheme {
    pub(crate) fn new(
        frame: Option<FrameTheme>,
        cell_frame: Option<FrameTheme>,
        ticks: Option<TickTheme>,
        grid: Option<StrokeTheme>,
        spacing: AxisSpacing,
    ) -> Result<Self> {
        let value = Self {
            frame,
            cell_frame,
            ticks,
            grid,
            spacing,
        };
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<()> {
        if let Some(value) = self.frame() {
            value.validate()?;
        }
        if let Some(value) = self.cell_frame() {
            value.validate()?;
        }
        if let Some(value) = self.ticks() {
            value.validate()?;
        }
        if let Some(value) = self.grid() {
            value.validate()?;
        }
        Ok(())
    }
    pub(crate) fn frame(&self) -> Option<&FrameTheme> {
        self.frame.as_ref()
    }
    pub(crate) fn cell_frame(&self) -> Option<&FrameTheme> {
        self.cell_frame.as_ref()
    }
    pub(crate) fn ticks(&self) -> Option<&TickTheme> {
        self.ticks.as_ref()
    }
    pub(crate) fn grid(&self) -> Option<&StrokeTheme> {
        self.grid.as_ref()
    }
    pub(crate) fn spacing(&self) -> &AxisSpacing {
        &self.spacing
    }
    pub(crate) fn coordinate_chrome(&self) -> AxisChrome<'_> {
        AxisChrome {
            frame: self.frame(),
            grid: self.grid(),
            ticks: self.ticks(),
        }
    }
    /// Cell outlines belong to the native grid; common cell axes paint only ticks
    /// and text. Natural sizing and Matrix planning share this same selection.
    pub(crate) fn cell_axis_chrome(&self) -> AxisChrome<'_> {
        AxisChrome {
            frame: None,
            grid: None,
            ticks: self.ticks(),
        }
    }
    pub(crate) fn with_frame(self, frame: Option<FrameTheme>) -> Result<Self> {
        Self::new(frame, self.cell_frame, self.ticks, self.grid, self.spacing)
    }
    pub(crate) fn with_cell_frame(self, cell_frame: Option<FrameTheme>) -> Result<Self> {
        Self::new(self.frame, cell_frame, self.ticks, self.grid, self.spacing)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private tick replacement awaits complete Theme exposure"
        )
    )]
    pub(crate) fn with_ticks(self, ticks: Option<TickTheme>) -> Result<Self> {
        Self::new(self.frame, self.cell_frame, ticks, self.grid, self.spacing)
    }
    pub(crate) fn with_grid(self, grid: Option<StrokeTheme>) -> Result<Self> {
        Self::new(self.frame, self.cell_frame, self.ticks, grid, self.spacing)
    }
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private spacing replacement awaits complete Theme exposure"
        )
    )]
    pub(crate) fn with_spacing(self, spacing: AxisSpacing) -> Result<Self> {
        Self::new(self.frame, self.cell_frame, self.ticks, self.grid, spacing)
    }

    pub(crate) fn share(&self, resources: &OperationGenerationResources) -> Result<Self> {
        self.validate()?;
        resources
            .claim_visits(6)
            .map_err(|error| Error::Render(format!("axis sharing: {error:?}")))?;
        Ok(self.clone())
    }

    pub(crate) fn same(
        &self,
        other: &Self,
        resources: &OperationGenerationResources,
    ) -> Result<bool> {
        self.validate()?;
        other.validate()?;
        resources
            .claim_visits(1)
            .map_err(|error| Error::Render(format!("axis identity: {error:?}")))?;
        if self.frame != other.frame
            || self.cell_frame != other.cell_frame
            || self.ticks != other.ticks
            || self.grid != other.grid
        {
            return Ok(false);
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
        Ok(true)
    }

    pub(crate) fn observe_feedback(&self, round: &mut FeedbackRound) -> Result<()> {
        for stroke in [
            self.frame().map(FrameTheme::stroke),
            self.cell_frame().map(FrameTheme::stroke),
            self.ticks().map(TickTheme::stroke),
            self.grid(),
        ] {
            round.observe_count(usize::from(stroke.is_some()))?;
            if let Some(stroke) = stroke {
                round.observe_number(f64::from(stroke.width()))?;
                round.observe_count(stroke.pattern() as usize)?;
                for channel in [
                    stroke.color().red(),
                    stroke.color().green(),
                    stroke.color().blue(),
                    stroke.color().alpha(),
                ] {
                    round.observe_count(usize::from(channel))?;
                }
            }
        }
        for frame in [self.frame(), self.cell_frame()].into_iter().flatten() {
            for side in frame.sides().drawing_order() {
                round.observe_count(usize::from(side))?;
            }
        }
        if let Some(ticks) = self.ticks() {
            round.observe_count(ticks.length() as usize)?;
        }
        for spacing in self.spacing().relationships() {
            spacing.observe_feedback(round)?;
        }
        Ok(())
    }
}

impl AxisLengths {
    pub(crate) fn before(
        value: &ExactLength,
        anchor: f64,
        resources: &OperationGenerationResources,
    ) -> Result<f64> {
        Self::pixels(value.before(anchor, resources)?)
    }
    /// Position/preference consequence of one exact local product.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "boundary fixtures inspect standalone products; painted positions retain their complete anchors"
        )
    )]
    pub(crate) fn position(
        value: &ExactLength,
        resources: &OperationGenerationResources,
    ) -> Result<f64> {
        Self::pixels(value.round(SpatialRounding::NearestEven, resources)?)
    }

    pub(crate) fn hard(
        value: &ExactLength,
        resources: &OperationGenerationResources,
    ) -> Result<f64> {
        Self::pixels(value.round(SpatialRounding::Ceil, resources)?)
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "the checked quanta bound guarantees exact integer conversion"
    )]
    fn pixels(value: crate::layout::kernel::LayoutScalar) -> Result<f64> {
        if value.quanta().unsigned_abs() > (1_u64 << 53) {
            return Err(Error::Render(
                "axis length cannot retain exact CSS quanta".into(),
            ));
        }
        Ok(value.quanta() as f64 / 1024.0)
    }
}

#[cfg(test)]
mod tests;
