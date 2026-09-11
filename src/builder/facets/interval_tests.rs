use super::*;
use crate::layout::TickInterval;

fn plot() -> Plot<crate::kind::Scatter> {
    Plot::scatter(
        crate::NumericValues::new("x", [0.0, 5.0, 10.0]),
        crate::NumericValues::new("y", [1.0, 3.0, 2.0]),
    )
    .x_limits(0.0, 10.0)
    .y_limits(0.0, 4.0)
}

#[test]
fn interval_copy_and_composed_identity_include_both_axes() {
    let mut plot = plot();
    plot.x_axis_spec.interval = Some(TickInterval::Additive(2.0));
    plot.y_axis_spec.interval = Some(TickInterval::Additive(1.0));
    let theme = crate::Theme::Publication.resolve();
    let retained = plot
        .retain_composed_inputs(theme.prepared_font_environment().unwrap())
        .unwrap();
    assert_eq!(retained.x_axis_spec.interval, plot.x_axis_spec.interval);
    assert_eq!(retained.y_axis_spec.interval, plot.y_axis_spec.interval);
    assert!(plot.same_composed_inputs(&retained));
    plot.x_axis_spec.interval = Some(TickInterval::Additive(3.0));
    assert!(!plot.same_composed_inputs(&retained));
    plot.x_axis_spec.interval = retained.x_axis_spec.interval;
    plot.y_axis_spec.interval = None;
    assert!(!plot.same_composed_inputs(&retained));
}

#[test]
fn equal_composed_endpoints_do_not_erase_range_provenance() {
    let theme = crate::Theme::Publication.resolve();
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
        .retain_composed_inputs(theme.prepared_font_environment().unwrap())
        .unwrap();
    assert_eq!(automatic.x_axis_spec.limits, configured.x_axis_spec.limits);
    assert_eq!(automatic.x_axis_spec.limits, shared.x_axis_spec.limits);
    assert!(automatic.same_composed_inputs(&retained));
    assert!(!configured.same_composed_inputs(&retained));
    assert!(!shared.same_composed_inputs(&retained));
    let shared_retained = shared
        .retain_composed_inputs(theme.prepared_font_environment().unwrap())
        .unwrap();
    assert!(shared.same_composed_inputs(&shared_retained));
    assert_eq!(
        shared_retained.x_axis_spec.provenance,
        crate::layout::RangeProvenance::Shared
    );
}

#[test]
fn invalid_intervals_reject_before_font_discovery() {
    for interval in [
        TickInterval::Additive(0.0),
        TickInterval::Additive(-1.0),
        TickInterval::Additive(f64::NAN),
        TickInterval::Additive(f64::INFINITY),
        TickInterval::Log10Decades(0),
        TickInterval::Log10Decades(2),
    ] {
        let mut plot = plot();
        plot.x_axis_spec.interval = Some(interval);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        assert!(matches!(plot.preflight(), Err(crate::Error::Data(_))));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}

#[test]
fn interval_census_uses_the_source_alphabet_and_exact_precedence_suppresses_it() {
    let theme = crate::Theme::Publication.resolve();
    let mut plot = plot().erase();
    plot.x_axis_spec.interval = Some(TickInterval::Additive(0.01));
    plot.y_axis_spec.interval = Some(TickInterval::Additive(0.1));
    for exact in [false, true] {
        if exact {
            plot.x_axis_spec.exact_ticks = Some(vec![crate::AxisTick::new(1.0, "")]);
        }
        let census = crate::layout::foundational_plot_requirements(
            &plot,
            PlotKind::Scatter,
            &theme,
            &crate::layout::ComponentAddress::root(),
            true,
            true,
            true,
            None,
        )
        .unwrap();
        let automatic = census
            .iter()
            .filter(|record| {
                record.text_role == crate::layout::PreparedTextRole::TickLabel
                    && record.generated_alphabet.as_ref() == crate::layout::NUMERIC_ALPHABET
            })
            .count();
        assert_eq!(automatic, if exact { 1 } else { 2 });
    }
}

#[test]
fn interval_rendering_and_exact_precedence_are_cache_independent() {
    for exact in [false, true] {
        let mut plot = plot();
        plot.x_axis_spec.interval = Some(TickInterval::Additive(2.0));
        plot.y_axis_spec.interval = Some(TickInterval::Additive(1.0));
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
            .size(1000, 480);
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
    .x_limits(0.01, 100.0);
    forest.x_axis_spec.interval = Some(TickInterval::Log10Decades(2));
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
            result.unwrap();
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
            assert!(
                svg.contains(">0.01</text>")
                    && svg.contains(">1</text>")
                    && svg.contains(">100</text>")
            );
            assert!(!svg.contains(">0.1</text>"));
        }
    }
}
