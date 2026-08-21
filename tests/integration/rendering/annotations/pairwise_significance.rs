use super::*;
use crate::composition_support::tempdir as composition_tempdir;

#[test]
fn grouped_and_horizontal_significance_use_typed_pairwise_endpoints_only() {
    let data = df!(
        "category" => ["a", "a", "b", "b"],
        "hue" => ["left", "right", "left", "right"],
        "value" => [2.0_f64, 3.0, 4.0, 5.0]
    )
    .unwrap();
    let comparison = ComparisonAnnotation {
        endpoint_a: ComparisonEndpoint::new("a")
            .with_group("left")
            .with_hue("left"),
        endpoint_b: ComparisonEndpoint::new("a")
            .with_group("right")
            .with_hue("right"),
        ..ComparisonAnnotation::new("a", "a", 0.01)
    };
    let dir = composition_tempdir("grouped-horizontal-significance");
    let vertical_path = dir.join("vertical.svg");
    let horizontal_path = dir.join("horizontal.svg");

    let mut vertical = Chart::new(&data)
        .group("hue")
        .hue("hue")
        .bar_aggregation(BarAggregation::Identity);
    vertical
        .x("category")
        .y("value")
        .add_comparisons([comparison.clone()])
        .bar();
    vertical.render_to_svg(&vertical_path).unwrap();

    let mut horizontal = Chart::new(&data)
        .group("hue")
        .hue("hue")
        .bar_aggregation(BarAggregation::Identity);
    horizontal
        .x("category")
        .y("value")
        .add_comparisons([comparison])
        .horizontal_bar();
    horizontal.render_to_svg(&horizontal_path).unwrap();

    assert_eq!(
        std::fs::read_to_string(vertical_path)
            .unwrap()
            .matches("class=\"significance-bracket\"")
            .count(),
        1
    );
    assert!(std::fs::read_to_string(horizontal_path)
        .unwrap()
        .contains("class=\"significance-bracket horizontal\""));
}
