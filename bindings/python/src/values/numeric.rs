//! Complete native numeric policies, physical callback roots and owned results.

use lichtung_plot as core;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::{is_omitted, omitted, text_intent, PyText};
use crate::callbacks::{self, CallbackRoots};
use crate::native_error;

mod format;
mod labels;
mod locale;
mod precision;
mod prefixes;
mod ticks;
mod unit;

fn cleared() -> PyErr {
    PyRuntimeError::new_err("numeric configuration has been cleared by garbage collection")
}

/// Python-immutable; only the GC clear slot can remove its retained native value.
#[pyclass(name = "NumericFormat", module = "lichtung_plot", weakref)]
pub(crate) struct PyNumericFormat {
    native: Option<core::NumericFormat>,
    roots: CallbackRoots,
}

impl PyNumericFormat {
    fn live_native(&self, py: Python<'_>) -> PyResult<&core::NumericFormat> {
        let native = self.native.as_ref().ok_or_else(cleared)?;
        callbacks::ensure_live(py, &self.roots)?;
        Ok(native)
    }

    pub(crate) fn snapshot(
        &self,
        py: Python<'_>,
    ) -> PyResult<(core::NumericFormat, CallbackRoots)> {
        let native = self.live_native(py)?;
        Ok((native.clone(), callbacks::clone_roots(py, &self.roots)?))
    }
}

#[pyclass(name = "TickGeneration", module = "lichtung_plot", weakref)]
pub(crate) struct PyTickGeneration {
    native: Option<core::TickGeneration>,
    roots: CallbackRoots,
}

impl PyTickGeneration {
    fn plain(native: core::TickGeneration) -> Self {
        Self {
            native: Some(native),
            roots: Vec::new(),
        }
    }

    fn live_native(&self, py: Python<'_>) -> PyResult<&core::TickGeneration> {
        let native = self.native.as_ref().ok_or_else(cleared)?;
        callbacks::ensure_live(py, &self.roots)?;
        Ok(native)
    }

    pub(crate) fn snapshot(
        &self,
        py: Python<'_>,
    ) -> PyResult<(core::TickGeneration, CallbackRoots)> {
        let native = self.live_native(py)?;
        Ok((native.clone(), callbacks::clone_roots(py, &self.roots)?))
    }
}

#[pyclass(name = "TickPositions", module = "lichtung_plot", frozen)]
pub(crate) struct PyTickPositions {
    pub(crate) native: core::TickPositions,
}

#[pymethods]
impl PyTickPositions {
    #[staticmethod]
    fn data(values: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Self {
            native: core::TickPositions::Data(callbacks::sequence(values)?),
        })
    }

    #[staticmethod]
    fn log10_exponents(values: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Self {
            native: core::TickPositions::Log10Exponents(callbacks::sequence(values)?),
        })
    }

    #[getter]
    fn kind(&self) -> &'static str {
        match self.native {
            core::TickPositions::Data(_) => "data",
            core::TickPositions::Log10Exponents(_) => "log10_exponents",
        }
    }

    #[getter]
    fn positions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        match &self.native {
            core::TickPositions::Data(values) | core::TickPositions::Log10Exponents(values) => {
                PyTuple::new(py, values)
            }
        }
    }

    #[pyo3(signature = (*, positions=omitted()),
        text_signature = "($self, *, positions=...)")]
    fn copy_with(&self, py: Python<'_>, positions: Py<PyAny>) -> PyResult<Self> {
        let positions = &positions.into_bound(py);
        if is_omitted(positions) {
            return Ok(Self {
                native: self.native.clone(),
            });
        }
        let values = callbacks::sequence(positions)?;
        Ok(Self {
            native: match self.native {
                core::TickPositions::Data(_) => core::TickPositions::Data(values),
                core::TickPositions::Log10Exponents(_) => {
                    core::TickPositions::Log10Exponents(values)
                }
            },
        })
    }
}

#[pyclass(name = "NumericLabels", module = "lichtung_plot", frozen)]
pub(crate) struct PyNumericLabels {
    pub(crate) native: core::NumericLabels,
}

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    precision::register(module)?;
    locale::register(module)?;
    prefixes::register(module)?;
    unit::register(module)?;
    module.add_class::<PyNumericFormat>()?;
    module.add_class::<PyTickGeneration>()?;
    module.add_class::<PyTickPositions>()?;
    module.add_class::<PyNumericLabels>()
}
