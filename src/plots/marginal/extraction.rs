use super::*;
use crate::font::canonical::OperationGenerationResources;
use crate::{CategoryValues, NumericValues};

fn source() -> Plot<crate::kind::Marginal> {
    Plot::marginal(
        NumericValues::new("x", [1.0, 1.2, 1.4, 3.0, 3.2, 3.4]),
        NumericValues::new("y", [2.0, 2.1, 2.3, 4.0, 4.2, 4.3]),
    )
    .hue(CategoryValues::new("hue", ["b", "b", "b", "a", "a", "a"]).unwrap())
    .size(800, 600)
}

#[test]
fn marginal_root_semantics_and_curves_have_exact_pre_font_admission() {
    let source = source().erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    STAGES.with(|counts| counts.set([0; 3]));
    let resources = OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    assert_eq!(STAGES.with(std::cell::Cell::get), [1, 1, 0]);
    let marks = inputs.marginal.as_ref().unwrap();
    assert_eq!(marks.ranges, inputs.axes[0].1.ranges);
    assert_eq!(marks.curves.x_grid.len(), 80);
    assert_eq!(marks.groups.len(), 2);
    let visits = resources.usage().unwrap().1;
    let bytes = resources.byte_usage().unwrap();
    for (visits, bytes) in [(visits - 1, bytes), (visits, bytes - 1)] {
        assert!(crate::plots::shared_x::prepare_plot(
            &source,
            &OperationGenerationResources::with_test_limits(visits, bytes)
        )
        .is_err());
    }
    crate::plots::shared_x::prepare_plot(
        &source,
        &OperationGenerationResources::with_test_limits(visits, bytes),
    )
    .unwrap();
    let attached = inputs.attach(source).unwrap();
    assert!(std::sync::Arc::ptr_eq(
        attached.prepared_marginal.as_ref().unwrap(),
        marks
    ));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn marginal_selected_geometry_and_poisoned_capture_share_operands() {
    let source = source().erase();
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
            .unwrap();
    let source = inputs.attach(source).unwrap();
    let theme = crate::Theme::Publication.resolve();
    let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
    let begins = crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get);
    let selected = source
        .measure_selected_interior(PlotRenderContext::new(
            source.data(),
            PanelBox::new(70.25, 670.75, 60.5, 450.25),
            &theme,
        ))
        .unwrap();
    assert_eq!(
        reads,
        crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get)
    );
    assert_eq!(
        begins,
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get)
    );
    let marks = selected.marks.as_ref().unwrap();
    let mut drawn = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
    drawn.begin(800, 600).unwrap();
    marks.draw(&mut drawn).unwrap();
    drawn.end().unwrap();
    assert_eq!(marks.geometry.bounds(), drawn.bounds());
    let crate::layout::interior::MarkPayload::Marginal(payload) = &marks.payload else {
        panic!("wrong payload")
    };
    assert_eq!(payload.clips[0], selected.layout.projection.panel_box());
    assert_eq!(payload.groups.len(), 2);
    assert_eq!(payload.groups[0].top.len(), 82);
    assert!(marks
        .geometry
        .bounds()
        .iter()
        .any(|ink| ink.left > payload.clips[0].x1));
    let poison = Plot::marginal(
        NumericValues::new("poison", [f64::NAN]),
        NumericValues::new("poison", [f64::INFINITY]),
    )
    .erase();
    let projections = crate::layout::Projection::test_projection_calls();
    let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
    let stages = STAGES.with(std::cell::Cell::get);
    crate::orchestrator::measurements::start();
    let mut backend = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
    backend.begin(800, 600).unwrap();
    poison
        .render_panel(
            crate::PlotKind::Marginal,
            PlotRenderContext::new(poison.data(), payload.clips[0], &theme)
                .with_retained_body(selected.retained_payload()),
            &mut backend,
        )
        .unwrap();
    backend.end().unwrap();
    let work = crate::orchestrator::measurements::finish();
    for counter in [
        crate::orchestrator::measurements::Counter::TickSource,
        crate::orchestrator::measurements::Counter::AxisRangePreparation,
        crate::orchestrator::measurements::Counter::TickMetricLookup,
    ] {
        assert_eq!(work.counters[counter as usize], 0);
    }
    assert_eq!(
        reads,
        crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get)
    );
    assert_eq!(
        projections,
        crate::layout::Projection::test_projection_calls()
    );
    assert_eq!(
        shapes,
        crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get)
    );
    assert_eq!(stages, STAGES.with(std::cell::Cell::get));
}

#[test]
fn marginal_invalid_semantics_fail_before_fonts_even_with_exact_limits() {
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    for values in [[1.0, 1.0], [f64::MAX, -f64::MAX]] {
        let plot = Plot::marginal(
            NumericValues::new("x", values),
            NumericValues::new("y", [1.0, 2.0]),
        )
        .x_limits(0.0, 3.0)
        .y_limits(0.0, 3.0);
        assert!(matches!(plot.preflight(), Err(Error::Data(_))));
    }
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn marginal_root_and_canvas_prepare_curves_once_per_component() {
    STAGES.with(|counts| counts.set([0; 3]));
    source().preflight().unwrap();
    assert_eq!(STAGES.with(std::cell::Cell::get)[..2], [1, 1]);
    STAGES.with(|counts| counts.set([0; 3]));
    crate::Canvas::grid(1, 2)
        .unwrap()
        .size(1600, 640)
        .panel(0, 0, source())
        .unwrap()
        .panel(0, 1, source())
        .unwrap()
        .preflight()
        .unwrap();
    assert_eq!(STAGES.with(std::cell::Cell::get)[..2], [2, 2]);
}

#[test]
fn marginal_render_grid_refusal_preserves_resource_errors_and_exact_limits() {
    let source = source().erase();
    let prepared = prepare(&source, source.data()).unwrap();
    let run = |resources: &OperationGenerationResources| {
        render_curves(
            &prepared.groups,
            prepared.ranges,
            source.data().with_axis_resources(resources),
        )
    };
    let resources = OperationGenerationResources::new();
    run(&resources).unwrap();
    let visits = resources.usage().unwrap().1;
    let bytes = resources.byte_usage().unwrap();
    for (visits, bytes) in [
        (visits - 1, bytes),
        (visits, bytes - 1),
        (0, bytes),
        (visits, 0),
    ] {
        assert!(matches!(
            run(&OperationGenerationResources::with_test_limits(
                visits, bytes
            )),
            Err(Error::Render(_))
        ));
    }
    run(&OperationGenerationResources::with_test_limits(
        visits, bytes,
    ))
    .unwrap();
}

#[test]
fn marginal_parent_shared_domain_is_applied_before_curve_preparation() {
    let source = source().erase();
    let resources = OperationGenerationResources::new();
    let marks = crate::plots::shared_x::PreparedReference::new(
        &source,
        source.data().with_axis_resources(&resources),
    )
    .unwrap();
    let inputs = crate::plots::shared_x::prepare_reference_plot(
        &source,
        &resources,
        marks,
        Some("panel-0"),
        Some((-5.0, 8.0)),
        None,
    )
    .unwrap();
    let marks = inputs.marginal.as_ref().unwrap();
    assert_eq!(marks.ranges[0], (-5.0, 8.0));
    assert_eq!(marks.ranges, inputs.axes[0].1.ranges);
    assert_eq!(marks.curves.x_grid.first(), Some(&-5.0));
    assert_eq!(marks.curves.x_grid.last(), Some(&8.0));
}

#[test]
fn marginal_parent_hue_order_controls_selected_colors_without_repreparation() {
    let source = source().erase();
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
            .unwrap();
    let source = inputs
        .attach(source)
        .unwrap()
        .with_global_hue_levels(vec!["a".into(), "b".into()]);
    let theme = crate::Theme::Publication.resolve();
    let stages = STAGES.with(std::cell::Cell::get);
    let selected = source
        .measure_selected_interior(PlotRenderContext::new(
            source.data(),
            PanelBox::new(70.0, 670.0, 60.0, 450.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Marginal(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("wrong payload")
    };
    assert_eq!(payload.groups[0].point_style.color, theme.colors[1]);
    assert_eq!(payload.groups[1].point_style.color, theme.colors[0]);
    assert_eq!(payload.groups[0].fill.color, theme.colors[1]);
    assert_eq!(stages[..2], STAGES.with(std::cell::Cell::get)[..2]);
}
