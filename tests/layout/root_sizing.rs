use std::path::Path;

use lichtung_plot::{
    kind, Canvas, CanvasLegendPolicy, CategoryValues, Error, FacetSpec, NumericFormat,
    NumericValues, Output, Plot, TextPanel,
};

use super::support::unique_dir;

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
    let destination = directory.join("plot.svg");
    let output = render_scene(&destination).unwrap_or_else(|error| panic!("{name}: {error}"));
    let dimensions = (output.width(), output.height());
    let svg = std::fs::read_to_string(destination).unwrap();
    assert_svg_dimensions(&svg, dimensions);
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

#[test]
fn standalone_sizing_calls_replace_the_complete_request_before_publication() {
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
fn matrix_root_keeps_native_cells_hue_guide_and_complete_prose() {
    let caption = caption();
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
    let scene = publish("public-matrix-sizing", |path| plot.svg_preserve_text(path));
    for source in [
        "Native matrix sizing",
        "Treatment",
        "Control",
        "Exposed",
        &caption,
    ] {
        assert_complete_source(&scene.svg, source);
    }
    assert!(scene.svg.contains(">Alpha value</text>") && scene.svg.contains(">Beta value</text>"));
    assert_eq!(scene.svg.matches("<clipPath ").count(), 4);
}

fn mixed_canvas(caption: &str, replace_children: bool) -> Canvas {
    let child = |name: &str| {
        let plot = scatter()
            .x_tick_format(NumericFormat::default().prefix(format!("{name}:")).unwrap())
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
        .panel_span(0, 1, 2, 1, child("Second"))
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
fn mixed_canvas_keeps_complete_content_and_ignores_standalone_child_requests() {
    let caption = caption();
    let scene = publish("public-mixed-canvas", |path| {
        mixed_canvas(&caption, false)
            .width(1101)
            .svg_preserve_text(path)
    });
    let replaced = publish("public-mixed-canvas-child-requests", |path| {
        mixed_canvas(&caption, true)
            .width(1101)
            .svg_preserve_text(path)
    });
    assert_eq!(scene.dimensions.0, 1101);
    assert_eq!(replaced.svg, scene.svg);
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
fn png_scene_limits_do_not_prevent_svg_publication() {
    let directory = unique_dir("public-sizing-png-local");
    for (name, plot, expected) in [
        ("axis", scatter().size(33_000, 200), "scene dimensions"),
        ("area", scatter().size(10_000, 10_001), "scene work"),
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
