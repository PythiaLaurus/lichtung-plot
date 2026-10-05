//! Render orchestration: stable chart entry points and coordinated child modules.

use crate::Error;

use crate::builder::Plot;
use crate::render::backend::Backend;
use crate::spec::{PlotKind, PlotRenderContext, PlotRenderResult};

mod artifacts;
mod backend;
mod facets;
pub(crate) use facets::text_census_panels;
pub(crate) mod layers;
mod lifecycle;
#[cfg(test)]
pub(crate) mod measurements;
mod output;
mod validation;

#[cfg(test)]
#[test]
fn classified_interior_rejects_wrong_route_before_backend_or_font_work() {
    let source = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [1.0, 2.0]),
    )
    .erase();
    for prepared in [false, true] {
        let plot = if prepared {
            crate::plots::shared_x::prepare_plot(
                &source,
                &crate::font::canonical::OperationGenerationResources::new(),
                None,
            )
            .unwrap()
            .attach(source.clone())
            .unwrap()
        } else {
            source.clone()
        };
        let theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        let context = PlotRenderContext::new(
            plot.data(),
            crate::layout::PanelBox::new(0.0, 400.0, 0.0, 300.0),
            &theme,
        );
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        RENDER_PANEL_CALLS.with(|calls| calls.set(0));
        SELECTED_INTERIOR_MEASUREMENTS.with(|calls| calls.set(0));
        measurements::start();
        let error = plot.measure_panel(PlotKind::Line, context).unwrap_err();
        assert!(matches!(
            error,
            crate::layout::fit::PlanError::Fatal(Error::Data(_))
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
        assert_eq!(
            measurements::finish().counters,
            [0; measurements::Counter::ALL.len()]
        );

        let environment = theme.prepared_font_environment().unwrap();
        let call = environment.claim_measurement_call().unwrap();
        let resources = environment.generation_resources();
        resources
            .claim_calls(1_000_000 - resources.call_cache_usage().0)
            .unwrap();
        measurements::start();
        for error in [
            plot.measure_panel(PlotKind::Line, context).unwrap_err(),
            plot.measure_panel_in_call(PlotKind::Line, context, &call)
                .unwrap_err(),
        ] {
            assert!(matches!(
                error,
                crate::layout::fit::PlanError::Fatal(Error::Data(_))
            ));
        }
        assert!(matches!(
            plot.preflight_measurement(
                PlotKind::Line,
                context.panel,
                &theme,
                &crate::layout::fit::FitScope::root(),
            ),
            Err(Error::Data(_))
        ));
        assert_eq!(resources.call_cache_usage().0, 1_000_000);
        assert_eq!(
            measurements::finish().counters,
            [0; measurements::Counter::ALL.len()]
        );
        assert_eq!(RENDER_PANEL_CALLS.with(std::cell::Cell::get), 0);
        assert_eq!(SELECTED_INTERIOR_MEASUREMENTS.with(std::cell::Cell::get), 0);
    }
}

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
    pub(crate) static SELECTED_INTERIOR_MEASUREMENTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static RENDER_PANEL_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
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
    ) -> Result<crate::spec::DecorationMode, Error> {
        match kind {
            PlotKind::Scatter
            | PlotKind::Line
            | PlotKind::Trajectory
            | PlotKind::LinePath
            | PlotKind::Histogram
            | PlotKind::Density
            | PlotKind::Qq
            | PlotKind::Regression
            | PlotKind::Dumbbell
            | PlotKind::PointInterval
            | PlotKind::Coefficient
            | PlotKind::Forest
            | PlotKind::Bar
            | PlotKind::HorizontalBar
            | PlotKind::StackedBar
            | PlotKind::StackedHorizontalBar
            | PlotKind::Boxplot
            | PlotKind::Violin
            | PlotKind::Raincloud
            | PlotKind::Interaction
            | PlotKind::Heatmap
            | PlotKind::Beeswarm
            | PlotKind::Waterfall
            | PlotKind::Marginal
            | PlotKind::Matrix => {
                crate::layout::interior::InteriorRoute::classify(kind).validate(self)?;
                let result = context.selected_interior.ok_or_else(|| {
                    Error::Render("extracted capture requires its selected interior payload".into())
                })?;
                result
                    .marks
                    .as_ref()
                    .ok_or_else(|| Error::Render("selected mark payload is missing".into()))?
                    .draw(backend)?;
                result
                    .annotations
                    .as_ref()
                    .ok_or_else(|| Error::Render("selected annotation payload is missing".into()))?
                    .draw(backend)?;
                Ok(result.decoration_mode)
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn measure_panel(
        &self,
        kind: PlotKind,
        context: PlotRenderContext<'_>,
    ) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
        crate::layout::interior::InteriorRoute::classify(kind).validate(self)?;
        let call = self.admit_interior_call(context.theme)?;
        self.measure_panel_in_call(kind, context, &call)
    }

    pub(crate) fn measure_panel_in_call(
        &self,
        kind: PlotKind,
        context: PlotRenderContext<'_>,
        call: &crate::font::canonical::MeasurementCall,
    ) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
        crate::layout::interior::InteriorRoute::classify(kind).validate(self)?;
        #[cfg(test)]
        {
            RENDER_PANEL_CALLS.with(|calls| calls.set(calls.get() + 1));
            RENDER_THEME_ADDRESSES.with(|addresses| {
                addresses
                    .borrow_mut()
                    .push(std::ptr::from_ref(context.theme).addr());
            });
        }
        let context = context.with_prepared_axes(self)?;
        match kind {
            PlotKind::Scatter
            | PlotKind::Line
            | PlotKind::Trajectory
            | PlotKind::LinePath
            | PlotKind::Histogram
            | PlotKind::Density
            | PlotKind::Qq
            | PlotKind::Regression
            | PlotKind::Dumbbell
            | PlotKind::PointInterval
            | PlotKind::Coefficient
            | PlotKind::Forest
            | PlotKind::Bar
            | PlotKind::HorizontalBar
            | PlotKind::StackedBar
            | PlotKind::StackedHorizontalBar
            | PlotKind::Boxplot
            | PlotKind::Violin
            | PlotKind::Raincloud
            | PlotKind::Interaction
            | PlotKind::Heatmap
            | PlotKind::Beeswarm
            | PlotKind::Waterfall
            | PlotKind::Marginal
            | PlotKind::Matrix => self.measure_selected_interior_in_call(context, call),
        }
    }

    pub(crate) fn measure_selected_interior(
        &self,
        context: PlotRenderContext<'_>,
    ) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
        let call = self.admit_interior_call(context.theme)?;
        self.measure_selected_interior_in_call(context, &call)
    }

    fn admit_interior_call(
        &self,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<crate::font::canonical::MeasurementCall, Error> {
        if crate::layout::interior::InteriorRoute::classify(self.kind()).is_coordinate()
            && self.prepared_axes.is_empty()
        {
            return Err(Error::Render(
                "native coordinate panel requires root-prepared axes".into(),
            ));
        }
        theme
            .prepared_font_environment()?
            .claim_measurement_call()
            .map_err(|error| Error::Render(format!("interior call admission: {error:?}")))
    }

    fn measure_selected_interior_in_call(
        &self,
        context: PlotRenderContext<'_>,
        _call: &crate::font::canonical::MeasurementCall,
    ) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
        use crate::layout::interior::{InteriorRoute as R, SpecializedGrid as G};
        #[cfg(test)]
        SELECTED_INTERIOR_MEASUREMENTS.with(|calls| calls.set(calls.get() + 1));
        #[cfg(test)]
        RENDER_THEME_ADDRESSES.with(|addresses| {
            addresses
                .borrow_mut()
                .push(std::ptr::from_ref(context.theme).addr());
        });
        let context = context.with_prepared_axes(self)?;
        let mut result = match R::classify(self.kind()) {
            route @ (R::StandardCoordinate(_) | R::OneCategoricalAxis(_)) => {
                route.measure(self, context)?
            }
            R::SpecializedGrid(G::Matrix) => crate::plots::matrix::measure(self, context)?,
            R::SpecializedGrid(G::Marginal) => crate::plots::marginal::measure(self, context)?,
            R::SpecializedGrid(G::Heatmap) => crate::plots::heatmap::measure(self, context)?,
            R::SpecializedGrid(G::Beeswarm) => crate::plots::beeswarm::measure(self, context)?,
            R::SpecializedGrid(G::Waterfall) => crate::plots::waterfall::measure(self, context)?,
        };
        let resources = context
            .theme
            .prepared_font_environment()?
            .generation_resources();
        let data = context.data.with_axis_resources(&resources);
        let mut geometry = crate::render::geometry::GeometrySnapshot::new(context.theme)?;
        let significance_styles = if matches!(
            self.kind(),
            PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud
        ) {
            let mut styles = Vec::new();
            if let Some(significance) = &result.significance {
                data.reserve_source(&mut styles, significance.brackets.len())?;
                for bracket in &significance.brackets {
                    let label = &bracket.label.plan;
                    let style = crate::render::backend::BracketStyle {
                        color: context.theme.annotations.bracket_stroke.color(),
                        width: context.theme.annotations.bracket_stroke.width(),
                        label_color: label.style.paint,
                        font_family: data.copy_font_stack(&label.style.stack)?,
                        font_size: label.style.font_size,
                        line_height: label.line_height(),
                        weight: label.style.weight,
                        label_gap: self.significance_mark_style_spec().gap_spec(),
                    };
                    geometry.bracket(crate::render::backend::PlannedSignificanceCall {
                        bracket: &bracket.geometry,
                        style: &style,
                        label: &bracket.label,
                    })?;
                    styles.push(style);
                }
            }
            Some(styles)
        } else {
            None
        };
        let guides = layers::GuideSnapshot::measure(self, &result, data, &mut geometry)?;
        validate_frozen_axes_and_guides(self, &mut result, context)?;
        result.panel_text = crate::decorations::panel_text::measure_plans(
            self,
            &result.layout,
            context.theme,
            context.fit_scope(),
        )?;
        let panel_clip = result.layout.projection.panel_box();
        let canonical_clip = crate::render::backend::validate_clip_region(panel_clip)?.panel();
        let text = data.collect_source(result.panel_text.iter().map(|plan| {
            let placed = plan.retained_placement();
            geometry.text(placed.placement, &placed.plan, Some(canonical_clip))?;
            Ok(placed)
        }))?;
        let panel_layer = if self.panel_text_annotations().is_empty() {
            None
        } else {
            let mut name = String::new();
            let length = self
                .axis_scope()
                .map_or(Some(10), |scope| scope.len().checked_add(11))
                .ok_or_else(|| Error::Render("panel text layer name overflow".into()))?;
            data.reserve_source_string(&mut name, length)?;
            if let Some(scope) = self.axis_scope() {
                name.push_str(scope);
                name.push('-');
            }
            name.push_str("panel-text");
            Some(name)
        };
        result.annotations = Some(data.source_arc(
            crate::layout::interior::SelectedAnnotationPlan {
                significance_styles,
                guides,
                panel_layer,
                panel_clip,
                text,
                geometry,
            },
        )?);
        Ok(result)
    }
}

fn validate_frozen_axes_and_guides(
    plot: &Plot,
    result: &mut PlotRenderResult,
    context: PlotRenderContext<'_>,
) -> Result<(), crate::layout::fit::PlanError> {
    if result.decoration_mode == crate::spec::DecorationMode::SelfManaged
        || context.retained_axes.is_some()
    {
        return Ok(());
    }
    let theme = context.theme;
    if result.axes.is_none() {
        return Err(Error::Render("native panel is missing its frozen axes".into()).into());
    }
    if !result.legend_entries.is_empty() {
        crate::decorations::legend::resolve_entries(
            plot,
            &mut result.legend_entries,
            &theme.prepared_font_environment()?.generation_resources(),
        )?;
    }
    Ok(())
}

#[cfg(test)]
fn validate_axis_text(
    layout: &crate::layout::Layout,
    theme: &crate::theme::ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<(), crate::layout::fit::PlanError> {
    layout
        .measure_axes(theme, scope, layout.test_semantics())
        .map(|_| ())
}

#[cfg(test)]
#[test]
fn missing_frozen_axes_do_not_trigger_label_reconstruction() {
    let plot = Plot::heatmap(
        crate::CategoryValues::new("column", ["first", "second"]).unwrap(),
        crate::CategoryValues::new("row", ["upper", "lower"]).unwrap(),
        crate::NumericValues::new("value", [1.0, 2.0]),
    )
    .x_label("Horizontal")
    .y_label("Vertical")
    .labels(crate::Labels::raw())
    .erase();
    let panel = crate::layout::PanelBox::new(0.0, 600.0, 0.0, 400.0);
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let mut result = PlotRenderResult::new(crate::layout::Layout::envelope(panel), Vec::new());
    let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
    let error = validate_frozen_axes_and_guides(
        &plot,
        &mut result,
        PlotRenderContext::new(plot.data(), panel, &theme),
    )
    .unwrap_err();
    assert!(
        matches!(error, crate::layout::fit::PlanError::Fatal(Error::Render(message)) if message.contains("missing its frozen axes"))
    );
    assert_eq!(
        shapes,
        crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get)
    );
    assert!(result.layout.x_axis.label.is_empty() && result.layout.y_axis.label.is_empty());
}

#[cfg(test)]
#[test]
fn later_axis_font_failure_wins_over_an_earlier_text_fit() {
    use crate::layout::fit::{FitScope, PlanError};
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    theme.prepare_font_environment().unwrap();
    let mut layout =
        crate::layout::Layout::envelope(crate::layout::PanelBox::new(80.0, 610.0, 60.0, 450.0));
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
