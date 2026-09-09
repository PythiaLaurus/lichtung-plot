use crate::Error;

use super::Plot;
use crate::layout::{
    CanvasLayout, CanvasLayoutSpec, Layout, PanelBox, PhysicalTextAlignment, TextBlock,
};
use crate::spec::PlotKind;
use crate::theme::ResolvedTheme;

impl<K: crate::kind::Marker> Plot<K> {
    /// Resolve the canvas layout for this chart's canvas and configured text.
    #[must_use]
    #[cfg(test)]
    pub(crate) fn canvas_layout(&self) -> CanvasLayout {
        let theme = self.theme.unwrap_or(crate::Theme::Publication).resolve();
        crate::kind::into_erased(self.clone()).canvas_layout_erased(&theme)
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
        let labels_present = self.axis_label_presence();
        let canvas = CanvasLayout::measured_single_panel(
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
            (
                self.title.as_deref(),
                self.subtitle.as_deref(),
                self.caption.as_deref(),
            ),
        )?;
        Ok(canvas)
    }

    fn axis_label_presence(&self) -> (bool, bool) {
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

    #[expect(
        clippy::cast_precision_loss,
        reason = "tick counts are bounded by finite canvas dimensions"
    )]
    pub(crate) fn measured_axis_title_gutters(
        &self,
        kind: PlotKind,
        candidate_panel: PanelBox,
        discovered_layout: &Layout,
        theme: &ResolvedTheme,
        scope: &crate::layout::fit::FitScope,
    ) -> Result<[f64; 4], crate::layout::fit::PlanError> {
        let x_scope = scope.child(crate::layout::AddressRole::AxisBottom, 0)?;
        let mut extra_bottom = 0.0;
        if !self.axis_text_measured
            && discovered_layout.projection.panel_box().y0 <= candidate_panel.y0 + f64::EPSILON
        {
            let tick_width =
                candidate_panel.width() / discovered_layout.x_axis.ticks.len().max(1) as f64;
            let maximum_height = discovered_layout
                .x_axis
                .ticks
                .iter()
                .filter(|tick| !tick.label.is_empty())
                .map(|tick| {
                    crate::layout::fit::measured(crate::layout::fit::text(
                        theme,
                        &tick.label,
                        &theme.typography.tick_label,
                        tick_width.max(1.0),
                        PhysicalTextAlignment::Center,
                        "x-axis tick label",
                        &x_scope,
                    ))
                    .map(TextBlock::from_resolved)
                    .map(|block| block.height())
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .fold(0.0_f64, f64::max);
            let extra = (maximum_height - theme.typography.tick_label.line_height()).max(0.0);
            if candidate_panel.height() - extra < 60.0 {
                return Err(x_scope.failure(crate::layout::fit::FitKind::TrackOrResidual,
                    "multiline x-axis tick labels do not fit while preserving the minimum 60px plot panel; increase chart height or shorten the tick labels"
                ).into());
            }
            extra_bottom = extra;
        }
        let lines = self.axis_title_line_counts(kind, candidate_panel, theme, scope)?;
        let line_height = theme.typography.axis_title.line_height();
        Ok([
            lines.1.saturating_sub(1) as f64 * line_height,
            0.0,
            extra_bottom + lines.0.saturating_sub(1) as f64 * line_height,
            0.0,
        ])
    }

    fn axis_title_line_counts(
        &self,
        kind: PlotKind,
        panel: PanelBox,
        theme: &ResolvedTheme,
        scope: &crate::layout::fit::FitScope,
    ) -> Result<(usize, usize), crate::layout::fit::PlanError> {
        let (x_label, y_label) = self.resolved_axis_labels(kind);
        let x_lines = if x_label.is_empty() {
            0
        } else {
            TextBlock::from_resolved(crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                &x_label,
                &theme.typography.axis_title,
                panel.width().max(1.0),
                PhysicalTextAlignment::Center,
                "x-axis title",
                &scope.child(crate::layout::AddressRole::AxisBottom, 0)?,
            ))?)
            .lines()
            .len()
        };
        let y_lines = if y_label.is_empty() {
            0
        } else {
            TextBlock::from_resolved(crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                &y_label,
                &theme.typography.axis_title,
                panel.height().max(1.0),
                PhysicalTextAlignment::Center,
                "y-axis title",
                &scope.child(crate::layout::AddressRole::AxisLeft, 0)?,
            ))?)
            .lines()
            .len()
        };
        Ok((x_lines, y_lines))
    }
}
