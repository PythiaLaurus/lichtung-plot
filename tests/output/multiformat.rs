use super::*;

fn plot() -> Plot<lichtung_plot::kind::Scatter> {
    Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0]),
        NumericValues::new("y", [3.0, 1.0, 2.0]),
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
