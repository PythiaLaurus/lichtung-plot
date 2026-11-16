use super::*;
use lichtung_plot::EstimateStatus;

#[derive(Clone)]
struct StatusData {
    term: CategoryValues,
    estimate: NumericValues,
    lower: NumericValues,
    upper: NumericValues,
    status: Vec<EstimateStatus>,
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

fn text_y(svg: &str, text: &str) -> f64 {
    let needle = format!(">{text}</text>");
    let end = svg.find(&needle).unwrap();
    let start = svg[..end].rfind("<text ").unwrap();
    attribute(&svg[start..end], "y")
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
