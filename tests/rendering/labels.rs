use std::sync::atomic::{AtomicU64, Ordering};

use lichtung_plot::{BarAggregation, Chart, Figure, Labels};
use polars::prelude::*;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn render(data: &DataFrame, configure: impl FnOnce(Chart<'_>) -> Chart<'_>) -> String {
    let directory = std::env::temp_dir().join(format!(
        "lichtung-labels-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("plot.svg");
    configure(Chart::new(data)).render_to_svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

#[test]
fn direct_chart_humanizes_axes_categories_legend_title_and_entries() {
    let data = df!(
        "treatment_group" => ["control_group", "active_group"],
        "mean_score" => [1.0_f64, 2.0],
        "study_arm" => ["control_arm", "active_arm"]
    )
    .unwrap();
    let svg = render(&data, |mut chart| {
        chart = chart
            .hue("study_arm")
            .bar_aggregation(BarAggregation::Identity);
        chart.x("treatment_group").y("mean_score").horizontal_bar();
        chart
    });

    for label in [
        "Mean score",
        "Treatment group",
        "Control group",
        "Active group",
        "Study arm",
        "Control arm",
        "Active arm",
    ] {
        assert!(
            svg.contains(&format!(">{label}</text>")),
            "missing {label:?}"
        );
    }
}

#[test]
fn raw_mode_preserves_automatic_identity_text() {
    let data = df!(
        "x_value" => [1.0_f64, 2.0],
        "y_value" => [2.0_f64, 3.0]
    )
    .unwrap();
    let svg = render(&data, |mut chart| {
        chart = chart.labels(Labels::raw());
        chart.x("x_value").y("y_value").scatter();
        chart
    });

    assert!(svg.contains(">x_value</text>") && svg.contains(">y_value</text>"));
}

#[test]
fn raw_mode_preserves_quotes_in_legend_entries() {
    let data = df!(
        "x" => [1.0_f64, 2.0, 3.0],
        "y" => [2.0_f64, 3.0, 4.0],
        "group" => ["control", "\"control\"", "'active'"]
    )
    .unwrap();
    let svg = render(&data, |mut chart| {
        chart = chart.labels(Labels::raw()).hue("group");
        chart.x("x").y("y").scatter();
        chart
    });

    assert!(svg.contains(">control</text>"));
    assert!(svg.contains(">&quot;control&quot;</text>"));
    assert!(svg.contains(">&apos;active&apos;</text>"));
}

#[test]
fn svg_preserves_repeated_spaces_in_vertical_axis_labels() {
    let data = df!(
        "category" => ["first", "first", "second", "second"],
        "value" => [1.0_f64, 1.5, 2.0, 2.5]
    )
    .unwrap();
    let svg = render(&data, |mut chart| {
        chart = chart.y_label("Response  score");
        chart.x("category").y("value").bar();
        chart
    });
    let label = svg
        .lines()
        .find(|line| line.contains(">Response  score</text>"))
        .unwrap();

    assert!(label.contains("xml:space=\"preserve\"") && label.contains("rotate(-90.0"));
}

#[test]
fn aliases_override_every_automatic_horizontal_bar_surface_and_escape_svg() {
    let data = df!(
        "feature_name" => ["raw_feature"],
        "mean_score" => [2.0_f64]
    )
    .unwrap();
    let labels = Labels::humanized()
        .alias("mean_score", "Score\n(points)")
        .alias("feature_name", "Feature")
        .alias("raw_feature", "A & B <feature>");
    let svg = render(&data, |mut chart| {
        chart = chart
            .labels(labels)
            .bar_aggregation(BarAggregation::Identity);
        chart.x("feature_name").y("mean_score").horizontal_bar();
        chart
    });

    assert!(svg.contains(">Score</text>") && svg.contains(">(points)</text>"));
    assert!(svg.contains(">A &amp; B &lt;feature&gt;</text>"));
    assert!(!svg.contains("Score\n(points)"));
}

#[test]
fn ambiguous_display_labels_fail_during_preflight() {
    let data = df!(
        "group" => ["control_group", "control-group"],
        "value" => [1.0_f64, 2.0]
    )
    .unwrap();
    let mut chart = Chart::new(&data).bar_aggregation(BarAggregation::Identity);
    chart.x("group").y("value").bar();

    let error = chart.preflight().unwrap_err().to_string();

    assert!(error.contains("label collision in x-axis categories"));
    assert!(error.contains("Labels::raw()"));
}

#[test]
fn heatmap_and_matrix_apply_aliases_to_self_managed_labels() {
    let heatmap = df!(
        "raw_x" => ["first_level", "second_level"],
        "raw_y" => ["row_level", "row_level"],
        "raw_fill" => [1.0_f64, 2.0]
    )
    .unwrap();
    let labels = Labels::humanized()
        .alias("first_level", "First cell")
        .alias("row_level", "Only row")
        .alias("raw_fill", "Measured fill");
    let heatmap_svg = render(&heatmap, |mut chart| {
        chart = chart.labels(labels);
        let _ = chart.x("raw_x").y("raw_y");
        chart = chart.color("raw_fill");
        chart.heatmap();
        chart
    });
    assert!(heatmap_svg.contains(">First cell</text>"));
    assert!(heatmap_svg.contains(">Only row</text>"));
    assert!(heatmap_svg.contains(">Measured fill</text>"));

    let matrix = df!(
        "first_measure" => [1.0_f64, 2.0, 3.0],
        "second_measure" => [3.0_f64, 2.0, 1.0]
    )
    .unwrap();
    let matrix_svg = render(&matrix, |mut chart| {
        chart = chart.labels(Labels::humanized().alias("first_measure", "Primary measure"));
        chart
            .matrix_variables(vec!["first_measure", "second_measure"])
            .matrix();
        chart
    });
    assert!(matrix_svg.contains(">Primary measure</text>"));
    assert!(matrix_svg.contains(">Second measure</text>"));
}

#[test]
fn composed_figure_label_policy_is_inherited_by_every_panel() {
    let data = df!(
        "x_value" => [1.0_f64, 2.0],
        "y_value" => [2.0_f64, 3.0]
    )
    .unwrap();
    let mut left = Chart::new(&data);
    left.x("x_value").y("y_value").scatter();
    let mut right = Chart::new(&data);
    right.x("x_value").y("y_value").line();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, left)
        .unwrap()
        .panel(0, 1, right)
        .unwrap()
        .labels(Labels::raw());
    let directory = std::env::temp_dir().join(format!(
        "lichtung-label-figure-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("figure.svg");

    figure.render_to_svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();

    assert_eq!(svg.matches(">x_value</text>").count(), 2);
    assert_eq!(svg.matches(">y_value</text>").count(), 2);
}
