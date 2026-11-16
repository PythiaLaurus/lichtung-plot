use super::*;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one sparse Canvas witnesses the same occurrence inventory before and after color, guide and publication boundaries"
)]
fn canvas_registry_keeps_dense_occurrences_and_distinct_appearances_through_guides() {
    let source = |hue| {
        Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 2.0]),
        )
        .hue(crate::CategoryValues::new("cohort", [hue; 2]).unwrap())
        .source_rows(crate::SourceRows::new([9001, 70]).unwrap())
        .with_alpha(
            crate::alpha::AlphaTarget::Point,
            crate::alpha::AlphaEncoding::observations(
                crate::alpha::AlphaValues::new("confidence", [64, 255]).unwrap(),
            ),
        )
    };
    let palette = |colors| crate::QualitativePalette::fixed(colors).unwrap();
    let mut root = Theme::nature();
    root.color_scales = root
        .color_scales
        .clone()
        .qualitative(palette([
            crate::OpaqueColor::rgb(10, 20, 30),
            crate::OpaqueColor::rgb(40, 50, 60),
        ]))
        .unwrap();
    let mut child = Theme::publication();
    child.color_scales = child
        .color_scales
        .clone()
        .qualitative(palette([
            crate::OpaqueColor::rgb(70, 80, 90),
            crate::OpaqueColor::rgb(100, 110, 120),
        ]))
        .unwrap();
    let canvas = Canvas::grid(2, 3)
        .unwrap()
        .theme(root)
        .legend_policy(CanvasLegendPolicy::Shared)
        .panel_span(0, 0, 2, 1, source("first"))
        .unwrap()
        .text_panel(0, 1, TextPanel::new("Text owner", "Body").unwrap())
        .unwrap()
        .panel(1, 2, source("second").theme(child))
        .unwrap();
    let authored = canvas.placements.plot(5).unwrap().theme_spec().unwrap();
    let stages = crate::plots::scatter::STAGES.with(std::cell::Cell::get);
    crate::layout::plan_root(
        crate::layout::RootRequest::ExactSize {
            width: 1,
            height: 1,
        },
        |stage| {
            let operation = stage.operation();
            let (_, appearances) = resolve_appearances(
                &canvas.theme,
                &canvas.placements,
                &canvas.guide_text_layouts,
            )?;
            let (mut registry, topology) = canvas.prepare_interiors(stage, appearances)?;
            let slots = &topology.slots;
            assert_eq!(registry.panels.len(), 2);
            assert!(matches!(
                slots[1],
                Some(crate::orchestrator::root::CanvasAnchor::Text)
            ));
            assert!(slots[2].is_none() && slots[3].is_none() && slots[4].is_none());
            let ids = [0, 5].map(|anchor| match &slots[anchor] {
                Some(crate::orchestrator::root::CanvasAnchor::Plot(id)) => id.clone(),
                _ => panic!("Plot anchor"),
            });
            for (slot, id) in ids.iter().enumerate() {
                assert_eq!(id.slot, slot);
                let data = registry.data(id, &operation)?;
                assert_eq!(data.plot_id(), id);
                assert_eq!(
                    (data.height(), data.row_index(1), data.source_row(1)),
                    (2, 1, 70)
                );
                let panel = &registry.panels[id.slot];
                assert!(panel.scope.numeric_owner.scope.is_none());
                assert_eq!(
                    panel.scope.numeric_owner.canvas,
                    Some(if slot == 0 { (0, 0) } else { (1, 2) })
                );
                assert_eq!(
                    panel.scope.axis_numeric_owner.scope.as_deref(),
                    Some(if slot == 0 { "panel-0" } else { "panel-5" })
                );
            }
            let domains = canvas.prepare_color_domains(&registry, slots)?;
            for panel in &registry.panels {
                crate::layout::interior::guide::validate_categorical_scale(
                    panel.source,
                    std::slice::from_ref(panel),
                    &panel.appearance,
                    Some(&domains[0].1),
                )?;
            }
            assert!(canvas
                .prepare_continuous_shared_guide(&registry, slots, &operation)?
                .is_none());
            let shared = canvas
                .prepare_discrete_guides(&mut registry, slots, &domains, &operation)?
                .unwrap();
            for id in &ids {
                assert_eq!(registry.data(id, &operation)?.plot_id(), id);
            }
            let plots = registry.finish()?;
            for (slot, anchor) in [0, 5].into_iter().enumerate() {
                let panel = plot_at(&plots, slots, anchor).unwrap();
                assert_eq!(panel.scope.id, ids[slot]);
                let data = panel.data(&operation)?;
                assert_eq!(data.plot_id(), &panel.scope.id);
                assert_eq!(data.source_row(0), 9001);
                let numeric = panel.numeric_binding(&operation)?;
                assert!(numeric.operation.same_operation(&operation));
                assert_eq!(numeric.plot_id, Some(&panel.scope.id));
                assert!(std::ptr::eq(
                    numeric.owner,
                    std::ptr::from_ref(&panel.scope.numeric_owner)
                ));
                assert!(numeric.owner.scope.is_none());
                assert_eq!(
                    numeric.owner.canvas,
                    Some(if slot == 0 { (0, 0) } else { (1, 2) })
                );
                assert_eq!(
                    panel.scope.axis_numeric_owner.scope.as_deref(),
                    Some(if slot == 0 { "panel-0" } else { "panel-5" })
                );
                numeric.check_data(data)?;
                let other_data = plots[1 - slot].data(&operation)?;
                assert!(numeric
                    .operation
                    .same_operation(other_data.source_resources()));
                assert_ne!(numeric.plot_id, Some(other_data.plot_id()));
                assert!(matches!(
                    numeric.check_data(other_data),
                    Err(Error::Render(message))
                        if message == "row source belongs to a different source or operation"
                ));
                let alpha = &panel.guide.alpha.sources[0];
                assert_eq!(alpha.owner, panel.scope.address);
                assert_eq!(alpha.scope.as_deref(), panel.scope.render_name.as_deref());
                assert_eq!(
                    panel.appearance.color_scales.get_qualitative().colors()[slot].as_rgb(),
                    if slot == 0 {
                        (10, 20, 30)
                    } else {
                        (100, 110, 120)
                    }
                );
            }
            crate::layout::plan_root(
                crate::layout::RootRequest::ExactSize {
                    width: 1,
                    height: 1,
                },
                |foreign_stage| {
                    let foreign = foreign_stage.operation();
                    assert!(!foreign.same_operation(&operation));
                    for panel in &plots {
                        assert!(matches!(
                            panel.numeric_binding(&foreign),
                            Err(Error::Render(message))
                                if message == "row source belongs to a different source or operation"
                        ));
                    }
                    Ok(())
                },
                |(), assigned, _| {
                    Ok(crate::layout::MeasuredLayout::new(
                        assigned.assigned_size()?,
                        (),
                    ))
                },
                |(), _, (), _| Ok(()),
            )?;
            assert_eq!(shared.sections[0].entries().len(), 2);
            assert!(plots[0].source.theme_spec().is_none());
            assert!(std::ptr::eq(
                plots[1].source.theme_spec().unwrap(),
                authored
            ));
            assert_eq!(
                crate::plots::scatter::STAGES.with(std::cell::Cell::get)[0],
                stages[0] + 2
            );
            Ok(())
        },
        |(), assigned, _| {
            Ok(crate::layout::MeasuredLayout::new(
                assigned.assigned_size()?,
                (),
            ))
        },
        |(), _, (), _| Ok(()),
    )
    .unwrap();
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one composed callback lifecycle follows local cells, a merged Shared domain, exact powers and all frozen publication paths"
)]
fn numeric_functions_materialize_semantic_contexts_once_and_never_run_during_publication() {
    use crate::{NumericFormat, NumericIdentity, NumericOperand, NumericPurpose, NumericRole};
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    };
    let frozen = Arc::new(AtomicBool::new(false));
    let guide_calls = Arc::new(AtomicUsize::new(0));
    let cells = Arc::new(Mutex::new(Vec::new()));
    let guide = {
        let invocations = guide_calls.clone();
        let frozen = frozen.clone();
        NumericFormat::from_specifier(".0f")
            .unwrap()
            .set_function(move |context| {
                assert!(!frozen.load(Ordering::SeqCst));
                assert_eq!(context.role(), NumericRole::ContinuousColor);
                assert!(context.owner().is_shared());
                assert_eq!(context.domain().unwrap().data_limits(), Some((1.01, 3.2)));
                assert_eq!(context.purpose(), NumericPurpose::ActualSet);
                invocations.fetch_add(1, Ordering::SeqCst);
                Ok(context.format_builtin()?.explanation("Combined guide"))
            })
    };
    let cell = {
        let cells = cells.clone();
        let frozen = frozen.clone();
        NumericFormat::default().value_function(move |context| {
            assert!(!frozen.load(Ordering::SeqCst));
            let owner = context.set().owner().canvas_cell().unwrap();
            let NumericIdentity::HeatmapCell { row, column } = context.value().identity() else {
                panic!("raw cell identity")
            };
            assert_eq!(row, "raw_row");
            assert!(matches!(column, "raw_a" | "raw_b"));
            cells.lock().unwrap().push((owner, context.index()));
            Ok(format!("raw012:{}", context.format_builtin()?))
        })
    };
    let heatmap = |values| {
        Plot::heatmap(
            crate::CategoryValues::new("column", ["raw_a", "raw_b"]).unwrap(),
            crate::CategoryValues::new("row", ["raw_row", "raw_row"]).unwrap(),
            crate::NumericValues::new("color", values),
        )
        .color_label_format(guide.clone())
        .annotations(crate::HeatmapAnnotations::Values(cell.clone()))
        .value_text_layout(
            crate::TextLayout::default()
                .bounds(crate::TextBounds::new(Some(70.0), None, None).unwrap())
                .fit(
                    crate::TextFit::range(10.25, 14.25)
                        .unwrap()
                        .grouping(crate::TextFitGrouping::Uniform),
                ),
        )
    };
    let contexts = Arc::new(Mutex::new(std::collections::BTreeSet::new()));
    let powers = {
        let contexts = contexts.clone();
        let frozen = frozen.clone();
        NumericFormat::default().set_function(move |context| {
            assert!(!frozen.load(Ordering::SeqCst));
            assert_eq!(context.owner().canvas_cell(), Some((0, 2)));
            assert_eq!(context.owner().plot_kind(), Some(crate::PlotKind::Forest));
            let domain = context.domain().unwrap();
            assert_eq!(domain.data_limits(), Some((0.1, 10.0)));
            assert_eq!(domain.scale(), crate::AxisScale::Log10);
            let operands: Vec<_> = context
                .values()
                .iter()
                .map(|value| match (context.purpose(), value.operand()) {
                    (NumericPurpose::IntrinsicProbe, NumericOperand::PowerOfTen { exponent }) => {
                        format!("probe:{:x}", exponent.to_bits())
                    }
                    (
                        NumericPurpose::ActualSet,
                        NumericOperand::Multiplicative {
                            anchor,
                            ratio,
                            index,
                        },
                    ) => {
                        assert_eq!(
                            (anchor.to_bits(), ratio.to_bits()),
                            (1.0_f64.to_bits(), 2.0_f64.to_bits())
                        );
                        index.to_owned()
                    }
                    _ => panic!("retained mathematical operand"),
                })
                .collect();
            let (lower, upper) = domain.scale_limits();
            assert!(contexts.lock().unwrap().insert((
                context.purpose() as u8,
                (lower.to_bits(), upper.to_bits()),
                operands
            )));
            context.format_builtin()
        })
    };
    let forest = Plot::forest(
        crate::CategoryValues::new("term", ["ratio"]).unwrap(),
        crate::NumericValues::new("estimate", [1.0]),
        crate::Bounds::new(
            crate::NumericValues::new("lower", [0.2]),
            crate::NumericValues::new("upper", [5.0]),
        ),
        1.0,
    )
    .scale(crate::AxisScale::Log10)
    .x_limits(0.1, 10.0)
    .x_tick_generation(
        crate::TickGeneration::multiplicative(1.0, 2.0, crate::TickRequirement::Fixed).unwrap(),
    )
    .x_tick_format(powers)
    .x_tick_text_layout(
        crate::TextLayout::default().fit(
            crate::TextFit::range(10.25, 14.25)
                .unwrap()
                .grouping(crate::TextFitGrouping::Uniform),
        ),
    );
    let root = Canvas::grid(1, 3)
        .unwrap()
        .size(1800, 650)
        .legend_policy(CanvasLegendPolicy::Shared)
        .panel(0, 0, heatmap([1.01, 1.02]))
        .unwrap()
        .panel(0, 1, heatmap([3.1, 3.2]))
        .unwrap()
        .panel(0, 2, forest)
        .unwrap()
        .preflight_plan()
        .unwrap();
    assert_eq!(guide_calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        *cells.lock().unwrap(),
        [((0, 0), 0), ((0, 0), 1), ((0, 1), 0), ((0, 1), 1)]
    );
    assert!(contexts
        .lock()
        .unwrap()
        .iter()
        .any(|key| key.0 == NumericPurpose::ActualSet as u8));
    assert!(contexts
        .lock()
        .unwrap()
        .iter()
        .any(|key| key.0 == NumericPurpose::IntrinsicProbe as u8));
    assert_eq!(
        root.plan()
            .retained_text()
            .iter()
            .filter(|(_, text)| text.source == "Combined guide")
            .count(),
        1
    );
    frozen.store(true, Ordering::SeqCst);
    let counters = capture_counters();
    let mut backend = root.plan().take_svg_backend().unwrap();
    Canvas::render_preflighted(&root, &mut backend).unwrap();
    let scene = RenderedScene::new(1800, 650, backend.into_output()).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let output = crate::Output::new(scene)
        .svg_preserve_text(directory.path().join("selectable"))
        .unwrap()
        .svg(directory.path().join("outlined"))
        .unwrap()
        .png_preview(directory.path().join("preview"))
        .unwrap();
    assert_eq!((output.width(), output.height()), (1800, 650));
    assert_eq!(capture_counters(), counters);
    assert_eq!(guide_calls.load(Ordering::SeqCst), 1);
    let suppressed = NumericFormat::default()
        .set_function(|_| panic!("exact ticks suppress every numeric callback"));
    Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .x_tick_format(suppressed)
    .x_tick_generation(crate::TickGeneration::function(|_| {
        panic!("exact ticks suppress the locator")
    }))
    .x_ticks(vec![
        crate::AxisTick::new(0.0, "origin"),
        crate::AxisTick::new(1.0, "end"),
    ])
    .preflight()
    .unwrap();
}

#[test]
fn position_functions_consume_shared_views_once_and_freeze_without_optional_reuse() {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };
    let observed = Arc::new(Mutex::new(std::collections::BTreeSet::new()));
    let calls = observed.clone();
    let frozen = Arc::new(AtomicBool::new(false));
    let guard = frozen.clone();
    let generation = crate::TickGeneration::function(move |context| {
        assert!(!guard.load(Ordering::SeqCst));
        assert_eq!(
            context.domain().origin(),
            crate::NumericRangeOrigin::ParentResolved
        );
        let range = context.domain().data_limits().unwrap();
        assert!(range.0 <= 0.0 && range.1 >= 120.0);
        assert!(calls.lock().unwrap().insert((
            context.owner().canvas_cell().unwrap(),
            range.0.to_bits(),
            range.1.to_bits(),
            context.span().to_bits()
        )));
        Ok(vec![80.0, 20.0, 20.0, -1e6, 1e6].into())
    });
    let guard = frozen.clone();
    let format = crate::NumericFormat::default().set_function(move |context| {
        assert!(!guard.load(Ordering::SeqCst));
        context.format_builtin()
    });
    let plot = |range| {
        Plot::scatter(
            crate::NumericValues::new("x", range),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .x_tick_generation(generation.clone())
        .x_tick_format(format.clone())
    };
    let canvas = Canvas::grid(2, 2)
        .unwrap()
        .size(1600, 600)
        .panel(0, 0, plot([0.0, 100.0]))
        .unwrap()
        .panel_span(0, 1, 2, 1, plot([20.0, 120.0]))
        .unwrap()
        .x_range_group(vec![(0, 0), (0, 1)])
        .unwrap();
    let directory = tempfile::tempdir().unwrap();
    let mut previous = None;
    for disabled in [false, true] {
        frozen.store(false, Ordering::SeqCst);
        observed.lock().unwrap().clear();
        crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
        let result = canvas.preflight_plan();
        crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
        let root = result.unwrap();
        assert!(observed
            .lock()
            .unwrap()
            .iter()
            .any(|entry| entry.0 == (0, 0)));
        assert!(observed
            .lock()
            .unwrap()
            .iter()
            .any(|entry| entry.0 == (0, 1)));
        frozen.store(true, Ordering::SeqCst);
        let counters = capture_counters();
        let mut backend = root.plan().take_svg_backend().unwrap();
        Canvas::render_preflighted(&root, &mut backend).unwrap();
        let output =
            crate::Output::new(RenderedScene::new(1600, 600, backend.into_output()).unwrap())
                .svg_preserve_text(directory.path().join("text"))
                .unwrap();
        let svg = std::fs::read(directory.path().join("text.svg")).unwrap();
        if let Some(previous) = previous {
            assert_eq!(previous, svg);
        }
        previous = Some(svg);
        if !disabled {
            output
                .svg(directory.path().join("outline"))
                .unwrap()
                .png_preview(directory.path().join("preview"))
                .unwrap();
        }
        assert_eq!(capture_counters(), counters);
    }
    Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .x_ticks(vec![
        crate::AxisTick::new(0.0, "start"),
        crate::AxisTick::new(1.0, "end"),
    ])
    .x_tick_generation(crate::TickGeneration::function(|_| {
        panic!("replaced locator")
    }))
    .preflight()
    .unwrap();
}

pub(super) fn capture_counters() -> ([usize; 6], [usize; 6]) {
    (
        [
            crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
            crate::layout::Projection::test_projection_calls(),
            crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
            crate::plots::shared_x::AXIS_PREPARATIONS.with(std::cell::Cell::get),
            crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
            crate::orchestrator::SELECTED_INTERIOR_MEASUREMENTS.with(std::cell::Cell::get),
        ],
        crate::font::canonical::planning_counts(),
    )
}
