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

fn assert_svg(path: &Path, preserves_text: bool) {
    let svg = std::fs::read_to_string(path).unwrap();
    assert!(svg.starts_with("<svg"));
    assert_eq!(svg.contains("<text"), preserves_text);
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
