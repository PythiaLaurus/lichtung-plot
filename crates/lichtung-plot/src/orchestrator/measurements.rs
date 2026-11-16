//! Thread-local capture/Null call observations for canonical failure cleanup.

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub(crate) struct Observations {
    // Preserve the capture/Null stage indices used by the retained fixture.
    pub calls: [usize; 4],
    session: Rc<()>,
}

thread_local! {
    static ACTIVE: RefCell<Option<Observations>> = const { RefCell::new(None) };
}

pub(crate) fn start() {
    ACTIVE.with(|active| *active.borrow_mut() = Some(Observations::default()));
}

pub(crate) fn finish() -> Observations {
    ACTIVE.with(|active| active.borrow_mut().take().unwrap())
}

pub(crate) struct StageCall(usize, Option<Rc<()>>);

impl StageCall {
    pub(crate) fn new(stage: usize) -> Self {
        Self(
            stage,
            ACTIVE.with(|active| {
                active
                    .borrow()
                    .as_ref()
                    .map(|observation| Rc::clone(&observation.session))
            }),
        )
    }
}

impl Drop for StageCall {
    fn drop(&mut self) {
        if let Some(session) = &self.1 {
            ACTIVE.with(|active| {
                if let Some(observation) = active
                    .borrow_mut()
                    .as_mut()
                    .filter(|observation| Rc::ptr_eq(&observation.session, session))
                {
                    observation.calls[self.0] += 1;
                }
            });
        }
    }
}
