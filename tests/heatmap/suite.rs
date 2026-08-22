use lichtung_plot::{
    Canvas, HeatmapAnnotations, HeatmapLayout, HeatmapScale, NumericLabelFormat, Plot, PlotKind,
};
use polars::prelude::*;

#[allow(
    clippy::duplicate_mod,
    reason = "each explicit integration target is an independent crate"
)]
#[path = "../support/native.rs"]
mod native;

fn fixture() -> DataFrame {
    df!(
        "column" => ["left", "right", "left", "right"],
        "row" => ["top", "top", "bottom", "bottom"],
        "value" => [0.0_f64, 1.0, 0.5, 0.75]
    )
    .unwrap()
}

fn heatmap(data: &DataFrame) -> Plot {
    Plot::heatmap(
        native::category(data, "column"),
        native::category(data, "row"),
        native::numeric(data, "value"),
    )
    .size(560, 360)
}

fn tempdir(name: &str) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "lichtung-heatmap-{name}-{}-{}",
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
fn native_heatmap_exposes_typed_configuration() {
    let data = fixture();
    let plot = heatmap(&data)
        .color_label("Response")
        .heatmap_layout(HeatmapLayout::UpperTriangle { diagonal: true })
        .heatmap_scale(HeatmapScale::Diverging { center: 0.5 })
        .heatmap_color_limits(-1.0, 2.0)
        .heatmap_annotations(HeatmapAnnotations::Values(
            NumericLabelFormat::new(2).unwrap(),
        ));

    assert_eq!(plot.plot_kind(), Some(PlotKind::Heatmap));
    assert_eq!(plot.color_label_text(), Some("Response"));
    assert_eq!(
        plot.heatmap_layout_spec(),
        HeatmapLayout::UpperTriangle { diagonal: true }
    );
    assert_eq!(plot.heatmap_color_limits_spec(), Some((-1.0, 2.0)));
}

#[test]
fn heatmap_rejects_duplicate_cells_before_creating_output() {
    let data = df!(
        "column" => ["left", "left"],
        "row" => ["top", "top"],
        "value" => [1.0_f64, 2.0]
    )
    .unwrap();
    let directory = tempdir("duplicates");
    let output = directory.join("plot.svg");

    let error = heatmap(&data).svg(&output).err().unwrap();

    assert!(error.to_string().contains("duplicate"));
    assert!(!output.exists());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn heatmap_highlights_and_value_annotations_reach_svg() {
    let mut data = fixture();
    data.with_column(Series::new("highlight".into(), [true, false, false, true]).into())
        .unwrap();
    let directory = tempdir("annotations");
    let output = directory.join("plot.svg");

    heatmap(&data)
        .heatmap_highlight(native::boolean(&data, "highlight"))
        .heatmap_annotations(HeatmapAnnotations::Values(
            NumericLabelFormat::new(2).unwrap(),
        ))
        .svg(&output)
        .unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.contains(">1.00</text>"));
    assert!(svg.contains(">0.75</text>"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn heatmaps_compose_as_owned_canvas_panels() {
    let data = fixture();
    let directory = tempdir("canvas");
    let output = directory.join("canvas.svg");
    let canvas = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, heatmap(&data).title("First"))
        .unwrap()
        .panel(0, 1, heatmap(&data).title("Second"))
        .unwrap()
        .size(900, 420);

    canvas.svg(&output).unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.contains("First"));
    assert!(svg.contains("Second"));
    std::fs::remove_dir_all(directory).unwrap();
}
