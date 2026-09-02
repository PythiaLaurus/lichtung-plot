use lichtung_plot::{
    CategoryValues, FacetSpec, LegendPolicy, LegendPosition, NumericValues, PanelCorner,
    PanelTextAnnotation, Plot, Theme,
};

use super::render_svg;

#[derive(Clone)]
struct ScatterData {
    x: NumericValues,
    y: NumericValues,
    group: CategoryValues,
}

fn scatter_data() -> ScatterData {
    ScatterData {
        x: NumericValues::new("x", [1.0, 2.0, 3.0]),
        y: NumericValues::new("y", [2.0, 1.0, 3.0]),
        group: CategoryValues::new("group", ["a", "b", "a"]).unwrap(),
    }
}

fn scatter(data: &ScatterData) -> Plot<lichtung_plot::kind::Scatter> {
    Plot::scatter(data.x.clone(), data.y.clone()).size(420, 300)
}

fn svg<K: lichtung_plot::kind::Marker>(chart: &Plot<K>) -> String {
    render_svg(chart.clone(), "panel-text-svg").unwrap()
}

#[test]
fn panel_text_constructor_rejects_empty_multiline_and_untrimmed_text() {
    for text in ["", "   ", " leading", "trailing ", "first\nsecond"] {
        assert!(PanelTextAnnotation::new(PanelCorner::TopLeft, text).is_err());
    }
}

#[test]
fn panel_text_renders_in_named_clipped_layer_with_theme_style_and_corner_anchors() {
    let data = scatter_data();
    let chart = scatter(&data)
        .panel_text_annotation(
            PanelTextAnnotation::new(PanelCorner::TopLeft, "Upper note").unwrap(),
        )
        .panel_text_annotation(
            PanelTextAnnotation::new(PanelCorner::BottomRight, "Lower note").unwrap(),
        )
        .theme(Theme::Nature);
    let output = svg(&chart);
    let layer = output
        .split_once("<g id=\"panel-text\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;

    assert!(layer.contains("clip-path=\"url(#clip-") && layer.contains("fill=\"#282828\""));
    let upper = layer
        .lines()
        .find(|line| line.contains("Upper note"))
        .unwrap();
    let lower = layer
        .lines()
        .find(|line| line.contains("Lower note"))
        .unwrap();
    let coordinate = |line: &str, attribute: &str| {
        line.split_once(&format!("{attribute}=\""))
            .unwrap()
            .1
            .split_once('"')
            .unwrap()
            .0
            .parse::<f64>()
            .unwrap()
    };
    assert!(upper.contains("text-anchor=\"start\""));
    assert!(lower.contains("text-anchor=\"start\""));
    assert!(coordinate(lower, "x") > coordinate(upper, "x"));
    assert!(coordinate(lower, "y") > coordinate(upper, "y"));
}

#[test]
fn panel_text_default_is_byte_inert() {
    let data = scatter_data();
    let first = svg(&scatter(&data));
    let second = svg(&scatter(&data));

    assert_eq!(first, second);
    assert!(!first.contains("panel-text"));
}

#[test]
fn panel_text_rejects_duplicate_corner() {
    let data = scatter_data();
    let chart = scatter(&data)
        .panel_text_annotation(PanelTextAnnotation::new(PanelCorner::TopLeft, "First").unwrap())
        .panel_text_annotation(PanelTextAnnotation::new(PanelCorner::TopLeft, "Second").unwrap());

    assert!(chart
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("duplicate panel text annotation corner"));
}

#[test]
fn panel_text_rejects_facets() {
    let data = scatter_data();
    let annotation = || PanelTextAnnotation::new(PanelCorner::TopLeft, "Note").unwrap();
    let faceted = scatter(&data)
        .panel_text_annotation(annotation())
        .facet(FacetSpec::row(data.group.clone()));
    assert!(faceted
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("faceting"));
}

#[test]
fn panel_text_rejects_colliding_inside_legend() {
    let data = scatter_data();
    let chart = scatter(&data)
        .hue(data.group.clone())
        .panel_text_annotation(
            PanelTextAnnotation::new(PanelCorner::TopRight, "Caller note").unwrap(),
        )
        .legend(LegendPolicy::Visible(LegendPosition::InsideTopRight));

    assert!(chart
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("incompatible with the resolved inside legend"));
}

#[test]
fn automatic_legend_falls_back_around_panel_text() {
    let data = scatter_data();
    let chart = scatter(&data)
        .hue(data.group.clone())
        .panel_text_annotation(
            PanelTextAnnotation::new(PanelCorner::TopRight, "Caller note").unwrap(),
        );

    let output = svg(&chart);

    assert!(output.contains(">Caller note</text>"));
    assert!(output.contains("<g id=\"legend\">"));
}

#[test]
fn separately_valid_panel_annotations_reject_pairwise_overlap_before_begin() {
    let data = scatter_data();
    let chart = scatter(&data)
        .panel_text_annotation(
            PanelTextAnnotation::new(
                PanelCorner::TopLeft,
                "Left annotation extends far into this panel",
            )
            .unwrap(),
        )
        .panel_text_annotation(
            PanelTextAnnotation::new(
                PanelCorner::TopRight,
                "Right annotation extends far into this panel",
            )
            .unwrap(),
        )
        .size(460, 300);

    let error = chart.preflight().unwrap_err().to_string();
    assert!(
        error.contains("panel text annotations at TopLeft and TopRight overlap"),
        "unexpected preflight error: {error}"
    );
    let directory = super::tempdir("panel-text-overlap");
    let output = directory.join("plot.svg");
    assert!(chart.svg(&output).is_err());
    assert!(!output.exists());
    std::fs::remove_dir_all(directory).unwrap();
}
