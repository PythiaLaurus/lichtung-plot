//! Nested renders own separate scoped callback failures and real Python pins.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread::{self, ThreadId};

use lichtung_plot as core;
use pyo3::exceptions::PyMemoryError;
use pyo3::prelude::*;

use super::interpreter::{self, InterpreterId};
use super::CallbackRoots;

pub(super) struct OperationKey;

#[derive(Clone)]
pub(super) struct OperationToken {
    pub(super) key: Arc<OperationKey>,
    pub(super) interpreter: InterpreterId,
    pub(super) thread: ThreadId,
}

#[derive(Debug)]
pub(super) enum CallbackKind {
    NumericValue,
    NumericSet,
    Tick,
}

pub(super) struct CallbackSite(String);

impl CallbackSite {
    pub(super) fn numeric(context: &core::NumericSetContext<'_>) -> Self {
        let owner = context.owner();
        Self(format!(
            "{:?} {:?}, plot {:?}, Canvas {:?}",
            context.role(),
            context.purpose(),
            owner.plot_kind(),
            owner.canvas_cell()
        ))
    }

    pub(super) fn tick(context: &core::TickContext<'_>) -> Self {
        let owner = context.owner();
        Self(format!(
            "{:?} positions, plot {:?}, Canvas {:?}",
            context.axis(),
            owner.plot_kind(),
            owner.canvas_cell()
        ))
    }
}

struct CallbackFailure {
    kind: CallbackKind,
    site: CallbackSite,
    error: PyErr,
}

impl CallbackFailure {
    fn into_error(self) -> PyErr {
        // Context is owned diagnostic data, not a borrowed renderer view. Do not
        // mutate the application's cause/context/traceback while restoring it.
        drop((self.kind, self.site.0));
        self.error
    }
}

pub(super) struct OperationFrame {
    token: OperationToken,
    failure: Mutex<Option<CallbackFailure>>,
}

impl OperationFrame {
    pub(super) const fn interpreter(&self) -> InterpreterId {
        self.token.interpreter
    }

    pub(super) fn check_callback(&self, interpreter: InterpreterId) -> core::Result<()> {
        if thread::current().id() != self.token.thread || interpreter != self.token.interpreter {
            return Err(core::Error::Render(
                "Python callback left its originating operation thread or interpreter".into(),
            ));
        }
        Ok(())
    }

    pub(super) fn record_failure(&self, kind: CallbackKind, site: CallbackSite, error: PyErr) {
        let mut incoming = Some(CallbackFailure { kind, site, error });
        {
            let mut failure = self
                .failure
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if failure.is_none() {
                *failure = incoming.take();
            }
        }
        // In the defensive multiple-failure case, DECREF outside the mutex.
        drop(incoming);
    }

    fn take_failure(&self) -> Option<CallbackFailure> {
        self.failure
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
}

thread_local! {
    static FRAMES: RefCell<Vec<Arc<OperationFrame>>> = const { RefCell::new(Vec::new()) };
}

pub(super) fn current_frame() -> core::Result<Arc<OperationFrame>> {
    let frame = FRAMES.with(|frames| frames.borrow().last().cloned());
    frame.ok_or_else(|| {
        core::Error::Render("Python callback requires an active render operation".into())
    })
}

pub(super) fn current_token() -> Option<OperationToken> {
    FRAMES.with(|frames| frames.borrow().last().map(|frame| frame.token.clone()))
}

pub(super) fn is_active(token: &OperationToken) -> bool {
    if thread::current().id() != token.thread {
        return false;
    }
    FRAMES.with(|frames| {
        frames
            .borrow()
            .iter()
            .any(|frame| Arc::ptr_eq(&frame.token.key, &token.key))
    })
}

struct OperationGuard {
    key: Arc<OperationKey>,
    // detach stays on this thread; this guard cannot be moved to a worker.
    _not_send: PhantomData<Rc<()>>,
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        let removed = FRAMES.with(|frames| {
            let mut frames = frames.borrow_mut();
            frames
                .iter()
                .position(|frame| Arc::ptr_eq(&frame.token.key, &self.key))
                .map(|index| frames.remove(index))
        });
        // A frame may own a PyErr. Never destroy it under the TLS borrow.
        drop(removed);
    }
}

/// Caller has already fixed its native version and released every PyO3 borrow.
/// Pins must have been acquired with actual INCREFs from that same version.
pub(crate) fn with_operation<T: Send>(
    py: Python<'_>,
    pins: CallbackRoots,
    render: impl FnOnce() -> core::Result<T> + Send,
) -> PyResult<T> {
    super::ensure_live(py, &pins)?;
    let token = OperationToken {
        key: Arc::new(OperationKey),
        interpreter: interpreter::current(py)?,
        thread: thread::current().id(),
    };
    let frame = Arc::new(OperationFrame {
        token,
        failure: Mutex::new(None),
    });
    FRAMES.with(|frames| {
        let mut frames = frames.borrow_mut();
        frames
            .try_reserve(1)
            .map_err(|error| PyMemoryError::new_err(error.to_string()))?;
        frames.push(Arc::clone(&frame));
        Ok::<_, PyErr>(())
    })?;
    let guard = OperationGuard {
        key: Arc::clone(&frame.token.key),
        _not_send: PhantomData,
    };

    // PyO3 reattaches before returning or unwinding. The frame guard was created
    // after frame, and pins are an argument: unwind also unregisters first.
    let result = py.detach(render);
    let failure = frame.take_failure();
    drop(guard);
    drop(frame);
    // DECREF may execute Python. The original interpreter is attached and there
    // are no TLS/mutex/PyO3 borrows; the old operation has already been removed.
    drop(pins);
    match failure {
        Some(failure) => Err(failure.into_error()),
        None => result.map_err(crate::native_error),
    }
}
