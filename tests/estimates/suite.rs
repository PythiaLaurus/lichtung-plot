use lichtung_plot::{
    EstimateSource, InferentialScale, IntervalKind, IntervalSemantics, Plot, PlotKind, ValueSort,
};
use polars::prelude::*;

#[allow(
    clippy::duplicate_mod,
    reason = "each explicit integration target is an independent crate"
)]
#[path = "../support/native.rs"]
mod native;

fn fixture() -> DataFrame {
    df!(
        "term" => ["intercept", "time", "group"],
        "estimate" => [1.0_f64, -2.0, 4.0],
        "lower" => [-1.0_f64, -2.5, 2.0],
        "upper" => [5.0_f64, 0.0, 5.0]
    )
    .unwrap()
}

fn confidence() -> IntervalSemantics {
    IntervalSemantics::new(
        IntervalKind::Confidence,
        0.95,
        InferentialScale::Response,
        "registered model",
    )
    .unwrap()
}

#[test]
fn point_interval_and_forest_own_complete_interval_inputs() {
    let data = fixture();
    let term = || native::category(&data, "term");
    let estimate = || native::numeric(&data, "estimate");
    let lower = || native::numeric(&data, "lower");
    let upper = || native::numeric(&data, "upper");

    let point = Plot::point_interval(term(), estimate(), lower(), upper())
        .interval_semantics(confidence())
        .value_sort(ValueSort::Descending);
    let forest = Plot::forest(term(), estimate(), lower(), upper())
        .interval_semantics(confidence())
        .reference_line(1.0)
        .value_sort(ValueSort::Ascending);

    assert_eq!(point.plot_kind(), Some(PlotKind::PointInterval));
    assert_eq!(forest.plot_kind(), Some(PlotKind::Forest));
    point.preflight().unwrap();
    forest.preflight().unwrap();
}

#[test]
fn interaction_constructor_requires_and_renders_complete_estimates() {
    let data = df!(
        "condition" => ["before", "after", "before", "after"],
        "estimate" => [2.0_f64, 3.0, 2.5, 4.0],
        "lower" => [1.5_f64, 2.5, 2.0, 3.5],
        "upper" => [2.5_f64, 3.5, 3.0, 4.5],
        "trace" => ["control", "control", "active", "active"]
    )
    .unwrap();
    let directory = std::env::temp_dir().join(format!(
        "lichtung-interaction-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("interaction.svg");

    Plot::interaction(
        native::category(&data, "condition"),
        native::numeric(&data, "estimate"),
        native::category(&data, "trace"),
        native::numeric(&data, "lower"),
        native::numeric(&data, "upper"),
    )
    .hue(native::category(&data, "trace"))
    .interval_semantics(confidence())
    .interaction_source(EstimateSource::EstimatedMarginalMean)
    .svg(&output)
    .unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.contains(">Control</text>") && svg.contains(">Active</text>"));
    assert!(!svg.contains("registered model"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn coefficient_constructor_accepts_direct_estimates_without_selector_state() {
    let data = fixture();
    let plot = Plot::coefficient(
        native::category(&data, "term"),
        native::numeric(&data, "estimate"),
    );

    assert_eq!(plot.plot_kind(), Some(PlotKind::Coefficient));
    plot.preflight().unwrap();
}

#[test]
fn interval_bounds_must_contain_each_estimate() {
    let error = Plot::point_interval(
        lichtung_plot::CategoryValues::new("term", ["a"]).unwrap(),
        lichtung_plot::NumericValues::new("estimate", [2.0]),
        lichtung_plot::NumericValues::new("lower", [3.0]),
        lichtung_plot::NumericValues::new("upper", [4.0]),
    )
    .interval_semantics(confidence())
    .preflight()
    .unwrap_err();

    assert!(error.to_string().contains("lower <= estimate <= upper"));
}
