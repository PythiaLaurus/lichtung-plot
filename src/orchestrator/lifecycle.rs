//! One-canvas chart lifecycle and immutable standalone scene planning.

use crate::Error;

use super::backend::{validate_with_null_backend, DeferredBackend};
use crate::builder::Plot;
use crate::decorations::plan::PanelPlan;
use crate::render::backend::Backend;
use crate::spec::{DecorationMode, PlotKind, PlotRenderContext, PlotRenderResult};
use crate::theme::ResolvedTheme;

pub(super) struct PlotPlan<'a> {
    source: &'a Plot,
    kind: PlotKind,
    theme: Box<ResolvedTheme>,
    root: crate::layout::SelectedRoot<PlotScenePlan>,
}

enum PlotScenePlan {
    Unfaceted(Box<UnfacetedPlotPlan>),
    Faceted(Box<super::facets::FacetPlan>),
}

struct UnfacetedPlotPlan {
    plot: Plot,
    panel_plan: PanelPlan,
}

impl<'a> PlotPlan<'a> {
    pub(super) fn new(source: &'a Plot, kind: PlotKind) -> Result<Self, Error> {
        #[cfg(test)]
        super::PLOT_PLAN_CALLS.with(|calls| calls.set(calls.get() + 1));
        let theme = Box::new(
            source
                .theme_spec()
                .unwrap_or(crate::Theme::Publication)
                .resolve(),
        );
        source.validate_configuration(kind)?;
        let root = crate::layout::plan_exact_root(source.width(), source.height(), kind, || {
            if source.has_facets() {
                Ok(PlotScenePlan::Faceted(Box::new(
                    super::facets::FacetPlan::new(source, kind, &theme)?,
                )))
            } else {
                Ok(PlotScenePlan::Unfaceted(Box::new(UnfacetedPlotPlan::new(
                    source, kind, &theme,
                )?)))
            }
        })?;
        let plan = Self {
            source,
            kind,
            theme,
            root,
        };
        let (width, height) = plan.dimensions();
        validate_with_null_backend(width, height, "prepared plot preflight", |backend| {
            plan.render_scene(backend)
        })?;
        Ok(plan)
    }

    pub(super) fn render(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        #[cfg(test)]
        super::FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(calls.get() + 1));
        let (width, height) = self.dimensions();
        let mut backend = DeferredBackend::new(backend, width, height);
        let rendered = self.render_scene(&mut backend);
        backend.finish(rendered, "backend canvas")
    }

    pub(super) fn dimensions(&self) -> (u32, u32) {
        self.root.dimensions()
    }

    fn render_scene(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        match self.root.plan() {
            PlotScenePlan::Unfaceted(plan) => {
                plan.render(self.source, self.kind, &self.theme, backend)
            }
            PlotScenePlan::Faceted(plan) => {
                plan.render(self.source, self.kind, &self.theme, backend)
            }
        }
    }
}

impl UnfacetedPlotPlan {
    fn new(source: &Plot, kind: PlotKind, theme: &ResolvedTheme) -> Result<Self, Error> {
        if !source.significance_annotations().is_empty()
            && !kind.supports_significance_annotations()
        {
            return Err(Error::Data(format!(
                "significance annotations are not supported for {kind:?}; use a supported categorical family or remove the annotations"
            )));
        }
        let cell = match kind {
            PlotKind::Matrix => crate::plots::matrix::panel_box(source, theme)?,
            _ => source.measured_canvas_layout(theme)?.panel_box,
        };
        let panel_plan = crate::decorations::plan::plan_panel(
            source,
            kind,
            cell,
            crate::decorations::plan::LegendRole::Panel,
            theme,
        )?;
        let plot = panel_plan.plot_with_planned_limits(source);
        Ok(Self { plot, panel_plan })
    }

    fn render(
        &self,
        source: &Plot,
        kind: PlotKind,
        theme: &ResolvedTheme,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        if self.panel_plan.result.decoration_mode == DecorationMode::Standard {
            crate::decorations::apply::apply_grid(source, &self.panel_plan.result, theme, backend)?;
        }
        let source_rows = self.plot.configured_source_rows()?;
        let context = source_rows.map_or_else(
            || PlotRenderContext::new(self.plot.data(), self.panel_plan.panel_box, theme),
            |rows| {
                PlotRenderContext::with_source_rows(
                    self.plot.data(),
                    rows,
                    self.panel_plan.panel_box,
                    theme,
                )
            },
        );
        let rendered = self.plot.render_panel(kind, context, backend)?;

        if !self.panel_plan.result.significance_brackets.is_empty() {
            crate::significance::render(
                source,
                &self.panel_plan.result.significance_brackets,
                theme,
                backend,
            )?;
        }

        match self.panel_plan.result.decoration_mode {
            DecorationMode::Standard => {
                crate::decorations::apply::apply_axes(
                    source,
                    &self.panel_plan.result,
                    theme,
                    backend,
                )?;
                crate::decorations::apply::apply_planned_legend(
                    source,
                    self.panel_plan.legend.as_ref(),
                    backend,
                )?;
                crate::decorations::apply::apply_canvas_text(source, theme, backend)?;
            }
            DecorationMode::SelfManaged => {}
        }
        debug_assert_eq!(
            rendered.decoration_mode,
            self.panel_plan.result.decoration_mode
        );
        Ok(())
    }
}

impl Plot {
    #[cfg(test)]
    pub(super) fn render_kind_erased(
        &self,
        kind: PlotKind,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        PlotPlan::new(self, kind)?.render(backend)
    }

    pub(crate) fn preflight_panel(
        &self,
        kind: PlotKind,
        panel: crate::layout::PanelBox,
        theme: &ResolvedTheme,
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
            || PlotRenderContext::new(self.data(), panel, theme),
            |rows| PlotRenderContext::with_source_rows(self.data(), rows, panel, theme),
        );
        validate_with_null_backend(self.width(), self.height(), "panel preflight", |backend| {
            self.render_panel(kind, context, backend)
        })
    }

    pub(crate) fn preflight_hue_levels(
        &self,
        kind: PlotKind,
        panel: crate::layout::PanelBox,
        theme: &ResolvedTheme,
    ) -> Result<Vec<String>, Error> {
        let result = self.preflight_panel(kind, panel, theme)?;
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
