use lichtung_plot::{BarAggregation, Plot};
#[allow(
    clippy::duplicate_mod,
    reason = "each explicit integration target is an independent crate"
)]
#[path = "../support/data.rs"]
mod data;
#[path = "../support/visual.rs"]
mod support;

use support::*;

#[test]
fn scatter_smoke_snapshot() {
    let (x, y) = fixture_values();
    let dir = tempdir();
    let out = dir.join("scatter.svg");
    Plot::scatter(x, y)
        .size(320, 240)
        .svg(&out)
        .expect("render must succeed");
    let bytes = std::fs::read(&out).expect("file must exist");
    insta::assert_snapshot!(normalize_embedded_font(&String::from_utf8_lossy(&bytes)));
}

#[test]
fn matrix_clip_structure_snapshot() {
    let data = data::df!(
        "x" => [1.0_f64, 2.0, 3.0, 4.0],
        "y" => [4.0_f64, 3.0, 2.0, 1.0],
        "group" => ["a", "a", "b", "b"]
    )
    .unwrap();
    let dir = tempdir();
    let out = dir.join("matrix-structure.svg");
    let chart = Plot::matrix(vec![data::numeric(&data, "x"), data::numeric(&data, "y")])
        .group(data::category(&data, "group"))
        .hue(data::category(&data, "group"))
        .title("Matrix structural title");

    chart.size(360, 280).svg(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Matrix structural title", "Group"]
    ));
}

#[test]
fn beeswarm_clip_structure_snapshot() {
    let data = data::df!(
        "category" => ["first", "first", "second", "second"],
        "value" => [-2.0_f64, -1.0, 1.0, 2.0],
        "color" => [0.1_f64, 0.3, 0.7, 1.0]
    )
    .unwrap();
    let dir = tempdir();
    let out = dir.join("beeswarm-structure.svg");
    let chart = Plot::beeswarm(
        data::category(&data, "category"),
        data::numeric(&data, "value"),
        data::numeric(&data, "color"),
    )
    .title("Beeswarm structural title");

    chart.size(360, 280).svg(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Beeswarm structural title", "Color", "0.100", "1.000"]
    ));
}

#[test]
fn horizontal_bar_clip_structure_snapshot() {
    let data = data::df!(
        "category" => ["a deliberately long category", "short"],
        "value" => [2.0_f64, 4.0]
    )
    .unwrap();
    let dir = tempdir();
    let out = dir.join("horizontal-structure.svg");
    let chart = Plot::horizontal_bar(
        data::category(&data, "category"),
        data::numeric(&data, "value"),
    )
    .title("Horizontal structural title")
    .bar_aggregation(BarAggregation::Identity)
    .size(360, 280);

    chart.svg(&out).unwrap();
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
    let data = data::df!(
        "x" => [1.0_f64, 2.0, 3.0, 4.0],
        "y" => [1.0_f64, 4.0, 2.0, 3.0]
    )
    .unwrap();
    let dir = tempdir();
    let out = dir.join("marginal-structure.svg");
    let chart = Plot::marginal(data::numeric(&data, "x"), data::numeric(&data, "y"))
        .title("Marginal structural title")
        .size(360, 280);

    chart.svg(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Marginal structural title"]
    ));
}
