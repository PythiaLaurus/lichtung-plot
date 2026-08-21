use std::path::PathBuf;

use lichtung_plot::{Chart, Figure};
#[cfg(feature = "png")]
use lichtung_plot::{PngOptions, PngSizing};
use polars::df;
use polars::prelude::DataFrame;

fn chart(data: &DataFrame) -> Chart<'_> {
    let mut chart = Chart::new(data).title("Selectable scene text");
    chart.x("x").y("y").size(320, 240).scatter();
    chart
}

fn figure(data: &DataFrame) -> Figure<'_> {
    Figure::grid(1, 1)
        .unwrap()
        .panel(0, 0, chart(data))
        .unwrap()
        .size(320, 260)
        .title("Selectable figure text")
}

fn fixture() -> DataFrame {
    df!(
        "x" => [1.0_f64, 2.0, 3.0],
        "y" => [3.0_f64, 1.0, 2.0]
    )
    .unwrap()
}

fn temp_path(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "lichtung-multiformat-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    directory.join(name)
}

#[test]
fn chart_scene_reuses_one_render_for_exact_svg_bytes() {
    let data = fixture();
    let scene = chart(&data).render_scene().unwrap();

    assert_eq!(scene.svg_bytes(), scene.svg_bytes());
    assert_eq!((scene.width_css_px(), scene.height_css_px()), (320, 240));
}

#[test]
fn figure_scene_reuses_one_render_for_exact_svg_bytes() {
    let data = fixture();
    let scene = figure(&data).render_scene().unwrap();

    assert_eq!(scene.svg_bytes(), scene.svg_bytes());
    assert_eq!((scene.width_css_px(), scene.height_css_px()), (320, 260));
}

#[test]
fn chart_legacy_svg_methods_preserve_scene_bytes() {
    let data = fixture();
    let chart = chart(&data);
    let expected = chart.render_scene().unwrap().into_svg_bytes();
    let path = temp_path("chart.svg");

    chart.render_to_svg(&path).unwrap();

    assert_eq!(chart.render_to_svg_bytes().unwrap(), expected);
    assert_eq!(std::fs::read(&path).unwrap(), expected);
}

#[test]
fn figure_legacy_svg_methods_preserve_scene_bytes() {
    let data = fixture();
    let figure = figure(&data);
    let expected = figure.render_scene().unwrap().into_svg_bytes();
    let path = temp_path("figure.svg");

    figure.render_to_svg(&path).unwrap();

    assert_eq!(figure.render_to_svg_bytes().unwrap(), expected);
    assert_eq!(std::fs::read(&path).unwrap(), expected);
}

#[cfg(feature = "text-as-path")]
#[test]
fn scene_can_convert_every_text_element_to_vector_paths() {
    let data = fixture();
    let scene = chart(&data).render_scene().unwrap();
    let outlined = String::from_utf8(scene.svg_with_text_as_paths().unwrap()).unwrap();

    assert!(
        scene
            .svg_bytes()
            .windows(5)
            .any(|window| window == b"<text")
            && !outlined.contains("<text")
            && outlined.contains("<path")
    );
}

#[cfg(feature = "png")]
#[test]
fn chart_scene_produces_deterministic_300_dpi_png_with_density() {
    let data = fixture();
    let scene = chart(&data).render_scene().unwrap();
    let options = dpi_300();

    let first = scene.png_bytes(&options).unwrap();
    let second = scene.png_bytes(&options).unwrap();

    assert_eq!(first, second);
    assert_eq!(png_dimensions(&first), (1000, 750));
    assert_eq!(png_density(&first), Some((11_811, 11_811, 1)));
}

#[cfg(feature = "png")]
#[test]
fn figure_supports_option_aware_in_memory_300_dpi_png() {
    let data = fixture();
    let figure = figure(&data);
    let scene = figure.render_scene().unwrap();
    let expected = scene.png_bytes(&dpi_300()).unwrap();

    let actual = figure.render_to_png_bytes_with_options(&dpi_300()).unwrap();

    assert_eq!(actual, expected);
    assert_eq!(png_dimensions(&actual), (1000, 813));
    assert_eq!(png_density(&actual), Some((11_811, 11_811, 1)));
}

#[cfg(feature = "png")]
#[test]
fn legacy_png_methods_preserve_default_dimensions_and_bytes() {
    let data = fixture();
    let chart = chart(&data);
    let expected = chart
        .render_scene()
        .unwrap()
        .png_bytes(&PngOptions::default())
        .unwrap();
    let path = temp_path("chart.png");

    chart.render_to_png(&path).unwrap();

    assert_eq!(std::fs::read(path).unwrap(), expected);
    assert_eq!(png_dimensions(&expected), (320, 240));
    assert_eq!(png_density(&expected), None);
}

#[cfg(feature = "png")]
#[test]
fn figure_legacy_png_method_preserves_default_scene_bytes() {
    let data = fixture();
    let figure = figure(&data);
    let expected = figure
        .render_scene()
        .unwrap()
        .png_bytes(&PngOptions::default())
        .unwrap();
    let path = temp_path("figure.png");

    figure.render_to_png(&path).unwrap();

    assert_eq!(std::fs::read(path).unwrap(), expected);
}

#[cfg(feature = "png")]
#[test]
fn png_options_reject_non_finite_zero_and_excessive_work() {
    let data = fixture();
    let scene = chart(&data).render_scene().unwrap();
    let invalid = [
        PngSizing::Dpi(f32::NAN),
        PngSizing::Dpi(0.0),
        PngSizing::Scale(1_000_000.0),
    ];

    for sizing in invalid {
        assert!(scene
            .png_bytes(&PngOptions {
                sizing,
                ..PngOptions::default()
            })
            .is_err());
    }
}

#[cfg(feature = "png")]
fn dpi_300() -> PngOptions {
    PngOptions {
        sizing: PngSizing::Dpi(300.0),
        ..PngOptions::default()
    }
}

#[cfg(feature = "png")]
fn png_dimensions(bytes: &[u8]) -> (u32, u32) {
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    (
        u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
        u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
    )
}

#[cfg(feature = "png")]
fn png_density(bytes: &[u8]) -> Option<(u32, u32, u8)> {
    let mut offset = 8;
    while offset + 12 <= bytes.len() {
        let length = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        let chunk = &bytes[offset + 4..offset + 8];
        let data = &bytes[offset + 8..offset + 8 + length];
        if chunk == b"pHYs" {
            return Some((
                u32::from_be_bytes(data[..4].try_into().unwrap()),
                u32::from_be_bytes(data[4..8].try_into().unwrap()),
                data[8],
            ));
        }
        offset += length + 12;
    }
    None
}
