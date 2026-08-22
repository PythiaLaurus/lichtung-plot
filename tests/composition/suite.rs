use lichtung_plot::{BarAggregation, Canvas, CanvasLegendPolicy, Plot, TextPanel};
use polars::prelude::*;

#[allow(
    clippy::duplicate_mod,
    reason = "each explicit integration target is an independent crate"
)]
#[path = "../support/native.rs"]
mod native;

fn tempdir(name: &str) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "lichtung-composition-{name}-{}-{}",
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
fn native_facets_render_each_observed_panel_with_unique_svg_ids() {
    let data = df!(
        "x" => [1.0_f64, 2.0, 1.0, 2.0],
        "y" => [2.0_f64, 3.0, 4.0, 5.0],
        "facet" => ["left", "left", "right", "right"]
    )
    .unwrap();
    let directory = tempdir("facets");
    let output = directory.join("facets.svg");
    Plot::scatter(native::numeric(&data, "x"), native::numeric(&data, "y"))
        .facet_col(native::category(&data, "facet"))
        .title("Faceted values")
        .size(720, 320)
        .svg(&output)
        .unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.contains(">Facet = Left</text>") && svg.contains(">Facet = Right</text>"));
    let mut ids = std::collections::HashSet::new();
    for remainder in svg.split(" id=\"").skip(1) {
        let id = remainder.split('"').next().unwrap();
        assert!(ids.insert(id), "duplicate SVG id {id:?}");
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn canvas_coordinates_shared_legends_headers_and_text_panels() {
    let data = df!(
        "category" => ["a", "a", "b", "b"],
        "group" => ["control", "active", "control", "active"],
        "value" => [1.0_f64, 2.0, 3.0, 4.0]
    )
    .unwrap();
    let plot = || {
        Plot::bar(
            native::category(&data, "category"),
            native::numeric(&data, "value"),
        )
        .group(native::category(&data, "group"))
        .hue(native::category(&data, "group"))
        .bar_aggregation(BarAggregation::Identity)
    };
    let directory = tempdir("canvas");
    let output = directory.join("canvas.svg");
    let canvas = Canvas::grid(1, 3)
        .unwrap()
        .panel(0, 0, plot().title("First plot"))
        .unwrap()
        .panel(0, 1, plot().title("Second plot"))
        .unwrap()
        .text_panel(
            0,
            2,
            TextPanel::new("Interpretation", "Both panels share one legend.").unwrap(),
        )
        .unwrap()
        .legend_policy(CanvasLegendPolicy::Shared)
        .column_header(0, 2, "Measured outcomes")
        .unwrap()
        .title("Composed result")
        .size(980, 380);

    canvas.svg(&output).unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    assert_eq!(svg.matches("<g id=\"figure-shared-legend\"").count(), 1);
    for text in [
        "Composed result",
        "Measured outcomes",
        "Interpretation",
        "Both",
        "legend.",
    ] {
        assert!(svg.contains(text), "missing {text:?}");
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn canvas_rejects_faceted_plot_nesting_during_preflight() {
    let plot = Plot::scatter(
        lichtung_plot::NumericValues::new("x", [1.0, 2.0]),
        lichtung_plot::NumericValues::new("y", [2.0, 3.0]),
    )
    .facet_col(lichtung_plot::CategoryValues::new("facet", ["a", "a"]).unwrap());
    let canvas = Canvas::grid(1, 1).unwrap().panel(0, 0, plot).unwrap();

    assert!(canvas
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("faceted chart cannot be nested"));
}

#[test]
fn canvas_grid_rejects_invalid_or_duplicate_cells_immediately() {
    assert!(Canvas::grid(0, 1).is_err());
    let plot = || {
        Plot::scatter(
            lichtung_plot::NumericValues::new("x", [1.0, 2.0]),
            lichtung_plot::NumericValues::new("y", [2.0, 3.0]),
        )
    };
    let canvas = Canvas::grid(1, 1).unwrap().panel(0, 0, plot()).unwrap();
    assert!(canvas.panel(0, 0, plot()).is_err());
}
