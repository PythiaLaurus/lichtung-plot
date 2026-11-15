//! Complete native locator requests; clones retain opaque context binding.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyString;
use pyo3::{PyTraverseError, PyVisit};

use super::{callbacks, core, is_omitted, native_error, omitted, PyTickGeneration};

fn space(value: &str) -> PyResult<core::TickSpace> {
    match value {
        "data" => Ok(core::TickSpace::Data),
        "scale" => Ok(core::TickSpace::Scale),
        _ => Err(PyValueError::new_err("space must be 'data' or 'scale'")),
    }
}

fn space_name(value: core::TickSpace) -> &'static str {
    match value {
        core::TickSpace::Data => "data",
        core::TickSpace::Scale => "scale",
    }
}

fn requirement(value: &str) -> PyResult<core::TickRequirement> {
    match value {
        "fixed" => Ok(core::TickRequirement::Fixed),
        "preferred" => Ok(core::TickRequirement::Preferred),
        _ => Err(PyValueError::new_err(
            "requirement must be 'fixed' or 'preferred'",
        )),
    }
}

fn requirement_name(value: core::TickRequirement) -> &'static str {
    match value {
        core::TickRequirement::Fixed => "fixed",
        core::TickRequirement::Preferred => "preferred",
    }
}

fn singleton(value: &Bound<'_, PyAny>) -> PyResult<core::TickSingleton> {
    if value.is_instance_of::<PyString>() {
        match value.extract::<&str>()? {
            "lower" => Ok(core::TickSingleton::Lower),
            "midpoint" => Ok(core::TickSingleton::Midpoint),
            "upper" => Ok(core::TickSingleton::Upper),
            _ => Err(PyValueError::new_err(
                "singleton must be 'lower', 'midpoint', 'upper', or a numeric position",
            )),
        }
    } else {
        Ok(core::TickSingleton::Position(value.extract()?))
    }
}

type IntervalParameters = (f64, f64, &'static str, &'static str);
type MultiplicativeParameters = (f64, f64, &'static str);

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
    fn log10_decades(decades: u32) -> PyResult<Self> {
        core::TickGeneration::log10_decades(decades)
            .map(Self::plain)
            .map_err(native_error)
    }

    #[staticmethod]
    #[pyo3(signature = (step, origin, *, space, requirement))]
    fn interval(step: f64, origin: f64, space: &str, requirement: &str) -> PyResult<Self> {
        core::TickGeneration::interval(
            step,
            origin,
            self::space(space)?,
            self::requirement(requirement)?,
        )
        .map(Self::plain)
        .map_err(native_error)
    }

    #[staticmethod]
    #[pyo3(signature = (anchor, ratio, *, requirement))]
    fn multiplicative(anchor: f64, ratio: f64, requirement: &str) -> PyResult<Self> {
        core::TickGeneration::multiplicative(anchor, ratio, self::requirement(requirement)?)
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
    #[pyo3(signature = (count, *, space="data", singleton=omitted()),
        text_signature = "(count, *, space='data', singleton='lower')")]
    fn exact_count(
        py: Python<'_>,
        count: usize,
        space: &str,
        singleton: Py<PyAny>,
    ) -> PyResult<Self> {
        let singleton = &singleton.into_bound(py);
        let mut native = core::TickGeneration::exact_count(count, self::space(space)?);
        if !is_omitted(singleton) {
            native = native
                .singleton(self::singleton(singleton)?)
                .map_err(native_error)?;
        }
        Ok(Self::plain(native))
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

    #[getter]
    fn additive_step(&self, py: Python<'_>) -> PyResult<Option<f64>> {
        Ok(self.live_native(py)?.additive_step())
    }

    #[getter]
    fn decade_step(&self, py: Python<'_>) -> PyResult<Option<u32>> {
        Ok(self.live_native(py)?.decade_step())
    }

    #[getter]
    fn interval_parameters(&self, py: Python<'_>) -> PyResult<Option<IntervalParameters>> {
        Ok(self
            .live_native(py)?
            .interval_parameters()
            .map(|(step, origin, space, requirement)| {
                (
                    step,
                    origin,
                    space_name(space),
                    requirement_name(requirement),
                )
            }))
    }

    #[getter]
    fn multiplicative_parameters(
        &self,
        py: Python<'_>,
    ) -> PyResult<Option<MultiplicativeParameters>> {
        Ok(self
            .live_native(py)?
            .multiplicative_parameters()
            .map(|(anchor, ratio, requirement)| (anchor, ratio, requirement_name(requirement))))
    }

    #[pyo3(signature = (*, singleton=omitted(), maximum_count=omitted()),
        text_signature = "($self, *, singleton=..., maximum_count=...)")]
    fn copy_with(
        slf: &Bound<'_, Self>,
        py: Python<'_>,
        singleton: Py<PyAny>,
        maximum_count: Py<PyAny>,
    ) -> PyResult<Self> {
        let (mut native, roots) = {
            let borrowed = slf.try_borrow()?;
            borrowed.snapshot(py)?
        };
        let singleton = &singleton.into_bound(py);
        let maximum_count = &maximum_count.into_bound(py);
        // Decode both inputs after ending the slf PyRef, before native edits.
        let singleton = (!is_omitted(singleton))
            .then(|| self::singleton(singleton))
            .transpose()?;
        let maximum_count = (!is_omitted(maximum_count))
            .then(|| maximum_count.extract())
            .transpose()?;
        if let Some(value) = singleton {
            native = native.singleton(value).map_err(native_error)?;
        }
        if let Some(value) = maximum_count {
            native = native.maximum_count(value).map_err(native_error)?;
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
