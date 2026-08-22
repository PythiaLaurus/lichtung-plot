use lichtung_plot::{
    AxisTick, GuideSpec, GuideStyle, InferentialScale, IntervalKind, IntervalSemantics, LineMode,
    LinePathStyle, Plot, PointLayer,
};

fn confidence() -> IntervalSemantics {
    IntervalSemantics::new(
        IntervalKind::Confidence,
        0.95,
        InferentialScale::Response,
        "model fit",
    )
    .unwrap()
}

#[test]
fn numeric_line_owns_interval_guides_ticks_and_labeled_point_layers() {
    let point_layer = PointLayer::new(
        lichtung_plot::NumericValues::new("peak x", [2.0]),
        lichtung_plot::NumericValues::new("peak y", [3.0]),
        (180, 45, 40),
        4,
        0.9,
    )
    .unwrap()
    .labels(lichtung_plot::CategoryValues::new("label", ["peak"]).unwrap())
    .legend_label("Selected peak");
    let directory = std::env::temp_dir().join(format!(
        "lichtung-lines-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("line.svg");

    Plot::line(
        lichtung_plot::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        lichtung_plot::NumericValues::new("y", [2.0, 3.0, 2.5, 4.0]),
    )
    .interval_bounds(
        lichtung_plot::NumericValues::new("lower", [1.5, 2.5, 2.0, 3.5]),
        lichtung_plot::NumericValues::new("upper", [2.5, 3.5, 3.0, 4.5]),
    )
    .interval_semantics(confidence())
    .x_ticks(vec![AxisTick::new(1.0, "start"), AxisTick::new(4.0, "end")])
    .guide(GuideSpec::horizontal(
        2.5,
        GuideStyle::new((90, 90, 90), 1.0, lichtung_plot::LinePattern::Dashed).unwrap(),
    ))
    .point_layer(point_layer)
    .svg(&output)
    .unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    for text in ["start", "end", "peak", "Selected peak"] {
        assert!(svg.contains(text), "missing {text:?}");
    }
    assert!(svg.contains("<polygon"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn trajectory_line_accepts_categorical_conditions_and_explicit_groups() {
    let plot = Plot::line(
        lichtung_plot::CategoryValues::new("condition", ["before", "after", "before", "after"])
            .unwrap(),
        lichtung_plot::NumericValues::new("value", [1.0, 2.0, 1.5, 3.0]),
    )
    .group(lichtung_plot::CategoryValues::new("subject", ["a", "a", "b", "b"]).unwrap())
    .hue(lichtung_plot::CategoryValues::new("arm", ["x", "x", "y", "y"]).unwrap())
    .line_mode(LineMode::Trajectory);

    plot.preflight().unwrap();
}

#[test]
fn path_line_preserves_source_order_and_requires_explicit_grouping() {
    let valid = Plot::line(
        lichtung_plot::NumericValues::new("x", [2.0, 1.0, 2.0, 1.0]),
        lichtung_plot::NumericValues::new("y", [3.0, 2.0, 4.0, 2.5]),
    )
    .group(lichtung_plot::CategoryValues::new("path", ["a", "a", "b", "b"]).unwrap())
    .hue(lichtung_plot::CategoryValues::new("hue", ["a", "a", "b", "b"]).unwrap())
    .line_mode(LineMode::Path)
    .line_path_style(LinePathStyle::Hue);
    valid.preflight().unwrap();

    let invalid = Plot::line(
        lichtung_plot::NumericValues::new("x", [1.0, 2.0]),
        lichtung_plot::NumericValues::new("y", [2.0, 3.0]),
    )
    .line_mode(LineMode::Path);
    assert!(invalid.preflight().is_err());
}

#[test]
fn point_layers_reject_non_finite_values_during_preflight() {
    let layer = PointLayer::new(
        lichtung_plot::NumericValues::new("x", [f64::NAN]),
        lichtung_plot::NumericValues::new("y", [1.0]),
        (10, 20, 30),
        3,
        0.8,
    )
    .unwrap();
    let error = Plot::line(
        lichtung_plot::NumericValues::new("x", [1.0, 2.0]),
        lichtung_plot::NumericValues::new("y", [2.0, 3.0]),
    )
    .point_layer(layer)
    .preflight()
    .unwrap_err();
    assert!(error.to_string().contains("non-finite"));
}
