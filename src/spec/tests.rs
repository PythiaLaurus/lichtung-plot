use super::{
    LegendClearance, LegendDefault, NumericLabelFormat, PlotKind, SignificanceMarkStyle, TextWeight,
};

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

#[test]
fn significance_mark_style_default_and_selective_values_are_exact() {
    let default = SignificanceMarkStyle::default();
    let configured = default
        .font_size(18)
        .unwrap()
        .weight(TextWeight::Bold)
        .gap(0.0)
        .unwrap();

    assert_eq!(
        (
            default.font_size_spec(),
            default.weight_spec(),
            default.gap_spec(),
            configured.font_size_spec(),
            configured.weight_spec(),
            configured.gap_spec(),
        ),
        (16, TextWeight::Normal, 8.0, 18, TextWeight::Bold, 0.0)
    );
}

#[test]
fn significance_mark_style_rejects_invalid_selective_values_immediately() {
    let errors = [
        SignificanceMarkStyle::default().font_size(0).unwrap_err(),
        SignificanceMarkStyle::default().gap(-1.0).unwrap_err(),
        SignificanceMarkStyle::default().gap(f64::NAN).unwrap_err(),
        SignificanceMarkStyle::default()
            .gap(f64::INFINITY)
            .unwrap_err(),
    ];

    assert!(errors.into_iter().all(|error| matches!(
        error,
        crate::Error::Data(message)
            if message == "significance mark style requires positive font size and finite nonnegative gap"
    )));
}

#[test]
fn plot_kinds_expose_their_semantic_legend_defaults() {
    for kind in [
        PlotKind::Bar,
        PlotKind::StackedBar,
        PlotKind::Line,
        PlotKind::Trajectory,
        PlotKind::LinePath,
        PlotKind::Interaction,
        PlotKind::Boxplot,
        PlotKind::Violin,
        PlotKind::Raincloud,
        PlotKind::Regression,
    ] {
        assert_eq!(
            kind.legend_default(false),
            LegendDefault::TopRight(LegendClearance::VerticalMaximum),
            "{kind:?}"
        );
    }
    for kind in [PlotKind::HorizontalBar, PlotKind::StackedHorizontalBar] {
        assert_eq!(
            kind.legend_default(false),
            LegendDefault::TopRight(LegendClearance::HorizontalMaximum),
            "{kind:?}"
        );
    }
    assert_eq!(
        PlotKind::Scatter.legend_default(false),
        LegendDefault::TopRight(LegendClearance::VerticalMaximum)
    );
    assert_eq!(
        PlotKind::Scatter.legend_default(true),
        LegendDefault::LowestOverlap
    );
    assert_eq!(
        PlotKind::Density.legend_default(false),
        LegendDefault::LowestOverlap
    );
    assert_eq!(
        PlotKind::Marginal.legend_default(false),
        LegendDefault::TopRight(LegendClearance::VerticalMaximum)
    );
    assert_eq!(
        PlotKind::Marginal.legend_default(true),
        LegendDefault::LowestOverlap
    );
    assert_eq!(
        PlotKind::Matrix.legend_default(false),
        LegendDefault::SelfManaged
    );
}
