use super::*;
use crate::orchestrator::measurements::{self, Counter};
use crate::{CategoryValues, NumericValues};

fn sources() -> Vec<Plot> {
    let x = NumericValues::new("x", [0.0, 1.0, 2.0]);
    let y = NumericValues::new("y", [2.0, 1.0, 3.0]);
    let group = CategoryValues::new("group", ["a", "a", "a"]).unwrap();
    let [histogram, density, qq, regression] = crate::layout::interior::distributions::sources();
    vec![
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

#[test]
fn every_native_route_uses_exact_assigned_projection_and_frozen_axes_without_legacy_constructors() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    for source in sources() {
        crate::orchestrator::with_prepared_plot_for_test(
            &source,
            &theme,
            |_| Ok(()),
            |plot, inputs, _theme, _operation, round| {
                let measurement = inputs.assignment_factory().assign(
                    plot.sizing_request()
                        .assigned_size()
                        .unwrap_or(crate::layout::RootSize::new(1, 1)),
                );
                let registered = &inputs.plots[0];
                let factory = inputs
                    .plot_assignment_factory(&measurement, &registered.scope.id)
                    .unwrap();
                let panel = PixelRect::new(80.25, 550.75, 70.5, 410.25);
                let ranges = [(-1.0, 3.0), (0.0, 4.0)];
                let mut entry = factory
                    .assign(
                        crate::orchestrator::assignment::PlotAllocationInputs::for_test(
                            measurement.measurement_size,
                            panel,
                            round,
                            &registered.scope.address,
                        ),
                        round,
                        None,
                    )
                    .unwrap();
                // This is the supplied-allocation projection contract,
                // not evidence of factory domain derivation.
                let crate::orchestrator::assignment::AxisDomainAssignments::Pair(pair) =
                    &mut entry.domains
                else {
                    unreachable!("coordinate pair")
                };
                for (domain, (lower, upper)) in pair.iter_mut().zip(ranges) {
                    domain.extent =
                        crate::orchestrator::registration::DisplayExtent { lower, upper };
                }
                measurements::start();
                let selected = crate::orchestrator::measure_plot(
                    &factory,
                    entry,
                    crate::decorations::legend::discrete::scoped::GuideConsumption {
                        use_kind: crate::decorations::legend::discrete::scoped::GuideUse::Local,
                        ..crate::decorations::legend::discrete::scoped::SUPPRESSED_GUIDES
                    },
                    &crate::layout::fit::FitScope::root(),
                )
                .unwrap()
                .1;
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
            },
        )
        .unwrap();
    }
}

#[test]
fn every_native_route_reports_intrinsic_fit_before_mark_projection_or_backend_creation() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    for source in sources() {
        let source = source.x_label("").y_label("");
        crate::orchestrator::with_prepared_plot_for_test(
            &source,
            &theme,
            |_| Ok(()),
            |plot, inputs, theme, operation, round| {
                let registered = inputs.plots.first();
                let intrinsic = inputs.plot_intrinsic(&inputs.plots[0].scope.id).unwrap();
                let axes = crate::orchestrator::root::coordinate_intrinsic(
                    intrinsic,
                    &crate::orchestrator::root::intrinsic_inputs(registered, theme, operation)
                        .unwrap(),
                )
                .unwrap()
                .minimum(
                    crate::orchestrator::registration::axis_tick_labels_visible(registered, 1),
                );
                let minimum = plot
                    .intrinsic_body_minimum(registered, intrinsic, operation, theme, None)
                    .unwrap();
                assert!(minimum.0 >= axes.0 && minimum.1 >= axes.1);
                let scope = fit::FitScope::new(
                    &crate::layout::ComponentAddress::root()
                        .child(crate::layout::AddressRole::PlotBody, 0)
                        .unwrap(),
                    7,
                );
                let before = Projection::test_projection_calls();
                crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
                let measurement = inputs.assignment_factory().assign(
                    plot.sizing_request()
                        .assigned_size()
                        .unwrap_or(crate::layout::RootSize::new(1, 1)),
                );
                let factory = inputs
                    .plot_assignment_factory(&measurement, &inputs.plots[0].scope.id)
                    .unwrap();
                let entry = factory
                    .assign(
                        crate::orchestrator::assignment::PlotAllocationInputs::for_test(
                            measurement.measurement_size,
                            PixelRect::new(0.0, minimum.0 - 1.0 / 1024.0, 0.0, minimum.1),
                            round,
                            &inputs.plots[0].scope.address,
                        ),
                        round,
                        None,
                    )
                    .unwrap();
                let error = crate::orchestrator::measure_plot(
                    &factory,
                    entry,
                    crate::decorations::legend::discrete::scoped::GuideConsumption {
                        use_kind: crate::decorations::legend::discrete::scoped::GuideUse::Local,
                        ..crate::decorations::legend::discrete::scoped::SUPPRESSED_GUIDES
                    },
                    &scope,
                )
                .unwrap_err();
                let PlanError::Fit(failure) = error else {
                    panic!("typed intrinsic fit")
                };
                assert_eq!(failure.reason.kind, FitKind::TrackOrResidual);
                assert_eq!(failure.reason.address, scope.address);
                assert_eq!(failure.reason.ordinal, 7);
                assert_eq!(Projection::test_projection_calls(), before);
                assert_eq!(
                    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
                    0
                );
            },
        )
        .unwrap();
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one categorical trajectory contract compares original slots, display text and independent paints"
)]
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
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    crate::orchestrator::with_prepared_plot_for_test(
        &source,
        &theme,
        |_| Ok(()),
        |plot, inputs, _theme, _operation, round| {
            let panel = PixelRect::new(30.0, 810.0, 50.0, 600.0);
            let measurement = inputs.assignment_factory().assign(
                plot.sizing_request()
                    .assigned_size()
                    .unwrap_or(crate::layout::RootSize::new(1, 1)),
            );
            let factory = inputs
                .plot_assignment_factory(&measurement, &inputs.plots[0].scope.id)
                .unwrap();
            let selected =
                crate::layout::interior::measure_for_test(&factory, panel, round).unwrap();
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
                inputs.plots[0]
                    .axes
                    .coordinate()
                    .unwrap()
                    .categories(0)
                    .unwrap()
                    .ticks
                    .iter()
                    .map(|tick| tick.label.as_str())
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
                    .filter_map(|mark| {
                        if let crate::plots::line::LineMark::Points(_, style) = mark {
                            Some(style.color.get_alpha())
                        } else {
                            None
                        }
                    })
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
        },
    )
    .unwrap();
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one real operation injects exact intent after science and checks downstream fit before mark projection"
)]
fn exact_tick_misses_keep_both_axis_scopes_without_coarsening_or_mark_projection() {
    // Exact-tick fitting precedes the family-specific mark dispatch. Keep both
    // numeric-axis orientations, not another matrix of already prepared families.
    let scatter = Plot::scatter(
        NumericValues::new("x", [0.0, 1.0, 2.0]),
        NumericValues::new("y", [2.0, 1.0, 3.0]),
    )
    .erase();
    let [bar, _, _, _] = super::super::bars::sources();
    for source in [scatter, bar] {
        let fault_source = std::sync::OnceLock::new();
        let appearance = crate::Theme::publication().resolve().unwrap();
        crate::layout::plan_root(
            crate::layout::RootRequest::ExactSize {
                width: 800,
                height: 600,
            },
            |stage| {
                let operation = stage.operation();
                let (mut registry, composition) = crate::plots::shared_x::prepare_plot(
                    source.description(),
                    &operation,
                    &appearance,
                )?;
                // Supply the original fixture's numeric display domains without
                // manufacturing authored limits or changing categorical slots.
                let axes = registry.panels[0].axes.as_mut().unwrap();
                if axes.categories(0).is_none() {
                    axes.sources_mut()[0].base_seed =
                        Some(crate::orchestrator::registration::DisplayExtent {
                            lower: 0.0,
                            upper: 2.0,
                        });
                }
                if matches!(
                    InteriorRoute::classify(source.kind()),
                    InteriorRoute::StandardCoordinate(_)
                ) {
                    axes.sources_mut()[1].base_seed =
                        Some(crate::orchestrator::registration::DisplayExtent {
                            lower: 0.0,
                            upper: 4.0,
                        });
                }
                // Exact text now belongs to authored intent. Rebind this real
                // occurrence at the original post-science injection point.
                let fault = source.clone().with_test_axis_ticks(
                    0,
                    vec![crate::layout::AxisTick::new(1.0, "W".repeat(200))],
                );
                let fault = if axes.categories(1).is_none() {
                    fault.with_test_axis_ticks(1, vec![crate::layout::AxisTick::new(2.0, "")])
                } else {
                    fault
                };
                let fault = fault_source.get_or_init(|| fault);
                let registered = &mut registry.panels[0];
                registered.source = fault.description();
                registered.row_source = crate::data::source::RowSourceContext::new(
                    registered.source,
                    &registered.scope,
                    &operation,
                );
                crate::orchestrator::complete_prepared_plot_for_test(
                    fault.description(),
                    appearance.clone(),
                    registry,
                    composition,
                    stage,
                )
            },
            |root, assigned, round| {
                let measurement = root.assignment_factory().assign(assigned.assigned_size()?);
                let registered = &root.plots[0];
                let factory = root.plot_assignment_factory(&measurement, &registered.scope.id)?;
                let entry = factory.assign(
                    crate::orchestrator::assignment::PlotAllocationInputs::for_test(
                        measurement.measurement_size,
                        PixelRect::new(30.0, 610.0, 50.0, 440.0),
                        round,
                        &registered.scope.address,
                    ),
                    round,
                    None,
                )?;
                let scope = fit::FitScope::new(
                    &crate::layout::ComponentAddress::root()
                        .child(crate::layout::AddressRole::PlotBody, 0)
                        .unwrap(),
                    4,
                );
                let projected = Projection::test_projection_calls();
                measurements::start();
                let error = crate::orchestrator::measure_plot(
                    &factory,
                    entry,
                    crate::decorations::legend::discrete::scoped::GuideConsumption {
                        use_kind: crate::decorations::legend::discrete::scoped::GuideUse::Local,
                        ..crate::decorations::legend::discrete::scoped::SUPPRESSED_GUIDES
                    },
                    &scope,
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
                Ok(crate::layout::MeasuredLayout::new(
                    assigned.assigned_size()?,
                    (),
                ))
            },
            |_, _, (), _| Ok(()),
        )
        .unwrap();
    }
}
