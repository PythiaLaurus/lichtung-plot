#![allow(
    dead_code,
    reason = "helpers are shared by independently compiled integration test targets"
)]

use lichtung_plot::{Canvas, Plot};

use crate::{df, DataFrame};

pub(super) fn estimate_data() -> DataFrame {
    df!(
        "x" => [2.0_f64, 1.0, 2.0, 1.0],
        "estimate" => [3.0_f64, 2.0, 4.0, 2.5],
        "lower" => [2.5_f64, 1.5, 3.5, 2.0],
        "upper" => [3.5_f64, 2.5, 4.5, 3.0],
        "trace" => ["control", "control", "active", "active"],
        "factor" => ["after", "before", "after", "before"],
        "term" => ["intercept", "time", "group", "interaction"],
        "facet" => ["one", "one", "two", "two"]
    )
    .unwrap()
}

pub(super) fn render_svg<K: lichtung_plot::kind::Marker>(
    plot: Plot<K>,
    name: &str,
) -> Result<String, lichtung_plot::Error> {
    let directory = tempdir(name);
    let output = directory.join("plot.svg");
    let result = plot
        .svg(&output)
        .and_then(|_| std::fs::read_to_string(&output).map_err(lichtung_plot::Error::Io));
    std::fs::remove_dir_all(directory).unwrap();
    result
}

pub(super) fn render_canvas(canvas: Canvas, name: &str) -> Result<String, lichtung_plot::Error> {
    let directory = tempdir(name);
    let output = directory.join("canvas.svg");
    let result = canvas
        .svg(&output)
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
    let directory = std::env::temp_dir().join(format!(
        "lichtung-plot-composition-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    directory
}
