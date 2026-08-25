use std::sync::atomic::{AtomicU64, Ordering};

use lichtung_plot::{BarInput, Canvas, CategoryValues, Labels, NumericValues, Plot};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn render<K: lichtung_plot::kind::Marker>(plot: Plot<K>) -> String {
    let directory = std::env::temp_dir().join(format!(
        "lichtung-labels-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("plot.svg");
    plot.svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

fn horizontal_bar(
    treatment_group: &CategoryValues,
    mean_score: &NumericValues,
) -> Plot<lichtung_plot::kind::HorizontalBar> {
    Plot::horizontal_bar(
        treatment_group.clone(),
        BarInput::estimates(mean_score.clone()),
    )
}

#[test]
fn direct_plot_humanizes_axes_categories_legend_title_and_entries() {
    let treatment_group =
        CategoryValues::new("treatment_group", ["control_group", "active_group"]).unwrap();
    let mean_score = NumericValues::new("mean_score", [1.0, 2.0]);
    let study_arm = CategoryValues::new("study_arm", ["control_arm", "active_arm"]).unwrap();
    let svg = render(horizontal_bar(&treatment_group, &mean_score).series(study_arm));

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
    let x = NumericValues::new("x_value", [1.0, 2.0]);
    let y = NumericValues::new("y_value", [2.0, 3.0]);
    let svg = render(Plot::scatter(x, y).labels(Labels::raw()));

    assert!(svg.contains(">x_value</text>") && svg.contains(">y_value</text>"));
}

#[test]
fn raw_mode_preserves_quotes_in_legend_entries() {
    let x = NumericValues::new("x", [1.0, 2.0, 3.0]);
    let y = NumericValues::new("y", [2.0, 3.0, 4.0]);
    let group = CategoryValues::new("group", ["control", "\"control\"", "'active'"]).unwrap();
    let svg = render(Plot::scatter(x, y).labels(Labels::raw()).hue(group));

    assert!(svg.contains(">control</text>"));
    assert!(svg.contains(">&quot;control&quot;</text>"));
    assert!(svg.contains(">&apos;active&apos;</text>"));
}

#[test]
fn svg_preserves_repeated_spaces_in_vertical_axis_labels() {
    let category = CategoryValues::new("category", ["first", "first", "second", "second"]).unwrap();
    let value = NumericValues::new("value", [1.0, 1.5, 2.0, 2.5]);
    let svg = render(Plot::bar(category, BarInput::samples(value)).y_label("Response  score"));
    let label = svg
        .lines()
        .find(|line| line.contains(">Response  score</text>"))
        .unwrap();

    assert!(label.contains("xml:space=\"preserve\"") && label.contains("rotate(-90.0"));
}

#[test]
fn aliases_override_every_automatic_horizontal_bar_surface_and_escape_svg() {
    let feature_name = CategoryValues::new("feature_name", ["raw_feature"]).unwrap();
    let mean_score = NumericValues::new("mean_score", [2.0]);
    let labels = Labels::humanized()
        .alias("mean_score", "Score\n(points)")
        .alias("feature_name", "Feature")
        .alias("raw_feature", "A & B <feature>");
    let svg =
        render(Plot::horizontal_bar(feature_name, BarInput::estimates(mean_score)).labels(labels));

    assert!(svg.contains(">Score</text>") && svg.contains(">(points)</text>"));
    assert!(svg.contains(">A &amp; B &lt;feature&gt;</text>"));
    assert!(!svg.contains("Score\n(points)"));
}

#[test]
fn ambiguous_display_labels_fail_during_preflight() {
    let group = CategoryValues::new("group", ["control_group", "control-group"]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0]);
    let plot = Plot::bar(group, BarInput::estimates(value));

    let error = plot.preflight().unwrap_err().to_string();

    assert!(error.contains("label collision in x-axis categories"));
    assert!(error.contains("Labels::raw()"));
}

#[test]
fn heatmap_and_matrix_apply_aliases_to_self_managed_labels() {
    let raw_x = CategoryValues::new("raw_x", ["first_level", "second_level"]).unwrap();
    let raw_y = CategoryValues::new("raw_y", ["row_level", "row_level"]).unwrap();
    let raw_fill = NumericValues::new("raw_fill", [1.0, 2.0]);
    let labels = Labels::humanized()
        .alias("first_level", "First cell")
        .alias("row_level", "Only row")
        .alias("raw_fill", "Measured fill");
    let heatmap_svg = render(Plot::heatmap(raw_x, raw_y, raw_fill).labels(labels));
    assert!(heatmap_svg.contains(">First cell</text>"));
    assert!(heatmap_svg.contains(">Only row</text>"));
    assert!(heatmap_svg.contains(">Measured fill</text>"));

    let matrix_svg = render(
        Plot::matrix(vec![
            NumericValues::new("first_measure", [1.0, 2.0, 3.0]),
            NumericValues::new("second_measure", [3.0, 2.0, 1.0]),
        ])
        .labels(Labels::humanized().alias("first_measure", "Primary measure")),
    );
    assert!(matrix_svg.contains(">Primary measure</text>"));
    assert!(matrix_svg.contains(">Second measure</text>"));
}

#[test]
fn composed_canvas_label_policy_is_inherited_by_every_panel() {
    let x = NumericValues::new("x_value", [1.0, 2.0]);
    let y = NumericValues::new("y_value", [2.0, 3.0]);
    let left = Plot::scatter(x.clone(), y.clone());
    let right = Plot::line(x, y);
    let canvas = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, left)
        .unwrap()
        .panel(0, 1, right)
        .unwrap()
        .labels(Labels::raw());
    let directory = std::env::temp_dir().join(format!(
        "lichtung-label-canvas-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("canvas.svg");

    canvas.svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();

    assert_eq!(svg.matches(">x_value</text>").count(), 2);
    assert_eq!(svg.matches(">y_value</text>").count(), 2);
}
