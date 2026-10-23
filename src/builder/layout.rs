use crate::Error;

use super::Plot;
use crate::layout::RootFramePlan;
#[cfg(test)]
use crate::spec::PlotKind;
use crate::theme::ResolvedTheme;

impl<K: crate::kind::Marker> Plot<K> {
    /// Resolve the canvas layout for this chart's canvas and configured text.
    #[must_use]
    #[cfg(test)]
    pub(crate) fn canvas_layout(&self) -> RootFramePlan {
        let resources = crate::font::canonical::OperationContext::new();
        let theme = self.resolve_theme().unwrap();
        let plot = crate::kind::into_erased(self.clone());
        let plot = crate::plots::shared_x::prepare_plot(&plot, &resources, None)
            .expect("valid test plot")
            .attach(plot)
            .expect("matching test route");
        plot.canvas_layout_erased(&theme)
    }
}

impl Plot {
    #[cfg(test)]
    fn canvas_layout_erased(&self, theme: &ResolvedTheme) -> RootFramePlan {
        let mut canvas = self
            .measured_canvas_layout(theme)
            .expect("test root must fit without fallback");
        if self.has_facets() {
            return canvas;
        }
        let kind = self.kind;
        if kind == PlotKind::Matrix {
            return canvas;
        }
        let cell = canvas.panel_box;
        let plan = crate::decorations::plan::plan_panel_for_test(
            self,
            kind,
            cell,
            crate::decorations::plan::LegendRole::Panel,
            theme,
        )
        .expect("test panel must fit without fallback");
        canvas.insets.left += plan.panel_box.x0 - cell.x0;
        canvas.insets.right += cell.x1 - plan.panel_box.x1;
        canvas.insets.bottom += plan.panel_box.y0 - cell.y0;
        canvas.insets.top += cell.y1 - plan.panel_box.y1;
        canvas.panel_box = plan.panel_box;
        canvas
    }

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
