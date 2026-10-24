use super::*;

struct TrajectoryData {
    condition: AxisValues,
    value: NumericValues,
    subject: CategoryValues,
    status: Option<CategoryValues>,
    facet: Option<CategoryValues>,
}

fn trajectory_data(
    condition: AxisValues,
    value: NumericValues,
    subject: CategoryValues,
) -> TrajectoryData {
    TrajectoryData {
        condition,
        value,
        subject,
        status: None,
        facet: None,
    }
}

fn numeric_trajectory_data() -> TrajectoryData {
    let mut data = trajectory_data(
        NumericValues::new("condition", [2.0, 2.0, 1.0, 1.0]).into(),
        NumericValues::new("value", [20.0, 40.0, 10.0, 30.0]),
        CategoryValues::new("subject", ["s1", "s2", "s1", "s2"]).unwrap(),
    );
    data.status =
        Some(CategoryValues::new("status", ["case", "control", "control", "case"]).unwrap());
    data
}

fn categorical_trajectory_data() -> TrajectoryData {
    trajectory_data(
        CategoryValues::new("condition", ["before", "after"])
            .unwrap()
            .into(),
        NumericValues::new("value", [100.0, 200.0]),
        CategoryValues::new("subject", ["s1", "s1"]).unwrap(),
    )
}

fn trajectory(data: &TrajectoryData) -> Plot<lichtung_plot::kind::Trajectory> {
    Plot::trajectory(
        data.condition.clone(),
        data.value.clone(),
        data.subject.clone(),
    )
}

fn trajectory_with_hue(data: &TrajectoryData) -> Plot<lichtung_plot::kind::Trajectory> {
    trajectory(data).hue(data.status.clone().unwrap())
}

fn svg<K: lichtung_plot::kind::Marker>(plot: Plot<K>, name: &str) -> String {
    render_svg(plot, name).unwrap()
}

fn connector_tags(svg: &str) -> Vec<&str> {
    svg.lines()
        .filter(|line| {
            line.starts_with("<polyline ") && line.contains("stroke-width=\"1.400390625\"")
        })
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
    let output = svg(
        Plot::line_path(
            NumericValues::new("rate", [0.0, 0.0, 1.0, 0.5]),
            NumericValues::new("sensitivity", [0.0, 0.4, 1.0, 0.8]),
            CategoryValues::new("curve", ["roc"; 4]).unwrap(),
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
fn neutral_path_with_hidden_markers_suppresses_hue_legend() {
    let data = numeric_trajectory_data();
    let output = svg(
        Plot::line_path(
            NumericValues::new("condition", [2.0, 2.0, 1.0, 1.0]),
            data.value.clone(),
            data.subject.clone(),
        )
        .hue(data.status.clone().unwrap())
        .markers(false),
        "neutral-path-hidden-markers",
    );

    assert_eq!(connector_tags(&output).len(), 2);
    assert_eq!(output.matches("<circle ").count(), 0);
    assert!(!output.contains("id=\"legend\""));
}

#[test]
fn changing_hue_colors_points_but_not_connector_or_group_legend() {
    let mut data = trajectory_data(
        NumericValues::new("condition", [1.0, 2.0, 3.0]).into(),
        NumericValues::new("value", [2.0, 5.0, 3.0]),
        CategoryValues::new("subject", ["subject-one"; 3]).unwrap(),
    );
    data.status = Some(CategoryValues::new("status", ["control", "case", "control"]).unwrap());
    let output = svg(trajectory_with_hue(&data), "changing-trajectory-hue");
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
fn categorical_conditions_collapse_whitespace_before_ordering_and_ticks() {
    let data = trajectory_data(
        CategoryValues::new(
            "condition",
            [
                "  before\tvisit ",
                "after visit",
                "before visit",
                " after   visit ",
            ],
        )
        .unwrap()
        .into(),
        NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]),
        CategoryValues::new("subject", ["s1", "s1", "s2", "s2"]).unwrap(),
    );
    let output = svg(trajectory(&data), "normalized-condition-ticks");
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
fn faceted_categorical_trajectories_keep_global_order_and_source_rows() {
    let mut data = trajectory_data(
        CategoryValues::new(
            "condition",
            [
                " after ",
                "before",
                "after",
                " before ",
                " during ",
                "before",
                "during",
                "\tbefore ",
            ],
        )
        .unwrap()
        .into(),
        NumericValues::new("value", [4.0, 1.0, 5.0, 2.0, 6.0, 3.0, 7.0, 4.0]),
        CategoryValues::new("subject", ["s1", "s1", "s2", "s2", "s1", "s1", "s2", "s2"]).unwrap(),
    );
    data.facet = Some(
        CategoryValues::new(
            "facet",
            [
                "left", "left", "left", "left", "right", "right", "right", "right",
            ],
        )
        .unwrap(),
    );
    let output = svg(
        trajectory(&data).facet(FacetSpec::column(data.facet.clone().unwrap())),
        "faceted-categorical-trajectories",
    );
    assert_eq!(connector_tags(&output).len(), 4);
    for condition in ["After", "Before", "During"] {
        assert_eq!(output.matches(&format!(">{condition}</text>")).count(), 2);
    }
}

#[test]
fn facet_local_singletons_and_duplicates_report_before_render() {
    let mut singleton = trajectory_data(
        CategoryValues::new(
            "condition",
            ["before", "after", "before", "before", "after"],
        )
        .unwrap()
        .into(),
        NumericValues::new("value", [1.0, 2.0, 3.0, 4.0, 5.0]),
        CategoryValues::new("subject", ["s1", "s1", "s1", "s2", "s2"]).unwrap(),
    );
    singleton.facet =
        Some(CategoryValues::new("facet", ["left", "left", "right", "right", "right"]).unwrap());
    let error = trajectory(&singleton)
        .facet(FacetSpec::column(singleton.facet.clone().unwrap()))
        .preflight()
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("group \"s1\" requires at least two retained conditions"));

    let mut duplicate = trajectory_data(
        CategoryValues::new("condition", ["a", "b", "a", "a"])
            .unwrap()
            .into(),
        NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]),
        CategoryValues::new("subject", ["s1", "s1", "s2", "s2"]).unwrap(),
    );
    duplicate.facet =
        Some(CategoryValues::new("facet", ["left", "left", "right", "right"]).unwrap());
    let error = trajectory(&duplicate)
        .facet(FacetSpec::column(duplicate.facet.clone().unwrap()))
        .preflight()
        .unwrap_err();
    assert!(error.to_string().contains("source row 3"));
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
            trajectory(&frame).x_ticks(vec![AxisTick::new(0.0, "before")]),
            "explicit numeric x ticks require a numeric x axis",
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
    let data = trajectory_data(
        NumericValues::new("condition", [1.0, 2.0, 1.0]).into(),
        NumericValues::new("value", [1.0, 2.0, 3.0]),
        CategoryValues::new("subject", ["s1", "s1", "s2"]).unwrap(),
    );
    assert!(trajectory(&data)
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("group \"s2\" requires at least two retained conditions"));
}
