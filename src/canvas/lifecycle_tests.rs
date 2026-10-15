use super::*;

#[test]
fn canvas_presets_share_inherited_roots_and_preserve_explicit_children() {
    for preset in [
        Theme::Publication,
        Theme::Nature,
        Theme::Science,
        Theme::CHI,
    ] {
        let plot = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 2.0]),
        );
        let plots = [
            Some(plot.clone().erase()),
            Some(plot.theme(Theme::Science).erase()),
        ];

        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        crate::render::svg_backend::SVG_BEGIN_CALLS.with(|calls| calls.set(0));
        let themes = ResolvedCanvasThemes::new(preset, &plots).unwrap();
        assert!(std::ptr::eq(themes.root(), themes.plot(0).unwrap()));
        assert!(!std::ptr::eq(themes.root(), themes.plot(1).unwrap()));
        let mut explicit = Theme::Science.resolve().unwrap();
        explicit.canvas_surface_override = Some(themes.root().surfaces.canvas_root);
        assert_eq!(themes.plot(1).unwrap(), &explicit);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        crate::render::svg_backend::SVG_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
    }
}

fn intrinsic_canvas() -> Canvas {
    Canvas::grid(2, 3)
        .unwrap()
        .panel(
            0,
            0,
            Plot::scatter(
                crate::NumericValues::new("x", [0.0, 1.0]),
                crate::NumericValues::new("y", [1.0, 2.0]),
            )
            .x_label("Inherited axis"),
        )
        .unwrap()
        .panel(
            0,
            2,
            Plot::trajectory(
                crate::CategoryValues::new("condition", ["first", "second"]).unwrap(),
                crate::NumericValues::new("y", [1.0, 2.0]),
                crate::CategoryValues::new("group", ["one", "one"]).unwrap(),
            )
            .x_label("Explicit axis")
            .theme(Theme::Nature),
        )
        .unwrap()
        .panel(
            1,
            1,
            Plot::heatmap(
                crate::CategoryValues::new("column", ["left", "right"]).unwrap(),
                crate::CategoryValues::new("row", ["r", "r"]).unwrap(),
                crate::NumericValues::new("value", [1.0, 2.0]),
            )
            .annotations(crate::HeatmapAnnotations::Values(
                crate::NumericLabelFormat::default(),
            )),
        )
        .unwrap()
        .text_panel(1, 2, TextPanel::new("Prose", "Retained body text").unwrap())
        .unwrap()
        .legend_policy(CanvasLegendPolicy::Hidden)
        .size(1501, 1101)
}

#[test]
fn post_font_canvas_intrinsics_are_scoped_once_before_allocation_and_rebuilt_for_new_roots() {
    use crate::orchestrator::measurements::{self, Counter};
    for (preset, title, category, uncached) in [
        (
            Theme::Publication,
            "CanvasRootTitle",
            "First observation",
            false,
        ),
        (
            Theme::Science,
            "ReplacementTitle",
            "Changed condition",
            true,
        ),
    ] {
        let canvas = intrinsic_canvas()
            .theme(preset)
            .title(title)
            .labels(Labels::raw().alias("first", category));
        measurements::start();
        crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(uncached));
        let result = canvas.preflight_plan();
        crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
        let root = result.unwrap();
        let work = measurements::finish();
        assert!(work.roots.len() > 1);
        assert_eq!(work.counters[Counter::StandardPanelIntrinsic as usize], 2);
        assert_eq!(work.counters[Counter::HeatmapIntrinsic as usize], 1);
        assert_eq!(work.counters[Counter::RootTitleIntrinsic as usize], 4);
        assert_eq!(work.intrinsics.len(), 7);
        for (counter, scope) in [
            (Counter::StandardPanelIntrinsic, Some("panel-0")),
            (Counter::StandardPanelIntrinsic, Some("panel-2")),
            (Counter::HeatmapIntrinsic, Some("panel-4")),
            (Counter::RootTitleIntrinsic, None),
            (Counter::RootTitleIntrinsic, Some("panel-0")),
            (Counter::RootTitleIntrinsic, Some("panel-2")),
            (Counter::RootTitleIntrinsic, Some("panel-4")),
        ] {
            assert_eq!(
                work.intrinsics
                    .iter()
                    .filter(|event| event.counter == counter
                        && event.scope.as_deref() == scope
                        && event.root_round.is_none())
                    .count(),
                1
            );
        }
        assert_eq!(work.capture_counters, [0; Counter::ALL.len()]);
        let prepared = root.prepared();
        assert!(prepared
            .themes
            .root()
            .prepared_font_environment()
            .unwrap()
            .prepared_pairs_are_sealed()
            .unwrap());
        let axes = &prepared.interiors[2].as_ref().unwrap().axes[0].1;
        assert_eq!(axes.display_categories[0].as_ref().unwrap()[0], category);
        let text = root.plan().retained_text();
        for (label, theme) in [
            ("Inherited axis", prepared.themes.root()),
            ("Explicit axis", prepared.themes.plot(2).unwrap()),
        ] {
            let plan = text
                .iter()
                .find(|(_, plan)| plan.source == label)
                .unwrap()
                .1;
            assert_eq!(plan.style.paint, theme.typography.axes.axis_title.paint);
            assert_eq!(plan.style.stack, theme.typography.axes.axis_title.stack);
        }
        let reference = preset.resolve().unwrap();
        let widths = crate::layout::fit::intrinsic_widths(
            &reference,
            title,
            &reference.typography.root.title,
            "root title reference",
        )
        .unwrap();
        assert_eq!(prepared.root_text.widths(), widths);
        assert_eq!(
            text.iter().filter(|(_, plan)| plan.source == title).count(),
            1
        );
        let before = capture_counters();
        let mut svg = root.plan().take_svg_backend().unwrap();
        root.plan()
            .replay(1501, 1101, &mut svg, "prepared Canvas intrinsic replay")
            .unwrap();
        assert!(!svg.into_bytes().unwrap().is_empty());
        assert_eq!(capture_counters(), before);
    }
}

#[test]
fn canvas_root_title_hard_width_rejects_before_child_measurement() {
    use crate::orchestrator::measurements::{self, Counter};
    let canvas = intrinsic_canvas().title("W".repeat(100)).size(333, 1101);
    let before = capture_counters();
    measurements::start();
    let error = canvas.preflight_plan().err().unwrap();
    let work = measurements::finish();
    let expected = FitScope::root()
        .child(crate::layout::AddressRole::RootTitle, 0)
        .unwrap()
        .failure(
            FitKind::Text,
            "root title block is narrower than its hard text intrinsic",
        )
        .into_error();
    assert_eq!(error.to_string(), expected.to_string());
    assert_eq!(work.counters[Counter::RootTitleIntrinsic as usize], 4);
    assert_eq!(
        work.counters[Counter::StandardCoordinateMeasure as usize],
        0
    );
    assert_eq!(work.counters[Counter::ChildCacheMiss as usize], 0);
    assert_eq!(work.calls[2], 0);
    let after = capture_counters();
    assert_eq!(after[1], before[1]);
    assert_eq!(after[4..], before[4..]);
}

#[test]
fn default_canvas_is_exact_960_by_640_and_matches_explicit_size() {
    let canvas = Canvas::grid(1, 1)
        .unwrap()
        .panel(
            0,
            0,
            Plot::scatter(
                crate::NumericValues::new("x", [1.0, 2.0]),
                crate::NumericValues::new("y", [2.0, 1.0]),
            )
            .size(1, 1),
        )
        .unwrap()
        .title("Canvas title\nSecond line")
        .caption("Canvas caption");
    let scene = canvas.render_scene().unwrap();
    assert!(std::str::from_utf8(scene.svg_bytes())
        .unwrap()
        .contains("viewBox=\"0 0 960 640\""));
    let explicit = canvas.size(960, 640).render_scene().unwrap();
    assert_eq!(scene.svg_bytes(), explicit.svg_bytes());
}

pub(super) fn capture_counters() -> [usize; 6] {
    [
        crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
        crate::layout::Projection::test_projection_calls(),
        crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
        crate::plots::shared_x::AXIS_PREPARATIONS.with(std::cell::Cell::get),
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
        crate::orchestrator::SELECTED_INTERIOR_MEASUREMENTS.with(std::cell::Cell::get),
    ]
}

/// Parent interaction uses representative coordinate, categorical, grid and
/// distribution children. The caller separately exercises every family as a root.
pub(crate) fn mixed_family_parent_audit(plots: &[(PlotKind, Plot)]) {
    assert_eq!(plots.len(), 25);
    let mut canvas = Canvas::grid(2, 2)
        .unwrap()
        .size(1601, 1201)
        .title("Mixed Canvas families")
        .subtitle("Retained child scopes")
        .caption("Independent root caption")
        .labels(Labels::raw().alias("x", "Shared sample"));
    let mut index = 0;
    for (kind, plot) in plots {
        if *kind == PlotKind::Matrix {
            let mut rejected = Canvas::grid(1, 1).unwrap();
            rejected.panels[0] = Some(plot.clone());
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            assert!(rejected.preflight_plan().is_err());
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
            continue;
        }
        if !matches!(
            kind,
            PlotKind::Scatter | PlotKind::HorizontalBar | PlotKind::Heatmap | PlotKind::Raincloud
        ) {
            continue;
        }
        canvas.panels[index] = Some(plot.clone().title(format!("{kind:?}")));
        index += 1;
    }
    assert_eq!(index, 4);
    crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS.with(|calls| calls.set(0));
    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
    crate::orchestrator::measurements::start();
    let cached = canvas.render_scene().unwrap();
    let work = crate::orchestrator::measurements::finish();
    assert_eq!(work.calls[2], 1);
    assert_eq!(work.calls[3], 1);
    assert_eq!(
        work.capture_counters,
        [0; crate::orchestrator::measurements::Counter::ALL.len()]
    );
    crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|disabled| disabled.set(true));
    let uncached = canvas.render_scene();
    crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|disabled| disabled.set(false));
    assert_eq!(cached.svg_bytes(), uncached.unwrap().svg_bytes());
    crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS
        .with(|calls| assert_eq!(calls.get(), 0));
    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| assert_eq!(calls.get(), 0));
}
