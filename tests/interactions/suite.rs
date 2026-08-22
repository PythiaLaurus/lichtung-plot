use lichtung_plot::{
    EstimateSource, InferentialScale, IntervalKind, IntervalSemantics, LinePattern, Plot,
    PointShape,
};

fn semantics() -> IntervalSemantics {
    IntervalSemantics::new(
        IntervalKind::Confidence,
        0.95,
        InferentialScale::Response,
        "model fit",
    )
    .unwrap()
}

fn interaction() -> Plot {
    Plot::interaction(
        lichtung_plot::CategoryValues::new("condition", ["before", "after", "before", "after"])
            .unwrap(),
        lichtung_plot::NumericValues::new("estimate", [2.0, 3.0, 2.5, 4.0]),
        lichtung_plot::CategoryValues::new("trace", ["control", "control", "active", "active"])
            .unwrap(),
        lichtung_plot::NumericValues::new("lower", [1.5, 2.5, 2.0, 3.5]),
        lichtung_plot::NumericValues::new("upper", [2.5, 3.5, 3.0, 4.5]),
    )
    .hue(
        lichtung_plot::CategoryValues::new("trace", ["control", "control", "active", "active"])
            .unwrap(),
    )
    .interval_semantics(semantics())
    .interaction_source(EstimateSource::EstimatedMarginalMean)
}

#[test]
fn interaction_styles_are_owned_and_rendered_per_trace() {
    let shapes = lichtung_plot::CategoryValues::new(
        "shape",
        [
            PointShape::Circle.as_str(),
            PointShape::Circle.as_str(),
            PointShape::TriangleUp.as_str(),
            PointShape::TriangleUp.as_str(),
        ],
    )
    .unwrap();
    let patterns = lichtung_plot::CategoryValues::new(
        "pattern",
        [
            LinePattern::Solid.as_str(),
            LinePattern::Solid.as_str(),
            LinePattern::Dashed.as_str(),
            LinePattern::Dashed.as_str(),
        ],
    )
    .unwrap();
    let directory = std::env::temp_dir().join(format!(
        "lichtung-interaction-style-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("interaction.svg");

    interaction()
        .interaction_styles(shapes, patterns)
        .svg(&output)
        .unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.contains("stroke-dasharray"));
    assert!(svg.contains("<polygon"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn interaction_rejects_nonconstant_style_within_trace() {
    let shapes =
        lichtung_plot::CategoryValues::new("shape", ["circle", "triangle-up", "circle", "circle"])
            .unwrap();
    let patterns =
        lichtung_plot::CategoryValues::new("pattern", ["solid", "solid", "dashed", "dashed"])
            .unwrap();

    let error = interaction()
        .interaction_styles(shapes, patterns)
        .preflight()
        .unwrap_err();
    assert!(error.to_string().contains("inconsistent point-shape"));
}

#[test]
fn interaction_rejects_incomplete_alignment_before_output() {
    let error = Plot::interaction(
        lichtung_plot::CategoryValues::new("condition", ["before", "after"]).unwrap(),
        lichtung_plot::NumericValues::new("estimate", [2.0]),
        lichtung_plot::CategoryValues::new("trace", ["control", "control"]).unwrap(),
        lichtung_plot::NumericValues::new("lower", [1.5, 2.5]),
        lichtung_plot::NumericValues::new("upper", [2.5, 3.5]),
    )
    .interval_semantics(semantics())
    .interaction_source(EstimateSource::EstimatedMarginalMean)
    .preflight()
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("plot value \"condition\" has length 2, expected 1"));
}
