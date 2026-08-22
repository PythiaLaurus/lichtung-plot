use lichtung_plot::{BarAggregation, Labels, Plot, PlotKind};

fn tempdir(name: &str) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "lichtung-rendering-{name}-{}-{}",
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
fn semantic_constructors_select_exact_plot_kinds() {
    let numeric = || lichtung_plot::NumericValues::new("value", [1.0, 2.0, 3.0]);
    let categories = || lichtung_plot::CategoryValues::new("category", ["a", "b", "c"]).unwrap();
    let plots = vec![
        (Plot::scatter(numeric(), numeric()), PlotKind::Scatter),
        (Plot::histogram(numeric()), PlotKind::Histogram),
        (Plot::density(numeric()), PlotKind::Density),
        (Plot::qq(numeric()), PlotKind::Qq),
        (Plot::line(numeric(), numeric()), PlotKind::Line),
        (
            Plot::bar(categories(), numeric()).bar_aggregation(BarAggregation::Identity),
            PlotKind::Bar,
        ),
        (
            Plot::horizontal_bar(categories(), numeric()).bar_aggregation(BarAggregation::Identity),
            PlotKind::HorizontalBar,
        ),
        (Plot::regression(numeric(), numeric()), PlotKind::Regression),
        (Plot::marginal(numeric(), numeric()), PlotKind::Marginal),
        (
            Plot::beeswarm(categories(), numeric(), numeric()),
            PlotKind::Beeswarm,
        ),
        (
            Plot::waterfall(categories(), numeric(), 0.0, 6.0),
            PlotKind::Waterfall,
        ),
    ];

    for (plot, expected) in plots {
        assert_eq!(plot.plot_kind(), Some(expected));
    }
}

#[test]
fn public_labels_drive_titles_axes_and_legend_text() {
    let plot = Plot::scatter(
        lichtung_plot::NumericValues::new("raw_x", [1.0, 2.0, 3.0, 4.0]),
        lichtung_plot::NumericValues::new("raw_y", [2.0, 3.0, 1.0, 4.0]),
    )
    .hue(
        lichtung_plot::CategoryValues::new("raw_group", ["control", "control", "active", "active"])
            .unwrap(),
    )
    .labels(
        Labels::humanized()
            .alias("raw_x", "Time")
            .alias("raw_y", "Response")
            .alias("raw_group", "Study arm"),
    )
    .title("Model result")
    .subtitle("Adjusted estimates")
    .caption("Source: integration fixture")
    .size(480, 340);
    let directory = tempdir("labels");
    let output = directory.join("plot.svg");

    plot.svg(&output).unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    for text in [
        "Time",
        "Response",
        "Study arm",
        "Model result",
        "Adjusted estimates",
        "Source: integration fixture",
    ] {
        assert!(svg.contains(text), "missing {text:?}");
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn preflight_failure_precedes_output_path_access() {
    let plot = Plot::scatter(
        lichtung_plot::NumericValues::new("x", [1.0, 2.0]),
        lichtung_plot::NumericValues::new("y", [1.0]),
    );
    let directory = tempdir("preflight");
    let output = directory.join("missing").join("plot.svg");

    let error = plot.svg(&output).err().unwrap();

    assert!(error.to_string().contains("has length 1, expected 2"));
    assert!(!output.exists());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn output_replaces_existing_destination_atomically() {
    let directory = tempdir("replace");
    let output = directory.join("plot.svg");
    std::fs::write(&output, b"old bytes").unwrap();

    Plot::scatter(
        lichtung_plot::NumericValues::new("x", [1.0, 2.0]),
        lichtung_plot::NumericValues::new("y", [2.0, 3.0]),
    )
    .svg(&output)
    .unwrap();

    let bytes = std::fs::read(&output).unwrap();
    assert!(bytes.starts_with(b"<svg"));
    assert!(!bytes.windows(9).any(|window| window == b"old bytes"));
    std::fs::remove_dir_all(directory).unwrap();
}
