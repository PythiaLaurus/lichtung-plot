//! Python references live at GC-visible holders, never inside native closures.

use std::sync::Weak;

use lichtung_plot as core;
use pyo3::conversion::FromPyObjectOwned;
use pyo3::exceptions::{PyMemoryError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

mod context;
mod gc;
mod interpreter;
mod operation;
mod scope;

pub(crate) use gc::CallbackHolder;
pub(crate) use operation::with_operation;

use gc::CallbackCell;
use operation::{CallbackKind, CallbackSite};

/// Every element is a real Python strong reference belonging to this root set.
pub(crate) type CallbackRoots = Vec<Py<CallbackHolder>>;

/// Copy actual references, not a shared Rust container hiding their multiplicity.
pub(crate) fn clone_roots(py: Python<'_>, roots: &[Py<CallbackHolder>]) -> PyResult<CallbackRoots> {
    let mut copied = Vec::new();
    copied
        .try_reserve(roots.len())
        .map_err(|error| PyMemoryError::new_err(error.to_string()))?;
    copied.extend(roots.iter().map(|root| root.clone_ref(py)));
    Ok(copied)
}

pub(crate) fn ensure_live(py: Python<'_>, roots: &[Py<CallbackHolder>]) -> PyResult<()> {
    for root in roots {
        root.try_borrow(py)?.ensure_live(py)?;
    }
    Ok(())
}

/// Callback result conversion must not suppress a sequence's Python length
/// exception or panic on an unchecked first-party capacity hint.
pub(crate) fn sequence<'py, T: FromPyObjectOwned<'py>>(
    value: &Bound<'py, PyAny>,
) -> PyResult<Vec<T>> {
    sequence_with_messages(
        value,
        "numeric callback sequences must not be str",
        "numeric callback values must be indexable sequences, not mappings or iterator-only inputs",
    )
}

/// Reuse the same fallible indexed protocol for immutable configuration inputs.
pub(crate) fn sequence_with_messages<'py, T: FromPyObjectOwned<'py>>(
    value: &Bound<'py, PyAny>,
    string_error: &'static str,
    sequence_error: &'static str,
) -> PyResult<Vec<T>> {
    if value.is_instance_of::<PyString>() {
        return Err(PyTypeError::new_err(string_error));
    }
    if value.is_instance_of::<PyDict>() {
        return Err(PyTypeError::new_err(sequence_error));
    }
    // PySequence's positive cast requires ABC registration. Instead, exclude
    // registered mappings with a fallible check, then use the indexed protocol.
    // Special methods live on the type; hasattr must preserve metaclass errors.
    let mapping = PyModule::import(value.py(), "collections.abc")?.getattr("Mapping")?;
    if value.is_instance(&mapping)? || !value.get_type().hasattr("__getitem__")? {
        return Err(PyTypeError::new_err(sequence_error));
    }
    let length = value.len()?;
    let mut values = Vec::new();
    values
        .try_reserve(length)
        .map_err(|error| PyMemoryError::new_err(error.to_string()))?;
    for item in value.try_iter()? {
        let value = item?.extract::<T>().map_err(Into::into)?;
        values
            .try_reserve(1)
            .map_err(|error| PyMemoryError::new_err(error.to_string()))?;
        values.push(value);
    }
    Ok(values)
}

fn invoke<T>(
    target: &Weak<CallbackCell>,
    kind: CallbackKind,
    site: CallbackSite,
    call: impl for<'py> FnOnce(Python<'py>, &Bound<'py, PyAny>) -> PyResult<T>,
) -> core::Result<T> {
    // No attachment, Python reference access, or weak upgrade substitutes for the
    // operation's actual strong holder pins.
    let frame = operation::current_frame()?;
    let callback_cell = target
        .upgrade()
        .ok_or_else(|| core::Error::Render("Python callback has been cleared".into()))?;
    frame.check_callback(callback_cell.interpreter())?;
    Python::attach(|py| {
        let interpreter = interpreter::current(py)
            .map_err(|_| core::Error::Render("cannot identify the callback interpreter".into()))?;
        if interpreter != frame.interpreter() {
            // Do not bind, clone, or destroy a foreign-interpreter Python object.
            return Err(core::Error::Render(
                "Python callback attachment changed the originating interpreter".into(),
            ));
        }
        let callable = callback_cell.callable(py)?;
        match call(py, callable.bind(py)) {
            Ok(value) => Ok(value),
            Err(error) => {
                frame.record_failure(kind, site, error);
                Err(core::Error::Render("Python callback failed".into()))
            }
        }
    })
}

pub(crate) fn value_function(
    py: Python<'_>,
    format: core::NumericFormat,
    callable: &Bound<'_, PyAny>,
) -> PyResult<(core::NumericFormat, CallbackRoots)> {
    let (target, roots) = gc::holder(py, callable)?;
    let native = format.value_function(move |view| {
        invoke(
            &target,
            CallbackKind::NumericValue,
            CallbackSite::numeric(view.set()),
            |py, callable| {
                scope::with_value_context(view, |key| {
                    key.check(py)?;
                    let context = Py::new(py, context::PyNumericValueContext::new(key))?;
                    callable.call1((context,))?.extract::<String>()
                })
            },
        )
    });
    Ok((native, roots))
}

pub(crate) fn set_function(
    py: Python<'_>,
    format: core::NumericFormat,
    callable: &Bound<'_, PyAny>,
) -> PyResult<(core::NumericFormat, CallbackRoots)> {
    let (target, roots) = gc::holder(py, callable)?;
    let native = format.set_function(move |view| {
        invoke(
            &target,
            CallbackKind::NumericSet,
            CallbackSite::numeric(view),
            |py, callable| {
                scope::with_set_context(view, |key| {
                    key.check(py)?;
                    let context = Py::new(py, context::PyNumericSetContext::new(key))?;
                    let returned = callable.call1((context,))?;
                    if returned.is_instance_of::<crate::values::numeric::PyNumericLabels>() {
                        return Ok(returned
                            .extract::<PyRef<'_, crate::values::numeric::PyNumericLabels>>()?
                            .native
                            .clone());
                    }
                    if returned.is_instance_of::<PyString>() {
                        return Err(PyTypeError::new_err(
                            "a set callback must return NumericLabels or a sequence of str",
                        ));
                    }
                    // Cardinality, label spelling and explanation validation
                    // remain native, after Python result extraction succeeds.
                    sequence::<String>(&returned).map(core::NumericLabels::new)
                })
            },
        )
    });
    Ok((native, roots))
}

pub(crate) fn tick_function(
    py: Python<'_>,
    callable: &Bound<'_, PyAny>,
) -> PyResult<(core::TickGeneration, CallbackRoots)> {
    let (target, roots) = gc::holder(py, callable)?;
    let native = core::TickGeneration::function(move |view| {
        invoke(
            &target,
            CallbackKind::Tick,
            CallbackSite::tick(view),
            |py, callable| {
                scope::with_tick_context(view, |key| {
                    key.check(py)?;
                    let context = Py::new(py, context::PyTickContext::new(key))?;
                    let returned = callable.call1((context,))?;
                    if returned.is_instance_of::<crate::values::numeric::PyTickPositions>() {
                        return Ok(returned
                            .extract::<PyRef<'_, crate::values::numeric::PyTickPositions>>()?
                            .native
                            .clone());
                    }
                    sequence::<f64>(&returned).map(core::TickPositions::Data)
                })
            },
        )
    });
    Ok((native, roots))
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<CallbackHolder>()?;
    context::register(module)
}
