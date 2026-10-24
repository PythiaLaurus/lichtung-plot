use crate::Error;

use super::Plot;
use crate::layout::RootFramePlan;
use crate::theme::ResolvedTheme;

impl Plot {
    pub(crate) fn measured_canvas_layout(
        &self,
        theme: &ResolvedTheme,
    ) -> Result<RootFramePlan, Error> {
        if !self.has_facets() {
            return crate::layout::interior::coordinate::canvas_layout(self, theme)
                .map_err(crate::layout::fit::PlanError::into_error);
        }
        RootFramePlan::measure(
            self.canvas_dimensions()?,
            theme,
            self.prose(),
            crate::decorations::title_block::plan::RootAlignments::resolve(
                theme,
                self.prose_alignments(),
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
