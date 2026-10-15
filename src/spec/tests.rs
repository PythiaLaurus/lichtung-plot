use super::NumericLabelFormat;

#[test]
fn counted_numeric_census_formatting_preserves_precision_suffix_and_extreme_values() {
    for precision in [0, 2, 12] {
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
        }
    }
}

#[test]
fn numeric_label_format_default_is_two_decimals_without_suffix() {
    let format = NumericLabelFormat::default();

    assert_eq!(NumericLabelFormat::new(2).unwrap(), format);
    assert_eq!(
        (format.configured_precision(), format.format(1.234)),
        (2, "1.23".to_string())
    );
}

#[test]
fn numeric_label_format_selective_builders_preserve_each_selection() {
    let format = NumericLabelFormat::default()
        .precision(12)
        .unwrap()
        .suffix(" units")
        .unwrap();

    assert_eq!(
        (format.configured_precision(), format.format(1.0)),
        (12, "1.000000000000 units".to_string())
    );
}

#[test]
fn numeric_label_format_rejects_invalid_selective_values_as_data_errors() {
    let errors = [
        NumericLabelFormat::default().precision(13).unwrap_err(),
        NumericLabelFormat::default().suffix("").unwrap_err(),
        NumericLabelFormat::default().suffix("\n").unwrap_err(),
    ];

    assert!(errors.into_iter().all(|error| matches!(
        error,
        crate::Error::Data(message) if message.starts_with("numeric label ")
    )));
}
