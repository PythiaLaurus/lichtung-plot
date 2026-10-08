use super::{coordinate::Intrinsics, InteriorRoute, OneCategoricalAxis};
use crate::font::canonical::OperationContext;
use crate::layout::fit::{FitKind, FitScope, PlanError};
use crate::layout::{AxisSide, PanelBox, Projection, TickSource};
use crate::orchestrator::measurements::{self, Counter};
use crate::spec::{AxisDomain, PlotRenderContext};
use crate::{Bounds, CategoryValues, CoefficientInput, DumbbellEndpoints, NumericValues, Plot};

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "compare all four retained families under actual facet and shared-Canvas roots"
)]
fn native_estimate_facets_and_shared_canvas_use_real_gutter_feedback_and_cache_identity() {
    use crate::decorations::plan::children::DISABLE_CHILD_REUSE;
    fn configured<K: crate::kind::ExactFamily + crate::kind::Facet>(
        plot: Plot<K>,
        faceted: bool,
    ) -> (Plot, Option<crate::Canvas>) {
        let plot = plot
            .x_label("First axis line\nSecond axis line\nThird axis line")
            .size(1200, 640);
        if faceted {
            (
                plot.facet(crate::FacetSpec::column(
                    CategoryValues::new("panel", ["a", "a", "b", "b"]).unwrap(),
                ))
                .erase(),
                None,
            )
        } else {
            let canvas = crate::Canvas::grid(1, 2)
                .unwrap()
                .panel(0, 0, plot.clone())
                .unwrap()
                .panel(0, 1, plot.clone())
                .unwrap()
                .x_range_group([(0, 0), (0, 1)])
                .unwrap()
                .size(1200, 640);
            (plot.erase(), Some(canvas))
        }
    }
    for faceted in [false, true] {
        let terms = CategoryValues::new(
            "term",
            if faceted {
                ["first", "second", "first", "second"]
            } else {
                ["first", "second", "third", "fourth"]
            },
        )
        .unwrap();
        let values = NumericValues::nullable("estimate", [None, None, Some(10.0), Some(20.0)]);
        let bounds = Bounds::new(
            NumericValues::nullable("lower", [None, None, Some(9.0), Some(19.0)]),
            NumericValues::nullable("upper", [None, None, Some(11.0), Some(21.0)]),
        );
        let statuses = [
            crate::EstimateStatus::NotEstimable,
            crate::EstimateStatus::NotEstimable,
            crate::EstimateStatus::Estimated,
            crate::EstimateStatus::Estimated,
        ];
        let plots = [
            configured(
                Plot::point_interval(terms.clone(), values.clone(), bounds.clone())
                    .estimate_status(statuses)
                    .zero_line(),
                faceted,
            ),
            configured(
                Plot::coefficient(
                    terms.clone(),
                    CoefficientInput::estimates_with_intervals(values.clone(), bounds.clone()),
                )
                .estimate_status(statuses),
                faceted,
            ),
            configured(
                Plot::forest(terms.clone(), values, bounds, 1.0)
                    .scale(crate::AxisScale::Log10)
                    .estimate_status(statuses),
                faceted,
            ),
            configured(
                Plot::dumbbell(
                    terms,
                    NumericValues::new("left", [1.0, 2.0, 3.0, 4.0]),
                    NumericValues::new("right", [4.0, 3.0, 2.0, 1.0]),
                    DumbbellEndpoints::new("Before", (20, 80, 140), "After", (180, 60, 40))
                        .unwrap(),
                ),
                faceted,
            ),
        ];
        for (family, (plot, canvas)) in plots.into_iter().enumerate() {
            let mut outputs = Vec::new();
            for disabled in [false, true] {
                crate::plots::interval_estimate::STAGES.with(|counts| counts.set([[0; 4]; 3]));
                crate::plots::dumbbell::STAGES.with(|counts| counts.set([0; 4]));
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
                result.unwrap();
                assert!(work.roots.len() >= 2);
                assert!(work.gutters.iter().any(|change| change.owner
                    != crate::layout::ComponentAddress::root()
                    && change.consumed.map(f64::to_bits) != change.requested.map(f64::to_bits)));
                assert!(work.counters[Counter::OneCategoricalAxisMeasure as usize] > 0);
                assert_eq!(work.counters[Counter::LegacyPanelConstructor as usize], 0);
                if disabled {
                    assert_eq!(work.counters[Counter::ChildCacheHit as usize], 0);
                    assert_eq!(work.counters[Counter::ChildFeedbackReplay as usize], 0);
                }
                let counts = if family < 3 {
                    crate::plots::interval_estimate::STAGES.with(std::cell::Cell::get)[family]
                } else {
                    crate::plots::dumbbell::STAGES.with(std::cell::Cell::get)
                };
                assert_eq!(counts[0], 2);
                assert_eq!(counts[1], counts[2]);
                assert_eq!(counts[3], 2);
                outputs.push(svg.into_bytes().unwrap());
            }
            assert_eq!(outputs[0], outputs[1]);
        }
    }
}

pub(crate) fn sources() -> [Plot; 4] {
    let terms = CategoryValues::new("term", ["after_visit", "before_visit", "last_visit"]).unwrap();
    let estimate = NumericValues::new("estimate", [10.0, 20.0, 30.0]);
    let bounds = Bounds::new(
        NumericValues::new("lower", [9.0, 19.0, 29.0]),
        NumericValues::new("upper", [11.0, 21.0, 31.0]),
    );
    [
        Plot::point_interval(terms.clone(), estimate.clone(), bounds.clone())
            .zero_line()
            .erase(),
        Plot::coefficient(terms.clone(), CoefficientInput::estimates(estimate.clone())).erase(),
        Plot::forest(terms.clone(), estimate.clone(), bounds, 1.0).erase(),
        Plot::dumbbell(
            terms,
            estimate,
            NumericValues::new("right", [15.0, 18.0, 34.0]),
            DumbbellEndpoints::new("Before", (20, 80, 140), "After", (180, 60, 40)).unwrap(),
        )
        .erase(),
    ]
}

#[test]
fn native_estimates_retain_orientation_raw_display_midpoints_and_selected_projection() {
    let routes = [
        OneCategoricalAxis::PointInterval,
        OneCategoricalAxis::Coefficient,
        OneCategoricalAxis::Forest,
        OneCategoricalAxis::Dumbbell,
    ];
    for (source, route) in sources().into_iter().zip(routes) {
        let source =
            source.labels(crate::Labels::humanized().alias("after_visit", "Follow-up visit"));
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        assert_eq!(inputs.route, InteriorRoute::OneCategoricalAxis(route));
        let plot = inputs.attach(source).unwrap();
        let theme = crate::Theme::Publication.resolve().unwrap();
        let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
        for panel in [
            PanelBox::new(80.25, 550.75, 70.5, 410.25),
            PanelBox::new(30.0, 810.0, 50.0, 600.0),
        ] {
            for range in [(0.0, 40.0), (10.0, 25.0)] {
                let plot = plot
                    .clone()
                    .with_planned_x_limits(range)
                    .with_planned_y_limits((-10.0, 10.0));
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
                assert_eq!(selected.layout.projection.panel_box(), panel);
                assert_eq!(selected.layout.projection.x_range(), range);
                assert_eq!(selected.layout.projection.y_range(), (0.0, 3.0));
                assert_eq!(selected.x_domain, AxisDomain::Continuous);
                assert_eq!(selected.y_domain, AxisDomain::Categorical);
                assert!(selected.x_tick_keys.is_none());
                assert_eq!(
                    selected.y_tick_keys.as_ref().unwrap(),
                    &["after_visit", "before_visit", "last_visit"]
                );
                assert_eq!(selected.layout.y_axis.source, TickSource::SUPPLIED);
                assert_eq!(
                    selected
                        .layout
                        .y_axis
                        .ticks
                        .iter()
                        .map(|tick| tick.position)
                        .collect::<Vec<_>>(),
                    [2.5, 1.5, 0.5]
                );
                assert_eq!(
                    selected
                        .layout
                        .y_axis
                        .ticks
                        .iter()
                        .map(|tick| tick.label.as_str())
                        .collect::<Vec<_>>(),
                    ["Follow-up visit", "Before visit", "Last visit"]
                );
                assert_eq!(
                    selected.axes.as_ref().unwrap().range(AxisSide::Bottom),
                    range
                );
                assert_eq!(
                    selected.axes.as_ref().unwrap().range(AxisSide::Left),
                    (0.0, 3.0)
                );
                assert_eq!(selected.annotations.as_ref().unwrap().panel_clip, panel);
                assert!(selected.marks.is_some());
                assert!(PlotRenderContext::new(plot.data(), panel, &theme)
                    .with_prepared_axes(&plot)
                    .unwrap()
                    .layout_with_axis_specs(
                        (0.0, 1.0),
                        (0.0, 1.0),
                        panel,
                        String::new(),
                        String::new(),
                        &crate::layout::AxisSpec::default(),
                        &crate::layout::AxisSpec::default(),
                        &theme
                    )
                    .is_err());
            }
        }
    }
}

#[test]
fn native_vertical_category_intrinsics_use_mandatory_lines_and_keep_fit_scope() {
    for source in sources() {
        let source = source.x_label("").y_label("").labels(
            crate::Labels::raw().alias("after_visit", "First line\nSecond line\nThird line"),
        );
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        let plot = inputs.attach(source).unwrap();
        let theme = crate::Theme::Publication.resolve().unwrap();
        let minimum = Intrinsics::prepared(&plot, &theme).unwrap().minimum(&plot);
        let label = theme
            .measure_text(
                "First line\nSecond line\nThird line",
                &theme.typography.axes.tick_label,
                crate::layout::PhysicalTextAlignment::Center,
                "category intrinsic test",
                false,
            )
            .unwrap();
        let range = plot.prepared_axis_inputs().unwrap().ranges[0];
        let source = crate::layout::panel::format_label(
            f64::midpoint(range.0, range.1),
            plot.prepared_axis_inputs().unwrap().logarithmic,
            false,
            0,
        )
        .unwrap();
        let tick = theme
            .measure_text(
                &source,
                &theme.typography.axes.tick_label,
                crate::layout::PhysicalTextAlignment::Center,
                "numeric midpoint intrinsic test",
                false,
            )
            .unwrap();
        assert_eq!(
            minimum.0.to_bits(),
            crate::layout::kernel::required_length(
                tick.width().max(tick.fit_width()) + 2.0 * crate::layout::panel::LABEL_CLEARANCE
            )
            .unwrap()
            .to_bits()
        );
        assert_eq!(
            minimum.1.to_bits(),
            (3.0 * (label.height() + 2.0 * crate::layout::panel::LABEL_CLEARANCE)).to_bits()
        );
        assert!(minimum.1 > crate::layout::canvas::MINIMUM_PANEL_SPAN);
        let scope = FitScope::new(
            &crate::layout::ComponentAddress::root()
                .child(crate::layout::AddressRole::PlotBody, 3)
                .unwrap(),
            4,
        );
        let before = Projection::test_projection_calls();
        let error = plot
            .measure_selected_interior(
                PlotRenderContext::new(
                    plot.data(),
                    PanelBox::new(0.0, 500.0, 0.0, minimum.1 - 1.0 / 1024.0),
                    &theme,
                )
                .with_fit_scope(&scope),
            )
            .unwrap_err();
        let PlanError::Fit(failure) = error else {
            panic!("scoped fit miss")
        };
        assert_eq!(failure.reason.kind, FitKind::TrackOrResidual);
        assert_eq!(failure.reason.address, scope.address);
        assert_eq!(failure.reason.ordinal, 4);
        assert_eq!(Projection::test_projection_calls(), before);
        plot.measure_selected_interior(PlotRenderContext::new(
            plot.data(),
            PanelBox::new(0.0, 500.0, 0.0, minimum.1.ceil() + 1.0),
            &theme,
        ))
        .unwrap();
    }
}

#[test]
fn native_forest_selects_sole_log_source_and_transforms_raw_exact_ticks_only_in_preparation() {
    let base = Plot::forest(
        CategoryValues::new("term", ["first", "second"]).unwrap(),
        NumericValues::new("estimate", [1.0, 10.0]),
        Bounds::new(
            NumericValues::new("lower", [0.1, 1.0]),
            NumericValues::new("upper", [10.0, 100.0]),
        ),
        1.0,
    )
    .scale(crate::AxisScale::Log10)
    .x_limits(0.01, 1000.0);
    for exact in [false, true] {
        let source = if exact {
            base.clone().x_ticks(vec![
                crate::AxisTick::new(0.01, "raw lower"),
                crate::AxisTick::new(1.0, ""),
                crate::AxisTick::new(1000.0, "raw upper"),
            ])
        } else {
            base.clone()
        }
        .erase();
        measurements::start();
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        let work = measurements::finish();
        assert_eq!(
            work.counters[Counter::ForestRawTickProjection as usize],
            if exact { 3 } else { 0 }
        );
        assert_eq!(inputs.axes[0].1.ranges[0], (-2.0, 3.0));
        let plot = inputs.attach(source).unwrap();
        let theme = crate::Theme::Publication.resolve().unwrap();
        for range in [(-2.0, 3.0), (-3.0, 4.0)] {
            let plot =
                plot.clone()
                    .with_resolved_facet_ranges(crate::PlotKind::Forest, Some(range), None);
            for panel in [
                PanelBox::new(80.25, 550.75, 70.5, 410.25),
                PanelBox::new(30.0, 810.0, 50.0, 600.0),
            ] {
                measurements::start();
                let selected = plot
                    .measure_selected_interior(PlotRenderContext::new(plot.data(), panel, &theme))
                    .unwrap();
                let work = measurements::finish();
                assert_eq!(work.counters[Counter::ForestRawTickProjection as usize], 0);
                assert_eq!(work.counters[Counter::AxisRangePreparation as usize], 0);
                assert_eq!(selected.layout.projection.x_range(), range);
                if exact {
                    assert_eq!(selected.layout.x_axis.source, TickSource::EXACT);
                    assert_eq!(
                        selected.layout.x_axis.ticks,
                        [
                            crate::AxisTick::new(-2.0, "raw lower"),
                            crate::AxisTick::new(0.0, ""),
                            crate::AxisTick::new(3.0, "raw upper")
                        ]
                    );
                    assert_eq!(work.counters[Counter::TickSource as usize], 0);
                } else {
                    let expected = crate::plots::interval_estimate::log_tick_plan(
                        range,
                        None,
                        panel,
                        &theme,
                        &FitScope::root(),
                    )
                    .unwrap();
                    assert_eq!(selected.layout.x_axis.source, expected.source);
                    assert_eq!(selected.layout.x_axis.ticks, expected.ticks);
                }
            }
        }
    }
}
