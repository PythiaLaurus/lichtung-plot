use super::*;

struct PointIntervalData {
    category: CategoryValues,
    estimate: NumericValues,
    lower: NumericValues,
    upper: NumericValues,
}

fn point_interval_data(
    category: CategoryValues,
    estimate: NumericValues,
    lower: NumericValues,
    upper: NumericValues,
) -> PointIntervalData {
    PointIntervalData {
        category,
        estimate,
        lower,
        upper,
    }
}

fn standard_data() -> PointIntervalData {
    point_interval_data(
        CategoryValues::new("category", ["first", "second", "third"]).unwrap(),
        NumericValues::new("estimate", [1.0, -2.0, 4.0]),
        NumericValues::new("lower", [-1.0, -2.5, 2.0]),
        NumericValues::new("upper", [5.0, 0.0, 5.0]),
    )
}

fn render(
    data: &PointIntervalData,
    configure: impl FnOnce(
        Plot<lichtung_plot::kind::PointInterval>,
    ) -> Plot<lichtung_plot::kind::PointInterval>,
) -> String {
    render_svg(
        configure(Plot::point_interval(
            data.category.clone(),
            data.estimate.clone(),
            Bounds::new(data.lower.clone(), data.upper.clone()),
        ))
        .size(520, 280),
        "point-interval",
    )
    .unwrap()
}

fn enable_zero_line(
    plot: Plot<lichtung_plot::kind::PointInterval>,
) -> Plot<lichtung_plot::kind::PointInterval> {
    plot.zero_line()
}

#[test]
fn point_interval_renders_asymmetric_bounds_and_endpoint_caps() {
    let data = point_interval_data(
        CategoryValues::new("category", ["first"]).unwrap(),
        NumericValues::new("estimate", [1.0]),
        NumericValues::new("lower", [-1.0]),
        NumericValues::new("upper", [5.0]),
    );
    let svg = render(&data, |chart| chart);
    let marks = clipped_marks(&svg);
    let lines = polylines(marks);
    let circle = circles(marks)[0];
    let interval = lines[0];

    assert_eq!((lines.len(), circles(marks).len()), (3, 1));
    assert_eq!(lines[1][0].0.to_bits(), interval[0].0.to_bits());
    assert_eq!(lines[2][0].0.to_bits(), interval[1].0.to_bits());
    for cap in &lines[1..] {
        let minimum = cap[0].1.min(cap[1].1);
        let maximum = cap[0].1.max(cap[1].1);
        assert!(minimum < circle.1 && circle.1 < maximum);
    }
    assert!((circle.0 - interval[0].0) < (interval[1].0 - circle.0));
}

#[test]
fn point_interval_keeps_first_row_at_top_and_sorts_by_estimate() {
    let data = standard_data();
    let observed = render(&data, |chart| chart);
    let ascending = render(&data, |chart| chart.category_order(ValueSort::Ascending));

    assert!(text_y(&observed, "First") < text_y(&observed, "Second"));
    assert!(text_y(&ascending, "Second") < text_y(&ascending, "First"));
}

#[test]
fn point_interval_auto_range_contains_every_interval_endpoint() {
    let data = standard_data();
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
    let data = point_interval_data(
        CategoryValues::new("category", ["outside"]).unwrap(),
        NumericValues::new("estimate", [0.0]),
        NumericValues::new("lower", [-10.0]),
        NumericValues::new("upper", [10.0]),
    );
    let svg = render(&data, |chart| chart.x_limits(-1.0, 1.0));
    let panel = clip_rect(&svg);
    let interval = polylines(clipped_marks(&svg))[0];

    assert!((interval[0].0 - panel.0).abs() < 0.01);
    assert!((interval[1].0 - (panel.0 + panel.2)).abs() < 0.01);
    assert!(svg.contains(">-1</text>") && svg.contains(">1</text>"));
}

#[test]
fn point_interval_clips_extreme_finite_bounds_without_projecting_infinity() {
    let data = point_interval_data(
        CategoryValues::new("category", ["extreme"]).unwrap(),
        NumericValues::new("estimate", [0.0]),
        NumericValues::new("lower", [-f64::MAX]),
        NumericValues::new("upper", [f64::MAX]),
    );
    let svg = render(&data, |chart| chart.x_limits(-1.0, 1.0));
    let panel = clip_rect(&svg);
    let interval = polylines(clipped_marks(&svg))[0];

    assert!((interval[0].0 - panel.0).abs() < 0.01);
    assert!((interval[1].0 - (panel.0 + panel.2)).abs() < 0.01);
    assert!(!svg.contains("NaN") && !svg.contains("inf"));
}

#[test]
fn point_interval_auto_range_rejects_uncontainable_extreme_ink_but_explicit_windows_clip() {
    let data = point_interval_data(
        CategoryValues::new("category", ["extreme"]).unwrap(),
        NumericValues::new("estimate", [0.0]),
        NumericValues::new("lower", [-f64::MAX]),
        NumericValues::new("upper", [f64::MAX]),
    );
    let error = Plot::point_interval(
        data.category.clone(),
        data.estimate.clone(),
        Bounds::new(data.lower.clone(), data.upper.clone()),
    )
    .size(520, 280)
    .preflight()
    .unwrap_err();
    assert!(matches!(&error, lichtung_plot::Error::Render(_)));
    assert!(error
        .to_string()
        .contains("automatic mark ink cannot obtain a finite data domain"));

    // No finite binary64 domain can extend beyond both extreme endpoints.
    // Requesting that exact window explicitly permits its boundary stroke clip.
    let svg = render(&data, |chart| chart.x_limits(-f64::MAX, f64::MAX));

    assert_eq!(circles(clipped_marks(&svg)).len(), 1);
    assert!(!svg.contains("NaN") && !svg.contains("inf"));
}

#[test]
fn point_interval_reserves_left_space_for_long_category_labels() {
    let short = point_interval_data(
        CategoryValues::new("category", ["short"]).unwrap(),
        NumericValues::new("estimate", [1.0]),
        NumericValues::new("lower", [0.0]),
        NumericValues::new("upper", [2.0]),
    );
    let long = point_interval_data(
        CategoryValues::new(
            "category",
            ["a deliberately long point interval category label"],
        )
        .unwrap(),
        NumericValues::new("estimate", [1.0]),
        NumericValues::new("lower", [0.0]),
        NumericValues::new("upper", [2.0]),
    );
    let short_panel = clip_rect(&render(&short, |chart| chart));
    let long_svg = render(&long, |chart| chart);
    let long_panel = clip_rect(&long_svg);

    assert!(long_panel.0 > short_panel.0);
    assert!(long_svg.contains("deliberately"));
}

#[test]
fn point_interval_draws_optional_zero_line_behind_marks() {
    let data = point_interval_data(
        CategoryValues::new("category", ["row"]).unwrap(),
        NumericValues::new("estimate", [1.0]),
        NumericValues::new("lower", [-2.0]),
        NumericValues::new("upper", [2.0]),
    );
    let without_zero = render(&data, |chart| chart);
    let with_zero = render(&data, enable_zero_line);
    let zero_lines = polylines(clipped_marks(&with_zero));

    assert_eq!(polylines(clipped_marks(&without_zero)).len(), 3);
    assert_eq!(zero_lines.len(), 4);
    assert!((zero_lines[0][0].0 - zero_lines[0][1].0).abs() < 0.01);
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
