//! The only interpreter-identity FFI boundary; no interpreter switching occurs.

#![allow(unsafe_code)]

use pyo3::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct InterpreterId(i64);

pub(super) fn current(py: Python<'_>) -> PyResult<InterpreterId> {
    // SAFETY: Python proves this thread is attached. These two public CPython
    // APIs are available in the 3.10 stable ABI. The pointer is consumed only by
    // GetID in this scope; it is never stored or used to switch thread states.
    let id = unsafe { pyo3::ffi::PyInterpreterState_GetID(pyo3::ffi::PyInterpreterState_Get()) };
    if id < 0 {
        Err(PyErr::fetch(py))
    } else {
        Ok(InterpreterId(id))
    }
}
