use super::*;

fn values(range: (f64, f64), step: f64, origin: f64) -> Vec<f64> {
    positions(
        range,
        None,
        false,
        (step, origin, TickSpace::Data, TickRequirement::Fixed),
        usize::MAX,
    )
    .unwrap()
    .unwrap()
    .coordinates
}

#[test]
fn data_intervals_follow_final_range_and_origin_without_density_or_endpoint_padding() {
    for (range, origin, expected) in [
        ((0.0, 100.0), 0.0, vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]),
        (
            (0.0, 200.0),
            0.0,
            (0..=10).map(|index| f64::from(index) * 20.0).collect(),
        ),
        ((40.0, 100.0), 0.0, vec![40.0, 60.0, 80.0, 100.0]),
        ((5.0, 95.0), 0.0, vec![20.0, 40.0, 60.0, 80.0]),
        ((5.0, 95.0), 5.0, vec![5.0, 25.0, 45.0, 65.0, 85.0]),
        ((-45.0, 35.0), 5.0, vec![-35.0, -15.0, 5.0, 25.0]),
    ] {
        assert_eq!(values(range, 20.0, origin), expected);
    }
    assert!(values((1.1, 1.2), 1.0, 0.0).is_empty());
    assert_eq!(
        values((-0.5, 0.75), 0.25, 0.125),
        [-0.375, -0.125, 0.125, 0.375, 0.625]
    );
    assert_eq!(values((0.0, 1.0), 1.0, -0.0), [0.0, 1.0]);
}

#[test]
fn exact_rounding_cells_admit_decimal_endpoints_and_indices_larger_than_binary64_integers() {
    let decimal = values((0.0, 1.0), 0.1, 0.0);
    assert_eq!(decimal.len(), 11);
    assert_eq!(decimal[3].to_bits(), 0.3_f64.next_up().to_bits());
    assert_eq!(decimal[10].to_bits(), 1.0_f64.to_bits());
    assert_eq!(values((0.0, 1.0), 0.5, 1e20), [0.0, 0.5, 1.0]);
    assert_eq!(
        values((-f64::MAX, f64::MAX), f64::MAX, 0.0),
        [-f64::MAX, 0.0, f64::MAX]
    );
    let parameters = (1.0, 0.0, TickSpace::Data, TickRequirement::Fixed);
    assert_eq!(
        positions((0.0, 10_000.0), None, false, parameters, 10_001)
            .unwrap()
            .unwrap()
            .coordinates
            .len(),
        10_001
    );
    assert!(positions((0.0, 10_000.0), None, false, parameters, 10_000)
        .unwrap()
        .is_none());
    assert!(fixed_is_empty((0.0, 10_000.0), None, false, parameters, 10_000).is_err());
    assert!(positions(
        (0.0, 1.0),
        None,
        false,
        (
            f64::from_bits(1),
            0.0,
            TickSpace::Data,
            TickRequirement::Fixed
        ),
        usize::MAX
    )
    .unwrap()
    .is_none());
    let adjacent = (1e20, 1e20_f64.next_up());
    assert!(positions(adjacent, None, false, parameters, usize::MAX).is_err());
}

#[test]
fn logarithmic_data_intervals_retain_physical_operands_independently_of_scale_intervals() {
    let range = (0.0, 2.0);
    let data = positions(
        range,
        Some((1.0, 100.0)),
        true,
        (20.0, 0.0, TickSpace::Data, TickRequirement::Fixed),
        100,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        data.data.as_ref().unwrap(),
        &[20.0, 40.0, 60.0, 80.0, 100.0]
    );
    assert_eq!(data.coordinates.last(), Some(&2.0));
    let scale = positions(
        range,
        Some((1.0, 100.0)),
        true,
        (1.0, 0.0, TickSpace::Scale, TickRequirement::Fixed),
        100,
    )
    .unwrap()
    .unwrap();
    assert_eq!(scale.coordinates, [0.0, 1.0, 2.0]);
    assert!(scale.data.is_none());
    let certified = positions(
        range,
        None,
        true,
        (20.0, 0.0, TickSpace::Data, TickRequirement::Fixed),
        100,
    )
    .unwrap()
    .unwrap();
    assert_eq!(certified.data, data.data);
    assert_eq!(certified.coordinates, data.coordinates);
    let symbolic = positions(
        (400.0, 500.0),
        None,
        true,
        (50.0, 0.0, TickSpace::Scale, TickRequirement::Fixed),
        100,
    )
    .unwrap()
    .unwrap();
    assert_eq!(symbolic.coordinates, [400.0, 450.0, 500.0]);
    crate::Plot::forest(
        crate::CategoryValues::new("term", ["ratio"]).unwrap(),
        crate::NumericValues::new("estimate", [2.0]),
        crate::Bounds::new(
            crate::NumericValues::new("lo", [1.5]),
            crate::NumericValues::new("hi", [3.0]),
        ),
        1.0,
    )
    .scale(crate::AxisScale::Log10)
    .x_limits(1.0, 100.0)
    .x_tick_generation(
        crate::TickGeneration::interval(20.0, 0.0, TickSpace::Data, TickRequirement::Fixed)
            .unwrap(),
    )
    .x_tick_format(
        crate::NumericFormat::from_specifier(".0f")
            .unwrap()
            .set_function(|context| {
                if context.purpose() == crate::NumericPurpose::ActualSet {
                    let values: Vec<_> = context
                        .values()
                        .iter()
                        .map(|value| match value.operand() {
                            crate::NumericOperand::Binary64(value) => value,
                            crate::NumericOperand::PowerOfTen { .. }
                            | crate::NumericOperand::Multiplicative { .. } => {
                                panic!("physical data interval became exponent spacing")
                            }
                        })
                        .collect();
                    assert_eq!(values, [20.0, 40.0, 60.0, 80.0, 100.0]);
                }
                context.format_builtin()
            }),
    )
    .size(1800, 500)
    .preflight()
    .unwrap();
}

#[test]
fn fixed_sources_refuse_caps_and_text_nonfit_while_preferred_sources_keep_complete_recovery() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let projection =
        super::super::Projection::new((0.0, 100.0), (0.0, 1.0), (0.0, 640.0, 0.0, 480.0));
    let plan = |requirement, maximum| {
        super::super::numeric_tick_plan(
            (0.0, 100.0),
            crate::TickGeneration::interval(1.0, 0.0, TickSpace::Data, requirement)
                .unwrap()
                .maximum_count(maximum)
                .unwrap(),
            false,
            &projection,
            super::super::AxisSide::Bottom,
            &theme,
            false,
            None,
            &crate::layout::fit::ROOT_SCOPE,
        )
    };
    assert!(plan(TickRequirement::Fixed, None).is_err());
    assert!(plan(TickRequirement::Fixed, Some(1)).is_err());
    let recovered = plan(TickRequirement::Preferred, Some(30)).unwrap();
    let super::super::SourceKind::Additive(step) = recovered.source.kind else {
        panic!("preferred interval changed locator")
    };
    assert!(step > 1.0 && recovered.source.ordinal > 0);
    assert!(recovered
        .ticks
        .windows(2)
        .all(|pair| (pair[1].position - pair[0].position).to_bits() == step.to_bits()));
}
