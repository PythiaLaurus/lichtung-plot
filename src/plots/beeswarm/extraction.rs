use super::*;
use crate::font::canonical::OperationGenerationResources;
use crate::{CategoryValues, NumericValues};

fn source() -> Plot<crate::kind::Beeswarm> {
    Plot::beeswarm(
        CategoryValues::new(
            "category",
            ["b", "a", "b", "a", "b", "b", "b", "b", "b", "b", "b", "b"],
        )
        .unwrap(),
        NumericValues::new("value", [0.0; 12]),
        NumericValues::new(
            "color_value",
            [0.5, 1.0, 0.0, -1.0, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
        ),
    )
    .color_limits(0.0, 1.0)
    .zero_line()
    .size(800, 600)
}

#[test]
fn root_semantics_admit_sort_bins_offsets_and_exact_colors_before_fonts() {
    let source = source().erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let resources = OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let marks = &inputs.beeswarm.as_ref().unwrap().panels[0].1;
    assert_eq!(marks.levels, ["b", "a"]);
    assert_eq!(
        marks.rows.iter().map(|row| row.3).collect::<Vec<_>>(),
        [2, 0, 4, 5, 6, 7, 8, 9, 10, 11, 3, 1]
    );
    let ys = [
        0.5, 0.41, 0.59, 0.32, 0.68, 0.23, 0.77, 0.14, 0.86, 0.14, 1.5, 1.41,
    ];
    for (point, expected) in marks.points.iter().zip(ys) {
        assert!((point.1 - expected).abs() < 1e-12);
    }
    assert_eq!(marks.points[0].2, (49, 130, 189));
    assert_eq!(marks.points[1].2, (157, 107, 124));
    assert_eq!(marks.points[10].2, (49, 130, 189));
    assert_eq!(marks.points[11].2, (222, 45, 38));
    assert_eq!(marks.values, ["1.000", "0.000"]);
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
        attached.prepared_beeswarm.as_ref().unwrap(),
        inputs.beeswarm.as_ref().unwrap()
    ));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn constant_beeswarm_guide_and_marks_use_the_same_oklab_midpoint() {
    let source = Plot::beeswarm(
        CategoryValues::new("category", ["one", "one"]).unwrap(),
        NumericValues::new("x", [0.0, 1.0]),
        NumericValues::new("fill", [3.0, 3.0]),
    )
    .erase();
    let resources = OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let marks = &inputs.beeswarm.as_ref().unwrap().panels[0].1;
    let color = marks.points[0].2;
    assert_eq!(color, (157, 107, 124));
    assert_eq!(marks.points[1].2, color);
    let stops = marks
        .guide_stops(source.data().with_axis_resources(&resources))
        .unwrap();
    assert_eq!(stops[0].color, color);
    assert_eq!(stops[1].color, color);
}

#[test]
fn root_components_prepare_once_without_layout_backend() {
    STAGES.with(|counts| counts.set([0; 3]));
    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|count| count.set(0));
    source().preflight().unwrap();
    assert_eq!(STAGES.with(std::cell::Cell::get)[0], 1);
    assert_eq!(STAGES.with(std::cell::Cell::get)[2], 1);
    assert_eq!(
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
        0
    );
    for scales in [
        crate::FacetScales::Fixed,
        crate::FacetScales::FreeX,
        crate::FacetScales::FreeY,
        crate::FacetScales::Free,
    ] {
        STAGES.with(|counts| counts.set([0; 3]));
        Plot::beeswarm(
            CategoryValues::new("category", ["b", "a", "b", "a"]).unwrap(),
            NumericValues::new("value", [0.0, 0.0, 0.001, 3.0]),
            NumericValues::new("color", [-1.0, 0.0, 1.0, 0.5]),
        )
        .facet(
            crate::FacetSpec::column(
                CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap(),
            )
            .scales(scales),
        )
        .size(1400, 600)
        .preflight()
        .unwrap();
        assert_eq!(STAGES.with(std::cell::Cell::get)[0], 2);
        assert_eq!(STAGES.with(std::cell::Cell::get)[2], 2);
    }
    STAGES.with(|counts| counts.set([0; 3]));
    crate::Canvas::grid(1, 2)
        .unwrap()
        .size(1600, 600)
        .panel(0, 0, source())
        .unwrap()
        .panel(0, 1, source())
        .unwrap()
        .x_range_group([(0, 0), (0, 1)])
        .unwrap()
        .preflight()
        .unwrap();
    assert_eq!(STAGES.with(std::cell::Cell::get)[0], 2);
    assert_eq!(STAGES.with(std::cell::Cell::get)[2], 2);
}

#[test]
fn selected_payload_preserves_bounds_baselines_and_poisoned_capture_does_no_work() {
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
        begins,
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get)
    );
    assert_eq!(
        reads,
        crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get)
    );
    let marks = selected.marks.as_ref().unwrap();
    let crate::layout::interior::MarkPayload::Beeswarm(payload) = &marks.payload else {
        panic!("wrong payload")
    };
    let mut drawn = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
    drawn.begin(800, 600).unwrap();
    marks.draw(&mut drawn).unwrap();
    drawn.end().unwrap();
    assert_eq!(marks.geometry.bounds(), drawn.bounds());
    assert_eq!(
        payload.colorbar.as_ref().unwrap().text.len(),
        selected.producer_text.len()
    );
    assert_eq!(payload.points.len(), 12);
    for (retained, selected) in payload
        .colorbar
        .as_ref()
        .unwrap()
        .text
        .iter()
        .zip(&selected.producer_text)
    {
        assert_eq!(retained.plan, selected.plan);
        assert_eq!(retained.placement, selected.placement);
    }
    let poison = Plot::beeswarm(
        CategoryValues::new("poison", ["wrong"]).unwrap(),
        NumericValues::new("poison", [f64::NAN]),
        NumericValues::new("poison", [f64::INFINITY]),
    )
    .hide_point_outline()
    .erase();
    let mut poisoned_theme = crate::Theme::Science.resolve();
    poisoned_theme.foreground = (0, 255, 0);
    poisoned_theme.detail_font_size = 0;
    let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
    let projections = crate::layout::Projection::test_projection_calls();
    let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
    let stages = STAGES.with(std::cell::Cell::get);
    crate::orchestrator::measurements::start();
    let mut backend = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
    backend.begin(800, 600).unwrap();
    poison
        .render_panel(
            crate::PlotKind::Beeswarm,
            PlotRenderContext::new(
                poison.data(),
                PanelBox::new(1.0, 2.0, 3.0, 4.0),
                &poisoned_theme,
            )
            .with_retained_body(&selected),
            &mut backend,
        )
        .unwrap();
    backend.end().unwrap();
    let work = crate::orchestrator::measurements::finish();
    for counter in [
        crate::orchestrator::measurements::Counter::TickSource,
        crate::orchestrator::measurements::Counter::TickDistinctList,
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
    assert_eq!(marks.geometry.bounds(), backend.bounds());
}

#[test]
fn missing_colors_and_unsupported_nesting_reject_before_fonts() {
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    for color in [None, Some(f64::NAN), Some(f64::INFINITY)] {
        let plot = Plot::beeswarm(
            CategoryValues::new("category", ["a"]).unwrap(),
            NumericValues::new("x", [1.0]),
            NumericValues::nullable("color", [color]),
        )
        .color_limits(0.0, 1.0);
        assert!(plot.preflight().is_err());
    }
    let nested = source().facet(crate::FacetSpec::column(
        CategoryValues::new("panel", ["one"; 12]).unwrap(),
    ));
    assert!(crate::Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, nested)
        .unwrap()
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("faceted chart cannot be nested"));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn selected_candidate_storage_refuses_at_exact_operation_limits() {
    let source = source().erase();
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
            .unwrap();
    let source = inputs.attach(source).unwrap();
    let run = |limits: Option<(u128, u64)>| {
        let theme = crate::Theme::Publication.resolve();
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        if let Some((visits, bytes)) = limits {
            resources.set_test_limits(visits, bytes);
        }
        let result = source.measure_selected_interior(PlotRenderContext::new(
            source.data(),
            PanelBox::new(80.0, 700.0, 60.0, 500.0),
            &theme,
        ));
        (resources, result)
    };
    let (resources, selected) = run(None);
    selected.unwrap();
    let visits = resources.usage().unwrap().1;
    let bytes = resources.byte_usage().unwrap();
    for limits in [
        (visits - 1, bytes),
        (visits, bytes - 1),
        (0, bytes),
        (visits, 0),
    ] {
        assert!(run(Some(limits)).1.is_err());
    }
    run(Some((visits, bytes))).1.unwrap();
}

#[test]
fn invalid_sources_win_over_unavailable_fonts_in_the_exact_root() {
    for valid in [false, true] {
        let source = Plot::beeswarm(
            CategoryValues::new("category", ["a"]).unwrap(),
            NumericValues::new("x", [1.0]),
            NumericValues::new("color", [if valid { 1.0 } else { f64::NAN }]),
        )
        .erase();
        let mut theme = crate::Theme::Publication.resolve();
        theme.typography.tick_label.stack = "T5a6a4d unavailable font 91f7".into();
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
        STAGES.with(|counts| counts.set([0; 3]));
        let result = crate::layout::plan_exact_root(
            640,
            480,
            |stage| {
                let inputs =
                    crate::plots::shared_x::prepare_plot(&source, &stage.generation_resources())?;
                let requirements = crate::layout::foundational_plot_requirements(
                    Some(&inputs),
                    &source,
                    source.kind(),
                    &theme,
                    &crate::layout::ComponentAddress::root(),
                    true,
                    true,
                    true,
                    Some(stage),
                )?;
                let mut presence = stage.reserve_scalar_presence_map()?;
                let mut census = crate::layout::PreparedTextCensus::materialize(
                    requirements,
                    Some(stage),
                    Some(&mut presence),
                )
                .map_err(|error| Error::Render(error.to_string()))?;
                theme.prepare_font_environment_from_census(&mut census, presence, stage)
            },
            |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |(), _, (), _| Ok(()),
        );
        assert!(result.is_err());
        assert_eq!(
            STAGES.with(std::cell::Cell::get),
            [1, 0, usize::from(valid)]
        );
        if valid {
            assert!(result.err().unwrap().to_string().contains("font"));
        } else {
            assert!(matches!(result, Err(Error::Data(_))));
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
        }
    }
}

#[test]
fn all_output_modes_reuse_the_selected_scene() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first.svg");
    let second = directory.path().join("second.svg");
    STAGES.with(|counts| counts.set([0; 3]));
    let output = source().svg_preserve_text(&first).unwrap();
    let stages = STAGES.with(std::cell::Cell::get);
    let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
    let projections = crate::layout::Projection::test_projection_calls();
    let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
    output
        .svg(directory.path().join("outline.svg"))
        .unwrap()
        .png_preview(directory.path().join("plot.png"))
        .unwrap()
        .svg_preserve_text(&second)
        .unwrap();
    assert_eq!(
        std::fs::read(first).unwrap(),
        std::fs::read(second).unwrap()
    );
    assert_eq!(stages, STAGES.with(std::cell::Cell::get));
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
}
