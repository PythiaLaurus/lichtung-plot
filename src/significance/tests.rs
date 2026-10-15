use super::*;

fn categories() -> Vec<String> {
    ["a", "b", "c", "d"].map(str::to_string).to_vec()
}

fn panel() -> PanelBox {
    PanelBox::new(50.0, 350.0, 40.0, 240.0)
}

fn comparison(endpoint_a: &str, endpoint_b: &str, p_value: f64) -> SignificanceAnnotation {
    Significance::between(endpoint_a, endpoint_b, SignificanceMark::PValue(p_value))
        .into_annotation()
}

#[test]
fn planner_rejects_unsafe_inside_legend_and_categorical_x_limits() {
    let comparisons = [comparison("a", "b", 0.01)];
    let legend_error = plan_comparisons(
        &comparisons,
        &categories(),
        &[1.0, 2.0, 3.0, 4.0],
        (0.0, 10.0),
        None,
        None,
        panel(),
        LegendAnnotationConstraint::UnsupportedInside,
        SignificanceMarkStyle::default(),
    )
    .unwrap_err();
    let x_limits_error = plan_comparisons(
        &comparisons,
        &categories(),
        &[1.0, 2.0, 3.0, 4.0],
        (0.0, 10.0),
        Some((0.0, 4.0)),
        None,
        panel(),
        LegendAnnotationConstraint::NoObstacle,
        SignificanceMarkStyle::default(),
    )
    .unwrap_err();

    assert!(legend_error
        .to_string()
        .contains("top-aligned, outside, or hidden"));
    assert!(x_limits_error.to_string().contains("categorical x_limits"));
}

#[test]
fn planner_top_ceiling_clears_global_uncompared_maximum() {
    let comparisons = [comparison("a", "b", 0.01)];
    let maxima = [1.0, 2.0, 100.0, 3.0];
    let auto = plan_comparisons(
        &comparisons,
        &categories(),
        &maxima,
        (0.0, 110.0),
        None,
        None,
        panel(),
        LegendAnnotationConstraint::TopCeiling(180.0),
        SignificanceMarkStyle::default(),
    )
    .unwrap();
    let explicit_error = plan_comparisons(
        &comparisons,
        &categories(),
        &maxima,
        (0.0, 110.0),
        None,
        Some((0.0, 110.0)),
        panel(),
        LegendAnnotationConstraint::TopCeiling(180.0),
        SignificanceMarkStyle::default(),
    )
    .unwrap_err();

    assert!(auto.y_range.1 > 110.0);
    let message = explicit_error.to_string();
    let suggested_text = message
        .split_once("y_max must be at least ")
        .and_then(|(_, suffix)| suffix.split_once(';'))
        .map(|(value, _)| value)
        .unwrap();
    let suggested = suggested_text.parse::<f64>().unwrap();
    assert!(suggested > 110.0);
    assert_eq!(suggested.to_string(), suggested_text);
}
