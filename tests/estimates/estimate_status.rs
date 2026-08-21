use lichtung_plot::layout::PanelBox;
use lichtung_plot::render::backend::{
    Backend, BracketStyle, FillStyle, GradientStop, LineStyle, Point2D, PointStyle,
    SignificanceBracket, TextStyle,
};
use lichtung_plot::Error;
use lichtung_plot::{
    AxisScale, Chart, EstimateStatus, InferentialScale, IntervalKind, IntervalSemantics, ValueSort,
};
use polars::prelude::*;

#[derive(Default)]
struct BeginCountingBackend {
    begins: usize,
}

impl Backend for BeginCountingBackend {
    fn begin(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        self.begins += 1;
        Ok(())
    }

    fn begin_clip(&mut self, _panel: PanelBox) -> Result<(), Error> {
        Ok(())
    }

    fn end_clip(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn draw_scatter(&mut self, _points: &[Point2D], _style: &PointStyle) -> Result<(), Error> {
        Ok(())
    }

    fn draw_line(&mut self, _points: &[Point2D], _style: &LineStyle) -> Result<(), Error> {
        Ok(())
    }

    fn draw_polygon(&mut self, _points: &[Point2D], _style: &FillStyle) -> Result<(), Error> {
        Ok(())
    }

    fn draw_linear_gradient(
        &mut self,
        _bounds: PanelBox,
        _start: Point2D,
        _end: Point2D,
        _stops: &[GradientStop],
    ) -> Result<(), Error> {
        Ok(())
    }

    fn draw_text(
        &mut self,
        _x: u32,
        _y: u32,
        _text: &str,
        _style: &TextStyle,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn draw_bracket(
        &mut self,
        _bracket: &SignificanceBracket,
        _style: &BracketStyle,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn end(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

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
        ],
    )
    .unwrap()
}

fn all_estimated_data() -> DataFrame {
    df!(
        "term" => ["first", "second"],
        "estimate" => [1.0_f64, 2.0],
        "lower" => [0.5_f64, 1.5],
        "upper" => [1.5_f64, 2.5],
        "status" => [EstimateStatus::Estimated.as_str(); 2],
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
        "status" => [EstimateStatus::NotEstimable.as_str(); 4],
    )
    .unwrap()
}

fn semantics(scale: InferentialScale) -> IntervalSemantics {
    IntervalSemantics::new(IntervalKind::Confidence, 0.95, scale, "status fixture").unwrap()
}

fn point_interval(data: &DataFrame, status: bool) -> Chart<'_> {
    let mut chart = Chart::new(data).interval_bounds("lower", "upper");
    if status {
        chart = chart.estimate_status("status");
    }
    chart
        .x("estimate")
        .y("term")
        .size(480, 300)
        .point_interval();
    chart
}

fn coefficient(data: &DataFrame, status: bool) -> Chart<'_> {
    let mut chart = Chart::new(data)
        .interval_bounds("lower", "upper")
        .interval_semantics(semantics(InferentialScale::Coefficient));
    if status {
        chart = chart.estimate_status("status");
    }
    chart.x("estimate").y("term").size(480, 300).coefficient();
    chart
}

fn forest(data: &DataFrame, status: bool) -> Chart<'_> {
    let mut chart = Chart::new(data)
        .interval_bounds("lower", "upper")
        .interval_semantics(semantics(InferentialScale::Ratio))
        .reference_line(1.0);
    if status {
        chart = chart.estimate_status("status");
    }
    chart.x("estimate").y("term").size(480, 300).forest();
    chart
}

fn svg(chart: &Chart<'_>) -> String {
    String::from_utf8(chart.render_to_svg_bytes().unwrap()).unwrap()
}

#[test]
fn estimate_status_keeps_unavailable_rows_visible_without_numeric_marks() {
    let data = mixed_data();
    for output in [
        svg(&point_interval(&data, true)),
        svg(&coefficient(&data, true)),
        svg(&forest(&data, true)),
    ] {
        assert_eq!(output.matches(">Not estimable</text>").count(), 1);
        assert_eq!(output.matches("<circle ").count(), 2);
        assert!(text_y(&output, "First") < text_y(&output, "Unavailable"));
        assert!(text_y(&output, "Unavailable") < text_y(&output, "Third"));
    }
}

#[test]
fn all_estimated_status_is_byte_identical_to_strict_default() {
    let data = all_estimated_data();

    assert_eq!(
        point_interval(&data, false).render_to_svg_bytes().unwrap(),
        point_interval(&data, true).render_to_svg_bytes().unwrap()
    );
    assert_eq!(
        coefficient(&data, false).render_to_svg_bytes().unwrap(),
        coefficient(&data, true).render_to_svg_bytes().unwrap()
    );
    assert_eq!(
        forest(&data, false).render_to_svg_bytes().unwrap(),
        forest(&data, true).render_to_svg_bytes().unwrap()
    );
}

#[test]
fn estimate_status_requires_exact_values() {
    let mut data = mixed_data();
    data.replace(
        "status",
        Series::new("status".into(), ["estimated", "Not estimable", "estimated"]).into(),
    )
    .unwrap();

    let error = point_interval(&data, true)
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains("expected exactly \"estimated\" or \"not_estimable\""));
}

#[test]
fn estimate_status_enforces_complete_null_tuples() {
    let invalid_cases = [
        (
            [Some(10.0_f64), None],
            [Some(9.0_f64), None],
            [Some(12.0_f64), None],
            ["not_estimable", "not_estimable"],
        ),
        (
            [Some(10.0_f64), None],
            [Some(9.0_f64), Some(8.0)],
            [Some(12.0_f64), None],
            ["estimated", "estimated"],
        ),
    ];
    for (estimate, lower, upper, status) in invalid_cases {
        let data = df!(
            "term" => ["first", "second"],
            "estimate" => estimate,
            "lower" => lower,
            "upper" => upper,
            "status" => status,
        )
        .unwrap();

        assert!(point_interval(&data, true)
            .preflight()
            .unwrap_err()
            .to_string()
            .contains("estimate tuple"));
    }
}

#[test]
fn all_unavailable_rows_require_and_accept_explicit_ranges() {
    let data = df!(
        "term" => ["first", "second"],
        "estimate" => [None::<f64>; 2],
        "lower" => [None::<f64>; 2],
        "upper" => [None::<f64>; 2],
        "status" => [EstimateStatus::NotEstimable.as_str(); 2],
    )
    .unwrap();
    assert!(point_interval(&data, true)
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("set explicit x-axis limits"));
    assert!(coefficient(&data, true)
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("set explicit x-axis limits"));
    assert!(forest(&data, true)
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("set explicit x-axis limits"));

    for output in [
        svg(&point_interval(&data, true).x_limits(0.5, 2.5)),
        svg(&coefficient(&data, true).x_limits(-1.0, 1.0)),
        svg(&forest(&data, true).x_limits(0.5, 2.5)),
    ] {
        assert_eq!(output.matches(">Not estimable</text>").count(), 2);
        assert_eq!(output.matches("<circle ").count(), 0);
    }
}

#[test]
fn estimate_status_auto_range_uses_only_estimated_rows() {
    let output = svg(&point_interval(&mixed_data(), true));
    let circles = output
        .lines()
        .filter(|line| line.starts_with("<circle "))
        .count();

    assert_eq!(circles, 2);
    assert!(!output.contains(">0</text>"));
}

#[test]
fn estimate_status_rejects_nonfinite_values_and_unsupported_kinds() {
    let data = df!(
        "term" => ["first"],
        "estimate" => [Some(f64::NAN)],
        "lower" => [Some(0.0_f64)],
        "upper" => [Some(1.0_f64)],
        "status" => [EstimateStatus::NotEstimable.as_str()],
    )
    .unwrap();
    assert!(point_interval(&data, true)
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("non-finite"));

    let mut scatter = Chart::new(&data).estimate_status("status");
    scatter.x("lower").y("upper").scatter();
    assert!(scatter
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("supported only by PointInterval, Coefficient, and Forest"));
}

#[test]
fn estimate_status_rejects_value_sort_to_preserve_source_order() {
    let data = mixed_data();
    let chart = point_interval(&data, true).value_sort(lichtung_plot::ValueSort::Ascending);

    assert!(chart
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("retain source order"));
}

#[test]
fn all_estimated_status_keeps_value_sort_byte_inert() {
    let data = all_estimated_data();
    let strict = point_interval(&data, false).value_sort(ValueSort::Descending);
    let status = point_interval(&data, true).value_sort(ValueSort::Descending);

    let output = svg(&status);
    assert_eq!(
        strict.render_to_svg_bytes().unwrap(),
        status.render_to_svg_bytes().unwrap()
    );
    assert!(text_y(&output, "Second") < text_y(&output, "First"));
}

#[test]
fn fixed_facets_inherit_estimated_sibling_range_for_unavailable_panels() {
    let data = df!(
        "panel" => ["estimated", "estimated", "unavailable", "unavailable"],
        "term" => ["first", "second", "first", "second"],
        "estimate" => [Some(10.0_f64), Some(20.0), None, None],
        "lower" => [Some(9.0_f64), Some(18.0), None, None],
        "upper" => [Some(12.0_f64), Some(23.0), None, None],
        "status" => [
            EstimateStatus::Estimated.as_str(),
            EstimateStatus::Estimated.as_str(),
            EstimateStatus::NotEstimable.as_str(),
            EstimateStatus::NotEstimable.as_str(),
        ],
    )
    .unwrap();

    for mut chart in [
        point_interval(&data, true).facet_col("panel"),
        coefficient(&data, true).facet_col("panel"),
        forest(&data, true).facet_col("panel"),
    ] {
        let _ = chart.size(960, 300);
        chart.preflight().unwrap();
        let output = svg(&chart);
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
fn all_unavailable_fixed_facets_require_exact_explicit_limits() {
    let data = all_unavailable_facet_data();
    let unavailable = point_interval(&data, true).facet_col("panel");

    assert!(unavailable
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("set explicit x-axis limits"));
}

#[test]
fn all_unavailable_fixed_facets_reject_status_text_that_cannot_fit() {
    let data = all_unavailable_facet_data();
    let unavailable = point_interval(&data, true)
        .facet_col("panel")
        .x_limits(-2.0, 2.0);

    let error = unavailable.preflight().unwrap_err().to_string();

    assert!(
        error.contains("Not estimable row text requires") && error.contains("chart panel provides"),
        "unexpected preflight error: {error}"
    );
}

#[test]
fn all_unavailable_fixed_facets_render_status_text_on_sufficient_canvas() {
    let data = all_unavailable_facet_data();
    let mut available = point_interval(&data, true)
        .facet_col("panel")
        .x_limits(-2.0, 2.0);
    let _ = available.size(960, 300);

    let output = svg(&available);

    assert_eq!(output.matches(">Not estimable</text>").count(), 4);
}

#[test]
fn all_unavailable_log_forest_uses_explicit_raw_range_without_fabricated_values() {
    let data = df!(
        "term" => ["first"],
        "estimate" => [None::<f64>],
        "lower" => [None::<f64>],
        "upper" => [None::<f64>],
        "status" => [EstimateStatus::NotEstimable.as_str()],
    )
    .unwrap();
    let output = svg(&forest(&data, true)
        .axis_scale(AxisScale::Log10)
        .x_limits(0.1, 10.0));

    assert!(output.contains(">0.1</text>") && output.contains(">10</text>"));
    assert!(!output.contains("<circle "));
}

#[test]
fn unavailable_text_avoids_coefficient_forest_and_point_interval_guides() {
    let data = df!(
        "term" => ["unavailable"],
        "estimate" => [None::<f64>],
        "lower" => [None::<f64>],
        "upper" => [None::<f64>],
        "status" => [EstimateStatus::NotEstimable.as_str()],
    )
    .unwrap();
    let coefficient = svg(&coefficient(&data, true).x_limits(-1.0, 1.0));
    let forest = svg(&forest(&data, true)
        .axis_scale(AxisScale::Log10)
        .x_limits(0.1, 10.0));
    let point_interval = svg(&point_interval(&data, true).zero_line().x_limits(-1.0, 1.0));

    let minimum_separation =
        lichtung_plot::layout::deterministic_text_width("Not estimable", 12.0) / 2.0 + 5.0;
    for output in [&coefficient, &forest] {
        assert!(
            (text_x(output, "Not estimable") - vertical_line_x(output, "stroke-dasharray")).abs()
                >= minimum_separation
        );
    }
    assert!(
        (text_x(&point_interval, "Not estimable")
            - vertical_line_x(&point_interval, "stroke-width=\"0.8\""))
        .abs()
            >= minimum_separation
    );
    assert!(forest.contains(">0.1</text>") && forest.contains(">10</text>"));
    assert!(coefficient.contains(">-1</text>") && coefficient.contains(">1</text>"));
    assert!(point_interval.contains(">-1</text>") && point_interval.contains(">1</text>"));
}

#[test]
fn unavailable_text_without_a_fitting_guide_side_fails_before_backend_begin() {
    let data = df!(
        "term" => ["x"],
        "estimate" => [None::<f64>],
        "lower" => [None::<f64>],
        "upper" => [None::<f64>],
        "status" => [EstimateStatus::NotEstimable.as_str()],
    )
    .unwrap();
    let mut chart = point_interval(&data, true).zero_line().x_limits(-1.0, 1.0);
    let _ = chart.size(224, 240);

    let error = chart.preflight().unwrap_err().to_string();
    assert!(
        error.contains("does not fit on either side"),
        "unexpected preflight error: {error}"
    );
    let mut backend = BeginCountingBackend::default();
    assert!(chart.render_with_backend(&mut backend).is_err());
    assert_eq!(backend.begins, 0);
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
