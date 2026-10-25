use super::*;
use crate::numeric::decimal::{NumericNotation, NumericRounding, ResolvedPrecision};
use crate::numeric::operand::Operand;
use crate::numeric::power::certified::PowerFormatter;
use generation::LatticeRange;

#[test]
fn certified_bounds_preserve_reciprocal_indices_anchors_and_exact_endpoint_cells() {
    let bounds = LatticeRange::new((0.125, 64.0), false, 1.0, 2.0).unwrap();
    assert_eq!(
        (&bounds.first, &bounds.last),
        (&BigInt::from(-3), &BigInt::from(6))
    );
    assert_eq!(bounds.count(), BigInt::from(10));
    let inverse = LatticeRange::new((0.125, 64.0), false, 1.0, 0.5).unwrap();
    let shifted = LatticeRange::new((0.125, 64.0), false, 8.0, 2.0).unwrap();
    let mut formatter = PowerFormatter::default();
    for (offset, expected) in [0.125_f64, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0]
        .into_iter()
        .enumerate()
    {
        let index = &bounds.first + offset;
        let reciprocal = inverse.operand(&index).unwrap();
        assert_eq!(reciprocal.index, -&index);
        for operand in [
            bounds.operand(&index).unwrap(),
            reciprocal,
            shifted.operand(&(&shifted.first + offset)).unwrap(),
        ] {
            assert_eq!(
                formatter.lattice_binary(&operand).unwrap().to_bits(),
                expected.to_bits()
            );
            assert!(operand
                .equals(
                    &ResolvedFormat::default(),
                    &ExactDecimal::decode(expected, 0).unwrap()
                )
                .unwrap());
        }
    }
    let excluded =
        LatticeRange::new((1.0_f64.next_up(), 64.0_f64.next_down()), false, 1.0, 2.0).unwrap();
    assert_eq!(
        (excluded.first, excluded.last),
        (BigInt::from(1), BigInt::from(5))
    );
    for view in [(-10.0, -1.0), (1.1, 1.2)] {
        assert_eq!(
            LatticeRange::new(view, false, 1.0, 2.0).unwrap().count(),
            BigInt::ZERO
        );
    }
    assert!(LatticeRange::new((-1.0, 1.0), false, 1.0, 2.0).is_err());
}

#[test]
fn geometric_values_use_original_rationals_for_binary_and_decimal_rounding() {
    let mut formatter = PowerFormatter::default();
    // Every listed value is the independently exact rational 3^k/2^k.
    for (index, expected, text) in [
        (0, 1.0, "1.000"),
        (1, 1.5, "1.500"),
        (2, 2.25, "2.250"),
        (3, 3.375, "3.375"),
        (4, 5.0625, "5.062"),
    ] {
        let value = LatticeOperand::new(1.0, 1.5, BigInt::from(index)).unwrap();
        assert_eq!(
            formatter.lattice_binary(&value).unwrap().to_bits(),
            f64::to_bits(expected)
        );
        assert_eq!(
            formatter
                .format_lattice(
                    &value,
                    &ResolvedFormat {
                        precision: ResolvedPrecision::Fractional(3),
                        ..ResolvedFormat::default()
                    }
                )
                .unwrap(),
            text
        );
    }
    let third = LatticeOperand::new(1.0, 1.5, BigInt::from(-1)).unwrap();
    // 2/3 is in the nearest-even binary cell of 0x3fe5555555555555.
    let rounded = formatter.lattice_binary(&third).unwrap();
    assert_eq!(rounded.to_bits(), 0x3fe5_5555_5555_5555);
    assert_eq!(
        formatter
            .format_lattice(
                &third,
                &ResolvedFormat {
                    precision: ResolvedPrecision::Fractional(20),
                    ..ResolvedFormat::default()
                }
            )
            .unwrap(),
        "0.66666666666666666667"
    );
    // Multiplication is exact even though 3^40/2^40 is no longer binary64.
    let large = LatticeOperand::new(1.0, 1.5, BigInt::from(40)).unwrap();
    let rounded = formatter.lattice_binary(&large).unwrap();
    let center = crate::numeric::binary::units(rounded).unwrap();
    let exact = BigInt::from(3_u8).pow(40) << 1035_usize;
    assert!(crate::numeric::binary::units(rounded.next_down()).unwrap() + &center < exact);
    assert!(crate::numeric::binary::units(rounded.next_up()).unwrap() + center > exact);
    let tie = LatticeOperand::new(1.0_f64.next_up(), 1.5, BigInt::from(1)).unwrap();
    let even = 1.5_f64.next_up().next_up();
    assert_eq!(
        formatter.lattice_binary(&tie).unwrap().to_bits(),
        even.to_bits()
    );
    let included = LatticeRange::new((1.0, even), false, 1.0_f64.next_up(), 1.5).unwrap();
    let excluded =
        LatticeRange::new((1.0, even.next_down()), false, 1.0_f64.next_up(), 1.5).unwrap();
    assert_eq!(
        (included.last, excluded.last),
        (BigInt::from(1), BigInt::ZERO)
    );
    // Independent correctly rounded log10(2) fixture; no log of a rounded power.
    assert_eq!(
        LatticeOperand::new(1.0, 2.0, BigInt::from(1))
            .unwrap()
            .coordinate()
            .unwrap()
            .to_bits(),
        0x3fd3_4413_509f_79ff
    );
}

#[test]
fn rational_ties_and_common_format_options_share_the_signed_quantizer() {
    let value = LatticeOperand::new(1.25, 2.0, BigInt::ZERO).unwrap();
    let mut formatter = PowerFormatter::default();
    for (rounding, positive, negative) in [
        (NumericRounding::NearestEven, "1.2", "-1.2"),
        (NumericRounding::NearestAway, "1.3", "-1.3"),
        (NumericRounding::NearestTowardZero, "1.2", "-1.2"),
        (NumericRounding::TowardZero, "1.2", "-1.2"),
        (NumericRounding::AwayFromZero, "1.3", "-1.3"),
        (NumericRounding::Floor, "1.2", "-1.3"),
        (NumericRounding::Ceiling, "1.3", "-1.2"),
    ] {
        for (multiplier, expected) in [(1.0, positive), (-1.0, negative)] {
            assert_eq!(
                formatter
                    .format_lattice(
                        &value,
                        &ResolvedFormat {
                            precision: ResolvedPrecision::Fractional(1),
                            rounding,
                            multiplier,
                            ..ResolvedFormat::default()
                        }
                    )
                    .unwrap(),
                expected
            );
        }
    }
    let values = [1, 2, 3].map(|index| {
        Operand::Multiplicative(
            crate::data::source::source_arc(
                LatticeOperand::new(1.0, 1.5, BigInt::from(index)).unwrap(),
            )
            .unwrap(),
        )
    });
    let scaled = crate::NumericFormat::from_specifier(".1f")
        .unwrap()
        .decimal_scale(2)
        .unwrap()
        .function_builtin(&values)
        .unwrap();
    assert_eq!(scaled.labels, ["150.0", "225.0", "337.5"]);
    let si = crate::NumericFormat::from_specifier(".3s")
        .unwrap()
        .function_builtin(&[Operand::Multiplicative(
            crate::data::source::source_arc(
                LatticeOperand::new(1000.0, 2.0, BigInt::ZERO).unwrap(),
            )
            .unwrap(),
        )])
        .unwrap();
    assert_eq!(si.labels, ["1.00k"]);
}

#[test]
fn symbolic_extremes_and_near_one_ratios_have_no_physical_power_or_float_index_bound() {
    let enormous = BigInt::from(1_u8) << 1200_usize;
    let value = LatticeOperand::new(1.0, 10.0, enormous.clone()).unwrap();
    assert_eq!(
        value.exact_decade(&ResolvedFormat::default()).unwrap(),
        Some(enormous.clone())
    );
    let label = PowerFormatter::default()
        .format_lattice(
            &value,
            &ResolvedFormat {
                notation: NumericNotation::Scientific,
                precision: ResolvedPrecision::Significant(3),
                ..ResolvedFormat::default()
            },
        )
        .unwrap();
    assert_eq!(label, format!("1.00e{enormous}"));
    let scale = LatticeRange::new((400.0, 402.0), true, 1.0, 10.0).unwrap();
    assert_eq!(
        (scale.first, scale.last),
        (BigInt::from(400), BigInt::from(402))
    );
    let tiny = LatticeOperand::new(1.0, 10.0, BigInt::from(-400)).unwrap();
    assert_eq!(
        PowerFormatter::default()
            .format_lattice(&tiny, &ResolvedFormat::default())
            .unwrap(),
        "0.00"
    );
    let near = LatticeRange::new((1.0, 2.0), false, 1.0, 1.0_f64.next_up()).unwrap();
    assert!(near.count() > BigInt::from(1_u64 << 51));
    let remote = LatticeRange::new(
        (1.0, 1.0_f64.next_up().next_up()),
        false,
        f64::MIN_POSITIVE,
        1.0_f64.next_up(),
    )
    .unwrap();
    assert!(remote.first > BigInt::from(1_u64 << 53));
    assert!(remote.count() > BigInt::ZERO);
}
