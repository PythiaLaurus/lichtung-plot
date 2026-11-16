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
