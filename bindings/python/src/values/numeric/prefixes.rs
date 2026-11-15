//! Owned public factor choices and opaque native SI/compact selection policies.

use lichtung_plot as core;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyString;

use crate::callbacks::sequence;
use crate::native_error;
use crate::values::{is_omitted, omitted};

#[pyclass(
    name = "SiPrefix",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PySiPrefix {
    pub(crate) native: core::SiPrefix,
}

#[pymethods]
impl PySiPrefix {
    #[staticmethod]
    fn from_exponent(exponent: i8) -> PyResult<Self> {
        Ok(Self {
            native: core::SiPrefix::from_exponent(exponent).map_err(native_error)?,
        })
    }

    #[getter]
    fn exponent(&self) -> i8 {
        self.native.exponent()
    }

    #[getter]
    fn symbol(&self) -> &str {
        self.native.symbol()
    }
}

#[pyclass(
    name = "NumericCompactRule",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyNumericCompactRule {
    pub(crate) native: core::NumericCompactRule,
}

#[pymethods]
impl PyNumericCompactRule {
    #[new]
    #[pyo3(signature = (threshold, divisor, *, prefix, suffix))]
    fn new(threshold: f64, divisor: f64, prefix: String, suffix: String) -> PyResult<Self> {
        Ok(Self {
            native: core::NumericCompactRule::new(threshold, divisor, prefix, suffix)
                .map_err(native_error)?,
        })
    }

    #[getter]
    fn threshold(&self) -> f64 {
        self.native.threshold()
    }

    #[getter]
    fn divisor(&self) -> f64 {
        self.native.divisor()
    }

    #[getter]
    fn prefix(&self) -> &str {
        self.native.prefix()
    }

    #[getter]
    fn suffix(&self) -> &str {
        self.native.suffix()
    }

    #[pyo3(signature = (*, threshold = omitted(), divisor = omitted(), prefix = omitted(), suffix = omitted()),
        text_signature = "($self, *, threshold=..., divisor=..., prefix=..., suffix=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        threshold: Py<PyAny>,
        divisor: Py<PyAny>,
        prefix: Py<PyAny>,
        suffix: Py<PyAny>,
    ) -> PyResult<Self> {
        let threshold = &threshold.into_bound(py);
        let divisor = &divisor.into_bound(py);
        let prefix = &prefix.into_bound(py);
        let suffix = &suffix.into_bound(py);
        Self::new(
            if is_omitted(threshold) {
                self.native.threshold()
            } else {
                threshold.extract()?
            },
            if is_omitted(divisor) {
                self.native.divisor()
            } else {
                divisor.extract()?
            },
            if is_omitted(prefix) {
                self.native.prefix().to_owned()
            } else {
                prefix.extract()?
            },
            if is_omitted(suffix) {
                self.native.suffix().to_owned()
            } else {
                suffix.extract()?
            },
        )
    }
}

fn require_omitted(values: &[&Bound<'_, PyAny>]) -> PyResult<()> {
    if values.iter().any(|value| !is_omitted(value)) {
        return Err(PyTypeError::new_err(
            "copy_with edits only payloads of the current factor branch",
        ));
    }
    Ok(())
}

#[pyclass(
    name = "NumericFactor",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyNumericFactor {
    pub(crate) native: core::NumericFactor,
}

#[pymethods]
impl PyNumericFactor {
    #[staticmethod]
    fn automatic_decimal() -> Self {
        Self {
            native: core::NumericFactor::AutomaticDecimal,
        }
    }

    #[staticmethod]
    fn from_value(value: f64) -> Self {
        Self {
            native: core::NumericFactor::Value(value),
        }
    }

    #[staticmethod]
    fn power_of_ten(exponent: String) -> Self {
        Self {
            native: core::NumericFactor::PowerOfTen(exponent),
        }
    }

    #[getter]
    fn kind(&self) -> &'static str {
        match &self.native {
            core::NumericFactor::AutomaticDecimal => "automatic_decimal",
            core::NumericFactor::Value(_) => "value",
            core::NumericFactor::PowerOfTen(_) => "power_of_ten",
        }
    }

    #[getter]
    fn value(&self) -> Option<f64> {
        match &self.native {
            core::NumericFactor::Value(value) => Some(*value),
            core::NumericFactor::AutomaticDecimal | core::NumericFactor::PowerOfTen(_) => None,
        }
    }

    #[getter]
    fn exponent(&self) -> Option<&str> {
        match &self.native {
            core::NumericFactor::PowerOfTen(exponent) => Some(exponent),
            core::NumericFactor::AutomaticDecimal | core::NumericFactor::Value(_) => None,
        }
    }

    #[pyo3(signature = (*, value = omitted(), exponent = omitted()),
        text_signature = "($self, *, value=..., exponent=...)")]
    fn copy_with(&self, py: Python<'_>, value: Py<PyAny>, exponent: Py<PyAny>) -> PyResult<Self> {
        let value = &value.into_bound(py);
        let exponent = &exponent.into_bound(py);
        let native = match &self.native {
            core::NumericFactor::AutomaticDecimal => {
                require_omitted(&[value, exponent])?;
                core::NumericFactor::AutomaticDecimal
            }
            core::NumericFactor::Value(inherited) => {
                require_omitted(&[exponent])?;
                core::NumericFactor::Value(if is_omitted(value) {
                    *inherited
                } else {
                    value.extract()?
                })
            }
            core::NumericFactor::PowerOfTen(inherited) => {
                require_omitted(&[value])?;
                core::NumericFactor::PowerOfTen(if is_omitted(exponent) {
                    inherited.clone()
                } else {
                    exponent.extract()?
                })
            }
        };
        Ok(Self { native })
    }
}

fn promotion(value: &str) -> PyResult<core::NumericPromotion> {
    match value {
        "keep_threshold" => Ok(core::NumericPromotion::KeepThreshold),
        "promote_on_rounding" => Ok(core::NumericPromotion::PromoteOnRounding),
        _ => Err(PyValueError::new_err(
            "promotion must be 'keep_threshold' or 'promote_on_rounding'",
        )),
    }
}

fn automatic_selection<T>(value: &str) -> PyResult<core::NumericSelection<T>> {
    match value {
        "per_value" => Ok(core::NumericSelection::PerValue),
        "whole_set" => Ok(core::NumericSelection::WholeSet),
        _ => Err(PyValueError::new_err(
            "selection must be 'per_value', 'whole_set', or a correctly typed fixed choice",
        )),
    }
}

fn si_selection(value: &Bound<'_, PyAny>) -> PyResult<core::NumericSelection<core::SiPrefix>> {
    if let Ok(prefix) = value.extract::<PyRef<'_, PySiPrefix>>() {
        let choice = prefix.native;
        drop(prefix);
        Ok(core::NumericSelection::Fixed(choice))
    } else {
        automatic_selection(value.extract::<&str>()?)
    }
}

fn compact_selection(value: &Bound<'_, PyAny>) -> PyResult<core::NumericSelection<usize>> {
    if value.is_instance_of::<PyString>() {
        automatic_selection(value.extract::<&str>()?)
    } else {
        Ok(core::NumericSelection::Fixed(value.extract()?))
    }
}

#[pyclass(
    name = "NumericPrefixes",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyNumericPrefixes {
    pub(crate) native: core::NumericPrefixes,
}

#[pymethods]
impl PyNumericPrefixes {
    #[staticmethod]
    fn none() -> Self {
        Self {
            native: core::NumericPrefixes::none(),
        }
    }

    #[staticmethod]
    #[pyo3(signature = (selection, *, promotion))]
    fn si(selection: &Bound<'_, PyAny>, promotion: &str) -> PyResult<Self> {
        let choice = si_selection(selection)?;
        let promotion = self::promotion(promotion)?;
        Ok(Self {
            native: core::NumericPrefixes::si(choice, promotion).map_err(native_error)?,
        })
    }

    #[staticmethod]
    #[pyo3(signature = (selection, *, promotion))]
    fn decimal_compact(selection: &Bound<'_, PyAny>, promotion: &str) -> PyResult<Self> {
        let choice = compact_selection(selection)?;
        let promotion = self::promotion(promotion)?;
        Ok(Self {
            native: core::NumericPrefixes::decimal_compact(choice, promotion)
                .map_err(native_error)?,
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rules, *, selection, promotion))]
    fn compact(
        rules: &Bound<'_, PyAny>,
        selection: &Bound<'_, PyAny>,
        promotion: &str,
    ) -> PyResult<Self> {
        let supplied = sequence::<Py<PyNumericCompactRule>>(rules)?;
        let rules = supplied
            .iter()
            .map(|value| value.bind(rules.py()).get().native.clone())
            .collect();
        drop(supplied);
        let choice = compact_selection(selection)?;
        let promotion = self::promotion(promotion)?;
        Ok(Self {
            native: core::NumericPrefixes::compact(rules, choice, promotion)
                .map_err(native_error)?,
        })
    }

    #[staticmethod]
    fn factor(factor: PyRef<'_, PyNumericFactor>) -> PyResult<Self> {
        let divisor = factor.native.clone();
        drop(factor);
        Ok(Self {
            native: core::NumericPrefixes::factor(divisor).map_err(native_error)?,
        })
    }
}

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PySiPrefix>()?;
    module.add_class::<PyNumericCompactRule>()?;
    module.add_class::<PyNumericFactor>()?;
    module.add_class::<PyNumericPrefixes>()
}
