use std::path::PathBuf;

use super::*;

static TEMP_DIRECTORY_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[derive(Clone)]
struct ScatterData {
    x: NumericValues,
    y: NumericValues,
}

fn scatter_data() -> ScatterData {
    ScatterData {
        x: NumericValues::new("x", [1.0, 2.0, 3.0]),
        y: NumericValues::new("y", [3.0, 1.0, 2.0]),
    }
}

fn plot(data: &ScatterData) -> Plot<lichtung_plot::kind::Scatter> {
    Plot::scatter(data.x.clone(), data.y.clone())
        .title("Selectable scene text")
        .size(320, 240)
}

fn canvas(data: &ScatterData) -> Canvas {
    Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, plot(data))
        .unwrap()
        .size(320, 260)
        .title("Selectable canvas text")
}

fn temp_dir(name: &str) -> PathBuf {
    loop {
        let sequence = TEMP_DIRECTORY_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "lichtung-multiformat-{name}-{}-{}",
            std::process::id(),
            sequence
        ));
        match std::fs::create_dir(&directory) {
            Ok(()) => return directory,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => panic!("tempdir: {error}"),
        }
    }
}

#[test]
fn output_reuses_one_plot_render_for_exact_svg_bytes() {
    let data = scatter_data();
    let directory = temp_dir("plot-output-reuse");
    let first = directory.join("first.svg");
    let second = directory.join("second.svg");
    plot(&data)
        .svg_preserve_text(&first)
        .unwrap()
        .svg_preserve_text(&second)
        .unwrap();
    let first_bytes = std::fs::read(&first).unwrap();
    assert_eq!(first_bytes, std::fs::read(&second).unwrap());
    assert!(String::from_utf8(first_bytes)
        .unwrap()
        .contains("width=\"320\" height=\"240\""));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn omitted_plot_size_preserves_exact_default_svg_bytes() {
    let data = scatter_data();
    let directory = temp_dir("plot-default-size");
    let first = directory.join("first.svg");
    let second = directory.join("second.svg");
    let explicit = directory.join("explicit.svg");
    Plot::scatter(data.x.clone(), data.y.clone())
        .svg_preserve_text(&first)
        .unwrap();
    Plot::scatter(data.x.clone(), data.y.clone())
        .svg_preserve_text(&second)
        .unwrap();
    Plot::scatter(data.x, data.y)
        .size(640, 480)
        .svg_preserve_text(&explicit)
        .unwrap();

    let first_bytes = std::fs::read(&first).unwrap();
    assert_eq!(first_bytes, std::fs::read(&second).unwrap());
    assert_eq!(first_bytes, std::fs::read(&explicit).unwrap());
    assert!(String::from_utf8(first_bytes)
        .unwrap()
        .contains("width=\"640\" height=\"480\""));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn output_reuses_one_canvas_render_for_exact_svg_bytes() {
    let data = scatter_data();
    let directory = temp_dir("canvas-output-reuse");
    let first = directory.join("first.svg");
    let second = directory.join("second.svg");
    canvas(&data)
        .svg_preserve_text(&first)
        .unwrap()
        .svg_preserve_text(&second)
        .unwrap();
    let first_bytes = std::fs::read(&first).unwrap();
    assert_eq!(first_bytes, std::fs::read(&second).unwrap());
    assert!(String::from_utf8(first_bytes)
        .unwrap()
        .contains("width=\"320\" height=\"260\""));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn plot_exposes_outlined_and_selectable_svg_publication() {
    let data = scatter_data();
    let directory = temp_dir("plot-text-representation");
    let selectable = directory.join("selectable.svg");
    let paths = directory.join("paths.svg");
    plot(&data).svg(&paths).unwrap();
    plot(&data).svg_preserve_text(&selectable).unwrap();
    let selectable = std::fs::read_to_string(selectable).unwrap();
    let paths = std::fs::read_to_string(paths).unwrap();
    assert!(selectable.contains("<text"));
    assert!(!paths.contains("<text") && paths.contains("<path"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn canvas_exposes_outlined_and_selectable_svg_publication() {
    let data = scatter_data();
    let directory = temp_dir("canvas-text-representation");
    let selectable = directory.join("selectable.svg");
    let paths = directory.join("paths.svg");
    canvas(&data).svg(&paths).unwrap();
    canvas(&data).svg_preserve_text(&selectable).unwrap();
    let selectable = std::fs::read_to_string(selectable).unwrap();
    let paths = std::fs::read_to_string(paths).unwrap();
    assert!(selectable.contains("<text"));
    assert!(!paths.contains("<text") && paths.contains("<path"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn output_svg_operations_are_call_local_and_reuse_one_scene() {
    let data = scatter_data();
    let directory = temp_dir("output-text-representation");
    let first_paths = directory.join("first-paths.svg");
    let selectable = directory.join("selectable.svg");
    let second_paths = directory.join("second-paths.svg");
    let second_selectable = directory.join("second-selectable.svg");
    plot(&data)
        .svg(&first_paths)
        .unwrap()
        .svg_preserve_text(&selectable)
        .unwrap()
        .svg(&second_paths)
        .unwrap()
        .svg_preserve_text(&second_selectable)
        .unwrap();

    let first_paths = std::fs::read(first_paths).unwrap();
    let selectable = std::fs::read(selectable).unwrap();
    assert_eq!(first_paths, std::fs::read(second_paths).unwrap());
    assert_eq!(selectable, std::fs::read(second_selectable).unwrap());
    assert!(!String::from_utf8(first_paths).unwrap().contains("<text"));
    assert!(String::from_utf8(selectable).unwrap().contains("<text"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn plain_png_is_deterministic_300_dpi_publication_output() {
    let data = scatter_data();
    let directory = temp_dir("png-output-reuse");
    let first = directory.join("first.png");
    let second = directory.join("second.png");
    plot(&data).png(&first).unwrap().png(&second).unwrap();
    let bytes = std::fs::read(first).unwrap();
    assert_eq!(bytes, std::fs::read(second).unwrap());
    assert_eq!(png_dimensions(&bytes), (1000, 750));
    assert_eq!(png_density(&bytes), Some((11_811, 11_811, 1)));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn canvas_exposes_publication_preview_and_custom_png_entry_points() {
    let data = scatter_data();
    let directory = temp_dir("canvas-dpi");
    let publication = directory.join("publication.png");
    let preview = directory.join("preview.png");
    let custom = directory.join("custom.png");
    canvas(&data).png(&publication).unwrap();
    canvas(&data).png_preview(&preview).unwrap();
    canvas(&data)
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
    assert_eq!(png_density(&publication), Some((11_811, 11_811, 1)));
    assert_eq!(png_dimensions(&preview), (320, 260));
    assert_eq!(png_density(&preview), None);
    assert_eq!(png_dimensions(&custom), (160, 130));
    assert_eq!(png_density(&custom), None);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn png_options_reject_non_finite_zero_and_excessive_work_without_artifacts() {
    let data = scatter_data();
    for (index, sizing) in [
        PngSizing::Dpi(f32::NAN),
        PngSizing::Dpi(0.0),
        PngSizing::Scale(1_000_000.0),
    ]
    .into_iter()
    .enumerate()
    {
        let directory = temp_dir("invalid-png-options");
        let path = directory.join(format!("invalid-{index}.png"));
        assert!(plot(&data)
            .png_with_options(
                &path,
                PngOptions {
                    sizing,
                    ..PngOptions::default()
                },
            )
            .is_err());
        assert!(!path.exists());
        std::fs::remove_dir_all(directory).unwrap();
    }
}

#[test]
fn output_png_options_are_call_local_across_preview_custom_and_publication() {
    let data = scatter_data();
    let directory = temp_dir("call-local-png-options");
    let preview = directory.join("preview.png");
    let custom = directory.join("custom.png");
    let publication = directory.join("publication.png");
    plot(&data)
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
        .png(&publication)
        .unwrap();

    let preview = std::fs::read(preview).unwrap();
    let custom = std::fs::read(custom).unwrap();
    let publication = std::fs::read(publication).unwrap();
    assert_eq!(png_dimensions(&preview), (320, 240));
    assert_eq!(png_density(&preview), None);
    assert_eq!(png_dimensions(&custom), (640, 480));
    assert_eq!(png_density(&custom), Some((7_559, 7_559, 1)));
    assert_eq!(png_dimensions(&publication), (1000, 750));
    assert_eq!(png_density(&publication), Some((11_811, 11_811, 1)));
    std::fs::remove_dir_all(directory).unwrap();
}

fn png_dimensions(bytes: &[u8]) -> (u32, u32) {
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    (
        u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
        u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
    )
}

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
