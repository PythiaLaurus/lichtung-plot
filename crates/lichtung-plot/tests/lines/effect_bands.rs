use super::*;

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
