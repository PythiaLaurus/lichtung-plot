use super::*;
use crate::decorations::legend::discrete::scoped::SUPPRESSED_GUIDES;
use crate::decorations::plan::children::MeasurementPhase;
use crate::layout::fit::FitScope;
use crate::orchestrator::assignment::{
    GuideTrialAssignment, ParentInsets, PlotAllocationInputs, PlotMeasureDimensions,
};
use crate::orchestrator::root::{Root, RootSource};
use crate::TickGeneration;

fn plot() -> Plot<crate::kind::Scatter> {
    Plot::scatter(
        crate::NumericValues::new("x", [0.0, 5.0, 10.0]),
        crate::NumericValues::new("y", [1.0, 3.0, 2.0]),
    )
    .x_limits(0.0, 10.0)
    .y_limits(0.0, 4.0)
}

fn with_composed<T>(
    source: &Plot,
    slot: usize,
    full: bool,
    edit: impl FnOnce(&mut Root<'_>),
    inspect: impl FnOnce(&PlotMeasureContext<'_, '_>, &FeedbackRound) -> T,
) -> T {
    let mut result = None;
    let mut inspect = Some(inspect);
    crate::layout::plan_root(
        crate::layout::RootRequest::ExactSize {
            width: 800,
            height: 600,
        },
        |stage| {
            let mut root = Root::prepare(RootSource::Plot(source.description()), stage)?;
            edit(&mut root);
            Ok(root)
        },
        |root, assigned, round| {
            let registered = &root.plots[slot];
            let measurement = root.assignment_factory().assign(assigned.assigned_size()?);
            let factory = root.plot_assignment_factory(&measurement, &registered.scope.id)?;
            let panel = crate::layout::PixelRect::new(20.0, 700.0, 20.0, 500.0);
            let entry = factory.assign(
                PlotAllocationInputs {
                    dimensions: PlotMeasureDimensions {
                        canvas: measurement.measurement_size,
                        output_override: None,
                    },
                    cell: panel,
                    frame: panel,
                    body: panel,
                    axis_label_width: panel.width(),
                    content_height: false,
                    fill_content_height: false,
                    content_preference: None,
                    insets: ParentInsets::consumed(round, &registered.scope.address),
                    guide: GuideTrialAssignment {
                        edge: crate::LegendPosition::Auto,
                        compact: false,
                        allocation: None,
                    },
                },
                round,
                (!full).then_some(MeasurementPhase::Probe),
            )?;
            let scope = if full {
                FitScope::new(&registered.scope.address, 0)
            } else {
                FitScope::root()
            };
            let context = factory.context(&entry, SUPPRESSED_GUIDES, &scope)?;
            result = Some(inspect.take().unwrap()(&context, round));
            Ok(crate::layout::MeasuredLayout::new(
                measurement.measurement_size,
                (),
            ))
        },
        |_, _, (), _| Ok(()),
    )
    .unwrap();
    result.unwrap()
}

fn retain_key(source: &Plot) -> ComposedInputs {
    with_composed(
        source,
        0,
        true,
        |_| {},
        |context, round| {
            ComposedInputs::retain(
                context,
                round,
                ScopedGuideInputs::capture(context).unwrap(),
                None,
            )
            .unwrap()
        },
    )
}

fn key_matches(source: &Plot, retained: &ComposedInputs) -> bool {
    with_composed(
        source,
        0,
        true,
        |_| {},
        |context, round| retained.matches(context, round, &retained.guides, None),
    )
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
    let retained = retain_key(&source);
    assert_eq!(
        retained.global_line_condition_levels.as_deref().unwrap(),
        ["after visit", "before visit", "last visit"]
    );
    assert!(key_matches(&source, &retained));
    assert!(!with_composed(
        &source,
        0,
        true,
        |root| {
            root.plots[0].axes.sources_mut()[0]
                .categories
                .as_mut()
                .unwrap()
                .raw
                .swap(0, 1);
        },
        |context, round| retained.matches(context, round, &retained.guides, None)
    ));
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
    let retained = with_composed(
        &source,
        1,
        true,
        |_| {},
        |context, round| {
            let crate::orchestrator::registration::PreparedFamily::Bars { marks, .. } =
                &context.registered.family
            else {
                panic!("prepared Bars")
            };
            assert!(marks.bars.iter().all(|bar| bar.series_index == Some(1)
                && bar.hue_key.as_ref().is_some_and(|key| {
                    let raw = key.as_ref();
                    raw.field == "cohort" && raw.value == "seen"
                })));
            ComposedInputs::retain(
                context,
                round,
                ScopedGuideInputs::capture(context).unwrap(),
                None,
            )
            .unwrap()
        },
    );
    assert_eq!(
        retained.global_group_levels.as_deref().unwrap(),
        ["absent", "seen"]
    );
    assert_eq!(
        retained.global_hue_levels.as_deref().unwrap(),
        ["absent", "seen"]
    );
    for (values, expected) in [
        (vec!["absent".into(), "seen".into()], true),
        (vec!["seen".into(), "absent".into()], false),
    ] {
        assert_eq!(
            with_composed(
                &source,
                1,
                true,
                |root| {
                    let panel = &mut root.plots[1];
                    let original = panel.parent_orders.hues.as_ref().unwrap();
                    let replacement = crate::data::source::source_arc(
                        crate::data::category::CategoryDomain::from_values(
                            source.description().hue_field().as_ref(),
                            values,
                        )
                        .unwrap(),
                    )
                    .unwrap();
                    assert!(!std::sync::Arc::ptr_eq(original, &replacement));
                    panel.parent_orders.hues = Some(std::sync::Arc::clone(&replacement));
                    panel.categorical.as_mut().unwrap().parent = Some(replacement);
                },
                |context, round| retained.matches(context, round, &retained.guides, None)
            ),
            expected
        );
    }
    assert!(!with_composed(
        &source,
        1,
        true,
        |root| {
            root.plots[1].parent_orders.groups = Some(
                crate::data::source::source_arc(vec!["seen".into(), "absent".into()]).unwrap(),
            );
        },
        |context, round| retained.matches(context, round, &retained.guides, None)
    ));
    let single = Plot::bar(
        crate::CategoryValues::new("category", ["one"]).unwrap(),
        crate::BarInput::estimates(crate::NumericValues::new("value", [1.0])),
    )
    .erase();
    assert!(retain_key(&single).global_group_levels.is_none());
}

#[test]
fn generation_cloning_and_composed_identity_include_both_axes() {
    let mut source = plot()
        .x_tick_generation(TickGeneration::additive(2.0).unwrap())
        .y_tick_generation(TickGeneration::additive(1.0).unwrap())
        .x_tick_format(crate::NumericFormat::default().prefix("x ").unwrap())
        .y_tick_format(crate::NumericFormat::default().suffix(" y").unwrap())
        .erase();
    let retained = retain_key(&source);
    assert_eq!(
        retained.x_axis_spec.2,
        source.description().x_axis_spec().generation
    );
    assert_eq!(
        retained.y_axis_spec.2,
        source.description().y_axis_spec().generation
    );
    assert_eq!(
        retained.x_axis_spec.4,
        source.description().x_axis_spec().format
    );
    assert_eq!(
        retained.y_axis_spec.4,
        source.description().y_axis_spec().format
    );
    assert!(key_matches(&source, &retained));
    source.description.payload_mut().axes[0].generation = TickGeneration::additive(3.0).unwrap();
    assert!(!key_matches(&source, &retained));
    source.description.payload_mut().axes[0]
        .generation
        .clone_from(&retained.x_axis_spec.2);
    source.description.payload_mut().axes[1].generation = TickGeneration::automatic();
    assert!(!key_matches(&source, &retained));
    source.description.payload_mut().axes[1]
        .generation
        .clone_from(&retained.y_axis_spec.2);
    for selection in [
        TickGeneration::target(5).unwrap(),
        TickGeneration::maximum(5).unwrap(),
    ] {
        let mut changed = source.clone();
        changed.description.payload_mut().axes[0].generation = selection.clone();
        assert!(!key_matches(&changed, &retained));
        let counted = retain_key(&changed);
        assert!(key_matches(&changed, &counted));
        changed.description.payload_mut().axes[0].generation = if selection.target_count().is_some()
        {
            TickGeneration::maximum(5).unwrap()
        } else {
            TickGeneration::target(5).unwrap()
        };
        assert!(!key_matches(&changed, &counted));
    }
    for side in 0..2 {
        let mut changed = source.clone();
        changed.description.payload_mut().axes[side].format = None;
        assert!(!key_matches(&changed, &retained));
    }
}

#[test]
fn equal_composed_endpoints_do_not_erase_range_provenance() {
    let make = || {
        Plot::scatter(
            crate::NumericValues::new("x", [0.0, 10.0]),
            crate::NumericValues::new("y", [0.0, 4.0]),
        )
        .facet(FacetSpec::column(
            crate::CategoryValues::new("panel", ["one"; 2]).unwrap(),
        ))
    };
    let source = make().erase();
    let retained = retain_key(&source);
    assert_eq!(retained.x_axis_spec.1, RangeProvenance::Shared);
    assert!(key_matches(&source, &retained));
    let x = retained.x_axis_spec.0.unwrap();
    let y = retained.y_axis_spec.0.unwrap();
    let configured = make().x_limits(x.0, x.1).y_limits(y.0, y.1).erase();
    let configured_key = retain_key(&configured);
    assert_eq!(configured_key.x_axis_spec.0, retained.x_axis_spec.0);
    assert_eq!(configured_key.x_axis_spec.1, RangeProvenance::Configured);
    assert!(!key_matches(&configured, &retained));
    with_composed(
        &source,
        0,
        false,
        |_| {},
        |context, round| {
            let preflight = ComposedInputs::retain(
                context,
                round,
                ScopedGuideInputs::capture(context).unwrap(),
                Some(MeasurementPhase::Probe),
            )
            .unwrap();
            assert_eq!(preflight.x_axis_spec.1, RangeProvenance::Automatic);
            assert!(preflight.x_axis_spec.0.is_none());
            assert!(!retained.matches(
                context,
                round,
                &retained.guides,
                Some(MeasurementPhase::Probe),
            ));
        },
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
            crate::TickRequirement::Fixed,
        )
        .is_err());
    }
    assert!(crate::TickGeneration::interval(
        1.0,
        f64::NAN,
        crate::TickSpace::Data,
        crate::TickRequirement::Fixed,
    )
    .is_err());
    assert!(TickGeneration::log10_decades(0).is_err());
    let source = plot().x_tick_generation(TickGeneration::log10_decades(2).unwrap());
    assert!(matches!(source.preflight(), Err(crate::Error::Data(_))));
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
        let mut source = plot()
            .x_tick_generation(TickGeneration::additive(2.0).unwrap())
            .y_tick_generation(TickGeneration::additive(1.0).unwrap())
            .x_tick_format(crate::NumericFormat::default().suffix(" u").unwrap());
        if exact {
            source = source.x_ticks(vec![
                crate::AxisTick::new(1.0, "one exact"),
                crate::AxisTick::new(9.0, ""),
            ]);
        }
        let canvas = crate::Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, source.clone())
            .unwrap()
            .panel(0, 1, source.title("Interval source identity"))
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
