//! One GC object owns each cell's one physical callable reference.

use std::sync::{Arc, Mutex, MutexGuard, Weak};

use lichtung_plot as core;
use pyo3::exceptions::{PyMemoryError, PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3::{PyTraverseError, PyVisit};

use super::interpreter::{self, InterpreterId};
use super::CallbackRoots;

pub(super) struct CallbackCell {
    interpreter: InterpreterId,
    callable: Mutex<Option<Py<PyAny>>>,
}

impl CallbackCell {
    fn lock(&self) -> MutexGuard<'_, Option<Py<PyAny>>> {
        // Poisoning never conceals an edge from GC. All locked operations below
        // are reference inspection/movement; none calls Python.
        self.callable
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(super) const fn interpreter(&self) -> InterpreterId {
        self.interpreter
    }

    pub(super) fn callable(&self, py: Python<'_>) -> core::Result<Py<PyAny>> {
        self.lock()
            .as_ref()
            .map(|callable| callable.clone_ref(py))
            .ok_or_else(|| core::Error::Render("Python callback has been cleared".into()))
    }
}

/// There is exactly one Python holder per cell. Copies clone this Python handle;
/// they must not create another holder around a cloned Arc of the same cell.
#[pyclass(name = "_CallbackHolder", module = "lichtung_plot._native")]
pub(crate) struct CallbackHolder {
    cell: Arc<CallbackCell>,
}

impl CallbackHolder {
    pub(super) fn ensure_live(&self, py: Python<'_>) -> PyResult<()> {
        if interpreter::current(py)? != self.cell.interpreter {
            return Err(PyRuntimeError::new_err(
                "callback belongs to another Python interpreter",
            ));
        }
        if self.cell.lock().is_none() {
            return Err(PyRuntimeError::new_err(
                "callback has been cleared by garbage collection",
            ));
        }
        Ok(())
    }
}

#[pymethods]
impl CallbackHolder {
    #[expect(
        clippy::needless_pass_by_value,
        reason = "PyO3 GC traversal requires an owned PyVisit argument"
    )]
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        // No attach, Python calls, clone_ref, try_lock, or recursive traversal.
        // Native closures hold Weak only, so this is the callable's sole visitor.
        visit.call(&*self.cell.lock())
    }

    fn __clear__(slf: &Bound<'_, Self>) -> PyResult<()> {
        let retired = {
            let holder = slf.try_borrow()?;
            let mut callable = holder.cell.lock();
            let retired = callable.take();
            drop(callable);
            retired
        };
        // Both the PyO3 borrow and mutex guard are gone before DECREF can run a
        // callable finalizer and reenter this holder or a configuration owner.
        drop(retired);
        Ok(())
    }
}

pub(super) fn holder(
    py: Python<'_>,
    callable: &Bound<'_, PyAny>,
) -> PyResult<(Weak<CallbackCell>, CallbackRoots)> {
    if !callable.is_callable() {
        return Err(PyTypeError::new_err("callback must be callable"));
    }
    let mut roots = Vec::new();
    roots
        .try_reserve(1)
        .map_err(|error| PyMemoryError::new_err(error.to_string()))?;
    let cell = Arc::new(CallbackCell {
        interpreter: interpreter::current(py)?,
        callable: Mutex::new(Some(callable.clone().unbind())),
    });
    let target = Arc::downgrade(&cell);
    roots.push(Py::new(py, CallbackHolder { cell })?);
    Ok((target, roots))
}
