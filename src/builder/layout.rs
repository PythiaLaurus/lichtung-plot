use crate::Error;

use super::Plot;
use crate::layout::{CanvasLayout, CanvasLayoutSpec};
use crate::spec::PlotKind;
use crate::theme::ResolvedTheme;

impl<K: crate::kind::Marker> Plot<K> {
    /// Resolve the canvas layout for this chart's canvas and configured text.
    #[must_use]
    #[cfg(test)]
    pub(crate) fn canvas_layout(&self) -> CanvasLayout {
        let theme = self.theme.unwrap_or(crate::Theme::Publication).resolve();
        let mut plot = crate::kind::into_erased(self.clone());
        if crate::layout::interior::InteriorRoute::classify(plot.kind()).is_native() {
            plot = crate::plots::shared_x::prepare_plot(
                &plot,
                &crate::font::canonical::OperationGenerationResources::new(),
            )
            .expect("valid test plot")
            .attach(plot)
            .expect("matching test route");
        }
        plot.canvas_layout_erased(&theme)
    }
}

impl Plot {
    #[cfg(test)]
    fn canvas_layout_erased(&self, theme: &ResolvedTheme) -> CanvasLayout {
        let mut canvas = self
            .measured_canvas_layout(theme)
            .unwrap_or_else(|_| self.resolve_canvas_layout(theme));
        if self.has_facets() {
            return canvas;
        }
        let kind = self.kind;
        if kind == PlotKind::Matrix {
            return canvas;
        }
        let cell = canvas.panel_box;
        if let Ok(plan) = crate::decorations::plan::plan_panel_for_test(
            self,
            kind,
            cell,
            crate::decorations::plan::LegendRole::Panel,
            theme,
        ) {
            canvas.insets.left += plan.panel_box.x0 - cell.x0;
            canvas.insets.right += cell.x1 - plan.panel_box.x1;
            canvas.insets.bottom += plan.panel_box.y0 - cell.y0;
            canvas.insets.top += cell.y1 - plan.panel_box.y1;
            canvas.panel_box = plan.panel_box;
        }
        canvas
    }

    pub(crate) fn base_canvas_layout(&self, theme: &ResolvedTheme) -> CanvasLayout {
        self.resolve_canvas_layout(theme)
    }

    pub(crate) fn measured_canvas_layout(
        &self,
        theme: &ResolvedTheme,
    ) -> Result<CanvasLayout, Error> {
        if crate::layout::interior::InteriorRoute::classify(self.kind()).is_native()
            && !self.has_facets()
        {
            return crate::layout::interior::standard::canvas_layout(self, theme)
                .map_err(crate::layout::fit::PlanError::into_error);
        }
        let labels_present = self.axis_label_presence();
        let measure = |spec, text| {
            CanvasLayout::measured_plot_root(
                self.width,
                self.height,
                theme,
                spec,
                text,
                crate::decorations::title_block::plan::RootAlignments::resolve(
                    theme,
                    [self.title_alignment, self.subtitle_alignment],
                    [None, None],
                ),
                !self.has_facets(),
            )
            .map_err(crate::layout::fit::PlanError::into_error)
        };
        let canvas = measure(
            CanvasLayoutSpec {
                has_title: self.title.as_ref().is_some_and(|value| !value.is_empty()),
                has_subtitle: self
                    .subtitle
                    .as_ref()
                    .is_some_and(|value| !value.is_empty()),
                has_caption: self.caption.as_ref().is_some_and(|value| !value.is_empty()),
                has_x_label: labels_present.0,
                has_y_label: labels_present.1,
            },
            (
                self.title.as_deref(),
                self.subtitle.as_deref(),
                self.caption.as_deref(),
            ),
        )?;
        Ok(canvas)
    }

    fn axis_label_presence(&self) -> (bool, bool) {
        if self.kind == PlotKind::Matrix {
            return (true, true);
        }
        let (x, y) = self.resolved_axis_labels(self.kind);
        (!x.is_empty(), !y.is_empty())
    }

    fn resolve_canvas_layout(&self, theme: &ResolvedTheme) -> CanvasLayout {
        let labels_present = self.axis_label_presence();
        CanvasLayout::single_panel(
            self.width,
            self.height,
            theme,
            CanvasLayoutSpec {
                has_title: self.title.as_ref().is_some_and(|value| !value.is_empty()),
                has_subtitle: self
                    .subtitle
                    .as_ref()
                    .is_some_and(|value| !value.is_empty()),
                has_caption: self.caption.as_ref().is_some_and(|value| !value.is_empty()),
                has_x_label: labels_present.0,
                has_y_label: labels_present.1,
            },
        )
    }
}
