use super::*;

#[test]
fn marginal_hue_smoke() {
    let df = df!(
        "x" => [1.0_f64, 1.2, 1.4, 3.0, 3.2, 3.4],
        "y" => [2.0_f64, 2.1, 2.3, 4.0, 4.2, 4.3],
        "group" => ["a", "a", "a", "b", "b", "b"]
    )
    .unwrap();

    let dir = unique_dir("marginal-hue-smoke");
    let out = dir.join("marginal.svg");
    let mut chart = Chart::new(&df).group("group").hue("group");
    let _ = chart.x("x").y("y").size(520, 400).marginal();
    chart
        .render_to_svg(&out)
        .expect("marginal render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    assert!(svg.contains("Group"), "legend title should be rendered");
}

#[test]
fn marginal_auto_range_pads_central_points_from_frame() {
    let df = df!(
        "x" => [0.0_f64, 10.0],
        "y" => [0.0_f64, 10.0]
    )
    .unwrap();

    let dir = unique_dir("marginal-auto-range-padding");
    let out = dir.join("marginal.svg");
    let mut chart = Chart::new(&df);
    let _ = chart.x("x").y("y").size(360, 260).marginal();
    let central = marginal_central_panel(&chart, 96.0, 82.0, 12.0);
    chart
        .render_to_svg(&out)
        .expect("marginal render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let circles = parse_circles(&svg);
    let (min_x, max_x, min_y, max_y) = circle_bounds(&circles);
    let top = 260.0 - central.y1;
    let bottom = 260.0 - central.y0;

    assert!(
        min_x > central.x0 + 2.0
            && max_x < central.x1 - 2.0
            && min_y > top + 2.0
            && max_y < bottom - 2.0,
        "marginal central points should be padded from frame: circles={circles:?}, panel={central:?}"
    );
}

#[test]
fn marginal_auto_range_includes_kde_support_beyond_raw_data() {
    let df = df!(
        "x" => [0.0_f64, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0],
        "y" => [25.0_f64, 50.0, 25.0, 30.0, 50.0, 20.0, 80.0, 5.0, 15.0],
        "group" => ["a", "a", "a", "b", "b", "b", "c", "c", "c"]
    )
    .unwrap();

    let dir = unique_dir("marginal-kde-support-range");
    let out = dir.join("marginal.svg");
    let mut chart = Chart::new(&df).group("group").hue("group");
    let _ = chart.x("x").y("y").size(420, 300).marginal();
    let central = panel_to_svg_rect(marginal_central_panel(&chart, 96.0, 82.0, 12.0), 300.0);
    chart
        .render_to_svg(&out)
        .expect("marginal render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let text = parse_text_elements(&svg);
    let x_ticks = numeric_text_values(
        text.iter()
            .filter(|entry| entry.y > central.y + central.height),
    );
    let y_ticks = numeric_text_values(text.iter().filter(|entry| entry.x < central.x));

    assert!(
        x_ticks.iter().any(|tick| *tick < 0.0) && x_ticks.iter().any(|tick| *tick > 2.0),
        "marginal x-axis should include KDE support beyond raw 0..2: x_ticks={x_ticks:?}"
    );
    assert!(
        y_ticks.iter().any(|tick| *tick < 5.0) && y_ticks.iter().any(|tick| *tick > 80.0),
        "marginal y-axis should include KDE support beyond raw 5..80: y_ticks={y_ticks:?}"
    );
}

#[test]
fn marginal_kde_range_does_not_create_excessive_blank_space() {
    let mz: Vec<f64> = (0..300)
        .map(|i| 100.0 + f64::from(i) * (1400.0 / 299.0))
        .collect();
    let intensity: Vec<f64> = mz
        .iter()
        .map(|value| {
            let peak = (-0.5 * ((value - 700.0) / 35.0).powi(2)).exp() * 80.0;
            5.0 + peak
        })
        .collect();
    let region: Vec<&str> = mz
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
        .collect();
    let df = DataFrame::new(
        mz.len(),
        vec![
            Series::new("mz".into(), mz).into(),
            Series::new("intensity".into(), intensity).into(),
            Series::new("region".into(), region).into(),
        ],
    )
    .unwrap();

    let dir = unique_dir("marginal-kde-range-not-excessive");
    let out = dir.join("marginal.svg");
    let mut chart = Chart::new(&df).group("region").hue("region");
    let _ = chart.legend_position(LegendPosition::Hidden);
    let _ = chart.x("mz").y("intensity").size(420, 300).marginal();
    chart
        .render_to_svg(&out)
        .expect("marginal render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let text = parse_text_elements(&svg);
    let x_ticks = numeric_text_values(text.iter().filter(|entry| entry.y > 250.0));

    assert!(
        x_ticks.iter().all(|tick| *tick > -250.0 && *tick < 1850.0),
        "marginal x-axis should not create excessive blank KDE tail space: x_ticks={x_ticks:?}"
    );
}

#[test]
fn marginal_explicit_limits_override_kde_support_range() {
    let df = df!(
        "x" => [0.0_f64, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0],
        "y" => [25.0_f64, 50.0, 25.0, 30.0, 50.0, 20.0, 80.0, 5.0, 15.0],
        "group" => ["a", "a", "a", "b", "b", "b", "c", "c", "c"]
    )
    .unwrap();

    let dir = unique_dir("marginal-explicit-limits");
    let out = dir.join("marginal.svg");
    let mut chart = Chart::new(&df)
        .group("group")
        .hue("group")
        .x_limits(0.0, 2.0)
        .y_limits(0.0, 100.0);
    let _ = chart.x("x").y("y").size(420, 300).marginal();
    let _ = chart.hide_legend();
    chart
        .render_to_svg(&out)
        .expect("marginal render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let text = parse_text_elements(&svg);
    let x_ticks = numeric_text_values(text.iter().filter(|entry| entry.y > 250.0));
    let y_ticks = numeric_text_values(text.iter().filter(|entry| entry.x < 70.0));

    assert!(
        x_ticks.iter().all(|tick| (0.0..=2.0).contains(tick)),
        "explicit x limits should not expand for KDE support: x_ticks={x_ticks:?}"
    );
    assert!(
        y_ticks.iter().all(|tick| (0.0..=100.0).contains(tick)),
        "explicit y limits should not expand for KDE support: y_ticks={y_ticks:?}"
    );
}

#[test]
fn marginal_hue_legend_headroom_is_bounded() {
    let df = df!(
        "x" => [8.0_f64, 8.4, 8.8, 9.2, 9.6, 1.0, 2.0, 3.0],
        "y" => [8.0_f64, 8.5, 8.2, 9.0, 9.4, 1.0, 2.0, 1.5],
        "group" => ["high", "high", "high", "high", "high", "low", "low", "mid"]
    )
    .unwrap();

    let dir = unique_dir("marginal-legend-bounded-headroom");
    let out = dir.join("marginal.svg");
    let mut chart = Chart::new(&df).group("group").hue("group");
    let _ = chart.x("x").y("y").size(420, 320).marginal();
    chart
        .render_to_svg(&out)
        .expect("marginal render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert!(
        !svg.contains(">100</text>")
            && !svg.contains(">150</text>")
            && !svg.contains(">200</text>"),
        "marginal legend headroom should not create runaway y-axis ticks"
    );
}

#[test]
fn marginal_legend_avoids_upper_right_point_cluster() {
    let df = df!(
        "x" => [8.0_f64, 8.4, 8.8, 9.2, 9.6, 1.0, 2.0, 3.0],
        "y" => [8.0_f64, 8.5, 8.2, 9.0, 9.4, 1.0, 2.0, 1.5],
        "group" => ["high", "high", "high", "high", "high", "low", "low", "mid"]
    )
    .unwrap();

    let dir = unique_dir("marginal-legend-avoids-upper-right");
    let out = dir.join("marginal.svg");
    let mut chart = Chart::new(&df).group("group").hue("group");
    let _ = chart.x("x").y("y").size(420, 320).marginal();
    chart
        .render_to_svg(&out)
        .expect("marginal render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    let legend_rect = parse_legend_rect(&svg);
    let (_, legend_y, _, _) = legend_rect;
    let central = marginal_central_panel(&chart, 96.0, 82.0, 12.0);
    let central_top_screen = f64::from(chart.height()) - central.y1;
    assert!(
        legend_y <= central_top_screen + 30.0,
        "marginal legend should reserve top space before falling back; SVG y={legend_y}, central_top={central_top_screen}"
    );
    assert_inside_legend_clear(
        &svg,
        panel_to_svg_rect(central, f64::from(chart.height())),
        "marginal",
    );
}

#[test]
fn matrix_hue_title_smoke() {
    let df = df!(
        "a" => [1.0_f64, 1.2, 1.4, 3.0, 3.2, 3.4],
        "b" => [2.0_f64, 2.1, 2.3, 4.0, 4.2, 4.3],
        "c" => [0.5_f64, 0.7, 0.9, 1.5, 1.7, 1.9],
        "group" => ["a", "a", "a", "b", "b", "b"]
    )
    .unwrap();

    let dir = unique_dir("matrix-hue-title-smoke");
    let out = dir.join("matrix.svg");
    let mut chart = Chart::new(&df)
        .group("group")
        .hue("group")
        .title("Pairplot Title");
    let _ = chart.matrix_variables(vec!["a", "b", "c"]);
    let _ = chart.size(520, 420).matrix();
    chart
        .render_to_svg(&out)
        .expect("matrix render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    assert!(
        svg.contains("Pairplot Title"),
        "matrix title should be rendered"
    );
}

#[test]
fn matrix_legend_stays_inside_canvas_for_long_labels() {
    let df = df!(
        "a" => [1.0_f64, 1.2, 1.4, 3.0, 3.2, 3.4],
        "b" => [2.0_f64, 2.1, 2.3, 4.0, 4.2, 4.3],
        "c" => [0.5_f64, 0.7, 0.9, 1.5, 1.7, 1.9],
        "group" => [
            "long high income",
            "long high income",
            "long high income",
            "long low income",
            "long low income",
            "long low income"
        ]
    )
    .unwrap();

    let dir = unique_dir("matrix-long-legend");
    let out = dir.join("matrix.svg");
    let mut chart = Chart::new(&df).group("group").hue("group");
    let _ = chart.matrix_variables(vec!["a", "b", "c"]);
    let _ = chart.size(520, 420).matrix();
    chart
        .render_to_svg(&out)
        .expect("matrix render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let (legend_x, _, legend_w, _) = parse_legend_rect(&svg);

    assert!(
        legend_x >= 0.0 && legend_x + legend_w <= 520.0,
        "matrix legend should fit within canvas: x={legend_x}, width={legend_w}"
    );
}

#[test]
fn matrix_legend_does_not_cover_points() {
    let df = df!(
        "a" => [1.0_f64, 1.2, 1.4, 8.0, 8.2, 8.4, 4.0, 4.2],
        "b" => [1.0_f64, 1.1, 1.3, 8.0, 8.1, 8.3, 4.0, 4.1],
        "c" => [0.5_f64, 0.7, 0.9, 9.0, 9.2, 9.4, 4.5, 4.7],
        "group" => ["alpha", "alpha", "alpha", "beta", "beta", "beta", "gamma", "gamma"]
    )
    .unwrap();

    let dir = unique_dir("matrix-legend-clear-of-points");
    let out = dir.join("matrix.svg");
    let mut chart = Chart::new(&df).group("group").hue("group");
    let _ = chart.size(420, 320);
    let _ = chart.matrix_variables(vec!["a", "b", "c"]);
    let _ = chart.matrix();
    chart
        .render_to_svg(&out)
        .expect("matrix render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");

    assert_legend_does_not_overlap_graphics(&svg, "matrix");
}

#[test]
fn matrix_outside_legend_stays_close_to_graphics() {
    let df = df!(
        "a" => [1.0_f64, 1.2, 1.4, 8.0, 8.2, 8.4, 4.0, 4.2],
        "b" => [1.0_f64, 1.1, 1.3, 8.0, 8.1, 8.3, 4.0, 4.1],
        "c" => [0.5_f64, 0.7, 0.9, 9.0, 9.2, 9.4, 4.5, 4.7],
        "group" => ["alpha", "alpha", "alpha", "beta", "beta", "beta", "gamma", "gamma"]
    )
    .unwrap();

    let dir = unique_dir("matrix-legend-gap");
    let out = dir.join("matrix.svg");
    let mut chart = Chart::new(&df).group("group").hue("group");
    let _ = chart.size(420, 320);
    let _ = chart.matrix_variables(vec!["a", "b", "c"]);
    let _ = chart.matrix();
    chart
        .render_to_svg(&out)
        .expect("matrix render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let gap = legend_gap_from_graphics(&svg);

    assert!(
        (10.0..=24.0).contains(&gap),
        "matrix legend should sit near, but not overlap, the graphics; gap={gap}"
    );
}

#[test]
fn matrix_title_and_caption_do_not_overlap_content() {
    let df = df!(
        "a" => [1.0_f64, 2.0, 3.0, 4.0, 5.0],
        "b" => [1.5_f64, 2.5, 3.5, 4.5, 5.5],
        "c" => [5.0_f64, 4.0, 3.0, 2.0, 1.0]
    )
    .unwrap();

    let dir = unique_dir("matrix-title-caption-spacing");
    let out = dir.join("matrix.svg");
    let mut chart = Chart::new(&df)
        .title("Matrix title")
        .caption("Matrix caption");
    let _ = chart.size(420, 320);
    let _ = chart.matrix_variables(vec!["a", "b", "c"]);
    let _ = chart.matrix();
    chart
        .render_to_svg(&out)
        .expect("matrix render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let text = parse_text_elements(&svg);
    let title_y = text_y(&text, "Matrix title");
    let caption_y = text_y(&text, "Matrix caption");
    let graphics = parse_graphic_bounds(&svg);
    let graphic_top = graphics
        .iter()
        .map(|bounds| bounds.y)
        .fold(f64::INFINITY, f64::min);
    let bottom_text_y = text
        .iter()
        .filter(|entry| entry.content != "Matrix title" && entry.content != "Matrix caption")
        .map(|entry| entry.y)
        .fold(f64::NEG_INFINITY, f64::max);

    assert!(
        title_y + 18.0 < graphic_top,
        "matrix title should be separated from graphics: title_y={title_y}, graphic_top={graphic_top}"
    );
    assert!(
        bottom_text_y + 12.0 < caption_y,
        "matrix caption should be separated from bottom labels: label_y={bottom_text_y}, caption_y={caption_y}"
    );
}

#[test]
fn matrix_outside_right_legend_can_be_center_aligned() {
    let df = df!(
        "a" => [1.0_f64, 1.2, 1.4, 8.0, 8.2, 8.4],
        "b" => [1.0_f64, 1.1, 1.3, 8.0, 8.1, 8.3],
        "c" => [0.5_f64, 0.7, 0.9, 9.0, 9.2, 9.4],
        "group" => ["alpha", "alpha", "alpha", "beta", "beta", "beta"]
    )
    .unwrap();

    let dir = unique_dir("matrix-legend-center-right");
    let out = dir.join("matrix.svg");
    let mut chart = Chart::new(&df).group("group").hue("group");
    let _ = chart
        .legend_position(LegendPosition::OutsideRight)
        .size(520, 420);
    let _ = chart.matrix_variables(vec!["a", "b", "c"]);
    let _ = chart.matrix();
    chart
        .render_to_svg(&out)
        .expect("matrix render must succeed");
    let svg = std::fs::read_to_string(&out).expect("svg must exist");
    let legend = parse_legend_svg_rect(&svg);
    let legend_mid = legend.y + legend.height / 2.0;

    assert!(
        (legend_mid - 210.0).abs() < 40.0,
        "matrix outside-right legend should be vertically centered: legend={legend:?}"
    );
}
