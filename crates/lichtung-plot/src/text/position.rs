//! Typed scientific, panel and owning-output references for authored annotations.

use super::layout::Finite;
use crate::{DataAreaCorner, Result};
pub(crate) mod resolve;

/// A typed scientific point, independent of drawing instructions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPosition {
    pub(crate) x: DataCoordinate,
    pub(crate) y: DataCoordinate,
}
impl DataPosition {
    /// Retain independent scientific x and y references.
    pub const fn new(x: DataCoordinate, y: DataCoordinate) -> Self {
        Self { x, y }
    }

    /// Borrow the authored x coordinate.
    #[must_use]
    pub const fn get_x(&self) -> &DataCoordinate {
        &self.x
    }

    /// Borrow the authored y coordinate.
    #[must_use]
    pub const fn get_y(&self) -> &DataCoordinate {
        &self.y
    }
}

/// A raw scientific coordinate. Category and series names are never aliases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataCoordinate(pub(crate) DataValue);
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DataValue {
    Numeric(Finite),
    Category(String),
    Series(String, String),
}
impl DataCoordinate {
    /// A finite numeric data value, transformed by the actual scientific scale.
    pub fn numeric(value: f64) -> Result<Self> {
        Ok(Self(DataValue::Numeric(Finite::new(value)?)))
    }
    /// An exact raw category identity, resolved against the actual local domain.
    pub fn category(raw: impl Into<String>) -> Self {
        Self(DataValue::Category(raw.into()))
    }
    /// An exact raw category and positioned-series identity.
    pub fn category_series(category: impl Into<String>, series: impl Into<String>) -> Self {
        Self(DataValue::Series(category.into(), series.into()))
    }

    /// Numeric payload, absent for category references.
    #[must_use]
    pub const fn get_numeric_value(&self) -> Option<f64> {
        match &self.0 {
            DataValue::Numeric(value) => Some(value.get()),
            DataValue::Category(_) | DataValue::Series(..) => None,
        }
    }

    /// Borrow the raw category of an unqualified category reference.
    #[must_use]
    pub fn get_category_value(&self) -> Option<&str> {
        match &self.0 {
            DataValue::Category(value) => Some(value),
            DataValue::Numeric(_) | DataValue::Series(..) => None,
        }
    }

    /// Borrow the exact raw category and positioned-series identities.
    #[must_use]
    pub fn get_category_series_value(&self) -> Option<(&str, &str)> {
        match &self.0 {
            DataValue::Series(category, series) => Some((category, series)),
            DataValue::Numeric(_) | DataValue::Category(_) => None,
        }
    }
}
/// One independently qualified coordinate of an authored text reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextCoordinate(pub(crate) Coordinate);
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Coordinate {
    Data(DataCoordinate),
    Panel(Finite),
    Figure(Finite),
}
impl TextCoordinate {
    /// Use a scientific coordinate on this actual axis.
    pub const fn data(value: DataCoordinate) -> Self {
        Self(Coordinate::Data(value))
    }
    /// Use a finite fraction of the actual scientific panel, positive y upward.
    pub fn data_area_fraction(value: f64) -> Result<Self> {
        Ok(Self(Coordinate::Panel(Finite::new(value)?)))
    }
    /// Use a finite fraction of the owning output root, including an embedding Canvas.
    pub fn figure_fraction(value: f64) -> Result<Self> {
        Ok(Self(Coordinate::Figure(Finite::new(value)?)))
    }
}
/// A semantic reference, separate from the selected point of the text block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPosition(pub(crate) Position);
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Position {
    Coordinates(TextCoordinate, TextCoordinate),
    Corner(DataAreaCorner),
}
impl TextPosition {
    /// Retain independently qualified x and y coordinates.
    pub const fn new(x: TextCoordinate, y: TextCoordinate) -> Self {
        Self(Position::Coordinates(x, y))
    }
    /// Preserve the panel corner's ordinary inset and placement defaults.
    pub const fn corner(value: DataAreaCorner) -> Self {
        Self(Position::Corner(value))
    }
    /// Inspect corner syntax without inventing one for unrelated coordinate frames.
    pub const fn as_corner(&self) -> Option<DataAreaCorner> {
        match self.0 {
            Position::Corner(value) => Some(value),
            Position::Coordinates(..) => None,
        }
    }
}
