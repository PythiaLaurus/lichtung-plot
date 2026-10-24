use std::sync::atomic::Ordering;

use super::{
    test_directory, BackendFailure, Plot, PlotKind, RecordingBackend, Significance,
    SignificanceMark, ValueSort, FINAL_PLOT_RENDER_CALLS, PLOT_PLAN_CALLS, RENDER_PANEL_CALLS,
    TEMP_FILE_COUNTER,
};
use crate::orchestrator::{DeferredBackend, TEXT_PLAN_CAPTURE_SESSIONS, TEXT_REPLAY_SESSIONS};
use crate::render::backend::{Backend, LineStyle, Point2D, SurfaceRole, TextAnchor, TextStyle};
use crate::{AxisValues, CategoryValues, NumericValues};

fn valid_line() -> ([Point2D; 2], LineStyle) {
    (
        [Point2D::new(1.0, 1.0), Point2D::new(2.0, 2.0)],
        LineStyle::solid(crate::theme::Color::rgb(0, 0, 0), 1.0),
    )
}

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
fn deferred_backend_enforces_dimensions_and_one_shot_lifecycle() {
    let mut recorded = RecordingBackend::default();
    {
        let mut backend = DeferredBackend::new(&mut recorded, 320, 200);
        assert!(backend.begin(319, 200).is_err());
        let (points, style) = valid_line();
        backend.draw_line(&points, &style).unwrap();
        assert!(backend.begin(320, 200).is_err());
        backend.end().unwrap();
        assert!(backend.draw_line(&points, &style).is_err());
        assert!(backend.end().is_err());
        backend.finish(Ok(()), "test canvas").unwrap();
    }

    assert_eq!(recorded.begin_sizes, [(320, 200)]);
    assert_eq!((recorded.begins, recorded.ends), (1, 1));
}

#[test]
fn deferred_backend_empty_success_starts_and_finalizes_once() {
    let mut recorded = RecordingBackend::default();

    DeferredBackend::new(&mut recorded, 320, 200)
        .finish(Ok(()), "test canvas")
        .unwrap();

    assert_eq!(recorded.begin_sizes, [(320, 200)]);
    assert_eq!((recorded.begins, recorded.ends), (1, 1));
}

#[test]
fn deferred_backend_forwards_signed_text_positions_unchanged() {
    let mut recorded = RecordingBackend::default();
    {
        let mut backend = DeferredBackend::new(&mut recorded, 320, 200);
        backend
            .draw_text(
                Point2D::new(-7.0, 225.0),
                "outside",
                &TextStyle {
                    color: crate::theme::Color::rgb(0, 0, 0),
                    font_family: crate::font::canonical::FontStack::new([
                        crate::font::canonical::FontFamily::generic(
                            crate::font::canonical::GenericFontFamily::SansSerif,
                        ),
                    ])
                    .unwrap(),
                    font_size: 12,
                    weight: crate::TextWeight::Normal,
                    anchor: TextAnchor::Start,
                    rotation: 0.0,
                },
            )
            .unwrap();
        backend.finish(Ok(()), "test canvas").unwrap();
    }

    assert_eq!(recorded.texts, [(-7.0, 225.0, "outside".into())]);
}

#[test]
fn deferred_backend_failed_start_and_end_are_terminal() {
    let (points, style) = valid_line();
    let mut begin_failure = RecordingBackend {
        failures: vec![BackendFailure::Begin],
        ..RecordingBackend::default()
    };
    {
        let mut backend = DeferredBackend::new(&mut begin_failure, 320, 200);
        assert!(backend.draw_line(&points, &style).is_err());
        assert!(backend.draw_line(&points, &style).is_err());
    }
    assert_eq!((begin_failure.begins, begin_failure.ends), (1, 0));

    let mut end_failure = RecordingBackend {
        failures: vec![BackendFailure::End],
        ..RecordingBackend::default()
    };
    {
        let mut backend = DeferredBackend::new(&mut end_failure, 320, 200);
        backend.draw_line(&points, &style).unwrap();
        assert!(backend.end().is_err());
        assert!(backend.end().is_err());
    }
    assert_eq!((end_failure.begins, end_failure.ends), (1, 1));
}

#[test]
fn deferred_backend_reports_cleanup_only_failure() {
    let mut recorded = RecordingBackend {
        failures: vec![BackendFailure::End],
        ..RecordingBackend::default()
    };

    let error = DeferredBackend::new(&mut recorded, 320, 200)
        .finish(Ok(()), "test canvas")
        .unwrap_err();

    assert!(error.to_string().contains("failed to close test canvas"));
    assert!(error.to_string().contains("canvas cleanup failed"));
    assert_eq!((recorded.begins, recorded.ends), (1, 1));
}

#[test]
fn impossible_wrapped_caption_fails_before_backend_begin() {
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .caption("diagnostic provenance and model adjustment details ".repeat(8))
    .size(160, 120);
    let mut backend = RecordingBackend::default();

    let error = plot.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("root title block"));
    assert!(error.to_string().contains("TrackOrResidual"));
    assert_eq!(backend.begins, 0);
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
fn exact_root_reservation_failures_precede_concrete_backend_begin() {
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0]),
        NumericValues::new("y", [3.0, 1.0, 2.0]),
    );
    for stage in [
        crate::layout::ReservationStage::Prepared,
        crate::layout::ReservationStage::Measured,
        crate::layout::ReservationStage::Final,
    ] {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        crate::layout::force_reservation_failure(stage);
        let mut backend = RecordingBackend::default();

        let error = plot
            .render_with_backend(&mut backend)
            .unwrap_err()
            .to_string();

        assert!(error.contains(&format!("reservation stage {}", stage.number())));
        assert_eq!((backend.begins, backend.ends), (0, 0));
        if stage == crate::layout::ReservationStage::Prepared {
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0,
                "the scalar-map reservation must precede host font discovery"
            );
        }
    }
}

#[test]
fn standalone_terminal_resolves_and_plans_once_with_one_final_lifecycle() {
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    crate::font::canonical::FONT_FINAL_SEALS.with(|calls| calls.set(0));
    PLOT_PLAN_CALLS.with(|calls| calls.set(0));
    FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(0));
    TEXT_PLAN_CAPTURE_SESSIONS.with(|calls| calls.set(0));
    TEXT_REPLAY_SESSIONS.with(|calls| calls.set(0));
    crate::font::canonical::FONT_PAIR_RESOLUTION_CALLS.with(|calls| calls.set(0));
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .theme(crate::Theme::nature())
    .size(640, 480);
    let mut backend = RecordingBackend::default();

    plot.render_with_backend(&mut backend).unwrap();

    assert_eq!(PLOT_PLAN_CALLS.with(std::cell::Cell::get), 1);
    assert_eq!(FINAL_PLOT_RENDER_CALLS.with(std::cell::Cell::get), 1);
    assert_eq!(TEXT_PLAN_CAPTURE_SESSIONS.with(std::cell::Cell::get), 1);
    assert_eq!(TEXT_REPLAY_SESSIONS.with(std::cell::Cell::get), 1);
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        1
    );
    assert_eq!(
        crate::font::canonical::FONT_FINAL_SEALS.with(std::cell::Cell::get),
        1
    );
    assert_eq!(
        crate::font::canonical::FONT_PAIR_RESOLUTION_CALLS.with(std::cell::Cell::get),
        1
    );
    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert_eq!(
        backend
            .surfaces
            .iter()
            .map(|surface| surface.role)
            .collect::<Vec<_>>(),
        [SurfaceRole::CanvasRoot, SurfaceRole::PlotPanel]
    );
    assert_eq!(
        backend.surfaces[0].bounds,
        crate::layout::PanelBox::new(0.0, 640.0, 0.0, 480.0)
    );
    assert_eq!(backend.surfaces[1].bounds, backend.clips[0]);
    assert!(backend
        .surfaces
        .iter()
        .all(|surface| surface.paint == crate::theme::Color::rgb(255, 255, 255)));
}

#[test]
fn public_preflight_uses_the_prepared_plan_without_a_final_lifecycle() {
    PLOT_PLAN_CALLS.with(|calls| calls.set(0));
    FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(0));
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .theme(crate::Theme::chi());

    plot.preflight().unwrap();

    assert_eq!(PLOT_PLAN_CALLS.with(std::cell::Cell::get), 1);
    assert_eq!(FINAL_PLOT_RENDER_CALLS.with(std::cell::Cell::get), 0);
}

#[test]
fn canvas_gutter_remeasurement_uses_the_assigned_projection() {
    crate::canvas::CANVAS_ASSIGNED_PANEL_EVIDENCE.with(|evidence| evidence.borrow_mut().clear());
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    );
    let canvas = crate::Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, plot.clone())
        .unwrap()
        .panel(0, 1, plot)
        .unwrap();
    let mut backend = RecordingBackend::default();
    crate::orchestrator::measurements::start();
    canvas.render_with_backend(&mut backend).unwrap();
    let work = crate::orchestrator::measurements::finish();

    crate::canvas::CANVAS_ASSIGNED_PANEL_EVIDENCE.with(|evidence| {
        let evidence = evidence.borrow();
        assert!(evidence.len() >= 4 && evidence.len().is_multiple_of(2));
        let root = backend.surfaces[0].bounds;
        for (assigned, projection) in evidence.iter() {
            assert_eq!(projection, assigned);
            assert!(assigned.width() > 0.0 && assigned.height() > 0.0);
            assert!(assigned.x0 >= root.x0 && assigned.x1 <= root.x1);
            assert!(assigned.y0 >= root.y0 && assigned.y1 <= root.y1);
        }
        for pair in evidence.chunks_exact(2) {
            assert!(pair[0].0.x1 < pair[1].0.x0);
        }
        for (_, projection) in &evidence[evidence.len() - 2..] {
            assert!(backend.clips.contains(projection));
        }
    });
    assert!((2..=8).contains(&work.roots.len()));
    assert!(!work.roots[0].successful && work.roots.last().unwrap().successful);
    assert!(work.gutters.len() >= 2);
    assert!(work.gutters[..2].iter().all(|gutter| {
        gutter.consumed.iter().all(|value| value.to_bits() == 0)
            && gutter.requested[0] > 0.0
            && gutter.requested[2] > 0.0
    }));
    assert_eq!(
        work.capture_counters,
        [0; crate::orchestrator::measurements::Counter::ALL.len()]
    );
    assert_eq!((backend.begins, backend.ends), (1, 1));
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
fn invalid_canvas_alignment_fails_before_backend_begin() {
    let plot = Plot::line(
        crate::NumericValues::new("x", [1.0, 2.0]),
        crate::NumericValues::new("y", [1.0, 2.0, 3.0]),
    );
    let canvas = crate::Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, plot)
        .unwrap();
    let mut backend = RecordingBackend::default();

    let error = canvas.render_with_backend(&mut backend).unwrap_err();

    assert!(error
        .to_string()
        .contains("plot value \"y\" has length 3, expected 2"));
    assert_eq!((backend.begins, backend.ends), (0, 0));
}

#[test]
fn zero_sizing_requests_refuse_before_font_discovery_or_backend_begin() {
    let source = Plot::scatter(
        NumericValues::new("x", [0.0, 1.0]),
        NumericValues::new("y", [1.0, 0.0]),
    );
    for plot in [
        source.clone().width(0),
        source.clone().size(0, 240),
        source.clone().size(320, 0),
    ] {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let mut backend = RecordingBackend::default();
        let error = plot.render_with_backend(&mut backend).unwrap_err();
        assert!(error
            .to_string()
            .contains("dimensions must be finite and positive"));
        assert_eq!((backend.begins, backend.ends), (0, 0));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
    for (width, height) in [(0, None), (0, Some(240)), (320, Some(0))] {
        let canvas = crate::Canvas::grid(1, 1)
            .unwrap()
            .panel(0, 0, source.clone())
            .unwrap();
        let canvas = match height {
            Some(height) => canvas.size(width, height),
            None => canvas.width(width),
        };
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let mut backend = RecordingBackend::default();
        let error = canvas.render_with_backend(&mut backend).unwrap_err();
        assert!(error
            .to_string()
            .contains("dimensions must be finite and positive"));
        assert_eq!((backend.begins, backend.ends), (0, 0));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}

#[test]
fn orchestration_begins_and_ends_once_and_projects_into_offset_panel() {
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .size(640, 480);
    let expected_panel = crate::layout::fit::measured(
        crate::layout::kernel::FrameRequirements {
            minimum_body: (1.0, 1.0),
            natural_insets: [0.0; 4],
        }
        .assign_fit(
            plot.canvas_layout().panel_box,
            [0.0; 4],
            &crate::layout::fit::FitScope::root(),
        ),
    )
    .unwrap();
    let mut backend = RecordingBackend::default();

    plot.render_kind(PlotKind::Scatter, &mut backend).unwrap();

    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert_eq!(
        (backend.clips.as_slice(), backend.clip_ends),
        (&[expected_panel][..], 1)
    );
    assert!(backend.scatter_points.iter().all(|point| {
        point.x >= expected_panel.x0
            && point.x <= expected_panel.x1
            && point.y >= expected_panel.y0
            && point.y <= expected_panel.y1
    }));
}

#[test]
fn orchestration_runs_measurement_planning_and_render_passes() {
    let x = NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]);
    let y = NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]);
    let plot = Plot::scatter(x.clone(), y.clone());
    RENDER_PANEL_CALLS.with(|calls| calls.set(0));
    plot.render_kind(PlotKind::Scatter, &mut RecordingBackend::default())
        .unwrap();
    let plot_calls = RENDER_PANEL_CALLS.with(std::cell::Cell::get);

    let faceted = Plot::scatter(x.clone(), y.clone()).facet(crate::FacetSpec::column(
        CategoryValues::new("facet", ["left", "left", "right", "right"]).unwrap(),
    ));
    RENDER_PANEL_CALLS.with(|calls| calls.set(0));
    faceted
        .render_kind(PlotKind::Scatter, &mut RecordingBackend::default())
        .unwrap();
    let facet_calls = RENDER_PANEL_CALLS.with(std::cell::Cell::get);

    let scatter = || Plot::scatter(x.clone(), y.clone());
    let panel = scatter();
    let canvas = crate::Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, panel)
        .unwrap();
    RENDER_PANEL_CALLS.with(|calls| calls.set(0));
    canvas
        .render_with_backend(&mut RecordingBackend::default())
        .unwrap();
    let canvas_calls = RENDER_PANEL_CALLS.with(std::cell::Cell::get);

    let preflight_panel = scatter();
    let preflight_canvas = crate::Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, preflight_panel)
        .unwrap();
    RENDER_PANEL_CALLS.with(|calls| calls.set(0));
    preflight_canvas.preflight().unwrap();
    let preflight_calls = RENDER_PANEL_CALLS.with(std::cell::Cell::get);

    let directory = test_directory("canvas-single-preflight");
    let output = directory.join("canvas.svg");
    let rendered_panel = scatter();
    let rendered_canvas = crate::Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, rendered_panel)
        .unwrap();
    RENDER_PANEL_CALLS.with(|calls| calls.set(0));
    rendered_canvas.svg_preserve_text(&output).unwrap();
    let rendered_canvas_calls = RENDER_PANEL_CALLS.with(std::cell::Cell::get);
    std::fs::remove_dir_all(directory).unwrap();

    // Native standalone/Canvas capture replays retained selected interiors and
    // never enters the old combined panel measurement/draw adapter. Generic
    // facets still make semantic probes, whose count is not a layout contract.
    assert_eq!(
        (
            plot_calls,
            canvas_calls,
            preflight_calls,
            rendered_canvas_calls
        ),
        (0, 0, 0, 0)
    );
    assert!(facet_calls > 0);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "explicit native constructors keep lifecycle coverage free of a generic fixture"
)]
fn every_canonical_plot_kind_renders_through_one_lifecycle() {
    let group = CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap();
    let interval_group = CategoryValues::new("interval_group", ["a", "b", "c", "d"]).unwrap();
    let hue = CategoryValues::new("hue", ["h1", "h2", "h1", "h2"]).unwrap();
    let series = CategoryValues::new("series", ["h1", "h1", "h2", "h2"]).unwrap();
    let x = NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]);
    let y = NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]);
    let color = NumericValues::new("color", [0.1, 0.3, 0.7, 1.0]);
    let lower = NumericValues::new("lower", [0.5, 1.0, 2.0, 3.0]);
    let upper = NumericValues::new("upper", [1.5, 3.0, 4.0, 5.0]);
    let delta = NumericValues::new("delta", [1.0, -1.0, 2.0, 0.0]);
    let bounds = || crate::Bounds::new(lower.clone(), upper.clone());
    let plots = vec![
        (
            PlotKind::Scatter,
            Plot::scatter(x.clone(), y.clone()).hue(hue.clone()).erase(),
        ),
        (
            PlotKind::Histogram,
            Plot::histogram(crate::HistogramInput::samples(x.clone()))
                .stat(crate::HistogramStat::Density)
                .composite(crate::HistogramComposite::KdeAndRug(
                    crate::HistogramKde::Internal,
                ))
                .erase(),
        ),
        (
            PlotKind::Density,
            Plot::density(x.clone()).hue(hue.clone()).erase(),
        ),
        (PlotKind::Qq, Plot::qq(x.clone()).erase()),
        (
            PlotKind::PointInterval,
            Plot::point_interval(interval_group.clone(), x.clone(), bounds()).erase(),
        ),
        (
            PlotKind::Line,
            Plot::line(x.clone(), y.clone())
                .hue(hue.clone())
                .band(bounds())
                .erase(),
        ),
        (
            PlotKind::Trajectory,
            Plot::trajectory(AxisValues::Numeric(x.clone()), y.clone(), group.clone())
                .hue(hue.clone())
                .erase(),
        ),
        (
            PlotKind::LinePath,
            Plot::line_path(x.clone(), y.clone(), group.clone())
                .hue(hue.clone())
                .erase(),
        ),
        (
            PlotKind::Interaction,
            Plot::interaction(group.clone(), y.clone(), hue.clone(), bounds())
                .hue(hue.clone())
                .erase(),
        ),
        (
            PlotKind::Coefficient,
            Plot::coefficient(
                interval_group.clone(),
                crate::CoefficientInput::estimates_with_intervals(x.clone(), bounds()),
            )
            .erase(),
        ),
        (
            PlotKind::Forest,
            Plot::forest(interval_group.clone(), x.clone(), bounds(), 0.0).erase(),
        ),
        (
            PlotKind::Dumbbell,
            Plot::dumbbell(
                group.clone(),
                x.clone(),
                y.clone(),
                crate::DumbbellEndpoints::new(
                    "Left",
                    crate::Color::rgb(20, 80, 140),
                    "Right",
                    crate::Color::rgb(180, 60, 40),
                )
                .unwrap(),
            )
            .group(interval_group.clone())
            .erase(),
        ),
        (
            PlotKind::Heatmap,
            Plot::heatmap(interval_group.clone(), group.clone(), color.clone()).erase(),
        ),
        (
            PlotKind::Bar,
            Plot::bar(
                group.clone(),
                crate::BarInput::samples(y.clone())
                    .error_bar(crate::ErrorBar::standard_deviation()),
            )
            .series(series.clone())
            .erase(),
        ),
        (
            PlotKind::HorizontalBar,
            Plot::horizontal_bar(
                group.clone(),
                crate::BarInput::samples(y.clone())
                    .error_bar(crate::ErrorBar::standard_deviation()),
            )
            .series(series.clone())
            .erase(),
        ),
        (
            PlotKind::StackedBar,
            Plot::stacked_bar(group.clone(), y.clone(), hue.clone()).erase(),
        ),
        (
            PlotKind::StackedHorizontalBar,
            Plot::stacked_horizontal_bar(group.clone(), y.clone(), hue.clone()).erase(),
        ),
        (
            PlotKind::Boxplot,
            Plot::boxplot(group.clone(), y.clone())
                .raw_overlay(crate::RawOverlay::Points)
                .series(series.clone())
                .erase(),
        ),
        (
            PlotKind::Violin,
            Plot::violin(group.clone(), y.clone())
                .raw_overlay(crate::RawOverlay::Points)
                .series(series.clone())
                .erase(),
        ),
        (
            PlotKind::Raincloud,
            Plot::raincloud(crate::RaincloudInput::samples(group.clone(), y.clone()))
                .series(series)
                .erase(),
        ),
        (
            PlotKind::Regression,
            Plot::regression(x.clone(), y.clone())
                .band(crate::RegressionBand::PointwiseMean95)
                .legend(crate::LegendPolicy::Visible(
                    crate::LegendPosition::OutsideRight,
                ))
                .erase(),
        ),
        (
            PlotKind::Marginal,
            Plot::marginal(x.clone(), y.clone())
                .hue(hue.clone())
                .erase(),
        ),
        (
            PlotKind::Matrix,
            Plot::matrix(vec![x.clone(), y.clone()]).hue(hue).erase(),
        ),
        (
            PlotKind::Beeswarm,
            Plot::beeswarm(group.clone(), x.clone(), color).erase(),
        ),
        (
            PlotKind::Waterfall,
            Plot::waterfall(group, delta, 10.0).erase(),
        ),
    ];
    let mut preparation_work = [0_u64; 3];
    let mut alpha_components = [0_usize; 3];
    crate::canvas::lifecycle_tests::mixed_family_parent_audit(&plots);
    for (kind, plot) in plots {
        let mut plot = plot.size(1200, 900);
        assert_prepared_kind_sections(kind, &plot);
        for &target in alpha_targets(kind) {
            let name = match target {
                crate::alpha::AlphaTarget::Point => "point confidence",
                crate::alpha::AlphaTarget::Line => "stroke confidence",
                crate::alpha::AlphaTarget::Fill => "fill confidence",
            };
            let values = crate::alpha::AlphaValues::linear(
                NumericValues::new(name, [0.5]),
                (0.0, 1.0),
                (0, 255),
            )
            .unwrap();
            plot = plot.with_alpha(target, crate::alpha::AlphaEncoding::single(values).unwrap());
            alpha_components[target.index()] += 1;
        }
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let domain =
            crate::plots::shared_x::domain(&plot, &crate::font::canonical::OperationContext::new());
        let resources = crate::font::canonical::OperationContext::new();
        let prepared = crate::plots::shared_x::prepare_plot(
            &plot,
            &resources,
            Some(&crate::theme::Theme::publication()),
        )
        .unwrap_or_else(|error| panic!("{kind:?}: {error}"));
        prepared
            .prepare_discrete_guides(
                &plot,
                &crate::Theme::publication().resolve().unwrap(),
                &resources,
                &crate::layout::ComponentAddress::root(),
                false,
            )
            .unwrap();
        assert_actual_alpha_sections(
            kind,
            prepared
                .discrete_guides
                .get()
                .unwrap()
                .root
                .as_ref()
                .unwrap(),
        );
        let prepared = prepared.attach(plot.clone()).unwrap();
        if matches!(
            kind,
            PlotKind::Interaction
                | PlotKind::Heatmap
                | PlotKind::Bar
                | PlotKind::StackedBar
                | PlotKind::Boxplot
                | PlotKind::Violin
                | PlotKind::Raincloud
                | PlotKind::Matrix
        ) {
            assert!(domain.is_err(), "{kind:?}");
        } else {
            let range = domain.unwrap_or_else(|error| panic!("{kind:?}: {error}"));
            assert_eq!(
                range,
                prepared.prepared_axis_inputs().unwrap().ranges[0],
                "{kind:?}: shared/standalone parity"
            );
            assert!(
                range.0.is_finite() && range.1.is_finite() && range.0 < range.1,
                "{kind:?}"
            );
        }
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0,
            "{kind:?}"
        );
        if kind != PlotKind::Matrix {
            let axes = prepared.prepared_axis_inputs().unwrap();
            for side in 0..2 {
                if axes.categories[side].is_some() {
                    continue;
                }
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
                let invalid = plot.clone().with_test_axis_ticks(
                    side,
                    vec![crate::AxisTick::new(
                        axes.ranges[side].1.next_up(),
                        "outside",
                    )],
                );
                let error = invalid
                    .render_kind_erased(kind, &mut RecordingBackend::default())
                    .unwrap_err();
                assert!(
                    error.to_string().contains("outside"),
                    "{kind:?} {side}: {error}"
                );
                assert_eq!(
                    crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                    0,
                    "{kind:?} {side}: {error}"
                );
            }
            let theme = crate::Theme::publication().resolve().unwrap();
            let preparations = crate::plots::shared_x::AXIS_PREPARATIONS.with(std::cell::Cell::get);
            let panel = prepared.measured_canvas_layout(&theme).unwrap().panel_box;
            prepared
                .preflight_panel(kind, panel, &theme)
                .unwrap_or_else(|error| panic!("{kind:?}: {error}"));
            assert_eq!(
                crate::plots::shared_x::AXIS_PREPARATIONS.with(std::cell::Cell::get),
                preparations,
                "{kind:?}: measurement must consume retained preparation"
            );
        }
        let mut backend = RecordingBackend::default();
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
        crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS.with(|calls| calls.set(0));
        crate::orchestrator::TEXT_PLAN_CAPTURE_SESSIONS.with(|calls| calls.set(0));
        crate::orchestrator::measurements::start();
        let plan = crate::orchestrator::lifecycle::PlotPlan::new(&plot, kind)
            .unwrap_or_else(|error| panic!("{kind:?} failed: {error}"));
        plan.render(&mut backend).unwrap();
        let work = crate::orchestrator::measurements::finish();
        assert_eq!(work.calls[2], 1, "{kind:?}: selected capture");
        assert_eq!(work.calls[3], 1, "{kind:?}: final Null validation");
        assert_eq!(
            work.capture_counters,
            [0; crate::orchestrator::measurements::Counter::ALL.len()],
            "{kind:?}: no foundational or deferred work in capture"
        );
        crate::orchestrator::TEXT_PLAN_CAPTURE_SESSIONS
            .with(|calls| assert_eq!(calls.get(), 1, "{kind:?}"));
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS
            .with(|calls| assert_eq!(calls.get(), 0, "{kind:?}"));
        crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS
            .with(|calls| assert_eq!(calls.get(), 0, "{kind:?}"));
        for (total, counter) in preparation_work.iter_mut().zip([
            crate::orchestrator::measurements::Counter::AxisRangePreparation,
            crate::orchestrator::measurements::Counter::AxisSpecPreparation,
            crate::orchestrator::measurements::Counter::AxisSourcePreparation,
        ]) {
            *total += work.counters[counter as usize];
        }
        assert_eq!((backend.begins, backend.ends), (1, 1), "{kind:?}");
        assert_eq!(plan.dimensions(), (1200, 900));
        // Dropping the source before every concrete output makes a source-size
        // or source-data reread impossible after exact-root promotion.
        drop(plot);
        let frozen_work = || {
            [
                crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                crate::layout::Projection::test_projection_calls(),
                crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
                crate::plots::shared_x::AXIS_PREPARATIONS.with(std::cell::Cell::get),
                crate::orchestrator::SELECTED_INTERIOR_MEASUREMENTS.with(std::cell::Cell::get),
            ]
        };
        let before = frozen_work();
        crate::orchestrator::measurements::start();
        let mut svg = plan.take_svg_backend().unwrap();
        plan.render(&mut svg).unwrap();
        let scene = crate::scene::RenderedScene::new(1200, 900, svg.into_output()).unwrap();
        let preserve = scene.svg_bytes().to_vec();
        assert!(std::str::from_utf8(&preserve)
            .unwrap()
            .contains("viewBox=\"0 0 1200 900\""));
        if matches!(kind, PlotKind::Heatmap | PlotKind::Beeswarm) {
            assert_eq!(
                std::str::from_utf8(&preserve)
                    .unwrap()
                    .matches("<linearGradient")
                    .count(),
                1,
                "{kind:?}: one native guide placement owner"
            );
        }
        let outlined = scene.svg_with_text_as_paths().unwrap();
        assert!(!std::str::from_utf8(&outlined).unwrap().contains("<text"));
        let png = scene
            .png_bytes(&crate::PngOptions {
                sizing: crate::PngSizing::Scale(1.0),
                ..crate::PngOptions::default()
            })
            .unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 1200);
        assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 900);
        assert_eq!(scene.svg_bytes(), preserve);
        assert_eq!(scene.svg_with_text_as_paths().unwrap(), outlined);
        let work = crate::orchestrator::measurements::finish();
        assert_eq!(
            work.counters,
            [0; crate::orchestrator::measurements::Counter::ALL.len()],
            "{kind:?}: frozen output work"
        );
        assert_eq!(frozen_work(), before, "{kind:?}: post-freeze work");
    }
    assert!(preparation_work.into_iter().all(|count| count > 0));
    assert_eq!(alpha_components, [17, 17, 15]);
}

fn alpha_targets(kind: PlotKind) -> &'static [crate::alpha::AlphaTarget] {
    use crate::alpha::AlphaTarget::{Fill, Line, Point};
    match kind {
        PlotKind::Scatter | PlotKind::Beeswarm => &[Point],
        PlotKind::Histogram | PlotKind::Density | PlotKind::Bar | PlotKind::HorizontalBar => {
            &[Line, Fill]
        }
        PlotKind::Qq
        | PlotKind::PointInterval
        | PlotKind::Coefficient
        | PlotKind::Forest
        | PlotKind::Trajectory
        | PlotKind::LinePath
        | PlotKind::Interaction
        | PlotKind::Dumbbell => &[Point, Line],
        PlotKind::Line
        | PlotKind::Boxplot
        | PlotKind::Violin
        | PlotKind::Raincloud
        | PlotKind::Regression => &[Point, Line, Fill],
        PlotKind::Heatmap
        | PlotKind::StackedBar
        | PlotKind::StackedHorizontalBar
        | PlotKind::Waterfall => &[Fill],
        PlotKind::Marginal | PlotKind::Matrix => &[Point, Fill],
    }
}

fn assert_actual_alpha_sections(
    kind: PlotKind,
    descriptor: &crate::decorations::legend::discrete::PreparedGuide,
) {
    use crate::alpha::{AlphaTarget, AlphaUnitKind};
    use crate::decorations::legend::discrete::{GuideSectionBody, GuideSectionIdentity};
    let sections = descriptor
        .sections
        .iter()
        .filter(|section| matches!(section.body, GuideSectionBody::Alpha(_)))
        .collect::<Vec<_>>();
    assert_eq!(sections.len(), alpha_targets(kind).len(), "{kind:?}");
    for (section, &target) in sections.into_iter().zip(alpha_targets(kind)) {
        let GuideSectionIdentity::Alpha(identity) = &section.identity else {
            panic!("typed alpha identity")
        };
        let unit = match (target, kind) {
            (AlphaTarget::Point, _)
            | (AlphaTarget::Fill, PlotKind::Heatmap | PlotKind::Waterfall)
            | (
                AlphaTarget::Line,
                PlotKind::PointInterval
                | PlotKind::Coefficient
                | PlotKind::Forest
                | PlotKind::Dumbbell,
            ) => AlphaUnitKind::Observation,
            (AlphaTarget::Fill, PlotKind::Histogram) => AlphaUnitKind::Bin,
            (AlphaTarget::Line, PlotKind::Histogram | PlotKind::Qq | PlotKind::Regression)
            | (AlphaTarget::Fill, PlotKind::Regression) => AlphaUnitKind::Single,
            (
                _,
                PlotKind::Bar
                | PlotKind::HorizontalBar
                | PlotKind::StackedBar
                | PlotKind::StackedHorizontalBar
                | PlotKind::Boxplot
                | PlotKind::Violin
                | PlotKind::Raincloud,
            ) => AlphaUnitKind::Aggregate,
            _ => AlphaUnitKind::Series,
        };
        assert_eq!(
            (identity.target, identity.unit, identity.semantics),
            (target, unit, crate::alpha::AlphaSemantics::Continuous),
            "{kind:?}"
        );
        assert_eq!(
            section.entries().len(),
            5,
            "{kind:?}: numeric examples are not reduced to the actual broadcast byte"
        );
        let GuideSectionBody::Alpha(alpha) = &section.body else {
            unreachable!()
        };
        assert_eq!(alpha.sources.len(), 1);
        assert_eq!(alpha.sources[0].producer, kind);
        assert!(alpha.sources[0]
            .glyphs
            .iter()
            .all(|glyph| !glyph.members.is_empty()
                && glyph
                    .members
                    .iter()
                    .all(|member| member.value.alpha == 128 && member.value.index == 0)));
    }
}

/// Independently specified symbol components and native color bodies for the
/// actual source fixture above, including authoritative absence where appropriate.
fn assert_prepared_kind_sections(kind: PlotKind, plot: &Plot) {
    use crate::decorations::legend::discrete::GuideCounts;
    use crate::LegendComposition::{Combined, Separate};
    let components = match kind {
        PlotKind::Scatter | PlotKind::Trajectory | PlotKind::LinePath | PlotKind::Dumbbell => {
            Some((2, 0, 0))
        }
        PlotKind::Line => Some((2, 2, 2)),
        PlotKind::Interaction => Some((2, 2, 0)),
        PlotKind::Density | PlotKind::Boxplot => Some((0, 2, 2)),
        PlotKind::Bar
        | PlotKind::HorizontalBar
        | PlotKind::StackedBar
        | PlotKind::StackedHorizontalBar
        | PlotKind::Violin => Some((0, 0, 2)),
        PlotKind::Raincloud | PlotKind::Marginal | PlotKind::Matrix => Some((2, 0, 2)),
        PlotKind::Regression => Some((1, 1, 1)),
        PlotKind::Histogram
        | PlotKind::Qq
        | PlotKind::PointInterval
        | PlotKind::Coefficient
        | PlotKind::Forest
        | PlotKind::Heatmap
        | PlotKind::Beeswarm
        | PlotKind::Waterfall => None,
    };
    let theme = crate::Theme::publication().resolve().unwrap();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    for composition in [Combined, Separate] {
        let plot = plot.clone().with_parent_legend_composition(composition);
        let operation = crate::font::canonical::OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&plot, &operation, Some(&theme)).unwrap();
        inputs
            .prepare_discrete_guides(
                &plot,
                &theme,
                &operation,
                &crate::layout::ComponentAddress::root(),
                false,
            )
            .unwrap_or_else(|error| panic!("{kind:?}, {composition:?}: {error}"));
        let descriptor = inputs.discrete_guides.get().unwrap().root.as_ref();
        match (components, descriptor) {
            (Some((points, lines, fills)), Some(descriptor)) => {
                assert_eq!(descriptor.composition, composition);
                assert_eq!(
                    descriptor.counts,
                    GuideCounts {
                        sections: 1,
                        entries: 2,
                        points,
                        lines,
                        fills,
                        primary_source_keys: 2,
                        observed_pair_source_keys: 0,
                        labeled_layer_source_keys: 0,
                        source_occurrences: 2,
                    },
                    "{kind:?}, {composition:?}"
                );
            }
            (None, None) => {}
            (None, Some(descriptor)) if matches!(kind, PlotKind::Heatmap | PlotKind::Beeswarm) => {
                assert_eq!(descriptor.composition, composition);
                assert_eq!(
                    descriptor.counts,
                    GuideCounts {
                        sections: 1,
                        ..GuideCounts::default()
                    }
                );
                assert_eq!(descriptor.sections.len(), 1);
                let color = descriptor.continuous().expect("actual native color body");
                let source = inputs
                    .continuous_source(&plot, &theme, plot.axis_scope())
                    .unwrap()
                    .unwrap();
                assert_eq!(color, &source);
                assert!(!color.numeric.labels.is_empty());
                assert!(!color.stops.is_empty());
                assert!(!descriptor.has_symbols());
            }
            _ => panic!("{kind:?}, {composition:?}: incorrect discrete presence"),
        }
    }
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| assert_eq!(calls.get(), 0));
}

#[test]
fn heatmap_validation_failure_happens_before_canvas_begin() {
    let plot = Plot::heatmap(
        CategoryValues::new("interval_group", ["a", "b", "c", "d"]).unwrap(),
        CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
        NumericValues::new("color", [0.1, 0.3, 0.7, 1.0]),
    )
    .x_category_order(ValueSort::Ascending);
    let mut backend = RecordingBackend::default();

    let error = plot
        .render_kind(PlotKind::Heatmap, &mut backend)
        .unwrap_err()
        .to_string();

    assert!(error.contains("cannot use ValueSort::Ascending"));
    assert_eq!((backend.begins, backend.ends), (0, 0));
}

#[test]
fn orchestration_does_not_end_a_canvas_that_failed_to_begin() {
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    );
    let mut backend = RecordingBackend {
        failures: vec![BackendFailure::Begin],
        ..RecordingBackend::default()
    };

    assert!(plot.render_kind(PlotKind::Scatter, &mut backend).is_err());
    assert_eq!((backend.begins, backend.ends), (1, 0));
}

#[test]
fn orchestration_rejects_every_invalid_axis_endpoint_before_begin() {
    let x = NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]);
    let y = NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]);
    let invalid_ranges = [
        (f64::NAN, 1.0),
        (0.0, f64::NAN),
        (f64::NEG_INFINITY, 1.0),
        (0.0, f64::INFINITY),
        (f64::INFINITY, 1.0),
        (0.0, f64::NEG_INFINITY),
        (1.0, 1.0),
        (2.0, 1.0),
    ];
    for limits in invalid_ranges {
        for x_axis in [true, false] {
            let mut plot = Plot::scatter(x.clone(), y.clone());
            plot = if x_axis {
                plot.x_limits(limits.0, limits.1)
            } else {
                plot.y_limits(limits.0, limits.1)
            };
            let mut backend = RecordingBackend::default();

            assert!(plot.render_with_backend(&mut backend).is_err());
            assert_eq!(backend.begins, 0, "limits={limits:?}, x_axis={x_axis}");
        }
    }
}

#[test]
fn orchestration_renders_valid_adjacent_subnormal_limits() {
    let upper = f64::from_bits(1);
    let plot = Plot::scatter(
        NumericValues::new("x", [0.0, upper]),
        NumericValues::new("y", [0.0, upper]),
    )
    .x_limits(0.0, upper)
    .y_limits(0.0, upper);
    let mut backend = RecordingBackend::default();

    plot.render_with_backend(&mut backend).unwrap();

    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert!(backend
        .scatter_points
        .iter()
        .all(|point| point.x.is_finite() && point.y.is_finite()));
}

#[test]
fn significance_annotations_are_drawn_after_marks_clip_closes() {
    let plot = Plot::boxplot(
        CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .significance([Significance::between(
        "a",
        "b",
        SignificanceMark::PValue(0.01),
    )])
    .size(640, 480);
    let mut backend = RecordingBackend::default();

    plot.render_kind(PlotKind::Boxplot, &mut backend).unwrap();

    assert_eq!(backend.bracket_clip_depths, [0]);
}

#[test]
fn attached_significance_plan_is_not_reshaped_during_final_render() {
    let plot = Plot::bar(
        CategoryValues::new("group", ["a", "b"]).unwrap(),
        crate::BarInput::estimates(NumericValues::new("value", [1.0, 2.0])),
    )
    .significance([Significance::at("a", SignificanceMark::OneStar)])
    .erase();
    crate::plots::bar_data::ATTACHED_SIGNIFICANCE_TEXT_PLANNING_CALLS.with(|calls| calls.set(0));
    crate::plots::bar_data::STAGES.with(|counts| counts.set([[0; 4]; 4]));
    let plan = crate::orchestrator::lifecycle::PlotPlan::new(&plot, PlotKind::Bar).unwrap();
    let planned_calls = crate::plots::bar_data::ATTACHED_SIGNIFICANCE_TEXT_PLANNING_CALLS
        .with(std::cell::Cell::get);
    assert!(planned_calls > 0);
    assert_eq!(
        planned_calls,
        crate::plots::bar_data::STAGES.with(std::cell::Cell::get)[0][1]
    );

    plan.render(&mut RecordingBackend::default()).unwrap();

    crate::plots::bar_data::ATTACHED_SIGNIFICANCE_TEXT_PLANNING_CALLS.with(|calls| {
        assert_eq!(calls.get(), planned_calls);
    });
}

#[test]
fn unclamped_projection_leaves_out_of_range_marks_for_backend_clipping() {
    let plot = Plot::scatter(
        NumericValues::new("x", [-1.0, 0.5, 2.0]),
        NumericValues::new("y", [0.5, 0.5, 0.5]),
    )
    .x_limits(0.0, 1.0)
    .y_limits(0.0, 1.0);
    let mut backend = RecordingBackend::default();

    plot.render_kind(PlotKind::Scatter, &mut backend).unwrap();

    let panel = backend.clips[0];
    assert!(backend.scatter_points[0].x < panel.x0);
    assert!(backend.scatter_points[1].x > panel.x0 && backend.scatter_points[1].x < panel.x1);
    assert!(backend.scatter_points[2].x > panel.x1);

    let dir = test_directory("out-of-range-svg");
    let output = dir.join("canvas.svg");
    plot.clone().svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(&output).unwrap();
    assert!(svg.contains(&format!("cx=\"{}\"", backend.scatter_points[0].x)));
    assert!(svg.contains(&format!("cx=\"{}\"", backend.scatter_points[2].x)));
    assert!(svg.contains("<g clip-path=\"url(#clip-0)\">"));
    std::fs::remove_dir_all(dir).unwrap();
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

#[test]
fn decoration_layer_closes_after_inner_draw_error() {
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    );
    let mut backend = RecordingBackend {
        failures: vec![BackendFailure::Line, BackendFailure::LayerEnd],
        ..RecordingBackend::default()
    };

    let error = plot
        .render_kind(PlotKind::Scatter, &mut backend)
        .unwrap_err()
        .to_string();

    assert!(error.contains("line draw failed"));
    assert!(error.contains("layer cleanup failed"));
    assert_eq!(backend.layer_ends, 1, "{error}");
    assert_eq!(backend.ends, 1, "{error}");
}

#[test]
fn matrix_legend_layer_closes_after_inner_draw_error() {
    let hue = CategoryValues::new("hue", ["h1", "h2", "h1", "h2"]).unwrap();
    let plot = Plot::matrix(vec![
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    ])
    .group(hue.clone())
    .hue(hue)
    .size(1200, 900);
    let mut backend = RecordingBackend {
        failures: vec![BackendFailure::PolygonInLayer, BackendFailure::LayerEnd],
        ..RecordingBackend::default()
    };

    let error = plot
        .render_kind(PlotKind::Matrix, &mut backend)
        .unwrap_err()
        .to_string();

    assert!(error.contains("layer polygon draw failed"), "{error}");
    assert!(error.contains("layer cleanup failed"));
    assert_eq!(backend.layer_ends, 1);
    assert_eq!(backend.ends, 1);
}
