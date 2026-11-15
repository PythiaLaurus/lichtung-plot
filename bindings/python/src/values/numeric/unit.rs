//! Literal unit requests and generated-or-authored explanation instructions.

use lichtung_plot as core;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::values::{is_omitted, omitted, text_intent, PyTextLayout, PyTextTheme};

fn placement(value: &str) -> PyResult<core::NumericUnitPlacement> {
    match value {
        "per_label" => Ok(core::NumericUnitPlacement::PerLabel),
        "shared" => Ok(core::NumericUnitPlacement::Shared),
        _ => Err(PyValueError::new_err(
            "placement must be 'per_label' or 'shared'",
        )),
    }
}

#[pyclass(
    name = "NumericUnit",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyNumericUnit {
    pub(crate) native: core::NumericUnit,
}

#[pymethods]
impl PyNumericUnit {
    #[new]
    #[pyo3(signature = (text, *, placement, separator = " "))]
    fn new(text: String, placement: &str, separator: &str) -> PyResult<Self> {
        Ok(Self {
            native: core::NumericUnit::new(text, self::placement(placement)?).separator(separator),
        })
    }

    #[getter]
    fn text(&self) -> &str {
        self.native.text()
    }

    #[getter]
    fn placement(&self) -> &'static str {
        match self.native.placement() {
            core::NumericUnitPlacement::PerLabel => "per_label",
            core::NumericUnitPlacement::Shared => "shared",
        }
    }

    #[getter]
    fn separator(&self) -> &str {
        self.native.get_separator()
    }

    #[pyo3(signature = (*, text = omitted(), placement = omitted(), separator = omitted()),
        text_signature = "($self, *, text=..., placement=..., separator=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        text: Py<PyAny>,
        placement: Py<PyAny>,
        separator: Py<PyAny>,
    ) -> PyResult<Self> {
        let text = &text.into_bound(py);
        let placement = &placement.into_bound(py);
        let separator = &separator.into_bound(py);
        let text = if is_omitted(text) {
            self.native.text().to_owned()
        } else {
            text.extract::<String>()?
        };
        let placement = if is_omitted(placement) {
            self.native.placement()
        } else {
            self::placement(placement.extract::<&str>()?)?
        };
        let separator = if is_omitted(separator) {
            self.native.get_separator().to_owned()
        } else {
            separator.extract::<String>()?
        };
        Ok(Self {
            native: core::NumericUnit::new(text, placement).separator(separator),
        })
    }
}

fn instructions(
    mut native: core::NumericExplanation,
    style: &Bound<'_, PyAny>,
    layout: &Bound<'_, PyAny>,
) -> PyResult<core::NumericExplanation> {
    if !is_omitted(style) {
        if style.is_none() {
            native = native.clear_style();
        } else {
            let incoming = style.extract::<PyRef<'_, PyTextTheme>>()?;
            let appearance = incoming.native.clone();
            drop(incoming);
            native = native.style(appearance);
        }
    }
    if !is_omitted(layout) {
        if layout.is_none() {
            native = native.clear_layout();
        } else {
            let incoming = layout.extract::<PyRef<'_, PyTextLayout>>()?;
            let request = incoming.native.clone();
            drop(incoming);
            native = native.layout(request);
        }
    }
    Ok(native)
}

#[pyclass(
    name = "NumericExplanation",
    module = "lichtung_plot",
    frozen,
    eq,
    skip_from_py_object
)]
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PyNumericExplanation {
    pub(crate) native: core::NumericExplanation,
}

#[pymethods]
impl PyNumericExplanation {
    #[staticmethod]
    #[pyo3(signature = (*, style = omitted(), layout = omitted()),
        text_signature = "(*, style=..., layout=...)")]
    fn automatic(py: Python<'_>, style: Py<PyAny>, layout: Py<PyAny>) -> PyResult<Self> {
        let style = &style.into_bound(py);
        let layout = &layout.into_bound(py);
        Ok(Self {
            native: instructions(core::NumericExplanation::automatic(), style, layout)?,
        })
    }

    #[staticmethod]
    #[pyo3(signature = (value, *, style = omitted(), layout = omitted()),
        text_signature = "(value, *, style=..., layout=...)")]
    fn text(
        py: Python<'_>,
        value: &Bound<'_, PyAny>,
        style: Py<PyAny>,
        layout: Py<PyAny>,
    ) -> PyResult<Self> {
        let style = &style.into_bound(py);
        let layout = &layout.into_bound(py);
        let unset = omitted().into_bound(py);
        let prepared = text_intent(value, style, layout, &unset)?;
        Ok(Self {
            native: core::NumericExplanation::text(prepared.native),
        })
    }

    #[pyo3(signature = (*, style = omitted(), layout = omitted()),
        text_signature = "($self, *, style=..., layout=...)")]
    fn copy_with(&self, py: Python<'_>, style: Py<PyAny>, layout: Py<PyAny>) -> PyResult<Self> {
        let style = &style.into_bound(py);
        let layout = &layout.into_bound(py);
        Ok(Self {
            native: instructions(self.native.clone(), style, layout)?,
        })
    }
}

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyNumericUnit>()?;
    module.add_class::<PyNumericExplanation>()
}
