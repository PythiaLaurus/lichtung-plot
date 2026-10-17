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
fn selected_theme_gap_and_tick_height_survive_canonical_capture_with_exact_precedence() {
    let chart = Plot::bar(
        crate::CategoryValues::new("group", ["a", "b", "c", "d"]).unwrap(),
        crate::BarInput::estimates(crate::NumericValues::new("value", [1.0, 2.0, 3.0, 4.0])),
    )
    .significance([Significance::between(
        "a",
        "b",
        SignificanceMark::PValue(0.01),
    )])
    .erase();
    let mut theme = crate::Theme::Publication.resolve().unwrap();
    let label = theme
        .typography
        .annotations()
        .significance_label()
        .clone()
        .with_font_size(24)
        .unwrap();
    let roles = theme
        .typography
        .annotations()
        .clone()
        .with_significance_label(label)
        .unwrap();
    theme.typography = theme.typography.clone().with_annotations(roles).unwrap();
    let geometry = theme
        .annotations
        .significance()
        .clone()
        .with_label_gap(crate::theme::RelativeSpacing::ratio(1, 1).unwrap())
        .unwrap()
        .with_tick_height(crate::theme::RelativeSpacing::ratio(1, 2).unwrap())
        .unwrap();
    theme.annotations = theme
        .annotations
        .clone()
        .with_significance(geometry)
        .unwrap();
    let selected = plan(
        &chart,
        &categories(),
        &[1.0, 2.0, 3.0, 4.0],
        (0.0, 10.0),
        panel(),
        &theme,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .unwrap();
    let bracket = &selected.brackets[0];
    assert!(!bracket.label.exact_override);
    assert_eq!(bracket.style.font_size, 24);
    assert_eq!(bracket.style.label_gap.to_bits(), 24.0_f64.to_bits());
    let call = PlannedSignificanceCall {
        bracket: &bracket.geometry,
        style: &bracket.style,
        label: &bracket.label,
    };
    let canonical = crate::render::backend::validate_planned_bracket(call).unwrap();
    assert_eq!(
        canonical.line_start.y.units() - canonical.first_tick_end.y.units(),
        12 * 1024
    );
    assert_eq!(
        canonical.label_position.y.units() - canonical.line_start.y.units(),
        24 * 1024
    );
    let captured =
        crate::render::primitives::CanonicalPlannedBracket::new(call, 300, None).unwrap();
    assert_eq!(
        captured.points[2].y.units() - captured.points[0].y.units(),
        12 * 1024
    );
    let exact = SignificanceMarkStyle::new(12, crate::TextWeight::Bold, 3.0).unwrap();
    let effective = theme.effective_significance(Some(exact)).unwrap();
    assert!(effective.exact_override);
    assert_eq!(effective.label.font_size(), 12);
    assert_eq!(effective.label.weight(), crate::TextWeight::Bold);
    assert_eq!(effective.gap_pixels().unwrap().to_bits(), 3.0_f64.to_bits());
    assert_eq!(
        effective.tick_height().unwrap().to_bits(),
        6.0_f64.to_bits()
    );
    assert_eq!(bracket.style.tick_height.to_bits(), 12.0_f64.to_bits());
    let mut malformed = bracket.style.clone();
    malformed.tick_height = -1.0;
    assert!(crate::render::backend::validate_bracket(&bracket.geometry, &malformed).is_err());
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
