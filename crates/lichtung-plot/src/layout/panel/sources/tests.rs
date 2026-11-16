#![allow(
    clippy::float_cmp,
    reason = "source-policy fixtures assert prescribed binary64 identities"
)]
use super::super::projection::{projection_key, ratio_position};

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
