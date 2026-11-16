use super::*;

#[test]
fn repeated_plot_exports_rerender_but_output_copies_retry_without_callbacks() {
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    };
    let directory = tempfile::tempdir().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let fail = Arc::new(AtomicBool::new(false));
    let callback_calls = calls.clone();
    let callback_fail = fail.clone();
    // This witnesses lifecycle/replay, not the minimum fit for complete function ticks.
    let plot = plot()
        .size(800, 600)
        .x_tick_generation(lichtung_plot::TickGeneration::function(move |_| {
            callback_calls.fetch_add(1, Ordering::SeqCst);
            if callback_fail.load(Ordering::SeqCst) {
                return Err(lichtung_plot::Error::Data("caller failure".into()));
            }
            Ok(lichtung_plot::TickPositions::Data(vec![1., 2., 3.]))
        }));
    let first = directory.path().join("first.svg");
    let output = plot.svg_preserve_text(&first).unwrap();
    let materialized = calls.load(Ordering::SeqCst);
    assert!(materialized > 0);
    let shared = output.clone();
    fail.store(true, Ordering::SeqCst);
    assert!(output
        .svg_preserve_text(directory.path().join("missing/failed.svg"))
        .is_err());
    assert!(output
        .png_with_options(
            directory.path().join("invalid.png"),
            PngOptions {
                sizing: PngSizing::Scale(0.0),
                compression: PngCompression::Fast,
            }
        )
        .is_err());
    shared
        .svg_preserve_text(directory.path().join("replay.svg"))
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), materialized);
    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(directory.path().join("replay.svg")).unwrap()
    );

    assert!(plot
        .svg_preserve_text(directory.path().join("callback-error.svg"))
        .is_err());
    assert!(calls.load(Ordering::SeqCst) > materialized);
    fail.store(false, Ordering::SeqCst);
    let before = calls.load(Ordering::SeqCst);
    assert!(plot
        .svg_preserve_text(directory.path().join("missing/new-render.svg"))
        .is_err());
    assert!(calls.load(Ordering::SeqCst) > before);
    let before = calls.load(Ordering::SeqCst);
    plot.svg_preserve_text(directory.path().join("new-render.svg"))
        .unwrap();
    assert!(calls.load(Ordering::SeqCst) > before);
    let edited = plot.clone().title("Later configuration");
    edited
        .svg_preserve_text(directory.path().join("edited.svg"))
        .unwrap();
    output
        .svg_preserve_text(directory.path().join("still-frozen.svg"))
        .unwrap();
    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(directory.path().join("still-frozen.svg")).unwrap()
    );
}

#[test]
fn canvas_exports_and_failed_placements_preserve_configuration_copies_and_frozen_outputs() {
    let directory = tempfile::tempdir().unwrap();
    let mut canvas = Canvas::grid(1, 1).unwrap();
    canvas.place_panel(0, 0, 1, 1, plot(), false).unwrap();
    let original = canvas.clone();
    let first = directory.path().join("first.svg");
    let output = canvas.svg_preserve_text(&first).unwrap();
    assert!(canvas.place_panel(0, 0, 0, 1, plot(), true).is_err());
    assert!(canvas.place_panel(0, 0, 1, 1, plot(), false).is_err());
    assert!(canvas
        .svg_preserve_text(directory.path().join("missing/failure.svg"))
        .is_err());
    canvas
        .svg_preserve_text(directory.path().join("unchanged.svg"))
        .unwrap();
    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(directory.path().join("unchanged.svg")).unwrap()
    );
    canvas
        .place_panel(0, 0, 1, 1, plot().title("Replacement capture"), true)
        .unwrap();
    canvas
        .svg_preserve_text(directory.path().join("replacement.svg"))
        .unwrap();
    original
        .svg_preserve_text(directory.path().join("original.svg"))
        .unwrap();
    assert!(output
        .svg(directory.path().join("missing/replay.svg"))
        .is_err());
    output
        .svg_preserve_text(directory.path().join("replay.svg"))
        .unwrap();
    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(directory.path().join("original.svg")).unwrap()
    );
    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(directory.path().join("replay.svg")).unwrap()
    );
    let changed = std::fs::read_to_string(directory.path().join("replacement.svg")).unwrap();
    assert!(changed.contains("Replacement capture") && !changed.contains("Selectable scene text"));
}

#[test]
fn guide_ellipsis_retains_original_metadata_and_replays_one_selected_view_across_outputs() {
    use lichtung_plot::{
        CategoryValues, GuideTextRole, TextBounds, TextElision, TextEllipsis, TextFailure,
        TextLayout, TextOverflow,
    };
    let directory = unique_dir("derived-guide-view");
    let first = directory.join("first.svg");
    let second = directory.join("second.svg");
    let original = "A very long guide identity with an important ending";
    let output = Plot::scatter(
        NumericValues::new("x", [0.0, 1.0]),
        NumericValues::new("y", [1.0, 0.0]),
    )
    .hue(CategoryValues::new("cohort", [original, original]).unwrap())
    .labels(lichtung_plot::Labels::raw())
    .guide_text_layout(
        GuideTextRole::Entry,
        TextLayout::default()
            .bounds(TextBounds::new(Some(70.0), None, None).unwrap())
            .overflow(TextOverflow::ellipsis(
                TextEllipsis::new(TextElision::Middle),
                TextFailure::Error,
            )),
    )
    .size(500, 350)
    .svg_preserve_text(&first)
    .unwrap();
    output
        .svg(directory.join("paths.svg"))
        .unwrap()
        .png_preview(directory.join("preview.png"))
        .unwrap()
        .svg_preserve_text(&second)
        .unwrap();
    let bytes = std::fs::read(first).unwrap();
    assert_eq!(bytes, std::fs::read(second).unwrap());
    let source = std::str::from_utf8(&bytes).unwrap();
    let document = roxmltree::Document::parse(source).unwrap();
    assert!(document
        .descendants()
        .any(|node| node.attribute("data-original-source") == Some(original)));
    let painted = document
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .filter_map(|node| node.text())
        .collect::<Vec<_>>();
    assert!(!painted.contains(&original));
    assert!(painted
        .iter()
        .any(|text| text.contains('…') && text.starts_with('A') && text.ends_with('g')));
    let outlined = std::fs::read_to_string(directory.join("paths.svg")).unwrap();
    assert!(!outlined.contains("<text"));
    assert!(outlined.contains("data-original-source="));
    assert_eq!(
        png_dimensions(&std::fs::read(directory.join("preview.png")).unwrap()),
        (500, 350)
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn assigned_canvas_freezes_combined_anchor_rotation_ellipsis_and_fractional_fit() {
    use lichtung_plot::{
        Text, TextAnchor, TextBounds, TextElision, TextEllipsis, TextFailure, TextFit,
        TextFitPriority, TextLayout, TextOverflow, TextPanel, TextRotation, TextRotationMode,
        Theme,
    };
    let directory = unique_dir("assigned-font-fit");
    let first = directory.join("first.svg");
    let second = directory.join("second.svg");
    let source = "An independently authored scientific explanation with a long ending";
    let style = Theme::publication()
        .typography()
        .get_composition()
        .get_body()
        .clone()
        .font_size(14.25)
        .unwrap();
    let body = Text::new(source).style(style).layout(
        TextLayout::default()
            .anchor(TextAnchor::center())
            .rotation(TextRotation::degrees(30.05).unwrap())
            .rotation_mode(TextRotationMode::AlignAfterRotation)
            .bounds(TextBounds::new(Some(100.0), Some(30.0), Some(1)).unwrap())
            .fit(TextFit::shrink_to(10.25).unwrap())
            .fit_priority(TextFitPriority::PreferredSizeFirst)
            .overflow(TextOverflow::ellipsis(
                TextEllipsis::new(TextElision::Middle),
                TextFailure::Error,
            )),
    );
    let output = Canvas::grid(1, 1)
        .unwrap()
        .text_panel(0, 0, TextPanel::new("Independent heading", body).unwrap())
        .unwrap()
        .max_width(400)
        .svg_preserve_text(&first)
        .unwrap();
    let dimensions = (output.width(), output.height());
    assert!(dimensions.0 <= 400);
    output
        .svg(directory.join("outlines.svg"))
        .unwrap()
        .png_preview(directory.join("preview.png"))
        .unwrap()
        .png_with_options(
            directory.join("scaled.png"),
            PngOptions {
                sizing: PngSizing::Scale(2.0),
                compression: PngCompression::Fast,
            },
        )
        .unwrap()
        .svg_preserve_text(&second)
        .unwrap();
    let bytes = std::fs::read(first).unwrap();
    assert_eq!(bytes, std::fs::read(second).unwrap());
    let svg = std::str::from_utf8(&bytes).unwrap();
    let document = roxmltree::Document::parse(svg).unwrap();
    let selected = document
        .descendants()
        .find(|node| node.attribute("data-original-source") == Some(source))
        .unwrap();
    let painted = selected
        .descendants()
        .find(|node| node.has_tag_name("text"))
        .unwrap();
    assert!(painted.text().unwrap().contains('…'));
    let size: f64 = painted.attribute("font-size").unwrap().parse().unwrap();
    assert!((10.25..=14.25).contains(&size));
    assert!(svg.contains("rotate(30.05"));
    assert_eq!(
        png_dimensions(&std::fs::read(directory.join("preview.png")).unwrap()),
        dimensions
    );
    assert_eq!(
        png_dimensions(&std::fs::read(directory.join("scaled.png")).unwrap()),
        (dimensions.0 * 2, dimensions.1 * 2)
    );
    std::fs::remove_dir_all(directory).unwrap();
}

fn plot() -> Plot<lichtung_plot::kind::Scatter> {
    Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0]),
        NumericValues::new("y", [3.0, 1.0, 2.0]),
    )
    .x_tick_format(
        lichtung_plot::NumericFormat::default()
            .prefix("x=")
            .unwrap(),
    )
    .y_tick_format(
        lichtung_plot::NumericFormat::new(
            lichtung_plot::NumericNotation::Engineering,
            lichtung_plot::NumericPrecision::Significant(3),
        )
        .unwrap()
        .prefix("y=")
        .unwrap(),
    )
    .title("Selectable scene text")
    .size(320, 240)
}

#[test]
fn plot_svg_operations_keep_complete_text_and_call_local_replay() {
    let directory = unique_dir("output-text-representation");
    let paths = directory.join("paths.svg");
    let selectable = directory.join("selectable.svg");
    let second_paths = directory.join("second-paths.svg");
    let second_selectable = directory.join("second-selectable.svg");
    plot()
        .svg(&paths)
        .unwrap()
        .svg_preserve_text(&selectable)
        .unwrap()
        .svg(&second_paths)
        .unwrap()
        .svg_preserve_text(&second_selectable)
        .unwrap();

    let paths = std::fs::read(paths).unwrap();
    let selectable = std::fs::read(selectable).unwrap();
    assert_eq!(paths, std::fs::read(second_paths).unwrap());
    assert_eq!(selectable, std::fs::read(second_selectable).unwrap());
    let paths = String::from_utf8(paths).unwrap();
    assert!(!paths.contains("<text") && paths.contains("<path"));
    let selectable = String::from_utf8(selectable).unwrap();
    assert!(selectable.contains(">Selectable scene text</text>"));
    assert!(selectable.contains(">x=") && selectable.contains(">y="));
    assert!(selectable.contains("width=\"320\" height=\"240\""));
    std::fs::remove_dir_all(directory).unwrap();
}
