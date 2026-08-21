use super::*;

#[test]
fn line_is_distinct_deterministic_clipped_and_draws_ribbon_before_markers() {
    let data = estimate_data();
    let dir = tempdir("line");
    let first = dir.join("first.svg");
    let second = dir.join("second.svg");
    for path in [&first, &second] {
        let mut chart = Chart::new(&data)
            .group("trace")
            .hue("trace")
            .interval_bounds("lower", "upper");
        chart.x("x").y("estimate").line();
        chart.render_to_svg(path).unwrap();
    }
    let svg = std::fs::read_to_string(first).unwrap();
    assert_eq!(svg, std::fs::read_to_string(second).unwrap());
    assert_eq!(svg.matches("<g clip-path=").count(), 1);
    let marks = svg.find("<g clip-path=").unwrap();
    assert!(svg[marks..].find("<polygon ").unwrap() < svg[marks..].find("<polyline ").unwrap());
}

#[test]
fn line_rejects_duplicate_x_within_hue_series() {
    let data = df!(
        "x" => [1.0_f64, 1.0],
        "y" => [2.0_f64, 3.0],
        "trace" => ["a", "a"]
    )
    .unwrap();
    let mut chart = Chart::new(&data).hue("trace");
    chart.x("x").y("y").line();
    let error = chart
        .render_with_backend(&mut LifecycleBackend::default())
        .unwrap_err()
        .to_string();
    assert!(error.contains("duplicate x value within series"));
}

#[test]
fn line_treats_signed_zero_as_a_duplicate_x_value() {
    let data = df!(
        "x" => [-0.0_f64, 0.0],
        "y" => [2.0_f64, 3.0]
    )
    .unwrap();
    let mut chart = Chart::new(&data);
    chart.x("x").y("y").line();

    let error = chart
        .render_with_backend(&mut LifecycleBackend::default())
        .unwrap_err()
        .to_string();

    assert!(error.contains("duplicate x value within series"));
}

#[test]
fn interaction_retains_typed_source_without_canvas_provenance() {
    let data = estimate_data();
    let dir = tempdir("interaction");
    let output = dir.join("interaction.svg");
    let mut chart = Chart::new(&data)
        .group("trace")
        .hue("trace")
        .interval_bounds("lower", "upper")
        .interval_semantics(confidence(InferentialScale::Response))
        .interaction_source(EstimateSource::EstimatedMarginalMean);
    chart.x("factor").y("estimate").interaction();
    chart.render_to_svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    assert!(!svg.contains("semantic-note"));
    assert!(!svg.contains("estimated marginal mean"));
    assert!(!svg.contains("model fit"));

    let mut invalid = Chart::new(&data)
        .group("trace")
        .hue("trace")
        .interaction_source(EstimateSource::ObservedCellSummary);
    invalid.x("factor").y("estimate").interaction();
    assert!(invalid
        .render_with_backend(&mut LifecycleBackend::default())
        .unwrap_err()
        .to_string()
        .contains("complete precomputed interval"));
}

#[test]
fn typed_inferential_provenance_never_consumes_canvas_space() {
    let data = estimate_data();
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
    let dir = tempdir("long-inferential-labels");

    let narrow_data = data.slice(0, 1);
    let mut narrow = Chart::new(&narrow_data)
        .interval_bounds("lower", "upper")
        .interval_semantics(interval())
        .x_label("")
        .y_label("");
    narrow.x("estimate").y("term").size(180, 140).coefficient();
    let mut backend = LifecycleBackend::default();
    narrow.render_with_backend(&mut backend).unwrap();
    assert_eq!((backend.begins, backend.ends), (1, 1));

    let interaction_path = dir.join("interaction.svg");
    let mut interaction = Chart::new(&data)
        .group("trace")
        .hue("trace")
        .interval_bounds("lower", "upper")
        .interval_semantics(interval())
        .interaction_source(EstimateSource::EstimatedMarginalMean);
    interaction
        .x("factor")
        .y("estimate")
        .size(300, 280)
        .interaction();
    interaction.render_to_svg(&interaction_path).unwrap();
    let interaction_svg = std::fs::read_to_string(interaction_path).unwrap();
    assert!(!interaction_svg.contains("semantic-note"));
    assert!(!interaction_svg.contains(provenance));

    let coefficient_path = dir.join("coefficient.svg");
    let mut coefficient = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .interval_semantics(interval());
    coefficient
        .x("estimate")
        .y("term")
        .size(360, 240)
        .coefficient();
    coefficient.render_to_svg(&coefficient_path).unwrap();
    let coefficient_svg = std::fs::read_to_string(coefficient_path).unwrap();
    assert!(!coefficient_svg.contains("semantic-note"));
    assert!(!coefficient_svg.contains(provenance));

    let forest_path = dir.join("forest.svg");
    let mut forest = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .interval_semantics(
            IntervalSemantics::new(
                IntervalKind::Credible,
                0.9,
                InferentialScale::Ratio,
                provenance,
            )
            .unwrap(),
        )
        .reference_line(1.0);
    forest.x("estimate").y("term").size(360, 240).forest();
    forest.render_to_svg(&forest_path).unwrap();
    let forest_svg = std::fs::read_to_string(forest_path).unwrap();
    assert!(!forest_svg.contains("semantic-note"));
    assert!(!forest_svg.contains(provenance));
}

#[test]
fn line_ribbons_and_point_intervals_retain_semantics_without_canvas_text() {
    let data = estimate_data();
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
    let dir = tempdir("line-point-interval-semantics");

    let line_path = dir.join("line.svg");
    let mut line = Chart::new(&data)
        .group("trace")
        .hue("trace")
        .interval_bounds("lower", "upper")
        .interval_semantics(semantics());
    line.x("x").y("estimate").size(240, 240).line();
    line.render_to_svg(&line_path).unwrap();

    let point_path = dir.join("point.svg");
    let mut point = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .interval_semantics(semantics());
    point
        .x("estimate")
        .y("term")
        .size(360, 240)
        .point_interval();
    point.render_to_svg(&point_path).unwrap();

    for svg in [
        std::fs::read_to_string(line_path).unwrap(),
        std::fs::read_to_string(point_path).unwrap(),
    ] {
        assert!(!svg.contains("semantic-note"));
        assert!(!svg.contains(provenance));
    }

    let mut invalid = Chart::new(&data).interval_semantics(semantics());
    invalid.x("x").y("estimate").line();
    assert!(invalid
        .render_with_backend(&mut LifecycleBackend::default())
        .unwrap_err()
        .to_string()
        .contains("require complete lower and upper bound columns"));
}

#[test]
fn coefficient_allows_absent_intervals_while_forest_requires_them_and_reference() {
    let data = estimate_data();
    let mut coefficient = Chart::new(&data);
    coefficient.x("estimate").y("term").coefficient();
    coefficient
        .render_with_backend(&mut LifecycleBackend::default())
        .unwrap();

    let mut inconsistent =
        Chart::new(&data).interval_semantics(confidence(InferentialScale::Coefficient));
    inconsistent.x("estimate").y("term").coefficient();
    assert!(inconsistent
        .render_with_backend(&mut LifecycleBackend::default())
        .unwrap_err()
        .to_string()
        .contains("require complete lower and upper bound columns"));

    let mut forest = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .interval_semantics(confidence(InferentialScale::Coefficient));
    forest.x("estimate").y("term").forest();
    assert!(forest
        .render_with_backend(&mut LifecycleBackend::default())
        .unwrap_err()
        .to_string()
        .contains("explicit scientific reference"));
}

#[test]
fn coefficient_fixed_value_labels_are_point_only_and_default_is_inert() {
    let data = estimate_data();
    let dir = tempdir("coefficient-value-labels");
    let default_path = dir.join("default.svg");
    let labeled_path = dir.join("labeled.svg");

    let mut default = Chart::new(&data);
    default.x("estimate").y("term").coefficient();
    default.render_to_svg(&default_path).unwrap();
    let default_svg = std::fs::read_to_string(default_path).unwrap();

    let mut labeled =
        Chart::new(&data).estimate_labels(NumericLabelFormat::with_suffix(2, " units").unwrap());
    labeled.x("estimate").y("term").coefficient();
    labeled.render_to_svg(&labeled_path).unwrap();
    let labeled_svg = std::fs::read_to_string(labeled_path).unwrap();

    assert!(!default_svg.contains(" units</text>"));
    assert!(labeled_svg.contains(" units</text>"));
    assert_eq!(
        default_svg.matches("<circle ").count(),
        labeled_svg.matches("<circle ").count()
    );

    let mut invalid = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .interval_semantics(confidence(InferentialScale::Coefficient))
        .estimate_labels(NumericLabelFormat::new(2).unwrap());
    invalid.x("estimate").y("term").coefficient();
    assert!(invalid
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("point-only displays without intervals"));
}

#[test]
fn coefficient_confidence_intervals_render_exact_caller_significance_marks() {
    let data = df!(
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
    let mut chart = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .interval_semantics(confidence(InferentialScale::Coefficient))
        .coefficient_significance("mark");
    chart.x("estimate").y("term").coefficient();

    let mut backend = LifecycleBackend::default();
    chart.render_with_backend(&mut backend).unwrap();
    let rendered_marks = backend
        .texts
        .iter()
        .map(|(_, _, text)| text.as_str())
        .filter(|text| matches!(*text, "*" | "**" | "***"))
        .collect::<Vec<_>>();

    assert_eq!(rendered_marks, ["*", "**", "***"]);
    assert!(chart.comparisons().is_empty());
}

#[test]
fn coefficient_significance_marks_fail_closed_during_preflight() {
    let make_data = |mark: Option<&str>| {
        df!(
            "term" => ["term"],
            "estimate" => [1.0_f64],
            "lower" => [0.5_f64],
            "upper" => [1.5_f64],
            "mark" => [mark]
        )
        .unwrap()
    };

    let invalid_data = make_data(Some("****"));
    let mut invalid = Chart::new(&invalid_data)
        .interval_bounds("lower", "upper")
        .interval_semantics(confidence(InferentialScale::Coefficient))
        .coefficient_significance("mark");
    invalid.x("estimate").y("term").coefficient();
    let mut backend = LifecycleBackend::default();
    let error = invalid.render_with_backend(&mut backend).unwrap_err();
    assert!(error.to_string().contains("expected exactly"), "{error}");
    assert_eq!((backend.begins, backend.ends), (0, 0));

    let missing_data = make_data(Some("*"));
    let mut missing = Chart::new(&missing_data).coefficient_significance("mark");
    missing.x("estimate").y("term").coefficient();
    assert!(missing
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("require complete confidence interval columns"));

    let credible_data = make_data(Some("*"));
    let mut credible = Chart::new(&credible_data)
        .interval_bounds("lower", "upper")
        .interval_semantics(
            IntervalSemantics::new(
                IntervalKind::Credible,
                0.95,
                InferentialScale::Coefficient,
                "posterior",
            )
            .unwrap(),
        )
        .coefficient_significance("mark");
    credible.x("estimate").y("term").coefficient();
    assert!(credible
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("require confidence intervals, not credible intervals"));

    let null_data = make_data(None);
    let mut null = Chart::new(&null_data)
        .interval_bounds("lower", "upper")
        .interval_semantics(confidence(InferentialScale::Coefficient))
        .coefficient_significance("mark");
    null.x("estimate").y("term").coefficient();
    assert!(null.preflight().unwrap_err().to_string().contains("null"));

    let other_data = make_data(Some("*"));
    let mut other = Chart::new(&other_data)
        .interval_bounds("lower", "upper")
        .interval_semantics(confidence(InferentialScale::Coefficient))
        .coefficient_significance("mark");
    other.x("estimate").y("term").point_interval();
    assert!(other
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("supported only by Coefficient"));
}

#[test]
fn forest_log_scale_renders_raw_power_ticks_and_rejects_nonpositive_values() {
    let data = df!(
        "term" => ["a", "b"],
        "estimate" => [0.5_f64, 2.0],
        "lower" => [0.1_f64, 1.0],
        "upper" => [1.0_f64, 10.0]
    )
    .unwrap();
    let dir = tempdir("forest-log");
    let output = dir.join("forest.svg");
    let mut chart = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .interval_semantics(confidence(InferentialScale::Ratio))
        .reference_line(1.0)
        .axis_scale(AxisScale::Log10);
    chart.x("estimate").y("term").forest();
    chart.render_to_svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.contains(">0.1</text>") && svg.contains(">1</text>") && svg.contains(">10</text>"));

    let invalid_data = df!(
        "term" => ["a"],
        "estimate" => [0.0_f64],
        "lower" => [0.0_f64],
        "upper" => [1.0_f64]
    )
    .unwrap();
    let mut invalid = Chart::new(&invalid_data)
        .interval_bounds("lower", "upper")
        .interval_semantics(confidence(InferentialScale::Ratio))
        .reference_line(1.0)
        .axis_scale(AxisScale::Log10);
    invalid.x("estimate").y("term").forest();
    assert!(invalid
        .render_with_backend(&mut LifecycleBackend::default())
        .unwrap_err()
        .to_string()
        .contains("strictly positive"));
}

#[test]
fn forest_narrow_log_range_keeps_raw_multiplicative_ticks() {
    let data = df!(
        "term" => ["a", "b"],
        "estimate" => [1.1_f64, 1.4],
        "lower" => [1.05_f64, 1.3],
        "upper" => [1.2_f64, 1.5]
    )
    .unwrap();
    let dir = tempdir("forest-narrow-log");
    let output = dir.join("forest.svg");
    let mut chart = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .interval_semantics(confidence(InferentialScale::Ratio))
        .reference_line(1.0)
        .axis_scale(AxisScale::Log10)
        .x_limits(1.0, 1.6);
    chart.x("estimate").y("term").forest();

    chart.render_to_svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();

    assert!(
        svg.contains(">1</text>") && svg.contains(">1.3</text>") && svg.contains(">1.6</text>")
    );
}
