use super::*;

struct ConnectorData {
    condition: CategoryValues,
    series: CategoryValues,
    estimate: NumericValues,
    lower: NumericValues,
    upper: NumericValues,
}

fn interaction_data() -> ConnectorData {
    ConnectorData {
        condition: CategoryValues::new("condition", ["Before", "After", "Before", "After"])
            .unwrap(),
        series: CategoryValues::new("series", ["Control", "Control", "Active", "Active"]).unwrap(),
        estimate: NumericValues::new("estimate", [1.0, 1.5, 1.2, 2.0]),
        lower: NumericValues::new("lower", [0.8, 1.3, 1.0, 1.8]),
        upper: NumericValues::new("upper", [1.2, 1.7, 1.4, 2.2]),
    }
}

fn connector_collision_data() -> ConnectorData {
    ConnectorData {
        condition: CategoryValues::new("condition", ["one", "two", "three", "four"]).unwrap(),
        series: CategoryValues::new("series", ["A"; 4]).unwrap(),
        estimate: NumericValues::new("estimate", [0.0, 0.0, 1.0, 0.6]),
        lower: NumericValues::new("lower", [-0.01, -0.01, 0.99, 0.59]),
        upper: NumericValues::new("upper", [0.01, 0.01, 1.01, 0.61]),
    }
}

fn chart(data: &ConnectorData) -> Plot<lichtung_plot::kind::Interaction> {
    Plot::interaction(
        data.condition.clone(),
        data.estimate.clone(),
        data.series.clone(),
        Bounds::new(data.lower.clone(), data.upper.clone()),
    )
    .hue(data.series.clone())
    .size(480, 320)
}

fn svg<K: lichtung_plot::kind::Marker>(plot: Plot<K>, name: &str) -> String {
    render_svg(plot, name).unwrap()
}

#[test]
fn explicit_visible_interaction_connectors_are_byte_identical_to_default() {
    let frame = interaction_data();
    assert_eq!(
        svg(chart(&frame), "connector-default"),
        svg(chart(&frame).connectors(true), "connector-explicit")
    );
}

#[test]
fn hidden_interaction_connectors_update_marks_and_exact_legend() {
    let frame = interaction_data();
    let visible = svg(chart(&frame), "connectors-visible");
    let hidden = svg(chart(&frame).connectors(false), "connectors-hidden");
    let visible_marks = clipped_marks(&visible);
    let hidden_marks = clipped_marks(&hidden);
    assert_eq!(visible_marks.matches("<polyline ").count(), 6);
    assert_eq!(hidden_marks.matches("<polyline ").count(), 4);
    assert_eq!(visible_marks.matches("<circle ").count(), 4);
    assert_eq!(hidden_marks.matches("<circle ").count(), 4);
    assert_eq!(legend(&visible).matches("<polyline ").count(), 2);
    assert_eq!(legend(&hidden).matches("<polyline ").count(), 0);
    assert_eq!(legend(&visible).matches("<circle ").count(), 2);
    assert_eq!(legend(&hidden).matches("<circle ").count(), 2);
    assert!(hidden.contains(">Before</text>") && hidden.contains(">After</text>"));
    assert!(hidden.contains(">Control</text>") && hidden.contains(">Active</text>"));
    assert_eq!(without_connector_polylines(&visible), hidden);
}

#[test]
fn hidden_interaction_connectors_reject_configured_styles() {
    let frame = interaction_data();
    let invalid = chart(&frame)
        .styles([
            InteractionStyle::new("Control"),
            InteractionStyle::new("Active"),
        ])
        .connectors(false);
    assert!(invalid
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("incompatible with configured styles"));
}

#[test]
fn interaction_rejects_hiding_both_markers_and_connectors_preflight() {
    let frame = interaction_data();
    assert!(chart(&frame)
        .markers(false)
        .connectors(false)
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("markers and connectors cannot both be hidden"));
}

#[test]
fn hidden_connectors_do_not_participate_in_legend_range_planning() {
    let frame = connector_collision_data();
    let visible = svg(chart(&frame), "collision-visible");
    let visible_without_legend = svg(
        chart(&frame).legend(LegendPolicy::Hidden),
        "collision-visible-no-legend",
    );
    assert_ne!(
        clipped_marks(&visible),
        clipped_marks(&visible_without_legend)
    );

    let hidden = svg(chart(&frame).connectors(false), "collision-hidden");
    let hidden_without_legend = svg(
        chart(&frame).connectors(false).legend(LegendPolicy::Hidden),
        "collision-hidden-no-legend",
    );
    assert_eq!(
        clipped_marks(&hidden),
        clipped_marks(&hidden_without_legend)
    );
}

fn clipped_marks(svg: &str) -> &str {
    svg.split_once("<g clip-path=\"url(#clip-0)\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0
}

fn legend(svg: &str) -> &str {
    svg.split_once("<g id=\"legend\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0
}

fn without_connector_polylines(svg: &str) -> String {
    let mut output = svg.to_owned();
    for line in clipped_marks(svg).lines().filter(|line| {
        if !line.starts_with("<polyline points=\"") {
            return false;
        }
        let points = line
            .split_once("points=\"")
            .unwrap()
            .1
            .split_once('"')
            .unwrap()
            .0
            .split_whitespace()
            .map(|point| point.split_once(',').unwrap().0)
            .collect::<Vec<_>>();
        points.windows(2).any(|pair| pair[0] != pair[1])
    }) {
        output = output.replacen(&format!("{line}\n"), "", 1);
    }
    for line in legend(svg)
        .lines()
        .filter(|line| line.starts_with("<polyline "))
    {
        output = output.replacen(&format!("{line}\n"), "", 1);
    }
    output
}
