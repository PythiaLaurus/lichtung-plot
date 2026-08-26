#![allow(
    dead_code,
    reason = "helpers are shared by independently compiled integration test targets"
)]

use lichtung_plot::{Canvas, CategoryValues, NumericValues, Plot};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
pub(super) struct EstimateData {
    pub(super) x: NumericValues,
    pub(super) estimate: NumericValues,
    pub(super) lower: NumericValues,
    pub(super) upper: NumericValues,
    pub(super) trace: CategoryValues,
    pub(super) factor: CategoryValues,
    pub(super) term: CategoryValues,
    pub(super) facet: CategoryValues,
}

pub(super) fn estimate_data() -> EstimateData {
    EstimateData {
        x: NumericValues::new("x", [2.0, 1.0, 2.0, 1.0]),
        estimate: NumericValues::new("estimate", [3.0, 2.0, 4.0, 2.5]),
        lower: NumericValues::new("lower", [2.5, 1.5, 3.5, 2.0]),
        upper: NumericValues::new("upper", [3.5, 2.5, 4.5, 3.0]),
        trace: CategoryValues::new("trace", ["control", "control", "active", "active"]).unwrap(),
        factor: CategoryValues::new("factor", ["after", "before", "after", "before"]).unwrap(),
        term: CategoryValues::new("term", ["intercept", "time", "group", "interaction"]).unwrap(),
        facet: CategoryValues::new("facet", ["one", "one", "two", "two"]).unwrap(),
    }
}

pub(super) fn render_svg<K: lichtung_plot::kind::Marker>(
    plot: Plot<K>,
    name: &str,
) -> Result<String, lichtung_plot::Error> {
    let directory = tempdir(name);
    let output = directory.join("plot.svg");
    let result = plot
        .svg_preserve_text(&output)
        .and_then(|_| std::fs::read_to_string(&output).map_err(lichtung_plot::Error::Io));
    std::fs::remove_dir_all(directory).unwrap();
    result
}

pub(super) fn render_canvas(canvas: Canvas, name: &str) -> Result<String, lichtung_plot::Error> {
    let directory = tempdir(name);
    let output = directory.join("canvas.svg");
    let result = canvas
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

pub(super) fn assert_grid_layers_before_marks(svg: &str, grid_ids: &[&str]) {
    let first_marks = svg.find("<g clip-path=").unwrap();
    for grid_id in grid_ids {
        let grid = svg.find(&format!("<g id=\"{grid_id}\">")).unwrap();
        assert!(
            grid < first_marks,
            "grid layer {grid_id:?} rendered after marks"
        );
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
