use super::*;
use crate::decorations::legend::discrete::scoped::SUPPRESSED_GUIDES;
use crate::decorations::plan::children::MeasurementPhase;
use crate::layout::fit::FitScope;
use crate::orchestrator::assignment::{
    GuideTrialAssignment, ParentInsets, PlotAllocationInputs, PlotMeasureDimensions,
};
use crate::orchestrator::root::{Root, RootSource};
use crate::TickGeneration;

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
