use super::*;
use lichtung_plot::EstimateStatus;

#[derive(Clone)]
struct StatusData {
    term: CategoryValues,
    estimate: NumericValues,
    lower: NumericValues,
    upper: NumericValues,
    status: Vec<EstimateStatus>,
    panel: Option<CategoryValues>,
}

fn status_data(
    term: CategoryValues,
    estimate: NumericValues,
    lower: NumericValues,
    upper: NumericValues,
    status: Vec<EstimateStatus>,
) -> StatusData {
    StatusData {
        term,
        estimate,
        lower,
        upper,
        status,
        panel: None,
    }
}

fn mixed_data() -> StatusData {
    status_data(
        CategoryValues::new("term", ["first", "unavailable", "third"]).unwrap(),
        NumericValues::nullable("estimate", [Some(10.0), None, Some(20.0)]),
        NumericValues::nullable("lower", [Some(9.0), None, Some(18.0)]),
        NumericValues::nullable("upper", [Some(12.0), None, Some(23.0)]),
        vec![
            EstimateStatus::Estimated,
            EstimateStatus::NotEstimable,
            EstimateStatus::Estimated,
        ],
    )
}

fn all_estimated_data() -> StatusData {
    status_data(
        CategoryValues::new("term", ["first", "second"]).unwrap(),
        NumericValues::new("estimate", [1.0, 2.0]),
        NumericValues::new("lower", [0.5, 1.5]),
        NumericValues::new("upper", [1.5, 2.5]),
        vec![EstimateStatus::Estimated; 2],
    )
}

fn all_unavailable_facet_data() -> StatusData {
    let mut data = status_data(
        CategoryValues::new("term", ["one", "two", "one", "two"]).unwrap(),
        NumericValues::nullable("estimate", [None; 4]),
        NumericValues::nullable("lower", [None; 4]),
        NumericValues::nullable("upper", [None; 4]),
        vec![EstimateStatus::NotEstimable; 4],
    );
    data.panel = Some(CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap());
    data
}

fn estimated_sibling_facet_data() -> StatusData {
    let mut data = status_data(
        CategoryValues::new("term", ["first", "second", "first", "second"]).unwrap(),
        NumericValues::nullable("estimate", [Some(10.0), Some(20.0), None, None]),
        NumericValues::nullable("lower", [Some(9.0), Some(18.0), None, None]),
        NumericValues::nullable("upper", [Some(12.0), Some(23.0), None, None]),
        vec![
            EstimateStatus::Estimated,
            EstimateStatus::Estimated,
            EstimateStatus::NotEstimable,
            EstimateStatus::NotEstimable,
        ],
    );
    data.panel = Some(
        CategoryValues::new(
            "panel",
            ["estimated", "estimated", "unavailable", "unavailable"],
        )
        .unwrap(),
    );
    data
}

fn with_status<K: lichtung_plot::kind::Marker + lichtung_plot::kind::EstimateStatus>(
    plot: Plot<K>,
    data: &StatusData,
    status: bool,
) -> Plot<K> {
    if status {
        plot.estimate_status(data.status.clone())
    } else {
        plot
    }
}

fn point_interval(data: &StatusData, status: bool) -> Plot<lichtung_plot::kind::PointInterval> {
    with_status(
        Plot::point_interval(
            data.term.clone(),
            data.estimate.clone(),
            Bounds::new(data.lower.clone(), data.upper.clone()),
        ),
        data,
        status,
    )
    .size(480, 300)
}

fn coefficient(data: &StatusData, status: bool) -> Plot<lichtung_plot::kind::Coefficient> {
    with_status(
        Plot::coefficient(
            data.term.clone(),
            CoefficientInput::estimates_with_intervals(
                data.estimate.clone(),
                Bounds::new(data.lower.clone(), data.upper.clone()),
            ),
        ),
        data,
        status,
    )
    .size(480, 300)
}

fn forest(data: &StatusData, status: bool) -> Plot<lichtung_plot::kind::Forest> {
    with_status(
        Plot::forest(
            data.term.clone(),
            data.estimate.clone(),
            Bounds::new(data.lower.clone(), data.upper.clone()),
            1.0,
        ),
        data,
        status,
    )
    .size(480, 300)
}

fn faceted_estimate_plots(
    data: &StatusData,
    scales: FacetScales,
    limits: Option<(f64, f64)>,
) -> [(&'static str, Plot); 3] {
    let facet = || FacetSpec::column(data.panel.clone().unwrap()).scales(scales);
    let point = point_interval(data, true).facet(facet()).size(960, 300);
    let coefficient = coefficient(data, true).facet(facet()).size(960, 300);
    let forest = forest(data, true).facet(facet()).size(960, 300);
    let point = match limits {
        Some((lower, upper)) => point.x_limits(lower, upper),
        None => point,
    };
    let coefficient = match limits {
        Some((lower, upper)) => coefficient.x_limits(lower, upper),
        None => coefficient,
    };
    let forest = match limits {
        Some((lower, upper)) => forest.x_limits(lower, upper),
        None => forest,
    };
    [
        ("point", point.erase()),
        ("coefficient", coefficient.erase()),
        ("forest", forest.erase()),
    ]
}

fn svg<K: lichtung_plot::kind::Marker>(plot: Plot<K>, name: &str) -> String {
    render_svg(plot, name).unwrap()
}

#[test]
fn estimate_status_keeps_unavailable_rows_visible_without_numeric_marks() {
    let frame = mixed_data();
    for (plot, name) in [
        (point_interval(&frame, true).erase(), "status-point"),
        (coefficient(&frame, true).erase(), "status-coefficient"),
        (forest(&frame, true).erase(), "status-forest"),
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
            point_interval(&frame, false).erase(),
            point_interval(&frame, true).erase(),
            "point",
        ),
        (
            coefficient(&frame, false).erase(),
            coefficient(&frame, true).erase(),
            "coefficient",
        ),
        (
            forest(&frame, false).erase(),
            forest(&frame, true).erase(),
            "forest",
        ),
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
fn estimate_status_values_must_align_with_estimate_rows() {
    let frame = all_estimated_data();
    assert!(point_interval(&frame, false)
        .estimate_status([EstimateStatus::Estimated])
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("plot estimate status values have length 1, expected 2"));
}

#[test]
fn estimate_status_enforces_complete_null_tuples() {
    let invalid = [
        (
            [Some(10.0), None],
            [Some(9.0), None],
            [Some(12.0), None],
            [EstimateStatus::NotEstimable, EstimateStatus::NotEstimable],
        ),
        (
            [Some(10.0), None],
            [Some(9.0), Some(8.0)],
            [Some(12.0), None],
            [EstimateStatus::Estimated, EstimateStatus::Estimated],
        ),
    ];
    for (estimate, lower, upper, status) in invalid {
        let data = status_data(
            CategoryValues::new("term", ["first", "second"]).unwrap(),
            NumericValues::nullable("estimate", estimate),
            NumericValues::nullable("lower", lower),
            NumericValues::nullable("upper", upper),
            Vec::new(),
        );
        assert!(point_interval(&data, false)
            .estimate_status(status)
            .preflight()
            .unwrap_err()
            .to_string()
            .contains("estimate tuple"));
    }
}

#[test]
fn all_unavailable_rows_require_and_accept_explicit_ranges() {
    let data = status_data(
        CategoryValues::new("term", ["first", "second"]).unwrap(),
        NumericValues::nullable("estimate", [None; 2]),
        NumericValues::nullable("lower", [None; 2]),
        NumericValues::nullable("upper", [None; 2]),
        vec![EstimateStatus::NotEstimable; 2],
    );
    for plot in [
        point_interval(&data, true).erase(),
        coefficient(&data, true).erase(),
        forest(&data, true).erase(),
    ] {
        assert!(plot
            .preflight()
            .unwrap_err()
            .to_string()
            .contains("set explicit x-axis limits"));
    }
    for (plot, name) in [
        (
            point_interval(&data, true).x_limits(0.5, 2.5).erase(),
            "all-unavailable-point",
        ),
        (
            coefficient(&data, true).x_limits(-1.0, 1.0).erase(),
            "all-unavailable-coefficient",
        ),
        (
            forest(&data, true).x_limits(0.5, 2.5).erase(),
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
fn estimate_status_rejects_nonfinite_values() {
    let data = status_data(
        CategoryValues::new("term", ["first"]).unwrap(),
        NumericValues::nullable("estimate", [Some(f64::NAN)]),
        NumericValues::nullable("lower", [Some(0.0)]),
        NumericValues::nullable("upper", [Some(1.0)]),
        Vec::new(),
    );
    assert!(point_interval(&data, false)
        .estimate_status([EstimateStatus::NotEstimable])
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("non-finite"));
}

#[test]
fn estimate_status_value_order_contract_preserves_source_order() {
    let mixed = mixed_data();
    assert!(point_interval(&mixed, true)
        .category_order(ValueSort::Ascending)
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("do not support value-derived category order"));

    let estimated = all_estimated_data();
    let strict = svg(
        point_interval(&estimated, false).category_order(ValueSort::Descending),
        "strict-sorted-status",
    );
    let status = svg(
        point_interval(&estimated, true).category_order(ValueSort::Descending),
        "sorted-status",
    );
    assert_eq!(strict, status);
    assert!(text_y(&status, "Second") < text_y(&status, "First"));
}

#[test]
fn estimate_status_rows_accept_explicit_category_order() {
    let mixed = mixed_data();

    let rendered = svg(
        point_interval(&mixed, true).category_order(CategoryOrder::explicit([
            "third",
            "unavailable",
            "first",
        ])),
        "explicit-status-order",
    );

    assert!(text_y(&rendered, "Third") < text_y(&rendered, "Unavailable"));
    assert!(text_y(&rendered, "Unavailable") < text_y(&rendered, "First"));
}

#[test]
fn shared_x_facets_inherit_estimated_sibling_range_for_unavailable_panels() {
    let frame = estimated_sibling_facet_data();
    for scales in [FacetScales::Fixed, FacetScales::FreeY] {
        for (family, plot) in faceted_estimate_plots(&frame, scales, None) {
            let output = svg(plot, &format!("facet-{scales:?}-{family}"));
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
}

#[test]
fn free_x_facets_require_explicit_limits_for_unavailable_panels() {
    let frame = estimated_sibling_facet_data();
    for scales in [FacetScales::FreeX, FacetScales::Free] {
        for (family, plot) in faceted_estimate_plots(&frame, scales, None) {
            let error = plot.preflight().unwrap_err().to_string();
            assert!(
                error.contains("all rows are not_estimable; set explicit x-axis limits"),
                "{family} with {scales:?}: {error}"
            );
        }
    }
}

#[test]
fn explicit_limits_allow_unavailable_panels_with_free_x_scales() {
    let frame = estimated_sibling_facet_data();
    for scales in [FacetScales::FreeX, FacetScales::Free] {
        for (family, plot) in faceted_estimate_plots(&frame, scales, Some((-5.0, 30.0))) {
            let output = svg(plot, &format!("facet-explicit-{scales:?}-{family}"));
            assert_eq!(output.matches(">Not estimable</text>").count(), 2);
            assert_eq!(output.matches("<circle ").count(), 2);
        }
    }
}

#[test]
fn all_unavailable_fixed_facets_require_limits_and_sufficient_space() {
    let frame = all_unavailable_facet_data();
    for scales in [FacetScales::Fixed, FacetScales::FreeY] {
        for (family, plot) in faceted_estimate_plots(&frame, scales, None) {
            let error = plot.preflight().unwrap_err().to_string();
            assert!(
                error.contains("set explicit x-axis limits"),
                "{family} with {scales:?}: {error}"
            );
        }
    }
    for (family, plot) in faceted_estimate_plots(&frame, FacetScales::FreeY, Some((-2.0, 2.0))) {
        let output = svg(plot, &format!("all-unavailable-facets-{family}"));
        assert_eq!(output.matches(">Not estimable</text>").count(), 4);
    }
    // Inward endpoint labels no longer reserve the old outer half-label gutters.
    let narrow = point_interval(&frame, true)
        .facet(FacetSpec::column(frame.panel.clone().unwrap()))
        .x_limits(-2.0, 2.0)
        .size(360, 300);
    let error = narrow.preflight().unwrap_err().to_string();
    assert!(
        error.contains("Not estimable row text requires") && error.contains("chart panel provides"),
        "{error}"
    );
    let output = svg(narrow.size(960, 300), "all-unavailable-facets");
    assert_eq!(output.matches(">Not estimable</text>").count(), 4);
}

#[test]
fn all_unavailable_log_forest_uses_explicit_raw_range_without_fabricated_values() {
    let data = status_data(
        CategoryValues::new("term", ["first"]).unwrap(),
        NumericValues::nullable("estimate", [None]),
        NumericValues::nullable("lower", [None]),
        NumericValues::nullable("upper", [None]),
        vec![EstimateStatus::NotEstimable],
    );
    let output = svg(
        forest(&data, true)
            .scale(AxisScale::Log10)
            .x_limits(0.1, 10.0),
        "all-unavailable-log-forest",
    );
    assert!(output.contains(">0.1</text>") && output.contains(">10</text>"));
    assert!(!output.contains("<circle "));
}

#[test]
fn unavailable_text_avoids_estimate_guides() {
    let data = status_data(
        CategoryValues::new("term", ["unavailable"]).unwrap(),
        NumericValues::nullable("estimate", [None]),
        NumericValues::nullable("lower", [None]),
        NumericValues::nullable("upper", [None]),
        vec![EstimateStatus::NotEstimable],
    );
    let coefficient = svg(
        coefficient(&data, true).x_limits(-1.0, 1.0),
        "unavailable-coefficient-guide",
    );
    let forest = svg(
        forest(&data, true)
            .scale(AxisScale::Log10)
            .x_limits(0.1, 10.0),
        "unavailable-forest-guide",
    );
    let point = svg(
        point_interval(&data, true).zero_line().x_limits(-1.0, 1.0),
        "unavailable-point-guide",
    );
    let separation = 6.0;
    for output in [&coefficient, &forest] {
        assert!(
            (text_x(output, "Not estimable") - vertical_line_x(output, "stroke-dasharray")).abs()
                >= separation
        );
    }
    assert!(
        (text_x(&point, "Not estimable")
            - vertical_line_x(&point, "stroke-width=\"0.7998046875\""))
        .abs()
            >= separation
    );
}

#[test]
fn unavailable_text_without_a_fitting_guide_side_fails_preflight() {
    let data = status_data(
        CategoryValues::new("term", ["x"]).unwrap(),
        NumericValues::nullable("estimate", [None]),
        NumericValues::nullable("lower", [None]),
        NumericValues::nullable("upper", [None]),
        vec![EstimateStatus::NotEstimable],
    );
    let error = point_interval(&data, true)
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
