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
