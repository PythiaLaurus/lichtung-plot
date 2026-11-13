use super::*;
use crate::orchestrator::registration::{PlotRegistryBuilder, PreparedFamily, RowSelection, Scope};

#[test]
fn central_axes_keep_actual_occurrence_context_and_later_fatal_over_an_earlier_fit() {
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
    let appearance = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let request = crate::text::TextRequestContext::new(&appearance, &text);
    crate::orchestrator::with_prepared_plot_for_test(
        &source,
        &request,
        |plot, inputs, request, operation| {
            let registered = &inputs.plots[0];
            let error = plot
                .measure_selected_interior(
                    crate::decorations::legend::discrete::scoped::GuideUse::Local,
                    PlotRenderContext::new(
                        registered.data(operation).unwrap(),
                        PixelRect::new(0.0, 800.0, 0.0, 600.0),
                        request,
                    )
                    .with_registration(registered, inputs.intrinsic(registered)),
                )
                .unwrap_err();
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
        },
    )
    .unwrap();
}

#[test]
fn shared_initial_axis_seed_drives_the_original_curve_finish_before_fonts() {
    let source = Plot::marginal(
        crate::NumericValues::new("x", [1.0, 2.0, 3.0]),
        crate::NumericValues::new("y", [2.0, 4.0, 3.0]),
    )
    .erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    crate::layout::plan_root(
        crate::layout::RootRequest::ExactSize {
            width: 1,
            height: 1,
        },
        |stage| {
            let operation = stage.operation();
            let scope = Scope::new(
                &source,
                &operation,
                0,
                crate::layout::ComponentAddress::root()
                    .child(crate::layout::AddressRole::PlotBody, 0)?,
                RowSelection::Full,
                None,
                None,
            )?;
            let mut registry = PlotRegistryBuilder::new(operation.root_identity());
            let id = registry.reserve(source.description(), scope, &operation)?;
            let panel = &mut registry.panels[id.slot];
            crate::plots::shared_x::prepare_plot_registration(panel, &source, None, &operation)?;
            let axes = panel.axes.as_ref().unwrap();
            let initial_y = axes.ranges()[1];
            assert!(axes
                .sources()
                .iter()
                .all(|source| source.label.get().is_none()));
            let Some(PreparedFamily::Marginal(marks)) = &panel.family else {
                panic!("prepared Marginal")
            };
            assert!(marks.curves.x_grid.is_empty() && marks.curves.y_grid.is_empty());

            let shared = (-10.0, 10.0);
            crate::plots::shared_x::prepare_reference_plot(
                &source,
                panel,
                Some(shared),
                false,
                &operation,
            )?;
            let axes = panel.axes.as_ref().unwrap();
            assert_eq!(axes.ranges(), [shared, initial_y]);
            assert_eq!(
                axes.sources()[0].base_provenance,
                layout::RangeProvenance::Shared
            );
            assert!(axes
                .sources()
                .iter()
                .all(|source| source.label.get().is_some()));
            let Some(PreparedFamily::Marginal(marks)) = &panel.family else {
                panic!("finished Marginal")
            };
            assert_eq!(marks.curves.x_grid.len(), 80);
            assert_eq!(marks.curves.y_grid.len(), 80);
            assert_eq!(marks.curves.x_grid.first(), Some(&shared.0));
            assert_eq!(marks.curves.x_grid.last(), Some(&shared.1));
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
            Ok(())
        },
        |(), assigned, _| {
            Ok(crate::layout::MeasuredLayout::new(
                assigned.assigned_size()?,
                (),
            ))
        },
        |(), _, (), _| Ok(()),
    )
    .unwrap();
}
