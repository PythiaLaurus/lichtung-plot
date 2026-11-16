use super::*;

#[test]
fn central_axes_keep_actual_occurrence_context_and_later_fatal_over_an_earlier_fit() {
    use crate::orchestrator::assignment::{
        GuideTrialAssignment, ParentInsets, PlotAllocationInputs, PlotMeasureDimensions,
    };
    use std::sync::{Arc, Mutex};

    let actual = Arc::new(Mutex::new(Vec::new()));
    let x_calls = Arc::clone(&actual);
    let y_calls = Arc::clone(&actual);
    let source = Plot::marginal(
        crate::NumericValues::new("x", [1.0, 2.0, 3.0]),
        crate::NumericValues::new("y", [2.0, 4.0, 3.0]),
    )
    .x_limits(0.0, 4.0)
    .y_limits(0.0, 5.0)
    .x_tick_generation(crate::TickGeneration::exact_count(
        1,
        crate::TickSpace::Data,
    ))
    .x_tick_format(
        crate::NumericFormat::default().set_function(move |context| {
            if context.purpose() == crate::NumericPurpose::IntrinsicProbe {
                return context.format_builtin();
            }
            assert_eq!(context.role(), crate::NumericRole::XAxis);
            assert_eq!(context.domain().unwrap().data_limits(), Some((0.0, 4.0)));
            assert_eq!(context.values().len(), 1);
            x_calls.lock().unwrap().push("x");
            Ok(crate::NumericLabels::new(vec!["W".repeat(200)]))
        }),
    )
    .y_tick_format(
        crate::NumericFormat::default().set_function(move |context| {
            if context.purpose() == crate::NumericPurpose::IntrinsicProbe {
                return context.format_builtin();
            }
            assert_eq!(context.role(), crate::NumericRole::YAxis);
            assert_eq!(context.domain().unwrap().data_limits(), Some((0.0, 5.0)));
            y_calls.lock().unwrap().push("y");
            Err(Error::Data("Marginal later y-axis callback failure".into()))
        }),
    )
    .erase();
    let mut witnessed = false;
    crate::layout::plan_root(
        crate::layout::RootRequest::ExactSize {
            width: 1000,
            height: 800,
        },
        |stage| {
            crate::orchestrator::root::Root::prepare(
                crate::orchestrator::root::RootSource::Plot(source.description()),
                stage,
            )
        },
        |root, assigned, round| {
            let size = assigned.assigned_size()?;
            if witnessed {
                return Ok(crate::layout::MeasuredLayout::new(size, ()));
            }
            let registered = &root.plots[0];
            let measurement = root.assignment_factory().assign(size);
            let factory = root.plot_assignment_factory(&measurement, &registered.scope.id)?;
            let panel = PixelRect::new(0.0, 800.0, 0.0, 600.0);
            let entry = factory.assign(
                PlotAllocationInputs {
                    dimensions: PlotMeasureDimensions {
                        canvas: size,
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
                None,
            )?;
            let scope = crate::layout::fit::FitScope::new(&registered.scope.address, 0);
            let local = registered.local_guide_slot()?;
            let context = factory.context(
                &entry,
                crate::decorations::legend::discrete::scoped::GuideConsumption {
                    use_kind: crate::decorations::legend::discrete::scoped::GuideUse::Local,
                    local_guide: local.as_ref(),
                    dependencies: &[],
                },
                &scope,
            )?;
            let error = crate::layout::interior::measure(&context, &context.request()).unwrap_err();
            let PlanError::Fatal(error) = error else {
                panic!("later y-axis failure must outrank x-axis fit: {error:?}")
            };
            assert!(
                error
                    .to_string()
                    .contains("Marginal later y-axis callback failure"),
                "{error}"
            );
            assert_eq!(*actual.lock().unwrap(), ["x", "y"]);
            witnessed = true;
            Ok(crate::layout::MeasuredLayout::new(size, ()))
        },
        |_, _, (), _| Ok(()),
    )
    .unwrap();
    assert!(witnessed);
}
