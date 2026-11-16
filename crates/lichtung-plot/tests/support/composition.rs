#![allow(
    dead_code,
    reason = "helpers are shared by independently compiled integration test targets"
)]

use lichtung_plot::{Canvas, Plot};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn render_svg<K: lichtung_plot::kind::Marker>(
    plot: impl std::borrow::Borrow<Plot<K>>,
    name: &str,
) -> Result<String, lichtung_plot::Error> {
    let directory = tempdir(name);
    let output = directory.join("plot.svg");
    let result = plot
        .borrow()
        .svg_preserve_text(&output)
        .and_then(|_| std::fs::read_to_string(&output).map_err(lichtung_plot::Error::Io));
    std::fs::remove_dir_all(directory).unwrap();
    result
}

pub(super) fn render_canvas(
    canvas: impl std::borrow::Borrow<Canvas>,
    name: &str,
) -> Result<String, lichtung_plot::Error> {
    let directory = tempdir(name);
    let output = directory.join("canvas.svg");
    let result = canvas
        .borrow()
        .svg_preserve_text(&output)
        .and_then(|_| std::fs::read_to_string(&output).map_err(lichtung_plot::Error::Io));
    std::fs::remove_dir_all(directory).unwrap();
    result
}

pub(super) fn assert_unique_svg_ids(svg: &str) {
    let mut ids = std::collections::HashSet::new();
    for remainder in svg.split(" id=\"").skip(1) {
        let id = remainder.split('"').next().unwrap();
        assert!(ids.insert(id), "duplicate SVG id {id:?}");
    }
}

pub(super) fn attribute<'a>(line: &'a str, name: &str) -> &'a str {
    let marker = format!("{name}=\"");
    let start = line.find(&marker).unwrap() + marker.len();
    let end = line[start..].find('"').unwrap() + start;
    &line[start..end]
}

pub(super) fn tempdir(name: &str) -> std::path::PathBuf {
    loop {
        let sequence = TEMP_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "lichtung-plot-composition-{name}-{}-{}",
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
