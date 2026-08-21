use lichtung_plot::{BarAggregation, Chart};
use polars::prelude::*;

#[path = "../support/visual.rs"]
mod support;

use support::*;

#[test]
fn scatter_smoke_snapshot() {
    let df = fixture_df();
    let dir = tempdir();
    let out = dir.join("scatter.svg");
    let mut chart = Chart::new(&df);
    chart.x("x").y("y").size(320, 240).scatter();
    chart.render_to_svg(&out).expect("render must succeed");
    let bytes = std::fs::read(&out).expect("file must exist");
    insta::assert_snapshot!(normalize_embedded_font(&String::from_utf8_lossy(&bytes)));
}

#[test]
fn matrix_clip_structure_snapshot() {
    let df = df!(
        "x" => [1.0_f64, 2.0, 3.0, 4.0],
        "y" => [4.0_f64, 3.0, 2.0, 1.0],
        "group" => ["a", "a", "b", "b"]
    )
    .unwrap();
    let dir = tempdir();
    let out = dir.join("matrix-structure.svg");
    let mut chart = Chart::new(&df)
        .group("group")
        .hue("group")
        .title("Matrix structural title");
    chart
        .size(360, 280)
        .matrix_variables(vec!["x", "y"])
        .matrix();

    chart.render_to_svg(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Matrix structural title", "Group"]
    ));
}

#[test]
fn beeswarm_clip_structure_snapshot() {
    let df = df!(
        "category" => ["first", "first", "second", "second"],
        "value" => [-2.0_f64, -1.0, 1.0, 2.0],
        "color" => [0.1_f64, 0.3, 0.7, 1.0]
    )
    .unwrap();
    let dir = tempdir();
    let out = dir.join("beeswarm-structure.svg");
    let mut chart = Chart::new(&df)
        .color("color")
        .title("Beeswarm structural title");
    chart.x("value").y("category").size(360, 280).beeswarm();

    chart.render_to_svg(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Beeswarm structural title", "Color", "0.100", "1.000"]
    ));
}

#[test]
fn horizontal_bar_clip_structure_snapshot() {
    let df = df!(
        "category" => ["a deliberately long category", "short"],
        "value" => [2.0_f64, 4.0]
    )
    .unwrap();
    let dir = tempdir();
    let out = dir.join("horizontal-structure.svg");
    let mut chart = Chart::new(&df)
        .title("Horizontal structural title")
        .bar_aggregation(BarAggregation::Identity);
    chart
        .x("category")
        .y("value")
        .size(360, 280)
        .horizontal_bar();

    chart.render_to_svg(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &[
            "Horizontal structural title",
            "A deliberately long category"
        ]
    ));
}

#[test]
fn marginal_clip_structure_snapshot() {
    let df = df!(
        "x" => [1.0_f64, 2.0, 3.0, 4.0],
        "y" => [1.0_f64, 4.0, 2.0, 3.0]
    )
    .unwrap();
    let dir = tempdir();
    let out = dir.join("marginal-structure.svg");
    let mut chart = Chart::new(&df).title("Marginal structural title");
    chart.x("x").y("y").size(360, 280).marginal();

    chart.render_to_svg(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Marginal structural title"]
    ));
}
