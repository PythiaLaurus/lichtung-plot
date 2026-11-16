use super::*;

struct TrajectoryData {
    condition: AxisValues,
    value: NumericValues,
    subject: CategoryValues,
    status: Option<CategoryValues>,
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
