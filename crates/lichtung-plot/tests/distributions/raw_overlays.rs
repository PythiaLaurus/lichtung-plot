use lichtung_plot::{Plot, RawOverlay, SourceRows};

use super::{parse_circles, unique_dir};

fn render<K: lichtung_plot::kind::Marker>(
    plot: impl std::borrow::Borrow<Plot<K>>,
    name: &str,
) -> Result<String, lichtung_plot::Error> {
    let directory = unique_dir(name);
    let output = directory.join("plot.svg");
    let result = plot
        .borrow()
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
