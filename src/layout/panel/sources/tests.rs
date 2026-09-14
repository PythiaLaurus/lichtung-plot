#![allow(
    clippy::float_cmp,
    reason = "source-policy fixtures assert prescribed binary64 identities"
)]
use super::super::projection::{projection_key, ratio_position};
use super::*;

#[test]
fn resource_audit_numeric_axis_ticket_charges_one_owner_and_refusal_never_coarsens() {
    let theme = crate::Theme::Publication.resolve();
    let environment = theme.prepared_font_environment().unwrap();
    let resources = environment.generation_resources();
    let projection = Projection::new((0.0, 1.0), (0.0, 1.0), (0.0, 1000.0, 0.0, 1000.0));
    let scope = crate::layout::fit::FitScope::root();
    let first = numeric_tick_plan(
        (0.0, 1.0),
        None,
        false,
        &projection,
        AxisSide::Bottom,
        &theme,
        false,
        &scope,
    )
    .unwrap();
    let calls = resources.call_cache_usage().0;
    let admission = environment.claim_measurement_call().unwrap();
    let second = NumericTickPlan::measure_in_call(
        (0.0, 1.0),
        None,
        false,
        &projection,
        AxisSide::Bottom,
        &theme,
        false,
        &scope,
        &admission,
    )
    .unwrap();
    assert_eq!(first.ticks, second.ticks);
    assert_eq!(resources.call_cache_usage().0, 2 * calls);
    resources.claim_calls(1_000_000 - 2 * calls).unwrap();
    let before = (
        resources.usage().unwrap(),
        resources.byte_usage().unwrap(),
        resources.call_cache_usage(),
    );
    crate::font::canonical::FONT_SHAPE_CALLS.with(|count| count.set(0));
    let result = numeric_tick_plan(
        (0.0, 1.0),
        None,
        false,
        &projection,
        AxisSide::Bottom,
        &theme,
        false,
        &scope,
    );
    assert!(matches!(
        result,
        Err(crate::layout::fit::PlanError::Fatal(_))
    ));
    assert_eq!(
        (
            resources.usage().unwrap(),
            resources.byte_usage().unwrap(),
            resources.call_cache_usage()
        ),
        before
    );
    crate::font::canonical::FONT_SHAPE_CALLS.with(|count| assert_eq!(count.get(), 0));
}

#[test]
fn six_ch_seed_has_quantized_boundaries_and_a_ten_thousand_bound() {
    assert_eq!(upper_seed_count(0.0, 1024).unwrap(), 2);
    assert_eq!(upper_seed_count(12.0 - 1.0 / 1024.0, 1024).unwrap(), 2);
    assert_eq!(upper_seed_count(12.0, 1024).unwrap(), 3);
    assert_eq!(upper_seed_count(60_000.0, 1024).unwrap(), 10_000);
    assert_eq!(upper_seed_count(2.0 / 1024.0, 0).unwrap(), 3);
    assert!(upper_seed_count(f64::INFINITY, 1024).is_err());
    assert!(upper_seed_count(1.0, i64::MAX).is_err());
}

#[test]
fn common_step_admits_complete_counts_and_exact_integral_indices() {
    assert_eq!(indices((0.0, 9_999.0), 1.0), Some((0, 10_000)));
    assert_eq!(indices((0.0, 10_000.0), 1.0), None);
    assert_eq!(indices((0.1, 0.9), 1.0), None);
    assert_eq!(
        indices((MAX_INDEX - 2.0, MAX_INDEX), 1.0),
        Some((9_007_199_254_740_990, 3))
    );
    assert_eq!(indices((MAX_INDEX, MAX_INDEX + 2.0), 1.0), None);
    assert_eq!(
        indices((-MAX_INDEX, -MAX_INDEX + 2.0), 1.0),
        Some((-9_007_199_254_740_992, 3))
    );
    assert_eq!(indices((0.0, 1.0), f64::from_bits(1)), None);
    for step in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(indices((0.0, 1.0), step), None);
    }
}

#[test]
fn common_step_uses_index_products_not_accumulated_addition_or_endpoint_tolerance() {
    let positions = linear_positions((0.0, 1.0), 0.1, None).unwrap().unwrap();
    for (index, position) in positions.iter().enumerate() {
        assert_eq!(position.to_bits(), (index as f64 * 0.1).to_bits());
    }
    assert_eq!(positions.last(), Some(&1.0));
    // Binary64 ceil/floor prescribes an index whose product is outside this
    // domain. The whole source misses, rather than dropping that first tick.
    let step = 0.1;
    let lo = 0.3_f64.next_up();
    if let Some(positions) = linear_positions((lo, 0.9), step, None).unwrap() {
        assert!(positions
            .iter()
            .all(|position| *position >= lo && *position <= 0.9));
    }
}

#[test]
fn adaptive_step_uses_decimal_table_ties_and_scaled_extreme_subtraction() {
    assert_eq!(
        adaptive_positions((0.0, 100.0), 5, None).unwrap().unwrap(),
        [0.0, 25.0, 50.0, 75.0, 100.0]
    );
    assert_eq!(
        adaptive_positions((-f64::MAX, f64::MAX), 5, None)
            .unwrap()
            .unwrap(),
        [-1e308, 0.0, 1e308]
    );
    let adjacent = (1.0, 1.0_f64.next_up());
    assert_eq!(
        adaptive_positions(adjacent, 10_000, None).unwrap().unwrap(),
        [adjacent.0, adjacent.1]
    );
    assert_eq!(decimal_power(-323), "1e-323".parse::<f64>().unwrap());
    assert_eq!(decimal_power(308), 1e308);
    for range in [
        (0.0, f64::from_bits(1)),
        (-f64::MAX, f64::MAX),
        (1e-320, 1e-319),
        (1.0, 1.000_000_000_000_01),
    ] {
        for seed in [1, 2, 10, 10_000] {
            if let Some(positions) = adaptive_positions(range, seed, None).unwrap() {
                assert!((2..=10_000).contains(&positions.len()));
                assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
                assert!(positions
                    .iter()
                    .all(|value| value.is_finite() && *value >= range.0 && *value <= range.1));
            }
        }
    }
}

#[test]
fn adaptive_source_order_is_seed_descending_then_one_midpoint() {
    let theme = crate::Theme::Publication.resolve();
    let environment = theme.prepared_font_environment().unwrap();
    let mut sources = Sources::new((0.0, 10.0), false, None, 4).unwrap();
    assert_eq!(sources.cap, 5);
    for (ordinal, kind) in [
        SourceKind::Adaptive(4),
        SourceKind::Adaptive(3),
        SourceKind::Adaptive(2),
        SourceKind::Adaptive(1),
        SourceKind::Midpoint,
    ]
    .into_iter()
    .enumerate()
    {
        let (source, _) = sources.next(environment).unwrap();
        assert_eq!(
            source,
            TickSource {
                kind,
                ordinal,
                cap: 5
            }
        );
    }
    assert!(sources
        .next(environment)
        .unwrap_err()
        .to_string()
        .contains("cap exhausted"));
}

#[test]
fn additive_sources_are_requested_first_then_complete_decimal_coarsenings() {
    let theme = crate::Theme::Publication.resolve();
    let environment = theme.prepared_font_environment().unwrap();
    let mut sources =
        Sources::new((-7.0, 9.0), false, Some(TickInterval::Additive(1.0)), 100).unwrap();
    for (ordinal, step) in [1.0, 2.0, 3.0, 5.0, 10.0, 20.0, 30.0, 50.0]
        .into_iter()
        .enumerate()
    {
        let (source, positions) = sources.next(environment).unwrap();
        assert_eq!(source.kind, SourceKind::Additive(step));
        assert_eq!(source.ordinal, ordinal);
        assert_eq!(
            positions,
            linear_positions((-7.0, 9.0), step, None).unwrap()
        );
    }
    let (source, positions) = sources.next(environment).unwrap();
    assert_eq!(source.kind, SourceKind::Midpoint);
    assert_eq!(positions.unwrap(), [1.0]);
}

#[test]
fn dense_and_empty_interval_misses_do_not_poison_coarser_sources() {
    let theme = crate::Theme::Publication.resolve();
    let environment = theme.prepared_font_environment().unwrap();
    let mut dense =
        Sources::new((0.0, 10_000.0), false, Some(TickInterval::Additive(1.0)), 2).unwrap();
    assert!(dense.next(environment).unwrap().1.is_none());
    let (source, positions) = dense.next(environment).unwrap();
    assert_eq!(source.kind, SourceKind::Additive(2.0));
    assert_eq!(positions.unwrap().len(), 5001);
    let mut empty = Sources::new((1.1, 1.2), false, Some(TickInterval::Additive(1.0)), 2).unwrap();
    loop {
        let (source, positions) = empty.next(environment).unwrap();
        if source.kind == SourceKind::Midpoint {
            assert_eq!(positions.unwrap(), [f64::midpoint(1.1, 1.2)]);
            break;
        }
        assert!(positions.is_none());
    }
}

#[test]
fn additive_full_binary64_span_stays_within_declared_cap() {
    let theme = crate::Theme::Publication.resolve();
    let environment = theme.prepared_font_environment().unwrap();
    let mut sources = Sources::new(
        (1.0, f64::MAX),
        false,
        Some(TickInterval::Additive(f64::from_bits(1))),
        2,
    )
    .unwrap();
    let mut previous = 0.0;
    loop {
        let (source, _) = sources.next(environment).unwrap();
        assert!(source.ordinal < source.cap);
        match source.kind {
            SourceKind::Additive(step) => {
                assert!(step > previous);
                previous = step;
            }
            SourceKind::Midpoint => break,
            _ => panic!("interval mode introduced an unrelated source"),
        }
    }
}

#[test]
fn logarithmic_interval_vectors_are_divisible_integral_exponents_not_subsequences() {
    let theme = crate::Theme::Publication.resolve();
    let environment = theme.prepared_font_environment().unwrap();
    let mut sources = Sources::new(
        (-400.0, 400.0),
        true,
        Some(TickInterval::Log10Decades(2)),
        2,
    )
    .unwrap();
    for step in [2, 4, 6, 10, 20, 40, 60, 100] {
        let (source, positions) = sources.next(environment).unwrap();
        assert_eq!(source.kind, SourceKind::LogInterval(step));
        let expected = (-323..=308)
            .filter(|exponent| i64::from(*exponent) % i64::try_from(step).unwrap() == 0)
            .map(f64::from)
            .collect::<Vec<_>>();
        assert_eq!(positions.unwrap(), expected);
    }
}

#[test]
fn logarithmic_empty_intersection_charges_one_miss_then_original_midpoint() {
    let theme = crate::Theme::Publication.resolve();
    let environment = theme.prepared_font_environment().unwrap();
    let mut sources = Sources::new(
        (400.0, 500.0),
        true,
        Some(TickInterval::Log10Decades(1)),
        100,
    )
    .unwrap();
    assert_eq!(sources.next(environment).unwrap().1, None);
    let (source, positions) = sources.next(environment).unwrap();
    assert_eq!(source.ordinal, 1);
    assert_eq!(source.kind, SourceKind::Midpoint);
    assert_eq!(positions.unwrap(), [450.0]);
}

#[test]
fn logarithmic_integer_overflow_terminates_without_wrapping() {
    let theme = crate::Theme::Publication.resolve();
    let environment = theme.prepared_font_environment().unwrap();
    let mut sources = Sources::new(
        (-3.0, 3.0),
        true,
        Some(TickInterval::Log10Decades(u64::MAX)),
        100,
    )
    .unwrap();
    assert_eq!(sources.next(environment).unwrap().1.unwrap(), [0.0]);
    assert_eq!(
        sources.next(environment).unwrap().0.kind,
        SourceKind::Midpoint
    );
}

#[test]
fn logarithmic_automatic_order_has_full_integrals_then_even_indices() {
    let theme = crate::Theme::Publication.resolve();
    let environment = theme.prepared_font_environment().unwrap();
    let mut sources = Sources::new((-3.0, 3.0), true, None, 3).unwrap();
    assert_eq!(
        sources.next(environment).unwrap().1.unwrap(),
        [-3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0]
    );
    assert_eq!(
        sources.next(environment).unwrap().1.unwrap(),
        [-3.0, 0.0, 3.0]
    );
    assert_eq!(sources.next(environment).unwrap().1.unwrap(), [-3.0, 3.0]);
    assert_eq!(
        sources.next(environment).unwrap().0.kind,
        SourceKind::Midpoint
    );
}

#[test]
fn source_semantics_reject_before_fonts() {
    for (range, log, interval) in [
        ((f64::NAN, 1.0), false, None),
        ((1.0, 1.0), false, None),
        ((0.0, 1.0), false, Some(TickInterval::Additive(0.0))),
        (
            (0.0, 1.0),
            false,
            Some(TickInterval::Additive(f64::INFINITY)),
        ),
        ((0.0, 1.0), true, Some(TickInterval::Log10Decades(0))),
        ((0.0, 1.0), true, Some(TickInterval::Additive(1.0))),
        ((0.0, 1.0), false, Some(TickInterval::Log10Decades(1))),
    ] {
        let theme = crate::Theme::Publication.resolve();
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let projection = Projection::new(range, range, (0.0, 1000.0, 0.0, 1000.0));
        assert!(numeric_tick_plan(
            range,
            interval,
            log,
            &projection,
            AxisSide::Bottom,
            &theme,
            false,
            &crate::layout::fit::ROOT_SCOPE,
        )
        .is_err());
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}

#[test]
fn source_admission_refuses_before_position_storage_or_ordinal_advancement() {
    let theme = crate::Theme::Publication.resolve();
    let environment = theme.prepared_font_environment().unwrap();
    let used = environment.generation_resources().usage().unwrap().0;
    environment
        .claim_text_generation_slots(1_000_000 - used)
        .unwrap();
    let before = environment.resource_usage().unwrap().font_bytes;
    let mut sources = Sources::new((0.0, 1.0), false, None, 10_000).unwrap();
    assert!(sources
        .next(environment)
        .unwrap_err()
        .to_string()
        .contains("axis source admission"));
    assert_eq!(sources.ordinal, 0);
    assert_eq!(environment.resource_usage().unwrap().font_bytes, before);
}

#[test]
fn dense_source_miss_has_no_position_allocation_charge() {
    let theme = crate::Theme::Publication.resolve();
    let environment = theme.prepared_font_environment().unwrap();
    let before = environment.resource_usage().unwrap().font_bytes;
    assert!(linear_positions((0.0, 10_000.0), 1.0, Some(environment))
        .unwrap()
        .is_none());
    assert_eq!(environment.resource_usage().unwrap().font_bytes, before);
}

#[test]
fn projection_keys_are_exact_at_endpoints_midpoints_and_subnormal_fractions() {
    assert_eq!(projection_key(0.0, (0.0, 1.0)), Some(0));
    assert_eq!(projection_key(1.0, (0.0, 1.0)), Some(u64::MAX));
    assert_eq!(projection_key(0.5, (0.0, 1.0)), Some(1 << 63));
    assert_eq!(projection_key(f64::from_bits(1), (0.0, 1.0)), Some(0));
    assert_eq!(projection_key(0.0, (-f64::MAX, f64::MAX)), Some(1 << 63));
    assert_eq!(projection_key(2.0, (0.0, 1.0)), None);
    assert_eq!(projection_key(1.0, (1.0, 1.0)), None);
    assert_eq!(projection_key(1.0, (f64::NAN, 1.0)), None);
    assert_eq!(projection_key(f64::INFINITY, (0.0, f64::INFINITY)), None);
    assert_eq!(ratio_position(0, 1024, 0, u128::from(u64::MAX)), Some(0));
    assert_eq!(
        ratio_position(0, 1024, u128::from(u64::MAX), u128::from(u64::MAX)),
        Some(1024)
    );
    assert_eq!(
        ratio_position(0, 1024, 1 << 63, u128::from(u64::MAX)),
        Some(512)
    );
    assert_eq!(ratio_position(1, 1, 1, 2), Some(2));
    assert_eq!(ratio_position(-1, 1, 1, 2), Some(0));
    assert_eq!(ratio_position(0, 1, 1, 2), Some(0));
    assert_eq!(ratio_position(i64::MAX, 1, 1, 1), None);
    assert_eq!(ratio_position(i64::MIN, i64::MAX, 1, 1), Some(-1));
    assert_eq!(ratio_position(0, -1, 1, 1), None);
    assert_eq!(ratio_position(0, 1, 1, 0), None);
    assert_eq!(ratio_position(0, i64::MAX, u128::MAX, 1), None);
}

#[test]
fn requested_interval_fit_wins_and_dense_requested_source_coarsens_locally() {
    let theme = crate::Theme::Publication.resolve();
    let projection = Projection::new((0.0, 100.0), (0.0, 100.0), (0.0, 1000.0, 0.0, 1000.0));
    let plan = numeric_tick_plan(
        (0.0, 100.0),
        Some(TickInterval::Additive(10.0)),
        false,
        &projection,
        AxisSide::Bottom,
        &theme,
        false,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap();
    assert_eq!(plan.source.kind, SourceKind::Additive(10.0));
    assert_eq!(plan.source.ordinal, 0);
    assert_eq!(plan.ticks.len(), 11);
    let dense = numeric_tick_plan(
        (0.0, 100.0),
        Some(TickInterval::Additive(0.001)),
        false,
        &projection,
        AxisSide::Bottom,
        &theme,
        false,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap();
    let SourceKind::Additive(step) = dense.source.kind else {
        panic!("a complete coarser interval should fit")
    };
    assert!(step > 0.001 && dense.source.ordinal > 0);
    assert_eq!(
        dense
            .ticks
            .iter()
            .map(|tick| tick.position)
            .collect::<Vec<_>>(),
        linear_positions((0.0, 100.0), step, None).unwrap().unwrap()
    );
}

#[test]
fn midpoint_is_conditional_and_must_itself_fit() {
    let theme = crate::Theme::Publication.resolve();
    let scope = crate::layout::fit::FitScope::root()
        .child(crate::layout::AddressRole::FacetCell, 3)
        .unwrap();
    for width in [1.0, 100.0] {
        let projection = Projection::new((1.1, 1.2), (0.0, 1.0), (0.0, width, 0.0, 100.0));
        let result = numeric_tick_plan(
            (1.1, 1.2),
            Some(TickInterval::Additive(1.0)),
            false,
            &projection,
            AxisSide::Bottom,
            &theme,
            false,
            &scope,
        );
        if width == 1.0 {
            let Err(crate::layout::fit::PlanError::Fit(failure)) = result else {
                panic!("terminal geometry must remain a typed fit miss")
            };
            assert_eq!(
                failure.reason,
                scope
                    .child(crate::layout::AddressRole::AxisBottom, 0)
                    .unwrap()
                    .failure(crate::layout::fit::FitKind::Tick, "")
                    .reason
            );
        } else {
            assert_eq!(result.unwrap().source.kind, SourceKind::Midpoint);
        }
    }
}

#[test]
fn numeric_source_resource_and_font_failures_remain_fatal() {
    use crate::layout::fit::{PlanError, ROOT_SCOPE};
    let theme = crate::Theme::Publication.resolve();
    let environment = theme.prepared_font_environment().unwrap();
    let used = environment.generation_resources().usage().unwrap().0;
    environment
        .claim_text_generation_slots(1_000_000 - used)
        .unwrap();
    let projection = Projection::new((1.1, 1.2), (0.0, 1.0), (0.0, 1.0, 0.0, 100.0));
    assert!(matches!(
        numeric_tick_plan(
            (1.1, 1.2),
            None,
            false,
            &projection,
            AxisSide::Bottom,
            &theme,
            false,
            &ROOT_SCOPE
        ),
        Err(PlanError::Fatal(_))
    ));
    let mut theme = crate::Theme::Publication.resolve();
    theme.typography.tick_label.font_size = 0;
    assert!(matches!(
        numeric_tick_plan(
            (1.1, 1.2),
            None,
            false,
            &projection,
            AxisSide::Bottom,
            &theme,
            false,
            &ROOT_SCOPE
        ),
        Err(PlanError::Fatal(_))
    ));
    assert!(matches!(
        numeric_tick_plan(
            (1.1, 1.1),
            None,
            false,
            &projection,
            AxisSide::Bottom,
            &theme,
            false,
            &ROOT_SCOPE
        ),
        Err(PlanError::Fatal(_))
    ));
}

#[test]
fn generated_labels_are_unique_and_covered_by_the_closed_census() {
    let theme = crate::Theme::Publication.resolve();
    for (range, log) in [
        ((1.000_000_000_000_01, 1.000_000_000_000_09), false),
        ((-400.0, 400.0), true),
        ((308.1, 308.2), true),
        ((-324.0, -323.5), true),
    ] {
        let projection = Projection::new(range, range, (0.0, 2000.0, 0.0, 1000.0));
        let plan = numeric_tick_plan(
            range,
            None,
            log,
            &projection,
            AxisSide::Bottom,
            &theme,
            false,
            &crate::layout::fit::ROOT_SCOPE,
        )
        .unwrap();
        for (index, tick) in plan.ticks.iter().enumerate() {
            assert!(tick
                .label
                .chars()
                .all(|scalar| NUMERIC_ALPHABET.contains(&scalar)));
            assert!(plan.ticks[..index]
                .iter()
                .all(|previous| previous.label != tick.label));
            if log
                && matches!(
                    plan.source.kind,
                    SourceKind::Adaptive(_) | SourceKind::Midpoint
                )
            {
                assert!(tick.label.starts_with("10^"));
            }
        }
    }
    assert_eq!(
        format_label(-323.0, true, true, 0, None).unwrap(),
        "1.00e-323"
    );
    assert_eq!(
        format_label(308.0, true, true, 0, None).unwrap(),
        "1.00e308"
    );
    assert_eq!(super::super::range_ticks::format_tick(-0.0), "0");
}

#[test]
fn numeric_formatting_uses_nearest_even_and_declared_thresholds() {
    for (value, expected) in [
        (-0.0, "0"),
        (0.01, "0.01"),
        (0.001, "1.00e-3"),
        (1.25, "1.2"),
        (1.75, "1.8"),
        (100.5, "100"),
        (101.5, "102"),
        (9999.0, "9999"),
        (10_000.0, "10k"),
        (1_000_000.0, "1M"),
        (1e12, "1.00e12"),
        (-1e12, "-1.00e12"),
    ] {
        assert_eq!(
            format_label(value, false, false, 0, None).unwrap(),
            expected
        );
    }
    assert_eq!(
        format_label(-0.0, false, false, 17, None).unwrap(),
        "0.00000000000000000e0"
    );
    assert_eq!(format_label(0.5, true, false, 0, None).unwrap(), "10^0.5");
}

#[test]
fn exact_source_overrides_intervals_and_provenance_survives_layout_cloning() {
    let theme = crate::Theme::Publication.resolve();
    let x = super::super::AxisSpec {
        provenance: super::super::RangeProvenance::Configured,
        limits: Some((0.0, 10.0)),
        interval: Some(TickInterval::Additive(1.0)),
        exact_ticks: Some(vec![AxisTick::new(1.0, ""), AxisTick::new(9.0, "same")]),
    };
    let y = super::super::AxisSpec {
        provenance: super::super::RangeProvenance::Configured,
        limits: Some((0.0, 10.0)),
        interval: Some(TickInterval::Additive(2.0)),
        exact_ticks: None,
    };
    let layout = super::super::Layout::new_with_axis_specs_in_panel_box(
        (0.0, 10.0),
        (0.0, 10.0),
        crate::layout::PanelBox::new(0.0, 1000.0, 0.0, 1000.0),
        String::new(),
        String::new(),
        &x,
        &y,
        &theme,
    )
    .unwrap();
    assert_eq!(layout.x_axis.source, TickSource::EXACT);
    assert_eq!(layout.x_axis.ticks, x.exact_ticks.unwrap());
    assert_eq!(layout.y_axis.source.kind, SourceKind::Additive(2.0));
    let cloned = layout.clone();
    assert_eq!(cloned.x_axis.source, layout.x_axis.source);
    assert_eq!(cloned.y_axis.source, layout.y_axis.source);
}
