use super::NumericLabelFormat;

#[test]
fn counted_numeric_census_formatting_preserves_precision_suffix_and_extreme_values() {
    for precision in 0..=12 {
        let suffix = "\u{0301} units";
        let format = NumericLabelFormat::new(precision)
            .unwrap()
            .suffix(suffix)
            .unwrap();
        for value in [0.0, -0.0, 1.2345, f64::MAX, f64::MIN, f64::from_bits(1)] {
            let expected = format!("{value:.precision$}{suffix}");
            let actual = crate::font::canonical::format_census_source(
                format_args!("{}", format.display(value)),
                None,
            )
            .unwrap();
            assert_eq!(actual, expected);
            assert_eq!(format.format(value), expected);
            let exact = crate::numeric::decimal::ResolvedFormat {
                precision: crate::numeric::decimal::ResolvedPrecision::Fractional(
                    u8::try_from(precision).unwrap(),
                ),
                suffix: Some(suffix),
                ..crate::numeric::decimal::ResolvedFormat::default()
            };
            assert_eq!(
                crate::numeric::decimal::format_finite(value, &exact).unwrap(),
                expected
            );
        }
    }
}
