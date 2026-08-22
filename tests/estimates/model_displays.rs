use super::*;

fn estimate_line(frame: &DataFrame) -> Plot {
    Plot::line(data::numeric(frame, "x"), data::numeric(frame, "estimate"))
        .group(data::category(frame, "trace"))
        .hue(data::category(frame, "trace"))
}

#[test]
fn line_is_distinct_deterministic_clipped_and_draws_ribbon_before_markers() {
    let build = || {
        let frame = estimate_data();
        estimate_line(&frame).interval_bounds(
            data::numeric(&frame, "lower"),
            data::numeric(&frame, "upper"),
        )
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
fn interaction_retains_typed_source_without_canvas_provenance() {
    let frame = estimate_data();
    let interaction = Plot::interaction(
        data::category(&frame, "factor"),
        data::numeric(&frame, "estimate"),
        data::category(&frame, "trace"),
        data::numeric(&frame, "lower"),
        data::numeric(&frame, "upper"),
    )
    .hue(data::category(&frame, "trace"))
    .interval_semantics(confidence(InferentialScale::Response))
    .interaction_source(EstimateSource::EstimatedMarginalMean);
    let svg = render_svg(interaction, "interaction-source").unwrap();
    assert!(!svg.contains("semantic-note"));
    assert!(!svg.contains("estimated marginal mean"));
    assert!(!svg.contains("model fit"));
}

#[test]
fn typed_inferential_provenance_never_consumes_canvas_space() {
    let frame = estimate_data();
    let provenance = "restricted maximum likelihood covariance with small-sample correction";
    let interval = || {
        IntervalSemantics::new(
            IntervalKind::Confidence,
            0.95,
            InferentialScale::Response,
            provenance,
        )
        .unwrap()
    };
    let coefficient = Plot::coefficient(
        data::category(&frame, "term"),
        data::numeric(&frame, "estimate"),
    )
    .interval_bounds(
        data::numeric(&frame, "lower"),
        data::numeric(&frame, "upper"),
    )
    .interval_semantics(interval())
    .size(360, 240);
    let interaction = Plot::interaction(
        data::category(&frame, "factor"),
        data::numeric(&frame, "estimate"),
        data::category(&frame, "trace"),
        data::numeric(&frame, "lower"),
        data::numeric(&frame, "upper"),
    )
    .interval_semantics(interval())
    .interaction_source(EstimateSource::EstimatedMarginalMean)
    .size(300, 280);
    let forest = Plot::forest(
        data::category(&frame, "term"),
        data::numeric(&frame, "estimate"),
        data::numeric(&frame, "lower"),
        data::numeric(&frame, "upper"),
    )
    .interval_semantics(
        IntervalSemantics::new(
            IntervalKind::Credible,
            0.9,
            InferentialScale::Ratio,
            provenance,
        )
        .unwrap(),
    )
    .reference_line(1.0)
    .size(360, 240);
    for (plot, name) in [
        (coefficient, "coefficient-provenance"),
        (interaction, "interaction-provenance"),
        (forest, "forest-provenance"),
    ] {
        let svg = render_svg(plot, name).unwrap();
        assert!(!svg.contains("semantic-note") && !svg.contains(provenance));
    }
}

#[test]
fn line_ribbons_and_point_intervals_require_and_retain_typed_semantics() {
    let frame = estimate_data();
    let provenance = "bootstrap percentile interval from 10000 deterministic resamples";
    let semantics = || {
        IntervalSemantics::new(
            IntervalKind::Confidence,
            0.95,
            InferentialScale::Response,
            provenance,
        )
        .unwrap()
    };
    let line = estimate_line(&frame)
        .interval_bounds(
            data::numeric(&frame, "lower"),
            data::numeric(&frame, "upper"),
        )
        .interval_semantics(semantics())
        .size(240, 240);
    let point = Plot::point_interval(
        data::category(&frame, "term"),
        data::numeric(&frame, "estimate"),
        data::numeric(&frame, "lower"),
        data::numeric(&frame, "upper"),
    )
    .interval_semantics(semantics())
    .size(360, 240);
    for (plot, name) in [(line, "line-semantics"), (point, "point-semantics")] {
        let svg = render_svg(plot, name).unwrap();
        assert!(!svg.contains("semantic-note") && !svg.contains(provenance));
    }
    assert!(estimate_line(&frame)
        .interval_semantics(semantics())
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("require complete lower and upper bound columns"));
}

#[test]
fn coefficient_allows_absent_intervals_while_forest_requires_reference() {
    let frame = estimate_data();
    Plot::coefficient(
        data::category(&frame, "term"),
        data::numeric(&frame, "estimate"),
    )
    .preflight()
    .unwrap();
    assert!(Plot::coefficient(
        data::category(&frame, "term"),
        data::numeric(&frame, "estimate"),
    )
    .interval_semantics(confidence(InferentialScale::Coefficient))
    .preflight()
    .unwrap_err()
    .to_string()
    .contains("require complete lower and upper bound columns"));
    assert!(Plot::forest(
        data::category(&frame, "term"),
        data::numeric(&frame, "estimate"),
        data::numeric(&frame, "lower"),
        data::numeric(&frame, "upper"),
    )
    .interval_semantics(confidence(InferentialScale::Coefficient))
    .preflight()
    .unwrap_err()
    .to_string()
    .contains("explicit scientific reference"));
}

#[test]
fn coefficient_fixed_value_labels_are_point_only_and_default_is_inert() {
    let frame = estimate_data();
    let base = || {
        Plot::coefficient(
            data::category(&frame, "term"),
            data::numeric(&frame, "estimate"),
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
    assert!(base()
        .interval_bounds(
            data::numeric(&frame, "lower"),
            data::numeric(&frame, "upper")
        )
        .interval_semantics(confidence(InferentialScale::Coefficient))
        .estimate_labels(NumericLabelFormat::new(2).unwrap())
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("point-only displays without intervals"));
}

#[test]
fn coefficient_confidence_intervals_render_exact_caller_significance_marks() {
    let frame = df!(
        "term" => ["none", "one", "two", "three"],
        "estimate" => [1.0_f64, 2.0, 3.0, 4.0],
        "lower" => [0.5_f64, 1.5, 2.5, 3.5],
        "upper" => [1.5_f64, 2.5, 3.5, 4.5],
        "mark" => [
            CoefficientSignificanceMark::None.as_str(),
            CoefficientSignificanceMark::OneStar.as_str(),
            CoefficientSignificanceMark::TwoStars.as_str(),
            CoefficientSignificanceMark::ThreeStars.as_str(),
        ]
    )
    .unwrap();
    let svg = render_svg(
        Plot::coefficient(
            data::category(&frame, "term"),
            data::numeric(&frame, "estimate"),
        )
        .interval_bounds(
            data::numeric(&frame, "lower"),
            data::numeric(&frame, "upper"),
        )
        .interval_semantics(confidence(InferentialScale::Coefficient))
        .coefficient_significance(data::category(&frame, "mark")),
        "coefficient-significance",
    )
    .unwrap();
    assert_eq!(svg.matches(">*</text>").count(), 1);
    assert_eq!(svg.matches(">**</text>").count(), 1);
    assert_eq!(svg.matches(">***</text>").count(), 1);
}

#[test]
fn coefficient_significance_marks_fail_closed_during_preflight() {
    let make = |mark: Option<&str>| {
        let marks = lichtung_plot::CategoryValues::nullable("mark", [mark]).unwrap();
        Plot::coefficient(
            lichtung_plot::CategoryValues::new("term", ["term"]).unwrap(),
            lichtung_plot::NumericValues::new("estimate", [1.0]),
        )
        .interval_bounds(
            lichtung_plot::NumericValues::new("lower", [0.5]),
            lichtung_plot::NumericValues::new("upper", [1.5]),
        )
        .interval_semantics(confidence(InferentialScale::Coefficient))
        .coefficient_significance(marks)
    };
    assert!(make(Some("****"))
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("expected exactly"));
    assert!(make(None)
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("null"));

    let missing = Plot::coefficient(
        lichtung_plot::CategoryValues::new("term", ["term"]).unwrap(),
        lichtung_plot::NumericValues::new("estimate", [1.0]),
    )
    .coefficient_significance(lichtung_plot::CategoryValues::new("mark", ["*"]).unwrap());
    assert!(missing
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("require complete confidence interval columns"));

    let credible = Plot::coefficient(
        lichtung_plot::CategoryValues::new("term", ["term"]).unwrap(),
        lichtung_plot::NumericValues::new("estimate", [1.0]),
    )
    .interval_bounds(
        lichtung_plot::NumericValues::new("lower", [0.5]),
        lichtung_plot::NumericValues::new("upper", [1.5]),
    )
    .interval_semantics(
        IntervalSemantics::new(
            IntervalKind::Credible,
            0.95,
            InferentialScale::Coefficient,
            "posterior",
        )
        .unwrap(),
    )
    .coefficient_significance(lichtung_plot::CategoryValues::new("mark", ["*"]).unwrap());
    assert!(credible
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("require confidence intervals, not credible intervals"));
}

#[test]
fn coefficient_significance_is_rejected_by_other_plot_kinds() {
    let frame = estimate_data();
    let error = Plot::point_interval(
        data::category(&frame, "term"),
        data::numeric(&frame, "estimate"),
        data::numeric(&frame, "lower"),
        data::numeric(&frame, "upper"),
    )
    .coefficient_significance(
        lichtung_plot::CategoryValues::new("mark", ["", "*", "**", "***"]).unwrap(),
    )
    .preflight()
    .unwrap_err();
    assert!(error.to_string().contains("supported only by Coefficient"));
}

#[test]
fn forest_log_scale_renders_raw_power_ticks_and_rejects_nonpositive_values() {
    let forest = |estimate: [f64; 2], lower: [f64; 2], upper: [f64; 2]| {
        Plot::forest(
            lichtung_plot::CategoryValues::new("term", ["a", "b"]).unwrap(),
            lichtung_plot::NumericValues::new("estimate", estimate),
            lichtung_plot::NumericValues::new("lower", lower),
            lichtung_plot::NumericValues::new("upper", upper),
        )
        .interval_semantics(confidence(InferentialScale::Ratio))
        .reference_line(1.0)
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
            lichtung_plot::NumericValues::new("lower", [1.05, 1.3]),
            lichtung_plot::NumericValues::new("upper", [1.2, 1.5]),
        )
        .interval_semantics(confidence(InferentialScale::Ratio))
        .reference_line(1.0)
        .axis_scale(AxisScale::Log10)
        .x_limits(1.0, 1.6),
        "forest-narrow-log",
    )
    .unwrap();
    assert!(
        svg.contains(">1</text>") && svg.contains(">1.3</text>") && svg.contains(">1.6</text>")
    );
}
