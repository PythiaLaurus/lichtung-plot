use super::*;

#[test]
fn distribution_series_color_keys_refuse_missing_raw_identity_before_fonts() {
    let plot = Plot::boxplot(
        CategoryValues::new("category", ["a", "a", "b", "b"]).unwrap(),
        NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]),
    )
    .series(CategoryValues::new("series", ["first", "first", "second", "second"]).unwrap())
    .categorical_colors(
        crate::CategoricalColors::new([
            ("first".into(), (15, 25, 35)),
            ("unused".into(), (15, 25, 35)),
        ])
        .unwrap(),
    )
    .erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let error = crate::plots::shared_x::prepare_plot(&plot, &OperationGenerationResources::new())
        .err()
        .expect("used missing series must refuse");
    assert!(matches!(error, Error::Data(message) if message.contains("second")));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn p2_retained_line_copy_admits_records_and_bytes_without_processing() {
    let source = crate::Plot::scatter(
        crate::NumericValues::new("x", [1.0]),
        crate::NumericValues::new("y", [1.0]),
    )
    .erase();
    let points = [Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0)];
    let style = LineStyle::solid((0, 0, 0), 1.0);
    let bytes = (std::mem::size_of_val(&points) + std::mem::size_of::<Mark>()) as u64;
    for records in [0, 2, 3, 4] {
        let resources =
            crate::font::canonical::OperationGenerationResources::with_test_limits(0, bytes);
        resources.claim_slots(1_000_000 - records).unwrap();
        let mut payload = DistributionPayload::new(
            crate::layout::PanelBox::new(0.0, 2.0, 0.0, 2.0),
            crate::PlotKind::Boxplot,
        );
        assert_eq!(
            payload
                .line(
                    &points,
                    &style,
                    source.data().with_axis_resources(&resources)
                )
                .is_ok(),
            records >= 3
        );
        assert_eq!(resources.usage().unwrap().1, 0);
        if records >= 3 {
            assert_eq!(payload.marks.len(), 1);
            assert_eq!(resources.byte_usage().unwrap(), bytes);
        } else {
            assert!(payload.marks.is_empty());
            assert_eq!(payload.marks.capacity(), 0);
            assert_eq!(
                resources.byte_usage().unwrap(),
                if records == 0 {
                    0
                } else {
                    std::mem::size_of_val(&points) as u64
                }
            );
        }
    }
    for available in [0, bytes - 1, bytes, bytes + 1] {
        let resources = OperationGenerationResources::with_test_limits(0, available);
        let mut payload = DistributionPayload::new(
            crate::layout::PanelBox::new(0.0, 2.0, 0.0, 2.0),
            crate::PlotKind::Boxplot,
        );
        let result = payload.line(
            &points,
            &style,
            source.data().with_axis_resources(&resources),
        );
        assert_eq!(result.is_ok(), available >= bytes);
        assert_eq!(
            resources.usage().unwrap(),
            (if available == 0 { 2 } else { 3 }, 0)
        );
        if available >= bytes {
            let Mark::Line(retained, retained_style) = &payload.marks[0] else {
                unreachable!()
            };
            assert_eq!(retained, &points);
            assert_eq!(retained_style, &style);
            assert_eq!(resources.byte_usage().unwrap(), bytes);
        } else {
            assert!(payload.marks.is_empty());
            assert_eq!(payload.marks.capacity(), 0);
            assert!(matches!(result, Err(crate::Error::Render(_))));
            assert_eq!(
                resources.byte_usage().unwrap(),
                if available == 0 {
                    0
                } else {
                    std::mem::size_of_val(&points) as u64
                }
            );
        }
    }
}
use crate::font::canonical::OperationGenerationResources;
use crate::spec::PlotRenderContext;
use crate::{CategoryValues, NumericValues, RaincloudInput};

pub(crate) fn sources(facet: Option<crate::FacetScales>) -> [Plot; 3] {
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

pub(crate) fn supplied() -> Plot {
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
fn assigned_panel_marks_match_canonical_projection_fingerprints() {
    use sha2::{Digest, Sha256};
    // T5c closes styled distribution ink and native slot coordinates.
    // Historical T5a6d and 8e3395a fingerprints and coordinate deltas are in
    // docs/local-search-benchmarks.md, "T5a6d Closing Audit".
    let expected = [
        "4b16fd7df8af5552f88c24d1cf32b486eef31e923aa3b178e79aa1c3f2cb660b",
        "2384114e60a3e17af3ad8edda471780eb580e1c4f20ebfed633959a16988871d",
        "598c5d93fa6c2c50afcf582838d02595f49f8e9e493b7b2078ed8d46e61f0b16",
        "488c40b9283428d4221314a9fdb8667e5050f66ccdf58a4a42b54679545bf81d",
    ];
    for (source, expected) in sources(None).into_iter().chain([supplied()]).zip(expected) {
        let mut fingerprint = Sha256::new();
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
                crate::layout::PanelBox::new(80.25, 550.75, 70.5, 410.25),
                crate::layout::PanelBox::new(30.0, 810.0, 50.0, 600.0),
            ] {
                let selected = plot
                    .measure_selected_interior(PlotRenderContext::new(plot.data(), panel, &theme))
                    .unwrap();
                let mut svg = crate::render::svg_backend::SvgBackend::in_memory(900, 700);
                svg.begin(900, 700).unwrap();
                selected.marks.as_ref().unwrap().draw(&mut svg).unwrap();
                svg.end().unwrap();
                fingerprint.update(svg.into_bytes().unwrap());
            }
        }
        assert_eq!(
            format!("{:x}", fingerprint.finalize()),
            expected,
            "{:?}",
            source.kind()
        );
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one odd-quantum assignment checks every distribution's scientific coordinates"
)]
fn canonical_distribution_coordinates_preserve_quartiles_and_observations() {
    for source in sources(None).into_iter().chain([supplied()]) {
        let source = source.labels(crate::Labels::raw());
        let plot =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap()
                .attach(source)
                .unwrap();
        let theme = crate::Theme::Publication.resolve();
        // Odd quantum spans and origins exercise projection before width/strip arithmetic.
        let panel = crate::layout::PanelBox::new(
            80.0 + 1.0 / 1024.0,
            550.0 + 4.0 / 1024.0,
            70.0 + 1.0 / 1024.0,
            410.0 + 4.0 / 1024.0,
        );
        let selected = plot
            .measure_selected_interior(PlotRenderContext::new(plot.data(), panel, &theme))
            .unwrap();
        let prepared = retained(&plot).unwrap();
        assert_eq!(
            selected.x_tick_keys.as_deref().unwrap(),
            prepared.categories()
        );
        assert_eq!(
            selected
                .layout
                .x_axis
                .ticks
                .iter()
                .map(|tick| tick.label.as_str())
                .collect::<Vec<_>>(),
            prepared.categories()
        );
        let crate::layout::interior::MarkPayload::Distribution(payload) =
            &selected.marks.as_ref().unwrap().payload
        else {
            unreachable!()
        };
        assert_eq!(payload.clip, panel);
        let supplied = plot.raincloud_geometry_input().is_some();
        let summaries: &[_] = if supplied {
            &[(1.0, 3.0, 5.0)]
        } else {
            &[(2.5, 3.5, 4.5), (3.5, 5.5, 7.5)]
        };
        let (stride, box_index, median_index) = match plot.kind() {
            PlotKind::Boxplot => (6, 0, 1),
            PlotKind::Violin => (6, 4, 5),
            PlotKind::Raincloud => (3, 1, 2),
            _ => unreachable!(),
        };
        let project_y = |value| selected.layout.projection.project(0.0, value).1;
        for (index, &(q1, median, q3)) in summaries.iter().enumerate() {
            let Mark::Polygon(points, _) = &payload.marks[index * stride + box_index] else {
                unreachable!()
            };
            assert_eq!(points.len(), 4);
            assert_eq!(
                points
                    .iter()
                    .filter(|point| point.y.to_bits() == project_y(q1).to_bits())
                    .count(),
                2
            );
            assert_eq!(
                points
                    .iter()
                    .filter(|point| point.y.to_bits() == project_y(q3).to_bits())
                    .count(),
                2
            );
            let Mark::Line(points, _) = &payload.marks[index * stride + median_index] else {
                unreachable!()
            };
            assert_eq!(points.len(), 2);
            assert!(points
                .iter()
                .all(|point| point.y.to_bits() == project_y(median).to_bits()));
        }
        let observations = payload
            .marks
            .iter()
            .filter_map(|mark| {
                if let Mark::Scatter(points, _) = mark {
                    Some(points)
                } else {
                    None
                }
            })
            .flatten()
            .map(|point| point.y.to_bits())
            .collect::<Vec<_>>();
        let values: &[_] = if supplied {
            &[2.0, 4.0]
        } else if plot.kind() == PlotKind::Violin {
            &[]
        } else {
            &[1.0, 4.0, 3.0, 6.0, 2.0, 7.0, 4.0, 9.0]
        };
        assert_eq!(
            observations,
            values
                .iter()
                .map(|&value| project_y(value).to_bits())
                .collect::<Vec<_>>()
        );
        if plot.kind() != PlotKind::Boxplot {
            for index in 0..summaries.len() {
                let Mark::Polygon(points, _) = &payload.marks[index * stride] else {
                    unreachable!()
                };
                let (lo, hi) = prepared.range();
                assert_eq!(
                    points
                        .iter()
                        .map(|point| point.y)
                        .min_by(f64::total_cmp)
                        .unwrap()
                        .to_bits(),
                    project_y(lo).to_bits()
                );
                assert_eq!(
                    points
                        .iter()
                        .map(|point| point.y)
                        .max_by(f64::total_cmp)
                        .unwrap()
                        .to_bits(),
                    project_y(hi).to_bits()
                );
            }
        }
    }
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
fn parent_palette_and_absent_series_slots_are_retained_before_native_projection() {
    for source in sources(None) {
        let source = source
            .with_global_group_levels(vec!["absent".into(), "two".into(), "one".into()])
            .with_global_hue_levels(vec!["absent".into(), "two".into(), "one".into()]);
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let plot = inputs.attach(source).unwrap();
        let sample = match retained(&plot).unwrap() {
            PreparedDistribution::Boxplot(input) => &input.sample,
            PreparedDistribution::Violin(input) => &input.sample,
            PreparedDistribution::Supplied(_) => unreachable!(),
        };
        assert_eq!(
            sample
                .groups
                .iter()
                .map(|group| (group.group_index, group.hue_label.as_deref()))
                .collect::<Vec<_>>(),
            [(2, Some("one")), (1, Some("two"))]
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
            let result = plot
                .measure_selected_interior(PlotRenderContext::new(
                    plot.data(),
                    crate::layout::PanelBox::new(50.0, 850.0, 60.0, 600.0),
                    &theme,
                ))
                .unwrap();
            let crate::layout::interior::MarkPayload::Distribution(payload) =
                &result.marks.as_ref().unwrap().payload
            else {
                unreachable!()
            };
            let colors = payload
                .marks
                .iter()
                .filter_map(|mark| match mark {
                    Mark::Polygon(_, fill) if fill.color != (255, 255, 255) => Some(fill.color),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let alpha = if plot.kind() == crate::PlotKind::Boxplot {
                theme.marks.box_body.fill_alpha
            } else {
                theme.marks.violin_body.fill_alpha
            };
            assert_eq!(
                colors,
                [theme.colors[2], theme.colors[1]]
                    .map(|color| crate::render::backend::Rgba::from(color).with_alpha(alpha))
            );
        }
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
                        .with_retained_body(&selected);
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
