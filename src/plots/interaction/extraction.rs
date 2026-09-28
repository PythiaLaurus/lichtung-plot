use super::*;
use crate::font::canonical::OperationGenerationResources;
use crate::{Bounds, CategoryValues, NumericValues};

pub(crate) fn source() -> Plot<crate::kind::Interaction> {
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
fn sparse_trace_points_claim_category_pitch_and_complete_numeric_ink() {
    let source = Plot::interaction(
        CategoryValues::new("condition", ["b", "a", "c", "b", "c"]).unwrap(),
        NumericValues::new("estimate", [1.0, 2.0, 3.0, 4.0, 5.0]),
        CategoryValues::new("group", ["one", "one", "one", "two", "two"]).unwrap(),
        Bounds::new(
            NumericValues::new("lower", [0.0, 1.0, 2.0, 3.0, 4.0]),
            NumericValues::new("upper", [2.0, 3.0, 4.0, 5.0, 6.0]),
        ),
    )
    .size(1400, 900)
    .erase();
    let mut theme = crate::Theme::Publication.resolve();
    theme.marks.point_sizes.interaction = 160;
    let resources = OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let hard = chart.intrinsic_body_minimum(&theme).unwrap();
    assert!(hard.0 > 1000.0);
    let cell = crate::layout::PanelBox::new(80.0, 1300.0, 80.0, 780.0);
    let selected = crate::decorations::plan::plan_panel_for_test(
        &chart,
        crate::PlotKind::Interaction,
        cell,
        crate::decorations::plan::LegendRole::Suppressed,
        &theme,
    )
    .unwrap();
    assert!(selected.planned_y_limits().is_some());
    let crate::layout::interior::MarkPayload::Interaction(payload) =
        &selected.result.marks.as_ref().unwrap().payload
    else {
        panic!("selected Interaction")
    };
    assert!(payload.domain_expansions.iter().all(Option::is_none));
    let panel = selected.result.layout.projection.panel_box();
    let mut count = 0;
    for trace in &payload.traces {
        count += trace.points.len();
        let Some(style) = &trace.marker else { continue };
        let ink = crate::render::geometry::point_ink_bounds(&theme, style)
            .unwrap()
            .unwrap();
        for point in &trace.points {
            assert!(point.x + ink.left >= panel.x0 && point.x + ink.right <= panel.x1);
            assert!(point.y + ink.bottom >= panel.y0 && point.y + ink.top <= panel.y1);
        }
    }
    assert_eq!(count, 5);
}

#[test]
fn custom_connector_appearance_keeps_exact_group_patterns_and_guide_identity() {
    let source = source()
        .styles([
            crate::InteractionStyle::new("two").pattern(LinePattern::Dashed),
            crate::InteractionStyle::new("one"),
        ])
        .y_limits(-2.0, 10.0)
        .erase();
    let resources = OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let panel = crate::layout::PanelBox::new(80.0, 550.0, 70.0, 410.0);
    let mut theme = crate::Theme::Publication.resolve();
    let old = chart
        .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
        .unwrap();
    theme.marks.interaction_connector.alpha = 91;
    theme.marks.interaction_connector.width = 2.5;
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
        .unwrap();
    let styles = |result: &PlotRenderResult| {
        let crate::layout::interior::MarkPayload::Interaction(payload) =
            &result.marks.as_ref().unwrap().payload
        else {
            panic!("selected Interaction")
        };
        payload
            .traces
            .iter()
            .map(|trace| (trace.points.clone(), trace.connector.clone().unwrap()))
            .collect::<Vec<_>>()
    };
    let original = styles(&old);
    let changed = styles(&selected);
    assert!(original
        .iter()
        .zip(&changed)
        .all(|((old_points, _), (new_points, _))| old_points == new_points));
    assert!(changed
        .iter()
        .all(|(_, line)| line.color.alpha() == 91 && line.width.to_bits() == 2.5_f32.to_bits()));
    assert!(changed
        .iter()
        .any(|(_, line)| line.pattern == crate::render::backend::StrokePattern::LongDash));
    assert!(changed
        .iter()
        .any(|(_, line)| line.pattern == crate::render::backend::StrokePattern::Solid));
    assert!(crate::decorations::legend::merge_exact_entries(
        [
            old.legend_entries[0].clone(),
            selected.legend_entries[0].clone()
        ],
        "custom interaction connector",
        &OperationGenerationResources::new(),
    )
    .unwrap_err()
    .to_string()
    .contains("glyphs must match exactly"));
}

#[test]
fn same_assignment_marks_match_canonical_projection_fingerprints() {
    use crate::layout::PanelBox;
    // 2026-09-14 T5a6d: canonical projected centers precede marker construction.
    // Historical a6f9091 hashes were fac25eefcd84c63d / 3ef60d2fc29c86ec /
    // bae40bd86df8a981. T5c1 point-alpha quantization later changed the two
    // marker-visible SVG byte hashes (formerly 66849c3c47bac743 and
    // 178ec0a9046a6eb6); the marker-hidden fingerprint stayed identical.
    for (markers, connectors, expected) in [
        (true, true, 0x44fc_6427_2cd5_00e1),
        (true, false, 0x6e44_598a_586b_59bc),
        (false, true, 0xbae4_0bd8_6df8_a981),
    ] {
        let source = source()
            .hue(CategoryValues::new("color", ["red", "red", "red", "blue", "blue"]).unwrap())
            .styles([
                crate::InteractionStyle::new("two")
                    .shape(PointShape::TriangleUp)
                    .pattern(LinePattern::Dashed),
                crate::InteractionStyle::new("one"),
            ])
            .markers(markers)
            .connectors(connectors)
            .y_limits(-2.0, 10.0)
            .erase();
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for preset in [
            crate::Theme::Publication,
            crate::Theme::Nature,
            crate::Theme::Science,
        ] {
            let theme = preset.resolve();
            let plot =
                crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                    .unwrap()
                    .attach(source.clone())
                    .unwrap();
            for panel in [
                PanelBox::new(80.25, 550.75, 70.5, 410.25),
                PanelBox::new(30.0, 810.0, 50.0, 600.0),
            ] {
                let result = plot
                    .measure_selected_interior(PlotRenderContext::new(plot.data(), panel, &theme))
                    .unwrap();
                let mut svg = crate::render::svg_backend::SvgBackend::in_memory(900, 700);
                svg.begin(900, 700).unwrap();
                result.marks.as_ref().unwrap().draw(&mut svg).unwrap();
                svg.end().unwrap();
                for byte in svg.into_bytes().unwrap() {
                    hash = (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3);
                }
            }
        }
        assert_eq!(hash, expected, "markers={markers}, connectors={connectors}");
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one sparse fixture compares marks, axes, grid and unchanged caller labels"
)]
fn sparse_marks_ticks_and_grid_share_odd_quantum_coordinates() {
    let source = source()
        .y_limits(-2.0, 10.0)
        .y_ticks(vec![crate::AxisTick::new(2.0, " raw_estimate ")])
        .labels(crate::Labels::raw().alias("b", " raw_b "))
        .erase();
    let plot = crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
        .unwrap()
        .attach(source)
        .unwrap();
    let theme = crate::Theme::Publication.resolve();
    let panel = crate::layout::PanelBox::new(
        80.0 + 1.0 / 1024.0,
        550.0 + 4.0 / 1024.0,
        70.0 + 1.0 / 1024.0,
        410.0 + 4.0 / 1024.0,
    );
    let selected = plot
        .measure_selected_interior(PlotRenderContext::new(plot.data(), panel, &theme))
        .unwrap();
    assert_eq!(selected.x_tick_keys.as_ref().unwrap(), &["b", "a", "c"]);
    assert_eq!(
        selected
            .layout
            .x_axis
            .ticks
            .iter()
            .map(|tick| tick.label.as_str())
            .collect::<Vec<_>>(),
        [" raw_b ", "a", "c"]
    );
    assert_eq!(selected.layout.y_axis.ticks[0].label, " raw_estimate ");
    let crate::layout::interior::MarkPayload::Interaction(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        unreachable!()
    };
    assert_eq!(payload.clip, panel);
    let point = |condition: u32, value| {
        let (x, y) = selected
            .layout
            .projection
            .project(f64::from(condition) + 0.5, value);
        Point2D::new(x, y)
    };
    for (trace, cells) in payload.traces.iter().zip([
        &[(0, 2.0), (1, 3.0), (2, 4.0)][..],
        &[(0, 5.0), (2, 6.0)][..],
    ]) {
        assert_eq!(
            trace.points,
            cells
                .iter()
                .map(|&(condition, value)| point(condition, value))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            trace.intervals,
            cells
                .iter()
                .map(|&(condition, value)| [
                    point(condition, value - 1.0),
                    point(condition, value + 1.0)
                ])
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(payload.traces.len(), 2);
    // Exact (2*i+1)/(2*3) category centers. The middle is a whole-position
    // nearest-even tie; a numeric u64 key would incorrectly put it one quantum higher.
    assert_eq!(
        payload.traces[0]
            .points
            .iter()
            .map(|point| point.x.to_bits())
            .collect::<Vec<_>>(),
        [162_135, 322_562, 482_990].map(|quanta| (f64::from(quanta) / 1024.0).to_bits())
    );
    let mark = payload.traces[0].points[0];
    for grid in [false, true] {
        let mut svg = crate::render::svg_backend::SvgBackend::in_memory(900, 700);
        svg.begin(900, 700).unwrap();
        let axes = selected.axes.as_ref().unwrap();
        if grid {
            axes.draw_grid(&mut svg).unwrap();
        } else {
            axes.draw(&mut svg).unwrap();
        }
        svg.end().unwrap();
        let svg = String::from_utf8(svg.into_bytes().unwrap()).unwrap();
        let document = roxmltree::Document::parse(&svg).unwrap();
        let lines = document
            .descendants()
            .filter(|node| node.has_tag_name("polyline"))
            .map(|node| {
                node.attribute("points")
                    .unwrap()
                    .split([',', ' '])
                    .filter(|word| !word.is_empty())
                    .map(|word| word.parse::<f64>().unwrap())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        for mark in &payload.traces[0].points {
            assert!(lines
                .iter()
                .any(|line| line[0].to_bits() == mark.x.to_bits()
                    && line[2].to_bits() == mark.x.to_bits()));
        }
        let svg_y = 700.0 - mark.y;
        assert!(lines.iter().any(
            |line| line[1].to_bits() == svg_y.to_bits() && line[3].to_bits() == svg_y.to_bits()
        ));
        if !grid {
            for label in [" raw_b ", " raw_estimate "] {
                assert!(document
                    .descendants()
                    .any(|node| node.has_tag_name("text") && node.text() == Some(label)));
            }
        }
    }
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
        assert!(matches!(
            crate::plots::shared_x::prepare_plot(
                &source,
                &OperationGenerationResources::with_test_limits(visits, bytes)
            ),
            Err(Error::Render(_))
        ));
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
fn trace_style_completeness_admits_processing_before_missing_style() {
    let source = source().styles([]).erase();
    let levels = ["one", "two", "three"].map(String::from);
    for available in [0, 1, 2] {
        let resources = OperationGenerationResources::with_test_limits(available, 1024);
        let error = resolve_trace_styles(
            &source,
            &levels,
            source.data().with_axis_resources(&resources),
        )
        .unwrap_err();
        if available == 0 {
            assert!(matches!(error, Error::Render(message)
                if message.contains("axis source work admission")
                    && message.contains("root processing visits")));
        } else {
            assert!(matches!(error, Error::Data(message)
                if message == "interaction: graphical group \"one\" has no configured style"));
        }
        assert_eq!(resources.usage().unwrap().1, available.min(1));
    }
}

#[test]
fn trace_style_processing_stops_at_first_match_and_first_missing_style() {
    let source = source()
        .styles([crate::InteractionStyle::new("one")])
        .erase();
    let levels = ["one", "two", "three"].map(String::from);
    // One lookup comparison plus two completeness checks; neither scans the tail.
    for available in [0, 1, 2, 3, 4] {
        let resources = OperationGenerationResources::with_test_limits(available, 1024);
        let error = resolve_trace_styles(
            &source,
            &levels,
            source.data().with_axis_resources(&resources),
        )
        .unwrap_err();
        if available < 3 {
            assert!(matches!(error, Error::Render(message)
                if message.contains("axis source work admission")
                    && message.contains("root processing visits")));
        } else {
            assert!(matches!(error, Error::Data(message)
                if message == "interaction: graphical group \"two\" has no configured style"));
        }
        assert_eq!(resources.usage().unwrap().1, available.min(3));
    }
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
    use crate::orchestrator::measurements::{self, Counter};
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
    measurements::start();
    poison
        .render_panel(
            crate::PlotKind::Interaction,
            PlotRenderContext::new(
                poison.data(),
                selected.layout.projection.panel_box(),
                &theme,
            )
            .with_retained_body(&selected),
            &mut backend,
        )
        .unwrap();
    backend.end().unwrap();
    let work = measurements::finish();
    for counter in [
        Counter::OneCategoricalAxisMeasure,
        Counter::StandardPanelIntrinsic,
        Counter::LegacyPanelConstructor,
        Counter::AxisRangePreparation,
        Counter::AxisSpecPreparation,
        Counter::AxisSourcePreparation,
        Counter::TickSource,
        Counter::TickMetricLookup,
        Counter::TextProposalVisit,
        Counter::TextExactTrial,
    ] {
        assert_eq!(work.counters[counter as usize], 0, "{counter:?}");
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

#[test]
fn invalid_intervals_and_final_ranges_fail_before_fonts() {
    use crate::orchestrator::measurements::{self, Counter};
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
        measurements::start();
        assert!(matches!(
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new()),
            Err(Error::Data(_))
        ));
        let work = measurements::finish();
        for counter in [
            Counter::OneCategoricalAxisMeasure,
            Counter::StandardPanelIntrinsic,
            Counter::LegacyPanelConstructor,
        ] {
            assert_eq!(work.counters[counter as usize], 0);
        }
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}
