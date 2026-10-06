use super::*;

#[test]
fn canvas_preset_construction_uses_one_prepared_ledger_for_root_and_explicit_children() {
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .erase();
    let plots = [Some(plot.clone()), Some(plot.theme(Theme::Science))];
    // Root plus one explicit child have four retained outline values; the
    // inherited child shares the root rather than constructing another preset.
    let outline_bytes = (std::mem::size_of::<crate::theme::RelativeSpacing>()
        + 2 * std::mem::size_of::<num_bigint::BigUint>()
        + std::mem::size_of::<f64>()
        + 2 * std::mem::size_of::<usize>()) as u64
        + 4 * (16 + std::mem::size_of::<num_bigint::BigUint>() as u64);
    for preset in [
        Theme::Publication,
        Theme::Nature,
        Theme::Science,
        Theme::CHI,
    ] {
        let seeds = if preset == Theme::Publication { 8 } else { 5 };
        let total_seeds = seeds + 5; // explicit Science child; inherited child shares the root
        let ratio_bytes = (std::mem::size_of::<crate::theme::RelativeSpacing>()
            + 2 * std::mem::size_of::<num_bigint::BigUint>()
            + std::mem::size_of::<f64>()
            + 2 * std::mem::size_of::<usize>()) as u64
            + 2 * (16 + std::mem::size_of::<num_bigint::BigUint>() as u64);
        let bytes = 4 * outline_bytes
            + 12 * ratio_bytes
            + 8 * (std::mem::size_of::<Vec<crate::theme::OpaqueColor>>()
                + 2 * std::mem::size_of::<usize>()) as u64
            + 2 * total_seeds * std::mem::size_of::<crate::theme::OpaqueColor>() as u64
            + 10 * std::mem::size_of::<crate::theme::ColorStop>() as u64;
        let root_axis_work = if preset == Theme::Nature { 31 } else { 38 };
        let work = 62_u128 + 2 * u128::from(total_seeds) + root_axis_work + 38;
        for (visits, byte_limit) in [
            (0, bytes),
            (work, 0),
            (work - 1, bytes),
            (work, bytes - 1),
            (work, bytes),
        ] {
            let retained = std::cell::RefCell::new(None);
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            crate::render::svg_backend::SVG_BEGIN_CALLS.with(|calls| calls.set(0));
            let result = crate::layout::plan_exact_root::<(), (), ()>(
                800,
                600,
                |stage| {
                    let resources = stage.generation_resources();
                    assert!(
                        resources.same_operation(&stage.font_resources().generation_resources())
                    );
                    resources.set_test_limits(visits, byte_limit);
                    *retained.borrow_mut() = Some(resources.clone());
                    let themes = ResolvedCanvasThemes::new(preset, &plots, &resources)?;
                    assert_eq!(resources.usage().unwrap().1, work);
                    assert_eq!(resources.byte_usage().unwrap(), bytes);
                    assert!(std::ptr::eq(themes.root(), themes.plot(0).unwrap()));
                    assert!(!std::ptr::eq(themes.root(), themes.plot(1).unwrap()));
                    for theme in [themes.root(), themes.plot(1).unwrap()] {
                        assert_eq!(
                            theme
                                .marks
                                .primary_point
                                .outline
                                .as_ref()
                                .unwrap()
                                .width_ratio
                                .as_f64()
                                .to_bits(),
                            0x3fb9_9999_9999_999a
                        );
                    }
                    Err(Error::Render(
                        "Canvas preset construction boundary reached".into(),
                    ))
                },
                |(), _, _| unreachable!("constructor boundary precedes measurement"),
                |(), _, (), _| unreachable!("constructor boundary precedes capture"),
            );
            let error = result.err().unwrap().to_string();
            let retained = retained.borrow();
            let resources = retained.as_ref().unwrap();
            if (visits, byte_limit) == (work, bytes) {
                assert!(error.contains("Canvas preset construction boundary reached"));
            } else {
                assert!(
                    error.contains("admission")
                        || error.contains("scale input")
                        || error.contains("gradient validation")
                        || error.contains("bytes limit"),
                    "{error}"
                );
                assert!(resources.byte_usage().unwrap() <= bytes);
                if byte_limit < bytes {
                    assert!(resources.byte_usage().unwrap() < bytes);
                }
            }
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| assert_eq!(calls.get(), 0));
            crate::render::svg_backend::SVG_BEGIN_CALLS.with(|calls| assert_eq!(calls.get(), 0));
        }
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
        let reference = preset
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
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
        .child(
            crate::layout::AddressRole::RootTitle,
            0,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
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

/// The exhaustive 25-family lifecycle fixture invokes this with its actual
/// source Plots. Matrix remains a root, never an expanded Canvas child.
pub(crate) fn mixed_family_parent_audit(plots: &[(PlotKind, Plot)]) {
    assert_eq!(plots.len(), 25);
    let mut canvas = Canvas::grid(4, 6)
        .unwrap()
        .size(2401, 2001)
        .title("All supported Canvas families")
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
        canvas.panels[index] = Some(plot.clone().title(format!("{kind:?}")));
        index += 1;
    }
    assert_eq!(index, 24);
    // Mixed four-child roots stay within the existing operation budget; do not
    // relax the cap merely to fit all 24 supported families in one operation.
    let panels = std::mem::take(&mut canvas.panels);
    canvas.rows = 2;
    canvas.columns = 2;
    canvas.width = 1601;
    canvas.height = 1201;
    canvas.text_panels = vec![None; 4];
    for group in panels.chunks(4) {
        canvas.panels = group.to_vec();
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
        crate::decorations::plan::children::DISABLE_CHILD_REUSE
            .with(|disabled| disabled.set(false));
        assert_eq!(cached.svg_bytes(), uncached.unwrap().svg_bytes());
        crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS
            .with(|calls| assert_eq!(calls.get(), 0));
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| assert_eq!(calls.get(), 0));
    }
}

#[test]
fn parent_metadata_exact_refusal_precedes_fonts_and_backends() {
    let canvas = Canvas::grid(3, 5)
        .unwrap()
        .column_header(0, 5, "column span")
        .unwrap()
        .row_header(0, 3, "row span")
        .unwrap()
        .row_title(2, "row title")
        .unwrap();
    let run = |limits: Option<(u128, u64)>| {
        let resources = crate::font::canonical::OperationGenerationResources::new();
        if let Some((visits, bytes)) = limits {
            resources.set_test_limits(visits, bytes);
        }
        canvas.admit_parent_metadata(&resources)?;
        Ok::<_, Error>((
            resources.usage().unwrap().1,
            resources.byte_usage().unwrap(),
        ))
    };
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
    let (visits, bytes) = run(None).unwrap();
    assert_eq!(run(Some((visits, bytes))).unwrap(), (visits, bytes));
    assert!(run(Some((visits - 1, bytes)))
        .unwrap_err()
        .to_string()
        .contains("admission"));
    assert!(run(Some((visits, bytes - 1)))
        .unwrap_err()
        .to_string()
        .contains("admission"));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
    assert_eq!(
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
        0
    );
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one exact-cap fixture inventories the complete prepared Canvas parent"
)]
fn complete_parent_admission_has_exact_work_and_storage_boundaries() {
    let canvas = Canvas::grid(2, 2)
        .unwrap()
        .panel(
            0,
            0,
            Plot::scatter(
                crate::NumericValues::new("raw_x", [1.0, 2.0, 3.0, 4.0]),
                crate::NumericValues::new("raw_y", [1.0, 3.0, 2.0, 4.0]),
            )
            .hue(crate::CategoryValues::new("hue", ["a", "a", "b", "b"]).unwrap())
            .x_ticks(vec![crate::AxisTick::new(2.0, "two")])
            .title("Panel title")
            .subtitle("Panel subtitle"),
        )
        .unwrap()
        .text_panel(
            1,
            1,
            TextPanel::new("Prose title", "First sentence and a second sentence.").unwrap(),
        )
        .unwrap()
        .row_header(0, 2, "Rows")
        .unwrap()
        .column_header(0, 2, "Columns")
        .unwrap()
        .row_title(1, "Row title")
        .unwrap()
        .title("Root title")
        .subtitle("Root subtitle")
        .caption("Root caption")
        .labels(Labels::raw().alias("raw_x", "Sample"))
        .legend_policy(CanvasLegendPolicy::Shared)
        .size(1201, 901);
    let run = |limits: Option<(u128, u64)>| {
        let themes = Arc::new(
            ResolvedCanvasThemes::new(
                canvas.theme,
                &canvas.panels,
                &crate::font::canonical::OperationGenerationResources::new(),
            )
            .unwrap(),
        );
        let mut children = crate::decorations::plan::children::ChildPlans::canvas(&canvas);
        crate::layout::plan_exact_root(
            canvas.width,
            canvas.height,
            |stage| {
                let resources = stage.generation_resources();
                if let Some((visits, bytes)) = limits {
                    resources.set_test_limits(visits, bytes);
                }
                canvas.admit_parent_metadata(&resources)?;
                let (interiors, shared) = canvas.prepare_interiors(stage, Some(&themes))?;
                let (mut census, presence) =
                    canvas.foundational_text_census(&themes, &interiors, None, stage)?;
                themes.prepare_font_environment(&mut census, presence, stage)?;
                let (root_text, text_intrinsics, plot_intrinsics) =
                    canvas.prepare_intrinsics(&themes, &interiors)?;
                Ok((
                    interiors,
                    shared,
                    resources,
                    root_text,
                    text_intrinsics,
                    plot_intrinsics,
                ))
            },
            |(interiors, shared, _, root_text, text_intrinsics, plot_intrinsics),
             assigned,
             round| {
                root_text
                    .check_width(
                        f64::from(assigned.dimensions()?.0) - 32.0,
                        &FitScope::root(),
                        round.resources(),
                    )
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                let plan = canvas.build_canvas_plan(
                    Arc::clone(&themes),
                    assigned,
                    round,
                    &mut children,
                    shared,
                    None,
                    None,
                    text_intrinsics,
                    plot_intrinsics,
                    interiors,
                    false,
                    None,
                )?;
                children.finish_round(round)?;
                Ok(crate::layout::MeasuredLayout::new(assigned, plan))
            },
            |(_, _, resources, _, _, _), _, _, _| {
                let usage = (
                    resources.usage().unwrap().1,
                    resources.byte_usage().unwrap(),
                );
                resources.set_test_limits(u128::MAX, u64::MAX);
                Ok(usage)
            },
        )
        .map(|root| *root.plan())
    };
    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
    let usage = run(None).unwrap();
    assert_eq!(run(Some(usage)).unwrap(), usage);
    for limits in [(usage.0 - 1, usage.1), (usage.0, usage.1 - 1)] {
        assert!(run(Some(limits)).is_err());
    }
    assert_eq!(
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
        0
    );
}
