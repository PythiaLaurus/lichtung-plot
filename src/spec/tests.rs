use std::collections::HashSet;

use super::{LegendClearance, LegendDefault, PlotKind};

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
