use super::*;

struct StyledInteractionData {
    condition: CategoryValues,
    group: CategoryValues,
    hue: CategoryValues,
    estimate: NumericValues,
    lower: NumericValues,
    upper: NumericValues,
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
        styles: vec![
            InteractionStyle::new("Control")
                .unwrap()
                .line_pattern(LinePattern::Dashed)
                .unwrap(),
            InteractionStyle::new("Semantic Agent")
                .unwrap()
                .point_shape(PointShape::TriangleUp)
                .unwrap(),
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
fn native_interaction_keeps_panel_and_prose_inside_the_requested_root() {
    use super::visual::{
        parse_first_clip_rect, parse_text_elements, png_dimensions, text_y, unique_dir, SvgRect,
    };
    let directory = unique_dir("native-interaction");
    let out = directory.join("plot.svg");
    Plot::interaction(
        CategoryValues::new(
            "condition",
            [
                "after_visit",
                "before_visit",
                "last_visit",
                "after_visit",
                "last_visit",
            ],
        )
        .unwrap(),
        NumericValues::new("estimate", [2.0, 3.0, 4.0, 5.0, 6.0]),
        CategoryValues::new("group", ["one", "one", "one", "two", "two"]).unwrap(),
        Bounds::new(
            NumericValues::new("lower", [1.0, 2.0, 3.0, 4.0, 5.0]),
            NumericValues::new("upper", [3.0, 4.0, 5.0, 6.0, 7.0]),
        ),
    )
    .hue(CategoryValues::new("hue", ["first", "first", "first", "second", "second"]).unwrap())
    .styles([
        InteractionStyle::new("two")
            .unwrap()
            .point_shape(PointShape::TriangleUp)
            .unwrap()
            .line_pattern(LinePattern::Dashed)
            .unwrap(),
        InteractionStyle::new("one").unwrap(),
    ])
    .labels(Labels::humanized().alias("after_visit", "A longer follow-up visit"))
    .size(641, 401)
    .title("Native interaction profiles")
    .x_label("First axis line\nSecond axis line\nThird axis line")
    .caption("Independent caption\nSecond caption line")
    .svg_preserve_text(&out)
    .unwrap()
    .png_preview(out.with_extension("png"))
    .unwrap();
    assert_eq!(
        png_dimensions(&std::fs::read(out.with_extension("png")).unwrap()),
        (641, 401)
    );
    let svg = std::fs::read_to_string(&out).unwrap();
    let panel = parse_first_clip_rect(&svg);
    assert!(SvgRect {
        x: 0.0,
        y: 0.0,
        width: 641.0,
        height: 401.0
    }
    .contains(panel));
    let text = parse_text_elements(&svg);
    assert!(text_y(&text, "Native interaction profiles") < panel.y);
    let caption = text_y(&text, "Independent caption");
    for label in ["First axis line", "Second axis line", "Third axis line"] {
        let y = text_y(&text, label);
        assert!(y > panel.y + panel.height && y < caption);
    }
    std::fs::remove_dir_all(directory).unwrap();
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
        .filter(|line| line.starts_with("<polygon points=") && line.contains("fill-opacity="))
        .map(triangle_vertices)
        .collect::<Vec<_>>();

    assert_eq!(output.matches("stroke-dasharray=\"5 3\"").count(), 2);
    assert_eq!(triangles.len(), 3);
    let alphas = output
        .lines()
        .filter(|line| line.starts_with("<polygon points="))
        .filter_map(|line| {
            line.split_once("fill-opacity=\"").map(|(_, value)| {
                value
                    .split_once('"')
                    .unwrap()
                    .0
                    .parse::<f64>()
                    .unwrap()
                    .to_bits()
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(alphas, [(209.0_f64 / 255.0).to_bits(); 3]);
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
        .styles([InteractionStyle::new("Unknown").unwrap()])
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains("unknown graphical group \"Unknown\""));
}

#[test]
fn interaction_styles_require_one_style_per_group() {
    let data = interaction_data();
    let error = chart(&data, false)
        .styles([InteractionStyle::new("Control").unwrap()])
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
            InteractionStyle::new("Control").unwrap(),
            InteractionStyle::new("Control")
                .unwrap()
                .line_pattern(LinePattern::Dashed)
                .unwrap(),
        ])
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains("graphical group \"Control\" has more than one configured style"));
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
    let colors = CategoricalColorMap::new([
        ("h1".to_owned(), OpaqueColor::rgb(51, 51, 51)),
        ("h2".to_owned(), OpaqueColor::rgb(0, 114, 178)),
    ])
    .unwrap();
    let output = svg(
        Plot::interaction(condition, estimate, group, Bounds::new(lower, upper))
            .hue(hue)
            .styles([
                InteractionStyle::new("g1")
                    .unwrap()
                    .line_pattern(LinePattern::Dashed)
                    .unwrap(),
                InteractionStyle::new("g2")
                    .unwrap()
                    .point_shape(PointShape::TriangleUp)
                    .unwrap(),
            ])
            .categorical_color_scale(CategoricalColorScale::from_map(colors))
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
    let colors = CategoricalColorMap::new([
        ("h1".to_owned(), OpaqueColor::rgb(51, 51, 51)),
        ("h2".to_owned(), OpaqueColor::rgb(0, 114, 178)),
    ])
    .unwrap();
    let output = svg(
        Plot::interaction(condition, estimate, group, Bounds::new(lower, upper))
            .hue(hue)
            .styles([
                InteractionStyle::new("g1")
                    .unwrap()
                    .line_pattern(LinePattern::Dashed)
                    .unwrap(),
                InteractionStyle::new("g2")
                    .unwrap()
                    .point_shape(PointShape::TriangleUp)
                    .unwrap(),
            ])
            .categorical_color_scale(CategoricalColorScale::from_map(colors)),
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
        InteractionStyle::new("Control")
            .unwrap()
            .line_pattern(LinePattern::Dashed)
            .unwrap(),
    );
    let semantic = single_series_data(
        CategoryValues::new("series", ["Semantic Agent", "Semantic Agent"]).unwrap(),
        NumericValues::new("estimate", [1.2, 2.0]),
        NumericValues::new("lower", [1.0, 1.8]),
        NumericValues::new("upper", [1.4, 2.2]),
        InteractionStyle::new("Semantic Agent")
            .unwrap()
            .point_shape(PointShape::TriangleUp)
            .unwrap(),
    );
    let colors = CategoricalColorMap::new([
        ("Control".to_owned(), OpaqueColor::rgb(51, 51, 51)),
        ("Semantic Agent".to_owned(), OpaqueColor::rgb(0, 114, 178)),
    ])
    .unwrap();
    let output = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(
                0,
                0,
                chart(&control, true)
                    .categorical_color_scale(CategoricalColorScale::from_map(colors.clone())),
            )
            .unwrap()
            .panel(
                0,
                1,
                chart(&semantic, true)
                    .categorical_color_scale(CategoricalColorScale::from_map(colors)),
            )
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
        InteractionStyle::new("Control")
            .unwrap()
            .line_pattern(LinePattern::Dashed)
            .unwrap(),
    );
    let gray =
        CategoricalColorMap::new([("Control".to_owned(), OpaqueColor::rgb(51, 51, 51))]).unwrap();
    let blue =
        CategoricalColorMap::new([("Control".to_owned(), OpaqueColor::rgb(0, 114, 178))]).unwrap();
    for composition in [
        lichtung_plot::LegendComposition::Combined,
        lichtung_plot::LegendComposition::Separate,
    ] {
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(
                0,
                0,
                chart(&data, true)
                    .categorical_color_scale(CategoricalColorScale::from_map(gray.clone())),
            )
            .unwrap()
            .panel(
                0,
                1,
                chart(&data, true)
                    .categorical_color_scale(CategoricalColorScale::from_map(blue.clone())),
            )
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared)
            .legend_composition(composition);

        let error = canvas.preflight().unwrap_err().to_string();

        assert!(error.contains("compatible semantic titles, labels, and styles"));
    }
}

#[test]
fn shared_canvas_legend_rejects_conflicting_custom_outline_alpha_before_fonts() {
    use lichtung_plot::{Color, FontFamily, FontStack, PointOutlineTheme, RelativeSpacing, Theme};
    let custom = |alpha| {
        let base = Theme::publication();
        let outline = PointOutlineTheme::new(
            Color::rgba(19, 53, 91, alpha),
            RelativeSpacing::ratio(3, 10).unwrap(),
        )
        .unwrap();
        let primary = base
            .marks()
            .points()
            .primary()
            .clone()
            .with_outline(Some(outline))
            .unwrap();
        let points = base.marks().points().clone().with_primary(primary).unwrap();
        let marks = base.marks().clone().with_points(points).unwrap();
        base.customize()
            .with_marks(marks)
            .with_font_stack(
                FontStack::new([FontFamily::named("Absent shared-key preflight font").unwrap()])
                    .unwrap(),
            )
            .unwrap()
            .build()
            .unwrap()
    };
    let x = NumericValues::new("x", [1.0, 2.0]);
    let y = NumericValues::new("y", [2.0, 3.0]);
    let series = CategoryValues::new("series", ["Control", "Control"]).unwrap();
    for composition in [
        lichtung_plot::LegendComposition::Combined,
        lichtung_plot::LegendComposition::Separate,
    ] {
        let first = Plot::scatter(x.clone(), y.clone())
            .hue(series.clone())
            .theme(custom(73));
        let second = Plot::scatter(x.clone(), y.clone())
            .hue(series.clone())
            .theme(custom(109));
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, first)
            .unwrap()
            .panel(0, 1, second)
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared)
            .legend_composition(composition);

        let error = canvas.preflight().unwrap_err();
        assert!(matches!(error, lichtung_plot::Error::Render(_)), "{error}");
        let error = error.to_string();

        assert!(error.contains("compatible semantic titles, labels, and styles"));
    }
}

#[test]
fn shared_separate_sections_keep_distinct_group_fields_and_one_common_hue() {
    use lichtung_plot::LegendComposition;
    let mut first = interaction_data();
    first.group = CategoryValues::new(
        "cohort",
        ["Control", "Control", "Semantic Agent", "Semantic Agent"],
    )
    .unwrap();
    first.hue = CategoryValues::new("shade", ["red", "red", "blue", "blue"]).unwrap();
    let mut second = interaction_data();
    second.group = CategoryValues::new(
        "model",
        ["Control", "Control", "Semantic Agent", "Semantic Agent"],
    )
    .unwrap();
    second.hue = first.hue.clone();
    let canvas = |composition, labels: Labels| {
        Canvas::grid(1, 2)
            .unwrap()
            .panel(
                0,
                0,
                chart(&first, true).legend_composition(LegendComposition::Combined),
            )
            .unwrap()
            .panel(
                0,
                1,
                chart(&second, true).legend_composition(LegendComposition::Separate),
            )
            .unwrap()
            .labels(labels)
            .legend_policy(CanvasLegendPolicy::Shared)
            .legend_composition(composition)
            .size(1200, 600)
    };
    let output = render_canvas(
        canvas(LegendComposition::Separate, Labels::raw()),
        "shared-distinct-group-sections",
    )
    .unwrap();
    let guide = output
        .split_once("<g id=\"canvas-shared-legend\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;
    let mut rest = guide;
    for label in [
        "cohort",
        "Control",
        "Semantic Agent",
        "model",
        "Control",
        "Semantic Agent",
        "shade",
        "red",
        "blue",
    ] {
        rest = rest.split_once(&format!(">{label}</text>")).unwrap().1;
    }
    assert_eq!(guide.matches(">shade</text>").count(), 1);
    assert_eq!(guide.matches("<circle ").count(), 4);
    assert_eq!(guide.matches("<polygon ").count(), 2);
    assert_eq!(guide.matches("<polyline ").count(), 6);
    assert_eq!(
        output.matches("<circle ").count() + output.matches("<polygon ").count(),
        14
    );
    let combined = canvas(LegendComposition::Combined, Labels::raw())
        .preflight()
        .unwrap_err();
    assert!(combined.to_string().contains("compatible semantic titles"));
    let ambiguous = canvas(
        LegendComposition::Separate,
        Labels::raw()
            .alias("cohort", "Group")
            .alias("model", "Group"),
    )
    .preflight()
    .unwrap_err();
    assert!(ambiguous
        .to_string()
        .contains("distinct section identities"));
}

#[test]
fn categorical_colors_reject_missing_trace_mapping() {
    let data = interaction_data();
    let colors =
        CategoricalColorMap::new([("Control".to_owned(), OpaqueColor::rgb(51, 51, 51))]).unwrap();
    let error = chart(&data, true)
        .categorical_color_scale(CategoricalColorScale::from_map(colors))
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains("Semantic Agent") && error.contains("no caller-owned color"));
}

#[test]
fn categorical_colors_reject_duplicate_or_malformed_labels() {
    assert!(CategoricalColorMap::new([
        ("Control".to_owned(), OpaqueColor::rgb(1, 2, 3)),
        ("Control".to_owned(), OpaqueColor::rgb(4, 5, 6)),
    ])
    .is_err());
    assert!(
        CategoricalColorMap::new([(" Control".to_owned(), OpaqueColor::rgb(1, 2, 3))]).is_err()
    );
}
