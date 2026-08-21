use lichtung_plot::layout::PanelBox;
use lichtung_plot::render::backend::{
    Backend, BracketStyle, FillStyle, GradientStop, LineStyle, Point2D, PointStyle,
    SignificanceBracket, TextStyle,
};
use lichtung_plot::ComparisonAnnotation;
use lichtung_plot::Error;
use lichtung_plot::{
    BarAggregation, BarPointLayer, Chart, Figure, InferentialScale, IntervalKind,
    IntervalSemantics, SignificanceStarLimit, TextPanel,
};
use polars::prelude::*;

fn interval_semantics() -> IntervalSemantics {
    IntervalSemantics::new(
        IntervalKind::Confidence,
        0.9,
        InferentialScale::Response,
        "caller bootstrap",
    )
    .unwrap()
}

fn identity_interval_data() -> DataFrame {
    df!(
        "category" => ["a", "b"],
        "value" => [2.0_f64, 5.0],
        "lower" => [1.0_f64, 3.5],
        "upper" => [4.0_f64, 8.0],
    )
    .unwrap()
}

fn identity_interval_chart(data: &DataFrame, horizontal: bool) -> Chart<'_> {
    let mut chart = Chart::new(data)
        .bar_aggregation(BarAggregation::Identity)
        .interval_bounds("lower", "upper")
        .interval_semantics(interval_semantics());
    let _ = chart.x("category").y("value");
    if horizontal {
        chart.horizontal_bar();
    } else {
        chart.bar();
    }
    chart
}

fn svg(chart: &Chart<'_>) -> String {
    String::from_utf8(chart.render_to_svg_bytes().unwrap()).unwrap()
}

#[test]
fn identity_bar_intervals_render_asymmetric_endpoints_without_canvas_provenance() {
    let data = identity_interval_data();
    let vertical = svg(&identity_interval_chart(&data, false));
    let horizontal = svg(&identity_interval_chart(&data, true));

    assert!(!vertical.contains("90% confidence interval · response scale · caller bootstrap"));
    assert!(!horizontal.contains("90% confidence interval · response scale · caller bootstrap"));
    let mut plain = Chart::new(&data).bar_aggregation(BarAggregation::Identity);
    plain.x("category").y("value").bar();
    let plain = svg(&plain);
    assert!(vertical.matches("<polyline ").count() > plain.matches("<polyline ").count());
    assert!(horizontal.matches("<polyline ").count() > plain.matches("<polyline ").count());
}

#[test]
fn generated_mean_bars_reject_explicit_identity_interval_contracts() {
    let data = identity_interval_data();
    let mut mean = Chart::new(&data)
        .interval_bounds("lower", "upper")
        .interval_semantics(interval_semantics());
    mean.x("category").y("value").bar();
    let mean_error = mean.render_to_svg_bytes().unwrap_err().to_string();

    assert!(mean_error.contains("generated mean intervals are owned by BarAggregation"));
}

#[test]
fn identity_bar_rejects_invalid_endpoint_order_before_backend_begin() {
    let data = df!(
        "category" => ["a"],
        "value" => [2.0_f64],
        "lower" => [3.0_f64],
        "upper" => [4.0_f64],
    )
    .unwrap();
    let chart = identity_interval_chart(&data, false);
    let mut backend = BeginCounter::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("lower <= value <= upper"));
    assert_eq!(backend.begin_count, 0);
}

#[test]
fn bar_point_layers_retain_rows_at_exact_grouped_centers_in_both_orientations() {
    let bars = df!(
        "category" => ["a", "a", "b", "b"],
        "group" => ["left", "right", "left", "right"],
        "value" => [2.0_f64, 3.0, 4.0, 5.0],
    )
    .unwrap();
    let points = df!(
        "point_category" => ["a", "a", "b"],
        "point_group" => ["left", "right", "left"],
        "point_value" => [1.0_f64, 7.0, 6.0],
    )
    .unwrap();
    let layer = || {
        BarPointLayer::new(
            &points,
            "point_category",
            Some("point_group"),
            "point_value",
            (123, 45, 200),
            5,
            0.4,
        )
        .unwrap()
    };
    let render = |horizontal| {
        let mut chart = Chart::new(&bars)
            .group("group")
            .bar_aggregation(BarAggregation::Identity)
            .bar_point_layer(layer());
        let _ = chart.x("category").y("value");
        if horizontal {
            chart.horizontal_bar();
        } else {
            chart.bar();
        }
        svg(&chart)
    };

    for output in [render(false), render(true)] {
        assert_eq!(
            output
                .lines()
                .filter(|line| line.starts_with("<circle ")
                    && line.contains("fill=\"#7B2DC8\"")
                    && line.contains("r=\"5\"")
                    && line.contains("fill-opacity=\"0.4\""))
                .count(),
            points.height()
        );
    }
}

#[test]
fn unknown_bar_point_category_rejects_before_backend_begin() {
    let bars = df!("category" => ["a"], "value" => [2.0_f64]).unwrap();
    let points = df!("category" => ["missing"], "value" => [3.0_f64]).unwrap();
    let layer = BarPointLayer::new(&points, "category", None, "value", (1, 2, 3), 3, 0.5).unwrap();
    let mut chart = Chart::new(&bars)
        .bar_aggregation(BarAggregation::Identity)
        .bar_point_layer(layer);
    chart.x("category").y("value").bar();
    let mut backend = BeginCounter::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("unknown category"));
    assert_eq!(backend.begin_count, 0);
}

#[test]
fn bar_point_layers_reject_nonfinite_mismatch_stacking_other_kinds_and_facets() {
    let bars = df!(
        "category" => ["a", "b"],
        "group" => ["left", "left"],
        "value" => [2.0_f64, 3.0],
        "facet" => ["one", "two"],
    )
    .unwrap();
    let nonfinite_points = df!("category" => ["a"], "value" => [f64::NAN]).unwrap();
    let grouped_points = df!(
        "category" => ["a"],
        "group" => ["left"],
        "value" => [2.5_f64],
    )
    .unwrap();
    let nonfinite = BarPointLayer::new(
        &nonfinite_points,
        "category",
        None,
        "value",
        (1, 2, 3),
        3,
        0.5,
    )
    .unwrap();
    let grouped = BarPointLayer::new(
        &grouped_points,
        "category",
        Some("group"),
        "value",
        (1, 2, 3),
        3,
        0.5,
    )
    .unwrap();
    let mut nonfinite_chart = Chart::new(&bars)
        .bar_aggregation(BarAggregation::Identity)
        .bar_point_layer(nonfinite);
    nonfinite_chart.x("category").y("value").bar();
    let mut mismatch_chart = Chart::new(&bars)
        .group("group")
        .bar_aggregation(BarAggregation::Identity)
        .bar_point_layer(
            BarPointLayer::new(
                &grouped_points,
                "category",
                None,
                "value",
                (1, 2, 3),
                3,
                0.5,
            )
            .unwrap(),
        );
    mismatch_chart.x("category").y("value").bar();
    let mut stacked_chart = Chart::new(&bars)
        .group("group")
        .bar_point_layer(grouped.clone());
    stacked_chart.x("category").y("value").stacked_bar();
    let mut scatter_chart = Chart::new(&bars).bar_point_layer(grouped.clone());
    scatter_chart.x("value").y("value").scatter();
    let mut faceted_chart = Chart::new(&bars)
        .group("group")
        .bar_aggregation(BarAggregation::Identity)
        .bar_point_layer(grouped)
        .facet_wrap("facet", 2);
    faceted_chart.x("category").y("value").bar();

    let cases = [
        (nonfinite_chart, "finite"),
        (mismatch_chart, "must match"),
        (stacked_chart, "ordinary Bar"),
        (scatter_chart, "ordinary Bar"),
        (faceted_chart, "faceting"),
    ];
    for (chart, expected) in cases {
        let mut backend = BeginCounter::default();
        let error = chart.render_with_backend(&mut backend).unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
        assert_eq!(backend.begin_count, 0);
    }
}

#[test]
fn three_star_limit_caps_planning_and_rendering_while_four_is_byte_inert() {
    let data = df!("category" => ["a", "b"], "value" => [2.0_f64, 4.0]).unwrap();
    let make = |limit: Option<SignificanceStarLimit>| {
        let base = Chart::new(&data).bar_aggregation(BarAggregation::Identity);
        let mut chart = limit.map_or(base.clone(), |limit| base.significance_star_limit(limit));
        chart
            .x("category")
            .y("value")
            .add_comparisons([ComparisonAnnotation::new("a", "b", 0.00001)])
            .bar();
        chart.render_to_svg_bytes().unwrap()
    };

    let default = make(None);
    let explicit_four = make(Some(SignificanceStarLimit::Four));
    let capped = String::from_utf8(make(Some(SignificanceStarLimit::Three))).unwrap();

    assert_eq!(default, explicit_four);
    assert!(capped.contains(">***</text>") && !capped.contains(">****</text>"));
}

#[test]
fn figure_supports_all_text_and_mixed_cells_without_text_legends() {
    let all_text = Figure::grid(1, 2)
        .unwrap()
        .text_panel(
            0,
            0,
            TextPanel::new("Context", "Deterministically wrapped prose.").unwrap(),
        )
        .unwrap()
        .text_panel(
            0,
            1,
            TextPanel::new("Result", "Centered explanatory text.").unwrap(),
        )
        .unwrap()
        .render_to_svg_bytes()
        .unwrap();
    let data = df!("category" => ["a"], "value" => [2.0_f64]).unwrap();
    let mut chart = Chart::new(&data).bar_aggregation(BarAggregation::Identity);
    chart.x("category").y("value").bar();
    let mixed = Figure::grid(1, 2)
        .unwrap()
        .panel(0, 0, chart)
        .unwrap()
        .text_panel(
            0,
            1,
            TextPanel::new("Interpretation", "One chart and one prose cell.").unwrap(),
        )
        .unwrap()
        .render_to_svg_bytes()
        .unwrap();

    let all_text = String::from_utf8(all_text).unwrap();
    let mixed = String::from_utf8(mixed).unwrap();
    assert_eq!(all_text.matches("<svg ").count(), 1);
    assert!(all_text.contains("Context") && !all_text.contains("legend"));
    assert!(mixed.contains("Interpretation") && mixed.contains("<polygon "));
}

#[test]
fn figure_text_cells_enforce_occupancy_and_fitting() {
    let panel = TextPanel::new("Title", "Body").unwrap();
    let occupied = Figure::grid(1, 1)
        .unwrap()
        .text_panel(0, 0, panel.clone())
        .unwrap()
        .text_panel(0, 0, panel)
        .err()
        .unwrap();
    let too_long = TextPanel::new("Title", "word ".repeat(500).trim().to_owned()).unwrap();
    let fitting = Figure::grid(1, 1)
        .unwrap()
        .text_panel(0, 0, too_long)
        .unwrap()
        .size(160, 120)
        .preflight()
        .unwrap_err();

    assert!(occupied.to_string().contains("already occupied"));
    assert!(fitting.to_string().contains("text panel"));
}

#[test]
fn all_text_figure_uses_one_backend_lifecycle() {
    let figure = Figure::grid(1, 1)
        .unwrap()
        .text_panel(
            0,
            0,
            TextPanel::new("Summary", "One deterministic text cell.").unwrap(),
        )
        .unwrap();
    let mut backend = BeginCounter::default();

    figure.render_with_backend(&mut backend).unwrap();

    assert_eq!((backend.begin_count, backend.end_count), (1, 1));
}

#[derive(Default)]
struct BeginCounter {
    begin_count: usize,
    end_count: usize,
}

impl Backend for BeginCounter {
    fn begin(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        self.begin_count += 1;
        Ok(())
    }

    fn begin_clip(&mut self, _region: PanelBox) -> Result<(), Error> {
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
        _x_pixel: u32,
        _y_pixel: u32,
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
        self.end_count += 1;
        Ok(())
    }
}
