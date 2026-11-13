use crate::Error;

use super::Plot;
use crate::layout::RootFramePlan;

impl Plot {
    pub(crate) fn measured_canvas_layout(
        &self,
        registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
        intrinsic: &crate::orchestrator::root::PlotIntrinsics,
        operation: &crate::font::canonical::OperationContext,
        theme: &crate::text::TextRequestContext<'_, '_>,
    ) -> Result<RootFramePlan, Error> {
        if !self.description().has_facets() {
            return crate::layout::interior::coordinate::canvas_layout(
                self, registered, intrinsic, operation, theme,
            )
            .map_err(crate::layout::fit::PlanError::into_error);
        }
        RootFramePlan::measure(
            self.canvas_dimensions()?,
            theme,
            self.description().prose(),
            crate::decorations::title_block::plan::RootAlignments::resolve(
                theme,
                self.description().prose_alignments(),
                [None, None],
            ),
            (
                crate::layout::canvas::MINIMUM_PANEL_SPAN,
                crate::layout::canvas::MINIMUM_PANEL_SPAN,
            ),
        )
        .map_err(crate::layout::fit::PlanError::into_error)
    }
}
