//! Per-call dimensions, distinct from the root's completed output size.

use crate::layout::RootSize;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PlotMeasureDimensions {
    pub(crate) canvas: RootSize,
    pub(crate) output_override: Option<RootSize>,
}

impl PlotMeasureDimensions {
    pub(crate) const fn canvas_size(self) -> RootSize {
        self.canvas
    }

    pub(crate) fn figure_size(self) -> RootSize {
        self.output_override.unwrap_or(self.canvas_size())
    }
}
