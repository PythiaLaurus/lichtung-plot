use super::*;
use crate::font::canonical::OperationGenerationResources;
use crate::layout::fit::{FitKind, FitScope, PlanError};
use crate::layout::interior::{
    coordinate::Intrinsics, InteriorRoute, MarkPayload, OneCategoricalAxis,
};
use crate::layout::{AxisSide, PanelBox, Projection, TickSource};
use crate::orchestrator::measurements::{self, Counter};
use crate::{Bounds, CategoryValues, NumericValues};

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one native assignment checks axes and every retained trace operand and style"
)]
fn native_bottom_conditions_keep_parent_trace_order_palette_sparse_connectors_and_intervals() {
    let source = extraction::source()
        .hue(CategoryValues::new("color", ["red", "red", "red", "blue", "blue"]).unwrap())
        .styles([
            crate::InteractionStyle::new("two")
                .shape(PointShape::TriangleUp)
                .pattern(LinePattern::Dashed),
            crate::InteractionStyle::new("absent"),
            crate::InteractionStyle::new("one"),
        ])
        .labels(crate::Labels::raw().alias("b", " Follow-up visit "))
        .erase()
        .with_global_group_levels(vec!["absent".into(), "two".into(), "one".into()])
        .with_global_hue_levels(vec!["absent".into(), "blue".into(), "red".into()])
        .with_global_collective_pairs(vec![
            crate::data::CollectivePair {
                group: Some("absent".into()),
                hue: Some("absent".into()),
            },
            crate::data::CollectivePair {
                group: Some("two".into()),
                hue: Some("blue".into()),
            },
            crate::data::CollectivePair {
                group: Some("one".into()),
                hue: Some("red".into()),
            },
        ]);
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
            .unwrap();
    assert_eq!(
        inputs.route,
        InteriorRoute::OneCategoricalAxis(OneCategoricalAxis::Interaction)
    );
    let plot = inputs.attach(source).unwrap();
    let prepared = &plot.prepared_interaction.as_ref().unwrap().panels[0].1;
    assert_eq!(prepared.conditions, ["b", "a", "c"]);
    assert_eq!(prepared.range, (1.0, 7.0));
    assert_eq!(
        prepared
            .traces
            .iter()
            .map(|trace| trace.identity.group.as_deref())
            .collect::<Vec<_>>(),
        [Some("two"), Some("one")]
    );
    assert_eq!(
        prepared.traces[0]
            .cells
            .iter()
            .map(|cell| cell.condition)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    for preset in [
        crate::Theme::Publication,
        crate::Theme::Nature,
        crate::Theme::Science,
    ] {
        let theme = preset.resolve();
        let plot =
            crate::plots::shared_x::prepare_plot(&plot, &OperationGenerationResources::new())
                .unwrap()
                .attach(plot.clone())
                .unwrap();
        let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
        for panel in [
            PanelBox::new(80.25, 550.75, 70.5, 410.25),
            PanelBox::new(30.0, 810.0, 50.0, 600.0),
        ] {
            for range in [(-2.0, 10.0), (2.0, 5.0)] {
                let plot = plot
                    .clone()
                    .with_planned_x_limits((-10.0, 10.0))
                    .with_planned_y_limits(range);
                measurements::start();
                let selected = plot
                    .measure_selected_interior(PlotRenderContext::new(plot.data(), panel, &theme))
                    .unwrap();
                let work = measurements::finish();
                assert_eq!(
                    work.counters[Counter::OneCategoricalAxisMeasure as usize],
                    1
                );
                assert_eq!(work.counters[Counter::LegacyPanelConstructor as usize], 0);
                for counter in [Counter::AxisRangePreparation, Counter::AxisSpecPreparation] {
                    assert_eq!(work.counters[counter as usize], 0, "{counter:?}");
                }
                assert!(work.counters[Counter::AxisSourcePreparation as usize] > 0);
                assert_eq!(
                    (selected.x_domain, selected.y_domain),
                    (
                        crate::spec::AxisDomain::Categorical,
                        crate::spec::AxisDomain::Continuous
                    )
                );
                assert_eq!(selected.x_tick_keys.as_ref().unwrap(), &["b", "a", "c"]);
                assert!(selected.y_tick_keys.is_none());
                assert_eq!(selected.layout.x_axis.source, TickSource::SUPPLIED);
                assert_eq!(
                    selected
                        .layout
                        .x_axis
                        .ticks
                        .iter()
                        .map(|tick| (tick.position, tick.label.as_str()))
                        .collect::<Vec<_>>(),
                    [(0.5, " Follow-up visit "), (1.5, "a"), (2.5, "c")]
                );
                assert_eq!(
                    selected.axes.as_ref().unwrap().range(AxisSide::Bottom),
                    (0.0, 3.0)
                );
                assert_eq!(selected.axes.as_ref().unwrap().range(AxisSide::Left), range);
                assert_eq!(selected.layout.projection.panel_box(), panel);
                let MarkPayload::Interaction(payload) = &selected.marks.as_ref().unwrap().payload
                else {
                    panic!("interaction payload")
                };
                assert_eq!(payload.clip, panel);
                assert_eq!(payload.traces.len(), prepared.traces.len());
                let resources = OperationGenerationResources::new();
                let colors = prepared
                    .resolved_colors(&plot, &theme, plot.data().with_axis_resources(&resources))
                    .unwrap();
                for (actual, trace) in payload.traces.iter().zip(&prepared.traces) {
                    let key = trace
                        .identity
                        .hue
                        .as_deref()
                        .or(trace.identity.group.as_deref())
                        .unwrap();
                    let color = colors.color(key, &resources).unwrap();
                    assert_eq!(
                        actual.interval,
                        LineStyle::solid(crate::theme::color_from_rgb(color), 1.2)
                    );
                    assert_eq!(
                        actual.connector,
                        Some(trace.style.line_pattern.line_style(color, 1.8))
                    );
                    assert_eq!(
                        actual.marker,
                        Some(
                            plot.point_style(&theme, color, 4)
                                .with_shape(trace.style.point_shape)
                        )
                    );
                    let point = |condition, value| {
                        let (x, y) = selected
                            .layout
                            .projection
                            .project(f64::from(u32::try_from(condition).unwrap()) + 0.5, value);
                        Point2D::new(x, y)
                    };
                    assert_eq!(
                        actual.points,
                        trace
                            .cells
                            .iter()
                            .map(|cell| point(cell.condition, cell.estimate))
                            .collect::<Vec<_>>()
                    );
                    assert_eq!(
                        actual.intervals,
                        trace
                            .cells
                            .iter()
                            .map(|cell| [
                                point(cell.condition, cell.lower),
                                point(cell.condition, cell.upper)
                            ])
                            .collect::<Vec<_>>()
                    );
                }
                assert_eq!(selected.annotations.as_ref().unwrap().panel_clip, panel);
                assert_eq!(
                    crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                    reads
                );
            }
        }
    }
}

#[test]
fn native_condition_hard_intrinsics_and_exact_numeric_labels_reject_before_marks() {
    let word = "W".repeat(20);
    let source = Plot::interaction(
        CategoryValues::new("condition", [word.as_str(), " raw_A ", "last"]).unwrap(),
        NumericValues::new("estimate", [2.0, 3.0, 4.0]),
        CategoryValues::new("group", ["one", "one", "one"]).unwrap(),
        Bounds::new(
            NumericValues::new("lo", [1.0, 2.0, 3.0]),
            NumericValues::new("hi", [3.0, 4.0, 5.0]),
        ),
    )
    .labels(crate::Labels::raw())
    .y_limits(0.0, 6.0)
    .y_ticks(vec![
        crate::AxisTick::new(0.0, "raw_lower"),
        crate::AxisTick::new(3.0, ""),
        crate::AxisTick::new(6.0, "raw_upper"),
    ])
    .erase();
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
            .unwrap();
    let plot = inputs.attach(source).unwrap();
    let theme = crate::Theme::Publication.resolve();
    let minimum = Intrinsics::prepared(&plot, &theme).unwrap().minimum(&plot);
    let hard = crate::layout::fit::intrinsic_widths(
        &theme,
        &word,
        &theme.typography.tick_label,
        "condition intrinsic",
    )
    .unwrap()
    .0;
    assert_eq!(minimum.0.to_bits(), ((hard + 4.0) * 3.0).to_bits());
    assert!(minimum.0 > 80.0);
    let scope = FitScope::new(
        &crate::layout::ComponentAddress::root()
            .child(
                crate::layout::AddressRole::PlotBody,
                0,
                &crate::font::canonical::OperationGenerationResources::new(),
            )
            .unwrap(),
        7,
    );
    let projections = Projection::test_projection_calls();
    let stages = STAGES.with(std::cell::Cell::get);
    let error = plot
        .measure_selected_interior(
            PlotRenderContext::new(
                plot.data(),
                PanelBox::new(0.0, minimum.0 - 1.0 / 1024.0, 0.0, 300.0),
                &theme,
            )
            .with_fit_scope(&scope),
        )
        .unwrap_err();
    let PlanError::Fit(failure) = error else {
        panic!("typed condition intrinsic miss")
    };
    assert_eq!(failure.reason.kind, FitKind::TrackOrResidual);
    assert_eq!(failure.reason.address, scope.address);
    assert_eq!(failure.reason.ordinal, 7);
    assert_eq!(Projection::test_projection_calls(), projections);
    assert_eq!(STAGES.with(std::cell::Cell::get), stages);
    let selected = plot
        .measure_selected_interior(PlotRenderContext::new(
            plot.data(),
            PanelBox::new(0.0, minimum.0.ceil() + 1.0, 0.0, 300.0),
            &theme,
        ))
        .unwrap();
    assert_eq!(
        selected.x_tick_keys.as_ref().unwrap(),
        &[word, " raw_A ".into(), "last".into()]
    );
    assert_eq!(selected.layout.projection.y_range(), (0.0, 6.0));
    assert_eq!(selected.layout.y_axis.source, TickSource::EXACT);
    assert_eq!(
        selected.layout.y_axis.ticks,
        *plot.y_axis_spec().exact_ticks.as_ref().unwrap()
    );
}

#[test]
fn native_parent_feedback_freezes_actual_marks_axes_and_gutters_at_the_assigned_panel() {
    use crate::decorations::plan::{plan_panel_for_test, LegendRole};
    for fixed in [false, true] {
        let mut source = extraction::source().guide(crate::GuideSpec::horizontal(
            4.0,
            crate::GuideStyle::new((90, 80, 70), 1.0, LinePattern::Dashed).unwrap(),
        ));
        if fixed {
            source = source.y_limits(0.0, 8.0);
        }
        let source = source.size(900, 700).erase();
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let range = inputs.axes[0].1.ranges[1];
        let plot = inputs.attach(source).unwrap();
        for preset in [
            crate::Theme::Publication,
            crate::Theme::Nature,
            crate::Theme::Science,
        ] {
            let theme = preset.resolve();
            let plot =
                crate::plots::shared_x::prepare_plot(&plot, &OperationGenerationResources::new())
                    .unwrap()
                    .attach(plot.clone())
                    .unwrap();
            for cell in [
                PanelBox::new(16.0, 625.0, 8.0, 390.0),
                PanelBox::new(30.25, 800.5, 40.75, 610.0),
            ] {
                measurements::start();
                let plan =
                    plan_panel_for_test(&plot, plot.kind(), cell, LegendRole::Suppressed, &theme)
                        .unwrap();
                let work = measurements::finish();
                assert!(work.roots.len() >= 2);
                assert_eq!(work.counters[Counter::LegacyPanelConstructor as usize], 0);
                let final_plot = plan.plot_with_planned_limits(&plot, plot.data()).unwrap();
                let selected = final_plot
                    .measure_selected_interior(PlotRenderContext::new(
                        final_plot.data(),
                        plan.panel_box,
                        &theme,
                    ))
                    .unwrap();
                assert_eq!(selected.layout.projection.y_range(), range);
                assert_eq!(
                    selected.layout.x_axis.ticks,
                    plan.result.layout.x_axis.ticks
                );
                assert_eq!(
                    selected.layout.y_axis.ticks,
                    plan.result.layout.y_axis.ticks
                );
                assert_eq!(
                    selected.marks.as_ref().unwrap().geometry.bounds(),
                    plan.result.marks.as_ref().unwrap().geometry.bounds()
                );
                let draw = |result: &PlotRenderResult| {
                    let mut svg = crate::render::svg_backend::SvgBackend::in_memory(900, 700);
                    svg.begin(900, 700).unwrap();
                    result.marks.as_ref().unwrap().draw(&mut svg).unwrap();
                    result.annotations.as_ref().unwrap().draw(&mut svg).unwrap();
                    result.axes.as_ref().unwrap().draw(&mut svg).unwrap();
                    svg.end().unwrap();
                    svg.into_bytes().unwrap()
                };
                assert_eq!(draw(&selected), draw(&plan.result));
                let required = selected.axes.as_ref().unwrap().requirements();
                assert_eq!(
                    [
                        plan.panel_box.x0 - cell.x0,
                        cell.x1 - plan.panel_box.x1,
                        plan.panel_box.y0 - cell.y0,
                        cell.y1 - plan.panel_box.y1
                    ]
                    .map(f64::to_bits),
                    [required.left, required.right, required.bottom, required.top]
                        .map(|inset| (inset + 4.0).ceil().to_bits())
                );
            }
        }
    }
}

#[test]
fn native_facets_and_shared_canvas_replay_identical_feedback_without_child_reuse() {
    use crate::decorations::plan::children::DISABLE_CHILD_REUSE;
    let source = Plot::interaction(
        CategoryValues::new("condition", ["b", "a", "c", "b", "a", "c"]).unwrap(),
        NumericValues::new("estimate", [2.0, 3.0, 4.0, 5.0, 6.0, 7.0]),
        CategoryValues::new("group", ["one", "one", "one", "two", "three", "two"]).unwrap(),
        Bounds::new(
            NumericValues::new("lo", [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
            NumericValues::new("hi", [3.0, 4.0, 5.0, 6.0, 7.0, 8.0]),
        ),
    )
    .hue(CategoryValues::new("hue", ["red", "red", "red", "blue", "blue", "blue"]).unwrap())
    .x_label("First axis line\nSecond axis line\nThird axis line")
    .size(1200, 640);
    for mode in [
        None,
        Some(crate::FacetScales::Fixed),
        Some(crate::FacetScales::FreeX),
        Some(crate::FacetScales::FreeY),
        Some(crate::FacetScales::Free),
    ] {
        let mut outputs = Vec::new();
        for disabled in [false, true] {
            STAGES.with(|counts| counts.set([0; 3]));
            DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
            measurements::start();
            let mut svg = crate::render::svg_backend::SvgBackend::in_memory(1200, 640);
            let result = if let Some(scales) = mode {
                source
                    .clone()
                    .facet(
                        crate::FacetSpec::column(
                            CategoryValues::new(
                                "panel",
                                ["left", "left", "left", "right", "right", "right"],
                            )
                            .unwrap(),
                        )
                        .scales(scales),
                    )
                    .render_with_backend(&mut svg)
            } else {
                crate::Canvas::grid(1, 2)
                    .unwrap()
                    .size(1200, 640)
                    .panel(0, 0, source.clone())
                    .unwrap()
                    .panel(0, 1, source.clone())
                    .unwrap()
                    .legend_policy(crate::CanvasLegendPolicy::Shared)
                    .render_with_backend(&mut svg)
            };
            let work = measurements::finish();
            DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
            result.unwrap();
            assert!(work.roots.len() >= 2);
            assert!(work
                .gutters
                .iter()
                .any(|change| change.consumed.map(f64::to_bits)
                    != change.requested.map(f64::to_bits)));
            assert_eq!(work.counters[Counter::LegacyPanelConstructor as usize], 0);
            if disabled {
                assert_eq!(work.counters[Counter::ChildCacheHit as usize], 0);
            }
            for counter in [
                Counter::OneCategoricalAxisMeasure,
                Counter::StandardPanelIntrinsic,
                Counter::TickSource,
                Counter::AxisRangePreparation,
                Counter::AxisSpecPreparation,
                Counter::AxisSourcePreparation,
            ] {
                assert_eq!(work.capture_counters[counter as usize], 0);
            }
            let [prepared, measured, drawn] = STAGES.with(std::cell::Cell::get);
            assert_eq!(prepared, 2);
            assert!(measured >= 2);
            assert_eq!(drawn, 2);
            outputs.push(svg.into_bytes().unwrap());
        }
        assert_eq!(outputs[0], outputs[1]);
    }
}
