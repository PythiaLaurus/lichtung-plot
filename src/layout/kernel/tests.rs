use std::cell::Cell;

use super::*;

#[test]
fn address_paths_preserve_scope_and_share_failure_copies() {
    let parent = ComponentAddress::root()
        .child(AddressRole::CanvasGrid, 0)
        .unwrap();
    {
        let result = parent.child(AddressRole::CanvasCell, 7);
        {
            let owner = result.unwrap();
            assert_eq!(owner.leaf_role(), AddressRole::CanvasCell);
            assert_eq!(owner.leaf_slot_index(), 7);

            let scope = crate::layout::fit::FitScope::new(&owner, 6);
            let failure = scope.clone().failure(FitKind::Text, String::new());
            assert_eq!(failure.reason.ordinal, 6);
            assert!(std::sync::Arc::ptr_eq(
                owner.descendants.as_ref().unwrap(),
                failure.reason.address.descendants.as_ref().unwrap()
            ));

            assert_eq!(
                scope
                    .child(AddressRole::Annotation, 0)
                    .unwrap()
                    .address
                    .leaf_role(),
                AddressRole::Annotation
            );
        }
    }
    let caption = ComponentAddress::root()
        .child(AddressRole::RootCaption, 0)
        .unwrap();
    assert!(parent.canonical_text_cmp(&caption).is_lt());
    assert_eq!(format!("{parent:?}"), "ComponentAddress { root: AddressSegment { role: Root, slot_index: 0 }, descendants: [AddressSegment { role: CanvasGrid, slot_index: 0 }] }");
}

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
            assert_eq!(assigned.dimensions()?, (317, 211));
            round.observe_number(f64::from(*payload))?;
            let plan = payload.clone();
            measured_address.set(std::ptr::from_ref(plan.as_ref()));
            Ok(MeasuredLayout::new(assigned, plan))
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
        320,
        240,
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
fn preparation_retains_shared_operation_identity() {
    let mut resources = crate::font::canonical::OperationContext::new();
    let mut reservations = ReservationDriver::new();
    let stage = PreparedStage {
        resources: &mut resources,
        reservations: &mut reservations,
    };

    let environment = crate::font::canonical::PreparedFontEnvironment::synthetic();
    assert!(environment.validate_operation(&stage.operation()).is_err());
    let shared = stage.operation();

    assert!(shared.same_operation(&stage.operation()));
    assert!(!shared.same_operation(&crate::font::canonical::OperationContext::new()));
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
            let panel = crate::layout::fit::measured(frame.assign_fit(
                crate::layout::PanelBox::new(0.0, 100.0, 0.0, 100.0),
                consumed,
                &crate::layout::fit::FitScope::root(),
            ))
            .map_err(crate::layout::fit::PlanError::into_error)?;
            assignments.push(panel);
            round.begin_owner(&address)?;
            round.observe_box(panel)?;
            let next = [(consumed[0] + 1.0).min(7.0), 0.0, 0.0, 0.0];
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
    assert!(assignments.windows(2).all(|pair| pair[0] != pair[1]));
    assert_eq!(*root.plan(), assignments[7]);
    assert_eq!(root.plan().x0.to_bits(), 9.0_f64.to_bits());
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
fn content_roots_choose_width_once_and_freeze_eighth_round_content_height() {
    for width in [None, Some(300)] {
        let mut calls = 0;
        let address = ComponentAddress::root();
        let root = plan_exact_root(
            width,
            None,
            |_| Ok(()),
            |(), assigned, round| {
                calls += 1;
                let (chosen, height) = assigned.requested_dimensions();
                assert_eq!(height, None);
                assert_eq!(
                    chosen,
                    if calls == 1 {
                        width
                    } else {
                        Some(width.unwrap_or(317))
                    }
                );
                let consumed = round.insets(&address, FeedbackKind::AxisGutter)[2];
                let next = (consumed + 1.0).min(7.0);
                round.request_insets(
                    &address,
                    &address,
                    FeedbackKind::AxisGutter,
                    [0.0, 0.0, next, 0.0],
                )?;
                let size = RootSize::new(
                    chosen.unwrap_or(317),
                    100 + u32::try_from(nearest_spatial_quanta(consumed)? / 1024).unwrap(),
                );
                Ok(MeasuredLayout::new(size, calls))
            },
            |(), assigned, plan, _| {
                assert_eq!(assigned.dimensions()?, (width.unwrap_or(317), 107));
                Ok(plan)
            },
        )
        .unwrap();
        assert_eq!(calls, 8);
        assert_eq!(*root.plan(), 8);
        assert_eq!(root.dimensions(), (width.unwrap_or(317), 107));
    }
}

#[test]
fn automatic_width_reselection_and_invalid_requests_refuse_before_capture() {
    let mut calls = 0;
    let owner = ComponentAddress::root();
    let result = plan_exact_root(
        None,
        None,
        |_| Ok(()),
        |(), _, round| {
            calls += 1;
            round.request_insets(&owner, &owner, FeedbackKind::AxisGutter, [1.0; 4])?;
            Ok(MeasuredLayout::new(RootSize::new(300 + calls, 100), ()))
        },
        |(), _, (), _| -> Result<(), Error> { panic!("width reselection captured") },
    );
    assert!(result.err().unwrap().to_string().contains("sizing request"));
    assert_eq!(calls, 2);
    for (width, height) in [(None, Some(10)), (Some(0), None), (Some(10), Some(0))] {
        let result = plan_exact_root(
            width,
            height,
            |_| -> Result<(), Error> { panic!("invalid request reached preparation") },
            |(), _, _| -> Result<MeasuredLayout<()>, Error> { panic!("invalid request measured") },
            |(), _, (), _| -> Result<(), Error> { panic!("invalid request captured") },
        );
        assert!(result.is_err());
    }
    let root = plan_exact_root(
        None,
        None,
        |_| Ok(()),
        |(), _, _| Ok(MeasuredLayout::new(RootSize::new(u32::MAX, u32::MAX), ())),
        |(), _, (), _| Ok(()),
    )
    .unwrap();
    assert_eq!(root.dimensions(), (u32::MAX, u32::MAX));
}

#[test]
fn both_content_modes_reject_feedback_cycles_before_finalization() {
    for width in [None, Some(300)] {
        let owner = ComponentAddress::root();
        let mut calls = 0;
        let result = plan_exact_root(
            width,
            None,
            |_| Ok(()),
            |(), assigned, round| {
                calls += 1;
                let previous = round.insets(&owner, FeedbackKind::AxisGutter)[0];
                round.request_insets(
                    &owner,
                    &owner,
                    FeedbackKind::AxisGutter,
                    [1.0 - previous, 0.0, 0.0, 0.0],
                )?;
                Ok(MeasuredLayout::new(
                    RootSize::new(assigned.requested_dimensions().0.unwrap_or(317), 211),
                    (),
                ))
            },
            |(), _, (), _| -> Result<(), Error> { panic!("cyclic content root captured") },
        );
        assert!(result.err().unwrap().to_string().contains("cycle"));
        assert!(calls <= 8);
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
            let domain = round.feedback_domain(&address, FeedbackKind::AutomaticDomainX);
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
fn changed_feedback_remeasures_and_freezes_the_current_plan_for_every_input_kind() {
    for kind in [
        FeedbackKind::AxisGutter,
        FeedbackKind::SharedAlignment,
        FeedbackKind::OwnedInk,
        FeedbackKind::ContentHeight,
        FeedbackKind::AutomaticDomainX,
        FeedbackKind::AutomaticDomainY,
    ] {
        let address = ComponentAddress::root();
        let mut consumed_inputs = Vec::new();
        let measured_address = Cell::new(std::ptr::null::<bool>());
        let root = plan_exact_root(
            100,
            100,
            |_| Ok(()),
            |(), assigned, round| {
                let consumed = match kind {
                    FeedbackKind::AutomaticDomainX | FeedbackKind::AutomaticDomainY => {
                        let consumed = round.feedback_domain(&address, kind).is_some();
                        round.request_domain(&address, &address, kind, (-1.0, 2.0))?;
                        consumed
                    }
                    _ => {
                        let consumed = round.insets(&address, kind)[0] > 0.0;
                        round.request_insets(&address, &address, kind, [1.0, 0.0, 0.0, 0.0])?;
                        consumed
                    }
                };
                consumed_inputs.push(consumed);
                // Even with no fit miss, changed inputs require a fresh plan.
                let plan = Box::new(consumed);
                measured_address.set(std::ptr::from_ref(plan.as_ref()));
                Ok(MeasuredLayout::new(assigned, plan))
            },
            |(), _, plan, _| {
                assert_eq!(std::ptr::from_ref(plan.as_ref()), measured_address.get());
                Ok(plan)
            },
        )
        .unwrap();
        assert_eq!(consumed_inputs, [false, true], "{kind:?}");
        assert!(**root.plan());
        assert_eq!(
            std::ptr::from_ref(root.plan().as_ref()),
            measured_address.get()
        );
    }
}

#[test]
fn empty_requests_do_not_accept_an_actual_ink_miss() {
    let mut calls = 0;
    let result = plan_exact_root(
        100,
        100,
        |_| Ok(()),
        |(), assigned, round| {
            calls += 1;
            round.miss(
                &ComponentAddress::root(),
                FitKind::OverlayCollision,
                &[],
                "actual ink exceeds assigned frame",
            )?;
            Ok(MeasuredLayout::new(assigned, ()))
        },
        |(), _, (), _| -> Result<(), Error> { panic!("unvalidated ink finalized") },
    );
    assert_eq!(calls, 1);
    assert!(result.err().unwrap().to_string().contains("actual ink"));
}

#[test]
fn current_complete_plan_is_accepted_without_probing_callback_purity() {
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
    assert_eq!(calls, 1);
    assert_eq!(root.plan().x0.to_bits(), 1.0_f64.to_bits());
}
