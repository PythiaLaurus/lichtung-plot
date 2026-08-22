use std::path::PathBuf;

use super::*;

fn fixture() -> DataFrame {
    df!("x" => [1.0_f64, 2.0, 3.0], "y" => [3.0_f64, 1.0, 2.0]).unwrap()
}

fn plot(frame: &DataFrame) -> Plot {
    Plot::scatter(data::numeric(frame, "x"), data::numeric(frame, "y"))
        .title("Selectable scene text")
        .size(320, 240)
}

fn canvas(frame: &DataFrame) -> Canvas {
    Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, plot(frame))
        .unwrap()
        .size(320, 260)
        .title("Selectable figure text")
}

fn temp_dir(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "lichtung-multiformat-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

#[test]
fn output_reuses_one_plot_render_for_exact_svg_bytes() {
    let frame = fixture();
    let directory = temp_dir("plot-output-reuse");
    let first = directory.join("first.svg");
    let second = directory.join("second.svg");
    plot(&frame).svg(&first).unwrap().svg(&second).unwrap();
    let first_bytes = std::fs::read(&first).unwrap();
    assert_eq!(first_bytes, std::fs::read(&second).unwrap());
    assert!(String::from_utf8(first_bytes)
        .unwrap()
        .contains("width=\"320\" height=\"240\""));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn output_reuses_one_canvas_render_for_exact_svg_bytes() {
    let frame = fixture();
    let directory = temp_dir("canvas-output-reuse");
    let first = directory.join("first.svg");
    let second = directory.join("second.svg");
    canvas(&frame).svg(&first).unwrap().svg(&second).unwrap();
    let first_bytes = std::fs::read(&first).unwrap();
    assert_eq!(first_bytes, std::fs::read(&second).unwrap());
    assert!(String::from_utf8(first_bytes)
        .unwrap()
        .contains("width=\"320\" height=\"260\""));
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(feature = "text-as-path")]
#[test]
fn selectable_and_path_text_outputs_use_the_configured_representation() {
    let frame = fixture();
    let directory = temp_dir("text-representation");
    let selectable = directory.join("selectable.svg");
    let paths = directory.join("paths.svg");
    plot(&frame).svg(&selectable).unwrap();
    plot(&frame).svg_text(SvgText::Paths).svg(&paths).unwrap();
    let selectable = std::fs::read_to_string(selectable).unwrap();
    let paths = std::fs::read_to_string(paths).unwrap();
    assert!(selectable.contains("<text"));
    assert!(!paths.contains("<text") && paths.contains("<path"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(feature = "png")]
#[test]
fn output_produces_deterministic_300_dpi_png_with_density() {
    let frame = fixture();
    let directory = temp_dir("png-output-reuse");
    let first = directory.join("first.png");
    let second = directory.join("second.png");
    plot(&frame)
        .png_options(dpi_300())
        .png(&first)
        .unwrap()
        .png(&second)
        .unwrap();
    let bytes = std::fs::read(first).unwrap();
    assert_eq!(bytes, std::fs::read(second).unwrap());
    assert_eq!(png_dimensions(&bytes), (1000, 750));
    assert_eq!(png_density(&bytes), Some((11_811, 11_811, 1)));
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(feature = "png")]
#[test]
fn canvas_supports_configured_300_dpi_png() {
    let frame = fixture();
    let directory = temp_dir("canvas-dpi");
    let path = directory.join("canvas.png");
    canvas(&frame).png_options(dpi_300()).png(&path).unwrap();
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(png_dimensions(&bytes), (1000, 813));
    assert_eq!(png_density(&bytes), Some((11_811, 11_811, 1)));
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(feature = "png")]
#[test]
fn png_options_reject_non_finite_zero_and_excessive_work_without_artifacts() {
    let frame = fixture();
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
        assert!(plot(&frame)
            .png_options(PngOptions {
                sizing,
                ..PngOptions::default()
            })
            .png(&path)
            .is_err());
        assert!(!path.exists());
        std::fs::remove_dir_all(directory).unwrap();
    }
}

#[cfg(feature = "png")]
#[test]
fn plot_canvas_and_output_publish_transactional_pairs() {
    let frame = fixture();
    let directory = temp_dir("transactional-pairs");
    let plot_svg = directory.join("plot.svg");
    let plot_png = directory.join("plot.png");
    plot(&frame).svg_and_png(&plot_svg, &plot_png).unwrap();
    assert_pair(&plot_svg, &plot_png, (320, 240));

    let canvas_svg = directory.join("canvas.svg");
    let canvas_png = directory.join("canvas.png");
    canvas(&frame)
        .svg_and_png(&canvas_svg, &canvas_png)
        .unwrap();
    assert_pair(&canvas_svg, &canvas_png, (320, 260));

    let reused_svg = directory.join("reused.svg");
    let reused_png = directory.join("reused.png");
    plot(&frame)
        .svg(directory.join("initial.svg"))
        .unwrap()
        .svg_and_png(&reused_svg, &reused_png)
        .unwrap();
    assert_pair(&reused_svg, &reused_png, (320, 240));
    assert_no_temporary_artifacts(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(feature = "png")]
#[test]
fn transactional_pair_replaces_both_existing_artifacts() {
    let frame = fixture();
    let directory = temp_dir("pair-replacement");
    let svg = directory.join("plot.svg");
    let png = directory.join("plot.png");
    std::fs::write(&svg, b"old svg").unwrap();
    std::fs::write(&png, b"old png").unwrap();
    plot(&frame).svg_and_png(&svg, &png).unwrap();
    assert_pair(&svg, &png, (320, 240));
    assert_no_temporary_artifacts(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(feature = "png")]
#[test]
fn transactional_pair_rejects_destination_aliases_without_mutation() {
    let frame = fixture();
    let directory = temp_dir("pair-alias");
    let destination = directory.join("artifact");
    std::fs::write(&destination, b"existing").unwrap();
    let error = plot(&frame)
        .svg_and_png(&destination, &destination)
        .err()
        .unwrap();
    assert!(error.to_string().contains("must be distinct"));
    assert_eq!(std::fs::read(&destination).unwrap(), b"existing");

    let nested = directory.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    let lexical = nested.join("..").join("artifact");
    let error = plot(&frame)
        .svg_and_png(&destination, &lexical)
        .err()
        .unwrap();
    assert!(error.to_string().contains("must be distinct"));
    assert_eq!(std::fs::read(&destination).unwrap(), b"existing");
    assert_no_temporary_artifacts(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(all(feature = "png", unix))]
#[test]
fn transactional_pair_rejects_symlink_aliases() {
    use std::os::unix::fs::symlink;

    let frame = fixture();
    let directory = temp_dir("pair-symlink-alias");
    let destination = directory.join("artifact");
    let alias = directory.join("alias");
    std::fs::write(&destination, b"existing").unwrap();
    symlink(&destination, &alias).unwrap();
    let error = plot(&frame)
        .svg_and_png(&destination, &alias)
        .err()
        .unwrap();
    assert!(error.to_string().contains("must be distinct"));
    assert_eq!(std::fs::read(&destination).unwrap(), b"existing");
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(feature = "png")]
#[test]
fn invalid_pair_destination_preserves_the_other_existing_artifact() {
    let frame = fixture();
    let directory = temp_dir("pair-invalid-destination");
    let svg = directory.join("plot.svg");
    let png = directory.join("plot.png");
    std::fs::write(&svg, b"existing svg").unwrap();
    std::fs::create_dir_all(&png).unwrap();
    let error = plot(&frame).svg_and_png(&svg, &png).err().unwrap();
    assert!(error.to_string().contains("is not a file"));
    assert_eq!(std::fs::read(&svg).unwrap(), b"existing svg");
    assert!(png.is_dir());
    assert_no_temporary_artifacts(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(feature = "png")]
fn assert_pair(svg: &std::path::Path, png: &std::path::Path, dimensions: (u32, u32)) {
    assert!(std::fs::read_to_string(svg).unwrap().contains("<svg"));
    assert_eq!(png_dimensions(&std::fs::read(png).unwrap()), dimensions);
}

#[cfg(feature = "png")]
fn assert_no_temporary_artifacts(directory: &std::path::Path) {
    let names = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(names
        .iter()
        .all(|name| !name.contains("lichtung-tmp") && !name.contains("backup")));
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
