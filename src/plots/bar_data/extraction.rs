use super::*;
use crate::font::canonical::OperationGenerationResources;
use crate::render::backend::Backend;
use crate::spec::PlotRenderContext;
use crate::{BarInput, Bounds, CategoryValues, FacetScales, FacetSpec, NumericValues, PlotKind};

#[test]
fn positioned_bar_colors_require_every_used_raw_series_before_fonts() {
    let source = Plot::bar(
        CategoryValues::new("category", ["a", "a"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [1.0, 2.0])),
    )
    .series(CategoryValues::new("series", ["first", "second"]).unwrap())
    .categorical_colors(
        crate::CategoricalColors::new([
            ("first".into(), (10, 20, 30)),
            ("unused".into(), (10, 20, 30)),
        ])
        .unwrap(),
    )
    .erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let error =
        crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new(), None)
            .err()
            .expect("missing positioned series must refuse");
    assert!(matches!(error, Error::Data(message) if message.contains("second")));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn displayed_series_alias_never_becomes_a_significance_target_or_color_key() {
    let source = Plot::bar(
        CategoryValues::new("category", ["same", "same"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [2.0, 3.0])),
    )
    .series(CategoryValues::new("series", ["g1", "g2"]).unwrap())
    .categorical_colors(
        crate::CategoricalColors::new([("g1".into(), (20, 40, 60)), ("g2".into(), (20, 40, 60))])
            .unwrap(),
    )
    .labels(crate::Labels::raw().alias("g1", "First display"))
    .significance([crate::Significance::at(
        crate::SignificanceTarget::category_series("same", "First display"),
        crate::SignificanceMark::OneStar,
    )])
    .erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let error =
        crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new(), None)
            .err()
            .expect("a display alias must not select a raw series");
    assert!(matches!(error, Error::Data(message)
        if message.contains("unknown series") && message.contains("First display")));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
fn signed_bar_entries_use_only_sign_identities_and_match_body_colors() {
    let source = Plot::horizontal_bar(
        CategoryValues::new("category", ["zero", "fall", "rise"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [0.0, -2.0, 3.0])),
    )
    .series(CategoryValues::new("series", ["one", "two", "three"]).unwrap())
    .categorical_colors(crate::CategoricalColors::new([("unused".into(), (1, 2, 3))]).unwrap())
    .color_by_sign()
    .erase();
    let resources = OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
    let prepared = &inputs.bars.as_ref().unwrap().panels[0].1;
    let mut theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    theme.marks.accents =
        crate::theme::AccentPalette::new(theme.marks.accents.seeds()[..1].iter().copied()).unwrap();
    let data = source.data().with_axis_resources(&resources);
    let (colors, accents) = prepared.resolved_colors(&source, &theme, data).unwrap();
    assert!(colors.colors().is_empty());
    let positive = accents.color(0).unwrap();
    let negative = accents.color(1).unwrap();
    assert_eq!(positive, theme.marks.accents.seeds()[0].as_rgb());
    assert_ne!(positive, negative);
    for bar in &prepared.bars {
        assert_eq!(
            prepared
                .bar_color(&source, bar, &colors, &accents, &resources)
                .unwrap(),
            if bar.end < bar.start {
                negative
            } else {
                positive
            }
        );
    }
    let mut entries = legend_entries(prepared, &source, &theme, data, &colors, &accents).unwrap();
    assert_eq!(entries.len(), 2);
    for (entry, decreasing) in entries.iter().zip([false, true]) {
        assert_eq!(entry.identity(), &LegendIdentity::bar_sign(decreasing));
        assert!(entry.coordinated_hue_value_for("series").is_none());
        let expected = LegendEntry::fill(
            LegendIdentity::bar_sign(decreasing),
            theme
                .marks
                .bar_body
                .backend_style(accents.color(usize::from(decreasing)).unwrap()),
        )
        .without_hue_level();
        assert!(entry.same_child_input(&expected));
    }
    crate::decorations::legend::resolve_entries(&source, &mut entries, &resources).unwrap();
    assert_eq!(entries[0].display_label(), "Nondecreasing");
    assert_eq!(entries[1].display_label(), "Decreasing");
    assert_eq!(
        crate::decorations::legend::resolved_title(&entries, &resources).unwrap(),
        "Change"
    );
}

#[test]
fn attached_bar_glyphs_cannot_escape_categorical_frame_under_numeric_feedback() {
    let categories = CategoryValues::new("category", ["a", "b", "c", "d"]).unwrap();
    let input = BarInput::estimates(NumericValues::new("value", [4.0, 5.0, 6.0, 7.0]));
    let plots = [
        Plot::bar(categories.clone(), input.clone())
            .significance([crate::Significance::at(
                "a",
                crate::SignificanceMark::OneStar,
            )])
            .erase(),
        Plot::horizontal_bar(categories, input)
            .significance([crate::Significance::at(
                "a",
                crate::SignificanceMark::OneStar,
            )])
            .erase(),
    ];
    let mut theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    theme.typography.annotations.significance_label.font_size = 1000;
    for plot in plots {
        let inputs =
            crate::plots::shared_x::prepare_plot(&plot, &OperationGenerationResources::new(), None)
                .unwrap();
        let chart = inputs.attach(plot).unwrap();
        let selected = chart.measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            crate::layout::PanelBox::new(50.0, 350.0, 50.0, 350.0),
            &theme,
        ));
        match selected {
            Err(crate::layout::fit::PlanError::Fit(failure)) => {
                assert_eq!(
                    failure.reason.kind,
                    crate::layout::fit::FitKind::OverlayCollision
                );
                assert!(
                    failure.message.contains("required Bar annotation ink"),
                    "{failure:?}"
                );
            }
            Err(error) => panic!("expected Bar annotation collision; got {error:?}"),
            Ok(_) => panic!("expected Bar annotation collision; selected a result"),
        }
    }
}

fn sources(facets: bool) -> [Plot; 4] {
    fn finish<K: crate::kind::ExactFamily + crate::kind::Facet>(
        plot: Plot<K>,
        facets: bool,
    ) -> Plot {
        if facets {
            plot.facet(FacetSpec::column(
                CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap(),
            ))
            .size(1000, 640)
            .erase()
        } else {
            plot.erase()
        }
    }
    let categories = CategoryValues::new("category", ["a", "b", "a", "b"]).unwrap();
    let values = NumericValues::new("value", [-2.0, 3.0, -4.0, 7.0]);
    let groups = CategoryValues::new(
        "series",
        if facets {
            ["g", "g", "h", "h"]
        } else {
            ["g", "g", "g", "g"]
        },
    )
    .unwrap();
    [
        finish(
            Plot::bar(categories.clone(), BarInput::samples(values.clone())).series(groups.clone()),
            facets,
        ),
        finish(
            Plot::horizontal_bar(categories.clone(), BarInput::samples(values.clone()))
                .series(groups.clone())
                .color_by_sign()
                .zero_line(),
            facets,
        ),
        finish(
            Plot::stacked_bar(categories.clone(), values.clone(), groups.clone()),
            facets,
        ),
        finish(
            Plot::stacked_horizontal_bar(categories, values, groups),
            facets,
        ),
    ]
}

fn annotated() -> [Plot; 2] {
    let categories = CategoryValues::new("category", ["a", "b"]).unwrap();
    let input = BarInput::estimates_with_error_bars(
        NumericValues::new("value", [-2.0, 3.0]),
        Bounds::new(
            NumericValues::new("lower", [-3.0, 2.0]),
            NumericValues::new("upper", [-1.0, 4.0]),
        ),
    );
    let layer = crate::BarPointLayer::new(
        categories.clone(),
        None,
        NumericValues::new("points", [-4.0, 5.0]),
        (12, 34, 56),
        3,
        0.5,
    )
    .unwrap();
    let marks = [
        crate::Significance::at("a", crate::SignificanceMark::TwoStars),
        crate::Significance::at("b", crate::SignificanceMark::PValue(0.03)),
    ];
    let comparison = [crate::Significance::between(
        "a",
        "b",
        crate::SignificanceMark::OneStar,
    )];
    let guide = crate::GuideStyle::new((90, 80, 70), 1.0, crate::LinePattern::Solid).unwrap();
    [
        Plot::bar(categories.clone(), input.clone())
            .point_layer(layer.clone())
            .significance(marks.clone())
            .significance(comparison.clone())
            .guide(crate::GuideSpec::horizontal(0.0, guide.clone()))
            .y_limits(-10.0, 10.0)
            .y_ticks(vec![
                crate::AxisTick::new(-10.0, "low"),
                crate::AxisTick::new(0.0, "zero"),
                crate::AxisTick::new(10.0, "high"),
            ])
            .erase(),
        Plot::horizontal_bar(categories, input)
            .point_layer(layer)
            .significance(marks)
            .significance(comparison)
            .guide(crate::GuideSpec::vertical(0.0, guide))
            .x_limits(-10.0, 10.0)
            .x_ticks(vec![
                crate::AxisTick::new(-10.0, "low"),
                crate::AxisTick::new(0.0, "zero"),
                crate::AxisTick::new(10.0, "high"),
            ])
            .zero_line()
            .erase(),
    ]
}

#[test]
fn p2_bar_levels_and_groups_stop_charging_at_the_first_match() {
    let source = Plot::bar(
        CategoryValues::new("category", ["a"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [1.0])),
    )
    .erase();
    let resources = OperationGenerationResources::with_test_limits(1, 0);
    let mut levels = ["a", "b", "c"].map(str::to_owned).to_vec();
    assert_eq!(
        level_index(
            &mut levels,
            "a".into(),
            source.data().with_axis_resources(&resources)
        )
        .unwrap(),
        0
    );
    assert_eq!(resources.usage().unwrap(), (0, 1));
    assert_eq!(levels, ["a", "b", "c"]);
    for grouped in [false, true] {
        let mut visits = Vec::new();
        for last in ["a", "c"] {
            let source = Plot::bar(
                CategoryValues::new("category", ["a", "b", "c", last]).unwrap(),
                BarInput::samples(NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]))
                    .error_bar(crate::ErrorBar::hidden()),
            );
            let source = if grouped {
                source.series(CategoryValues::new("series", ["s"; 4]).unwrap())
            } else {
                source
            }
            .erase();
            let resources = OperationGenerationResources::new();
            prepare(
                &source,
                source.data().with_axis_resources(&resources),
                if grouped {
                    BarMode::Grouped
                } else {
                    BarMode::Single
                },
            )
            .unwrap();
            visits.push(resources.usage().unwrap().1);
        }
        // The last row examines one rather than three existing category levels
        // and one rather than three existing group cells. Unified series also
        // checks the category/group-to-hue table; other passes are equal.
        assert_eq!(visits[1] - visits[0], if grouped { 6 } else { 4 });
    }
}

#[test]
fn bar_legend_identity_admits_long_dimensions_once_per_candidate() {
    let dimension = "caller-controlled-series-dimension".repeat(1024);
    let source = Plot::bar(
        CategoryValues::new("category", ["a", "a"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [1.0, 2.0])),
    )
    .series(CategoryValues::new(&dimension, ["first", "second"]).unwrap())
    .erase();
    let prepared = prepare(&source, source.data(), BarMode::Grouped).unwrap();
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let candidate = |resources: &OperationGenerationResources| {
        let data = source.data().with_axis_resources(resources);
        let (colors, accents) = prepared.resolved_colors(&source, &theme, data)?;
        legend_entries(&prepared, &source, &theme, data, &colors, &accents)
    };
    let probe = OperationGenerationResources::new();
    candidate(&probe).unwrap();
    let (records, visits) = probe.usage().unwrap();
    let bytes = probe.byte_usage().unwrap();
    let exact = OperationGenerationResources::with_test_limits(visits, bytes);
    let entries = candidate(&exact).unwrap();
    assert_eq!(exact.usage().unwrap().1, visits);
    assert_eq!(exact.usage().unwrap().0, records);
    assert_eq!(exact.byte_usage().unwrap(), bytes);
    for (entry, value) in entries.iter().zip(&prepared.hues) {
        assert_eq!(entry.identity(), &source.legend_identity(value));
    }
    for (visits_limit, byte_limit) in [(visits - 1, bytes), (visits, bytes - 1)] {
        let resources = OperationGenerationResources::with_test_limits(visits_limit, byte_limit);
        let error = candidate(&resources).unwrap_err();
        assert!(matches!(error, Error::Render(_)));
        // A refused candidate never refunds its already admitted work.
        assert!(resources.usage().unwrap().0 > 0);
        assert!(resources.usage().unwrap().1 > 0 || visits_limit < visits);
    }
    let limited = OperationGenerationResources::with_test_limits(visits, bytes);
    limited.claim_slots(1_000_000 - records + 1).unwrap();
    assert!(candidate(&limited)
        .unwrap_err()
        .to_string()
        .contains("record admission"));
    assert!(limited.byte_usage().unwrap() <= bytes);
    let cumulative = OperationGenerationResources::with_test_limits(2 * visits, 2 * bytes);
    candidate(&cumulative).unwrap();
    candidate(&cumulative).unwrap();
    assert_eq!(cumulative.usage().unwrap().1, 2 * visits);
    assert_eq!(cumulative.byte_usage().unwrap(), 2 * bytes);
    assert!(candidate(&cumulative).is_err());
}

#[test]
fn bar_semantics_admit_exact_resources_and_attach_without_copying() {
    for source in sources(false)
        .into_iter()
        .chain(annotated())
        .chain(sources(true))
    {
        STAGES.with(|counts| counts.set([[0; 4]; 4]));
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
        let resources = OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
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
                &OperationGenerationResources::with_test_limits(visits, bytes),
                None,
            )
            .is_err());
        }
        crate::plots::shared_x::prepare_plot(
            &source,
            &OperationGenerationResources::with_test_limits(visits, bytes),
            None,
        )
        .unwrap();
        let attached = inputs.attach(source).unwrap();
        assert!(std::sync::Arc::ptr_eq(
            attached.prepared_bars.as_ref().unwrap(),
            inputs.bars.as_ref().unwrap()
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}

#[test]
fn bar_standalone_facets_and_mixed_shared_canvas_prepare_once() {
    for source in sources(false).into_iter().chain(sources(true)) {
        let expected = if source.has_facets() { 2 } else { 1 };
        STAGES.with(|counts| counts.set([[0; 4]; 4]));
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|count| count.set(0));
        source.preflight().unwrap();
        let active = STAGES
            .with(std::cell::Cell::get)
            .into_iter()
            .filter(|count| count[0] != 0)
            .collect::<Vec<_>>();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0][0], expected);
        assert_eq!(active[0][1], active[0][2]);
        assert_eq!(active[0][3], expected);
    }
    let categories = CategoryValues::new("category", ["a", "b"]).unwrap();
    let values = NumericValues::new("value", [-2.0, 7.0]);
    let groups = CategoryValues::new("series", ["g", "g"]).unwrap();
    let bar = Plot::bar(categories.clone(), BarInput::samples(values.clone()));
    let horizontal = Plot::horizontal_bar(categories.clone(), BarInput::samples(values.clone()));
    let stacked = Plot::stacked_bar(categories.clone(), values.clone(), groups.clone());
    let stacked_horizontal = Plot::stacked_horizontal_bar(categories, values, groups);
    STAGES.with(|counts| counts.set([[0; 4]; 4]));
    crate::Canvas::grid(2, 2)
        .unwrap()
        .size(1200, 900)
        .panel(0, 0, bar)
        .unwrap()
        .panel(0, 1, stacked)
        .unwrap()
        .panel(1, 0, horizontal)
        .unwrap()
        .panel(1, 1, stacked_horizontal)
        .unwrap()
        .x_range_group([(1, 0), (1, 1)])
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
fn bar_geometry_matches_payload_with_caps_points_and_attached_text() {
    for source in sources(false).into_iter().chain(annotated()) {
        let theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        let inputs = crate::plots::shared_x::prepare_plot(
            &source,
            &OperationGenerationResources::new(),
            None,
        )
        .unwrap();
        let source = inputs.attach(source).unwrap();
        let selected = source
            .measure_selected_interior(PlotRenderContext::new(
                source.data(),
                crate::layout::PanelBox::new(20.25, 520.75, 30.5, 350.25),
                &theme,
            ))
            .unwrap();
        let marks = selected.marks.as_ref().unwrap();
        let mut drawn = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
        drawn.begin(640, 480).unwrap();
        marks.draw(&mut drawn).unwrap();
        crate::significance::render(
            &source,
            selected.significance.as_ref().unwrap(),
            &theme,
            &mut drawn,
        )
        .unwrap();
        drawn.end().unwrap();
        assert_eq!(marks.geometry.bounds(), drawn.bounds());
        let crate::layout::interior::MarkPayload::Bars(payload) = &marks.payload else {
            unreachable!()
        };
        let prepared = &inputs.bars.as_ref().unwrap().panels[0].1;
        if !prepared.significance.is_empty() {
            let ticks = if source.kind() == PlotKind::Bar {
                &selected.layout.y_axis.ticks
            } else {
                &selected.layout.x_axis.ticks
            };
            assert_eq!(
                ticks
                    .iter()
                    .map(|tick| tick.label.as_str())
                    .collect::<Vec<_>>(),
                ["low", "zero", "high"]
            );
        }
        assert_eq!(payload.bars.len(), prepared.bars.len());
        assert_eq!(payload.points.len(), prepared.points.len());
        assert_eq!(payload.text.len(), prepared.significance.len());
        for (shape, bar) in payload.bars.iter().zip(&prepared.bars) {
            assert_eq!(shape.interval.is_some(), bar.interval.is_some());
        }
    }
}

#[test]
fn bar_facets_retain_selected_annotations_global_hues_and_final_ticks() {
    let source = Plot::bar(
        CategoryValues::new("category", ["same", "same"]).unwrap(),
        BarInput::samples(NumericValues::new("value", [1.0, 10.0])),
    )
    .series(CategoryValues::new("series", ["first", "second"]).unwrap())
    .significance([
        crate::Significance::at("same", crate::SignificanceMark::TwoStars)
            .facet(crate::FacetCell::single("right")),
    ])
    .facet(FacetSpec::column(
        CategoryValues::new("panel", ["left", "right"]).unwrap(),
    ))
    .size(1000, 640)
    .erase();
    source.preflight().unwrap();
    let source = source.with_test_axis_ticks(
        1,
        vec![
            crate::AxisTick::new(0.0, "zero"),
            crate::AxisTick::new(10.0, "ten"),
        ],
    );
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new(), None)
            .unwrap();
    let panels = &inputs.bars.as_ref().unwrap().panels;
    assert!(panels[0].1.significance.is_empty());
    assert_eq!(panels[1].1.significance[0].label, "**");
    assert_eq!(panels[0].1.hues, ["first", "second"]);
    assert_eq!(panels[1].1.bars[0].hue_index, Some(1));
    for (_, panel) in panels {
        assert_eq!(panel.series, ["first", "second"]);
        assert_eq!(
            panel.bars.len(),
            1,
            "an absent series must not acquire a zero bar"
        );
    }
    assert_eq!(inputs.axes[0].1.ranges[1], inputs.axes[1].1.ranges[1]);
}

#[test]
fn bar_invalid_sources_and_anchors_fail_before_fonts() {
    let category = CategoryValues::new("category", ["a"]).unwrap();
    let invalid = NumericValues::new("value", [f64::INFINITY]);
    let group = CategoryValues::new("series", ["g"]).unwrap();
    let sources = [
        Plot::bar(category.clone(), BarInput::samples(invalid.clone())).erase(),
        Plot::horizontal_bar(category.clone(), BarInput::samples(invalid.clone())).erase(),
        Plot::stacked_bar(category.clone(), invalid.clone(), group.clone()).erase(),
        Plot::stacked_horizontal_bar(category.clone(), invalid, group).erase(),
        Plot::bar(
            category,
            BarInput::estimates(NumericValues::new("value", [1.0])),
        )
        .significance([crate::Significance::at(
            "missing",
            crate::SignificanceMark::OneStar,
        )])
        .erase(),
    ];
    for source in sources {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
        assert!(matches!(
            crate::plots::shared_x::prepare_plot(
                &source,
                &OperationGenerationResources::new(),
                None
            ),
            Err(Error::Data(_))
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}

#[test]
fn bar_bootstrap_is_retained_bit_exact_in_each_facet() {
    let source = Plot::bar(
        CategoryValues::new("category", ["a", "a", "a", "a", "a", "a"]).unwrap(),
        BarInput::samples(NumericValues::new("value", [1.0, 3.0, 7.0, 2.0, 5.0, 9.0])),
    )
    .facet(
        FacetSpec::column(
            CategoryValues::new("panel", ["left", "left", "left", "right", "right", "right"])
                .unwrap(),
        )
        .scales(FacetScales::Free),
    )
    .erase();
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new(), None)
            .unwrap();
    for (index, rows) in [[0, 1, 2], [3, 4, 5]].iter().enumerate() {
        let data = crate::data::PlotData::with_rows(&source, rows, rows);
        let expected = prepare(&source, data, BarMode::Single).unwrap();
        let actual = &inputs.bars.as_ref().unwrap().panels[index].1;
        assert_eq!(actual.bars[0].end.to_bits(), expected.bars[0].end.to_bits());
        assert_eq!(
            actual.bars[0]
                .interval
                .map(|(lo, hi)| (lo.to_bits(), hi.to_bits())),
            expected.bars[0]
                .interval
                .map(|(lo, hi)| (lo.to_bits(), hi.to_bits()))
        );
    }
    BOOTSTRAP_DRAWS.with(|draws| draws.set(0));
    let attached = inputs.attach(source).unwrap();
    let _ = attached.prepared_bars.as_ref().unwrap().panels[0]
        .1
        .value_range;
    assert_eq!(BOOTSTRAP_DRAWS.with(std::cell::Cell::get), 0);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one operation proves source-free selected capture and both frozen replays"
)]
fn bar_poisoned_capture_repeats_no_preparation_projection_axis_or_shaping() {
    use crate::orchestrator::measurements::{self, Counter};
    for source in sources(false).into_iter().chain(annotated()) {
        STAGES.with(|counts| counts.set([[0; 4]; 4]));
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|count| count.set(0));
        let theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        let root = crate::layout::plan_exact_root(
            640,
            480,
            |stage| {
                let inputs = crate::plots::shared_x::prepare_plot(
                    &source,
                    &stage.generation_resources(),
                    None,
                )?;
                let requirements = crate::layout::foundational_plot_requirements(
                    Some(&inputs),
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
                let draws = BOOTSTRAP_DRAWS.with(std::cell::Cell::get);
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
                assert_eq!(BOOTSTRAP_DRAWS.with(std::cell::Cell::get), draws);
                Ok(crate::layout::MeasuredLayout::new(assigned, selected))
            },
            |_, _, selected, stage| {
                let terms = CategoryValues::new("poison", ["unusable"]).unwrap();
                let invalid = NumericValues::new("unusable", [f64::INFINITY]);
                let groups = CategoryValues::new("poison series", ["unusable"]).unwrap();
                let poison = match source.kind() {
                    PlotKind::Bar => Plot::bar(terms, BarInput::samples(invalid)).erase(),
                    PlotKind::HorizontalBar => {
                        Plot::horizontal_bar(terms, BarInput::samples(invalid)).erase()
                    }
                    PlotKind::StackedBar => Plot::stacked_bar(terms, invalid, groups).erase(),
                    PlotKind::StackedHorizontalBar => {
                        Plot::stacked_horizontal_bar(terms, invalid, groups).erase()
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
                    "selected bars",
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
                        crate::significance::render(
                            &source,
                            selected.significance.as_ref().unwrap(),
                            &theme,
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
                    Counter::OneCategoricalAxisMeasure,
                    Counter::StandardPanelIntrinsic,
                    Counter::LegacyPanelConstructor,
                    Counter::SignificanceBisection,
                    Counter::SignificanceRangeGrowth,
                ] {
                    assert_eq!(work.counters[counter as usize], 0, "{counter:?}");
                }
                Ok(captured)
            },
        )
        .unwrap();
        let active = STAGES
            .with(std::cell::Cell::get)
            .into_iter()
            .filter(|counts| counts[0] != 0)
            .collect::<Vec<_>>();
        assert_eq!(active, [[1, 1, 1, 1]]);
        assert_eq!(
            crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
            0
        );
        crate::orchestrator::validate_text_replay_plan(640, 480, root.plan(), "bars Null replay")
            .unwrap();
        let before = STAGES.with(std::cell::Cell::get);
        let mut svg = root.plan().take_svg_backend().unwrap();
        root.plan()
            .replay(640, 480, &mut svg, "bars SVG replay")
            .unwrap();
        assert_eq!(STAGES.with(std::cell::Cell::get), before);
    }
}
