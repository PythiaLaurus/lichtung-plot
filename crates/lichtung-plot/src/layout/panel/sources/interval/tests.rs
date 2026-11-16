use super::*;

fn values(range: (f64, f64), step: f64, origin: f64) -> Vec<f64> {
    positions(
        range,
        None,
        false,
        (step, origin, TickSpace::Data, TickRequirement::Fixed),
        usize::MAX,
    )
    .unwrap()
    .unwrap()
    .coordinates
}

#[test]
fn exact_rounding_cells_admit_decimal_endpoints_and_indices_larger_than_binary64_integers() {
    let decimal = values((0.0, 1.0), 0.1, 0.0);
    assert_eq!(decimal.len(), 11);
    assert_eq!(decimal[3].to_bits(), 0.3_f64.next_up().to_bits());
    assert_eq!(decimal[10].to_bits(), 1.0_f64.to_bits());
    assert_eq!(values((0.0, 1.0), 0.5, 1e20), [0.0, 0.5, 1.0]);
    assert_eq!(
        values((-f64::MAX, f64::MAX), f64::MAX, 0.0),
        [-f64::MAX, 0.0, f64::MAX]
    );
    let parameters = (1.0, 0.0, TickSpace::Data, TickRequirement::Fixed);
    assert_eq!(
        positions((0.0, 10_000.0), None, false, parameters, 10_001)
            .unwrap()
            .unwrap()
            .coordinates
            .len(),
        10_001
    );
    assert!(positions((0.0, 10_000.0), None, false, parameters, 10_000)
        .unwrap()
        .is_none());
    assert!(fixed_is_empty((0.0, 10_000.0), None, false, parameters, 10_000).is_err());
    assert!(positions(
        (0.0, 1.0),
        None,
        false,
        (
            f64::from_bits(1),
            0.0,
            TickSpace::Data,
            TickRequirement::Fixed
        ),
        usize::MAX
    )
    .unwrap()
    .is_none());
    let adjacent = (1e20, 1e20_f64.next_up());
    assert!(positions(adjacent, None, false, parameters, usize::MAX).is_err());
}
