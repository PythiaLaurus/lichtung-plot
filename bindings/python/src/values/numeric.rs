//! Native numeric policies and owned callback results; no Python formatter.

use lichtung_plot as core;
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyString, PyTuple};
use pyo3::{PyTraverseError, PyVisit};

use super::{is_omitted, omitted, PyText};
use crate::callbacks::{self, CallbackRoots};
use crate::native_error;

fn cleared() -> PyErr {
    PyRuntimeError::new_err("numeric configuration has been cleared by garbage collection")
}

fn field(width: Option<usize>) -> core::Result<Option<core::NumericField>> {
    width
        .map(|width| core::NumericField::new(width, ' ', core::NumericAlignment::Right))
        .transpose()
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

    fn construct(
        py: Python<'_>,
        fractional_digits: u8,
        field_width: Option<usize>,
        value_function: Option<&Bound<'_, PyAny>>,
        set_function: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        if value_function.is_some() && set_function.is_some() {
            return Err(PyTypeError::new_err(
                "value_function and set_function are mutually exclusive",
            ));
        }
        let native = core::NumericFormat::new(
            core::NumericNotation::Decimal,
            core::NumericPrecision::Fractional(fractional_digits),
        )
        .and_then(|native| native.field(field(field_width)?))
        .map_err(native_error)?;
        let (native, roots) = if let Some(callable) = value_function {
            callbacks::value_function(py, native, callable)?
        } else if let Some(callable) = set_function {
            callbacks::set_function(py, native, callable)?
        } else {
            (native, Vec::new())
        };
        Ok(Self {
            native: Some(native),
            roots,
        })
    }
}

#[pymethods]
impl PyNumericFormat {
    #[new]
    #[pyo3(signature = (*, fractional_digits=2, field_width=None, value_function=None, set_function=None))]
    fn new(
        py: Python<'_>,
        fractional_digits: u8,
        field_width: Option<usize>,
        value_function: Option<&Bound<'_, PyAny>>,
        set_function: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        Self::construct(
            py,
            fractional_digits,
            field_width,
            value_function,
            set_function,
        )
    }

    #[staticmethod]
    #[pyo3(signature = (callback, *, fractional_digits=2, field_width=None))]
    fn value_function(
        py: Python<'_>,
        callback: &Bound<'_, PyAny>,
        fractional_digits: u8,
        field_width: Option<usize>,
    ) -> PyResult<Self> {
        Self::construct(py, fractional_digits, field_width, Some(callback), None)
    }

    #[staticmethod]
    #[pyo3(signature = (callback, *, fractional_digits=2, field_width=None))]
    fn set_function(
        py: Python<'_>,
        callback: &Bound<'_, PyAny>,
        fractional_digits: u8,
        field_width: Option<usize>,
    ) -> PyResult<Self> {
        Self::construct(py, fractional_digits, field_width, None, Some(callback))
    }

    #[getter]
    fn fractional_digits(&self, py: Python<'_>) -> PyResult<Option<u8>> {
        Ok(match self.live_native(py)?.get_precision() {
            core::NumericPrecision::Fractional(value) => Some(value),
            _ => None,
        })
    }

    #[pyo3(signature = (*, fractional_digits=omitted(), field_width=omitted(),
        value_function=omitted(), set_function=omitted()),
        text_signature = "($self, *, fractional_digits=..., field_width=..., value_function=..., set_function=...)")]
    fn copy_with(
        slf: &Bound<'_, Self>,
        py: Python<'_>,
        fractional_digits: Py<PyAny>,
        field_width: Py<PyAny>,
        value_function: Py<PyAny>,
        set_function: Py<PyAny>,
    ) -> PyResult<Self> {
        let fractional_digits = &fractional_digits.into_bound(py);
        let field_width = &field_width.into_bound(py);
        let value_function = &value_function.into_bound(py);
        let set_function = &set_function.into_bound(py);
        let value_present = !is_omitted(value_function);
        let set_present = !is_omitted(set_function);
        if value_present && set_present {
            return Err(PyTypeError::new_err(
                "replace only one of value_function and set_function",
            ));
        }
        let (mut native, mut roots) = slf.try_borrow()?.snapshot(py)?;
        if !is_omitted(fractional_digits) {
            native = native
                .precision(core::NumericPrecision::Fractional(
                    fractional_digits.extract()?,
                ))
                .map_err(native_error)?;
        }
        if !is_omitted(field_width) {
            native = native
                .field(field(field_width.extract()?).map_err(native_error)?)
                .map_err(native_error)?;
        }
        if value_present || set_present {
            let callable = if value_present {
                value_function
            } else {
                set_function
            };
            if callable.is_none() {
                native = native.builtin();
                roots.clear();
            } else {
                let replaced = if value_present {
                    callbacks::value_function(py, native, callable)?
                } else {
                    callbacks::set_function(py, native, callable)?
                };
                native = replaced.0;
                roots = replaced.1;
            }
        }
        Ok(Self {
            native: Some(native),
            roots,
        })
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "PyO3 GC traversal requires an owned PyVisit argument"
    )]
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        for root in &self.roots {
            visit.call(root)?;
        }
        Ok(())
    }

    fn __clear__(slf: &Bound<'_, Self>) -> PyResult<()> {
        let retired = {
            let mut value = slf.try_borrow_mut()?;
            (value.native.take(), std::mem::take(&mut value.roots))
        };
        drop(retired);
        Ok(())
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

#[pymethods]
impl PyTickGeneration {
    #[new]
    fn new() -> Self {
        Self::plain(core::TickGeneration::automatic())
    }

    #[staticmethod]
    fn automatic() -> Self {
        Self::new()
    }

    #[staticmethod]
    fn additive(step: f64) -> PyResult<Self> {
        core::TickGeneration::additive(step)
            .map(Self::plain)
            .map_err(native_error)
    }

    #[staticmethod]
    fn target(count: usize) -> PyResult<Self> {
        core::TickGeneration::target(count)
            .map(Self::plain)
            .map_err(native_error)
    }

    #[staticmethod]
    fn maximum(count: usize) -> PyResult<Self> {
        core::TickGeneration::maximum(count)
            .map(Self::plain)
            .map_err(native_error)
    }

    #[staticmethod]
    #[pyo3(signature = (count, *, space="data"))]
    fn exact_count(count: usize, space: &str) -> PyResult<Self> {
        let space = match space {
            "data" => core::TickSpace::Data,
            "scale" => core::TickSpace::Scale,
            _ => return Err(PyValueError::new_err("space must be 'data' or 'scale'")),
        };
        Ok(Self::plain(core::TickGeneration::exact_count(count, space)))
    }

    #[staticmethod]
    fn function(py: Python<'_>, callback: &Bound<'_, PyAny>) -> PyResult<Self> {
        let (native, roots) = callbacks::tick_function(py, callback)?;
        Ok(Self {
            native: Some(native),
            roots,
        })
    }

    #[getter]
    fn is_automatic(&self, py: Python<'_>) -> PyResult<bool> {
        Ok(self.live_native(py)?.is_automatic())
    }

    #[getter]
    fn target_count(&self, py: Python<'_>) -> PyResult<Option<usize>> {
        Ok(self.live_native(py)?.target_count())
    }

    #[getter]
    fn maximum_count(&self, py: Python<'_>) -> PyResult<Option<usize>> {
        Ok(self.live_native(py)?.get_maximum_count())
    }

    #[pyo3(signature = (*, maximum_count=omitted()),
        text_signature = "($self, *, maximum_count=...)")]
    fn copy_with(
        slf: &Bound<'_, Self>,
        py: Python<'_>,
        maximum_count: Py<PyAny>,
    ) -> PyResult<Self> {
        let maximum_count = &maximum_count.into_bound(py);
        let (mut native, roots) = slf.try_borrow()?.snapshot(py)?;
        if !is_omitted(maximum_count) {
            native = native
                .maximum_count(maximum_count.extract()?)
                .map_err(native_error)?;
        }
        Ok(Self {
            native: Some(native),
            roots,
        })
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "PyO3 GC traversal requires an owned PyVisit argument"
    )]
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        for root in &self.roots {
            visit.call(root)?;
        }
        Ok(())
    }

    fn __clear__(slf: &Bound<'_, Self>) -> PyResult<()> {
        let retired = {
            let mut value = slf.try_borrow_mut()?;
            (value.native.take(), std::mem::take(&mut value.roots))
        };
        drop(retired);
        Ok(())
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

fn explanation(value: &Bound<'_, PyAny>) -> PyResult<Option<core::Text>> {
    if value.is_none() {
        Ok(None)
    } else if value.is_instance_of::<PyString>() {
        Ok(Some(core::Text::new(value.extract::<String>()?)))
    } else {
        Ok(Some(value.extract::<PyRef<'_, PyText>>()?.native.clone()))
    }
}

#[pyclass(name = "NumericLabels", module = "lichtung_plot", frozen)]
pub(crate) struct PyNumericLabels {
    pub(crate) native: core::NumericLabels,
}

#[pymethods]
impl PyNumericLabels {
    #[new]
    #[pyo3(signature = (labels, *, explanation=None))]
    fn new(labels: &Bound<'_, PyAny>, explanation: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        let mut native = core::NumericLabels::new(callbacks::sequence(labels)?);
        if let Some(text) = explanation.map(self::explanation).transpose()?.flatten() {
            native = native.explanation(text);
        }
        Ok(Self { native })
    }

    #[getter]
    fn labels<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.native.labels())
    }

    #[getter]
    fn explanation(&self) -> Option<PyText> {
        self.native
            .get_explanation()
            .cloned()
            .map(|native| PyText { native })
    }

    #[pyo3(signature = (*, labels=omitted(), explanation=omitted()),
        text_signature = "($self, *, labels=..., explanation=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        labels: Py<PyAny>,
        explanation: Py<PyAny>,
    ) -> PyResult<Self> {
        let labels = &labels.into_bound(py);
        let explanation = &explanation.into_bound(py);
        let labels = if is_omitted(labels) {
            self.native.labels().to_vec()
        } else {
            callbacks::sequence(labels)?
        };
        let explanation = if is_omitted(explanation) {
            self.native.get_explanation().cloned()
        } else {
            self::explanation(explanation)?
        };
        let mut native = core::NumericLabels::new(labels);
        if let Some(explanation) = explanation {
            native = native.explanation(explanation);
        }
        Ok(Self { native })
    }
}

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyNumericFormat>()?;
    module.add_class::<PyTickGeneration>()?;
    module.add_class::<PyTickPositions>()?;
    module.add_class::<PyNumericLabels>()
}
