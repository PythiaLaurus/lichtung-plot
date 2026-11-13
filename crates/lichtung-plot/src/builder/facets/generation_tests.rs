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
fn composed_conditions_borrow_the_normalized_parent_axis_and_preserve_raw_order_identity() {
    let source = Plot::trajectory(
        crate::CategoryValues::new(
            "condition",
            [
                " after  visit ",
                "before visit",
                "last visit",
                "before  visit",
                "after visit",
                "last visit",
            ],
        )
        .unwrap(),
        crate::NumericValues::new("value", [2.0, 1.0, 3.0, 4.0, 5.0, 6.0]),
        crate::CategoryValues::new("group", ["g"; 6]).unwrap(),
    )
    .facet(FacetSpec::column(
        crate::CategoryValues::new("panel", ["a", "a", "a", "b", "b", "b"]).unwrap(),
    ))
    .erase();
    let operation = crate::font::canonical::OperationContext::new();
    let mut inputs = crate::plots::shared_x::prepare_plot(&source, &operation, None).unwrap();
    let plot = source
        .clone()
        .without_facets()
        .with_render_scope(inputs.plots[0].scope.render_name.clone().unwrap());
    let registered = &inputs.plots[0];
    let retained = plot
        .retain_composed_inputs(ScopedGuideInputs::for_test(&plot), Some(registered), None)
        .unwrap();
    assert_eq!(
        retained.global_line_condition_levels.as_deref().unwrap(),
        ["after visit", "before visit", "last visit"]
    );
    assert!(plot.same_composed_inputs(&retained, &retained.guides, Some(registered), None));
    inputs.plots[0].axes.sources_mut()[0]
        .categories
        .as_mut()
        .unwrap()
        .raw
        .swap(0, 1);
    assert!(!plot.same_composed_inputs(&retained, &retained.guides, Some(&inputs.plots[0]), None));
    assert!(crate::orchestrator::registration::line_condition_levels(None).is_none());
}

#[test]
fn composed_groups_preserve_parent_order_without_exposing_local_empty_orders() {
    let source = Plot::bar(
        crate::CategoryValues::new("category", ["one", "two", "one", "two"]).unwrap(),
        crate::BarInput::estimates(crate::NumericValues::new("value", [9.0, 8.0, 1.0, 2.0])),
    )
    .series(crate::CategoryValues::new("cohort", ["absent", "absent", "seen", "seen"]).unwrap())
    .facet(FacetSpec::column(
        crate::CategoryValues::new("panel", ["parent", "parent", "child", "child"]).unwrap(),
    ))
    .erase();
    let operation = crate::font::canonical::OperationContext::new();
    let mut inputs = crate::plots::shared_x::prepare_plot(&source, &operation, None).unwrap();
    let plot = source
        .clone()
        .without_facets()
        .with_render_scope(inputs.plots[1].scope.render_name.clone().unwrap());
    let registered = &inputs.plots[1];
    let crate::orchestrator::registration::PreparedFamily::Bars { marks, .. } = &registered.family
    else {
        panic!("prepared Bars")
    };
    assert!(marks
        .bars
        .iter()
        .all(|bar| bar.series_index == Some(1) && bar.hue_index == Some(1)));
    let retained = plot
        .retain_composed_inputs(ScopedGuideInputs::for_test(&plot), Some(registered), None)
        .unwrap();
    assert_eq!(
        retained.global_group_levels.as_deref().unwrap(),
        ["absent", "seen"]
    );
    assert!(plot.same_composed_inputs(&retained, &retained.guides, Some(registered), None));
    inputs.plots[1].policy.coordinated.groups =
        Some(crate::data::source::source_arc(vec!["seen".into(), "absent".into()]).unwrap());
    assert!(!plot.same_composed_inputs(&retained, &retained.guides, Some(&inputs.plots[1]), None));

    let single = Plot::bar(
        crate::CategoryValues::new("category", ["one"]).unwrap(),
        crate::BarInput::estimates(crate::NumericValues::new("value", [1.0])),
    )
    .erase();
    let inputs = crate::plots::shared_x::prepare_plot(&single, &operation, None).unwrap();
    assert!(inputs.plots[0]
        .policy
        .coordinated
        .groups
        .as_ref()
        .unwrap()
        .is_empty());
    let retained = single
        .retain_composed_inputs(
            ScopedGuideInputs::for_test(&single),
            Some(&inputs.plots[0]),
            None,
        )
        .unwrap();
    assert!(retained.global_group_levels.is_none());
}

#[test]
fn generation_cloning_and_composed_identity_include_both_axes() {
    let mut plot = plot()
        .x_tick_generation(TickGeneration::additive(2.0).unwrap())
        .y_tick_generation(TickGeneration::additive(1.0).unwrap())
        .x_tick_format(
            crate::numeric::format::NumericFormat::default()
                .prefix("x ")
                .unwrap(),
        )
        .y_tick_format(
            crate::numeric::format::NumericFormat::default()
                .suffix(" y")
                .unwrap(),
        )
        .erase();

    let retained = plot
        .retain_composed_inputs(ScopedGuideInputs::for_test(&plot), None, None)
        .unwrap();
    assert_eq!(
        retained.x_axis_spec.2,
        plot.description().x_axis_spec().generation
    );
    assert_eq!(
        retained.y_axis_spec.2,
        plot.description().y_axis_spec().generation
    );
    assert_eq!(
        retained.x_axis_spec.4,
        plot.description().x_axis_spec().format
    );
    assert_eq!(
        retained.y_axis_spec.4,
        plot.description().y_axis_spec().format
    );
    assert!(plot.same_composed_inputs(&retained, &retained.guides, None, None));
    plot.description.payload_mut().axes[0].generation = TickGeneration::additive(3.0).unwrap();
    assert!(!plot.same_composed_inputs(&retained, &retained.guides, None, None));
    plot.description.payload_mut().axes[0]
        .generation
        .clone_from(&retained.x_axis_spec.2);
    plot.description.payload_mut().axes[1].generation = TickGeneration::automatic();
    assert!(!plot.same_composed_inputs(&retained, &retained.guides, None, None));
    plot.description.payload_mut().axes[1]
        .generation
        .clone_from(&retained.y_axis_spec.2);
    for selection in [
        TickGeneration::target(5).unwrap(),
        TickGeneration::maximum(5).unwrap(),
    ] {
        let mut changed = plot.clone();
        changed.description.payload_mut().axes[0].generation = selection.clone();
        assert!(!changed.same_composed_inputs(&retained, &retained.guides, None, None));
        let counted = changed
            .retain_composed_inputs(ScopedGuideInputs::for_test(&changed), None, None)
            .unwrap();
        assert!(changed.same_composed_inputs(&counted, &counted.guides, None, None));
        changed.description.payload_mut().axes[0].generation = if selection.target_count().is_some()
        {
            TickGeneration::maximum(5).unwrap()
        } else {
            TickGeneration::target(5).unwrap()
        };
        assert!(!changed.same_composed_inputs(&counted, &counted.guides, None, None));
    }
    for side in 0..2 {
        let mut changed = plot.clone();
        if side == 0 {
            changed.description.payload_mut().axes[0].format = None;
        } else {
            changed.description.payload_mut().axes[1].format = None;
        }
        assert!(!changed.same_composed_inputs(&retained, &retained.guides, None, None));
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
        .retain_composed_inputs(ScopedGuideInputs::for_test(&automatic), None, None)
        .unwrap();
    assert_eq!(
        automatic.description().x_axis_spec().window,
        configured.description().x_axis_spec().window
    );
    assert_eq!(
        automatic.description().x_axis_spec().window,
        shared.description().x_axis_spec().window
    );
    assert!(automatic.same_composed_inputs(&retained, &retained.guides, None, None));
    assert!(!configured.same_composed_inputs(&retained, &retained.guides, None, None));
    assert!(!shared.same_composed_inputs(&retained, &retained.guides, None, None));
    let shared_retained = shared
        .retain_composed_inputs(ScopedGuideInputs::for_test(&shared), None, None)
        .unwrap();
    assert!(shared.same_composed_inputs(&shared_retained, &shared_retained.guides, None, None));
    assert_eq!(
        shared_retained.x_axis_spec.1,
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
        assert!(crate::TickGeneration::interval(
            step,
            0.0,
            crate::TickSpace::Data,
            crate::TickRequirement::Fixed
        )
        .is_err());
    }
    assert!(crate::TickGeneration::interval(
        1.0,
        f64::NAN,
        crate::TickSpace::Data,
        crate::TickRequirement::Fixed
    )
    .is_err());
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
        .font_stack(
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
    let numeric = Plot::trajectory(
        crate::NumericValues::new("time", [0.0, 1.0]),
        crate::NumericValues::new("value", [1.0, 2.0]),
        crate::CategoryValues::new("group", ["a", "a"]).unwrap(),
    )
    .x_category_labels(crate::CategoryLabelSelection::every(2).unwrap());
    assert!(matches!(numeric.preflight(), Err(crate::Error::Data(_))));
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
                    .suffix(" u")
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
