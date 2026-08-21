//! One-canvas chart lifecycle and unfaceted scene coordination.

use crate::Error;

use super::backend::{DeferredBackend, NullBackend};
use crate::builder::Chart;
use crate::render::backend::Backend;
use crate::spec::{
    DecorationMode, HeatmapLabelFit, HeatmapLayout, PlotKind, PlotRenderContext, PlotRenderResult,
};

impl Chart<'_> {
    pub(super) fn render_kind(
        &self,
        kind: PlotKind,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        self.x_axis_spec().validate("x-axis")?;
        self.y_axis_spec().validate("y-axis")?;
        if kind != PlotKind::Heatmap && self.heatmap_layout_spec() != HeatmapLayout::Cartesian {
            return Err(Error::Data(format!(
                "{kind:?}: non-Cartesian HeatmapLayout is supported only by PlotKind::Heatmap"
            )));
        }
        if kind != PlotKind::Heatmap && self.heatmap_label_fit_spec() != HeatmapLabelFit::Elide {
            return Err(Error::Data(format!(
                "{kind:?}: HeatmapLabelFit::Wrap is supported only by PlotKind::Heatmap"
            )));
        }
        if kind != PlotKind::Heatmap && !self.heatmap_null_row_annotations_spec().is_empty() {
            return Err(Error::Data(format!(
                "{kind:?}: fully-null heatmap row annotations are supported only by PlotKind::Heatmap"
            )));
        }
        self.validate_channel_configuration(kind)?;
        self.validate_interval_configuration(kind)?;
        self.validate_facet_configuration(kind)?;
        self.validate_supplemental_configuration(kind)?;
        let mut backend = DeferredBackend::new(backend, self.width(), self.height());
        let rendered = if self.has_facets() {
            self.render_facets(kind, &mut backend)
        } else {
            self.render_open_canvas(kind, &mut backend)
        };
        backend.finish(rendered, "backend canvas")
    }

    fn render_open_canvas(&self, kind: PlotKind, backend: &mut dyn Backend) -> Result<(), Error> {
        if !self.comparisons().is_empty() && !kind.supports_significance() {
            return Err(Error::Render(format!(
                "significance comparisons are not supported for {kind:?}; use a supported categorical family or remove the comparisons"
            )));
        }
        let palette = self.palette();
        let cell = match kind {
            PlotKind::Matrix => crate::plots::matrix::panel_box(self)?,
            _ => self.measured_figure_layout()?.panel_box,
        };
        let plan = crate::decorations::plan::plan_panel(
            self,
            kind,
            cell,
            crate::decorations::plan::LegendRole::Panel,
        )?;
        if plan.result.decoration_mode == DecorationMode::Standard {
            crate::decorations::apply::apply_grid(self, &plan.result, &palette, backend)?;
        }
        let source_rows = self.configured_source_rows()?;
        let render_chart = plan.chart_for_render(self);
        let context = source_rows.as_deref().map_or_else(
            || PlotRenderContext::new(render_chart.data(), plan.panel),
            |rows| PlotRenderContext::with_source_rows(render_chart.data(), rows, plan.panel),
        );
        let rendered = render_chart.render_panel(kind, context, backend)?;

        if !plan.result.significance_brackets.is_empty() {
            crate::significance::render(self, &plan.result.significance_brackets, backend)?;
        }

        match plan.result.decoration_mode {
            DecorationMode::Standard => {
                crate::decorations::apply::apply_axes(self, &plan.result, &palette, backend)?;
                crate::decorations::apply::apply_planned_legend(
                    self,
                    plan.legend.as_ref(),
                    backend,
                )?;
                crate::decorations::apply::apply_figure_text(self, &palette, backend)?;
            }
            DecorationMode::SelfManaged => {}
        }
        debug_assert_eq!(rendered.decoration_mode, plan.result.decoration_mode);
        Ok(())
    }

    pub(crate) fn preflight_panel(
        &self,
        kind: PlotKind,
        panel: crate::layout::PanelBox,
    ) -> Result<PlotRenderResult, Error> {
        if !self.comparisons().is_empty() && !kind.supports_significance() {
            return Err(Error::Render(format!(
                "significance comparisons are not supported for {kind:?}; use a supported categorical family or remove the comparisons"
            )));
        }
        let source_rows = self.configured_source_rows()?;
        let context = source_rows.as_deref().map_or_else(
            || PlotRenderContext::new(self.data(), panel),
            |rows| PlotRenderContext::with_source_rows(self.data(), rows, panel),
        );
        self.render_panel(kind, context, &mut NullBackend)
    }

    pub(crate) fn preflight_hue_levels(
        &self,
        kind: PlotKind,
        panel: crate::layout::PanelBox,
    ) -> Result<Vec<String>, Error> {
        let result = self.preflight_panel(kind, panel)?;
        if result.legend_keys.is_empty() {
            Ok(result
                .legend_entries
                .into_iter()
                .map(|entry| entry.label)
                .collect())
        } else {
            Ok(result.legend_keys)
        }
    }

    pub(crate) fn configured_source_rows(
        &self,
    ) -> Result<Option<std::borrow::Cow<'_, [usize]>>, Error> {
        if let Some(rows) = self.source_row_map() {
            if rows.len() != self.data().height() {
                return Err(Error::Data(format!(
                    "chart source-row map has length {}, but chart data has {} rows",
                    rows.len(),
                    self.data().height()
                )));
            }
        }
        if let Some(column) = self.raw_overlay_source_row_col() {
            return crate::data::strict_source_rows(self.data(), column, "raw overlay")
                .map(std::borrow::Cow::Owned)
                .map(Some);
        }
        Ok(self.source_row_map().map(std::borrow::Cow::Borrowed))
    }
}
