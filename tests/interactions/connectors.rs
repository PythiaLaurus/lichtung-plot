use super::*;

fn interaction_data() -> DataFrame {
    df!(
        "condition" => ["Before", "After", "Before", "After"],
        "series" => ["Control", "Control", "Active", "Active"],
        "estimate" => [1.0_f64, 1.5, 1.2, 2.0],
        "lower" => [0.8_f64, 1.3, 1.0, 1.8],
        "upper" => [1.2_f64, 1.7, 1.4, 2.2],
        "shape" => [PointShape::Circle.as_str(); 4],
        "pattern" => [LinePattern::Solid.as_str(); 4]
    )
    .unwrap()
}

fn connector_collision_data() -> DataFrame {
    df!(
        "condition" => ["one", "two", "three", "four"],
        "series" => ["A"; 4],
        "estimate" => [0.0_f64, 0.0, 1.0, 0.6],
        "lower" => [-0.01_f64, -0.01, 0.99, 0.59],
        "upper" => [0.01_f64, 0.01, 1.01, 0.61],
        "shape" => [PointShape::Circle.as_str(); 4],
        "pattern" => [LinePattern::Solid.as_str(); 4]
    )
    .unwrap()
}

fn chart(frame: &DataFrame) -> Plot<lichtung_plot::kind::Interaction> {
    Plot::interaction(
        data::category(frame, "condition"),
        data::numeric(frame, "estimate"),
        data::category(frame, "series"),
        Bounds::new(data::numeric(frame, "lower"), data::numeric(frame, "upper")),
    )
    .hue(data::category(frame, "series"))
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
        svg(
            chart(&frame).interaction_connectors(true),
            "connector-explicit",
        )
    );
}

#[test]
fn hidden_interaction_connectors_omit_only_between_category_lines() {
    let frame = interaction_data();
    let visible = svg(chart(&frame), "connectors-visible");
    let hidden = svg(
        chart(&frame).interaction_connectors(false),
        "connectors-hidden",
    );
    let visible_marks = clipped_marks(&visible);
    let hidden_marks = clipped_marks(&hidden);
    assert_eq!(visible_marks.matches("<polyline ").count(), 6);
    assert_eq!(hidden_marks.matches("<polyline ").count(), 4);
    assert_eq!(visible_marks.matches("<circle ").count(), 4);
    assert_eq!(hidden_marks.matches("<circle ").count(), 4);
    assert_eq!(legend(&visible), legend(&hidden));
    assert!(hidden.contains(">Before</text>") && hidden.contains(">After</text>"));
    assert!(hidden.contains(">Control</text>") && hidden.contains(">Active</text>"));
    assert_eq!(without_connector_polylines(&visible), hidden);
}

#[test]
fn hidden_interaction_connectors_reject_line_pattern_styles() {
    let frame = interaction_data();
    let invalid = chart(&frame)
        .interaction_styles(
            data::category(&frame, "shape"),
            data::category(&frame, "pattern"),
        )
        .interaction_connectors(false);
    assert!(invalid
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("incompatible with line-pattern style columns"));
}

#[test]
fn interaction_rejects_hiding_both_markers_and_connectors_preflight() {
    let frame = interaction_data();
    assert!(chart(&frame)
        .markers(false)
        .interaction_connectors(false)
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

    let hidden = svg(
        chart(&frame).interaction_connectors(false),
        "collision-hidden",
    );
    let hidden_without_legend = svg(
        chart(&frame)
            .interaction_connectors(false)
            .legend(LegendPolicy::Hidden),
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
    output
}
