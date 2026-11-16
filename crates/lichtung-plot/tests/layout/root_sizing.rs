use std::path::Path;

use lichtung_plot::{
    kind, Canvas, CanvasLegendPolicy, CategoryValues, Error, NumericFormat, NumericValues, Output,
    Plot, TextPanel,
};

#[path = "../support/visual.rs"]
mod support;

use support::unique_dir;

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
fn automatic_roots_expand_for_complete_unbreakable_text_and_capped_width_refuses_it() {
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
        .title(&heading)
        .width(300)
        .svg_preserve_text(&destination)
        .err()
        .unwrap();
    assert!(error.to_string().contains("layout fit"), "{error}");
    assert!(!destination.exists());
    std::fs::write(&destination, "existing destination").unwrap();
    let error = scatter()
        .title(heading)
        .max_width(300)
        .svg_preserve_text(&destination)
        .err()
        .unwrap();
    assert!(error.to_string().contains("layout fit"), "{error}");
    assert_eq!(
        std::fs::read_to_string(destination).unwrap(),
        "existing destination"
    );
    std::fs::remove_dir_all(directory).unwrap();
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
            plot.size(0, 0).width(0).max_width(0)
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
            .max_width(1101)
            .svg_preserve_text(path)
    });
    let replaced = publish("public-mixed-canvas-child-requests", |path| {
        mixed_canvas(&caption, true)
            .max_width(1101)
            .svg_preserve_text(path)
    });
    assert!(scene.dimensions.0 <= 1101);
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
