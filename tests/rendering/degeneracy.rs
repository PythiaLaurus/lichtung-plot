use super::*;
use lichtung_plot::{
    Error, HistogramBin, HistogramComposite, HistogramDimensions, HistogramGeometry,
    HistogramInput, HistogramKdePoint, HistogramRug,
};

fn assert_collapse<K: lichtung_plot::kind::Marker>(plot: Plot<K>, context: &str) {
    let error = plot.preflight().unwrap_err();
    assert!(matches!(error, Error::Render(_)), "{error}");
    let message = error.to_string();
    assert!(
        message.contains(context) && message.contains("collapse"),
        "{message}"
    );
    assert!(!message.contains("layout fit"), "{message}");
    let directory = tempfile::tempdir().unwrap();
    let output = directory.path().join("plot.svg");
    assert_eq!(
        plot.svg_preserve_text(&output).err().unwrap().to_string(),
        message
    );
    assert_eq!(directory.path().read_dir().unwrap().count(), 0);
}

fn polygons<K: lichtung_plot::kind::Marker>(plot: Plot<K>) -> Vec<Vec<(f64, f64)>> {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("plot.svg");
    plot.svg_preserve_text(&path).unwrap();
    let svg = std::fs::read_to_string(path).unwrap();
    let marks = svg
        .split_once("<g clip-path=\"url(#clip-0)\">")
        .unwrap()
        .1
        .split_once("</g>")
        .unwrap()
        .0;
    marks
        .split("<polygon points=\"")
        .skip(1)
        .map(|part| {
            part.split_once('"')
                .unwrap()
                .0
                .split_whitespace()
                .map(|point| {
                    let (x, y) = point.split_once(',').unwrap();
                    (x.parse().unwrap(), y.parse().unwrap())
                })
                .collect()
        })
        .collect()
}

fn bars(values: [f64; 2], range: Option<(f64, f64)>) -> [Plot; 4] {
    let categories = CategoryValues::new("category", ["a", "b"]).unwrap();
    let stacked = CategoryValues::new("category", ["a", "a"]).unwrap();
    let series = CategoryValues::new("series", ["first", "second"]).unwrap();
    let values = NumericValues::new("value", values);
    let mut bar = Plot::bar(categories.clone(), BarInput::estimates(values.clone()));
    let mut horizontal = Plot::horizontal_bar(categories, BarInput::estimates(values.clone()));
    let mut stacked_bar = Plot::stacked_bar(stacked.clone(), values.clone(), series.clone());
    let mut stacked_horizontal = Plot::stacked_horizontal_bar(stacked, values, series);
    if let Some((min, max)) = range {
        bar = bar.y_limits(min, max);
        horizontal = horizontal.x_limits(min, max);
        stacked_bar = stacked_bar.y_limits(min, max);
        stacked_horizontal = stacked_horizontal.x_limits(min, max);
    }
    [
        bar.erase(),
        horizontal.erase(),
        stacked_bar.erase(),
        stacked_horizontal.erase(),
    ]
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "canonical SVG spans are exact quantum differences"
)]
fn bar_spans_reject_subquantum_values_but_preserve_normal_signed_and_zero_values() {
    for (value, collapse) in [
        (0.25, false),
        (-0.25, false),
        (0.0, false),
        (-0.0, false),
        (1e-9, true),
        (-1e-9, true),
    ] {
        for (family, plot) in bars([0.5, value], Some((-1.0, 1.0)))
            .into_iter()
            .enumerate()
        {
            if collapse {
                assert_collapse(plot, "bar polygon");
            } else {
                let polygons = polygons(plot);
                assert_eq!(polygons.len(), 2);
                let bar = &polygons[1];
                assert_eq!(bar.len(), 4);
                let span = if family % 2 == 0 {
                    bar[0].1 - bar[1].1
                } else {
                    bar[1].0 - bar[0].0
                };
                if value == 0.0 {
                    assert_eq!(span, 0.0);
                } else {
                    assert!(span * value > 0.0);
                }
            }
        }
    }
}

#[test]
fn bar_spans_reject_cumulative_precision_loss_and_auto_domain_collapse() {
    for sign in [1.0, -1.0] {
        for plot in bars([sign * 1e16, sign], None) {
            assert_collapse(plot, "bar polygon");
        }
    }
}

fn histogram_geometry(edge: f64, density: f64) -> HistogramGeometry {
    HistogramGeometry {
        dimensions: HistogramDimensions {
            value: "value".into(),
            density: "density".into(),
        },
        bins: vec![
            HistogramBin {
                lower: 0.0,
                upper: edge,
                density,
            },
            HistogramBin {
                lower: edge,
                upper: 1.0,
                density: 1.0,
            },
        ],
        kde: vec![
            HistogramKdePoint { x: 0.0, y: 0.5 },
            HistogramKdePoint { x: 1.0, y: 0.5 },
        ],
        rugs: vec![HistogramRug {
            x: 0.75,
            baseline: 0.0,
            top: 0.1,
        }],
    }
}

fn histogram(edge: f64, density: f64) -> Plot<lichtung_plot::kind::Histogram> {
    Plot::histogram(HistogramInput::geometry(histogram_geometry(edge, density)))
        .composite(HistogramComposite::HistogramOnly)
        .x_limits(0.0, 1.0)
        .y_limits(0.0, 1.0)
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "canonical SVG spans are exact quantum differences"
)]
fn histogram_spans_preserve_zero_density_and_reject_collapsed_width_or_positive_height() {
    for density in [0.0, -0.0, 0.5] {
        let polygons = polygons(histogram(0.5, density));
        assert_eq!(polygons.len(), 2);
        let bin = &polygons[0];
        assert_eq!(bin.len(), 4);
        assert!(bin[1].0 > bin[0].0);
        let height = bin[0].1 - bin[2].1;
        if density == 0.0 {
            assert_eq!(height, 0.0);
        } else {
            assert!(height > 0.0);
        }
    }
    assert_collapse(histogram(1e-9, 0.5), "histogram polygon width");
    assert_collapse(histogram(0.5, 1e-9), "histogram polygon height");
    assert_collapse(
        Plot::histogram(HistogramInput::samples(NumericValues::new(
            "value",
            [0.0, 0.25, 0.75, 1.0],
        )))
        .bins(lichtung_plot::HistogramBins::Count(2))
        .composite(HistogramComposite::HistogramOnly)
        .y_limits(0.0, 1e16)
        .erase(),
        "histogram polygon height",
    );
    let mut geometry = histogram_geometry(0.5, 0.5);
    geometry.rugs[0].top = 1e-9;
    assert_collapse(
        Plot::histogram(HistogramInput::geometry(geometry)).y_limits(0.0, 1.0),
        "histogram rug interval",
    );
}

#[test]
fn rectangle_clipping_and_failed_canvas_roots_preserve_publication_contracts() {
    for range in [(0.2, 0.6), (2.0, 3.0)] {
        for plot in bars([0.5, 0.25], Some(range)) {
            plot.preflight().unwrap();
        }
    }
    histogram(0.5, 0.5).x_limits(2.0, 3.0).preflight().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let canvas = lichtung_plot::Canvas::grid(1, 2)
        .unwrap()
        .panel(
            0,
            0,
            Plot::bar(
                CategoryValues::new("category", ["a"]).unwrap(),
                BarInput::estimates(NumericValues::new("value", [0.5])),
            ),
        )
        .unwrap()
        .panel(0, 1, histogram(0.5, 1e-9))
        .unwrap();
    let error = canvas
        .svg_preserve_text(directory.path().join("canvas.svg"))
        .err()
        .unwrap();
    assert!(matches!(error, Error::Render(_)), "{error}");
    assert!(
        error.to_string().contains("histogram polygon height"),
        "{error}"
    );
    assert_eq!(directory.path().read_dir().unwrap().count(), 0);
}

fn intervals(extent: f64) -> [Plot; 7] {
    let category = || CategoryValues::new("category", ["a"]).unwrap();
    let estimate = || NumericValues::new("estimate", [0.0]);
    let bounds = || {
        lichtung_plot::Bounds::new(
            NumericValues::new("lower", [-extent]),
            NumericValues::new("upper", [extent]),
        )
    };
    [
        Plot::bar(
            category(),
            BarInput::estimates_with_error_bars(estimate(), bounds()),
        )
        .y_limits(-1.0, 1.0)
        .erase(),
        Plot::horizontal_bar(
            category(),
            BarInput::estimates_with_error_bars(estimate(), bounds()),
        )
        .x_limits(-1.0, 1.0)
        .erase(),
        Plot::point_interval(category(), estimate(), bounds())
            .x_limits(-1.0, 1.0)
            .erase(),
        Plot::forest(category(), estimate(), bounds(), 0.0)
            .x_limits(-1.0, 1.0)
            .erase(),
        Plot::coefficient(
            category(),
            lichtung_plot::CoefficientInput::estimates_with_intervals(estimate(), bounds()),
        )
        .x_limits(-1.0, 1.0)
        .erase(),
        Plot::dumbbell(
            category(),
            estimate(),
            NumericValues::new("right", [extent]),
            lichtung_plot::DumbbellEndpoints::new(
                "left",
                lichtung_plot::Color::rgb(1, 2, 3),
                "right",
                lichtung_plot::Color::rgb(4, 5, 6),
            )
            .unwrap(),
        )
        .x_limits(-1.0, 1.0)
        .erase(),
        Plot::interaction(
            category(),
            estimate(),
            CategoryValues::new("group", ["g"]).unwrap(),
            bounds(),
        )
        .y_limits(-1.0, 1.0)
        .erase(),
    ]
}

fn area_plots(range: (f64, f64)) -> [Plot; 6] {
    let values = || NumericValues::new("value", [-1.0, 0.0, 1.0, 2.0]);
    let categories = || CategoryValues::new("category", ["a"; 4]).unwrap();
    [
        Plot::line(
            NumericValues::new("x", [0.0, 1.0]),
            NumericValues::new("y", [0.0, 0.0]),
        )
        .band(lichtung_plot::Bounds::new(
            NumericValues::new("lower", [-0.5, -0.5]),
            NumericValues::new("upper", [0.5, 0.5]),
        ))
        .y_limits(range.0, range.1)
        .erase(),
        Plot::regression(
            NumericValues::new("x", [0.0, 1.0, 2.0, 3.0]),
            NumericValues::new("y", [0.0, 1.0, 1.0, 0.0]),
        )
        .y_limits(range.0, range.1)
        .erase(),
        Plot::boxplot(categories(), values())
            .y_limits(range.0, range.1)
            .erase(),
        Plot::violin(categories(), values())
            .y_limits(range.0, range.1)
            .erase(),
        Plot::raincloud(lichtung_plot::RaincloudInput::samples(
            categories(),
            values(),
        ))
        .y_limits(range.0, range.1)
        .erase(),
        Plot::density(values()).y_limits(range.0, range.1).erase(),
    ]
}

#[test]
fn scientific_intervals_reject_collapse_but_preserve_normal_and_zero_spans() {
    for extent in [0.0, 0.25] {
        for plot in intervals(extent) {
            plot.preflight().unwrap();
        }
    }
    for plot in intervals(1e-9) {
        assert_collapse(plot, "interval");
    }
}

#[test]
fn scientific_area_rejects_collapsed_polygons_but_preserves_normal_geometry() {
    for plot in area_plots((-10.0, 10.0)) {
        plot.preflight().unwrap();
    }
    for plot in area_plots((-1e16, 1e16)) {
        assert_collapse(plot, "polygon");
    }
}

#[test]
fn ribbon_and_density_support_widths_cannot_collapse() {
    let line = Plot::line(
        NumericValues::new("x", [0.0, 1.0]),
        NumericValues::new("y", [0.0, 0.0]),
    )
    .band(lichtung_plot::Bounds::new(
        NumericValues::new("lower", [-0.5, -0.5]),
        NumericValues::new("upper", [0.5, 0.5]),
    ))
    .x_limits(-1e16, 1e16);
    assert_collapse(line, "line ribbon polygon width");
    let regression = Plot::regression(
        NumericValues::new("x", [0.0, 1.0, 2.0, 3.0]),
        NumericValues::new("y", [0.0, 1.0, 1.0, 0.0]),
    )
    .x_limits(-1e16, 1e16);
    assert_collapse(regression, "regression band polygon width");
    assert_collapse(
        Plot::density(NumericValues::new("value", [0.0, 1.0, 2.0, 3.0])).x_limits(-1e16, 1e16),
        "density polygon",
    );
}

#[test]
fn true_zero_area_and_data_clipped_interval_touches_remain_legal() {
    Plot::line(
        NumericValues::new("x", [0.0, 1.0]),
        NumericValues::new("y", [-0.5, 0.5]),
    )
    .band(lichtung_plot::Bounds::new(
        NumericValues::new("lower", [-0.5, 0.5]),
        NumericValues::new("upper", [-0.5, 0.5]),
    ))
    .x_limits(-1e16, 1e16)
    .y_limits(-1.0, 1.0)
    .preflight()
    .unwrap();
    Plot::regression(
        NumericValues::new("x", [0.0, 1.0, 2.0]),
        NumericValues::new("y", [0.0, 1.0, 2.0]),
    )
    .x_limits(-1e16, 1e16)
    .y_limits(-1.0, 3.0)
    .preflight()
    .unwrap();
    Plot::line(
        NumericValues::new("x", [0.0, 1.0]),
        NumericValues::new("y", [0.0, 0.0]),
    )
    .band(lichtung_plot::Bounds::new(
        NumericValues::new("lower", [-0.0, 0.0]),
        NumericValues::new("upper", [0.0, -0.0]),
    ))
    .y_limits(-1.0, 1.0)
    .preflight()
    .unwrap();
    Plot::boxplot(
        CategoryValues::new("category", ["a"; 4]).unwrap(),
        NumericValues::new("value", [0.0; 4]),
    )
    .y_limits(-1.0, 1.0)
    .preflight()
    .unwrap();
    for (lower, estimate, upper) in [(-1.0, -0.5, 0.0), (-2.0, -1.5, -1.0), (-1.0, 0.0, 1.0)] {
        Plot::point_interval(
            CategoryValues::new("category", ["a"]).unwrap(),
            NumericValues::new("estimate", [estimate]),
            lichtung_plot::Bounds::new(
                NumericValues::new("lower", [lower]),
                NumericValues::new("upper", [upper]),
            ),
        )
        .x_limits(0.0, 1.0)
        .preflight()
        .unwrap();
    }
}

#[test]
fn supplied_raincloud_preserves_zero_quartiles_but_not_collapsed_positive_body_or_box() {
    let plot = |width, spread: f64| {
        Plot::raincloud(lichtung_plot::RaincloudInput::geometry(
            lichtung_plot::RaincloudGeometry {
                dimensions: lichtung_plot::RaincloudDimensions {
                    category: "category".into(),
                    value: "value".into(),
                    hue: None,
                },
                groups: vec![lichtung_plot::RaincloudGroup {
                    category: "a".into(),
                    hue: None,
                    density: vec![
                        lichtung_plot::RaincloudDensityPoint {
                            y: -1.0,
                            normalized_width: width,
                        },
                        lichtung_plot::RaincloudDensityPoint {
                            y: 1.0,
                            normalized_width: width,
                        },
                    ],
                    q1: -spread,
                    median: 0.0,
                    q3: spread,
                    points: vec![lichtung_plot::RaincloudPoint {
                        y: 0.0,
                        normalized_strip: 0.5,
                    }],
                }],
            },
        ))
        .y_limits(-1.0, 1.0)
    };
    plot(0.5, 0.0).preflight().unwrap();
    plot(0.5, 0.25).preflight().unwrap();
    for width in [1e-9, 1e-300] {
        assert_collapse(plot(width, 0.25), "raincloud polygon width");
    }
    assert_collapse(
        plot(0.5, 0.25).y_limits(-1e16, 1e16),
        "raincloud polygon support",
    );
    assert_collapse(plot(0.5, 1e-9), "raincloud box polygon");
}

fn line_families(x: &[f64], y: &[f64]) -> [Plot; 3] {
    let x = NumericValues::new("x", x.iter().copied());
    let y = NumericValues::new("y", y.iter().copied());
    let groups = CategoryValues::new("group", std::iter::repeat_n("a", x.len())).unwrap();
    [
        Plot::line(x.clone(), y.clone())
            .x_limits(0.0, 1e9)
            .y_limits(-1.0, 1.0)
            .erase(),
        Plot::trajectory(x.clone(), y.clone(), groups.clone())
            .x_limits(0.0, 1e9)
            .y_limits(-1.0, 1.0)
            .erase(),
        Plot::line_path(x, y, groups)
            .x_limits(0.0, 1e9)
            .y_limits(-1.0, 1.0)
            .erase(),
    ]
}

#[test]
fn line_families_reject_global_source_point_collapse_after_ordering() {
    for x in [&[0.0, 1.0][..], &[1.0, 0.0, 1e9], &[0.0, 1e9, 1.0]] {
        for plot in line_families(x, &vec![0.0; x.len()]) {
            assert_collapse(plot, "line distinct source points");
        }
    }
    assert_collapse(
        Plot::line(
            NumericValues::new("x", [0.0, 1.0]),
            NumericValues::new("y", [0.0, 0.0]),
        )
        .band(lichtung_plot::Bounds::new(
            NumericValues::new("lower", [0.0, 0.0]),
            NumericValues::new("upper", [0.0, 0.0]),
        ))
        .x_limits(0.0, 1e9)
        .y_limits(-1.0, 1.0),
        "line distinct source points",
    );
}

#[test]
fn line_source_identity_preserves_repeats_and_distinct_projected_points() {
    for (x, y) in [([0.0, 1e9], [0.0, 0.0]), ([0.0, 1.0], [-0.5, 0.5])] {
        for plot in line_families(&x, &y) {
            plot.preflight().unwrap();
        }
    }
    for (x, y) in [
        (vec![0.0, -0.0], vec![-0.0, 0.0]),
        (vec![0.0, -0.0, 1e9], vec![0.0, -0.0, 0.0]),
        (vec![0.0, 1e9, -0.0], vec![0.0, 0.0, -0.0]),
    ] {
        Plot::line_path(
            NumericValues::new("x", x),
            NumericValues::new("y", y.clone()),
            CategoryValues::new("group", std::iter::repeat_n("a", y.len())).unwrap(),
        )
        .x_limits(0.0, 1e9)
        .y_limits(-1.0, 1.0)
        .preflight()
        .unwrap();
    }
    Plot::scatter(
        NumericValues::new("x", [0.0, 1.0]),
        NumericValues::new("y", [0.0, 0.0]),
    )
    .x_limits(0.0, 1e9)
    .y_limits(-1.0, 1.0)
    .preflight()
    .unwrap();
}

#[test]
fn forest_log_intervals_retain_raw_nonzero_intent() {
    let plot = |upper| {
        Plot::forest(
            CategoryValues::new("term", ["a"]).unwrap(),
            NumericValues::new("estimate", [1e16]),
            lichtung_plot::Bounds::new(
                NumericValues::new("lower", [1e16]),
                NumericValues::new("upper", [upper]),
            ),
            1e16,
        )
        .scale(lichtung_plot::AxisScale::Log10)
        .x_limits(1e15, 1e17)
    };
    plot(1e16).preflight().unwrap();
    plot(2e16).preflight().unwrap();
    assert_collapse(plot(1e16 + 2.0), "estimate interval");
}
