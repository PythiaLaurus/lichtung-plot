use super::*;

struct EffectBandData {
    x: NumericValues,
    estimate: NumericValues,
    lower: NumericValues,
    upper: NumericValues,
}

fn effect_band_data() -> EffectBandData {
    EffectBandData {
        x: NumericValues::new("x", [2.0, 0.0, 1.0]),
        estimate: NumericValues::new("estimate", [6.0, 2.0, 4.0]),
        lower: NumericValues::new("lower", [5.0, 1.0, 3.0]),
        upper: NumericValues::new("upper", [7.0, 3.0, 5.0]),
    }
}

fn line(data: &EffectBandData) -> Plot<lichtung_plot::kind::Line> {
    Plot::line(data.x.clone(), data.estimate.clone())
        .band(Bounds::new(data.lower.clone(), data.upper.clone()))
}

#[test]
fn generic_line_band_uses_exact_precomputed_geometry() {
    let data = effect_band_data();
    let output = render_svg(line(&data), "line-band").unwrap();

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
