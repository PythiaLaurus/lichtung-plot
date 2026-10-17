use super::*;

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

#[test]
fn integral_powers_emit_symbolically_and_refuse_giant_decimal_runs_before_materialization() {
    use super::super::decimal::{NegativeZero, TrailingZeros};
    use NumericNotation::{Decimal, Engineering, Scientific};
    use ResolvedPrecision::{Fractional, Significant};

    for (exponent, scale, notation, precision, expected) in [
        (0.0, 0, Decimal, Fractional(2), "1.00"),
        (-0.0, 0, Scientific, Significant(3), "1.00e0"),
        (-1.0, 0, Engineering, Fractional(2), "100.00e-3"),
        (-2.0, 0, Engineering, Significant(2), "10e-3"),
        (324.0, 0, Scientific, Fractional(2), "1.00e324"),
        (-324.0, 0, Scientific, Fractional(2), "1.00e-324"),
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
            0,
            Engineering,
            Fractional(2),
            "10.00e1152921504606846975",
        ),
    ] {
        let format = ResolvedFormat {
            notation,
            precision,
            decimal_scale: scale,
            ..ResolvedFormat::default()
        };
        let operand = Log10Operand::new(exponent, scale).unwrap();
        assert_eq!(operand.format_integral(&format).unwrap(), expected);
    }
    let format = ResolvedFormat::default();
    let maximum = (BigUint::from(1_u8) << 1024_usize) - (BigUint::from(1_u8) << 971_usize);
    for sign in [1.0, -1.0] {
        let operand = Log10Operand::new(sign * f64::MAX, 0).unwrap();
        let scientific = ResolvedFormat {
            notation: Scientific,
            ..format
        };
        let expected = format!("1.00e{}{maximum}", if sign < 0.0 { "-" } else { "" });
        let text = operand.format_integral(&scientific).unwrap();
        assert_eq!(text, expected);
        assert_eq!(text.len(), if sign < 0.0 { 315 } else { 314 });
        if sign > 0.0 {
            assert!(matches!(operand.plan(&format), Err(Error::Render(_))));
        } else {
            assert_eq!(operand.format_integral(&format).unwrap(), "0.00");
            let significant = ResolvedFormat {
                precision: Significant(3),
                ..format
            };
            assert!(matches!(operand.plan(&significant), Err(Error::Render(_))));
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
        assert_eq!(operand.format_integral(&significant).unwrap(), expected);
    }
    let format = ResolvedFormat {
        prefix: Some("["),
        suffix: Some("\u{0301}%00"),
        negative_zero: NegativeZero::Preserve,
        trailing_zeros: TrailingZeros::Trim,
        ..format
    };
    assert_eq!(
        Log10Operand::new(-f64::MAX, 0)
            .unwrap()
            .format_integral(&format)
            .unwrap(),
        "[0\u{0301}%00"
    );
    let fractional = Log10Operand::new(-2000.5, 0).unwrap();
    assert!(fractional.plan(&format).unwrap().zero);
    assert!(fractional.format_integral(&format).is_err());
    let fractional = Log10Operand::new(-0.5, 0).unwrap();
    let plan = fractional.plan(&format).unwrap();
    assert_eq!(plan.quantum, BigInt::from(-2));
    assert_eq!(plan.multiplier, BigInt::from(1));
    assert!(!plan.zero);
}
