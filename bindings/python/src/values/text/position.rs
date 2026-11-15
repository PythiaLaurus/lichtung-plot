//! Scalar authored data references required by data-direction orientation.

use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::{core, is_omitted, native_error, omitted};

#[pyclass(
    name = "DataCoordinate",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(super) struct PyDataCoordinate {
    pub(super) native: core::DataCoordinate,
}

#[pymethods]
impl PyDataCoordinate {
    #[staticmethod]
    fn numeric(value: f64) -> PyResult<Self> {
        Ok(Self {
            native: core::DataCoordinate::numeric(value).map_err(native_error)?,
        })
    }

    #[staticmethod]
    fn category(raw: String) -> Self {
        Self {
            native: core::DataCoordinate::category(raw),
        }
    }

    #[staticmethod]
    fn category_series(category: String, series: String) -> Self {
        Self {
            native: core::DataCoordinate::category_series(category, series),
        }
    }

    #[getter]
    fn kind(&self) -> &'static str {
        if self.native.get_numeric_value().is_some() {
            "numeric"
        } else if self.native.get_category_value().is_some() {
            "category"
        } else if self.native.get_category_series_value().is_some() {
            "category_series"
        } else {
            unreachable!("a native coordinate has one authored branch")
        }
    }

    #[getter]
    fn value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        if let Some(value) = self.native.get_numeric_value() {
            Ok(value.into_pyobject(py)?.into_any())
        } else if let Some(value) = self.native.get_category_value() {
            Ok(value.into_pyobject(py)?.into_any())
        } else {
            let value = self
                .native
                .get_category_series_value()
                .expect("a native coordinate has one authored branch");
            Ok(value.into_pyobject(py)?.into_any())
        }
    }

    #[pyo3(signature = (*, value = omitted()),
        text_signature = "($self, *, value=...)")]
    fn copy_with(&self, py: Python<'_>, value: Py<PyAny>) -> PyResult<Self> {
        let value = &value.into_bound(py);
        if is_omitted(value) {
            return Ok(self.clone());
        }
        if self.native.get_numeric_value().is_some() {
            Self::numeric(value.extract()?)
        } else if self.native.get_category_value().is_some() {
            Ok(Self::category(value.extract()?))
        } else {
            let (category, series) = value.cast::<PyTuple>()?.extract::<(String, String)>()?;
            Ok(Self::category_series(category, series))
        }
    }
}

#[pyclass(
    name = "DataPosition",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(super) struct PyDataPosition {
    pub(super) native: core::DataPosition,
}

#[pymethods]
impl PyDataPosition {
    #[new]
    #[pyo3(signature = (*, x, y))]
    fn new(x: PyRef<'_, PyDataCoordinate>, y: PyRef<'_, PyDataCoordinate>) -> Self {
        let horizontal = x.native.clone();
        let vertical = y.native.clone();
        drop(x);
        drop(y);
        Self {
            native: core::DataPosition::new(horizontal, vertical),
        }
    }

    #[getter]
    fn x(&self) -> PyDataCoordinate {
        PyDataCoordinate {
            native: self.native.get_x().clone(),
        }
    }

    #[getter]
    fn y(&self) -> PyDataCoordinate {
        PyDataCoordinate {
            native: self.native.get_y().clone(),
        }
    }

    #[pyo3(signature = (*, x = omitted(), y = omitted()),
        text_signature = "($self, *, x=..., y=...)")]
    fn copy_with(&self, py: Python<'_>, x: Py<PyAny>, y: Py<PyAny>) -> PyResult<Self> {
        let x = &x.into_bound(py);
        let y = &y.into_bound(py);
        let x = if is_omitted(x) {
            self.native.get_x().clone()
        } else {
            x.extract::<PyRef<'_, PyDataCoordinate>>()?.native.clone()
        };
        let y = if is_omitted(y) {
            self.native.get_y().clone()
        } else {
            y.extract::<PyRef<'_, PyDataCoordinate>>()?.native.clone()
        };
        Ok(Self {
            native: core::DataPosition::new(x, y),
        })
    }
}
