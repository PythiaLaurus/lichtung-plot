use super::*;

fn categories() -> Vec<String> {
    ["a", "b", "c", "d"].map(str::to_string).to_vec()
}

fn panel() -> PanelBox {
    PanelBox::new(50.0, 350.0, 40.0, 240.0)
}

fn comparison(endpoint_a: &str, endpoint_b: &str, p_value: f64) -> SignificanceAnnotation {
    SignificanceAnnotation::pairwise(endpoint_a, endpoint_b, SignificanceMark::PValue(p_value))
}

fn default_metrics() -> BracketMetrics {
    let style = SignificanceMarkStyle::default();
    bracket_metrics(style)
}

#[test]
fn planner_packs_disjoint_spans_into_minimal_deterministic_levels() {
    let comparisons = vec![
        comparison("c", "d", 0.01),
        comparison("a", "c", 0.01),
        comparison("a", "b", 0.01),
    ];
    let plan = plan_comparisons(
        &comparisons,
        &categories(),
        &[2.0, 4.0, 3.0, 5.0],
        (0.0, 10.0),
        None,
        None,
        panel(),
        LegendAnnotationConstraint::NoObstacle,
        SignificanceMarkStyle::default(),
    )
    .unwrap();

    assert_eq!(plan.brackets.len(), 3);
    assert!((plan.brackets[0].geometry.y - plan.brackets[1].geometry.y).abs() < f64::EPSILON);
    assert!(plan.brackets[2].geometry.y > plan.brackets[0].geometry.y);
}

#[test]
fn horizontal_levels_clear_the_full_previous_significance_label() {
    let comparisons = vec![
        comparison("a", "c", 0.000_01),
        comparison("a", "b", 0.000_01),
    ];
    let plan = plan_comparisons_at(
        &comparisons,
        &categories(),
        &[2.0, 4.0, 3.0, 5.0],
        None,
        (0.0, 20.0),
        None,
        None,
        panel(),
        LegendAnnotationConstraint::NoObstacle,
        true,
        SignificanceMarkStyle::default(),
    )
    .unwrap();
    let metrics = default_metrics();
    let label_width = plan.brackets[0].label.plan.width();

    assert!(
        plan.brackets[1].geometry.y - plan.brackets[0].geometry.y
            >= metrics.label_gap + label_width + metrics.panel_padding
    );
}

#[test]
fn planner_packing_is_stable_across_input_permutations() {
    let first = vec![
        comparison("a", "b", 0.04),
        comparison("c", "d", 0.03),
        comparison("b", "c", 0.004),
        comparison("a", "d", 0.0004),
    ];
    let second = vec![
        first[3].clone(),
        first[2].clone(),
        first[0].clone(),
        first[1].clone(),
    ];
    let plan = |comparisons: &[SignificanceAnnotation]| {
        plan_comparisons(
            comparisons,
            &categories(),
            &[2.0, 4.0, 3.0, 5.0],
            (0.0, 20.0),
            None,
            Some((0.0, 20.0)),
            panel(),
            LegendAnnotationConstraint::NoObstacle,
            SignificanceMarkStyle::default(),
        )
        .unwrap()
    };
    let first_plan = plan(&first);
    let second_plan = plan(&second);

    assert_eq!(first_plan.brackets.len(), second_plan.brackets.len());
    for (left, right) in first_plan.brackets.iter().zip(&second_plan.brackets) {
        assert!((left.geometry.x_left - right.geometry.x_left).abs() < f64::EPSILON);
        assert!((left.geometry.x_right - right.geometry.x_right).abs() < f64::EPSILON);
        assert!((left.geometry.y - right.geometry.y).abs() < f64::EPSILON);
        assert_eq!(left.geometry.label, right.geometry.label);
        assert_eq!(left.label.plan.source, right.label.plan.source);
        assert_eq!(left.label.plan.style, right.label.plan.style);
        assert_eq!(left.label.plan.lines, right.label.plan.lines);
    }
}

#[test]
fn planner_rejects_duplicate_unordered_comparison_pairs() {
    let comparisons = [comparison("a", "b", 0.01), comparison("b", "a", 0.02)];
    let error = plan_comparisons(
        &comparisons,
        &categories(),
        &[1.0, 2.0, 3.0, 4.0],
        (0.0, 10.0),
        None,
        None,
        panel(),
        LegendAnnotationConstraint::NoObstacle,
        SignificanceMarkStyle::default(),
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("duplicate significance comparison"));
    assert!(error.to_string().contains("unordered category pair"));
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

#[test]
fn planner_level_maximum_covers_every_category_under_each_span() {
    let comparisons = [comparison("a", "d", 0.01)];
    let plan = plan_comparisons(
        &comparisons,
        &categories(),
        &[1.0, 8.0, 2.0, 3.0],
        (0.0, 11.0),
        None,
        Some((0.0, 11.0)),
        panel(),
        LegendAnnotationConstraint::NoObstacle,
        SignificanceMarkStyle::default(),
    )
    .unwrap();
    let data_y = panel().y0 + (8.0 / 11.0) * panel().height();

    assert!(
        plan.brackets[0].geometry.y
            >= data_y + default_metrics().tick_height + default_metrics().data_gap
    );
}

#[test]
fn planner_retains_the_exact_effective_significance_label() {
    let style = SignificanceMarkStyle::new(23, crate::TextWeight::Bold, 9.0).unwrap();
    let plan_theme = crate::Theme::Nature.resolve();
    let label = plan_theme
        .typography
        .effective_significance_label(Some(style));
    let plan = plan_comparisons_with_label(
        &[comparison("a", "b", 0.01)],
        &categories(),
        &[1.0, 2.0, 3.0, 4.0],
        (0.0, 10.0),
        None,
        None,
        panel(),
        LegendAnnotationConstraint::NoObstacle,
        style,
        &label,
        &plan_theme,
        true,
    )
    .unwrap();

    let planned_label = &plan.brackets[0].label;
    assert_eq!(planned_label.plan.style.stack_source, label.stack);
    assert_eq!(planned_label.plan.style.paint, label.paint);
    assert_eq!(planned_label.plan.style.font_size, label.font_size);
    assert_eq!(planned_label.plan.style.weight, label.weight);
    assert!((planned_label.plan.line_height() - label.line_height()).abs() < 0.001);
    assert!(planned_label.exact_override && planned_label.plan.style.exact_override);
    assert_eq!(
        planned_label.plan.style.provenance.owner,
        "significance bracket label"
    );

    let chart = Plot::bar(
        crate::CategoryValues::new("category", ["a", "b"]).unwrap(),
        crate::BarInput::estimates(crate::NumericValues::new("value", [1.0, 2.0])),
    )
    .significance_mark_style(style)
    .erase();
    let render_theme = crate::Theme::Publication.resolve();
    let mut backend = crate::render::svg_backend::SvgBackend::in_memory(400, 300);
    backend.begin(400, 300).unwrap();
    render(&chart, &plan, &render_theme, &mut backend).unwrap();
    backend.end().unwrap();
    let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
    let label = svg.lines().find(|line| line.starts_with("<text ")).unwrap();

    assert!(label.contains("fill=\"#282828\""));
    assert!(label.contains("font-size=\"23\"") && label.contains("font-weight=\"700\""));
}

#[test]
fn planner_expands_auto_range_but_preserves_sufficient_explicit_limits() {
    let comparisons = [comparison("a", "b", 0.01)];
    let auto = plan_comparisons(
        &comparisons,
        &categories(),
        &[9.0, 9.0, 1.0, 1.0],
        (0.0, 10.0),
        None,
        None,
        panel(),
        LegendAnnotationConstraint::NoObstacle,
        SignificanceMarkStyle::default(),
    )
    .unwrap();
    let explicit = plan_comparisons(
        &comparisons,
        &categories(),
        &[5.0, 5.0, 1.0, 1.0],
        (0.0, 10.0),
        None,
        Some((0.0, 10.0)),
        panel(),
        LegendAnnotationConstraint::NoObstacle,
        SignificanceMarkStyle::default(),
    )
    .unwrap();

    assert!(auto.y_range.1 > 10.0);
    assert_eq!(explicit.y_range, (0.0, 10.0));
}

#[test]
fn planner_rejects_invalid_comparisons_limits_and_small_panels() {
    let unknown = [comparison("a", "missing", 0.01)];
    let invalid_p = [comparison("a", "b", f64::NAN)];
    let valid = [comparison("a", "b", 0.01)];

    let unknown_error = plan_comparisons(
        &unknown,
        &categories(),
        &[1.0; 4],
        (0.0, 10.0),
        None,
        None,
        panel(),
        LegendAnnotationConstraint::NoObstacle,
        SignificanceMarkStyle::default(),
    )
    .unwrap_err();
    let p_error = plan_comparisons(
        &invalid_p,
        &categories(),
        &[1.0; 4],
        (0.0, 10.0),
        None,
        None,
        panel(),
        LegendAnnotationConstraint::NoObstacle,
        SignificanceMarkStyle::default(),
    )
    .unwrap_err();
    let limits_error = plan_comparisons(
        &valid,
        &categories(),
        &[1.0; 4],
        (2.0, 1.0),
        None,
        Some((2.0, 1.0)),
        panel(),
        LegendAnnotationConstraint::NoObstacle,
        SignificanceMarkStyle::default(),
    )
    .unwrap_err();
    let nonfinite_limits_error = plan_comparisons(
        &valid,
        &categories(),
        &[1.0; 4],
        (0.0, f64::NAN),
        None,
        Some((0.0, f64::NAN)),
        panel(),
        LegendAnnotationConstraint::NoObstacle,
        SignificanceMarkStyle::default(),
    )
    .unwrap_err();
    let panel_error = plan_comparisons(
        &valid,
        &categories(),
        &[1.0; 4],
        (0.0, 10.0),
        None,
        None,
        PanelBox::new(0.0, 100.0, 0.0, 20.0),
        LegendAnnotationConstraint::NoObstacle,
        SignificanceMarkStyle::default(),
    )
    .unwrap_err();

    assert!(unknown_error.to_string().contains("unknown category"));
    assert!(p_error
        .to_string()
        .contains("significance p-value must be finite"));
    assert!(limits_error
        .to_string()
        .contains("finite ascending explicit y_limits"));
    assert!(nonfinite_limits_error
        .to_string()
        .contains("finite ascending explicit y_limits"));
    assert!(panel_error.to_string().contains("too short"));
}
