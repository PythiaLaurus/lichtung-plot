use super::*;
use crate::font::canonical::OperationGenerationResources;
use crate::{CategoryValues, NumericValues};

fn configured<
    K: crate::kind::Marker
        + crate::kind::Facet
        + crate::kind::Guides
        + crate::kind::PanelText
        + crate::kind::Legend,
>(
    mut plot: Plot<K>,
    facet: bool,
    guide: Option<f64>,
) -> Plot {
    if facet {
        plot = plot.facet(crate::FacetSpec::column(
            CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap(),
        ));
    }
    if let Some(y) = guide {
        plot = plot
            .guide(crate::GuideSpec::horizontal(
                y,
                crate::GuideStyle::new((10, 20, 30), 1.0, crate::LinePattern::Dashed).unwrap(),
            ))
            .panel_text_annotation(
                crate::PanelTextAnnotation::new(
                    crate::PanelCorner::TopLeft,
                    "retained panel label",
                )
                .unwrap(),
            )
            .legend(crate::LegendPolicy::Hidden);
    }
    crate::kind::into_erased(plot)
}

fn sources(facet: bool, guide: Option<f64>) -> [Plot; 4] {
    let x = NumericValues::new("x", [2.0, 1.0, 4.0, 3.0]);
    let y = NumericValues::new("y", [4.0, 2.0, 8.0, 6.0]);
    let group = CategoryValues::new("group", ["b", "b", "a", "a"]).unwrap();
    let hue = CategoryValues::new("hue", ["blue", "blue", "red", "red"]).unwrap();
    [
        configured(
            Plot::line(x.clone(), y.clone())
                .group(group.clone())
                .hue(hue.clone()),
            facet,
            guide,
        ),
        configured(
            Plot::trajectory(x.clone(), y.clone(), group.clone()).hue(hue.clone()),
            facet,
            guide,
        ),
        configured(
            Plot::trajectory(
                CategoryValues::new("condition", [" after ", "before", "after", "later"]).unwrap(),
                y.clone(),
                group.clone(),
            )
            .hue(hue.clone()),
            facet,
            guide,
        ),
        configured(Plot::line_path(x, y, group).hue(hue), facet, guide),
    ]
}

fn decorated_line() -> Plot {
    configured(
        Plot::line(
            NumericValues::new("x", [2.0, 1.0]),
            NumericValues::new("y", [4.0, 2.0]),
        )
        .hue(CategoryValues::new("hue", ["one", "one"]).unwrap())
        .band(crate::Bounds::new(
            NumericValues::new("lower", [3.0, 1.0]),
            NumericValues::new("upper", [5.0, 3.0]),
        ))
        .point_layer(
            crate::PointLayer::new(
                NumericValues::new("px", [1.5]),
                NumericValues::new("py", [3.0]),
            )
            .labels(CategoryValues::new("label", ["retained point label"]).unwrap())
            .legend_label("observations"),
        ),
        false,
        Some(3.0),
    )
}

#[test]
fn semantic_order_ranges_conditions_and_exact_admission_precede_fonts() {
    for plot in sources(false, None).into_iter().chain([decorated_line()]) {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let resources = OperationGenerationResources::new();
        let prepared =
            PreparedLine::new(&plot, plot.data().with_axis_resources(&resources), None).unwrap();
        assert!(prepared.groups.iter().all(|group| !group.points.is_empty()));
        if plot.kind() == PlotKind::LinePath {
            assert_eq!(
                prepared.groups[0]
                    .points
                    .iter()
                    .map(|point| point.x)
                    .collect::<Vec<_>>(),
                [2.0, 1.0]
            );
        } else {
            assert!(prepared
                .groups
                .iter()
                .all(|group| group.points.windows(2).all(|pair| pair[0].x < pair[1].x)));
        }
        if let Some(levels) = &prepared.conditions {
            assert_eq!(levels, &["after", "before", "later"]);
            assert_eq!(prepared.ranges[0], (0.0, 3.0));
        }
        let visits = resources.usage().unwrap().1;
        let bytes = resources.byte_usage().unwrap();
        for (visits, bytes) in [(visits - 1, bytes), (visits, bytes - 1)] {
            let limited = OperationGenerationResources::with_test_limits(visits, bytes);
            assert!(
                PreparedLine::new(&plot, plot.data().with_axis_resources(&limited), None).is_err()
            );
        }
        let exact = OperationGenerationResources::with_test_limits(visits, bytes);
        PreparedLine::new(&plot, plot.data().with_axis_resources(&exact), None).unwrap();
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}

#[test]
fn each_facet_and_shared_canvas_component_prepares_once_before_measurement() {
    fn canvas<K: crate::kind::ExactFamily>(plot: Plot<K>) -> crate::Canvas {
        crate::Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, plot.clone())
            .unwrap()
            .panel(0, 1, plot)
            .unwrap()
            .x_range_group([(0, 0), (0, 1)])
            .unwrap()
    }
    for facets in sources(true, None) {
        let facets = facets.size(960, 640);
        STAGES.with(|stages| stages.set([[0; 4]; 3]));
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
        facets.preflight().unwrap();
        let counts = STAGES.with(std::cell::Cell::get);
        assert_eq!(counts.iter().map(|counts| counts[0]).sum::<usize>(), 2);
        assert_eq!(counts.iter().map(|counts| counts[3]).sum::<usize>(), 2);
        assert!(counts.iter().all(|counts| counts[1] == counts[2]));
    }
    let x = || NumericValues::new("x", [2.0, 1.0]);
    let y = || NumericValues::new("y", [4.0, 2.0]);
    let group = || CategoryValues::new("group", ["one", "one"]).unwrap();
    for canvas in [
        canvas(Plot::line(x(), y())),
        canvas(Plot::trajectory(x(), y(), group())),
        canvas(Plot::line_path(x(), y(), group())),
    ] {
        STAGES.with(|stages| stages.set([[0; 4]; 3]));
        canvas.size(960, 640).preflight().unwrap();
        let counts = STAGES.with(std::cell::Cell::get);
        assert_eq!(counts.iter().map(|counts| counts[0]).sum::<usize>(), 2);
        assert_eq!(counts.iter().map(|counts| counts[3]).sum::<usize>(), 2);
        assert!(counts.iter().all(|counts| counts[1] == counts[2]));
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "root preparation, selected measurement, capture and replay are independently observed for every line family"
)]
fn selected_capture_never_prepares_projects_reads_sources_or_shapes_and_replays_without_work() {
    use crate::orchestrator::measurements::{self, Counter};
    for plot in sources(false, Some(3.0))
        .into_iter()
        .chain([decorated_line()])
    {
        let plot = plot.size(640, 480);
        let theme = crate::Theme::Publication.resolve();
        let kind = plot.kind();
        STAGES.with(|stages| stages.set([[0; 4]; 3]));
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
        let root = crate::layout::plan_exact_root(
            640,
            480,
            |stage| {
                let inputs =
                    crate::plots::shared_x::prepare_plot(&plot, &stage.generation_resources())?;
                let requirements = crate::layout::foundational_plot_requirements(
                    None,
                    &plot,
                    kind,
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
                theme.prepare_font_environment_from_census(&mut census, presence, stage)?;
                Ok(inputs)
            },
            |inputs, assigned, _| {
                let attached = inputs.attach(plot.clone())?;
                assert!(std::sync::Arc::ptr_eq(
                    attached.prepared_line.as_ref().unwrap(),
                    inputs.line.as_ref().unwrap()
                ));
                let selected = attached
                    .measure_selected_interior(PlotRenderContext::new(
                        attached.data(),
                        crate::layout::PanelBox::new(80.25, 550.75, 70.5, 410.25),
                        &theme,
                    ))
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                assert!(!selected
                    .marks
                    .as_ref()
                    .unwrap()
                    .geometry
                    .bounds()
                    .is_empty());
                Ok(crate::layout::MeasuredLayout::new(assigned, selected))
            },
            |_, _, selected, stage| {
                let counts = STAGES.with(std::cell::Cell::get);
                let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
                let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
                let projections = crate::layout::Projection::test_projection_calls();
                let poison = match kind {
                    PlotKind::Line => Plot::line(
                        NumericValues::new("x", [f64::NAN]),
                        NumericValues::new("y", [f64::INFINITY]),
                    )
                    .erase(),
                    PlotKind::Trajectory => Plot::trajectory(
                        NumericValues::new("x", [f64::NAN]),
                        NumericValues::new("y", [f64::INFINITY]),
                        CategoryValues::new("group", ["poison"]).unwrap(),
                    )
                    .erase(),
                    PlotKind::LinePath => Plot::line_path(
                        NumericValues::new("x", [f64::NAN]),
                        NumericValues::new("y", [f64::INFINITY]),
                        CategoryValues::new("group", ["poison"]).unwrap(),
                    )
                    .erase(),
                    _ => unreachable!(),
                };
                measurements::start();
                let captured = crate::orchestrator::capture_text_replay_plan(
                    640,
                    480,
                    "selected line capture",
                    &theme,
                    stage,
                    |backend| {
                        poison.render_panel(
                            kind,
                            PlotRenderContext::new(
                                poison.data(),
                                selected.layout.projection.panel_box(),
                                &theme,
                            )
                            .with_retained_body(selected.retained_payload()),
                            backend,
                        )?;
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
                let after = STAGES.with(std::cell::Cell::get);
                for (before, after) in counts.iter().zip(after) {
                    assert_eq!(&before[..3], &after[..3]);
                }
                assert_eq!(
                    after.iter().map(|counts| counts[3]).sum::<usize>(),
                    counts.iter().map(|counts| counts[3]).sum::<usize>() + 1
                );
                assert_eq!(
                    crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
                    shapes
                );
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
                    assert_eq!(work.counters[counter as usize], 0, "{kind:?}: {counter:?}");
                }
                assert!(captured
                    .retained_text()
                    .iter()
                    .any(|(_, plan)| plan.source == "retained panel label"));
                if !plot.point_layers().is_empty() {
                    assert!(captured
                        .retained_text()
                        .iter()
                        .any(|(_, plan)| plan.source == "retained point label"));
                }
                Ok(captured)
            },
        )
        .unwrap();
        assert_eq!(
            crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
            0
        );
        let counts = STAGES.with(std::cell::Cell::get);
        assert_eq!(counts.iter().map(|counts| counts[0]).sum::<usize>(), 1);
        crate::orchestrator::validate_text_replay_plan(
            640,
            480,
            root.plan(),
            "selected line Null replay",
        )
        .unwrap();
        let mut backend = root.plan().take_svg_backend().unwrap();
        root.plan()
            .replay(640, 480, &mut backend, "selected line SVG replay")
            .unwrap();
        assert_eq!(STAGES.with(std::cell::Cell::get), counts);
    }
}

#[test]
fn invalid_semantics_and_missing_preparation_reject_without_fonts() {
    for (plot, invalid) in sources(false, None)
        .into_iter()
        .zip(sources(false, Some(f64::INFINITY)))
    {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let theme = crate::Theme::Publication.resolve();
        let context = PlotRenderContext::new(
            plot.data(),
            crate::layout::PanelBox::new(0.0, 100.0, 0.0, 100.0),
            &theme,
        );
        assert!(plot
            .measure_selected_interior(context)
            .unwrap_err()
            .into_error()
            .to_string()
            .contains("root-prepared"));
        let invalid = invalid.size(1, 1);
        assert!(matches!(invalid.preflight(), Err(Error::Data(_))));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
    let invalid = [
        Plot::line(
            NumericValues::new("x", [0.0, -0.0]),
            NumericValues::new("y", [1.0, 2.0]),
        )
        .erase(),
        Plot::trajectory(
            NumericValues::new("x", [0.0, -0.0]),
            NumericValues::new("y", [1.0, 2.0]),
            CategoryValues::new("group", ["one"; 2]).unwrap(),
        )
        .erase(),
        Plot::trajectory(
            CategoryValues::new("x", [" after ", "after"]).unwrap(),
            NumericValues::new("y", [1.0, 2.0]),
            CategoryValues::new("group", ["one"; 2]).unwrap(),
        )
        .erase(),
        Plot::line_path(
            NumericValues::new("x", [1.0]),
            NumericValues::new("y", [2.0]),
            CategoryValues::new("group", ["one"]).unwrap(),
        )
        .erase(),
        Plot::line(
            NumericValues::new("x", [1.0, 2.0]),
            NumericValues::new("y", [1.0, 2.0]),
        )
        .band(crate::Bounds::new(
            NumericValues::new("lower", [0.0, 3.0]),
            NumericValues::new("upper", [2.0, 4.0]),
        ))
        .erase(),
        Plot::line(
            NumericValues::new("x", [1.0, 2.0]),
            NumericValues::new("y", [1.0, 2.0]),
        )
        .point_layer(crate::PointLayer::new(
            NumericValues::new("px", [f64::NAN]),
            NumericValues::new("py", [1.0]),
        ))
        .erase(),
    ];
    for plot in invalid {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        assert!(matches!(plot.size(1, 1).preflight(), Err(Error::Data(_))));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}

#[test]
fn projected_buffers_share_exact_admission_and_payload_geometry_matches_clipped_ink() {
    for plot in sources(false, None).into_iter().chain([decorated_line()]) {
        let theme = crate::Theme::Publication.resolve();
        let resources = OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&plot, &resources).unwrap();
        let attached = inputs.attach(plot.clone()).unwrap();
        let selected = attached
            .measure_selected_interior(PlotRenderContext::new(
                attached.data(),
                crate::layout::PanelBox::new(10.25, 410.75, 20.5, 320.25),
                &theme,
            ))
            .unwrap();
        let prepared = &inputs.line.as_ref().unwrap().panels[0].1;
        let resources = OperationGenerationResources::new();
        let selected = project(
            &attached,
            prepared,
            selected.layout,
            &theme,
            attached.data().with_axis_resources(&resources),
        )
        .unwrap();
        let visits = resources.usage().unwrap().1;
        let bytes = resources.byte_usage().unwrap();
        for (visits, bytes) in [(visits - 1, bytes), (visits, bytes - 1)] {
            let limited = OperationGenerationResources::with_test_limits(visits, bytes);
            assert!(project(
                &attached,
                prepared,
                selected.layout.clone(),
                &theme,
                attached.data().with_axis_resources(&limited)
            )
            .is_err());
        }
        let exact = OperationGenerationResources::with_test_limits(visits, bytes);
        project(
            &attached,
            prepared,
            selected.layout.clone(),
            &theme,
            attached.data().with_axis_resources(&exact),
        )
        .unwrap();
        let marks = selected.marks.as_ref().unwrap();
        let mut legacy = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
        legacy.begin(640, 480).unwrap();
        marks.draw(&mut legacy).unwrap();
        legacy.end().unwrap();
        assert_eq!(marks.geometry.bounds(), legacy.bounds());
        let clip =
            crate::render::backend::validate_clip_region(selected.layout.projection.panel_box())
                .unwrap()
                .panel();
        assert!(marks.geometry.bounds().iter().all(|ink| ink.left >= clip.x0
            && ink.right <= clip.x1
            && ink.bottom >= clip.y0
            && ink.top <= clip.y1));
    }
}

#[test]
fn sparse_intervals_do_not_create_empty_series_and_path_policies_keep_draw_order() {
    let plot = decorated_line().with_global_collective_pairs(vec![
        CollectivePair {
            group: None,
            hue: Some("absent".into()),
        },
        CollectivePair {
            group: None,
            hue: Some("one".into()),
        },
    ]);
    let resources = OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&plot, &resources).unwrap();
    let prepared = &inputs.line.as_ref().unwrap().panels[0].1;
    assert_eq!(prepared.groups.len(), 1);
    assert!(prepared.groups[0]
        .points
        .iter()
        .all(|point| point.interval.is_some()));
    let plot = inputs.attach(plot).unwrap();
    let theme = crate::Theme::Publication.resolve();
    let selected = plot
        .measure_selected_interior(PlotRenderContext::new(
            plot.data(),
            crate::layout::PanelBox::new(20.25, 400.75, 20.25, 300.75),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Line(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("wrong payload")
    };
    assert!(matches!(
        payload.marks.as_slice(),
        [
            LineMark::Ribbon(..),
            LineMark::Line(..),
            LineMark::Points(..)
        ]
    ));
    assert_eq!(payload.layers.len(), 1);
    assert_eq!(payload.layers[0].2.len(), 1);
    for hue_paths in [false, true] {
        for markers in [false, true] {
            let plot = Plot::line_path(
                NumericValues::new("x", [2.0, 1.0, 1.0]),
                NumericValues::new("y", [4.0, 2.0, 3.0]),
                CategoryValues::new("group", ["one"; 3]).unwrap(),
            )
            .hue(CategoryValues::new("hue", ["one"; 3]).unwrap())
            .markers(markers)
            .style(if hue_paths {
                LinePathStyle::Hue
            } else {
                LinePathStyle::Neutral
            })
            .erase();
            let inputs = crate::plots::shared_x::prepare_plot(&plot, &resources).unwrap();
            let plot = inputs.attach(plot).unwrap();
            let selected = plot
                .measure_selected_interior(PlotRenderContext::new(
                    plot.data(),
                    crate::layout::PanelBox::new(20.25, 400.75, 20.25, 300.75),
                    &theme,
                ))
                .unwrap();
            let crate::layout::interior::MarkPayload::Line(payload) =
                &selected.marks.as_ref().unwrap().payload
            else {
                panic!("wrong payload")
            };
            let LineMark::Line(points, _) = &payload.marks[0] else {
                panic!("connector must paint first")
            };
            assert!(points[0].x > points[1].x);
            assert_eq!(points[1].x.to_bits(), points[2].x.to_bits());
            assert_eq!(payload.marks.len(), if markers { 4 } else { 1 });
            assert_eq!(
                selected.legend_entries.len(),
                usize::from(markers || hue_paths)
            );
        }
    }
}
