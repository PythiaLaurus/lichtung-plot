use super::*;

mod oracle;

fn text(operand: &Log10Operand, format: &ResolvedFormat<'_>) -> String {
    certified::PowerFormatter::default()
        .format(operand, format)
        .unwrap()
        .text
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
