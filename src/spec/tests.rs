use std::collections::HashSet;

use super::{
    EffectBandCoverage, EffectBandSemantics, InferentialScale, IntervalKind, IntervalSemantics,
    LegendClearance, LegendDefault, PlotKind,
};

#[test]
fn plot_kinds_expose_their_semantic_legend_defaults() {
    for kind in [
        PlotKind::Bar,
        PlotKind::StackedBar,
        PlotKind::Line,
        PlotKind::Interaction,
        PlotKind::Boxplot,
        PlotKind::Violin,
        PlotKind::MassSpectrum,
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
fn interval_semantics_reject_closed_probability_boundaries() {
    for level in [0.0, 1.0, f64::NAN] {
        assert!(IntervalSemantics::new(
            IntervalKind::Confidence,
            level,
            InferentialScale::Response,
            "fixture",
        )
        .is_err());
    }
}

#[test]
fn interval_semantics_reject_empty_provenance() {
    assert!(IntervalSemantics::new(
        IntervalKind::Confidence,
        0.95,
        InferentialScale::Response,
        "  ",
    )
    .is_err());
}

#[test]
fn interval_semantics_defensive_validation_rejects_forged_state() {
    let invalid_level = IntervalSemantics {
        kind: IntervalKind::Confidence,
        level: 1.5,
        scale: InferentialScale::Response,
        provenance: "fit".into(),
    };
    let invalid_provenance = IntervalSemantics {
        kind: IntervalKind::Confidence,
        level: 0.95,
        scale: InferentialScale::Response,
        provenance: " ".into(),
    };

    assert!(invalid_level.validate().is_err());
    assert!(invalid_provenance.validate().is_err());
}

#[test]
fn interval_semantics_accessors_preserve_validated_claim() {
    let semantics = IntervalSemantics::new(
        IntervalKind::Credible,
        0.9,
        InferentialScale::Link,
        "posterior quantiles",
    )
    .unwrap();

    assert_eq!(semantics.kind(), IntervalKind::Credible);
    assert!((semantics.level() - 0.9).abs() < f64::EPSILON);
    assert_eq!(semantics.scale(), InferentialScale::Link);
    assert_eq!(semantics.provenance(), "posterior quantiles");
}

fn effect_interval() -> IntervalSemantics {
    IntervalSemantics::new(
        IntervalKind::Confidence,
        0.95,
        InferentialScale::Response,
        "precomputed covariance projection",
    )
    .unwrap()
}

#[test]
fn effect_band_rejects_empty_provenance_and_simultaneous_identity() {
    assert!(EffectBandSemantics::pointwise(effect_interval(), "", "grid").is_err());
    assert!(EffectBandSemantics::pointwise(effect_interval(), "model", " ").is_err());
    assert!(
        EffectBandSemantics::simultaneous(effect_interval(), "", "x grid", "model", "grid",)
            .is_err()
    );
    assert!(
        EffectBandSemantics::simultaneous(effect_interval(), "Scheffe", " ", "model", "grid",)
            .is_err()
    );
}

#[test]
fn simultaneous_effect_band_retains_named_method_and_family() {
    let effect = EffectBandSemantics::simultaneous(
        effect_interval(),
        "Scheffe projection",
        "all x values in the declared grid",
        "externally fitted Gaussian model",
        "declared adjusted grid",
    )
    .unwrap();

    assert_eq!(effect.coverage(), EffectBandCoverage::Simultaneous);
    assert_eq!(effect.simultaneous_method(), Some("Scheffe projection"));
    assert_eq!(
        effect.simultaneous_family(),
        Some("all x values in the declared grid")
    );
    assert!(effect.label().contains("simultaneous"));
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
        (PlotKind::Regression, "regression plot", "regression-plot"),
        (PlotKind::Marginal, "marginal plot", "marginal-plot"),
        (PlotKind::Matrix, "scatter matrix", "scatter-matrix"),
        (PlotKind::MassSpectrum, "mass spectrum", "mass-spectrum"),
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
