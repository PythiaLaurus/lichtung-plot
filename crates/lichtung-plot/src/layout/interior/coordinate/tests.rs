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

fn all_sources() -> impl Iterator<Item = Plot> {
    sources()
        .into_iter()
        .chain(super::super::estimates::sources())
        .chain(super::super::bars::sources())
        .chain(super::super::categorical_distributions::sources())
        .chain([crate::plots::interaction::extraction::source().erase()])
        .chain([crate::plots::categorical_distribution::extraction::supplied()])
}

#[test]
fn every_native_route_uses_exact_assigned_projection_and_frozen_axes_without_legacy_constructors() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    for source in sources() {
        crate::orchestrator::with_prepared_plot_for_test(&source, &theme, |plot, inputs, theme| {
            let operation = theme.text.prepared_font_environment().unwrap().operation();
            for panel in [
                PixelRect::new(80.25, 550.75, 70.5, 410.25),
                PixelRect::new(30.0, 810.0, 50.0, 600.0),
            ] {
                for ranges in [[(-1.0, 3.0), (0.0, 4.0)], [(-2.0, 5.0), (-1.0, 6.0)]] {
                    let plot = plot
                        .clone()
                        .with_planned_x_limits(ranges[0])
                        .with_planned_y_limits(ranges[1]);
                    measurements::start();
                    let selected = plot
                        .measure_selected_interior(
                            PlotRenderContext::new(
                                inputs.plots[0].data(&operation).unwrap(),
                                panel,
                                theme,
                            )
                            .with_registration(&inputs.plots[0]),
                        )
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
        })
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
        crate::orchestrator::with_prepared_plot_for_test(&source, &theme, |plot, inputs, theme| {
            let operation = theme.text.prepared_font_environment().unwrap().operation();
            let registered = inputs.plots.first();
            let axes = Intrinsics::prepared(plot, registered, theme)
                .unwrap()
                .minimum(plot);
            let minimum = plot.intrinsic_body_minimum(registered, theme).unwrap();
            assert!(minimum.0 >= axes.0 && minimum.1 >= axes.1);
            let scope = fit::FitScope::new(
                &crate::layout::ComponentAddress::root()
                    .child(crate::layout::AddressRole::PlotBody, 0)
                    .unwrap(),
                7,
            );
            let before = Projection::test_projection_calls();
            crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
            let error = plot
                .measure_selected_interior(
                    PlotRenderContext::new(
                        inputs.plots[0].data(&operation).unwrap(),
                        PixelRect::new(0.0, minimum.0 - 1.0 / 1024.0, 0.0, minimum.1),
                        theme,
                    )
                    .with_registration(&inputs.plots[0])
                    .with_fit_scope(&scope),
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
        })
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
    crate::orchestrator::with_prepared_plot_for_test(&source, &theme, |plot, inputs, theme| {
        let operation = theme.text.prepared_font_environment().unwrap().operation();
        let panel = PixelRect::new(30.0, 810.0, 50.0, 600.0);
        let selected = plot
            .measure_selected_interior(
                PlotRenderContext::new(inputs.plots[0].data(&operation).unwrap(), panel, theme)
                    .with_registration(&inputs.plots[0]),
            )
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
            inputs.plots[0].axes.as_ref().unwrap().display_categories[0]
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
    })
    .unwrap();
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one real operation injects the original prepared-axis fault after science and checks downstream fit before projection"
)]
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
        let theme = crate::Theme::publication().resolve().unwrap();
        let text = crate::text::TextService::default();
        let theme = crate::text::TextRequestContext::new(&theme, &text);
        crate::layout::plan_root(
            crate::layout::RootRequest::ExactSize {
                width: 800,
                height: 600,
            },
            |stage| {
                let operation = stage.operation();
                let mut inputs = crate::plots::shared_x::prepare_plot(&source, &operation, None)?;
                let axes = inputs.plots[0].axes.as_mut().unwrap();
                axes.specs[0].exact_ticks =
                    Some(vec![crate::layout::AxisTick::new(1.0, "W".repeat(200))]);
                if axes.categories[1].is_none() {
                    axes.specs[1].exact_ticks = Some(vec![crate::layout::AxisTick::new(2.0, "")]);
                }
                inputs.prepare_discrete_guides(
                    &source,
                    &theme,
                    &operation,
                    &crate::layout::ComponentAddress::root(),
                    false,
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
                text.prepare_font_environment_from_census(&mut census, presence, stage)?;
                Ok(inputs)
            },
            |inputs, assigned, _| {
                let (width, height) = assigned.assigned_size()?.dimensions()?;
                let plot = inputs
                    .attach(source.clone())?
                    .with_canvas_size(width, height);
                let operation = theme.text.prepared_font_environment().unwrap().operation();
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
                            inputs.plots[0].data(&operation).unwrap(),
                            PixelRect::new(30.0, 610.0, 50.0, 440.0),
                            &theme,
                        )
                        .with_registration(&inputs.plots[0])
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
