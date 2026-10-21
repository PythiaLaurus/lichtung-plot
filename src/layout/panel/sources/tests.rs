#![allow(
    clippy::float_cmp,
    reason = "source-policy fixtures assert prescribed binary64 identities"
)]
use super::super::projection::{projection_key, ratio_position};
use super::*;

#[test]
fn repeated_numeric_axis_selection_preserves_ticks_without_reshaping() {
    let theme = crate::Theme::publication().resolve().unwrap();

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
        None,
        &scope,
    )
    .unwrap();

    let second = NumericTickPlan::measure_selected_source(
        (0.0, 1.0),
        None,
        false,
        &projection,
        AxisSide::Bottom,
        &theme,
        false,
        None,
        &scope,
    )
    .unwrap();
    assert_eq!(first.ticks, second.ticks);

    crate::font::canonical::FONT_SHAPE_CALLS.with(|count| count.set(0));
    let result = numeric_tick_plan(
        (0.0, 1.0),
        None,
        false,
        &projection,
        AxisSide::Bottom,
        &theme,
        false,
        None,
        &scope,
    );
    assert_eq!(result.unwrap().ticks, first.ticks);

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
    let positions = linear_positions((0.0, 1.0), 0.1, MAX_TICKS)
        .unwrap()
        .unwrap();
    for (index, position) in positions.iter().enumerate() {
        assert_eq!(position.to_bits(), (index as f64 * 0.1).to_bits());
    }
    assert_eq!(positions.last(), Some(&1.0));
    // Binary64 ceil/floor prescribes an index whose product is outside this
    // domain. The whole source misses, rather than dropping that first tick.
    let step = 0.1;
    let lo = 0.3_f64.next_up();
    if let Some(positions) = linear_positions((lo, 0.9), step, MAX_TICKS).unwrap() {
        assert!(positions
            .iter()
            .all(|position| *position >= lo && *position <= 0.9));
    }
}

#[test]
fn adaptive_step_uses_decimal_table_ties_and_scaled_extreme_subtraction() {
    assert_eq!(
        adaptive_positions((0.0, 100.0), 5, MAX_TICKS)
            .unwrap()
            .unwrap(),
        [0.0, 25.0, 50.0, 75.0, 100.0]
    );
    assert_eq!(
        adaptive_positions((-f64::MAX, f64::MAX), 5, MAX_TICKS)
            .unwrap()
            .unwrap(),
        [-1e308, 0.0, 1e308]
    );
    let adjacent = (1.0, 1.0_f64.next_up());
    assert_eq!(
        adaptive_positions(adjacent, 10_000, MAX_TICKS)
            .unwrap()
            .unwrap(),
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
            if let Some(positions) = adaptive_positions(range, seed, MAX_TICKS).unwrap() {
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
fn count_targets_allow_nice_deviation_and_maxima_filter_before_formatting() {
    use crate::orchestrator::measurements::Counter;
    let mut target = Sources::new(
        (0.0, 1.0),
        false,
        Some(TickInterval::target(4).unwrap()),
        1000,
    )
    .unwrap();
    let (source, positions) = target.next().unwrap();
    assert_eq!(source.kind, SourceKind::Adaptive(4));
    assert_eq!(positions.unwrap(), [0.0, 0.5, 1.0]);
    let mut five =
        Sources::new((0.0, 1.0), false, Some(TickInterval::target(5).unwrap()), 2).unwrap();
    assert_eq!(five.next().unwrap().1.unwrap(), [0.0, 0.25, 0.5, 0.75, 1.0]);
    assert!(adaptive_positions((0.0, 1.0), 5, 4).unwrap().is_none());
    let adjacent = (1.0, 1.0_f64.next_up());
    assert!(adaptive_positions(adjacent, 5, 1).unwrap().is_none());
    assert_eq!(
        adaptive_positions(adjacent, 5, 2).unwrap().unwrap(),
        [adjacent.0, adjacent.1]
    );
    let theme = crate::Theme::publication().resolve().unwrap();
    let projection = Projection::new((0.0, 1.0), (0.0, 1.0), (0.0, 2000.0, 0.0, 1000.0));
    crate::orchestrator::measurements::start();
    let selected = numeric_tick_plan(
        (0.0, 1.0),
        Some(TickInterval::maximum(4).unwrap()),
        false,
        &projection,
        AxisSide::Bottom,
        &theme,
        false,
        None,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap();
    let work = crate::orchestrator::measurements::finish();
    assert_eq!(
        selected
            .ticks
            .iter()
            .map(|tick| tick.position)
            .collect::<Vec<_>>(),
        [0.0, 0.5, 1.0]
    );
    assert!(work.counters[Counter::TickSource as usize] > 1);
    assert_eq!(work.counters[Counter::TickDistinctList as usize], 1);
    assert_eq!(work.counters[Counter::TickMetricLookup as usize], 3);
}

#[test]
fn additive_sources_are_requested_first_then_complete_decimal_coarsenings() {
    let mut sources = Sources::new(
        (-7.0, 9.0),
        false,
        Some(TickInterval::additive(1.0).unwrap()),
        100,
    )
    .unwrap();
    for (ordinal, step) in [1.0, 2.0, 3.0, 5.0, 10.0, 20.0, 30.0, 50.0]
        .into_iter()
        .enumerate()
    {
        let (source, positions) = sources.next().unwrap();
        assert_eq!(source.kind, SourceKind::Additive(step));
        assert_eq!(source.ordinal, ordinal);
        assert_eq!(
            positions,
            linear_positions((-7.0, 9.0), step, MAX_TICKS).unwrap()
        );
    }
    let (source, positions) = sources.next().unwrap();
    assert_eq!(source.kind, SourceKind::Midpoint);
    assert_eq!(positions.unwrap(), [1.0]);
}

#[test]
fn dense_and_empty_interval_misses_do_not_poison_coarser_sources() {
    let mut dense = Sources::new(
        (0.0, 10_000.0),
        false,
        Some(TickInterval::additive(1.0).unwrap()),
        2,
    )
    .unwrap();
    assert!(dense.next().unwrap().1.is_none());
    let (source, positions) = dense.next().unwrap();
    assert_eq!(source.kind, SourceKind::Additive(2.0));
    assert_eq!(positions.unwrap().len(), 5001);
    let mut empty = Sources::new(
        (1.1, 1.2),
        false,
        Some(TickInterval::additive(1.0).unwrap()),
        2,
    )
    .unwrap();
    loop {
        let (source, positions) = empty.next().unwrap();
        if source.kind == SourceKind::Midpoint {
            assert_eq!(positions.unwrap(), [f64::midpoint(1.1, 1.2)]);
            break;
        }
        assert!(positions.is_none());
    }
}

#[test]
fn additive_full_binary64_span_stays_within_declared_cap() {
    let mut sources = Sources::new(
        (1.0, f64::MAX),
        false,
        Some(TickInterval::additive(f64::from_bits(1)).unwrap()),
        2,
    )
    .unwrap();
    let mut previous = 0.0;
    loop {
        let (source, _) = sources.next().unwrap();
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
    let mut sources = Sources::new(
        (-400.0, 400.0),
        true,
        Some(TickInterval::log10_decades(2).unwrap()),
        2,
    )
    .unwrap();
    for step in [2, 4, 6, 10, 20, 40, 60, 100] {
        let (source, positions) = sources.next().unwrap();
        assert_eq!(source.kind, SourceKind::LogInterval(step));
        let expected = (-323..=308)
            .filter(|exponent| i64::from(*exponent) % i64::try_from(step).unwrap() == 0)
            .map(f64::from)
            .collect::<Vec<_>>();
        assert_eq!(positions.unwrap(), expected);
    }
}

#[test]
fn logarithmic_empty_intersection_falls_back_to_the_original_midpoint() {
    let mut sources = Sources::new(
        (400.0, 500.0),
        true,
        Some(TickInterval::log10_decades(1).unwrap()),
        100,
    )
    .unwrap();
    assert_eq!(sources.next().unwrap().1, None);
    let (source, positions) = sources.next().unwrap();
    assert_eq!(source.ordinal, 1);
    assert_eq!(source.kind, SourceKind::Midpoint);
    assert_eq!(positions.unwrap(), [450.0]);
}

#[test]
fn maximum_public_decade_step_widens_and_terminates_without_wrapping() {
    let mut sources = Sources::new(
        (-3.0, 3.0),
        true,
        Some(TickInterval::log10_decades(u32::MAX).unwrap()),
        100,
    )
    .unwrap();
    for factor in [1, 2, 3, 5] {
        let (source, positions) = sources.next().unwrap();
        assert_eq!(
            source.kind,
            SourceKind::LogInterval(u64::from(u32::MAX) * factor)
        );
        assert_eq!(positions.unwrap(), [0.0]);
    }
    assert_eq!(sources.next().unwrap().0.kind, SourceKind::Midpoint);
}

#[test]
fn logarithmic_count_sources_preserve_automatic_order_but_target_the_initial_subset() {
    let mut sources = Sources::new((-3.0, 3.0), true, None, 3).unwrap();
    assert_eq!(
        sources.next().unwrap().1.unwrap(),
        [-3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0]
    );
    assert_eq!(sources.next().unwrap().1.unwrap(), [-3.0, 0.0, 3.0]);
    assert_eq!(sources.next().unwrap().1.unwrap(), [-3.0, 3.0]);
    assert_eq!(sources.next().unwrap().0.kind, SourceKind::Midpoint);
    let mut target = Sources::new(
        (-3.0, 3.0),
        true,
        Some(TickInterval::target(4).unwrap()),
        1000,
    )
    .unwrap();
    assert_eq!(target.cap, 4);
    let (source, positions) = target.next().unwrap();
    assert_eq!(source.kind, SourceKind::LogSubset(4));
    assert_eq!(positions.unwrap(), [-3.0, -1.0, 1.0, 3.0]);
    assert_eq!(target.next().unwrap().1.unwrap(), [-3.0, 0.0, 3.0]);
    assert_eq!(target.next().unwrap().1.unwrap(), [-3.0, 3.0]);
    assert_eq!(target.next().unwrap().1.unwrap(), [0.0]);
    assert!(target.next().is_err());
    let mut maximum = Sources::new(
        (-3.0, 3.0),
        true,
        Some(TickInterval::maximum(4).unwrap()),
        7,
    )
    .unwrap();
    let (source, positions) = maximum.next().unwrap();
    assert_eq!(source.kind, SourceKind::LogIntegral);
    assert!(positions.is_none());
    for desired in [7, 6, 5] {
        let (source, positions) = maximum.next().unwrap();
        assert_eq!(source.kind, SourceKind::LogSubset(desired));
        assert!(positions.is_none());
    }
    assert_eq!(maximum.next().unwrap().1.unwrap(), [-3.0, -1.0, 1.0, 3.0]);
}

#[test]
fn source_semantics_reject_before_fonts() {
    let discoveries = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
    for count in [0, 10_001, u32::MAX] {
        assert!(matches!(TickInterval::target(count), Err(Error::Data(_))));
        assert!(matches!(TickInterval::maximum(count), Err(Error::Data(_))));
    }
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        discoveries
    );
    for (range, log, interval) in [
        ((f64::NAN, 1.0), false, None),
        ((1.0, 1.0), false, None),
        ((0.0, 1.0), true, Some(TickInterval::additive(1.0).unwrap())),
        (
            (0.0, 1.0),
            false,
            Some(TickInterval::log10_decades(1).unwrap()),
        ),
    ] {
        let theme = crate::Theme::publication().resolve().unwrap();
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
            None,
            &crate::layout::fit::ROOT_SCOPE,
        )
        .is_err());
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let error = crate::Plot::trajectory(
        crate::CategoryValues::new("condition", ["a", "b"]).unwrap(),
        crate::NumericValues::new("response", [1.0, 2.0]),
        crate::CategoryValues::new("group", ["g", "g"]).unwrap(),
    )
    .x_tick_format(crate::numeric::format::NumericFormat::default())
    .preflight()
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("categories do not support a numeric tick format"));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
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
    use crate::numeric::{
        decimal::NumericNotation,
        format::{NumericFormat, NumericPrecision},
    };
    let theme = crate::Theme::publication().resolve().unwrap();
    let projection = Projection::new((0.0, 100.0), (0.0, 100.0), (0.0, 1000.0, 0.0, 1000.0));
    let plan = numeric_tick_plan(
        (0.0, 100.0),
        Some(TickInterval::additive(10.0).unwrap()),
        false,
        &projection,
        AxisSide::Bottom,
        &theme,
        false,
        None,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap();
    assert_eq!(plan.source.kind, SourceKind::Additive(10.0));
    assert_eq!(plan.source.ordinal, 0);
    assert_eq!(plan.ticks.len(), 11);
    let dense = numeric_tick_plan(
        (0.0, 100.0),
        Some(TickInterval::additive(0.001).unwrap()),
        false,
        &projection,
        AxisSide::Bottom,
        &theme,
        false,
        None,
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
        linear_positions((0.0, 100.0), step, MAX_TICKS)
            .unwrap()
            .unwrap()
    );
    let projection = Projection::new((0.0, 0.04), (0.0, 1.0), (0.0, 1000.0, 0.0, 1000.0));
    for (precision, expected_ordinal, expected_digits) in [
        (NumericPrecision::Fractional(0), false, 0),
        (
            NumericPrecision::AdaptiveFractional { min: 0, max: 2 },
            true,
            2,
        ),
    ] {
        let format = NumericFormat::new(NumericNotation::Decimal, precision).unwrap();
        let selected = numeric_tick_plan(
            (0.0, 0.04),
            Some(TickInterval::additive(0.01).unwrap()),
            false,
            &projection,
            AxisSide::Bottom,
            &theme,
            false,
            Some(&format),
            &crate::layout::fit::ROOT_SCOPE,
        )
        .unwrap();
        assert_eq!(selected.source.ordinal == 0, expected_ordinal);
        assert_eq!(selected.source.precision, Some(expected_digits));
        if expected_ordinal {
            assert_eq!(
                selected
                    .ticks
                    .iter()
                    .map(|tick| tick.label.as_str())
                    .collect::<Vec<_>>(),
                ["0.00", "0.01", "0.02", "0.03", "0.04"]
            );
        }
    }
}

#[test]
fn midpoint_is_conditional_and_must_itself_fit() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let scope = crate::layout::fit::FitScope::root()
        .child(crate::layout::AddressRole::FacetCell, 3)
        .unwrap();
    for width in [1.0, 100.0] {
        for selection in [
            TickInterval::additive(1.0).unwrap(),
            TickInterval::target(1).unwrap(),
            TickInterval::maximum(1).unwrap(),
        ] {
            let projection = Projection::new((1.1, 1.2), (0.0, 1.0), (0.0, width, 0.0, 100.0));
            let result = numeric_tick_plan(
                (1.1, 1.2),
                Some(selection),
                false,
                &projection,
                AxisSide::Bottom,
                &theme,
                false,
                None,
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
}

#[test]
fn numeric_source_invalid_range_and_font_failures_remain_fatal() {
    use crate::layout::fit::{PlanError, ROOT_SCOPE};
    let projection = Projection::new((1.1, 1.2), (0.0, 1.0), (0.0, 1.0, 0.0, 100.0));
    let mut theme = crate::Theme::publication().resolve().unwrap();
    theme.typography.axes.tick_label.font_size = 0;
    assert!(matches!(
        numeric_tick_plan(
            (1.1, 1.2),
            None,
            false,
            &projection,
            AxisSide::Bottom,
            &theme,
            false,
            None,
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
            None,
            &ROOT_SCOPE
        ),
        Err(PlanError::Fatal(_))
    ));
}

#[test]
fn generated_labels_are_unique_and_covered_by_the_closed_census() {
    let theme = crate::Theme::publication().resolve().unwrap();
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
            None,
            &crate::layout::fit::ROOT_SCOPE,
        )
        .unwrap();
        for (index, tick) in plan.ticks.iter().enumerate() {
            assert!(tick.label.chars().all(|scalar| if log {
                FOREST_NUMERIC_ALPHABET
            } else {
                NUMERIC_ALPHABET
            }
            .contains(&scalar)));
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
    for (exponent, expected) in [
        (-323.0, "1.00e-323"),
        (-2.0, "0.01"),
        (-1.0, "0.1"),
        (-0.0, "1"),
        (1.0, "10"),
        (2.0, "100"),
        (3.0, "1000"),
        (4.0, "10k"),
        (5.0, "100k"),
        (6.0, "1M"),
        (7.0, "10M"),
        (8.0, "100M"),
        (9.0, "1000M"),
        (10.0, "10000M"),
        (11.0, "100000M"),
        (308.0, "1.00e308"),
    ] {
        assert_eq!(format_label(exponent, true, true, 0).unwrap(), expected);
    }
    assert_eq!(super::super::range_ticks::format_tick(-0.0), "0");
    let range = (0.5_f64.next_down(), 0.5_f64.next_up());
    let projection = Projection::new(range, (0.0, 1.0), (0.0, 800.0, 0.0, 100.0));
    let configured = crate::numeric::format::NumericFormat::default();
    let selected = numeric_tick_plan(
        range,
        None,
        true,
        &projection,
        AxisSide::Bottom,
        &theme,
        false,
        Some(&configured),
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap();
    assert_eq!(selected.source.kind, SourceKind::Midpoint);
    assert_eq!(selected.ticks[0].label, "3.16");
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
        (12_350.0, "12.3k"),
        (999_999.0, "1000k"),
        (1_000_000.0, "1M"),
        (1_150_000.0, "1.1M"),
        (1e12, "1.00e12"),
        (-1e12, "-1.00e12"),
        (f64::MAX, "1.80e308"),
        (f64::from_bits(1), "4.94e-324"),
    ] {
        assert_eq!(format_label(value, false, false, 0).unwrap(), expected);
        assert_eq!(format_guide_label(value, 0).unwrap(), expected);
    }
    assert_eq!(
        format_label(-0.0, false, false, 17).unwrap(),
        "0.00000000000000000e0"
    );
    assert_eq!(format_label(0.5, true, false, 0).unwrap(), "10^0.5");

    // Literal default boundaries and independent Rust scientific spelling verify
    // the actual axis/guide profiles, including retries through fractional 17.
    for (threshold, labels) in [
        (0.01_f64, ["1.00e-2", "0.01", "0.01"]),
        (1.0, ["1", "1", "1"]),
        (100.0, ["100", "100", "100"]),
        (10_000.0, ["10000", "10k", "10k"]),
        (1_000_000.0, ["1000k", "1M", "1M"]),
        (1e12, ["1000000M", "1.00e12", "1.00e12"]),
    ] {
        for (magnitude, label) in [threshold.next_down(), threshold, threshold.next_up()]
            .into_iter()
            .zip(labels)
        {
            for (value, expected) in [
                (magnitude, label.to_owned()),
                (-magnitude, format!("-{label}")),
            ] {
                assert_eq!(format_label(value, false, false, 0).unwrap(), expected);
                assert_eq!(format_guide_label(value, 0).unwrap(), expected);
                for precision in 2..=17 {
                    let expected = format!("{value:.precision$e}");
                    assert_eq!(
                        format_label(value, false, false, precision).unwrap(),
                        expected
                    );
                    assert_eq!(format_guide_label(value, precision).unwrap(), expected);
                }
            }
        }
    }
}

#[test]
fn exact_source_overrides_intervals_and_provenance_survives_layout_cloning() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let x = super::super::AxisSpec {
        provenance: super::super::RangeProvenance::Configured,
        limits: Some((0.0, 10.0)),
        interval: Some(TickInterval::additive(1.0).unwrap()),
        format: None,
        exact_ticks: Some(vec![AxisTick::new(1.0, ""), AxisTick::new(9.0, "same")]),
    };
    let y = super::super::AxisSpec {
        provenance: super::super::RangeProvenance::Configured,
        limits: Some((0.0, 10.0)),
        interval: Some(TickInterval::additive(2.0).unwrap()),
        format: None,
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
