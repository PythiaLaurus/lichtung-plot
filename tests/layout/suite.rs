use lichtung_plot::layout::{deterministic_text_width, AxisSpec, AxisTick};
use lichtung_plot::{LegendPosition, Plot, Theme};

#[path = "../support/visual.rs"]
mod support;

#[test]
fn deterministic_text_measurement_distinguishes_wide_and_narrow_glyphs() {
    let narrow = deterministic_text_width("iiii", 12.0);
    let wide = deterministic_text_width("WWWW", 12.0);
    let repeated = deterministic_text_width("WWWW", 12.0);
    assert!(wide > narrow);
    assert!((wide - repeated).abs() < f64::EPSILON);
}

#[test]
fn axis_spec_validates_limits_and_explicit_ticks_before_rendering() {
    let valid = AxisSpec {
        limits: Some((0.0, 4.0)),
        ..AxisSpec::default()
    };
    let plot = Plot::scatter(
        lichtung_plot::NumericValues::new("x", [1.0, 2.0, 3.0]),
        lichtung_plot::NumericValues::new("y", [3.0, 1.0, 2.0]),
    )
    .x_limits(0.0, 4.0)
    .x_ticks(vec![AxisTick::new(0.0, "zero"), AxisTick::new(4.0, "four")]);
    assert_eq!(valid.limits, Some((0.0, 4.0)));
    plot.preflight().unwrap();
}

#[test]
fn outside_legend_stays_disjoint_from_public_svg_marks() {
    let plot = Plot::scatter(
        lichtung_plot::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        lichtung_plot::NumericValues::new("y", [3.0, 1.0, 4.0, 2.0]),
    )
    .hue(lichtung_plot::CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap())
    .legend_position(LegendPosition::OutsideRight)
    .theme(Theme::Nature)
    .size(480, 300);
    let directory = support::unique_dir("outside-legend");
    let output = directory.join("plot.svg");

    plot.svg(&output).unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    support::assert_legend_does_not_overlap_graphics(&svg, "scatter");
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn impossible_small_canvas_is_rejected_during_preflight() {
    let plot = Plot::scatter(
        lichtung_plot::NumericValues::new("x", [1.0, 2.0]),
        lichtung_plot::NumericValues::new("y", [2.0, 3.0]),
    )
    .size(100, 100);
    assert!(plot.preflight().is_err());
}
