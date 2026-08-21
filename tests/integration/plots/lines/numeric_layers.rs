use lichtung_plot::{AxisTick, Chart, GuideSpec, GuideStyle, LinePattern, PointLayer};
use polars::prelude::*;

fn line_data() -> DataFrame {
    df!(
        "grid_x" => [0.0_f64, 1.0, 2.0],
        "estimate" => [1.0_f64, 2.0, 3.0],
        "lower" => [0.5_f64, 1.5, 2.5],
        "upper" => [1.5_f64, 2.5, 3.5],
        "panel" => ["a", "a", "a"],
    )
    .unwrap()
}

fn observations() -> DataFrame {
    df!(
        "observed_x" => [0.0_f64, 0.5, 1.0, 1.5, 2.0],
        "observed_y" => [0.8_f64, 1.7, 2.0, 2.8, 3.2],
    )
    .unwrap()
}

fn line_chart(data: &DataFrame) -> Chart<'_> {
    let mut chart = Chart::new(data).interval_bounds("lower", "upper");
    chart.x("grid_x").y("estimate").line();
    chart
}

fn svg(chart: &Chart<'_>) -> String {
    String::from_utf8(chart.render_to_svg_bytes().unwrap()).unwrap()
}

fn attribute(line: &str, name: &str) -> f64 {
    let prefix = format!("{name}=\"");
    line.split_once(&prefix)
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .parse()
        .unwrap()
}

fn text_line<'a>(svg: &'a str, text: &str) -> &'a str {
    svg.lines()
        .find(|line| line.contains(&format!(">{text}</text>")))
        .unwrap()
}

fn style(color: (u8, u8, u8), pattern: LinePattern) -> GuideStyle {
    GuideStyle::new(color, 1.25, pattern).unwrap()
}

#[test]
fn explicit_numeric_ticks_render_verbatim_text_at_caller_positions_on_both_axes() {
    let data = line_data();
    let chart = line_chart(&data)
        .x_limits(0.0, 2.0)
        .y_limits(0.0, 4.0)
        .x_ticks(vec![
            AxisTick::new(0.5, "early x"),
            AxisTick::new(1.5, "late x"),
        ])
        .y_ticks(vec![
            AxisTick::new(1.0, "low y"),
            AxisTick::new(3.0, "high y"),
        ]);

    let output = svg(&chart);
    let early_x = attribute(text_line(&output, "early x"), "x");
    let late_x = attribute(text_line(&output, "late x"), "x");
    let low_y = attribute(text_line(&output, "low y"), "y");
    let high_y = attribute(text_line(&output, "high y"), "y");

    assert!(early_x < late_x && low_y > high_y);
}

#[test]
fn horizontal_vertical_and_diagonal_guides_render_closed_styles() {
    let data = line_data();
    let chart = line_chart(&data)
        .x_limits(0.0, 2.0)
        .y_limits(0.0, 4.0)
        .guide(GuideSpec::horizontal(
            2.0,
            style((200, 17, 34), LinePattern::Dashed),
        ))
        .guide(GuideSpec::vertical(
            1.0,
            style((17, 180, 51), LinePattern::Solid),
        ))
        .guide(GuideSpec::segment(
            0.0,
            0.5,
            2.0,
            3.5,
            style((34, 51, 204), LinePattern::Dashed),
        ));

    let output = svg(&chart);

    assert_eq!(
        ["#C81122", "#11B433", "#2233CC"]
            .iter()
            .filter(|color| output.contains(**color))
            .count(),
        3
    );
    assert_eq!(output.matches("stroke-dasharray=\"5 3\"").count(), 2);
}

#[test]
fn separate_dataframe_point_layer_preserves_row_count_radius_and_opacity() {
    let data = line_data();
    let observations = observations();
    let second_observations = df!(
        "second_x" => [0.25_f64, 1.75],
        "second_y" => [1.2_f64, 3.0],
    )
    .unwrap();
    let layer = PointLayer::new(
        &observations,
        "observed_x",
        "observed_y",
        (123, 45, 200),
        5,
        0.35,
    )
    .unwrap();
    let second_layer = PointLayer::new(
        &second_observations,
        "second_x",
        "second_y",
        (123, 45, 200),
        5,
        0.35,
    )
    .unwrap();
    let chart = line_chart(&data)
        .markers(false)
        .point_layer(layer)
        .point_layer(second_layer);

    let output = svg(&chart);
    let points = output
        .lines()
        .filter(|line| {
            line.starts_with("<circle ")
                && line.contains("fill=\"#7B2DC8\"")
                && line.contains("r=\"5\"")
                && line.contains("fill-opacity=\"0.35\"")
        })
        .count();

    assert_eq!(points, observations.height() + second_observations.height());
}

#[test]
fn point_layer_and_line_ribbon_share_one_projection() {
    let data = line_data();
    let observations = df!("raw_x" => [1.0_f64], "raw_y" => [2.0_f64]).unwrap();
    let layer = PointLayer::new(&observations, "raw_x", "raw_y", (123, 45, 200), 4, 0.4).unwrap();
    let output = svg(&line_chart(&data).markers(false).point_layer(layer));
    let point = output
        .lines()
        .find(|line| line.starts_with("<circle ") && line.contains("fill=\"#7B2DC8\""))
        .unwrap();
    let coordinate = format!(
        "{:.3},{:.3}",
        attribute(point, "cx"),
        attribute(point, "cy")
    );
    let line = output
        .lines()
        .find(|line| line.starts_with("<polyline ") && line.contains("stroke-width=\"1.8\""))
        .unwrap();
    let ribbon = output
        .lines()
        .find(|line| line.starts_with("<polygon ") && line.contains("fill-opacity=\"0.18\""))
        .unwrap();
    let x_prefix = format!("{:.3},", attribute(point, "cx"));

    assert!(line.contains(&coordinate) && ribbon.matches(&x_prefix).count() == 2);
}

#[test]
fn out_of_range_ticks_and_guides_are_rejected() {
    let data = line_data();
    let tick_error = line_chart(&data)
        .x_limits(0.0, 2.0)
        .x_ticks(vec![AxisTick::new(3.0, "outside")])
        .preflight()
        .unwrap_err()
        .to_string();
    let guide_error = line_chart(&data)
        .x_limits(0.0, 2.0)
        .y_limits(0.0, 4.0)
        .guide(GuideSpec::segment(
            0.0,
            1.0,
            3.0,
            2.0,
            style((0, 0, 0), LinePattern::Solid),
        ))
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(tick_error.contains("outside resolved range") && guide_error.contains("outside"));
}

#[test]
fn point_layers_reject_incomplete_or_non_finite_rows() {
    let data = line_data();
    let invalid = df!(
        "raw_x" => [Some(0.0_f64), None, Some(2.0)],
        "raw_y" => [1.0_f64, 2.0, f64::NAN],
    )
    .unwrap();
    let layer = PointLayer::new(&invalid, "raw_x", "raw_y", (0, 0, 0), 3, 0.5).unwrap();

    let output = std::env::temp_dir().join(format!(
        "lichtung-invalid-point-layer-{}.svg",
        std::process::id()
    ));
    if output.exists() {
        std::fs::remove_file(&output).unwrap();
    }
    let error = line_chart(&data)
        .point_layer(layer)
        .render_to_svg(&output)
        .unwrap_err()
        .to_string();

    assert!(error.contains("line point layer 0") && error.contains("source row 1"));
    assert!(!output.exists());
}

#[test]
fn supplemental_geometry_rejects_incompatible_kinds_and_facets() {
    let data = line_data();
    let observations = observations();
    let layer =
        PointLayer::new(&observations, "observed_x", "observed_y", (0, 0, 0), 3, 0.5).unwrap();
    let mut scatter = Chart::new(&data).point_layer(layer.clone());
    scatter.x("grid_x").y("estimate").scatter();
    let scatter_error = scatter.preflight().unwrap_err().to_string();

    let faceted_error = line_chart(&data)
        .facet_wrap("panel", 1)
        .point_layer(layer)
        .preflight()
        .unwrap_err()
        .to_string();

    let mut categorical = Chart::new(&data).x_ticks(vec![AxisTick::new(0.5, "bad")]);
    categorical.x("panel").y("estimate").bar();
    let categorical_error = categorical.preflight().unwrap_err().to_string();

    let mut segment_on_bar = Chart::new(&data).guide(GuideSpec::segment(
        0.0,
        1.0,
        1.0,
        2.0,
        style((0, 0, 0), LinePattern::Solid),
    ));
    segment_on_bar.x("panel").y("estimate").bar();
    let segment_error = segment_on_bar.preflight().unwrap_err().to_string();

    assert!(
        scatter_error.contains("only by LineMode::Series")
            && faceted_error.contains("not supported with faceting")
            && categorical_error.contains("numeric x axis")
            && segment_error.contains("requires numeric x and y axes")
    );
}

#[test]
fn configured_numeric_layers_emit_deterministic_bytes() {
    let data = line_data();
    let observations = observations();
    let build = || {
        line_chart(&data)
            .x_ticks(vec![AxisTick::new(0.0, "start"), AxisTick::new(2.0, "end")])
            .guide(GuideSpec::vertical(
                1.0,
                style((200, 17, 34), LinePattern::Dashed),
            ))
            .point_layer(
                PointLayer::new(
                    &observations,
                    "observed_x",
                    "observed_y",
                    (123, 45, 200),
                    3,
                    0.35,
                )
                .unwrap(),
            )
    };

    assert_eq!(svg(&build()), svg(&build()));
}

#[test]
fn unused_numeric_layer_defaults_leave_standard_line_scene_inert() {
    let data = line_data();
    let first = svg(&line_chart(&data));
    let second = svg(&line_chart(&data));

    assert_eq!(first, second);
    assert!(!first.contains("#C81122") && !first.contains("#7B2DC8"));
    assert!(first.contains("fill-opacity=\"0.82\""));
}
