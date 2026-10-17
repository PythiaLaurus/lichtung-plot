use std::path::Path;

use lichtung_plot::{
    kind, BarInput, Bounds, Canvas, CanvasLegendPolicy, CategoryValues, CoefficientInput, Color,
    DumbbellEndpoints, Error, FacetSpec, HeatmapAnnotations, HistogramInput, NumericFormat,
    NumericNotation, NumericPrecision, NumericValues, Output, Plot, PlotKind, PngCompression,
    PngOptions, PngSizing, RaincloudInput, TextPanel, Theme,
};

use super::support::{png_dimensions, unique_dir};

#[derive(Clone, Copy, Debug)]
enum Sizing {
    Automatic,
    Width(u32),
    Exact(u32, u32),
}

impl Sizing {
    fn plot<K: kind::Marker>(self, plot: Plot<K>) -> Plot<K> {
        match self {
            Self::Automatic => plot,
            Self::Width(width) => plot.width(width),
            Self::Exact(width, height) => plot.size(width, height),
        }
    }

    fn canvas(self, canvas: Canvas) -> Canvas {
        match self {
            Self::Automatic => canvas,
            Self::Width(width) => canvas.width(width),
            Self::Exact(width, height) => canvas.size(width, height),
        }
    }

    fn assert_dimensions(self, dimensions: (u32, u32)) {
        assert!(dimensions.0 > 0 && dimensions.1 > 0);
        match self {
            Self::Automatic => {}
            Self::Width(width) => assert_eq!(dimensions.0, width),
            Self::Exact(width, height) => assert_eq!(dimensions, (width, height)),
        }
    }
}

struct Scene {
    dimensions: (u32, u32),
    svg: String,
}

fn publish(name: &str, render_scene: impl FnOnce(&Path) -> Result<Output, Error>) -> Scene {
    let directory = unique_dir(name);
    let preserved = directory.join("preserved.svg");
    let outlined = directory.join("outlined.svg");
    let preview = directory.join("preview.png");
    let half = directory.join("half.png");
    let repeated = directory.join("repeated.svg");
    let output = render_scene(&preserved).unwrap_or_else(|error| panic!("{name}: {error}"));
    let dimensions = (output.width(), output.height());
    let svg = std::fs::read_to_string(&preserved).unwrap();
    assert_svg_dimensions(&svg, dimensions);
    let output = output
        .svg(&outlined)
        .unwrap()
        .png_preview(&preview)
        .unwrap()
        .png_with_options(
            &half,
            PngOptions {
                sizing: PngSizing::Scale(0.5),
                compression: PngCompression::Fast,
            },
        )
        .unwrap()
        .svg_preserve_text(&repeated)
        .unwrap();
    assert_eq!((output.width(), output.height()), dimensions);
    assert_eq!(std::fs::read_to_string(repeated).unwrap(), svg);
    let outlined = std::fs::read_to_string(outlined).unwrap();
    assert_svg_dimensions(&outlined, dimensions);
    assert!(!outlined.contains("<text"));
    assert!(outlined.contains("<path"));
    let bytes = std::fs::read(preview).unwrap();
    assert_eq!(png_dimensions(&bytes), dimensions);
    let mut reader = png::Decoder::new(std::io::Cursor::new(&bytes))
        .read_info()
        .unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let decoded = reader.next_frame(&mut pixels).unwrap();
    assert_eq!((decoded.width, decoded.height), dimensions);
    assert!(pixels[..decoded.buffer_size()]
        .iter()
        .any(|byte| *byte < 255));
    assert_eq!(
        png_dimensions(&std::fs::read(half).unwrap()),
        (dimensions.0.div_ceil(2), dimensions.1.div_ceil(2))
    );
    std::fs::remove_dir_all(directory).unwrap();
    Scene { dimensions, svg }
}

fn assert_svg_dimensions(svg: &str, dimensions: (u32, u32)) {
    let document = roxmltree::Document::parse(svg).unwrap();
    let root = document.root_element();
    assert!(root.has_tag_name("svg"));
    assert_eq!(
        root.attribute("width"),
        Some(dimensions.0.to_string().as_str())
    );
    assert_eq!(
        root.attribute("height"),
        Some(dimensions.1.to_string().as_str())
    );
    assert_eq!(
        root.attribute("viewBox"),
        Some(format!("0 0 {} {}", dimensions.0, dimensions.1).as_str())
    );
}

fn assert_complete_source(svg: &str, source: &str) {
    let document = roxmltree::Document::parse(svg).unwrap();
    let words = document
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .filter_map(|node| node.text())
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>();
    let required = source.split_whitespace().collect::<Vec<_>>();
    assert!(!required.is_empty());
    assert_eq!(
        words
            .windows(required.len())
            .filter(|words| *words == required)
            .count(),
        1,
        "the complete source must occur exactly once: {source}"
    );
}

fn caption() -> String {
    format!(
        "CaptionBegin {}CaptionEnd",
        "Measurements retain every observation and its scientific identity while complete explanatory prose reflows at the selected width without shrinking fonts omitting words or changing the requested geometry. ".repeat(8)
    )
}

fn scatter() -> Plot<kind::Scatter> {
    Plot::scatter(
        NumericValues::new("dose", [0.0, 1.0, 2.0, 3.0]),
        NumericValues::new("response", [1.0, 3.0, 2.0, 4.0]),
    )
}

fn assert_family_content_roots<K: kind::ExactFamily>(source: Plot<K>, markers: &[&str]) {
    let directory = unique_dir("public-family-content-roots");
    for (sizing, source) in [
        Sizing::Automatic,
        Sizing::Width(801),
        Sizing::Exact(1001, 901),
    ]
    .into_iter()
    .zip([source.clone(), source.clone(), source])
    {
        let destination = directory.join(format!("{sizing:?}-plot.svg"));
        let output = sizing
            .plot(source.clone())
            .svg_preserve_text(&destination)
            .unwrap_or_else(|error| panic!("{:?}, {sizing:?}: {error}", source.kind()));
        let dimensions = (output.width(), output.height());
        sizing.assert_dimensions(dimensions);
        let svg = std::fs::read_to_string(destination).unwrap();
        assert_svg_dimensions(&svg, dimensions);
        for marker in markers {
            assert!(
                svg.contains(&format!(">{marker}")),
                "{:?}, {sizing:?}: missing {marker}",
                source.kind()
            );
        }
        if source.kind() != PlotKind::Matrix {
            let destination = directory.join(format!("{sizing:?}-canvas.svg"));
            let canvas = Canvas::grid(1, 1)
                .unwrap()
                .panel(0, 0, source.clone())
                .unwrap();
            let output = sizing
                .canvas(canvas)
                .svg_preserve_text(&destination)
                .unwrap_or_else(|error| panic!("Canvas {:?}, {sizing:?}: {error}", source.kind()));
            let dimensions = (output.width(), output.height());
            sizing.assert_dimensions(dimensions);
            let svg = std::fs::read_to_string(destination).unwrap();
            assert_svg_dimensions(&svg, dimensions);
            for marker in markers {
                assert!(
                    svg.contains(&format!(">{marker}")),
                    "Canvas {:?}, {sizing:?}: missing {marker}",
                    source.kind()
                );
            }
        }
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one existing typed family census covers each actual numeric surface in all root modes"
)]
fn all_public_families_render_content_requests_with_their_actual_axes_and_chrome() {
    let format = |prefix| NumericFormat::default().with_prefix(prefix).unwrap();
    macro_rules! xy {
        ($plot:expr) => {
            assert_family_content_roots(
                $plot
                    .x_tick_format(format("x="))
                    .y_tick_format(format("y=")),
                &["x=", "y="],
            );
        };
    }
    macro_rules! x {
        ($plot:expr) => {
            assert_family_content_roots($plot.x_tick_format(format("x=")), &["x="]);
        };
    }
    macro_rules! y {
        ($plot:expr) => {
            assert_family_content_roots($plot.y_tick_format(format("y=")), &["y="]);
        };
    }
    let x = NumericValues::new("x", [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let y = NumericValues::new("response", [1.0, 3.0, 2.0, 5.0, 4.0, 6.0]);
    let group = CategoryValues::new("group", ["a", "a", "a", "b", "b", "b"]).unwrap();
    let terms = CategoryValues::new("term", ["a", "b", "c", "d", "e", "f"]).unwrap();
    let hue = CategoryValues::new("treatment", ["one", "two", "one", "two", "one", "two"]).unwrap();
    let bounds = || {
        Bounds::new(
            NumericValues::new("lower", [0.5, 1.0, 2.0, 3.0, 4.0, 5.0]),
            NumericValues::new("upper", [1.5, 3.0, 4.0, 5.0, 6.0, 7.0]),
        )
    };
    xy!(Plot::scatter(x.clone(), y.clone()));
    xy!(Plot::histogram(HistogramInput::samples(x.clone())));
    xy!(Plot::density(x.clone()));
    xy!(Plot::qq(x.clone()));
    x!(Plot::point_interval(terms.clone(), x.clone(), bounds()));
    xy!(Plot::line(x.clone(), y.clone()));
    xy!(Plot::trajectory(x.clone(), y.clone(), group.clone()));
    xy!(Plot::line_path(x.clone(), y.clone(), group.clone()));
    y!(Plot::interaction(
        CategoryValues::new(
            "condition",
            ["before", "during", "after", "before", "during", "after"],
        )
        .unwrap(),
        y.clone(),
        group.clone(),
        bounds(),
    ));
    assert_family_content_roots(
        Plot::coefficient(terms.clone(), CoefficientInput::estimates(x.clone()))
            .x_tick_format(format("x="))
            .estimate_labels(format("v=")),
        &["x=", "v="],
    );
    assert_family_content_roots(
        Plot::forest(terms.clone(), x.clone(), bounds(), 1.0)
            .scale(lichtung_plot::AxisScale::Log10)
            .x_tick_format(
                NumericFormat::new(
                    NumericNotation::Scientific,
                    NumericPrecision::Significant(3),
                )
                .unwrap()
                .with_prefix("x=")
                .unwrap(),
            ),
        &["x="],
    );
    x!(Plot::dumbbell(
        terms.clone(),
        x.clone(),
        y.clone(),
        DumbbellEndpoints::new(
            "Left",
            Color::rgb(20, 80, 140),
            "Right",
            Color::rgb(180, 60, 40),
        )
        .unwrap(),
    ));
    assert_family_content_roots(
        Plot::heatmap(
            CategoryValues::new(
                "column",
                ["first", "second", "third", "first", "second", "third"],
            )
            .unwrap(),
            group.clone(),
            x.clone(),
        )
        .color_label_format(format("c="))
        .annotations(HeatmapAnnotations::Values(format("v="))),
        &["c=", "v="],
    );
    y!(Plot::bar(group.clone(), BarInput::samples(y.clone())));
    x!(Plot::horizontal_bar(
        group.clone(),
        BarInput::samples(y.clone()),
    ));
    y!(Plot::stacked_bar(group.clone(), y.clone(), hue.clone()));
    x!(Plot::stacked_horizontal_bar(group.clone(), y.clone(), hue));
    y!(Plot::boxplot(group.clone(), y.clone()));
    y!(Plot::violin(group.clone(), y.clone()));
    y!(Plot::raincloud(RaincloudInput::samples(
        group.clone(),
        y.clone(),
    )));
    xy!(Plot::regression(x.clone(), y.clone()));
    xy!(Plot::marginal(x.clone(), y.clone()));
    assert_family_content_roots(Plot::matrix(vec![x.clone(), y]), &[]);
    assert_family_content_roots(
        Plot::beeswarm(group, x.clone(), x)
            .x_tick_format(format("x="))
            .color_label_format(format("c=")),
        &["x=", "c="],
    );
    assert_family_content_roots(
        Plot::waterfall(
            terms,
            NumericValues::new("delta", [1.0, -1.0, 2.0, -2.0, 3.0, 0.0]),
            10.0,
        )
        .x_tick_format(format("x="))
        .total_format(format("n=")),
        &["x=", "base n=", "total n="],
    );
}

#[test]
fn standalone_sizing_calls_replace_the_complete_request_before_frozen_publication() {
    let caption = caption();
    let source = scatter()
        .title("Standalone request replacement")
        .caption(&caption);
    let direct = publish("standalone-width", |path| {
        source.clone().width(601).svg_preserve_text(path)
    });
    let replaced = publish("standalone-replaced-width", |path| {
        source
            .clone()
            .size(0, 0)
            .width(481)
            .width(601)
            .svg_preserve_text(path)
    });
    assert_eq!(direct.dimensions.0, 601);
    assert_eq!(replaced.svg, direct.svg);
    assert_complete_source(&direct.svg, &caption);
    let short = publish("standalone-short-caption", |path| {
        scatter()
            .caption("Short caption.")
            .width(601)
            .svg_preserve_text(path)
    });
    assert!(direct.dimensions.1 > short.dimensions.1);
    let exact = publish("standalone-exact-replacement", |path| {
        source
            .width(0)
            .size(700, 1)
            .size(801, 901)
            .svg_preserve_text(path)
    });
    assert_eq!(exact.dimensions, (801, 901));
    assert_complete_source(&exact.svg, &caption);
}

#[test]
fn automatic_roots_expand_for_complete_unbreakable_text_and_exact_width_refuses_it() {
    let heading = "W".repeat(96);
    let ordinary = publish("automatic-short-heading", |path| {
        scatter().title("Short heading").svg_preserve_text(path)
    });
    let expanded = publish("automatic-unbreakable-heading", |path| {
        scatter().title(&heading).svg_preserve_text(path)
    });
    assert!(expanded.dimensions.0 > ordinary.dimensions.0);
    assert_complete_source(&expanded.svg, &heading);
    let directory = unique_dir("exact-width-unbreakable-refusal");
    let destination = directory.join("must-not-exist.svg");
    let error = scatter()
        .title(heading)
        .width(300)
        .svg_preserve_text(&destination)
        .err()
        .unwrap();
    assert!(error.to_string().contains("layout fit"), "{error}");
    assert!(!destination.exists());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn owning_facets_keep_sparse_wrap_and_complete_prose_in_all_public_root_modes() {
    let caption = caption();
    for sizing in [
        Sizing::Automatic,
        Sizing::Width(1001),
        Sizing::Exact(1201, 1401),
    ] {
        let plot = Plot::scatter(
            NumericValues::new("dose", [0.0, 1.0, 0.0, 1.0, 0.0, 1.0]),
            NumericValues::new("response", [1.0, 2.0, 2.0, 1.0, 1.0, 3.0]),
        )
        .facet(
            FacetSpec::wrap(
                CategoryValues::new(
                    "panel",
                    ["First", "First", "Second", "Second", "Third", "Third"],
                )
                .unwrap(),
                2,
            )
            .unwrap(),
        )
        .title("Owning facet title")
        .subtitle("Independent facet subtitle")
        .caption(&caption);
        let scene = publish("public-facet-sizing", |path| {
            sizing.plot(plot).svg_preserve_text(path)
        });
        sizing.assert_dimensions(scene.dimensions);
        for source in [
            "Owning facet title",
            "Independent facet subtitle",
            "Panel = First",
            "Panel = Second",
            "Panel = Third",
            &caption,
        ] {
            assert_complete_source(&scene.svg, source);
        }
        assert_eq!(scene.svg.matches("<clipPath ").count(), 3);
    }
}

#[test]
fn matrix_roots_keep_native_cells_and_their_hue_guide_in_all_public_root_modes() {
    let caption = caption();
    for sizing in [
        Sizing::Automatic,
        Sizing::Width(801),
        Sizing::Exact(1001, 1201),
    ] {
        let plot = Plot::matrix(vec![
            NumericValues::new("alpha_value", [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
            NumericValues::new("beta_value", [2.0, 1.0, 4.0, 3.0, 6.0, 5.0]),
        ])
        .hue(
            CategoryValues::new(
                "treatment",
                [
                    "Control", "Control", "Control", "Exposed", "Exposed", "Exposed",
                ],
            )
            .unwrap(),
        )
        .title("Native matrix sizing")
        .caption(&caption);
        let scene = publish(&format!("public-matrix-sizing-{sizing:?}"), |path| {
            sizing.plot(plot).svg_preserve_text(path)
        });
        sizing.assert_dimensions(scene.dimensions);
        for source in [
            "Native matrix sizing",
            "Treatment",
            "Control",
            "Exposed",
            &caption,
        ] {
            assert_complete_source(&scene.svg, source);
        }
        assert!(
            scene.svg.contains(">Alpha value</text>") && scene.svg.contains(">Beta value</text>")
        );
        assert_eq!(scene.svg.matches("<clipPath ").count(), 4);
    }
}

fn mixed_canvas(caption: &str, replace_children: bool) -> Canvas {
    let child = |name: &str| {
        let plot = scatter()
            .x_tick_format(
                NumericFormat::default()
                    .with_prefix(format!("{name}:"))
                    .unwrap(),
            )
            .hue(
                CategoryValues::new("treatment", ["Control", "Exposed", "Control", "Exposed"])
                    .unwrap(),
            )
            .title(format!("{name} panel title"))
            .subtitle(format!("{name} panel subtitle"))
            .caption(format!("{name} local caption"));
        if replace_children {
            plot.size(0, 0).width(0)
        } else {
            plot
        }
    };
    Canvas::grid(2, 2)
        .unwrap()
        .panel(0, 0, child("First"))
        .unwrap()
        .panel(0, 1, child("Second"))
        .unwrap()
        .text_panel(
            1,
            0,
            TextPanel::new(
                "Interpretation",
                "A complete prose cell alongside two scientific panels. 0.125% remains literal caller text.",
            )
            .unwrap(),
        )
        .unwrap()
        .column_header(0, 2, "Shared scientific observations")
        .unwrap()
        .x_range_group([(0, 0), (0, 1)])
        .unwrap()
        .legend_policy(CanvasLegendPolicy::Shared)
        .title("Canvas root title")
        .subtitle("Independent Canvas subtitle")
        .caption(caption)
}

#[test]
fn mixed_canvas_owns_all_modes_and_ignores_standalone_child_requests() {
    let caption = caption();
    for sizing in [
        Sizing::Automatic,
        Sizing::Width(1101),
        Sizing::Exact(1301, 1401),
    ] {
        let scene = publish("public-mixed-canvas", |path| {
            sizing
                .canvas(mixed_canvas(&caption, false))
                .svg_preserve_text(path)
        });
        sizing.assert_dimensions(scene.dimensions);
        let replaced = publish("public-mixed-canvas-child-requests", |path| {
            sizing
                .canvas(mixed_canvas(&caption, true))
                .svg_preserve_text(path)
        });
        assert_eq!(replaced.svg, scene.svg, "{sizing:?}");
        for source in [
            "Canvas root title",
            "Independent Canvas subtitle",
            "First panel title",
            "First panel subtitle",
            "First local caption",
            "Second panel title",
            "Second panel subtitle",
            "Second local caption",
            "Interpretation",
            "A complete prose cell alongside two scientific panels.",
            "0.125% remains literal caller text.",
            "Shared scientific observations",
            "Treatment",
            "Control",
            "Exposed",
            &caption,
        ] {
            assert_complete_source(&scene.svg, source);
        }
        assert_eq!(scene.svg.matches("<clipPath ").count(), 2);
        assert!(scene.svg.contains(">First:") && scene.svg.contains(">Second:"));
    }
}

#[test]
fn all_text_canvas_reflows_complete_content_and_replaces_root_requests() {
    let body = caption();
    let canvas = || {
        Canvas::grid(1, 1)
            .unwrap()
            .text_panel(0, 0, TextPanel::new("Explanation", &body).unwrap())
            .unwrap()
            .title("Canvas prose sizing")
            .caption("Independent root footnote.")
    };
    for sizing in [
        Sizing::Automatic,
        Sizing::Width(601),
        Sizing::Exact(801, 1101),
    ] {
        let scene = publish("public-text-canvas", |path| {
            sizing.canvas(canvas()).svg_preserve_text(path)
        });
        sizing.assert_dimensions(scene.dimensions);
        assert_complete_source(&scene.svg, &body);
        assert_complete_source(&scene.svg, "Independent root footnote.");
    }
    let direct = publish("public-text-canvas-width", |path| {
        canvas().width(601).svg_preserve_text(path)
    });
    let replaced = publish("public-text-canvas-replaced-width", |path| {
        canvas()
            .size(0, 0)
            .width(481)
            .width(601)
            .svg_preserve_text(path)
    });
    assert_eq!(direct.svg, replaced.svg);
    let exact = publish("public-text-canvas-replaced-size", |path| {
        canvas()
            .width(0)
            .size(700, 1)
            .size(801, 1101)
            .svg_preserve_text(path)
    });
    assert_eq!(exact.dimensions, (801, 1101));
    assert_complete_source(&exact.svg, &body);
}

#[test]
fn valid_svg_keeps_png_axis_and_area_refusals_local_in_all_root_modes() {
    let directory = unique_dir("public-sizing-png-local");
    for (name, plot, expected) in [
        (
            "automatic-axis",
            scatter().title("W".repeat(3000)),
            "scene dimensions",
        ),
        ("width-axis", scatter().width(33_000), "scene dimensions"),
        (
            "exact-axis",
            scatter().size(33_000, 200),
            "scene dimensions",
        ),
        (
            "automatic-area",
            scatter().title("W".repeat(900)),
            "scene work",
        ),
        ("width-area", scatter().width(16_000), "scene work"),
        ("exact-area", scatter().size(10_000, 10_001), "scene work"),
    ] {
        let preserved = directory.join(format!("{name}.svg"));
        let outlined = directory.join(format!("{name}-outlined.svg"));
        let refused = directory.join(format!("{name}.png"));
        let output = plot.svg_preserve_text(&preserved).unwrap();
        let dimensions = (output.width(), output.height());
        let svg = std::fs::read_to_string(&preserved).unwrap();
        assert_svg_dimensions(&svg, dimensions);
        let output = output.svg(&outlined).unwrap();
        assert_eq!((output.width(), output.height()), dimensions);
        let error = output.png_preview(&refused).err().unwrap();
        assert!(error.to_string().contains(expected), "{name}: {error}");
        assert!(!refused.exists());
        assert_eq!(std::fs::read_to_string(preserved).unwrap(), svg);
        assert_svg_dimensions(&std::fs::read_to_string(outlined).unwrap(), dimensions);
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn default_png_target_rounding_keeps_axis_and_area_boundaries_independent() {
    let directory = unique_dir("public-default-png-boundaries");
    // At 300 DPI, targets round (25 * source) / 8 to the nearest integer.
    // 10_485 -> 32_766 fits the axis gate; 10_486 -> 32_769 does not.
    // 3_200² -> 10_000² is exactly the target-area limit; the next row exceeds it.
    for (name, dimensions, target) in [
        ("axis", (10_485, 240), (32_766, 750)),
        ("area", (3_200, 3_200), (10_000, 10_000)),
    ] {
        let preserved = directory.join(format!("{name}.svg"));
        let raster = directory.join(format!("{name}.png"));
        let output = scatter()
            .size(dimensions.0, dimensions.1)
            .svg_preserve_text(&preserved)
            .unwrap()
            .png(&raster)
            .unwrap();
        assert_eq!((output.width(), output.height()), dimensions);
        assert_eq!(png_dimensions(&std::fs::read(raster).unwrap()), target);
        assert_svg_dimensions(&std::fs::read_to_string(preserved).unwrap(), dimensions);
    }
    for (name, dimensions, expected) in [
        ("axis-refusal", (10_486, 240), "computed PNG dimensions"),
        ("area-refusal", (3_200, 3_201), "computed PNG work"),
    ] {
        let preserved = directory.join(format!("{name}.svg"));
        let refused = directory.join(format!("{name}.png"));
        let output = scatter()
            .size(dimensions.0, dimensions.1)
            .svg_preserve_text(&preserved)
            .unwrap();
        assert_eq!((output.width(), output.height()), dimensions);
        let error = output.png(&refused).err().unwrap();
        assert!(error.to_string().contains(expected), "{name}: {error}");
        assert!(!refused.exists());
        assert_svg_dimensions(&std::fs::read_to_string(preserved).unwrap(), dimensions);
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn large_canonical_svg_and_actual_outlined_parser_payload_stay_format_local_in_all_modes() {
    const MARKS: usize = 900_000;
    const PARSER_LIMIT: usize = 64 * 1024 * 1024;
    let base = Theme::publication();
    let tick_label = base
        .typography()
        .axes()
        .tick_label()
        .clone()
        .with_font_size(8)
        .unwrap();
    let axes = base
        .typography()
        .axes()
        .clone()
        .with_tick_label(tick_label)
        .unwrap();
    let typography = base.typography().clone().with_axes(axes).unwrap();
    let theme = base
        .customize()
        .with_typography(typography)
        .build()
        .unwrap();
    let source = Plot::scatter(
        NumericValues::new("x", vec![0.0; MARKS]),
        NumericValues::new("y", vec![1.0; MARKS]),
    )
    .theme(theme);
    let directory = unique_dir("public-sizing-large-parser-payload");
    for (index, sizing) in [
        Sizing::Automatic,
        Sizing::Width(10_000),
        Sizing::Exact(10_000, 8_000),
    ]
    .into_iter()
    .enumerate()
    {
        let preserved = directory.join(format!("{index}.svg"));
        let outlined = directory.join(format!("{index}-outlined.svg"));
        let repeated = directory.join(format!("{index}-repeated.svg"));
        let refused = directory.join(format!("{index}.png"));
        let output = sizing
            .plot(source.clone())
            .svg_preserve_text(&preserved)
            .unwrap();
        let dimensions = (output.width(), output.height());
        sizing.assert_dimensions(dimensions);
        assert!(dimensions.0 <= 32_768 && dimensions.1 <= 32_768);
        assert!(u64::from(dimensions.0) * u64::from(dimensions.1) <= 100_000_000);
        let svg = std::fs::read_to_string(&preserved).unwrap();
        assert!(
            svg.len() > PARSER_LIMIT,
            "{sizing:?}: actual canonical bytes must cross the PNG-only gate"
        );
        assert_eq!(
            svg.matches("<circle ").count(),
            MARKS,
            "coincident marks cannot disappear"
        );
        let output = output
            .svg(&outlined)
            .unwrap()
            .svg_preserve_text(&repeated)
            .unwrap();
        assert_eq!((output.width(), output.height()), dimensions);
        assert!(std::fs::metadata(outlined).unwrap().len() > PARSER_LIMIT as u64);
        assert_eq!(std::fs::read_to_string(repeated).unwrap(), svg);
        let error = output.png_preview(&refused).err().unwrap();
        assert!(
            error.to_string().contains("parser input size"),
            "{sizing:?}: {error}"
        );
        assert!(!refused.exists());
    }
    std::fs::remove_dir_all(directory).unwrap();
}
