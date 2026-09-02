use lichtung_plot::{BarInput, CategoryValues, NumericValues, Plot};
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
        .svg_preserve_text(&out)
        .expect("render must succeed");
    let bytes = std::fs::read(&out).expect("file must exist");
    insta::assert_snapshot!(normalize_embedded_font(&String::from_utf8_lossy(&bytes)));
}

#[test]
fn matrix_clip_structure_snapshot() {
    let x = NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]);
    let y = NumericValues::new("y", [4.0, 3.0, 2.0, 1.0]);
    let group = CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap();
    let dir = tempdir();
    let out = dir.join("matrix-structure.svg");
    let chart = Plot::matrix(vec![x, y])
        .group(group.clone())
        .hue(group)
        .title("Matrix structural title");

    chart.size(360, 280).svg_preserve_text(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Matrix structural title", "Group"]
    ));
}

#[test]
fn beeswarm_clip_structure_snapshot() {
    let category = CategoryValues::new("category", ["one", "one", "two", "two"]).unwrap();
    let value = NumericValues::new("value", [-2.0, -1.0, 1.0, 2.0]);
    let color = NumericValues::new("color", [0.1, 0.3, 0.7, 1.0]);
    let dir = tempdir();
    let out = dir.join("beeswarm-structure.svg");
    let chart = Plot::beeswarm(category, value, color).title("Beeswarm structural title");

    chart.size(360, 280).svg_preserve_text(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Beeswarm structural title", "Color", "0.100", "1.000"]
    ));
}

#[test]
fn horizontal_bar_clip_structure_snapshot() {
    let category = CategoryValues::new("category", ["a notably long category", "short"]).unwrap();
    let value = NumericValues::new("value", [2.0, 4.0]);
    let dir = tempdir();
    let out = dir.join("horizontal-structure.svg");
    let chart = Plot::horizontal_bar(category, BarInput::estimates(value))
        .title("Horizontal structural title")
        .size(360, 280);

    chart.svg_preserve_text(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Horizontal structural title", "A notably long category"]
    ));
}

#[test]
fn marginal_clip_structure_snapshot() {
    let x = NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]);
    let y = NumericValues::new("y", [1.0, 4.0, 2.0, 3.0]);
    let dir = tempdir();
    let out = dir.join("marginal-structure.svg");
    let chart = Plot::marginal(x, y)
        .title("Marginal structural title")
        .size(360, 280);

    chart.svg_preserve_text(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Marginal structural title"]
    ));
}
