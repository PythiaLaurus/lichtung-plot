use super::*;

mod oracle;

fn text(operand: &Log10Operand, format: &ResolvedFormat<'_>) -> String {
    certified::PowerFormatter::default()
        .format(operand, format)
        .unwrap()
        .text
}

#[test]
fn mathematical_power_factors_keep_signed_cells_and_symbolic_extreme_exponents() {
    let format = crate::NumericFormat::new(
        NumericNotation::Scientific,
        crate::NumericPrecision::Significant(21),
    )
    .unwrap()
    .multiplier(4.0)
    .unwrap();
    // Square the proposed half-cell edges: (4*sqrt(10))^2 = 160.
    let coefficient = BigUint::parse_bytes(b"126491106406735173280", 10).unwrap();
    let twice = &coefficient * 2_u8;
    let radicand = BigUint::from(640_u16) * BigUint::from(10_u8).pow(38);
    assert!((&twice - 1_u8).pow(2) < radicand);
    assert!((&twice + 1_u8).pow(2) > radicand);
    assert_eq!(
        format.format_singleton(0.5, true).unwrap(),
        "1.26491106406735173280e1"
    );
    let negative = crate::NumericFormat::new(
        NumericNotation::Decimal,
        crate::NumericPrecision::Fractional(0),
    )
    .unwrap()
    .multiplier(-1.0)
    .unwrap()
    .rounding(crate::NumericRounding::Floor)
    .unwrap();
    assert_eq!(negative.format_singleton(0.5, true).unwrap(), "-4");
    assert_eq!(negative.format_singleton(-f64::MAX, true).unwrap(), "-1");
    let zero = format.multiplier(0.0).unwrap();
    assert_eq!(
        zero.format_singleton(f64::MAX, true).unwrap(),
        "0.00000000000000000000e0"
    );
    let rational = crate::NumericFormat::new(
        NumericNotation::Decimal,
        crate::NumericPrecision::Fractional(1),
    )
    .unwrap()
    .multiplier(1.25)
    .unwrap();
    assert_eq!(rational.format_singleton(0.0, true).unwrap(), "1.2");
}

#[test]
fn directed_powers_separate_integer_boundaries_and_keep_tiny_shortcuts_symbolic() {
    use crate::numeric::decimal::NumericRounding::{Ceiling, Floor};
    // Monotonicity puts 10^(+/-2^-1074) on opposite sides of 1;
    // 3^2 < 10 < 4^2 independently bounds the sqrt(10) cell.
    let mut formatter = certified::PowerFormatter::default();
    for (rounding, below, above, root, tiny) in
        [(Floor, "0", "1", "3", "0"), (Ceiling, "1", "2", "4", "1")]
    {
        let format = ResolvedFormat {
            precision: ResolvedPrecision::Fractional(0),
            rounding,
            ..ResolvedFormat::default()
        };
        for (exponent, expected) in [
            (-f64::from_bits(1), below),
            (f64::from_bits(1), above),
            (0.5, root),
        ] {
            let operand = Log10Operand::new(exponent, 0).unwrap();
            assert_eq!(formatter.format(&operand, &format).unwrap().text, expected);
        }
        let operand = Log10Operand::new(-f64::MAX, 0).unwrap();
        assert_eq!(formatter.format(&operand, &format).unwrap().text, tiny);
        assert!(
            formatter.attempts.is_empty(),
            "tiny values do not construct a huge power"
        );
    }
}

#[test]
fn exact_exponent_reduction_retains_scaled_floor_and_full_dyadic_remainders() {
    // Independent common-denominator representation: every finite binary64 is
    // an integer multiple of 2^-1074. This oracle never reduces a denominator or
    // relies on signed truncating division for the expected floor.
    let unit = BigInt::from(1_u8) << 1074_usize;
    for bits in [
        0,
        1,
        2,
        0x000f_ffff_ffff_ffff,
        0x0010_0000_0000_0000,
        0x3fd0_0000_0000_0000,
        0x3fe0_0000_0000_0000,
        0x3fef_ffff_ffff_ffff,
        0x3ff0_0000_0000_0000,
        0x4073_4800_0000_0000,
        0x43b0_0000_0000_0000,
        0x7fef_ffff_ffff_ffff,
    ] {
        for bits in [bits, bits | (1_u64 << 63)] {
            let stored = (bits >> 52) & 0x7ff;
            let mantissa = bits & ((1_u64 << 52) - 1);
            let magnitude = if stored == 0 {
                BigUint::from(mantissa)
            } else {
                BigUint::from(mantissa + (1_u64 << 52)) << usize::try_from(stored - 1).unwrap()
            };
            let numerator = BigInt::from_biguint(
                if bits >> 63 == 0 {
                    Sign::Plus
                } else {
                    Sign::Minus
                },
                magnitude,
            );
            for scale in [-324, 0, 1, 324] {
                let operand = Log10Operand::new(f64::from_bits(bits), scale).unwrap();
                let scaled = &numerator + &unit * scale;
                let remainder = BigInt::from(operand.fraction.clone())
                    << (1074 - operand.denominator_bits) as usize;
                assert_eq!(&operand.decade * &unit + remainder, scaled);
                assert!(&operand.decade * &unit <= scaled);
                assert!((&operand.decade + 1_u8) * &unit > scaled);
                assert!(
                    operand.fraction < (BigUint::from(1_u8) << operand.denominator_bits as usize)
                );
                assert_eq!(
                    operand.exponent_bits,
                    if f64::from_bits(bits) == 0.0 { 0 } else { bits }
                );
            }
        }
    }
    let negative_tiny = Log10Operand::new(-f64::from_bits(1), 0).unwrap();
    assert_eq!(negative_tiny.decade, BigInt::from(-1));
    assert_eq!(negative_tiny.denominator_bits, 1074);
    assert_eq!(
        negative_tiny.fraction,
        (BigUint::from(1_u8) << 1074_usize) - 1_u8
    );
    let scaled_large = Log10Operand::new(f64::from_bits(0x43b0_0000_0000_0000), 1).unwrap();
    assert_eq!(
        scaled_large.decade,
        BigInt::from(1_152_921_504_606_846_977_u64)
    );
    for exponent in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            Log10Operand::new(exponent, 0),
            Err(Error::Data(_))
        ));
    }
}

type IntegralVector = (f64, i16, NumericNotation, ResolvedPrecision, &'static str);

const INTEGRAL_VECTORS: [IntegralVector; 14] = {
    use NumericNotation::{Decimal, Engineering, Scientific};
    use ResolvedPrecision::{Fractional, Significant};

    [
        (0.0, 0, Decimal, Fractional(2), "1.00"),
        (-0.0, 0, Decimal, Fractional(2), "1.00"),
        (-0.0, 0, Scientific, Significant(3), "1.00e0"),
        (-1.0, 0, Engineering, Fractional(2), "100.00e-3"),
        (-2.0, 0, Engineering, Significant(2), "10e-3"),
        (324.0, 0, Scientific, Fractional(2), "1.00e324"),
        (324.0, 0, Engineering, Fractional(2), "1.00e324"),
        (-324.0, 0, Scientific, Fractional(2), "1.00e-324"),
        (-324.0, 0, Engineering, Fractional(2), "1.00e-324"),
        (-324.0, 0, Decimal, Fractional(2), "0.00"),
        (0.0, 2, Decimal, Fractional(2), "100.00"),
        (
            1_152_921_504_606_846_976.0,
            1,
            Scientific,
            Fractional(2),
            "1.00e1152921504606846977",
        ),
        (
            1_152_921_504_606_846_976.0,
            -324,
            Scientific,
            Fractional(2),
            "1.00e1152921504606846652",
        ),
        (
            1_152_921_504_606_846_976.0,
            0,
            Engineering,
            Fractional(2),
            "10.00e1152921504606846975",
        ),
    ]
};

#[test]
fn integral_powers_emit_symbolically_and_refuse_giant_decimal_runs_before_materialization() {
    use super::super::decimal::{NegativeZero, TrailingZeros};
    use NumericNotation::Scientific;
    use ResolvedPrecision::Significant;

    for (exponent, scale, notation, precision, expected) in INTEGRAL_VECTORS {
        let format = ResolvedFormat {
            notation,
            precision,
            decimal_scale: scale,
            ..ResolvedFormat::default()
        };
        let operand = Log10Operand::new(exponent, scale).unwrap();
        assert_eq!(text(&operand, &format), expected);
    }
    let format = ResolvedFormat::default();
    assert_eq!(
        text(&Log10Operand::new(324.0, 0).unwrap(), &format),
        format!("1{}.00", "0".repeat(324))
    );
    let maximum = (BigUint::from(1_u8) << 1024_usize) - (BigUint::from(1_u8) << 971_usize);
    for sign in [1.0, -1.0] {
        let operand = Log10Operand::new(sign * f64::MAX, 0).unwrap();
        let scientific = ResolvedFormat {
            notation: Scientific,
            ..format
        };
        let expected = format!("1.00e{}{maximum}", if sign < 0.0 { "-" } else { "" });
        let label = text(&operand, &scientific);
        assert_eq!(label, expected);
        assert_eq!(label.len(), if sign < 0.0 { 315 } else { 314 });
        if sign > 0.0 {
            assert!(matches!(
                certified::PowerFormatter::default().format(&operand, &format),
                Err(Error::Render(_))
            ));
        } else {
            assert_eq!(text(&operand, &format), "0.00");
            let significant = ResolvedFormat {
                precision: Significant(3),
                ..format
            };
            assert!(matches!(
                certified::PowerFormatter::default().format(&operand, &significant),
                Err(Error::Render(_))
            ));
        }
    }
    for exponent in [-324.0, 324.0] {
        let operand = Log10Operand::new(exponent, 0).unwrap();
        let significant = ResolvedFormat {
            precision: Significant(3),
            ..format
        };
        let expected = if exponent < 0.0 {
            format!("0.{}100", "0".repeat(323))
        } else {
            format!("1{}", "0".repeat(324))
        };
        assert_eq!(text(&operand, &significant), expected);
    }
    let format = ResolvedFormat {
        prefix: Some("["),
        suffix: Some("\u{0301}%00"),
        negative_zero: NegativeZero::Preserve,
        trailing_zeros: TrailingZeros::Trim,
        ..format
    };
    assert_eq!(
        text(&Log10Operand::new(-f64::MAX, 0).unwrap(), &format),
        "[0\u{0301}%00"
    );
    let fractional = Log10Operand::new(-2000.5, 0).unwrap();
    assert_eq!(text(&fractional, &format), "[0\u{0301}%00");
    let fractional = Log10Operand::new(-0.5, 0).unwrap();
    assert_eq!(text(&fractional, &format), "[0.32\u{0301}%00");
}

#[test]
fn outward_power_stages_agree_with_independent_certified_root_products() {
    let mut formatter = certified::PowerFormatter::default();
    for bits in [
        0x3fd0_0000_0000_0000,
        0x3fe0_0000_0000_0000,
        0x3fe8_0000_0000_0000,
        0xbfe0_0000_0000_0000,
        0x3fef_ffff_ffff_ffff,
        0x4007_ffff_ffff_ffff,
        1,
        0x8000_0000_0000_0001,
        0x4073_4800_0000_0000,
        0x4043_c000_0000_0000,
    ] {
        let operand = Log10Operand::new(f64::from_bits(bits), 0).unwrap();
        let independent = oracle::decode(bits, 0);
        for precision in [128, 256, 512] {
            let (lower, upper) = formatter.mantissa(&operand, precision).unwrap();
            assert_eq!(formatter.stages.last().unwrap(), &(lower, upper));
            let mut error = BigUint::from(8 * precision + 32);
            for (stage, (lower, upper)) in formatter.stages.iter().enumerate() {
                let (oracle_lower, oracle_upper) = oracle::bounds(
                    &independent.fraction,
                    independent.denominator_bits + 4 - stage,
                    precision + 128,
                );
                // Compare on the same rational scale. These are independently
                // computed enclosures; no nesting assumption across production
                // precisions is made. The declared stage width also holds.
                assert!((lower << 128_usize) <= oracle_upper);
                assert!((upper << 128_usize) >= oracle_lower);
                assert!(upper - lower <= &error * 2_u8);
                let bits = certified::scratch_bits(precision, 0).unwrap();
                assert!(upper.bits() <= u64::try_from(bits).unwrap());
                error = error * 22_u8 + 1_u8;
            }
        }
    }
    assert!(formatter.constant_hits > 0);
    assert_eq!(formatter.constant_misses, 3);
    let misses = formatter.constant_misses;
    let operand = Log10Operand::new(0.5, 0).unwrap();
    assert!(formatter.mantissa(&operand, usize::MAX).is_err());
    assert_eq!(formatter.constant_misses, misses);
    assert!(certified::scratch_bits(128, usize::MAX).is_err());
}

type FractionalVector = (
    u64,
    i16,
    NumericNotation,
    ResolvedPrecision,
    &'static str,
    i32,
    &'static str,
);

const FRACTIONAL_VECTORS: [FractionalVector; 24] = {
    use NumericNotation::{Decimal, Engineering, Scientific};
    use ResolvedPrecision::{Fractional, Significant};

    [
        (
            0x3fe0_0000_0000_0000,
            0,
            Scientific,
            Significant(21),
            "316227766016837933200",
            -20,
            "3.16227766016837933200e0",
        ),
        (
            0x3fe0_0000_0000_0000,
            0,
            Decimal,
            Fractional(2),
            "316",
            -2,
            "3.16",
        ),
        (
            0x3fe0_0000_0000_0000,
            2,
            Decimal,
            Fractional(2),
            "31623",
            -2,
            "316.23",
        ),
        (
            0xbfe0_0000_0000_0000,
            0,
            Decimal,
            Fractional(2),
            "32",
            -2,
            "0.32",
        ),
        (
            0xbfe0_0000_0000_0000,
            0,
            Scientific,
            Fractional(2),
            "316",
            -3,
            "3.16e-1",
        ),
        (
            0xbfe0_0000_0000_0000,
            0,
            Engineering,
            Fractional(2),
            "31623",
            -5,
            "316.23e-3",
        ),
        (
            0x3fd0_0000_0000_0000,
            0,
            Decimal,
            Fractional(2),
            "178",
            -2,
            "1.78",
        ),
        (
            0x3fe8_0000_0000_0000,
            0,
            Decimal,
            Fractional(2),
            "562",
            -2,
            "5.62",
        ),
        (
            0xbfd0_0000_0000_0000,
            0,
            Decimal,
            Fractional(2),
            "56",
            -2,
            "0.56",
        ),
        (
            0xbfe8_0000_0000_0000,
            0,
            Decimal,
            Fractional(2),
            "18",
            -2,
            "0.18",
        ),
        (
            0x3fe0_0000_0000_0000,
            0,
            Scientific,
            Significant(3),
            "316",
            -2,
            "3.16e0",
        ),
        (
            0x3fe0_0000_0000_0000,
            0,
            Engineering,
            Significant(3),
            "316",
            -2,
            "3.16e0",
        ),
        (
            0x3fe0_0000_0000_0000,
            324,
            Scientific,
            Fractional(2),
            "316",
            322,
            "3.16e324",
        ),
        (
            0x3fe0_0000_0000_0000,
            -324,
            Scientific,
            Fractional(2),
            "316",
            -326,
            "3.16e-324",
        ),
        (
            0x3fef_ffff_ffff_ffff,
            0,
            Scientific,
            Fractional(2),
            "1000",
            -2,
            "1.00e1",
        ),
        (
            0x3fef_ffff_ffff_ffff,
            0,
            Decimal,
            Fractional(2),
            "1000",
            -2,
            "10.00",
        ),
        (
            0x4007_ffff_ffff_ffff,
            0,
            Engineering,
            Fractional(2),
            "100000",
            -2,
            "1.00e3",
        ),
        (
            0x4007_ffff_ffff_ffff,
            0,
            Engineering,
            Significant(3),
            "1000",
            0,
            "1.00e3",
        ),
        (
            1,
            0,
            Scientific,
            Significant(17),
            "10000000000000000",
            -16,
            "1.0000000000000000e0",
        ),
        (
            2,
            0,
            Scientific,
            Significant(17),
            "10000000000000000",
            -16,
            "1.0000000000000000e0",
        ),
        (
            0x8000_0000_0000_0001,
            0,
            Scientific,
            Significant(17),
            "100000000000000000",
            -17,
            "1.0000000000000000e0",
        ),
        (
            0x4073_4800_0000_0000,
            0,
            Scientific,
            Fractional(2),
            "316",
            306,
            "3.16e308",
        ),
        (
            0x4043_c000_0000_0000,
            0,
            Decimal,
            Fractional(0),
            "3162277660168379331998893544432718533720",
            0,
            "3162277660168379331998893544432718533720",
        ),
        (
            0xbfe0_0000_0000_0000,
            0,
            Decimal,
            Fractional(0),
            "0",
            0,
            "0",
        ),
    ]
};

#[test]
fn fractional_powers_certify_literal_cells_carries_refinement_and_cache_independence() {
    use super::super::decimal::TrailingZeros;

    let mut formatter = certified::PowerFormatter::default();
    for (bits, scale, notation, precision, coefficient, quantum, expected) in FRACTIONAL_VECTORS {
        let coefficient = BigUint::parse_bytes(coefficient.as_bytes(), 10).unwrap();
        let oracle_precision = oracle::certify_cell(bits, scale, quantum, &coefficient);
        assert!(oracle_precision >= 256);
        oracle::certify_algebraic_cell(bits, scale, quantum, &coefficient);
        let format = ResolvedFormat {
            notation,
            precision,
            decimal_scale: scale,
            ..ResolvedFormat::default()
        };
        let operand = Log10Operand::new(f64::from_bits(bits), scale).unwrap();
        let certified = formatter.format(&operand, &format).unwrap();
        assert_eq!(certified.text, expected);
        assert_eq!(certified.rounded.coefficient, coefficient);
        assert_eq!(certified.rounded.exponent, BigInt::from(quantum));
        println!(
            "log10_certificate,bits={bits:016x},scale={scale},quantum={quantum},coefficient={coefficient},oracle_bits={oracle_precision},production_bits={:?}",
            formatter.attempts
        );
        let mut fresh = certified::PowerFormatter::default();
        let independent_cache = fresh.format(&operand, &format).unwrap();
        assert_eq!(certified.text, independent_cache.text);
        assert_eq!(certified.rounded, independent_cache.rounded);
        if bits == 0x4043_c000_0000_0000 {
            assert_eq!(formatter.attempts[0], 128);
            assert!(formatter.attempts.len() > 1);
            assert!(*formatter.attempts.last().unwrap() <= 512);
        }
    }
    assert!(formatter.constant_hits > 0);
    let format = ResolvedFormat {
        decimal_scale: 2,
        prefix: Some("rate "),
        suffix: Some("%"),
        ..ResolvedFormat::default()
    };
    assert_eq!(
        text(&Log10Operand::new(0.5, 2).unwrap(), &format),
        "rate 316.23%"
    );
    let format = ResolvedFormat {
        notation: NumericNotation::Engineering,
        prefix: Some("["),
        suffix: Some("\u{0301}%00"),
        trailing_zeros: TrailingZeros::Trim,
        ..ResolvedFormat::default()
    };
    assert_eq!(
        text(
            &Log10Operand::new(f64::from_bits(0x4007_ffff_ffff_ffff), 0).unwrap(),
            &format
        ),
        "[1e3\u{0301}%00"
    );
    for (bits, quantum, expected) in [
        (
            0x432f_ffff_ffff_ffff,
            4_503_599_627_370_493_i64,
            "3.16e4503599627370495",
        ),
        (
            0xc32f_ffff_ffff_ffff,
            -4_503_599_627_370_498,
            "3.16e-4503599627370496",
        ),
    ] {
        let format = ResolvedFormat {
            notation: NumericNotation::Scientific,
            ..ResolvedFormat::default()
        };
        let value = formatter
            .format(
                &Log10Operand::new(f64::from_bits(bits), 0).unwrap(),
                &format,
            )
            .unwrap();
        // These largest fractional exponent magnitudes have fraction exactly
        // one half. The already independent sqrt(10) cell scales symbolically.
        assert_eq!(value.rounded.coefficient, BigUint::from(316_u16));
        assert_eq!(value.rounded.exponent, BigInt::from(quantum));
        assert_eq!(value.text, expected);
    }
    let misses = formatter.constant_misses;
    let huge = Log10Operand::new(f64::MAX, 0).unwrap();
    assert!(formatter.format(&huge, &ResolvedFormat::default()).is_err());
    assert!(formatter.attempts.is_empty());
    assert_eq!(formatter.constant_misses, misses);
}
