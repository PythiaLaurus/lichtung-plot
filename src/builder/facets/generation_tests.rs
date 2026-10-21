use super::*;
use crate::decorations::legend::discrete::scoped::ScopedGuideInputs;
use crate::TickGeneration;

fn plot() -> Plot<crate::kind::Scatter> {
    Plot::scatter(
        crate::NumericValues::new("x", [0.0, 5.0, 10.0]),
        crate::NumericValues::new("y", [1.0, 3.0, 2.0]),
    )
    .x_limits(0.0, 10.0)
    .y_limits(0.0, 4.0)
}

#[test]
fn generation_copy_and_composed_identity_include_both_axes() {
    let mut plot = plot()
        .x_tick_generation(TickGeneration::additive(2.0).unwrap())
        .y_tick_generation(TickGeneration::additive(1.0).unwrap())
        .x_tick_format(
            crate::numeric::format::NumericFormat::default()
                .with_prefix("x ")
                .unwrap(),
        )
        .y_tick_format(
            crate::numeric::format::NumericFormat::default()
                .with_suffix(" y")
                .unwrap(),
        )
        .erase();

    let retained = plot
        .retain_composed_inputs(ScopedGuideInputs::for_test(&plot))
        .unwrap();
    assert_eq!(retained.x_axis_spec.generation, plot.x_axis_spec.generation);
    assert_eq!(retained.y_axis_spec.generation, plot.y_axis_spec.generation);
    assert_eq!(retained.x_axis_spec.format, plot.x_axis_spec.format);
    assert_eq!(retained.y_axis_spec.format, plot.y_axis_spec.format);
    assert!(plot.same_composed_inputs(&retained, &retained.guides));
    plot.x_axis_spec.generation = TickGeneration::additive(3.0).unwrap();
    assert!(!plot.same_composed_inputs(&retained, &retained.guides));
    plot.x_axis_spec.generation = retained.x_axis_spec.generation;
    plot.y_axis_spec.generation = TickGeneration::automatic();
    assert!(!plot.same_composed_inputs(&retained, &retained.guides));
    plot.y_axis_spec.generation = retained.y_axis_spec.generation;
    for selection in [
        TickGeneration::target(5).unwrap(),
        TickGeneration::maximum(5).unwrap(),
    ] {
        let mut changed = plot.clone();
        changed.x_axis_spec.generation = selection;
        assert!(!changed.same_composed_inputs(&retained, &retained.guides));
        let counted = changed
            .retain_composed_inputs(ScopedGuideInputs::for_test(&changed))
            .unwrap();
        assert!(changed.same_composed_inputs(&counted, &counted.guides));
        changed.x_axis_spec.generation = if selection.target_count().is_some() {
            TickGeneration::maximum(5).unwrap()
        } else {
            TickGeneration::target(5).unwrap()
        };
        assert!(!changed.same_composed_inputs(&counted, &counted.guides));
    }
    for side in 0..2 {
        let mut changed = plot.clone();
        if side == 0 {
            changed.x_axis_spec.format = None;
        } else {
            changed.y_axis_spec.format = None;
        }
        assert!(!changed.same_composed_inputs(&retained, &retained.guides));
    }
}

#[test]
fn equal_composed_endpoints_do_not_erase_range_provenance() {
    let configured = plot().erase();
    let automatic = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 10.0]),
        crate::NumericValues::new("y", [0.0, 4.0]),
    )
    .erase()
    .with_planned_x_limits((0.0, 10.0))
    .with_planned_y_limits((0.0, 4.0));
    let shared = automatic.clone().with_resolved_facet_ranges(
        PlotKind::Scatter,
        Some((0.0, 10.0)),
        Some((0.0, 4.0)),
    );
    let retained = automatic
        .retain_composed_inputs(ScopedGuideInputs::for_test(&automatic))
        .unwrap();
    assert_eq!(automatic.x_axis_spec.limits, configured.x_axis_spec.limits);
    assert_eq!(automatic.x_axis_spec.limits, shared.x_axis_spec.limits);
    assert!(automatic.same_composed_inputs(&retained, &retained.guides));
    assert!(!configured.same_composed_inputs(&retained, &retained.guides));
    assert!(!shared.same_composed_inputs(&retained, &retained.guides));
    let shared_retained = shared
        .retain_composed_inputs(ScopedGuideInputs::for_test(&shared))
        .unwrap();
    assert!(shared.same_composed_inputs(&shared_retained, &shared_retained.guides));
    assert_eq!(
        shared_retained.x_axis_spec.provenance,
        crate::layout::RangeProvenance::Shared
    );
}

#[test]
fn invalid_generation_rejects_before_font_discovery() {
    let discoveries = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
    for step in [0.0, -0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            TickGeneration::additive(step),
            Err(crate::Error::Data(_))
        ));
    }
    assert!(TickGeneration::log10_decades(0).is_err());
    let plot = plot().x_tick_generation(TickGeneration::log10_decades(2).unwrap());
    assert!(matches!(plot.preflight(), Err(crate::Error::Data(_))));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        discoveries
    );
}

#[test]
fn categorical_trajectory_generation_refuses_before_fonts_in_standalone_and_canvas() {
    let theme = crate::Theme::publication()
        .customize()
        .with_font_stack(
            crate::FontStack::new([
                crate::FontFamily::named("Absent categorical interval font").unwrap()
            ])
            .unwrap(),
        )
        .unwrap()
        .build()
        .unwrap();
    let trajectory = Plot::trajectory(
        crate::CategoryValues::new("condition", ["first", "second"]).unwrap(),
        crate::NumericValues::new("value", [1.0, 2.0]),
        crate::CategoryValues::new("group", ["a", "a"]).unwrap(),
    )
    .theme(theme);
    let discoveries = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
    for generation in [
        TickGeneration::additive(1.0).unwrap(),
        TickGeneration::target(4).unwrap(),
        TickGeneration::maximum(4).unwrap(),
    ] {
        let trajectory = trajectory.clone().x_tick_generation(generation);
        let error = trajectory.preflight().unwrap_err();
        assert!(matches!(error, crate::Error::Data(_)), "{error}");
        assert!(error
            .to_string()
            .contains("x-axis categories do not support numeric tick generation"));
        let error = crate::Canvas::grid(1, 1)
            .unwrap()
            .panel(0, 0, trajectory)
            .unwrap()
            .preflight()
            .unwrap_err();
        assert!(matches!(error, crate::Error::Data(_)), "{error}");
        assert!(error
            .to_string()
            .contains("x-axis categories do not support numeric tick generation"));
    }
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        discoveries
    );
}

#[test]
fn generation_rendering_and_exact_precedence_are_cache_independent() {
    for exact in [false, true] {
        let mut plot = plot()
            .x_tick_generation(TickGeneration::additive(2.0).unwrap())
            .y_tick_generation(TickGeneration::additive(1.0).unwrap())
            .x_tick_format(
                crate::numeric::format::NumericFormat::default()
                    .with_suffix(" u")
                    .unwrap(),
            );
        if exact {
            plot = plot.x_ticks(vec![
                crate::AxisTick::new(1.0, "one exact"),
                crate::AxisTick::new(9.0, ""),
            ]);
        }
        let canvas = crate::Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, plot.clone())
            .unwrap()
            .panel(0, 1, plot.title("Interval source identity"))
            .unwrap()
            .width(1000);
        let mut outputs = Vec::new();
        for disabled in [false, true] {
            crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
            let mut backend = crate::render::svg_backend::SvgBackend::in_memory(1000, 480);
            let result = canvas.render_with_backend(&mut backend);
            crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
            result.unwrap();
            outputs.push(backend.into_bytes().unwrap());
        }
        assert_eq!(outputs[0], outputs[1]);
        let output = String::from_utf8(outputs.remove(0)).unwrap();
        assert_eq!(output.contains("one exact"), exact);
    }
}

#[test]
fn forest_log_interval_is_applied_in_exponent_space_and_exact_ticks_override_it() {
    let mut forest = Plot::forest(
        crate::CategoryValues::new("term", ["a", "b"]).unwrap(),
        crate::NumericValues::new("estimate", [0.5, 2.0]),
        crate::Bounds::new(
            crate::NumericValues::new("lower", [0.1, 1.0]),
            crate::NumericValues::new("upper", [1.0, 10.0]),
        ),
        1.0,
    )
    .scale(AxisScale::Log10)
    .x_limits(0.01, 100.0)
    .x_tick_generation(TickGeneration::log10_decades(2).unwrap());
    for exact in [false, true] {
        if exact {
            forest = forest.x_ticks(vec![crate::AxisTick::new(1.0, "raw exact")]);
        }
        let mut outputs = Vec::new();
        for disabled in [false, true] {
            let canvas = crate::Canvas::grid(1, 1)
                .unwrap()
                .panel(0, 0, forest.clone())
                .unwrap();
            crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
            let mut backend = crate::render::svg_backend::SvgBackend::in_memory(640, 480);
            crate::orchestrator::measurements::start();
            let result = canvas.render_with_backend(&mut backend);
            let work = crate::orchestrator::measurements::finish();
            crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
            result.unwrap_or_else(|error| {
                panic!(
                    "exact={exact}, disabled={disabled}: {error}; roots={:?}; gutters={:?}",
                    work.roots, work.gutters
                )
            });
            for counter in [
                crate::orchestrator::measurements::Counter::TickSource,
                crate::orchestrator::measurements::Counter::TickDistinctList,
                crate::orchestrator::measurements::Counter::TickMetricLookup,
            ] {
                assert_eq!(
                    work.capture_counters[counter as usize], 0,
                    "exact={exact}, disabled={disabled}: {counter:?}"
                );
            }
            outputs.push(backend.into_bytes().unwrap());
        }
        assert_eq!(outputs[0], outputs[1]);
        let svg = std::str::from_utf8(&outputs[0]).unwrap();
        if exact {
            assert!(svg.contains("raw exact"));
            assert!(!svg.contains(">0.01</text>"));
        } else {
            // Automatic width may coarsen the complete two-decade preference.
            // Its integral coarsenings retain exponent zero, never odd decades.
            assert!(svg.contains(">1</text>"));
            assert!(!svg.contains(">0.1</text>"));
            assert!(!svg.contains(">10</text>"));
        }
    }
}
