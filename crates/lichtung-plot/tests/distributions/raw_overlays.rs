use lichtung_plot::{FacetSpec, Plot, PlotKind, RawOverlay, SourceRows};

use super::{parse_circles, unique_dir};

fn values(values: &[f64]) -> (lichtung_plot::CategoryValues, lichtung_plot::NumericValues) {
    (
        lichtung_plot::CategoryValues::new("category", vec!["a"; values.len()]).unwrap(),
        lichtung_plot::NumericValues::new("value", values.iter().copied()),
    )
}

fn boxplot(samples: &[f64], overlay: RawOverlay) -> Plot<lichtung_plot::kind::Boxplot> {
    let (category, values) = values(samples);
    Plot::boxplot(category, values).raw_overlay(overlay)
}

fn violin(samples: &[f64], overlay: RawOverlay) -> Plot<lichtung_plot::kind::Violin> {
    let (category, values) = values(samples);
    Plot::violin(category, values).raw_overlay(overlay)
}

fn plot(samples: &[f64], kind: PlotKind, overlay: RawOverlay) -> Plot {
    match kind {
        PlotKind::Boxplot => boxplot(samples, overlay).erase(),
        PlotKind::Violin => violin(samples, overlay).erase(),
        _ => panic!("unsupported raw-overlay fixture kind {kind:?}"),
    }
}

fn render<K: lichtung_plot::kind::Marker>(
    plot: Plot<K>,
    name: &str,
) -> Result<String, lichtung_plot::Error> {
    let directory = unique_dir(name);
    let output = directory.join("plot.svg");
    let result = plot
        .svg_preserve_text(&output)
        .and_then(|_| std::fs::read_to_string(&output).map_err(lichtung_plot::Error::Io));
    std::fs::remove_dir_all(directory).unwrap();
    result
}

fn overlay_circles(svg: &str) -> Vec<(f64, f64)> {
    parse_circles(svg)
}

#[test]
fn plot_source_rows_preserve_jitter_after_compaction_and_reordering() {
    let make = |values: [f64; 3], rows: [usize; 3]| {
        Plot::boxplot(
            lichtung_plot::CategoryValues::new("category", ["a"; 3]).unwrap(),
            lichtung_plot::NumericValues::new("value", values),
        )
        .source_rows(SourceRows::new(rows).unwrap())
        .raw_overlay(RawOverlay::Jittered)
    };
    let first = overlay_circles(&render(make([1.0, 2.0, 3.0], [5, 1, 9]), "source-first").unwrap());
    let reordered =
        overlay_circles(&render(make([3.0, 1.0, 2.0], [9, 5, 1]), "source-second").unwrap());
    assert_eq!(reordered[0].0.to_bits(), first[2].0.to_bits());
    assert_eq!(reordered[1].0.to_bits(), first[0].0.to_bits());
    assert_eq!(reordered[2].0.to_bits(), first[1].0.to_bits());
}

#[test]
fn facets_keep_original_source_rows_for_jitter() {
    let plot = Plot::boxplot(
        lichtung_plot::CategoryValues::new("category", ["a"; 8]).unwrap(),
        lichtung_plot::NumericValues::new("value", [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]),
    )
    .raw_overlay(RawOverlay::Jittered)
    .facet(FacetSpec::column(
        lichtung_plot::CategoryValues::new(
            "facet",
            [
                "left", "right", "left", "right", "left", "right", "left", "right",
            ],
        )
        .unwrap(),
    ))
    .size(900, 480);
    let svg = render(plot, "faceted-jitter").unwrap();
    assert_eq!(svg.matches("<g clip-path=").count(), 2);
    assert_eq!(overlay_circles(&svg).len(), 8);
}

#[test]
fn visible_boxplot_overlay_suppresses_separate_fliers_but_keeps_all_rows() {
    let hidden = render(
        plot(&[1.0, 1.0, 1.0, 9.0], PlotKind::Boxplot, RawOverlay::Hidden),
        "hidden-fliers",
    )
    .unwrap();
    let visible = render(
        plot(&[1.0, 1.0, 1.0, 9.0], PlotKind::Boxplot, RawOverlay::Points),
        "visible-fliers",
    )
    .unwrap();
    assert_eq!(overlay_circles(&hidden).len(), 1);
    assert_eq!(overlay_circles(&visible).len(), 4);
}

#[test]
fn overlay_is_clipped_by_explicit_limits_and_drawn_between_summaries_and_axes() {
    let (category, values) = values(&[1.0, 2.0, 3.0, 4.0]);
    let svg = render(
        Plot::boxplot(category, values)
            .raw_overlay(RawOverlay::Points)
            .y_limits(0.0, 2.0),
        "overlay-clipping",
    )
    .unwrap();
    let marks = svg.find("<g clip-path=").unwrap();
    let scatter = svg.find("<circle ").unwrap();
    let axes = svg.find("<g id=\"axes\"").unwrap();
    assert!(marks < scatter && scatter < axes);
    assert_eq!(overlay_circles(&svg).len(), 4);
}
