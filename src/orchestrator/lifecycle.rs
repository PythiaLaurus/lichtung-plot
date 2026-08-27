//! One-canvas chart lifecycle and unfaceted scene coordination.

use crate::Error;

use super::backend::{validate_with_null_backend, DeferredBackend};
use crate::builder::Plot;
use crate::render::backend::Backend;
use crate::spec::{DecorationMode, PlotKind, PlotRenderContext, PlotRenderResult};

impl Plot {
    pub(super) fn render_kind_erased(
        &self,
        kind: PlotKind,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        self.validate_configuration(kind)?;
        let mut backend = DeferredBackend::new(backend, self.width(), self.height());
        let rendered = if self.has_facets() {
            self.render_facets(kind, &mut backend)
        } else {
            self.render_open_canvas(kind, &mut backend)
        };
        backend.finish(rendered, "backend canvas")
    }

    fn render_open_canvas(&self, kind: PlotKind, backend: &mut dyn Backend) -> Result<(), Error> {
        if !self.significance_annotations().is_empty() && !kind.supports_significance_annotations()
        {
            return Err(Error::Data(format!(
                "significance annotations are not supported for {kind:?}; use a supported categorical family or remove the annotations"
            )));
        }
        let palette = self.palette();
        let cell = match kind {
            PlotKind::Matrix => crate::plots::matrix::panel_box(self)?,
            _ => self.measured_canvas_layout()?.panel_box,
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
        let context = source_rows.map_or_else(
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
                crate::decorations::apply::apply_canvas_text(self, &palette, backend)?;
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
        self.validate_category_order_configuration(kind)?;
        if !self.significance_annotations().is_empty() && !kind.supports_significance_annotations()
        {
            return Err(Error::Data(format!(
                "significance annotations are not supported for {kind:?}; use a supported categorical family or remove the annotations"
            )));
        }
        let source_rows = self.configured_source_rows()?;
        let context = source_rows.map_or_else(
            || PlotRenderContext::new(self.data(), panel),
            |rows| PlotRenderContext::with_source_rows(self.data(), rows, panel),
        );
        validate_with_null_backend(self.width(), self.height(), "panel preflight", |backend| {
            self.render_panel(kind, context, backend)
        })
    }

    pub(crate) fn preflight_hue_levels(
        &self,
        kind: PlotKind,
        panel: crate::layout::PanelBox,
    ) -> Result<Vec<String>, Error> {
        let result = self.preflight_panel(kind, panel)?;
        let Some(field) = self.hue_dimension_key() else {
            return Ok(Vec::new());
        };
        Ok(result
            .legend_entries
            .iter()
            .fold(Vec::new(), |mut levels, entry| {
                if let Some(value) = entry.coordinated_hue_value_for(field) {
                    if !levels.iter().any(|level| level == value) {
                        levels.push(value.to_owned());
                    }
                }
                levels
            }))
    }

    pub(crate) fn configured_source_rows(&self) -> Result<Option<&[usize]>, Error> {
        if let Some(rows) = self.source_row_map() {
            if rows.len() != self.data().height() {
                return Err(Error::Data(format!(
                    "chart source-row map has length {}, but chart data has {} rows",
                    rows.len(),
                    self.data().height()
                )));
            }
        }
        Ok(self.source_row_map())
    }
}
