//! The native label policy, without a parallel mode or aliases store.

use lichtung_plot::Labels;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

#[pyclass(name = "Labels", module = "lichtung_plot._native", frozen)]
pub(crate) struct PyLabels {
    pub(crate) native: Labels,
}

fn policy(mode: &str) -> PyResult<Labels> {
    match mode {
        "humanized" => Ok(Labels::humanized()),
        "raw" => Ok(Labels::raw()),
        _ => Err(PyValueError::new_err(
            "label mode must be 'humanized' or 'raw'",
        )),
    }
}

fn aliases(value: &Bound<'_, PyAny>) -> PyResult<Vec<(String, String)>> {
    crate::callbacks::sequence_with_messages(
        value,
        "label aliases must not be str",
        "label aliases must be an indexable sequence of string pairs, not a mapping or iterator",
    )
}

#[pymethods]
impl PyLabels {
    #[new]
    #[pyo3(signature = (*, mode="humanized", aliases=None))]
    fn new(mode: &str, aliases: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        let mut native = policy(mode)?;
        if let Some(value) = aliases.filter(|value| !value.is_none()) {
            native = native.aliases(self::aliases(value)?);
        }
        Ok(Self { native })
    }

    #[getter]
    fn mode(&self) -> &'static str {
        if self.native.is_humanized() {
            "humanized"
        } else {
            "raw"
        }
    }

    #[getter]
    fn aliases(&self, py: Python<'_>) -> PyResult<Option<Py<PyTuple>>> {
        self.native
            .get_aliases()
            .map(|aliases| PyTuple::new(py, aliases).map(Bound::unbind))
            .transpose()
    }

    #[pyo3(signature = (*, mode=super::omitted(), aliases=super::omitted()),
        text_signature = "($self, *, mode=..., aliases=...)")]
    fn copy_with(&self, py: Python<'_>, mode: Py<PyAny>, aliases: Py<PyAny>) -> PyResult<Self> {
        let mode = &mode.into_bound(py);
        let aliases = &aliases.into_bound(py);
        let mut native = if super::is_omitted(mode) {
            policy(self.mode())?
        } else {
            policy(&mode.extract::<String>()?)?
        };
        if super::is_omitted(aliases) {
            if let Some(aliases) = self.native.get_aliases() {
                native = native.aliases(aliases);
            }
        } else if !aliases.is_none() {
            native = native.aliases(self::aliases(aliases)?);
        }
        Ok(Self { native })
    }

    fn resolve(&self, raw: &str) -> PyResult<String> {
        self.native.resolve(raw).map_err(crate::native_error)
    }
}
