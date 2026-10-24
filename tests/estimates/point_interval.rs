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
