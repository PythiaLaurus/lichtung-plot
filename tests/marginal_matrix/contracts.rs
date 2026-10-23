use super::*;

fn render<K: lichtung_plot::kind::Marker>(plot: Plot<K>, name: &str) -> String {
    let directory = unique_dir(name);
    let output = directory.join("plot.svg");
    plot.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

struct MarginalData {
    x: NumericValues,
    y: NumericValues,
    group: Option<CategoryValues>,
    hue: Option<CategoryValues>,
}

struct MatrixData {
    values: [NumericValues; 3],
    group: Option<CategoryValues>,
    hue: Option<CategoryValues>,
}

fn marginal(data: &MarginalData) -> Plot<lichtung_plot::kind::Marginal> {
    Plot::marginal(data.x.clone(), data.y.clone())
}

fn matrix(data: &MatrixData) -> Plot<lichtung_plot::kind::Matrix> {
    Plot::matrix(data.values.to_vec())
}

#[test]
fn marginal_legend_omits_hues_from_discarded_nonfinite_rows() {
    Plot::marginal(
        NumericValues::new("x", [1.0, 1.2, 1.4, f64::NAN]),
        NumericValues::new("y", [2.0, 2.1, 2.3, 4.0]),
    )
    .hue(CategoryValues::new("group", ["kept", "kept", "kept", "\u{10FFFF}"]).unwrap())
    .preflight()
    .unwrap();
}

#[test]
fn hue_only_marginal_and_matrix_split_multiple_hues() {
    let hue = CategoryValues::new("hue", ["h1", "h1", "h1", "h2", "h2", "h2"]).unwrap();
    let marginal_data = MarginalData {
        x: NumericValues::new("x", [1.0, 1.2, 1.4, 3.0, 3.2, 3.4]),
        y: NumericValues::new("y", [2.0, 2.1, 2.3, 4.0, 4.2, 4.3]),
        group: None,
        hue: Some(hue.clone()),
    };
    let matrix_data = MatrixData {
        values: [
            NumericValues::new("a", [0.5, 0.7, 0.9, 1.5, 1.7, 1.9]),
            NumericValues::new("b", [1.5, 1.7, 1.9, 2.5, 2.7, 2.9]),
            NumericValues::new("c", [2.5, 2.7, 2.9, 3.5, 3.7, 3.9]),
        ],
        group: None,
        hue: Some(hue),
    };
    let marginal_svg = render(
        marginal(&marginal_data)
            .hue(marginal_data.hue.clone().unwrap())
            .size(520, 400),
        "marginal-hue-only",
    );
    let matrix_svg = render(
        matrix(&matrix_data)
            .hue(matrix_data.hue.clone().unwrap())
            .size(620, 520),
        "matrix-hue-only",
    );

    assert!(marginal_svg.contains(">H 1</text>") && marginal_svg.contains(">H 2</text>"));
    assert!(matrix_svg.contains(">H 1</text>") && matrix_svg.contains(">H 2</text>"));
}

#[test]
fn marginal_and_matrix_render_each_observed_crossed_pair() {
    let group = CategoryValues::new(
        "group",
        ["g1", "g1", "g1", "g1", "g1", "g1", "g2", "g2", "g2"],
    )
    .unwrap();
    let hue = CategoryValues::new(
        "hue",
        ["h1", "h1", "h1", "h2", "h2", "h2", "h2", "h2", "h2"],
    )
    .unwrap();
    let marginal_data = MarginalData {
        x: NumericValues::new("x", [1.0, 1.2, 1.4, 2.0, 2.2, 2.4, 3.0, 3.2, 3.4]),
        y: NumericValues::new("y", [2.0, 2.2, 2.4, 3.0, 3.2, 3.4, 4.0, 4.2, 4.4]),
        group: Some(group.clone()),
        hue: Some(hue.clone()),
    };
    let matrix_data = MatrixData {
        values: [
            NumericValues::new("a", [0.5, 0.7, 0.9, 1.5, 1.7, 1.9, 2.5, 2.7, 2.9]),
            NumericValues::new("b", [1.5, 1.7, 1.9, 2.5, 2.7, 2.9, 3.5, 3.7, 3.9]),
            NumericValues::new("c", [2.5, 2.7, 2.9, 3.5, 3.7, 3.9, 4.5, 4.7, 4.9]),
        ],
        group: Some(group),
        hue: Some(hue),
    };
    let marginal_svg = render(
        marginal(&marginal_data)
            .group(marginal_data.group.clone().unwrap())
            .hue(marginal_data.hue.clone().unwrap())
            .size(520, 400),
        "marginal-crossed-pairs",
    );
    let matrix_svg = render(
        matrix(&matrix_data)
            .group(matrix_data.group.clone().unwrap())
            .hue(matrix_data.hue.clone().unwrap())
            .size(620, 520),
        "matrix-crossed-pairs",
    );

    assert_eq!(
        remove_legend_layers(&marginal_svg)
            .matches("fill-opacity=\"0.45098039215686275\"")
            .count(),
        6
    );
    assert_eq!(
        remove_legend_layers(&matrix_svg)
            .matches("fill-opacity=\"0.4\"")
            .count(),
        9
    );
}

#[test]
fn marginal_collectives_ignore_rows_dropped_from_numeric_geometry() {
    let data = MarginalData {
        x: NumericValues::nullable("x", [Some(1.0), Some(2.0), Some(3.0), None]),
        y: NumericValues::new("y", [2.0, 3.0, 5.0, 8.0]),
        group: Some(
            CategoryValues::nullable("group", [Some("kept"), Some("kept"), Some("kept"), None])
                .unwrap(),
        ),
        hue: Some(
            CategoryValues::nullable(
                "hue",
                [Some("visible"), Some("visible"), Some("visible"), None],
            )
            .unwrap(),
        ),
    };
    let svg = render(
        marginal(&data)
            .group(data.group.clone().unwrap())
            .hue(data.hue.clone().unwrap())
            .size(520, 400),
        "marginal-retained-collectives",
    );

    assert!(!svg.contains("Dropped") && !svg.contains("null"));
}

#[test]
fn marginal_same_name_validation_ignores_dropped_rows() {
    let plot = Plot::marginal(
        lichtung_plot::NumericValues::nullable("x", [Some(1.0), Some(2.0), Some(3.0), None]),
        lichtung_plot::NumericValues::new("y", [2.0, 3.0, 5.0, 8.0]),
    )
    .group(
        lichtung_plot::CategoryValues::new("identity", ["kept", "kept", "kept", "dropped group"])
            .unwrap(),
    )
    .hue(
        lichtung_plot::CategoryValues::new("identity", ["kept", "kept", "kept", "dropped hue"])
            .unwrap(),
    );

    plot.preflight().unwrap();
}

#[test]
fn marginal_reports_insufficient_retained_pair_with_group_and_hue() {
    let data = MarginalData {
        x: NumericValues::nullable("x", [Some(1.0), Some(2.0), Some(3.0), Some(4.0), None]),
        y: NumericValues::new("y", [2.0, 3.0, 5.0, 6.0, 7.0]),
        group: Some(CategoryValues::new("group", ["g1", "g1", "g1", "g2", "g2"]).unwrap()),
        hue: Some(CategoryValues::new("hue", ["h1", "h1", "h1", "h2", "h2"]).unwrap()),
    };
    let error = marginal(&data)
        .group(data.group.clone().unwrap())
        .hue(data.hue.clone().unwrap())
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error
        .contains("x distribution for graphical group \"g2\" with hue \"h2\" requires at least 2"));
}

#[test]
fn matrix_pair_error_identifies_group_and_hue() {
    let data = MatrixData {
        values: [
            NumericValues::new("a", [1.0, 2.0, 3.0]),
            NumericValues::new("b", [2.0, 3.0, 4.0]),
            NumericValues::new("c", [3.0, 4.0, 5.0]),
        ],
        group: Some(CategoryValues::new("group", ["small", "valid", "valid"]).unwrap()),
        hue: Some(CategoryValues::new("hue", ["red", "blue", "blue"]).unwrap()),
    };
    let error = matrix(&data)
        .group(data.group.clone().unwrap())
        .hue(data.hue.clone().unwrap())
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains(
        "column \"a\" in graphical group \"group\"=\"small\" with hue \"hue\"=\"red\" requires at least 2 observations"
    ));
}

#[test]
fn marginal_auto_range_includes_kde_support_beyond_raw_data() {
    let group =
        CategoryValues::new("group", ["a", "a", "a", "b", "b", "b", "c", "c", "c"]).unwrap();
    let data = MarginalData {
        x: NumericValues::new("x", [0.0, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0]),
        y: NumericValues::new("y", [25.0, 50.0, 25.0, 30.0, 50.0, 20.0, 80.0, 5.0, 15.0]),
        group: Some(group.clone()),
        hue: Some(group),
    };
    let plot = marginal(&data)
        .group(data.group.clone().unwrap())
        .hue(data.hue.clone().unwrap())
        .legend(LegendPolicy::Hidden)
        .size(420, 300);
    let svg = render(plot, "marginal-kde-support");
    let central = parse_first_clip_rect(&svg);
    // Domain support is independent of the adaptive source's selected positions.
    // The observed extrema (0..2 and 5..80) must project strictly inside it.
    let (min_x, max_x, min_y, max_y) = circle_bounds(&parse_circles(&svg));
    assert!(min_x > central.x && max_x < central.x + central.width);
    assert!(min_y > central.y && max_y < central.y + central.height);
}

#[test]
fn marginal_explicit_limits_override_kde_support_range() {
    let group =
        CategoryValues::new("group", ["a", "a", "a", "b", "b", "b", "c", "c", "c"]).unwrap();
    let data = MarginalData {
        x: NumericValues::new("x", [0.0, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0]),
        y: NumericValues::new("y", [25.0, 50.0, 25.0, 30.0, 50.0, 20.0, 80.0, 5.0, 15.0]),
        group: Some(group.clone()),
        hue: Some(group),
    };
    let svg = render(
        marginal(&data)
            .group(data.group.clone().unwrap())
            .hue(data.hue.clone().unwrap())
            .x_limits(0.0, 2.0)
            .y_limits(0.0, 100.0)
            .legend(LegendPolicy::Hidden)
            .size(420, 300),
        "marginal-explicit-limits",
    );
    let text = parse_text_elements(&svg);
    let x_ticks = numeric_text_values(text.iter().filter(|entry| entry.y > 250.0));
    let y_ticks = numeric_text_values(text.iter().filter(|entry| entry.x < 70.0));
    assert!(x_ticks.iter().all(|tick| (0.0..=2.0).contains(tick)));
    assert!(y_ticks.iter().all(|tick| (0.0..=100.0).contains(tick)));
}

#[test]
fn marginal_hue_legend_avoids_the_central_data() {
    let group = CategoryValues::new(
        "group",
        [
            "high", "high", "high", "high", "high", "low", "low", "mid", "mid",
        ],
    )
    .unwrap();
    let data = MarginalData {
        x: NumericValues::new("x", [8.0, 8.4, 8.8, 9.2, 9.6, 1.0, 2.0, 3.0, 3.4]),
        y: NumericValues::new("y", [8.0, 8.5, 8.2, 9.0, 9.4, 1.0, 2.0, 1.5, 1.8]),
        group: Some(group.clone()),
        hue: Some(group),
    };
    let plot = marginal(&data)
        .group(data.group.clone().unwrap())
        .hue(data.hue.clone().unwrap())
        .legend(LegendPolicy::Visible(LegendPosition::InsideTopRight))
        .size(420, 320);
    let svg = render(plot, "marginal-legend");
    let central = parse_first_clip_rect(&svg);
    assert_inside_legend_clear(&svg, central, "marginal");
}

fn matrix_data(groups: &[&str]) -> MatrixData {
    let group = CategoryValues::new("group", groups.iter().copied()).unwrap();
    MatrixData {
        values: [
            NumericValues::new("a", [1.0, 1.2, 1.4, 3.0, 3.2, 3.4]),
            NumericValues::new("b", [2.0, 2.1, 2.3, 4.0, 4.2, 4.3]),
            NumericValues::new("c", [0.5, 0.7, 0.9, 1.5, 1.7, 1.9]),
        ],
        group: Some(group.clone()),
        hue: Some(group),
    }
}

#[test]
fn matrix_legend_does_not_cover_points() {
    let group = CategoryValues::new(
        "group",
        [
            "alpha", "alpha", "alpha", "beta", "beta", "beta", "gamma", "gamma",
        ],
    )
    .unwrap();
    let data = MatrixData {
        values: [
            NumericValues::new("a", [1.0, 1.2, 1.4, 8.0, 8.2, 8.4, 4.0, 4.2]),
            NumericValues::new("b", [1.0, 1.1, 1.3, 8.0, 8.1, 8.3, 4.0, 4.1]),
            NumericValues::new("c", [0.5, 0.7, 0.9, 9.0, 9.2, 9.4, 4.5, 4.7]),
        ],
        group: Some(group.clone()),
        hue: Some(group),
    };
    let svg = render(
        matrix(&data)
            .group(data.group.clone().unwrap())
            .hue(data.hue.clone().unwrap())
            .size(420, 320),
        "matrix-legend-clear",
    );
    assert_legend_does_not_overlap_graphics(&svg, "matrix");
}

#[test]
fn matrix_hidden_legend_policy_suppresses_self_managed_legend() {
    let data = matrix_data(&["alpha", "alpha", "alpha", "beta", "beta", "beta"]);
    let svg = render(
        matrix(&data)
            .group(data.group.clone().unwrap())
            .hue(data.hue.clone().unwrap())
            .legend(LegendPolicy::Hidden)
            .size(420, 320),
        "matrix-hidden-legend",
    );

    assert!(!has_legend(&svg));
}

#[test]
fn matrix_title_and_caption_do_not_overlap_content() {
    let data = MatrixData {
        values: [
            NumericValues::new("a", [1.0, 2.0, 3.0, 4.0, 5.0]),
            NumericValues::new("b", [1.5, 2.5, 3.5, 4.5, 5.5]),
            NumericValues::new("c", [5.0, 4.0, 3.0, 2.0, 1.0]),
        ],
        group: None,
        hue: None,
    };
    let svg = render(
        matrix(&data)
            .title("Matrix title")
            .caption("Matrix caption")
            .size(420, 320),
        "matrix-title-caption",
    );
    let text = parse_text_elements(&svg);
    let title_y = text_y(&text, "Matrix title");
    let caption_y = text_y(&text, "Matrix caption");
    let graphics = parse_graphic_bounds(&svg);
    let graphic_top = graphics
        .iter()
        .map(|bounds| bounds.y)
        .fold(f64::INFINITY, f64::min);
    let bottom_text = text
        .iter()
        .filter(|entry| entry.content != "Matrix title" && entry.content != "Matrix caption")
        .map(|entry| entry.y)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(title_y + 18.0 < graphic_top);
    assert!(bottom_text + 12.0 < caption_y);
}
