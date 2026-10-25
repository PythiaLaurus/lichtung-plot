use super::*;

#[test]
fn fixed_sources_keep_every_original_power_and_refuse_caps_or_text_nonfit() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let view = (1.0, 64.0);
    let range = LatticeRange::new(view, false, 1.0, 2.0).unwrap();
    let positions = positions(
        &range,
        &range.first,
        &range.last,
        &BigInt::from(1_u8),
        false,
        7,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        positions.coordinates,
        [1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0]
    );
    for (index, operand) in positions.operands.iter().enumerate() {
        let Operand::Multiplicative(operand) = operand else {
            panic!("lattice operand")
        };
        assert_eq!(operand.index_text, index.to_string());
    }
    let projection = Projection::new(view, (0.0, 1.0), (0.0, 1800.0, 0.0, 500.0));
    let fixed = (1.0, 2.0, TickRequirement::Fixed);
    assert!(plan(
        view,
        false,
        fixed,
        Some(6),
        &projection,
        AxisSide::Bottom,
        &theme,
        None,
        &crate::layout::fit::ROOT_SCOPE
    )
    .is_err());
    // Linear geometry compresses the first powers: it cannot silently thin them.
    let narrow = Projection::new(view, (0.0, 1.0), (0.0, 80.0, 0.0, 500.0));
    assert!(plan(
        view,
        false,
        fixed,
        None,
        &narrow,
        AxisSide::Bottom,
        &theme,
        None,
        &crate::layout::fit::ROOT_SCOPE
    )
    .is_err());
    let exact = plan(
        view,
        false,
        fixed,
        None,
        &projection,
        AxisSide::Bottom,
        &theme,
        None,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap();
    assert_eq!(
        exact
            .ticks
            .iter()
            .map(|tick| tick.label.as_str())
            .collect::<Vec<_>>(),
        ["1", "2", "4", "8", "16", "32", "64"]
    );
    assert_eq!(exact.numeric.unwrap().members, positions.operands);
    let symbolic_view = (400.0, 402.0);
    let symbolic_projection = Projection::new(symbolic_view, (0.0, 1.0), (0.0, 900.0, 0.0, 500.0));
    let symbolic = plan(
        symbolic_view,
        true,
        (1.0, 10.0, TickRequirement::Fixed),
        None,
        &symbolic_projection,
        AxisSide::Bottom,
        &theme,
        None,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap();
    assert_eq!(
        symbolic
            .ticks
            .iter()
            .map(|tick| tick.label.as_str())
            .collect::<Vec<_>>(),
        ["1e400", "1e401", "1e402"]
    );
}

#[test]
fn preferred_sublattices_obey_the_cap_without_inventing_a_nonlattice_midpoint() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let view = (0.0, 6.0);
    let projection = Projection::new(view, (0.0, 1.0), (0.0, 900.0, 0.0, 500.0));
    let selected = plan(
        view,
        true,
        (1.0, 2.0, TickRequirement::Preferred),
        Some(3),
        &projection,
        AxisSide::Bottom,
        &theme,
        None,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap();
    assert!((1..=3).contains(&selected.ticks.len()));
    assert!(selected.source.ordinal > 0 && selected.source.ordinal < selected.source.cap);
    for operand in &selected.numeric.as_ref().unwrap().members {
        let Operand::Multiplicative(operand) = operand else {
            panic!("original lattice member")
        };
        assert!(operand.index >= BigInt::ZERO && operand.index <= BigInt::from(19));
    }
    let singleton = plan(
        view,
        true,
        (1.0, 2.0, TickRequirement::Preferred),
        Some(1),
        &projection,
        AxisSide::Bottom,
        &theme,
        None,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap();
    let Operand::Multiplicative(operand) = &singleton.numeric.unwrap().members[0] else {
        panic!("lattice singleton")
    };
    assert_eq!(operand.index_text, "9");
    assert_eq!(singleton.ticks[0].label, "512");
}

#[test]
fn empty_and_capped_huge_sources_do_not_materialize_numeric_functions() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let policy =
        crate::NumericFormat::default().set_function(|_| panic!("unreached numeric owner"));
    let format = crate::numeric::source::AxisRequest::for_test(&policy);
    for view in [(-10.0, -1.0), (1.1, 1.2)] {
        let projection = Projection::new(view, (0.0, 1.0), (0.0, 500.0, 0.0, 500.0));
        let empty = plan(
            view,
            false,
            (1.0, 2.0, TickRequirement::Fixed),
            None,
            &projection,
            AxisSide::Bottom,
            &theme,
            Some(&format),
            &crate::layout::fit::ROOT_SCOPE,
        )
        .unwrap();
        assert!(empty.ticks.is_empty() && empty.numeric.is_none());
        crate::Plot::scatter(
            crate::NumericValues::new("x", [view.0, view.1]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .x_limits(view.0, view.1)
        .x_tick_generation(
            crate::TickGeneration::multiplicative(1.0, 2.0, TickRequirement::Fixed).unwrap(),
        )
        .x_tick_format(policy.clone())
        .preflight()
        .unwrap();
    }
    let view = (0.0, f64::MAX);
    let projection = Projection::new(view, (0.0, 1.0), (0.0, 500.0, 0.0, 500.0));
    assert!(plan(
        view,
        true,
        (1.0, 1.0_f64.next_up(), TickRequirement::Fixed),
        Some(10),
        &projection,
        AxisSide::Bottom,
        &theme,
        Some(&format),
        &crate::layout::fit::ROOT_SCOPE
    )
    .is_err());
    assert!(crate::Plot::scatter(
        crate::NumericValues::new("x", [1.0, 64.0]),
        crate::NumericValues::new("y", [0.0, 1.0])
    )
    .x_limits(1.0, 64.0)
    .x_tick_generation(
        crate::TickGeneration::multiplicative(1.0, 2.0, TickRequirement::Fixed)
            .unwrap()
            .maximum_count(Some(6))
            .unwrap()
    )
    .x_tick_format(policy)
    .preflight()
    .is_err());
}
