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

fn interval() -> IntervalSemantics {
    IntervalSemantics::new(
        IntervalKind::Confidence,
        0.95,
        InferentialScale::Response,
        "precomputed projected covariance",
    )
    .unwrap()
}

fn pointwise() -> EffectBandSemantics {
    EffectBandSemantics::pointwise(
        interval(),
        "ordinary Gaussian model fit",
        "age grid with adjusted nuisance values",
    )
    .unwrap()
}

fn line(frame: &DataFrame) -> Plot {
    Plot::line(data::numeric(frame, "x"), data::numeric(frame, "estimate"))
        .interval_bounds(data::numeric(frame, "lower"), data::numeric(frame, "upper"))
}

#[test]
fn pointwise_and_external_simultaneous_bands_use_exact_precomputed_line_geometry() {
    let frame = frame();
    let simultaneous = EffectBandSemantics::simultaneous(
        interval(),
        "Scheffe projection",
        "all declared x coordinates",
        "external Gaussian model fit",
        "external adjusted grid",
    )
    .unwrap();
    let pointwise_svg = render_svg(
        line(&frame).effect_band_semantics(pointwise()),
        "pointwise-band",
    )
    .unwrap();
    let simultaneous_svg = render_svg(
        line(&frame).effect_band_semantics(simultaneous),
        "simultaneous-band",
    )
    .unwrap();
    assert_eq!(pointwise_svg.matches("fill-opacity=\"0.18\"").count(), 1);
    assert_eq!(pointwise_svg.matches("stroke-width=\"1.8\"").count(), 1);
    for provenance in [
        "ordinary Gaussian model fit",
        "age grid with adjusted nuisance values",
        "Scheffe projection",
        "all declared x coordinates",
        "external Gaussian model fit",
        "external adjusted grid",
    ] {
        assert!(!pointwise_svg.contains(provenance));
        assert!(!simultaneous_svg.contains(provenance));
    }
}

#[test]
fn invalid_effect_band_configurations_fail_preflight() {
    let frame = frame();
    let cases = [
        Plot::scatter(
            data::numeric(&frame, "x"),
            data::numeric(&frame, "estimate"),
        )
        .interval_bounds(
            data::numeric(&frame, "lower"),
            data::numeric(&frame, "upper"),
        )
        .effect_band_semantics(pointwise()),
        Plot::line(
            data::numeric(&frame, "x"),
            data::numeric(&frame, "estimate"),
        )
        .effect_band_semantics(pointwise()),
        line(&frame)
            .group(lichtung_plot::CategoryValues::new("group", ["a", "a", "a"]).unwrap())
            .effect_band_semantics(pointwise())
            .line_mode(LineMode::Trajectory),
    ];
    for plot in cases {
        assert!(plot.preflight().is_err());
    }
}

#[test]
fn generic_line_with_ribbon_keeps_generic_semantics() {
    let output = render_svg(line(&frame()), "generic-line-ribbon").unwrap();
    assert!(!output.contains("model-adjusted effect"));
    assert!(!output.contains("pointwise"));
}
