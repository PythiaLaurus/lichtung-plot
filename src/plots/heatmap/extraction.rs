use super::*;
use crate::font::canonical::OperationContext;
use crate::{BooleanValues, CategoryValues, NumericValues};

fn source() -> Plot<crate::kind::Heatmap> {
    Plot::heatmap(
        CategoryValues::new("column", ["b", "a", "b", "a", "b"]).unwrap(),
        CategoryValues::new("row", ["top", "top", "middle", "middle", "bottom"]).unwrap(),
        NumericValues::nullable("value", [Some(-1.0), Some(2.0), Some(0.0), None, None]),
    )
    .heatmap_highlight(BooleanValues::new(
        "highlight",
        [true, false, false, false, false],
    ))
    .scale(HeatmapScale::Diverging { center: 0.0 })
    .annotations(HeatmapAnnotations::Values(
        crate::NumericLabelFormat::default(),
    ))
    .heatmap_null_row_annotation(
        crate::HeatmapNullRowAnnotation::new("bottom", "Unavailable row").unwrap(),
    )
    .size(800, 600)
}

#[test]
fn local_and_shared_heatmap_numeric_labels_are_unique_before_fonts() {
    use crate::decorations::legend::descriptor::ContinuousContributor;
    let source = Plot::heatmap(
        CategoryValues::new("column", ["a", "b"]).unwrap(),
        CategoryValues::new("row", ["r", "r"]).unwrap(),
        NumericValues::new("color", [0.1, 0.100_000_1]),
    )
    .erase();
    let theme = crate::Theme::Publication.resolve().unwrap();
    let resources = OperationContext::new();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let inputs = crate::plots::shared_x::prepare_plot(
        &source,
        &resources,
        Some(&crate::theme::scale_values::test_defaults()),
    )
    .unwrap();
    let marks = &inputs.heatmap.as_ref().unwrap().panels[0].1;
    let contributor = ContinuousContributor::for_plot(&source, &inputs, &theme)
        .unwrap()
        .unwrap();
    let shared = ContinuousContributor::merge([contributor])
        .unwrap()
        .unwrap();
    assert_ne!(marks.colorbar_values[0], marks.colorbar_values[1]);
    assert_eq!(shared.labels, marks.colorbar_values);
    assert_eq!(shared.domain, (0.1, 0.100_000_1));
    assert_eq!(shared.stops, marks.colorbar_stops);
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn legacy_signed_zero_diverging_centers_prepare_identical_neutral_colors_before_fonts() {
    for explicit in [None, Some((-2.0, 3.0))] {
        let mut previous = None;
        for center in [-0.0, 0.0] {
            let mut source = Plot::heatmap(
                CategoryValues::new("column", ["low", "neutral", "high"]).unwrap(),
                CategoryValues::new("row", ["one"; 3]).unwrap(),
                NumericValues::new("value", [-1.0, -0.0, 1.0]),
            )
            .scale(HeatmapScale::Diverging { center });
            if let Some((minimum, maximum)) = explicit {
                source = source.color_limits(minimum, maximum);
            }
            let source = source.erase();
            let resources = OperationContext::new();
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            let prepared = prepare(
                &source,
                source.data().with_operation(&resources),
                &crate::theme::scale_values::test_defaults(),
            )
            .unwrap();
            assert_eq!(prepared.cells.len(), 3);
            assert_eq!(prepared.cells[1].unwrap().to_bits(), (-0.0_f64).to_bits());
            assert_eq!(prepared.color.selection.center().unwrap().to_bits(), 0);
            assert_eq!(prepared.color.domain().center.unwrap().to_bits(), 0);
            assert_eq!(prepared.fills[1], Some(OpaqueColor::rgb(247, 247, 247)));
            let domain = prepared.color.domain();
            let offset = crate::theme::ContinuousColors::normalized_fraction(
                (domain.minimum, domain.maximum),
                0.0,
            )
            .unwrap();
            assert!(prepared
                .colorbar_stops
                .iter()
                .any(|stop| stop.offset.to_bits() == offset.to_bits()
                    && stop.color == (247, 247, 247)));
            if let Some(previous) = previous {
                assert_eq!(prepared, previous);
            }
            previous = Some(prepared);
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
        }
    }
}

#[test]
fn signed_zero_local_heatmap_normalizes_its_exact_domain_and_keeps_one_numeric_label() {
    let source = Plot::heatmap(
        CategoryValues::new("column", ["a", "b"]).unwrap(),
        CategoryValues::new("row", ["r", "r"]).unwrap(),
        NumericValues::new("color", [-0.0, 0.0]),
    )
    .color_limits(-0.0, 0.0)
    .erase();
    let resources = OperationContext::new();
    let inputs = crate::plots::shared_x::prepare_plot(
        &source,
        &resources,
        Some(&crate::theme::scale_values::test_defaults()),
    )
    .unwrap();
    let domain = inputs.heatmap.as_ref().unwrap().panels[0].1.color.domain();
    assert_eq!(domain.minimum.to_bits(), 0.0_f64.to_bits());
    assert_eq!(domain.maximum.to_bits(), 0.0_f64.to_bits());
    let plot = inputs.attach(source).unwrap();
    let theme = crate::Theme::Publication.resolve().unwrap();
    let selected = plot
        .measure_selected_interior(PlotRenderContext::new(
            plot.data(),
            PanelBox::new(20.0, 700.0, 20.0, 500.0),
            &theme,
        ))
        .unwrap();
    assert_eq!(
        selected
            .producer_text
            .iter()
            .filter(|text| text.plan.source == "0")
            .count(),
        1
    );
}

#[test]
fn heatmap_hidden_y_labels_use_the_same_cached_annotation_and_ink_summary() {
    let source = Plot::heatmap(
        CategoryValues::new("x", ["a", "b"]).unwrap(),
        CategoryValues::new("y", ["r", "s"]).unwrap(),
        NumericValues::new("value", [1.0, 2.0]),
    )
    .labels(crate::Labels::raw().alias("r", "One\nTwo\nThree\nFour\nFive"))
    .x_label("")
    .y_label("")
    .erase();
    let inputs = crate::plots::shared_x::prepare_plot(
        &source,
        &OperationContext::new(),
        Some(&crate::theme::scale_values::test_defaults()),
    )
    .unwrap();
    let mut chart = inputs.attach(source).unwrap();
    let theme = crate::Theme::Publication.resolve().unwrap();
    let visible = intrinsic_body_minimum(&chart, &theme).unwrap();
    chart = chart.with_y_tick_labels_visible(false);
    let hidden = intrinsic_body_minimum(&chart, &theme).unwrap();
    assert!(hidden.1 < visible.1);
    assert_eq!(hidden.0.to_bits(), visible.0.to_bits());
    chart = chart.with_y_tick_labels_visible(true);
    assert_eq!(intrinsic_body_minimum(&chart, &theme).unwrap(), visible);
}

#[test]
fn semantic_grids_text_and_styles_are_retained_before_fonts() {
    let source = source().erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let resources = OperationContext::new();
    let inputs = crate::plots::shared_x::prepare_plot(
        &source,
        &resources,
        Some(&crate::theme::scale_values::test_defaults()),
    )
    .unwrap();
    let marks = &inputs.heatmap.as_ref().unwrap().panels[0].1;
    assert_eq!(marks.x_levels, ["b", "a"]);
    assert_eq!(marks.y_levels, ["top", "middle", "bottom"]);
    assert_eq!(
        marks.cells,
        [Some(-1.0), Some(2.0), Some(0.0), None, None, None]
    );
    assert_eq!(marks.highlighted, [true, false, false, false, false, false]);
    assert_eq!(
        marks.null_row_annotations[2].as_deref(),
        Some("Unavailable row")
    );
    assert_eq!(marks.value_labels[0].as_deref(), Some("-1.00"));
    assert_eq!(marks.colorbar_values, ["-2", "2", "0", "Missing"]);
    assert!(marks.has_missing);

    crate::plots::shared_x::prepare_plot(
        &source,
        &OperationContext::new(),
        Some(&crate::theme::scale_values::test_defaults()),
    )
    .unwrap();
    let attached = inputs.attach(source).unwrap();
    assert!(std::sync::Arc::ptr_eq(
        attached.prepared_heatmap.as_ref().unwrap(),
        inputs.heatmap.as_ref().unwrap()
    ));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn borrowed_canvas_category_policy_keeps_raw_keys() {
    let source = Plot::heatmap(
        CategoryValues::new("x_axis", [" a_b ", "a b", "a_b", "a b"]).unwrap(),
        CategoryValues::new("y_axis", ["c_d", "c_d", "c d", "c d"]).unwrap(),
        NumericValues::new("color_value", [1.0, 2.0, 3.0, 4.0]),
    )
    .erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    for (policy, expected) in [
        (crate::Labels::raw(), [["a_b", "a b"], ["c_d", "c d"]]),
        (
            crate::Labels::humanized()
                .alias("a_b", "First X")
                .alias("a b", "Second X")
                .alias("c_d", "First Y")
                .alias("c d", "Second Y"),
            [["First X", "Second X"], ["First Y", "Second Y"]],
        ),
    ] {
        let prepare = |resources: &OperationContext| {
            let marks = crate::plots::shared_x::PreparedReference::new(
                &source,
                source.data().with_operation(resources),
                Some(&crate::theme::scale_values::test_defaults()),
            )?;
            crate::plots::shared_x::prepare_reference_plot(
                &source,
                marks,
                Some("panel-0"),
                None,
                Some(&policy),
            )
        };
        let resources = OperationContext::new();
        let inputs = prepare(&resources).unwrap();
        let axes = &inputs.axes[0].1;
        for (side, raw) in [["a_b", "a b"], ["c_d", "c d"]].iter().enumerate() {
            assert_eq!(
                axes.display_categories[side].as_ref().unwrap(),
                &expected[side]
            );
            assert_eq!(
                axes.categories[side]
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|tick| tick.label.as_str())
                    .collect::<Vec<_>>(),
                raw
            );
        }

        prepare(&OperationContext::new()).unwrap();
    }
    assert!(crate::plots::shared_x::prepare_plot(
        &source,
        &OperationContext::new(),
        Some(&crate::theme::scale_values::test_defaults())
    )
    .err()
    .expect("the source policy is unchanged")
    .to_string()
    .contains("label collision"));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn root_prepares_standalone_facet_and_canvas_components_once() {
    crate::orchestrator::measurements::start();
    STAGES.with(|counts| counts.set([0; 3]));
    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|count| count.set(0));
    source().preflight().unwrap();
    let work = crate::orchestrator::measurements::finish();
    let intrinsic = work
        .intrinsics
        .iter()
        .filter(|event| {
            event.counter == crate::orchestrator::measurements::Counter::HeatmapIntrinsic
        })
        .collect::<Vec<_>>();
    assert_eq!(intrinsic.len(), 1);
    assert!(intrinsic[0].root_round.is_none());
    assert_eq!(
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
        0
    );
    assert_eq!(STAGES.with(std::cell::Cell::get)[0], 1);
    assert_eq!(STAGES.with(std::cell::Cell::get)[2], 1);
    for scales in [
        crate::FacetScales::Fixed,
        crate::FacetScales::FreeX,
        crate::FacetScales::FreeY,
        crate::FacetScales::Free,
    ] {
        STAGES.with(|counts| counts.set([0; 3]));
        Plot::heatmap(
            CategoryValues::new("column", ["b", "a", "b", "a"]).unwrap(),
            CategoryValues::new("row", ["top", "top", "top", "top"]).unwrap(),
            NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]),
        )
        .facet(
            crate::FacetSpec::column(
                CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap(),
            )
            .scales(scales),
        )
        .size(1200, 640)
        .preflight()
        .unwrap();
        assert_eq!(STAGES.with(std::cell::Cell::get)[0], 2);
        assert_eq!(STAGES.with(std::cell::Cell::get)[2], 2);
    }
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
    assert_eq!(STAGES.with(std::cell::Cell::get)[0], 2);
    assert_eq!(STAGES.with(std::cell::Cell::get)[2], 2);
}

#[test]
fn cell_label_minimum_precedes_constrained_heatmap_discovery_and_is_retained() {
    let source = source()
        .annotations(HeatmapAnnotations::Values(
            crate::NumericLabelFormat::default()
                .suffix(" long unit label")
                .unwrap(),
        ))
        .erase();
    let inputs = crate::plots::shared_x::prepare_plot(
        &source,
        &OperationContext::new(),
        Some(&crate::theme::scale_values::test_defaults()),
    )
    .unwrap();
    let theme = crate::Theme::Publication.resolve().unwrap();
    inputs.prepare_intrinsics(&source, &theme).unwrap();
    let source = inputs.attach(source).unwrap();
    let minimum = intrinsic_body_minimum(&source, &theme).unwrap();
    assert!(minimum.0 > 80.0);

    assert_eq!(intrinsic_body_minimum(&source, &theme).unwrap(), minimum);

    STAGES.with(|counts| counts.set([0; 3]));
    let root = crate::layout::plan_exact_root(
        800,
        600,
        |_| Ok(()),
        |(), assigned, round| {
            let outcome = crate::decorations::plan::plan_panel(
                &source,
                source.kind(),
                PanelBox::new(0.0, minimum.0 - 1.0 / 1024.0, 0.0, 400.0),
                crate::decorations::plan::LegendRole::Suppressed,
                &theme,
                round,
                &crate::layout::ComponentAddress::root(),
            )?;
            assert!(matches!(outcome, crate::layout::fit::FitOutcome::Miss(_)));
            Ok(crate::layout::MeasuredLayout::new(assigned, ()))
        },
        |(), _, (), _| Ok(()),
    );
    root.unwrap();
    assert_eq!(STAGES.with(std::cell::Cell::get), [0; 3]);
}

#[test]
fn selected_payload_geometry_and_poisoned_capture_preserve_every_operand() {
    for policy in [HeatmapLabelFit::Wrap, HeatmapLabelFit::Elide] {
        let source = source().heatmap_label_fit(policy).erase();
        let inputs = crate::plots::shared_x::prepare_plot(
            &source,
            &OperationContext::new(),
            Some(&crate::theme::scale_values::test_defaults()),
        )
        .unwrap();
        let source = inputs.attach(source).unwrap();
        let theme = crate::Theme::Publication.resolve().unwrap();
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
        let crate::layout::interior::MarkPayload::Heatmap(payload) = &marks.payload else {
            panic!("wrong payload")
        };
        assert_eq!(payload.cells.len(), 5);
        assert!(payload.cells[3].cross.is_some());
        assert!(payload.cells[4].text.is_some());
        let stops = payload
            .colorbar
            .as_ref()
            .unwrap()
            .calls
            .iter()
            .find_map(|call| match call {
                crate::decorations::legend::LegendCall::Gradient(_, _, _, stops) => Some(stops),
                _ => None,
            })
            .unwrap();
        assert_eq!(stops.len(), 4);
        assert_eq!(stops[1].offset.to_bits(), 0.25_f64.to_bits());
        assert_eq!(stops[2].color, (247, 247, 247));
        assert!(marks
            .geometry
            .bounds()
            .iter()
            .any(|ink| ink.left > payload.clip.x1));
        let poison = Plot::heatmap(
            CategoryValues::new("poison", ["x"]).unwrap(),
            CategoryValues::new("poison", ["y"]).unwrap(),
            NumericValues::new("poison", [f64::NAN]),
        )
        .erase();
        let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
        let projections = crate::layout::Projection::test_projection_calls();
        let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
        let stages = STAGES.with(std::cell::Cell::get);
        crate::orchestrator::measurements::start();
        let mut backend = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
        backend.begin(800, 600).unwrap();
        poison
            .render_panel(
                crate::PlotKind::Heatmap,
                PlotRenderContext::new(poison.data(), payload.clip, &theme)
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
            crate::orchestrator::measurements::Counter::HeatmapPrefixTrial,
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
        assert_eq!(stages[..2], STAGES.with(std::cell::Cell::get)[..2]);
    }
}

#[test]
fn selected_cells_guides_and_unique_labels_fit_the_assigned_panel() {
    let source = source().erase();
    let inputs = crate::plots::shared_x::prepare_plot(
        &source,
        &OperationContext::new(),
        Some(&crate::theme::scale_values::test_defaults()),
    )
    .unwrap();
    let source = inputs.attach(source).unwrap();
    let theme = crate::Theme::Publication.resolve().unwrap();
    let selected = source
        .measure_selected_interior(PlotRenderContext::new(
            source.data(),
            PanelBox::new(70.0, 670.0, 60.0, 450.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Heatmap(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("wrong payload")
    };
    let columns = [
        payload.cells[0].corners[0].x,
        payload.cells[0].corners[1].x,
        payload.cells[1].corners[1].x,
    ];
    let rows = [
        payload.cells[0].corners[2].y,
        payload.cells[0].corners[0].y,
        payload.cells[2].corners[0].y,
        payload.cells[4].corners[0].y,
    ];
    let marks = retained(&source).unwrap();
    let text = PlotRenderContext::new(
        source.data(),
        selected.layout.projection.panel_box(),
        &theme,
    )
    .producer_text()
    .unwrap();
    let annotations = measure_annotations(
        &source,
        marks,
        &columns,
        &theme,
        &crate::layout::fit::FitScope::root(),
        &text,
    )
    .unwrap();
    let labels = [
        "Long category alpha".to_owned(),
        "Distinct category beta".to_owned(),
    ];
    let (_, guide) = guide::measure(
        marks,
        selected.layout.projection.panel_box(),
        1.0 / 1024.0,
        &theme,
        &crate::layout::fit::FitScope::root(),
        None,
    )
    .unwrap();
    for component in 0..3 {
        let run = || -> Result<(), Error> {
            if component == 0 {
                fitted_unique_labels(
                    &theme,
                    &labels,
                    |_| 55.0,
                    "x",
                    &crate::layout::fit::FitScope::root(),
                )
                .map_err(crate::layout::fit::PlanError::into_error)?;
            } else if component == 1 {
                select_cells(marks, &columns, &rows, &theme, &annotations)?;
            } else {
                guide::select(&guide, selected.layout.projection.panel_box(), &theme)?;
            }
            Ok(())
        };
        run().unwrap();
    }
}

#[test]
fn shared_continuous_ranges_and_nested_facets_stay_rejected_before_fonts() {
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let error = crate::Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, source())
        .unwrap()
        .panel(0, 1, source())
        .unwrap()
        .x_range_group([(0, 0), (0, 1)])
        .unwrap()
        .preflight()
        .unwrap_err();
    assert!(error.to_string().contains("categorical x axis"));
    let nested = source().facet(crate::FacetSpec::column(
        CategoryValues::new("facet", ["one", "one", "one", "one", "one"]).unwrap(),
    ));
    let canvas = crate::Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, nested)
        .unwrap();
    assert!(canvas
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("faceted chart cannot be nested"));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}
