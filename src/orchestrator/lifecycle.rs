//! One-canvas chart lifecycle and immutable standalone scene planning.

use crate::Error;

use super::backend::{
    capture_text_replay_plan, validate_text_replay_plan, validate_with_null_backend, TextReplayPlan,
};
use crate::builder::Plot;
use crate::decorations::plan::PanelPlan;
use crate::layout::PanelBox;
use crate::render::backend::{Backend, SurfaceRole};
use crate::spec::{DecorationMode, PlotKind, PlotRenderContext, PlotRenderResult};
use crate::theme::ResolvedTheme;

pub(super) struct PlotPlan {
    root: crate::layout::SelectedRoot<TextReplayPlan, PreparedPlotInputs>,
}

struct PreparedPlotInputs {
    theme: Box<ResolvedTheme>,
    text_census: crate::layout::PreparedTextCensus,
}

enum PlotLayoutPlan {
    Unfaceted(Box<UnfacetedPlotPlan>),
    Faceted(Box<super::facets::FacetPlan>),
}

struct UnfacetedPlotPlan {
    plot: Plot,
    panel_plan: PanelPlan,
    canvas_layout: crate::layout::CanvasLayout,
}

impl PlotPlan {
    pub(super) fn new(source: &Plot, kind: PlotKind) -> Result<Self, Error> {
        Self::new_with_theme_factory(source, kind, || {
            Box::new(
                source
                    .theme_spec()
                    .unwrap_or(crate::Theme::Publication)
                    .resolve(),
            )
        })
    }

    #[cfg(test)]
    pub(super) fn new_with_resolved_theme(
        source: &Plot,
        kind: PlotKind,
        theme: ResolvedTheme,
    ) -> Result<Self, Error> {
        Self::new_with_theme_factory(source, kind, || Box::new(theme))
    }

    fn new_with_theme_factory(
        source: &Plot,
        kind: PlotKind,
        resolve_theme: impl FnOnce() -> Box<ResolvedTheme>,
    ) -> Result<Self, Error> {
        #[cfg(test)]
        super::PLOT_PLAN_CALLS.with(|calls| calls.set(calls.get() + 1));
        let root = crate::layout::plan_exact_root(
            source.width(),
            source.height(),
            kind,
            |prepared_stage| {
                let theme = resolve_theme();
                source.validate_configuration(kind)?;
                if source.has_facets() {
                    super::facets::validate_before_font_preparation(source, kind)?;
                }
                let requirements = crate::layout::foundational_plot_requirements(
                    source,
                    kind,
                    theme.as_ref(),
                    &crate::layout::ComponentAddress::root(),
                    true,
                    true,
                    true,
                    None,
                )?;
                let text_topology =
                    crate::layout::PreparedTextCensus::structural_address_count(&requirements)
                        .map_err(|error| Error::Render(format!("prepared text census: {error}")))?;
                prepared_stage.claim_topology_slots(text_topology)?;
                let mut scalar_presence = prepared_stage.reserve_scalar_presence_map()?;
                let mut census = crate::layout::PreparedTextCensus::materialize(
                    requirements,
                    Some(prepared_stage),
                    Some(&mut scalar_presence),
                )
                .map_err(|error| Error::Render(format!("prepared text census: {error}")))?;
                if census.counts.structural_addresses != text_topology {
                    return Err(Error::Render(format!(
                        "prepared text topology counted {text_topology} addresses but materialized {}",
                        census.counts.structural_addresses
                    )));
                }
                theme.prepare_font_environment_from_census(
                    &mut census,
                    scalar_presence,
                    prepared_stage,
                )?;
                Ok(PreparedPlotInputs {
                    theme,
                    text_census: census,
                })
            },
            |prepared| {
                if source.has_facets() {
                    Ok(PlotLayoutPlan::Faceted(Box::new(
                        super::facets::FacetPlan::new(source, kind, &prepared.theme)?,
                    )))
                } else {
                    Ok(PlotLayoutPlan::Unfaceted(Box::new(UnfacetedPlotPlan::new(
                        source,
                        kind,
                        &prepared.theme,
                    )?)))
                }
            },
            |prepared, layout, final_stage| {
                let replay = capture_text_replay_plan(
                    source.width(),
                    source.height(),
                    "prepared plot scene capture",
                    &prepared.theme,
                    final_stage,
                    |backend| {
                        render_plot_scene(
                            source,
                            kind,
                            source.width(),
                            source.height(),
                            &prepared.theme,
                            &layout,
                            backend,
                        )
                    },
                )?;
                Ok(replay)
            },
        )?;
        let (width, height) = root.dimensions();
        validate_text_replay_plan(width, height, root.plan(), "prepared plot Null replay")?;
        Ok(Self { root })
    }

    pub(super) fn render(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        #[cfg(test)]
        super::FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(calls.get() + 1));
        let _ = &self.root.prepared().text_census;
        let (width, height) = self.dimensions();
        self.root
            .plan()
            .replay(width, height, backend, "prepared plot scene replay")
    }

    pub(super) fn dimensions(&self) -> (u32, u32) {
        self.root.dimensions()
    }

    pub(super) fn font_environment(
        &self,
    ) -> Result<&crate::font::canonical::PreparedFontEnvironment, Error> {
        self.root.prepared().theme.prepared_font_environment()
    }
}

fn render_plot_scene(
    source: &Plot,
    kind: PlotKind,
    width: u32,
    height: u32,
    theme: &ResolvedTheme,
    layout: &PlotLayoutPlan,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    backend.draw_surface(
        PanelBox::new(0.0, f64::from(width), 0.0, f64::from(height)),
        theme.surfaces.canvas_root,
        SurfaceRole::CanvasRoot,
    )?;
    match layout {
        PlotLayoutPlan::Unfaceted(plan) => plan.render(source, kind, theme, backend),
        PlotLayoutPlan::Faceted(plan) => plan.render(source, kind, theme, backend),
    }
}

impl UnfacetedPlotPlan {
    fn new(source: &Plot, kind: PlotKind, theme: &ResolvedTheme) -> Result<Self, Error> {
        let canvas_layout = source.measured_canvas_layout(theme)?;
        let cell = match kind {
            PlotKind::Matrix => crate::plots::matrix::panel_box(source, theme)?,
            _ => canvas_layout.panel_box,
        };
        let panel_plan = crate::decorations::plan::plan_panel(
            source,
            kind,
            cell,
            crate::decorations::plan::LegendRole::Panel,
            theme,
        )?;
        let plot = panel_plan.plot_with_planned_limits(source);
        Ok(Self {
            plot,
            panel_plan,
            canvas_layout,
        })
    }

    fn render(
        &self,
        source: &Plot,
        kind: PlotKind,
        theme: &ResolvedTheme,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        backend.draw_surface(
            self.panel_plan.panel_box,
            theme.surfaces.plot_panel,
            SurfaceRole::PlotPanel,
        )?;
        if self.panel_plan.result.decoration_mode == DecorationMode::Standard {
            crate::decorations::apply::apply_grid(source, &self.panel_plan.result, theme, backend)?;
        }
        let source_rows = self.plot.configured_source_rows()?;
        let context = source_rows
            .map_or_else(
                || PlotRenderContext::new(self.plot.data(), self.panel_plan.panel_box, theme),
                |rows| {
                    PlotRenderContext::with_source_rows(
                        self.plot.data(),
                        rows,
                        self.panel_plan.panel_box,
                        theme,
                    )
                },
            )
            .with_deferred_annotation_text(&self.panel_plan.result.deferred_annotation_text);
        let rendered = self.plot.render_panel(kind, context, backend)?;

        if let Some(significance) = &self.panel_plan.result.significance {
            crate::significance::render(source, significance, theme, backend)?;
        }

        match self.panel_plan.result.decoration_mode {
            DecorationMode::Standard => {
                crate::decorations::apply::apply_axes(source, &self.panel_plan, theme, backend)?;
                crate::decorations::apply::apply_planned_legend(
                    source,
                    self.panel_plan.legend.as_ref(),
                    backend,
                )?;
            }
            DecorationMode::SelfManaged => {}
        }
        crate::decorations::apply::apply_canvas_text(source, &self.canvas_layout, backend)?;
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
