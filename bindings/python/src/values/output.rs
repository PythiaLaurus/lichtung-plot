//! Immutable call-local PNG options; scientific sizing validation stays native.

use lichtung_plot as core;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;

use super::{is_omitted, omitted};

#[pyclass(
    name = "PngSizing",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq)]
pub(crate) struct PyPngSizing {
    pub(crate) native: core::PngSizing,
}

impl PyPngSizing {
    fn from_value(kind: &str, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let native = match kind {
            "source" if value.is_none() => core::PngSizing::Source,
            "source" => return Err(PyTypeError::new_err("source sizing takes no value")),
            "dpi" => core::PngSizing::Dpi(value.extract()?),
            "scale" => core::PngSizing::Scale(value.extract()?),
            "width" => core::PngSizing::Width(value.extract()?),
            "height" => core::PngSizing::Height(value.extract()?),
            _ => {
                return Err(PyValueError::new_err(
                    "kind must be 'source', 'dpi', 'scale', 'width', or 'height'",
                ))
            }
        };
        Ok(Self { native })
    }
}

#[pymethods]
impl PyPngSizing {
    #[new]
    #[pyo3(signature = (kind, value = None))]
    fn new(kind: &str, value: Option<&Bound<'_, PyAny>>, py: Python<'_>) -> PyResult<Self> {
        let none = py.None();
        Self::from_value(kind, value.unwrap_or(none.bind(py)))
    }

    #[getter]
    fn kind(&self) -> &'static str {
        match self.native {
            core::PngSizing::Source => "source",
            core::PngSizing::Dpi(_) => "dpi",
            core::PngSizing::Scale(_) => "scale",
            core::PngSizing::Width(_) => "width",
            core::PngSizing::Height(_) => "height",
        }
    }

    #[getter]
    fn value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match self.native {
            core::PngSizing::Source => Ok(py.None().into_bound(py)),
            core::PngSizing::Dpi(value) | core::PngSizing::Scale(value) => {
                Ok(value.into_pyobject(py)?.into_any())
            }
            core::PngSizing::Width(value) | core::PngSizing::Height(value) => {
                Ok(value.into_pyobject(py)?.into_any())
            }
        }
    }

    #[pyo3(signature = (*, kind = omitted(), value = omitted()),
        text_signature = "($self, *, kind=..., value=...)")]
    fn copy_with(&self, py: Python<'_>, kind: Py<PyAny>, value: Py<PyAny>) -> PyResult<Self> {
        let kind = &kind.into_bound(py);
        let value = &value.into_bound(py);
        if is_omitted(kind) && is_omitted(value) {
            return Ok(self.clone());
        }
        let kind = if is_omitted(kind) {
            self.kind()
        } else {
            kind.extract()?
        };
        if is_omitted(value) {
            Self::from_value(kind, &self.value(py)?)
        } else {
            Self::from_value(kind, value)
        }
    }
}

fn compression(value: &str) -> PyResult<core::PngCompression> {
    match value {
        "none" => Ok(core::PngCompression::None),
        "fast" => Ok(core::PngCompression::Fast),
        "balanced" => Ok(core::PngCompression::Balanced),
        "high" => Ok(core::PngCompression::High),
        _ => Err(PyValueError::new_err(
            "compression must be 'none', 'fast', 'balanced', or 'high'",
        )),
    }
}

#[pyclass(
    name = "PngOptions",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq)]
pub(crate) struct PyPngOptions {
    pub(crate) native: core::PngOptions,
}

#[pymethods]
impl PyPngOptions {
    #[new]
    #[pyo3(signature = (*, sizing = omitted(), compression = "balanced"),
        text_signature = "(*, sizing=..., compression='balanced')")]
    fn new(py: Python<'_>, sizing: Py<PyAny>, compression: &str) -> PyResult<Self> {
        let sizing = &sizing.into_bound(py);
        Ok(Self {
            native: core::PngOptions {
                sizing: if is_omitted(sizing) {
                    core::PngOptions::default().sizing
                } else {
                    sizing.extract::<PyRef<'_, PyPngSizing>>()?.native
                },
                compression: self::compression(compression)?,
            },
        })
    }

    #[getter]
    fn sizing(&self) -> PyPngSizing {
        PyPngSizing {
            native: self.native.sizing,
        }
    }

    #[getter]
    fn compression(&self) -> &'static str {
        match self.native.compression {
            core::PngCompression::None => "none",
            core::PngCompression::Fast => "fast",
            core::PngCompression::Balanced => "balanced",
            core::PngCompression::High => "high",
        }
    }

    #[pyo3(signature = (*, sizing = omitted(), compression = omitted()),
        text_signature = "($self, *, sizing=..., compression=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        sizing: Py<PyAny>,
        compression: Py<PyAny>,
    ) -> PyResult<Self> {
        let sizing = &sizing.into_bound(py);
        let compression = &compression.into_bound(py);
        Ok(Self {
            native: core::PngOptions {
                sizing: if is_omitted(sizing) {
                    self.native.sizing
                } else {
                    sizing.extract::<PyRef<'_, PyPngSizing>>()?.native
                },
                compression: if is_omitted(compression) {
                    self.native.compression
                } else {
                    self::compression(compression.extract::<&str>()?)?
                },
            },
        })
    }
}
