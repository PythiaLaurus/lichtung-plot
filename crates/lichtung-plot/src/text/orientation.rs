//! Finite screen angles and scientific direction requests.

use super::layout::Finite;
use crate::{DataPosition, Error, Result};

pub(crate) fn components(angle: f64) -> (f64, f64) {
    match angle {
        0.0 => (0.0, 1.0),
        90.0 => (1.0, 0.0),
        -90.0 => (-1.0, 0.0),
        -180.0 => (0.0, -1.0),
        _ => angle.to_radians().sin_cos(),
    }
}

/// One normalized clockwise screen angle, without decimal quantization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRotation(pub(crate) Finite);
impl TextRotation {
    /// Normalize finite degrees to [-180, 180), including signed zero.
    pub fn degrees(value: f64) -> Result<Self> {
        if !value.is_finite() {
            return Err(Error::Data("text rotation must be finite".into()));
        }
        let value = value % 360.0;
        Ok(Self(Finite::new(if value >= 180.0 {
            value - 360.0
        } else if value < -180.0 {
            value + 360.0
        } else {
            value
        })?))
    }
    /// No rotation.
    pub const fn horizontal() -> Self {
        Self(Finite::constant(0.0))
    }
    /// Forty-five degrees counterclockwise on screen.
    pub const fn tilt_up() -> Self {
        Self(Finite::constant(-45.0))
    }
    /// Forty-five degrees clockwise on screen.
    pub const fn tilt_down() -> Self {
        Self(Finite::constant(45.0))
    }
    /// Text reads upward on screen.
    pub const fn vertical_up() -> Self {
        Self(Finite::constant(-90.0))
    }
    /// Text reads downward on screen.
    pub const fn vertical_down() -> Self {
        Self(Finite::constant(90.0))
    }

    /// Retained normalized clockwise screen angle in degrees.
    #[must_use]
    pub const fn get_degrees(&self) -> f64 {
        self.0.get()
    }
}

/// How the selected block point relates to a rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRotationMode {
    /// Rotate the complete block around its selected point.
    RotateAboutAnchor,
    /// Align the rotated bounding box to the reference.
    AlignAfterRotation,
}

/// A fixed screen angle, projected data direction or ordered tick-angle program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextOrientation(pub(crate) Orientation);
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Orientation {
    Fixed(TextRotation),
    Data(DataPosition, DataPosition),
    Automatic(Vec<TextRotation>),
}
impl TextOrientation {
    /// Use one fixed screen angle.
    pub const fn fixed(value: TextRotation) -> Self {
        Self(Orientation::Fixed(value))
    }
    /// Follow the direction between two actual scientific references.
    pub fn data_direction(from: DataPosition, to: DataPosition) -> Self {
        Self(Orientation::Data(from, to))
    }
    /// Try the retained angle order on an actual tick group.
    pub fn automatic_ticks(angles: Vec<TextRotation>) -> Self {
        Self(Orientation::Automatic(angles))
    }

    /// Fixed screen rotation, absent for the other orientation programs.
    #[must_use]
    pub const fn get_fixed_rotation(&self) -> Option<TextRotation> {
        match &self.0 {
            Orientation::Fixed(rotation) => Some(*rotation),
            Orientation::Data(..) | Orientation::Automatic(_) => None,
        }
    }

    /// Borrow the authored data-direction endpoints without projecting them.
    #[must_use]
    pub const fn get_data_direction(&self) -> Option<(&DataPosition, &DataPosition)> {
        match &self.0 {
            Orientation::Data(from, to) => Some((from, to)),
            Orientation::Fixed(_) | Orientation::Automatic(_) => None,
        }
    }

    /// Borrow the retained tick-angle order; an empty program remains present.
    #[must_use]
    pub fn get_automatic_angles(&self) -> Option<&[TextRotation]> {
        match &self.0 {
            Orientation::Automatic(angles) => Some(angles),
            Orientation::Fixed(_) | Orientation::Data(..) => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct TextReferenceResolver<'v, 'src> {
    pub(crate) registered: &'v crate::orchestrator::registration::PlotRegistration<'src>,
    pub(crate) layout: &'v crate::layout::Layout,
    pub(crate) dimensions: crate::orchestrator::assignment::PlotMeasureDimensions,
}
impl std::fmt::Debug for TextReferenceResolver<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextReferenceResolver")
            .field("layout", &self.layout)
            .finish_non_exhaustive()
    }
}
impl TextReferenceResolver<'_, '_> {
    #[expect(
        clippy::type_complexity,
        reason = "preserves the original narrow direction comparison projection"
    )]
    pub(crate) fn snapshot(
        &self,
    ) -> (
        [(f64, f64); 2],
        crate::layout::PixelRect,
        [Option<Vec<(f64, f64)>>; 2],
        Option<String>,
    ) {
        (
            [
                self.layout.projection.x_range(),
                self.layout.projection.y_range(),
            ],
            self.layout.projection.panel_box(),
            [
                self.layout.x_axis.assigned_cells.clone(),
                self.layout.y_axis.assigned_cells.clone(),
            ],
            self.registered.scope.render_name.clone(),
        )
    }

    #[expect(
        clippy::type_complexity,
        reason = "preserves the original narrow direction comparison projection"
    )]
    pub(crate) fn matches_snapshot(
        &self,
        other: &(
            [(f64, f64); 2],
            crate::layout::PixelRect,
            [Option<Vec<(f64, f64)>>; 2],
            Option<String>,
        ),
    ) -> bool {
        other.0[0] == self.layout.projection.x_range()
            && other.0[1] == self.layout.projection.y_range()
            && other.1 == self.layout.projection.panel_box()
            && other.2[0] == self.layout.x_axis.assigned_cells
            && other.2[1] == self.layout.y_axis.assigned_cells
            && other.3.as_deref() == self.registered.scope.render_name.as_deref()
    }
    pub(crate) fn rotation(
        &self,
        from: &DataPosition,
        to: &DataPosition,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<TextRotation> {
        let point = |point: &DataPosition| {
            let position = crate::TextPosition::new(
                crate::TextCoordinate::data(point.x.clone()),
                crate::TextCoordinate::data(point.y.clone()),
            );
            Ok::<_, Error>(crate::text::position::resolve::point(
                crate::text::position::resolve::resolve(&position, self, theme)?,
                self.layout.projection.panel_box(),
            ))
        };
        let a = point(from)?;
        let b = point(to)?;
        if a.x.to_bits() == b.x.to_bits() && a.y.to_bits() == b.y.to_bits() {
            return Err(Error::Render(
                "text data direction projects to coincident points".into(),
            ));
        }
        TextRotation::degrees((a.y - b.y).atan2(b.x - a.x).to_degrees())
    }
}
