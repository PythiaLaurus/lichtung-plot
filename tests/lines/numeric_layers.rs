use super::*;

fn line_data() -> DataFrame {
    df!(
        "grid_x" => [0.0_f64, 1.0, 2.0],
        "estimate" => [1.0_f64, 2.0, 3.0],
        "lower" => [0.5_f64, 1.5, 2.5],
        "upper" => [1.5_f64, 2.5, 3.5],
        "panel" => ["a", "a", "a"]
    )
    .unwrap()
}

fn observations() -> DataFrame {
    df!(
        "observed_x" => [0.0_f64, 0.5, 1.0, 1.5, 2.0],
        "observed_y" => [0.8_f64, 1.7, 2.0, 2.8, 3.2]
    )
    .unwrap()
}

fn line_plot(frame: &DataFrame) -> Plot<lichtung_plot::kind::Line> {
    Plot::line(
        data::numeric(frame, "grid_x"),
        data::numeric(frame, "estimate"),
    )
    .band(Bounds::new(
        data::numeric(frame, "lower"),
        data::numeric(frame, "upper"),
    ))
}

fn svg<K: lichtung_plot::kind::Marker>(plot: Plot<K>) -> String {
    render_svg(plot, "numeric-layers").unwrap()
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

fn point_layer(frame: &DataFrame, x: &str, y: &str, radius: u32, opacity: f64) -> PointLayer {
    PointLayer::new(
        data::numeric(frame, x),
        data::numeric(frame, y),
        (123, 45, 200),
        radius,
        opacity,
    )
    .unwrap()
}

#[test]
fn explicit_numeric_ticks_render_verbatim_text_at_caller_positions_on_both_axes() {
    let frame = line_data();
    let output = svg(line_plot(&frame)
        .x_limits(0.0, 2.0)
        .y_limits(0.0, 4.0)
        .x_ticks(vec![
            AxisTick::new(0.5, "early x"),
            AxisTick::new(1.5, "late x"),
        ])
        .y_ticks(vec![
            AxisTick::new(1.0, "low y"),
            AxisTick::new(3.0, "high y"),
        ]));
    assert!(
        attribute(text_line(&output, "early x"), "x")
            < attribute(text_line(&output, "late x"), "x")
    );
    assert!(
        attribute(text_line(&output, "low y"), "y") > attribute(text_line(&output, "high y"), "y")
    );
}

#[test]
fn horizontal_vertical_and_diagonal_guides_render_closed_styles() {
    let frame = line_data();
    let output = svg(line_plot(&frame)
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
        )));
    assert!(["#C81122", "#11B433", "#2233CC"]
        .iter()
        .all(|color| output.contains(color)));
    assert_eq!(output.matches("stroke-dasharray=\"5 3\"").count(), 2);
}

#[test]
fn separate_point_layers_preserve_row_count_radius_and_opacity() {
    let frame = line_data();
    let first = observations();
    let second = df!("second_x" => [0.25_f64, 1.75], "second_y" => [1.2_f64, 3.0]).unwrap();
    let output = svg(line_plot(&frame)
        .markers(false)
        .point_layer(point_layer(&first, "observed_x", "observed_y", 5, 0.35))
        .point_layer(point_layer(&second, "second_x", "second_y", 5, 0.35)));
    let count = output
        .lines()
        .filter(|line| {
            line.starts_with("<circle ")
                && line.contains("fill=\"#7B2DC8\"")
                && line.contains("r=\"5\"")
                && line.contains("fill-opacity=\"0.35\"")
        })
        .count();
    assert_eq!(count, 7);
}

#[test]
fn point_layer_and_line_ribbon_share_one_projection() {
    let frame = line_data();
    let raw = df!("raw_x" => [1.0_f64], "raw_y" => [2.0_f64]).unwrap();
    let output = svg(line_plot(&frame)
        .markers(false)
        .point_layer(point_layer(&raw, "raw_x", "raw_y", 4, 0.4)));
    let point = output
        .lines()
        .find(|line| line.starts_with("<circle ") && line.contains("#7B2DC8"))
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
    let frame = line_data();
    let tick = line_plot(&frame)
        .x_limits(0.0, 2.0)
        .x_ticks(vec![AxisTick::new(3.0, "outside")])
        .preflight()
        .unwrap_err()
        .to_string();
    let guide = line_plot(&frame)
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
    assert!(tick.contains("outside resolved range") && guide.contains("outside"));
}

#[test]
fn point_layers_reject_incomplete_or_non_finite_rows() {
    let frame = line_data();
    let invalid = PointLayer::new(
        lichtung_plot::NumericValues::nullable("raw_x", [Some(0.0), None, Some(2.0)]),
        lichtung_plot::NumericValues::new("raw_y", [1.0, 2.0, f64::NAN]),
        (0, 0, 0),
        3,
        0.5,
    )
    .unwrap();
    let error = line_plot(&frame)
        .point_layer(invalid)
        .preflight()
        .unwrap_err()
        .to_string();
    assert!(error.contains("line point layer 0") && error.contains("source row 1"));
}

#[test]
fn supplemental_geometry_rejects_facets_and_incompatible_guides() {
    let frame = line_data();
    let observed = observations();
    let layer = point_layer(&observed, "observed_x", "observed_y", 3, 0.5);
    let faceted = line_plot(&frame)
        .facet(FacetSpec::column(data::category(&frame, "panel")))
        .point_layer(layer)
        .preflight()
        .unwrap_err()
        .to_string();
    let segment = Plot::bar(
        data::category(&frame, "panel"),
        lichtung_plot::BarInput::samples(data::numeric(&frame, "estimate")),
    )
    .guide(GuideSpec::segment(
        0.0,
        1.0,
        1.0,
        2.0,
        style((0, 0, 0), LinePattern::Solid),
    ))
    .preflight()
    .unwrap_err()
    .to_string();
    assert!(faceted.contains("not supported with faceting"));
    assert!(segment.contains("requires numeric x and y axes"));
}

#[test]
fn configured_numeric_layers_emit_deterministic_bytes() {
    let build = || {
        let frame = line_data();
        let observed = observations();
        line_plot(&frame)
            .x_ticks(vec![AxisTick::new(0.0, "start"), AxisTick::new(2.0, "end")])
            .guide(GuideSpec::vertical(
                1.0,
                style((200, 17, 34), LinePattern::Dashed),
            ))
            .point_layer(point_layer(&observed, "observed_x", "observed_y", 3, 0.35))
    };
    assert_eq!(svg(build()), svg(build()));
}

#[test]
fn unused_numeric_layer_defaults_leave_standard_line_scene_inert() {
    let first = svg(line_plot(&line_data()));
    let second = svg(line_plot(&line_data()));
    assert_eq!(first, second);
    assert!(!first.contains("#C81122") && !first.contains("#7B2DC8"));
    assert!(first.contains("fill-opacity=\"0.82\""));
}
