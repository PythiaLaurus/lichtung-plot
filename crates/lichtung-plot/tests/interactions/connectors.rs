use super::*;

const ESTIMATES: [f64; 4] = [1.0, 1.5, 1.2, 2.0];
const LOWERS: [f64; 4] = [0.8, 1.3, 1.0, 1.8];
const UPPERS: [f64; 4] = [1.2, 1.7, 1.4, 2.2];

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
        estimate: NumericValues::new("estimate", ESTIMATES),
        lower: NumericValues::new("lower", LOWERS),
        upper: NumericValues::new("upper", UPPERS),
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
    assert_interaction_marks(&visible, true);
    assert_interaction_marks(&hidden, false);
    let (visible_panel, visible_guide) = assert_interaction_guide(&visible, true);
    let (hidden_panel, hidden_guide) = assert_interaction_guide(&hidden, false);
    for index in [0, 1, 3] {
        assert_eq!(
            visible_panel[index].to_bits(),
            hidden_panel[index].to_bits()
        );
    }
    // A round-capped 1.8px connector extends beyond both ends of the nominal
    // 11px handle. Point-only keys do not consume those outward ink quanta.
    let connector = legend(&visible)
        .lines()
        .find(|line| line.starts_with("<polyline "))
        .unwrap();
    let stroke: f64 = attribute(connector, "stroke-width").parse().unwrap();
    let outward = 2.0 * (stroke * 1024.0 / 2.0).ceil() / 1024.0;
    assert_eq!(
        (hidden_panel[2] - visible_panel[2]).to_bits(),
        outward.to_bits()
    );
    assert_eq!(
        (visible_guide[2] - hidden_guide[2]).to_bits(),
        outward.to_bits()
    );
    assert_eq!(
        (visible_guide[0] + visible_guide[2]).to_bits(),
        (hidden_guide[0] + hidden_guide[2]).to_bits()
    );
}

#[test]
fn hidden_interaction_connectors_reject_configured_styles() {
    let frame = interaction_data();
    let invalid = chart(&frame)
        .styles([
            InteractionStyle::new("Control").unwrap(),
            InteractionStyle::new("Active").unwrap(),
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
    let inside = || {
        chart(&frame).legend(LegendPolicy::Visible(
            lichtung_plot::LegendPosition::InsideTopRight,
        ))
    };
    let visible = svg(inside(), "collision-visible");
    let visible_without_legend = svg(
        chart(&frame).legend(LegendPolicy::Hidden),
        "collision-visible-no-legend",
    );
    assert_ne!(
        clipped_marks(&visible),
        clipped_marks(&visible_without_legend)
    );

    let hidden = svg(inside().connectors(false), "collision-hidden");
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

fn number(node: roxmltree::Node<'_, '_>, name: &str) -> f64 {
    node.attribute(name).unwrap().parse().unwrap()
}

fn points(node: roxmltree::Node<'_, '_>) -> Vec<(f64, f64)> {
    node.attribute("points")
        .unwrap()
        .split_whitespace()
        .map(|point| {
            let (x, y) = point.split_once(',').unwrap();
            (x.parse().unwrap(), y.parse().unwrap())
        })
        .collect()
}

fn panel(document: &roxmltree::Document<'_>) -> [f64; 4] {
    let clip = document
        .descendants()
        .find(|node| node.attribute("id") == Some("clip-0"))
        .unwrap();
    let rect = clip
        .children()
        .find(|node| node.has_tag_name("rect"))
        .unwrap();
    ["x", "y", "width", "height"].map(|name| number(rect, name))
}

#[expect(
    clippy::too_many_lines,
    reason = "one SVG oracle retains trace order, all source estimates/bounds/styles and connector endpoints"
)]
fn assert_interaction_marks(svg: &str, connectors: bool) {
    let document = roxmltree::Document::parse(svg).unwrap();
    let [left, _, width, _] = panel(&document);
    let marks = document
        .descendants()
        .find(|node| node.attribute("clip-path") == Some("url(#clip-0)"))
        .unwrap();
    let children = marks
        .children()
        .filter(roxmltree::Node::is_element)
        .collect::<Vec<_>>();
    let tags = children
        .iter()
        .map(|node| node.tag_name().name())
        .collect::<Vec<_>>();
    let trace = if connectors {
        vec!["polyline", "polyline", "polyline", "circle", "circle"]
    } else {
        vec!["polyline", "polyline", "circle", "circle"]
    };
    assert_eq!(
        tags,
        trace.repeat(2),
        "trace/source order and connector-only omission"
    );
    let markers = children
        .iter()
        .filter(|node| node.has_tag_name("circle"))
        .collect::<Vec<_>>();
    let intervals = children
        .iter()
        .filter(|node| {
            node.has_tag_name("polyline") && {
                let line = points(**node);
                line[0].0.to_bits() == line[1].0.to_bits()
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(markers.len(), 4);
    assert_eq!(intervals.len(), 4);
    let axes = document
        .descendants()
        .find(|node| node.attribute("id") == Some("axes"))
        .unwrap();
    let y_tick = |label| {
        let text = axes
            .children()
            .find(|node| node.has_tag_name("text") && node.text() == Some(label))
            .unwrap();
        let text_y = number(text, "y");
        axes.children()
            .filter(|node| node.has_tag_name("polyline"))
            .map(points)
            .filter(|line| {
                line.len() == 2
                    && line[0].0 < left
                    && line[1].0 < left
                    && line[0].1.to_bits() == line[1].1.to_bits()
            })
            .min_by(|a, b| (a[0].1 - text_y).abs().total_cmp(&(b[0].1 - text_y).abs()))
            .unwrap()[0]
            .1
    };
    let (one, two) = (y_tick("1"), y_tick("2"));
    assert!(two < one);
    let y = |value| one + (value - 1.0) * (two - one);
    for (index, (marker, interval)) in markers.iter().zip(&intervals).enumerate() {
        let condition = [0.25, 0.75, 0.25, 0.75][index];
        let expected_x = left + width * condition;
        let color = ["#0072B2", "#0072B2", "#D55E00", "#D55E00"][index];
        assert!((number(**marker, "cx") - expected_x).abs() <= 1.0 / 1024.0);
        assert!((number(**marker, "cy") - y(ESTIMATES[index])).abs() <= 2.0 / 1024.0);
        assert_eq!(marker.attribute("fill"), Some(color));
        assert_eq!(marker.attribute("stroke"), Some("#FFFFFF"));
        assert_eq!(number(**marker, "r").to_bits(), 4.0_f64.to_bits());
        assert_eq!(
            number(**marker, "fill-opacity").to_bits(),
            (209.0_f64 / 255.0).to_bits()
        );
        assert_eq!(
            number(**marker, "stroke-width").to_bits(),
            (410.0_f64 / 1024.0).to_bits()
        );
        let ends = points(**interval);
        for (actual, value) in ends.iter().zip([LOWERS[index], UPPERS[index]]) {
            assert!((actual.0 - expected_x).abs() <= 1.0 / 1024.0);
            assert!((actual.1 - y(value)).abs() <= 2.0 / 1024.0);
        }
        assert_eq!(interval.attribute("stroke"), Some(color));
        assert_eq!(
            number(**interval, "stroke-width").to_bits(),
            (1229.0_f64 / 1024.0).to_bits()
        );
    }
    if connectors {
        let lines = children
            .iter()
            .filter(|node| {
                node.has_tag_name("polyline") && {
                    let line = points(**node);
                    line[0].0.to_bits() != line[1].0.to_bits()
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        for (trace, line) in lines.iter().enumerate() {
            assert_eq!(
                line.attribute("stroke"),
                Some(["#0072B2", "#D55E00"][trace])
            );
            assert_eq!(
                number(**line, "stroke-width").to_bits(),
                (1843.0_f64 / 1024.0).to_bits()
            );
            for (offset, point) in points(**line).iter().enumerate() {
                let marker = markers[2 * trace + offset];
                assert_eq!(point.0.to_bits(), number(*marker, "cx").to_bits());
                assert_eq!(point.1.to_bits(), number(*marker, "cy").to_bits());
            }
        }
    }
}

fn assert_interaction_guide(svg: &str, connectors: bool) -> ([f64; 4], [f64; 4]) {
    let document = roxmltree::Document::parse(svg).unwrap();
    let panel = panel(&document);
    let legend = document
        .descendants()
        .find(|node| node.attribute("id") == Some("legend"))
        .unwrap();
    assert!(legend
        .ancestors()
        .all(|node| node.attribute("clip-path").is_none()));
    let rect = legend
        .children()
        .find(|node| node.has_tag_name("rect"))
        .unwrap();
    let frame = ["x", "y", "width", "height"].map(|name| number(rect, name));
    assert!(frame[0] > panel[0] + panel[2]);
    let labels = legend
        .children()
        .filter(|node| node.has_tag_name("text"))
        .filter_map(|node| node.text())
        .collect::<Vec<_>>();
    assert_eq!(labels, ["Series", "Control", "Active"]);
    let markers = legend
        .children()
        .filter(|node| node.has_tag_name("circle"))
        .collect::<Vec<_>>();
    assert_eq!(markers.len(), 2);
    for (index, marker) in markers.iter().enumerate() {
        assert_eq!(
            marker.attribute("fill"),
            Some(["#0072B2", "#D55E00"][index])
        );
        let radius = number(*marker, "r") + number(*marker, "stroke-width") / 2.0;
        assert!(
            number(*marker, "cx") - radius >= frame[0]
                && number(*marker, "cx") + radius <= frame[0] + frame[2]
        );
        assert!(
            number(*marker, "cy") - radius >= frame[1]
                && number(*marker, "cy") + radius <= frame[1] + frame[3]
        );
    }
    let lines = legend
        .children()
        .filter(|node| node.has_tag_name("polyline"))
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), if connectors { 2 } else { 0 });
    for (index, line) in lines.iter().enumerate() {
        assert_eq!(line.attribute("stroke"), markers[index].attribute("fill"));
        let ends = points(*line);
        let half = number(*line, "stroke-width") / 2.0;
        assert!(ends[0].0 - half >= frame[0] && ends[1].0 + half <= frame[0] + frame[2]);
        assert_eq!(ends[0].1.to_bits(), number(markers[index], "cy").to_bits());
        assert_eq!(ends[1].1.to_bits(), number(markers[index], "cy").to_bits());
    }
    (panel, frame)
}
