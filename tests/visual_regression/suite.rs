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
        &["Matrix structural title", "Group"]
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
