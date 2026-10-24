use super::*;

#[test]
fn guide_ellipsis_retains_original_metadata_and_replays_one_selected_view_across_outputs() {
    use lichtung_plot::{
        CategoryValues, GuideTextRole, TextBounds, TextElision, TextEllipsis, TextFailure,
        TextLayout, TextOverflow,
    };
    let directory = unique_dir("derived-guide-view");
    let first = directory.join("first.svg");
    let second = directory.join("second.svg");
    let original = "A very long guide identity with an important ending";
    let output = Plot::scatter(
        NumericValues::new("x", [0.0, 1.0]),
        NumericValues::new("y", [1.0, 0.0]),
    )
    .hue(CategoryValues::new("cohort", [original, original]).unwrap())
    .labels(lichtung_plot::Labels::raw())
    .guide_text_layout(
        GuideTextRole::Entry,
        TextLayout::default()
            .bounds(TextBounds::new(Some(70.0), None, None).unwrap())
            .overflow(TextOverflow::ellipsis(
                TextEllipsis::new(TextElision::Middle),
                TextFailure::Error,
            )),
    )
    .size(500, 350)
    .svg_preserve_text(&first)
    .unwrap();
    output
        .svg(directory.join("paths.svg"))
        .unwrap()
        .png_preview(directory.join("preview.png"))
        .unwrap()
        .svg_preserve_text(&second)
        .unwrap();
    let bytes = std::fs::read(first).unwrap();
    assert_eq!(bytes, std::fs::read(second).unwrap());
    let source = std::str::from_utf8(&bytes).unwrap();
    let document = roxmltree::Document::parse(source).unwrap();
    assert!(document
        .descendants()
        .any(|node| node.attribute("data-original-source") == Some(original)));
    let painted = document
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .filter_map(|node| node.text())
        .collect::<Vec<_>>();
    assert!(!painted.contains(&original));
    assert!(painted
        .iter()
        .any(|text| text.contains('…') && text.starts_with('A') && text.ends_with('g')));
    let outlined = std::fs::read_to_string(directory.join("paths.svg")).unwrap();
    assert!(!outlined.contains("<text"));
    assert!(outlined.contains("data-original-source="));
    assert_eq!(
        png_dimensions(&std::fs::read(directory.join("preview.png")).unwrap()),
        (500, 350)
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn assigned_canvas_freezes_combined_anchor_rotation_ellipsis_and_fractional_fit() {
    use lichtung_plot::{
        Text, TextAnchor, TextBounds, TextElision, TextEllipsis, TextFailure, TextFit,
        TextFitPriority, TextLayout, TextOverflow, TextPanel, TextRotation, TextRotationMode,
        Theme,
    };
    let directory = unique_dir("assigned-font-fit");
    let first = directory.join("first.svg");
    let second = directory.join("second.svg");
    let source = "An independently authored scientific explanation with a long ending";
    let style = Theme::publication()
        .typography()
        .get_composition()
        .get_body()
        .clone()
        .font_size(14.25)
        .unwrap();
    let body = Text::new(source).style(style).layout(
        TextLayout::default()
            .anchor(TextAnchor::center())
            .rotation(TextRotation::degrees(30.05).unwrap())
            .rotation_mode(TextRotationMode::AlignAfterRotation)
            .bounds(TextBounds::new(Some(100.0), Some(30.0), Some(1)).unwrap())
            .fit(TextFit::shrink_to(10.25).unwrap())
            .fit_priority(TextFitPriority::PreferredSizeFirst)
            .overflow(TextOverflow::ellipsis(
                TextEllipsis::new(TextElision::Middle),
                TextFailure::Error,
            )),
    );
    let output = Canvas::grid(1, 1)
        .unwrap()
        .text_panel(0, 0, TextPanel::new("Independent heading", body).unwrap())
        .unwrap()
        .max_width(400)
        .svg_preserve_text(&first)
        .unwrap();
    let dimensions = (output.width(), output.height());
    assert!(dimensions.0 <= 400);
    output
        .svg(directory.join("outlines.svg"))
        .unwrap()
        .png_preview(directory.join("preview.png"))
        .unwrap()
        .png_with_options(
            directory.join("scaled.png"),
            PngOptions {
                sizing: PngSizing::Scale(2.0),
                compression: PngCompression::Fast,
            },
        )
        .unwrap()
        .svg_preserve_text(&second)
        .unwrap();
    let bytes = std::fs::read(first).unwrap();
    assert_eq!(bytes, std::fs::read(second).unwrap());
    let svg = std::str::from_utf8(&bytes).unwrap();
    let document = roxmltree::Document::parse(svg).unwrap();
    let selected = document
        .descendants()
        .find(|node| node.attribute("data-original-source") == Some(source))
        .unwrap();
    let painted = selected
        .descendants()
        .find(|node| node.has_tag_name("text"))
        .unwrap();
    assert!(painted.text().unwrap().contains('…'));
    let size: f64 = painted.attribute("font-size").unwrap().parse().unwrap();
    assert!((10.25..=14.25).contains(&size));
    assert!(svg.contains("rotate(30.05"));
    assert_eq!(
        png_dimensions(&std::fs::read(directory.join("preview.png")).unwrap()),
        dimensions
    );
    assert_eq!(
        png_dimensions(&std::fs::read(directory.join("scaled.png")).unwrap()),
        (dimensions.0 * 2, dimensions.1 * 2)
    );
    std::fs::remove_dir_all(directory).unwrap();
}

fn plot() -> Plot<lichtung_plot::kind::Scatter> {
    Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0]),
        NumericValues::new("y", [3.0, 1.0, 2.0]),
    )
    .x_tick_format(
        lichtung_plot::NumericFormat::default()
            .prefix("x=")
            .unwrap(),
    )
    .y_tick_format(
        lichtung_plot::NumericFormat::new(
            lichtung_plot::NumericNotation::Engineering,
            lichtung_plot::NumericPrecision::Significant(3),
        )
        .unwrap()
        .prefix("y=")
        .unwrap(),
    )
    .title("Selectable scene text")
    .size(320, 240)
}

fn canvas() -> Canvas {
    Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, plot())
        .unwrap()
        .size(320, 260)
        .title("Selectable canvas text")
}

#[test]
fn omitted_plot_size_reports_frozen_dimensions_and_explicit_size_stays_exact() {
    let directory = unique_dir("plot-content-size");
    let first = directory.join("first.svg");
    let explicit = directory.join("explicit.svg");
    let (x, y) = fixture_values();
    let plot = Plot::scatter(x, y);
    let automatic = plot.clone().svg_preserve_text(&first).unwrap();
    let exact = plot.size(640, 480).svg_preserve_text(&explicit).unwrap();

    assert!(automatic.width() > 0 && automatic.height() > 0);
    assert!(std::fs::read_to_string(first).unwrap().contains(&format!(
        "width=\"{}\" height=\"{}\"",
        automatic.width(),
        automatic.height()
    )));
    assert_eq!((exact.width(), exact.height()), (640, 480));
    assert!(std::fs::read_to_string(explicit)
        .unwrap()
        .contains("width=\"640\" height=\"480\""));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn plot_and_canvas_svg_operations_keep_complete_text_and_call_local_replay() {
    let directory = unique_dir("output-text-representation");
    for (name, output, height, title) in [
        (
            "plot",
            plot().svg(directory.join("plot-paths.svg")),
            240,
            "Selectable scene text",
        ),
        (
            "canvas",
            canvas().svg(directory.join("canvas-paths.svg")),
            260,
            "Selectable canvas text",
        ),
    ] {
        let selectable = directory.join(format!("{name}-selectable.svg"));
        let second_paths = directory.join(format!("{name}-second-paths.svg"));
        let second_selectable = directory.join(format!("{name}-second-selectable.svg"));
        output
            .unwrap()
            .svg_preserve_text(&selectable)
            .unwrap()
            .svg(&second_paths)
            .unwrap()
            .svg_preserve_text(&second_selectable)
            .unwrap();

        let paths = std::fs::read(directory.join(format!("{name}-paths.svg"))).unwrap();
        let selectable = std::fs::read(selectable).unwrap();
        assert_eq!(paths, std::fs::read(second_paths).unwrap());
        assert_eq!(selectable, std::fs::read(second_selectable).unwrap());
        let paths = String::from_utf8(paths).unwrap();
        assert!(!paths.contains("<text") && paths.contains("<path"));
        let selectable = String::from_utf8(selectable).unwrap();
        assert!(selectable.contains(&format!(">{title}</text>")));
        assert!(selectable.contains(">x=") && selectable.contains(">y="));
        assert!(selectable.contains(&format!("width=\"320\" height=\"{height}\"")));
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn canvas_exposes_publication_preview_and_custom_png_entry_points() {
    let directory = unique_dir("canvas-dpi");
    let publication = directory.join("publication.png");
    let preview = directory.join("preview.png");
    let custom = directory.join("custom.png");
    canvas().png(&publication).unwrap();
    canvas().png_preview(&preview).unwrap();
    canvas()
        .png_with_options(
            &custom,
            PngOptions {
                sizing: PngSizing::Width(160),
                compression: PngCompression::None,
            },
        )
        .unwrap();

    let publication = std::fs::read(publication).unwrap();
    let preview = std::fs::read(preview).unwrap();
    let custom = std::fs::read(custom).unwrap();
    assert_eq!(png_dimensions(&publication), (1000, 813));
    assert_eq!(png_phys(&publication), Some((11_811, 11_811, 1)));
    assert_eq!(png_dimensions(&preview), (320, 260));
    assert_eq!(png_phys(&preview), None);
    assert_eq!(png_dimensions(&custom), (160, 130));
    assert_eq!(png_phys(&custom), None);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn output_png_options_are_call_local_across_preview_custom_and_publication() {
    let directory = unique_dir("call-local-png-options");
    let preview = directory.join("preview.png");
    let custom = directory.join("custom.png");
    let publication = directory.join("publication.png");
    let repeated = directory.join("repeated.png");
    plot()
        .png(&publication)
        .unwrap()
        .png_preview(&preview)
        .unwrap()
        .png_with_options(
            &custom,
            PngOptions {
                sizing: PngSizing::Dpi(192.0),
                compression: PngCompression::High,
            },
        )
        .unwrap()
        .png(&repeated)
        .unwrap();

    let preview = std::fs::read(preview).unwrap();
    let custom = std::fs::read(custom).unwrap();
    let publication = std::fs::read(publication).unwrap();
    assert_eq!(publication, std::fs::read(repeated).unwrap());
    assert_eq!(png_dimensions(&preview), (320, 240));
    assert_eq!(png_phys(&preview), None);
    assert_eq!(png_dimensions(&custom), (640, 480));
    assert_eq!(png_phys(&custom), Some((7_559, 7_559, 1)));
    assert_eq!(png_dimensions(&publication), (1000, 750));
    assert_eq!(png_phys(&publication), Some((11_811, 11_811, 1)));
    std::fs::remove_dir_all(directory).unwrap();
}
