use super::*;

struct StyledInteractionData {
    condition: CategoryValues,
    group: CategoryValues,
    hue: CategoryValues,
    estimate: NumericValues,
    lower: NumericValues,
    upper: NumericValues,
    panel: Option<CategoryValues>,
    styles: Vec<InteractionStyle>,
}

fn interaction_data() -> StyledInteractionData {
    let series = CategoryValues::new(
        "series",
        ["Control", "Control", "Semantic Agent", "Semantic Agent"],
    )
    .unwrap();
    StyledInteractionData {
        condition: CategoryValues::new("condition", ["Before", "After", "Before", "After"])
            .unwrap(),
        group: series.clone(),
        hue: series,
        estimate: NumericValues::new("estimate", [1.0, 1.5, 1.2, 2.0]),
        lower: NumericValues::new("lower", [0.8, 1.3, 1.0, 1.8]),
        upper: NumericValues::new("upper", [1.2, 1.7, 1.4, 2.2]),
        panel: None,
        styles: vec![
            InteractionStyle::new("Control").pattern(LinePattern::Dashed),
            InteractionStyle::new("Semantic Agent").shape(PointShape::TriangleUp),
        ],
    }
}

fn single_series_data(
    series: CategoryValues,
    estimate: NumericValues,
    lower: NumericValues,
    upper: NumericValues,
    style: InteractionStyle,
) -> StyledInteractionData {
    StyledInteractionData {
        condition: CategoryValues::new("condition", ["Before", "After"]).unwrap(),
        group: series.clone(),
        hue: series,
        estimate,
        lower,
        upper,
        panel: None,
        styles: vec![style],
    }
}

fn chart(data: &StyledInteractionData, styled: bool) -> Plot<lichtung_plot::kind::Interaction> {
    let plot = Plot::interaction(
        data.condition.clone(),
        data.estimate.clone(),
        data.group.clone(),
        Bounds::new(data.lower.clone(), data.upper.clone()),
    )
    .labels(Labels::raw())
    .hue(data.hue.clone());
    if styled {
        plot.styles(data.styles.clone())
    } else {
        plot
    }
}

fn svg<K: lichtung_plot::kind::Marker>(plot: Plot<K>) -> String {
    render_svg(plot, "interaction-style").unwrap()
}

#[test]
fn extracted_interaction_preserves_original_svg_bytes() {
    // Full-output fingerprints captured from the original combined renderer at ae5591f.
    for (theme, expected) in [
        (
            lichtung_plot::Theme::Publication,
            [
                (11174, 0x27e8_e930_2ede_82fd_u64),
                (11518, 0x6bd5_2e5c_dd5b_0609),
            ],
        ),
        (
            lichtung_plot::Theme::Science,
            [
                (11021, 0x3ec0_90f9_5970_8127),
                (11365, 0xd835_28b0_4c40_759f),
            ],
        ),
    ] {
        for (styled, expected) in [false, true].into_iter().zip(expected) {
            let output = svg(chart(&interaction_data(), styled)
                .theme(theme)
                .size(800, 600));
            let hash = output
                .bytes()
                .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
                    (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
                });
            assert_eq!((output.len(), hash), expected, "{theme:?}, styled={styled}");
        }
    }
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
    let output = svg(chart(&interaction_data(), true));
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
    let output = svg(chart(&interaction_data(), true));
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
fn interaction_styles_reject_unknown_groups() {
    let data = interaction_data();
    let error = chart(&data, false)
        .styles([InteractionStyle::new("Unknown")])
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains("unknown graphical group \"Unknown\""));
}

#[test]
fn interaction_styles_require_one_style_per_group() {
    let data = interaction_data();
    let error = chart(&data, false)
        .styles([InteractionStyle::new("Control")])
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains("graphical group \"Semantic Agent\" has no configured style"));
}

#[test]
fn interaction_styles_reject_duplicate_groups() {
    let data = interaction_data();
    let error = chart(&data, false)
        .styles([
            InteractionStyle::new("Control"),
            InteractionStyle::new("Control").pattern(LinePattern::Dashed),
        ])
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains("graphical group \"Control\" has more than one configured style"));
}

#[test]
fn styled_interaction_output_is_deterministic() {
    let data = interaction_data();

    assert_eq!(svg(chart(&data, true)), svg(chart(&data, true)));
}

#[test]
fn interaction_styles_apply_by_group_across_facets() {
    let mut data = interaction_data();
    data.panel = Some(CategoryValues::new("panel", ["Left", "Left", "Right", "Right"]).unwrap());

    let output = svg(chart(&data, true)
        .facet(FacetSpec::column(data.panel.clone().unwrap()))
        .size(960, 320));

    assert!(output.contains(">panel = Left</text>") && output.contains(">panel = Right</text>"));
    assert!(output.contains("stroke-dasharray=\"5 3\"") && output.contains("<polygon points="));
}

#[test]
fn faceted_interactions_keep_sparse_global_pair_styles_and_colors() {
    let panel = CategoryValues::new(
        "panel",
        [
            "Left", "Left", "Left", "Left", "Right", "Right", "Right", "Right",
        ],
    )
    .unwrap();
    let condition = CategoryValues::new(
        "condition",
        [
            "Before", "After", "Before", "After", "Before", "After", "Before", "After",
        ],
    )
    .unwrap();
    let group =
        CategoryValues::new("group", ["g1", "g1", "g1", "g1", "g1", "g1", "g2", "g2"]).unwrap();
    let hue = CategoryValues::new("hue", ["h2", "h2", "h1", "h1", "h1", "h1", "h2", "h2"]).unwrap();
    let estimate = NumericValues::new("estimate", [1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5]);
    let lower = NumericValues::new("lower", [0.8, 1.3, 1.8, 2.3, 2.8, 3.3, 3.8, 4.3]);
    let upper = NumericValues::new("upper", [1.2, 1.7, 2.2, 2.7, 3.2, 3.7, 4.2, 4.7]);
    let colors = CategoricalColors::new([
        ("h1".to_owned(), (51, 51, 51)),
        ("h2".to_owned(), (0, 114, 178)),
    ])
    .unwrap();
    let output = svg(
        Plot::interaction(condition, estimate, group, Bounds::new(lower, upper))
            .hue(hue)
            .styles([
                InteractionStyle::new("g1").pattern(LinePattern::Dashed),
                InteractionStyle::new("g2").shape(PointShape::TriangleUp),
            ])
            .categorical_colors(colors)
            .facet(FacetSpec::column(panel))
            .size(960, 320),
    );
    let mut clipped = false;
    let connectors = output
        .lines()
        .filter(|line| {
            if line.starts_with("<g clip-path=") {
                clipped = true;
                return false;
            }
            if clipped && *line == "</g>" {
                clipped = false;
                return false;
            }
            clipped
                && line.starts_with("<polyline ")
                && line.contains("stroke-width=\"1.7998046875\"")
        })
        .collect::<Vec<_>>();

    assert_eq!(connectors.len(), 4);
    assert_eq!(
        connectors
            .iter()
            .filter(|line| line.contains("stroke-dasharray=\"5 3\""))
            .count(),
        3
    );
    assert_eq!(
        connectors
            .iter()
            .filter(|line| line.contains("stroke=\"#333333\""))
            .count(),
        2
    );
    assert_eq!(
        connectors
            .iter()
            .filter(|line| line.contains("stroke=\"#0072B2\""))
            .count(),
        2
    );
}

#[test]
fn unconfigured_interaction_styles_preserve_circle_solid_defaults() {
    let data = interaction_data();
    let first = svg(chart(&data, false));
    let second = svg(chart(&data, false));

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
    let output = svg(chart(&data, true).categorical_colors(colors));

    assert!(output.matches("#333333").count() >= 4);
    assert!(output.matches("#0072B2").count() >= 4);
}

#[test]
fn crossed_group_hue_interactions_render_only_observed_pair_traces() {
    let condition = CategoryValues::new(
        "condition",
        ["Before", "After", "Before", "After", "Before", "After"],
    )
    .unwrap();
    let group = CategoryValues::new("group", ["g1", "g1", "g1", "g1", "g2", "g2"]).unwrap();
    let hue = CategoryValues::new("hue", ["h1", "h1", "h2", "h2", "h2", "h2"]).unwrap();
    let estimate = NumericValues::new("estimate", [1.0, 1.5, 2.0, 2.5, 3.0, 3.5]);
    let lower = NumericValues::new("lower", [0.8, 1.3, 1.8, 2.3, 2.8, 3.3]);
    let upper = NumericValues::new("upper", [1.2, 1.7, 2.2, 2.7, 3.2, 3.7]);
    let colors = CategoricalColors::new([
        ("h1".to_owned(), (51, 51, 51)),
        ("h2".to_owned(), (0, 114, 178)),
    ])
    .unwrap();
    let output = svg(
        Plot::interaction(condition, estimate, group, Bounds::new(lower, upper))
            .hue(hue)
            .styles([
                InteractionStyle::new("g1").pattern(LinePattern::Dashed),
                InteractionStyle::new("g2").shape(PointShape::TriangleUp),
            ])
            .categorical_colors(colors),
    );
    let marks = output
        .split_once("<g clip-path=")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;
    let connectors = marks
        .lines()
        .filter(|line| {
            line.starts_with("<polyline ") && line.contains("stroke-width=\"1.7998046875\"")
        })
        .collect::<Vec<_>>();

    assert_eq!(connectors.len(), 3);
    assert_eq!(
        connectors
            .iter()
            .filter(|line| line.contains("stroke-dasharray=\"5 3\""))
            .count(),
        2
    );
    assert_eq!(
        connectors
            .iter()
            .filter(|line| line.contains("stroke=\"#333333\""))
            .count(),
        1
    );
    assert_eq!(
        connectors
            .iter()
            .filter(|line| line.contains("stroke=\"#0072B2\""))
            .count(),
        2
    );
}

#[test]
fn interaction_duplicate_identity_includes_condition_group_and_hue() {
    let condition = CategoryValues::new("condition", ["Before", "Before", "Before"]).unwrap();
    let group = CategoryValues::new("group", ["g1", "g1", "g1"]).unwrap();
    let hue = CategoryValues::new("hue", ["h1", "h2", "h1"]).unwrap();
    let estimate = NumericValues::new("estimate", [1.0, 2.0, 3.0]);
    let lower = NumericValues::new("lower", [0.8, 1.8, 2.8]);
    let upper = NumericValues::new("upper", [1.2, 2.2, 3.2]);
    let error = Plot::interaction(condition, estimate, group, Bounds::new(lower, upper))
        .hue(hue)
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains("duplicate (condition, group, hue)") && error.contains("source row 2"));
}

#[test]
fn shared_canvas_legend_preserves_caller_colors_and_composite_trace_glyphs() {
    let control = single_series_data(
        CategoryValues::new("series", ["Control", "Control"]).unwrap(),
        NumericValues::new("estimate", [1.0, 1.5]),
        NumericValues::new("lower", [0.8, 1.3]),
        NumericValues::new("upper", [1.2, 1.7]),
        InteractionStyle::new("Control").pattern(LinePattern::Dashed),
    );
    let semantic = single_series_data(
        CategoryValues::new("series", ["Semantic Agent", "Semantic Agent"]).unwrap(),
        NumericValues::new("estimate", [1.2, 2.0]),
        NumericValues::new("lower", [1.0, 1.8]),
        NumericValues::new("upper", [1.4, 2.2]),
        InteractionStyle::new("Semantic Agent").shape(PointShape::TriangleUp),
    );
    let colors = CategoricalColors::new([
        ("Control".to_owned(), (51, 51, 51)),
        ("Semantic Agent".to_owned(), (0, 114, 178)),
    ])
    .unwrap();
    let output = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(
                0,
                0,
                chart(&control, true).categorical_colors(colors.clone()),
            )
            .unwrap()
            .panel(0, 1, chart(&semantic, true).categorical_colors(colors))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared),
        "shared-interaction-legend",
    )
    .unwrap();
    let legend = output
        .split_once("<g id=\"canvas-shared-legend\">")
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
fn shared_canvas_legend_rejects_conflicting_caller_color_mapping() {
    let data = single_series_data(
        CategoryValues::new("series", ["Control", "Control"]).unwrap(),
        NumericValues::new("estimate", [1.0, 1.5]),
        NumericValues::new("lower", [0.8, 1.3]),
        NumericValues::new("upper", [1.2, 1.7]),
        InteractionStyle::new("Control").pattern(LinePattern::Dashed),
    );
    let gray = CategoricalColors::new([("Control".to_owned(), (51, 51, 51))]).unwrap();
    let blue = CategoricalColors::new([("Control".to_owned(), (0, 114, 178))]).unwrap();
    let canvas = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, chart(&data, true).categorical_colors(gray))
        .unwrap()
        .panel(0, 1, chart(&data, true).categorical_colors(blue))
        .unwrap()
        .legend_policy(CanvasLegendPolicy::Shared);

    let error = canvas.preflight().unwrap_err().to_string();

    assert!(error.contains("compatible semantic titles, labels, and styles"));
}

#[test]
fn shared_canvas_legend_rejects_conflicting_point_outline_mapping() {
    let x = NumericValues::new("x", [1.0, 2.0]);
    let y = NumericValues::new("y", [2.0, 3.0]);
    let series = CategoryValues::new("series", ["Control", "Control"]).unwrap();
    let outlined = Plot::scatter(x.clone(), y.clone()).hue(series.clone());
    let plain = Plot::scatter(x, y).hue(series).hide_point_outline();
    let canvas = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, outlined)
        .unwrap()
        .panel(0, 1, plain)
        .unwrap()
        .legend_policy(CanvasLegendPolicy::Shared);

    let error = canvas.preflight().unwrap_err().to_string();

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
