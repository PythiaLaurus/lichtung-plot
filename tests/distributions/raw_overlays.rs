use lichtung_plot::{Canvas, Plot, PlotKind, RawOverlay, SourceRows, Theme};

use super::{parse_circles, unique_dir};

fn values(values: &[f64]) -> (lichtung_plot::CategoryValues, lichtung_plot::NumericValues) {
    (
        lichtung_plot::CategoryValues::new("category", vec!["a"; values.len()]).unwrap(),
        lichtung_plot::NumericValues::new("value", values.iter().copied()),
    )
}

fn plot(samples: &[f64], kind: PlotKind, overlay: RawOverlay) -> Plot {
    let (category, values) = values(samples);
    let plot = match kind {
        PlotKind::Boxplot => Plot::boxplot(category, values),
        PlotKind::Violin => Plot::violin(category, values),
        _ => panic!("unsupported raw-overlay fixture kind {kind:?}"),
    };
    plot.raw_overlay(overlay)
}

fn render(plot: Plot, name: &str) -> Result<String, lichtung_plot::Error> {
    let directory = unique_dir(name);
    let output = directory.join("plot.svg");
    let result = plot
        .svg(&output)
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
            plot(&[1.0, 2.0, 3.0, 4.0], kind, RawOverlay::Hidden),
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
            PlotKind::Boxplot => Plot::boxplot(category.clone(), value.clone()),
            PlotKind::Violin => Plot::violin(category.clone(), value.clone()),
            _ => unreachable!(),
        };
        let svg = render(plot.raw_overlay(RawOverlay::Points), "overlay-cell-count").unwrap();
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
fn visible_overlay_rejects_every_other_kind_before_backend_begin() {
    let category = || lichtung_plot::CategoryValues::new("category", ["a", "a", "a"]).unwrap();
    let numeric = || lichtung_plot::NumericValues::new("value", [1.0, 2.0, 3.0]);
    let unsupported = vec![
        Plot::scatter(numeric(), numeric()),
        Plot::histogram(numeric()),
        Plot::density(numeric()),
        Plot::qq(numeric()),
        Plot::line(numeric(), numeric()),
        Plot::bar(category(), numeric()),
        Plot::horizontal_bar(category(), numeric()),
        Plot::regression(numeric(), numeric()),
        Plot::beeswarm(category(), numeric(), numeric()),
    ]
    .into_boxed_slice();
    for plot in unsupported {
        assert!(plot
            .raw_overlay(RawOverlay::Points)
            .preflight()
            .unwrap_err()
            .to_string()
            .contains("visible RawOverlay"));
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
fn explicit_source_row_column_preserves_jitter_after_compaction_and_reordering() {
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
fn explicit_source_row_column_rejects_duplicate_identities() {
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
    .facet_col(
        lichtung_plot::CategoryValues::new(
            "facet",
            [
                "left", "right", "left", "right", "left", "right", "left", "right",
            ],
        )
        .unwrap(),
    )
    .size(900, 480);
    let svg = render(plot, "faceted-jitter").unwrap();
    assert_eq!(svg.matches("<g clip-path=").count(), 2);
    assert_eq!(overlay_circles(&svg).len(), 8);
}

#[test]
fn faceted_raw_overlay_does_not_draw_provenance_inside_panels() {
    let svg = render(
        Plot::boxplot(
            lichtung_plot::CategoryValues::new("category", ["a"; 4]).unwrap(),
            lichtung_plot::NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]),
        )
        .raw_overlay(RawOverlay::Jittered)
        .facet_col(lichtung_plot::CategoryValues::new("facet", ["only"; 4]).unwrap()),
        "faceted-provenance",
    )
    .unwrap();
    assert!(!svg.contains("original strict-sample observations"));
}

#[test]
fn grouped_hue_points_follow_dodge_color_and_global_first_observed_order() {
    let plot = Plot::violin(
        lichtung_plot::CategoryValues::new("category", ["a"; 8]).unwrap(),
        lichtung_plot::NumericValues::new("value", [1.0, 2.0, 3.0, 4.0, 2.0, 3.0, 4.0, 5.0]),
    )
    .group(
        lichtung_plot::CategoryValues::new(
            "group",
            ["g2", "g2", "g2", "g2", "g1", "g1", "g1", "g1"],
        )
        .unwrap(),
    )
    .hue(
        lichtung_plot::CategoryValues::new("hue", ["h2", "h2", "h2", "h2", "h1", "h1", "h1", "h1"])
            .unwrap(),
    )
    .raw_overlay(RawOverlay::Points);
    let svg = render(plot, "grouped-overlay").unwrap();
    let palette = Theme::Publication.palette().colors;
    for color in [palette[0], palette[1]] {
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
    let svg = render(
        plot(&[1.0, 2.0, 3.0, 4.0], PlotKind::Boxplot, RawOverlay::Points).y_limits(0.0, 2.0),
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
fn composed_figure_preserves_overlay_points_and_one_canvas_lifecycle() {
    let canvas = Canvas::grid(1, 2)
        .unwrap()
        .panel(
            0,
            0,
            plot(&[1.0, 2.0, 3.0, 4.0], PlotKind::Boxplot, RawOverlay::Points),
        )
        .unwrap()
        .panel(
            0,
            1,
            plot(&[1.0, 2.0, 3.0, 4.0], PlotKind::Violin, RawOverlay::Points),
        )
        .unwrap();
    let directory = unique_dir("composed-overlay");
    let output = directory.join("plot.svg");
    canvas.svg(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    assert_eq!(overlay_circles(&svg).len(), 8);
    assert_eq!(svg.matches("<svg ").count(), 1);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn narrow_composed_figure_rejects_before_lifecycle_or_artifact_creation() {
    let canvas = Canvas::grid(1, 1)
        .unwrap()
        .panel(
            0,
            0,
            plot(
                &[1.0, 2.0, 3.0, 4.0],
                PlotKind::Violin,
                RawOverlay::Jittered,
            ),
        )
        .unwrap()
        .size(180, 160);
    let directory = unique_dir("narrow-overlay-canvas");
    let output = directory.join("plot.svg");
    let error = canvas.svg(&output).err().unwrap();
    assert!(error.to_string().contains("figure layout is too dense"));
    assert!(!output.exists());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn adequate_composed_figure_accepts_raw_overlay() {
    Canvas::grid(1, 1)
        .unwrap()
        .panel(
            0,
            0,
            plot(
                &[1.0, 2.0, 3.0, 4.0],
                PlotKind::Violin,
                RawOverlay::Jittered,
            ),
        )
        .unwrap()
        .size(640, 480)
        .preflight()
        .unwrap();
}

#[test]
fn raw_overlay_report_provenance_is_retained_but_not_drawn() {
    let svg = render(
        plot(
            &[1.0, 2.0, 3.0, 4.0],
            PlotKind::Violin,
            RawOverlay::Jittered,
        ),
        "overlay-provenance",
    )
    .unwrap();
    assert!(!svg.contains("original strict-sample observations"));
    let provenance = RawOverlay::Jittered.provenance();
    assert!(provenance.contains("deterministic source-row displacement"));
    assert!(provenance.contains("summaries unchanged"));
}
