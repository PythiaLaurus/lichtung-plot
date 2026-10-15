use super::{coordinate::Intrinsics, InteriorRoute, MarkPayload, OneCategoricalAxis};
use crate::font::canonical::OperationContext;
use crate::layout::{AxisSide, PanelBox, TickSource};
use crate::orchestrator::measurements::{self, Counter};
use crate::spec::{AxisDomain, PlotRenderContext};
use crate::{BarInput, CategoryValues, NumericValues, Plot};

pub(crate) fn sources() -> [Plot; 4] {
    configured_sources(false)
}

fn configured_sources(annotated: bool) -> [Plot; 4] {
    fn finish<K: crate::kind::ExactFamily + crate::kind::Significance<crate::Pairwise>>(
        plot: Plot<K>,
        annotated: bool,
    ) -> Plot {
        plot.significance(
            [crate::Significance::between(
                "after_visit",
                "before_visit",
                crate::SignificanceMark::OneStar,
            )]
            .into_iter()
            .filter(|_| annotated),
        )
        .erase()
    }
    let attached = || {
        [
            crate::Significance::at("after_visit", crate::SignificanceMark::PValue(0.0123)),
            crate::Significance::at("before_visit", crate::SignificanceMark::TwoStars),
        ]
        .into_iter()
        .filter(|_| annotated)
    };
    let categories = CategoryValues::new(
        "category",
        [
            "after_visit",
            "before_visit",
            "after_visit",
            "before_visit",
            "after_visit",
            "before_visit",
        ],
    )
    .unwrap();
    let values = NumericValues::new("value", [-2.0, 3.0, -4.0, 7.0, 5.0, -1.0]);
    let series = CategoryValues::new(
        "series",
        ["first", "first", "second", "second", "third", "third"],
    )
    .unwrap();
    [
        finish(
            Plot::bar(categories.clone(), BarInput::estimates(values.clone()))
                .series(series.clone())
                .significance(attached()),
            annotated,
        ),
        finish(
            Plot::horizontal_bar(categories.clone(), BarInput::estimates(values.clone()))
                .series(series.clone())
                .color_by_sign()
                .zero_line()
                .significance(attached()),
            annotated,
        ),
        finish(
            Plot::stacked_bar(categories.clone(), values.clone(), series.clone()),
            annotated,
        ),
        finish(
            Plot::stacked_horizontal_bar(categories, values, series),
            annotated,
        ),
    ]
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "each orientation compares prepared signed geometry at multiple exact parent assignments"
)]
fn native_bars_preserve_signed_stacks_midpoints_raw_labels_and_assigned_projection() {
    let routes = [
        OneCategoricalAxis::Bar,
        OneCategoricalAxis::HorizontalBar,
        OneCategoricalAxis::StackedBar,
        OneCategoricalAxis::StackedHorizontalBar,
    ];
    for (family, (source, route)) in sources().into_iter().zip(routes).enumerate() {
        let horizontal = family % 2 == 1;
        let source =
            source.labels(crate::Labels::humanized().alias("after_visit", "Follow-up visit"));
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        assert_eq!(inputs.route, InteriorRoute::OneCategoricalAxis(route));
        let plot = inputs.attach(source).unwrap();
        let prepared = crate::plots::bar_data::retained(&plot).unwrap();
        if family >= 2 {
            assert_eq!(prepared.value_range, (-6.0, 10.0));
            assert_eq!(
                prepared
                    .bars
                    .iter()
                    .map(|bar| (bar.start, bar.end))
                    .collect::<Vec<_>>(),
                [
                    (0.0, -2.0),
                    (0.0, 3.0),
                    (-2.0, -6.0),
                    (3.0, 10.0),
                    (0.0, 5.0),
                    (0.0, -1.0)
                ]
            );
        }
        let theme = crate::Theme::Publication.resolve().unwrap();
        let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
        for panel in [
            PanelBox::new(80.25, 550.75, 70.5, 410.25),
            PanelBox::new(30.0, 810.0, 50.0, 600.0),
        ] {
            for range in [(-12.0, 15.0), (-3.0, 8.0)] {
                let plot = if horizontal {
                    plot.clone()
                        .with_planned_x_limits(range)
                        .with_planned_y_limits((-10.0, 10.0))
                } else {
                    plot.clone()
                        .with_planned_y_limits(range)
                        .with_planned_x_limits((-10.0, 10.0))
                };
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
                assert_eq!(
                    crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                    reads
                );
                let (category, keys, category_side, numeric_side) = if horizontal {
                    assert_eq!(
                        (selected.x_domain, selected.y_domain),
                        (AxisDomain::Continuous, AxisDomain::Categorical)
                    );
                    (
                        &selected.layout.y_axis,
                        &selected.y_tick_keys,
                        AxisSide::Left,
                        AxisSide::Bottom,
                    )
                } else {
                    assert_eq!(
                        (selected.x_domain, selected.y_domain),
                        (AxisDomain::Categorical, AxisDomain::Continuous)
                    );
                    (
                        &selected.layout.x_axis,
                        &selected.x_tick_keys,
                        AxisSide::Bottom,
                        AxisSide::Left,
                    )
                };
                assert_eq!(keys.as_ref().unwrap(), &["after_visit", "before_visit"]);
                assert_eq!(category.source, TickSource::SUPPLIED);
                assert_eq!(
                    category
                        .ticks
                        .iter()
                        .map(|tick| (tick.position, tick.label.as_str()))
                        .collect::<Vec<_>>(),
                    [(0.5, "Follow-up visit"), (1.5, "Before visit")]
                );
                assert_eq!(
                    selected.axes.as_ref().unwrap().range(category_side),
                    (0.0, 2.0)
                );
                assert_eq!(selected.axes.as_ref().unwrap().range(numeric_side), range);
                assert_eq!(selected.layout.projection.panel_box(), panel);
                let MarkPayload::Bars(payload) = &selected.marks.as_ref().unwrap().payload else {
                    panic!("bars")
                };
                assert_eq!(payload.clip, panel);
                if let Some(expansion) = payload.domain_expansion {
                    assert_eq!(
                        expansion.axis,
                        if horizontal {
                            super::ClearanceAxis::X
                        } else {
                            super::ClearanceAxis::Y
                        }
                    );
                    assert!(expansion.limits.0 <= range.0 && expansion.limits.1 >= range.1);
                }
                assert_eq!(payload.zero.is_some(), family == 1);
                for (shape, bar) in payload.bars.iter().zip(&prepared.bars) {
                    let project = |value| {
                        if horizontal {
                            selected.layout.projection.project(value, 0.0).0
                        } else {
                            selected.layout.projection.project(0.0, value).1
                        }
                    };
                    let (start, end) = if horizontal {
                        (shape.polygon[0].x, shape.polygon[1].x)
                    } else {
                        (shape.polygon[0].y, shape.polygon[1].y)
                    };
                    assert_eq!((start, end), (project(bar.start), project(bar.end)));
                }
            }
        }
    }
}

#[test]
fn native_hidden_horizontal_categories_preserve_identity_without_intrinsic_or_gutter_reserve() {
    for source in sources().into_iter().skip(1).step_by(2) {
        let source = source
            .x_label("")
            .y_label("")
            .labels(crate::Labels::raw().alias(
                "after_visit",
                "First line\nSecond line\nThird line\nFourth line",
            ));
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        let plot = inputs.attach(source).unwrap();
        let theme = crate::Theme::Publication.resolve().unwrap();
        let visible = Intrinsics::prepared(&plot, &theme).unwrap().minimum(&plot);
        let hidden = plot.clone().with_y_tick_labels_visible(false);
        assert!(
            visible.1
                > Intrinsics::prepared(&hidden, &theme)
                    .unwrap()
                    .minimum(&hidden)
                    .1
        );
        let panel = PanelBox::new(80.0, 600.0, 70.0, 450.0);
        let selected = |plot: &Plot| {
            plot.measure_selected_interior(PlotRenderContext::new(plot.data(), panel, &theme))
                .unwrap()
        };
        let shown = selected(&plot);
        let hidden = selected(&hidden);
        assert_eq!(shown.y_tick_keys, hidden.y_tick_keys);
        assert_eq!(
            shown.layout.projection.y_range(),
            hidden.layout.projection.y_range()
        );
        assert_eq!(
            hidden
                .layout
                .y_axis
                .ticks
                .iter()
                .map(|tick| (tick.position, tick.label.as_str()))
                .collect::<Vec<_>>(),
            [(0.5, ""), (1.5, "")]
        );
        assert!(
            hidden.axes.as_ref().unwrap().requirements().left
                < shown.axes.as_ref().unwrap().requirements().left
        );
    }
}

#[test]
fn native_bar_domain_feedback_is_conditional_and_final_axes_match_marks_at_the_parent_panel() {
    use crate::decorations::plan::{plan_panel_for_test, LegendRole};
    use crate::render::backend::Backend;
    for (family, source) in configured_sources(true).into_iter().enumerate() {
        let horizontal = family % 2 == 1;
        let source = source.with_suppressed_legend().size(900, 700);
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        let plot = inputs.attach(source).unwrap();
        let theme = crate::Theme::Publication.resolve().unwrap();
        for cell in [
            PanelBox::new(16.0, 625.0, 8.0, 390.0),
            PanelBox::new(30.25, 800.5, 40.75, 610.0),
        ] {
            let raw_range = crate::plots::bar_data::retained(&plot).unwrap().value_range;
            let plot = if horizontal {
                plot.clone().with_planned_x_limits(raw_range)
            } else {
                plot.clone().with_planned_y_limits(raw_range)
            };
            let initial = plot
                .measure_selected_interior(PlotRenderContext::new(plot.data(), cell, &theme))
                .unwrap();
            let MarkPayload::Bars(payload) = &initial.marks.as_ref().unwrap().payload else {
                panic!("bars")
            };
            assert!(payload.domain_expansion.is_some());
            let plan =
                plan_panel_for_test(&plot, plot.kind(), cell, LegendRole::Suppressed, &theme)
                    .unwrap();
            let MarkPayload::Bars(payload) = &plan.result.marks.as_ref().unwrap().payload else {
                panic!("bars")
            };
            assert!(payload.domain_expansion.is_none());
            let projection = &plan.result.layout.projection;
            let final_plot = plan.plot_with_planned_limits(&plot);
            let selected = final_plot
                .measure_selected_interior(PlotRenderContext::new(
                    final_plot.data(),
                    plan.panel_box,
                    &theme,
                ))
                .unwrap();
            assert_eq!(
                selected.layout.x_axis.ticks,
                plan.result.layout.x_axis.ticks
            );
            assert_eq!(
                selected.layout.y_axis.ticks,
                plan.result.layout.y_axis.ticks
            );
            let value_range = if horizontal {
                projection.x_range()
            } else {
                projection.y_range()
            };
            assert_eq!(
                plan.result.significance.as_ref().unwrap().y_range,
                value_range
            );
            let render = |result: &crate::spec::PlotRenderResult| {
                let mut svg = crate::render::svg_backend::SvgBackend::in_memory(900, 700);
                svg.begin(900, 700).unwrap();
                result.marks.as_ref().unwrap().draw(&mut svg).unwrap();
                crate::significance::render(
                    &final_plot,
                    result.significance.as_ref().unwrap(),
                    &theme,
                    &mut svg,
                )
                .unwrap();
                result.annotations.as_ref().unwrap().draw(&mut svg).unwrap();
                result.axes.as_ref().unwrap().draw(&mut svg).unwrap();
                svg.end().unwrap();
                svg.into_bytes().unwrap()
            };
            assert_eq!(render(&selected), render(&plan.result));
            let required = selected.axes.as_ref().unwrap().requirements();
            assert_eq!(
                (
                    plan.panel_box.x0 - cell.x0,
                    cell.x1 - plan.panel_box.x1,
                    plan.panel_box.y0 - cell.y0,
                    cell.y1 - plan.panel_box.y1
                ),
                (
                    (required.left + 4.0).ceil(),
                    (required.right + 4.0).ceil(),
                    (required.bottom + 4.0).ceil(),
                    (required.top + 4.0).ceil()
                )
            );
        }
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "public fixed-limit configurations exercise both annotations and semantic refusal in every family"
)]
fn native_bar_fixed_limits_and_exact_numeric_labels_never_expand_and_invalid_sources_precede_fonts()
{
    for sufficient in [false, true] {
        let range = if sufficient {
            (-40.0, 40.0)
        } else {
            (-1.0, 1.0)
        };
        let categories = CategoryValues::new("category", ["a", "b"]).unwrap();
        let values = NumericValues::new("value", [-2.0, 3.0]);
        let series = CategoryValues::new("series", ["g", "g"]).unwrap();
        let attached = || {
            [
                crate::Significance::at("a", crate::SignificanceMark::TwoStars),
                crate::Significance::at("b", crate::SignificanceMark::PValue(0.01)),
            ]
        };
        let comparison = || {
            [crate::Significance::between(
                "a",
                "b",
                crate::SignificanceMark::OneStar,
            )]
        };
        let sources = [
            Plot::bar(categories.clone(), BarInput::estimates(values.clone()))
                .significance(attached())
                .significance(comparison())
                .y_limits(range.0, range.1)
                .erase(),
            Plot::horizontal_bar(categories.clone(), BarInput::estimates(values.clone()))
                .significance(attached())
                .significance(comparison())
                .x_limits(range.0, range.1)
                .erase(),
            Plot::stacked_bar(categories.clone(), values.clone(), series.clone())
                .significance(comparison())
                .y_limits(range.0, range.1)
                .erase(),
            Plot::stacked_horizontal_bar(categories, values, series)
                .significance(comparison())
                .x_limits(range.0, range.1)
                .erase(),
        ];
        for (family, source) in sources.into_iter().enumerate() {
            let side = usize::from(family % 2 == 0);
            let source = source.with_suppressed_legend();
            let exact = source.clone().with_test_axis_ticks(
                side,
                vec![
                    crate::AxisTick::new(range.0, "raw_lower"),
                    crate::AxisTick::new(0.0, ""),
                    crate::AxisTick::new(range.1, "raw_upper"),
                ],
            );
            let inputs =
                crate::plots::shared_x::prepare_plot(&exact, &OperationContext::new(), None)
                    .unwrap();
            let plot = inputs.attach(exact).unwrap();
            let theme = crate::Theme::Publication.resolve().unwrap();
            let result = plot.measure_selected_interior(PlotRenderContext::new(
                plot.data(),
                PanelBox::new(80.0, 600.0, 70.0, 450.0),
                &theme,
            ));
            if sufficient {
                let result = result.unwrap();
                let axis = if side == 0 {
                    &result.layout.x_axis
                } else {
                    &result.layout.y_axis
                };
                assert_eq!(axis.source, TickSource::EXACT);
                assert_eq!(
                    axis.ticks
                        .iter()
                        .map(|tick| tick.label.as_str())
                        .collect::<Vec<_>>(),
                    ["raw_lower", "", "raw_upper"]
                );
                let MarkPayload::Bars(payload) = &result.marks.as_ref().unwrap().payload else {
                    panic!("bars")
                };
                assert!(payload.domain_expansion.is_none());
                assert_eq!(result.significance.as_ref().unwrap().y_range, range);
            } else {
                assert!(matches!(result, Err(crate::layout::fit::PlanError::Fit(_))));
            }
            for ticks in [
                vec![],
                vec![crate::AxisTick::new(f64::NAN, "invalid")],
                vec![crate::AxisTick::new(1e100, "outside")],
            ] {
                let invalid = source.clone().with_test_axis_ticks(side, ticks);
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
                assert!(matches!(invalid.preflight(), Err(crate::Error::Data(_))));
                assert_eq!(
                    crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                    0
                );
            }
        }
    }
}

#[test]
fn native_bar_signs_keep_the_scientific_zero_anchor_in_both_orientations() {
    for values in [[1.0, 2.0], [-1.0, -2.0], [0.0, 0.0], [-1.0, 2.0]] {
        let categories = CategoryValues::new("category", ["a", "b"]).unwrap();
        let series = CategoryValues::new("series", ["g", "g"]).unwrap();
        let numeric = NumericValues::new("value", values);
        let plots = [
            Plot::bar(categories.clone(), BarInput::estimates(numeric.clone())).erase(),
            Plot::horizontal_bar(categories.clone(), BarInput::estimates(numeric.clone()))
                .zero_line()
                .erase(),
            Plot::stacked_bar(categories.clone(), numeric.clone(), series.clone()).erase(),
            Plot::stacked_horizontal_bar(categories, numeric, series).erase(),
        ];
        for (family, source) in plots.into_iter().enumerate() {
            let inputs =
                crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None)
                    .unwrap();
            let range = inputs.axes[0].1.ranges[usize::from(family % 2 == 0)];
            if values.iter().all(|value| *value >= 0.0) {
                assert_eq!(range.0.to_bits(), 0.0_f64.to_bits());
            }
            if values.iter().all(|value| *value < 0.0) {
                assert_eq!(range.1.to_bits(), 0.0_f64.to_bits());
            }
            assert!(range.0 <= values[0].min(values[1]) && range.1 >= values[0].max(values[1]));
            for size in [(401, 301), (641, 401)] {
                source.clone().size(size.0, size.1).preflight().unwrap();
            }
        }
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "actual fixed/free facets and shared Canvas roots exercise all four family caches and feedback"
)]
fn native_bar_facets_and_shared_canvas_retain_scientific_inputs_and_replay_feedback_identically() {
    use crate::decorations::plan::children::DISABLE_CHILD_REUSE;
    fn configured<
        K: crate::kind::ExactFamily + crate::kind::Facet + crate::kind::Significance<crate::Pairwise>,
    >(
        plot: Plot<K>,
        mode: usize,
        horizontal: bool,
    ) -> (Plot, Option<crate::Canvas>) {
        let comparison =
            crate::Significance::between("a", "b", crate::SignificanceMark::PValue(0.01));
        let plot = plot
            .x_label("First axis line\nSecond axis line\nThird axis line")
            .size(1200, 640);
        if mode < 2 {
            (
                plot.significance([comparison.facet(crate::FacetCell::single("left"))])
                    .facet(
                        crate::FacetSpec::column(
                            CategoryValues::new(
                                "panel",
                                [
                                    "left", "left", "left", "left", "right", "right", "right",
                                    "right",
                                ],
                            )
                            .unwrap(),
                        )
                        .scales(if mode == 0 {
                            crate::FacetScales::Fixed
                        } else {
                            crate::FacetScales::Free
                        }),
                    )
                    .erase(),
                None,
            )
        } else {
            let plot = plot.significance([comparison]);
            let mut canvas = crate::Canvas::grid(1, 2)
                .unwrap()
                .panel(0, 0, plot.clone())
                .unwrap()
                .panel(0, 1, plot.clone())
                .unwrap()
                .size(1200, 640);
            if horizontal {
                canvas = canvas
                    .x_range_group([(0, 0), (0, 1)])
                    .unwrap()
                    .category_labels(crate::CanvasCategoryLabels::FirstColumnPerRow);
            }
            (plot.erase(), Some(canvas))
        }
    }
    for mode in 0..3 {
        let categories =
            CategoryValues::new("category", ["a", "b", "a", "b", "a", "b", "a", "b"]).unwrap();
        let values = NumericValues::new("value", [-2.0, 3.0, 4.0, -1.0, -3.0, 2.0, 6.0, -5.0]);
        let series = CategoryValues::new(
            "series",
            [
                "first", "first", "second", "second", "first", "first", "second", "second",
            ],
        )
        .unwrap();
        let plots = [
            configured(
                Plot::bar(categories.clone(), BarInput::samples(values.clone()))
                    .series(series.clone()),
                mode,
                false,
            ),
            configured(
                Plot::horizontal_bar(categories.clone(), BarInput::samples(values.clone()))
                    .series(series.clone())
                    .color_by_sign()
                    .zero_line(),
                mode,
                true,
            ),
            configured(
                Plot::stacked_bar(categories.clone(), values.clone(), series.clone()),
                mode,
                false,
            ),
            configured(
                Plot::stacked_horizontal_bar(categories, values, series),
                mode,
                true,
            ),
        ];
        for (family, (plot, canvas)) in plots.into_iter().enumerate() {
            let mut outputs = Vec::new();
            for disabled in [false, true] {
                crate::plots::bar_data::STAGES.with(|counts| counts.set([[0; 4]; 4]));
                DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
                measurements::start();
                let mut svg = crate::render::svg_backend::SvgBackend::in_memory(1200, 640);
                let result = if let Some(canvas) = &canvas {
                    canvas.render_with_backend(&mut svg)
                } else {
                    plot.render_with_backend(&mut svg)
                };
                let work = measurements::finish();
                DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
                result.unwrap_or_else(|error| panic!("family {family}, mode {mode}: {error}"));
                assert!(work.roots.len() >= 2);
                assert!(work.gutters.iter().any(|change| change.owner
                    != crate::layout::ComponentAddress::root()
                    && change.consumed.map(f64::to_bits) != change.requested.map(f64::to_bits)));
                assert_eq!(work.counters[Counter::LegacyPanelConstructor as usize], 0);
                for counter in [
                    Counter::OneCategoricalAxisMeasure,
                    Counter::StandardPanelIntrinsic,
                    Counter::TickSource,
                    Counter::SignificanceBisection,
                ] {
                    assert_eq!(work.capture_counters[counter as usize], 0);
                }
                if disabled {
                    assert_eq!(work.counters[Counter::ChildCacheHit as usize], 0);
                }
                let counts = crate::plots::bar_data::STAGES.with(std::cell::Cell::get)[family];
                assert_eq!(counts[0], 2);
                assert_eq!(counts[1], counts[2]);
                assert_eq!(counts[3], 2);
                outputs.push(svg.into_bytes().unwrap());
            }
            assert_eq!(outputs[0], outputs[1]);
        }
    }
}
