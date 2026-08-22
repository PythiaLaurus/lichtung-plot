#[cfg(feature = "text-as-path")]
use lichtung_plot::SvgText;
use lichtung_plot::{Canvas, Plot};
#[cfg(feature = "png")]
use lichtung_plot::{PngOptions, PngSizing};
use polars::prelude::*;

#[allow(
    clippy::duplicate_mod,
    reason = "each explicit integration target is an independent crate"
)]
#[path = "../support/native.rs"]
mod native;
#[path = "../support/visual.rs"]
mod support;

fn fixture() -> DataFrame {
    df!(
        "x" => [1.0_f64, 2.0, 3.0],
        "y" => [3.0_f64, 1.0, 2.0]
    )
    .unwrap()
}

fn scatter(data: &DataFrame) -> Plot {
    Plot::scatter(native::numeric(data, "x"), native::numeric(data, "y"))
        .title("Selectable scene text")
        .size(320, 240)
}

#[test]
fn plot_output_reuses_one_scene_for_chained_svg_publication() {
    let data = fixture();
    let directory = support::unique_dir("plot-svg-chain");
    let first = directory.join("first.svg");
    let second = directory.join("second.svg");

    scatter(&data).svg(&first).unwrap().svg(&second).unwrap();

    let first = std::fs::read(first).unwrap();
    let second = std::fs::read(second).unwrap();
    assert_eq!(first, second);
    assert!(first.windows(5).any(|window| window == b"<text"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn canvas_publishes_a_complete_composed_svg() {
    let data = fixture();
    let directory = support::unique_dir("canvas-svg");
    let output = directory.join("canvas.svg");
    let canvas = Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, scatter(&data))
        .unwrap()
        .size(320, 260)
        .title("Selectable canvas text");

    canvas.svg(&output).unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.contains("Selectable canvas text"));
    assert!(svg.contains("<circle"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(feature = "text-as-path")]
#[test]
fn svg_text_paths_are_selected_before_terminal_output() {
    let data = fixture();
    let directory = support::unique_dir("svg-text-paths");
    let output = directory.join("outlined.svg");

    scatter(&data)
        .svg_text(SvgText::Paths)
        .svg(&output)
        .unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    assert!(!svg.contains("<text"));
    assert!(svg.contains("<path"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(feature = "png")]
#[test]
fn png_options_control_dimensions_and_density() {
    let data = fixture();
    let directory = support::unique_dir("png-options");
    let scaled = directory.join("scaled.png");
    let dpi = directory.join("dpi.png");

    scatter(&data)
        .png_options(PngOptions {
            sizing: PngSizing::Scale(2.0),
            ..PngOptions::default()
        })
        .png(&scaled)
        .unwrap();
    scatter(&data)
        .png_options(PngOptions {
            sizing: PngSizing::Dpi(192.0),
            ..PngOptions::default()
        })
        .png(&dpi)
        .unwrap();

    let scaled = std::fs::read(scaled).unwrap();
    let dpi = std::fs::read(dpi).unwrap();
    assert_eq!(support::png_dimensions(&scaled), (640, 480));
    assert_eq!(support::png_phys(&scaled), None);
    assert_eq!(support::png_dimensions(&dpi), (640, 480));
    assert_eq!(support::png_phys(&dpi), Some((7559, 7559, 1)));
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(feature = "png")]
#[test]
fn one_terminal_render_can_publish_svg_then_png() {
    let data = fixture();
    let directory = support::unique_dir("svg-png-chain");
    let svg = directory.join("plot.svg");
    let png = directory.join("plot.png");

    scatter(&data).svg(&svg).unwrap().png(&png).unwrap();

    assert!(std::fs::read_to_string(svg).unwrap().contains("<svg"));
    assert_eq!(
        support::png_dimensions(&std::fs::read(png).unwrap()),
        (320, 240)
    );
    std::fs::remove_dir_all(directory).unwrap();
}
