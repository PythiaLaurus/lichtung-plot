//! Complete explicit numeric tokens, without a named-ID or host-locale mirror.

use lichtung_plot as core;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::callbacks::sequence;
use crate::native_error;
use crate::values::{is_omitted, omitted};

fn repeat(value: &str) -> PyResult<core::NumericGroupRepeat> {
    match value {
        "cycle" => Ok(core::NumericGroupRepeat::Cycle),
        "last" => Ok(core::NumericGroupRepeat::Last),
        "stop" => Ok(core::NumericGroupRepeat::Stop),
        _ => Err(PyValueError::new_err(
            "repeat must be 'cycle', 'last', or 'stop'",
        )),
    }
}

#[pyclass(
    name = "NumericGrouping",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyNumericGrouping {
    pub(crate) native: core::NumericGrouping,
}

#[pymethods]
impl PyNumericGrouping {
    #[new]
    #[pyo3(signature = (separator, sizes, *, repeat))]
    fn new(separator: String, sizes: &Bound<'_, PyAny>, repeat: &str) -> PyResult<Self> {
        let sizes = sequence(sizes)?;
        let repeat = self::repeat(repeat)?;
        Ok(Self {
            native: core::NumericGrouping::new(separator, sizes, repeat).map_err(native_error)?,
        })
    }

    #[getter]
    fn separator(&self) -> &str {
        self.native.separator()
    }

    #[getter]
    fn sizes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.native.sizes())
    }

    #[getter]
    fn repeat(&self) -> &'static str {
        match self.native.repeat() {
            core::NumericGroupRepeat::Cycle => "cycle",
            core::NumericGroupRepeat::Last => "last",
            core::NumericGroupRepeat::Stop => "stop",
        }
    }

    #[pyo3(signature = (*, separator = omitted(), sizes = omitted(), repeat = omitted()),
        text_signature = "($self, *, separator=..., sizes=..., repeat=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        separator: Py<PyAny>,
        sizes: Py<PyAny>,
        repeat: Py<PyAny>,
    ) -> PyResult<Self> {
        let separator = &separator.into_bound(py);
        let sizes = &sizes.into_bound(py);
        let repeat = &repeat.into_bound(py);
        let separator = if is_omitted(separator) {
            self.native.separator().to_owned()
        } else {
            separator.extract()?
        };
        let sizes = if is_omitted(sizes) {
            self.native.sizes().to_vec()
        } else {
            sequence(sizes)?
        };
        let repeat = if is_omitted(repeat) {
            self.native.repeat()
        } else {
            self::repeat(repeat.extract::<&str>()?)?
        };
        Ok(Self {
            native: core::NumericGrouping::new(separator, sizes, repeat).map_err(native_error)?,
        })
    }
}

#[pyclass(
    name = "NumericSymbols",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyNumericSymbols {
    pub(crate) native: core::NumericSymbols,
}

#[pymethods]
impl PyNumericSymbols {
    #[new]
    fn new(
        positive: String,
        negative: String,
        percent: String,
        per_mille: String,
    ) -> PyResult<Self> {
        Ok(Self {
            native: core::NumericSymbols::new(positive, negative, percent, per_mille)
                .map_err(native_error)?,
        })
    }

    #[getter]
    fn positive(&self) -> &str {
        self.native.positive()
    }

    #[getter]
    fn negative(&self) -> &str {
        self.native.negative()
    }

    #[getter]
    fn percent(&self) -> &str {
        self.native.percent()
    }

    #[getter]
    fn per_mille(&self) -> &str {
        self.native.per_mille()
    }

    #[pyo3(signature = (*, positive = omitted(), negative = omitted(), percent = omitted(), per_mille = omitted()),
        text_signature = "($self, *, positive=..., negative=..., percent=..., per_mille=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        positive: Py<PyAny>,
        negative: Py<PyAny>,
        percent: Py<PyAny>,
        per_mille: Py<PyAny>,
    ) -> PyResult<Self> {
        let positive = &positive.into_bound(py);
        let negative = &negative.into_bound(py);
        let percent = &percent.into_bound(py);
        let per_mille = &per_mille.into_bound(py);
        Self::new(
            if is_omitted(positive) {
                self.native.positive().to_owned()
            } else {
                positive.extract()?
            },
            if is_omitted(negative) {
                self.native.negative().to_owned()
            } else {
                negative.extract()?
            },
            if is_omitted(percent) {
                self.native.percent().to_owned()
            } else {
                percent.extract()?
            },
            if is_omitted(per_mille) {
                self.native.per_mille().to_owned()
            } else {
                per_mille.extract()?
            },
        )
    }
}

#[pyclass(
    name = "NumericCurrency",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyNumericCurrency {
    pub(crate) native: core::NumericCurrency,
}

#[pymethods]
impl PyNumericCurrency {
    #[new]
    fn new(prefix: String, suffix: String) -> PyResult<Self> {
        Ok(Self {
            native: core::NumericCurrency::new(prefix, suffix).map_err(native_error)?,
        })
    }

    #[getter]
    fn prefix(&self) -> &str {
        self.native.prefix()
    }

    #[getter]
    fn suffix(&self) -> &str {
        self.native.suffix()
    }

    #[pyo3(signature = (*, prefix = omitted(), suffix = omitted()),
        text_signature = "($self, *, prefix=..., suffix=...)")]
    fn copy_with(&self, py: Python<'_>, prefix: Py<PyAny>, suffix: Py<PyAny>) -> PyResult<Self> {
        let prefix = &prefix.into_bound(py);
        let suffix = &suffix.into_bound(py);
        Self::new(
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

fn digit_tokens(value: &Bound<'_, PyAny>) -> PyResult<[String; 10]> {
    sequence::<String>(value)?
        .try_into()
        .map_err(|_| PyValueError::new_err("digits must contain exactly ten string tokens"))
}

#[pyclass(
    name = "NumericLocale",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyNumericLocale {
    pub(crate) native: core::NumericLocale,
}

#[pymethods]
impl PyNumericLocale {
    #[staticmethod]
    fn invariant() -> Self {
        Self {
            native: core::NumericLocale::invariant(),
        }
    }

    #[staticmethod]
    fn named(id: &str) -> PyResult<Self> {
        Ok(Self {
            native: core::NumericLocale::named(id).map_err(native_error)?,
        })
    }

    #[staticmethod]
    fn named_ids(py: Python<'_>) -> PyResult<Bound<'_, PyTuple>> {
        PyTuple::new(py, core::NumericLocale::named_ids())
    }

    #[staticmethod]
    #[pyo3(signature = (decimal, digits, *, grouping, symbols, currency))]
    fn custom(
        decimal: String,
        digits: &Bound<'_, PyAny>,
        grouping: Option<PyRef<'_, PyNumericGrouping>>,
        symbols: PyRef<'_, PyNumericSymbols>,
        currency: PyRef<'_, PyNumericCurrency>,
    ) -> PyResult<Self> {
        let pattern = grouping.as_ref().map(|value| value.native.clone());
        let tokens = symbols.native.clone();
        let affixes = currency.native.clone();
        drop(grouping);
        drop(symbols);
        drop(currency);
        let digits = digit_tokens(digits)?;
        Ok(Self {
            native: core::NumericLocale::custom(decimal, digits, pattern, tokens, affixes)
                .map_err(native_error)?,
        })
    }

    #[getter]
    fn decimal(&self) -> &str {
        self.native.decimal()
    }

    #[getter]
    fn digits<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.native.digits())
    }

    #[getter]
    fn grouping(&self) -> Option<PyNumericGrouping> {
        self.native
            .grouping()
            .cloned()
            .map(|native| PyNumericGrouping { native })
    }

    #[getter]
    fn symbols(&self) -> PyNumericSymbols {
        PyNumericSymbols {
            native: self.native.symbols().clone(),
        }
    }

    #[getter]
    fn currency(&self) -> PyNumericCurrency {
        PyNumericCurrency {
            native: self.native.currency().clone(),
        }
    }

    #[pyo3(signature = (*, decimal = omitted(), digits = omitted(), grouping = omitted(), symbols = omitted(), currency = omitted()),
        text_signature = "($self, *, decimal=..., digits=..., grouping=..., symbols=..., currency=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        decimal: Py<PyAny>,
        digits: Py<PyAny>,
        grouping: Py<PyAny>,
        symbols: Py<PyAny>,
        currency: Py<PyAny>,
    ) -> PyResult<Self> {
        let decimal = &decimal.into_bound(py);
        let digits = &digits.into_bound(py);
        let grouping = &grouping.into_bound(py);
        let symbols = &symbols.into_bound(py);
        let currency = &currency.into_bound(py);
        let decimal = if is_omitted(decimal) {
            self.native.decimal().to_owned()
        } else {
            decimal.extract()?
        };
        let digits = if is_omitted(digits) {
            self.native.digits().clone()
        } else {
            digit_tokens(digits)?
        };
        let grouping = if is_omitted(grouping) {
            self.native.grouping().cloned()
        } else {
            let incoming = grouping.extract::<Option<PyRef<'_, PyNumericGrouping>>>()?;
            let pattern = incoming.as_ref().map(|value| value.native.clone());
            drop(incoming);
            pattern
        };
        let symbols = if is_omitted(symbols) {
            self.native.symbols().clone()
        } else {
            let incoming = symbols.extract::<PyRef<'_, PyNumericSymbols>>()?;
            let tokens = incoming.native.clone();
            drop(incoming);
            tokens
        };
        let currency = if is_omitted(currency) {
            self.native.currency().clone()
        } else {
            let incoming = currency.extract::<PyRef<'_, PyNumericCurrency>>()?;
            let affixes = incoming.native.clone();
            drop(incoming);
            affixes
        };
        Ok(Self {
            native: core::NumericLocale::custom(decimal, digits, grouping, symbols, currency)
                .map_err(native_error)?,
        })
    }
}

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyNumericGrouping>()?;
    module.add_class::<PyNumericSymbols>()?;
    module.add_class::<PyNumericCurrency>()?;
    module.add_class::<PyNumericLocale>()
}
