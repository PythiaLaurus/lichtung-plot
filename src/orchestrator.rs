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
#[cfg(test)]
pub(crate) mod measurements;
mod output;
mod validation;

pub(crate) use artifacts::write_atomic;
#[cfg(test)]
pub(crate) use artifacts::TEMP_RESERVATION_CALLS;
pub(crate) use backend::{capture_text_replay_plan, validate_text_replay_plan, TextReplayPlan};
#[cfg(test)]
pub(crate) use backend::{DeferredBackend, TEXT_PLAN_CAPTURE_SESSIONS, TEXT_REPLAY_SESSIONS};
pub(crate) use facets::text_census_labels as facet_text_census_labels;
pub(crate) use facets::text_census_panels as facet_text_census_panels;

#[cfg(test)]
thread_local! {
    static RENDER_PANEL_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static RENDER_THEME_ADDRESSES: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) };
    static PLOT_PLAN_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static FINAL_PLOT_RENDER_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
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
        crate::kind::into_erased(self.clone()).preflight_erased()
    }

    #[cfg(test)]
    pub(crate) fn render_with_backend(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        crate::kind::into_erased(self.clone()).render_with_backend_erased(backend)
    }

    #[cfg(test)]
    pub(crate) fn render_kind(
        &self,
        kind: PlotKind,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        crate::kind::into_erased(self.clone()).render_kind_erased(kind, backend)
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
    #[cfg(test)]
    fn render_with_backend_erased(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        lifecycle::PlotPlan::new(self, self.kind())?.render(backend)
    }

    fn preflight_erased(&self) -> Result<(), Error> {
        lifecycle::PlotPlan::new(self, self.kind()).map(|_| ())
    }

    pub(crate) fn render_panel(
        &self,
        kind: PlotKind,
        context: PlotRenderContext<'_>,
        backend: &mut dyn Backend,
    ) -> Result<PlotRenderResult, Error> {
        self.measure_panel(kind, context, backend)
            .map_err(crate::layout::fit::PlanError::into_error)
    }

    pub(crate) fn measure_panel(
        &self,
        kind: PlotKind,
        context: PlotRenderContext<'_>,
        backend: &mut dyn Backend,
    ) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
        #[cfg(test)]
        {
            RENDER_PANEL_CALLS.with(|calls| calls.set(calls.get() + 1));
            RENDER_THEME_ADDRESSES.with(|addresses| {
                addresses
                    .borrow_mut()
                    .push(std::ptr::from_ref(context.theme).addr());
            });
        }
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
            PlotKind::Marginal => {
                crate::plots::marginal::render(self, context, backend).map_err(Into::into)
            }
            PlotKind::Matrix => {
                crate::plots::matrix::render(self, context, backend).map_err(Into::into)
            }
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
        layers::apply(
            self,
            &mut result,
            context.theme,
            backend,
            context.fit_scope(),
        )?;
        let (x_label, y_label) = self.resolved_axis_labels(kind);
        result.layout.x_axis.label = x_label;
        result.layout.y_axis.label = y_label;
        resolve_rendered_labels(self, &mut result, context.theme, context.fit_scope())?;
        result.panel_text = match context.panel_text {
            Some(plans) => plans.to_vec(),
            None => crate::decorations::panel_text::measure_plans(
                self,
                &result.layout,
                context.theme,
                context.fit_scope(),
            )?,
        };
        crate::decorations::panel_text::draw(self, &result.layout, &result.panel_text, backend)?;
        Ok(result)
    }
}

fn resolve_rendered_labels(
    plot: &Plot,
    result: &mut PlotRenderResult,
    theme: &crate::theme::ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<(), crate::layout::fit::PlanError> {
    if result.decoration_mode == crate::spec::DecorationMode::SelfManaged {
        return Ok(());
    }
    if result.x_domain == AxisDomain::Categorical && result.x_tick_keys.is_none() {
        let raw = result
            .layout
            .x_axis
            .ticks
            .iter()
            .map(|tick| tick.label.clone())
            .collect::<Vec<_>>();
        let display =
            plot.resolve_label_domain("x-axis categories", raw.iter().map(String::as_str))?;
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
            plot.resolve_label_domain("y-axis categories", raw.iter().map(String::as_str))?;
        for (tick, label) in result.layout.y_axis.ticks.iter_mut().zip(display) {
            tick.label = label;
        }
        result.y_tick_keys = Some(raw);
    }
    if !result.legend_entries.is_empty() {
        crate::decorations::legend::resolve_entries(plot, &mut result.legend_entries)?;
    }
    validate_axis_text(&result.layout, theme, scope)?;
    Ok(())
}

#[expect(
    clippy::cast_precision_loss,
    reason = "tick counts are bounded by the finite panel width and allocated tick vector"
)]
fn validate_axis_text(
    layout: &crate::layout::Layout,
    theme: &crate::theme::ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<(), crate::layout::fit::PlanError> {
    use crate::layout::fit;
    let mut miss = None;
    let mut record = |result: Result<fit::FitOutcome<crate::layout::ResolvedTextPlan>, Error>| -> Result<(), Error> {
        if let fit::FitOutcome::Miss(failure) = result? { fit::FitFailure::retain_least(&mut miss, failure); }
        Ok(())
    };
    let x_scope = scope.child(crate::layout::AddressRole::AxisBottom, 0)?;
    let y_scope = scope.child(crate::layout::AddressRole::AxisLeft, 0)?;
    let panel = layout.projection.panel_box();
    let x_tick_width = panel.width() / layout.x_axis.ticks.len().max(1) as f64;
    for tick in layout
        .x_axis
        .ticks
        .iter()
        .filter(|tick| !tick.label.is_empty())
    {
        record(fit::text(
            theme,
            &tick.label,
            &theme.typography.tick_label,
            x_tick_width.max(1.0),
            crate::layout::PhysicalTextAlignment::Center,
            "x-axis tick label",
            &x_scope,
        ))?;
    }
    if !layout.x_axis.label.is_empty() {
        record(fit::text(
            theme,
            &layout.x_axis.label,
            &theme.typography.axis_title,
            panel.width().max(1.0),
            crate::layout::PhysicalTextAlignment::Center,
            "x-axis title",
            &x_scope,
        ))?;
    }

    if !layout.y_axis.label.is_empty() {
        record(fit::text(
            theme,
            &layout.y_axis.label,
            &theme.typography.axis_title,
            panel.height().max(1.0),
            crate::layout::PhysicalTextAlignment::Center,
            "y-axis title",
            &y_scope,
        ))?;
    }
    let y_tick_width =
        layout
            .y_axis
            .tick_label_width(panel, crate::layout::AxisSide::Left, theme)?;
    for tick in layout
        .y_axis
        .ticks
        .iter()
        .filter(|tick| !tick.label.is_empty())
    {
        record(fit::text(
            theme,
            &tick.label,
            &theme.typography.tick_label,
            y_tick_width,
            crate::layout::PhysicalTextAlignment::End,
            "y-axis tick label",
            &y_scope,
        ))?;
    }
    match miss {
        Some(failure) => Err(failure.into()),
        None => Ok(()),
    }
}

#[cfg(test)]
#[test]
fn later_axis_font_failure_wins_over_an_earlier_text_fit() {
    use crate::layout::fit::{FitScope, PlanError};
    let theme = crate::Theme::Publication.resolve();
    theme.prepare_font_environment().unwrap();
    let mut layout = crate::layout::Layout::self_managed(640, 480);
    layout.x_axis.label = "W".repeat(200);
    let scope = FitScope::root();
    assert!(matches!(
        validate_axis_text(&layout, &theme, &scope),
        Err(PlanError::Fit(_))
    ));
    layout.y_axis.label = "\u{10ffff}".into();
    assert!(matches!(
        validate_axis_text(&layout, &theme, &scope),
        Err(PlanError::Fatal(_))
    ));
}

#[cfg(test)]
#[path = "orchestrator/tests.rs"]
mod tests;
