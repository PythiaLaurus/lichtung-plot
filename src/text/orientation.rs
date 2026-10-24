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
}

#[derive(Clone)]
pub(crate) struct DirectionFrame {
    pub(crate) plot: crate::Plot,
    pub(crate) layout: crate::layout::Layout,
}
impl std::fmt::Debug for DirectionFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirectionFrame")
            .field("layout", &self.layout)
            .finish_non_exhaustive()
    }
}
impl DirectionFrame {
    pub(crate) fn same(&self, other: &Self) -> bool {
        self.layout.projection.x_range() == other.layout.projection.x_range()
            && self.layout.projection.y_range() == other.layout.projection.y_range()
            && self.layout.projection.pixel_box() == other.layout.projection.pixel_box()
            && self.layout.x_axis.assigned_cells == other.layout.x_axis.assigned_cells
            && self.layout.y_axis.assigned_cells == other.layout.y_axis.assigned_cells
            && self.plot.axis_scope() == other.plot.axis_scope()
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
                crate::text::position::resolve::resolve(
                    &position,
                    &self.plot,
                    &self.layout,
                    theme,
                )?,
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
