use std::sync::atomic::Ordering;

use super::{
    test_directory, BackendFailure, Plot, PlotKind, RecordingBackend, Significance,
    SignificanceMark, ValueSort, FINAL_PLOT_RENDER_CALLS, PLOT_PLAN_CALLS, RENDER_PANEL_CALLS,
    RENDER_THEME_ADDRESSES, TEMP_FILE_COUNTER,
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

fn assert_scene_routes(expected_routes: &[&str]) {
    crate::scene::SCENE_ROUTE_EVIDENCE.with(|evidence| {
        let evidence = evidence.borrow();
        assert_eq!(
            evidence
                .iter()
                .map(|(route, _, _)| *route)
                .collect::<Vec<_>>(),
            expected_routes
        );
        let preserve = evidence
            .iter()
            .filter(|(route, _, _)| *route == "preserve")
            .map(|(_, address, length)| (*address, *length))
            .collect::<Vec<_>>();
        let outlined = evidence
            .iter()
            .filter(|(route, _, _)| *route != "preserve")
            .map(|(_, address, length)| (*address, *length))
            .collect::<Vec<_>>();
        assert!(preserve.windows(2).all(|pair| pair[0] == pair[1]));
        assert!(outlined.windows(2).all(|pair| pair[0] == pair[1]));
        if !preserve.is_empty() && !outlined.is_empty() {
            assert_ne!(preserve[0], outlined[0]);
        }
    });
}

fn reset_theme_evidence() {
    crate::theme::THEME_RESOLUTION_CALLS.with(|calls| calls.set(0));
    RENDER_THEME_ADDRESSES.with(|addresses| addresses.borrow_mut().clear());
    crate::canvas::CANVAS_THEME_EVIDENCE.with(|evidence| evidence.borrow_mut().clear());
    crate::canvas::CANVAS_SHARED_LEGEND_THEME_EVIDENCE
        .with(|evidence| evidence.borrow_mut().clear());
}

fn resolved_theme_count() -> usize {
    crate::theme::THEME_RESOLUTION_CALLS.with(std::cell::Cell::get)
}

fn rendered_theme_identity_count() -> usize {
    RENDER_THEME_ADDRESSES.with(|addresses| {
        let mut addresses = addresses.borrow().clone();
        addresses.sort_unstable();
        addresses.dedup();
        addresses.len()
    })
}

fn assert_canvas_theme_identity(owner: usize, stages: &[&str]) {
    let rounds = crate::layout::kernel::ROOT_FEEDBACK_ROUNDS.with(std::cell::Cell::get);
    assert!((1..=8).contains(&rounds));
    let first_render = stages
        .iter()
        .position(|stage| *stage == "render" || stage.ends_with("-render"))
        .unwrap_or(stages.len());
    let mut expected = vec![stages[0]];
    expected.extend(
        stages[1..first_render]
            .iter()
            .copied()
            .filter(|stage| *stage == "prepared-shared-hue"),
    );
    for _ in 0..rounds {
        expected.extend(
            stages[1..first_render]
                .iter()
                .copied()
                .filter(|stage| *stage != "prepared-shared-hue"),
        );
    }
    expected.extend_from_slice(&stages[first_render..]);
    crate::canvas::CANVAS_THEME_EVIDENCE.with(|evidence| {
        let evidence = evidence.borrow();
        let records = evidence
            .iter()
            .filter(|(candidate, _, _)| *candidate == owner)
            .collect::<Vec<_>>();
        assert!(!records.is_empty());
        assert!(records
            .iter()
            .all(|(_, _, address)| *address == records[0].2));
        assert_eq!(
            records
                .iter()
                .map(|(_, stage, _)| *stage)
                .collect::<Vec<_>>(),
            expected
        );
    });
}

fn assert_canvas_theme_identities_differ(left_owner: usize, right_owner: usize) {
    crate::canvas::CANVAS_THEME_EVIDENCE.with(|evidence| {
        let evidence = evidence.borrow();
        let left = evidence
            .iter()
            .find(|(owner, _, _)| *owner == left_owner)
            .expect("left Canvas theme owner has evidence");
        let right = evidence
            .iter()
            .find(|(owner, _, _)| *owner == right_owner)
            .expect("right Canvas theme owner has evidence");
        assert_ne!(left.2, right.2);
    });
}

fn assert_canvas_theme_identities_match(left_owner: usize, right_owner: usize) {
    crate::canvas::CANVAS_THEME_EVIDENCE.with(|evidence| {
        let evidence = evidence.borrow();
        let left = evidence
            .iter()
            .find(|(owner, _, _)| *owner == left_owner)
            .expect("left Canvas theme owner has evidence");
        let right = evidence
            .iter()
            .find(|(owner, _, _)| *owner == right_owner)
            .expect("right Canvas theme owner has evidence");
        assert_eq!(left.2, right.2);
    });
}

fn assert_canvas_theme_owners(expected: &[usize]) {
    crate::canvas::CANVAS_THEME_EVIDENCE.with(|evidence| {
        let mut owners = evidence
            .borrow()
            .iter()
            .map(|(owner, _, _)| *owner)
            .collect::<Vec<_>>();
        owners.sort_unstable();
        owners.dedup();
        let mut expected = expected.to_vec();
        expected.sort_unstable();
        assert_eq!(owners, expected);
    });
}

fn assert_rendered_themes_match_canvas_owners(owners: &[usize]) {
    let mut expected = crate::canvas::CANVAS_THEME_EVIDENCE.with(|evidence| {
        let evidence = evidence.borrow();
        owners
            .iter()
            .map(|expected_owner| {
                evidence
                    .iter()
                    .find(|(owner, stage, _)| owner == expected_owner && *stage == "resolved")
                    .expect("rendered Canvas theme owner has resolution evidence")
                    .2
            })
            .collect::<Vec<_>>()
    });
    expected.sort_unstable();
    expected.dedup();
    RENDER_THEME_ADDRESSES.with(|addresses| {
        let mut addresses = addresses.borrow().clone();
        addresses.sort_unstable();
        addresses.dedup();
        assert_eq!(addresses, expected);
    });
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
                    font_family: "sans-serif".into(),
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
            (0..10_001)
                .map(|index| crate::AxisTick::new(f64::from(index), ""))
                .collect(),
            "exceeds 10000",
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
    crate::theme::THEME_RESOLUTION_CALLS.with(|calls| calls.set(0));
    PLOT_PLAN_CALLS.with(|calls| calls.set(0));
    FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(0));
    TEXT_PLAN_CAPTURE_SESSIONS.with(|calls| calls.set(0));
    TEXT_REPLAY_SESSIONS.with(|calls| calls.set(0));
    crate::font::canonical::FONT_PAIR_RESOLUTION_CALLS.with(|calls| calls.set(0));
    RENDER_THEME_ADDRESSES.with(|addresses| addresses.borrow_mut().clear());
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .theme(crate::Theme::Nature);
    let mut backend = RecordingBackend::default();

    plot.render_with_backend(&mut backend).unwrap();

    assert_eq!(
        crate::theme::THEME_RESOLUTION_CALLS.with(std::cell::Cell::get),
        1
    );
    assert_eq!(PLOT_PLAN_CALLS.with(std::cell::Cell::get), 1);
    assert_eq!(FINAL_PLOT_RENDER_CALLS.with(std::cell::Cell::get), 1);
    assert_eq!(TEXT_PLAN_CAPTURE_SESSIONS.with(std::cell::Cell::get), 1);
    assert_eq!(TEXT_REPLAY_SESSIONS.with(std::cell::Cell::get), 1);
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
    RENDER_THEME_ADDRESSES.with(|addresses| {
        let addresses = addresses.borrow();
        assert!(!addresses.is_empty());
        assert!(addresses.iter().all(|address| *address == addresses[0]));
    });
}

#[test]
fn standalone_lifecycle_preserves_distinct_transparent_root_and_panel_surfaces() {
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .erase();
    let mut theme = crate::Theme::Publication.resolve();
    theme.surfaces.canvas_root = crate::theme::Color::rgba(17, 34, 51, 128);
    theme.surfaces.plot_panel = crate::theme::Color::rgba(68, 85, 102, 64);
    let plan = crate::orchestrator::lifecycle::PlotPlan::new_with_resolved_theme(
        &plot,
        PlotKind::Scatter,
        theme,
    )
    .unwrap();
    let mut recorded = RecordingBackend::default();

    plan.render(&mut recorded).unwrap();

    assert_eq!(recorded.surfaces.len(), 2);
    assert_eq!(
        (recorded.surfaces[0].role, recorded.surfaces[0].paint),
        (
            SurfaceRole::CanvasRoot,
            crate::theme::Color::rgba(17, 34, 51, 128)
        )
    );
    assert_eq!(
        (recorded.surfaces[1].role, recorded.surfaces[1].paint),
        (
            SurfaceRole::PlotPanel,
            crate::theme::Color::rgba(68, 85, 102, 64)
        )
    );

    let (width, height) = plan.dimensions();
    let mut backend = crate::render::svg_backend::SvgBackend::in_memory(width, height);
    plan.render(&mut backend).unwrap();
    let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
    let surfaces = svg
        .lines()
        .filter(|line| line.starts_with("<rect "))
        .collect::<Vec<_>>();

    assert!(surfaces[0].contains("fill=\"#112233\" fill-opacity="));
    assert!(surfaces[1].contains("fill=\"#445566\" fill-opacity="));
    assert_eq!(surfaces[0].matches("fill-opacity=").count(), 1);
    assert_eq!(surfaces[1].matches("fill-opacity=").count(), 1);
}

#[test]
fn unthemed_standalone_selects_publication_once() {
    reset_theme_evidence();
    PLOT_PLAN_CALLS.with(|calls| calls.set(0));
    FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(0));
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    );
    let mut backend = RecordingBackend::default();

    plot.render_with_backend(&mut backend).unwrap();
    assert_eq!(
        crate::layout::kernel::ROOT_FEEDBACK_ROUNDS.with(std::cell::Cell::get),
        3
    );

    assert_eq!(resolved_theme_count(), 1);
    assert_eq!(PLOT_PLAN_CALLS.with(std::cell::Cell::get), 1);
    assert_eq!(FINAL_PLOT_RENDER_CALLS.with(std::cell::Cell::get), 1);
    assert_eq!(rendered_theme_identity_count(), 1);
    assert_eq!((backend.begins, backend.ends), (1, 1));
}

#[test]
fn public_preflight_uses_the_prepared_plan_without_a_final_lifecycle() {
    crate::theme::THEME_RESOLUTION_CALLS.with(|calls| calls.set(0));
    PLOT_PLAN_CALLS.with(|calls| calls.set(0));
    FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(0));
    RENDER_THEME_ADDRESSES.with(|addresses| addresses.borrow_mut().clear());
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .theme(crate::Theme::CHI);

    plot.preflight().unwrap();

    assert_eq!(
        crate::theme::THEME_RESOLUTION_CALLS.with(std::cell::Cell::get),
        1
    );
    assert_eq!(PLOT_PLAN_CALLS.with(std::cell::Cell::get), 1);
    assert_eq!(FINAL_PLOT_RENDER_CALLS.with(std::cell::Cell::get), 0);
    RENDER_THEME_ADDRESSES.with(|addresses| {
        let addresses = addresses.borrow();
        assert!(!addresses.is_empty());
        assert!(addresses.iter().all(|address| *address == addresses[0]));
    });
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
        assert_eq!(evidence.len(), 4);
        for (assigned, projection) in evidence.iter() {
            assert_eq!(projection, assigned);
            assert!(assigned.width() < 480.0);
        }
        assert!(evidence[0].0.x1 < evidence[1].0.x0);
        // The complete-gutter probe already allocates the final projection.
        // Publishing those measured root inputs still requires the next round.
        assert_eq!(evidence[0].0, evidence[2].0);
        for (_, projection) in &evidence[2..4] {
            assert!(backend.clips.contains(projection));
        }
    });
    assert_eq!(work.roots.len(), 2);
    assert!(!work.roots[0].successful && work.roots[1].successful);
    assert_eq!(work.gutters.len(), 2);
    assert!(work.gutters.iter().all(|gutter| {
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
fn canvas_inherited_children_share_the_root_and_explicit_publication_does_not() {
    let x = NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]);
    let y = NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]);
    let canvas = crate::Canvas::grid(1, 3)
        .unwrap()
        .panel(0, 0, Plot::scatter(x.clone(), y.clone()))
        .unwrap()
        .panel(0, 1, Plot::scatter(x.clone(), y.clone()))
        .unwrap()
        .panel(0, 2, Plot::scatter(x, y).theme(crate::Theme::Publication))
        .unwrap();
    reset_theme_evidence();
    let mut backend = RecordingBackend::default();

    canvas.render_with_backend(&mut backend).unwrap();

    assert_eq!(
        (resolved_theme_count(), rendered_theme_identity_count()),
        (2, 2)
    );
    assert_canvas_theme_owners(&[usize::MAX, 0, 1, 2]);
    assert_canvas_theme_identities_match(usize::MAX, 0);
    assert_canvas_theme_identities_match(usize::MAX, 1);
    assert_canvas_theme_identities_differ(usize::MAX, 2);
    assert_canvas_theme_identity(usize::MAX, &["resolved", "layout", "render"]);
    assert_eq!(backend.surfaces.len(), 4);
    assert_eq!(backend.surfaces[0].role, SurfaceRole::CanvasRoot);
    assert_eq!(
        backend.surfaces[0].bounds,
        crate::layout::PanelBox::new(0.0, 960.0, 0.0, 640.0)
    );
    assert!(backend.surfaces[1..]
        .iter()
        .all(|surface| surface.role == SurfaceRole::PlotPanel));
    for owner in [0, 1, 2] {
        assert_canvas_theme_identity(
            owner,
            &[
                "resolved",
                "panel-measure",
                "panel-plan",
                "grid-render",
                "panel-render",
            ],
        );
    }
    assert_rendered_themes_match_canvas_owners(&[usize::MAX, 2]);
}

#[test]
fn canvas_text_panel_uses_one_private_nonpublication_root() {
    let text_canvas = || {
        crate::Canvas::grid(1, 1)
            .unwrap()
            .theme(crate::Theme::Nature)
            .text_panel(0, 0, crate::TextPanel::new("Title", "Body").unwrap())
            .unwrap()
    };
    reset_theme_evidence();
    TEXT_PLAN_CAPTURE_SESSIONS.with(|calls| calls.set(0));
    TEXT_REPLAY_SESSIONS.with(|calls| calls.set(0));
    text_canvas().preflight().unwrap();
    assert_eq!(TEXT_PLAN_CAPTURE_SESSIONS.with(std::cell::Cell::get), 1);
    assert_eq!(TEXT_REPLAY_SESSIONS.with(std::cell::Cell::get), 0);
    assert_eq!(
        (resolved_theme_count(), rendered_theme_identity_count()),
        (1, 0)
    );
    assert_canvas_theme_identity(
        usize::MAX,
        &["resolved", "layout", "text-plan", "render", "text-render"],
    );
    reset_theme_evidence();
    TEXT_PLAN_CAPTURE_SESSIONS.with(|calls| calls.set(0));
    TEXT_REPLAY_SESSIONS.with(|calls| calls.set(0));
    let mut backend = RecordingBackend::default();
    text_canvas().render_with_backend(&mut backend).unwrap();
    assert_eq!(TEXT_PLAN_CAPTURE_SESSIONS.with(std::cell::Cell::get), 1);
    assert_eq!(TEXT_REPLAY_SESSIONS.with(std::cell::Cell::get), 1);
    assert_eq!(
        (resolved_theme_count(), rendered_theme_identity_count()),
        (1, 0)
    );
    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert_eq!(backend.surfaces.len(), 1);
    assert_eq!(backend.surfaces[0].role, SurfaceRole::CanvasRoot);
    assert_canvas_theme_identity(
        usize::MAX,
        &["resolved", "layout", "text-plan", "render", "text-render"],
    );
    for (text, size) in [("Title", 19), ("Body", 16)] {
        let style = &backend
            .text_draws
            .iter()
            .find(|(_, _, candidate, _)| candidate == text)
            .expect("private Nature root text is rendered")
            .3;
        assert_eq!(style.color, crate::theme::Color::rgb(40, 40, 40));
        assert_eq!(style.font_size, size);
    }
}

#[test]
fn caller_content_presence_is_independent_from_foundational_appearance() {
    let plot = || {
        Plot::scatter(
            NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
        )
        .theme(crate::Theme::Nature)
    };
    let mut populated = RecordingBackend::default();
    plot()
        .title("Caller title")
        .subtitle("Caller subtitle")
        .caption("Caller caption")
        .x_label("Caller x")
        .y_label("Caller y")
        .render_with_backend(&mut populated)
        .unwrap();

    for expected in [
        "Caller title",
        "Caller subtitle",
        "Caller caption",
        "Caller x",
        "Caller y",
    ] {
        assert!(populated
            .text_draws
            .iter()
            .any(|(_, _, text, _)| text == expected));
    }

    let mut bare = RecordingBackend::default();
    plot()
        .x_label("")
        .y_label("")
        .render_with_backend(&mut bare)
        .unwrap();
    let mut empty = RecordingBackend::default();
    plot()
        .title("")
        .subtitle("")
        .caption("")
        .x_label("")
        .y_label("")
        .render_with_backend(&mut empty)
        .unwrap();

    assert!(!empty
        .text_draws
        .iter()
        .any(|(_, _, text, _)| text.is_empty()));
    assert_eq!(empty.surfaces, bare.surfaces);
    assert_eq!(empty.clips, bare.clips);
    assert_eq!(empty.scatter_points, bare.scatter_points);
}

#[test]
fn canvas_preflight_and_render_reuse_one_chrome_and_one_theme_per_child() {
    let x = NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]);
    let y = NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]);
    let nature_color = crate::Theme::Nature.resolve().colors[0];
    let science_color = crate::Theme::Science.resolve().colors[0];
    let canvas = |themes: [crate::Theme; 2]| {
        crate::Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, Plot::scatter(x.clone(), y.clone()).theme(themes[0]))
            .unwrap()
            .panel(0, 1, Plot::scatter(x.clone(), y.clone()).theme(themes[1]))
            .unwrap()
    };
    for themes in [
        [crate::Theme::Nature, crate::Theme::Nature],
        [crate::Theme::Nature, crate::Theme::Science],
    ] {
        reset_theme_evidence();
        canvas(themes).preflight().unwrap();
        assert_eq!(
            (resolved_theme_count(), rendered_theme_identity_count()),
            (3, 2)
        );
        assert_canvas_theme_identity(usize::MAX, &["resolved", "layout", "render"]);
        assert_canvas_theme_identity(
            0,
            &[
                "resolved",
                "panel-measure",
                "panel-plan",
                "grid-render",
                "panel-render",
            ],
        );
        assert_canvas_theme_identity(
            1,
            &[
                "resolved",
                "panel-measure",
                "panel-plan",
                "grid-render",
                "panel-render",
            ],
        );

        reset_theme_evidence();
        let mut backend = RecordingBackend::default();
        canvas(themes).render_with_backend(&mut backend).unwrap();
        assert_eq!(
            (resolved_theme_count(), rendered_theme_identity_count()),
            (3, 2)
        );
        assert_eq!((backend.begins, backend.ends), (1, 1));
        assert_canvas_theme_identity(usize::MAX, &["resolved", "layout", "render"]);
        assert_canvas_theme_identity(
            0,
            &[
                "resolved",
                "panel-measure",
                "panel-plan",
                "grid-render",
                "panel-render",
            ],
        );
        assert_canvas_theme_identity(
            1,
            &[
                "resolved",
                "panel-measure",
                "panel-plan",
                "grid-render",
                "panel-render",
            ],
        );
        let colors = backend
            .scatter_draws
            .iter()
            .map(|(_, style, _)| style.color)
            .collect::<Vec<_>>();
        let expected = if themes[0] == themes[1] {
            vec![nature_color, nature_color]
        } else {
            vec![nature_color, science_color]
        };
        assert_eq!(
            colors,
            expected
                .into_iter()
                .enumerate()
                .map(|(index, color)| {
                    crate::theme::color_from_rgb(color)
                        .with_alpha(themes[index].resolve().marks.primary_point.fill_alpha)
                })
                .collect::<Vec<_>>()
        );
    }
}

fn mixed_theme_canvas(themes: [crate::Theme; 2]) -> crate::Canvas {
    let x = NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]);
    let y = NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]);
    crate::Canvas::grid(1, 2)
        .unwrap()
        .theme(crate::Theme::CHI)
        .panel(0, 0, Plot::scatter(x.clone(), y.clone()).theme(themes[0]))
        .unwrap()
        .panel(0, 1, Plot::scatter(x, y).theme(themes[1]))
        .unwrap()
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "capture and replay retain explicit root and child theme-stage evidence"
)]
fn canvas_shared_headers_use_canvas_root_for_plan_and_render() {
    let canvas = || {
        mixed_theme_canvas([crate::Theme::Nature, crate::Theme::Science])
            .column_header(0, 2, "Shared model")
            .unwrap()
            .row_header(0, 1, "Outcome")
            .unwrap()
    };

    reset_theme_evidence();
    canvas().preflight().unwrap();
    assert_eq!(
        (resolved_theme_count(), rendered_theme_identity_count()),
        (3, 2)
    );
    assert_canvas_theme_identity(
        usize::MAX,
        &[
            "resolved",
            "layout",
            "shared-header-plan",
            "render",
            "shared-header-render",
        ],
    );
    for owner in [0, 1] {
        assert_canvas_theme_identity(
            owner,
            &[
                "resolved",
                "panel-measure",
                "panel-plan",
                "grid-render",
                "panel-render",
            ],
        );
    }
    assert_canvas_theme_identities_differ(usize::MAX, 0);

    reset_theme_evidence();
    let mut backend = RecordingBackend::default();
    canvas().render_with_backend(&mut backend).unwrap();
    assert_eq!(
        (resolved_theme_count(), rendered_theme_identity_count()),
        (3, 2)
    );
    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert_canvas_theme_identity(
        usize::MAX,
        &[
            "resolved",
            "layout",
            "shared-header-plan",
            "render",
            "shared-header-render",
        ],
    );
    assert_canvas_theme_identity(
        0,
        &[
            "resolved",
            "panel-measure",
            "panel-plan",
            "grid-render",
            "panel-render",
        ],
    );
    assert_canvas_theme_identity(
        1,
        &[
            "resolved",
            "panel-measure",
            "panel-plan",
            "grid-render",
            "panel-render",
        ],
    );
    assert_canvas_theme_identities_differ(usize::MAX, 0);
    for text in ["Shared model", "Outcome"] {
        let style = &backend
            .text_draws
            .iter()
            .find(|(_, _, candidate, _)| candidate == text)
            .expect("shared Canvas header is rendered")
            .3;
        assert_eq!(style.color, crate::theme::Color::rgb(60, 60, 60));
        assert_eq!(style.font_size, 19);
    }

    let mut reversed = RecordingBackend::default();
    mixed_theme_canvas([crate::Theme::Science, crate::Theme::Nature])
        .column_header(0, 2, "Shared model")
        .unwrap()
        .row_header(0, 1, "Outcome")
        .unwrap()
        .render_with_backend(&mut reversed)
        .unwrap();
    for text in ["Shared model", "Outcome"] {
        let selected = |backend: &RecordingBackend| {
            let (x, y, _, style) = backend
                .text_draws
                .iter()
                .find(|(_, _, candidate, _)| candidate == text)
                .expect("shared Canvas header is rendered");
            (*x, *y, style.color, style.font_size)
        };
        assert_eq!(selected(&backend), selected(&reversed));
    }
}

#[test]
fn canvas_row_titles_use_canvas_root_for_plan_and_render() {
    let canvas = || {
        mixed_theme_canvas([crate::Theme::Nature, crate::Theme::Science])
            .row_title(0, "Shared outcome")
            .unwrap()
    };

    reset_theme_evidence();
    canvas().preflight().unwrap();
    assert_eq!(
        (resolved_theme_count(), rendered_theme_identity_count()),
        (3, 2)
    );
    assert_canvas_theme_identity(
        usize::MAX,
        &[
            "resolved",
            "layout",
            "row-title-plan",
            "render",
            "row-title-render",
        ],
    );
    for owner in [0, 1] {
        assert_canvas_theme_identity(
            owner,
            &[
                "resolved",
                "panel-measure",
                "panel-plan",
                "grid-render",
                "panel-render",
            ],
        );
    }
    assert_canvas_theme_identities_differ(usize::MAX, 0);

    reset_theme_evidence();
    let mut backend = RecordingBackend::default();
    canvas().render_with_backend(&mut backend).unwrap();
    assert_eq!(
        (resolved_theme_count(), rendered_theme_identity_count()),
        (3, 2)
    );
    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert_canvas_theme_identity(
        usize::MAX,
        &[
            "resolved",
            "layout",
            "row-title-plan",
            "render",
            "row-title-render",
        ],
    );
    assert_canvas_theme_identity(
        0,
        &[
            "resolved",
            "panel-measure",
            "panel-plan",
            "grid-render",
            "panel-render",
        ],
    );
    assert_canvas_theme_identity(
        1,
        &[
            "resolved",
            "panel-measure",
            "panel-plan",
            "grid-render",
            "panel-render",
        ],
    );
    assert_canvas_theme_identities_differ(usize::MAX, 0);
    let style = &backend
        .text_draws
        .iter()
        .find(|(_, _, text, _)| text == "Shared outcome")
        .expect("Canvas row title is rendered")
        .3;
    assert_eq!(style.color, crate::theme::Color::rgb(60, 60, 60));
    assert_eq!(style.font_size, 19);

    let mut reversed = RecordingBackend::default();
    mixed_theme_canvas([crate::Theme::Science, crate::Theme::Nature])
        .row_title(0, "Shared outcome")
        .unwrap()
        .render_with_backend(&mut reversed)
        .unwrap();
    let selected = |backend: &RecordingBackend| {
        let (_, y, _, style) = backend
            .text_draws
            .iter()
            .find(|(_, _, text, _)| text == "Shared outcome")
            .expect("Canvas row title is rendered");
        (*y, style.color, style.font_size)
    };
    // Adaptive ticks can change child gutters and hence the shared title's x
    // allocation. Its vertical position and root-owned appearance stay fixed.
    assert_eq!(selected(&backend), selected(&reversed));
}

fn sparse_shared_legend_canvas() -> crate::Canvas {
    let first_x = NumericValues::new("x", [1.0, 2.0]);
    let first_y = NumericValues::new("y", [1.0, 2.0]);
    let first_hue = CategoryValues::new("trace", ["control", "control"]).unwrap();
    let second_x = NumericValues::new("x", [1.0, 2.0]);
    let second_y = NumericValues::new("y", [2.0, 3.0]);
    let second_hue = CategoryValues::new("trace", ["control", "control"]).unwrap();
    crate::Canvas::grid(1, 4)
        .unwrap()
        .theme(crate::Theme::Nature)
        .panel(0, 0, Plot::line(first_x, first_y).hue(first_hue))
        .unwrap()
        .text_panel(
            0,
            1,
            crate::TextPanel::new("Context", "Sparse cell").unwrap(),
        )
        .unwrap()
        .panel(
            0,
            3,
            Plot::line(second_x, second_y)
                .hue(second_hue)
                .theme(crate::Theme::Nature),
        )
        .unwrap()
        .legend_policy(crate::CanvasLegendPolicy::Shared)
}

#[test]
fn sparse_canvas_coordination_keeps_theme_identity_at_declared_indices() {
    reset_theme_evidence();
    sparse_shared_legend_canvas().preflight().unwrap();
    assert_eq!(
        (resolved_theme_count(), rendered_theme_identity_count()),
        (2, 2)
    );
    assert_canvas_theme_owners(&[usize::MAX, 0, 3]);
    assert_canvas_theme_identity(
        usize::MAX,
        &["resolved", "layout", "text-plan", "render", "text-render"],
    );
    for owner in [0, 3] {
        assert_canvas_theme_identity(
            owner,
            &[
                "resolved",
                "prepared-shared-hue",
                "shared-legend-preflight",
                "panel-measure",
                "panel-plan",
                "grid-render",
                "panel-render",
            ],
        );
    }
    assert_rendered_themes_match_canvas_owners(&[0, 3]);
    crate::canvas::CANVAS_SHARED_LEGEND_THEME_EVIDENCE.with(|evidence| {
        assert_eq!(
            evidence.borrow().as_slice(),
            &[(0, "inherited-root"), (3, "explicit-plot")]
                .repeat(crate::layout::kernel::ROOT_FEEDBACK_ROUNDS.with(std::cell::Cell::get))
        );
    });

    reset_theme_evidence();
    let mut backend = RecordingBackend::default();
    sparse_shared_legend_canvas()
        .render_with_backend(&mut backend)
        .unwrap();
    assert_eq!(
        (resolved_theme_count(), rendered_theme_identity_count()),
        (2, 2)
    );
    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert_canvas_theme_owners(&[usize::MAX, 0, 3]);
    assert_canvas_theme_identity(
        usize::MAX,
        &["resolved", "layout", "text-plan", "render", "text-render"],
    );
    for owner in [0, 3] {
        assert_canvas_theme_identity(
            owner,
            &[
                "resolved",
                "prepared-shared-hue",
                "shared-legend-preflight",
                "panel-measure",
                "panel-plan",
                "grid-render",
                "panel-render",
            ],
        );
    }
    assert_rendered_themes_match_canvas_owners(&[0, 3]);
    crate::canvas::CANVAS_SHARED_LEGEND_THEME_EVIDENCE.with(|evidence| {
        assert_eq!(
            evidence.borrow().as_slice(),
            &[(0, "inherited-root"), (3, "explicit-plot")]
                .repeat(crate::layout::kernel::ROOT_FEEDBACK_ROUNDS.with(std::cell::Cell::get))
        );
    });
}

fn coordinated_horizontal_bar_canvas() -> crate::Canvas {
    let category = CategoryValues::new("outcome", ["alpha", "beta"]).unwrap();
    let left = NumericValues::new("value", [1.0, -0.5]);
    let right = NumericValues::new("value", [8.0, -6.0]);
    crate::Canvas::grid(1, 2)
        .unwrap()
        .theme(crate::Theme::CHI)
        .panel(
            0,
            0,
            Plot::horizontal_bar(category.clone(), crate::BarInput::estimates(left))
                .theme(crate::Theme::Nature),
        )
        .unwrap()
        .panel(
            0,
            1,
            Plot::horizontal_bar(category, crate::BarInput::estimates(right))
                .theme(crate::Theme::Science),
        )
        .unwrap()
        .column_header(0, 1, "Correlation")
        .unwrap()
        .column_header(1, 1, "Regression")
        .unwrap()
        .row_header(0, 1, "Outcome")
        .unwrap()
        .row_title(0, "Shared outcome")
        .unwrap()
        .category_labels(crate::CanvasCategoryLabels::FirstColumnPerRow)
        .x_range_group([(0, 0), (0, 1)])
        .unwrap()
}

#[test]
fn mixed_theme_canvas_coordination_reuses_each_stored_child_theme() {
    reset_theme_evidence();
    coordinated_horizontal_bar_canvas().preflight().unwrap();
    assert_eq!(
        (resolved_theme_count(), rendered_theme_identity_count()),
        (3, 2)
    );
    assert_canvas_theme_owners(&[usize::MAX, 0, 1]);
    assert_canvas_theme_identity(
        usize::MAX,
        &[
            "resolved",
            "layout",
            "shared-header-plan",
            "row-title-plan",
            "render",
            "row-title-render",
            "shared-header-render",
        ],
    );
    assert_canvas_theme_identity(
        0,
        &[
            "resolved",
            "shared-category-label-validation",
            "panel-measure",
            "panel-plan",
            "grid-render",
            "panel-render",
        ],
    );
    assert_canvas_theme_identity(
        1,
        &[
            "resolved",
            "shared-category-label-validation",
            "panel-measure",
            "panel-plan",
            "grid-render",
            "panel-render",
        ],
    );
    assert_rendered_themes_match_canvas_owners(&[0, 1]);

    reset_theme_evidence();
    let mut backend = RecordingBackend::default();
    coordinated_horizontal_bar_canvas()
        .render_with_backend(&mut backend)
        .unwrap();
    assert_eq!(
        (resolved_theme_count(), rendered_theme_identity_count()),
        (3, 2)
    );
    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert_canvas_theme_owners(&[usize::MAX, 0, 1]);
    assert_canvas_theme_identity(
        usize::MAX,
        &[
            "resolved",
            "layout",
            "shared-header-plan",
            "row-title-plan",
            "render",
            "row-title-render",
            "shared-header-render",
        ],
    );
    assert_canvas_theme_identity(
        0,
        &[
            "resolved",
            "shared-category-label-validation",
            "panel-measure",
            "panel-plan",
            "grid-render",
            "panel-render",
        ],
    );
    assert_canvas_theme_identity(
        1,
        &[
            "resolved",
            "shared-category-label-validation",
            "panel-measure",
            "panel-plan",
            "grid-render",
            "panel-render",
        ],
    );
    assert_rendered_themes_match_canvas_owners(&[0, 1]);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one chained terminal exercise keeps Plot, Canvas, Scene, SVG, and PNG evidence together"
)]
fn chained_terminals_reuse_one_canonical_scene_and_plan_each_png() {
    FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(0));
    crate::scene::SCENE_CONSTRUCTION_CALLS.with(|calls| calls.set(0));
    crate::scene::SCENE_ROUTE_EVIDENCE.with(|evidence| evidence.borrow_mut().clear());
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
    output
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
    assert_scene_routes(&["preserve", "outline", "png", "png", "png", "preserve"]);

    crate::canvas::CANVAS_SCENE_RENDER_CALLS.with(|calls| calls.set(0));
    crate::scene::SCENE_CONSTRUCTION_CALLS.with(|calls| calls.set(0));
    crate::scene::SCENE_ROUTE_EVIDENCE.with(|evidence| evidence.borrow_mut().clear());
    crate::scene::PNG_TARGET_VALIDATION_CALLS.with(|calls| calls.set(0));
    crate::render::svg_backend::PNG_TARGET_PLAN_CALLS.with(|calls| calls.set(0));
    let canvas_first = directory.join("canvas-first.svg");
    let canvas_second = directory.join("canvas-second.svg");
    let canvas_preview = directory.join("canvas-preview.png");
    let canvas_custom = directory.join("canvas-custom.png");
    let canvas_publication = directory.join("canvas-publication.png");
    crate::Canvas::grid(1, 1)
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
    assert_scene_routes(&["preserve", "preserve", "png", "png", "png"]);
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
fn orchestration_begins_and_ends_once_and_projects_into_offset_panel() {
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    );
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

    assert_eq!(
        (
            plot_calls,
            facet_calls,
            canvas_calls,
            preflight_calls,
            rendered_canvas_calls
        ),
        // This counter belongs to measure_panel_in_call, not render_panel.
        // Facets make four semantic probes; each Canvas measures its complete
        // provisional gutters in two rounds. Capture/replay uses retained plans
        // and the Canvas capture guard forbids this measurement entry point.
        (0, 4, 2, 2, 2)
    );
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
            Plot::scatter(x.clone(), y.clone()).erase(),
        ),
        (
            PlotKind::Histogram,
            Plot::histogram(crate::HistogramInput::samples(x.clone())).erase(),
        ),
        (PlotKind::Density, Plot::density(x.clone()).erase()),
        (PlotKind::Qq, Plot::qq(x.clone()).erase()),
        (
            PlotKind::PointInterval,
            Plot::point_interval(interval_group.clone(), x.clone(), bounds()).erase(),
        ),
        (PlotKind::Line, Plot::line(x.clone(), y.clone()).erase()),
        (
            PlotKind::Trajectory,
            Plot::trajectory(AxisValues::Numeric(x.clone()), y.clone(), group.clone()).erase(),
        ),
        (
            PlotKind::LinePath,
            Plot::line_path(x.clone(), y.clone(), group.clone()).erase(),
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
                crate::DumbbellEndpoints::new("Left", (20, 80, 140), "Right", (180, 60, 40))
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
            Plot::bar(group.clone(), crate::BarInput::samples(y.clone())).erase(),
        ),
        (
            PlotKind::HorizontalBar,
            Plot::horizontal_bar(group.clone(), crate::BarInput::samples(y.clone())).erase(),
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
            Plot::boxplot(group.clone(), y.clone()).erase(),
        ),
        (
            PlotKind::Violin,
            Plot::violin(group.clone(), y.clone()).erase(),
        ),
        (
            PlotKind::Raincloud,
            Plot::raincloud(crate::RaincloudInput::samples(group.clone(), y.clone())).erase(),
        ),
        (
            PlotKind::Regression,
            Plot::regression(x.clone(), y.clone()).erase(),
        ),
        (
            PlotKind::Marginal,
            Plot::marginal(x.clone(), y.clone()).erase(),
        ),
        (
            PlotKind::Matrix,
            Plot::matrix(vec![x.clone(), y.clone()]).erase(),
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
    for (kind, plot) in &plots {
        for (width, height) in [(None, None), (Some(800), None)] {
            let plan = crate::orchestrator::lifecycle::PlotPlan::new_with_sizing(
                plot, *kind, width, height,
            )
            .unwrap_or_else(|error| panic!("natural {kind:?} {width:?}/{height:?}: {error}"));
            let dimensions = plan.dimensions();
            assert!(dimensions.0 > 0 && dimensions.1 > 0);
            if let Some(width) = width {
                assert_eq!(dimensions.0, width);
            }
        }
    }
    crate::canvas::admission_tests::mixed_family_parent_audit(&plots);
    for (kind, plot) in plots {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let domain = crate::plots::shared_x::domain(
            &plot,
            &crate::font::canonical::OperationGenerationResources::new(),
        );
        let prepared = crate::plots::shared_x::prepare_plot(
            &plot,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap_or_else(|error| panic!("{kind:?}: {error}"))
        .attach(plot.clone())
        .unwrap();
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
            let theme = crate::Theme::Publication.resolve();
            let preparations = crate::plots::shared_x::AXIS_PREPARATIONS.with(std::cell::Cell::get);
            let panel = prepared.measured_canvas_layout(&theme).unwrap().panel_box;
            // Extracted families have no unprepared combined route.
            let expected_source = if matches!(
                kind,
                PlotKind::Scatter
                    | PlotKind::Line
                    | PlotKind::Trajectory
                    | PlotKind::LinePath
                    | PlotKind::Histogram
                    | PlotKind::Density
                    | PlotKind::Qq
                    | PlotKind::Regression
                    | PlotKind::Dumbbell
                    | PlotKind::PointInterval
                    | PlotKind::Coefficient
                    | PlotKind::Forest
                    | PlotKind::Bar
                    | PlotKind::HorizontalBar
                    | PlotKind::StackedBar
                    | PlotKind::StackedHorizontalBar
                    | PlotKind::Boxplot
                    | PlotKind::Violin
                    | PlotKind::Raincloud
                    | PlotKind::Interaction
                    | PlotKind::Heatmap
                    | PlotKind::Marginal
                    | PlotKind::Beeswarm
                    | PlotKind::Waterfall
            ) {
                &prepared
            } else {
                &plot
            };
            let expected = expected_source
                .preflight_panel(kind, panel, &theme)
                .unwrap_or_else(|error| panic!("{kind:?}: {error}"));
            let actual = prepared
                .preflight_panel(kind, panel, &theme)
                .unwrap_or_else(|error| panic!("{kind:?}: {error}"));
            assert_eq!(
                actual.layout.projection.x_range(),
                expected.layout.projection.x_range(),
                "{kind:?} x"
            );
            assert_eq!(
                actual.layout.projection.y_range(),
                expected.layout.projection.y_range(),
                "{kind:?} y"
            );
            assert_eq!(
                actual.layout.x_axis.ticks, expected.layout.x_axis.ticks,
                "{kind:?} x source"
            );
            assert_eq!(
                actual.layout.y_axis.ticks, expected.layout.y_axis.ticks,
                "{kind:?} y source"
            );
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
        for counter in [
            crate::orchestrator::measurements::Counter::TickSource,
            crate::orchestrator::measurements::Counter::TickDistinctList,
            crate::orchestrator::measurements::Counter::TickMetricLookup,
            crate::orchestrator::measurements::Counter::AxisRangePreparation,
            crate::orchestrator::measurements::Counter::AxisSpecPreparation,
            crate::orchestrator::measurements::Counter::AxisSourcePreparation,
        ] {
            assert_eq!(
                work.capture_counters[counter as usize], 0,
                "{kind:?}: {counter:?}"
            );
        }
        assert_eq!((backend.begins, backend.ends), (1, 1), "{kind:?}");
        assert_eq!(plan.dimensions(), (640, 480));
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
        let scene = crate::scene::RenderedScene::new(640, 480, svg.into_output()).unwrap();
        let preserve = scene.svg_bytes().to_vec();
        assert!(std::str::from_utf8(&preserve)
            .unwrap()
            .contains("viewBox=\"0 0 640 480\""));
        let outlined = scene.svg_with_text_as_paths().unwrap();
        assert!(!std::str::from_utf8(&outlined).unwrap().contains("<text"));
        let png = scene
            .png_bytes(&crate::PngOptions {
                sizing: crate::PngSizing::Scale(1.0),
                ..crate::PngOptions::default()
            })
            .unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 640);
        assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 480);
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
    )]);
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
    .hue(hue);
    let mut backend = RecordingBackend {
        failures: vec![BackendFailure::PolygonInLayer, BackendFailure::LayerEnd],
        ..RecordingBackend::default()
    };

    let error = plot
        .render_kind(PlotKind::Matrix, &mut backend)
        .unwrap_err()
        .to_string();

    assert!(error.contains("layer polygon draw failed"));
    assert!(error.contains("layer cleanup failed"));
    assert_eq!(backend.layer_ends, 1);
    assert_eq!(backend.ends, 1);
}
