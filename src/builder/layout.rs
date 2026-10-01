use crate::Error;

use super::Plot;
use crate::layout::{LegacyRootAxes, RootFramePlan};
use crate::spec::PlotKind;
use crate::theme::ResolvedTheme;

impl<K: crate::kind::Marker> Plot<K> {
    /// Resolve the canvas layout for this chart's canvas and configured text.
    #[must_use]
    #[cfg(test)]
    pub(crate) fn canvas_layout(&self) -> RootFramePlan {
        let theme = self.theme.unwrap_or(crate::Theme::Publication).resolve();
        let plot = crate::kind::into_erased(self.clone());
        let plot = crate::plots::shared_x::prepare_plot(
            &plot,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
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

    pub(crate) fn legacy_root_axes(&self) -> LegacyRootAxes {
        let labels_present = self.axis_label_presence();
        LegacyRootAxes {
            has_x_label: labels_present.0,
            has_y_label: labels_present.1,
        }
    }

    pub(crate) fn measured_canvas_layout(
        &self,
        theme: &ResolvedTheme,
    ) -> Result<RootFramePlan, Error> {
        if !self.has_facets() {
            return crate::layout::interior::coordinate::canvas_layout(self, theme)
                .map_err(crate::layout::fit::PlanError::into_error);
        }
        RootFramePlan::measured_plot_root(
            (self.width, self.height),
            theme,
            self.legacy_root_axes(),
            (
                self.title.as_deref(),
                self.subtitle.as_deref(),
                self.caption.as_deref(),
            ),
            crate::decorations::title_block::plan::RootAlignments::resolve(
                theme,
                [self.title_alignment, self.subtitle_alignment],
                [None, None],
            ),
            !self.has_facets(),
            if self.has_facets() {
                (
                    crate::layout::canvas::MINIMUM_PANEL_SPAN,
                    crate::layout::canvas::MINIMUM_PANEL_SPAN,
                )
            } else {
                self.intrinsic_body_minimum(theme)?
            },
        )
        .map_err(crate::layout::fit::PlanError::into_error)
    }

    fn axis_label_presence(&self) -> (bool, bool) {
        if self.kind == PlotKind::Matrix {
            return (true, true);
        }
        let (x, y) = self.resolved_axis_labels(self.kind);
        (!x.is_empty(), !y.is_empty())
    }
}
