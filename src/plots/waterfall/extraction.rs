use super::*;
use crate::font::canonical::OperationContext;
use crate::{CategoryValues, NumericValues};

fn source() -> Plot<crate::kind::Waterfall> {
    Plot::waterfall(
        CategoryValues::new("step", ["gain", "loss", "gain", "loss"]).unwrap(),
        NumericValues::new("delta", [4.0, -2.0, -0.0, 1.0]),
        10.0,
    )
    .size(800, 600)
}

#[test]
fn clipped_base_keeps_the_visible_terminal_containment_failure_named() {
    let source = source().x_limits(11.0, 13.0).erase();
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
    let chart = inputs.attach(source).unwrap();
    let theme = crate::Theme::Publication.resolve().unwrap();
    let scope = crate::layout::fit::FitScope::root()
        .child(crate::layout::AddressRole::CanvasCell, 7)
        .unwrap();
    let error = chart
        .measure_selected_interior(
            PlotRenderContext::new(
                chart.data(),
                PanelBox::new(30.0, 630.0, 30.0, 430.0),
                &theme,
            )
            .with_fit_scope(&scope),
        )
        .unwrap_err();
    let crate::layout::fit::PlanError::Fit(failure) = error else {
        panic!("terminal shelf FIT")
    };
    assert_eq!(
        failure.reason.address,
        scope
            .child(crate::layout::AddressRole::WaterfallTerminalLabel, 0)
            .unwrap()
            .address
    );
    assert!(failure.message.contains("annotation shelf"));
}

#[test]
fn root_semantics_preserve_span_labels_and_exact_deltas_before_fonts() {
    let source = source().erase();
    let resources = OperationContext::new();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
    let marks = &inputs.waterfall.as_ref().unwrap().panels[0].1;
    assert_eq!(marks.labels, ["gain", "loss", "gain", "loss"]);
    assert_eq!(
        marks
            .spans
            .iter()
            .map(|span| (span.start, span.end, span.delta))
            .collect::<Vec<_>>(),
        [
            (10.0, 14.0, 4.0),
            (14.0, 12.0, -2.0),
            (12.0, 12.0, -0.0),
            (12.0, 13.0, 1.0)
        ]
    );
    assert_eq!(marks.spans[2].delta.to_bits(), (-0.0_f64).to_bits());
    assert_eq!(marks.range, (10.0, 14.0));
    assert_eq!(
        marks.totals,
        [(10.0, "base 10.000".into()), (13.0, "total 13.000".into())]
    );

    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn finite_cumulative_endpoints_do_not_require_a_finite_detached_delta_sum() {
    let plot = Plot::waterfall(
        CategoryValues::new("step", ["one", "two"]).unwrap(),
        NumericValues::new("delta", [f64::MAX, f64::MAX]),
        -f64::MAX,
    )
    .erase();
    let resources = OperationContext::new();
    let marks = prepare(&plot, plot.data().with_operation(&resources)).unwrap();
    assert_eq!(marks.spans[0].end.to_bits(), 0.0_f64.to_bits());
    assert_eq!(marks.spans[1].end.to_bits(), f64::MAX.to_bits());
}

#[test]
fn cumulative_spans_preserve_source_ids_and_reject_nonfinite_values() {
    let plot = source().erase();
    let rows = [1];
    let ids = [99];
    {
        let resources = OperationContext::new();

        let data = crate::data::PlotData::with_rows(&plot, &rows, &ids).with_operation(&resources);
        let result = prepare_spans(10.0, &[4.0], data);
        {
            let (spans, range) = result.unwrap();
            assert_eq!(
                spans,
                [WaterfallSpan {
                    source_row: 99,
                    start: 10.0,
                    end: 14.0,
                    delta: 4.0
                }]
            );
            assert_eq!(range, (10.0, 14.0));
            assert_eq!(spans[0].end.to_bits(), 14.0_f64.to_bits());
        }
    }
    let resources = OperationContext::new();
    let data = crate::data::PlotData::with_rows(&plot, &rows, &ids).with_operation(&resources);
    assert!(matches!(
        prepare_spans(10.0, &[f64::NAN], data),
        Err(Error::Data(_))
    ));
}

#[test]
fn original_source_ids_survive_preparation_selection_and_feedback() {
    let ids = [5, 20, 47, 99];
    let raw = source()
        .source_rows(crate::SourceRows::new(ids).unwrap())
        .erase();
    let resources = OperationContext::new();
    let inputs = crate::plots::shared_x::prepare_plot(&raw, &resources, None).unwrap();
    assert_eq!(
        inputs.waterfall.as_ref().unwrap().panels[0]
            .1
            .spans
            .iter()
            .map(|span| span.source_row)
            .collect::<Vec<_>>(),
        ids
    );
    let plot = inputs.attach(raw).unwrap();
    let theme = crate::Theme::Publication.resolve().unwrap();
    let selected = plot
        .measure_selected_interior(PlotRenderContext::with_source_rows(
            plot.data(),
            &ids,
            PanelBox::new(70.0, 670.0, 60.0, 450.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Waterfall(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("Waterfall source identities")
    };
    assert_eq!(payload.source_rows, ids);
    assert_eq!(payload.source_rows.len(), payload.bars.len());
}

#[test]
fn root_components_prepare_once_without_a_geometry_measurement_backend() {
    STAGES.with(|counts| counts.set([0; 2]));
    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|count| count.set(0));
    source().preflight().unwrap();
    assert_eq!(STAGES.with(std::cell::Cell::get)[0], 1);
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
        STAGES.with(|counts| counts.set([0; 2]));
        source()
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
    }
    STAGES.with(|counts| counts.set([0; 2]));
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
}

#[test]
fn selected_geometry_and_poisoned_capture_preserve_all_operands_without_work() {
    let plot = source().erase();
    let inputs =
        crate::plots::shared_x::prepare_plot(&plot, &OperationContext::new(), None).unwrap();
    let plot = inputs.attach(plot).unwrap();
    let theme = crate::Theme::Publication.resolve().unwrap();
    let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
    let selected = plot
        .measure_selected_interior(PlotRenderContext::new(
            plot.data(),
            PanelBox::new(70.25, 670.75, 60.5, 450.25),
            &theme,
        ))
        .unwrap();
    assert_eq!(
        reads,
        crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get)
    );
    let marks = selected.marks.as_ref().unwrap();
    let crate::layout::interior::MarkPayload::Waterfall(payload) = &marks.payload else {
        panic!("wrong payload")
    };
    assert_eq!(payload.bars.len(), 4);
    assert_eq!(payload.connectors.len(), 3);
    assert_eq!(payload.text.len(), 2);
    for (text, producer) in payload.text.iter().zip(&selected.producer_text) {
        assert_eq!(text.plan, producer.plan);
        assert_eq!(text.placement, producer.placement);
    }
    let poison = Plot::waterfall(
        CategoryValues::new("poison", ["wrong"]).unwrap(),
        NumericValues::new("poison", [f64::NAN]),
        f64::INFINITY,
    )
    .erase();
    let mut poisoned_theme = crate::Theme::Science.resolve().unwrap();
    poisoned_theme.marks.accents =
        crate::theme::AccentPalette::new([crate::theme::OpaqueColor::rgb(255, 0, 0)]).unwrap();
    poisoned_theme.detail_font_size = 0;
    let projections = crate::layout::Projection::test_projection_calls();
    let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
    let stages = STAGES.with(std::cell::Cell::get);
    crate::orchestrator::measurements::start();
    let mut backend = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
    backend.begin(800, 600).unwrap();
    poison
        .render_panel(
            crate::PlotKind::Waterfall,
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
    assert_eq!(marks.geometry.bounds(), backend.bounds());
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
fn missing_labels_and_invalid_intermediate_endpoints_win_before_unavailable_fonts() {
    for case in 0..3 {
        let plot = Plot::waterfall(
            CategoryValues::nullable(
                "step",
                [
                    Some("one"),
                    if case == 0 { None } else { Some("two") },
                    Some("three"),
                ],
            )
            .unwrap(),
            NumericValues::new(
                "delta",
                if case == 1 {
                    [f64::MAX, f64::MAX, -f64::MAX]
                } else {
                    [1.0, 2.0, 3.0]
                },
            ),
            0.0,
        )
        .source_rows(crate::SourceRows::new([19, 43, 71]).unwrap())
        .x_limits(-10.0, 10.0)
        .erase();
        let mut theme = crate::Theme::Publication.resolve().unwrap();
        theme.typography.axes.tick_label.stack =
            crate::font::canonical::named_stack(&["T5a6a4e unavailable font 91f7"]);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let result = crate::layout::plan_exact_root(
            640,
            480,
            |stage| {
                let inputs = crate::plots::shared_x::prepare_plot(&plot, &stage.operation(), None)?;
                let requirements = crate::layout::foundational_plot_requirements(
                    Some(&inputs),
                    &plot,
                    plot.kind(),
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
        let error = result.err().unwrap();
        if case < 2 {
            assert!(matches!(error, Error::Data(_)), "{error}");
            assert!(error.to_string().contains("source row 43"), "{error}");
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
        } else {
            assert!(error.to_string().contains("font"), "{error}");
        }
    }
}

#[test]
fn total_labels_keep_typed_fit_identity_and_mark_panel_reserve() {
    use crate::layout::fit::{FitKind, FitOutcome, FitScope};
    use crate::layout::{AddressRole, ComponentAddress};
    let owner = ComponentAddress::root()
        .child(AddressRole::CanvasCell, 2)
        .unwrap();
    let scope = FitScope::new(&owner, 7);
    for (labels, panel, kind, role) in [
        (
            crate::WaterfallLabels::new("W".repeat(200), "total").unwrap(),
            PanelBox::new(80.0, 500.0, 80.0, 350.0),
            FitKind::Text,
            Some(AddressRole::WaterfallBaseLabel),
        ),
        (
            crate::WaterfallLabels::new("base", "total").unwrap(),
            PanelBox::new(80.0, 500.0, 80.0, 110.0),
            FitKind::SpecializedGeometry,
            None,
        ),
    ] {
        let plot = source().total_labels(labels).erase();
        let inputs =
            crate::plots::shared_x::prepare_plot(&plot, &OperationContext::new(), None).unwrap();
        let plot = inputs.attach(plot).unwrap();
        let theme = crate::Theme::Publication.resolve().unwrap();
        let result = plot
            .preflight_measurement(crate::PlotKind::Waterfall, panel, &theme, &scope)
            .unwrap();
        let FitOutcome::Miss(failure) = result else {
            panic!("unexpected fit")
        };
        assert_eq!(failure.reason.kind, kind);
        assert_eq!(
            failure.reason.address,
            role.map_or_else(|| owner.clone(), |role| owner.child(role, 0).unwrap())
        );
        assert_eq!(failure.reason.ordinal, if role.is_some() { 0 } else { 7 });
    }
}

#[test]
fn output_replay_uses_one_selected_scene_and_nesting_restrictions_remain() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first.svg");
    let second = directory.path().join("second.svg");
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
    let nested = source().facet(crate::FacetSpec::column(
        CategoryValues::new("panel", ["one"; 4]).unwrap(),
    ));
    assert!(crate::Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, nested)
        .unwrap()
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("faceted chart cannot be nested"));
}
