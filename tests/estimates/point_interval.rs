use std::sync::atomic::{AtomicU64, Ordering};

use lichtung_plot::{
    Chart, InferentialScale, IntervalKind, IntervalSemantics, PlotKind, ValueSort,
};
use polars::prelude::*;

static TEST_DIRECTORY_COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture() -> DataFrame {
    df!(
        "category" => ["first", "second", "third"],
        "estimate" => [1.0_f64, -2.0, 4.0],
        "lower" => [-1.0_f64, -2.5, 2.0],
        "upper" => [5.0_f64, 0.0, 5.0]
    )
    .unwrap()
}

fn render(data: &DataFrame, configure: impl FnOnce(Chart<'_>) -> Chart<'_>) -> String {
    let directory = std::env::temp_dir().join(format!(
        "lichtung-point-interval-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        TEST_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("plot.svg");
    let mut chart = configure(Chart::new(data).interval_bounds("lower", "upper"));
    chart
        .x("estimate")
        .y("category")
        .size(520, 280)
        .point_interval();
    chart.render_to_svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    svg
}

fn enable_zero_line(chart: Chart<'_>) -> Chart<'_> {
    chart.zero_line()
}

#[test]
fn point_interval_api_selects_kind_and_exposes_bound_columns() {
    let data = fixture();
    let mut chart = Chart::new(&data).interval_bounds("lower", "upper");

    chart.point_interval();

    assert_eq!(
        (chart.plot_kind(), chart.interval_bound_cols()),
        (Some(PlotKind::PointInterval), Some(("lower", "upper")))
    );
}

#[test]
fn typed_interval_provenance_is_not_drawn_on_canvas() {
    let data = fixture();
    let semantics = || {
        IntervalSemantics::new(
            IntervalKind::Confidence,
            0.95,
            InferentialScale::Response,
            "registered model",
        )
        .unwrap()
    };
    let svg = render(&data, |chart| chart.interval_semantics(semantics()));

    assert!(!svg.contains("semantic-note"));
    assert!(!svg.contains("registered model"));
}

#[test]
fn svg_bytes_match_file_rendering() {
    let data = fixture();
    let mut chart = Chart::new(&data).interval_bounds("lower", "upper");
    chart
        .x("estimate")
        .y("category")
        .size(520, 280)
        .point_interval();
    let directory = std::env::temp_dir().join(format!(
        "lichtung-point-interval-byte-parity-{}-{}",
        std::process::id(),
        TEST_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("plot.svg");

    let bytes = chart.render_to_svg_bytes().unwrap();
    chart.render_to_svg(&output).unwrap();
    let file_bytes = std::fs::read(output).unwrap();
    std::fs::remove_dir_all(directory).unwrap();

    assert_eq!(bytes, file_bytes);
}

#[test]
fn point_interval_renders_asymmetric_intervals_with_fixed_pixel_caps() {
    let data = fixture().slice(0, 1);
    let svg = render(&data, |chart| chart);
    let marks = clipped_marks(&svg);
    let lines = polylines(marks);
    let circle = circles(marks)[0];
    let interval = lines[0];

    assert_eq!((lines.len(), circles(marks).len()), (3, 1));
    assert!(((lines[1][1].1 - lines[1][0].1).abs() - 8.0).abs() < 0.01);
    assert!(((lines[2][1].1 - lines[2][0].1).abs() - 8.0).abs() < 0.01);
    assert!((circle.0 - interval[0].0) < (interval[1].0 - circle.0));
}

#[test]
fn point_interval_keeps_first_row_at_top_and_sorts_by_estimate() {
    let data = fixture();
    let observed = render(&data, |chart| chart);
    let ascending = render(&data, |chart| chart.value_sort(ValueSort::Ascending));

    assert!(text_y(&observed, "First") < text_y(&observed, "Second"));
    assert!(text_y(&ascending, "Second") < text_y(&ascending, "First"));
}

#[test]
fn point_interval_auto_range_contains_every_interval_endpoint() {
    let data = fixture();
    let svg = render(&data, |chart| chart);
    let panel = clip_rect(&svg);
    let interval_lines = polylines(clipped_marks(&svg));

    assert!(interval_lines
        .iter()
        .step_by(3)
        .all(|line| { line[0].0 >= panel.0 && line[1].0 <= panel.0 + panel.2 }));
}

#[test]
fn point_interval_explicit_x_limits_remain_exact_and_clip_marks_in_data_space() {
    let data = df!(
        "category" => ["outside"],
        "estimate" => [0.0_f64],
        "lower" => [-10.0_f64],
        "upper" => [10.0_f64]
    )
    .unwrap();
    let svg = render(&data, |chart| chart.x_limits(-1.0, 1.0));
    let panel = clip_rect(&svg);
    let interval = polylines(clipped_marks(&svg))[0];

    assert!((interval[0].0 - panel.0).abs() < 0.01);
    assert!((interval[1].0 - (panel.0 + panel.2)).abs() < 0.01);
    assert!(svg.contains(">-1</text>") && svg.contains(">1</text>"));
}

#[test]
fn point_interval_clips_extreme_finite_bounds_without_projecting_infinity() {
    let data = df!(
        "category" => ["extreme"],
        "estimate" => [0.0_f64],
        "lower" => [-f64::MAX],
        "upper" => [f64::MAX]
    )
    .unwrap();
    let svg = render(&data, |chart| chart.x_limits(-1.0, 1.0));
    let panel = clip_rect(&svg);
    let interval = polylines(clipped_marks(&svg))[0];

    assert!((interval[0].0 - panel.0).abs() < 0.01);
    assert!((interval[1].0 - (panel.0 + panel.2)).abs() < 0.01);
    assert!(!svg.contains("NaN") && !svg.contains("inf"));
}

#[test]
fn point_interval_auto_range_handles_opposite_extreme_finite_bounds() {
    let data = df!(
        "category" => ["extreme"],
        "estimate" => [0.0_f64],
        "lower" => [-f64::MAX],
        "upper" => [f64::MAX]
    )
    .unwrap();
    let svg = render(&data, |chart| chart);

    assert_eq!(circles(clipped_marks(&svg)).len(), 1);
    assert!(!svg.contains("NaN") && !svg.contains("inf"));
}

#[test]
fn point_interval_reserves_left_space_for_long_category_labels() {
    let short = df!(
        "category" => ["short"],
        "estimate" => [1.0_f64],
        "lower" => [0.0_f64],
        "upper" => [2.0_f64]
    )
    .unwrap();
    let long = df!(
        "category" => ["a deliberately long point interval category label"],
        "estimate" => [1.0_f64],
        "lower" => [0.0_f64],
        "upper" => [2.0_f64]
    )
    .unwrap();
    let short_panel = clip_rect(&render(&short, |chart| chart));
    let long_svg = render(&long, |chart| chart);
    let long_panel = clip_rect(&long_svg);

    assert!(long_panel.0 > short_panel.0);
    assert!(long_svg.contains("deliberately"));
}

#[test]
fn point_interval_draws_optional_zero_line_behind_marks() {
    let data = df!(
        "category" => ["row"],
        "estimate" => [1.0_f64],
        "lower" => [-2.0_f64],
        "upper" => [2.0_f64]
    )
    .unwrap();
    let without_zero = render(&data, |chart| chart);
    let with_zero = render(&data, enable_zero_line);
    let zero_lines = polylines(clipped_marks(&with_zero));

    assert_eq!(polylines(clipped_marks(&without_zero)).len(), 3);
    assert_eq!(zero_lines.len(), 4);
    assert!((zero_lines[0][0].0 - zero_lines[0][1].0).abs() < 0.01);
}

#[test]
fn point_interval_renders_one_constant_row_without_invalid_geometry() {
    let data = df!(
        "category" => ["only"],
        "estimate" => [5.0_f64],
        "lower" => [5.0_f64],
        "upper" => [5.0_f64]
    )
    .unwrap();
    let svg = render(&data, |chart| chart);

    assert_eq!(circles(clipped_marks(&svg)).len(), 1);
    assert!(!svg.contains("NaN") && !svg.contains("inf"));
}

fn clipped_marks(svg: &str) -> &str {
    svg.split_once("<g clip-path=\"url(#clip-0)\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0
}

fn clip_rect(svg: &str) -> (f64, f64, f64, f64) {
    let clip = svg
        .split_once("<clipPath id=\"clip-0\">")
        .unwrap()
        .1
        .split_once("</clipPath>")
        .unwrap()
        .0;
    (
        attr(clip, "x"),
        attr(clip, "y"),
        attr(clip, "width"),
        attr(clip, "height"),
    )
}

fn polylines(svg: &str) -> Vec<[(f64, f64); 2]> {
    svg.split("<polyline points=\"")
        .skip(1)
        .map(|segment| {
            let points = segment.split_once('"').unwrap().0;
            let mut points = points.split_whitespace().map(|point| {
                let (x, y) = point.split_once(',').unwrap();
                (x.parse().unwrap(), y.parse().unwrap())
            });
            [points.next().unwrap(), points.next().unwrap()]
        })
        .collect()
}

fn circles(svg: &str) -> Vec<(f64, f64)> {
    svg.split("<circle ")
        .skip(1)
        .map(|circle| (attr(circle, "cx"), attr(circle, "cy")))
        .collect()
}

fn text_y(svg: &str, text: &str) -> f64 {
    let needle = format!(">{text}</text>");
    let end = svg.find(&needle).unwrap();
    let start = svg[..end].rfind("<text ").unwrap();
    attr(&svg[start..end], "y")
}

fn attr(tag: &str, name: &str) -> f64 {
    tag.split_once(&format!("{name}=\""))
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .parse()
        .unwrap()
}
