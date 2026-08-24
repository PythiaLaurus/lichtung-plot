use super::*;

fn interaction_data() -> DataFrame {
    df!(
        "condition" => ["Before", "After", "Before", "After"],
        "series" => ["Control", "Control", "Semantic Agent", "Semantic Agent"],
        "estimate" => [1.0_f64, 1.5, 1.2, 2.0],
        "lower" => [0.8_f64, 1.3, 1.0, 1.8],
        "upper" => [1.2_f64, 1.7, 1.4, 2.2],
    )
    .unwrap()
}

fn chart(frame: &DataFrame, styled: bool) -> Plot<lichtung_plot::kind::Interaction> {
    let mut plot = Plot::interaction(
        data::category(frame, "condition"),
        data::numeric(frame, "estimate"),
        data::category(frame, "series"),
        Bounds::new(data::numeric(frame, "lower"), data::numeric(frame, "upper")),
    )
    .labels(Labels::raw())
    .hue(data::category(frame, "series"));
    if styled {
        let configurations = data::category_levels(frame, "series")
            .into_iter()
            .map(|group| match group.as_str() {
                "Control" => InteractionStyle::new(group).pattern(LinePattern::Dashed),
                "Semantic Agent" => InteractionStyle::new(group).shape(PointShape::TriangleUp),
                _ => InteractionStyle::new(group),
            });
        plot = plot.interaction_styles(configurations);
    }
    plot
}

fn svg<K: lichtung_plot::kind::Marker>(plot: Plot<K>) -> String {
    render_svg(plot, "interaction-style").unwrap()
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
        .interaction_styles([InteractionStyle::new("Unknown")])
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains("unknown graphical group \"Unknown\""));
}

#[test]
fn interaction_styles_require_one_style_per_group() {
    let data = interaction_data();
    let error = chart(&data, false)
        .interaction_styles([InteractionStyle::new("Control")])
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains("graphical group \"Semantic Agent\" has no configured style"));
}

#[test]
fn interaction_styles_reject_duplicate_groups() {
    let data = interaction_data();
    let error = chart(&data, false)
        .interaction_styles([
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
    let data = df!(
        "panel" => ["Left", "Left", "Right", "Right"],
        "condition" => ["Before", "After", "Before", "After"],
        "series" => ["Control", "Control", "Semantic Agent", "Semantic Agent"],
        "estimate" => [1.0_f64, 1.5, 1.2, 2.0],
        "lower" => [0.8_f64, 1.3, 1.0, 1.8],
        "upper" => [1.2_f64, 1.7, 1.4, 2.2],
    )
    .unwrap();

    let output = svg(chart(&data, true)
        .facet(FacetSpec::column(data::category(&data, "panel")))
        .size(960, 320));

    assert!(output.contains(">panel = Left</text>") && output.contains(">panel = Right</text>"));
    assert!(output.contains("stroke-dasharray=\"5 3\"") && output.contains("<polygon points="));
}

#[test]
fn faceted_interactions_keep_sparse_global_pair_styles_and_colors() {
    let data = df!(
        "panel" => ["Left", "Left", "Left", "Left", "Right", "Right", "Right", "Right"],
        "condition" => ["Before", "After", "Before", "After", "Before", "After", "Before", "After"],
        "group" => ["g1", "g1", "g1", "g1", "g1", "g1", "g2", "g2"],
        "hue" => ["h2", "h2", "h1", "h1", "h1", "h1", "h2", "h2"],
        "estimate" => [1.0_f64, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5],
        "lower" => [0.8_f64, 1.3, 1.8, 2.3, 2.8, 3.3, 3.8, 4.3],
        "upper" => [1.2_f64, 1.7, 2.2, 2.7, 3.2, 3.7, 4.2, 4.7],
    )
    .unwrap();
    let colors = CategoricalColors::new([
        ("h1".to_owned(), (51, 51, 51)),
        ("h2".to_owned(), (0, 114, 178)),
    ])
    .unwrap();
    let output = svg(Plot::interaction(
        data::category(&data, "condition"),
        data::numeric(&data, "estimate"),
        data::category(&data, "group"),
        Bounds::new(data::numeric(&data, "lower"), data::numeric(&data, "upper")),
    )
    .hue(data::category(&data, "hue"))
    .interaction_styles([
        InteractionStyle::new("g1").pattern(LinePattern::Dashed),
        InteractionStyle::new("g2").shape(PointShape::TriangleUp),
    ])
    .categorical_colors(colors)
    .facet(FacetSpec::column(data::category(&data, "panel")))
    .size(960, 320));
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
            clipped && line.starts_with("<polyline ") && line.contains("stroke-width=\"1.8\"")
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
    let data = df!(
        "condition" => ["Before", "After", "Before", "After", "Before", "After"],
        "group" => ["g1", "g1", "g1", "g1", "g2", "g2"],
        "hue" => ["h1", "h1", "h2", "h2", "h2", "h2"],
        "estimate" => [1.0_f64, 1.5, 2.0, 2.5, 3.0, 3.5],
        "lower" => [0.8_f64, 1.3, 1.8, 2.3, 2.8, 3.3],
        "upper" => [1.2_f64, 1.7, 2.2, 2.7, 3.2, 3.7],
    )
    .unwrap();
    let colors = CategoricalColors::new([
        ("h1".to_owned(), (51, 51, 51)),
        ("h2".to_owned(), (0, 114, 178)),
    ])
    .unwrap();
    let output = svg(Plot::interaction(
        data::category(&data, "condition"),
        data::numeric(&data, "estimate"),
        data::category(&data, "group"),
        Bounds::new(data::numeric(&data, "lower"), data::numeric(&data, "upper")),
    )
    .hue(data::category(&data, "hue"))
    .interaction_styles([
        InteractionStyle::new("g1").pattern(LinePattern::Dashed),
        InteractionStyle::new("g2").shape(PointShape::TriangleUp),
    ])
    .categorical_colors(colors));
    let marks = output
        .split_once("<g clip-path=")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;
    let connectors = marks
        .lines()
        .filter(|line| line.starts_with("<polyline ") && line.contains("stroke-width=\"1.8\""))
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
    let data = df!(
        "condition" => ["Before", "Before", "Before"],
        "group" => ["g1", "g1", "g1"],
        "hue" => ["h1", "h2", "h1"],
        "estimate" => [1.0_f64, 2.0, 3.0],
        "lower" => [0.8_f64, 1.8, 2.8],
        "upper" => [1.2_f64, 2.2, 3.2],
    )
    .unwrap();
    let error = Plot::interaction(
        data::category(&data, "condition"),
        data::numeric(&data, "estimate"),
        data::category(&data, "group"),
        Bounds::new(data::numeric(&data, "lower"), data::numeric(&data, "upper")),
    )
    .hue(data::category(&data, "hue"))
    .preflight()
    .unwrap_err()
    .to_string();

    assert!(error.contains("duplicate (condition, group, hue)") && error.contains("source row 2"));
}

#[test]
fn shared_figure_legend_preserves_caller_colors_and_composite_trace_glyphs() {
    let control = df!(
        "condition" => ["Before", "After"],
        "series" => ["Control", "Control"],
        "estimate" => [1.0_f64, 1.5],
        "lower" => [0.8_f64, 1.3],
        "upper" => [1.2_f64, 1.7],
    )
    .unwrap();
    let semantic = df!(
        "condition" => ["Before", "After"],
        "series" => ["Semantic Agent", "Semantic Agent"],
        "estimate" => [1.2_f64, 2.0],
        "lower" => [1.0_f64, 1.8],
        "upper" => [1.4_f64, 2.2],
    )
    .unwrap();
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
    )
    .unwrap();
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
fn shared_figure_legend_rejects_conflicting_point_outline_mapping() {
    let data = df!(
        "x" => [1.0_f64, 2.0],
        "y" => [2.0_f64, 3.0],
        "series" => ["Control", "Control"],
    )
    .unwrap();
    let outlined = Plot::scatter(data::numeric(&data, "x"), data::numeric(&data, "y"))
        .hue(data::category(&data, "series"));
    let plain = Plot::scatter(data::numeric(&data, "x"), data::numeric(&data, "y"))
        .hue(data::category(&data, "series"))
        .hide_point_outline();
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
