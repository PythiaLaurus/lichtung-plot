//! Immutable Python configuration values backed only by complete native values.

pub(crate) mod numeric;
mod output;
mod text;
mod theme;

pub(crate) use output::PyPngOptions;
pub(crate) use text::{text_intent, PyText, PyTextLayout, TextIntent};
pub(crate) use theme::{PyTextTheme, PyTheme};

use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;

/// A private boundary marker, never retained by a configuration value.
#[pyclass(frozen)]
struct Omitted;

static OMITTED: PyOnceLock<Py<Omitted>> = PyOnceLock::new();

fn omitted() -> Py<PyAny> {
    Python::attach(|py| {
        OMITTED
            .get(py)
            .expect("configuration values are registered before calls")
            .clone_ref(py)
            .into_any()
    })
}

fn is_omitted(value: &Bound<'_, PyAny>) -> bool {
    OMITTED
        .get(value.py())
        .is_some_and(|omitted| value.is(omitted.bind(value.py())))
}

pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    OMITTED.get_or_try_init(m.py(), || Py::new(m.py(), Omitted))?;
    m.add_class::<PyTheme>()?;
    m.add_class::<PyTextTheme>()?;
    theme::register_components(m)?;
    m.add_class::<PyText>()?;
    m.add_class::<PyTextLayout>()?;
    text::register_components(m)?;
    m.add_class::<output::PyPngSizing>()?;
    m.add_class::<PyPngOptions>()?;
    numeric::register(m)
}
