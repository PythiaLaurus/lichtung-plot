use std::cell::Cell;

use super::*;

#[test]
fn binary64_quantum_rounding_is_exact_for_edges_and_ties() {
    for (value, rounding, expected) in [
        (1.0 / 2048.0, SpatialRounding::Floor, 0),
        (1.0 / 2048.0, SpatialRounding::Ceil, 1),
        (1.0 / 2048.0, SpatialRounding::NearestEven, 0),
        (3.0 / 2048.0, SpatialRounding::NearestEven, 2),
        (5.0 / 2048.0, SpatialRounding::NearestEven, 2),
        (-1.0 / 2048.0, SpatialRounding::Floor, -1),
        (-1.0 / 2048.0, SpatialRounding::Ceil, 0),
        (f64::from_bits(1), SpatialRounding::Ceil, 1),
        (-0.0, SpatialRounding::Floor, 0),
    ] {
        assert_eq!(
            LayoutScalar::from_css_pixels(value, rounding).unwrap(),
            LayoutScalar(expected)
        );
    }
    for invalid in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX,
        f64::from_bits(1141_u64 << 52),
    ] {
        assert!(LayoutScalar::from_css_pixels(invalid, SpatialRounding::NearestEven).is_err());
    }
}

#[test]
fn exact_root_prepares_measures_and_finalizes_once_with_assigned_dimensions() {
    let prepares = Cell::new(0);
    let measures = Cell::new(0);
    let finalizes = Cell::new(0);
    let payload = Box::new(17_u8);
    let measured_address = Cell::new(std::ptr::null::<u8>());
    let root = plan_root(
        RootRequest::ExactSize {
            width: 317,
            height: 211,
        },
        |_| {
            prepares.set(prepares.get() + 1);
            Ok(23)
        },
        |prepared, assigned, round| {
            measures.set(measures.get() + 1);
            assert_eq!(*prepared, 23);
            assert_eq!(assigned.dimensions()?, (317, 211));
            round.observe_number(f64::from(*payload))?;
            let plan = payload.clone();
            measured_address.set(std::ptr::from_ref(plan.as_ref()));
            Ok(MeasuredLayout::new(assigned.assigned_size()?, plan))
        },
        |prepared, assigned, plan, _| {
            finalizes.set(finalizes.get() + 1);
            assert_eq!(*prepared, 23);
            assert_eq!(assigned.dimensions()?, (317, 211));
            assert_eq!(*plan, 17);
            assert_eq!(std::ptr::from_ref(plan.as_ref()), measured_address.get());
            Ok(plan)
        },
    )
    .unwrap();
    assert_eq!((prepares.get(), measures.get(), finalizes.get()), (1, 1, 1));
    assert_eq!(root.dimensions(), (317, 211));
    assert_eq!(**root.plan(), 17);
    assert_eq!(
        std::ptr::from_ref(root.plan().as_ref()),
        measured_address.get()
    );
}

#[test]
fn feedback_cycles_exhaustion_and_unchanged_misses_never_finalize() {
    for policy in 0..3 {
        let address = ComponentAddress::root();
        let mut calls = 0;
        let result = plan_root(
            RootRequest::ExactSize {
                width: 100,
                height: 100,
            },
            |_| Ok(()),
            |(), assigned, round| {
                calls += 1;
                let consumed = round.insets(&address, FeedbackKind::AxisGutter);
                let next = match policy {
                    0 => 1.0 - consumed[0],
                    1 => consumed[0] + 1.0,
                    _ => 1.0,
                };
                let request = round.request_insets(
                    &address,
                    &address,
                    FeedbackKind::AxisGutter,
                    [next, 0.0, 0.0, 0.0],
                )?;
                round.observe_number(consumed[0])?;
                round.miss(
                    &address,
                    FitKind::Tick,
                    &[request],
                    "unchanged gutter cannot resolve this fit",
                )?;
                Ok(MeasuredLayout::new(assigned.assigned_size()?, ()))
            },
            |(), _, (), _| -> Result<(), Error> { panic!("miss finalized") },
        );
        let error = result.err().unwrap().to_string();
        match policy {
            0 => assert!(error.contains("cycle")),
            1 => {
                assert!(error.contains("eight rounds"));
                assert_eq!(calls, 8);
            }
            _ => {
                assert!(error.contains("unchanged gutter"));
                assert_eq!(calls, 2);
            }
        }
    }
}

#[test]
fn content_roots_choose_width_once_and_freeze_eighth_round_content_height() {
    for request in [
        RootRequest::Natural,
        RootRequest::ExactWidth(300),
        RootRequest::MaxWidth(301),
    ] {
        let width = request.exact_width();
        let selected = request.select_width(317.25).unwrap();
        let mut calls = 0;
        let address = ComponentAddress::root();
        let root = plan_root(
            request,
            |_| Ok(()),
            |(), assigned, round| {
                calls += 1;
                let chosen = assigned.exact_width();
                assert!(assigned.automatic_height());
                assert_eq!(chosen, if calls == 1 { width } else { Some(selected) });
                let consumed = round.insets(&address, FeedbackKind::AxisGutter)[2];
                let next = (consumed + 1.0).min(7.0);
                round.request_insets(
                    &address,
                    &address,
                    FeedbackKind::AxisGutter,
                    [0.0, 0.0, next, 0.0],
                )?;
                let size = RootSize::new(
                    assigned.select_width(317.25 + consumed)?,
                    100 + u32::try_from(nearest_spatial_quanta(consumed)? / 1024).unwrap(),
                );
                Ok(MeasuredLayout::new(size, calls))
            },
            |(), assigned, plan, _| {
                assert_eq!(assigned.dimensions()?, (selected, 107));
                Ok(plan)
            },
        )
        .unwrap();
        assert_eq!(calls, 8);
        assert_eq!(*root.plan(), 8);
        assert_eq!(root.dimensions(), (selected, 107));
    }
}
