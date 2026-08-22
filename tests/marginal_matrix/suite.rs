use lichtung_plot::{LegendPosition, Plot};
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
        "x" => [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0],
        "y" => [2.0_f64, 1.0, 4.0, 3.0, 6.0, 5.0],
        "group" => ["a", "a", "a", "b", "b", "b"]
    )
    .unwrap()
}

#[test]
fn grouped_marginal_renders_joint_and_density_panels() {
    let data = fixture();
    let directory = support::unique_dir("marginal");
    let output = directory.join("marginal.svg");
    Plot::marginal(native::numeric(&data, "x"), native::numeric(&data, "y"))
        .group(native::category(&data, "group"))
        .hue(native::category(&data, "group"))
        .legend_position(LegendPosition::OutsideRight)
        .title("Marginal distribution")
        .size(520, 400)
        .svg(&output)
        .unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.contains("Marginal distribution"));
    assert!(svg.matches("<g clip-path=").count() >= 3);
    assert!(svg.contains(">A</text>") && svg.contains(">B</text>"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn matrix_owns_ordered_named_variables_and_group_identity() {
    let data = fixture();
    let directory = support::unique_dir("matrix");
    let output = directory.join("matrix.svg");
    Plot::matrix(vec![
        native::numeric(&data, "x"),
        native::numeric(&data, "y"),
    ])
    .group(native::category(&data, "group"))
    .hue(native::category(&data, "group"))
    .title("Scatter matrix")
    .size(520, 440)
    .svg(&output)
    .unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.contains("Scatter matrix"));
    assert!(svg.contains(">X</text>") && svg.contains(">Y</text>"));
    assert!(svg.matches("<g clip-path=").count() >= 4);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn matrix_rejects_misaligned_and_degenerate_variables() {
    let misaligned = Plot::matrix(vec![
        lichtung_plot::NumericValues::new("x", [1.0, 2.0, 3.0]),
        lichtung_plot::NumericValues::new("y", [1.0, 2.0]),
    ]);
    assert!(misaligned.preflight().is_err());

    let constant = Plot::matrix(vec![
        lichtung_plot::NumericValues::new("x", [1.0, 1.0, 1.0]),
        lichtung_plot::NumericValues::new("y", [1.0, 2.0, 3.0]),
    ]);
    assert!(constant
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("constant"));
}
