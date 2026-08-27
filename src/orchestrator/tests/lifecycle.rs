use std::sync::atomic::Ordering;

use super::{
    test_directory, BackendFailure, Plot, PlotKind, RecordingBackend, Significance,
    SignificanceMark, ValueSort, FINAL_PLOT_RENDER_CALLS, PLOT_PLAN_CALLS, RENDER_PANEL_CALLS,
    RENDER_THEME_ADDRESSES, TEMP_FILE_COUNTER,
};
use crate::orchestrator::DeferredBackend;
use crate::render::backend::{Backend, LineStyle, Point2D, TextAnchor, TextStyle};
use crate::{AxisValues, CategoryValues, NumericValues};

fn valid_line() -> ([Point2D; 2], LineStyle) {
    (
        [Point2D::new(1.0, 1.0), Point2D::new(2.0, 2.0)],
        LineStyle::solid((0, 0, 0), 1.0),
    )
}

fn reset_theme_evidence() {
    crate::theme::THEME_RESOLUTION_CALLS.with(|calls| calls.set(0));
    RENDER_THEME_ADDRESSES.with(|addresses| addresses.borrow_mut().clear());
    crate::canvas::CANVAS_THEME_EVIDENCE.with(|evidence| evidence.borrow_mut().clear());
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
        for stage in stages {
            assert!(records.iter().any(|(_, candidate, _)| candidate == stage));
        }
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
                    color: (0, 0, 0),
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
    let chart = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .caption("diagnostic provenance and model adjustment details ".repeat(20))
    .size(160, 120);
    let mut backend = RecordingBackend::default();

    let error = chart.render_with_backend(&mut backend).unwrap_err();

    assert!(error.to_string().contains("measured canvas text"));
    assert_eq!(backend.begins, 0);
}

#[test]
fn standalone_terminal_resolves_and_plans_once_with_one_final_lifecycle() {
    crate::theme::THEME_RESOLUTION_CALLS.with(|calls| calls.set(0));
    PLOT_PLAN_CALLS.with(|calls| calls.set(0));
    FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(0));
    RENDER_THEME_ADDRESSES.with(|addresses| addresses.borrow_mut().clear());
    let chart = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .theme(crate::Theme::Nature);
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    assert_eq!(
        crate::theme::THEME_RESOLUTION_CALLS.with(std::cell::Cell::get),
        1
    );
    assert_eq!(PLOT_PLAN_CALLS.with(std::cell::Cell::get), 1);
    assert_eq!(FINAL_PLOT_RENDER_CALLS.with(std::cell::Cell::get), 1);
    assert_eq!((backend.begins, backend.ends), (1, 1));
    RENDER_THEME_ADDRESSES.with(|addresses| {
        let addresses = addresses.borrow();
        assert!(!addresses.is_empty());
        assert!(addresses.iter().all(|address| *address == addresses[0]));
    });
}

#[test]
fn public_preflight_uses_the_prepared_plan_without_a_final_lifecycle() {
    crate::theme::THEME_RESOLUTION_CALLS.with(|calls| calls.set(0));
    PLOT_PLAN_CALLS.with(|calls| calls.set(0));
    FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(0));
    RENDER_THEME_ADDRESSES.with(|addresses| addresses.borrow_mut().clear());
    let chart = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .theme(crate::Theme::CHI);

    chart.preflight().unwrap();

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
fn canvas_preflight_and_render_reuse_one_chrome_and_one_theme_per_child() {
    let text_canvas = || {
        crate::Canvas::grid(1, 1)
            .unwrap()
            .text_panel(0, 0, crate::TextPanel::new("Title", "Body").unwrap())
            .unwrap()
    };
    reset_theme_evidence();
    text_canvas().preflight().unwrap();
    assert_eq!(
        (resolved_theme_count(), rendered_theme_identity_count()),
        (1, 0)
    );
    assert_canvas_theme_identity(
        usize::MAX,
        &["resolved", "layout", "header-plan", "text-plan"],
    );
    reset_theme_evidence();
    let mut backend = RecordingBackend::default();
    text_canvas().render_with_backend(&mut backend).unwrap();
    assert_eq!(
        (resolved_theme_count(), rendered_theme_identity_count()),
        (1, 0)
    );
    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert_canvas_theme_identity(
        usize::MAX,
        &[
            "resolved",
            "layout",
            "header-plan",
            "text-plan",
            "render",
            "header-render",
            "text-render",
        ],
    );

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
        assert_canvas_theme_identity(usize::MAX, &["resolved", "layout"]);
        assert_canvas_theme_identity(0, &["resolved", "header-plan", "panel-plan"]);
        assert_canvas_theme_identity(1, &["resolved", "panel-plan"]);

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
                "header-plan",
                "panel-plan",
                "header-render",
                "grid-render",
                "panel-render",
            ],
        );
        assert_canvas_theme_identity(
            1,
            &["resolved", "panel-plan", "grid-render", "panel-render"],
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
        assert_eq!(colors, expected);
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one chained terminal exercise keeps Plot, Canvas, Scene, SVG, and PNG evidence together"
)]
fn chained_terminals_reuse_one_canonical_scene_and_one_png_target_plan() {
    FINAL_PLOT_RENDER_CALLS.with(|calls| calls.set(0));
    crate::scene::SCENE_CONSTRUCTION_CALLS.with(|calls| calls.set(0));
    crate::scene::SCENE_ROUTE_EVIDENCE.with(|evidence| evidence.borrow_mut().clear());
    crate::scene::PNG_TARGET_VALIDATION_CALLS.with(|calls| calls.set(0));
    crate::render::svg_backend::PNG_TARGET_PLAN_CALLS.with(|calls| calls.set(0));
    let directory = test_directory("canonical-scene-routing");
    let first = directory.join("first.svg");
    let outlined = directory.join("outlined.svg");
    let png = directory.join("preview.png");
    let second = directory.join("second.svg");
    Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0]),
        NumericValues::new("y", [3.0, 1.0, 2.0]),
    )
    .size(320, 240)
    .title("Canonical scene")
    .svg_preserve_text(&first)
    .unwrap()
    .svg(&outlined)
    .unwrap()
    .png_preview(&png)
    .unwrap()
    .svg_preserve_text(&second)
    .unwrap();

    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(&second).unwrap()
    );
    assert!(!std::fs::read_to_string(outlined).unwrap().contains("<text"));
    let png = std::fs::read(png).unwrap();
    assert_eq!(
        (
            u32::from_be_bytes(png[16..20].try_into().unwrap()),
            u32::from_be_bytes(png[20..24].try_into().unwrap()),
        ),
        (320, 240)
    );
    assert_eq!(FINAL_PLOT_RENDER_CALLS.with(std::cell::Cell::get), 1);
    assert_eq!(
        crate::scene::SCENE_CONSTRUCTION_CALLS.with(std::cell::Cell::get),
        1
    );
    assert_eq!(
        crate::render::svg_backend::PNG_TARGET_PLAN_CALLS.with(std::cell::Cell::get),
        1
    );
    assert_eq!(
        crate::scene::PNG_TARGET_VALIDATION_CALLS.with(std::cell::Cell::get),
        1
    );
    crate::scene::SCENE_ROUTE_EVIDENCE.with(|evidence| {
        let evidence = evidence.borrow();
        assert_eq!(
            evidence
                .iter()
                .map(|(route, _, _)| *route)
                .collect::<Vec<_>>(),
            ["preserve", "outline", "png", "preserve"]
        );
        assert!(evidence
            .iter()
            .all(|(_, address, length)| (*address, *length) == (evidence[0].1, evidence[0].2)));
    });

    crate::canvas::CANVAS_SCENE_RENDER_CALLS.with(|calls| calls.set(0));
    crate::scene::SCENE_CONSTRUCTION_CALLS.with(|calls| calls.set(0));
    crate::scene::SCENE_ROUTE_EVIDENCE.with(|evidence| evidence.borrow_mut().clear());
    let canvas_first = directory.join("canvas-first.svg");
    let canvas_second = directory.join("canvas-second.svg");
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
        .svg_preserve_text(&canvas_first)
        .unwrap()
        .svg_preserve_text(&canvas_second)
        .unwrap();
    assert_eq!(
        std::fs::read(canvas_first).unwrap(),
        std::fs::read(canvas_second).unwrap()
    );
    assert_eq!(
        crate::canvas::CANVAS_SCENE_RENDER_CALLS.with(std::cell::Cell::get),
        1
    );
    assert_eq!(
        crate::scene::SCENE_CONSTRUCTION_CALLS.with(std::cell::Cell::get),
        1
    );
    crate::scene::SCENE_ROUTE_EVIDENCE.with(|evidence| {
        let evidence = evidence.borrow();
        assert_eq!(evidence.len(), 2);
        assert_eq!(evidence[0], evidence[1]);
    });
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn invalid_chart_preflights_before_output_path_access() {
    let chart = Plot::scatter(
        crate::NumericValues::new("x", [1.0, 2.0]),
        crate::NumericValues::new("y", [1.0]),
    );
    let parent = std::env::temp_dir().join(format!(
        "lichtung-invalid-preflight-{}-{}",
        std::process::id(),
        TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let output = parent.join("missing").join("chart.svg");

    let error = chart.svg(&output).err().unwrap();

    assert!(error
        .to_string()
        .contains("plot value \"y\" has length 1, expected 2"));
    assert!(!parent.exists());
}

#[test]
fn invalid_canvas_alignment_fails_before_backend_begin() {
    let chart = Plot::line(
        crate::NumericValues::new("x", [1.0, 2.0]),
        crate::NumericValues::new("y", [1.0, 2.0, 3.0]),
    );
    let canvas = crate::Canvas::grid(1, 1)
        .unwrap()
        .panel(0, 0, chart)
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
    let chart = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    );
    let expected_panel = chart.canvas_layout().panel_box;
    let mut backend = RecordingBackend::default();

    chart.render_kind(PlotKind::Scatter, &mut backend).unwrap();

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
    let chart = Plot::scatter(x.clone(), y.clone());
    RENDER_PANEL_CALLS.with(|calls| calls.set(0));
    chart
        .render_kind(PlotKind::Scatter, &mut RecordingBackend::default())
        .unwrap();
    let chart_calls = RENDER_PANEL_CALLS.with(std::cell::Cell::get);

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
            chart_calls,
            facet_calls,
            canvas_calls,
            preflight_calls,
            rendered_canvas_calls
        ),
        (3, 10, 3, 2, 3)
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
    let charts = vec![
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
    for (kind, chart) in charts {
        let mut backend = RecordingBackend::default();

        chart
            .render_kind(kind, &mut backend)
            .unwrap_or_else(|error| panic!("{kind:?} failed: {error}"));
        assert_eq!((backend.begins, backend.ends), (1, 1), "{kind:?}");
    }
}

#[test]
fn heatmap_validation_failure_happens_before_canvas_begin() {
    let chart = Plot::heatmap(
        CategoryValues::new("interval_group", ["a", "b", "c", "d"]).unwrap(),
        CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
        NumericValues::new("color", [0.1, 0.3, 0.7, 1.0]),
    )
    .x_category_order(ValueSort::Ascending);
    let mut backend = RecordingBackend::default();

    let error = chart
        .render_kind(PlotKind::Heatmap, &mut backend)
        .unwrap_err()
        .to_string();

    assert!(error.contains("cannot use ValueSort::Ascending"));
    assert_eq!((backend.begins, backend.ends), (0, 0));
}

#[test]
fn orchestration_does_not_end_a_canvas_that_failed_to_begin() {
    let chart = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    );
    let mut backend = RecordingBackend {
        failures: vec![BackendFailure::Begin],
        ..RecordingBackend::default()
    };

    assert!(chart.render_kind(PlotKind::Scatter, &mut backend).is_err());
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
            let mut chart = Plot::scatter(x.clone(), y.clone());
            chart = if x_axis {
                chart.x_limits(limits.0, limits.1)
            } else {
                chart.y_limits(limits.0, limits.1)
            };
            let mut backend = RecordingBackend::default();

            assert!(chart.render_with_backend(&mut backend).is_err());
            assert_eq!(backend.begins, 0, "limits={limits:?}, x_axis={x_axis}");
        }
    }
}

#[test]
fn orchestration_renders_valid_adjacent_subnormal_limits() {
    let upper = f64::from_bits(1);
    let chart = Plot::scatter(
        NumericValues::new("x", [0.0, upper]),
        NumericValues::new("y", [0.0, upper]),
    )
    .x_limits(0.0, upper)
    .y_limits(0.0, upper);
    let mut backend = RecordingBackend::default();

    chart.render_with_backend(&mut backend).unwrap();

    assert_eq!((backend.begins, backend.ends), (1, 1));
    assert!(backend
        .scatter_points
        .iter()
        .all(|point| point.x.is_finite() && point.y.is_finite()));
}

#[test]
fn significance_annotations_are_drawn_after_marks_clip_closes() {
    let chart = Plot::boxplot(
        CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .significance([Significance::between(
        "a",
        "b",
        SignificanceMark::PValue(0.01),
    )]);
    let mut backend = RecordingBackend::default();

    chart.render_kind(PlotKind::Boxplot, &mut backend).unwrap();

    assert_eq!(backend.bracket_clip_depths, [0]);
}

#[test]
fn unclamped_projection_leaves_out_of_range_marks_for_backend_clipping() {
    let chart = Plot::scatter(
        NumericValues::new("x", [-1.0, 0.5, 2.0]),
        NumericValues::new("y", [0.5, 0.5, 0.5]),
    )
    .x_limits(0.0, 1.0)
    .y_limits(0.0, 1.0);
    let mut backend = RecordingBackend::default();

    chart.render_kind(PlotKind::Scatter, &mut backend).unwrap();

    let panel = backend.clips[0];
    assert!(backend.scatter_points[0].x < panel.x0);
    assert!(backend.scatter_points[1].x > panel.x0 && backend.scatter_points[1].x < panel.x1);
    assert!(backend.scatter_points[2].x > panel.x1);

    let dir = test_directory("out-of-range-svg");
    let output = dir.join("canvas.svg");
    chart.clone().svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(&output).unwrap();
    assert!(svg.contains(&format!("cx=\"{:.3}\"", backend.scatter_points[0].x)));
    assert!(svg.contains(&format!("cx=\"{:.3}\"", backend.scatter_points[2].x)));
    assert!(svg.contains("<g clip-path=\"url(#clip-0)\">"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn scope_and_canvas_cleanup_errors_preserve_primary_context() {
    let chart = Plot::scatter(
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

    let error = chart
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
    let chart = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    );
    let mut backend = RecordingBackend {
        failures: vec![BackendFailure::Line, BackendFailure::LayerEnd],
        ..RecordingBackend::default()
    };

    let error = chart
        .render_kind(PlotKind::Scatter, &mut backend)
        .unwrap_err()
        .to_string();

    assert!(error.contains("line draw failed"));
    assert!(error.contains("layer cleanup failed"));
    assert_eq!(backend.layer_ends, 1);
    assert_eq!(backend.ends, 1);
}

#[test]
fn matrix_legend_layer_closes_after_inner_draw_error() {
    let hue = CategoryValues::new("hue", ["h1", "h2", "h1", "h2"]).unwrap();
    let chart = Plot::matrix(vec![
        NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    ])
    .group(hue.clone())
    .hue(hue);
    let mut backend = RecordingBackend {
        failures: vec![BackendFailure::PolygonInLayer, BackendFailure::LayerEnd],
        ..RecordingBackend::default()
    };

    let error = chart
        .render_kind(PlotKind::Matrix, &mut backend)
        .unwrap_err()
        .to_string();

    assert!(error.contains("layer polygon draw failed"));
    assert!(error.contains("layer cleanup failed"));
    assert_eq!(backend.layer_ends, 1);
    assert_eq!(backend.ends, 1);
}
