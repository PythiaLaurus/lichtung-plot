use super::*;
use crate::layout::fit;
use crate::orchestrator::assignment::PlotAllocationInputs;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one Root-driven capacity contract checks outside discovery reuse, inside final remeasurement and the original feedback stream"
)]
fn local_outside_width_survives_reuse_and_inside_remeasure_and_reaches_feedback() {
    for position in [LegendPosition::OutsideRight, LegendPosition::InsideTopRight] {
        let source = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [1.0, 2.0]),
        )
        .hue(crate::CategoryValues::new("group", ["a", "b"]).unwrap())
        .x_limits(0.0, 10.0)
        .y_limits(0.0, 10.0)
        .legend(LegendPolicy::Visible(position))
        .erase();
        let mut saw_reuse = false;
        let mut saw_remeasure = false;
        let frozen = crate::layout::plan_root(
            crate::layout::RootRequest::ExactSize {
                width: 1600,
                height: 1000,
            },
            |stage| {
                crate::orchestrator::root::Root::prepare(
                    crate::orchestrator::root::RootSource::Plot(source.description()),
                    stage,
                )
            },
            |root, assigned, round| {
                let size = assigned.assigned_size()?;
                let measurement = root.assignment_factory().assign(size);
                let registered = &root.plots[0];
                let factory = root.plot_assignment_factory(&measurement, &registered.scope.id)?;
                let local = registered
                    .local_guide_slot()?
                    .expect("the actual two-group descriptor is present");
                let entry = factory.assign(
                    PlotAllocationInputs::for_test(
                        size,
                        PixelRect::new(100.0, 1300.0, 100.0, 900.0),
                        round,
                        &registered.scope.address,
                    ),
                    round,
                    None,
                )?;
                let before =
                    crate::orchestrator::SELECTED_INTERIOR_MEASUREMENTS.with(std::cell::Cell::get);
                let planned = fit::measured(plan_plot(
                    &factory,
                    entry,
                    LegendRole::Panel,
                    GuideConsumption {
                        use_kind: crate::decorations::legend::discrete::scoped::GuideUse::Local,
                        local_guide: Some(&local),
                        dependencies: &[],
                    },
                    &FitScope::new(&registered.scope.address, 0),
                    round,
                ))
                .map_err(crate::layout::fit::PlanError::into_error)?;
                let measurements = crate::orchestrator::SELECTED_INTERIOR_MEASUREMENTS
                    .with(std::cell::Cell::get)
                    - before;
                let resolved = planned.legend.as_ref().expect("visible local guide");
                if is_outside(position) {
                    let Some(GuideAllocation::Width(width)) = planned.selected.guide.allocation
                    else {
                        panic!("a successful outside plan must retain its measured capacity");
                    };
                    assert_eq!(width, resolved.legend.available_width());
                    let capacity_pixels = f64::from(u32::try_from(width).unwrap()) / 1024.0;
                    assert!(capacity_pixels > resolved.legend.compute_box_width());
                    // Outside performs the original preliminary discovery and
                    // one assigned-body measurement; no final remeasurement.
                    assert_eq!(measurements, 2);
                    saw_reuse = true;

                    // Compare the existing opaque observation histories, not a
                    // copied legend observation program or a coordinate golden.
                    let start = round.child_checkpoint();
                    resolved.observe_feedback(round, width)?;
                    let original = round.retain_child_feedback(start)?.unwrap();
                    let start = round.child_checkpoint();
                    resolved.observe_feedback(round, width + 1)?;
                    let changed = round.retain_child_feedback(start)?.unwrap();
                    assert_ne!(original, changed);
                } else {
                    assert!(planned.selected.guide.allocation.is_none());
                    // Inside bypasses preliminary discovery. Its second actual
                    // measurement is the final branch after consumed gutters.
                    assert!(matches!(measurements, 1 | 2));
                    saw_reuse |= measurements == 1;
                    saw_remeasure |= measurements == 2;
                }
                Ok(crate::layout::MeasuredLayout::new(size, planned))
            },
            |_, _, planned, _| Ok(planned),
        )
        .unwrap();
        let selected = frozen.plan();
        if is_outside(position) {
            assert!(saw_reuse);
            assert_eq!(
                selected.selected.guide.allocation,
                Some(GuideAllocation::Width(
                    selected.legend.as_ref().unwrap().legend.available_width()
                ))
            );
        } else {
            assert!(saw_remeasure, "the final remeasurement branch must execute");
            assert!(selected.selected.guide.allocation.is_none());
        }
    }
}
