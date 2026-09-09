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
fn exact_root_prepares_once_measures_twice_and_finalizes_once_with_assigned_dimensions() {
    let prepares = Cell::new(0);
    let measures = Cell::new(0);
    let finalizes = Cell::new(0);
    let payload = Box::new(17_u8);
    let measured_address = Cell::new(std::ptr::null::<u8>());
    let root = plan_exact_root(
        317,
        211,
        |_| {
            prepares.set(prepares.get() + 1);
            Ok(23)
        },
        |prepared, assigned, round| {
            measures.set(measures.get() + 1);
            assert_eq!(*prepared, 23);
            assert_eq!(assigned.dimensions(), (317, 211));
            round.observe_number(f64::from(*payload))?;
            let plan = payload.clone();
            measured_address.set(std::ptr::from_ref(plan.as_ref()));
            Ok(MeasuredLayout::new(assigned, plan))
        },
        |prepared, assigned, plan, _| {
            finalizes.set(finalizes.get() + 1);
            assert_eq!(*prepared, 23);
            assert_eq!(assigned.dimensions(), (317, 211));
            assert_eq!(*plan, 17);
            assert_eq!(std::ptr::from_ref(plan.as_ref()), measured_address.get());
            Ok(plan)
        },
    )
    .unwrap();
    assert_eq!((prepares.get(), measures.get(), finalizes.get()), (1, 2, 1));
    assert_eq!(root.dimensions(), (317, 211));
    assert_eq!(*root.prepared(), 23);
    assert_eq!(**root.plan(), 17);
    assert_eq!(
        std::ptr::from_ref(root.plan().as_ref()),
        measured_address.get()
    );
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
            |(), _, _| {
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
        |(), assigned, _| Ok(MeasuredLayout::new(assigned, ())),
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
        |(), _, _| -> Result<MeasuredLayout<()>, Error> { panic!("preparation failed") },
        |(), _, (), _| -> Result<(), Error> { panic!("preparation failed") },
    );
    assert!(matches!(result, Err(Error::Data(ref message)) if message == "preparation refused"));
    let result = plan_exact_root(
        320,
        240,
        |_| Ok(()),
        |(), _, _| -> Result<MeasuredLayout<()>, Error> {
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
            |_, assigned, _| {
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
        assert_eq!(
            measures.get(),
            2 * u8::from(stage == ReservationStage::Final)
        );
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

#[test]
fn parent_feedback_reallocates_real_geometry_and_converges_on_round_eight() {
    let address = ComponentAddress::root();
    let frame = FrameRequirements {
        minimum_body: (1.0, 1.0),
        natural_insets: [2.0; 4],
    };
    let mut assignments = Vec::new();
    let root = plan_exact_root(
        100,
        100,
        |_| Ok(()),
        |(), assigned, round| {
            let consumed = round.insets(&address, FeedbackKind::AxisGutter);
            let panel = frame.assign(
                crate::layout::PanelBox::new(0.0, 100.0, 0.0, 100.0),
                consumed,
            )?;
            assignments.push(panel);
            round.begin_owner(&address)?;
            round.observe_box(panel)?;
            let next = [(consumed[0] + 1.0).min(6.0), 0.0, 0.0, 0.0];
            if let Some(request) =
                round.request_insets(&address, &address, FeedbackKind::AxisGutter, next)?
            {
                round.miss(
                    &address,
                    FitKind::Tick,
                    &[Some(request)],
                    "assigned axis needs its measured gutter",
                )?;
            }
            Ok(MeasuredLayout::new(assigned, panel))
        },
        |(), _, plan, _| Ok(plan),
    )
    .unwrap();
    assert_eq!(assignments.len(), 8);
    assert!(assignments[..7].windows(2).all(|pair| pair[0] != pair[1]));
    assert_eq!(assignments[6], assignments[7]);
    assert_eq!(root.plan().x0.to_bits(), 8.0_f64.to_bits());
}

#[test]
fn feedback_cycles_exhaustion_and_unchanged_misses_never_finalize() {
    for policy in 0..3 {
        let address = ComponentAddress::root();
        let mut calls = 0;
        let result = plan_exact_root(
            100,
            100,
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
                Ok(MeasuredLayout::new(assigned, ()))
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
fn later_fatal_measurement_wins_over_an_earlier_typed_fit_miss() {
    let result = plan_exact_root(
        100,
        100,
        |_| Ok(()),
        |(), _, round| -> Result<MeasuredLayout<()>, Error> {
            round.miss(
                &ComponentAddress::root(),
                FitKind::OverlayCollision,
                &[],
                "earlier collision",
            )?;
            Err(Error::Render("later font failure".into()))
        },
        |(), _, (), _| -> Result<(), Error> { panic!("fatal measurement finalized") },
    );
    assert!(result
        .err()
        .unwrap()
        .to_string()
        .contains("later font failure"));

    let address = ComponentAddress::root();
    let result = plan_exact_root(
        100,
        100,
        |_| Ok(()),
        |(), assigned, round| {
            let request = round.request_insets(
                &address,
                &address,
                FeedbackKind::AxisGutter,
                [1.0, 0.0, 0.0, 0.0],
            )?;
            round.miss(
                &address,
                FitKind::OverlayCollision,
                &[request],
                "provisional collision",
            )?;
            round.miss(
                &address,
                FitKind::OverlayCollision,
                &[],
                "terminal collision",
            )?;
            Ok(MeasuredLayout::new(assigned, ()))
        },
        |(), _, (), _| -> Result<(), Error> { panic!("terminal miss finalized") },
    );
    assert!(result
        .err()
        .unwrap()
        .to_string()
        .contains("terminal collision"));
}

#[test]
fn feedback_reserves_and_domains_aggregate_without_reapplying_owned_ink() {
    let address = ComponentAddress::root();
    let root = plan_exact_root(
        100,
        100,
        |_| Ok(()),
        |(), assigned, round| {
            round.request_insets(
                &address,
                &address,
                FeedbackKind::OwnedInk,
                [2.0, 0.0, 0.0, 0.0],
            )?;
            round.request_insets(
                &address,
                &address,
                FeedbackKind::OwnedInk,
                [1.0, 0.0, 0.0, 0.0],
            )?;
            round.request_domain(
                &address,
                &address,
                FeedbackKind::AutomaticDomainX,
                (-1.0, 5.0),
            )?;
            round.request_domain(
                &address,
                &address,
                FeedbackKind::AutomaticDomainX,
                (0.0, 6.0),
            )?;
            let insets = round.insets(&address, FeedbackKind::OwnedInk);
            let domain = round.domain(&address, FeedbackKind::AutomaticDomainX);
            Ok(MeasuredLayout::new(assigned, (insets, domain)))
        },
        |(), _, plan, _| Ok(plan),
    )
    .unwrap();
    assert_eq!(
        root.plan().0.map(f64::to_bits),
        [2.0_f64, 0.0, 0.0, 0.0].map(f64::to_bits)
    );
    assert_eq!(root.plan().1, Some((-1.0, 6.0)));
}

#[test]
fn feedback_admission_refuses_before_measurement_or_finalization() {
    let result = plan_exact_root(
        100,
        100,
        |stage| {
            stage
                .generation_resources()
                .claim_slots(1_000_000)
                .map_err(|error| Error::Render(format!("test admission: {error:?}")))?;
            Ok(())
        },
        |(), _, _| -> Result<MeasuredLayout<()>, Error> { panic!("unadmitted round measured") },
        |(), _, (), _| -> Result<(), Error> { panic!("unadmitted round finalized") },
    );
    assert!(result
        .err()
        .unwrap()
        .to_string()
        .contains("layout feedback admission"));
}

#[test]
fn changed_owner_geometry_is_not_reused_on_equal_feedback() {
    let mut calls = 0;
    let root = plan_exact_root(
        100,
        100,
        |_| Ok(()),
        |(), assigned, round| {
            calls += 1;
            let panel =
                crate::layout::PanelBox::new(if calls == 1 { 1.0 } else { 2.0 }, 90.0, 2.0, 90.0);
            round.observe_box(panel)?;
            Ok(MeasuredLayout::new(assigned, panel))
        },
        |(), _, panel, _| Ok(panel),
    )
    .unwrap();
    assert_eq!(calls, 3);
    assert_eq!(root.plan().x0.to_bits(), 2.0_f64.to_bits());
}
