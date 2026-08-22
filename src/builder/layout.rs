use crate::Error;

use super::Plot;
use crate::layout::{FigureLayout, FigureLayoutSpec, Layout, PanelBox, TextBlock};
use crate::spec::PlotKind;
use crate::theme::ThemePalette;

impl Plot {
    /// Borrow the active theme palette.
    #[must_use]
    pub fn palette(&self) -> ThemePalette {
        self.theme.palette()
    }

    /// Resolve the figure layout for this chart's canvas and configured text.
    #[must_use]
    pub fn figure_layout(&self) -> FigureLayout {
        let mut figure = self
            .measured_figure_layout()
            .unwrap_or_else(|_| self.resolve_figure_layout());
        let Some(kind) = self.kind.filter(|_| !self.has_facets()) else {
            return figure;
        };
        if kind == PlotKind::Matrix {
            return figure;
        }
        let cell = figure.panel_box;
        if let Ok(plan) = crate::decorations::plan::plan_panel(
            self,
            kind,
            cell,
            crate::decorations::plan::LegendRole::Panel,
        ) {
            figure.insets.left += plan.panel.x0 - cell.x0;
            figure.insets.right += cell.x1 - plan.panel.x1;
            figure.insets.bottom += plan.panel.y0 - cell.y0;
            figure.insets.top += cell.y1 - plan.panel.y1;
            figure.panel_box = plan.panel;
        }
        figure
    }

    pub(crate) fn base_figure_layout(&self) -> FigureLayout {
        self.resolve_figure_layout()
    }

    pub(crate) fn measured_figure_layout(&self) -> Result<FigureLayout, Error> {
        let palette = self.palette();
        let labels_present = self.axis_label_presence();
        let mut figure = FigureLayout::measured_single_panel(
            self.width,
            self.height,
            &palette,
            FigureLayoutSpec {
                has_title: palette.show_title && self.title.as_ref().is_some_and(|s| !s.is_empty()),
                has_subtitle: palette.show_subtitle
                    && self.subtitle.as_ref().is_some_and(|s| !s.is_empty()),
                has_caption: palette.show_caption
                    && self.caption.as_ref().is_some_and(|s| !s.is_empty()),
                has_x_label: palette.show_x_label && labels_present.0,
                has_y_label: palette.show_y_label && labels_present.1,
            },
            (
                self.title.as_deref(),
                self.subtitle.as_deref(),
                self.caption.as_deref(),
            ),
        )?;
        self.reserve_multiline_axis_titles(&mut figure, true)?;
        Ok(figure)
    }

    fn axis_label_presence(&self) -> (bool, bool) {
        self.kind.map_or((false, false), |kind| {
            let (x, y) = self.resolved_axis_labels(kind);
            (!x.is_empty(), !y.is_empty())
        })
    }

    fn resolve_figure_layout(&self) -> FigureLayout {
        let palette = self.palette();
        let labels_present = self.axis_label_presence();
        let mut figure = FigureLayout::single_panel(
            self.width,
            self.height,
            &palette,
            FigureLayoutSpec {
                has_title: palette.show_title && self.title.as_ref().is_some_and(|s| !s.is_empty()),
                has_subtitle: palette.show_subtitle
                    && self.subtitle.as_ref().is_some_and(|s| !s.is_empty()),
                has_caption: palette.show_caption
                    && self.caption.as_ref().is_some_and(|s| !s.is_empty()),
                has_x_label: palette.show_x_label && labels_present.0,
                has_y_label: palette.show_y_label && labels_present.1,
            },
        );
        let _ = self.reserve_multiline_axis_titles(&mut figure, false);
        figure
    }

    fn reserve_multiline_axis_titles(
        &self,
        figure: &mut FigureLayout,
        checked: bool,
    ) -> Result<(), Error> {
        let Some(kind) = self.kind else {
            return Ok(());
        };
        let baseline = self.axis_title_line_counts(kind, figure.panel_box)?;
        let baseline = (baseline.0.min(1), baseline.1.min(1));
        let (panel, extra_left, extra_bottom) =
            self.adjusted_axis_title_panel(kind, baseline, figure.panel_box, checked)?;
        figure.panel_box = panel;
        figure.insets.left += extra_left;
        figure.insets.bottom += extra_bottom;
        Ok(())
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "tick counts are bounded by finite canvas dimensions"
    )]
    pub(crate) fn reserve_rewrapped_axis_titles(
        &self,
        kind: PlotKind,
        reserved_panel: PanelBox,
        mut candidate_panel: PanelBox,
        discovered_layout: &Layout,
    ) -> Result<PanelBox, Error> {
        let baseline = self.axis_title_line_counts(kind, reserved_panel)?;
        if !self.axis_text_measured
            && discovered_layout.projection.panel_box().y0 <= reserved_panel.y0 + f64::EPSILON
        {
            let palette = self.palette();
            let tick_width =
                candidate_panel.width() / discovered_layout.x_axis.ticks.len().max(1) as f64 * 0.9;
            let maximum_height = discovered_layout
                .x_axis
                .ticks
                .iter()
                .filter(|tick| !tick.label.is_empty())
                .map(|tick| {
                    TextBlock::layout(
                        &tick.label,
                        palette.tick_font_size,
                        tick_width.max(1.0),
                        "x-axis tick label",
                    )
                    .map(|block| block.height())
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .fold(0.0_f64, f64::max);
            let extra = (maximum_height - f64::from(palette.tick_font_size) * 1.2).max(0.0);
            if candidate_panel.height() - extra < 60.0 {
                return Err(Error::Render(
                    "multiline x-axis tick labels do not fit while preserving the minimum 60px plot panel; increase chart height or reduce the tick count"
                        .into(),
                ));
            }
            candidate_panel.y0 += extra;
        }
        self.adjusted_axis_title_panel(kind, baseline, candidate_panel, true)
            .map(|(panel, _, _)| panel)
    }

    fn axis_title_line_counts(
        &self,
        kind: PlotKind,
        panel: PanelBox,
    ) -> Result<(usize, usize), Error> {
        let palette = self.palette();
        let (x_label, y_label) = self.resolved_axis_labels(kind);
        let x_lines = if palette.show_x_label && !x_label.is_empty() {
            TextBlock::layout(
                &x_label,
                palette.label_font_size,
                panel.width().max(1.0),
                "x-axis title",
            )?
            .lines()
            .len()
        } else {
            0
        };
        let y_lines = if palette.show_y_label && !y_label.is_empty() {
            TextBlock::layout(
                &y_label,
                palette.label_font_size,
                panel.height().max(1.0),
                "y-axis title",
            )?
            .lines()
            .len()
        } else {
            0
        };
        Ok((x_lines, y_lines))
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "wrapped line counts are bounded by finite u32 canvas dimensions"
    )]
    fn adjusted_axis_title_panel(
        &self,
        kind: PlotKind,
        baseline: (usize, usize),
        candidate: PanelBox,
        checked: bool,
    ) -> Result<(PanelBox, f64, f64), Error> {
        let line_height = f64::from(self.palette().label_font_size) * 1.2;
        let mut panel = candidate;
        for _ in 0..4 {
            let lines = self.axis_title_line_counts(kind, panel)?;
            let extra_bottom = lines.0.saturating_sub(baseline.0) as f64 * line_height;
            let extra_left = lines.1.saturating_sub(baseline.1) as f64 * line_height;
            if checked
                && (candidate.width() - extra_left < 80.0
                    || candidate.height() - extra_bottom < 60.0)
            {
                return Err(Error::Render(
                    "multiline axis titles do not fit while preserving the minimum 80x60 plot panel; increase chart size"
                        .into(),
                ));
            }
            let next = PanelBox::new(
                (candidate.x0 + extra_left).min(candidate.x1 - 80.0),
                candidate.x1,
                (candidate.y0 + extra_bottom).min(candidate.y1 - 60.0),
                candidate.y1,
            );
            if next == panel {
                return Ok((next, extra_left, extra_bottom));
            }
            panel = next;
        }
        Err(Error::Render(
            "multiline axis-title layout did not stabilize; increase chart size or shorten labels"
                .into(),
        ))
    }
}
