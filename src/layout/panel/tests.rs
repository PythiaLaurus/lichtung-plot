use super::*;

#[test]
fn continuous_auto_range_pads_when_data_hits_nice_bounds() {
    let range = continuous_auto_range((0.0, 10.0), &AxisSpec::default());

    assert!(
        range.0 < 0.0 && range.1 > 10.0,
        "continuous auto range should pad both data endpoints: {range:?}"
    );
}

#[test]
fn continuous_auto_range_preserves_finite_extreme_endpoints_when_padding_overflows() {
    let range = continuous_auto_range((-f64::MAX, f64::MAX), &AxisSpec::default());

    assert_eq!(range, (-f64::MAX, f64::MAX));
    let projection = Projection::new(range, range, (0.0, 100.0, 0.0, 100.0));
    assert_eq!(projection.project(0.0, 0.0), (50.0, 50.0));
}

#[test]
fn anchored_auto_range_keeps_zero_side_exact_for_positive_data() {
    let range = anchored_auto_range((0.0, 10.0), &AxisSpec::default(), 0.0);

    assert_eq!(range, (0.0, 10.5));
}

#[test]
fn anchored_auto_range_uses_relative_padding_for_tiny_positive_density() {
    let maximum = 2.4e-294;

    let range = anchored_auto_range((0.0, maximum), &AxisSpec::default(), 0.0);

    assert!(range.0.total_cmp(&0.0).is_eq());
    assert!(range.1 > maximum && range.1 < 1e-292, "{range:?}");
}

#[test]
fn anchored_auto_range_advances_adjacent_subnormal_density_endpoint() {
    let maximum = f64::from_bits(1);

    let range = anchored_auto_range((0.0, maximum), &AxisSpec::default(), 0.0);

    assert_eq!(range, (0.0, maximum.next_up()));
}

#[test]
fn continuous_auto_range_keeps_nice_ticks_inside_limits() {
    let range = continuous_auto_range((0.0, 10.0), &AxisSpec::default());
    let ticks = nice_ticks(range.0, range.1, AxisSpec::default().tick_count);

    assert!(
        ticks.iter().any(|tick| tick.abs() < f64::EPSILON)
            && ticks
                .iter()
                .any(|tick| (*tick - 10.0).abs() < f64::EPSILON)
            && range.0 < 0.0
            && range.1 > 10.0,
        "auto range should broaden limits while preserving nice endpoint ticks: range={range:?}, ticks={ticks:?}"
    );
}

#[test]
fn categorical_y_ticks_preserve_the_supplied_row_range() {
    let y_ticks = (0..7)
        .map(|index| AxisTick::new(f64::from(index) + 0.5, format!("row {index}")))
        .collect();
    let layout = Layout::new_with_ticks_and_axis_specs(
        (0.0, 10.0),
        (0.0, 7.0),
        640,
        480,
        "x".into(),
        "y".into(),
        Vec::new(),
        y_ticks,
        &AxisSpec::default(),
        &AxisSpec::default(),
    );

    assert_eq!(layout.projection.y_range(), (0.0, 7.0));
}

#[test]
fn projection_preserves_out_of_range_canvas_coordinates() {
    let projection = Projection::new((0.0, 1.0), (0.0, 1.0), (10.0, 20.0, 30.0, 50.0));

    assert_eq!(projection.project(-0.5, 1.5), (5.0, 60.0));
}

#[test]
fn projection_scales_opposite_sign_extreme_axis_boundaries_without_overflow() {
    let projection = Projection::new(
        (-f64::MAX, f64::MAX),
        (-1.0, 1.0),
        (10.0, 110.0, 20.0, 220.0),
    );

    assert_eq!(projection.project(-f64::MAX, -1.0), (10.0, 20.0));
    assert_eq!(projection.project(f64::MAX, 1.0), (110.0, 220.0));
}

#[test]
fn nice_ticks_return_adjacent_subnormal_endpoints_without_zero_step() {
    let upper = f64::from_bits(1);

    assert_eq!(nice_ticks(0.0, upper, 5), [0.0, upper]);
    assert_eq!(nice_range(0.0, upper, 5), (0.0, upper));
    assert!(labels_are_unique(&distinct_tick_labels(&[0.0, upper])));
}

#[test]
fn adjacent_ordinary_and_large_ticks_keep_both_distinguishable_endpoints() {
    for lower in [1.0_f64, 1e300, f64::MAX.next_down()] {
        let upper = lower.next_up();
        let ticks = nice_ticks(lower, upper, 5);
        let labels = distinct_tick_labels(&ticks);

        assert_eq!(ticks, [lower, upper], "{lower}..{upper}");
        assert_ne!(labels[0], labels[1], "{lower}..{upper}: {labels:?}");
    }
}

#[test]
fn opposite_extreme_ticks_use_finite_endpoints_and_distinct_labels() {
    let ticks = nice_ticks(-f64::MAX, f64::MAX, 5);
    let labels = distinct_tick_labels(&ticks);

    assert_eq!(ticks, [-f64::MAX, f64::MAX]);
    assert_ne!(labels[0], labels[1]);
}

#[test]
fn numeric_layout_preserves_explicit_adjacent_range_and_ticks() {
    let upper = f64::from_bits(1);
    let spec = AxisSpec::default().limits(0.0, upper);
    let layout = Layout::new_with_axis_specs(
        (0.0, upper),
        (0.0, upper),
        640,
        480,
        "x".into(),
        "y".into(),
        &spec,
        &spec,
    );

    assert_eq!(layout.projection.x_range(), (0.0, upper));
    assert_eq!(
        layout
            .x_axis
            .ticks
            .iter()
            .map(|tick| tick.position)
            .collect::<Vec<_>>(),
        [0.0, upper]
    );
    assert!(layout.projection.project(upper, upper).0.is_finite());
}
