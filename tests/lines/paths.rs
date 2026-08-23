use super::*;

fn numeric_trajectory_data() -> DataFrame {
    df!(
        "subject" => ["s1", "s2", "s1", "s2"],
        "condition" => [2.0_f64, 2.0, 1.0, 1.0],
        "value" => [20.0_f64, 40.0, 10.0, 30.0],
        "status" => ["case", "control", "control", "case"]
    )
    .unwrap()
}

fn categorical_trajectory_data() -> DataFrame {
    df!(
        "subject" => ["s1", "s1"],
        "condition" => ["before", "after"],
        "value" => [100.0_f64, 200.0]
    )
    .unwrap()
}

fn trajectory(frame: &DataFrame) -> Plot<lichtung_plot::kind::Trajectory> {
    Plot::trajectory(
        data::axis(frame, "condition"),
        data::numeric(frame, "value"),
        data::category(frame, "subject"),
    )
}

fn trajectory_with_hue(frame: &DataFrame) -> Plot<lichtung_plot::kind::Trajectory> {
    trajectory(frame).hue(data::category(frame, "status"))
}

fn svg<K: lichtung_plot::kind::Marker>(plot: Plot<K>, name: &str) -> String {
    render_svg(plot, name).unwrap()
}

fn connector_tags(svg: &str) -> Vec<&str> {
    svg.lines()
        .filter(|line| line.starts_with("<polyline ") && line.contains("stroke-width=\"1.4\""))
        .collect()
}

fn points(tag: &str) -> Vec<(f64, f64)> {
    tag.split_once("points=\"")
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .split_whitespace()
        .map(|point| {
            let (x, y) = point.split_once(',').unwrap();
            (x.parse().unwrap(), y.parse().unwrap())
        })
        .collect()
}

#[test]
fn numeric_trajectory_sorts_within_groups_and_never_connects_groups() {
    let output = svg(
        trajectory_with_hue(&numeric_trajectory_data()),
        "numeric-trajectory",
    );
    let connectors = connector_tags(&output);
    assert_eq!(connectors.len(), 2);
    assert!(connectors.iter().all(|line| {
        let points = points(line);
        points.len() == 2 && points[0].0 < points[1].0
    }));
}

#[test]
fn numeric_path_preserves_source_order_and_repeated_x_values() {
    let frame = df!(
        "curve" => ["roc"; 4], "rate" => [0.0_f64, 0.0, 1.0, 0.5],
        "sensitivity" => [0.0_f64, 0.4, 1.0, 0.8]
    )
    .unwrap();
    let output = svg(
        Plot::line_path(
            data::numeric(&frame, "rate"),
            data::numeric(&frame, "sensitivity"),
            data::category(&frame, "curve"),
        ),
        "numeric-path",
    );
    let connector = points(connector_tags(&output)[0]);
    assert_eq!(connector.len(), 4);
    assert_eq!(connector[0].0.to_bits(), connector[1].0.to_bits());
    assert!(connector[2].0 > connector[3].0);
    assert_eq!(output.matches("<circle ").count(), 4);
}

#[test]
fn changing_hue_colors_points_but_not_connector_or_group_legend() {
    let frame = df!(
        "subject" => ["subject-one"; 3], "condition" => [1.0_f64, 2.0, 3.0],
        "value" => [2.0_f64, 5.0, 3.0], "status" => ["control", "case", "control"]
    )
    .unwrap();
    let output = svg(trajectory_with_hue(&frame), "changing-trajectory-hue");
    let connector = connector_tags(&output)[0];
    assert!(connector.contains("stroke=\"#000000\""));
    let marks = output
        .split_once("<g clip-path=")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;
    let blue = marks.find("fill=\"#0072B2\"").unwrap();
    let orange = marks.find("fill=\"#D55E00\"").unwrap();
    assert!(blue < orange && marks[orange..].contains("fill=\"#0072B2\""));
    assert!(output.contains(">Control</text>") && output.contains(">Case</text>"));
    assert!(!output.contains("subject-one") && !output.contains("Subject one"));
}

#[test]
fn categorical_trajectory_uses_global_first_observed_condition_order() {
    let frame = df!(
        "subject" => ["s1"; 3], "condition" => ["after", "before", "during"],
        "value" => [30.0_f64, 10.0, 20.0]
    )
    .unwrap();
    let output = svg(trajectory(&frame), "categorical-order");
    let connector = points(connector_tags(&output)[0]);
    assert!(connector[0].0 < connector[1].0 && connector[1].0 < connector[2].0);
    assert!(output.find(">After</text>").unwrap() < output.find(">Before</text>").unwrap());
    assert!(output.find(">Before</text>").unwrap() < output.find(">During</text>").unwrap());
}

#[test]
fn categorical_conditions_collapse_whitespace_before_ordering_and_ticks() {
    let frame = df!(
        "subject" => ["s1", "s1", "s2", "s2"],
        "condition" => ["  before\tvisit ", "after visit", "before visit", " after   visit "],
        "value" => [1.0_f64, 2.0, 3.0, 4.0]
    )
    .unwrap();
    let output = svg(trajectory(&frame), "normalized-condition-ticks");
    assert_eq!(output.matches(">Before visit</text>").count(), 1);
    assert_eq!(output.matches(">After visit</text>").count(), 1);
    assert!(
        output.find(">Before visit</text>").unwrap() < output.find(">After visit</text>").unwrap()
    );
}

#[test]
fn categorical_condition_duplicates_and_empty_values_fail_after_normalization() {
    let duplicate = Plot::trajectory(
        lichtung_plot::CategoryValues::new("condition", ["before visit", " before\tvisit "])
            .unwrap(),
        lichtung_plot::NumericValues::new("value", [1.0, 2.0]),
        lichtung_plot::CategoryValues::new("subject", ["s1", "s1"]).unwrap(),
    )
    .preflight()
    .unwrap_err()
    .to_string();
    assert!(duplicate.contains("duplicate normalized") && duplicate.contains("source row 1"));

    let empty = Plot::trajectory(
        lichtung_plot::CategoryValues::new("condition", ["before", " \t\n "]).unwrap(),
        lichtung_plot::NumericValues::new("value", [1.0, 2.0]),
        lichtung_plot::CategoryValues::new("subject", ["s1", "s1"]).unwrap(),
    )
    .preflight()
    .unwrap_err()
    .to_string();
    assert!(
        empty.contains("condition label is empty after whitespace normalization at source row 1")
    );
}

#[test]
fn numeric_looking_string_conditions_remain_categorical_and_first_observed() {
    let frame = df!(
        "subject" => ["s1", "s1"], "condition" => ["10", "2"],
        "value" => [100.0_f64, 200.0]
    )
    .unwrap();
    let output = svg(trajectory(&frame), "numeric-looking-categories");
    assert!(output.find(">10</text>").unwrap() < output.find(">2</text>").unwrap());
}

#[test]
fn faceted_categorical_trajectories_keep_global_order_and_source_rows() {
    let frame = df!(
        "facet" => ["left", "left", "left", "left", "right", "right", "right", "right"],
        "subject" => ["s1", "s1", "s2", "s2", "s1", "s1", "s2", "s2"],
        "condition" => [" after ", "before", "after", " before ", " during ", "before", "during", "\tbefore "],
        "value" => [4.0_f64, 1.0, 5.0, 2.0, 6.0, 3.0, 7.0, 4.0]
    )
    .unwrap();
    let output = svg(
        trajectory(&frame).facet(FacetSpec::column(data::category(&frame, "facet"))),
        "faceted-categorical-trajectories",
    );
    assert_eq!(connector_tags(&output).len(), 4);
    for condition in ["After", "Before", "During"] {
        assert_eq!(output.matches(&format!(">{condition}</text>")).count(), 2);
    }
}

#[test]
fn faceted_hues_use_global_first_observed_colors_and_one_complete_legend() {
    let frame = df!(
        "facet" => ["left", "left", "left", "left", "right", "right", "right", "right"],
        "subject" => ["s1", "s1", "s2", "s2", "s1", "s1", "s2", "s2"],
        "condition" => ["before", "after", "before", "after", "before", "after", "before", "after"],
        "value" => [1.0_f64, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 5.0],
        "status" => ["late", "late", "late", "late", "early", "early", "early", "early"]
    )
    .unwrap();
    let output = svg(
        trajectory_with_hue(&frame).facet(FacetSpec::column(data::category(&frame, "facet"))),
        "faceted-trajectory-hues",
    );
    assert_eq!(output.matches(">Late</text>").count(), 1);
    assert_eq!(output.matches(">Early</text>").count(), 1);
    assert!(output.find(">Late</text>").unwrap() < output.find(">Early</text>").unwrap());
    assert!(output.matches("fill=\"#0072B2\"").count() >= 4);
    assert!(output.matches("fill=\"#D55E00\"").count() >= 4);
}

#[test]
fn facet_local_singletons_and_duplicates_report_before_render() {
    let singleton = df!(
        "facet" => ["left", "left", "right", "right", "right"],
        "subject" => ["s1", "s1", "s1", "s2", "s2"],
        "condition" => ["before", "after", "before", "before", "after"],
        "value" => [1.0_f64, 2.0, 3.0, 4.0, 5.0]
    )
    .unwrap();
    let error = trajectory(&singleton)
        .facet(FacetSpec::column(data::category(&singleton, "facet")))
        .preflight()
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("group \"s1\" requires at least two retained conditions"));

    let duplicate = df!(
        "facet" => ["left", "left", "right", "right"],
        "subject" => ["s1", "s1", "s2", "s2"],
        "condition" => ["a", "b", "a", "a"],
        "value" => [1.0_f64, 2.0, 3.0, 4.0]
    )
    .unwrap();
    let error = trajectory(&duplicate)
        .facet(FacetSpec::column(data::category(&duplicate, "facet")))
        .preflight()
        .unwrap_err();
    assert!(error.to_string().contains("source row 3"));
}

#[test]
fn composed_canvas_renders_trajectory_without_cross_panel_lifecycle() {
    let trajectory_frame = numeric_trajectory_data();
    let canvas = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, trajectory_with_hue(&trajectory_frame))
        .unwrap()
        .panel(
            0,
            1,
            Plot::line(
                lichtung_plot::NumericValues::new("x", [1.0, 2.0]),
                lichtung_plot::NumericValues::new("y", [1.0, 3.0]),
            ),
        )
        .unwrap();
    let output = render_canvas(canvas, "trajectory-canvas").unwrap();
    assert_eq!(output.matches("<svg ").count(), 1);
    assert_eq!(connector_tags(&output).len(), 2);
}

#[test]
fn categorical_trajectory_rejects_numeric_x_axis_overrides_in_plot_and_canvas() {
    let frame = categorical_trajectory_data();
    let configurations = [
        (
            trajectory(&frame).x_limits(0.0, 2.0),
            "categorical condition x does not support numeric x limits",
        ),
        (
            trajectory(&frame).x_tick_count(5),
            "categorical condition x does not support numeric x tick-count overrides",
        ),
    ];
    for (plot, expected) in configurations {
        assert!(plot.preflight().unwrap_err().to_string().contains(expected));
    }
    let canvas_error = Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, trajectory(&frame).x_limits(0.0, 2.0))
        .unwrap()
        .preflight()
        .unwrap_err();
    assert!(canvas_error
        .to_string()
        .contains("categorical condition x does not support numeric x limits"));
}

#[test]
fn numeric_trajectory_preserves_x_limits_and_tick_count_behavior() {
    let output = svg(
        trajectory_with_hue(&numeric_trajectory_data())
            .x_limits(0.5, 2.5)
            .x_tick_count(3),
        "numeric-trajectory-limits",
    );
    assert_eq!(connector_tags(&output).len(), 2);
}

#[test]
fn trajectory_rejects_signed_zero_duplicate() {
    let error = Plot::trajectory(
        lichtung_plot::NumericValues::new("condition", [0.0, -0.0]),
        lichtung_plot::NumericValues::new("value", [1.0, 2.0]),
        lichtung_plot::CategoryValues::new("subject", ["s1", "s1"]).unwrap(),
    )
    .preflight()
    .unwrap_err()
    .to_string();
    assert!(error.contains("duplicate normalized") && error.contains("source row 1"));
}

#[test]
fn trajectory_rejects_null_group_and_nonfinite_conditions_with_source_rows() {
    let null_group = Plot::trajectory(
        lichtung_plot::NumericValues::new("condition", [1.0, 2.0]),
        lichtung_plot::NumericValues::new("value", [1.0, 2.0]),
        lichtung_plot::CategoryValues::nullable("subject", [Some("s1"), None]).unwrap(),
    )
    .preflight()
    .unwrap_err()
    .to_string();
    assert!(null_group.contains("null at source row 1"));

    for value in [f64::NAN, f64::INFINITY] {
        let error = Plot::trajectory(
            lichtung_plot::NumericValues::new("condition", [1.0, value]),
            lichtung_plot::NumericValues::new("value", [1.0, 2.0]),
            lichtung_plot::CategoryValues::new("subject", ["s1", "s1"]).unwrap(),
        )
        .preflight()
        .unwrap_err()
        .to_string();
        assert!(error.contains("source row 1"));
    }
}

#[test]
fn trajectory_requires_two_conditions_per_group() {
    let frame = df!(
        "subject" => ["s1", "s1", "s2"], "condition" => [1.0_f64, 2.0, 1.0],
        "value" => [1.0_f64, 2.0, 3.0]
    )
    .unwrap();
    assert!(trajectory(&frame)
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("group \"s2\" requires at least two retained conditions"));
}

#[test]
fn trajectory_svg_is_deterministic_and_exposes_neutral_connector_and_hue_points() {
    let frame = df!(
        "subject" => ["subject-one", "subject-one"],
        "condition" => ["before", "after"], "value" => [1.0_f64, 3.0],
        "status" => ["control", "case"]
    )
    .unwrap();
    let first = svg(trajectory_with_hue(&frame), "trajectory-first");
    let second = svg(trajectory_with_hue(&frame), "trajectory-second");
    assert_eq!(first, second);
    assert!(first.contains("stroke=\"#000000\" stroke-width=\"1.4\""));
    assert!(first.contains("fill=\"#0072B2\"") && first.contains("fill=\"#D55E00\""));
    assert!(!first.contains("subject-one"));
}
