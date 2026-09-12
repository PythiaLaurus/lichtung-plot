use super::*;
use crate::font::canonical::OperationGenerationResources;
use crate::spec::PlotRenderContext;
use crate::{CategoryValues, NumericValues, RaincloudInput};

fn sources(facet: Option<crate::FacetScales>) -> [Plot; 3] {
    fn finish<K: crate::kind::ExactFamily + crate::kind::Facet>(
        mut plot: Plot<K>,
        facet: Option<crate::FacetScales>,
    ) -> Plot {
        if let Some(scales) = facet {
            plot = plot.facet(
                crate::FacetSpec::column(
                    CategoryValues::new(
                        "panel",
                        [
                            "left", "left", "left", "left", "right", "right", "right", "right",
                        ],
                    )
                    .unwrap(),
                )
                .scales(scales),
            );
        }
        plot.size(1100, 640).erase()
    }
    let categories =
        CategoryValues::new("category", ["b", "b", "a", "a", "b", "b", "a", "a"]).unwrap();
    let values = NumericValues::new("value", [1.0, 4.0, 2.0, 7.0, 3.0, 6.0, 4.0, 9.0]);
    let hues = CategoryValues::new(
        "color",
        ["one", "one", "two", "two", "one", "one", "two", "two"],
    )
    .unwrap();
    [
        finish(
            Plot::boxplot(categories.clone(), values.clone())
                .series(hues.clone())
                .raw_overlay(crate::RawOverlay::Jittered),
            facet,
        ),
        finish(
            Plot::violin(categories.clone(), values.clone()).series(hues.clone()),
            facet,
        ),
        finish(
            Plot::raincloud(RaincloudInput::samples(categories, values)).series(hues),
            facet,
        ),
    ]
}

fn supplied() -> Plot {
    Plot::raincloud(RaincloudInput::geometry(crate::RaincloudGeometry {
        dimensions: crate::RaincloudDimensions {
            category: "category".into(),
            value: "value".into(),
            hue: Some("color".into()),
        },
        groups: vec![crate::RaincloudGroup {
            category: "supplied".into(),
            hue: Some("one".into()),
            density: vec![
                crate::RaincloudDensityPoint {
                    y: 0.0,
                    normalized_width: 0.0,
                },
                crate::RaincloudDensityPoint {
                    y: 3.0,
                    normalized_width: 0.8,
                },
                crate::RaincloudDensityPoint {
                    y: 6.0,
                    normalized_width: 0.0,
                },
            ],
            q1: 1.0,
            median: 3.0,
            q3: 5.0,
            points: vec![
                crate::RaincloudPoint {
                    y: 2.0,
                    normalized_strip: 0.25,
                },
                crate::RaincloudPoint {
                    y: 4.0,
                    normalized_strip: 0.75,
                },
            ],
        }],
    }))
    .erase()
}

fn annotated() -> Plot {
    Plot::violin(
        CategoryValues::new("category", ["a", "a", "b", "b"]).unwrap(),
        NumericValues::new("value", [1.0, 3.0, 2.0, 5.0]),
    )
    .significance([crate::Significance::between(
        "a",
        "b",
        crate::SignificanceMark::PValue(0.03),
    )])
    .guide(crate::GuideSpec::horizontal(
        0.0,
        crate::GuideStyle::new((90, 80, 70), 1.0, crate::LinePattern::Solid).unwrap(),
    ))
    .panel_text_annotation(
        crate::PanelTextAnnotation::new(crate::PanelCorner::BottomLeft, "retained annotation")
            .unwrap(),
    )
    .y_limits(-10.0, 15.0)
    .y_ticks(vec![
        crate::AxisTick::new(-10.0, "low"),
        crate::AxisTick::new(0.0, "zero"),
        crate::AxisTick::new(15.0, "high"),
    ])
    .erase()
}

#[test]
fn semantics_are_admitted_once_before_fonts_and_shared_on_attachment() {
    for source in sources(None)
        .into_iter()
        .chain(sources(Some(crate::FacetScales::Fixed)))
        .chain([supplied()])
    {
        STAGES.with(|counts| counts.set([[0; 4]; 3]));
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
        let resources = OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        assert_eq!(
            STAGES
                .with(std::cell::Cell::get)
                .iter()
                .map(|count| count[0])
                .sum::<usize>(),
            if source.has_facets() { 2 } else { 1 }
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
            attached.prepared_distributions.as_ref().unwrap(),
            inputs.distributions.as_ref().unwrap()
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}

#[test]
fn supplied_inputs_share_shapes_across_plot_clones() {
    let raincloud = supplied();
    let clone = raincloud.clone();
    assert!(std::ptr::eq(
        raincloud.raincloud_geometry_input().unwrap(),
        clone.raincloud_geometry_input().unwrap()
    ));
    let histogram = Plot::histogram(crate::HistogramInput::geometry(crate::HistogramGeometry {
        dimensions: crate::HistogramDimensions {
            value: "value".into(),
            density: "density".into(),
        },
        bins: vec![crate::HistogramBin {
            lower: 0.0,
            upper: 1.0,
            density: 1.0,
        }],
        kde: Vec::new(),
        rugs: Vec::new(),
    }));
    let clone = histogram.clone();
    assert!(std::ptr::eq(
        histogram.histogram_composite_geometry_input().unwrap(),
        clone.histogram_composite_geometry_input().unwrap()
    ));
}

#[test]
fn invalid_sample_statistics_fail_before_font_discovery() {
    let category = CategoryValues::new("category", ["a", "a"]).unwrap();
    for source in [
        Plot::boxplot(
            category.clone(),
            NumericValues::new("value", [1.0, f64::INFINITY]),
        )
        .erase(),
        Plot::violin(category.clone(), NumericValues::new("value", [4.0, 4.0])).erase(),
        Plot::raincloud(RaincloudInput::samples(
            category,
            NumericValues::new("value", [4.0, 4.0]),
        ))
        .erase(),
    ] {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
        assert!(matches!(
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new()),
            Err(Error::Data(_))
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}

#[test]
fn standalone_facets_and_canvas_retain_component_statistics() {
    for scales in [
        None,
        Some(crate::FacetScales::Fixed),
        Some(crate::FacetScales::FreeX),
        Some(crate::FacetScales::FreeY),
        Some(crate::FacetScales::Free),
    ] {
        for source in sources(scales)
            .into_iter()
            .chain(scales.is_none().then(supplied))
        {
            STAGES.with(|counts| counts.set([[0; 4]; 3]));
            source.preflight().unwrap();
            let counts = STAGES.with(std::cell::Cell::get);
            let active = counts.iter().find(|count| count[0] != 0).unwrap();
            let expected = if scales.is_some() { 2 } else { 1 };
            assert_eq!(active[0], expected);
            assert!(active[1] > 0);
            assert_eq!(active[1], active[2]);
            assert_eq!(active[3], expected);
        }
    }
    let categories = CategoryValues::new("category", ["a", "a", "b", "b"]).unwrap();
    let values = NumericValues::new("value", [1.0, 3.0, 2.0, 5.0]);
    let boxplot = Plot::boxplot(categories.clone(), values.clone());
    let violin = Plot::violin(categories.clone(), values.clone());
    let raincloud = Plot::raincloud(RaincloudInput::samples(categories, values));
    STAGES.with(|counts| counts.set([[0; 4]; 3]));
    crate::Canvas::grid(1, 3)
        .unwrap()
        .size(1800, 640)
        .panel(0, 0, boxplot)
        .unwrap()
        .panel(0, 1, violin)
        .unwrap()
        .panel(0, 2, raincloud)
        .unwrap()
        .preflight()
        .unwrap();
    for count in STAGES.with(std::cell::Cell::get) {
        assert_eq!(count[0], 1);
        assert_eq!(count[1], count[2]);
        assert_eq!(count[3], 1);
    }
}

#[test]
fn selected_mark_snapshot_matches_drawn_geometry() {
    for source in sources(None).into_iter().chain([supplied(), annotated()]) {
        let theme = crate::Theme::Publication.resolve();
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let attached = inputs.attach(source).unwrap();
        let selected = attached
            .measure_selected_interior(PlotRenderContext::new(
                attached.data(),
                crate::layout::PanelBox::new(20.25, 520.75, 30.5, 350.25),
                &theme,
            ))
            .unwrap();
        let marks = selected.marks.as_ref().unwrap();
        let mut drawn = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
        drawn.begin(640, 480).unwrap();
        marks.draw(&mut drawn).unwrap();
        drawn.end().unwrap();
        assert_eq!(marks.geometry.bounds(), drawn.bounds());
        let annotations = selected.annotations.as_ref().unwrap();
        let mut drawn = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
        drawn.begin(640, 480).unwrap();
        crate::significance::render_selected(&attached, &selected, &theme, &mut drawn).unwrap();
        annotations.draw(&mut drawn).unwrap();
        drawn.end().unwrap();
        assert_eq!(annotations.geometry.bounds(), drawn.bounds());
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one operation proves source-free selected capture and frozen replays"
)]
fn poisoned_capture_repeats_no_sources_statistics_projection_axes_or_shaping() {
    use crate::orchestrator::measurements::{self, Counter};
    for source in sources(None).into_iter().chain([supplied(), annotated()]) {
        STAGES.with(|counts| counts.set([[0; 4]; 3]));
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|count| count.set(0));
        let theme = crate::Theme::Publication.resolve();
        let root = crate::layout::plan_exact_root(
            640,
            480,
            |stage| {
                let inputs =
                    crate::plots::shared_x::prepare_plot(&source, &stage.generation_resources())?;
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
                .map_err(|error| Error::Render(error.to_string()))?;
                theme.prepare_font_environment_from_census(&mut census, presence, stage)?;
                Ok(inputs)
            },
            |inputs, assigned, _| {
                let attached = inputs.attach(source.clone())?;
                let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
                let selected = attached
                    .measure_selected_interior(PlotRenderContext::new(
                        attached.data(),
                        crate::layout::PanelBox::new(80.25, 550.75, 70.5, 410.25),
                        &theme,
                    ))
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                assert_eq!(
                    crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                    reads
                );
                Ok(crate::layout::MeasuredLayout::new(assigned, selected))
            },
            |_, _, selected, stage| {
                let categories = CategoryValues::new("poison", ["unusable"]).unwrap();
                let values = NumericValues::new("unusable", [f64::INFINITY]);
                let poison = match source.kind() {
                    PlotKind::Boxplot => Plot::boxplot(categories, values).erase(),
                    PlotKind::Violin => Plot::violin(categories, values).erase(),
                    PlotKind::Raincloud => {
                        Plot::raincloud(RaincloudInput::samples(categories, values)).erase()
                    }
                    _ => unreachable!(),
                };
                let counts = STAGES.with(std::cell::Cell::get);
                let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
                let projections = crate::layout::Projection::test_projection_calls();
                let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
                measurements::start();
                let captured = crate::orchestrator::capture_text_replay_plan(
                    640,
                    480,
                    "selected categorical distributions",
                    &theme,
                    stage,
                    |backend| {
                        let context = PlotRenderContext::new(
                            poison.data(),
                            selected.layout.projection.panel_box(),
                            &theme,
                        )
                        .with_retained_body(selected.retained_payload());
                        poison.render_panel(source.kind(), context, backend)?;
                        crate::significance::render_selected(&poison, &selected, &theme, backend)?;
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
                for (before, after) in counts.iter().zip(STAGES.with(std::cell::Cell::get)) {
                    assert_eq!(&before[..3], &after[..3]);
                }
                for counter in [
                    Counter::AxisRangePreparation,
                    Counter::AxisSpecPreparation,
                    Counter::AxisSourcePreparation,
                    Counter::TickSource,
                    Counter::TickMetricLookup,
                    Counter::TextProposalVisit,
                    Counter::TextExactTrial,
                    Counter::SignificanceBisection,
                    Counter::SignificanceRangeGrowth,
                ] {
                    assert_eq!(work.counters[counter as usize], 0, "{counter:?}");
                }
                Ok(captured)
            },
        )
        .unwrap();
        assert_eq!(
            STAGES
                .with(std::cell::Cell::get)
                .into_iter()
                .filter(|counts| counts[0] != 0)
                .collect::<Vec<_>>(),
            [[1, 1, 1, 1]]
        );
        assert_eq!(
            crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
            0
        );
        crate::orchestrator::validate_text_replay_plan(
            640,
            480,
            root.plan(),
            "categorical distributions Null replay",
        )
        .unwrap();
        let before = STAGES.with(std::cell::Cell::get);
        let mut svg = root.plan().take_svg_backend().unwrap();
        root.plan()
            .replay(640, 480, &mut svg, "categorical distributions SVG replay")
            .unwrap();
        assert_eq!(STAGES.with(std::cell::Cell::get), before);
    }
}
