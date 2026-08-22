use lichtung_plot::{BarAggregation, BarSpacing, DumbbellEndpoints, Labels, Plot};
use polars::prelude::*;

#[allow(
    clippy::duplicate_mod,
    reason = "each explicit integration target is an independent crate"
)]
#[path = "../support/native.rs"]
mod native;
#[path = "../support/visual.rs"]
mod support;

fn fixture() -> DataFrame {
    df!(
        "category" => ["a", "a", "a", "b", "b", "b"],
        "group" => ["control", "active", "control", "control", "active", "active"],
        "value" => [1.0_f64, 2.0, 1.5, 3.0, 4.0, 3.5],
        "right" => [1.4_f64, 2.4, 1.8, 3.4, 4.4, 3.9]
    )
    .unwrap()
}

#[test]
fn categorical_constructors_preflight_complete_native_inputs() {
    let data = fixture();
    let category = || native::category(&data, "category");
    let value = || native::numeric(&data, "value");
    let group = || native::category(&data, "group");

    let plots = vec![
        Plot::bar(category(), value()),
        Plot::horizontal_bar(category(), value()),
        Plot::stacked_bar(category(), value(), group()).hue(group()),
        Plot::stacked_horizontal_bar(category(), value(), group()).hue(group()),
        Plot::boxplot(category(), value()),
        Plot::violin(category(), value()),
        Plot::dumbbell(
            lichtung_plot::CategoryValues::new("category", ["a", "b"]).unwrap(),
            lichtung_plot::NumericValues::new("value", [1.0, 3.0]),
            lichtung_plot::NumericValues::new("right", [1.4, 3.4]),
        )
        .dumbbell_endpoints(
            DumbbellEndpoints::new("Before", (49, 130, 189), "After", (222, 45, 38)).unwrap(),
        ),
    ];

    for plot in plots {
        plot.preflight().unwrap();
    }
}

#[test]
fn grouped_bar_spacing_remains_observable_in_public_svg() {
    let compact = support::grouped_bar_internal_gap(BarSpacing::Compact);
    let comfort = support::grouped_bar_internal_gap(BarSpacing::Comfort);
    assert!(comfort > compact);
}

#[test]
fn categorical_labels_and_hue_identity_render_to_svg() {
    let data = df!(
        "category" => ["a", "a", "b", "b"],
        "group" => ["control", "active", "control", "active"],
        "value" => [1.0_f64, 2.0, 3.0, 4.0]
    )
    .unwrap();
    let directory = support::unique_dir("categorical-labels");
    let output = directory.join("bar.svg");
    let plot = Plot::bar(
        native::category(&data, "category"),
        native::numeric(&data, "value"),
    )
    .group(native::category(&data, "group"))
    .hue(native::category(&data, "group"))
    .bar_aggregation(BarAggregation::Identity)
    .labels(Labels::raw())
    .title("Grouped outcomes")
    .size(420, 300);

    plot.svg(&output).unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.contains("Grouped outcomes"));
    assert!(svg.contains(">control</text>"));
    assert!(svg.contains(">active</text>"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn null_categorical_values_fail_without_opening_an_output_path() {
    let category = lichtung_plot::CategoryValues::nullable("category", [Some("a"), None]).unwrap();
    let value = lichtung_plot::NumericValues::new("value", [1.0, 2.0]);
    let directory = support::unique_dir("categorical-null");
    let output = directory.join("missing").join("plot.svg");

    let error = Plot::bar(category, value)
        .bar_aggregation(BarAggregation::Identity)
        .svg(&output)
        .err()
        .unwrap();

    assert!(error.to_string().contains("contains null"));
    assert!(!output.exists());
    std::fs::remove_dir_all(directory).unwrap();
}
