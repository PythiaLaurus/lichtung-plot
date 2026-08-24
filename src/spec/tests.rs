use std::collections::HashSet;

use super::{
    LegendClearance, LegendDefault, NumericLabelFormat, PlotKind, SignificanceMarkStyle, TextWeight,
};

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

#[test]
fn plot_kind_metadata_is_pinned_in_stable_order() {
    let expected = [
        (PlotKind::Scatter, "scatter plot", "scatter-plot"),
        (PlotKind::Histogram, "histogram", "histogram"),
        (PlotKind::Density, "density plot", "density-plot"),
        (PlotKind::Qq, "Q-Q plot", "qq-plot"),
        (
            PlotKind::PointInterval,
            "point-interval plot",
            "point-interval-plot",
        ),
        (PlotKind::Line, "line plot", "line-plot"),
        (PlotKind::Trajectory, "trajectory plot", "trajectory-plot"),
        (PlotKind::LinePath, "line-path plot", "line-path-plot"),
        (
            PlotKind::Interaction,
            "interaction plot",
            "interaction-plot",
        ),
        (
            PlotKind::Coefficient,
            "coefficient plot",
            "coefficient-plot",
        ),
        (PlotKind::Forest, "forest plot", "forest-plot"),
        (PlotKind::Dumbbell, "dumbbell plot", "dumbbell-plot"),
        (PlotKind::Heatmap, "heatmap", "heatmap"),
        (PlotKind::Bar, "bar plot", "bar-plot"),
        (
            PlotKind::HorizontalBar,
            "horizontal bar plot",
            "horizontal-bar-plot",
        ),
        (PlotKind::StackedBar, "stacked bar plot", "stacked-bar-plot"),
        (
            PlotKind::StackedHorizontalBar,
            "stacked horizontal bar plot",
            "stacked-horizontal-bar-plot",
        ),
        (PlotKind::Boxplot, "boxplot", "boxplot"),
        (PlotKind::Violin, "violin plot", "violin-plot"),
        (PlotKind::Raincloud, "raincloud plot", "raincloud-plot"),
        (PlotKind::Regression, "regression plot", "regression-plot"),
        (PlotKind::Marginal, "marginal plot", "marginal-plot"),
        (PlotKind::Matrix, "scatter matrix", "scatter-matrix"),
        (PlotKind::Beeswarm, "beeswarm plot", "beeswarm-plot"),
        (PlotKind::Waterfall, "waterfall plot", "waterfall-plot"),
    ];

    assert_eq!(PlotKind::ALL, &expected.map(|(kind, _, _)| kind));
    assert_eq!(
        PlotKind::ALL
            .iter()
            .map(|kind| (kind.label(), kind.key_segment()))
            .collect::<Vec<_>>(),
        expected.map(|(_, label, key)| (label, key)).to_vec()
    );

    let labels = PlotKind::ALL
        .iter()
        .copied()
        .map(PlotKind::label)
        .collect::<HashSet<_>>();
    let keys = PlotKind::ALL
        .iter()
        .copied()
        .map(PlotKind::key_segment)
        .collect::<HashSet<_>>();

    assert_eq!(
        (labels.len(), keys.len()),
        (PlotKind::ALL.len(), PlotKind::ALL.len())
    );
}
