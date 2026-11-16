//! Render orchestration over stable descriptions and one root operation.

use crate::builder::Plot;
#[cfg(test)]
use crate::render::backend::Backend;
use crate::spec::{PlotKind, PlotMeasureContext, PlotRenderResult};
use crate::Error;

mod artifacts;
pub(crate) mod assignment;
mod backend;
mod facets;
pub(crate) use facets::{prepare_discrete_guide, prepare_facet_scopes};
pub(crate) mod layers;
mod lifecycle;
#[cfg(test)]
pub(crate) use lifecycle::{complete_prepared_plot_for_test, with_prepared_plot_for_test};
#[cfg(test)]
pub(crate) mod measurements;
mod output;
pub(crate) mod plan;
pub(crate) mod registration;
pub(crate) mod root;
mod validation;

pub(crate) use artifacts::write_atomic;
pub(crate) use backend::{capture_calls, validate_text_replay_plan, TextReplayPlan};
pub(crate) use facets::text_census_labels as facet_text_census_labels;
pub(crate) use lifecycle::{capture_selected, plan_scene, render_scene};

#[cfg(test)]
thread_local! {
    pub(crate) static SELECTED_INTERIOR_MEASUREMENTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static FINAL_PLOT_RENDER_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

impl<K: crate::kind::Marker> Plot<K> {
    /// Validate the complete selected chart without publishing an output.
    ///
    /// # Errors
    ///
    /// Returns an error when preparation, measurement or validating replay fails.
    pub fn preflight(&self) -> Result<(), Error> {
        plan_scene(root::RootSource::Plot(self.description())).map(|_| ())
    }

    #[cfg(test)]
    pub(crate) fn render_with_backend(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        let frozen = plan_scene(root::RootSource::Plot(self.description()))?;
        let (width, height) = frozen.dimensions();
        FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(calls.get() + 1));
        frozen
            .plan()
            .replay(width, height, backend, "prepared plot scene replay")
    }

    #[cfg(test)]
    pub(crate) fn render_kind(
        &self,
        kind: PlotKind,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        self.description().validate_configuration(kind)?;
        self.render_with_backend(backend)
    }
}

pub(crate) fn validate_interior_preparation(
    registered: &registration::PlotRegistration<'_>,
) -> Result<(), Error> {
    if crate::layout::interior::InteriorRoute::classify(registered.source.kind()).is_coordinate()
        && registered.axes.coordinate().is_none()
    {
        return Err(Error::Render(
            "native coordinate panel requires root-prepared axes".into(),
        ));
    }
    Ok(())
}

pub(crate) fn measure_plot(
    factory: &assignment::PlotMeasureAssignmentFactory<'_, '_>,
    entry: assignment::PlotMeasureAssignment,
    guides: crate::decorations::legend::discrete::scoped::GuideConsumption<'_>,
    scope: &crate::layout::fit::FitScope,
) -> Result<(assignment::PlotMeasureAssignment, PlotRenderResult), crate::layout::fit::PlanError> {
    let request =
        crate::text::TextRequestContext::new(&factory.registered.appearance, factory.text);
    measure_plot_with_request(factory, entry, guides, scope, &request)
}

pub(crate) fn measure_probe(
    factory: &assignment::PlotMeasureAssignmentFactory<'_, '_>,
    entry: assignment::PlotMeasureAssignment,
    guides: crate::decorations::legend::discrete::scoped::GuideConsumption<'_>,
    scope: &crate::layout::fit::FitScope,
) -> Result<(assignment::PlotMeasureAssignment, PlotRenderResult), crate::layout::fit::PlanError> {
    let source = factory.registered.source;
    crate::layout::interior::InteriorRoute::classify(source.kind()).validate(source)?;
    let mut frame = crate::layout::Layout::envelope(entry.body);
    {
        let context = factory.context(&entry, guides, scope)?;
        let [x, y] = if matches!(
            entry.domains,
            assignment::AxisDomainAssignments::Matrix { .. }
        ) {
            [(0.0, 1.0); 2]
        } else {
            [
                context.axis_range(crate::layout::AxisSide::Bottom)?,
                context.axis_range(crate::layout::AxisSide::Left)?,
            ]
        };
        frame.projection = crate::layout::Projection::new(x, y, entry.body.as_tuple());
    }
    let appearance = factory.registered.appearance.clone();
    let request = crate::text::TextRequestContext::new(&appearance, factory.text);
    let request = request.with_text_projection(factory.registered, &frame, entry.dimensions);
    let guides = crate::decorations::legend::discrete::scoped::GuideConsumption {
        use_kind: registration::discrete_guide_use(source, Some(factory.registered)),
        ..guides
    };
    measure_plot_with_request(factory, entry, guides, scope, &request)
}

fn measure_plot_with_request(
    factory: &assignment::PlotMeasureAssignmentFactory<'_, '_>,
    entry: assignment::PlotMeasureAssignment,
    guides: crate::decorations::legend::discrete::scoped::GuideConsumption<'_>,
    scope: &crate::layout::fit::FitScope,
    request: &crate::text::TextRequestContext<'_, '_>,
) -> Result<(assignment::PlotMeasureAssignment, PlotRenderResult), crate::layout::fit::PlanError> {
    validate_interior_preparation(factory.registered)?;
    #[cfg(test)]
    SELECTED_INTERIOR_MEASUREMENTS.with(|calls| calls.set(calls.get() + 1));
    let (entry, explanation) =
        crate::numeric::values::reserve(factory, entry, guides, request, scope)?;
    let result = {
        let context = factory.context(&entry, guides, scope)?;
        let result = crate::layout::interior::measure(&context, request)?;
        finish_selected_interior(&context, result, explanation, request)?
    };
    Ok((entry, result))
}

/// Finish the original selected-interior tail after the scientific producer.
/// The caller supplies its already selected request and explanation, never a
/// second measurement or a reconstructed context.
fn finish_selected_interior(
    context: &PlotMeasureContext<'_, '_>,
    mut result: PlotRenderResult,
    explanation: Option<crate::render::backend::PlacedText>,
    request: &crate::text::TextRequestContext<'_, '_>,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let source = context.registered.source;
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    if let Some(explanation) = &explanation {
        geometry.text(explanation.placement, &explanation.plan, None)?;
    }
    if matches!(
        source.kind(),
        PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud
    ) {
        if let Some(significance) = &result.significance {
            for bracket in &significance.brackets {
                geometry.bracket(crate::render::backend::PlannedSignificanceCall {
                    bracket: &bracket.geometry,
                    style: &bracket.style,
                    label: &bracket.label,
                })?;
            }
        }
    }
    let guides = layers::GuideSnapshot::measure(source, &result, &mut geometry)?;
    validate_frozen_axes_and_guides(Some(context.registered), &mut result)?;
    result.panel_text = crate::decorations::panel_text::measure_plans(
        context.registered,
        context.assignment.local.dimensions,
        &result.layout,
        request,
        context.fit_scope,
    )?;
    let panel_clip = if source.panel_text_annotations().iter().any(|annotation| {
        annotation.position().as_corner().is_none()
            || annotation.content().get_layout().is_some_and(|layout| {
                layout
                    .instructions()
                    .overflow
                    .as_ref()
                    .is_some_and(|overflow| overflow.failure == crate::TextFailure::Visible)
            })
    }) {
        let (width, height) = context
            .assignment
            .local
            .dimensions
            .figure_size()
            .dimensions()?;
        crate::layout::PixelRect::new(0.0, f64::from(width), 0.0, f64::from(height))
    } else {
        result.layout.projection.panel_box()
    };
    let canonical_clip = crate::render::backend::validate_clip_region(panel_clip)?.panel();
    let text = crate::data::source::collect_source(result.panel_text.iter().map(|plan| {
        let placed = plan.retained_placement();
        geometry.text(placed.placement, &placed.plan, Some(canonical_clip))?;
        Ok(placed)
    }))?;
    let panel_layer = if source.panel_text_annotations().is_empty() {
        None
    } else {
        let mut name = String::new();
        let length = context
            .registered
            .scope
            .render_name
            .as_ref()
            .map_or(Some(10), |scope| scope.len().checked_add(11))
            .ok_or_else(|| Error::Render("panel text layer name overflow".into()))?;
        crate::data::source::reserve_source_string(&mut name, length)?;
        if let Some(scope) = &context.registered.scope.render_name {
            name.push_str(scope);
            name.push('-');
        }
        name.push_str("panel-text");
        Some(name)
    };
    result.annotations = Some(crate::data::source::source_arc(
        crate::layout::interior::SelectedAnnotationPlan {
            guides,
            panel_layer,
            panel_clip,
            text,
            geometry,
            explanation,
        },
    )?);
    Ok(result)
}

fn validate_frozen_axes_and_guides(
    registered: Option<&registration::PlotRegistration<'_>>,
    result: &mut PlotRenderResult,
) -> Result<(), crate::layout::fit::PlanError> {
    if result.decoration_mode != crate::spec::DecorationMode::SelfManaged && result.axes.is_none() {
        return Err(Error::Render("native panel is missing its frozen axes".into()).into());
    }
    if result.discrete_guide != registration::guide_descriptor(registered)? {
        return Err(
            Error::Render("selected producer changed its prepared discrete guide".into()).into(),
        );
    }
    Ok(())
}

#[cfg(test)]
fn validate_axis_text(
    layout: &crate::layout::Layout,
    theme: &crate::text::TextRequestContext<'_, '_>,
    scope: &crate::layout::fit::FitScope,
) -> Result<(), crate::layout::fit::PlanError> {
    layout
        .measure_axes(theme, scope, layout.test_semantics())
        .map(|_| ())
}

#[cfg(test)]
#[test]
fn later_axis_font_failure_wins_over_an_earlier_text_fit() {
    use crate::layout::fit::{FitScope, PlanError};
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    theme.text.prepare_font_environment().unwrap();
    let mut layout =
        crate::layout::Layout::envelope(crate::layout::PixelRect::new(80.0, 610.0, 60.0, 450.0));
    layout.x_axis.label = Some("W".repeat(200).into());
    let scope = FitScope::root();
    assert!(matches!(
        validate_axis_text(&layout, &theme, &scope),
        Err(PlanError::Fit(_))
    ));
    layout.y_axis.label = Some("\u{10ffff}".into());
    assert!(matches!(
        validate_axis_text(&layout, &theme, &scope),
        Err(PlanError::Fatal(_))
    ));
}

#[cfg(test)]
#[path = "orchestrator/tests.rs"]
mod tests;
