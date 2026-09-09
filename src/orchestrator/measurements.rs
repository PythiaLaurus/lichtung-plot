//! Opt-in, thread-local observations for the ignored release baseline.
use std::cell::RefCell;
use std::time::{Duration, Instant};

pub(crate) const STAGES: [&str; 8] = [
    "prepare_including_fonts",
    "root_measure",
    "capture",
    "null_validation",
    "publication",
    "svg_outline",
    "png_raster_encode",
    "svg_replay_serialization",
];

#[derive(Clone, Default)]
pub(crate) struct Observations {
    pub times: [Duration; 8],
    pub calls: [usize; 8],
    pub producers: usize,
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

pub(crate) fn producer() {
    ACTIVE.with(|active| {
        if let Some(observation) = active.borrow_mut().as_mut() {
            observation.producers += 1;
        }
    });
}

pub(crate) struct Timer(usize, Option<Instant>);

impl Timer {
    pub(crate) fn new(stage: usize) -> Self {
        Self(
            stage,
            ACTIVE.with(|active| active.borrow().as_ref().map(|_| Instant::now())),
        )
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if let Some(start) = self.1 {
            ACTIVE.with(|active| {
                if let Some(observation) = active.borrow_mut().as_mut() {
                    observation.times[self.0] += start.elapsed();
                    observation.calls[self.0] += 1;
                }
            });
        }
    }
}
