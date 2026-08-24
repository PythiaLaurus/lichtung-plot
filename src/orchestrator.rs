//! Render orchestration: stable chart entry points and coordinated child modules.

use crate::Error;

use crate::builder::Plot;
use crate::render::backend::Backend;
use crate::spec::{AxisDomain, PlotKind, PlotRenderContext, PlotRenderResult};

mod artifacts;
mod backend;
mod facets;
mod layers;
mod lifecycle;
mod output;
mod validation;

#[cfg(feature = "png")]
pub(crate) use artifacts::{
    paths_resolve_to_same_destination, publish_staged_pair, reserve_staging_pair,
};
pub(crate) use artifacts::{
    publish_staged, reserve_unique_sibling_temp_path, with_temporary_artifacts,
};
pub(crate) use backend::DeferredBackend;

#[cfg(test)]
thread_local! {
    static RENDER_PANEL_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

impl<K: crate::kind::Marker> Plot<K> {
    /// Validate the complete selected chart without opening or publishing an output.
    ///
    /// This erases the compile-time marker and runs the same closed runtime
    /// dispatch, data, geometry, facet, decoration, and representability path
    /// used by rendering through a validating null backend.
    ///
    /// # Errors
    ///
    /// Returns every error that would be detected before or during rendering.
    pub fn preflight(&self) -> Result<(), Error> {
        self.clone().erase().preflight_erased()
    }

    #[cfg(test)]
    pub(crate) fn render_with_backend(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        self.clone().erase().render_with_backend_erased(backend)
    }

    #[cfg(test)]
    pub(crate) fn render_kind(
        &self,
        kind: PlotKind,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        self.clone().erase().render_kind_erased(kind, backend)
    }
}

impl Plot {
    /// Render the selected plot through a caller-owned backend.
    ///
    /// This performs the canonical one-canvas lifecycle but does not create or
    /// publish files. After a successful `begin`, `end` is attempted exactly
    /// once even if plot rendering fails. If rendering and finalization both
    /// fail, the returned error preserves both contexts.
    ///
    /// # Errors
    ///
    /// Returns an error when backend startup or finalization fails, or the
    /// selected renderer rejects its data or options.
    fn render_with_backend_erased(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        self.render_kind_erased(self.kind(), backend)
    }

    fn preflight_erased(&self) -> Result<(), Error> {
        self.render_with_backend_erased(&mut backend::NullBackend)
    }

    pub(crate) fn render_panel(
        &self,
        kind: PlotKind,
        context: PlotRenderContext<'_>,
        backend: &mut dyn Backend,
    ) -> Result<PlotRenderResult, Error> {
        #[cfg(test)]
        RENDER_PANEL_CALLS.with(|calls| calls.set(calls.get() + 1));
        let mut result = match kind {
            PlotKind::Scatter => crate::plots::scatter::render(self, context, backend),
            PlotKind::Histogram => crate::plots::histogram::render(self, context, backend),
            PlotKind::Density => crate::plots::density::render(self, context, backend),
            PlotKind::Qq => crate::plots::qq::render(self, context, backend),
            PlotKind::PointInterval => crate::plots::point_interval::render(self, context, backend),
            PlotKind::Line => crate::plots::line::render(self, context, backend),
            PlotKind::Trajectory => crate::plots::line::render_trajectory(self, context, backend),
            PlotKind::LinePath => crate::plots::line::render_path(self, context, backend),
            PlotKind::Interaction => crate::plots::interaction::render(self, context, backend),
            PlotKind::Coefficient => crate::plots::coefficient::render(self, context, backend),
            PlotKind::Forest => crate::plots::forest::render(self, context, backend),
            PlotKind::Dumbbell => crate::plots::dumbbell::render(self, context, backend),
            PlotKind::Heatmap => crate::plots::heatmap::render(self, context, backend),
            PlotKind::Bar | PlotKind::StackedBar => {
                crate::plots::bar::render(self, context, backend)
            }
            PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar => {
                crate::plots::horizontal_bar::render(self, context, backend)
            }
            PlotKind::Boxplot => crate::plots::boxplot::render(self, context, backend),
            PlotKind::Violin => crate::plots::violin::render(self, context, backend),
            PlotKind::Raincloud => crate::plots::violin::render_raincloud(self, context, backend),
            PlotKind::Regression => crate::plots::regression::render(self, context, backend),
            PlotKind::Marginal => crate::plots::marginal::render(self, context, backend),
            PlotKind::Matrix => crate::plots::matrix::render(self, context, backend),
            PlotKind::Beeswarm => crate::plots::beeswarm::render(self, context, backend),
            PlotKind::Waterfall => crate::plots::waterfall::render(self, context, backend),
        }?;
        let domains = match kind {
            PlotKind::Interaction
            | PlotKind::Bar
            | PlotKind::StackedBar
            | PlotKind::Boxplot
            | PlotKind::Violin
            | PlotKind::Raincloud => (AxisDomain::Categorical, AxisDomain::Continuous),
            PlotKind::PointInterval
            | PlotKind::Coefficient
            | PlotKind::Forest
            | PlotKind::Dumbbell
            | PlotKind::HorizontalBar
            | PlotKind::StackedHorizontalBar
            | PlotKind::Beeswarm
            | PlotKind::Waterfall => (AxisDomain::Continuous, AxisDomain::Categorical),
            PlotKind::Heatmap => (AxisDomain::Categorical, AxisDomain::Categorical),
            PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
                (result.x_domain, result.y_domain)
            }
            _ => (AxisDomain::Continuous, AxisDomain::Continuous),
        };
        result.x_domain = domains.0;
        result.y_domain = domains.1;
        layers::apply(self, &mut result, backend)?;
        let (x_label, y_label) = self.resolved_axis_labels(kind);
        result.layout.x_axis.label = x_label;
        result.layout.y_axis.label = y_label;
        resolve_rendered_labels(self, &mut result)?;
        crate::decorations::panel_text::draw(self, &result.layout, backend)?;
        Ok(result)
    }
}

fn resolve_rendered_labels(chart: &Plot, result: &mut PlotRenderResult) -> Result<(), Error> {
    if result.x_domain == AxisDomain::Categorical && result.x_tick_keys.is_none() {
        let raw = result
            .layout
            .x_axis
            .ticks
            .iter()
            .map(|tick| tick.label.clone())
            .collect::<Vec<_>>();
        let display =
            chart.resolve_label_domain("x-axis categories", raw.iter().map(String::as_str))?;
        for (tick, label) in result.layout.x_axis.ticks.iter_mut().zip(display) {
            tick.label = label;
        }
        result.x_tick_keys = Some(raw);
    }
    if result.y_domain == AxisDomain::Categorical && result.y_tick_keys.is_none() {
        let raw = result
            .layout
            .y_axis
            .ticks
            .iter()
            .map(|tick| tick.label.clone())
            .collect::<Vec<_>>();
        let display =
            chart.resolve_label_domain("y-axis categories", raw.iter().map(String::as_str))?;
        for (tick, label) in result.layout.y_axis.ticks.iter_mut().zip(display) {
            tick.label = label;
        }
        result.y_tick_keys = Some(raw);
    }
    if !result.legend_entries.is_empty() {
        crate::decorations::legend::resolve_entries(chart, &mut result.legend_entries)?;
    }
    validate_axis_text(chart, &result.layout)?;
    Ok(())
}

#[expect(
    clippy::cast_precision_loss,
    reason = "tick counts are bounded by the finite panel width and allocated tick vector"
)]
fn validate_axis_text(chart: &Plot, layout: &crate::layout::Layout) -> Result<(), Error> {
    let palette = chart.palette();
    let panel = layout.projection.panel_box();
    let x_tick_width = panel.width() / layout.x_axis.ticks.len().max(1) as f64 * 0.9;
    for tick in layout
        .x_axis
        .ticks
        .iter()
        .filter(|tick| !tick.label.is_empty())
    {
        crate::layout::TextBlock::layout(
            &tick.label,
            palette.tick_font_size,
            x_tick_width.max(1.0),
            "x-axis tick label",
        )?;
    }
    if !layout.x_axis.label.is_empty() {
        crate::layout::TextBlock::layout(
            &layout.x_axis.label,
            palette.label_font_size,
            panel.width().max(1.0),
            "x-axis title",
        )?;
    }

    if !layout.y_axis.label.is_empty() {
        crate::layout::TextBlock::layout(
            &layout.y_axis.label,
            palette.label_font_size,
            panel.height().max(1.0),
            "y-axis title",
        )?;
    }
    let y_tick_width = (panel.width() * 0.4).max(1.0);
    for tick in layout
        .y_axis
        .ticks
        .iter()
        .filter(|tick| !tick.label.is_empty())
    {
        crate::layout::TextBlock::layout(
            &tick.label,
            palette.tick_font_size,
            y_tick_width,
            "y-axis tick label",
        )?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "orchestrator/tests.rs"]
mod tests;
