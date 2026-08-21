use lichtung_plot::{
    CategoricalColors, Chart, EstimateSource, Figure, FigureLegendPolicy, InferentialScale,
    IntervalKind, IntervalSemantics, Labels, LinePattern, PointShape,
};
use polars::prelude::*;

fn interaction_data() -> DataFrame {
    df!(
        "condition" => ["Before", "After", "Before", "After"],
        "series" => ["Control", "Control", "Semantic Agent", "Semantic Agent"],
        "estimate" => [1.0_f64, 1.5, 1.2, 2.0],
        "lower" => [0.8_f64, 1.3, 1.0, 1.8],
        "upper" => [1.2_f64, 1.7, 1.4, 2.2],
        "point_shape" => [
            PointShape::Circle.as_str(),
            PointShape::Circle.as_str(),
            PointShape::TriangleUp.as_str(),
            PointShape::TriangleUp.as_str(),
        ],
        "line_pattern" => [
            LinePattern::Dashed.as_str(),
            LinePattern::Dashed.as_str(),
            LinePattern::Solid.as_str(),
            LinePattern::Solid.as_str(),
        ],
    )
    .unwrap()
}

fn chart(data: &DataFrame, styled: bool) -> Chart<'_> {
    let mut chart = Chart::new(data)
        .labels(Labels::raw())
        .group("series")
        .hue("series")
        .interval_bounds("lower", "upper")
        .interval_semantics(
            IntervalSemantics::new(
                IntervalKind::Confidence,
                0.95,
                InferentialScale::Response,
                "interaction style fixture",
            )
            .unwrap(),
        )
        .interaction_source(EstimateSource::EstimatedMarginalMean);
    if styled {
        chart = chart.interaction_styles("point_shape", "line_pattern");
    }
    chart.x("condition").y("estimate").interaction();
    chart
}

fn svg(chart: &Chart<'_>) -> String {
    String::from_utf8(chart.render_to_svg_bytes().unwrap()).unwrap()
}

fn triangle_vertices(line: &str) -> Vec<(f64, f64)> {
    let points = line
        .split_once("points=\"")
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0;
    points
        .split_whitespace()
        .map(|point| {
            let (x, y) = point.split_once(',').unwrap();
            (x.parse().unwrap(), y.parse().unwrap())
        })
        .collect()
}

#[test]
fn interaction_styles_render_dashed_circle_and_solid_triangle_geometry() {
    let output = svg(&chart(&interaction_data(), true));
    let triangles = output
        .lines()
        .filter(|line| {
            line.starts_with("<polygon points=") && line.contains("fill-opacity=\"0.82\"")
        })
        .map(triangle_vertices)
        .collect::<Vec<_>>();

    assert_eq!(output.matches("stroke-dasharray=\"5 3\"").count(), 2);
    assert_eq!(triangles.len(), 3);
    assert!(triangles.iter().all(|vertices| {
        vertices.len() == 3
            && vertices[0].1 < vertices[1].1
            && (vertices[1].1 - vertices[2].1).abs() < 0.001
            && (vertices[0].0 - f64::midpoint(vertices[1].0, vertices[2].0)).abs() < 0.001
    }));
}

#[test]
fn interaction_style_legend_matches_each_trace_glyph() {
    let output = svg(&chart(&interaction_data(), true));
    let legend = output
        .split_once("<g id=\"legend\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;
    let control_dash = legend.find("stroke-dasharray=\"5 3\"").unwrap();
    let control_circle = legend.find("<circle ").unwrap();
    let control_label = legend.find(">Control</text>").unwrap();
    let semantic_line = legend[control_label..].find("<polyline ").unwrap() + control_label;
    let semantic_triangle = legend.find("<polygon ").unwrap();
    let semantic_label = legend.find(">Semantic Agent</text>").unwrap();

    assert!(
        control_dash < control_circle
            && control_circle < control_label
            && control_label < semantic_line
            && semantic_line < semantic_triangle
            && semantic_triangle < semantic_label
            && !legend[semantic_line..semantic_triangle].contains("stroke-dasharray")
    );
}

#[test]
fn interaction_styles_reject_missing_style_columns() {
    let data = interaction_data();
    for (shape, pattern, missing) in [
        ("missing_shape", "line_pattern", "missing_shape"),
        ("point_shape", "missing_pattern", "missing_pattern"),
    ] {
        let mut invalid = chart(&data, false).interaction_styles(shape, pattern);
        invalid.interaction();
        let error = invalid.preflight().unwrap_err().to_string();
        assert!(error.contains(missing), "{error}");
    }
}

#[test]
fn interaction_styles_reject_non_exact_values() {
    let data = df!(
        "condition" => ["Before", "After"],
        "series" => ["Control", "Control"],
        "estimate" => [1.0_f64, 1.5],
        "lower" => [0.8_f64, 1.3],
        "upper" => [1.2_f64, 1.7],
        "point_shape" => ["Circle", "Circle"],
        "line_pattern" => ["dashed", "dashed"],
    )
    .unwrap();

    let error = chart(&data, true).preflight().unwrap_err().to_string();

    assert!(error.contains("expected exactly \"circle\" or \"triangle-up\""));
}

#[test]
fn interaction_styles_reject_non_exact_line_patterns() {
    let data = df!(
        "condition" => ["Before", "After"],
        "series" => ["Control", "Control"],
        "estimate" => [1.0_f64, 1.5],
        "lower" => [0.8_f64, 1.3],
        "upper" => [1.2_f64, 1.7],
        "point_shape" => ["circle", "circle"],
        "line_pattern" => ["Dashed", "Dashed"],
    )
    .unwrap();

    let error = chart(&data, true).preflight().unwrap_err().to_string();

    assert!(error.contains("expected exactly \"solid\" or \"dashed\""));
}

#[test]
fn interaction_styles_reject_inconsistent_values_within_a_trace() {
    let mut data = interaction_data();
    data.replace(
        "line_pattern",
        Series::new("line_pattern".into(), ["dashed", "solid", "solid", "solid"]).into(),
    )
    .unwrap();

    let error = chart(&data, true).preflight().unwrap_err().to_string();

    assert!(error.contains("each trace requires one exact style pair"));
}

#[test]
fn styled_interaction_output_is_deterministic() {
    let data = interaction_data();

    assert_eq!(svg(&chart(&data, true)), svg(&chart(&data, true)));
}

#[test]
fn unused_interaction_style_columns_preserve_circle_solid_defaults() {
    let data = interaction_data();
    let first = svg(&chart(&data, false));
    let second = svg(&chart(&data, false));

    assert_eq!(first, second);
    assert_eq!(first.matches("<circle ").count(), 6);
    assert!(!first.contains("<polygon points=") && !first.contains("stroke-dasharray=\"5 3\""));
}

#[test]
fn categorical_colors_apply_exact_trace_rgb_to_marks_and_legend() {
    let data = interaction_data();
    let colors = CategoricalColors::new([
        ("Control".to_owned(), (51, 51, 51)),
        ("Semantic Agent".to_owned(), (0, 114, 178)),
    ])
    .unwrap();
    let output = svg(&chart(&data, true).categorical_colors(colors));

    assert!(output.matches("#333333").count() >= 4);
    assert!(output.matches("#0072B2").count() >= 4);
}

#[test]
fn shared_figure_legend_preserves_caller_colors_and_composite_trace_glyphs() {
    let control = df!(
        "condition" => ["Before", "After"],
        "series" => ["Control", "Control"],
        "estimate" => [1.0_f64, 1.5],
        "lower" => [0.8_f64, 1.3],
        "upper" => [1.2_f64, 1.7],
        "point_shape" => [PointShape::Circle.as_str(); 2],
        "line_pattern" => [LinePattern::Dashed.as_str(); 2],
    )
    .unwrap();
    let semantic = df!(
        "condition" => ["Before", "After"],
        "series" => ["Semantic Agent", "Semantic Agent"],
        "estimate" => [1.2_f64, 2.0],
        "lower" => [1.0_f64, 1.8],
        "upper" => [1.4_f64, 2.2],
        "point_shape" => [PointShape::TriangleUp.as_str(); 2],
        "line_pattern" => [LinePattern::Solid.as_str(); 2],
    )
    .unwrap();
    let colors = CategoricalColors::new([
        ("Control".to_owned(), (51, 51, 51)),
        ("Semantic Agent".to_owned(), (0, 114, 178)),
    ])
    .unwrap();
    let output = String::from_utf8(
        Figure::grid(1, 2)
            .unwrap()
            .panel(
                0,
                0,
                chart(&control, true).categorical_colors(colors.clone()),
            )
            .unwrap()
            .panel(0, 1, chart(&semantic, true).categorical_colors(colors))
            .unwrap()
            .legend_policy(FigureLegendPolicy::Shared)
            .render_to_svg_bytes()
            .unwrap(),
    )
    .unwrap();
    let legend = output
        .split_once("<g id=\"figure-shared-legend\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;
    let control_dash = legend.find("stroke=\"#333333\"").unwrap();
    let control_circle = legend.find("<circle ").unwrap();
    let control_label = legend.find(">Control</text>").unwrap();
    let semantic_line = legend[control_label..].find("stroke=\"#0072B2\"").unwrap() + control_label;
    let semantic_triangle = legend.find("<polygon ").unwrap();
    let semantic_label = legend.find(">Semantic Agent</text>").unwrap();

    assert!(legend[control_dash..control_circle].contains("stroke-dasharray=\"5 3\""));
    assert!(
        control_dash < control_circle
            && control_circle < control_label
            && control_label < semantic_line
            && semantic_line < semantic_triangle
            && semantic_triangle < semantic_label
            && !legend[semantic_line..semantic_triangle].contains("stroke-dasharray")
    );
}

#[test]
fn shared_figure_legend_rejects_conflicting_caller_color_mapping() {
    let data = df!(
        "condition" => ["Before", "After"],
        "series" => ["Control", "Control"],
        "estimate" => [1.0_f64, 1.5],
        "lower" => [0.8_f64, 1.3],
        "upper" => [1.2_f64, 1.7],
        "point_shape" => [PointShape::Circle.as_str(); 2],
        "line_pattern" => [LinePattern::Dashed.as_str(); 2],
    )
    .unwrap();
    let gray = CategoricalColors::new([("Control".to_owned(), (51, 51, 51))]).unwrap();
    let blue = CategoricalColors::new([("Control".to_owned(), (0, 114, 178))]).unwrap();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, chart(&data, true).categorical_colors(gray))
        .unwrap()
        .panel(0, 1, chart(&data, true).categorical_colors(blue))
        .unwrap()
        .legend_policy(FigureLegendPolicy::Shared);

    let error = figure.preflight().unwrap_err().to_string();

    assert!(error.contains("compatible semantic titles, labels, and styles"));
}

#[test]
fn shared_figure_legend_rejects_conflicting_point_outline_mapping() {
    let data = df!(
        "x" => [1.0_f64, 2.0],
        "y" => [2.0_f64, 3.0],
        "series" => ["Control", "Control"],
    )
    .unwrap();
    let mut outlined = Chart::new(&data).hue("series");
    outlined.x("x").y("y").scatter();
    let mut plain = Chart::new(&data).hue("series").hide_point_outline();
    plain.x("x").y("y").scatter();
    let figure = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, outlined)
        .unwrap()
        .panel(0, 1, plain)
        .unwrap()
        .legend_policy(FigureLegendPolicy::Shared);

    let error = figure.preflight().unwrap_err().to_string();

    assert!(error.contains("compatible semantic titles, labels, and styles"));
}

#[test]
fn categorical_colors_reject_missing_trace_mapping() {
    let data = interaction_data();
    let colors = CategoricalColors::new([("Control".to_owned(), (51, 51, 51))]).unwrap();
    let error = chart(&data, true)
        .categorical_colors(colors)
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains("Semantic Agent") && error.contains("no caller-owned color"));
}

#[test]
fn categorical_colors_reject_duplicate_or_malformed_labels() {
    assert!(CategoricalColors::new([
        ("Control".to_owned(), (1, 2, 3)),
        ("Control".to_owned(), (4, 5, 6)),
    ])
    .is_err());
    assert!(CategoricalColors::new([(" Control".to_owned(), (1, 2, 3))]).is_err());
}
