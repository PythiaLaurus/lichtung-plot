use super::plan_panel_for_test as plan_panel;
use super::*;

#[test]
fn p2_panel_call_refusal_precedes_metadata_retention_and_body_work() {
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [1.0]),
        crate::NumericValues::new("y", [2.0]),
    )
    .title("metadata ".repeat(8192))
    .erase();
    let theme = theme();
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    resources.claim_calls(1_000_000).unwrap();
    let before = (resources.usage().unwrap(), resources.byte_usage().unwrap());
    crate::layout::plan_exact_root(
        800,
        600,
        |_| Ok(()),
        |(), assigned, round| {
            let result = super::plan_panel(
                &plot,
                plot.kind(),
                PanelBox::new(0.0, 800.0, 0.0, 600.0),
                LegendRole::Panel,
                &theme,
                round,
                &ComponentAddress::root(),
            );
            let error = result
                .err()
                .expect("call refusal before unprepared body or clone");
            assert!(error.to_string().contains("panel frame call admission"));
            assert_eq!(
                (resources.usage().unwrap(), resources.byte_usage().unwrap()),
                before
            );
            Ok(crate::layout::MeasuredLayout::new(assigned, ()))
        },
        |(), _, (), _| Ok(()),
    )
    .unwrap();
}

#[test]
fn p2_planned_limit_metadata_is_admitted_for_every_copy_without_processing() {
    let title = "metadata ".repeat(64);
    let caption = "retained caption";
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [1.0]),
        crate::NumericValues::new("y", [2.0]),
    )
    .title(&title)
    .caption(caption)
    .erase();
    let plan = PanelPlan {
        panel_box: PanelBox::new(0.0, 800.0, 0.0, 600.0),
        result: PlotRenderResult::new(crate::layout::Layout::self_managed(800, 600), Vec::new()),
        legend: None,
        planned_x_limits: Some((0.0, 3.0)),
        planned_y_limits: Some((0.0, 4.0)),
    };
    let bytes = (title.len() + caption.len()) as u64;
    let resources =
        crate::font::canonical::OperationGenerationResources::with_test_limits(0, 2 * bytes);
    let data = plot.data().with_axis_resources(&resources);
    for count in 1..=2 {
        let retained = plan.plot_with_planned_limits(&plot, data).unwrap();
        assert_eq!(retained.title_text(), Some(title.as_str()));
        assert_eq!(retained.caption_text(), Some(caption));
        assert_eq!(resources.usage().unwrap(), (3 * u128::from(count), 0));
        assert_eq!(resources.byte_usage().unwrap(), count * bytes);
    }
    assert!(plan.plot_with_planned_limits(&plot, data).is_err());
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one real frame fixture checks Auto recovery, fixed scoped refusal, and Top source work"
)]
fn marginal_auto_recovers_from_scoped_terminal_tick_miss() {
    use crate::layout::fit::FitOutcome;
    use crate::orchestrator::measurements::{self, Counter, GuideOutcome, GuidePhase};
    let plot = Plot::marginal(
        crate::NumericValues::new("x", [-1e-100, -2e-100, -3e-100]),
        crate::NumericValues::new("y", [0.0, 1.0, 2.0]),
    )
    .hue(crate::CategoryValues::new("group", ["Group"; 3]).unwrap())
    .x_label("")
    .y_label("")
    .x_limits(-4e-100, -0.5e-100)
    .size(640, 480)
    .legend(LegendPolicy::Visible(LegendPosition::Auto))
    .erase();
    let mut theme = crate::Theme::Publication.resolve();
    theme.typography.tick_label.font_size = 48;
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let prepared = crate::plots::shared_x::prepare_plot(&plot, &resources).unwrap();
    let plot = prepared.attach(plot).unwrap();
    let cell = PanelBox::new(80.0, 480.0, 60.0, 450.0);
    measurements::start();
    let auto = plan_panel(&plot, plot.kind(), cell, LegendRole::Panel, &theme);
    let observed = measurements::finish();
    assert!(auto.is_ok(), "{:?}; {:?}", auto.err(), observed.guides);
    assert!(
        observed
            .guides
            .iter()
            .any(|trial| trial.edge == LegendPosition::OutsideRight
                && trial.phase == GuidePhase::Body
                && trial.outcome == GuideOutcome::Rejected),
        "{:?}",
        observed.guides
    );
    assert!(
        observed
            .guides
            .iter()
            .any(|trial| trial.edge == LegendPosition::OutsideTop
                && trial.outcome == GuideOutcome::Accepted),
        "{:?}",
        observed.guides
    );
    assert_eq!(
        observed.counters[Counter::TickFitMiss as usize],
        u64::try_from(
            observed
                .guides
                .iter()
                .filter(|trial| trial.edge == LegendPosition::OutsideRight
                    && trial.phase == GuidePhase::Body
                    && trial.outcome == GuideOutcome::Rejected)
                .count()
        )
        .unwrap()
    );
    assert!(observed
        .guides
        .iter()
        .all(|trial| trial.outcome != GuideOutcome::Aborted));

    let fixed = plot
        .clone()
        .with_legend_position(LegendPosition::OutsideRight);
    let mut failure = None;
    measurements::start();
    let result = crate::layout::plan_exact_root(
        fixed.width(),
        fixed.height(),
        |_| Ok(()),
        |(), _, round| match super::plan_panel(
            &fixed,
            fixed.kind(),
            cell,
            LegendRole::Panel,
            &theme,
            round,
            &ComponentAddress::root(),
        )? {
            FitOutcome::Miss(miss) => {
                let error = Error::Render(miss.to_string());
                failure = Some(miss);
                Err::<crate::layout::MeasuredLayout<()>, _>(error)
            }
            FitOutcome::Measured(_) => panic!("fixed Right must not recover to Top"),
        },
        |(), _, (), _| Ok(()),
    );
    assert!(result.is_err());
    let fixed_observed = measurements::finish();
    let failure = failure.expect("fixed Right must retain a typed miss");
    assert_eq!(failure.reason.kind, FitKind::Tick, "{failure}");
    assert_eq!(
        failure.reason.address,
        ComponentAddress::root()
            .child(AddressRole::AxisBottom, 0)
            .unwrap()
    );
    assert_eq!(fixed_observed.counters[Counter::TickFitMiss as usize], 1);
    assert!(fixed_observed
        .guides
        .iter()
        .all(|trial| trial.edge == LegendPosition::OutsideRight
            && trial.outcome == GuideOutcome::Rejected));
    measurements::start();
    plan_panel(
        &plot.with_legend_position(LegendPosition::OutsideTop),
        PlotKind::Marginal,
        cell,
        LegendRole::Panel,
        &theme,
    )
    .unwrap();
    let top = measurements::finish();
    assert_eq!(top.counters[Counter::TickFitMiss as usize], 0);
    assert!(
        observed.counters[Counter::TickSource as usize]
            > top.counters[Counter::TickSource as usize]
    );
}

#[test]
fn automatic_guide_recovers_from_terminal_numeric_tick_miss() {
    use crate::orchestrator::measurements::{self, Counter, GuideOutcome, GuidePhase};
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [-1e-100, -2e-100]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .hue(crate::CategoryValues::new("group", ["MMMMMMMMMMMMMMMMMM"; 2]).unwrap())
    .x_label("")
    .y_label("")
    .size(640, 480)
    .legend(LegendPolicy::Visible(LegendPosition::Auto))
    .erase();
    let mut theme = crate::Theme::Publication.resolve();
    theme.typography.tick_label.font_size = 40;
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let prepared = crate::plots::shared_x::prepare_plot(&plot, &resources).unwrap();
    let plot = prepared.attach(plot).unwrap();
    let cell = PanelBox::new(80.0, 480.0, 60.0, 450.0);
    measurements::start();
    let before = resources.usage().unwrap();
    let auto = plan_panel(&plot, plot.kind(), cell, LegendRole::Panel, &theme);
    let observed = measurements::finish();
    assert!(auto.is_ok(), "{:?}; {:?}", auto.err(), observed.guides);
    assert!(
        observed
            .guides
            .iter()
            .any(|trial| trial.edge == LegendPosition::OutsideRight
                && trial.phase == GuidePhase::Body
                && trial.outcome == GuideOutcome::Rejected),
        "{:?}",
        observed.guides
    );
    assert!(
        observed
            .guides
            .iter()
            .any(|trial| trial.edge == LegendPosition::OutsideTop
                && trial.outcome == GuideOutcome::Accepted),
        "{:?}",
        observed.guides
    );
    assert!(observed.counters[Counter::TickSource as usize] > 0);
    assert_eq!(observed.counters[Counter::TickFitMiss as usize], 3);
    assert!(resources.usage().unwrap().1 > before.1);
    assert!(observed
        .guides
        .iter()
        .all(|trial| trial.outcome != GuideOutcome::Aborted));
    measurements::start();
    let error = plan_panel(
        &plot
            .clone()
            .with_legend_position(LegendPosition::OutsideRight),
        PlotKind::Scatter,
        cell,
        LegendRole::Panel,
        &theme,
    )
    .err()
    .expect("fixed right must fail");
    let fixed = measurements::finish();
    assert!(error.to_string().contains("layout fit"), "{error}");
    assert_eq!(fixed.counters[Counter::TickFitMiss as usize], 1);
    assert!(fixed
        .guides
        .iter()
        .all(|trial| trial.edge == LegendPosition::OutsideRight
            && trial.outcome == GuideOutcome::Rejected));
    measurements::start();
    plan_panel(
        &plot.with_legend_position(LegendPosition::OutsideTop),
        PlotKind::Scatter,
        cell,
        LegendRole::Panel,
        &theme,
    )
    .unwrap();
    let top = measurements::finish();
    assert_eq!(top.counters[Counter::TickFitMiss as usize], 0);
    assert!(
        observed.counters[Counter::TickSource as usize]
            > top.counters[Counter::TickSource as usize]
    );
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one freeze-to-capture harness covers all specialized producer families"
)]
fn specialized_producer_plans_survive_first_capture_without_shaping() {
    let source = format!(
        "{}AA\u{2029}A \u{202b}\u{5d0} A \u{5d0} \u{202c}A",
        "A\u{ad}".repeat(60)
    );
    let plots = [
        Plot::beeswarm(
            crate::CategoryValues::new("category", ["one", "two"]).unwrap(),
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("color", [0.0, 1.0]),
        )
        .color_label(&source)
        .erase(),
        Plot::heatmap(
            crate::CategoryValues::new("column", ["left", "right", "left", "right"]).unwrap(),
            crate::CategoryValues::new("row", ["finite", "finite", "missing", "missing"]).unwrap(),
            crate::NumericValues::nullable("value", [Some(0.12), Some(0.88), None, None]),
        )
        .color_label(&source)
        .annotations(crate::HeatmapAnnotations::Values(
            crate::NumericLabelFormat::default(),
        ))
        .heatmap_null_row_annotation(
            crate::HeatmapNullRowAnnotation::new("missing", &source).unwrap(),
        )
        .erase(),
        Plot::matrix(vec![
            crate::NumericValues::new(&source, [0.0, 1.0, 2.0, 3.0]),
            crate::NumericValues::new("other", [1.0, 0.0, 3.0, 2.0]),
        ])
        .labels(crate::Labels::raw())
        .hue(
            crate::CategoryValues::new("group", ["A\u{301}", "A\u{301}", "A\u{301}", "A\u{301}"])
                .unwrap(),
        )
        .erase(),
        Plot::line(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .point_layer(
            crate::PointLayer::new(
                crate::NumericValues::new("px", [0.25]),
                crate::NumericValues::new("py", [0.5]),
            )
            .labels(
                crate::CategoryValues::new("label", ["A\u{301}\u{2029}A \u{202b}\u{5d0}\u{202c}"])
                    .unwrap(),
            ),
        )
        .panel_text_annotation(
            crate::PanelTextAnnotation::new(crate::PanelCorner::TopLeft, "A\u{301} \u{5d0}")
                .unwrap(),
        )
        .erase(),
    ];
    for plot in plots {
        let plot = plot.with_canvas_size(1000, 1000);
        let kind = plot.kind();
        let theme = crate::Theme::Publication.resolve();
        crate::layout::plan_exact_root(
            plot.width(),
            plot.height(),
            |stage| {
                let inputs = if matches!(
                    kind,
                    PlotKind::Line | PlotKind::Heatmap | PlotKind::Matrix | PlotKind::Beeswarm
                ) {
                    Some(crate::plots::shared_x::prepare_plot(
                        &plot,
                        &stage.generation_resources(),
                    )?)
                } else {
                    None
                };
                let requirements = crate::layout::foundational_plot_requirements(
                    inputs.as_ref(),
                    &plot,
                    kind,
                    &theme,
                    &ComponentAddress::root(),
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
                let plot = inputs
                    .as_ref()
                    .map_or_else(|| Ok(plot.clone()), |inputs| inputs.attach(plot.clone()))?;
                let panel = PanelBox::new(100.0, 800.0, 100.0, 850.0);
                let result = plot
                    .measure_panel(kind, PlotRenderContext::new(plot.data(), panel, &theme))
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                Ok(crate::layout::MeasuredLayout::new(
                    assigned,
                    (panel, result),
                ))
            },
            |inputs, assigned, (panel, result), stage| {
                let plot = inputs
                    .as_ref()
                    .map_or_else(|| Ok(plot.clone()), |inputs| inputs.attach(plot.clone()))?;
                assert!(
                    result.marks.is_some() || !result.producer_text.is_empty(),
                    "{kind:?}"
                );
                if !matches!(kind, PlotKind::Line | PlotKind::Matrix) {
                    assert!(
                        result
                            .producer_text
                            .iter()
                            .any(|text| text.plan.source.contains('\u{2029}')),
                        "{kind:?}"
                    );
                    assert!(
                        result.producer_text.iter().any(|text| text
                            .plan
                            .lines
                            .iter()
                            .any(|line| line.selected_soft_hyphen.is_some())),
                        "{kind:?}"
                    );
                }
                crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
                crate::orchestrator::measurements::start();
                let (width, height) = assigned.dimensions();
                let replay = crate::orchestrator::capture_text_replay_plan(
                    width,
                    height,
                    "specialized producer closure",
                    &theme,
                    stage,
                    |backend| {
                        plot.render_panel(
                            kind,
                            PlotRenderContext::new(plot.data(), panel, &theme)
                                .with_retained_body(&result),
                            backend,
                        )
                        .map(|_| ())
                    },
                )?;
                let capture_work = crate::orchestrator::measurements::finish();
                for counter in [
                    crate::orchestrator::measurements::Counter::TickSource,
                    crate::orchestrator::measurements::Counter::TickDistinctList,
                    crate::orchestrator::measurements::Counter::TickMetricLookup,
                ] {
                    assert_eq!(
                        capture_work.counters[counter as usize], 0,
                        "{kind:?}: {counter:?}"
                    );
                }
                crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| {
                    assert_eq!(calls.get(), 0, "{kind:?} shaped during first capture");
                });
                let captured = replay.retained_text();
                if kind == PlotKind::Matrix {
                    // Matrix now retains its text inside the common axis plans.
                    assert!(result.producer_text.is_empty());
                    assert!(captured
                        .iter()
                        .any(|(_, text)| text.source.contains('\u{2029}')));
                    assert!(captured.iter().any(|(_, text)| text
                        .lines
                        .iter()
                        .any(|line| line.selected_soft_hyphen.is_some())));
                    for text in result.self_managed_legend.as_ref().unwrap().text_plans() {
                        assert!(captured
                            .iter()
                            .any(|(_, actual)| actual.shares_payload(text) && *actual == text));
                    }
                }
                for text in &result.producer_text {
                    assert!(
                        captured
                            .iter()
                            .any(|(placement, actual)| *placement == text.placement
                                && actual.shares_payload(&text.plan)
                                && **actual == text.plan),
                        "{kind:?}"
                    );
                }
                for text in &result.panel_text {
                    assert!(captured
                        .iter()
                        .any(|(_, actual)| actual.shares_payload(&text.text_plan)
                            && **actual == text.text_plan));
                }
                Ok(())
            },
        )
        .unwrap_or_else(|error| panic!("{kind:?}: {error}"));
    }
}

#[test]
fn automatic_outside_guide_respects_native_body_intrinsics_before_allocation() {
    let source = Plot::scatter(
        NumericValues::new("x", [0.0, 1.0, 2.0, 3.0]),
        NumericValues::new("y", [1.0, 2.0, 1.5, 2.5]),
    )
    .hue(
        CategoryValues::new(
            "hue",
            [
                "Complete explanatory first condition",
                "Complete explanatory first condition",
                "Complete explanatory second condition",
                "Complete explanatory second condition",
            ],
        )
        .unwrap(),
    )
    .x_label("MMMMMMMMMMMMMMMMMMMMMMMMMMMM")
    .size(640, 480)
    .erase();
    let cell = PanelBox::new(80.0, 600.0, 80.0, 440.0);
    for automatic in [true, false] {
        let plot = if automatic {
            source.clone()
        } else {
            source
                .clone()
                .with_legend_position(LegendPosition::OutsideRight)
        };
        let theme = theme();
        let root = crate::layout::plan_exact_root(
            plot.width(),
            plot.height(),
            |stage| {
                let inputs =
                    crate::plots::shared_x::prepare_plot(&plot, &stage.generation_resources())?;
                let requirements = crate::layout::foundational_plot_requirements(
                    None,
                    &plot,
                    PlotKind::Scatter,
                    &theme,
                    &ComponentAddress::root(),
                    true,
                    true,
                    true,
                    None,
                )?;
                let mut presence = stage.reserve_scalar_presence_map()?;
                let mut census = crate::layout::PreparedTextCensus::materialize(
                    requirements,
                    Some(stage),
                    Some(&mut presence),
                )
                .map_err(|error| Error::Render(format!("D1 text census: {error}")))?;
                theme.prepare_font_environment_from_census(&mut census, presence, stage)?;
                Ok(inputs)
            },
            |inputs, assigned, round| {
                let plot = inputs.attach(plot.clone())?;
                Ok(crate::layout::MeasuredLayout::new(
                    assigned,
                    crate::layout::fit::measured(super::plan_panel(
                        &plot,
                        PlotKind::Scatter,
                        cell,
                        LegendRole::Panel,
                        &theme,
                        round,
                        &ComponentAddress::root(),
                    ))
                    .map_err(crate::layout::fit::PlanError::into_error)?,
                ))
            },
            |_, _, plan, _| Ok(plan),
        );
        if automatic {
            let root = root.unwrap();
            let plan = root.plan();
            let guide = plan.legend.as_ref().unwrap();
            assert!(
                guide.bounds.bottom > plan.panel_box.y1,
                "the complete top frame, not merely a right guide, must win"
            );
            assert_eq!(
                plan.result.layout.x_axis.label,
                "MMMMMMMMMMMMMMMMMMMMMMMMMMMM"
            );
        } else {
            let error = root.err().unwrap();
            assert!(error.to_string().contains("layout fit"), "{error}");
        }
    }
}

#[test]
fn explicit_left_and_bottom_guides_reserve_complete_axis_ink() {
    let source = Plot::scatter(
        NumericValues::new("x", [0.0, 1.0, 2.0, 3.0]),
        NumericValues::new("y", [1.0, 2.0, 1.5, 2.5]),
    )
    .hue(CategoryValues::new("hue", ["first", "first", "second", "second"]).unwrap())
    .erase();
    for edge in [LegendPosition::OutsideLeft, LegendPosition::OutsideBottom] {
        let plot = source.clone().with_legend_position(edge);
        let theme = theme();
        let plan = plan_panel(
            &plot,
            PlotKind::Scatter,
            PanelBox::new(80.0, 600.0, 80.0, 440.0),
            LegendRole::Panel,
            &theme,
        )
        .unwrap();
        let guide = plan.legend.as_ref().unwrap();
        let mut chrome = GeometryBackend::new(&theme).unwrap();
        chrome.begin(plot.width(), plot.height()).unwrap();
        plan.result
            .axes
            .as_ref()
            .unwrap()
            .draw(&mut chrome)
            .unwrap();
        chrome.end().unwrap();
        for bounds in chrome.bounds() {
            assert!(
                bounds.right <= guide.bounds.left
                    || bounds.left >= guide.bounds.right
                    || bounds.top <= guide.bounds.bottom
                    || bounds.bottom >= guide.bounds.top
            );
        }
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "explicit boundary cases trace fatal arithmetic through every clearance owner"
)]
fn clearance_arithmetic_failures_are_fatal_through_every_owner() {
    let plot =
        crate::kind::into_erased(line_plot(&line_data(["upper", "upper", "lower", "lower"])));
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let plot = crate::plots::shared_x::prepare_plot(&plot, &resources)
        .unwrap()
        .attach(plot)
        .unwrap();
    let theme = theme();
    let cell = PanelBox::new(60.0, 500.0, 50.0, 350.0);
    let (mut result, _) = discover_geometry(
        &plot,
        PlotKind::Line,
        PlotRenderContext::new(plot.data(), cell, &theme),
    )
    .unwrap();
    let legend = crate::decorations::legend::test_assigned_legend(
        "hue",
        result.legend_entries.clone(),
        (cell.width(), cell.height()),
        &theme,
    )
    .unwrap();
    let geometry = [crate::render::geometry::GeometryBox {
        left: cell.x0,
        right: cell.x1,
        bottom: cell.y0,
        top: cell.y1,
        weight: 1.0,
    }];
    for range in [
        (-f64::MAX, f64::MAX),
        (0.0, f64::MAX / 2.0),
        (f64::MAX * 0.8, f64::MAX * 0.9),
    ] {
        result.layout.projection =
            crate::layout::Projection::new(range, (0.0, 1.0), cell.as_tuple());
        let bounds = legend.box_at(
            cell.x0 + cell.width() * 0.1 + legend.metrics.font_size.max(6.0),
            cell.y0,
        );
        assert!(expanded_axis_limits(
            PlotKind::Line,
            &plot,
            &result,
            &legend,
            bounds,
            ClearanceEdge::Right
        )
        .unwrap_err()
        .to_string()
        .contains("clearance arithmetic"));
        assert!(explicit_clearance(
            PlotKind::Line,
            &plot,
            &result,
            &legend,
            bounds,
            ClearanceEdge::Right
        )
        .is_err());
    }
    result.layout.projection = crate::layout::Projection::new(
        (f64::MAX * 0.8, f64::MAX * 0.9),
        (0.0, 1.0),
        cell.as_tuple(),
    );
    let bounds = legend.box_at(
        cell.x0 + cell.width() * 0.5 + legend.metrics.font_size.max(6.0),
        cell.y0,
    );
    assert!(expanded_axis_limits(
        PlotKind::Line,
        &plot,
        &result,
        &legend,
        bounds,
        ClearanceEdge::Right
    )
    .unwrap_err()
    .to_string()
    .contains("expanded domain maximum"));
    result.layout.projection = crate::layout::Projection::new(
        (-f64::MAX * 0.9, -f64::MAX * 0.8),
        (0.0, 1.0),
        cell.as_tuple(),
    );
    let bounds = legend.box_at(
        cell.x1
            - cell.width() * 0.5
            - legend.metrics.font_size.max(6.0)
            - legend.compute_box_width(),
        cell.y0,
    );
    assert!(expanded_axis_limits(
        PlotKind::Line,
        &plot,
        &result,
        &legend,
        bounds,
        ClearanceEdge::Left
    )
    .unwrap_err()
    .to_string()
    .contains("expanded domain minimum"));
    result.layout.projection =
        crate::layout::Projection::new((-f64::MAX, f64::MAX), (0.0, 1.0), cell.as_tuple());
    assert!(top_right_clearance(
        PlotKind::Line,
        &plot,
        &result,
        &legend,
        &geometry,
        LegendClearance::HorizontalMaximum,
        0,
        &theme,
        &crate::layout::fit::ROOT_SCOPE
    )
    .is_err());
    assert!(automatic_clearance(
        PlotKind::Line,
        &plot,
        &result,
        &legend,
        &geometry,
        LegendDefault::LowestOverlap,
        0,
        &theme,
        &crate::layout::fit::ROOT_SCOPE
    )
    .is_err());
    assert!(expansion_ratio((0.0, f64::from_bits(1)), (0.0, f64::MAX))
        .unwrap_err()
        .to_string()
        .contains("expansion ratio"));
}

#[test]
fn annotation_conflict_links_only_to_a_resolving_assignment() {
    let theme = theme();
    let note = "Caller annotation";
    let source = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 0.0]),
    )
    .hue(crate::CategoryValues::new("hue", ["a", "b"]).unwrap())
    .legend(LegendPolicy::Visible(LegendPosition::InsideRightTop));
    let initial_source = source.clone().erase();
    let initial_source = crate::plots::shared_x::prepare_plot(
        &initial_source,
        &theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources(),
    )
    .unwrap()
    .attach(initial_source)
    .unwrap();
    let initial = initial_source
        .preflight_panel(
            PlotKind::Scatter,
            PanelBox::new(100.0, 500.0, 80.0, 400.0),
            &theme,
        )
        .unwrap();
    let legend = crate::decorations::legend::test_assigned_legend(
        "hue",
        initial.legend_entries,
        (400.0, 320.0),
        &theme,
    )
    .unwrap();
    let note_width = theme
        .measure_legacy_text(
            note,
            theme.detail_font_size,
            crate::layout::PhysicalTextAlignment::Start,
            "panel text annotation",
        )
        .unwrap()
        .width();
    let required = initial.axes.as_ref().unwrap().requirements();
    let axis_extra = (required.left + 4.0).ceil() + (required.right + 4.0).ceil();
    let width = note_width + legend.compute_box_width() + 36.0 + axis_extra;
    let cell = PanelBox::new(100.0, 100.0 + width, 80.0, 400.0);
    let source = source.panel_text_annotation(
        crate::PanelTextAnnotation::new(crate::PanelCorner::TopLeft, note).unwrap(),
    );
    let address = ComponentAddress::root();
    let mut calls = 0;
    crate::layout::plan_exact_root(
        source.width(),
        source.height(),
        |stage| {
            crate::plots::shared_x::prepare_plot(
                &source.clone().erase(),
                &stage.generation_resources(),
            )
        },
        |inputs, assigned, round| {
            let source = inputs.attach(source.clone().erase())?;
            calls += 1;
            if calls == 1 {
                round.seed_insets_for_test(&address, [axis_extra + 40.0, 0.0, 0.0, 0.0])?;
            }
            let plan = crate::layout::fit::measured(super::plan_panel(
                &source,
                PlotKind::Scatter,
                cell,
                LegendRole::Panel,
                &theme,
                round,
                &address,
            ))
            .map_err(crate::layout::fit::PlanError::into_error)?;
            Ok(crate::layout::MeasuredLayout::new(assigned, plan))
        },
        |_, _, _, _| Ok(()),
    )
    .unwrap();
    assert!(calls >= 2);
    let narrow = PanelBox::new(cell.x0 + 40.0, cell.x1, cell.y0, cell.y1);
    let result = plan_panel(
        &source,
        PlotKind::Scatter,
        narrow,
        LegendRole::Panel,
        &theme,
    );
    assert!(result
        .err()
        .unwrap()
        .to_string()
        .contains("panel text annotation"));
}
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "the automatic regression retains all four blocked corners under distinct initial allocations"
)]
fn automatic_outside_guides_avoid_annotations_at_all_inside_corners() {
    let note = "Caller annotation";
    let source = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 0.0]),
    )
    .hue(crate::CategoryValues::new("hue", ["a", "b"]).unwrap())
    .legend(LegendPolicy::Automatic)
    .panel_text_annotation(
        crate::PanelTextAnnotation::new(crate::PanelCorner::TopLeft, note).unwrap(),
    )
    .panel_text_annotation(
        crate::PanelTextAnnotation::new(crate::PanelCorner::BottomLeft, note).unwrap(),
    )
    .erase();
    let address = ComponentAddress::root();
    for (resolves, seed) in [
        (true, [40.0, 0.0, 0.0, 0.0]),
        (false, [0.0; 4]),
        (false, [0.0, 0.0, 40.0, 0.0]),
    ] {
        let theme = theme();
        let mut calls = 0;
        let result = crate::layout::plan_exact_root(
            source.width(),
            source.height(),
            |stage| {
                let inputs =
                    crate::plots::shared_x::prepare_plot(&source, &stage.generation_resources())?;
                let source = inputs.attach(source.clone())?;
                let requirements = crate::layout::foundational_plot_requirements(
                    None,
                    &source,
                    PlotKind::Scatter,
                    &theme,
                    &address,
                    true,
                    true,
                    true,
                    None,
                )?;
                stage.claim_topology_slots(
                    crate::layout::PreparedTextCensus::structural_address_count(&requirements)
                        .map_err(|error| {
                            Error::Render(format!("automatic annotation test topology: {error}"))
                        })?,
                )?;
                let mut presence = stage.reserve_scalar_presence_map()?;
                let mut census = crate::layout::PreparedTextCensus::materialize(
                    requirements,
                    Some(stage),
                    Some(&mut presence),
                )
                .map_err(|error| {
                    Error::Render(format!("automatic annotation test census: {error}"))
                })?;
                theme.prepare_font_environment_from_census(&mut census, presence, stage)?;
                let initial = source.preflight_panel(
                    PlotKind::Scatter,
                    PanelBox::new(100.0, 500.0, 80.0, 400.0),
                    &theme,
                )?;
                let title = crate::decorations::legend::resolved_title(
                    &initial.legend_entries,
                    &theme.prepared_font_environment()?.generation_resources(),
                )?;
                let legend = crate::decorations::legend::test_assigned_legend(
                    &title,
                    initial.legend_entries,
                    (400.0, 320.0),
                    &theme,
                )?;
                let note_width = theme
                    .measure_legacy_text(
                        note,
                        theme.detail_font_size,
                        crate::layout::PhysicalTextAlignment::Start,
                        "panel text annotation",
                    )?
                    .width();
                let required = initial.axes.as_ref().unwrap().requirements();
                let axis_width = (required.left + 4.0).ceil() + (required.right + 4.0).ceil();
                let cell = PanelBox::new(
                    100.0,
                    100.0 + note_width + legend.compute_box_width() + 36.0,
                    80.0,
                    400.0,
                );
                let narrow = PanelBox::new(cell.x0 + 40.0, cell.x1, cell.y0, cell.y1);
                let mut current = initial.layout;
                current.projection = crate::layout::Projection::new(
                    current.projection.x_range(),
                    current.projection.y_range(),
                    narrow.as_tuple(),
                );
                let annotations = crate::decorations::panel_text::plans(&source, &current, &theme)?;
                for bounds in legend.inside_candidates(narrow.as_tuple()) {
                    assert!(annotations
                        .iter()
                        .any(|plan| !plan.avoids_legend_in(narrow, bounds)));
                }
                let mut containing = if resolves { cell } else { narrow };
                containing.x1 += axis_width;
                Ok((containing, annotations, inputs))
            },
            |(cell, _, inputs), assigned, round| {
                let source = inputs.attach(source.clone())?;
                calls += 1;
                if calls == 1 {
                    round.seed_insets_for_test(&address, seed)?;
                }
                let plan = crate::layout::fit::measured(super::plan_panel(
                    &source,
                    PlotKind::Scatter,
                    *cell,
                    LegendRole::Panel,
                    &theme,
                    round,
                    &address,
                ))
                .map_err(crate::layout::fit::PlanError::into_error)?;
                Ok(crate::layout::MeasuredLayout::new(assigned, plan))
            },
            |_, _, plan, _| Ok(plan),
        );
        {
            let root = result.unwrap();
            assert!((1..=8).contains(&calls));
            let plan = root.plan();
            let legend = plan.legend.as_ref().unwrap();
            assert!(!legend_fits_panel(
                legend.bounds,
                plan.result.layout.projection.panel_box()
            ));
            assert!(root.prepared().1.iter().all(|annotation| annotation
                .avoids_legend_in(plan.result.layout.projection.panel_box(), legend.bounds)));
            let mut invalid_theme = crate::Theme::Publication.resolve();
            invalid_theme.detail_font_size = 0;
            assert!(auto_candidate_avoiding_panel_text(
                &source,
                &plan.result,
                &legend.legend,
                0,
                &invalid_theme,
                &crate::layout::fit::FitScope::root()
            )
            .is_err());
        }
    }
}

use crate::render::backend::{Backend, LineStyle, Point2D};
use crate::{CategoryValues, NumericValues};

fn theme() -> crate::theme::ResolvedTheme {
    crate::Theme::Publication.resolve()
}

struct LineValues {
    x: NumericValues,
    y: NumericValues,
    low: NumericValues,
    high: NumericValues,
    hue: CategoryValues,
}

fn line_data(labels: [&str; 4]) -> LineValues {
    LineValues {
        x: NumericValues::new("x", [0.0, 1.0, 0.0, 1.0]),
        y: NumericValues::new("y", [8.0, 10.0, 1.0, 2.0]),
        low: NumericValues::new("low", [7.0, 9.0, 0.5, 1.5]),
        high: NumericValues::new("high", [9.0, 11.0, 1.5, 2.5]),
        hue: CategoryValues::new("hue", labels).unwrap(),
    }
}

fn line_plot(data: &LineValues) -> Plot<crate::kind::Line> {
    Plot::line(data.x.clone(), data.y.clone())
        .group(data.hue.clone())
        .hue(data.hue.clone())
        .band(crate::Bounds::new(data.low.clone(), data.high.clone()))
}

struct DenseBarValues {
    category: CategoryValues,
    value: NumericValues,
    hue: CategoryValues,
}

fn dense_bar_plot(data: &DenseBarValues, horizontal: bool) -> Plot {
    let input = crate::BarInput::estimates(data.value.clone());
    if horizontal {
        Plot::horizontal_bar(data.category.clone(), input)
            .series(data.hue.clone())
            .erase()
    } else {
        Plot::bar(data.category.clone(), input)
            .series(data.hue.clone())
            .erase()
    }
}

fn dense_horizontal_data() -> DenseBarValues {
    signed_dense_bar_data(1.0)
}

fn signed_dense_bar_data(sign: f64) -> DenseBarValues {
    DenseBarValues {
        category: CategoryValues::new(
            "category",
            ["a", "a", "b", "b", "c", "c", "d", "d", "e", "e", "f", "f"],
        )
        .unwrap(),
        value: NumericValues::new(
            "value",
            [
                10.0, 9.0, 10.0, 9.0, 10.0, 9.0, 10.0, 9.0, 10.0, 9.0, 10.0, 9.0,
            ]
            .map(|value| value * sign),
        ),
        hue: CategoryValues::new(
            "hue",
            [
                "one", "two", "one", "two", "one", "two", "one", "two", "one", "two", "one", "two",
            ],
        )
        .unwrap(),
    }
}

#[test]
fn auto_legend_avoids_recorded_line_and_ribbon_geometry() {
    let data = line_data(["upper", "upper", "lower", "lower"]);
    let plot = line_plot(&data);
    let cell = PanelBox::new(60.0, 440.0, 50.0, 310.0);

    let plan = plan_panel(&plot, PlotKind::Line, cell, LegendRole::Panel, &theme()).unwrap();
    let legend = plan.legend.as_ref().unwrap();
    let plot = plot.erase();
    let plot_with_planned_limits = plan.plot_with_planned_limits(&plot, plot.data()).unwrap();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let plot_with_planned_limits =
        crate::plots::shared_x::prepare_plot(&plot_with_planned_limits, &resources)
            .unwrap()
            .attach(plot_with_planned_limits)
            .unwrap();
    let (result, geometry) = discover_geometry(
        &plot_with_planned_limits,
        PlotKind::Line,
        PlotRenderContext::new(
            crate::data::PlotData::new(&plot_with_planned_limits),
            plan.panel_box,
            &theme(),
        ),
    )
    .unwrap();
    let margin = legend.legend.metrics.font_size.max(6.0);

    assert!(
        overlap_score_within_panel(
            legend.bounds,
            geometry.bounds(),
            margin,
            result.layout.projection.panel_box(),
        ) <= f64::EPSILON
    );
}

#[test]
fn production_geometry_discovery_returns_a_backend_independent_snapshot() {
    let data = line_data(["upper", "upper", "lower", "lower"]);
    let plot = crate::kind::into_erased(line_plot(&data));
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let plot = crate::plots::shared_x::prepare_plot(&plot, &resources)
        .unwrap()
        .attach(plot)
        .unwrap();
    let cell = PanelBox::new(60.0, 440.0, 50.0, 310.0);
    let (_, geometry) = discover_geometry(
        &plot,
        PlotKind::Line,
        PlotRenderContext::new(crate::data::PlotData::new(&plot), cell, &theme()),
    )
    .unwrap();

    assert!(!geometry.bounds().is_empty());
}

#[test]
fn automatic_legend_ignores_zero_width_lines_but_avoids_painted_lines() {
    let data = line_data(["upper", "upper", "lower", "lower"]);
    let plot = crate::kind::into_erased(line_plot(&data));
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let plot = crate::plots::shared_x::prepare_plot(&plot, &resources)
        .unwrap()
        .attach(plot)
        .unwrap();
    let cell = PanelBox::new(60.0, 440.0, 50.0, 310.0);
    let (result, _) = discover_geometry(
        &plot,
        PlotKind::Line,
        PlotRenderContext::new(crate::data::PlotData::new(&plot), cell, &theme()),
    )
    .unwrap();
    let legend = crate::decorations::legend::test_assigned_legend(
        "hue",
        result.legend_entries,
        (cell.width(), cell.height()),
        &theme(),
    )
    .unwrap();
    let panel = result.layout.projection.panel_box();
    let top_right = legend.inside_candidates(result.layout.projection.pixel_box())[0];
    let line = [
        Point2D::new(
            top_right.left,
            f64::midpoint(top_right.bottom, top_right.top),
        ),
        Point2D::new(
            top_right.right,
            f64::midpoint(top_right.bottom, top_right.top),
        ),
    ];
    let resolved_theme = theme();
    let mut zero_width = GeometryBackend::new(&resolved_theme).unwrap();
    zero_width.begin(plot.width(), plot.height()).unwrap();
    zero_width
        .draw_line(&line, &LineStyle::solid((0, 0, 0), 0.0))
        .unwrap();
    zero_width.end().unwrap();
    let mut painted = GeometryBackend::new(&resolved_theme).unwrap();
    painted.begin(plot.width(), plot.height()).unwrap();
    painted
        .draw_line(&line, &LineStyle::solid((0, 0, 0), 4.0))
        .unwrap();
    painted.end().unwrap();

    assert_eq!(
        initial_auto_candidate(
            &legend,
            panel,
            zero_width.bounds(),
            LegendDefault::LowestOverlap,
        ),
        0
    );
    assert_ne!(
        initial_auto_candidate(
            &legend,
            panel,
            painted.bounds(),
            LegendDefault::LowestOverlap,
        ),
        0
    );
}

#[test]
fn outside_legend_reservation_uses_measured_label_width_and_stays_in_cell() {
    let short_data = line_data(["a", "a", "b", "b"]);
    let long_data = line_data([
        "a very long treatment label",
        "a very long treatment label",
        "another long control label",
        "another long control label",
    ]);
    let cell = PanelBox::new(40.0, 620.0, 50.0, 390.0);
    let short = line_plot(&short_data).legend(LegendPolicy::Visible(LegendPosition::OutsideRight));
    let long = line_plot(&long_data).legend(LegendPolicy::Visible(LegendPosition::OutsideRight));

    let short = plan_panel(&short, PlotKind::Line, cell, LegendRole::Panel, &theme()).unwrap();
    let long = plan_panel(&long, PlotKind::Line, cell, LegendRole::Panel, &theme()).unwrap();
    let long_legend = long.legend.unwrap().bounds;

    assert!(long.panel_box.x1 < short.panel_box.x1);
    assert!(long_legend.left > long.panel_box.x1);
    assert!(long_legend.right <= cell.x1);
}

#[test]
fn outside_legend_remeasures_and_reserves_newly_wrapped_axis_title() {
    let data = line_data([
        "first deliberately wide legend entry",
        "first deliberately wide legend entry",
        "second deliberately wide legend entry",
        "second deliberately wide legend entry",
    ]);
    let cell = PanelBox::new(40.0, 700.0, 50.0, 390.0);
    let initial = line_plot(&data).legend(LegendPolicy::Visible(LegendPosition::OutsideRight));
    let initial = plan_panel(&initial, PlotKind::Line, cell, LegendRole::Panel, &theme()).unwrap();
    let resolved = theme();
    let label = (1..=200)
        .map(|length| vec!["x"; length].join(" "))
        .find(|label| {
            crate::layout::TextBlock::from_resolved(
                resolved
                    .plan_text(
                        label,
                        &resolved.typography.axis_title,
                        cell.width(),
                        crate::layout::PhysicalTextAlignment::Center,
                        "x-axis title",
                        false,
                    )
                    .unwrap(),
            )
            .lines()
            .len()
                == 1
                && crate::layout::TextBlock::from_resolved(
                    resolved
                        .plan_text(
                            label,
                            &resolved.typography.axis_title,
                            initial.panel_box.width(),
                            crate::layout::PhysicalTextAlignment::Center,
                            "x-axis title",
                            false,
                        )
                        .unwrap(),
                )
                .lines()
                .len()
                    > 1
        })
        .expect("legend reservation should expose an axis-title rewrap threshold");
    let plot = line_plot(&data)
        .x_label(label)
        .legend(LegendPolicy::Visible(LegendPosition::OutsideRight));

    let plan = plan_panel(&plot, PlotKind::Line, cell, LegendRole::Panel, &theme()).unwrap();

    assert!((plan.panel_box.x1 - initial.panel_box.x1).abs() <= f64::EPSILON);
    assert!(plan.panel_box.y0 > initial.panel_box.y0);
}

#[test]
fn standalone_plan_reserves_wrapped_explicit_x_tick_height() {
    let plot = Plot::scatter(
        NumericValues::new("x", [0.0, 1.0, 2.0]),
        NumericValues::new("y", [1.0, 2.0, 3.0]),
    )
    .x_limits(0.0, 2.0)
    .x_ticks(vec![
        crate::layout::AxisTick::new(0.0, "one two"),
        crate::layout::AxisTick::new(1.0, "red blue"),
        crate::layout::AxisTick::new(2.0, "six ten"),
    ]);
    let cell = PanelBox::new(16.0, 290.0, 8.0, 220.0);

    let plan = plan_panel(&plot, PlotKind::Scatter, cell, LegendRole::Panel, &theme()).unwrap();

    assert!(plan.panel_box.y0 > cell.y0);
}

#[test]
fn categorical_gutter_uses_root_feedback_and_real_residual_fit_not_a_percentage_cap() {
    let source = Plot::horizontal_bar(
        crate::CategoryValues::new("category", ["W".repeat(20)]).unwrap(),
        crate::BarInput::estimates(NumericValues::new("value", [1.0])),
    )
    .size(800, 480)
    .erase();
    let theme = theme();
    let cell = PanelBox::new(80.0, 680.0, 80.0, 380.0);
    let plan = plan_panel(
        &source,
        PlotKind::HorizontalBar,
        cell,
        LegendRole::Panel,
        &theme,
    )
    .unwrap();
    assert!(plan.panel_box.x0 > cell.x0);
    assert!(plan.result.axes.as_ref().unwrap().requirements().left > cell.width() * 0.4);
    assert_eq!(plan.result.layout.y_axis.ticks[0].label, "W".repeat(20));
    let narrow = PanelBox::new(80.0, 250.0, 80.0, 380.0);
    assert!(plan_panel(
        &source,
        PlotKind::HorizontalBar,
        narrow,
        LegendRole::Panel,
        &theme
    )
    .is_err());
}

#[test]
fn hidden_outside_and_shared_suppression_leave_automatic_domains_unchanged() {
    let data = line_data(["upper", "upper", "lower", "lower"]);
    let cell = PanelBox::new(60.0, 500.0, 50.0, 350.0);
    let auto = line_plot(&data);
    let outside = line_plot(&data).legend(LegendPolicy::Visible(LegendPosition::OutsideRight));
    let hidden = line_plot(&data).legend(LegendPolicy::Hidden);

    let auto = plan_panel(&auto, PlotKind::Line, cell, LegendRole::Panel, &theme()).unwrap();
    let outside = plan_panel(&outside, PlotKind::Line, cell, LegendRole::Panel, &theme()).unwrap();
    let hidden = plan_panel(&hidden, PlotKind::Line, cell, LegendRole::Panel, &theme()).unwrap();
    let shared = plan_panel(
        &line_plot(&data),
        PlotKind::Line,
        cell,
        LegendRole::Suppressed,
        &theme(),
    )
    .unwrap();

    assert!(auto.planned_x_limits().is_none() && auto.planned_y_limits().is_none());
    assert_eq!(
        auto.result.layout.projection.x_range(),
        hidden.result.layout.projection.x_range()
    );
    assert_eq!(
        auto.result.layout.projection.y_range(),
        hidden.result.layout.projection.y_range()
    );
    assert_eq!(
        outside.result.layout.projection.x_range(),
        hidden.result.layout.projection.x_range()
    );
    assert_eq!(
        outside.result.layout.projection.y_range(),
        hidden.result.layout.projection.y_range()
    );
    assert_eq!(
        shared.result.layout.projection.x_range(),
        hidden.result.layout.projection.x_range()
    );
    assert_eq!(
        shared.result.layout.projection.y_range(),
        hidden.result.layout.projection.y_range()
    );
}

#[test]
fn scatter_automatic_guide_is_outside_without_domain_expansion() {
    let visible = Plot::scatter(
        NumericValues::new("x", [0.0, 0.5, 9.5, 10.0]),
        NumericValues::new("y", [0.0, 0.5, 9.5, 10.0]),
    )
    .hue(CategoryValues::new("hue", ["a", "a", "b", "b"]).unwrap());
    let hidden = visible.clone().legend(LegendPolicy::Hidden);
    let cell = PanelBox::new(60.0, 500.0, 50.0, 350.0);

    let visible = plan_panel(
        &visible,
        PlotKind::Scatter,
        cell,
        LegendRole::Panel,
        &theme(),
    )
    .unwrap();
    let hidden = plan_panel(
        &hidden,
        PlotKind::Scatter,
        cell,
        LegendRole::Panel,
        &theme(),
    )
    .unwrap();

    let legend = visible.legend.as_ref().unwrap();
    assert!(visible.planned_x_limits().is_none());
    assert!(visible.planned_y_limits().is_none());
    assert!(legend.bounds.left > visible.panel_box.x1);
    assert_eq!(
        visible.result.layout.projection.y_range(),
        hidden.result.layout.projection.y_range()
    );
}

#[test]
fn scatter_painted_edges_expand_both_automatic_axes_in_one_root_feedback_driver() {
    let source = Plot::scatter(
        NumericValues::new("x", [0.0, 10.0]),
        NumericValues::new("y", [0.0, 10.0]),
    )
    .size(640, 480)
    .legend(LegendPolicy::Hidden)
    .erase();
    let mut theme = theme();
    theme.marks.point_sizes.scatter = 18;
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let cell = PanelBox::new(70.0, 270.0, 70.0, 270.0);
    let result = plan_panel(&chart, PlotKind::Scatter, cell, LegendRole::Panel, &theme).unwrap();
    assert!(result.planned_x_limits().is_some());
    assert!(result.planned_y_limits().is_some());
    let crate::layout::interior::MarkPayload::Scatter(payload) =
        &result.result.marks.as_ref().unwrap().payload
    else {
        panic!("selected Scatter")
    };
    assert!(payload.domain_expansions.iter().all(Option::is_none));
    let ink = crate::render::geometry::point_ink_bounds(
        &theme,
        &chart.point_style(&theme, (0, 0, 0), 18),
    )
    .unwrap()
    .unwrap();
    let panel = result.result.layout.projection.panel_box();
    for point in &payload.groups[0].0 {
        assert!(
            point.x + ink.left >= panel.x0 && point.x + ink.right <= panel.x1,
            "point={point:?} ink={ink:?} panel={panel:?} domains={:?}/{:?}",
            result.planned_x_limits(),
            result.planned_y_limits()
        );
        assert!(point.y + ink.bottom >= panel.y0 && point.y + ink.top <= panel.y1);
    }
}

#[test]
fn line_marker_ink_uses_the_same_root_domain_feedback_without_explicit_clipping() {
    let source = Plot::line(
        NumericValues::new("x", [0.0, 10.0]),
        NumericValues::new("y", [0.0, 10.0]),
    )
    .markers(true)
    .size(640, 480)
    .legend(LegendPolicy::Hidden)
    .erase();
    let mut theme = theme();
    theme.marks.point_sizes.path = 18;
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let cell = PanelBox::new(70.0, 270.0, 70.0, 270.0);
    let result = plan_panel(&chart, PlotKind::Line, cell, LegendRole::Panel, &theme).unwrap();
    assert!(result.planned_x_limits().is_some());
    assert!(result.planned_y_limits().is_some());
    let crate::layout::interior::MarkPayload::Line(payload) =
        &result.result.marks.as_ref().unwrap().payload
    else {
        panic!("selected Line")
    };
    assert!(payload.domain_expansions.iter().all(Option::is_none));
    let ink = crate::render::geometry::point_ink_bounds(
        &theme,
        &chart.point_style(&theme, (0, 0, 0), 18),
    )
    .unwrap()
    .unwrap();
    let panel = result.result.layout.projection.panel_box();
    for value in [0.0, 10.0] {
        let (x, y) = result.result.layout.projection.project(value, value);
        assert!(x + ink.left >= panel.x0 && x + ink.right <= panel.x1);
        assert!(y + ink.bottom >= panel.y0 && y + ink.top <= panel.y1);
    }
}

#[test]
fn qq_observation_ink_expands_two_automatic_domains_without_moving_source_quantiles() {
    let source = Plot::qq(NumericValues::new("value", [0.0, 1.0, 2.0, 3.0]))
        .size(640, 480)
        .erase();
    let mut theme = theme();
    theme.marks.point_sizes.qq = 60;
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let cell = PanelBox::new(70.0, 420.0, 70.0, 420.0);
    let result = plan_panel(&chart, PlotKind::Qq, cell, LegendRole::Panel, &theme).unwrap();
    assert!(
        result.planned_x_limits().is_some(),
        "Qq domains {:?}/{:?}",
        result.planned_x_limits(),
        result.planned_y_limits()
    );
    assert!(result.planned_y_limits().is_some());
    let crate::layout::interior::MarkPayload::Qq(payload) =
        &result.result.marks.as_ref().unwrap().payload
    else {
        panic!("selected Qq")
    };
    assert!(payload.domain_expansions.iter().all(Option::is_none));
    let ink = crate::render::geometry::point_ink_bounds(
        &theme,
        &chart.point_style(&theme, (0, 0, 0), 60),
    )
    .unwrap()
    .unwrap();
    let panel = result.result.layout.projection.panel_box();
    assert_eq!(payload.points.len(), 4);
    for point in &payload.points {
        assert!(point.x + ink.left >= panel.x0 && point.x + ink.right <= panel.x1);
        assert!(point.y + ink.bottom >= panel.y0 && point.y + ink.top <= panel.y1);
    }

    let source = Plot::qq(NumericValues::new("value", [0.0, 1.0, 2.0, 3.0]))
        .x_limits(-2.0, 2.0)
        .size(640, 480)
        .erase();
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let exact = plan_panel(&chart, PlotKind::Qq, cell, LegendRole::Panel, &theme).unwrap();
    assert!(exact.planned_x_limits().is_none());
    assert!(exact.planned_y_limits().is_some());
}

#[test]
fn right_edge_legend_expands_generic_continuous_x_axis() {
    let hue = CategoryValues::new("hue", ["a", "a", "b", "b"]).unwrap();
    let plot = Plot::line(
        NumericValues::new("x", [8.0, 10.0, 8.0, 10.0]),
        NumericValues::new("y", [5.0; 4]),
    )
    .group(hue.clone())
    .hue(hue)
    .legend(LegendPolicy::Visible(LegendPosition::InsideRight));
    let cell = PanelBox::new(60.0, 500.0, 50.0, 350.0);

    let plan = plan_panel(&plot, PlotKind::Line, cell, LegendRole::Panel, &theme()).unwrap();
    let legend = plan.legend.as_ref().unwrap().bounds;
    let rightmost_point = plan.result.layout.projection.project(10.0, 5.0).0;

    assert!(plan.planned_x_limits().is_some());
    assert!(
        (2..=8).contains(&crate::layout::kernel::ROOT_FEEDBACK_ROUNDS.with(std::cell::Cell::get))
    );
    assert!(rightmost_point < legend.left);
}

#[test]
fn auto_legend_leaves_dense_categorical_domains_unchanged_in_normal_flow() {
    let data = dense_horizontal_data();
    let plot = dense_bar_plot(&data, true);
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let plot = crate::plots::shared_x::prepare_plot(&plot, &resources)
        .unwrap()
        .attach(plot)
        .unwrap();
    let cell = PanelBox::new(90.0, 500.0, 50.0, 310.0);

    let (initial, initial_geometry) = discover_geometry(
        &plot,
        PlotKind::HorizontalBar,
        PlotRenderContext::new(crate::data::PlotData::new(&plot), cell, &theme()),
    )
    .unwrap();
    let initial_legend = crate::decorations::legend::test_assigned_legend(
        "hue",
        initial.legend_entries,
        (cell.width(), cell.height()),
        &theme(),
    )
    .unwrap();
    let margin = initial_legend.metrics.font_size.max(6.0);
    assert!(
        initial_legend
            .inside_candidates(initial.layout.projection.pixel_box())
            .iter()
            .all(|bounds| {
                overlap_score_within_panel(
                    *bounds,
                    initial_geometry.bounds(),
                    margin,
                    initial.layout.projection.panel_box(),
                ) > 0.0
            }),
        "test data must collide at every initial corner"
    );

    let plan = plan_panel(
        &plot,
        PlotKind::HorizontalBar,
        cell,
        LegendRole::Panel,
        &theme(),
    )
    .unwrap();
    let legend = plan.legend.as_ref().unwrap();
    let plot_with_planned_limits = plan.plot_with_planned_limits(&plot, plot.data()).unwrap();
    let (final_result, final_geometry) = discover_geometry(
        &plot_with_planned_limits,
        PlotKind::HorizontalBar,
        PlotRenderContext::new(
            crate::data::PlotData::new(&plot_with_planned_limits),
            plan.panel_box,
            &theme(),
        ),
    )
    .unwrap();

    assert!(plan.planned_x_limits().is_none());
    assert!(plan.planned_y_limits().is_none());
    assert_eq!(
        final_result.layout.projection.x_range().0.to_bits(),
        0.0_f64.to_bits()
    );
    assert!(legend.bounds.left > plan.panel_box.x1);
    assert!(!legend_fits_panel(
        legend.bounds,
        final_result.layout.projection.panel_box()
    ));
    assert!(
        overlap_score_within_panel(
            legend.bounds,
            final_geometry.bounds(),
            margin,
            final_result.layout.projection.panel_box(),
        ) <= f64::EPSILON
    );
}

#[test]
fn negative_horizontal_bars_keep_zero_maximum_with_deterministic_outside_guide() {
    let data = signed_dense_bar_data(-1.0);
    let plot = dense_bar_plot(&data, true);
    let cell = PanelBox::new(90.0, 500.0, 50.0, 310.0);

    let first = plan_panel(
        &plot,
        PlotKind::HorizontalBar,
        cell,
        LegendRole::Panel,
        &theme(),
    )
    .unwrap();
    let second = plan_panel(
        &plot,
        PlotKind::HorizontalBar,
        cell,
        LegendRole::Panel,
        &theme(),
    )
    .unwrap();
    let legend = first.legend.as_ref().unwrap();

    assert_eq!(
        first.result.layout.projection.x_range().1.to_bits(),
        0.0_f64.to_bits()
    );
    assert!(legend.bounds.left > first.panel_box.x1);
    assert_eq!(first.planned_x_limits(), second.planned_x_limits());
    assert_eq!(legend.bounds, second.legend.unwrap().bounds);
}

#[test]
fn positive_vertical_bars_keep_zero_minimum_with_outside_guide() {
    let data = signed_dense_bar_data(1.0);
    let plot = dense_bar_plot(&data, false);
    let cell = PanelBox::new(90.0, 500.0, 50.0, 310.0);

    let plan = plan_panel(&plot, PlotKind::Bar, cell, LegendRole::Panel, &theme()).unwrap();
    let legend = plan.legend.as_ref().unwrap();

    assert_eq!(
        plan.result.layout.projection.y_range().0.to_bits(),
        0.0_f64.to_bits()
    );
    assert!(legend.bounds.left > plan.panel_box.x1);
    assert!(plan.planned_y_limits().is_none());
}

#[test]
fn negative_vertical_bars_keep_zero_maximum_with_outside_guide() {
    let data = signed_dense_bar_data(-1.0);
    let plot = dense_bar_plot(&data, false);
    let cell = PanelBox::new(90.0, 500.0, 50.0, 310.0);

    let plan = plan_panel(&plot, PlotKind::Bar, cell, LegendRole::Panel, &theme()).unwrap();
    let legend = plan.legend.as_ref().unwrap();

    assert_eq!(
        plan.result.layout.projection.y_range().1.to_bits(),
        0.0_f64.to_bits()
    );
    assert!(legend.bounds.left > plan.panel_box.x1);
    assert!(plan.planned_y_limits().is_none());
}

#[test]
fn automatic_clearance_does_not_expand_categorical_domains() {
    let data = dense_horizontal_data();
    let plot = dense_bar_plot(&data, true);
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let plot = crate::plots::shared_x::prepare_plot(&plot, &resources)
        .unwrap()
        .attach(plot)
        .unwrap();
    let cell = PanelBox::new(90.0, 500.0, 50.0, 310.0);
    let (mut result, geometry) = discover_geometry(
        &plot,
        PlotKind::HorizontalBar,
        PlotRenderContext::new(crate::data::PlotData::new(&plot), cell, &theme()),
    )
    .unwrap();
    result.x_domain = crate::spec::AxisDomain::Categorical;
    result.y_domain = crate::spec::AxisDomain::Categorical;
    let legend = crate::decorations::legend::test_assigned_legend(
        "hue",
        result.legend_entries.clone(),
        (cell.width(), cell.height()),
        &theme(),
    )
    .unwrap();

    assert!(automatic_clearance(
        PlotKind::HorizontalBar,
        &plot,
        &result,
        &legend,
        geometry.bounds(),
        PlotKind::HorizontalBar.legend_default(false),
        0,
        &theme(),
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap()
    .is_none());
}

#[test]
fn one_assigned_frame_reserves_guide_and_gap_on_each_selected_edge() {
    use crate::layout::fit::FitOutcome;
    let cell = PanelBox::new(10.0, 510.0, 20.0, 320.0);
    let minimum = (100.0, 100.0);
    let guide = (80.25, 60.25);
    let gap = 17.6;
    for (edge, expected_width, expected_height) in [
        (LegendPosition::OutsideRight, guide.0, 0.0),
        (LegendPosition::OutsideLeft, guide.0, 0.0),
        (LegendPosition::OutsideTop, 0.0, guide.1),
        (LegendPosition::OutsideBottom, 0.0, guide.1),
    ] {
        let frame = AssignedGuideFrame::new(edge, minimum, gap);
        let available = frame.available(cell, [0.0; 4]);
        assert!(guide.0 <= available.0 && guide.1 <= available.1);
        let FitOutcome::Measured(body) = frame
            .assign_fit(cell, Some(guide), [0.0; 4], &FitScope::root())
            .unwrap()
        else {
            panic!("measured guide must leave the containing body fit");
        };
        let reserved_width = cell.width() - body.width();
        let reserved_height = cell.height() - body.height();
        let expected_width = if expected_width > 0.0 {
            expected_width + gap
        } else {
            0.0
        };
        let expected_height = if expected_height > 0.0 {
            expected_height + gap
        } else {
            0.0
        };
        assert!(reserved_width >= expected_width && reserved_width < expected_width + 1.0 / 1024.0);
        assert!(
            reserved_height >= expected_height && reserved_height < expected_height + 1.0 / 1024.0
        );
        assert!(body.width() >= minimum.0 && body.height() >= minimum.1);
        let placed = frame.place(body, guide);
        let clearance = match edge {
            LegendPosition::OutsideRight => placed.x0 - body.x1,
            LegendPosition::OutsideLeft => body.x0 - placed.x1,
            LegendPosition::OutsideTop => placed.y0 - body.y1,
            LegendPosition::OutsideBottom => body.y0 - placed.y1,
            _ => unreachable!("the fixture uses the four outside edges"),
        };
        assert!((clearance - gap).abs() < 1.0e-9);
        assert!(placed.x0 >= cell.x0 && placed.x1 <= cell.x1);
        assert!(placed.y0 >= cell.y0 && placed.y1 <= cell.y1);
    }
}
