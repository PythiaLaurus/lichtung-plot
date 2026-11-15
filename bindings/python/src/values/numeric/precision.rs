//! Owned precision choices and opaque native scalar-width fields.

use lichtung_plot as core;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;

use crate::native_error;
use crate::values::{is_omitted, omitted};

fn replacement(value: &Bound<'_, PyAny>, inherited: u8) -> PyResult<u8> {
    if is_omitted(value) {
        Ok(inherited)
    } else {
        value.extract()
    }
}

fn require_omitted(values: &[&Bound<'_, PyAny>]) -> PyResult<()> {
    if values.iter().any(|value| !is_omitted(value)) {
        return Err(PyTypeError::new_err(
            "copy_with edits only payloads of the current precision branch",
        ));
    }
    Ok(())
}

#[pyclass(
    name = "NumericPrecision",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyNumericPrecision {
    pub(crate) native: core::NumericPrecision,
}

#[pymethods]
impl PyNumericPrecision {
    #[staticmethod]
    fn fractional(digits: u8) -> Self {
        Self {
            native: core::NumericPrecision::Fractional(digits),
        }
    }

    #[staticmethod]
    fn significant(digits: u8) -> Self {
        Self {
            native: core::NumericPrecision::Significant(digits),
        }
    }

    #[staticmethod]
    fn adaptive_fractional(minimum: u8, maximum: u8) -> Self {
        Self {
            native: core::NumericPrecision::AdaptiveFractional {
                min: minimum,
                max: maximum,
            },
        }
    }

    #[staticmethod]
    fn adaptive_significant(minimum: u8, maximum: u8) -> Self {
        Self {
            native: core::NumericPrecision::AdaptiveSignificant {
                min: minimum,
                max: maximum,
            },
        }
    }

    #[getter]
    fn kind(&self) -> &'static str {
        match self.native {
            core::NumericPrecision::Fractional(_) => "fractional",
            core::NumericPrecision::Significant(_) => "significant",
            core::NumericPrecision::AdaptiveFractional { .. } => "adaptive_fractional",
            core::NumericPrecision::AdaptiveSignificant { .. } => "adaptive_significant",
        }
    }

    #[getter]
    fn digits(&self) -> Option<u8> {
        match self.native {
            core::NumericPrecision::Fractional(digits)
            | core::NumericPrecision::Significant(digits) => Some(digits),
            core::NumericPrecision::AdaptiveFractional { .. }
            | core::NumericPrecision::AdaptiveSignificant { .. } => None,
        }
    }

    #[getter]
    fn minimum(&self) -> Option<u8> {
        match self.native {
            core::NumericPrecision::AdaptiveFractional { min, .. }
            | core::NumericPrecision::AdaptiveSignificant { min, .. } => Some(min),
            core::NumericPrecision::Fractional(_) | core::NumericPrecision::Significant(_) => None,
        }
    }

    #[getter]
    fn maximum(&self) -> Option<u8> {
        match self.native {
            core::NumericPrecision::AdaptiveFractional { max, .. }
            | core::NumericPrecision::AdaptiveSignificant { max, .. } => Some(max),
            core::NumericPrecision::Fractional(_) | core::NumericPrecision::Significant(_) => None,
        }
    }

    #[pyo3(signature = (*, digits = omitted(), minimum = omitted(), maximum = omitted()),
        text_signature = "($self, *, digits=..., minimum=..., maximum=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        digits: Py<PyAny>,
        minimum: Py<PyAny>,
        maximum: Py<PyAny>,
    ) -> PyResult<Self> {
        let digits = &digits.into_bound(py);
        let minimum = &minimum.into_bound(py);
        let maximum = &maximum.into_bound(py);
        let native = match self.native {
            core::NumericPrecision::Fractional(inherited) => {
                require_omitted(&[minimum, maximum])?;
                core::NumericPrecision::Fractional(replacement(digits, inherited)?)
            }
            core::NumericPrecision::Significant(inherited) => {
                require_omitted(&[minimum, maximum])?;
                core::NumericPrecision::Significant(replacement(digits, inherited)?)
            }
            core::NumericPrecision::AdaptiveFractional { min, max } => {
                require_omitted(&[digits])?;
                core::NumericPrecision::AdaptiveFractional {
                    min: replacement(minimum, min)?,
                    max: replacement(maximum, max)?,
                }
            }
            core::NumericPrecision::AdaptiveSignificant { min, max } => {
                require_omitted(&[digits])?;
                core::NumericPrecision::AdaptiveSignificant {
                    min: replacement(minimum, min)?,
                    max: replacement(maximum, max)?,
                }
            }
        };
        Ok(Self { native })
    }
}

fn alignment(value: &str) -> PyResult<core::NumericAlignment> {
    match value {
        "left" => Ok(core::NumericAlignment::Left),
        "right" => Ok(core::NumericAlignment::Right),
        "center" => Ok(core::NumericAlignment::Center),
        "after_sign" => Ok(core::NumericAlignment::AfterSign),
        _ => Err(PyValueError::new_err(
            "alignment must be 'left', 'right', 'center', or 'after_sign'",
        )),
    }
}

#[pyclass(
    name = "NumericField",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyNumericField {
    pub(crate) native: core::NumericField,
}

#[pymethods]
impl PyNumericField {
    #[new]
    #[pyo3(signature = (width, *, fill = ' ', alignment = "right"))]
    fn new(width: usize, fill: char, alignment: &str) -> PyResult<Self> {
        Ok(Self {
            native: core::NumericField::new(width, fill, self::alignment(alignment)?)
                .map_err(native_error)?,
        })
    }
}

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyNumericPrecision>()?;
    module.add_class::<PyNumericField>()
}
