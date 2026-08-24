use super::*;

fn render<K: lichtung_plot::kind::Marker>(plot: Plot<K>, name: &str) -> String {
    let directory = unique_dir(name);
    let output = directory.join("plot.svg");
    plot.svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

fn marginal(frame: &DataFrame) -> Plot<lichtung_plot::kind::Marginal> {
    Plot::marginal(data::numeric(frame, "x"), data::numeric(frame, "y"))
}

fn matrix(frame: &DataFrame) -> Plot<lichtung_plot::kind::Matrix> {
    Plot::matrix(vec![
        data::numeric(frame, "a"),
        data::numeric(frame, "b"),
        data::numeric(frame, "c"),
    ])
}

#[test]
fn marginal_hue_smoke() {
    let frame = df!(
        "x" => [1.0_f64, 1.2, 1.4, 3.0, 3.2, 3.4],
        "y" => [2.0_f64, 2.1, 2.3, 4.0, 4.2, 4.3],
        "group" => ["a", "a", "a", "b", "b", "b"]
    )
    .unwrap();
    let svg = render(
        marginal(&frame)
            .group(data::category(&frame, "group"))
            .hue(data::category(&frame, "group"))
            .size(520, 400),
        "marginal-hue-smoke",
    );
    assert!(svg.contains(">Group</text>"));
}

#[test]
fn hue_only_marginal_and_matrix_split_multiple_hues() {
    let frame = df!(
        "x" => [1.0_f64, 1.2, 1.4, 3.0, 3.2, 3.4],
        "y" => [2.0_f64, 2.1, 2.3, 4.0, 4.2, 4.3],
        "a" => [0.5_f64, 0.7, 0.9, 1.5, 1.7, 1.9],
        "b" => [1.5_f64, 1.7, 1.9, 2.5, 2.7, 2.9],
        "c" => [2.5_f64, 2.7, 2.9, 3.5, 3.7, 3.9],
        "hue" => ["h1", "h1", "h1", "h2", "h2", "h2"]
    )
    .unwrap();
    let marginal_svg = render(
        marginal(&frame)
            .hue(data::category(&frame, "hue"))
            .size(520, 400),
        "marginal-hue-only",
    );
    let matrix_svg = render(
        matrix(&frame)
            .hue(data::category(&frame, "hue"))
            .size(620, 520),
        "matrix-hue-only",
    );

    assert!(marginal_svg.contains(">H 1</text>") && marginal_svg.contains(">H 2</text>"));
    assert!(matrix_svg.contains(">H 1</text>") && matrix_svg.contains(">H 2</text>"));
}

#[test]
fn marginal_and_matrix_render_each_observed_crossed_pair() {
    let frame = df!(
        "x" => [1.0_f64, 1.2, 1.4, 2.0, 2.2, 2.4, 3.0, 3.2, 3.4],
        "y" => [2.0_f64, 2.2, 2.4, 3.0, 3.2, 3.4, 4.0, 4.2, 4.4],
        "a" => [0.5_f64, 0.7, 0.9, 1.5, 1.7, 1.9, 2.5, 2.7, 2.9],
        "b" => [1.5_f64, 1.7, 1.9, 2.5, 2.7, 2.9, 3.5, 3.7, 3.9],
        "c" => [2.5_f64, 2.7, 2.9, 3.5, 3.7, 3.9, 4.5, 4.7, 4.9],
        "group" => ["g1", "g1", "g1", "g1", "g1", "g1", "g2", "g2", "g2"],
        "hue" => ["h1", "h1", "h1", "h2", "h2", "h2", "h2", "h2", "h2"]
    )
    .unwrap();
    let marginal_svg = render(
        marginal(&frame)
            .group(data::category(&frame, "group"))
            .hue(data::category(&frame, "hue"))
            .size(520, 400),
        "marginal-crossed-pairs",
    );
    let matrix_svg = render(
        matrix(&frame)
            .group(data::category(&frame, "group"))
            .hue(data::category(&frame, "hue"))
            .size(620, 520),
        "matrix-crossed-pairs",
    );

    assert_eq!(marginal_svg.matches("fill-opacity=\"0.45\"").count(), 6);
    assert_eq!(matrix_svg.matches("fill-opacity=\"0.40\"").count(), 9);
}

#[test]
fn marginal_collectives_ignore_rows_dropped_from_numeric_geometry() {
    let frame = DataFrame::from_columns(vec![
        (
            "x",
            data::FixtureColumn::Numeric(vec![Some(1.0), Some(2.0), Some(3.0), None]),
        ),
        (
            "y",
            data::FixtureColumn::Numeric(vec![Some(2.0), Some(3.0), Some(5.0), Some(8.0)]),
        ),
        (
            "group",
            data::FixtureColumn::Category(vec![
                Some("kept".to_owned()),
                Some("kept".to_owned()),
                Some("kept".to_owned()),
                None,
            ]),
        ),
        (
            "hue",
            data::FixtureColumn::Category(vec![
                Some("visible".to_owned()),
                Some("visible".to_owned()),
                Some("visible".to_owned()),
                None,
            ]),
        ),
    ])
    .unwrap();
    let svg = render(
        marginal(&frame)
            .group(data::category(&frame, "group"))
            .hue(data::category(&frame, "hue"))
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
    let frame = DataFrame::from_columns(vec![
        (
            "x",
            data::FixtureColumn::Numeric(vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0), None]),
        ),
        (
            "y",
            data::FixtureColumn::Numeric(vec![
                Some(2.0),
                Some(3.0),
                Some(5.0),
                Some(6.0),
                Some(7.0),
            ]),
        ),
        (
            "group",
            data::FixtureColumn::Category(
                ["g1", "g1", "g1", "g2", "g2"]
                    .into_iter()
                    .map(|value| Some(value.to_owned()))
                    .collect(),
            ),
        ),
        (
            "hue",
            data::FixtureColumn::Category(
                ["h1", "h1", "h1", "h2", "h2"]
                    .into_iter()
                    .map(|value| Some(value.to_owned()))
                    .collect(),
            ),
        ),
    ])
    .unwrap();
    let error = marginal(&frame)
        .group(data::category(&frame, "group"))
        .hue(data::category(&frame, "hue"))
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error
        .contains("x distribution for graphical group \"g2\" with hue \"h2\" requires at least 2"));
}

#[test]
fn matrix_pair_error_identifies_group_and_hue() {
    let frame = df!(
        "a" => [1.0_f64, 2.0, 3.0],
        "b" => [2.0_f64, 3.0, 4.0],
        "c" => [3.0_f64, 4.0, 5.0],
        "group" => ["small", "valid", "valid"],
        "hue" => ["red", "blue", "blue"]
    )
    .unwrap();
    let error = matrix(&frame)
        .group(data::category(&frame, "group"))
        .hue(data::category(&frame, "hue"))
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains(
        "column \"a\" in graphical group \"group\"=\"small\" with hue \"hue\"=\"red\" requires at least 2 observations"
    ));
}

#[test]
fn marginal_auto_range_pads_central_points_from_frame() {
    let frame = df!("x" => [0.0_f64, 10.0], "y" => [0.0_f64, 10.0]).unwrap();
    let plot = marginal(&frame).size(360, 260);
    let central = panel_to_svg_rect(marginal_central_panel(&plot, 96.0, 82.0, 12.0), 260.0);
    let svg = render(plot, "marginal-auto-range-padding");
    let (min_x, max_x, min_y, max_y) = circle_bounds(&parse_circles(&svg));
    assert!(min_x > central.x + 2.0 && max_x < central.x + central.width - 2.0);
    assert!(min_y > central.y + 2.0 && max_y < central.y + central.height - 2.0);
}

#[test]
fn marginal_auto_range_includes_kde_support_beyond_raw_data() {
    let frame = df!(
        "x" => [0.0_f64, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0],
        "y" => [25.0_f64, 50.0, 25.0, 30.0, 50.0, 20.0, 80.0, 5.0, 15.0],
        "group" => ["a", "a", "a", "b", "b", "b", "c", "c", "c"]
    )
    .unwrap();
    let plot = marginal(&frame)
        .group(data::category(&frame, "group"))
        .hue(data::category(&frame, "group"))
        .size(420, 300);
    let central = panel_to_svg_rect(marginal_central_panel(&plot, 96.0, 82.0, 12.0), 300.0);
    let svg = render(plot, "marginal-kde-support");
    let text = parse_text_elements(&svg);
    let x_ticks = numeric_text_values(
        text.iter()
            .filter(|entry| entry.y > central.y + central.height),
    );
    let y_ticks = numeric_text_values(text.iter().filter(|entry| entry.x < central.x));
    assert!(x_ticks.iter().any(|tick| *tick < 0.0) && x_ticks.iter().any(|tick| *tick > 2.0));
    assert!(y_ticks.iter().any(|tick| *tick < 5.0) && y_ticks.iter().any(|tick| *tick > 80.0));
}

#[test]
fn marginal_kde_range_does_not_create_excessive_blank_space() {
    let x = (0..300)
        .map(|index| 100.0 + f64::from(index) * (1400.0 / 299.0))
        .collect::<Vec<_>>();
    let y = x
        .iter()
        .map(|value| 5.0 + (-0.5 * ((value - 700.0) / 35.0).powi(2)).exp() * 80.0)
        .collect::<Vec<_>>();
    let region = x
        .iter()
        .map(|value| {
            if *value < 500.0 {
                "low"
            } else if *value < 1000.0 {
                "mid"
            } else {
                "high"
            }
        })
        .collect::<Vec<_>>();
    let plot = Plot::marginal(
        lichtung_plot::NumericValues::new("mz", x),
        lichtung_plot::NumericValues::new("intensity", y),
    )
    .group(lichtung_plot::CategoryValues::new("region", region.clone()).unwrap())
    .hue(lichtung_plot::CategoryValues::new("region", region).unwrap())
    .legend(LegendPolicy::Hidden)
    .size(420, 300);
    let svg = render(plot, "marginal-bounded-kde-range");
    let text = parse_text_elements(&svg);
    let ticks = numeric_text_values(text.iter().filter(|entry| entry.y > 250.0));
    assert!(ticks.iter().all(|tick| *tick > -250.0 && *tick < 1850.0));
}

#[test]
fn marginal_explicit_limits_override_kde_support_range() {
    let frame = df!(
        "x" => [0.0_f64, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0],
        "y" => [25.0_f64, 50.0, 25.0, 30.0, 50.0, 20.0, 80.0, 5.0, 15.0],
        "group" => ["a", "a", "a", "b", "b", "b", "c", "c", "c"]
    )
    .unwrap();
    let svg = render(
        marginal(&frame)
            .group(data::category(&frame, "group"))
            .hue(data::category(&frame, "group"))
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
fn marginal_hue_legend_headroom_is_bounded_and_avoids_cluster() {
    let frame = df!(
        "x" => [8.0_f64, 8.4, 8.8, 9.2, 9.6, 1.0, 2.0, 3.0, 3.4],
        "y" => [8.0_f64, 8.5, 8.2, 9.0, 9.4, 1.0, 2.0, 1.5, 1.8],
        "group" => ["high", "high", "high", "high", "high", "low", "low", "mid", "mid"]
    )
    .unwrap();
    let plot = marginal(&frame)
        .group(data::category(&frame, "group"))
        .hue(data::category(&frame, "group"))
        .size(420, 320);
    let central = panel_to_svg_rect(marginal_central_panel(&plot, 96.0, 82.0, 12.0), 320.0);
    let svg = render(plot, "marginal-legend");
    assert!(
        !svg.contains(">100</text>")
            && !svg.contains(">150</text>")
            && !svg.contains(">200</text>")
    );
    assert!(parse_legend_rect(&svg).1 <= central.y + 30.0);
    assert_inside_legend_clear(&svg, central, "marginal");
}

fn matrix_frame(groups: &[&str]) -> DataFrame {
    DataFrame::from_columns(vec![
        (
            "a",
            data::FixtureColumn::Numeric(vec![
                Some(1.0),
                Some(1.2),
                Some(1.4),
                Some(3.0),
                Some(3.2),
                Some(3.4),
            ]),
        ),
        (
            "b",
            data::FixtureColumn::Numeric(vec![
                Some(2.0),
                Some(2.1),
                Some(2.3),
                Some(4.0),
                Some(4.2),
                Some(4.3),
            ]),
        ),
        (
            "c",
            data::FixtureColumn::Numeric(vec![
                Some(0.5),
                Some(0.7),
                Some(0.9),
                Some(1.5),
                Some(1.7),
                Some(1.9),
            ]),
        ),
        (
            "group",
            data::FixtureColumn::Category(
                groups
                    .iter()
                    .map(|value| Some((*value).to_owned()))
                    .collect(),
            ),
        ),
    ])
    .unwrap()
}

#[test]
fn matrix_hue_title_smoke_and_long_legend_stays_inside_canvas() {
    let frame = matrix_frame(&[
        "long high income",
        "long high income",
        "long high income",
        "long low income",
        "long low income",
        "long low income",
    ]);
    let svg = render(
        matrix(&frame)
            .group(data::category(&frame, "group"))
            .hue(data::category(&frame, "group"))
            .title("Pairplot Title")
            .size(520, 420),
        "matrix-title-long-legend",
    );
    assert!(svg.contains(">Pairplot Title</text>"));
    let legend = parse_legend_svg_rect(&svg);
    assert!(legend.x >= 0.0 && legend.x + legend.width <= 520.0);
}

#[test]
fn matrix_legend_does_not_cover_points_and_stays_close_to_graphics() {
    let frame = df!(
        "a" => [1.0_f64, 1.2, 1.4, 8.0, 8.2, 8.4, 4.0, 4.2],
        "b" => [1.0_f64, 1.1, 1.3, 8.0, 8.1, 8.3, 4.0, 4.1],
        "c" => [0.5_f64, 0.7, 0.9, 9.0, 9.2, 9.4, 4.5, 4.7],
        "group" => ["alpha", "alpha", "alpha", "beta", "beta", "beta", "gamma", "gamma"]
    )
    .unwrap();
    let svg = render(
        matrix(&frame)
            .group(data::category(&frame, "group"))
            .hue(data::category(&frame, "group"))
            .size(420, 320),
        "matrix-legend-clear",
    );
    assert_legend_does_not_overlap_graphics(&svg, "matrix");
    assert!((10.0..=24.0).contains(&legend_gap_from_graphics(&svg)));
}

#[test]
fn matrix_hidden_legend_policy_suppresses_self_managed_legend() {
    let frame = matrix_frame(&["alpha", "alpha", "alpha", "beta", "beta", "beta"]);
    let svg = render(
        matrix(&frame)
            .group(data::category(&frame, "group"))
            .hue(data::category(&frame, "group"))
            .legend(LegendPolicy::Hidden)
            .size(420, 320),
        "matrix-hidden-legend",
    );

    assert!(!has_legend(&svg));
}

#[test]
fn matrix_title_and_caption_do_not_overlap_content() {
    let frame = df!(
        "a" => [1.0_f64, 2.0, 3.0, 4.0, 5.0],
        "b" => [1.5_f64, 2.5, 3.5, 4.5, 5.5],
        "c" => [5.0_f64, 4.0, 3.0, 2.0, 1.0]
    )
    .unwrap();
    let svg = render(
        matrix(&frame)
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

#[test]
fn matrix_outside_right_legend_can_be_center_aligned() {
    let frame = matrix_frame(&["alpha", "alpha", "alpha", "beta", "beta", "beta"]);
    let svg = render(
        matrix(&frame)
            .group(data::category(&frame, "group"))
            .hue(data::category(&frame, "group"))
            .legend(LegendPolicy::Visible(LegendPosition::OutsideRight))
            .size(520, 420),
        "matrix-centered-legend",
    );
    let legend = parse_legend_svg_rect(&svg);
    assert!((legend.y + legend.height / 2.0 - 210.0).abs() < 40.0);
}
