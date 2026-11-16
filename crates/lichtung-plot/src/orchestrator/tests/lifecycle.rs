use std::sync::atomic::Ordering;

use super::{
    test_directory, BackendFailure, Plot, PlotKind, RecordingBackend, FINAL_PLOT_RENDER_CALLS,
    TEMP_FILE_COUNTER,
};
use crate::NumericValues;

fn png_dimensions(bytes: &[u8]) -> (u32, u32) {
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    (
        u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
        u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
    )
}

fn png_density(bytes: &[u8]) -> Option<(u32, u32, u8)> {
    let mut offset = 8;
    while offset + 12 <= bytes.len() {
        let length = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        let chunk = &bytes[offset + 4..offset + 8];
        let data = &bytes[offset + 8..offset + 8 + length];
        if chunk == b"pHYs" {
            return Some((
                u32::from_be_bytes(data[..4].try_into().unwrap()),
                u32::from_be_bytes(data[4..8].try_into().unwrap()),
                data[8],
            ));
        }
        offset += length + 12;
    }
    None
}

#[test]
fn malformed_exact_tick_sources_fail_before_font_discovery_and_backend_begin() {
    let cases = [
        (Vec::new(), "must not be empty"),
        (vec![crate::AxisTick::new(f64::NAN, "")], "must be finite"),
        (
            vec![crate::AxisTick::new(2.0, ""), crate::AxisTick::new(1.0, "")],
            "strictly ascending",
        ),
        (
            vec![crate::AxisTick::new(4.0, "")],
            "outside resolved range",
        ),
    ];
    for (ticks, expected) in cases {
        for x_axis in [true, false] {
            let plot = Plot::scatter(
                NumericValues::new("x", [1.0, 2.0, 3.0]),
                NumericValues::new("y", [3.0, 1.0, 2.0]),
            )
            .x_limits(0.0, 3.0)
            .y_limits(0.0, 3.0);
            let plot = if x_axis {
                plot.x_ticks(ticks.clone())
            } else {
                plot.y_ticks(ticks.clone())
            };
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            let mut backend = RecordingBackend::default();
            let error = plot.render_with_backend(&mut backend).unwrap_err();
            assert!(
                matches!(&error, crate::Error::Data(message) if message.contains(expected)),
                "{error}"
            );
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
            assert_eq!((backend.begins, backend.ends), (0, 0));
        }
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one chained terminal exercise keeps Plot, Canvas, Scene, SVG, and PNG evidence together"
)]
fn chained_terminals_reuse_one_canonical_scene_and_plan_each_png() {
    FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(0));
    crate::scene::SCENE_CONSTRUCTION_CALLS.with(|calls| calls.set(0));
    crate::scene::PNG_TARGET_VALIDATION_CALLS.with(|calls| calls.set(0));
    crate::render::svg_backend::PNG_TARGET_PLAN_CALLS.with(|calls| calls.set(0));
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
    crate::font::canonical::FONT_OUTLINE_GLYPHS.with(|glyphs| glyphs.borrow_mut().clear());
    let directory = test_directory("canonical-scene-routing");
    let first = directory.join("first.svg");
    let outlined = directory.join("outlined.svg");
    let preview = directory.join("preview.png");
    let custom = directory.join("custom.png");
    let publication = directory.join("publication.png");
    let second = directory.join("second.svg");
    let output = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0]),
        NumericValues::new("y", [3.0, 1.0, 2.0]),
    )
    .size(320, 240)
    .title("Canonical scene")
    .svg_preserve_text(&first)
    .unwrap();
    assert_eq!((output.width(), output.height()), (320, 240));
    let font_terminal_counts = (
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
        crate::font::canonical::FONT_OUTLINE_GLYPHS.with(|glyphs| glyphs.borrow().len()),
    );
    let output = output.svg(&outlined).unwrap();
    let font_after_outline_counts = (
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
        crate::font::canonical::FONT_OUTLINE_GLYPHS.with(|glyphs| glyphs.borrow().len()),
    );
    let output = output
        .png_preview(&preview)
        .unwrap()
        .png_with_options(
            &custom,
            crate::PngOptions {
                sizing: crate::PngSizing::Dpi(192.0),
                compression: crate::PngCompression::High,
            },
        )
        .unwrap()
        .png(&publication)
        .unwrap()
        .svg_preserve_text(&second)
        .unwrap();
    assert_eq!((output.width(), output.height()), (320, 240));

    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(&second).unwrap()
    );
    assert!(!std::fs::read_to_string(outlined).unwrap().contains("<text"));
    let preview = std::fs::read(preview).unwrap();
    let custom = std::fs::read(custom).unwrap();
    let publication = std::fs::read(publication).unwrap();
    assert_eq!(
        (png_dimensions(&preview), png_density(&preview)),
        ((320, 240), None)
    );
    assert_eq!(
        (png_dimensions(&custom), png_density(&custom)),
        ((640, 480), Some((7_559, 7_559, 1)))
    );
    assert_eq!(
        (png_dimensions(&publication), png_density(&publication)),
        ((1000, 750), Some((11_811, 11_811, 1)))
    );
    assert_eq!(FINAL_PLOT_RENDER_CALLS.with(std::cell::Cell::get), 1);
    assert_eq!(
        crate::scene::SCENE_CONSTRUCTION_CALLS.with(std::cell::Cell::get),
        1
    );
    assert_eq!(
        crate::render::svg_backend::PNG_TARGET_PLAN_CALLS.with(std::cell::Cell::get),
        3
    );
    assert_eq!(
        crate::scene::PNG_TARGET_VALIDATION_CALLS.with(std::cell::Cell::get),
        3
    );
    assert_eq!(
        (
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
            crate::font::canonical::FONT_OUTLINE_GLYPHS.with(|glyphs| glyphs.borrow().len()),
        ),
        font_after_outline_counts,
        "later terminal conversions must reuse the lazily outlined scene"
    );
    assert_eq!(
        (font_after_outline_counts.0, font_after_outline_counts.1),
        (font_terminal_counts.0, font_terminal_counts.1),
        "terminal outlining must not discover or shape fonts"
    );
    assert!(font_after_outline_counts.2 > font_terminal_counts.2);

    crate::canvas::CANVAS_SCENE_RENDER_CALLS.with(|calls| calls.set(0));
    crate::scene::SCENE_CONSTRUCTION_CALLS.with(|calls| calls.set(0));
    crate::scene::PNG_TARGET_VALIDATION_CALLS.with(|calls| calls.set(0));
    crate::render::svg_backend::PNG_TARGET_PLAN_CALLS.with(|calls| calls.set(0));
    let canvas_first = directory.join("canvas-first.svg");
    let canvas_second = directory.join("canvas-second.svg");
    let canvas_preview = directory.join("canvas-preview.png");
    let canvas_custom = directory.join("canvas-custom.png");
    let canvas_publication = directory.join("canvas-publication.png");
    let output = crate::Canvas::grid(1, 1)
        .unwrap()
        .panel(
            0,
            0,
            Plot::scatter(
                NumericValues::new("x", [1.0, 2.0, 3.0]),
                NumericValues::new("y", [3.0, 1.0, 2.0]),
            ),
        )
        .unwrap()
        .size(320, 240)
        .svg_preserve_text(&canvas_first)
        .unwrap()
        .svg_preserve_text(&canvas_second)
        .unwrap()
        .png_preview(&canvas_preview)
        .unwrap()
        .png_with_options(
            &canvas_custom,
            crate::PngOptions {
                sizing: crate::PngSizing::Dpi(192.0),
                compression: crate::PngCompression::High,
            },
        )
        .unwrap()
        .png(&canvas_publication)
        .unwrap();
    assert_eq!((output.width(), output.height()), (320, 240));
    assert_eq!(
        std::fs::read(canvas_first).unwrap(),
        std::fs::read(canvas_second).unwrap()
    );
    let preview = std::fs::read(canvas_preview).unwrap();
    let custom = std::fs::read(canvas_custom).unwrap();
    let publication = std::fs::read(canvas_publication).unwrap();
    assert_eq!(
        (png_dimensions(&preview), png_density(&preview)),
        ((320, 240), None)
    );
    assert_eq!(
        (png_dimensions(&custom), png_density(&custom)),
        ((640, 480), Some((7_559, 7_559, 1)))
    );
    assert_eq!(
        (png_dimensions(&publication), png_density(&publication)),
        ((1000, 750), Some((11_811, 11_811, 1)))
    );
    assert_eq!(
        crate::canvas::CANVAS_SCENE_RENDER_CALLS.with(std::cell::Cell::get),
        1
    );
    assert_eq!(
        crate::scene::SCENE_CONSTRUCTION_CALLS.with(std::cell::Cell::get),
        1
    );
    assert_eq!(
        crate::render::svg_backend::PNG_TARGET_PLAN_CALLS.with(std::cell::Cell::get),
        3
    );
    assert_eq!(
        crate::scene::PNG_TARGET_VALIDATION_CALLS.with(std::cell::Cell::get),
        3
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn invalid_plot_preflights_before_output_path_access() {
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [1.0, 2.0]),
        crate::NumericValues::new("y", [1.0]),
    );
    let parent = std::env::temp_dir().join(format!(
        "lichtung-invalid-preflight-{}-{}",
        std::process::id(),
        TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let output = parent.join("missing").join("chart.svg");

    let error = plot.svg(&output).err().unwrap();

    assert!(error
        .to_string()
        .contains("plot value \"y\" has length 1, expected 2"));
    assert!(!parent.exists());
}

#[test]
fn scope_and_canvas_cleanup_errors_preserve_primary_context() {
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    );
    let mut backend = RecordingBackend {
        failures: vec![
            BackendFailure::Scatter,
            BackendFailure::ClipEnd,
            BackendFailure::End,
        ],
        ..RecordingBackend::default()
    };

    let error = plot
        .render_kind(PlotKind::Scatter, &mut backend)
        .unwrap_err()
        .to_string();

    assert!(error.contains("scatter draw failed"));
    assert!(error.contains("clip cleanup failed"));
    assert!(error.contains("canvas cleanup failed"));
    assert_eq!((backend.clip_ends, backend.ends), (1, 1));
}
