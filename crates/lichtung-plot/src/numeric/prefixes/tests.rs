use super::*;
use crate::{NumericFormat, NumericNotation, NumericPrecision, NumericRounding};

fn decimal(digits: u8) -> NumericFormat {
    NumericFormat::new(
        NumericNotation::Decimal,
        NumericPrecision::Fractional(digits),
    )
    .unwrap()
}

#[test]
fn rational_divisors_quantize_the_original_ratio_and_keep_symbolic_powers() {
    let thirds = decimal(20)
        .prefixes(NumericPrefixes::factor(NumericFactor::Value(3.0)).unwrap())
        .unwrap();
    assert_eq!(
        thirds.format_singleton(1.0, false).unwrap(),
        "0.33333333333333333333"
    );
    assert_eq!(
        thirds.format_singleton(0.5, true).unwrap(),
        "1.05409255338945977733"
    );
    // (3*coefficient ± 1.5)^2 brackets 10 at the exact 10^-20 quantum.
    let coefficient = BigInt::parse_bytes(b"105409255338945977733", 10).unwrap();
    let center = coefficient * 6_u8;
    let radicand = BigInt::from(40_u8) * BigInt::from(10_u8).pow(40);
    assert!((&center - 3_u8).pow(2) < radicand);
    assert!((&center + 3_u8).pow(2) > radicand);
    let negative = decimal(0)
        .prefixes(NumericPrefixes::factor(NumericFactor::Value(-3.0)).unwrap())
        .unwrap()
        .rounding(NumericRounding::Floor)
        .unwrap();
    assert_eq!(negative.format_singleton(4.0, false).unwrap(), "-2");
    assert_eq!(negative.format_singleton(0.5, true).unwrap(), "-2");
    let maximum = crate::numeric::power::Log10Operand::new(f64::MAX, 0)
        .unwrap()
        .decade;
    let canceled = decimal(2)
        .prefixes(NumericPrefixes::factor(NumericFactor::PowerOfTen(maximum.to_string())).unwrap())
        .unwrap();
    assert_eq!(canceled.format_singleton(f64::MAX, true).unwrap(), "1.00");
    let huge = NumericFormat::new(
        NumericNotation::Scientific,
        NumericPrecision::Significant(3),
    )
    .unwrap()
    .prefixes(
        NumericPrefixes::factor(NumericFactor::PowerOfTen("100000000000000000000000".into()))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        huge.format_singleton(1.0, false).unwrap(),
        "1.00e-100000000000000000000000"
    );
    assert!(decimal(0)
        .prefixes(
            NumericPrefixes::factor(NumericFactor::PowerOfTen(
                "-100000000000000000000000".into()
            ))
            .unwrap()
        )
        .unwrap()
        .format_singleton(1.0, false)
        .is_err());
}

#[test]
fn si_carry_uses_signed_original_cells_instead_of_rerounding_mantissas() {
    let keep = decimal(0)
        .prefixes(
            NumericPrefixes::si(NumericSelection::PerValue, NumericPromotion::KeepThreshold)
                .unwrap(),
        )
        .unwrap();
    let promote = decimal(0)
        .prefixes(
            NumericPrefixes::si(
                NumericSelection::PerValue,
                NumericPromotion::PromoteOnRounding,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(keep.format_singleton(999.5, false).unwrap(), "1000");
    assert_eq!(promote.format_singleton(999.5, false).unwrap(), "1k");
    assert_eq!(
        promote
            .format_singleton(f64::from_bits(999.5_f64.to_bits() - 1), false)
            .unwrap(),
        "999"
    );
    let inward = promote
        .clone()
        .rounding(NumericRounding::NearestTowardZero)
        .unwrap();
    assert_eq!(inward.format_singleton(999.5, false).unwrap(), "999");
    let floor = promote.clone().rounding(NumericRounding::Floor).unwrap();
    assert_eq!(floor.format_singleton(999.25, false).unwrap(), "999");
    assert_eq!(floor.format_singleton(-999.25, false).unwrap(), "-1k");
    let ceiling = promote.rounding(NumericRounding::Ceiling).unwrap();
    assert_eq!(ceiling.format_singleton(999.25, false).unwrap(), "1k");
    assert_eq!(ceiling.format_singleton(-999.25, false).unwrap(), "-999");
}

#[test]
fn common_prefixes_ignore_zeros_and_freeze_the_furthest_signed_carry_for_every_order() {
    let format = decimal(0)
        .prefixes(
            NumericPrefixes::si(
                NumericSelection::WholeSet,
                NumericPromotion::PromoteOnRounding,
            )
            .unwrap(),
        )
        .unwrap()
        .rounding(NumericRounding::Floor)
        .unwrap();
    let values = [999.25, -999.25, 0.0];
    let labels = format
        .format_set(&values, false, false)
        .unwrap()
        .unwrap()
        .labels;
    assert_eq!(labels, ["0k", "-1k", "0k"]);
    let reverse = format
        .format_set(&[0.0, -999.25, 999.25], false, false)
        .unwrap()
        .unwrap();
    assert_eq!(
        reverse.labels,
        [labels[2].as_str(), labels[1].as_str(), labels[0].as_str()]
    );
    let small = decimal(2)
        .prefixes(
            NumericPrefixes::si(NumericSelection::WholeSet, NumericPromotion::KeepThreshold)
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        small
            .format_set(&[0.0, 0.000_125], false, false)
            .unwrap()
            .unwrap()
            .labels,
        ["0.00µ", "125.00µ"]
    );
    assert_eq!(
        small
            .format_set(&[-0.0, 0.0], false, false)
            .unwrap()
            .unwrap()
            .labels,
        ["-0.00", "0.00"]
    );
    let auto = decimal(1)
        .prefixes(NumericPrefixes::factor(NumericFactor::AutomaticDecimal).unwrap())
        .unwrap();
    assert_eq!(
        auto.format_set(&[0.0, -25.0, 12.5], false, false)
            .unwrap()
            .unwrap()
            .labels,
        ["0.0", "-2.5", "1.2"]
    );
}

#[test]
fn compact_rules_keep_literal_affixes_and_support_nonintegral_carry_thresholds() {
    let rules = vec![
        NumericCompactRule::new(10.25, 2.0, "A0 ", " end9").unwrap(),
        NumericCompactRule::new(20.75, 4.0, "B ", "!").unwrap(),
    ];
    let format = decimal(0)
        .prefixes(
            NumericPrefixes::compact(
                rules,
                NumericSelection::PerValue,
                NumericPromotion::PromoteOnRounding,
            )
            .unwrap(),
        )
        .unwrap()
        .prefix("[")
        .unwrap()
        .suffix("]")
        .unwrap();
    // The next threshold is 10.375 at divisor 2. Its first reachable rounded
    // coefficient is 11, whose even-nearest lower edge is 10.5 (exclusive).
    assert_eq!(
        format.format_singleton(20.5, false).unwrap(),
        "[A0 10 end9]"
    );
    assert_eq!(format.format_singleton(20.75, false).unwrap(), "[B 5!]");
    let short = decimal(0)
        .prefixes(
            NumericPrefixes::decimal_compact(
                NumericSelection::PerValue,
                NumericPromotion::PromoteOnRounding,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(short.format_singleton(999.5, false).unwrap(), "1K");
    assert_eq!(short.format_singleton(999_500.0, false).unwrap(), "1M");
    assert_eq!(short.format_singleton(12.5, false).unwrap(), "12");
}

#[test]
fn prefix_construction_normalizes_requests_and_covers_the_complete_si_table() {
    for (exponent, symbol) in [
        (-30, "q"),
        (-27, "r"),
        (-6, "µ"),
        (-2, "c"),
        (-1, "d"),
        (0, ""),
        (1, "da"),
        (2, "h"),
        (27, "R"),
        (30, "Q"),
    ] {
        let prefix = SiPrefix::from_exponent(exponent).unwrap();
        assert_eq!(prefix.symbol(), symbol);
        let format = decimal(2)
            .prefixes(
                NumericPrefixes::si(
                    NumericSelection::Fixed(prefix),
                    NumericPromotion::KeepThreshold,
                )
                .unwrap(),
            )
            .unwrap();
        assert!(format
            .format_singleton(1.0, false)
            .unwrap()
            .ends_with(symbol));
    }
    assert!(SiPrefix::from_exponent(4).is_err());
    assert!(NumericPrefixes::si(
        NumericSelection::Fixed(SiPrefix::from_exponent(3).unwrap()),
        NumericPromotion::PromoteOnRounding
    )
    .is_err());
    assert_eq!(
        NumericPrefixes::compact(
            vec![],
            NumericSelection::WholeSet,
            NumericPromotion::KeepThreshold
        )
        .unwrap(),
        NumericPrefixes::none()
    );
    assert!(NumericPrefixes::compact(
        vec![],
        NumericSelection::Fixed(0),
        NumericPromotion::KeepThreshold
    )
    .is_err());
    assert_eq!(
        NumericPrefixes::factor(NumericFactor::PowerOfTen("+00006".into())).unwrap(),
        NumericPrefixes::factor(NumericFactor::PowerOfTen("6".into())).unwrap()
    );
    assert_eq!(
        NumericPrefixes::factor(NumericFactor::PowerOfTen("-000".into())).unwrap(),
        NumericPrefixes::factor(NumericFactor::PowerOfTen("0".into())).unwrap()
    );
    for source in ["", "+", "1.0", "١", " 1", "1 "] {
        assert!(NumericPrefixes::factor(NumericFactor::PowerOfTen(source.into())).is_err());
    }
    for value in [0.0, -0.0, f64::NAN, f64::INFINITY] {
        assert!(NumericPrefixes::factor(NumericFactor::Value(value)).is_err());
    }
    let si = decimal(2)
        .prefixes(
            NumericPrefixes::si(NumericSelection::PerValue, NumericPromotion::KeepThreshold)
                .unwrap(),
        )
        .unwrap();
    assert_eq!(si.format_singleton(-40.0, true).unwrap(), "0.00q");
    assert_eq!(si.format_singleton(33.0, true).unwrap(), "1000.00Q");
}
