use super::*;
use crate::font::canonical::OperationContext;
use crate::{Bounds, CategoryValues, CoefficientInput, FacetScales, FacetSpec, NumericValues};

fn sources(facets: Option<FacetScales>, guide: Option<&crate::GuideSpec>) -> [Plot; 3] {
    fn finish<K: crate::kind::ExactFamily + crate::kind::Facet + crate::kind::Guides>(
        mut plot: Plot<K>,
        facets: Option<FacetScales>,
        guide: Option<&crate::GuideSpec>,
    ) -> Plot {
        if let Some(guide) = guide {
            plot = plot.guide(guide.clone());
        }
        if let Some(scales) = facets {
            plot.facet(
                FacetSpec::column(
                    CategoryValues::new("panel", ["missing", "missing", "estimated", "estimated"])
                        .unwrap(),
                )
                .scales(scales),
            )
            .erase()
        } else {
            plot.erase()
        }
    }
    let terms = CategoryValues::new(
        "term",
        if facets.is_some() {
            ["a", "b", "a", "b"]
        } else {
            ["a", "b", "c", "d"]
        },
    )
    .unwrap();
    let values = if facets.is_some() {
        [None, None, Some(10.0), Some(20.0)]
    } else {
        [Some(10.0), None, Some(20.0), None]
    };
    let estimate = NumericValues::nullable("estimate", values);
    let bounds = Bounds::new(
        NumericValues::nullable("lower", values.map(|value| value.map(|value| value - 1.0))),
        NumericValues::nullable("upper", values.map(|value| value.map(|value| value + 1.0))),
    );
    let statuses = values.map(|value| {
        if value.is_some() {
            EstimateStatus::Estimated
        } else {
            EstimateStatus::NotEstimable
        }
    });
    [
        finish(
            Plot::point_interval(terms.clone(), estimate.clone(), bounds.clone())
                .estimate_status(statuses)
                .zero_line(),
            facets,
            guide,
        ),
        finish(
            Plot::coefficient(
                terms.clone(),
                CoefficientInput::estimates_with_intervals(estimate.clone(), bounds.clone()),
            )
            .estimate_status(statuses),
            facets,
            guide,
        ),
        finish(
            Plot::forest(terms, estimate, bounds, 1.0)
                .estimate_status(statuses)
                .scale(AxisScale::Log10),
            facets,
            guide,
        ),
    ]
}

fn labels(significance: bool) -> Plot {
    let plot = Plot::coefficient(
        CategoryValues::new("term", ["a", "b"]).unwrap(),
        CoefficientInput::estimates(NumericValues::new("estimate", [10.0, 20.0])),
    )
    .x_limits(0.0, 30.0);
    if significance {
        plot.significance([
            crate::Significance::at("a", crate::SignificanceMark::OneStar),
            crate::Significance::at("b", crate::SignificanceMark::PValue(0.008)),
        ])
        .erase()
    } else {
        plot.estimate_labels(
            crate::NumericLabelFormat::default()
                .suffix(" units")
                .unwrap(),
        )
        .erase()
    }
}

#[test]
fn wide_estimate_labels_request_automatic_headroom_but_not_exact_x() {
    let source = |exact| {
        let plot = Plot::coefficient(
            CategoryValues::new("term", ["low", "middle", "high"]).unwrap(),
            CoefficientInput::estimates(NumericValues::new("estimate", [0.0, 5.0, 10.0])),
        )
        .estimate_labels(
            crate::NumericLabelFormat::default()
                .suffix("W".repeat(42))
                .unwrap(),
        )
        .size(1000, 650);
        if exact {
            plot.x_limits(0.0, 10.0).erase()
        } else {
            plot.erase()
        }
    };
    let theme = crate::Theme::Publication.resolve().unwrap();
    let panel = crate::layout::PanelBox::new(80.0, 920.0, 80.0, 580.0);
    for exact in [false, true] {
        let source = source(exact);
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        let chart = inputs.attach(source).unwrap();
        let selected =
            chart.measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme));
        if exact {
            assert!(
                matches!(selected, Err(crate::layout::fit::PlanError::Fit(failure))
                if failure.reason.kind == crate::layout::fit::FitKind::OverlayCollision)
            );
        } else {
            let selected = selected.unwrap();
            let crate::layout::interior::MarkPayload::Estimate(payload) =
                &selected.marks.as_ref().unwrap().payload
            else {
                panic!("selected Estimate")
            };
            assert!(payload.domain_expansions[0].is_some());
            let final_plan = crate::decorations::plan::plan_panel_for_test(
                &chart,
                crate::PlotKind::Coefficient,
                panel,
                crate::decorations::plan::LegendRole::Panel,
                &theme,
            )
            .unwrap();
            let crate::layout::interior::MarkPayload::Estimate(final_payload) =
                &final_plan.result.marks.as_ref().unwrap().payload
            else {
                panic!("final Estimate")
            };
            assert!(final_payload.domain_expansions.iter().all(Option::is_none));
        }
    }
}

#[test]
fn semantics_and_label_strings_are_prepared_before_fonts() {
    for source in sources(None, None)
        .into_iter()
        .chain([labels(false), labels(true)])
    {
        STAGES.with(|counts| counts.set([[0; 4]; 3]));
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
        let resources = OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
        assert_eq!(
            STAGES
                .with(std::cell::Cell::get)
                .iter()
                .map(|counts| counts[0])
                .sum::<usize>(),
            1
        );

        let attached = inputs.attach(source).unwrap();
        assert!(std::sync::Arc::ptr_eq(
            attached.prepared_estimate.as_ref().unwrap(),
            inputs.estimate.as_ref().unwrap()
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    let format = crate::NumericLabelFormat::default()
        .precision(12)
        .unwrap()
        .suffix("s".repeat(20_000))
        .unwrap();
    for value in [0.0, -0.0, f64::MAX, -f64::MAX, f64::MIN_POSITIVE] {
        assert_eq!(format.format_source(value).unwrap(), format.format(value));
    }
}

#[test]
fn missing_facets_and_mixed_shared_canvas_prepare_each_family_once() {
    for scales in [
        FacetScales::Fixed,
        FacetScales::FreeX,
        FacetScales::FreeY,
        FacetScales::Free,
    ] {
        for (family, source) in sources(Some(scales), None).into_iter().enumerate() {
            STAGES.with(|counts| counts.set([[0; 4]; 3]));
            let resources = OperationContext::new();
            let prepared = crate::plots::shared_x::prepare_plot(&source, &resources, None);
            assert_eq!(STAGES.with(std::cell::Cell::get)[family], [2, 0, 0, 0]);
            if matches!(scales, FacetScales::FreeX | FacetScales::Free) {
                assert!(matches!(prepared, Err(Error::Data(_))));
                let range = if family == 2 {
                    (0.1, 100.0)
                } else {
                    (-1.0, 30.0)
                };
                source
                    .with_planned_x_limits(range)
                    .size(1000, 640)
                    .preflight()
                    .unwrap();
            } else {
                let prepared = prepared.unwrap();
                assert_eq!(prepared.axes[0].1.ranges[0], prepared.axes[1].1.ranges[0]);
                assert!(prepared.estimate.as_ref().unwrap().panels[0]
                    .1
                    .estimates
                    .iter()
                    .all(Option::is_none));
                STAGES.with(|counts| counts.set([[0; 4]; 3]));
                source.size(1000, 640).preflight().unwrap();
                let counts = STAGES.with(std::cell::Cell::get)[family];
                assert_eq!(counts[0], 2);
                assert_eq!(counts[1], counts[2]);
                assert_eq!(counts[3], 2);
            }
        }
    }
    let terms = CategoryValues::new("term", ["a", "b"]).unwrap();
    let values = NumericValues::nullable("estimate", [Some(10.0), None]);
    let bounds = Bounds::new(
        NumericValues::nullable("lower", [Some(9.0), None]),
        NumericValues::nullable("upper", [Some(11.0), None]),
    );
    let statuses = [EstimateStatus::Estimated, EstimateStatus::NotEstimable];
    let point = Plot::point_interval(terms.clone(), values.clone(), bounds.clone())
        .estimate_status(statuses);
    let coefficient = Plot::coefficient(
        terms.clone(),
        CoefficientInput::estimates_with_intervals(values.clone(), bounds.clone()),
    )
    .estimate_status(statuses);
    let forest = Plot::forest(terms, values, bounds, 1.0)
        .estimate_status(statuses)
        .scale(AxisScale::Log10);
    STAGES.with(|counts| counts.set([[0; 4]; 3]));
    crate::Canvas::grid(1, 3)
        .unwrap()
        .size(1500, 480)
        .panel(0, 0, point)
        .unwrap()
        .panel(0, 1, coefficient)
        .unwrap()
        .panel(0, 2, forest)
        .unwrap()
        .x_range_group([(0, 0), (0, 1)])
        .unwrap()
        .preflight()
        .unwrap();
    for counts in STAGES.with(std::cell::Cell::get) {
        assert_eq!(counts[0], 1);
        assert_eq!(counts[1], counts[2]);
        assert_eq!(counts[3], 1);
    }
}

#[test]
fn projected_limits_and_geometry_match_selected_draw_order() {
    for source in sources(None, None)
        .into_iter()
        .chain([labels(false), labels(true)])
    {
        let theme = crate::Theme::Publication.resolve().unwrap();
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        let attached = inputs.attach(source).unwrap();
        let context = PlotRenderContext::new(
            attached.data(),
            crate::layout::PanelBox::new(10.25, 510.75, 20.5, 320.25),
            &theme,
        );
        let selected = attached.measure_selected_interior(context).unwrap();
        let project = || {
            super::project(
                &attached,
                &inputs.estimate.as_ref().unwrap().panels[0].1,
                selected.layout.clone(),
                context,
            )
        };
        let projected = project().unwrap();
        project().unwrap();
        let plan = projected.marks.unwrap();
        let mut drawn = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
        drawn.begin(640, 480).unwrap();
        plan.draw(&mut drawn).unwrap();
        drawn.end().unwrap();
        assert_eq!(plan.geometry.bounds(), drawn.bounds());
        let crate::layout::interior::MarkPayload::Estimate(payload) = &plan.payload else {
            unreachable!()
        };
        let semantics = &inputs.estimate.as_ref().unwrap().panels[0].1;
        for (index, row) in payload.rows.iter().enumerate() {
            if semantics.estimates[index].is_none() {
                assert!(row.point.is_none() && row.interval.is_none());
                assert_eq!(row.text.as_ref().unwrap().plan.source, NOT_ESTIMABLE_TEXT);
            }
            for cap in row.caps.iter().flatten() {
                assert_eq!((cap[1].y - cap[0].y).to_bits(), 8.0_f64.to_bits());
            }
        }
    }
}

#[test]
fn raw_interval_intent_survives_correctly_rounded_logarithms() {
    for (upper, nonzero) in [(1e16, false), (1e16 + 2.0, true)] {
        let source = Plot::forest(
            crate::CategoryValues::new("term", ["a"]).unwrap(),
            crate::NumericValues::new("estimate", [1e16]),
            crate::Bounds::new(
                crate::NumericValues::new("lower", [1e16]),
                crate::NumericValues::new("upper", [upper]),
            ),
            1e16,
        )
        .scale(crate::AxisScale::Log10)
        .x_limits(1e15, 1e17)
        .erase();
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        let tuple = inputs.estimate.as_ref().unwrap().panels[0].1.estimates[0].unwrap();
        assert_eq!(tuple.lower.unwrap().to_bits(), 16.0_f64.to_bits());
        assert_eq!(tuple.upper.unwrap().to_bits(), 16.0_f64.to_bits());
        assert_eq!(tuple.nonzero_interval, nonzero);
        let source = inputs.attach(source).unwrap();
        let theme = crate::Theme::Publication.resolve().unwrap();
        let result = source.measure_selected_interior(PlotRenderContext::new(
            source.data(),
            crate::layout::PanelBox::new(80.0, 600.0, 60.0, 400.0),
            &theme,
        ));
        if nonzero {
            assert!(matches!(
                result,
                Err(crate::layout::fit::PlanError::Fatal(Error::Render(_)))
            ));
        } else {
            let selected = result.unwrap();
            let crate::layout::interior::MarkPayload::Estimate(payload) =
                &selected.marks.as_ref().unwrap().payload
            else {
                panic!("wrong payload")
            };
            let interval = payload.rows[0].interval.unwrap();
            assert_eq!(interval[0], interval[1]);
        }
    }
}

#[test]
fn facet_owned_significance_is_retained_only_by_its_selected_panel() {
    let source = Plot::coefficient(
        CategoryValues::new("term", ["same", "same"]).unwrap(),
        CoefficientInput::estimates(NumericValues::nullable("estimate", [None, Some(10.0)])),
    )
    .estimate_status([EstimateStatus::NotEstimable, EstimateStatus::Estimated])
    .significance([
        crate::Significance::at("same", crate::SignificanceMark::TwoStars)
            .facet(crate::FacetCell::single("estimated")),
    ])
    .facet(FacetSpec::column(
        CategoryValues::new("panel", ["missing", "estimated"]).unwrap(),
    ))
    .size(1000, 480)
    .erase();
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
    let panels = &inputs.estimate.as_ref().unwrap().panels;
    assert_eq!(panels[0].1.adjacent, [None]);
    assert_eq!(panels[1].1.adjacent, [Some(("**".into(), true))]);
    STAGES.with(|counts| counts.set([[0; 4]; 3]));
    source.preflight().unwrap();
    let counts = STAGES.with(std::cell::Cell::get)[1];
    assert_eq!(counts[0], 2);
    assert_eq!(counts[3], 2);
}

#[test]
fn final_guides_ticks_and_invalid_tuples_precede_unavailable_fonts() {
    fn with_unavailable_fonts(source: &Plot) -> Result<(), Error> {
        let mut theme = crate::Theme::Publication.resolve().unwrap();
        theme.typography.axes.tick_label.stack =
            crate::font::canonical::named_stack(&["T5a6a3c unavailable font 91f7"]);
        crate::layout::plan_exact_root(
            640,
            480,
            |stage| {
                crate::plots::shared_x::prepare_plot(source, &stage.operation(), None)?;
                let requirements = crate::layout::foundational_plot_requirements(
                    None,
                    source,
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
                theme.prepare_font_environment_from_census(&mut census, presence, stage)
            },
            |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |(), _, (), _| Ok(()),
        )
        .map(|_| ())
    }
    let mut invalid = sources(None, None)
        .map(|source| source.with_test_axis_ticks(0, vec![crate::AxisTick::new(1000.0, "outside")]))
        .to_vec();
    for scales in [FacetScales::FreeX, FacetScales::Free] {
        invalid.extend(sources(Some(scales), None));
    }
    let terms = CategoryValues::new("term", ["invalid"]).unwrap();
    let value = NumericValues::new("estimate", [2.0]);
    let bounds = Bounds::new(
        NumericValues::new("lower", [3.0]),
        NumericValues::new("upper", [1.0]),
    );
    invalid.extend([
        Plot::point_interval(terms.clone(), value.clone(), bounds.clone()).erase(),
        Plot::coefficient(
            terms.clone(),
            CoefficientInput::estimates_with_intervals(value.clone(), bounds.clone()),
        )
        .erase(),
        Plot::forest(terms, value, bounds, 1.0)
            .scale(AxisScale::Log10)
            .erase(),
    ]);
    for source in invalid {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
        assert!(matches!(
            with_unavailable_fonts(&source),
            Err(Error::Data(_))
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
    for value in [f64::NAN, f64::INFINITY, 1000.0] {
        let guide = crate::GuideSpec::vertical(
            value,
            crate::GuideStyle::new((0, 0, 0), 1.0, crate::LinePattern::Solid).unwrap(),
        );
        for source in sources(Some(FacetScales::Fixed), Some(&guide)) {
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
            let error = with_unavailable_fonts(&source).unwrap_err();
            assert!(matches!(error, Error::Data(_)), "{error}");
            assert!(error.to_string().contains("guide"), "{error}");
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
        }
    }
    for source in sources(Some(FacetScales::Fixed), None) {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
        assert!(with_unavailable_fonts(&source)
            .unwrap_err()
            .to_string()
            .contains("font"));
        assert!(crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get) > 0);
    }
}

#[test]
fn logarithmic_shared_limits_and_raw_exact_ticks_are_transformed_once() {
    let source = Plot::forest(
        CategoryValues::new("term", ["a"]).unwrap(),
        NumericValues::new("estimate", [10.0]),
        Bounds::new(
            NumericValues::new("lower", [1.0]),
            NumericValues::new("upper", [100.0]),
        ),
        1.0,
    )
    .scale(AxisScale::Log10)
    .x_ticks(vec![
        crate::AxisTick::new(0.1, "low"),
        crate::AxisTick::new(1000.0, "high"),
    ])
    .erase();
    let resources = OperationContext::new();
    let marks = crate::plots::shared_x::PreparedReference::new(
        &source,
        source.data().with_operation(&resources),
        None,
    )
    .unwrap();
    let inputs = crate::plots::shared_x::prepare_reference_plot(
        &source,
        marks,
        Some("panel-0"),
        Some((-1.0, 3.0)),
        Some(&crate::Labels::humanized()),
    )
    .unwrap();
    assert_eq!(inputs.axes[0].1.ranges[0], (-1.0, 3.0));
    assert_eq!(inputs.axes[0].1.specs[0].limits, Some((-1.0, 3.0)));
    let ticks = inputs.axes[0].1.specs[0].exact_ticks.as_ref().unwrap();
    assert_eq!(
        ticks.iter().map(|tick| tick.position).collect::<Vec<_>>(),
        [-1.0, 3.0]
    );
    assert_eq!(
        inputs.estimate.as_ref().unwrap().panels[0].1.estimates[0]
            .unwrap()
            .estimate
            .to_bits(),
        1.0_f64.to_bits()
    );
}

#[test]
fn borrowed_views_preserve_original_source_row_diagnostics() {
    let terms = CategoryValues::new("term", ["a", "b", "c"]).unwrap();
    let values = NumericValues::new("estimate", [1.0, 2.0, 3.0]);
    let bounds = Bounds::new(
        NumericValues::new("lower", [0.5, 1.0, 4.0]),
        NumericValues::new("upper", [2.0, 3.0, 5.0]),
    );
    let sources = [
        Plot::point_interval(terms.clone(), values.clone(), bounds.clone()).erase(),
        Plot::coefficient(
            terms.clone(),
            CoefficientInput::estimates_with_intervals(values.clone(), bounds.clone()),
        )
        .erase(),
        Plot::forest(terms, values, bounds, 1.0)
            .scale(AxisScale::Log10)
            .erase(),
    ];
    for source in sources {
        let resources = OperationContext::new();
        let data =
            crate::data::PlotData::with_rows(&source, &[2], &[42]).with_operation(&resources);
        let error =
            PreparedEstimate::new(&source, data, source.significance_annotations()).unwrap_err();
        assert!(error.to_string().contains("source row 42"), "{error}");
    }
}
