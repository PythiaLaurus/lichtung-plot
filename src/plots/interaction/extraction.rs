use super::*;
use crate::font::canonical::OperationGenerationResources;
use crate::{Bounds, CategoryValues, NumericValues};

fn source() -> Plot<crate::kind::Interaction> {
    Plot::interaction(
        CategoryValues::new("condition", ["b", "a", "c", "b", "c"]).unwrap(),
        NumericValues::new("estimate", [2.0, 3.0, 4.0, 5.0, 6.0]),
        CategoryValues::new("group", ["one", "one", "one", "two", "two"]).unwrap(),
        Bounds::new(
            NumericValues::new("lower", [1.0, 2.0, 3.0, 4.0, 5.0]),
            NumericValues::new("upper", [3.0, 4.0, 5.0, 6.0, 7.0]),
        ),
    )
}

#[test]
fn admitted_semantics_preserve_sparse_cells_and_prefont_refusal() {
    let source = source().erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
    let resources = OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let marks = &inputs.interaction.as_ref().unwrap().panels[0].1;
    assert_eq!(marks.conditions, ["b", "a", "c"]);
    assert_eq!(marks.range, (1.0, 7.0));
    assert_eq!(
        marks.traces[1]
            .cells
            .iter()
            .map(|cell| cell.condition)
            .collect::<Vec<_>>(),
        [0, 2]
    );
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
        attached.prepared_interaction.as_ref().unwrap(),
        inputs.interaction.as_ref().unwrap()
    ));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn root_prepares_each_component_once() {
    for scales in [
        crate::FacetScales::Fixed,
        crate::FacetScales::FreeX,
        crate::FacetScales::FreeY,
        crate::FacetScales::Free,
    ] {
        STAGES.with(|counts| counts.set([0; 3]));
        let outcome = source()
            .facet(
                crate::FacetSpec::column(
                    CategoryValues::new("panel", ["left", "left", "left", "right", "right"])
                        .unwrap(),
                )
                .scales(scales),
            )
            .size(1100, 640)
            .preflight();
        let counts = STAGES.with(std::cell::Cell::get);
        assert_eq!(counts[0], 2);
        if matches!(
            scales,
            crate::FacetScales::Fixed | crate::FacetScales::FreeY
        ) {
            assert!(outcome
                .unwrap_err()
                .to_string()
                .contains("identical categorical levels"));
            assert_eq!(counts[2], 0);
            continue;
        }
        outcome.unwrap();
        assert!(counts[1] >= 2);
        assert_eq!(counts[2], 2);
    }
    STAGES.with(|counts| counts.set([0; 3]));
    crate::Canvas::grid(1, 2)
        .unwrap()
        .size(1400, 640)
        .panel(0, 0, source())
        .unwrap()
        .panel(0, 1, source())
        .unwrap()
        .preflight()
        .unwrap();
    let counts = STAGES.with(std::cell::Cell::get);
    assert_eq!(counts[0], 2);
    assert_eq!(counts[2], 2);
}

#[test]
fn selected_geometry_and_capture_are_immutable() {
    let source = source()
        .hue(CategoryValues::new("color", ["red", "red", "red", "blue", "blue"]).unwrap())
        .erase();
    let theme = crate::Theme::Publication.resolve();
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
            .unwrap();
    let attached = inputs.attach(source).unwrap();
    let reads_before_measurement = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
    let geometry_begins =
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get);
    let selected = attached
        .measure_selected_interior(PlotRenderContext::new(
            attached.data(),
            crate::layout::PanelBox::new(20.25, 520.75, 30.5, 350.25),
            &theme,
        ))
        .unwrap();
    assert_eq!(
        reads_before_measurement,
        crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get)
    );
    assert_eq!(
        geometry_begins,
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get)
    );
    let marks = selected.marks.as_ref().unwrap();
    let mut drawn = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
    drawn.begin(640, 480).unwrap();
    marks.draw(&mut drawn).unwrap();
    drawn.end().unwrap();
    assert_eq!(marks.geometry.bounds(), drawn.bounds());
    let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
    let projections = crate::layout::Projection::test_projection_calls();
    let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
    let stages = STAGES.with(std::cell::Cell::get);
    let poison = Plot::interaction(
        CategoryValues::new("poison", ["x"]).unwrap(),
        NumericValues::new("poison", [f64::NAN]),
        CategoryValues::new("poison", ["x"]).unwrap(),
        Bounds::new(
            NumericValues::new("lo", [f64::NAN]),
            NumericValues::new("hi", [f64::NAN]),
        ),
    )
    .erase();
    let mut backend = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
    backend.begin(640, 480).unwrap();
    poison
        .render_panel(
            crate::PlotKind::Interaction,
            PlotRenderContext::new(
                poison.data(),
                selected.layout.projection.panel_box(),
                &theme,
            )
            .with_retained_body(selected.retained_payload()),
            &mut backend,
        )
        .unwrap();
    backend.end().unwrap();
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
    assert_eq!(stages[..2], STAGES.with(std::cell::Cell::get)[..2]);
}

#[test]
fn invalid_intervals_and_final_ranges_fail_before_fonts() {
    let invalid = Plot::interaction(
        CategoryValues::new("condition", ["a"]).unwrap(),
        NumericValues::new("estimate", [2.0]),
        CategoryValues::new("group", ["one"]).unwrap(),
        Bounds::new(
            NumericValues::new("lower", [3.0]),
            NumericValues::new("upper", [4.0]),
        ),
    )
    .erase();
    let guide = crate::GuideSpec::horizontal(
        20.0,
        crate::GuideStyle::new((90, 80, 70), 1.0, LinePattern::Solid).unwrap(),
    );
    for source in [
        invalid,
        source().y_limits(0.0, 10.0).guide(guide).erase(),
        source()
            .y_limits(0.0, 10.0)
            .y_ticks(vec![crate::AxisTick::new(11.0, "outside")])
            .erase(),
    ] {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
        assert!(crate::plots::shared_x::prepare_plot(
            &source,
            &OperationGenerationResources::new()
        )
        .is_err());
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}
