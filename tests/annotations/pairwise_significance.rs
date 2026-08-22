use super::*;

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
    let dir = tempdir("grouped-horizontal-significance");
    let vertical_path = dir.join("vertical.svg");
    let horizontal_path = dir.join("horizontal.svg");

    Plot::bar(
        data::category(&data, "category"),
        data::numeric(&data, "value"),
    )
    .group(data::category(&data, "hue"))
    .hue(data::category(&data, "hue"))
    .bar_aggregation(BarAggregation::Identity)
    .add_comparisons([comparison.clone()])
    .svg(&vertical_path)
    .unwrap();

    Plot::horizontal_bar(
        data::category(&data, "category"),
        data::numeric(&data, "value"),
    )
    .group(data::category(&data, "hue"))
    .hue(data::category(&data, "hue"))
    .bar_aggregation(BarAggregation::Identity)
    .add_comparisons([comparison])
    .svg(&horizontal_path)
    .unwrap();

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
