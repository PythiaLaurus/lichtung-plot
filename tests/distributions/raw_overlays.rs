use lichtung_plot::{Canvas, FacetSpec, Plot, PlotKind, RawOverlay, SourceRows};

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

fn plot_with_default_overlay(samples: &[f64], kind: PlotKind) -> Plot {
    let (category, values) = values(samples);
    match kind {
        PlotKind::Boxplot => Plot::boxplot(category, values).erase(),
        PlotKind::Violin => Plot::violin(category, values).erase(),
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

fn circle_tags(svg: &str) -> Vec<&str> {
    svg.split("<circle ")
        .skip(1)
        .filter_map(|tail| tail.split_once("/>").map(|(tag, _)| tag))
        .collect()
}

#[test]
fn explicit_hidden_is_byte_identical_to_the_default_for_boxplot_and_violin() {
    for kind in [PlotKind::Boxplot, PlotKind::Violin] {
        let default = render(
            plot_with_default_overlay(&[1.0, 2.0, 3.0, 4.0], kind),
            "overlay-default",
        )
        .unwrap();
        let explicit = render(
            plot(&[1.0, 2.0, 3.0, 4.0], kind, RawOverlay::Hidden),
            "overlay-explicit-hidden",
        )
        .unwrap();
        assert_eq!(default, explicit, "{kind:?}");
    }
}

#[test]
fn visible_modes_draw_exactly_one_fixed_style_point_per_observation() {
    for kind in [PlotKind::Boxplot, PlotKind::Violin] {
        for overlay in [RawOverlay::Points, RawOverlay::Jittered] {
            let svg = render(
                plot(&[1.0, 2.0, 3.0, 4.0], kind, overlay),
                "visible-overlay",
            )
            .unwrap();
            let tags = circle_tags(&svg);
            assert_eq!(tags.len(), 4, "{kind:?} {overlay:?}");
            assert!(tags.iter().all(|tag| tag.contains("r=\"2\"")));
            assert!(tags.iter().all(|tag| tag.contains("stroke=")));
            assert!(svg.contains("<g clip-path="));
        }
    }
}

#[test]
fn singleton_two_point_and_repeated_cells_preserve_one_overlay_point_per_row() {
    let category = lichtung_plot::CategoryValues::new(
        "category",
        [
            "singleton",
            "pair",
            "pair",
            "repeated",
            "repeated",
            "repeated",
        ],
    )
    .unwrap();
    let value = lichtung_plot::NumericValues::new("value", [1.0, 2.0, 4.0, 3.0, 3.0, 3.0]);
    for kind in [PlotKind::Boxplot, PlotKind::Violin] {
        let plot = match kind {
            PlotKind::Boxplot => Plot::boxplot(category.clone(), value.clone())
                .raw_overlay(RawOverlay::Points)
                .erase(),
            PlotKind::Violin => Plot::violin(category.clone(), value.clone())
                .raw_overlay(RawOverlay::Points)
                .erase(),
            _ => unreachable!(),
        };
        let svg = render(plot, "overlay-cell-count").unwrap();
        assert_eq!(overlay_circles(&svg).len(), 6, "{kind:?}");
        let mut xs = overlay_circles(&svg)
            .into_iter()
            .map(|(x, _)| x.to_bits())
            .collect::<Vec<_>>();
        xs.sort_unstable();
        xs.dedup();
        assert_eq!(xs.len(), 3, "{kind:?}");
    }
}

#[test]
fn jitter_bytes_are_deterministic_and_value_reordering_does_not_move_source_rows() {
    for kind in [PlotKind::Boxplot, PlotKind::Violin] {
        let first = render(
            plot(&[1.0, 2.0, 3.0, 4.0], kind, RawOverlay::Jittered),
            "jitter-first",
        )
        .unwrap();
        let second = render(
            plot(&[1.0, 2.0, 3.0, 4.0], kind, RawOverlay::Jittered),
            "jitter-second",
        )
        .unwrap();
        assert_eq!(first, second, "{kind:?}");
    }
    let first = overlay_circles(
        &render(
            plot(
                &[1.0, 2.0, 3.0, 4.0],
                PlotKind::Boxplot,
                RawOverlay::Jittered,
            ),
            "jitter-order-first",
        )
        .unwrap(),
    )
    .into_iter()
    .map(|(x, _)| x.to_bits())
    .collect::<Vec<_>>();
    let reordered = overlay_circles(
        &render(
            plot(
                &[4.0, 3.0, 2.0, 1.0],
                PlotKind::Boxplot,
                RawOverlay::Jittered,
            ),
            "jitter-order-second",
        )
        .unwrap(),
    )
    .into_iter()
    .map(|(x, _)| x.to_bits())
    .collect::<Vec<_>>();
    assert_eq!(first, reordered);
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
fn plot_source_rows_reject_duplicate_identities() {
    assert!(SourceRows::new([4, 4])
        .unwrap_err()
        .to_string()
        .contains("duplicate 4"));
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
fn series_points_follow_dodge_color_and_global_first_observed_order() {
    let plot = Plot::violin(
        lichtung_plot::CategoryValues::new("category", ["a"; 8]).unwrap(),
        lichtung_plot::NumericValues::new("value", [1.0, 2.0, 3.0, 4.0, 2.0, 3.0, 4.0, 5.0]),
    )
    .series(
        lichtung_plot::CategoryValues::new("hue", ["h2", "h2", "h2", "h2", "h1", "h1", "h1", "h1"])
            .unwrap(),
    )
    .raw_overlay(RawOverlay::Points);
    let svg = render(plot, "series-overlay").unwrap();
    for color in [(0, 114, 178), (213, 94, 0)] {
        let hex = format!("#{:02X}{:02X}{:02X}", color.0, color.1, color.2);
        assert!(svg.contains(&hex));
    }
    assert!(svg.find(">H 2</text>").unwrap() < svg.find(">H 1</text>").unwrap());
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

#[test]
fn composed_canvas_preserves_overlay_points_and_one_canvas_lifecycle() {
    let canvas = Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, boxplot(&[1.0, 2.0, 3.0, 4.0], RawOverlay::Points))
        .unwrap()
        .panel(0, 1, violin(&[1.0, 2.0, 3.0, 4.0], RawOverlay::Points))
        .unwrap();
    let directory = unique_dir("composed-overlay");
    let output = directory.join("plot.svg");
    canvas.svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    assert_eq!(overlay_circles(&svg).len(), 8);
    assert_eq!(svg.matches("<svg ").count(), 1);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn narrow_composed_canvas_rejects_before_lifecycle_or_artifact_creation() {
    let canvas = Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, violin(&[1.0, 2.0, 3.0, 4.0], RawOverlay::Jittered))
        .unwrap()
        .size(180, 160);
    let directory = unique_dir("narrow-overlay-canvas");
    let output = directory.join("plot.svg");
    let error = canvas.svg(&output).err().unwrap();
    assert!(
        error
            .to_string()
            .contains("assigned tracks cannot preserve their hard requirements"),
        "{error}"
    );
    assert!(!output.exists());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn adequate_composed_canvas_accepts_raw_overlay() {
    Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, violin(&[1.0, 2.0, 3.0, 4.0], RawOverlay::Jittered))
        .unwrap()
        .size(640, 480)
        .preflight()
        .unwrap();
}
