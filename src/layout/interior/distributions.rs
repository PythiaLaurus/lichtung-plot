use crate::{Error, HistogramInput, NumericValues, Plot, PlotKind};

thread_local! {
    pub(crate) static STAGES: std::cell::Cell<[[usize; 4]; 4]> = const { std::cell::Cell::new([[0; 4]; 4]) };
}

pub(crate) fn count(family: usize, stage: usize) {
    STAGES.with(|counts| {
        let mut next = counts.get();
        next[family][stage] += 1;
        counts.set(next);
    });
}

pub(crate) fn sources() -> [Plot; 4] {
    sources_faceted(None)
}

fn sources_faceted(facet: Option<&crate::FacetSpec>) -> [Plot; 4] {
    fn finish<K: crate::kind::ExactFamily + crate::kind::Facet>(
        mut plot: Plot<K>,
        facet: Option<&crate::FacetSpec>,
    ) -> Plot {
        if let Some(facet) = facet {
            plot = plot.facet(facet.clone());
        }
        plot.erase()
    }
    let x = NumericValues::new("sample", [1.0, 2.0, 4.0, 5.0, 7.0, 8.0]);
    [
        finish(
            Plot::histogram(HistogramInput::samples(x.clone()))
                .bins(crate::HistogramBins::Count(8))
                .stat(crate::HistogramStat::Density)
                .composite(crate::HistogramComposite::KdeAndRug(
                    crate::HistogramKde::Internal,
                )),
            facet,
        ),
        finish(
            Plot::density(x.clone())
                .hue(crate::CategoryValues::new("series", ["a", "a", "a", "b", "b", "b"]).unwrap()),
            facet,
        ),
        finish(Plot::qq(x.clone()), facet),
        finish(
            Plot::regression(
                x,
                NumericValues::new("response", [1.0, 4.0, 2.0, 8.0, 6.0, 9.0]),
            ),
            facet,
        ),
    ]
}

#[test]
fn semantic_storage_and_statistics_are_prepared_once_before_fonts() {
    use crate::font::canonical::OperationContext;
    for (family, source) in sources().into_iter().enumerate() {
        STAGES.with(|counts| counts.set([[0; 4]; 4]));
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
        let resources = OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
        assert_eq!(STAGES.with(std::cell::Cell::get)[family], [1, 0, 0, 0]);

        let resources = OperationContext::new();
        crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
        let attached = inputs.attach(source).unwrap();
        assert!(std::sync::Arc::ptr_eq(
            &attached.prepared_axes,
            &inputs.axes
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}

#[test]
fn facets_and_shared_canvas_retain_each_components_statistics_once() {
    for scales in [
        crate::FacetScales::Fixed,
        crate::FacetScales::FreeX,
        crate::FacetScales::FreeY,
        crate::FacetScales::Free,
    ] {
        let facet = crate::FacetSpec::column(
            crate::CategoryValues::new(
                "panel",
                ["left", "left", "left", "right", "right", "right"],
            )
            .unwrap(),
        )
        .scales(scales);
        for (family, source) in sources_faceted(Some(&facet)).into_iter().enumerate() {
            STAGES.with(|counts| counts.set([[0; 4]; 4]));
            source.size(960, 640).preflight().unwrap();
            let counts = STAGES.with(std::cell::Cell::get)[family];
            assert_eq!(counts[0], 2);
            assert_eq!(counts[1], counts[2]);
            assert_eq!(counts[3], 2);
        }
    }
    STAGES.with(|counts| counts.set([[0; 4]; 4]));
    let x = NumericValues::new("sample", [1.0, 2.0, 4.0, 5.0, 7.0, 8.0]);
    crate::Canvas::grid(1, 4)
        .unwrap()
        .size(1600, 480)
        .panel(0, 0, Plot::histogram(HistogramInput::samples(x.clone())))
        .unwrap()
        .panel(0, 1, Plot::density(x.clone()))
        .unwrap()
        .panel(0, 2, Plot::qq(x.clone()))
        .unwrap()
        .panel(
            0,
            3,
            Plot::regression(
                x,
                NumericValues::new("response", [1.0, 4.0, 2.0, 8.0, 6.0, 9.0]),
            ),
        )
        .unwrap()
        .x_range_group([(0, 0), (0, 1), (0, 2), (0, 3)])
        .unwrap()
        .preflight()
        .unwrap();
    for counts in STAGES.with(std::cell::Cell::get) {
        assert_eq!(counts[0], 1);
        assert!(counts[1] > 0);
        assert_eq!(counts[1], counts[2]);
        assert_eq!(counts[3], 1);
    }
}

#[test]
fn selected_extents_match_drawn_ink() {
    use crate::font::canonical::OperationContext;
    use crate::render::backend::Backend;
    for source in sources() {
        let source = source
            .with_planned_x_limits((2.0, 6.0))
            .with_planned_y_limits((0.0, 5.0));
        let theme = crate::Theme::Publication.resolve().unwrap();
        let resources = OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
        let attached = inputs.attach(source.clone()).unwrap();
        let selected = attached
            .measure_selected_interior(crate::spec::PlotRenderContext::new(
                attached.data(),
                crate::layout::PanelBox::new(10.25, 410.75, 20.5, 320.25),
                &theme,
            ))
            .unwrap();
        // The parent-assigned panel is identical: only standalone root allocation
        // changes in the native migration, never mark geometry or paint policy.
        let axes = attached.prepared_axis_inputs().unwrap();
        let prior = crate::layout::Layout::new_with_axis_specs_in_panel_box(
            (2.0, 6.0),
            (0.0, 5.0),
            selected.layout.projection.panel_box(),
            axes.labels[0].clone(),
            axes.labels[1].clone(),
            &axes.specs[0],
            &axes.specs[1],
            &theme,
        )
        .unwrap();
        assert_eq!(prior.x_axis.ticks, selected.layout.x_axis.ticks);
        assert_eq!(prior.y_axis.ticks, selected.layout.y_axis.ticks);
        assert_eq!(prior.x_axis.source, selected.layout.x_axis.source);
        assert_eq!(prior.y_axis.source, selected.layout.y_axis.source);
        let project = |resources: &OperationContext| {
            let data = attached.data().with_operation(resources);
            let layout = prior.clone();
            match source.kind() {
                PlotKind::Histogram => crate::plots::histogram::project(
                    &inputs.histogram.as_ref().unwrap().panels[0].1,
                    layout,
                    &theme,
                ),
                PlotKind::Density => crate::plots::density::project(
                    &attached,
                    &inputs.density.as_ref().unwrap().panels[0].1,
                    layout,
                    &theme,
                    data,
                ),
                PlotKind::Qq => crate::plots::qq::project(
                    &attached,
                    &inputs.qq.as_ref().unwrap().panels[0].1,
                    layout,
                    &theme,
                ),
                PlotKind::Regression => crate::plots::regression::project(
                    &attached,
                    &inputs.regression.as_ref().unwrap().panels[0].1,
                    layout,
                    &theme,
                ),
                _ => unreachable!(),
            }
        };
        let resources = OperationContext::new();
        let projected = project(&resources).unwrap();

        let exact = OperationContext::new();
        project(&exact).unwrap();
        let plan = projected.marks.unwrap();
        assert_eq!(
            plan.geometry.bounds(),
            selected.marks.as_ref().unwrap().geometry.bounds()
        );
        let mut outputs = Vec::new();
        for marks in [&plan, selected.marks.as_ref().unwrap()] {
            let mut svg = crate::render::svg_backend::SvgBackend::in_memory(640, 480);
            svg.begin(640, 480).unwrap();
            marks.draw(&mut svg).unwrap();
            svg.end().unwrap();
            outputs.push(svg.into_bytes().unwrap());
        }
        assert_eq!(outputs[0], outputs[1], "{:?} retained marks", source.kind());
        let mut drawn = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
        drawn.begin(640, 480).unwrap();
        plan.draw(&mut drawn).unwrap();
        drawn.end().unwrap();
        assert_eq!(plan.geometry.bounds(), drawn.bounds());
    }
}

#[test]
fn native_scientific_ranges_retain_full_sample_statistics_across_assignments() {
    use crate::font::canonical::OperationContext;
    use crate::layout::interior::{InteriorRoute, StandardCoordinate};
    let routes = [
        StandardCoordinate::Histogram,
        StandardCoordinate::Density,
        StandardCoordinate::Qq,
        StandardCoordinate::Regression,
    ];
    for (family, source) in sources().into_iter().enumerate() {
        let resources = OperationContext::new();
        let data = source.data().with_operation(&resources);
        let expected = match source.kind() {
            PlotKind::Histogram => crate::plots::histogram::axis_ranges(&source, data),
            PlotKind::Density => crate::plots::density::axis_ranges(&source, data),
            PlotKind::Qq => Ok([
                crate::layout::continuous_auto_range(
                    (
                        crate::stats::distribution::standard_normal_quantile(0.5 / 6.0).unwrap(),
                        crate::stats::distribution::standard_normal_quantile(5.5 / 6.0).unwrap(),
                    ),
                    source.x_axis_spec(),
                )
                .unwrap(),
                crate::layout::continuous_auto_range((1.0, 8.0), source.y_axis_spec()).unwrap(),
            ]),
            PlotKind::Regression => crate::plots::regression::axis_ranges(&source, data),
            _ => unreachable!(),
        }
        .unwrap();
        STAGES.with(|counts| counts.set([[0; 4]; 4]));
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        assert_eq!(
            inputs.route,
            InteriorRoute::StandardCoordinate(routes[family])
        );
        let plot = inputs.attach(source).unwrap();
        assert_eq!(plot.prepared_axis_inputs().unwrap().ranges, expected);
        let theme = crate::Theme::Publication.resolve().unwrap();
        let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
        for panel in [
            crate::layout::PanelBox::new(30.0, 610.0, 50.0, 440.0),
            crate::layout::PanelBox::new(40.25, 340.75, 70.5, 280.25),
        ] {
            for ranges in [expected, [(-0.5, 0.5), (2.0, 6.0)]] {
                let assigned = plot
                    .clone()
                    .with_planned_x_limits(ranges[0])
                    .with_planned_y_limits(ranges[1]);
                let selected = assigned
                    .measure_selected_interior(crate::spec::PlotRenderContext::new(
                        assigned.data(),
                        panel,
                        &theme,
                    ))
                    .unwrap();
                assert_eq!(selected.layout.projection.x_range(), ranges[0]);
                assert_eq!(selected.layout.projection.y_range(), ranges[1]);
                assert_eq!(plot.prepared_axis_inputs().unwrap().ranges, expected);
                assert!(std::sync::Arc::ptr_eq(
                    &plot.prepared_axes,
                    &assigned.prepared_axes
                ));
                assert_eq!(STAGES.with(std::cell::Cell::get)[family][0], 1);
                assert_eq!(
                    crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                    reads
                );
            }
        }
    }
}

#[test]
fn native_facets_and_shared_canvas_remeasure_actual_component_feedback_without_repreparation() {
    use crate::decorations::plan::children::DISABLE_CHILD_REUSE;
    use crate::orchestrator::measurements::{self, Counter};
    fn configured<K: crate::kind::ExactFamily + crate::kind::Facet>(
        plot: Plot<K>,
        faceted: bool,
    ) -> (Plot, Option<crate::Canvas>) {
        let plot = plot
            .x_label("First axis line\nSecond axis line\nThird axis line")
            .size(960, 640);
        if faceted {
            (
                plot.facet(crate::FacetSpec::column(
                    crate::CategoryValues::new("panel", ["a", "a", "a", "b", "b", "b"]).unwrap(),
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
                .size(960, 640);
            (plot.erase(), Some(canvas))
        }
    }
    for faceted in [false, true] {
        let x = NumericValues::new("sample", [1.0, 2.0, 4.0, 5.0, 7.0, 8.0]);
        let plots = [
            configured(Plot::histogram(HistogramInput::samples(x.clone())), faceted),
            configured(Plot::density(x.clone()), faceted),
            configured(Plot::qq(x.clone()), faceted),
            configured(
                Plot::regression(
                    x,
                    NumericValues::new("response", [1.0, 4.0, 2.0, 8.0, 6.0, 9.0]),
                ),
                faceted,
            ),
        ];
        for (family, (source, canvas)) in plots.into_iter().enumerate() {
            let mut outputs = Vec::new();
            for disabled in [false, true] {
                STAGES.with(|counts| counts.set([[0; 4]; 4]));
                DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
                measurements::start();
                let mut svg = crate::render::svg_backend::SvgBackend::in_memory(960, 640);
                let result = if let Some(canvas) = &canvas {
                    canvas.render_with_backend(&mut svg)
                } else {
                    source.render_with_backend(&mut svg)
                };
                let work = measurements::finish();
                DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
                result.unwrap();
                assert!(work.gutters.iter().any(|change| {
                    change.owner != crate::layout::ComponentAddress::root()
                        && change.consumed.map(f64::to_bits) != change.requested.map(f64::to_bits)
                }));
                assert!(work.roots.len() >= 2);
                assert_eq!(work.counters[Counter::LegacyPanelConstructor as usize], 0);
                assert!(work.counters[Counter::StandardCoordinateMeasure as usize] > 0);
                // Every changed assignment may be a miss; the retained-child
                // trial test separately forces hits and rejected ancestors.
                if disabled {
                    assert_eq!(work.counters[Counter::ChildCacheHit as usize], 0);
                    assert_eq!(work.counters[Counter::ChildFeedbackReplay as usize], 0);
                }
                let counts = STAGES.with(std::cell::Cell::get)[family];
                assert_eq!(counts[0], 2);
                assert_eq!(counts[1], counts[2]);
                assert_eq!(counts[3], 2);
                outputs.push(svg.into_bytes().unwrap());
            }
            assert_eq!(outputs[0], outputs[1]);
        }
    }
}

#[test]
fn invalid_statistics_and_exact_sources_reject_before_unavailable_fonts() {
    let invalid = [
        Plot::histogram(HistogramInput::samples(NumericValues::new(
            "value", [1.0; 3],
        )))
        .stat(crate::HistogramStat::Density)
        .composite(crate::HistogramComposite::KdeAndRug(
            crate::HistogramKde::Internal,
        ))
        .erase(),
        Plot::density(NumericValues::new("value", [1.0; 3])).erase(),
        Plot::qq(NumericValues::new("value", [-f64::MAX, f64::MAX])).erase(),
        Plot::regression(
            NumericValues::new("x", [1.0, 2.0]),
            NumericValues::new("y", [1.0, 2.0]),
        )
        .erase(),
    ];
    for source in invalid.into_iter().chain(
        sources()
            .into_iter()
            .chain(super::estimates::sources())
            .map(|source| {
                source
                    .with_planned_x_limits((0.0, 1.0))
                    .with_test_axis_ticks(0, vec![crate::AxisTick::new(2.0, "outside")])
            }),
    ) {
        let mut theme = crate::Theme::Publication.resolve().unwrap();
        theme.typography.axes.tick_label.stack =
            crate::font::canonical::named_stack(&["Lichtung unavailable T5a6a3b 901e"]);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
        let result = crate::layout::plan_exact_root(
            640,
            480,
            |stage| {
                crate::plots::shared_x::prepare_plot(&source, &stage.operation(), None)?;
                let requirements = crate::layout::foundational_plot_requirements(
                    None,
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
                .map_err(|e| Error::Render(e.to_string()))?;
                theme.prepare_font_environment_from_census(&mut census, presence, stage)
            },
            |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |(), _, (), _| Ok(()),
        );
        assert!(matches!(result, Err(Error::Data(_))));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "exercise preparation, selected measurement, poisoned capture and both replays under one root"
)]
fn selected_capture_and_replay_do_no_source_projection_statistics_axis_or_shaping_work() {
    use crate::orchestrator::measurements::{self, Counter};
    use crate::spec::PlotRenderContext;
    for (family, source) in sources().into_iter().enumerate() {
        STAGES.with(|counts| counts.set([[0; 4]; 4]));
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|count| count.set(0));
        crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS.with(|count| count.set(0));
        let theme = crate::Theme::Publication.resolve().unwrap();
        let root = crate::layout::plan_exact_root(
            640,
            480,
            |stage| {
                let inputs =
                    crate::plots::shared_x::prepare_plot(&source, &stage.operation(), None)?;
                let requirements = crate::layout::foundational_plot_requirements(
                    None,
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
                .map_err(|e| Error::Render(e.to_string()))?;
                theme.prepare_font_environment_from_census(&mut census, presence, stage)?;
                Ok(inputs)
            },
            |inputs, assigned, _| {
                let attached = inputs.attach(source.clone())?;
                let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
                let result = attached
                    .measure_selected_interior(PlotRenderContext::new(
                        attached.data(),
                        crate::layout::PanelBox::new(80.25, 550.75, 70.5, 410.25),
                        &theme,
                    ))
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                assert_eq!(
                    crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                    reads,
                    "{:?} measurement reread columns",
                    source.kind()
                );
                assert!(!result.marks.as_ref().unwrap().geometry.bounds().is_empty());
                Ok(crate::layout::MeasuredLayout::new(assigned, result))
            },
            |_, _, selected, stage| {
                let counts = STAGES.with(std::cell::Cell::get);
                let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
                let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
                let projections = crate::layout::Projection::test_projection_calls();
                let invalid = NumericValues::new("unusable", [f64::INFINITY]);
                let poison = match source.kind() {
                    PlotKind::Histogram => {
                        Plot::histogram(HistogramInput::samples(invalid)).erase()
                    }
                    PlotKind::Density => Plot::density(invalid).erase(),
                    PlotKind::Qq => Plot::qq(invalid).erase(),
                    PlotKind::Regression => Plot::regression(invalid.clone(), invalid).erase(),
                    _ => unreachable!(),
                };
                measurements::start();
                let captured = crate::orchestrator::capture_text_replay_plan(
                    640,
                    480,
                    "selected distribution/reference",
                    &theme,
                    stage,
                    |backend| {
                        let context = PlotRenderContext::new(
                            poison.data(),
                            selected.layout.projection.panel_box(),
                            &theme,
                        )
                        .with_retained_body(&selected);
                        poison.render_panel(source.kind(), context, backend)?;
                        selected.axes.as_ref().unwrap().draw(backend)
                    },
                )?;
                let work = measurements::finish();
                assert_eq!(
                    crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                    reads
                );
                assert_eq!(
                    crate::layout::Projection::test_projection_calls(),
                    projections
                );
                assert_eq!(
                    crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
                    shapes
                );
                let after = STAGES.with(std::cell::Cell::get)[family];
                assert_eq!(&after[..3], &counts[family][..3]);
                assert_eq!(after[3], counts[family][3] + 1);
                for counter in [
                    Counter::StandardPanelIntrinsic,
                    Counter::StandardCoordinateMeasure,
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
                Ok(captured)
            },
        )
        .unwrap();
        let counts = STAGES.with(std::cell::Cell::get);
        assert_eq!(counts[family], [1, 1, 1, 1]);
        assert_eq!(
            crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
            0
        );
        assert_eq!(
            crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS.with(std::cell::Cell::get),
            0
        );
        crate::orchestrator::validate_text_replay_plan(
            640,
            480,
            root.plan(),
            "reference Null replay",
        )
        .unwrap();
        let mut backend = root.plan().take_svg_backend().unwrap();
        root.plan()
            .replay(640, 480, &mut backend, "reference SVG replay")
            .unwrap();
        assert_eq!(STAGES.with(std::cell::Cell::get), counts);
    }
}
