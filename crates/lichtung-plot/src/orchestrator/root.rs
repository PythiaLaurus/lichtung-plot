//! Operation-local layout caches, borrowed together for one measurement round.

use crate::decorations::plan::children::ChildPlans;

pub(crate) struct RootLayoutCaches {
    pub(crate) children: ChildPlans,
    pub(crate) heights: crate::canvas::HeightMeasurements,
    pub(crate) frames: Option<CachedRootFrame>,
}

pub(crate) struct CachedRootFrame {
    pub(crate) prose: [crate::Text; 3],
    pub(crate) dimensions: (u32, u32),
    pub(crate) minimum: [u64; 2],
    pub(crate) frame: std::sync::Arc<crate::layout::RootFramePlan>,
}

impl RootLayoutCaches {
    pub(crate) fn new(children: ChildPlans) -> Self {
        Self {
            children,
            heights: crate::canvas::HeightMeasurements::default(),
            frames: None,
        }
    }
}
