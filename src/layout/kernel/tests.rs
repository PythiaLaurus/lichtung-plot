use std::cell::Cell;

use super::*;

fn candidate(
    width: u32,
    height: u32,
    kind: PlotKind,
    structural_tag: u8,
    plan: u8,
) -> MeasuredLayout<u8> {
    let constraints = LayoutConstraints::exact_css_pixels(width, height).unwrap();
    let size = LayoutSize::exact_css_pixels(width, height).unwrap();
    let mut signature = StructuralSignature::exact_plot_root(kind, constraints, size);
    signature.root.arrangement_decisions = Box::new([structural_tag]);
    let quality = RootQualityKey::exact(size, signature).unwrap();
    MeasuredLayout {
        size,
        plan,
        quality,
        eligibility_reasons: Box::new([]),
    }
}

fn convergence_state(tag: u32, painted_ink_inside_root: bool) -> ConvergenceState {
    let width = 200 + tag;
    let constraints = LayoutConstraints::exact_css_pixels(width, 120).unwrap();
    let size = LayoutSize::exact_css_pixels(width, 120).unwrap();
    let signature = StructuralSignature::exact_plot_root(PlotKind::Scatter, constraints, size);
    let quality = RootQualityKey::exact(size, signature).unwrap();
    let mut frontier = MeasurementFrontier::exact_plot(PlotKind::Scatter);
    frontier.prepared_semantic_identity = u128::from(tag);
    ConvergenceState {
        selected_size: size,
        selected_quality: quality,
        selected_candidate_index: 0,
        selected_child_identities: Box::new([]),
        selected_call_constraints: constraints,
        assigned_contexts: Box::new([]),
        provisional_root_box: LayoutBox::root(size),
        accounting: MeasurementAccounting::exact_root(),
        next_frontier: frontier,
        painted_ink_inside_root,
    }
}

fn prepared_reservations() -> ReservationDriver {
    let mut reservations = ReservationDriver::new();
    let mut prepared = Vec::<u8>::new();
    reservations
        .reserve_additional(
            ReservationStage::Prepared,
            &mut prepared,
            0,
            "test preparation",
        )
        .unwrap();
    reservations.finish_prepared();
    reservations
}

fn observe(
    tracker: &mut ConvergenceTracker,
    reservations: &mut ReservationDriver,
    state: ConvergenceState,
) -> Result<ConvergenceStatus, Error> {
    reservations.start_pass()?;
    tracker.observe(state, reservations)
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
    let half_quantum = 1.0 / 2048.0;
    let one_and_a_half = 3.0 / 2048.0;
    let two_and_a_half = 5.0 / 2048.0;

    assert_eq!(
        LayoutScalar::from_css_pixels(half_quantum, SpatialRounding::Floor).unwrap(),
        LayoutScalar(0)
    );
    assert_eq!(
        LayoutScalar::from_css_pixels(half_quantum, SpatialRounding::Ceil).unwrap(),
        LayoutScalar(1)
    );
    assert_eq!(
        LayoutScalar::from_css_pixels(half_quantum, SpatialRounding::NearestEven).unwrap(),
        LayoutScalar(0)
    );
    assert_eq!(
        LayoutScalar::from_css_pixels(one_and_a_half, SpatialRounding::NearestEven).unwrap(),
        LayoutScalar(2)
    );
    assert_eq!(
        LayoutScalar::from_css_pixels(two_and_a_half, SpatialRounding::NearestEven).unwrap(),
        LayoutScalar(2)
    );
    assert_eq!(
        LayoutScalar::from_css_pixels(-half_quantum, SpatialRounding::Floor).unwrap(),
        LayoutScalar(-1)
    );
    assert_eq!(
        LayoutScalar::from_css_pixels(-half_quantum, SpatialRounding::Ceil).unwrap(),
        LayoutScalar(0)
    );
    assert_eq!(
        LayoutScalar::from_css_pixels(f64::from_bits(1), SpatialRounding::Ceil).unwrap(),
        LayoutScalar(1)
    );
    assert_eq!(
        LayoutScalar::from_css_pixels(-0.0, SpatialRounding::Floor).unwrap(),
        LayoutScalar(0)
    );
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MAX] {
        assert!(LayoutScalar::from_css_pixels(invalid, SpatialRounding::NearestEven).is_err());
    }
    let first_u128_overflow = f64::from_bits(1141_u64 << 52);
    assert!(
        LayoutScalar::from_css_pixels(first_u128_overflow, SpatialRounding::NearestEven).is_err()
    );
}

#[test]
fn successful_measurement_is_nonempty_sorted_and_structurally_deduplicated() {
    assert!(MeasuredCandidates::<u8>::new(Vec::new()).is_err());

    let candidates = MeasuredCandidates::new(vec![
        candidate(320, 240, PlotKind::Scatter, 2, 2),
        candidate(320, 240, PlotKind::Scatter, 1, 1),
        candidate(320, 240, PlotKind::Scatter, 1, 1),
    ])
    .unwrap();

    assert_eq!(candidates.candidates.len(), 2);
    assert_eq!(candidates.candidates[0].plan, 1);
    assert_eq!(candidates.candidates[1].plan, 2);
}

#[test]
fn complete_cartesian_product_retains_every_child_combination() {
    let mut resources = ResourceState::new();
    let mut reservations = prepared_reservations();
    reservations.start_pass().unwrap();

    let combinations =
        complete_cartesian_product(&[2, 3], &mut resources, &mut reservations).unwrap();

    assert_eq!(
        combinations,
        [
            Box::<[usize]>::from([0, 0]),
            Box::<[usize]>::from([0, 1]),
            Box::<[usize]>::from([0, 2]),
            Box::<[usize]>::from([1, 0]),
            Box::<[usize]>::from([1, 1]),
            Box::<[usize]>::from([1, 2]),
        ]
    );
    assert_eq!(
        (resources.generation_slots, resources.generation_work),
        (18, 36)
    );

    let mut resources = ResourceState::new();
    let mut reservations = prepared_reservations();
    reservations.start_pass().unwrap();
    assert!(
        complete_cartesian_product(&[usize::MAX, 2], &mut resources, &mut reservations)
            .unwrap_err()
            .to_string()
            .contains("candidate-generation slots limit")
    );
}

#[test]
fn selector_uses_complete_quality_then_structural_order() {
    let select = |input| {
        let candidates = MeasuredCandidates::new(input).unwrap();
        candidates.get(select_root(&candidates).unwrap()).plan
    };

    assert_eq!(
        select(vec![
            candidate(320, 240, PlotKind::Scatter, 2, 2),
            candidate(320, 240, PlotKind::Scatter, 1, 1),
        ]),
        1
    );
    assert_eq!(
        select(vec![
            candidate(320, 240, PlotKind::Scatter, 1, 1),
            candidate(320, 240, PlotKind::Scatter, 2, 2),
        ]),
        1
    );

    let mut worse = candidate(320, 240, PlotKind::Scatter, 0, 3);
    worse.quality.panel_rank = AggregateRank {
        maximum: 1,
        sum: 1,
        preorder: Box::new([1]),
    };
    let better = candidate(320, 240, PlotKind::Scatter, 1, 4);
    assert_eq!(select(vec![worse, better]), 4);
}

#[test]
fn selector_applies_first_empty_frontier_order() {
    let mut semantic_failure = candidate(320, 240, PlotKind::Scatter, 0, 0);
    semantic_failure.quality.semantic_gate = 1;
    semantic_failure.eligibility_reasons = Box::new([EligibilityReason::root(
        1,
        EligibilityViolation::MissingSemantic,
    )]);
    let error = select_root(&MeasuredCandidates::new(vec![semantic_failure]).unwrap())
        .unwrap_err()
        .to_string();
    assert!(error.contains("filter 1") && error.contains("MissingSemantic"));

    let zero = LayoutSize::new(LayoutScalar(0), LayoutScalar(1024)).unwrap();
    let constraints = LayoutConstraints {
        width: LengthConstraint::new(LayoutScalar(0), Some(LayoutScalar(0))).unwrap(),
        height: LengthConstraint::new(LayoutScalar(1024), Some(LayoutScalar(1024))).unwrap(),
    };
    let signature = StructuralSignature::exact_plot_root(PlotKind::Scatter, constraints, zero);
    let zero_candidate = MeasuredLayout {
        size: zero,
        plan: 0,
        quality: RootQualityKey::exact(zero, signature).unwrap(),
        eligibility_reasons: Box::new([]),
    };
    assert!(
        select_root(&MeasuredCandidates::new(vec![zero_candidate]).unwrap())
            .unwrap_err()
            .to_string()
            .contains("filter 4")
    );
}

#[test]
fn convergence_accepts_pass_two_and_pass_eight() {
    let mut tracker = ConvergenceTracker::new();
    let mut reservations = prepared_reservations();
    assert_eq!(
        observe(&mut tracker, &mut reservations, convergence_state(0, true)).unwrap(),
        ConvergenceStatus::Continue
    );
    assert_eq!(
        observe(&mut tracker, &mut reservations, convergence_state(0, true)).unwrap(),
        ConvergenceStatus::Stable
    );

    let mut tracker = ConvergenceTracker::new();
    let mut reservations = prepared_reservations();
    for tag in 0..7 {
        assert_eq!(
            observe(
                &mut tracker,
                &mut reservations,
                convergence_state(tag, true)
            )
            .unwrap(),
            ConvergenceStatus::Continue
        );
    }
    assert_eq!(
        observe(&mut tracker, &mut reservations, convergence_state(6, true)).unwrap(),
        ConvergenceStatus::Stable
    );
}

#[test]
fn convergence_driver_consumes_the_complete_next_frontier() {
    let mut resources = ResourceState::new();
    let mut reservations = prepared_reservations();
    let initial = MeasurementFrontier::exact_plot(PlotKind::Scatter);
    let mut calls = 0;

    let selected = run_convergence(
        initial,
        &mut resources,
        &mut reservations,
        |frontier, _, _| {
            calls += 1;
            let tag = u32::try_from(frontier.prepared_semantic_identity).unwrap();
            let mut state = convergence_state(tag, true);
            state.next_frontier.prepared_semantic_identity = 1;
            Ok(state)
        },
    )
    .unwrap();

    assert_eq!(calls, 3);
    assert_eq!(resources.measurement_calls, 3);
    assert_eq!(selected.next_frontier.prepared_semantic_identity, 1);
    assert_eq!(
        selected.selected_size,
        LayoutSize::exact_css_pixels(201, 120).unwrap()
    );
}

#[test]
fn convergence_rejects_cycles_exhaustion_and_stable_outward_ink() {
    let mut tracker = ConvergenceTracker::new();
    let mut reservations = prepared_reservations();
    observe(&mut tracker, &mut reservations, convergence_state(0, true)).unwrap();
    observe(&mut tracker, &mut reservations, convergence_state(1, true)).unwrap();
    assert!(
        observe(&mut tracker, &mut reservations, convergence_state(0, true))
            .unwrap_err()
            .to_string()
            .contains("repeated a nonfinal state")
    );

    let mut tracker = ConvergenceTracker::new();
    let mut reservations = prepared_reservations();
    for tag in 0..7 {
        observe(
            &mut tracker,
            &mut reservations,
            convergence_state(tag, true),
        )
        .unwrap();
    }
    assert!(
        observe(&mut tracker, &mut reservations, convergence_state(7, true))
            .unwrap_err()
            .to_string()
            .contains("eight complete passes")
    );

    let mut tracker = ConvergenceTracker::new();
    let mut reservations = prepared_reservations();
    observe(&mut tracker, &mut reservations, convergence_state(0, false)).unwrap();
    assert!(
        observe(&mut tracker, &mut reservations, convergence_state(0, false))
            .unwrap_err()
            .to_string()
            .contains("painted ink leaves the root box")
    );
}

#[test]
fn cumulative_resources_admit_limit_and_refuse_limit_plus_one_independently() {
    let kinds = [
        ResourceKind::CandidateEvaluations,
        ResourceKind::GenerationSlots,
        ResourceKind::GenerationWork,
        ResourceKind::MeasurementCalls,
        ResourceKind::CacheEntries,
        ResourceKind::TopologySlots,
    ];
    for kind in kinds {
        let mut resources = ResourceState::new();
        resources.claim(kind, RESOURCE_LIMIT - 1).unwrap();
        resources.claim(kind, 1).unwrap();
        let error = resources.claim(kind, 1).unwrap_err().to_string();
        assert!(error.contains(kind.name()) && error.contains("limit of 1000000"));
    }
}

fn assert_reservation_refusal(stage: ReservationStage) {
    let mut reservations = ReservationDriver::new();
    let mut values = Vec::<u8>::new();
    if stage != ReservationStage::Prepared {
        reservations
            .reserve_additional(
                ReservationStage::Prepared,
                &mut values,
                0,
                "test preparation",
            )
            .unwrap();
        reservations.finish_prepared();
        reservations.start_pass().unwrap();
    }
    if matches!(
        stage,
        ReservationStage::Candidates | ReservationStage::Convergence | ReservationStage::Final
    ) {
        reservations
            .reserve_additional(
                ReservationStage::Generation,
                &mut values,
                0,
                "test generation",
            )
            .unwrap();
    }
    if matches!(
        stage,
        ReservationStage::Convergence | ReservationStage::Final
    ) {
        reservations
            .reserve_additional(
                ReservationStage::Candidates,
                &mut values,
                0,
                "test candidates",
            )
            .unwrap();
    }
    if stage == ReservationStage::Final {
        reservations
            .reserve_additional(
                ReservationStage::Convergence,
                &mut values,
                0,
                "test convergence",
            )
            .unwrap();
        reservations.mark_stable().unwrap();
    }

    let error = reservations
        .reserve_additional(stage, &mut values, usize::MAX, "deliberate refusal")
        .unwrap_err()
        .to_string();
    assert!(error.contains(&format!("stage {}", stage.number())));
}

#[test]
fn every_reservation_stage_propagates_capacity_refusal() {
    for stage in [
        ReservationStage::Prepared,
        ReservationStage::Generation,
        ReservationStage::Candidates,
        ReservationStage::Convergence,
        ReservationStage::Final,
    ] {
        assert_reservation_refusal(stage);
    }
}

#[test]
fn cache_identity_includes_constraints_assigned_context_and_frontier() {
    let constraints = LayoutConstraints::exact_css_pixels(320, 240).unwrap();
    let frontier = MeasurementFrontier::exact_plot(PlotKind::Scatter);
    let key = MeasurementCacheKey {
        constraints,
        assigned: None,
        frontier,
    };
    let reason = FitReason {
        kind: FitKind::Text,
        component_address: ComponentAddress::root(),
        raw_descriptor_ordinal: 3,
    };
    let accounting = MeasurementAccounting {
        descriptor_count: 4,
        evaluation_count: 4,
        rejected_count: 4,
        admitted_before_dedup_count: 0,
        returned_count: 0,
    };
    let mut cache = MeasurementCache {
        entries: Vec::new(),
    };
    let index = cache.insert(
        key.clone(),
        CachedMeasurement::<u8> {
            outcome: MeasurementOutcome::Unsatisfied(reason.clone()),
            accounting,
        },
    );
    assert_eq!(cache.position(&key), Some(index));
    assert_eq!(cache.entries[index].1.accounting, accounting);
    match &cache.entries[index].1.outcome {
        MeasurementOutcome::Unsatisfied(cached) => assert_eq!(cached, &reason),
        MeasurementOutcome::Candidates(_) => panic!("fit miss was not cached"),
    }

    let mut changed_frontier = key.clone();
    changed_frontier.frontier.transferred_feedback = Box::new([1]);
    assert_eq!(cache.position(&changed_frontier), None);

    let mut changed_context = key;
    changed_context.assigned = Some(AssignedGeometryContext {
        containing_size: LayoutSize::exact_css_pixels(320, 240).unwrap(),
        projection_identity: 7,
        target_geometry: LayoutBox::root(LayoutSize::exact_css_pixels(320, 240).unwrap()),
    });
    assert_eq!(cache.position(&changed_context), None);
}

#[test]
fn exact_root_converges_from_one_cached_measurement_and_promotes_identity() {
    let builds = Cell::new(0);
    let payload = Box::new(17_u8);
    let payload_address = std::ptr::from_ref(payload.as_ref()).addr();

    let selected = plan_exact_root(317, 211, PlotKind::Scatter, || {
        builds.set(builds.get() + 1);
        Ok(payload)
    })
    .unwrap();

    assert_eq!(builds.get(), 1);
    assert_eq!(selected.dimensions(), (317, 211));
    assert_eq!(
        std::ptr::from_ref(selected.plan().as_ref()).addr(),
        payload_address
    );

    let default = plan_exact_root(640, 480, PlotKind::Scatter, || Ok(())).unwrap();
    assert_eq!(default.dimensions(), (640, 480));
}
