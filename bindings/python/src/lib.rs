//! Python configuration owners and native renderer boundaries.

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

mod callbacks;
mod input;
mod output;
mod owner;
mod values;

pyo3::create_exception!(lichtung_plot, DataError, PyValueError);
pyo3::create_exception!(lichtung_plot, RenderError, PyRuntimeError);

/// Preserve the renderer's three error domains at the Python boundary.
pub(crate) fn native_error(error: lichtung_plot::Error) -> PyErr {
    match error {
        lichtung_plot::Error::Io(error) => error.into(),
        lichtung_plot::Error::Data(message) => DataError::new_err(message),
        lichtung_plot::Error::Render(message) => RenderError::new_err(message),
    }
}

#[pymodule(gil_used = true)]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("DataError", module.py().get_type::<DataError>())?;
    module.add("RenderError", module.py().get_type::<RenderError>())?;
    callbacks::register(module)?;
    owner::register(module)?;
    output::register(module)?;
    values::register(module)
}
