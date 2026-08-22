use super::*;
use lichtung_plot::EstimateStatus;

fn mixed_data() -> DataFrame {
    df!(
        "term" => ["first", "unavailable", "third"],
        "estimate" => [Some(10.0_f64), None, Some(20.0)],
        "lower" => [Some(9.0_f64), None, Some(18.0)],
        "upper" => [Some(12.0_f64), None, Some(23.0)],
        "status" => [
            EstimateStatus::Estimated.as_str(),
            EstimateStatus::NotEstimable.as_str(),
            EstimateStatus::Estimated.as_str(),
        ]
    )
    .unwrap()
}

fn all_estimated_data() -> DataFrame {
    df!(
        "term" => ["first", "second"],
        "estimate" => [1.0_f64, 2.0],
        "lower" => [0.5_f64, 1.5],
        "upper" => [1.5_f64, 2.5],
        "status" => [EstimateStatus::Estimated.as_str(); 2]
    )
    .unwrap()
}

fn all_unavailable_facet_data() -> DataFrame {
    df!(
        "panel" => ["left", "left", "right", "right"],
        "term" => ["first", "second", "first", "second"],
        "estimate" => [None::<f64>; 4],
        "lower" => [None::<f64>; 4],
        "upper" => [None::<f64>; 4],
        "status" => [EstimateStatus::NotEstimable.as_str(); 4]
    )
    .unwrap()
}

fn semantics(scale: InferentialScale) -> IntervalSemantics {
    IntervalSemantics::new(IntervalKind::Confidence, 0.95, scale, "status fixture").unwrap()
}

fn with_status(plot: Plot, frame: &DataFrame, status: bool) -> Plot {
    if status {
        plot.estimate_status(data::category(frame, "status"))
    } else {
        plot
    }
}

fn point_interval(frame: &DataFrame, status: bool) -> Plot {
    with_status(
        Plot::point_interval(
            data::category(frame, "term"),
            data::numeric(frame, "estimate"),
            data::numeric(frame, "lower"),
            data::numeric(frame, "upper"),
        ),
        frame,
        status,
    )
    .size(480, 300)
}

fn coefficient(frame: &DataFrame, status: bool) -> Plot {
    with_status(
        Plot::coefficient(
            data::category(frame, "term"),
            data::numeric(frame, "estimate"),
        )
        .interval_bounds(data::numeric(frame, "lower"), data::numeric(frame, "upper"))
        .interval_semantics(semantics(InferentialScale::Coefficient)),
        frame,
        status,
    )
    .size(480, 300)
}

fn forest(frame: &DataFrame, status: bool) -> Plot {
    with_status(
        Plot::forest(
            data::category(frame, "term"),
            data::numeric(frame, "estimate"),
            data::numeric(frame, "lower"),
            data::numeric(frame, "upper"),
        )
        .interval_semantics(semantics(InferentialScale::Ratio))
        .reference_line(1.0),
        frame,
        status,
    )
    .size(480, 300)
}

fn svg(plot: Plot, name: &str) -> String {
    render_svg(plot, name).unwrap()
}

#[test]
fn estimate_status_keeps_unavailable_rows_visible_without_numeric_marks() {
    let frame = mixed_data();
    for (plot, name) in [
        (point_interval(&frame, true), "status-point"),
        (coefficient(&frame, true), "status-coefficient"),
        (forest(&frame, true), "status-forest"),
    ] {
        let output = svg(plot, name);
        assert_eq!(output.matches(">Not estimable</text>").count(), 1);
        assert_eq!(output.matches("<circle ").count(), 2);
        assert!(text_y(&output, "First") < text_y(&output, "Unavailable"));
        assert!(text_y(&output, "Unavailable") < text_y(&output, "Third"));
    }
}

#[test]
fn all_estimated_status_is_byte_identical_to_strict_default() {
    let frame = all_estimated_data();
    for (without, with, name) in vec![
        (
            point_interval(&frame, false),
            point_interval(&frame, true),
            "point",
        ),
        (
            coefficient(&frame, false),
            coefficient(&frame, true),
            "coefficient",
        ),
        (forest(&frame, false), forest(&frame, true), "forest"),
    ]
    .into_boxed_slice()
    {
        assert_eq!(
            svg(without, &format!("strict-{name}")),
            svg(with, &format!("status-{name}"))
        );
    }
}

#[test]
fn estimate_status_requires_exact_values() {
    let frame = df!(
        "term" => ["first", "unavailable", "third"],
        "estimate" => [Some(10.0_f64), None, Some(20.0)],
        "lower" => [Some(9.0_f64), None, Some(18.0)],
        "upper" => [Some(12.0_f64), None, Some(23.0)],
        "status" => ["estimated", "Not estimable", "estimated"]
    )
    .unwrap();
    assert!(point_interval(&frame, true)
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("expected exactly \"estimated\" or \"not_estimable\""));
}

#[test]
fn estimate_status_enforces_complete_null_tuples() {
    let invalid = [
        (
            [Some(10.0), None],
            [Some(9.0), None],
            [Some(12.0), None],
            ["not_estimable", "not_estimable"],
        ),
        (
            [Some(10.0), None],
            [Some(9.0), Some(8.0)],
            [Some(12.0), None],
            ["estimated", "estimated"],
        ),
    ];
    for (estimate, lower, upper, status) in invalid {
        let frame = df!(
            "term" => ["first", "second"], "estimate" => estimate,
            "lower" => lower, "upper" => upper, "status" => status
        )
        .unwrap();
        assert!(point_interval(&frame, true)
            .preflight()
            .unwrap_err()
            .to_string()
            .contains("estimate tuple"));
    }
}

#[test]
fn all_unavailable_rows_require_and_accept_explicit_ranges() {
    let frame = df!(
        "term" => ["first", "second"],
        "estimate" => [None::<f64>; 2], "lower" => [None::<f64>; 2], "upper" => [None::<f64>; 2],
        "status" => [EstimateStatus::NotEstimable.as_str(); 2]
    )
    .unwrap();
    for plot in [
        point_interval(&frame, true),
        coefficient(&frame, true),
        forest(&frame, true),
    ] {
        assert!(plot
            .preflight()
            .unwrap_err()
            .to_string()
            .contains("set explicit x-axis limits"));
    }
    for (plot, name) in [
        (
            point_interval(&frame, true).x_limits(0.5, 2.5),
            "all-unavailable-point",
        ),
        (
            coefficient(&frame, true).x_limits(-1.0, 1.0),
            "all-unavailable-coefficient",
        ),
        (
            forest(&frame, true).x_limits(0.5, 2.5),
            "all-unavailable-forest",
        ),
    ] {
        let output = svg(plot, name);
        assert_eq!(output.matches(">Not estimable</text>").count(), 2);
        assert_eq!(output.matches("<circle ").count(), 0);
    }
}

#[test]
fn estimate_status_auto_range_uses_only_estimated_rows() {
    let output = svg(point_interval(&mixed_data(), true), "status-auto-range");
    assert_eq!(
        output
            .lines()
            .filter(|line| line.starts_with("<circle "))
            .count(),
        2
    );
    assert!(!output.contains(">0</text>"));
}

#[test]
fn estimate_status_rejects_nonfinite_values_and_unsupported_kinds() {
    let frame = df!(
        "term" => ["first"], "estimate" => [Some(f64::NAN)],
        "lower" => [Some(0.0_f64)], "upper" => [Some(1.0_f64)],
        "status" => [EstimateStatus::NotEstimable.as_str()]
    )
    .unwrap();
    assert!(point_interval(&frame, true)
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("non-finite"));
    let scatter = Plot::scatter(
        data::numeric(&frame, "lower"),
        data::numeric(&frame, "upper"),
    )
    .estimate_status(data::category(&frame, "status"));
    assert!(scatter
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("supported only by PointInterval, Coefficient, and Forest"));
}

#[test]
fn estimate_status_value_sort_contract_preserves_source_order() {
    let mixed = mixed_data();
    assert!(point_interval(&mixed, true)
        .value_sort(ValueSort::Ascending)
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("retain source order"));

    let estimated = all_estimated_data();
    let strict = svg(
        point_interval(&estimated, false).value_sort(ValueSort::Descending),
        "strict-sorted-status",
    );
    let status = svg(
        point_interval(&estimated, true).value_sort(ValueSort::Descending),
        "sorted-status",
    );
    assert_eq!(strict, status);
    assert!(text_y(&status, "Second") < text_y(&status, "First"));
}

#[test]
fn fixed_facets_inherit_estimated_sibling_range_for_unavailable_panels() {
    let frame = df!(
        "panel" => ["estimated", "estimated", "unavailable", "unavailable"],
        "term" => ["first", "second", "first", "second"],
        "estimate" => [Some(10.0_f64), Some(20.0), None, None],
        "lower" => [Some(9.0_f64), Some(18.0), None, None],
        "upper" => [Some(12.0_f64), Some(23.0), None, None],
        "status" => ["estimated", "estimated", "not_estimable", "not_estimable"]
    )
    .unwrap();
    for (plot, name) in [
        (point_interval(&frame, true), "facet-point"),
        (coefficient(&frame, true), "facet-coefficient"),
        (forest(&frame, true), "facet-forest"),
    ] {
        let output = svg(
            plot.facet_col(data::category(&frame, "panel"))
                .size(960, 300),
            name,
        );
        assert_eq!(output.matches(">Not estimable</text>").count(), 2);
        assert_eq!(output.matches("<circle ").count(), 2);
        let first = text_ys(&output, "First");
        let second = text_ys(&output, "Second");
        assert_eq!((first.len(), second.len()), (2, 2));
        assert!(first
            .iter()
            .zip(second)
            .all(|(first, second)| first < &second));
    }
}

#[test]
fn all_unavailable_fixed_facets_require_limits_and_sufficient_space() {
    let frame = all_unavailable_facet_data();
    assert!(point_interval(&frame, true)
        .facet_col(data::category(&frame, "panel"))
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("set explicit x-axis limits"));
    let narrow = point_interval(&frame, true)
        .facet_col(data::category(&frame, "panel"))
        .x_limits(-2.0, 2.0);
    let error = narrow.preflight().unwrap_err().to_string();
    assert!(
        error.contains("Not estimable row text requires") && error.contains("chart panel provides")
    );
    let output = svg(narrow.size(960, 300), "all-unavailable-facets");
    assert_eq!(output.matches(">Not estimable</text>").count(), 4);
}

#[test]
fn all_unavailable_log_forest_uses_explicit_raw_range_without_fabricated_values() {
    let frame = df!(
        "term" => ["first"], "estimate" => [None::<f64>], "lower" => [None::<f64>],
        "upper" => [None::<f64>], "status" => [EstimateStatus::NotEstimable.as_str()]
    )
    .unwrap();
    let output = svg(
        forest(&frame, true)
            .axis_scale(AxisScale::Log10)
            .x_limits(0.1, 10.0),
        "all-unavailable-log-forest",
    );
    assert!(output.contains(">0.1</text>") && output.contains(">10</text>"));
    assert!(!output.contains("<circle "));
}

#[test]
fn unavailable_text_avoids_estimate_guides() {
    let frame = df!(
        "term" => ["unavailable"], "estimate" => [None::<f64>], "lower" => [None::<f64>],
        "upper" => [None::<f64>], "status" => [EstimateStatus::NotEstimable.as_str()]
    )
    .unwrap();
    let coefficient = svg(
        coefficient(&frame, true).x_limits(-1.0, 1.0),
        "unavailable-coefficient-guide",
    );
    let forest = svg(
        forest(&frame, true)
            .axis_scale(AxisScale::Log10)
            .x_limits(0.1, 10.0),
        "unavailable-forest-guide",
    );
    let point = svg(
        point_interval(&frame, true).zero_line().x_limits(-1.0, 1.0),
        "unavailable-point-guide",
    );
    let separation =
        lichtung_plot::layout::deterministic_text_width("Not estimable", 12.0) / 2.0 + 5.0;
    for output in [&coefficient, &forest] {
        assert!(
            (text_x(output, "Not estimable") - vertical_line_x(output, "stroke-dasharray")).abs()
                >= separation
        );
    }
    assert!(
        (text_x(&point, "Not estimable") - vertical_line_x(&point, "stroke-width=\"0.8\"")).abs()
            >= separation
    );
}

#[test]
fn unavailable_text_without_a_fitting_guide_side_fails_preflight() {
    let frame = df!(
        "term" => ["x"], "estimate" => [None::<f64>], "lower" => [None::<f64>],
        "upper" => [None::<f64>], "status" => [EstimateStatus::NotEstimable.as_str()]
    )
    .unwrap();
    let error = point_interval(&frame, true)
        .zero_line()
        .x_limits(-1.0, 1.0)
        .size(224, 240)
        .preflight()
        .unwrap_err();
    assert!(error.to_string().contains("does not fit on either side"));
}

fn text_y(svg: &str, text: &str) -> f64 {
    let needle = format!(">{text}</text>");
    let end = svg.find(&needle).unwrap();
    let start = svg[..end].rfind("<text ").unwrap();
    attribute(&svg[start..end], "y")
}

fn text_x(svg: &str, text: &str) -> f64 {
    let needle = format!(">{text}</text>");
    let end = svg.find(&needle).unwrap();
    let start = svg[..end].rfind("<text ").unwrap();
    attribute(&svg[start..end], "x")
}

fn vertical_line_x(svg: &str, identifying_attribute: &str) -> f64 {
    let line = svg
        .lines()
        .find(|line| line.starts_with("<polyline ") && line.contains(identifying_attribute))
        .unwrap();
    line.split_once("points=\"")
        .unwrap()
        .1
        .split_once(',')
        .unwrap()
        .0
        .parse()
        .unwrap()
}

fn text_ys(svg: &str, text: &str) -> Vec<f64> {
    let needle = format!(">{text}</text>");
    let mut remaining = svg;
    let mut values = Vec::new();
    while let Some(end) = remaining.find(&needle) {
        let start = remaining[..end].rfind("<text ").unwrap();
        values.push(attribute(&remaining[start..end], "y"));
        remaining = &remaining[end + needle.len()..];
    }
    values
}

fn attribute(tag: &str, name: &str) -> f64 {
    tag.split_once(&format!("{name}=\""))
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .parse()
        .unwrap()
}
