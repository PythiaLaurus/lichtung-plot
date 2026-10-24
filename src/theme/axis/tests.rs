use super::*;
#[test]
fn complete_hard_sums_keep_fractional_gaps_until_the_single_rounding_boundary() {
    let first = RelativeSpacing::ratio(1, 3072)
        .unwrap()
        .product(LocalMetric::css_pixels(1.0).unwrap());
    let sum = first
        .add_length(&first)
        .add_length(&first)
        .add_pixels(1.0)
        .unwrap();
    assert_eq!(
        AxisLengths::hard(&sum).unwrap().to_bits(),
        (1025.0_f64 / 1024.0).to_bits()
    );
    assert_eq!(AxisLengths::position(&first).unwrap().to_bits(), 0);
    assert_eq!(
        AxisLengths::hard(&first).unwrap().to_bits(),
        (1.0_f64 / 1024.0).to_bits()
    );
    let huge = ExactLength::pixels(f64::MAX).unwrap();
    assert!(AxisLengths::position(&huge).is_err());
}

#[test]
fn whole_positions_round_half_quantum_ties_with_the_final_anchor_parity() {
    let half = RelativeSpacing::ratio(1, 2048)
        .unwrap()
        .product(LocalMetric::css_pixels(1.0).unwrap());
    for (anchor, expected) in [
        (0.0, 0.0),
        (1.0 / 1024.0, 0.0),
        (2.0 / 1024.0, 2.0 / 1024.0),
        (-1.0 / 1024.0, -2.0 / 1024.0),
    ] {
        assert_eq!(
            AxisLengths::before(&half, anchor).unwrap().to_bits(),
            f64::to_bits(expected)
        );
    }
}
