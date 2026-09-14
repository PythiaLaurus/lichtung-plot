use super::*;
use crate::font::canonical::OperationGenerationResources;
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

#[test]
fn native_multiline_axis_gutters_cannot_consume_root_caption_or_title_reserves() {
    use crate::decorations::plan::{plan_panel_for_test, LegendRole};
    use crate::render::backend::Backend;
    use crate::render::geometry::GeometryBackend;
    for source in sources().into_iter().chain([categorical_source()]) {
        let source = source
            .x_label("First axis line\nSecond axis line\nThird axis line")
            .title("Root title\nSecond title line")
            .caption("First caption line\nSecond caption line\nThird caption line")
            .with_suppressed_legend()
            .size(401, 401);
        let theme = crate::Theme::Publication.resolve();
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let plot = inputs.attach(source).unwrap();
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
        let crate::layout::canvas::CanvasRootText::Planned(text) = &canvas.root_text else {
            panic!("planned root text")
        };
        let mut roots = GeometryBackend::new(&theme).unwrap();
        roots.begin(plot.width(), plot.height()).unwrap();
        text.draw_retained(&mut roots).unwrap();
        roots.end().unwrap();
        let roots = roots.into_snapshot();
        let caption = roots.bounds().last().unwrap();
        let title = roots.bounds().first().unwrap();
        let axes = plan
            .result
            .axes
            .as_ref()
            .unwrap()
            .geometry_snapshot(&theme)
            .unwrap();
        for ink in axes.bounds() {
            assert!(
                ink.bottom
                    >= caption.top
                        + theme
                            .composition
                            .root_title_block
                            .caption_to_content_clearance
            );
            assert!(
                ink.top
                    <= title.bottom
                        - theme
                            .composition
                            .root_title_block
                            .title_block_to_content_clearance
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
fn native_narrow_exact_root_selects_actual_ticks_instead_of_rejecting_endpoint_estimates() {
    use crate::decorations::plan::{plan_panel_for_test, LegendRole};
    for source in sources() {
        let source = source
            .with_planned_y_limits((0.001, 1.0))
            .x_label("")
            .y_label("")
            .with_suppressed_legend()
            .size(175, 300);
        let theme = crate::Theme::Publication.resolve();
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let plot = inputs.attach(source).unwrap();
        let canvas = canvas_layout(&plot, &theme).unwrap();
        let endpoint = fit::intrinsic_widths(
            &theme,
            "1.00e-3",
            &theme.typography.tick_label,
            "endpoint estimate",
        )
        .unwrap()
        .1;
        assert!(
            175.0 - 40.0 - endpoint - theme.axes.spacing.y_tick_label_gap - 5.0
                < DeferredLegacyPanelMinimum::SIZE.0
        );
        let plan = plan_panel_for_test(
            &plot,
            plot.kind(),
            canvas.panel_box,
            LegendRole::Suppressed,
            &theme,
        )
        .unwrap();
        assert!(plan.panel_box.width() >= DeferredLegacyPanelMinimum::SIZE.0);
        assert_eq!(plan.result.layout.projection.y_range(), (0.001, 1.0));
        assert_eq!(
            plan.result
                .layout
                .y_axis
                .ticks
                .iter()
                .map(|tick| tick.label.as_str())
                .collect::<Vec<_>>(),
            ["0.25", "0.5", "0.75", "1"]
        );
        let selected = plot
            .measure_selected_interior(PlotRenderContext::new(plot.data(), plan.panel_box, &theme))
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
    for source in sources().into_iter().chain([categorical_source()]) {
        let source = source
            .caption("First caption line\nSecond caption line")
            .with_suppressed_legend();
        let theme = crate::Theme::Publication.resolve();
        let owner = ComponentAddress::root()
            .child(AddressRole::PlotBody, 0)
            .unwrap();
        let mut outputs = Vec::new();
        for seeded in [false, true] {
            let mut first = true;
            let mut contracted = false;
            let root = crate::layout::plan_exact_root(
                source.width(),
                source.height(),
                |stage| {
                    crate::plots::shared_x::prepare_plot(&source, &stage.generation_resources())
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
fn native_hard_title_spans_and_category_slots_exceed_the_named_parity_floor() {
    let theme = crate::Theme::Publication.resolve();
    let word = "W".repeat(20);
    let category = Plot::trajectory(
        CategoryValues::new("condition", [word.as_str(), "middle", "last"]).unwrap(),
        NumericValues::new("y", [2.0, 1.0, 3.0]),
        CategoryValues::new("group", ["a", "a", "a"]).unwrap(),
    )
    .erase();
    for (source, axis) in sources()
        .into_iter()
        .flat_map(|source| {
            [
                (source.clone().x_label(&word), 0),
                (source.y_label(&word), 1),
            ]
        })
        .chain([(category, 0)])
    {
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let plot = inputs.attach(source).unwrap();
        let minimum = Intrinsics::new(&plot, &theme).unwrap().minimum_body;
        let is_category = plot.prepared_axis_inputs().unwrap().display_categories[0].is_some();
        let hard = fit::intrinsic_widths(
            &theme,
            &word,
            if is_category {
                &theme.typography.tick_label
            } else {
                &theme.typography.axis_title
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
        assert!(expected > 80.0);
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
        plot.measure_selected_interior(PlotRenderContext::new(
            plot.data(),
            PanelBox::new(0.0, minimum.0.ceil() + 1.0, 0.0, minimum.1.ceil() + 1.0),
            &theme,
        ))
        .unwrap();
    }
}

#[test]
fn native_resource_refusal_precedes_mark_materialization_and_backend_creation() {
    for source in sources().into_iter().chain([categorical_source()]) {
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let plot = inputs.attach(source).unwrap();
        let theme = crate::Theme::Publication.resolve();
        Intrinsics::new(&plot, &theme).unwrap();
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        resources.set_test_limits(
            resources.usage().unwrap().1,
            resources.byte_usage().unwrap(),
        );
        crate::plots::scatter::STAGES.with(|counts| counts.set([0; 4]));
        crate::plots::line::STAGES.with(|counts| counts.set([[0; 4]; 3]));
        crate::layout::interior::distributions::STAGES.with(|counts| counts.set([[0; 4]; 4]));
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
        crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS.with(|calls| calls.set(0));
        let before = Projection::test_projection_calls();
        let error = plot
            .measure_selected_interior(PlotRenderContext::new(
                plot.data(),
                PanelBox::new(30.0, 610.0, 50.0, 440.0),
                &theme,
            ))
            .unwrap_err();
        assert!(matches!(error, PlanError::Fatal(Error::Render(_))));
        assert_eq!(Projection::test_projection_calls(), before);
        assert_eq!(
            crate::plots::scatter::STAGES.with(std::cell::Cell::get),
            [0; 4]
        );
        assert_eq!(
            crate::plots::line::STAGES.with(std::cell::Cell::get),
            [[0; 4]; 3]
        );
        assert_eq!(
            crate::layout::interior::distributions::STAGES.with(std::cell::Cell::get),
            [[0; 4]; 4]
        );
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

#[test]
fn native_real_compositions_remeasure_gutters_and_reject_guides_identically_without_cache() {
    use crate::decorations::plan::children::DISABLE_CHILD_REUSE;
    use crate::orchestrator::measurements::GuideOutcome;
    fn configured<K: crate::kind::Hue + crate::kind::Facet + crate::kind::ExactFamily>(
        plot: Plot<K>,
        faceted: bool,
    ) -> (Plot, Option<crate::Canvas>) {
        let plot = plot
            .hue(CategoryValues::new("group", vec!["M".repeat(34); 6]).unwrap())
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
        let plots = [
            configured(Plot::scatter(x.clone(), y.clone()), faceted),
            configured(Plot::line(x.clone(), y.clone()), faceted),
            configured(Plot::density(x.clone()), faceted),
            configured(
                Plot::trajectory(x.clone(), y.clone(), group.clone()),
                faceted,
            ),
            configured(Plot::line_path(x, y.clone(), group.clone()), faceted),
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
                        .any(|guide| guide.outcome == GuideOutcome::Rejected),
                    "{:?}, faceted={faceted}: {:?}",
                    plot.kind(),
                    work.guides
                );
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
    let theme = crate::Theme::Publication.resolve();
    for source in sources() {
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        assert!(inputs.route.is_native());
        let plot = inputs.attach(source).unwrap();
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
    let theme = crate::Theme::Publication.resolve();
    for source in sources() {
        let source = source.x_label("").y_label("");
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let plot = inputs.attach(source).unwrap();
        let minimum = Intrinsics::new(&plot, &theme).unwrap().minimum_body;
        assert_eq!(minimum, DeferredLegacyPanelMinimum::SIZE);
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
fn categorical_trajectory_retains_bottom_slots_raw_and_display_identity() {
    let source = Plot::trajectory(
        CategoryValues::new("condition", [" after_visit ", "before_visit", "last_visit"]).unwrap(),
        NumericValues::new("y", [2.0, 1.0, 3.0]),
        CategoryValues::new("group", ["a", "a", "a"]).unwrap(),
    )
    .erase();
    let inputs =
        crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
            .unwrap();
    let plot = inputs.attach(source).unwrap();
    let theme = crate::Theme::Publication.resolve();
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
}

#[test]
fn every_native_route_admits_intrinsics_and_selected_candidates_at_exact_resource_caps() {
    for source in sources() {
        let inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let plot = inputs.attach(source).unwrap();
        for intrinsic_only in [true, false] {
            let consume = |theme: &crate::theme::ResolvedTheme| -> Result<(), PlanError> {
                if intrinsic_only {
                    Intrinsics::new(&plot, theme)?;
                } else {
                    plot.measure_selected_interior(PlotRenderContext::new(
                        plot.data(),
                        PanelBox::new(30.0, 610.0, 50.0, 440.0),
                        theme,
                    ))?;
                }
                Ok(())
            };
            let theme = crate::Theme::Publication.resolve();
            consume(&theme).unwrap();
            let resources = theme
                .prepared_font_environment()
                .unwrap()
                .generation_resources();
            let before = (
                resources.usage().unwrap().1,
                resources.byte_usage().unwrap(),
            );
            consume(&theme).unwrap();
            let (visits, bytes) = (
                resources.usage().unwrap().1 - before.0,
                resources.byte_usage().unwrap() - before.1,
            );
            for (visits, bytes, succeeds) in [
                (visits, bytes, true),
                (visits - 1, bytes, false),
                (visits, bytes - 1, false),
            ] {
                let theme = crate::Theme::Publication.resolve();
                consume(&theme).unwrap();
                let resources = theme
                    .prepared_font_environment()
                    .unwrap()
                    .generation_resources();
                resources
                    .claim_visits(1_000_000 - resources.usage().unwrap().1 - visits)
                    .unwrap();
                resources
                    .claim_bytes(536_870_912 - resources.byte_usage().unwrap() - bytes)
                    .unwrap();
                let result = consume(&theme);
                if succeeds {
                    result.unwrap();
                } else {
                    assert!(matches!(result, Err(PlanError::Fatal(Error::Render(_)))));
                }
            }
        }
    }
}

#[test]
fn every_native_root_preserves_default_and_explicit_dimensions_and_avoids_layout_backends() {
    for source in sources() {
        for size in [None, Some((400, 300)), Some((801, 601))] {
            let plot = size.map_or_else(|| source.clone(), |(w, h)| source.clone().size(w, h));
            let (w, h) = size.unwrap_or((640, 480));
            assert_eq!((plot.width(), plot.height()), (w, h));
            crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
            crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS.with(|calls| calls.set(0));
            measurements::start();
            let mut backend = crate::render::svg_backend::SvgBackend::in_memory(w, h);
            plot.render_with_backend(&mut backend).unwrap();
            let work = measurements::finish();
            let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
            assert!(svg.contains(&format!("viewBox=\"0 0 {w} {h}\"")));
            assert!(work.counters[Counter::StandardCoordinateMeasure as usize] > 0);
            assert!(work.counters[Counter::StandardPanelIntrinsic as usize] > 0);
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
    for source in sources() {
        let source = source
            .with_planned_x_limits((0.0, 2.0))
            .with_planned_y_limits((0.0, 4.0));
        let mut inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let axes = &mut std::sync::Arc::get_mut(&mut inputs.axes).unwrap()[0].1;
        axes.specs[0].exact_ticks = Some(vec![
            crate::layout::AxisTick::new(0.0, "first"),
            crate::layout::AxisTick::new(2.0, "last"),
        ]);
        axes.specs[1].exact_ticks = Some(vec![crate::layout::AxisTick::new(2.0, "")]);
        let plot = inputs.attach(source).unwrap();
        let theme = crate::Theme::Publication.resolve();
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
        assert_eq!(
            selected.layout.y_axis.ticks,
            plot.prepared_axis_inputs().unwrap().specs[1]
                .exact_ticks
                .as_ref()
                .unwrap()
                .as_slice()
        );
        assert!(context
            .with_prepared_axes(&plot)
            .axis_layout(|| panic!("legacy fallback"))
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
    for source in sources() {
        let source = source
            .with_planned_x_limits((0.0, 2.0))
            .with_planned_y_limits((0.0, 4.0));
        let mut inputs =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let axes = &mut std::sync::Arc::get_mut(&mut inputs.axes).unwrap()[0].1;
        axes.specs[0].exact_ticks = Some(vec![crate::layout::AxisTick::new(1.0, "W".repeat(200))]);
        axes.specs[1].exact_ticks = Some(vec![crate::layout::AxisTick::new(2.0, "")]);
        let plot = inputs.attach(source).unwrap();
        let theme = crate::Theme::Publication.resolve();
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
fn native_child_reuse_and_rejected_ancestor_trials_preserve_selected_bytes_and_charge_work() {
    use crate::decorations::plan::children::{ChildPlans, DISABLE_CHILD_REUSE};
    use crate::decorations::plan::LegendRole;
    use crate::layout::kernel::{FeedbackKind, TrialOutcome};
    use crate::layout::{AddressRole, ComponentAddress};
    for source in sources().into_iter().chain([categorical_source()]) {
        let mut outputs = Vec::new();
        for disabled in [false, true] {
            DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
            let theme = crate::Theme::Publication.resolve();
            let owner = ComponentAddress::root()
                .child(AddressRole::FacetCell, 0)
                .unwrap();
            let mut children = ChildPlans::facets(&source);
            measurements::start();
            let root = crate::layout::plan_exact_root(
                640,
                480,
                |stage| {
                    let inputs = crate::plots::shared_x::prepare_plot(
                        &source,
                        &stage.generation_resources(),
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
                    let plot = inputs.attach(source.clone())?.with_parent_axis_gutters();
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
                    let requested = round
                        .next_insets(&owner, FeedbackKind::AxisGutter)
                        .map(f64::to_bits);
                    let cached = fit::measured(children.plan(
                        &plot,
                        LegendRole::Suppressed,
                        context,
                        None,
                        round,
                        &owner,
                    ))
                    .map_err(PlanError::into_error)?;
                    assert_eq!(std::sync::Arc::ptr_eq(&first, &cached), !disabled);
                    assert_eq!(
                        round
                            .next_insets(&owner, FeedbackKind::AxisGutter)
                            .map(f64::to_bits),
                        requested
                    );
                    let resources = theme.prepared_font_environment()?.generation_resources();
                    let before = (
                        resources.usage().unwrap().1,
                        resources.byte_usage().unwrap(),
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
                    assert_eq!(rejected.result.layout.projection.x_range(), (-10.0, 10.0));
                    assert_ne!(first.panel_box, rejected.panel_box);
                    round.miss(
                        &owner,
                        FitKind::AssignedChild,
                        &[],
                        "rejected ancestor trial",
                    )?;
                    assert!(matches!(round.finish_trial(trial), TrialOutcome::Miss(_)));
                    assert!(resources.usage().unwrap().1 > before.0);
                    assert!(resources.byte_usage().unwrap() > before.1);
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
                    children.finish_round(round);
                    Ok(crate::layout::MeasuredLayout::new(assigned, selected))
                },
                |_, _, selected, stage| {
                    crate::orchestrator::capture_text_replay_plan(
                        640,
                        480,
                        "native child selection",
                        &theme,
                        stage,
                        |backend| {
                            let axes = selected.result.axes.as_ref().unwrap();
                            axes.draw_grid(backend)?;
                            source.render_panel(
                                source.kind(),
                                PlotRenderContext::new(source.data(), selected.panel_box, &theme)
                                    .with_retained_body(selected.result.retained_payload()),
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
