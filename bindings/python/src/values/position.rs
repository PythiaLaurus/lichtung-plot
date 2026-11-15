//! Opaque authored text references with independently qualified native axes.

use lichtung_plot as core;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use super::text::PyDataCoordinate;
use crate::native_error;

#[pyclass(
    name = "TextCoordinate",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyTextCoordinate {
    pub(crate) native: core::TextCoordinate,
}

#[pymethods]
impl PyTextCoordinate {
    #[staticmethod]
    fn data(value: PyRef<'_, PyDataCoordinate>) -> Self {
        let native = value.native.clone();
        drop(value);
        Self {
            native: core::TextCoordinate::data(native),
        }
    }

    #[staticmethod]
    fn data_area_fraction(value: f64) -> PyResult<Self> {
        Ok(Self {
            native: core::TextCoordinate::data_area_fraction(value).map_err(native_error)?,
        })
    }

    #[staticmethod]
    fn figure_fraction(value: f64) -> PyResult<Self> {
        Ok(Self {
            native: core::TextCoordinate::figure_fraction(value).map_err(native_error)?,
        })
    }
}

#[pyclass(
    name = "TextPosition",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyTextPosition {
    pub(crate) native: core::TextPosition,
}

#[pymethods]
impl PyTextPosition {
    #[staticmethod]
    #[pyo3(signature = (*, x, y))]
    fn coordinates(x: PyRef<'_, PyTextCoordinate>, y: PyRef<'_, PyTextCoordinate>) -> Self {
        let horizontal = x.native.clone();
        let vertical = y.native.clone();
        drop(x);
        drop(y);
        Self {
            native: core::TextPosition::new(horizontal, vertical),
        }
    }

    #[staticmethod]
    fn corner(value: &str) -> PyResult<Self> {
        let corner = match value {
            "top_left" => core::DataAreaCorner::TopLeft,
            "top_right" => core::DataAreaCorner::TopRight,
            "bottom_left" => core::DataAreaCorner::BottomLeft,
            "bottom_right" => core::DataAreaCorner::BottomRight,
            _ => {
                return Err(PyValueError::new_err(
                    "corner must be 'top_left', 'top_right', 'bottom_left', or 'bottom_right'",
                ))
            }
        };
        Ok(Self {
            native: core::TextPosition::corner(corner),
        })
    }
}

pub(super) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<super::PyTextCoordinate>()?;
    m.add_class::<super::PyTextPosition>()
}
