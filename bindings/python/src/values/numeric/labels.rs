//! Owned callback labels use the same authored Text preparation as owners.

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::{callbacks, core, is_omitted, omitted, text_intent, PyNumericLabels, PyText};

fn replacement(
    py: Python<'_>,
    current: Option<&core::Text>,
    incoming: Option<&Bound<'_, PyAny>>,
    style: &Bound<'_, PyAny>,
    layout: &Bound<'_, PyAny>,
) -> PyResult<Option<core::Text>> {
    let unset = omitted().into_bound(py);
    if let Some(value) = incoming {
        if !value.is_none() {
            return Ok(Some(text_intent(value, style, layout, &unset)?.native));
        }
    } else if is_omitted(style) && is_omitted(layout) {
        return Ok(current.cloned());
    } else if let Some(native) = current {
        let selected = Py::new(
            py,
            PyText {
                native: native.clone(),
            },
        )?
        .into_bound(py);
        return Ok(Some(
            text_intent(selected.as_any(), style, layout, &unset)?.native,
        ));
    }
    // TextIntent clears instructions on this replacement Text only. Its absence
    // is not a clear command for the enclosing format's explanation slot.
    if !is_omitted(style) || !is_omitted(layout) {
        return Err(PyTypeError::new_err(
            "style and layout require an actual str or Text explanation",
        ));
    }
    Ok(None)
}

#[pymethods]
impl PyNumericLabels {
    #[new]
    #[pyo3(signature = (labels, *, explanation=None, style=omitted(), layout=omitted()),
        text_signature = "(labels, *, explanation=None, style=..., layout=...)")]
    fn new(
        py: Python<'_>,
        labels: &Bound<'_, PyAny>,
        explanation: Option<&Bound<'_, PyAny>>,
        style: Py<PyAny>,
        layout: Py<PyAny>,
    ) -> PyResult<Self> {
        let style = &style.into_bound(py);
        let layout = &layout.into_bound(py);
        let labels = callbacks::sequence(labels)?;
        let explanation = replacement(py, None, explanation, style, layout)?;
        let mut native = core::NumericLabels::new(labels);
        if let Some(explanation) = explanation {
            native = native.explanation(explanation);
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

    #[pyo3(signature = (*, labels=omitted(), explanation=omitted(), style=omitted(), layout=omitted()),
        text_signature = "($self, *, labels=..., explanation=..., style=..., layout=...)")]
    fn copy_with(
        &self,
        py: Python<'_>,
        labels: Py<PyAny>,
        explanation: Py<PyAny>,
        style: Py<PyAny>,
        layout: Py<PyAny>,
    ) -> PyResult<Self> {
        let labels = &labels.into_bound(py);
        let explanation = &explanation.into_bound(py);
        let style = &style.into_bound(py);
        let layout = &layout.into_bound(py);
        let labels = if is_omitted(labels) {
            self.native.labels().to_vec()
        } else {
            callbacks::sequence(labels)?
        };
        let explanation = replacement(
            py,
            self.native.get_explanation(),
            (!is_omitted(explanation)).then_some(explanation),
            style,
            layout,
        )?;
        let mut native = core::NumericLabels::new(labels);
        if let Some(explanation) = explanation {
            native = native.explanation(explanation);
        }
        Ok(Self { native })
    }
}
