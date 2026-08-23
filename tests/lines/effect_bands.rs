use super::*;

fn frame() -> DataFrame {
    df!(
        "x" => [2.0_f64, 0.0, 1.0],
        "estimate" => [6.0_f64, 2.0, 4.0],
        "lower" => [5.0_f64, 1.0, 3.0],
        "upper" => [7.0_f64, 3.0, 5.0]
    )
    .unwrap()
}

fn line(frame: &DataFrame) -> Plot<lichtung_plot::kind::Line> {
    Plot::line(data::numeric(frame, "x"), data::numeric(frame, "estimate")).band(Bounds::new(
        data::numeric(frame, "lower"),
        data::numeric(frame, "upper"),
    ))
}

#[test]
fn generic_line_band_uses_exact_precomputed_geometry() {
    let frame = frame();
    let output = render_svg(line(&frame), "line-band").unwrap();

    assert_eq!(output.matches("fill-opacity=\"0.18\"").count(), 1);
    assert_eq!(output.matches("stroke-width=\"1.8\"").count(), 1);
}

#[test]
fn line_band_rejects_invalid_bound_data_at_runtime() {
    let invalid_order = Plot::line(
        lichtung_plot::NumericValues::new("x", [0.0]),
        lichtung_plot::NumericValues::new("estimate", [2.0]),
    )
    .band(Bounds::new(
        lichtung_plot::NumericValues::new("lower", [3.0]),
        lichtung_plot::NumericValues::new("upper", [4.0]),
    ));
    let non_finite = Plot::line(
        lichtung_plot::NumericValues::new("x", [0.0]),
        lichtung_plot::NumericValues::new("estimate", [2.0]),
    )
    .band(Bounds::new(
        lichtung_plot::NumericValues::new("lower", [f64::NAN]),
        lichtung_plot::NumericValues::new("upper", [4.0]),
    ));
    let misaligned = Plot::line(
        lichtung_plot::NumericValues::new("x", [0.0]),
        lichtung_plot::NumericValues::new("estimate", [2.0]),
    )
    .band(Bounds::new(
        lichtung_plot::NumericValues::new("lower", [1.0, 1.5]),
        lichtung_plot::NumericValues::new("upper", [4.0]),
    ));

    assert!(invalid_order.preflight().is_err());
    assert!(non_finite.preflight().is_err());
    assert!(misaligned.preflight().is_err());
}
