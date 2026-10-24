use super::*;

#[test]
fn fractional_quantization_matches_independent_small_dyadic_rationals() {
    // Independent bounded u128 oracle: the source is the exact rational n/2^b.
    // All floating construction operations here are exact, and no float-to-text
    // conversion supplies an expected answer. Cover every admitted fractional
    // precision and both quotient parities, including negative rounded zero.
    for numerator in [0_u32, 1, 3, 5, 7, 9, 15, 31, 63, 125, 1999] {
        for bits in 0..=10 {
            let denominator = 1_u32 << bits;
            let value = f64::from(numerator) / f64::from(denominator);
            for scale in -3_i16..=3 {
                for precision in 0..=20 {
                    let shift = i32::from(scale) + i32::from(precision);
                    let (top, bottom) = if shift >= 0 {
                        (
                            u128::from(numerator) * 10_u128.pow(shift.unsigned_abs()),
                            u128::from(denominator),
                        )
                    } else {
                        (
                            u128::from(numerator),
                            u128::from(denominator) * 10_u128.pow(shift.unsigned_abs()),
                        )
                    };
                    let floor = top / bottom;
                    let remainder = top % bottom;
                    let coefficient = floor
                        + u128::from(
                            2 * remainder > bottom || (2 * remainder == bottom && floor % 2 == 1),
                        );
                    let places = usize::from(precision);
                    let mut expected = format!("{coefficient:0width$}", width = places + 1);
                    if places > 0 {
                        expected.insert(expected.len() - places, '.');
                    }
                    let format = ResolvedFormat {
                        precision: ResolvedPrecision::Fractional(precision),
                        decimal_scale: scale,
                        ..ResolvedFormat::default()
                    };
                    assert_eq!(format_finite(value, &format).unwrap(), expected);
                    assert_eq!(
                        format_finite(-value, &format).unwrap(),
                        format!("-{expected}")
                    );
                }
            }
        }
    }
}

#[test]
fn decimal_emission_obeys_notation_precision_and_zero_occupancy() {
    use NumericNotation::{Decimal, Engineering, Scientific};
    use ResolvedPrecision::{Fractional, Significant};

    // Handwritten dyadic/integer results, including single-round carries.
    for (value, notation, precision, expected) in [
        (1.25, Decimal, Fractional(1), "1.2"),
        (1.75, Decimal, Fractional(1), "1.8"),
        (100.5, Decimal, Fractional(0), "100"),
        (101.5, Decimal, Fractional(0), "102"),
        (1.0, Decimal, Significant(3), "1.00"),
        (0.125, Decimal, Significant(5), "0.12500"),
        (0.001_953_125, Decimal, Significant(3), "0.00195"),
        (12345.0, Decimal, Significant(3), "12300"),
        (999.5, Decimal, Significant(3), "1000"),
        (9.5, Scientific, Fractional(0), "1e1"),
        (0.125, Scientific, Fractional(2), "1.25e-1"),
        (12345.0, Scientific, Fractional(2), "1.23e4"),
        (0.125, Engineering, Fractional(2), "125.00e-3"),
        (12345.0, Engineering, Fractional(2), "12.34e3"),
        (999.5, Engineering, Significant(3), "1.00e3"),
        (999.5, Engineering, Fractional(0), "1e3"),
        (999.5, Engineering, Significant(2), "1.0e3"),
        (999.5, Engineering, Significant(1), "1e3"),
        (12345.0, Engineering, Significant(1), "10e3"),
        (0.0, Decimal, Significant(3), "0.00"),
        (0.0, Scientific, Significant(3), "0.00e0"),
        (0.0, Engineering, Significant(3), "0.00e0"),
    ] {
        let format = ResolvedFormat {
            notation,
            precision,
            ..ResolvedFormat::default()
        };
        assert_eq!(format_finite(value, &format).unwrap(), expected);
        assert_eq!(
            format_finite(-value, &format).unwrap(),
            format!("-{expected}")
        );
    }

    for notation in [Decimal, Scientific, Engineering] {
        let format = ResolvedFormat {
            notation,
            precision: Significant(3),
            prefix: Some("  μ["),
            suffix: Some("\u{0301} units00 "),
            negative_zero: NegativeZero::Normalize,
            trailing_zeros: TrailingZeros::Trim,
            ..ResolvedFormat::default()
        };
        let expected = if notation == Decimal {
            "  μ[0\u{0301} units00 "
        } else {
            "  μ[0e0\u{0301} units00 "
        };
        assert_eq!(format_finite(-0.0, &format).unwrap(), expected);
    }
    let format = ResolvedFormat {
        prefix: Some("["),
        suffix: Some("]"),
        ..ResolvedFormat::default()
    };
    assert_eq!(format_finite(-0.003_906_25, &format).unwrap(), "[-0.00]");
    assert_eq!(
        format_finite(
            -0.003_906_25,
            &ResolvedFormat {
                negative_zero: NegativeZero::Normalize,
                ..format
            },
        )
        .unwrap(),
        "[0.00]"
    );
    let percent = ResolvedFormat {
        precision: Fractional(1),
        decimal_scale: 2,
        suffix: Some("%"),
        ..ResolvedFormat::default()
    };
    assert_eq!(format_finite(0.125, &percent).unwrap(), "12.5%");
    assert_eq!(
        format_finite(
            0.125,
            &ResolvedFormat {
                decimal_scale: 0,
                ..percent
            }
        )
        .unwrap(),
        "0.1%"
    );
    assert_eq!(
        format_finite(
            1000.0,
            &ResolvedFormat {
                trailing_zeros: TrailingZeros::Trim,
                ..format
            }
        )
        .unwrap(),
        "[1000]"
    );
}

#[test]
fn extended_precision_matches_independent_binary_rational_cells() {
    for (value, numerator, denominator_bits, notation, precision, coefficient, expected) in [
        (
            0.1,
            3_602_879_701_896_397_u64,
            55,
            NumericNotation::Decimal,
            ResolvedPrecision::Fractional(20),
            10_000_000_000_000_000_555_u128,
            "0.10000000000000000555",
        ),
        (
            1.0_f64.next_up(),
            (1_u64 << 52) + 1,
            52,
            NumericNotation::Scientific,
            ResolvedPrecision::Significant(21),
            100_000_000_000_000_022_204,
            "1.00000000000000022204e0",
        ),
    ] {
        certify_rational_cell(
            BigUint::from(numerator),
            BigUint::from(1_u8) << denominator_bits,
            20,
            coefficient,
        );
        let format = ResolvedFormat {
            notation,
            precision,
            ..ResolvedFormat::default()
        };
        assert_eq!(format_finite(value, &format).unwrap(), expected);
        assert_eq!(
            format_finite(-value, &format).unwrap(),
            format!("-{expected}")
        );
    }
}

#[test]
fn finite_extremes_keep_exact_scaled_values_and_refuse_unrepresentable_output() {
    // Literal 17-significant coefficients, certified independently below against
    // binary rational magnitudes, not the converter's power-of-five decimal form.
    for (value, scale, expected, coefficient, quantum, numerator, denominator) in [
        (
            f64::MAX,
            324,
            "1.7976931348623157e632",
            17_976_931_348_623_157_u128,
            616_i32,
            (BigUint::from(1_u8) << 1024_usize) - (BigUint::from(1_u8) << 971_usize),
            BigUint::from(1_u8),
        ),
        (
            f64::MIN_POSITIVE,
            -324,
            "2.2250738585072014e-632",
            22_250_738_585_072_014,
            -648,
            BigUint::from(1_u8),
            BigUint::from(1_u8) << 1022_usize,
        ),
        (
            f64::from_bits(1),
            -324,
            "4.9406564584124654e-648",
            49_406_564_584_124_654,
            -664,
            BigUint::from(1_u8),
            BigUint::from(1_u8) << 1074_usize,
        ),
    ] {
        certify_rational_cell(
            numerator,
            denominator,
            i32::from(scale) - quantum,
            coefficient,
        );
        let format = ResolvedFormat {
            notation: NumericNotation::Scientific,
            precision: ResolvedPrecision::Significant(17),
            decimal_scale: scale,
            ..ResolvedFormat::default()
        };
        assert_eq!(format_finite(value, &format).unwrap(), expected);
        assert_eq!(
            format_finite(-value, &format).unwrap(),
            format!("-{expected}")
        );
    }
    let format = ResolvedFormat {
        precision: ResolvedPrecision::Significant(3),
        decimal_scale: -324,
        ..ResolvedFormat::default()
    };
    assert_eq!(
        format_finite(1.0, &format).unwrap(),
        format!("0.{}100", "0".repeat(323))
    );
    assert_eq!(
        format_finite(
            1.0,
            &ResolvedFormat {
                decimal_scale: 324,
                ..format
            }
        )
        .unwrap(),
        format!("1{}", "0".repeat(324))
    );

    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(format_finite(value, &format), Err(Error::Data(_))));
    }
    assert!(matches!(
        DecimalBuffer::reserve(usize::MAX, 0),
        Err(Error::Render(_))
    ));
    for invalid in [
        ResolvedFormat {
            decimal_scale: -325,
            ..format
        },
        ResolvedFormat {
            decimal_scale: 325,
            ..format
        },
    ] {
        assert!(matches!(format_finite(1.0, &invalid), Err(Error::Data(_))));
    }
    let huge = RoundedDecimal {
        negative: false,
        coefficient: BigUint::from(1_u8),
        exponent: BigInt::from(usize::MAX),
    };
    assert!(matches!(huge.emit(&format), Err(Error::Render(_))));
    assert!(matches!(
        RoundedDecimal {
            exponent: -huge.exponent,
            ..huge
        }
        .emit(&format),
        Err(Error::Render(_))
    ));
}

/// Strict rounding-cell inequalities certify these non-tie rational vectors.
/// This oracle uses their binary denominator, not production decimal decoding.
fn certify_rational_cell(numerator: BigUint, denominator: BigUint, shift: i32, coefficient: u128) {
    let (top, bottom) = if shift >= 0 {
        (
            numerator * BigUint::from(10_u8).pow(shift.unsigned_abs()),
            denominator,
        )
    } else {
        (
            numerator,
            denominator * BigUint::from(10_u8).pow(shift.unsigned_abs()),
        )
    };
    let twice = &top << 1_usize;
    assert!(twice > &bottom * (2 * coefficient - 1));
    assert!(twice < bottom * (2 * coefficient + 1));
}
