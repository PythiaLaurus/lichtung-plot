//! Complete native statistical policies, without retained Python input context.

use lichtung_plot as core;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyString;

use super::{is_omitted, omitted};
use crate::native_error;

macro_rules! scalar_policy {
    ($wrapper:ident, $name:literal, $native:ident, $factory:ident,
        $field:ident, $reader:ident, $default:tt, $copy_signature:literal) => {
        #[pyclass(name = $name, module = "lichtung_plot", frozen, skip_from_py_object)]
        #[derive(Clone)]
        pub(crate) struct $wrapper {
            pub(crate) native: core::$native,
        }

        #[pymethods]
        impl $wrapper {
            #[new]
            #[pyo3(signature = (*, $field = $default))]
            fn new($field: f64) -> PyResult<Self> {
                Ok(Self {
                    native: core::ErrorBar::$factory()
                        .$field($field)
                        .map_err(native_error)?,
                })
            }

            #[getter]
            fn $field(&self) -> f64 {
                self.native.$reader()
            }

            #[pyo3(signature = (*, $field = omitted()), text_signature = $copy_signature)]
            fn copy_with(&self, py: Python<'_>, $field: Py<PyAny>) -> PyResult<Self> {
                let $field = &$field.into_bound(py);
                let native = if is_omitted($field) {
                    self.native
                } else {
                    self.native
                        .$field($field.extract()?)
                        .map_err(native_error)?
                };
                Ok(Self { native })
            }
        }
    };
}

scalar_policy!(
    PyStandardDeviation,
    "StandardDeviation",
    StandardDeviation,
    standard_deviation,
    scale,
    get_scale,
    1.0,
    "($self, *, scale=...)"
);
scalar_policy!(
    PyStandardError,
    "StandardError",
    StandardError,
    standard_error,
    scale,
    get_scale,
    1.0,
    "($self, *, scale=...)"
);
scalar_policy!(
    PyPercentileInterval,
    "PercentileInterval",
    PercentileInterval,
    percentile_interval,
    level,
    get_level,
    0.95,
    "($self, *, level=...)"
);

#[pyclass(
    name = "Bootstrap",
    module = "lichtung_plot",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyBootstrap {
    pub(crate) native: core::Bootstrap,
}

#[pymethods]
impl PyBootstrap {
    #[new]
    #[pyo3(signature = (*, resamples = 1000, seed = 0, units = None, data = None))]
    fn new(
        resamples: usize,
        seed: u64,
        units: Option<&Bound<'_, PyAny>>,
        data: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let mut native = core::Bootstrap::new()
            .resamples(resamples)
            .map_err(native_error)?
            .seed(seed);
        if let Some(units) = units {
            native = native.units(crate::input::bootstrap_units(data, units)?);
        }
        Ok(Self { native })
    }

    #[getter]
    fn resamples(&self) -> usize {
        self.native.get_resamples()
    }

    #[getter]
    fn seed(&self) -> u64 {
        self.native.get_seed()
    }

    #[pyo3(signature = (*, resamples = omitted(), seed = omitted(), units = omitted(), data = None),
        text_signature = "($self, *, resamples=..., seed=..., units=..., data=None)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        resamples: Py<PyAny>,
        seed: Py<PyAny>,
        units: Py<PyAny>,
        data: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let resamples = &resamples.into_bound(py);
        let seed = &seed.into_bound(py);
        let units = &units.into_bound(py);
        let mut native = if units.is_none() {
            // There is deliberately no units reader or Python-side units mirror.
            core::Bootstrap::new()
                .resamples(self.native.get_resamples())
                .map_err(native_error)?
                .seed(self.native.get_seed())
        } else {
            self.native.clone()
        };
        if !is_omitted(resamples) {
            native = native
                .resamples(resamples.extract()?)
                .map_err(native_error)?;
        }
        if !is_omitted(seed) {
            native = native.seed(seed.extract()?);
        }
        if !is_omitted(units) && !units.is_none() {
            native = native.units(crate::input::bootstrap_units(data, units)?);
        }
        Ok(Self { native })
    }
}

#[pyclass(
    name = "ConfidenceInterval",
    module = "lichtung_plot",
    frozen,
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyConfidenceInterval {
    pub(crate) native: core::ConfidenceInterval,
}

fn interval_bootstrap(
    native: core::ConfidenceInterval,
    bootstrap: &Bound<'_, PyAny>,
) -> PyResult<core::ConfidenceInterval> {
    if is_omitted(bootstrap) {
        Ok(native)
    } else if bootstrap.is_none() {
        Ok(native.student_t())
    } else {
        let incoming = bootstrap.extract::<PyRef<'_, PyBootstrap>>()?;
        let snapshot = incoming.native.clone();
        drop(incoming);
        Ok(native.bootstrap(snapshot))
    }
}

#[pymethods]
impl PyConfidenceInterval {
    #[new]
    #[pyo3(signature = (*, level = 0.95, bootstrap = omitted()),
        text_signature = "(*, level=0.95, bootstrap=...)")]
    fn new(py: Python<'_>, level: f64, bootstrap: Py<PyAny>) -> PyResult<Self> {
        let bootstrap = &bootstrap.into_bound(py);
        let native = core::ErrorBar::confidence_interval()
            .level(level)
            .map_err(native_error)?;
        Ok(Self {
            native: interval_bootstrap(native, bootstrap)?,
        })
    }

    #[getter]
    fn level(&self) -> f64 {
        self.native.get_level()
    }

    #[getter]
    fn bootstrap(&self) -> Option<PyBootstrap> {
        self.native
            .get_bootstrap()
            .cloned()
            .map(|native| PyBootstrap { native })
    }

    #[pyo3(signature = (*, level = omitted(), bootstrap = omitted()),
        text_signature = "($self, *, level=..., bootstrap=...)")]
    fn copy_with(&self, py: Python<'_>, level: Py<PyAny>, bootstrap: Py<PyAny>) -> PyResult<Self> {
        let level = &level.into_bound(py);
        let bootstrap = &bootstrap.into_bound(py);
        let mut native = self.native.clone();
        if !is_omitted(level) {
            native = native.level(level.extract()?).map_err(native_error)?;
        }
        Ok(Self {
            native: interval_bootstrap(native, bootstrap)?,
        })
    }
}

/// Snapshot one complete sample-only policy before its consumer borrows an owner.
pub(crate) fn error_bar(value: &Bound<'_, PyAny>) -> PyResult<core::ErrorBar> {
    if let Ok(value) = value.extract::<PyRef<'_, PyStandardDeviation>>() {
        let native = value.native;
        drop(value);
        return Ok(native.into());
    }
    if let Ok(value) = value.extract::<PyRef<'_, PyStandardError>>() {
        let native = value.native;
        drop(value);
        return Ok(native.into());
    }
    if let Ok(value) = value.extract::<PyRef<'_, PyPercentileInterval>>() {
        let native = value.native;
        drop(value);
        return Ok(native.into());
    }
    if let Ok(value) = value.extract::<PyRef<'_, PyConfidenceInterval>>() {
        let native = value.native.clone();
        drop(value);
        return Ok(native.into());
    }
    if value.is_instance_of::<PyString>() && value.extract::<&str>()? == "hidden" {
        return Ok(core::ErrorBar::hidden());
    }
    Err(PyTypeError::new_err(
        "error_bar must be StandardDeviation, StandardError, PercentileInterval, ConfidenceInterval, or 'hidden'",
    ))
}

pub(super) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<super::PyStandardDeviation>()?;
    m.add_class::<super::PyStandardError>()?;
    m.add_class::<super::PyPercentileInterval>()?;
    m.add_class::<super::PyBootstrap>()?;
    m.add_class::<super::PyConfidenceInterval>()
}
