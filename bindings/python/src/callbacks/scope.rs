//! A synchronous portal from Python keys to borrowed public native contexts.
//!
//! Unsafe edges are limited to the three tagged pointer dereferences below.
//! Python objects contain only an Arc identity and selectors, never a pointer.

#![allow(unsafe_code)]

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, ThreadId};

use lichtung_plot as core;
use pyo3::exceptions::{PyMemoryError, PyRuntimeError};
use pyo3::prelude::*;

use super::context::{self, OwnedReply, Request};
use super::interpreter;
use super::operation::{self, OperationToken};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Value,
    Set,
    Tick,
}

struct KeyState {
    thread: ThreadId,
    operation: Option<OperationToken>,
    kind: Kind,
    active: AtomicBool,
    allocation_failed: bool,
}

/// Arc identity cannot be reused while an escaped context still holds this key.
#[derive(Clone)]
pub(crate) struct ContextKey(Arc<KeyState>);

impl ContextKey {
    pub(crate) fn check(&self, py: Python<'_>) -> PyResult<()> {
        if self.0.thread != thread::current().id() {
            return Err(PyRuntimeError::new_err(
                "callback context belongs to another thread",
            ));
        }
        if self.0.allocation_failed {
            return Err(PyMemoryError::new_err(
                "cannot reserve a native callback context scope",
            ));
        }
        if !self.0.active.load(Ordering::Acquire) {
            return Err(expired());
        }
        let token = self.0.operation.as_ref().ok_or_else(expired)?;
        if interpreter::current(py)? != token.interpreter || !operation::is_active(token) {
            return Err(expired());
        }
        Ok(())
    }

    pub(super) fn read(&self, py: Python<'_>, request: Request) -> PyResult<OwnedReply> {
        self.check(py)?;
        let entry = ENTRIES.with(|entries| {
            entries
                .borrow()
                .iter()
                .find(|entry| Arc::ptr_eq(&entry.key.0, &self.0))
                .cloned()
        });
        // No TLS borrow survives this lookup or any native getter/helper call.
        let entry = entry.ok_or_else(expired)?;
        if entry.kind != self.0.kind {
            return Err(expired());
        }
        // SAFETY, for each of the only three lifetime-erasure edges:
        // - only the matching with_* function can create this private tag/pointer;
        // - pointer addresses the original immovably borrowed callback argument,
        //   not a temporary child view or a cloned context;
        // - the !Send guard unregisters/deactivates before that borrow can end,
        //   on both return and unwind;
        // - check above verifies thread, interpreter, operation identity and
        //   liveness, and lookup matches the unique retained Arc identity;
        // - native dispatch is synchronous, does not call Python, and returns
        //   OwnedReply with no borrowed fields. Python allocation happens later.
        let reply = unsafe {
            match entry.kind {
                Kind::Value => context::read_value(
                    &*entry.pointer.cast::<core::NumericValueContext<'_>>(),
                    request,
                ),
                Kind::Set => context::read_set(
                    &*entry.pointer.cast::<core::NumericSetContext<'_>>(),
                    request,
                ),
                Kind::Tick => {
                    context::read_tick(&*entry.pointer.cast::<core::TickContext<'_>>(), request)
                }
            }
        };
        reply.map_err(crate::native_error)
    }
}

fn expired() -> PyErr {
    PyRuntimeError::new_err("callback context is no longer active in its originating operation")
}

#[derive(Clone)]
struct RawEntry {
    key: ContextKey,
    pointer: *const (),
    kind: Kind,
}

thread_local! {
    static ENTRIES: RefCell<Vec<RawEntry>> = const { RefCell::new(Vec::new()) };
}

struct ScopeGuard {
    key: ContextKey,
    registered: bool,
    _not_send: PhantomData<Rc<()>>,
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        self.key.0.active.store(false, Ordering::Release);
        if self.registered {
            ENTRIES.with(|entries| {
                let mut entries = entries.borrow_mut();
                if let Some(index) = entries
                    .iter()
                    .position(|entry| Arc::ptr_eq(&entry.key.0, &self.key.0))
                {
                    entries.remove(index);
                }
            });
        }
    }
}

fn enter(pointer: *const (), kind: Kind) -> ScopeGuard {
    // Failure remains a key that check() rejects before constructing/calling a
    // Python context. This keeps the safe with_* API generic in its result R.
    ENTRIES.with(|entries| {
        let mut entries = entries.borrow_mut();
        let allocation_failed = entries.try_reserve(1).is_err();
        let key = ContextKey(Arc::new(KeyState {
            thread: thread::current().id(),
            operation: operation::current_token(),
            kind,
            active: AtomicBool::new(!allocation_failed),
            allocation_failed,
        }));
        if !allocation_failed {
            entries.push(RawEntry {
                key: key.clone(),
                pointer,
                kind,
            });
        }
        ScopeGuard {
            key,
            registered: !allocation_failed,
            _not_send: PhantomData,
        }
    })
}

pub(crate) fn with_value_context<R>(
    context: &core::NumericValueContext<'_>,
    call: impl FnOnce(ContextKey) -> R,
) -> R {
    let guard = enter(std::ptr::from_ref(context).cast(), Kind::Value);
    call(guard.key.clone())
}

pub(crate) fn with_set_context<R>(
    context: &core::NumericSetContext<'_>,
    call: impl FnOnce(ContextKey) -> R,
) -> R {
    let guard = enter(std::ptr::from_ref(context).cast(), Kind::Set);
    call(guard.key.clone())
}

pub(crate) fn with_tick_context<R>(
    context: &core::TickContext<'_>,
    call: impl FnOnce(ContextKey) -> R,
) -> R {
    let guard = enter(std::ptr::from_ref(context).cast(), Kind::Tick);
    call(guard.key.clone())
}
