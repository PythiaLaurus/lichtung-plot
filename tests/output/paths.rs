use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use super::*;

fn scatter() -> Plot<lichtung_plot::kind::Scatter> {
    Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0]),
        NumericValues::new("y", [3.0, 1.0, 2.0]),
    )
    .size(320, 240)
}

fn canvas() -> Canvas {
    Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, scatter())
        .unwrap()
        .size(320, 260)
}

fn assert_svg(path: &Path, preserves_text: bool) {
    let svg = std::fs::read_to_string(path).unwrap();
    assert!(svg.starts_with("<svg"));
    assert_eq!(svg.contains("<text"), preserves_text);
}

fn assert_png(path: &Path) {
    assert!(std::fs::read(path)
        .unwrap()
        .starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]));
}

fn assert_no_staging_files(directory: &Path) {
    assert!(std::fs::read_dir(directory).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .as_encoded_bytes()
            .starts_with(b".lichtung-tmp-")
    }));
}

fn assert_invalid_input(error: lichtung_plot::Error) {
    assert!(matches!(
        error,
        lichtung_plot::Error::Io(error) if error.kind() == std::io::ErrorKind::InvalidInput
    ));
}

#[test]
fn plot_and_reusable_output_complete_literal_svg_suffixes() {
    let directory = tempdir();
    let extensionless = directory.join("outlined");
    let exact = directory.join("exact.svg");
    let mixed_case = directory.join("editable.SvG");
    let versioned = directory.join("report.v1");
    let matching_versioned = directory.join("report.v2.SVG");
    let contradictory = directory.join("diagram.png");
    let trailing_dot = directory.join("trailing.svg.");
    let dotfile = directory.join(".chart");
    let matching_dotfile = directory.join(".svg");

    scatter()
        .svg(&extensionless)
        .unwrap()
        .svg(&exact)
        .unwrap()
        .svg_preserve_text(&mixed_case)
        .unwrap()
        .svg_preserve_text(&versioned)
        .unwrap()
        .svg_preserve_text(&matching_versioned)
        .unwrap()
        .svg(&contradictory)
        .unwrap()
        .svg_preserve_text(&trailing_dot)
        .unwrap()
        .svg_preserve_text(&dotfile)
        .unwrap()
        .svg_preserve_text(&matching_dotfile)
        .unwrap();

    assert_svg(&directory.join("outlined.svg"), false);
    assert_svg(&exact, false);
    assert_svg(&mixed_case, true);
    assert_svg(&directory.join("report.v1.svg"), true);
    assert_svg(&matching_versioned, true);
    assert_svg(&directory.join("diagram.png.svg"), false);
    assert_svg(&directory.join("trailing.svg..svg"), true);
    assert_svg(&directory.join(".chart.svg"), true);
    assert_svg(&matching_dotfile, true);
    assert!(!extensionless.exists());
    assert!(!directory.join("exact.svg.svg").exists());
    assert!(!directory.join("editable.SvG.svg").exists());
    assert_no_staging_files(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn canvas_and_reusable_output_complete_literal_png_suffixes() {
    let directory = tempdir();
    let extensionless = directory.join("publication");
    let exact = directory.join("preview.png");
    let mixed_case = directory.join("second.PnG");
    let versioned = directory.join("custom.v1");
    let contradictory = directory.join("raster.svg");
    let dotfile = directory.join(".image");
    let matching_dotfile = directory.join(".PNG");
    let custom = PngOptions {
        sizing: PngSizing::Width(160),
        compression: PngCompression::None,
    };

    canvas()
        .png(&extensionless)
        .unwrap()
        .png_preview(&exact)
        .unwrap()
        .png(&mixed_case)
        .unwrap()
        .png_with_options(&versioned, custom)
        .unwrap()
        .png_preview(&contradictory)
        .unwrap()
        .png_with_options(&dotfile, custom)
        .unwrap()
        .png_preview(&matching_dotfile)
        .unwrap();

    for path in [
        directory.join("publication.png"),
        exact,
        mixed_case,
        directory.join("custom.v1.png"),
        directory.join("raster.svg.png"),
        directory.join(".image.png"),
        matching_dotfile,
    ] {
        assert_png(&path);
    }
    assert!(!extensionless.exists());
    assert!(!directory.join("preview.png.png").exists());
    assert!(!directory.join("second.PnG.png").exists());
    assert_no_staging_files(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn completed_destination_replaces_existing_file_without_touching_base() {
    let directory = tempdir();
    let base = directory.join("report");
    let completed = directory.join("report.svg");
    std::fs::write(&base, "unsuffixed source artifact").unwrap();
    std::fs::write(&completed, "old completed artifact").unwrap();

    scatter().svg_preserve_text(&base).unwrap();

    assert_eq!(
        std::fs::read_to_string(&base).unwrap(),
        "unsuffixed source artifact"
    );
    assert_svg(&completed, true);
    assert_no_staging_files(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn existing_directory_is_rejected_without_completed_sibling_or_staging() {
    let directory = tempdir();
    let destination = directory.join("destination");
    std::fs::create_dir(&destination).unwrap();

    let error = canvas().svg_preserve_text(&destination).err().unwrap();

    assert_invalid_input(error);
    assert!(destination.is_dir());
    assert!(!directory.join("destination.svg").exists());
    assert_no_staging_files(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn publication_failure_preserves_existing_completed_directory() {
    let directory = tempdir();
    let base = directory.join("destination");
    let completed = directory.join("destination.svg");
    std::fs::create_dir(&completed).unwrap();
    std::fs::write(completed.join("existing"), "existing artifact").unwrap();

    assert!(scatter().svg_preserve_text(&base).is_err());

    assert_eq!(
        std::fs::read_to_string(completed.join("existing")).unwrap(),
        "existing artifact"
    );
    assert_no_staging_files(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(unix)]
#[test]
fn symlink_to_directory_is_rejected_before_suffix_completion() {
    use std::os::unix::fs::symlink;

    let directory = tempdir();
    let target = directory.join("target");
    let destination = directory.join("destination");
    std::fs::create_dir(&target).unwrap();
    symlink(&target, &destination).unwrap();

    let error = scatter().png_preview(&destination).err().unwrap();

    assert_invalid_input(error);
    assert!(destination.is_dir());
    assert!(!directory.join("destination.png").exists());
    assert_no_staging_files(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn missing_filename_and_parent_fail_without_artifacts() {
    let directory = tempdir();
    let missing_filename = directory.join("absent").join("..");
    assert!(missing_filename.file_name().is_none());
    assert!(!missing_filename.is_dir());

    let error = scatter()
        .svg_preserve_text(&missing_filename)
        .err()
        .unwrap();
    assert_invalid_input(error);

    let missing_parent = directory.join("missing").join("plot");
    let error = scatter().svg_preserve_text(&missing_parent).err().unwrap();
    assert!(matches!(
        error,
        lichtung_plot::Error::Io(error) if error.kind() == std::io::ErrorKind::NotFound
    ));
    assert!(!directory.join("missing").exists());
    assert_no_staging_files(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

struct PathAccessProbe<'a>(&'a AtomicBool);

impl AsRef<Path> for PathAccessProbe<'_> {
    fn as_ref(&self) -> &Path {
        self.0.store(true, Ordering::Relaxed);
        Path::new("")
    }
}

#[test]
fn png_conversion_failure_precedes_destination_path_access() {
    let directory = tempdir();
    let scene = scatter()
        .svg_preserve_text(directory.join("scene"))
        .unwrap();
    let accessed = AtomicBool::new(false);

    let error = scene
        .png_with_options(
            PathAccessProbe(&accessed),
            PngOptions {
                sizing: PngSizing::Scale(0.0),
                ..PngOptions::default()
            },
        )
        .err()
        .unwrap();

    assert!(error.to_string().contains("png scale must be positive"));
    assert!(!accessed.load(Ordering::Relaxed));
    assert_no_staging_files(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(unix)]
#[test]
fn suffix_append_over_component_limit_leaves_base_and_no_staging() {
    let directory = tempdir();
    let base = directory.join("d".repeat(255));
    std::fs::write(&base, "unsuffixed source artifact").unwrap();

    let error = scatter().svg_preserve_text(&base).err().unwrap();

    assert!(matches!(error, lichtung_plot::Error::Io(_)));
    assert_eq!(
        std::fs::read_to_string(&base).unwrap(),
        "unsuffixed source artifact"
    );
    assert_no_staging_files(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(unix)]
#[test]
fn matching_suffix_at_component_limit_is_not_appended() {
    let directory = tempdir();
    let destination = directory.join(format!("{}.svg", "d".repeat(251)));

    scatter().svg_preserve_text(&destination).unwrap();

    assert_svg(&destination, true);
    assert!(!directory
        .join(format!("{}.svg.svg", "d".repeat(251)))
        .exists());
    assert_no_staging_files(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(unix)]
#[test]
fn unix_non_utf8_filename_units_are_preserved_when_matching_and_appending() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let directory = tempdir();
    let matching_leaf = OsString::from_vec(vec![b'n', 0xff, b'.', b'S', b'v', b'G']);
    let matching = directory.join(&matching_leaf);
    let base_leaf = OsString::from_vec(vec![b'n', 0xfe]);
    let base = directory.join(&base_leaf);
    let mut completed_leaf = base_leaf;
    completed_leaf.push(".png");
    let completed = directory.join(&completed_leaf);

    scatter()
        .svg_preserve_text(&matching)
        .unwrap()
        .png_preview(&base)
        .unwrap();

    assert_svg(&matching, true);
    assert_png(&completed);
    assert!(!base.exists());
    let mut incorrectly_appended = matching_leaf;
    incorrectly_appended.push(".svg");
    assert!(!directory.join(incorrectly_appended).exists());
    assert_no_staging_files(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(windows)]
#[test]
fn windows_non_ascii_utf16_units_are_preserved_when_matching_and_appending() {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    let directory = tempdir();
    let matching_leaf = OsString::from_wide(&[0x5831, 0x544a, 0x2e, 0x53, 0x76, 0x47]);
    let matching = directory.join(&matching_leaf);
    let base_leaf = OsString::from_wide(&[0x753b, 0x50cf]);
    let base = directory.join(&base_leaf);
    let mut completed_leaf = base_leaf;
    completed_leaf.push(".png");
    let completed = directory.join(&completed_leaf);

    scatter()
        .svg_preserve_text(&matching)
        .unwrap()
        .png_preview(&base)
        .unwrap();

    assert_svg(&matching, true);
    assert_png(&completed);
    assert_eq!(
        completed
            .file_name()
            .unwrap()
            .encode_wide()
            .collect::<Vec<_>>(),
        completed_leaf.encode_wide().collect::<Vec<_>>()
    );
    assert!(!base.exists());
    let mut incorrectly_appended = matching_leaf;
    incorrectly_appended.push(".svg");
    assert!(!directory.join(incorrectly_appended).exists());
    assert_no_staging_files(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[cfg(windows)]
#[test]
fn windows_validates_the_completed_leaf_before_staging() {
    let directory = tempdir();
    let base = directory.join("NUL");

    let error = scatter().svg_preserve_text(&base).err().unwrap();

    assert!(error
        .to_string()
        .contains("invalid or reserved Windows file name"));
    assert!(!base.exists() && !directory.join("NUL.svg").exists());
    assert_no_staging_files(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}
