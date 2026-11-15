//! Authored absolute fit programs, never sizes resolved against a Theme role.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use super::{core, is_omitted, native_error, omitted};

string_choice!(grouping, grouping_name, core::TextFitGrouping, "grouping", {
    "independent" => core::TextFitGrouping::Independent,
    "uniform" => core::TextFitGrouping::Uniform,
});

#[pyclass(
    name = "TextFit",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(super) struct PyTextFit {
    pub(super) native: core::TextFit,
}

#[pymethods]
impl PyTextFit {
    #[staticmethod]
    #[pyo3(signature = (*, grouping = "independent"))]
    fn fixed(grouping: &str) -> PyResult<Self> {
        Ok(Self {
            native: core::TextFit::fixed().grouping(self::grouping(grouping)?),
        })
    }

    #[staticmethod]
    #[pyo3(signature = (minimum, *, grouping = "independent"))]
    fn shrink_to(minimum: f64, grouping: &str) -> PyResult<Self> {
        let grouping = self::grouping(grouping)?;
        Ok(Self {
            native: core::TextFit::shrink_to(minimum)
                .map_err(native_error)?
                .grouping(grouping),
        })
    }

    #[staticmethod]
    #[pyo3(signature = (minimum, maximum, *, grouping = "independent"))]
    fn range(minimum: f64, maximum: f64, grouping: &str) -> PyResult<Self> {
        let grouping = self::grouping(grouping)?;
        Ok(Self {
            native: core::TextFit::range(minimum, maximum)
                .map_err(native_error)?
                .grouping(grouping),
        })
    }

    #[getter]
    fn minimum(&self) -> Option<f64> {
        self.native.get_minimum()
    }

    #[getter]
    fn maximum(&self) -> Option<f64> {
        self.native.get_maximum()
    }

    #[getter]
    fn grouping(&self) -> &'static str {
        grouping_name(self.native.get_grouping())
    }

    #[pyo3(signature = (*, grouping = omitted()),
        text_signature = "($self, *, grouping=...)")]
    fn copy_with(&self, py: Python<'_>, grouping: Py<PyAny>) -> PyResult<Self> {
        let grouping = &grouping.into_bound(py);
        let grouping = if is_omitted(grouping) {
            self.native.get_grouping()
        } else {
            self::grouping(grouping.extract::<&str>()?)?
        };
        Ok(Self {
            native: self.native.grouping(grouping),
        })
    }
}
