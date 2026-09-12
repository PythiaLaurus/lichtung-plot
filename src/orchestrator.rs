//! Render orchestration: stable chart entry points and coordinated child modules.

use crate::Error;

use crate::builder::Plot;
use crate::render::backend::Backend;
use crate::spec::{AxisDomain, PlotKind, PlotRenderContext, PlotRenderResult};

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
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [1.0, 2.0]),
    )
    .erase();
    let theme = crate::Theme::Publication.resolve();
    let context = PlotRenderContext::new(
        plot.data(),
        crate::layout::PanelBox::new(0.0, 400.0, 0.0, 300.0),
        &theme,
    );
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let mut backend = backend::NullBackend::default();
    let error = crate::layout::interior::InteriorRoute::classify(PlotKind::Line)
        .combined(&plot, context, &mut backend)
        .err()
        .unwrap();
    assert!(matches!(
        error,
        crate::layout::fit::PlanError::Fatal(Error::Data(_))
    ));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
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
    ) -> Result<crate::spec::DecorationMode, Error> {
        if matches!(
            kind,
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
                | PlotKind::Marginal
                | PlotKind::Matrix
        ) {
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
            return Ok(result.decoration_mode);
        }
        self.measure_panel(kind, context, backend)
            .map(|result| result.decoration_mode)
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
        let context = context.with_prepared_axes(self);
        if matches!(
            kind,
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
                | PlotKind::Marginal
                | PlotKind::Matrix
        ) {
            return self.measure_selected_interior(context);
        }
        let mut result = crate::layout::interior::InteriorRoute::classify(kind)
            .combined(self, context, backend)?;
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
            PlotKind::Scatter
            | PlotKind::Histogram
            | PlotKind::Density
            | PlotKind::Qq
            | PlotKind::Regression
            | PlotKind::Marginal
            | PlotKind::Matrix => (AxisDomain::Continuous, AxisDomain::Continuous),
        };
        result.x_domain = domains.0;
        result.y_domain = domains.1;
        layers::apply(self, &mut result, backend)?;
        resolve_rendered_labels(self, kind, &mut result, context)?;
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

    #[expect(
        clippy::too_many_lines,
        reason = "selected marks and common annotation snapshot share one measurement boundary"
    )]
    pub(crate) fn measure_selected_interior(
        &self,
        context: PlotRenderContext<'_>,
    ) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
        #[cfg(test)]
        RENDER_THEME_ADDRESSES.with(|addresses| {
            addresses
                .borrow_mut()
                .push(std::ptr::from_ref(context.theme).addr());
        });
        let context = context.with_prepared_axes(self);
        let mut result = match self.kind() {
            PlotKind::Matrix => crate::plots::matrix::measure(self, context)?,
            PlotKind::Marginal => crate::plots::marginal::measure(self, context)?,
            PlotKind::Heatmap => crate::plots::heatmap::measure(self, context)?,
            PlotKind::Beeswarm => crate::plots::beeswarm::measure(self, context)?,
            PlotKind::Interaction => crate::plots::interaction::measure(self, context)?,
            PlotKind::Boxplot => crate::plots::boxplot::measure(self, context)?,
            PlotKind::Violin | PlotKind::Raincloud => crate::plots::violin::measure(self, context)?,
            PlotKind::Bar | PlotKind::StackedBar => crate::plots::bar::measure(self, context)?,
            PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar => {
                crate::plots::horizontal_bar::measure(self, context)?
            }
            PlotKind::Scatter => crate::plots::scatter::measure(self, context)?,
            PlotKind::Histogram => crate::plots::histogram::measure(self, context)?,
            PlotKind::Density => crate::plots::density::measure(self, context)?,
            PlotKind::Qq => crate::plots::qq::measure(self, context)?,
            PlotKind::Dumbbell => crate::plots::dumbbell::measure(self, context)?,
            PlotKind::PointInterval | PlotKind::Coefficient | PlotKind::Forest => {
                crate::plots::interval_estimate::measure(self, context)?
            }
            PlotKind::Regression => crate::plots::regression::measure(self, context)?,
            PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => {
                crate::plots::line::measure(self, context)?
            }
            PlotKind::Waterfall => {
                return Err(Error::Render(
                    "selected measurement requires an extracted family".into(),
                )
                .into())
            }
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
                        color: context.theme.foreground.into(),
                        width: context.theme.bracket_width,
                        label_color: label.style.paint,
                        font_family: data.copy_source_string(&label.style.stack_source)?,
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
        resolve_rendered_labels(self, self.kind(), &mut result, context)?;
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
            crate::layout::interior::DeferredAnnotationPlan {
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

#[expect(
    clippy::too_many_lines,
    reason = "raw category identities, parent-suppressed display labels, and measured axes share one resolution boundary"
)]
fn resolve_rendered_labels(
    plot: &Plot,
    kind: PlotKind,
    result: &mut PlotRenderResult,
    context: PlotRenderContext<'_>,
) -> Result<(), crate::layout::fit::PlanError> {
    if result.decoration_mode == crate::spec::DecorationMode::SelfManaged
        || context.retained_axes.is_some()
    {
        return Ok(());
    }
    let theme = context.theme;
    let scope = context.fit_scope();
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = context.data.with_axis_resources(&resources);
    let (x_label, y_label) = match plot.prepared_axis_inputs() {
        Some(axes) => (
            data.copy_source_string(&axes.labels[0])?,
            data.copy_source_string(&axes.labels[1])?,
        ),
        None => plot.resolved_axis_labels(kind),
    };
    result.layout.x_axis.label = x_label;
    result.layout.y_axis.label = y_label;
    if result.x_domain == AxisDomain::Categorical && result.x_tick_keys.is_none() {
        let raw = data.collect_source(
            result
                .layout
                .x_axis
                .ticks
                .iter()
                .map(|tick| data.copy_source_string(&tick.label)),
        )?;
        let display = match context.prepared_category_labels(0) {
            Some(labels) => {
                data.collect_source(labels.iter().map(|label| data.copy_source_string(label)))?
            }
            None => plot.resolve_axis_label_domain(
                "x-axis categories",
                raw.iter().map(String::as_str),
                data,
            )?,
        };
        for (tick, label) in result.layout.x_axis.ticks.iter_mut().zip(display) {
            tick.label = label;
        }
        result.x_tick_keys = Some(raw);
    }
    if result.y_domain == AxisDomain::Categorical && result.y_tick_keys.is_none() {
        let raw = data.collect_source(
            result
                .layout
                .y_axis
                .ticks
                .iter()
                .map(|tick| data.copy_source_string(&tick.label)),
        )?;
        let display = match context.prepared_category_labels(1) {
            Some(labels) => {
                data.collect_source(labels.iter().map(|label| data.copy_source_string(label)))?
            }
            None => plot.resolve_axis_label_domain(
                "y-axis categories",
                raw.iter().map(String::as_str),
                data,
            )?,
        };
        for (tick, label) in result.layout.y_axis.ticks.iter_mut().zip(display) {
            tick.label = label;
        }
        result.y_tick_keys = Some(raw);
    }
    if matches!(
        kind,
        PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
    ) && !plot.y_tick_labels_visible()
    {
        data.claim_source_visits(result.layout.y_axis.ticks.len())?;
        for tick in &mut result.layout.y_axis.ticks {
            tick.label.clear();
        }
    }
    if !result.legend_entries.is_empty() {
        crate::decorations::legend::resolve_entries(plot, &mut result.legend_entries, theme)?;
    }
    if result.y_domain == AxisDomain::Categorical
        && result.layout.y_axis.assigned_tick_width.is_none()
    {
        let height = result.layout.projection.panel_box().height();
        if plot.parent_owns_axis_gutters() {
            // The composition reserves the reported intrinsic gutter. Its residual
            // mark width is not a second, narrower label allocation.
            result.layout.y_axis.assigned_tick_width = Some(
                theme.intrinsic_tick_label_width(
                    result
                        .layout
                        .y_axis
                        .ticks
                        .iter()
                        .map(|tick| tick.label.as_str()),
                    height,
                )?,
            );
        } else {
            result
                .layout
                .y_axis
                .assign_label_width(context.axis_label_width, height, theme)?;
        }
    }
    result.axes = Some(result.layout.measure_axes(
        theme,
        scope,
        plot.axis_semantic_inputs([result.x_domain, result.y_domain]),
    )?);
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
#[expect(
    clippy::too_many_lines,
    reason = "one fixture checks copy-only refusal and warm measured-consumption limits"
)]
fn rendered_axis_labels_refuse_at_measured_consumption_limits() {
    use crate::font::canonical::OperationGenerationResources;
    use crate::layout::fit::PlanError;
    use crate::layout::{Axis, PanelBox, Projection, TickSource};
    let plot = Plot::heatmap(
        crate::CategoryValues::new("column", ["first", "second"]).unwrap(),
        crate::CategoryValues::new("row", ["upper", "lower"]).unwrap(),
        crate::NumericValues::new("value", [1.0, 2.0]),
    )
    .x_label("Horizontal")
    .y_label("Vertical")
    .labels(crate::Labels::raw())
    .erase();
    let prepared =
        crate::plots::shared_x::prepare_plot(&plot, &OperationGenerationResources::new()).unwrap();
    let plot = prepared.attach(plot).unwrap();
    let axes = plot.prepared_axis_inputs().unwrap();
    let panel = PanelBox::new(0.0, 600.0, 0.0, 400.0);
    let template = crate::layout::Layout {
        projection: Projection::new(axes.ranges[0], axes.ranges[1], panel.as_tuple()),
        x_axis: Axis {
            label: String::new(),
            ticks: axes.categories[0].clone().unwrap(),
            explicit_ticks: false,
            assigned_tick_width: None,
            source: TickSource::SUPPLIED,
        },
        y_axis: Axis {
            label: String::new(),
            ticks: axes.categories[1].clone().unwrap(),
            explicit_ticks: false,
            assigned_tick_width: Some(200.0),
            source: TickSource::SUPPLIED,
        },
    };
    let consume = |theme: &crate::theme::ResolvedTheme| {
        let mut result = PlotRenderResult::new(template.clone(), Vec::new());
        result.x_domain = AxisDomain::Categorical;
        result.y_domain = AxisDomain::Categorical;
        resolve_rendered_labels(
            &plot,
            plot.kind(),
            &mut result,
            PlotRenderContext::new(plot.data(), panel, theme).with_prepared_axes(&plot),
        )?;
        Ok::<_, PlanError>(result)
    };
    // Stop at the copy boundary, before any final axis-plan admission. This
    // distinguishes admitted source copies from charging only the final plan.
    let copy_bytes = (18 + 2 * 21 + 8 * std::mem::size_of::<String>()) as u64;
    for shortfall in [0, 1] {
        let theme = crate::Theme::Publication.resolve();
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        resources
            .claim_bytes(536_870_912 - resources.byte_usage().unwrap() - copy_bytes + shortfall)
            .unwrap();
        let mut result = PlotRenderResult::new(template.clone(), Vec::new());
        result.x_domain = AxisDomain::Categorical;
        result.y_domain = AxisDomain::Categorical;
        let error = resolve_rendered_labels(
            &plot,
            plot.kind(),
            &mut result,
            PlotRenderContext::new(plot.data(), panel, &theme).with_prepared_axes(&plot),
        )
        .unwrap_err();
        let PlanError::Fatal(Error::Render(message)) = error else {
            panic!("copy resource refusal must remain fatal")
        };
        assert!(
            message.contains(if shortfall == 0 {
                "axis plan admission"
            } else {
                "axis source storage admission"
            }),
            "{message}"
        );
        assert!(result.x_tick_keys.is_some());
        assert_eq!(result.y_tick_keys.is_some(), shortfall == 0);
        assert!(result.axes.is_none());
    }
    for shortfall in [None, Some(false), Some(true)] {
        let theme = crate::Theme::Publication.resolve();
        consume(&theme).unwrap();
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        let before = (
            resources.usage().unwrap().1,
            resources.byte_usage().unwrap(),
        );
        let measured = consume(&theme).unwrap();
        assert_eq!(measured.layout.x_axis.label, "Horizontal");
        assert_eq!(measured.layout.y_axis.label, "Vertical");
        assert_eq!(measured.x_tick_keys.as_ref().unwrap(), &["first", "second"]);
        assert_eq!(measured.y_tick_keys.as_ref().unwrap(), &["upper", "lower"]);
        let visits = resources.usage().unwrap().1 - before.0;
        let bytes = resources.byte_usage().unwrap() - before.1;
        assert!(bytes >= copy_bytes);
        resources
            .claim_visits(
                1_000_000 - resources.usage().unwrap().1 - visits
                    + u128::from(shortfall == Some(false)),
            )
            .unwrap();
        resources
            .claim_bytes(
                536_870_912 - resources.byte_usage().unwrap() - bytes
                    + u64::from(shortfall == Some(true)),
            )
            .unwrap();
        let result = consume(&theme);
        if shortfall.is_none() {
            result.unwrap();
        } else {
            assert!(matches!(result, Err(PlanError::Fatal(Error::Render(_)))));
        }
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
