use super::*;

fn estimate_line(frame: &DataFrame) -> Plot<lichtung_plot::kind::Line> {
    Plot::line(data::numeric(frame, "x"), data::numeric(frame, "estimate"))
        .group(data::category(frame, "trace"))
        .hue(data::category(frame, "trace"))
}

#[test]
fn line_is_distinct_deterministic_clipped_and_draws_ribbon_before_markers() {
    let build = || {
        let frame = estimate_data();
        estimate_line(&frame).band(Bounds::new(
            data::numeric(&frame, "lower"),
            data::numeric(&frame, "upper"),
        ))
    };
    let first = render_svg(build(), "estimate-line-first").unwrap();
    let second = render_svg(build(), "estimate-line-second").unwrap();
    assert_eq!(first, second);
    assert_eq!(first.matches("<g clip-path=").count(), 1);
    let marks = first.find("<g clip-path=").unwrap();
    assert!(first[marks..].find("<polygon ").unwrap() < first[marks..].find("<polyline ").unwrap());
}

#[test]
fn line_rejects_duplicate_x_within_hue_series_including_signed_zero() {
    let duplicate = Plot::line(
        lichtung_plot::NumericValues::new("x", [1.0, 1.0]),
        lichtung_plot::NumericValues::new("y", [2.0, 3.0]),
    )
    .hue(lichtung_plot::CategoryValues::new("trace", ["a", "a"]).unwrap());
    assert!(duplicate
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("duplicate x value within series"));

    let signed_zero = Plot::line(
        lichtung_plot::NumericValues::new("x", [-0.0, 0.0]),
        lichtung_plot::NumericValues::new("y", [2.0, 3.0]),
    );
    assert!(signed_zero
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("duplicate x value within series"));
}

#[test]
fn interaction_and_forest_constructors_accept_complete_atomic_bounds() {
    let frame = estimate_data();
    let bounds = || {
        Bounds::new(
            data::numeric(&frame, "lower"),
            data::numeric(&frame, "upper"),
        )
    };
    Plot::interaction(
        data::category(&frame, "factor"),
        data::numeric(&frame, "estimate"),
        data::category(&frame, "trace"),
        bounds(),
    )
    .preflight()
    .unwrap();
    Plot::forest(
        data::category(&frame, "term"),
        data::numeric(&frame, "estimate"),
        bounds(),
        0.0,
    )
    .preflight()
    .unwrap();
}

#[test]
fn coefficient_fixed_value_labels_are_point_only_and_default_is_inert() {
    let frame = estimate_data();
    let base = || {
        Plot::coefficient(
            data::category(&frame, "term"),
            CoefficientInput::estimates(data::numeric(&frame, "estimate")),
        )
    };
    let default = render_svg(base(), "coefficient-default-labels").unwrap();
    let labeled = render_svg(
        base().estimate_labels(NumericLabelFormat::with_suffix(2, " units").unwrap()),
        "coefficient-value-labels",
    )
    .unwrap();
    assert!(!default.contains(" units</text>"));
    assert!(labeled.contains(" units</text>"));
    assert_eq!(
        default.matches("<circle ").count(),
        labeled.matches("<circle ").count()
    );
    assert!(Plot::coefficient(
        data::category(&frame, "term"),
        CoefficientInput::estimates_with_intervals(
            data::numeric(&frame, "estimate"),
            Bounds::new(
                data::numeric(&frame, "lower"),
                data::numeric(&frame, "upper"),
            ),
        ),
    )
    .estimate_labels(NumericLabelFormat::new(2).unwrap())
    .preflight()
    .unwrap_err()
    .to_string()
    .contains("point-only displays without intervals"));
}

#[test]
fn coefficient_bounds_render_exact_caller_significance_marks() {
    let frame = df!(
        "term" => ["none", "one", "two", "three", "four", "p-value"],
        "estimate" => [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0],
        "lower" => [0.5_f64, 1.5, 2.5, 3.5, 4.5, 5.5],
        "upper" => [1.5_f64, 2.5, 3.5, 4.5, 5.5, 6.5]
    )
    .unwrap();
    let svg = render_svg(
        Plot::coefficient(
            data::category(&frame, "term"),
            CoefficientInput::estimates_with_intervals(
                data::numeric(&frame, "estimate"),
                Bounds::new(
                    data::numeric(&frame, "lower"),
                    data::numeric(&frame, "upper"),
                ),
            ),
        )
        .significance([
            Significance::at("none", SignificanceMark::NotSignificant),
            Significance::at("one", SignificanceMark::OneStar),
            Significance::at("two", SignificanceMark::TwoStars),
            Significance::at("three", SignificanceMark::ThreeStars),
            Significance::at("four", SignificanceMark::FourStars),
            Significance::at("p-value", SignificanceMark::PValue(0.008)),
        ]),
        "coefficient-significance",
    )
    .unwrap();
    assert_eq!(svg.matches(">*</text>").count(), 1);
    assert_eq!(svg.matches(">**</text>").count(), 2);
    assert_eq!(svg.matches(">***</text>").count(), 1);
    assert_eq!(svg.matches(">****</text>").count(), 1);
    assert_eq!(svg.matches(">ns</text>").count(), 1);
}

#[test]
fn coefficient_significance_renders_without_bounds() {
    let output = render_svg(
        Plot::coefficient(
            lichtung_plot::CategoryValues::new("term", ["term"]).unwrap(),
            CoefficientInput::estimates(lichtung_plot::NumericValues::new("estimate", [1.0])),
        )
        .significance([Significance::at("term", SignificanceMark::OneStar)]),
        "point-only-coefficient-significance",
    )
    .unwrap();

    assert!(output.contains(">*</text>"));
}

#[test]
fn coefficient_significance_rejects_unknown_terms() {
    let unknown = Plot::coefficient(
        lichtung_plot::CategoryValues::new("term", ["term"]).unwrap(),
        CoefficientInput::estimates(lichtung_plot::NumericValues::new("estimate", [1.0])),
    )
    .significance([Significance::at("missing", SignificanceMark::OneStar)]);
    assert!(unknown
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("unknown term"));
}

#[test]
fn forest_log_scale_renders_raw_power_ticks_and_rejects_nonpositive_values() {
    let forest = |estimate: [f64; 2], lower: [f64; 2], upper: [f64; 2]| {
        Plot::forest(
            lichtung_plot::CategoryValues::new("term", ["a", "b"]).unwrap(),
            lichtung_plot::NumericValues::new("estimate", estimate),
            Bounds::new(
                lichtung_plot::NumericValues::new("lower", lower),
                lichtung_plot::NumericValues::new("upper", upper),
            ),
            1.0,
        )
        .axis_scale(AxisScale::Log10)
    };
    let svg = render_svg(forest([0.5, 2.0], [0.1, 1.0], [1.0, 10.0]), "forest-log").unwrap();
    assert!(svg.contains(">0.1</text>") && svg.contains(">1</text>") && svg.contains(">10</text>"));
    assert!(forest([0.0, 1.0], [0.0, 0.5], [1.0, 2.0])
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("strictly positive"));
}

#[test]
fn forest_narrow_log_range_keeps_raw_multiplicative_ticks() {
    let svg = render_svg(
        Plot::forest(
            lichtung_plot::CategoryValues::new("term", ["a", "b"]).unwrap(),
            lichtung_plot::NumericValues::new("estimate", [1.1, 1.4]),
            Bounds::new(
                lichtung_plot::NumericValues::new("lower", [1.05, 1.3]),
                lichtung_plot::NumericValues::new("upper", [1.2, 1.5]),
            ),
            1.0,
        )
        .axis_scale(AxisScale::Log10)
        .x_limits(1.0, 1.6),
        "forest-narrow-log",
    )
    .unwrap();
    assert!(
        svg.contains(">1</text>") && svg.contains(">1.3</text>") && svg.contains(">1.6</text>")
    );
}
