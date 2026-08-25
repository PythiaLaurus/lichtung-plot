use super::*;

#[test]
fn vertical_and_horizontal_significance_use_category_series_targets() {
    let category = CategoryValues::new("category", ["a", "a", "b", "b"]).unwrap();
    let hue = CategoryValues::new("hue", ["left", "right", "left", "right"]).unwrap();
    let value = NumericValues::new("value", [2.0, 3.0, 4.0, 5.0]);
    let comparison = Significance::between(
        SignificanceTarget::category_series("a", "left"),
        SignificanceTarget::category_series("a", "right"),
        SignificanceMark::TwoStars,
    );
    let dir = tempdir("grouped-horizontal-significance");
    let vertical_path = dir.join("vertical.svg");
    let horizontal_path = dir.join("horizontal.svg");

    Plot::bar(category.clone(), BarInput::estimates(value.clone()))
        .series(hue.clone())
        .significance([comparison.clone()])
        .svg_preserve_text(&vertical_path)
        .unwrap();

    Plot::horizontal_bar(category, BarInput::estimates(value))
        .series(hue)
        .significance([comparison])
        .svg_preserve_text(&horizontal_path)
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
