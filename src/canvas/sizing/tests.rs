use super::*;
use crate::decorations::legend::discrete::scoped::{DependencyUse, GuideAllocation};
use crate::numeric::{decimal::NegativeZero, format::NumericFormat};

#[test]
fn recorded_heights_require_complete_numeric_and_guide_inputs_at_equal_width_and_domain() {
    let format = NumericFormat::default();
    let make = |guide, values| {
        Plot::heatmap(
            crate::CategoryValues::new("x", ["a"]).unwrap(),
            crate::CategoryValues::new("y", ["one"]).unwrap(),
            crate::NumericValues::new("value", [1.0]),
        )
        .color_label_format(guide)
        .annotations(crate::HeatmapAnnotations::Values(values))
        .erase()
    };
    let operation = crate::font::canonical::OperationContext::new();
    let source = make(format.clone(), format.clone());
    let inputs = crate::plots::shared_x::prepare_plot(
        &source,
        &operation,
        Some(&crate::Theme::publication()),
    )
    .unwrap();
    inputs
        .prepare_discrete_guides(
            &source,
            &crate::Theme::publication().resolve().unwrap(),
            &operation,
            &ComponentAddress::root(),
            false,
        )
        .unwrap();
    let source = inputs.attach(source).unwrap();
    let normalized = format
        .clone()
        .with_negative_zero(NegativeZero::Normalize)
        .unwrap();
    let variants = [
        inputs
            .attach(make(normalized.clone(), format.clone()))
            .unwrap(),
        inputs.attach(make(format, normalized)).unwrap(),
    ];
    let measurements = HeightMeasurements::default();
    crate::layout::plan_exact_root(
        800,
        600,
        |_| Ok(()),
        |(), assigned, round| {
            let owner = ComponentAddress::root().child(AddressRole::CanvasCell, 0)?;
            let ancestor = ComponentAddress::root().child(AddressRole::CanvasSharedLegend, 0)?;
            let dependencies = [ConsumedGuideDependency::new(
                ancestor.clone(),
                DependencyUse::AllocationOnly,
                GuideAllocation::width(400.0)?,
                round,
            )?];
            measurements.record(&source, 400.0, 120.0, [0; 4], round, &owner, &dependencies)?;
            let same =
                HeightInputs::new(&source, 400.0, [0; 4], round, &owner, true, &dependencies)?;
            assert_eq!(measurements.measured(&same, &owner, round)?, Some(120.0));
            let undeclared = HeightInputs::new(&source, 400.0, [0; 4], round, &owner, true, &[])?;
            assert_eq!(measurements.measured(&undeclared, &owner, round)?, None);
            let narrower = [ConsumedGuideDependency::new(
                ancestor,
                DependencyUse::AllocationOnly,
                GuideAllocation::width(400.0 - 1.0 / 1024.0)?,
                round,
            )?];
            let different =
                HeightInputs::new(&source, 400.0, [0; 4], round, &owner, true, &narrower)?;
            assert_eq!(same.width, different.width);
            assert_eq!(measurements.measured(&different, &owner, round)?, None);
            for changed in &variants {
                let different =
                    HeightInputs::new(changed, 400.0, [0; 4], round, &owner, true, &dependencies)?;
                assert_eq!(same.width, different.width);
                assert_eq!(same.ranges, different.ranges);
                assert_eq!(measurements.measured(&different, &owner, round)?, None);
                measurements.record(changed, 400.0, 120.0, [0; 4], round, &owner, &dependencies)?;
                assert_eq!(
                    measurements.measured(&different, &owner, round)?,
                    Some(120.0)
                );
                assert_eq!(measurements.measured(&same, &owner, round)?, None);
            }
            Ok(crate::layout::MeasuredLayout::new(assigned, ()))
        },
        |(), _, (), _| Ok(()),
    )
    .unwrap();
}
