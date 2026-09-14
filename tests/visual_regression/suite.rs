use lichtung_plot::{BarInput, CategoryValues, NumericValues, Plot};
#[path = "../support/visual.rs"]
mod support;

use support::*;

#[test]
fn canvas_root_title_block_odd_size_snapshot() {
    let (x, y) = fixture_values();
    let out = tempdir().join("canvas-root-title-block.svg");
    lichtung_plot::Canvas::grid(1, 1)
        .unwrap()
        .panel(
            0,
            0,
            Plot::scatter(x, y)
                .title("Panel title")
                .subtitle("Panel subtitle"),
        )
        .unwrap()
        .title("Canvas title\nShort")
        .subtitle("Independent subtitle\n\nFinal line")
        .caption("Start caption\nSecond line")
        .size(433, 501)
        .svg_preserve_text(&out)
        .unwrap();
    let svg = std::fs::read_to_string(out).unwrap();
    insta::assert_snapshot!(normalize_embedded_font(&svg));
}

#[test]
fn root_title_block_explicit_odd_size_snapshot() {
    let (x, y) = fixture_values();
    let out = tempdir().join("root-title-block.svg");
    Plot::scatter(x, y)
        .title("Root title\nShort")
        .subtitle("Independent subtitle\n\nFinal line")
        .caption("Start caption\nSecond line")
        .size(333, 401)
        .svg_preserve_text(&out)
        .unwrap();
    let svg = std::fs::read_to_string(out).unwrap();
    insta::assert_snapshot!(normalize_embedded_font(&svg));
}

#[test]
fn scatter_smoke_snapshot() {
    let (x, y) = fixture_values();
    let dir = tempdir();
    let out = dir.join("scatter.svg");
    Plot::scatter(x, y)
        .size(320, 240)
        .svg_preserve_text(&out)
        .expect("render must succeed");
    let bytes = std::fs::read(&out).expect("file must exist");
    insta::assert_snapshot!(normalize_embedded_font(&String::from_utf8_lossy(&bytes)));
}

#[test]
fn native_path_snapshots() {
    let x = NumericValues::new("x", [0.0, 1.0, 2.0]);
    let y = NumericValues::new("y", [2.0, 1.0, 3.0]);
    let group = CategoryValues::new("group", ["a", "a", "a"]).unwrap();
    let paths = [
        ("line", Plot::line(x.clone(), y.clone()).erase()),
        (
            "trajectory",
            Plot::trajectory(x.clone(), y.clone(), group.clone()).erase(),
        ),
        (
            "line_path",
            Plot::line_path(x, y.clone(), group.clone()).erase(),
        ),
        (
            "categorical_trajectory",
            Plot::trajectory(
                CategoryValues::new("condition", ["before", "during", "after"]).unwrap(),
                y,
                group,
            )
            .erase(),
        ),
    ];
    for (name, plot) in paths {
        let out = tempdir().join(format!("native-{name}.svg"));
        plot.size(400, 300).svg_preserve_text(&out).unwrap();
        let svg = std::fs::read_to_string(out).unwrap();
        insta::assert_snapshot!(format!("native_{name}"), normalize_embedded_font(&svg));
    }
}

#[test]
fn native_distribution_reference_snapshots() {
    let x = NumericValues::new("sample", [1.0, 2.0, 4.0, 5.0, 7.0, 8.0]);
    let plots = [
        (
            "histogram",
            Plot::histogram(lichtung_plot::HistogramInput::samples(x.clone()))
                .bins(lichtung_plot::HistogramBins::Count(8))
                .stat(lichtung_plot::HistogramStat::Density)
                .composite(lichtung_plot::HistogramComposite::KdeAndRug(
                    lichtung_plot::HistogramKde::Internal,
                ))
                .erase(),
        ),
        (
            "density",
            Plot::density(x.clone())
                .hue(CategoryValues::new("series", ["a", "a", "a", "b", "b", "b"]).unwrap())
                .erase(),
        ),
        (
            "qq",
            Plot::qq(x.clone())
                .x_limits(-0.75, 0.75)
                .y_limits(2.0, 6.0)
                .erase(),
        ),
        (
            "regression",
            Plot::regression(
                x,
                NumericValues::new("response", [1.0, 4.0, 2.0, 8.0, 6.0, 9.0]),
            )
            .erase(),
        ),
    ];
    for (name, plot) in plots {
        let out = tempdir().join(format!("native-{name}.svg"));
        plot.size(481, 401)
            .title("Native distribution and reference")
            .x_label("First axis line\nSecond axis line\nThird axis line")
            .caption("Independent caption\nSecond caption line")
            .svg_preserve_text(&out)
            .unwrap()
            .png_preview(out.with_extension("png"))
            .unwrap();
        assert_eq!(
            png_dimensions(&std::fs::read(out.with_extension("png")).unwrap()),
            (481, 401)
        );
        let svg = std::fs::read_to_string(out).unwrap();
        insta::assert_snapshot!(format!("native_{name}"), normalize_embedded_font(&svg));
    }
}

#[test]
fn matrix_clip_structure_snapshot() {
    let x = NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]);
    let y = NumericValues::new("y", [4.0, 3.0, 2.0, 1.0]);
    let group = CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap();
    let dir = tempdir();
    let out = dir.join("matrix-structure.svg");
    let chart = Plot::matrix(vec![x, y])
        .group(group.clone())
        .hue(group)
        .title("Matrix structural title");

    chart.size(360, 280).svg_preserve_text(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Matrix structural title", "Group", "X", "Y", "0", "5"]
    ));
}

#[test]
fn native_estimate_display_snapshots() {
    use lichtung_plot::{
        AxisScale, AxisTick, Bounds, CoefficientInput, DumbbellEndpoints, EstimateStatus,
        Significance, SignificanceMark,
    };
    let terms = CategoryValues::new(
        "term",
        [
            "a deliberately long category",
            "unavailable row",
            "last visit",
        ],
    )
    .unwrap();
    let values = NumericValues::nullable("estimate", [Some(2.0), None, Some(8.0)]);
    let bounds = Bounds::new(
        NumericValues::nullable("lower", [Some(1.0), None, Some(4.0)]),
        NumericValues::nullable("upper", [Some(4.0), None, Some(12.0)]),
    );
    let statuses = [
        EstimateStatus::Estimated,
        EstimateStatus::NotEstimable,
        EstimateStatus::Estimated,
    ];
    let plots = [
        (
            "point_interval",
            Plot::point_interval(terms.clone(), values.clone(), bounds.clone())
                .estimate_status(statuses)
                .zero_line()
                .x_limits(-2.0, 15.0)
                .erase(),
        ),
        (
            "coefficient",
            Plot::coefficient(terms.clone(), CoefficientInput::estimates(values.clone()))
                .estimate_status(statuses)
                .significance([Significance::at("last visit", SignificanceMark::TwoStars)])
                .erase(),
        ),
        (
            "forest",
            Plot::forest(terms.clone(), values, bounds, 1.0)
                .estimate_status(statuses)
                .scale(AxisScale::Log10)
                .x_limits(0.1, 100.0)
                .x_ticks(vec![
                    AxisTick::new(0.1, "0.1"),
                    AxisTick::new(1.0, "1"),
                    AxisTick::new(10.0, "10"),
                    AxisTick::new(100.0, "100"),
                ])
                .erase(),
        ),
        (
            "dumbbell",
            Plot::dumbbell(
                terms,
                NumericValues::new("before", [2.0, 4.0, 8.0]),
                NumericValues::new("after", [4.0, 3.0, 6.0]),
                DumbbellEndpoints::new("Before", (20, 80, 140), "After", (180, 60, 40)).unwrap(),
            )
            .erase(),
        ),
    ];
    for (name, plot) in plots {
        let out = tempdir().join(format!("native-{name}.svg"));
        plot.size(641, 401)
            .title("Native estimate displays")
            .x_label("First axis line\nSecond axis line\nThird axis line")
            .caption("Independent caption\nSecond caption line")
            .svg_preserve_text(&out)
            .unwrap()
            .png_preview(out.with_extension("png"))
            .unwrap();
        assert_eq!(
            png_dimensions(&std::fs::read(out.with_extension("png")).unwrap()),
            (641, 401)
        );
        let svg = std::fs::read_to_string(out).unwrap();
        insta::assert_snapshot!(format!("native_{name}"), normalize_embedded_font(&svg));
    }
}

#[test]
fn native_bar_family_snapshots() {
    use lichtung_plot::{Bounds, Significance, SignificanceMark, SignificanceTarget};
    let categories = CategoryValues::new(
        "category",
        [
            "a long category",
            "second visit",
            "last visit",
            "a long category",
            "second visit",
            "last visit",
        ],
    )
    .unwrap();
    let values = NumericValues::new("value", [-3.0, 5.0, 2.0, -2.0, 7.0, -4.0]);
    let series = CategoryValues::new(
        "series",
        ["first", "first", "first", "second", "second", "second"],
    )
    .unwrap();
    let input = BarInput::estimates_with_error_bars(
        values.clone(),
        Bounds::new(
            NumericValues::new("lower", [-4.0, 4.0, 1.0, -3.0, 6.0, -5.0]),
            NumericValues::new("upper", [-2.0, 6.0, 3.0, -1.0, 8.0, -3.0]),
        ),
    );
    let attached = || {
        [
            Significance::at(
                SignificanceTarget::category_series("a long category", "first"),
                SignificanceMark::TwoStars,
            ),
            Significance::at("second visit", SignificanceMark::PValue(0.01)),
        ]
    };
    let pairwise = || {
        [Significance::between(
            "second visit",
            "last visit",
            SignificanceMark::OneStar,
        )]
    };
    let plots = [
        (
            "bar",
            Plot::bar(categories.clone(), input.clone())
                .series(series.clone())
                .significance(attached())
                .significance(pairwise())
                .y_limits(-10.0, 20.0)
                .erase(),
        ),
        (
            "horizontal_bar",
            Plot::horizontal_bar(categories.clone(), input)
                .series(series.clone())
                .significance(attached())
                .significance(pairwise())
                .color_by_sign()
                .zero_line()
                .x_limits(-10.0, 20.0)
                .erase(),
        ),
        (
            "stacked_bar",
            Plot::stacked_bar(categories.clone(), values.clone(), series.clone())
                .significance(pairwise())
                .erase(),
        ),
        (
            "stacked_horizontal_bar",
            Plot::stacked_horizontal_bar(categories, values, series)
                .significance(pairwise())
                .erase(),
        ),
    ];
    for (name, plot) in plots {
        let out = tempdir().join(format!("native-{name}.svg"));
        plot.size(641, 401)
            .title("Native signed bar families")
            .x_label("First axis line\nSecond axis line\nThird axis line")
            .caption("Independent caption\nSecond caption line")
            .svg_preserve_text(&out)
            .unwrap()
            .png_preview(out.with_extension("png"))
            .unwrap();
        assert_eq!(
            png_dimensions(&std::fs::read(out.with_extension("png")).unwrap()),
            (641, 401)
        );
        let svg = std::fs::read_to_string(out).unwrap();
        insta::assert_snapshot!(format!("native_{name}"), normalize_embedded_font(&svg));
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "three complete family fixtures include caller-supplied Raincloud geometry"
)]
fn native_categorical_distribution_snapshots() {
    use lichtung_plot::{
        RaincloudDensityPoint, RaincloudPoint, RawOverlay, Significance, SignificanceMark,
    };
    let categories = CategoryValues::new(
        "category",
        [
            "after_visit",
            "after_visit",
            "before_visit",
            "before_visit",
            "after_visit",
            "after_visit",
        ],
    )
    .unwrap();
    let values = NumericValues::new("value", [1.25, 4.5, -2.5, 3.125, 2.25, 7.75]);
    let series = CategoryValues::new(
        "series",
        ["first", "first", "first", "first", "second", "second"],
    )
    .unwrap();
    let comparison = || {
        [Significance::between(
            "after_visit",
            "before_visit",
            SignificanceMark::PValue(0.0123),
        )]
    };
    let plots = [
        (
            "boxplot",
            Plot::boxplot(categories.clone(), values.clone())
                .series(series.clone())
                .raw_overlay(RawOverlay::Jittered)
                .significance(comparison())
                .erase(),
        ),
        (
            "violin",
            Plot::violin(categories, values)
                .series(series)
                .raw_overlay(RawOverlay::Points)
                .significance(comparison())
                .erase(),
        ),
        (
            "raincloud",
            Plot::raincloud(lichtung_plot::RaincloudInput::geometry(
                lichtung_plot::RaincloudGeometry {
                    dimensions: lichtung_plot::RaincloudDimensions {
                        category: "category".into(),
                        value: "value".into(),
                        hue: Some("series".into()),
                    },
                    groups: ["after_visit", "before_visit"]
                        .into_iter()
                        .enumerate()
                        .map(|(index, category)| {
                            let offset = f64::from(u32::try_from(index).unwrap()) * 2.0;
                            lichtung_plot::RaincloudGroup {
                                category: category.into(),
                                hue: Some(if index == 0 { "first" } else { "second" }.into()),
                                density: vec![
                                    RaincloudDensityPoint {
                                        y: offset,
                                        normalized_width: 0.0,
                                    },
                                    RaincloudDensityPoint {
                                        y: offset + 3.25,
                                        normalized_width: 0.8,
                                    },
                                    RaincloudDensityPoint {
                                        y: offset + 6.5,
                                        normalized_width: 0.0,
                                    },
                                ],
                                q1: offset + 1.25,
                                median: offset + 3.25,
                                q3: offset + 5.5,
                                points: vec![
                                    RaincloudPoint {
                                        y: offset + 2.25,
                                        normalized_strip: 0.25,
                                    },
                                    RaincloudPoint {
                                        y: offset + 4.5,
                                        normalized_strip: 0.75,
                                    },
                                ],
                            }
                        })
                        .collect(),
                },
            ))
            .erase(),
        ),
    ];
    for (name, plot) in plots {
        let out = tempdir().join(format!("native-{name}.svg"));
        plot.size(641, 401)
            .title("Native categorical distributions")
            .labels(
                lichtung_plot::Labels::humanized().alias("after_visit", "A longer follow-up visit"),
            )
            .x_label("First axis line\nSecond axis line\nThird axis line")
            .caption("Independent caption\nSecond caption line")
            .svg_preserve_text(&out)
            .unwrap()
            .png_preview(out.with_extension("png"))
            .unwrap();
        assert_eq!(
            png_dimensions(&std::fs::read(out.with_extension("png")).unwrap()),
            (641, 401)
        );
        let svg = std::fs::read_to_string(out).unwrap();
        insta::assert_snapshot!(format!("native_{name}"), normalize_embedded_font(&svg));
    }
}

#[test]
fn native_interaction_snapshot() {
    use lichtung_plot::{Bounds, InteractionStyle, Labels, LinePattern, PointShape};
    let out = tempdir().join("native-interaction.svg");
    Plot::interaction(
        CategoryValues::new(
            "condition",
            [
                "after_visit",
                "before_visit",
                "last_visit",
                "after_visit",
                "last_visit",
            ],
        )
        .unwrap(),
        NumericValues::new("estimate", [2.0, 3.0, 4.0, 5.0, 6.0]),
        CategoryValues::new("group", ["one", "one", "one", "two", "two"]).unwrap(),
        Bounds::new(
            NumericValues::new("lower", [1.0, 2.0, 3.0, 4.0, 5.0]),
            NumericValues::new("upper", [3.0, 4.0, 5.0, 6.0, 7.0]),
        ),
    )
    .hue(CategoryValues::new("hue", ["first", "first", "first", "second", "second"]).unwrap())
    .styles([
        InteractionStyle::new("two")
            .shape(PointShape::TriangleUp)
            .pattern(LinePattern::Dashed),
        InteractionStyle::new("one"),
    ])
    .labels(Labels::humanized().alias("after_visit", "A longer follow-up visit"))
    .size(641, 401)
    .title("Native interaction profiles")
    .x_label("First axis line\nSecond axis line\nThird axis line")
    .caption("Independent caption\nSecond caption line")
    .svg_preserve_text(&out)
    .unwrap()
    .png_preview(out.with_extension("png"))
    .unwrap();
    assert_eq!(
        png_dimensions(&std::fs::read(out.with_extension("png")).unwrap()),
        (641, 401)
    );
    let svg = std::fs::read_to_string(&out).unwrap();
    let panel = parse_first_clip_rect(&svg);
    assert_eq!(
        [panel.x, panel.y, panel.width, panel.height].map(f64::to_bits),
        [85.0, 51.0, 420.698_242_187_5, 175.0].map(f64::to_bits)
    );
    assert!(SvgRect {
        x: 0.0,
        y: 0.0,
        width: 641.0,
        height: 401.0
    }
    .contains(panel));
    let text = parse_text_elements(&svg);
    assert!(text_y(&text, "Native interaction profiles") < panel.y);
    let caption = text_y(&text, "Independent caption");
    for label in ["First axis line", "Second axis line", "Third axis line"] {
        let y = text_y(&text, label);
        assert!(y > panel.y + panel.height && y < caption);
    }
    insta::assert_snapshot!("native_interaction", normalize_embedded_font(&svg));
}

#[test]
fn beeswarm_clip_structure_snapshot() {
    let category = CategoryValues::new("category", ["first", "first", "second", "second"]).unwrap();
    let value = NumericValues::new("value", [-2.0, -1.0, 1.0, 2.0]);
    let color = NumericValues::new("color", [0.1, 0.3, 0.7, 1.0]);
    let dir = tempdir();
    let out = dir.join("beeswarm-structure.svg");
    let chart = Plot::beeswarm(category, value, color).title("Beeswarm structural title");

    chart.size(360, 280).svg_preserve_text(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Beeswarm structural title", "Color", "0.100", "1.000"]
    ));
}

#[test]
fn horizontal_bar_clip_structure_snapshot() {
    let category =
        CategoryValues::new("category", ["a deliberately long category", "short"]).unwrap();
    let value = NumericValues::new("value", [2.0, 4.0]);
    let dir = tempdir();
    let out = dir.join("horizontal-structure.svg");
    let chart = Plot::horizontal_bar(category, BarInput::estimates(value))
        .title("Horizontal structural title")
        .size(360, 280);

    chart.svg_preserve_text(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &[
            "Horizontal structural title",
            "A ",
            "deliberately ",
            "long ",
            "category"
        ]
    ));
}

#[test]
fn marginal_clip_structure_snapshot() {
    let x = NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]);
    let y = NumericValues::new("y", [1.0, 4.0, 2.0, 3.0]);
    let dir = tempdir();
    let out = dir.join("marginal-structure.svg");
    let chart = Plot::marginal(x, y)
        .title("Marginal structural title")
        .size(360, 280);

    chart.svg_preserve_text(&out).unwrap();
    let svg = std::fs::read_to_string(out).unwrap();

    insta::assert_snapshot!(specialized_svg_structure(
        &svg,
        &["Marginal structural title"]
    ));
}
