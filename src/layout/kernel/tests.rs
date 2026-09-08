use std::cell::Cell;

use super::*;

#[test]
fn exact_quantum_conversion_and_constraints_preserve_boundaries() {
    let exact = LayoutScalar::exact_css_pixels(320).unwrap();
    assert_eq!(exact, LayoutScalar(327_680));
    assert_eq!(exact.canonical_dimension(), Some(320));
    assert_eq!(LayoutScalar(1).canonical_dimension(), Some(1));
    assert_eq!(LayoutScalar(1025).canonical_dimension(), Some(2));
    assert_eq!(LayoutScalar(i64::MAX).canonical_dimension(), None);
    let interval = LengthConstraint::new(LayoutScalar(1), Some(LayoutScalar(2))).unwrap();
    assert!(interval.contains(LayoutScalar(1)));
    assert!(interval.contains(LayoutScalar(2)));
    assert!(!interval.contains(LayoutScalar(0)));
    assert!(LengthConstraint::new(LayoutScalar(-1), None).is_err());
    assert!(LengthConstraint::new(LayoutScalar(2), Some(LayoutScalar(1))).is_err());
    assert!(LayoutSize::new(LayoutScalar(-1), LayoutScalar(1)).is_err());
}

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
    let address = std::ptr::from_ref(payload.as_ref());
    let root = plan_exact_root(
        317,
        211,
        |_| {
            prepares.set(prepares.get() + 1);
            Ok(23)
        },
        |prepared, assigned| {
            measures.set(measures.get() + 1);
            assert_eq!(*prepared, 23);
            assert_eq!(assigned.dimensions(), (317, 211));
            Ok(MeasuredLayout::new(assigned, payload))
        },
        |prepared, assigned, plan, _| {
            finalizes.set(finalizes.get() + 1);
            assert_eq!(*prepared, 23);
            assert_eq!(assigned.dimensions(), (317, 211));
            assert_eq!(std::ptr::from_ref(plan.as_ref()), address);
            Ok(plan)
        },
    )
    .unwrap();
    assert_eq!((prepares.get(), measures.get(), finalizes.get()), (1, 1, 1));
    assert_eq!(root.dimensions(), (317, 211));
    assert_eq!(*root.prepared(), 23);
    assert_eq!(std::ptr::from_ref(root.plan().as_ref()), address);
}

#[test]
fn exact_root_rejects_mismatched_measurement_and_nonpositive_size_before_finalization() {
    for (assigned, measured, message) in [
        ((320, 240), (321, 240), "exact assignment"),
        ((320, 240), (320, 241), "exact assignment"),
        ((0, 240), (0, 240), "finite and positive"),
        ((320, 0), (320, 0), "finite and positive"),
    ] {
        let result = plan_exact_root(
            assigned.0,
            assigned.1,
            |_| Ok(()),
            |(), _| {
                Ok(MeasuredLayout::new(
                    RootSize::new(measured.0, measured.1),
                    (),
                ))
            },
            |(), _, (), _| -> Result<(), Error> { panic!("invalid measurement finalized") },
        );
        assert!(matches!(result, Err(Error::Render(ref error)) if error.contains(message)));
    }
    let root = plan_exact_root(
        u32::MAX,
        u32::MAX,
        |_| Ok(()),
        |(), assigned| Ok(MeasuredLayout::new(assigned, ())),
        |(), _, (), _| Ok(()),
    )
    .unwrap();
    assert_eq!(root.dimensions(), (u32::MAX, u32::MAX));
}

#[test]
fn preparation_and_measurement_errors_are_preserved_without_finalization() {
    let result = plan_exact_root(
        0,
        0,
        |_| -> Result<(), Error> { Err(Error::Data("preparation refused".into())) },
        |(), _| -> Result<MeasuredLayout<()>, Error> { panic!("preparation failed") },
        |(), _, (), _| -> Result<(), Error> { panic!("preparation failed") },
    );
    assert!(matches!(result, Err(Error::Data(ref message)) if message == "preparation refused"));
    let result = plan_exact_root(
        320,
        240,
        |_| Ok(()),
        |(), _| -> Result<MeasuredLayout<()>, Error> {
            Err(Error::Render("measurement refused".into()))
        },
        |(), _, (), _| -> Result<(), Error> { panic!("measurement failed") },
    );
    assert!(matches!(result, Err(Error::Render(ref message)) if message == "measurement refused"));
}

#[test]
#[expect(
    clippy::redundant_closure_for_method_calls,
    reason = "the method item cannot satisfy the callback's higher-ranked PreparedStage lifetime"
)]
fn actual_reservation_stages_refuse_before_the_next_callback() {
    for stage in [
        ReservationStage::Prepared,
        ReservationStage::Measured,
        ReservationStage::Final,
    ] {
        force_reservation_failure(stage);
        let measures = Cell::new(0);
        let finalizes = Cell::new(0);
        let result = plan_exact_root(
            320,
            240,
            |stage| stage.reserve_scalar_presence_map(),
            |_, assigned| {
                measures.set(measures.get() + 1);
                Ok(MeasuredLayout::new(assigned, ()))
            },
            |_, _, (), _| {
                finalizes.set(finalizes.get() + 1);
                Ok(())
            },
        );
        assert!(
            matches!(result, Err(Error::Render(ref message)) if message.contains(&format!("reservation stage {}", stage.number())))
        );
        assert_eq!(measures.get(), u8::from(stage == ReservationStage::Final));
        assert_eq!(finalizes.get(), 0);
    }
}

#[test]
fn capacity_refusal_and_out_of_order_reservation_are_errors() {
    for stage in [
        ReservationStage::Prepared,
        ReservationStage::Measured,
        ReservationStage::Final,
    ] {
        let mut driver = ReservationDriver { phase: stage };
        let error = driver
            .reserve_additional(stage, &mut Vec::<u8>::new(), usize::MAX, "test capacity")
            .unwrap_err();
        assert!(error
            .to_string()
            .contains(&format!("stage {}", stage.number())));
    }
    let error = ReservationDriver::new()
        .reserve_additional(
            ReservationStage::Final,
            &mut Vec::<u8>::new(),
            1,
            "out of order",
        )
        .unwrap_err();
    assert!(error.to_string().contains("out of order"));
}

#[test]
fn preparation_retains_shared_font_and_generation_resources_and_topology_limit() {
    let mut resources = ResourceState::new();
    let mut reservations = ReservationDriver::new();
    let mut stage = PreparedStage {
        resources: &mut resources,
        reservations: &mut reservations,
    };
    stage.claim_topology_slots(TOPOLOGY_SLOT_LIMIT - 1).unwrap();
    stage.claim_topology_slots(1).unwrap();
    assert!(stage
        .claim_topology_slots(1)
        .unwrap_err()
        .to_string()
        .contains("limit of 1000000"));
    let environment = crate::font::canonical::PreparedFontEnvironment::synthetic();
    environment
        .share_generation_resources(stage.generation_resources())
        .unwrap();
    let shared = stage.generation_resources();
    environment
        .claim_text_generation_slots(TOPOLOGY_SLOT_LIMIT - 1)
        .unwrap();
    shared.claim_slots(1).unwrap();
    assert!(environment.claim_text_generation_slots(1).is_err());
    environment
        .claim_text_generation_work(TOPOLOGY_SLOT_LIMIT - 1)
        .unwrap();
    shared.claim_work(1).unwrap();
    assert!(environment.claim_text_generation_work(1).is_err());
}
