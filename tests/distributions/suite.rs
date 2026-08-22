use lichtung_plot::{HistogramBins, HistogramStat, Plot, QqLine, Theme};
use polars::prelude::*;

#[allow(
    clippy::duplicate_mod,
    reason = "each explicit integration target is an independent crate"
)]
#[path = "../support/native.rs"]
mod native;
#[path = "../support/visual.rs"]
mod support;

fn fixture() -> DataFrame {
    df!(
        "value" => [1.0_f64, 1.5, 2.0, 2.5, 3.0, 3.5],
        "group" => ["a", "a", "a", "b", "b", "b"]
    )
    .unwrap()
}

#[test]
fn histogram_density_and_qq_use_closed_native_constructors() {
    let data = fixture();
    let values = || native::numeric(&data, "value");
    let mut histogram = Plot::histogram(values());
    let _ = histogram
        .histogram_bins(HistogramBins::Count(4))
        .histogram_stat(HistogramStat::Density);
    let mut qq = Plot::qq(values());
    let _ = qq.qq_line(QqLine::Quartiles);
    let plots = [histogram, Plot::density(values()), qq];

    for plot in plots {
        plot.preflight().unwrap();
    }
}

#[test]
fn grouped_density_preserves_distinct_group_and_hue_identity() {
    let data = fixture();
    let directory = support::unique_dir("grouped-density");
    let output = directory.join("density.svg");
    Plot::density(native::numeric(&data, "value"))
        .group(native::category(&data, "group"))
        .hue(native::category(&data, "group"))
        .theme(Theme::Nature)
        .title("Grouped density")
        .svg(&output)
        .unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.contains("Grouped density"));
    assert!(svg.contains(">A</text>") && svg.contains(">B</text>"));
    assert_eq!(svg.matches("<g id=\"legend\"").count(), 1);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn histogram_options_are_reflected_in_public_svg_labels() {
    let data = fixture();
    let directory = support::unique_dir("histogram-density");
    let output = directory.join("histogram.svg");
    let mut plot = Plot::histogram(native::numeric(&data, "value"));
    let _ = plot
        .histogram_bins(HistogramBins::Width(0.5))
        .histogram_stat(HistogramStat::Probability);
    plot.svg(&output).unwrap();

    let svg = std::fs::read_to_string(output).unwrap();
    assert!(svg.contains(">Probability</text>"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn strict_distribution_inputs_reject_non_finite_values() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error = Plot::density(lichtung_plot::NumericValues::new(
            "value",
            [1.0, value, 2.0],
        ))
        .preflight()
        .unwrap_err();
        assert!(error.to_string().contains("non-finite"));
    }
}
