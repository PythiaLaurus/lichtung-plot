use std::sync::atomic::{AtomicU64, Ordering};

use lichtung_plot::{BarInput, CategoryValues, Labels, NumericValues, Plot};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn render<K: lichtung_plot::kind::Marker>(plot: impl std::borrow::Borrow<Plot<K>>) -> String {
    let directory = std::env::temp_dir().join(format!(
        "lichtung-labels-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("plot.svg");
    plot.borrow().svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

fn text_layer<'a>(svg: &'a str, name: &str) -> &'a str {
    svg.split_once(&format!("<g id=\"{name}\">"))
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0
}

fn horizontal_category_text(svg: &str, count: u32) -> Vec<String> {
    let panel = super::parse_first_clip_rect(svg);
    // Exclude the rotated title and other layers before joining wrapped text
    // within each bottom-first category slot. Keep XML entities and spaces exact.
    let axes = text_layer(svg, "axes")
        .lines()
        .filter(|line| !line.contains("transform="))
        .collect::<Vec<_>>()
        .join("\n");
    let text = super::parse_text_elements(&axes);
    (0..count)
        .map(|index| {
            let top = panel.y + panel.height * (1.0 - f64::from(index + 1) / f64::from(count));
            let bottom = panel.y + panel.height * (1.0 - f64::from(index) / f64::from(count));
            text.iter()
                .filter(|entry| entry.x < panel.x && entry.y >= top && entry.y < bottom)
                .map(|entry| entry.content.as_str())
                .collect()
        })
        .collect()
}

#[test]
fn aliases_override_every_automatic_horizontal_bar_surface_and_escape_svg() {
    let feature_name = CategoryValues::new("feature_name", ["raw_feature"]).unwrap();
    let mean_score = NumericValues::new("mean_score", [2.0]);
    let labels = Labels::humanized()
        .alias("mean_score", "Score\n(points)")
        .alias("feature_name", "Feature")
        .alias("raw_feature", "A & B <feature>");
    let resolved = labels.resolve("mean_score").unwrap();
    assert_eq!(resolved, "Score\n(points)");
    let svg =
        render(Plot::horizontal_bar(feature_name, BarInput::estimates(mean_score)).labels(labels));

    let axes = super::parse_text_elements(text_layer(&svg, "axes"));
    for label in resolved.lines().chain(std::iter::once("Feature")) {
        assert!(
            axes.iter().any(|entry| entry.content == label),
            "missing {label:?}"
        );
    }
    assert_eq!(
        horizontal_category_text(&svg, 1),
        ["A &amp; B &lt;feature&gt;"]
    );
    assert!(!svg.contains("Score\n(points)"));
}

#[test]
fn ambiguous_display_labels_fail_during_preflight() {
    let group = CategoryValues::new("group", ["control_group", "control-group"]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0]);
    let plot = Plot::bar(group, BarInput::estimates(value));

    let error = plot.clone().preflight().unwrap_err().to_string();

    assert!(error.contains("label collision in x-axis categories"));
    assert!(error.contains("Labels::raw()"));
    let error = plot
        .labels(Labels::raw().aliases([("control_group", ""), ("control-group", "")]))
        .preflight()
        .unwrap_err()
        .to_string();
    assert!(error.contains("label collision in x-axis categories"));
    assert!(error.contains("colliding aliases"));
}
