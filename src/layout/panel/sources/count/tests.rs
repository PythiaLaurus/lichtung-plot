use super::*;

#[test]
fn affine_counts_round_each_original_combination_and_refuse_collapsed_sources() {
    let values = |range, count| {
        positions(
            range,
            None,
            (count, TickSpace::Scale, TickSingleton::Lower),
            false,
        )
        .unwrap()
        .coordinates
    };
    assert_eq!(
        values((0.0, 10.0), 4)
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        [
            0,
            0x400a_aaaa_aaaa_aaab,
            0x401a_aaaa_aaaa_aaab,
            10.0_f64.to_bits()
        ]
    );
    assert_eq!(values((-f64::MAX, f64::MAX), 3), [-f64::MAX, 0.0, f64::MAX]);
    assert_eq!(
        values((f64::from_bits(1), f64::from_bits(5)), 3)
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        [1, 3, 5]
    );
    assert!(positions(
        (0.0, f64::from_bits(1)),
        None,
        (3, TickSpace::Data, TickSingleton::Lower),
        false
    )
    .is_err());
    assert!(values((0.0, 1.0), 0).is_empty());
    assert_eq!(values((-0.0, 1.0), 2)[0].to_bits(), (-0.0_f64).to_bits());
    assert!(positions(
        (0.0, 1.0),
        None,
        (usize::MAX, TickSpace::Data, TickSingleton::Lower),
        false
    )
    .is_err());
}

#[test]
fn singleton_and_space_requests_do_not_substitute_density_or_another_domain() {
    for (singleton, expected) in [
        (TickSingleton::Lower, 0.0),
        (TickSingleton::Midpoint, 5.0),
        (TickSingleton::Upper, 10.0),
        (TickSingleton::Position(3.0), 3.0),
    ] {
        assert_eq!(
            positions((0.0, 10.0), None, (1, TickSpace::Data, singleton), false)
                .unwrap()
                .coordinates,
            [expected]
        );
    }
    assert!(positions(
        (0.0, 10.0),
        None,
        (1, TickSpace::Data, TickSingleton::Position(20.0)),
        false
    )
    .is_err());
    assert_eq!(
        positions(
            (0.0, 10.0),
            None,
            (2, TickSpace::Data, TickSingleton::Position(20.0)),
            false
        )
        .unwrap()
        .coordinates,
        [0.0, 10.0]
    );
    let scale = positions(
        (-1.0, 1.0),
        Some((0.1, 10.0)),
        (3, TickSpace::Scale, TickSingleton::Lower),
        true,
    )
    .unwrap();
    assert_eq!(scale.coordinates, [-1.0, 0.0, 1.0]);
    assert!(scale.power && scale.data.is_none());
    let data = positions(
        (-1.0, 1.0),
        Some((0.1, 10.0)),
        (3, TickSpace::Data, TickSingleton::Lower),
        true,
    )
    .unwrap();
    assert_eq!(data.data.unwrap(), [0.1, 5.05, 10.0]);
    assert!(!data.power);
    assert_eq!(
        data.coordinates[1].to_bits(),
        crate::numeric::log10(5.05).unwrap().to_bits()
    );
    let mathematical = positions(
        (0.0, 2.0),
        None,
        (3, TickSpace::Data, TickSingleton::Lower),
        true,
    )
    .unwrap();
    assert_eq!(mathematical.data.unwrap(), [1.0, 50.5, 100.0]);
    let radical = positions(
        (0.5, 1.5),
        None,
        (3, TickSpace::Data, TickSingleton::Lower),
        true,
    )
    .unwrap()
    .data
    .unwrap()[1];
    let center = crate::numeric::binary::units(radical).unwrap();
    // The exact midpoint is 11*sqrt(10)/2; square its two binary rounding edges.
    let squared = num_bigint::BigInt::from(1210_u32) << 2148;
    assert!(
        (crate::numeric::binary::units(radical.next_down()).unwrap() + &center).pow(2) < squared
    );
    assert!((crate::numeric::binary::units(radical.next_up()).unwrap() + center).pow(2) > squared);
    let request = crate::TickGeneration::exact_count(4, TickSpace::Data);
    assert!(request.clone().maximum_count(Some(3)).is_err());
    assert_eq!(
        request
            .clone()
            .maximum_count(Some(4))
            .unwrap()
            .maximum_count(None)
            .unwrap(),
        request
    );
    assert!(crate::TickGeneration::target(4)
        .unwrap()
        .singleton(TickSingleton::Upper)
        .is_err());
    assert!(request.maximum_count(Some(0)).is_err());
}

#[test]
fn explicit_sources_above_ten_thousand_keep_their_cardinality_and_refuse_text_nonfit() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let projection = Projection::new((0.0, 1.0), (0.0, 1.0), (0.0, 2_000_000.0, 0.0, 480.0));
    let format =
        crate::NumericFormat::default().value_function(|context| Ok(context.index().to_string()));
    let request = crate::numeric::source::AxisRequest::for_test(&format);
    let selected = plan(
        (0.0, 1.0),
        (10_001, TickSpace::Data, TickSingleton::Lower),
        false,
        None,
        &projection,
        AxisSide::Bottom,
        &theme,
        Some(&request),
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap();
    assert_eq!(selected.ticks.len(), 10_001);
    assert_eq!(selected.source.kind, SourceKind::ExactCount);
    let collision = crate::NumericFormat::from_specifier(".0f").unwrap();
    let request = crate::numeric::source::AxisRequest::for_test(&collision);
    assert!(plan(
        (0.0, 1.0),
        (4, TickSpace::Data, TickSingleton::Lower),
        false,
        None,
        &projection,
        AxisSide::Bottom,
        &theme,
        Some(&request),
        &crate::layout::fit::ROOT_SCOPE
    )
    .is_err());
}
