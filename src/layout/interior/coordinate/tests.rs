use super::*;
use crate::font::canonical::OperationContext;
use crate::orchestrator::measurements::{self, Counter};
use crate::{CategoryValues, NumericValues};

fn sources() -> [Plot; 8] {
    let x = NumericValues::new("x", [0.0, 1.0, 2.0]);
    let y = NumericValues::new("y", [2.0, 1.0, 3.0]);
    let group = CategoryValues::new("group", ["a", "a", "a"]).unwrap();
    let [histogram, density, qq, regression] = crate::layout::interior::distributions::sources();
    [
        Plot::scatter(x.clone(), y.clone()).erase(),
        Plot::line(x.clone(), y.clone()).erase(),
        Plot::trajectory(x.clone(), y.clone(), group.clone()).erase(),
        Plot::line_path(x, y, group).erase(),
        histogram,
        density,
        qq,
        regression,
    ]
}

fn categorical_source() -> Plot {
    Plot::trajectory(
        CategoryValues::new("condition", ["before", "during", "after"]).unwrap(),
        NumericValues::new("y", [2.0, 1.0, 3.0]),
        CategoryValues::new("group", ["a", "a", "a"]).unwrap(),
    )
    .erase()
}

fn ordinary_sources() -> impl Iterator<Item = Plot> {
    sources()
        .into_iter()
        .chain(super::super::estimates::sources())
        .chain(super::super::bars::sources())
        .chain(super::super::categorical_distributions::sources())
        .chain([crate::plots::interaction::extraction::source().erase()])
}

fn all_sources() -> impl Iterator<Item = Plot> {
    ordinary_sources().chain([crate::plots::categorical_distribution::extraction::supplied()])
}

#[test]
fn ordinary_prepared_inputs_attach_repeatedly_without_font_discovery() {
    for source in ordinary_sources().chain([categorical_source()]) {
        {
            let resources = OperationContext::new();

            crate::data::SOURCE_COLUMN_READS.with(|count| count.set(0));
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
            let result = crate::plots::shared_x::prepare_plot(&source, &resources, None);
            {
                let inputs = result.unwrap();
                inputs.attach(source.clone()).unwrap();
                inputs.attach(source.clone()).unwrap();
            }
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| assert_eq!(count.get(), 0));
        }
    }
    let source = Plot::scatter(
        NumericValues::nullable("x", [Some(1.0), None, Some(3.0)]),
        NumericValues::new("y", [1.0, 2.0, 3.0]),
    )
    .erase();
    let resources = OperationContext::new();
    crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
}

#[test]
fn prepared_intrinsic_hits_reuse_the_summary_without_reanalysis() {
    let source = categorical_source();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
    assert!(inputs.axes[0].1.intrinsics.get().is_none());
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
    let theme = crate::Theme::publication().resolve().unwrap();
    let plot = inputs.attach_with_guides_for_test(source, &theme).unwrap();
    inputs.prepare_intrinsics(&plot, &theme).unwrap();
    let summary = Intrinsics::prepared(&plot, &theme).unwrap();
    measurements::start();
    for width in [400.0, 580.0] {
        let changed = plot
            .clone()
            .with_canvas_size(800, 600)
            .with_planned_y_limits((-1.0, 5.0));
        assert!(std::ptr::eq(
            summary,
            Intrinsics::prepared(&changed, &theme).unwrap()
        ));
        changed
            .measure_selected_interior(PlotRenderContext::new(
                changed.data(),
                PanelBox::new(30.0, 30.0 + width, 50.0, 440.0),
                &theme,
            ))
            .unwrap();
    }
    let work = measurements::finish();
    assert_eq!(work.counters[Counter::StandardPanelIntrinsic as usize], 0);
    assert_eq!(
        work.counters[Counter::StandardCoordinateMeasure as usize],
        2
    );

    assert!(std::ptr::eq(
        summary,
        Intrinsics::prepared(&plot, &theme).unwrap()
    ));
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one native root harness pins both terminal policies and the selected eighth-round payload"
)]
fn native_root_producers_obey_eighth_round_and_cycle_limits_under_parent_feedback() {
    use crate::decorations::plan::{plan_panel, LegendRole};
    use crate::layout::kernel::{FeedbackKind, FrameRequirements};
    use crate::layout::{AddressRole, ComponentAddress};
    // Controlled deferred parent consequences exercise the production native
    // axis/title/body path, not a claim that this is a new packing policy.
    for source in [sources()[0].clone(), categorical_source()] {
        let source = source
            .title("Root title\nSecond line")
            .caption("Independent caption")
            .with_suppressed_legend();
        for policy in 0..3 {
            let theme = crate::Theme::publication().resolve().unwrap();
            let parent = ComponentAddress::root()
                .child(AddressRole::RootBody, 0)
                .unwrap();
            let owner = parent.child(AddressRole::PlotBody, 0).unwrap();
            let mut assignments = Vec::new();
            let mut captures = 0;
            let root = crate::layout::plan_exact_root(
                640,
                480,
                |stage| {
                    let inputs =
                        crate::plots::shared_x::prepare_plot(&source, &stage.operation(), None)?;
                    inputs.prepare_discrete_guides(
                        &source,
                        &theme,
                        &stage.operation(),
                        &owner,
                        false,
                    )?;
                    Ok(inputs)
                },
                |inputs, assigned, round| {
                    let (width, height) = assigned.dimensions()?;
                    let plot = inputs
                        .attach(source.clone())?
                        .with_canvas_size(width, height);
                    let canvas = canvas_layout(&plot, &theme).map_err(PlanError::into_error)?;
                    round.observe_canvas(&canvas)?;
                    let consumed = round.insets(&parent, FeedbackKind::AxisGutter);
                    let cell = fit::measured(
                        FrameRequirements {
                            minimum_body: plot.intrinsic_body_minimum(&theme)?,
                            natural_insets: [0.0; 4],
                        }
                        .assign_fit(
                            canvas.panel_box,
                            consumed,
                            &fit::FitScope::root(),
                        ),
                    )
                    .map_err(PlanError::into_error)?;
                    let plan = fit::measured(plan_panel(
                        &plot,
                        plot.kind(),
                        cell,
                        LegendRole::Suppressed,
                        &theme,
                        round,
                        &owner,
                    ))
                    .map_err(PlanError::into_error)?;
                    assignments.push(plan.panel_box);
                    assert_eq!(plan.result.layout.projection.panel_box(), plan.panel_box);
                    let next = match policy {
                        0 => (consumed[1] + 1.0).min(7.0),
                        1 => consumed[1] + 1.0,
                        _ => 1.0 - consumed[1],
                    };
                    let request =
                        super::super::InteriorReserveRequest::AxisGutter([0.0, next, 0.0, 0.0])
                            .submit(round, &parent, &parent)?;
                    if request.is_some() {
                        round.miss(
                            &parent,
                            FitKind::TrackOrResidual,
                            &[request],
                            "deferred containing frame requires its next assigned reserve",
                        )?;
                    }
                    Ok(crate::layout::MeasuredLayout::new(assigned, (plan, canvas)))
                },
                |_, assigned, (plan, canvas), stage| {
                    captures += 1;
                    assert_eq!(policy, 0, "unstable native roots cannot capture");
                    let (width, height) = assigned.dimensions()?;
                    crate::orchestrator::capture_text_replay_plan(
                        width,
                        height,
                        "native eighth-round closure",
                        &theme,
                        stage,
                        |backend| {
                            canvas.root_text.draw_retained(backend)?;
                            let axes = plan.result.axes.as_ref().unwrap();
                            axes.draw_grid(backend)?;
                            plan.result.marks.as_ref().unwrap().draw(backend)?;
                            plan.result.annotations.as_ref().unwrap().draw(backend)?;
                            axes.draw(backend)
                        },
                    )
                },
            );
            if policy == 0 {
                assert_eq!(assignments.len(), 8);
                assert_eq!(captures, 1);
                assert!(assignments.windows(2).all(|pair| pair[1].x1 < pair[0].x1));
                let root = root.unwrap();
                crate::orchestrator::validate_text_replay_plan(
                    640,
                    480,
                    root.plan(),
                    "native eighth-round Null",
                )
                .unwrap();
                let mut svg = root.plan().take_svg_backend().unwrap();
                root.plan()
                    .replay(640, 480, &mut svg, "native eighth-round SVG")
                    .unwrap();
                assert!(!svg.into_bytes().unwrap().is_empty());
            } else {
                assert_eq!(captures, 0);
                let error = root.err().unwrap().to_string();
                if policy == 1 {
                    assert_eq!(assignments.len(), 8);
                    assert!(error.contains("eight rounds"), "{error}");
                } else {
                    assert!(assignments.len() < 8);
                    assert!(error.contains("cycle"), "{error}");
                }
            }
        }
    }
}

#[test]
fn native_multiline_axis_gutters_cannot_consume_root_caption_or_title_reserves() {
    use crate::decorations::plan::{plan_panel_for_test, LegendRole};
    use crate::render::backend::Backend;
    use crate::render::geometry::GeometryBackend;
    for source in all_sources().chain([categorical_source()]) {
        let source = source
            .x_label("First axis line\nSecond axis line\nThird axis line")
            .title("Root title\nSecond title line")
            .caption("First caption line\nSecond caption line\nThird caption line")
            .with_suppressed_legend()
            .size(401, 401);
        let theme = crate::Theme::publication().resolve().unwrap();
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        let plot = inputs.attach_with_guides_for_test(source, &theme).unwrap();
        let canvas = canvas_layout(&plot, &theme).unwrap();
        let plan = plan_panel_for_test(
            &plot,
            plot.kind(),
            canvas.panel_box,
            LegendRole::Suppressed,
            &theme,
        )
        .unwrap();
        let required = plan.result.axes.as_ref().unwrap().requirements();
        assert_eq!(
            (plan.panel_box.y0 - canvas.panel_box.y0).to_bits(),
            (required.bottom + 4.0).ceil().to_bits()
        );
        let text = &canvas.root_text;
        let mut roots = GeometryBackend::new(&theme).unwrap();
        let (width, height) = plot.canvas_dimensions().unwrap();
        roots.begin(width, height).unwrap();
        text.draw_retained(&mut roots).unwrap();
        roots.end().unwrap();
        let roots = roots.into_snapshot();
        let caption = roots.bounds().last().unwrap();
        let title = roots.bounds().first().unwrap();
        let text_sizes = crate::decorations::title_block::plan::RootTitleBlockPlan::appearances(
            &theme,
            plot.prose(),
        )
        .map(|role| role.theme.get_font_size());
        let axes = plan
            .result
            .axes
            .as_ref()
            .unwrap()
            .geometry_snapshot()
            .unwrap();
        for ink in axes.bounds() {
            assert!(
                ink.bottom
                    >= caption.top
                        + crate::theme::composition::hard(
                            &theme
                                .root_title_lengths(text_sizes)
                                .unwrap()
                                .caption_to_content_clearance
                        )
                        .unwrap()
            );
            assert!(
                ink.top
                    <= title.bottom
                        - crate::theme::composition::hard(
                            &theme
                                .root_title_lengths(text_sizes)
                                .unwrap()
                                .title_block_to_content_clearance
                        )
                        .unwrap()
            );
        }
        let mut final_output = crate::render::svg_backend::SvgBackend::in_memory(401, 401);
        plot.render_with_backend(&mut final_output).unwrap();
        for ink in axes
            .bounds()
            .iter()
            .chain(roots.bounds())
            .chain(plan.result.marks.as_ref().unwrap().geometry.bounds())
        {
            assert!(
                ink.left >= 0.0 && ink.right <= 401.0 && ink.bottom >= 0.0 && ink.top <= 401.0,
                "{ink:?}"
            );
        }
    }
}

#[test]
fn native_narrow_root_selects_actual_ticks_and_automatic_painted_domains() {
    use crate::decorations::plan::{plan_panel_for_test, LegendRole};
    for source in sources()
        .into_iter()
        .chain(
            super::super::categorical_distributions::sources().map(|source| {
                source.labels(
                    crate::Labels::raw()
                        .alias("after_visit", "a")
                        .alias("before_visit", "b"),
                )
            }),
        )
        .chain([crate::plots::interaction::extraction::source().erase()])
    {
        let source = source
            .with_planned_y_limits((0.001, 1.0))
            .x_label("")
            .y_label("")
            .with_suppressed_legend()
            .size(175, 300);
        let theme = crate::Theme::publication().resolve().unwrap();
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        let plot = inputs.attach_with_guides_for_test(source, &theme).unwrap();
        let canvas = canvas_layout(&plot, &theme).unwrap();
        let endpoint = fit::intrinsic_widths(
            &theme,
            "1.00e-3",
            &theme.typography.axes.tick_label,
            "endpoint estimate",
        )
        .unwrap()
        .1;
        assert!(175.0 - 40.0 - endpoint - 6.0 - 5.0 > 0.0);
        let plan = plan_panel_for_test(
            &plot,
            plot.kind(),
            canvas.panel_box,
            LegendRole::Suppressed,
            &theme,
        )
        .unwrap();
        assert!(plan.panel_box.width() >= plot.intrinsic_body_minimum(&theme).unwrap().0);
        let final_range = plan.result.layout.projection.y_range();
        assert!(final_range.0 <= 0.001 && final_range.1 >= 1.0);
        assert!(!plan.result.layout.y_axis.ticks.is_empty());
        let planned_plot = plan.plot_with_planned_limits(&plot);
        let selected = planned_plot
            .measure_selected_interior(PlotRenderContext::new(
                planned_plot.data(),
                plan.panel_box,
                &theme,
            ))
            .unwrap();
        assert_eq!(
            selected.layout.y_axis.ticks,
            plan.result.layout.y_axis.ticks
        );
        assert_eq!(
            selected.layout.y_axis.source,
            plan.result.layout.y_axis.source
        );
        let mut svg = crate::render::svg_backend::SvgBackend::in_memory(175, 300);
        plot.render_with_backend(&mut svg).unwrap();
        assert!(String::from_utf8(svg.into_bytes().unwrap())
            .unwrap()
            .contains("viewBox=\"0 0 175 300\""));
    }
}

#[test]
fn native_absolute_gutter_feedback_contracts_independently_of_multiline_caption() {
    use crate::decorations::plan::{plan_panel, LegendRole};
    use crate::layout::kernel::FeedbackKind;
    use crate::layout::{AddressRole, ComponentAddress};
    for source in all_sources().chain([categorical_source()]) {
        let source = source
            .caption("First caption line\nSecond caption line")
            .with_suppressed_legend()
            .size(640, 480);
        let theme = crate::Theme::publication().resolve().unwrap();
        let owner = ComponentAddress::root()
            .child(AddressRole::PlotBody, 0)
            .unwrap();
        let mut outputs = Vec::new();
        for seeded in [false, true] {
            let mut first = true;
            let mut contracted = false;
            let root = crate::layout::plan_exact_root(
                source.sizing_request().requested_dimensions().0,
                source.sizing_request().requested_dimensions().1,
                |stage| {
                    let inputs =
                        crate::plots::shared_x::prepare_plot(&source, &stage.operation(), None)?;
                    inputs.prepare_discrete_guides(
                        &source,
                        &theme,
                        &stage.operation(),
                        &owner,
                        false,
                    )?;
                    Ok(inputs)
                },
                |inputs, assigned, round| {
                    if first && seeded {
                        round.seed_insets_for_test(&owner, [180.0, 20.0, 160.0, 20.0])?;
                    }
                    first = false;
                    let plot = inputs.attach(source.clone())?;
                    let canvas = canvas_layout(&plot, &theme).map_err(PlanError::into_error)?;
                    let plan = fit::measured(plan_panel(
                        &plot,
                        plot.kind(),
                        canvas.panel_box,
                        LegendRole::Suppressed,
                        &theme,
                        round,
                        &owner,
                    ))
                    .map_err(PlanError::into_error)?;
                    let prior = round.insets(&owner, FeedbackKind::AxisGutter);
                    let next = round.next_insets(&owner, FeedbackKind::AxisGutter);
                    contracted |= next[0] < prior[0] && next[2] < prior[2];
                    Ok(crate::layout::MeasuredLayout::new(assigned, (plan, canvas)))
                },
                |_, _, plan, _| Ok(plan),
            )
            .unwrap();
            assert_eq!(contracted, seeded);
            let (plan, canvas) = root.plan();
            outputs.push((
                plan.panel_box,
                plan.result.layout.x_axis.ticks.clone(),
                plan.result.layout.y_axis.ticks.clone(),
                canvas.clone(),
            ));
        }
        assert_eq!(outputs[0], outputs[1]);
    }
}

#[test]
fn native_hard_title_spans_and_category_slots_come_from_shaped_text() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let word = "W".repeat(20);
    let category = Plot::trajectory(
        CategoryValues::new("condition", [word.as_str(), "middle", "last"]).unwrap(),
        NumericValues::new("y", [2.0, 1.0, 3.0]),
        CategoryValues::new("group", ["a", "a", "a"]).unwrap(),
    )
    .erase();
    for (source, axis) in all_sources()
        .flat_map(|source| {
            [
                (source.clone().x_label(&word), 0),
                (source.y_label(&word), 1),
            ]
        })
        .chain([(category, 0)])
    {
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        let plot = inputs.attach_with_guides_for_test(source, &theme).unwrap();
        let minimum = Intrinsics::prepared(&plot, &theme).unwrap().minimum(&plot);
        let is_category = plot.prepared_axis_inputs().unwrap().display_categories[0]
            .as_ref()
            .is_some_and(|labels| labels.iter().any(|label| label == &word));
        let hard = fit::intrinsic_widths(
            &theme,
            &word,
            if is_category {
                &theme.typography.axes.tick_label
            } else {
                &theme.typography.axes.axis_title
            },
            "hard span",
        )
        .unwrap()
        .0;
        let expected = if is_category {
            (hard + 2.0 * crate::layout::panel::LABEL_CLEARANCE) * 3.0
        } else {
            hard
        };
        assert!(expected > crate::layout::canvas::MINIMUM_PANEL_SPAN);
        assert_eq!(
            if axis == 0 { minimum.0 } else { minimum.1 }.to_bits(),
            expected.to_bits()
        );
        let mut extent = [minimum.0, minimum.1];
        extent[axis] -= 1.0 / 1024.0;
        let before = Projection::test_projection_calls();
        let error = plot
            .measure_selected_interior(PlotRenderContext::new(
                plot.data(),
                PanelBox::new(0.0, extent[0], 0.0, extent[1]),
                &theme,
            ))
            .unwrap_err();
        assert!(
            matches!(error, PlanError::Fit(failure) if failure.reason.kind == FitKind::TrackOrResidual)
        );
        assert_eq!(Projection::test_projection_calls(), before);
        let tick_span = 4.0_f64
            * theme.typography.axes.tick_label.line_pitch_pixels().max(
                fit::intrinsic_widths(
                    &theme,
                    "1.00e-3",
                    &theme.typography.axes.tick_label,
                    "native tick capacity test",
                )
                .unwrap()
                .1,
            );
        plot.measure_selected_interior(PlotRenderContext::new(
            plot.data(),
            PanelBox::new(
                0.0,
                minimum.0.max(tick_span).ceil() + 1.0,
                0.0,
                minimum.1.max(tick_span).ceil() + 1.0,
            ),
            &theme,
        ))
        .unwrap();
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "real native compositions compare rejected guides, bounded trials and gutter feedback with both cache modes"
)]
fn native_real_compositions_remeasure_gutters_and_select_guides_identically_without_cache() {
    use crate::decorations::plan::children::DISABLE_CHILD_REUSE;
    use crate::orchestrator::measurements::GuideOutcome;
    fn configured<K: crate::kind::Hue + crate::kind::Facet + crate::kind::ExactFamily>(
        plot: Plot<K>,
        faceted: bool,
    ) -> (Plot, Option<crate::Canvas>) {
        let plot = plot
            .hue(CategoryValues::new("group", vec!["M".repeat(42); 6]).unwrap())
            .x_label("First axis line\nSecond axis line\nThird axis line")
            .size(640, 640);
        if faceted {
            (
                plot.facet(crate::FacetSpec::column(
                    CategoryValues::new("facet", ["a", "a", "a", "b", "b", "b"]).unwrap(),
                ))
                .erase(),
                None,
            )
        } else {
            let canvas = crate::Canvas::grid(1, 1)
                .unwrap()
                .panel(0, 0, plot.clone())
                .unwrap()
                .size(640, 640);
            (plot.erase(), Some(canvas))
        }
    }
    for faceted in [false, true] {
        let x = NumericValues::new("x", [0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
        let y = NumericValues::new("y", [2.0, 1.0, 3.0, 2.0, 1.0, 3.0]);
        let group = CategoryValues::new("subject", ["a", "a", "a", "b", "b", "b"]).unwrap();
        let plots = vec![
            configured(Plot::scatter(x.clone(), y.clone()), faceted),
            configured(Plot::line(x.clone(), y.clone()), faceted),
            configured(Plot::density(x.clone()), faceted),
            configured(
                Plot::trajectory(x.clone(), y.clone(), group.clone()),
                faceted,
            ),
            configured(Plot::line_path(x, y.clone(), group.clone()), faceted),
            configured(
                Plot::interaction(
                    CategoryValues::new(
                        "condition",
                        ["before", "during", "after", "before", "during", "after"],
                    )
                    .unwrap(),
                    y.clone(),
                    group.clone(),
                    crate::Bounds::new(
                        NumericValues::new("lo", [1.0, 0.0, 2.0, 1.0, 0.0, 2.0]),
                        NumericValues::new("hi", [3.0, 2.0, 4.0, 3.0, 2.0, 4.0]),
                    ),
                ),
                faceted,
            ),
            configured(
                Plot::trajectory(
                    CategoryValues::new(
                        "condition",
                        ["before", "during", "after", "before", "during", "after"],
                    )
                    .unwrap(),
                    y,
                    group,
                ),
                faceted,
            ),
        ];
        for (plot, canvas) in plots {
            let mut outputs = Vec::new();
            for disabled in [false, true] {
                DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
                measurements::start();
                let mut svg = crate::render::svg_backend::SvgBackend::in_memory(640, 640);
                let result = if let Some(canvas) = &canvas {
                    canvas.render_with_backend(&mut svg)
                } else {
                    plot.render_with_backend(&mut svg)
                };
                let work = measurements::finish();
                DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
                result.unwrap_or_else(|error| {
                    panic!(
                        "{:?}, faceted={faceted}: {error}; {:?}",
                        plot.kind(),
                        work.guides
                    )
                });
                assert!(work.roots.len() >= 2);
                assert!(
                    work.gutters
                        .iter()
                        .any(|change| change.consumed.map(f64::to_bits)
                            != change.requested.map(f64::to_bits)
                            && change.owner != crate::layout::ComponentAddress::root()),
                    "{:?}",
                    work.gutters
                );
                assert!(
                    work.guides
                        .iter()
                        .any(|guide| guide.outcome == GuideOutcome::Accepted),
                    "{:?}, faceted={faceted}: {:?}",
                    plot.kind(),
                    work.guides
                );
                for guide in &work.guides {
                    assert!(
                        work.guides
                            .iter()
                            .filter(|attempt| attempt.owner == guide.owner
                                && attempt.root_round == guide.root_round)
                            .count()
                            <= 8
                    );
                }
                if disabled {
                    assert_eq!(work.counters[Counter::ChildCacheHit as usize], 0);
                }
                assert_eq!(work.counters[Counter::LegacyPanelConstructor as usize], 0);
                outputs.push(svg.into_bytes().unwrap());
            }
            assert_eq!(outputs[0], outputs[1]);
        }
    }
}

#[test]
fn every_native_route_uses_exact_assigned_projection_and_frozen_axes_without_legacy_constructors() {
    let theme = crate::Theme::publication().resolve().unwrap();
    for source in sources() {
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        assert!(inputs.route.is_coordinate());
        let plot = inputs.attach_with_guides_for_test(source, &theme).unwrap();
        for panel in [
            PanelBox::new(80.25, 550.75, 70.5, 410.25),
            PanelBox::new(30.0, 810.0, 50.0, 600.0),
        ] {
            for ranges in [[(-1.0, 3.0), (0.0, 4.0)], [(-2.0, 5.0), (-1.0, 6.0)]] {
                let plot = plot
                    .clone()
                    .with_planned_x_limits(ranges[0])
                    .with_planned_y_limits(ranges[1]);
                measurements::start();
                let selected = plot
                    .measure_selected_interior(PlotRenderContext::new(plot.data(), panel, &theme))
                    .unwrap();
                let work = measurements::finish();
                assert_eq!(
                    work.counters[Counter::StandardCoordinateMeasure as usize],
                    1
                );
                assert_eq!(work.counters[Counter::LegacyPanelConstructor as usize], 0);
                assert_eq!(selected.layout.projection.panel_box(), panel);
                assert_eq!(selected.layout.projection.x_range(), ranges[0]);
                assert_eq!(selected.layout.projection.y_range(), ranges[1]);
                assert_eq!(
                    selected.layout.projection.project(ranges[0].0, ranges[1].0),
                    (panel.x0, panel.y0)
                );
                assert_eq!(
                    selected.layout.projection.project(ranges[0].1, ranges[1].1),
                    (panel.x1, panel.y1)
                );
                for side in [AxisSide::Bottom, AxisSide::Left] {
                    assert_eq!(
                        selected.axes.as_ref().unwrap().range(side),
                        ranges[usize::from(side == AxisSide::Left)]
                    );
                }
                assert!(selected.marks.is_some());
                assert_eq!(selected.annotations.as_ref().unwrap().panel_clip, panel);
            }
        }
    }
}

#[test]
fn every_native_route_reports_intrinsic_fit_before_mark_projection_or_backend_creation() {
    let theme = crate::Theme::publication().resolve().unwrap();
    for source in sources() {
        let source = source.x_label("").y_label("");
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        let plot = inputs.attach_with_guides_for_test(source, &theme).unwrap();
        let axes = Intrinsics::prepared(&plot, &theme).unwrap().minimum(&plot);
        let minimum = plot.intrinsic_body_minimum(&theme).unwrap();
        assert!(minimum.0 >= axes.0 && minimum.1 >= axes.1);
        let scope = fit::FitScope::new(
            &crate::layout::ComponentAddress::root()
                .child(crate::layout::AddressRole::PlotBody, 0)
                .unwrap(),
            7,
        );
        let before = crate::layout::Projection::test_projection_calls();
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
        let error = plot
            .measure_selected_interior(
                PlotRenderContext::new(
                    plot.data(),
                    PanelBox::new(0.0, minimum.0 - 1.0 / 1024.0, 0.0, minimum.1),
                    &theme,
                )
                .with_fit_scope(&scope),
            )
            .unwrap_err();
        let PlanError::Fit(failure) = error else {
            panic!("typed intrinsic fit")
        };
        assert_eq!(failure.reason.kind, FitKind::TrackOrResidual);
        assert_eq!(failure.reason.address, scope.address);
        assert_eq!(failure.reason.ordinal, 7);
        assert_eq!(crate::layout::Projection::test_projection_calls(), before);
        assert_eq!(
            crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
            0
        );
    }
}

#[test]
fn exact_scatter_marks_fit_below_the_retired_generic_panel_floor() {
    let source = Plot::scatter(
        NumericValues::new("x", [0.0, 1.0]),
        NumericValues::new("y", [0.0, 1.0]),
    )
    .x_limits(0.0, 1.0)
    .y_limits(0.0, 1.0)
    .x_label("")
    .y_label("")
    .erase();
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
    let theme = crate::Theme::publication().resolve().unwrap();
    let plot = inputs.attach_with_guides_for_test(source, &theme).unwrap();
    let hard = plot.intrinsic_body_minimum(&theme).unwrap();
    assert!(hard.0 < 80.0 && hard.1 < 60.0);
    let panel = PanelBox::new(5.0, 75.0, 5.0, 63.0);
    let selected = plot
        .measure_selected_interior(PlotRenderContext::new(plot.data(), panel, &theme))
        .unwrap();
    assert_eq!(selected.layout.projection.panel_box(), panel);
    assert_eq!(selected.layout.projection.x_range(), (0.0, 1.0));
    assert_eq!(selected.layout.projection.y_range(), (0.0, 1.0));
}

#[test]
fn categorical_trajectory_retains_bottom_slots_raw_and_display_identity() {
    let source = Plot::trajectory(
        CategoryValues::new("condition", [" after_visit ", "before_visit", "last_visit"]).unwrap(),
        NumericValues::new("y", [2.0, 1.0, 3.0]),
        CategoryValues::new("group", ["a", "a", "a"]).unwrap(),
    )
    .with_alpha(
        crate::alpha::AlphaTarget::Line,
        crate::alpha::AlphaEncoding::keyed(
            [CategoryValues::new("group", ["a"]).unwrap()],
            crate::alpha::AlphaValues::new("trajectory confidence", [128]).unwrap(),
        )
        .unwrap(),
    )
    .with_alpha(
        crate::alpha::AlphaTarget::Point,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::new("confidence", [0, 128, 255]).unwrap(),
        ),
    )
    .erase();
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
    let theme = crate::Theme::publication().resolve().unwrap();
    let plot = inputs.attach_with_guides_for_test(source, &theme).unwrap();
    let panel = PanelBox::new(30.0, 810.0, 50.0, 600.0);
    let selected = plot
        .measure_selected_interior(PlotRenderContext::new(plot.data(), panel, &theme))
        .unwrap();
    assert_eq!(selected.x_domain, AxisDomain::Categorical);
    assert_eq!(selected.y_domain, AxisDomain::Continuous);
    assert_eq!(selected.layout.projection.x_range(), (0.0, 3.0));
    assert_eq!(
        selected.x_tick_keys.as_ref().unwrap(),
        &["after_visit", "before_visit", "last_visit"]
    );
    assert_eq!(
        selected
            .layout
            .x_axis
            .ticks
            .iter()
            .map(|tick| tick.position)
            .collect::<Vec<_>>(),
        [0.5, 1.5, 2.5]
    );
    assert_eq!(
        selected
            .layout
            .x_axis
            .ticks
            .iter()
            .map(|tick| tick.label.as_str())
            .collect::<Vec<_>>(),
        plot.prepared_axis_inputs().unwrap().display_categories[0]
            .as_ref()
            .unwrap()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        selected.layout.x_axis.source,
        crate::layout::TickSource::SUPPLIED
    );
    assert!(selected.y_tick_keys.is_none());
    let crate::layout::interior::MarkPayload::Line(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("Trajectory payload")
    };
    assert_eq!(
        payload
            .marks
            .iter()
            .filter_map(
                |mark| if let crate::plots::line::LineMark::Points(_, style) = mark {
                    Some(style.color.get_alpha())
                } else {
                    None
                }
            )
            .collect::<Vec<_>>(),
        [0, 128, 255]
    );
    assert_eq!(
        payload
            .marks
            .iter()
            .filter_map(
                |mark| if let crate::plots::line::LineMark::Line(_, style) = mark {
                    Some(style.color.get_alpha())
                } else {
                    None
                }
            )
            .collect::<Vec<_>>(),
        [128]
    );
}

#[test]
fn coordinate_root_classes_preserve_dimensions_and_prepared_intrinsics() {
    for source in [sources()[0].clone(), categorical_source()] {
        let source = source.labels(
            crate::Labels::humanized().alias("after_visit", "A deliberately long category label"),
        );
        for (w, h) in [(640, 480), (400, 300), (801, 601)] {
            let plot = source.clone().size(w, h);
            crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
            crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS.with(|calls| calls.set(0));
            measurements::start();
            let mut backend = crate::render::svg_backend::SvgBackend::in_memory(w, h);
            plot.render_with_backend(&mut backend).unwrap();
            let work = measurements::finish();
            let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
            assert!(svg.contains(&format!("viewBox=\"0 0 {w} {h}\"")));
            let counter = match InteriorRoute::classify(plot.kind()) {
                InteriorRoute::OneCategoricalAxis(_) => Counter::OneCategoricalAxisMeasure,
                _ => Counter::StandardCoordinateMeasure,
            };
            assert!(work.counters[counter as usize] > 0);
            assert_eq!(work.counters[Counter::StandardPanelIntrinsic as usize], 1);
            assert_eq!(work.counters[Counter::RootTitleIntrinsic as usize], 1);
            assert_eq!(work.intrinsics.len(), 2);
            assert!(work.intrinsics.iter().all(|event| event.scope.is_none()
                && event.root_round.is_none()
                && matches!(
                    event.counter,
                    Counter::StandardPanelIntrinsic | Counter::RootTitleIntrinsic
                )));
            assert_eq!(
                work.capture_counters[Counter::StandardPanelIntrinsic as usize],
                0
            );
            assert_eq!(
                work.capture_counters[Counter::RootTitleIntrinsic as usize],
                0
            );
            assert_eq!(work.capture_counters[Counter::LegendSelection as usize], 0);
            assert_eq!(work.counters[Counter::LegacyPanelConstructor as usize], 0);
            assert_eq!(
                crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
                0
            );
            assert_eq!(
                crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS.with(std::cell::Cell::get),
                0
            );
        }
    }
}

#[test]
fn native_exact_sources_are_indivisible_and_missing_axes_never_fall_back() {
    for source in all_sources() {
        let source = source.with_planned_x_limits((0.0, 2.0));
        let source = if matches!(
            InteriorRoute::classify(source.kind()),
            InteriorRoute::StandardCoordinate(_)
        ) {
            source.with_planned_y_limits((0.0, 4.0))
        } else {
            source
        };
        let mut inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        let axes = &mut std::sync::Arc::get_mut(&mut inputs.axes).unwrap()[0].1;
        axes.specs[0].exact_ticks = Some(vec![
            crate::layout::AxisTick::new(0.0, "first"),
            crate::layout::AxisTick::new(
                if axes.categories[0].is_some() {
                    axes.ranges[0].1
                } else {
                    2.0
                },
                "last",
            ),
        ]);
        if axes.categories[1].is_none() {
            axes.specs[1].exact_ticks = Some(vec![crate::layout::AxisTick::new(2.0, "")]);
        }
        let theme = crate::Theme::publication().resolve().unwrap();
        let plot = inputs.attach_with_guides_for_test(source, &theme).unwrap();
        let context =
            PlotRenderContext::new(plot.data(), PanelBox::new(30.0, 610.0, 50.0, 440.0), &theme);
        let selected = plot.measure_selected_interior(context).unwrap();
        assert_eq!(
            selected.layout.x_axis.ticks,
            plot.prepared_axis_inputs().unwrap().specs[0]
                .exact_ticks
                .as_ref()
                .unwrap()
                .as_slice()
        );
        if let Some(exact) = &plot.prepared_axis_inputs().unwrap().specs[1].exact_ticks {
            assert_eq!(selected.layout.y_axis.ticks, *exact);
        } else {
            assert_eq!(
                selected.layout.y_axis.source,
                crate::layout::TickSource::SUPPLIED
            );
        }
        assert!(context
            .with_prepared_axes(&plot)
            .unwrap()
            .layout_with_axis_specs(
                (0.0, 1.0),
                (0.0, 1.0),
                context.panel,
                None,
                None,
                &crate::layout::AxisSpec::default(),
                &crate::layout::AxisSpec::default(),
                &theme
            )
            .is_err());
        let mut invalid = plot.clone();
        let axes = std::sync::Arc::get_mut(&mut invalid.prepared_axes);
        assert!(axes.is_none(), "prepared axes are shared, not cloned");
        invalid.prepared_axes = std::sync::Arc::new(Vec::new());
        assert!(matches!(
            invalid.measure_selected_interior(PlotRenderContext::new(
                invalid.data(),
                context.panel,
                &theme
            )),
            Err(PlanError::Fatal(Error::Render(_)))
        ));
    }
}

#[test]
fn native_exact_tick_misses_keep_axis_scope_and_never_coarsen_or_project_marks() {
    for source in all_sources() {
        let source = source.with_planned_x_limits((0.0, 2.0));
        let source = if matches!(
            InteriorRoute::classify(source.kind()),
            InteriorRoute::StandardCoordinate(_)
        ) {
            source.with_planned_y_limits((0.0, 4.0))
        } else {
            source
        };
        let mut inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationContext::new(), None).unwrap();
        let axes = &mut std::sync::Arc::get_mut(&mut inputs.axes).unwrap()[0].1;
        axes.specs[0].exact_ticks = Some(vec![crate::layout::AxisTick::new(1.0, "W".repeat(200))]);
        if axes.categories[1].is_none() {
            axes.specs[1].exact_ticks = Some(vec![crate::layout::AxisTick::new(2.0, "")]);
        }
        let theme = crate::Theme::publication().resolve().unwrap();
        let plot = inputs.attach_with_guides_for_test(source, &theme).unwrap();
        let scope = fit::FitScope::new(
            &crate::layout::ComponentAddress::root()
                .child(crate::layout::AddressRole::PlotBody, 0)
                .unwrap(),
            4,
        );
        let projected = Projection::test_projection_calls();
        measurements::start();
        let error = plot
            .measure_selected_interior(
                PlotRenderContext::new(
                    plot.data(),
                    PanelBox::new(30.0, 610.0, 50.0, 440.0),
                    &theme,
                )
                .with_fit_scope(&scope),
            )
            .unwrap_err();
        let work = measurements::finish();
        let PlanError::Fit(failure) = error else {
            panic!("exact labels must return a typed fit miss")
        };
        assert_eq!(
            failure.reason.address,
            scope
                .child(crate::layout::AddressRole::AxisBottom, 0)
                .unwrap()
                .address
        );
        assert_eq!(failure.reason.ordinal, 0);
        assert!(matches!(failure.reason.kind, FitKind::Tick | FitKind::Text));
        assert_eq!(Projection::test_projection_calls(), projected);
        assert_eq!(work.counters[Counter::TickSource as usize], 0);
        assert_eq!(work.counters[Counter::LegacyPanelConstructor as usize], 0);
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "each route compares complete selected replay after cached, changed, and rejected parent assignments"
)]
fn native_child_reuse_and_rejected_ancestor_trials_preserve_selected_bytes() {
    use crate::decorations::plan::children::{ChildPlans, DISABLE_CHILD_REUSE};
    use crate::decorations::plan::LegendRole;
    use crate::layout::kernel::{FeedbackKind, TrialOutcome};
    use crate::layout::{AddressRole, ComponentAddress};
    for source in [
        sources()[0].clone(),
        categorical_source(),
        crate::plots::categorical_distribution::extraction::supplied(),
    ] {
        let source = source
            .title("Measured root title")
            .caption("Independent caption");
        let mut outputs = Vec::new();
        for disabled in [false, true] {
            DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
            let theme = crate::Theme::publication().resolve().unwrap();
            let owner = ComponentAddress::root()
                .child(AddressRole::FacetCell, 0)
                .unwrap();
            let sibling = ComponentAddress::root()
                .child(AddressRole::FacetCell, 1)
                .unwrap();
            let mut children = ChildPlans::facets(&source);
            let mut previous_selected = None;
            measurements::start();
            let root = crate::layout::plan_exact_root(
                640,
                480,
                |stage| {
                    let inputs =
                        crate::plots::shared_x::prepare_plot(&source, &stage.operation(), None)?;
                    inputs.prepare_discrete_guides(
                        &source,
                        &theme,
                        &stage.operation(),
                        &owner,
                        false,
                    )?;
                    let requirements = crate::layout::foundational_plot_requirements(
                        Some(&inputs),
                        &source,
                        source.kind(),
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
                |inputs, assigned, round| {
                    let (width, height) = assigned.dimensions()?;
                    let plot = inputs
                        .attach(source.clone())?
                        .with_canvas_size(width, height)
                        .with_parent_axis_gutters();
                    let canvas = canvas_layout(&plot, &theme).map_err(PlanError::into_error)?;
                    round.observe_canvas(&canvas)?;
                    let panel = PanelBox::new(80.0, 600.0, 80.0, 440.0);
                    let context = PlotRenderContext::new(plot.data(), panel, &theme);
                    let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
                    let first = fit::measured(children.plan(
                        &plot,
                        LegendRole::Suppressed,
                        context,
                        None,
                        round,
                        &owner,
                    ))
                    .map_err(PlanError::into_error)?;
                    if let Some(previous) = &previous_selected {
                        assert_eq!(
                            std::sync::Arc::ptr_eq(previous, &first),
                            !disabled,
                            "first child call must reuse the previous round, not just this round"
                        );
                    }
                    let requested = round
                        .next_insets(&owner, FeedbackKind::AxisGutter)
                        .map(f64::to_bits);
                    let stable_for_reuse = round.child_stable(std::slice::from_ref(&owner));
                    let cached = fit::measured(children.plan(
                        &plot,
                        LegendRole::Suppressed,
                        context,
                        None,
                        round,
                        &owner,
                    ))
                    .map_err(PlanError::into_error)?;
                    assert_eq!(
                        std::sync::Arc::ptr_eq(&first, &cached),
                        !disabled && stable_for_reuse
                    );
                    assert_eq!(
                        round
                            .next_insets(&owner, FeedbackKind::AxisGutter)
                            .map(f64::to_bits),
                        requested
                    );
                    let trial = round.begin_trial()?;
                    let changed = plot
                        .clone()
                        .with_planned_x_limits((-10.0, 10.0))
                        .with_planned_y_limits((-5.0, 5.0));
                    let rejected = fit::measured(children.plan(
                        &changed,
                        LegendRole::Suppressed,
                        context.with_panel(PanelBox::new(100.0, 580.0, 100.0, 430.0)),
                        None,
                        round,
                        &owner,
                    ))
                    .map_err(PlanError::into_error)?;
                    let axes = plot.prepared_axis_inputs().unwrap();
                    let with_existing_domain =
                        |manual: (f64, f64), kind| -> Result<_, crate::Error> {
                            Ok(round.feedback_domain(&owner, kind).map_or(manual, |range| {
                                (manual.0.min(range.0), manual.1.max(range.1))
                            }))
                        };
                    assert_eq!(
                        rejected.result.layout.projection.x_range(),
                        if axes.categories[0].is_some() {
                            axes.ranges[0]
                        } else {
                            with_existing_domain((-10.0, 10.0), FeedbackKind::AutomaticDomainX)?
                        }
                    );
                    assert_eq!(
                        rejected.result.layout.projection.y_range(),
                        if axes.categories[1].is_some() {
                            axes.ranges[1]
                        } else {
                            with_existing_domain((-5.0, 5.0), FeedbackKind::AutomaticDomainY)?
                        }
                    );
                    assert_ne!(first.panel_box, rejected.panel_box);
                    round.miss(
                        &owner,
                        FitKind::AssignedChild,
                        &[],
                        "rejected ancestor trial",
                    )?;
                    assert!(matches!(round.finish_trial(trial), TrialOutcome::Miss(_)));
                    let selected = fit::measured(children.plan(
                        &plot,
                        LegendRole::Suppressed,
                        context,
                        None,
                        round,
                        &owner,
                    ))
                    .map_err(PlanError::into_error)?;
                    assert_eq!(
                        selected.result.layout.projection.x_range(),
                        first.result.layout.projection.x_range()
                    );
                    assert_eq!(
                        selected.result.layout.projection.y_range(),
                        first.result.layout.projection.y_range()
                    );
                    assert_eq!(selected.panel_box, first.panel_box);
                    assert_eq!(
                        round
                            .next_insets(&owner, FeedbackKind::AxisGutter)
                            .map(f64::to_bits),
                        requested
                    );
                    assert_eq!(
                        crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                        reads
                    );
                    // A controlled deferred sibling reserve keeps this root active
                    // without changing the native child's keyed allocation. Unlike
                    // guide reflow, this proves reuse from a previous root round.
                    let consumed = round.insets(&sibling, FeedbackKind::OwnedInk);
                    let sibling_box = fit::measured(
                        crate::layout::kernel::FrameRequirements {
                            minimum_body: (1.0, 1.0),
                            natural_insets: [0.0; 4],
                        }
                        .assign_fit(
                            PanelBox::new(16.0, 60.0, 80.0, 440.0),
                            consumed,
                            &fit::FitScope::root(),
                        ),
                    )
                    .map_err(PlanError::into_error)?;
                    round.observe_box(sibling_box)?;
                    let request = super::super::InteriorReserveRequest::OwnedInk([
                        (consumed[0] + 1.0).min(2.0),
                        0.0,
                        0.0,
                        0.0,
                    ])
                    .submit(round, &sibling, &sibling)?;
                    if request.is_some() {
                        round.miss(
                            &sibling,
                            FitKind::OverlayCollision,
                            &[request],
                            "deferred sibling requires a changed owned-ink reserve",
                        )?;
                    }
                    // A child's own pending mark-domain request invalidates its
                    // provisional key even when a sibling keeps the root alive.
                    let reusable = round.child_stable(std::slice::from_ref(&owner));
                    children.finish_round(round);
                    previous_selected = reusable.then(|| std::sync::Arc::clone(&selected));
                    Ok(crate::layout::MeasuredLayout::new(
                        assigned,
                        (selected, canvas),
                    ))
                },
                |_, _, (selected, canvas), stage| {
                    crate::orchestrator::capture_text_replay_plan(
                        640,
                        480,
                        "native child selection",
                        &theme,
                        stage,
                        |backend| {
                            canvas.root_text.draw_retained(backend)?;
                            let axes = selected.result.axes.as_ref().unwrap();
                            axes.draw_grid(backend)?;
                            source.render_panel(
                                source.kind(),
                                PlotRenderContext::new(source.data(), selected.panel_box, &theme)
                                    .with_retained_body(&selected.result),
                                backend,
                            )?;
                            axes.draw(backend)
                        },
                    )
                },
            )
            .unwrap();
            let work = measurements::finish();
            DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
            assert_eq!(work.counters[Counter::LegacyPanelConstructor as usize], 0);
            assert_eq!(work.roots.len(), 3);
            assert_eq!(
                work.children
                    .iter()
                    .any(|event| event.root_round > 0
                        && event.counter == Counter::ChildFeedbackReplay),
                !disabled
            );
            assert_eq!(
                work.counters[Counter::ChildCacheHit as usize] > 0,
                !disabled
            );
            assert_eq!(
                work.counters[Counter::ChildFeedbackReplay as usize] > 0,
                !disabled
            );
            crate::orchestrator::validate_text_replay_plan(
                640,
                480,
                root.plan(),
                "native child Null",
            )
            .unwrap();
            let mut svg = root.plan().take_svg_backend().unwrap();
            root.plan()
                .replay(640, 480, &mut svg, "native child SVG")
                .unwrap();
            outputs.push(svg.into_bytes().unwrap());
        }
        assert_eq!(outputs[0], outputs[1]);
    }
}
