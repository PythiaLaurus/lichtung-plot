use lichtung_plot::{
    BarAggregation, ComparisonAnnotation, ComparisonEndpoint, PanelCorner, PanelTextAnnotation,
    Plot, Theme,
};
use polars::prelude::*;

#[allow(
    clippy::duplicate_mod,
    reason = "each explicit integration target is an independent crate"
)]
#[path = "../support/native.rs"]
mod native;

fn tempdir(name: &str) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "lichtung-annotations-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

#[test]
fn grouped_vertical_and_horizontal_bars_render_typed_comparisons() {
    let data = df!(
        "category" => ["a", "a", "b", "b"],
        "group" => ["left", "right", "left", "right"],
        "value" => [2.0_f64, 3.0, 4.0, 5.0]
    )
    .unwrap();
    let comparison = ComparisonAnnotation {
        endpoint_a: ComparisonEndpoint::new("a")
            .with_group("left")
            .with_hue("left"),
        endpoint_b: ComparisonEndpoint::new("a")
            .with_group("right")
            .with_hue("right"),
        ..ComparisonAnnotation::new("a", "a", 0.01)
    };
    let category = || native::category(&data, "category");
    let value = || native::numeric(&data, "value");
    let group = || native::category(&data, "group");
    let directory = tempdir("pairwise");
    let vertical = directory.join("vertical.svg");
    let horizontal = directory.join("horizontal.svg");

    Plot::bar(category(), value())
        .group(group())
        .hue(group())
        .bar_aggregation(BarAggregation::Identity)
        .add_comparisons([comparison.clone()])
        .svg(&vertical)
        .unwrap();
    Plot::horizontal_bar(category(), value())
        .group(group())
        .hue(group())
        .bar_aggregation(BarAggregation::Identity)
        .add_comparisons([comparison])
        .svg(&horizontal)
        .unwrap();

    assert_eq!(
        std::fs::read_to_string(vertical)
            .unwrap()
            .matches("class=\"significance-bracket\"")
            .count(),
        1
    );
    assert!(std::fs::read_to_string(horizontal)
        .unwrap()
        .contains("class=\"significance-bracket horizontal\""));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn panel_text_uses_named_clipped_svg_layer_and_corner_anchors() {
    let data = df!(
        "x" => [1.0_f64, 2.0, 3.0],
        "y" => [2.0_f64, 1.0, 3.0]
    )
    .unwrap();
    let directory = tempdir("panel-text");
    let output = directory.join("plot.svg");
    Plot::scatter(native::numeric(&data, "x"), native::numeric(&data, "y"))
        .panel_text_annotation(
            PanelTextAnnotation::new(PanelCorner::TopLeft, "Upper note").unwrap(),
        )
        .panel_text_annotation(
            PanelTextAnnotation::new(PanelCorner::BottomRight, "Lower note").unwrap(),
        )
        .theme(Theme::Nature)
        .svg(&output)
        .unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    let layer = svg
        .split_once("<g id=\"panel-text\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;
    assert!(layer.contains("clip-path=\"url(#clip-"));
    assert!(layer
        .lines()
        .find(|line| line.contains("Upper note"))
        .unwrap()
        .contains("text-anchor=\"start\""));
    assert!(layer
        .lines()
        .find(|line| line.contains("Lower note"))
        .unwrap()
        .contains("text-anchor=\"end\""));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn panel_text_constructor_and_duplicate_corners_are_rejected() {
    for text in ["", "   ", " leading", "trailing ", "first\nsecond"] {
        assert!(PanelTextAnnotation::new(PanelCorner::TopLeft, text).is_err());
    }
    let plot = Plot::scatter(
        lichtung_plot::NumericValues::new("x", [1.0, 2.0]),
        lichtung_plot::NumericValues::new("y", [2.0, 1.0]),
    )
    .panel_text_annotation(PanelTextAnnotation::new(PanelCorner::TopLeft, "First").unwrap())
    .panel_text_annotation(PanelTextAnnotation::new(PanelCorner::TopLeft, "Second").unwrap());

    assert!(plot
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("duplicate panel text annotation corner"));
}
