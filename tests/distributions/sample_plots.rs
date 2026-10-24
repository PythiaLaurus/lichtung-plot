use super::*;

#[test]
fn histogram_rejects_null_nonfinite_and_empty_values() {
    let cases = [
        NumericValues::nullable("value", [Some(1.0), None]),
        NumericValues::new("value", [1.0, f64::INFINITY]),
        NumericValues::new("value", []),
    ];
    for (index, values) in cases.into_iter().enumerate() {
        let dir = unique_dir(&format!("histogram-invalid-{index}"));
        let output = dir.join("histogram.svg");
        let chart = Plot::histogram(HistogramInput::samples(values));
        assert!(chart.svg(&output).is_err(), "invalid case {index}");
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn density_clips_large_finite_support_before_projection() {
    let base = 1e307_f64;
    let values = [base, base + 1e293, base + 2e293, base + 3e293];
    let dir = unique_dir("density-large-support-explicit-limits");
    let output = dir.join("density.svg");
    let chart = Plot::density(NumericValues::new("value", values))
        .x_limits(values[1], values[2])
        .y_limits(0.0, 2.4e-294);
    chart.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    let panel = parse_first_clip_rect(&svg);
    let marks = svg
        .split("<g clip-path=\"url(#clip-0)\">")
        .nth(1)
        .unwrap()
        .split("</g>")
        .next()
        .unwrap();

    assert!(!svg.contains("NaN") && !svg.contains("inf"));
    assert!(parse_graphic_bounds(marks).into_iter().all(|bounds| {
        bounds.x >= panel.x - 1.0
            && bounds.y >= panel.y - 1.0
            && bounds.x + bounds.width <= panel.x + panel.width + 1.0
            && bounds.y + bounds.height <= panel.y + panel.height + 1.0
    }));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn density_rejects_null_and_nonfinite_values() {
    for (index, values) in [
        NumericValues::nullable("value", [Some(1.0), None, Some(2.0)]),
        NumericValues::new("value", [1.0, f64::INFINITY, 2.0]),
    ]
    .into_iter()
    .enumerate()
    {
        let dir = unique_dir(&format!("density-invalid-{index}"));
        let output = dir.join("density.svg");
        assert!(Plot::density(values).svg(&output).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn qq_rejects_null_and_nonfinite_values() {
    for (index, values) in [
        NumericValues::nullable("value", [Some(1.0), None]),
        NumericValues::new("value", [1.0, f64::NAN]),
    ]
    .into_iter()
    .enumerate()
    {
        let dir = unique_dir(&format!("qq-invalid-{index}"));
        let output = dir.join("qq.svg");
        assert!(Plot::qq(values).svg(&output).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn qq_renders_representable_constants_and_rejects_uncontainable_auto_ink() {
    for (name, value) in [
        ("zero", 0.0),
        ("subnormal", f64::from_bits(1)),
        ("maximum", f64::MAX),
        ("minimum", -f64::MAX),
    ] {
        let dir = unique_dir(&format!("qq-constant-{name}"));
        let output = dir.join("qq.svg");
        let result =
            Plot::qq(NumericValues::new("value", [value, value, value])).svg_preserve_text(&output);
        if matches!(name, "maximum" | "minimum") {
            let error = result
                .err()
                .expect("uncontainable automatic mark ink")
                .to_string();
            assert!(
                error.contains("automatic mark ink cannot extend this representable data domain"),
                "{name}: {error}"
            );
            assert!(
                !output.exists(),
                "unresolved automatic mark ink must not publish output"
            );
        } else {
            result.unwrap();
            let svg = std::fs::read_to_string(&output).unwrap();
            assert_eq!(svg.matches("<circle ").count(), 3, "{name}");
            assert!(!svg.contains("NaN") && !svg.contains("inf"), "{name}");
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn qq_explicit_limits_skip_opposite_sign_extreme_outliers_before_projection() {
    let dir = unique_dir("qq-opposite-extremes-explicit");
    let output = dir.join("qq.svg");
    let plot = Plot::qq(NumericValues::new("value", [-f64::MAX, 0.0, f64::MAX]))
        .x_limits(-2.0, 2.0)
        .y_limits(-1.0, 1.0);
    let plot = plot.reference_line(QqLine::Hidden);
    plot.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();

    assert_eq!(svg.matches("<circle ").count(), 1);
    assert!(!svg.contains("NaN") && !svg.contains("=\"inf"));
    std::fs::remove_dir_all(dir).unwrap();
}
