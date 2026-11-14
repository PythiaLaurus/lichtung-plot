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
    let authored = canvas
        .placements
        .plot(5)
        .unwrap()
        .description()
        .theme_spec()
        .unwrap();
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
            let (mut registry, slots, _) = canvas.prepare_interiors(stage, appearances)?;
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
            let domains = canvas.prepare_color_domains(&registry, &slots)?;
            for panel in &registry.panels {
                crate::layout::interior::guide::validate_categorical_scale(
                    panel.source,
                    std::slice::from_ref(panel),
                    &panel.appearance,
                    Some(&domains[0].1),
                )?;
            }
            assert!(canvas
                .prepare_continuous_shared_guide(&registry, &slots, &operation)?
                .is_none());
            let shared = canvas
                .prepare_discrete_guides(&mut registry, &slots, &domains, &operation)?
                .unwrap();
            for id in &ids {
                registry.data(id, &operation)?;
            }
            let plots = registry.finish()?;
            for (slot, anchor) in [0, 5].into_iter().enumerate() {
                let panel = plot_at(&plots, &slots, anchor).unwrap();
                assert_eq!(panel.scope.id, ids[slot]);
                assert_eq!(panel.data(&operation)?.source_row(0), 9001);
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
fn fixed_guide_use_keeps_shared_contributions_and_authored_visibility_distinct() {
    use crate::decorations::legend::discrete::scoped::GuideUse;
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [1.0, 2.0]),
    )
    .hue(crate::CategoryValues::new("group", ["first", "second"]).unwrap());
    for policy in [
        CanvasLegendPolicy::PerPlot,
        CanvasLegendPolicy::Shared,
        CanvasLegendPolicy::Hidden,
    ] {
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .size(1000, 500)
            .legend_policy(policy)
            .panel(0, 0, plot.clone())
            .unwrap()
            .panel(0, 1, plot.clone().legend(crate::LegendPolicy::Hidden))
            .unwrap();
        canvas
            .build_preflight_plan(|prepared, _| {
                for index in 0..2 {
                    let registered = prepared.plot(index).unwrap();
                    let expected = if index == 1 || policy == CanvasLegendPolicy::Hidden {
                        GuideUse::Hidden
                    } else if policy == CanvasLegendPolicy::Shared {
                        GuideUse::SharedSuppressed
                    } else {
                        GuideUse::Local
                    };
                    assert_eq!(registered.policy.discrete_guide_use, expected);
                    assert_eq!(registered.policy.continuous_guide_use, expected);
                    assert_eq!(
                        registered.source.legend_policy_spec() == crate::LegendPolicy::Hidden,
                        index == 1
                    );
                    assert_eq!(
                        registered.guide.descriptor.is_some(),
                        expected != GuideUse::Hidden
                    );
                    assert_eq!(
                        registered.policy.continuous_materialization,
                        if policy == CanvasLegendPolicy::Hidden {
                            crate::orchestrator::registration::ContinuousMaterialization::Suppressed
                        } else {
                            crate::orchestrator::registration::ContinuousMaterialization::Local
                        }
                    );
                }
            })
            .unwrap();
    }
}

#[test]
fn effective_label_policy_is_borrowed_without_rewriting_authored_child_labels() {
    let labels = Labels::raw()
        .alias("raw_x", "Canvas X")
        .alias("raw_y", "Canvas Y")
        .alias("raw_series", "Canvas series")
        .alias("raw_group", "Canvas member")
        .alias("raw_confidence", "Canvas confidence")
        .alias("raw_low", "Canvas low")
        .alias("raw_high", "Canvas high")
        .alias("raw_value", "Canvas value");
    let child = Labels::raw()
        .alias("raw_x", "Child X")
        .alias("raw_value", "Child value");
    let scatter = Plot::scatter(
        crate::NumericValues::new("raw_x", [0.0, 1.0]),
        crate::NumericValues::new("raw_y", [1.0, 2.0]),
    )
    .hue(crate::CategoryValues::new("raw_series", ["raw_group"; 2]).unwrap())
    .labels(child.clone())
    .with_alpha(
        crate::alpha::AlphaTarget::Point,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::categorical(
                crate::CategoryValues::new("raw_confidence", ["raw_low", "raw_high"]).unwrap(),
                |value| if value == "raw_low" { 64 } else { 255 },
            )
            .unwrap(),
        ),
    );
    let heatmap = Plot::heatmap(
        crate::CategoryValues::new("raw_x", ["first", "second"]).unwrap(),
        crate::CategoryValues::new("raw_y", ["row"; 2]).unwrap(),
        crate::NumericValues::new("raw_value", [1.0, 2.0]),
    )
    .labels(child);
    let canvas = Canvas::grid(1, 2)
        .unwrap()
        .size(1200, 600)
        .legend_policy(CanvasLegendPolicy::PerPlot)
        .labels(labels)
        .panel(0, 0, scatter)
        .unwrap()
        .panel(0, 1, heatmap)
        .unwrap();
    let root = canvas
        .build_preflight_plan(|prepared, _| {
            for registered in &prepared.plots {
                assert_eq!(
                    registered.policy.labels.resolve("raw_x").unwrap(),
                    "Canvas X"
                );
                assert_eq!(
                    registered.source.labels_policy().resolve("raw_x").unwrap(),
                    "Child X"
                );
            }
        })
        .unwrap();
    let text = root.plan().retained_text();
    for expected in [
        "Canvas X",
        "Canvas Y",
        "Canvas series",
        "Canvas member",
        "Canvas confidence (observation point alpha)",
        "Canvas low",
        "Canvas high",
        "Canvas value",
    ] {
        assert!(
            text.iter().any(|(_, plan)| plan.source == expected),
            "{expected}"
        );
    }
    assert!(!text
        .iter()
        .any(|(_, plan)| plan.source.starts_with("Child ")));
}

#[test]
fn shared_interaction_group_colors_do_not_become_local_scientific_orders() {
    let source = |group| {
        Plot::interaction(
            crate::CategoryValues::new("condition", ["before", "after"]).unwrap(),
            crate::NumericValues::new("estimate", [1.0, 2.0]),
            crate::CategoryValues::new("cohort", [group; 2]).unwrap(),
            crate::Bounds::new(
                crate::NumericValues::new("lower", [0.5, 1.5]),
                crate::NumericValues::new("upper", [1.5, 2.5]),
            ),
        )
    };
    for policy in [CanvasLegendPolicy::Shared, CanvasLegendPolicy::PerPlot] {
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .size(1000, 500)
            .legend_policy(policy)
            .panel(0, 0, source("first"))
            .unwrap()
            .panel(0, 1, source("second"))
            .unwrap();
        let mut second_color = (0, 0, 0);
        let root = canvas
            .build_preflight_plan(|prepared, _| {
                second_color = prepared
                    .plot(1)
                    .unwrap()
                    .appearance
                    .color_scales
                    .get_qualitative()
                    .colors()[1]
                    .as_rgb();
                for (index, raw) in ["first", "second"].into_iter().enumerate() {
                    let registered = prepared.plot(index).unwrap();
                    let groups =
                        crate::orchestrator::registration::composed_group_levels(Some(registered));
                    if policy == CanvasLegendPolicy::Shared {
                        assert_eq!(
                            groups.unwrap().values().collect::<Vec<_>>(),
                            ["first", "second"]
                        );
                        assert!(registered.parent_orders.groups.is_none());
                    } else {
                        assert!(groups.is_none());
                    }
                    let crate::orchestrator::registration::PreparedFamily::Interaction(marks) =
                        &registered.family
                    else {
                        panic!("prepared Interaction")
                    };
                    assert_eq!(marks.color_keys().collect::<Vec<_>>(), [raw]);
                }
            })
            .unwrap();
        let mut backend = root.plan().take_svg_backend().unwrap();
        Canvas::render_preflighted(&root, &mut backend).unwrap();
        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        let second = format!(
            "fill=\"#{:02X}{:02X}{:02X}\"",
            second_color.0, second_color.1, second_color.2
        );
        assert_eq!(svg.contains(&second), policy == CanvasLegendPolicy::Shared);
    }
}

#[test]
fn shared_bar_palette_preserves_raw_color_identity_without_guide_contributions() {
    let source = |series| {
        Plot::bar(
            crate::CategoryValues::new("category", ["one"]).unwrap(),
            crate::BarInput::estimates(crate::NumericValues::new("value", [1.0])),
        )
        .series(crate::CategoryValues::new("cohort", [series]).unwrap())
        .legend(crate::LegendPolicy::Hidden)
    };
    let canvas = Canvas::grid(1, 2)
        .unwrap()
        .size(1000, 500)
        .legend_policy(CanvasLegendPolicy::Shared)
        .panel(0, 0, source("first"))
        .unwrap()
        .panel(0, 1, source("second"))
        .unwrap();
    let mut second_color = (0, 0, 0);
    let root = canvas
        .build_preflight_plan(|prepared, _| {
            assert!(prepared.shared_guide.is_none());
            let first = prepared
                .plot(0)
                .unwrap()
                .categorical
                .as_ref()
                .unwrap()
                .parent
                .as_ref()
                .unwrap();
            assert_eq!(first.values().collect::<Vec<_>>(), ["first", "second"]);
            for (index, raw) in ["first", "second"].into_iter().enumerate() {
                let registered = prepared.plot(index).unwrap();
                let binding = registered.categorical.as_ref().unwrap();
                assert_eq!(binding.local_values().collect::<Vec<_>>(), [raw]);
                assert!(Arc::ptr_eq(first, binding.parent.as_ref().unwrap()));
                assert!(Arc::ptr_eq(
                    binding.parent.as_ref().unwrap(),
                    registered.parent_orders.hues.as_ref().unwrap()
                ));
                let crate::orchestrator::registration::PreparedFamily::Bars { marks, .. } =
                    &registered.family
                else {
                    panic!("prepared Bars")
                };
                let identity = marks.bars[0].hue_key.as_ref().unwrap().as_ref();
                assert_eq!(identity.field, "cohort");
                assert_eq!(identity.value, raw);
                assert!(registered.guide.descriptor.is_none());
                assert!(registered.guide.alpha.sources.is_empty());
            }
            second_color = prepared
                .plot(1)
                .unwrap()
                .appearance
                .color_scales
                .get_qualitative()
                .colors()[1]
                .as_rgb();
        })
        .unwrap();
    let mut backend = root.plan().take_svg_backend().unwrap();
    Canvas::render_preflighted(&root, &mut backend).unwrap();
    let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
    assert!(svg.contains(&format!(
        "fill=\"#{:02X}{:02X}{:02X}\"",
        second_color.0, second_color.1, second_color.2,
    )));
}

#[test]
fn hidden_native_materialization_distinguishes_authored_visibility_from_canvas_policy() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let source = Plot::heatmap(
        crate::CategoryValues::new("column", ["C", "C"]).unwrap(),
        crate::CategoryValues::new("row", ["A", "B"]).unwrap(),
        crate::NumericValues::new("value", [1.0, 2.0]),
    )
    .legend(crate::LegendPolicy::Hidden)
    .color_label_format(
        crate::NumericFormat::default().set_function(move |context| {
            assert_eq!(context.purpose(), crate::NumericPurpose::ActualSet);
            observed.fetch_add(1, Ordering::SeqCst);
            context.format_builtin()
        }),
    )
    .size(800, 600);
    source.preflight().unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    Canvas::grid(1, 1)
        .unwrap()
        .size(800, 600)
        .legend_policy(CanvasLegendPolicy::Hidden)
        .panel(0, 0, source)
        .unwrap()
        .build_preflight_plan(|prepared, _| {
            let registered = prepared.plot(0).unwrap();
            assert_eq!(
                registered.policy.continuous_materialization,
                crate::orchestrator::registration::ContinuousMaterialization::Suppressed
            );
            let crate::orchestrator::registration::PreparedFamily::Heatmap(marks) =
                &registered.family
            else {
                panic!("prepared Heatmap family")
            };
            assert!(marks
                .guide_descriptor(
                    "value",
                    &crate::text::TextRequestContext::new(&registered.appearance, &prepared.text)
                )
                .unwrap()
                .is_none());
        })
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn shared_native_numeric_binding_reduces_the_original_owners_once_before_capture() {
    let owners = Arc::new(std::sync::Mutex::new(Vec::new()));
    let observed = owners.clone();
    let format = crate::NumericFormat::default().set_function(move |context| {
        assert_eq!(context.role(), crate::NumericRole::ContinuousColor);
        assert_eq!(context.purpose(), crate::NumericPurpose::ActualSet);
        assert_eq!(context.domain().unwrap().data_limits(), Some((1.0, 4.0)));
        observed.lock().unwrap().push(context.owner().0.try_copy()?);
        context.format_builtin()
    });
    let heatmap = Plot::heatmap(
        crate::CategoryValues::new("column", ["C", "C"]).unwrap(),
        crate::CategoryValues::new("row", ["A", "B"]).unwrap(),
        crate::NumericValues::new("value", [1.0, 2.0]),
    )
    .color_label_format(format.clone());
    let swarm = Plot::beeswarm(
        crate::CategoryValues::new("row", ["A", "B"]).unwrap(),
        crate::NumericValues::new("x", [1.0, 2.0]),
        crate::NumericValues::new("value", [3.0, 4.0]),
    )
    .color_label_format(format);
    let root = Canvas::grid(1, 2)
        .unwrap()
        .size(1400, 600)
        .legend_policy(CanvasLegendPolicy::Shared)
        .panel(0, 0, heatmap)
        .unwrap()
        .panel(0, 1, swarm)
        .unwrap()
        .preflight_plan()
        .unwrap();
    {
        let owners = owners.lock().unwrap();
        assert_eq!(
            owners.len(),
            1,
            "Shared materializes one complete set before fonts"
        );
        let owner = &owners[0];
        assert!(
            owner.kind.is_none(),
            "the original contributors are heterogeneous"
        );
        assert!(owner.canvas.is_none() && owner.facet.is_none() && owner.shared);
        assert_eq!(owner.scope.as_deref(), Some("shared continuous value"));
    }
    let mut backend = root.plan().take_svg_backend().unwrap();
    Canvas::render_preflighted(&root, &mut backend).unwrap();
    assert_eq!(
        owners.lock().unwrap().len(),
        1,
        "replay never calls the formatter"
    );
}

#[test]
fn authored_display_slots_preserve_common_source_and_real_blank_line_metrics() {
    const SOURCE: &str = "\n  Alpha \r\n\r\nBeta  \n";
    let plot = Plot::line(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [1.0, 0.0]),
    )
    .title(SOURCE)
    .subtitle(SOURCE)
    .caption(SOURCE)
    .x_label(SOURCE)
    .y_label(SOURCE)
    .hue(crate::CategoryValues::new("cohort", ["single"; 2]).unwrap())
    .labels(Labels::raw().alias("cohort", "").alias("single", SOURCE))
    .point_layer(
        crate::PointLayer::new(
            crate::NumericValues::new("selected x", [0.5]),
            crate::NumericValues::new("selected y", [0.5]),
        )
        .labels(crate::CategoryValues::new("note", [SOURCE]).unwrap()),
    )
    .legend(crate::LegendPolicy::Visible(
        crate::LegendPosition::OutsideRight,
    ))
    .text_annotation(
        crate::PlotTextAnnotation::new(crate::DataAreaCorner::TopLeft, SOURCE).unwrap(),
    );
    let canvas = Canvas::grid(2, 1)
        .unwrap()
        .size(1200, 2000)
        .title(SOURCE)
        .subtitle("")
        .caption(SOURCE)
        .column_header(0, 1, SOURCE)
        .unwrap()
        .row_header(0, 2, SOURCE)
        .unwrap()
        .row_title(1, SOURCE)
        .unwrap()
        .panel(0, 0, plot)
        .unwrap()
        .text_panel(1, 0, TextPanel::new(SOURCE, SOURCE).unwrap())
        .unwrap();
    let root = canvas.preflight_plan().unwrap();
    let retained = root.plan().retained_text();
    let blocks: Vec<_> = retained
        .iter()
        .filter(|(_, text)| text.source == SOURCE)
        .collect();
    // Six Plot slots, a point label, two root prose slots, three headers,
    // two TextPanel slots and a guide entry.
    assert_eq!(blocks.len(), 15);
    for (_, text) in blocks {
        assert_eq!(text.line_texts(), ["", "  Alpha ", "", "Beta  ", ""]);
        assert!(text.line_pitch_quanta > 0);
        assert!(text.height_quanta > 4 * text.line_pitch_quanta);
        for lines in text.lines.windows(2) {
            assert_eq!(
                lines[1].translation_y_quanta - lines[0].translation_y_quanta,
                text.line_pitch_quanta
            );
        }
        for index in [0, 2, 4] {
            assert!(text.lines[index].runs.is_empty());
        }
    }
    let empty: Vec<_> = retained
        .iter()
        .filter(|(_, text)| text.source.is_empty())
        .collect();
    assert_eq!(
        empty.len(),
        2,
        "the explicit root subtitle and guide title must survive capture"
    );
    for (_, text) in empty {
        assert_eq!(text.line_texts(), [""]);
        assert!(text.height_quanta > 0);
    }
    let counters = capture_counters();
    let mut backend = root.plan().take_svg_backend().unwrap();
    Canvas::render_preflighted(&root, &mut backend).unwrap();
    assert_eq!(capture_counters(), counters);
}

#[test]
fn native_category_selection_uses_final_orders_and_skips_only_unselected_glyphs() {
    let unavailable = "\u{10ffff}";
    let plot = Plot::heatmap(
        crate::CategoryValues::new("column", ["C0", unavailable, "C1", "C2"].repeat(3)).unwrap(),
        crate::CategoryValues::new(
            "row",
            ["R0", unavailable, "R2"]
                .into_iter()
                .flat_map(|row| std::iter::repeat_n(row, 4)),
        )
        .unwrap(),
        crate::NumericValues::new("value", [1.0; 12]),
    )
    .labels(Labels::raw())
    .x_category_order(crate::CategoryOrder::explicit([
        unavailable,
        "C0",
        "C1",
        "C2",
    ]))
    .x_category_labels(
        crate::CategoryLabelSelection::every(2)
            .unwrap()
            .offset(1)
            .unwrap(),
    )
    .y_category_labels(crate::CategoryLabelSelection::every(2).unwrap());
    let canvas = Canvas::grid(1, 1)
        .unwrap()
        .size(800, 600)
        .legend_policy(CanvasLegendPolicy::Hidden)
        .panel(0, 0, plot.clone())
        .unwrap();
    let root = canvas.preflight_plan().unwrap();
    let text = root.plan().retained_text();
    for source in ["C0", "C2", "R0", "R2"] {
        assert_eq!(
            text.iter()
                .filter(|(_, plan)| plan.source == source)
                .count(),
            1
        );
    }
    assert!(!text
        .iter()
        .any(|(_, plan)| plan.source == unavailable || plan.source == "C1"));
    let counters = capture_counters();
    let mut backend = root.plan().take_svg_backend().unwrap();
    Canvas::render_preflighted(&root, &mut backend).unwrap();
    let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
    assert_eq!(svg.matches("<polygon ").count(), 12);
    assert_eq!(capture_counters(), counters);
    let used = plot.text_annotation(
        crate::PlotTextAnnotation::new(crate::DataAreaCorner::TopLeft, unavailable).unwrap(),
    );
    let error = Canvas::grid(1, 1)
        .unwrap()
        .size(800, 600)
        .legend_policy(CanvasLegendPolicy::Hidden)
        .panel(0, 0, used)
        .unwrap()
        .preflight()
        .unwrap_err();
    assert!(error.to_string().contains("U+10FFFF"), "{error}");
}

#[test]
fn sparse_native_rows_preserve_packed_points_and_waterfall_sources() {
    let selection = crate::CategoryLabelSelection::every(2).unwrap();
    let swarm = Plot::beeswarm(
        crate::CategoryValues::new("row", ["R0", "R0", "\u{10ffff}", "\u{10ffff}", "R2", "R2"])
            .unwrap(),
        crate::NumericValues::new("x", [1.0, 1.1, 2.0, 2.1, 3.0, 3.1]),
        crate::NumericValues::new("color", [1.0; 6]),
    )
    .labels(Labels::raw())
    .y_category_labels(selection.clone());
    let waterfall = Plot::waterfall(
        crate::CategoryValues::new("row", ["R0", "\u{10ffff}", "R2"]).unwrap(),
        crate::NumericValues::new("delta", [1.0, -0.5, 1.0]),
        2.0,
    )
    .labels(Labels::raw())
    .y_category_labels(selection);
    let root = Canvas::grid(1, 2)
        .unwrap()
        .size(1200, 600)
        .legend_policy(CanvasLegendPolicy::Hidden)
        .panel(0, 0, swarm)
        .unwrap()
        .panel(0, 1, waterfall)
        .unwrap()
        .preflight_plan()
        .unwrap();
    let text = root.plan().retained_text();
    for label in ["R0", "R2"] {
        assert_eq!(
            text.iter().filter(|(_, plan)| plan.source == label).count(),
            2
        );
    }
    let mut backend = root.plan().take_svg_backend().unwrap();
    Canvas::render_preflighted(&root, &mut backend).unwrap();
    let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
    assert_eq!(svg.matches("<circle ").count(), 6);
    assert_eq!(svg.matches("<polygon ").count(), 3);
}

#[test]
fn authored_headers_use_independent_fonts_alignment_and_selected_band_padding() {
    use crate::{FontFamily, FontStack, HeaderSpacing, Text, TextAlignment};
    let mut theme = Theme::publication();
    let role = theme.typography.composition.composition_header.clone();
    theme.typography.composition.composition_header = role
        .clone()
        .stack(FontStack::new([FontFamily::named("Unused Canvas header font").unwrap()]).unwrap())
        .unwrap();
    let one = crate::RelativeSpacing::ratio(1, 1).unwrap();
    theme.composition = theme
        .composition
        .headers(HeaderSpacing::new(one.clone(), one.clone(), one.clone(), one).unwrap())
        .unwrap();
    let column = Text::new("Column heading\nshort")
        .style(role.clone().font_size(25.0).unwrap())
        .alignment(TextAlignment::Start);
    let row = Text::new("Row heading\nshort")
        .style(role.clone().font_size(17.0).unwrap())
        .alignment(TextAlignment::End);
    let title = Text::new("Across the row\nshort")
        .style(role.font_size(21.0).unwrap())
        .alignment(TextAlignment::Center);
    let canvas = Canvas::grid(1, 2)
        .unwrap()
        .width(650)
        .theme(theme)
        .column_header(0, 2, column)
        .unwrap()
        .row_header(0, 1, row)
        .unwrap()
        .row_title(0, title)
        .unwrap()
        .text_panel(0, 0, crate::TextPanel::new("Left", "Body").unwrap())
        .unwrap()
        .text_panel(0, 1, crate::TextPanel::new("Right", "Body").unwrap())
        .unwrap();
    canvas
        .build_preflight_plan(|prepared, _| {
            let outer = PixelRect::new(0.0, 650.0, 0.0, 500.0);
            let plan = crate::layout::fit::measured(tracks::allocate(
                outer,
                &canvas.placements,
                &[None, None],
                prepared,
                &prepared.operation,
                &prepared.text_panel_intrinsics,
                &prepared.plot_titles,
                &canvas.column_headers,
                &canvas.row_headers,
                &canvas.row_titles,
                &[],
                &prepared.root_request(),
                &FitScope::root(),
                None,
            ))
            .unwrap();
            let column = &plan.headers[0].text.plan;
            let row = &plan.headers[1].text.plan;
            let title = &plan.row_titles[0].text.plan;
            for (text, size, alignment) in [
                (column, 25, crate::layout::PhysicalTextAlignment::Start),
                (row, 17, crate::layout::PhysicalTextAlignment::End),
                (title, 21, crate::layout::PhysicalTextAlignment::Center),
            ] {
                assert_eq!(text.style.font_size.to_bits(), f64::from(size).to_bits());
                assert_eq!(text.style.alignment, alignment);
                assert!(text.style.exact_override);
                assert_eq!(text.lines.len(), 2);
            }
            let cell = plan.owners[0].unwrap();
            let quanta = crate::layout::kernel::nearest_spatial_quanta;
            assert_eq!(
                quanta(cell.x0 - outer.x0).unwrap(),
                row.width_quanta + 17 * 1024
            );
            assert_eq!(
                quanta(outer.y1 - cell.y1).unwrap(),
                column.height_quanta + 25 * 1024 + title.height_quanta + 21 * 1024,
            );
            assert_eq!(
                quanta(cell.y1).unwrap(),
                quanta(plan.owners[1].unwrap().y1).unwrap()
            );
            assert_eq!(
                plan.headers[0].text.placement.anchor,
                crate::render::backend::TextAnchor::Start
            );
            assert_eq!(
                plan.headers[1].text.placement.anchor,
                crate::render::backend::TextAnchor::End
            );
        })
        .unwrap();
}

#[test]
fn spanning_owner_uses_declared_header_rows_and_one_prepared_source_per_anchor() {
    CANVAS_ASSIGNED_PANEL_EVIDENCE.with(|evidence| evidence.borrow_mut().clear());
    let plot = || {
        Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
    };
    let canvas = Canvas::grid(2, 2)
        .unwrap()
        .max_width(1001)
        .panel_span(0, 0, 2, 1, plot())
        .unwrap()
        .panel(0, 1, plot())
        .unwrap()
        .panel(1, 1, plot())
        .unwrap()
        .row_header(0, 1, "Upper")
        .unwrap()
        .row_header(1, 1, "Lower")
        .unwrap();
    let root = canvas
        .build_preflight_plan(|prepared, _| {
            assert!(prepared.plot(2).is_none());
            assert_eq!(prepared.plots.len(), 3);
            assert_eq!(prepared.plot_intrinsics.len(), 3);
            for (slot, anchor) in [0, 1, 3].into_iter().enumerate() {
                let registered = prepared.plot(anchor).unwrap();
                assert!(std::ptr::eq(registered, &raw const prepared.plots[slot]));
                assert_eq!(registered.data(&prepared.operation).unwrap().height(), 2);
                assert_eq!(registered.scope.id.slot, slot);
                assert_eq!(
                    registered.scope.address,
                    crate::layout::ComponentAddress::root()
                        .child(
                            crate::layout::AddressRole::CanvasCell,
                            u32::try_from(anchor).unwrap()
                        )
                        .unwrap()
                );
            }
        })
        .unwrap();
    assert!(root.dimensions().0 <= 1001);
    let evidence = CANVAS_ASSIGNED_PANEL_EVIDENCE.with(|evidence| evidence.borrow().clone());
    let assignments = &evidence[evidence.len() - 3..];
    let tall = assignments[0].0;
    let top = assignments[1].0;
    let bottom = assignments[2].0;
    for assignment in [tall, top, bottom] {
        assert!(assignment.x0 >= 0.0 && assignment.x1 <= f64::from(root.dimensions().0));
        assert!(assignment.y0 >= 0.0 && assignment.y1 <= f64::from(root.dimensions().1));
    }
    assert_eq!(tall.y1.to_bits(), top.y1.to_bits());
    assert_eq!(tall.y0.to_bits(), bottom.y0.to_bits());
    assert!(tall.height() > top.height() + bottom.height());
    let counters = capture_counters();
    let mut backend = root.plan().take_svg_backend().unwrap();
    Canvas::render_preflighted(&root, &mut backend).unwrap();
    assert_eq!(capture_counters(), counters);
    let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
    let y = |source: &str| -> f64 {
        let line = svg
            .lines()
            .find(|line| line.contains(&format!(">{source}<")))
            .unwrap();
        line.split(" y=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .parse()
            .unwrap()
    };
    assert!(y("Upper") < y("Lower"));
    let invalid = Canvas::grid(2, 2)
        .unwrap()
        .panel_span(0, 0, 2, 1, plot())
        .unwrap()
        .panel(0, 1, plot())
        .unwrap()
        .panel(1, 1, plot())
        .unwrap();
    assert!(invalid
        .row_title(1, "Cuts through span")
        .unwrap()
        .preflight()
        .is_err());
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one composed root witnesses content edits, whole resets, local style census and independent parent precedence"
)]
fn authored_root_prose_preserves_edits_and_resolves_whole_request_precedence() {
    use crate::layout::PhysicalTextAlignment::{Center, End};
    use crate::{Color, FontFamily, FontStack, Text, TextAlignment, TextLayout, TextWeight};
    let base = Theme::publication();
    let title_style = base
        .typography()
        .get_root()
        .get_title()
        .clone()
        .font_size(27.0)
        .unwrap()
        .weight(TextWeight::Bold)
        .unwrap()
        .color(Color::rgb(77, 33, 99))
        .unwrap();
    let caption_style = base
        .typography()
        .get_root()
        .get_caption()
        .clone()
        .font_size(19.0)
        .unwrap()
        .weight(TextWeight::Bold)
        .unwrap();
    let unused = base
        .typography()
        .get_root()
        .get_title()
        .clone()
        .stack(
            FontStack::new([FontFamily::named("Unused inherited prose title font").unwrap()])
                .unwrap(),
        )
        .unwrap();
    let typography = base
        .typography()
        .clone()
        .root(base.typography().get_root().clone().title(unused).unwrap())
        .unwrap();
    let zero = crate::RelativeSpacing::ratio(0, 1).unwrap();
    let one = crate::RelativeSpacing::ratio(1, 1).unwrap();
    let composition = base
        .composition()
        .clone()
        .root_padding(
            crate::RootPadding::new(zero.clone(), zero.clone(), zero.clone(), zero).unwrap(),
        )
        .unwrap()
        .root_title_block(crate::RootTitleSpacing::new(one.clone(), one.clone(), one).unwrap())
        .unwrap();
    let theme = base
        .customize()
        .typography(typography)
        .composition(composition)
        .build()
        .unwrap();
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [1.0, 0.0]),
    )
    .title_alignment(TextAlignment::End)
    .title(Text::new("\u{10ffff}").style(title_style.clone()))
    .title(Text::new("reset").layout(TextLayout::default()))
    .title("Child heading\nshort")
    .subtitle_alignment(TextAlignment::Start)
    .subtitle(Text::new("Child subtitle").alignment(TextAlignment::End))
    .caption(
        Text::new("old caption")
            .style(caption_style.clone())
            .alignment(TextAlignment::Center),
    )
    .caption("Child caption");
    let canvas = Canvas::grid(1, 1)
        .unwrap()
        .width(650)
        .theme(theme)
        .panel_title_alignment(TextAlignment::Center)
        .panel_subtitle_alignment(TextAlignment::Start)
        .title(
            Text::new("discarded")
                .style(title_style.clone())
                .alignment(TextAlignment::End),
        )
        .title("Canvas heading\nshort")
        .subtitle("Canvas subtitle")
        .caption(
            Text::new("Canvas caption")
                .style(caption_style.clone())
                .alignment(TextAlignment::Center),
        )
        .panel(0, 0, plot)
        .unwrap();
    let root = canvas
        .build_preflight_plan(|prepared, replay| {
            assert!(prepared
                .text_census
                .requirements
                .iter()
                .all(|text| !text.source.contains('\u{10ffff}')));
            let retained = replay.retained_text();
            for (source, alignment, exact) in [
                ("Canvas heading\nshort", End, Some(&title_style)),
                ("Canvas subtitle", End, None),
                ("Canvas caption", Center, Some(&caption_style)),
                ("Child heading\nshort", Center, Some(&title_style)),
                ("Child subtitle", End, None),
                ("Child caption", Center, Some(&caption_style)),
            ] {
                let text = retained
                    .iter()
                    .find(|(_, text)| text.source == source)
                    .unwrap()
                    .1;
                assert_eq!(text.style.alignment, alignment, "{source}");
                assert_eq!(text.style.exact_override, exact.is_some());
                if let Some(style) = exact {
                    assert_eq!(
                        text.style.font_size.to_bits(),
                        style.get_font_size().to_bits()
                    );
                    assert_eq!(text.style.paint, style.get_color());
                    assert_eq!(text.style.weight, style.get_weight());
                    assert_eq!(&text.style.stack, style.get_stack());
                    let requirement = prepared
                        .text_census
                        .requirements
                        .iter()
                        .find(|text| text.source == source)
                        .unwrap();
                    assert_eq!(requirement.weight, style.get_weight());
                }
            }
            let frame = crate::layout::RootFramePlan::measure(
                (650, 900),
                &prepared.root_request(),
                canvas.prose(),
                crate::decorations::title_block::plan::RootAlignments::resolve(
                    &prepared.root_appearance,
                    canvas.prose_alignments(),
                    [None; 2],
                ),
                (1.0, 1.0),
            )
            .unwrap();
            let prose: Vec<_> = frame
                .root_text
                .placed_text()
                .map(|(_, text)| text)
                .collect();
            let quanta = crate::layout::kernel::nearest_spatial_quanta;
            assert_eq!(
                quanta(frame.insets.top).unwrap(),
                prose[0].height_quanta + prose[1].height_quanta + 2 * 27 * 1024
            );
            assert_eq!(
                quanta(frame.insets.bottom).unwrap(),
                prose[2].height_quanta + 19 * 1024
            );
        })
        .unwrap();
    let counters = capture_counters();
    let mut backend = root.plan().take_svg_backend().unwrap();
    Canvas::render_preflighted(&root, &mut backend).unwrap();
    let (width, height) = root.dimensions();
    let scene = RenderedScene::new(width, height, backend.into_output()).unwrap();
    assert_eq!(
        scene.svg_with_text_as_paths().unwrap(),
        scene.svg_with_text_as_paths().unwrap()
    );
    assert_eq!(capture_counters(), counters);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one authored TextPanel contract follows selected fonts, actual role-relative geometry, reflow and frozen replay"
)]
fn authored_text_panel_resolves_independent_styles_alignment_and_local_metrics() {
    use crate::layout::PhysicalTextAlignment;
    use crate::{Color, FontFamily, FontStack, GenericFontFamily, Text, TextAlignment, TextWeight};
    const TITLE: &str = "Heading\nshort";
    const BODY: &str = "A complete paragraph with enough distinct words to need additional lines when the assigned content width becomes narrower.";
    let base = Theme::publication();
    let role = |family, color, size, weight, leading| {
        crate::TextTheme::new(
            FontStack::new(
                [FontFamily::generic(family)].into_iter().chain(
                    base.typography()
                        .get_composition()
                        .get_body()
                        .get_stack()
                        .families()
                        .iter()
                        .cloned(),
                ),
            )
            .unwrap(),
            color,
            size,
            weight,
            leading,
        )
        .unwrap()
    };
    let title_style = role(
        GenericFontFamily::Monospace,
        Color::rgba(91, 37, 117, 180),
        31.25,
        TextWeight::Bold,
        1.8,
    );
    let body_style = role(
        GenericFontFamily::Serif,
        Color::rgb(23, 89, 53),
        17.5,
        TextWeight::Normal,
        1.35,
    );
    let panel = TextPanel::new(
        Text::new(TITLE)
            .style(title_style.clone())
            .alignment(TextAlignment::End),
        Text::new(BODY)
            .style(body_style.clone())
            .alignment(TextAlignment::Center),
    )
    .unwrap();
    let spacing = crate::RelativeSpacing::ratio(1, 1).unwrap();
    let composition = base
        .composition()
        .clone()
        .text_panel(crate::TextPanelSpacing::new(spacing.clone(), spacing).unwrap())
        .unwrap();
    let missing =
        FontStack::new([FontFamily::named("Unused inherited text panel font").unwrap()]).unwrap();
    let theme = base
        .customize()
        .composition(composition)
        .font_stack(missing)
        .unwrap()
        .build()
        .unwrap();
    let canvas = Canvas::grid(1, 1)
        .unwrap()
        .width(600)
        .theme(theme)
        .text_panel(0, 0, panel.clone())
        .unwrap();
    let discoveries = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
    let root = canvas
        .build_preflight_plan(|prepared, replay| {
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                discoveries + 1
            );
            for (source, appearance) in [(TITLE, &title_style), (BODY, &body_style)] {
                let requirement = prepared
                    .text_census
                    .requirements
                    .iter()
                    .find(|text| text.source == source)
                    .unwrap();
                assert_eq!(&requirement.stack, appearance.get_stack());
                assert_eq!(requirement.weight, appearance.get_weight());
                assert!(prepared
                    .text_census
                    .requirements
                    .iter()
                    .any(|text| text.source == "0"
                        && text.stack == requirement.stack
                        && text.weight == requirement.weight));
            }
            let retained = replay.retained_text();
            for (source, appearance, alignment) in [
                (TITLE, &title_style, PhysicalTextAlignment::End),
                (BODY, &body_style, PhysicalTextAlignment::Center),
            ] {
                let text = retained
                    .iter()
                    .find(|(_, text)| text.source == source)
                    .unwrap()
                    .1;
                assert_eq!(text.style.stack, *appearance.get_stack());
                assert_eq!(text.style.paint, appearance.get_color());
                assert_eq!(
                    text.style.font_size.to_bits(),
                    appearance.get_font_size().to_bits()
                );
                assert_eq!(text.style.weight, appearance.get_weight());
                assert_eq!(text.style.alignment, alignment);
                assert!(text.style.exact_override);
                assert_eq!(text.line_texts().concat(), source.replace('\n', ""));
            }
            let theme = &prepared.root_request();
            let wide = text_panel::natural_height(&panel, 600.0, theme).unwrap();
            let narrow = text_panel::natural_height(&panel, 280.0, theme).unwrap();
            assert!(
                narrow > wide,
                "selected styles participate in width-dependent height"
            );
            let measured = crate::layout::fit::measured(TextPanelPlan::measure(
                &panel,
                0,
                PixelRect::new(0.0, 600.0, 0.0, 900.0),
                theme,
                prepared.text_panel_intrinsics[0].unwrap(),
                &crate::layout::fit::FitScope::root(),
            ))
            .unwrap();
            let quanta = crate::layout::kernel::nearest_spatial_quanta;
            assert_eq!(
                quanta(measured.title_placement.position.x).unwrap(),
                600 * 1024 - 17 * 1024 - 512
            );
            assert_eq!(
                quanta(measured.body_placement.position.x).unwrap(),
                300 * 1024
            );
            let title = measured.title.plan();
            let body = measured.body.plan();
            let title_bottom = quanta(measured.title_placement.position.y).unwrap()
                + title.first_baseline_quanta.unwrap()
                - title.height_quanta;
            let body_top = quanta(measured.body_placement.position.y).unwrap()
                + body.first_baseline_quanta.unwrap();
            assert_eq!(
                title_bottom - body_top,
                31 * 1024 + 256,
                "gap uses the selected maximum font size"
            );
            assert!(title.lines[1].translation_x_quanta > title.lines[0].translation_x_quanta);
        })
        .unwrap();
    let counters = capture_counters();
    let mut backend = root.plan().take_svg_backend().unwrap();
    Canvas::render_preflighted(&root, &mut backend).unwrap();
    let (width, height) = root.dimensions();
    let scene = RenderedScene::new(width, height, backend.into_output()).unwrap();
    let outlined = scene.svg_with_text_as_paths().unwrap();
    assert_eq!(scene.svg_with_text_as_paths().unwrap(), outlined);
    let options = crate::PngOptions {
        sizing: crate::PngSizing::Source,
        compression: crate::PngCompression::Fast,
    };
    let png = scene.png_bytes(&options).unwrap();
    assert_eq!(scene.png_bytes(&options).unwrap(), png);
    assert_eq!(capture_counters(), counters);
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        discoveries + 1
    );
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one Shared authority contract checks effective policy, source counts, pre-font section slots and selected order"
)]
fn shared_composition_uses_parent_policy_in_both_source_orders() {
    use crate::LegendComposition::{Combined, Separate};
    let layers = Plot::line(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .markers(false)
    .point_layer(
        crate::PointLayer::new(
            crate::NumericValues::new("selected x", [0.5]),
            crate::NumericValues::new("selected y", [0.5]),
        )
        .color(crate::Color::rgba(80, 40, 120, 180))
        .legend_label("chosen raw"),
    )
    .legend_composition(Separate)
    .legend_composition(Combined)
    .legend_composition(Separate);
    // Stored child selections differ; only the Shared parent's effective value
    // owns contribution preparation, without mutating those stored selections.
    let primary = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [1.0, 0.0]),
    )
    .hue(crate::CategoryValues::new("treatment", ["control", "treated"]).unwrap());
    for (composition, layer_first) in [
        (Combined, true),
        (Combined, false),
        (Separate, true),
        (Separate, false),
    ] {
        let canvas = Canvas::grid(1, 2).unwrap().size(1000, 500);
        let canvas = if composition == Combined {
            // Omission, rather than an explicit enum, exercises the public
            // Shared default against differently selected stored children.
            canvas.legend_policy(CanvasLegendPolicy::Shared)
        } else if layer_first {
            canvas
                .legend_policy(CanvasLegendPolicy::Shared)
                .legend_composition(Combined)
                .legend_composition(composition)
        } else {
            canvas
                .legend_composition(composition)
                .legend_policy(CanvasLegendPolicy::Shared)
        };
        let mut canvas = if layer_first {
            canvas
                .panel(0, 0, layers.clone())
                .unwrap()
                .panel(0, 1, primary.clone())
                .unwrap()
        } else {
            canvas
                .panel(0, 0, primary.clone())
                .unwrap()
                .panel(0, 1, layers.clone())
                .unwrap()
        };
        let root = canvas
            .build_preflight_plan(|prepared, _| {
                let Some(descriptor) = &prepared.shared_guide else {
                    panic!("prepared Shared discrete descriptor");
                };
                assert_eq!(descriptor.composition, composition);
                assert_eq!(
                    descriptor.counts.sections,
                    if composition == Combined { 1 } else { 2 }
                );
                assert_eq!(descriptor.counts.entries, 3);
                assert_eq!(descriptor.counts.points, 3);
                assert_eq!(descriptor.counts.lines, 0);
                assert_eq!(descriptor.counts.fills, 0);
                assert_eq!(descriptor.counts.primary_source_keys, 2);
                assert_eq!(descriptor.counts.labeled_layer_source_keys, 1);
                assert_eq!(descriptor.counts.source_occurrences, 3);
                let owner = crate::layout::ComponentAddress::root()
                    .child(crate::layout::AddressRole::CanvasSharedLegend, 0)
                    .unwrap();
                let guide_text = prepared
                    .text_census
                    .requirements
                    .iter()
                    .filter(|requirement| {
                        matches!(
                            requirement.text_role,
                            crate::layout::PreparedTextRole::LegendTitle
                                | crate::layout::PreparedTextRole::LegendLabel
                        )
                    })
                    .collect::<Vec<_>>();
                let addresses = if composition == Combined {
                    vec![(0, None), (0, Some(0)), (0, Some(1)), (0, Some(2))]
                } else {
                    vec![
                        (0, None),
                        (0, Some(0)),
                        (0, Some(1)),
                        (1, None),
                        (1, Some(0)),
                    ]
                };
                assert_eq!(guide_text.len(), addresses.len());
                for (text, (section, entry)) in guide_text.iter().zip(addresses) {
                    let section = owner
                        .child(crate::layout::AddressRole::LegendSection, section)
                        .unwrap();
                    let expected = if let Some(entry) = entry {
                        section
                            .child(crate::layout::AddressRole::LegendEntry, entry)
                            .unwrap()
                            .child(crate::layout::AddressRole::LegendLabel, 0)
                            .unwrap()
                    } else {
                        section
                            .child(crate::layout::AddressRole::LegendTitle, 0)
                            .unwrap()
                    };
                    assert_eq!(text.component_address, expected,);
                }
            })
            .unwrap();
        let texts = root.plan().retained_text();
        for source in ["Treatment", "Control", "Treated", "chosen raw"] {
            assert_eq!(
                texts
                    .iter()
                    .filter(|(_, text)| text.source == source)
                    .count(),
                1
            );
        }
        assert_eq!(
            texts
                .iter()
                .filter(|(_, text)| text.source == "series")
                .count(),
            usize::from(composition == Separate)
        );
        let position = |source| {
            texts
                .iter()
                .position(|(_, text)| text.source == source)
                .unwrap()
        };
        assert!(position("Treatment") < position("Control"));
        assert!(position("Treated") < position("chosen raw"));
        if composition == Separate {
            assert!(position("Treated") < position("series"));
            assert!(position("series") < position("chosen raw"));
        }
        let layer_slot = usize::from(!layer_first);
        assert_eq!(
            canvas
                .placements
                .plot(layer_slot)
                .unwrap()
                .description()
                .legend_composition_spec(),
            Separate
        );
        if composition == Separate && layer_first {
            canvas.legend_policy = CanvasLegendPolicy::PerPlot;
            canvas
                .build_preflight_plan(|prepared, _| {
                    assert!(prepared.shared_guide.is_none());
                    let composition = |index: usize| {
                        prepared
                            .plot(index)
                            .unwrap()
                            .guide
                            .descriptor
                            .as_ref()
                            .unwrap()
                            .composition
                    };
                    assert_eq!(composition(0), Separate);
                    assert_eq!(composition(1), Combined);
                })
                .unwrap();
        }
    }
}

#[test]
fn shared_layer_legend_text_uses_local_fonts_and_compares_complete_requests() {
    use crate::{Color, FontFamily, FontStack, PointLayer, Text, TextAlignment, TextLayout};
    let mut theme = Theme::publication();
    let style = theme
        .typography
        .guides
        .legend_label
        .clone()
        .font_size(21.0)
        .unwrap();
    theme.typography.guides.legend_label.stack =
        FontStack::new([FontFamily::named("Unused inherited legend entry font").unwrap()]).unwrap();
    let title = Text::new("Selected\nshort")
        .style(style)
        .alignment(TextAlignment::End);
    let plot = |text| {
        Plot::line(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .point_layer(
            PointLayer::new(
                crate::NumericValues::new("selected x", [0.5]),
                crate::NumericValues::new("selected y", [0.5]),
            )
            .color(Color::rgb(20, 80, 140))
            .legend_label(text),
        )
    };
    let canvas = |second| {
        Canvas::grid(1, 2)
            .unwrap()
            .width(900)
            .theme(theme.clone())
            .legend_policy(CanvasLegendPolicy::Shared)
            .panel(0, 0, plot(title.clone()))
            .unwrap()
            .panel(0, 1, plot(second))
            .unwrap()
    };
    let source = canvas(title.clone());
    let root = source
        .build_preflight_plan(|prepared, _| {
            let descriptor = prepared.shared_guide.as_ref().unwrap();
            assert_eq!(
                descriptor.sections[0].entries()[0].identity,
                crate::decorations::legend::GuideEntryIdentity::supplemental("Selected\nshort")
                    .unwrap()
            );
        })
        .unwrap();
    let retained = root.plan().retained_text();
    let text: Vec<_> = retained
        .iter()
        .filter(|(_, text)| text.source == title.as_str())
        .collect();
    assert_eq!(text.len(), 1);
    assert_eq!(text[0].1.style.font_size.to_bits(), 21.0_f64.to_bits());
    assert!(text[0].1.style.exact_override);
    assert_eq!(
        text[0].1.style.alignment,
        crate::layout::PhysicalTextAlignment::End
    );
    let geometry = &text[0].0 .0;
    assert_eq!(
        geometry.lines[0].origin.x.units(),
        geometry.center.x.units() - text[0].1.width_quanta
            + text[0].1.lines[0].translation_x_quanta
    );
    for changed in [
        title.clone().layout(TextLayout::default()),
        title
            .clone()
            .style(title.get_style().unwrap().clone().font_size(22.0).unwrap()),
    ] {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let error = canvas(changed)
            .preflight_plan()
            .err()
            .expect("different authored entry requests");
        assert!(error
            .to_string()
            .contains("conflicting labels for raw identity"));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}

#[test]
fn shared_color_titles_reject_style_and_request_presence_conflicts_before_fonts() {
    use crate::{FontFamily, FontStack, Text, TextLayout};
    let base = crate::Theme::publication();
    let unavailable = base
        .typography()
        .get_guides()
        .get_legend_title()
        .clone()
        .stack(
            FontStack::new([FontFamily::named("Unavailable shared color title font").unwrap()])
                .unwrap(),
        )
        .unwrap();
    let title = Text::new("Signal").style(unavailable);
    let plot = |title| {
        Plot::heatmap(
            crate::CategoryValues::new("x", ["a"]).unwrap(),
            crate::CategoryValues::new("y", ["b"]).unwrap(),
            crate::NumericValues::new("signal", [1.0]),
        )
        .color_label(title)
    };
    for incompatible in [
        title
            .clone()
            .style(base.typography().get_guides().get_legend_title().clone()),
        title.clone().layout(TextLayout::default()),
    ] {
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared)
            .panel(0, 0, plot(title.clone()))
            .unwrap()
            .panel(0, 1, plot(incompatible))
            .unwrap();
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let error = canvas
            .preflight_plan()
            .err()
            .expect("incompatible complete title");
        assert!(error.to_string().contains("incompatible identity, title"));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "the Shared final-set contract retains style, one final font seal and pre-font policy refusals together"
)]
fn shared_numeric_guides_resolve_final_sets_and_reject_equal_text_policy_conflicts_before_fonts() {
    use crate::numeric::{
        decimal::{NegativeZero, NumericNotation, TrailingZeros},
        format::{NumericFormat, NumericPrecision},
    };
    let format = NumericFormat::new(NumericNotation::Decimal, NumericPrecision::Fractional(0))
        .unwrap()
        .suffix(" u")
        .unwrap();
    let title_style = Theme::publication()
        .typography()
        .get_guides()
        .get_legend_title()
        .clone()
        .font_size(21.0)
        .unwrap();
    let format = format
        .unit(Some(crate::NumericUnit::new(
            "guide units",
            crate::NumericUnitPlacement::Shared,
        )))
        .unwrap()
        .explanation(crate::NumericExplanation::automatic().style(title_style.clone()));
    let plot = |values, format| {
        Plot::heatmap(
            crate::CategoryValues::new("x", ["a", "b"]).unwrap(),
            crate::CategoryValues::new("y", ["row", "row"]).unwrap(),
            crate::NumericValues::new("value", values),
        )
        .color_label_format(format)
        .color_label(crate::Text::new("Final scale").style(title_style.clone()))
    };
    let first = plot([1.01, 1.02], format.clone());
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    assert!(first
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("not unique"));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
    let canvas = |second| {
        Canvas::grid(1, 2)
            .unwrap()
            .size(1000, 500)
            .legend_composition(crate::LegendComposition::Separate)
            .legend_policy(CanvasLegendPolicy::Shared)
            .panel(0, 0, first.clone())
            .unwrap()
            .panel(0, 1, plot([3.1, 3.2], second))
            .unwrap()
    };
    crate::font::canonical::FONT_FINAL_SEALS.with(|calls| calls.set(0));
    let root = canvas(format.clone()).preflight_plan().unwrap();
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        1
    );
    assert_eq!(
        crate::font::canonical::FONT_FINAL_SEALS.with(std::cell::Cell::get),
        1
    );
    let title = root
        .plan()
        .retained_text()
        .into_iter()
        .find(|(_, text)| text.source == "Final scale")
        .unwrap()
        .1;
    assert_eq!(title.style.font_size.to_bits(), 21.0_f64.to_bits());
    assert!(title.style.exact_override);
    let common: Vec<_> = root
        .plan()
        .retained_text()
        .into_iter()
        .filter(|(_, text)| text.source == "guide units")
        .collect();
    assert_eq!(common.len(), 1);
    assert_eq!(common[0].1.style.font_size.to_bits(), 21.0_f64.to_bits());
    assert!(common[0].1.style.exact_override);
    for source in ["1 u", "3 u"] {
        assert_eq!(
            root.plan()
                .retained_text()
                .iter()
                .filter(|(_, text)| text.source == source)
                .count(),
            1
        );
    }
    Canvas::grid(1, 1)
        .unwrap()
        .size(600, 400)
        .legend_policy(CanvasLegendPolicy::Hidden)
        .legend_composition(crate::LegendComposition::Separate)
        .panel(
            0,
            0,
            plot([1.01, 1.02], format.clone().suffix("\u{10ffff}").unwrap()),
        )
        .unwrap()
        .preflight_plan()
        .unwrap();
    for different in [
        format
            .clone()
            .notation(NumericNotation::Engineering)
            .unwrap(),
        format
            .clone()
            .precision(NumericPrecision::AdaptiveFractional { min: 0, max: 2 })
            .unwrap(),
        format.clone().decimal_scale(2).unwrap(),
        format.clone().prefix("unit ").unwrap(),
        format.clone().suffix(" other").unwrap(),
        format
            .clone()
            .negative_zero(NegativeZero::Normalize)
            .unwrap(),
        format
            .clone()
            .rounding(crate::NumericRounding::Floor)
            .unwrap(),
        format.clone().multiplier(1.01).unwrap(),
        format
            .clone()
            .prefixes(crate::NumericPrefixes::factor(crate::NumericFactor::Value(1.01)).unwrap())
            .unwrap(),
        format
            .clone()
            .locale(crate::NumericLocale::named("en-US").unwrap())
            .unwrap(),
        format.trailing_zeros(TrailingZeros::Trim).unwrap(),
    ] {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let error = canvas(different)
            .preflight_plan()
            .err()
            .expect("incompatible complete policy");
        assert!(error.to_string().contains("numeric format"));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
    assert_shared_alpha_declarations();
    assert_shared_color_and_alpha_sections();
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "checks independent numeric owners and the actual frozen, clipped text ink before publication"
)]
fn common_numeric_text_has_distinct_measured_owners_and_frozen_contained_output() {
    use crate::{
        NumericExplanation, NumericFactor, NumericFormat, NumericNotation, NumericPrecision,
        NumericPrefixes, NumericUnit, NumericUnitPlacement,
    };
    let unit = |text| {
        NumericFormat::default()
            .unit(Some(NumericUnit::new(text, NumericUnitPlacement::Shared)))
            .unwrap()
    };
    let factor = NumericFormat::new(NumericNotation::Decimal, NumericPrecision::Fractional(0))
        .unwrap()
        .prefixes(NumericPrefixes::factor(NumericFactor::PowerOfTen("6".into())).unwrap())
        .unwrap()
        .unit(Some(NumericUnit::new("", NumericUnitPlacement::Shared)))
        .unwrap();
    let axes = Plot::scatter(
        crate::NumericValues::new("x", [1e6, 3e6]),
        crate::NumericValues::new("y", [1.0, 3.0]),
    )
    .x_limits(1e6, 3e6)
    .y_limits(1.0, 3.0)
    .x_tick_format(factor)
    .y_tick_format(unit("axis unit"));
    let cells = Plot::heatmap(
        crate::CategoryValues::new("column", ["a", "b"]).unwrap(),
        crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
        crate::NumericValues::new("value", [1.0, 2.0]),
    )
    .annotations(crate::HeatmapAnnotations::Values(unit("cell unit")))
    .color_label_format(unit("color unit"));
    let estimates = Plot::coefficient(
        crate::CategoryValues::new("term", ["a", "b"]).unwrap(),
        crate::CoefficientInput::estimates(crate::NumericValues::new("estimate", [1.0, 2.0])),
    )
    .estimate_labels(unit("estimate unit"));
    let totals = Plot::waterfall(
        crate::CategoryValues::new("step", ["gain"]).unwrap(),
        crate::NumericValues::new("delta", [1.0]),
        1.0,
    )
    .total_format(unit("total unit"));
    let canvas = Canvas::grid(2, 2)
        .unwrap()
        .size(1400, 1100)
        .panel(0, 0, axes)
        .unwrap()
        .panel(0, 1, cells)
        .unwrap()
        .panel(1, 0, estimates)
        .unwrap()
        .panel(1, 1, totals)
        .unwrap();
    let root = canvas.preflight_plan().unwrap();
    for source in [
        "×10⁶",
        "axis unit",
        "cell unit",
        "color unit",
        "estimate unit",
        "total unit",
    ] {
        let texts: Vec<_> = root
            .plan()
            .retained_text()
            .into_iter()
            .filter(|(_, text)| text.source == source)
            .collect();
        assert_eq!(texts.len(), 1, "{source}");
        let (placement, plan) = &texts[0];
        let geometry = &placement.0;
        #[expect(
            clippy::cast_precision_loss,
            reason = "test inspects selected text quanta in CSS pixels"
        )]
        let pixels = |value: i64| value as f64 / 1024.0;
        let (sine, cosine) = geometry.rotation_sin_cos;
        let center = geometry.center;
        for (line, placed) in plan.lines.iter().zip(&geometry.lines) {
            let Some(ink) = line.ink_quanta else { continue };
            let mut bounds = [
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ];
            for (x, y) in [
                (ink.left, ink.bottom),
                (ink.left, ink.top),
                (ink.right, ink.bottom),
                (ink.right, ink.top),
            ] {
                let x = placed.origin.x.value() + pixels(x) - center.x.value();
                let y = placed.origin.y.value() - pixels(y) - center.y.value();
                let point = (
                    center.x.value() + x * cosine - y * sine,
                    center.y.value() + x * sine + y * cosine,
                );
                bounds[0] = bounds[0].min(point.0);
                bounds[1] = bounds[1].max(point.0);
                bounds[2] = bounds[2].min(point.1);
                bounds[3] = bounds[3].max(point.1);
            }
            if let Some(clip) = geometry.clip {
                bounds[0] = bounds[0].max(clip.x.value());
                bounds[1] = bounds[1].min(clip.x.value() + clip.width.value());
                bounds[2] = bounds[2].max(clip.y.value());
                bounds[3] = bounds[3].min(clip.y.value() + clip.height.value());
            }
            assert!(
                bounds[0] >= 0.0 && bounds[1] <= 1400.0 && bounds[2] >= 0.0 && bounds[3] <= 1100.0,
                "{source}: {bounds:?}"
            );
        }
    }
    let counters = capture_counters();
    let mut backend = root.plan().take_svg_backend().unwrap();
    Canvas::render_preflighted(&root, &mut backend).unwrap();
    let scene = RenderedScene::new(1400, 1100, backend.into_output()).unwrap();
    assert_eq!(
        scene.svg_with_text_as_paths().unwrap(),
        scene.svg_with_text_as_paths().unwrap()
    );
    assert_eq!(capture_counters(), counters);
    let narrow = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .x_tick_format(unit("m").explanation(NumericExplanation::text("W".repeat(128))))
    .width(220);
    assert!(narrow.preflight().is_err());
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

#[test]
fn numeric_functions_keep_raw_facet_levels_without_optional_child_reuse() {
    use std::sync::{Arc, Mutex};
    let observed = Arc::new(Mutex::new(std::collections::BTreeSet::new()));
    let contexts = observed.clone();
    let locator_seen = Arc::new(Mutex::new(std::collections::BTreeSet::new()));
    let locator_calls = locator_seen.clone();
    let format = crate::NumericFormat::default().set_function(move |context| {
        let owner = context.owner();
        let facet = owner.facet_cell().unwrap();
        assert!(facet.single_level().is_none());
        let (row, column) = facet.grid_levels().unwrap();
        assert_eq!(row, "raw_row");
        assert!(matches!(column, "raw_a" | "raw_b"));
        let values: Vec<_> = context
            .values()
            .iter()
            .map(|value| match value.operand() {
                crate::NumericOperand::Binary64(value) => value.to_bits(),
                crate::NumericOperand::PowerOfTen { .. }
                | crate::NumericOperand::Multiplicative { .. } => panic!("ordinary facet"),
            })
            .collect();
        let domain = context.domain().unwrap().scale_limits();
        assert!(contexts.lock().unwrap().insert((
            column.to_owned(),
            context.purpose() as u8,
            (domain.0.to_bits(), domain.1.to_bits()),
            values
        )));
        context.format_builtin()
    });
    let source = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0, 0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0, 1.0, 0.0]),
    )
    .x_tick_generation(crate::TickGeneration::function(move |context| {
        let facet = context.owner().facet_cell().unwrap();
        let (row, column) = facet.grid_levels().unwrap();
        assert_eq!(row, "raw_row");
        assert!(locator_calls
            .lock()
            .unwrap()
            .insert((column.to_owned(), context.span().to_bits())));
        let (lower, upper) = context.domain().scale_limits();
        Ok(vec![lower, upper].into())
    }))
    .facet(
        crate::FacetSpec::grid(
            crate::CategoryValues::new("r", ["raw_row"; 4]).unwrap(),
            crate::CategoryValues::new("c", ["raw_a", "raw_a", "raw_b", "raw_b"]).unwrap(),
        )
        .unwrap(),
    )
    .x_tick_format(format)
    .size(1000, 600);
    crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(true));
    let result = source.preflight();
    crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
    result.unwrap();
    assert!(locator_seen
        .lock()
        .unwrap()
        .iter()
        .any(|key| key.0 == "raw_a"));
    assert!(locator_seen
        .lock()
        .unwrap()
        .iter()
        .any(|key| key.0 == "raw_b"));
    let contexts = observed.lock().unwrap();
    for column in ["raw_a", "raw_b"] {
        assert!(contexts
            .iter()
            .any(|key| key.0 == column && key.1 == crate::NumericPurpose::ActualSet as u8));
    }
}

#[test]
fn exact_generated_counts_preserve_linear_data_and_mathematical_logarithmic_operands() {
    use crate::{NumericFormat, NumericOperand, NumericPurpose, TickGeneration, TickSpace};
    let numeric = NumericFormat::from_specifier(".2~f")
        .unwrap()
        .prefix("E:")
        .unwrap();
    let ordinary = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 10.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .x_limits(0.0, 10.0)
    .x_tick_generation(TickGeneration::exact_count(4, TickSpace::Data))
    .x_tick_format(numeric);
    let forest = |space| {
        Plot::forest(
            crate::CategoryValues::new("term", ["ratio"]).unwrap(),
            crate::NumericValues::new("estimate", [1.0]),
            crate::Bounds::new(
                crate::NumericValues::new("lo", [0.2]),
                crate::NumericValues::new("hi", [5.0]),
            ),
            1.0,
        )
        .scale(crate::AxisScale::Log10)
        .x_limits(0.1, 10.0)
        .x_tick_generation(TickGeneration::exact_count(3, space))
        .x_tick_format(NumericFormat::default().set_function(move |context| {
            if context.purpose() == NumericPurpose::ActualSet {
                let values: Vec<_> = context
                    .values()
                    .iter()
                    .map(|value| match (space, value.operand()) {
                        (TickSpace::Data, NumericOperand::Binary64(value)) => value,
                        (TickSpace::Scale, NumericOperand::PowerOfTen { exponent }) => exponent,
                        _ => panic!("exact-count operand space changed"),
                    })
                    .collect();
                assert_eq!(
                    values,
                    if space == TickSpace::Data {
                        vec![0.1, 5.05, 10.0]
                    } else {
                        vec![-1.0, 0.0, 1.0]
                    }
                );
            }
            context.format_builtin()
        }))
    };
    let root = Canvas::grid(1, 3)
        .unwrap()
        .size(2400, 600)
        .panel(0, 0, ordinary)
        .unwrap()
        .panel(0, 1, forest(TickSpace::Data))
        .unwrap()
        .panel(0, 2, forest(TickSpace::Scale))
        .unwrap()
        .preflight_plan()
        .unwrap();
    let sources: Vec<_> = root
        .plan()
        .retained_text()
        .into_iter()
        .filter(|(_, text)| text.source.starts_with("E:"))
        .map(|(_, text)| text.source.as_str())
        .collect();
    assert_eq!(sources, ["E:0", "E:3.33", "E:6.67", "E:10"]);
    Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .x_tick_generation(TickGeneration::exact_count(0, TickSpace::Data))
    .x_tick_format(
        NumericFormat::default()
            .set_function(|_| panic!("empty source must not invoke numeric functions")),
    )
    .preflight()
    .unwrap();
}

#[test]
fn exact_counts_follow_parent_domains_but_keep_independent_canvas_owners() {
    use std::sync::{Arc, Mutex};
    let observed = Arc::new(Mutex::new(std::collections::BTreeMap::new()));
    let contexts = observed.clone();
    let format = crate::NumericFormat::from_specifier(".2~f")
        .unwrap()
        .set_function(move |context| {
            if context.purpose() == crate::NumericPurpose::ActualSet {
                let domain = context.domain().unwrap();
                assert_eq!(domain.origin(), crate::NumericRangeOrigin::ParentResolved);
                let operands: Vec<_> = context
                    .values()
                    .iter()
                    .map(|value| match value.operand() {
                        crate::NumericOperand::Binary64(value) => value.to_bits(),
                        crate::NumericOperand::PowerOfTen { .. }
                        | crate::NumericOperand::Multiplicative { .. } => {
                            panic!("linear shared domain")
                        }
                    })
                    .collect();
                assert_eq!(operands.len(), 4);
                let (lower, upper) = domain.scale_limits();
                assert_eq!(
                    (operands[0], operands[3]),
                    (lower.to_bits(), upper.to_bits())
                );
                contexts
                    .lock()
                    .unwrap()
                    .insert(context.owner().canvas_cell().unwrap(), operands);
            }
            context.format_builtin()
        });
    let plot = |values| {
        Plot::scatter(
            crate::NumericValues::new("x", values),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .x_tick_generation(crate::TickGeneration::exact_count(
            4,
            crate::TickSpace::Data,
        ))
        .x_tick_format(format.clone())
    };
    Canvas::grid(1, 2)
        .unwrap()
        .panel(0, 0, plot([0.0, 10.0]))
        .unwrap()
        .panel(0, 1, plot([100.0, 200.0]))
        .unwrap()
        .x_range_group(vec![(0, 0), (0, 1)])
        .unwrap()
        .size(1400, 600)
        .preflight()
        .unwrap();
    let observed = observed.lock().unwrap();
    assert_eq!(observed.len(), 2);
    assert_eq!(observed[&(0, 0)], observed[&(0, 1)]);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one interval contract follows parent resolution across changed assignment, typography and optional reuse, including the empty-owner boundary"
)]
fn fixed_intervals_consume_final_parent_domains_and_empty_sources_have_no_callbacks() {
    use crate::{
        NumericFormat, NumericOperand, NumericPurpose, TickGeneration, TickRequirement, TickSpace,
    };
    use std::sync::{Arc, Mutex};
    let observed = Arc::new(Mutex::new(std::collections::BTreeMap::new()));
    let contexts = observed.clone();
    let format = NumericFormat::from_specifier(".0f")
        .unwrap()
        .set_function(move |context| {
            if context.purpose() == NumericPurpose::ActualSet {
                assert_eq!(
                    context.domain().unwrap().origin(),
                    crate::NumericRangeOrigin::ParentResolved
                );
                let values: Vec<_> = context
                    .values()
                    .iter()
                    .map(|value| match value.operand() {
                        NumericOperand::Binary64(value) => value,
                        NumericOperand::PowerOfTen { .. }
                        | NumericOperand::Multiplicative { .. } => {
                            panic!("data interval changed operand space")
                        }
                    })
                    .collect();
                assert!(values.len() > 5);
                assert!(values
                    .windows(2)
                    .all(|pair| (pair[1] - pair[0]).to_bits() == 20.0_f64.to_bits()));
                contexts.lock().unwrap().insert(
                    context.owner().canvas_cell().unwrap(),
                    values
                        .iter()
                        .map(|value| value.to_bits())
                        .collect::<Vec<_>>(),
                );
            }
            context.format_builtin()
        });
    let plot = |values| {
        Plot::scatter(
            crate::NumericValues::new("x", values),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .x_tick_generation(
            TickGeneration::interval(20.0, 5.0, TickSpace::Data, TickRequirement::Fixed).unwrap(),
        )
        .x_tick_format(format.clone())
    };
    let mut previous = None;
    for (width, size, disabled) in [(1800, 14, false), (2400, 18, true)] {
        let base = Theme::publication();
        let role = base
            .typography()
            .get_axes()
            .get_tick_label()
            .clone()
            .font_size(f64::from(size))
            .unwrap();
        let typography = base
            .typography()
            .clone()
            .axes(
                base.typography()
                    .get_axes()
                    .clone()
                    .tick_label(role)
                    .unwrap(),
            )
            .unwrap();
        let theme = base.customize().typography(typography).build().unwrap();
        crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
        let result = Canvas::grid(1, 2)
            .unwrap()
            .theme(theme)
            .panel(0, 0, plot([0.0, 100.0]))
            .unwrap()
            .panel(0, 1, plot([0.0, 200.0]))
            .unwrap()
            .x_range_group(vec![(0, 0), (0, 1)])
            .unwrap()
            .size(width, 600)
            .preflight();
        crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
        result.unwrap();
        let current = observed.lock().unwrap().clone();
        assert_eq!(current.len(), 2);
        assert_eq!(current[&(0, 0)], current[&(0, 1)]);
        if let Some(previous) = previous {
            assert_eq!(current, previous);
        }
        previous = Some(current);
    }
    Plot::scatter(
        crate::NumericValues::new("x", [1.1, 1.2]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .x_limits(1.1, 1.2)
    .x_tick_generation(
        TickGeneration::interval(1.0, 0.0, TickSpace::Data, TickRequirement::Fixed).unwrap(),
    )
    .x_tick_format(
        NumericFormat::default()
            .set_function(|_| panic!("empty interval invokes no numeric function")),
    )
    .preflight()
    .unwrap();
}

#[expect(
    clippy::too_many_lines,
    reason = "one Shared declaration contract checks actual union members, Hidden, aliases and complete custom recipes before fonts"
)]
fn assert_shared_alpha_declarations() {
    use crate::alpha::{AlphaEncoding, AlphaTarget, AlphaValues};
    use crate::decorations::legend::discrete::GuideSectionBody;
    let values = AlphaValues::categorical(
        crate::CategoryValues::new("quality", ["low", "high", "\u{10ffff}"]).unwrap(),
        |value| match value {
            "low" => 0,
            "high" => 230,
            _ => 17,
        },
    )
    .unwrap();
    let scatter = |values| {
        Plot::scatter(
            crate::NumericValues::new("x", [0.5]),
            crate::NumericValues::new("y", [0.5]),
        )
        .labels(crate::Labels::raw())
        .x_limits(0.0, 1.0)
        .y_limits(0.0, 1.0)
        .with_alpha(AlphaTarget::Point, AlphaEncoding::observations(values))
    };
    let shared = |first, second| {
        Canvas::grid(2, 2)
            .unwrap()
            .size(1200, 700)
            .legend_policy(CanvasLegendPolicy::Shared)
            .panel(0, 0, first)
            .unwrap()
            .panel_span(0, 1, 2, 1, second)
            .unwrap()
    };
    let root = shared(
        scatter(values.select([0]).unwrap()),
        scatter(values.select([1]).unwrap()),
    )
    .build_preflight_plan(|prepared, _| {
        let descriptor = prepared.shared_guide.as_ref().unwrap();
        assert_eq!(descriptor.sections.len(), 1);
        assert_eq!(
            descriptor.sections[0]
                .entries()
                .iter()
                .map(|entry| entry.display_label.as_str())
                .collect::<Vec<_>>(),
            ["low", "high"]
        );
        let GuideSectionBody::Alpha(body) = &descriptor.sections[0].body else {
            panic!("Shared alpha body")
        };
        assert_eq!(body.sources.len(), 2);
        assert_eq!(
            body.sources
                .iter()
                .map(|source| source.glyphs[0].members[0].value.alpha)
                .collect::<Vec<_>>(),
            [0, 230]
        );
    })
    .unwrap();
    assert!(root
        .plan()
        .retained_text()
        .iter()
        .all(|(_, text)| !text.source.contains('\u{10ffff}')));
    Canvas::grid(1, 1)
        .unwrap()
        .size(600, 400)
        .legend_policy(CanvasLegendPolicy::Hidden)
        .panel(0, 0, scatter(values.select([2]).unwrap()))
        .unwrap()
        .preflight_plan()
        .unwrap();
    let labels = crate::Labels::raw()
        .alias("low", "same")
        .alias("high", "same");
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let error = shared(
        scatter(values.select([0]).unwrap()).labels(labels.clone()),
        scatter(values.select([1]).unwrap()).labels(labels),
    )
    .preflight_plan()
    .err()
    .expect("distinct alpha categories cannot collapse in Shared text");
    assert!(error
        .to_string()
        .contains("distinct declared alpha members"));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
    let custom = |different| {
        AlphaValues::continuous(
            crate::NumericValues::new("quality", [0.25, 0.375, 0.75]),
            (0.0, 1.0),
            |value| {
                if different && value.to_bits() == 0.375_f64.to_bits() {
                    99
                } else {
                    0
                }
            },
        )
        .unwrap()
        .select([0])
        .unwrap()
    };
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let error = shared(scatter(custom(false)), scatter(custom(true)))
        .preflight_plan()
        .err()
        .expect("unequal complete custom recipes");
    assert!(error.to_string().contains("alpha declarations"));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

fn assert_shared_color_and_alpha_sections() {
    use crate::alpha::{AlphaEncoding, AlphaTarget, AlphaValues};
    use crate::decorations::legend::discrete::{GuideSectionBody, GuideSectionIdentity};
    let values = |source| {
        AlphaValues::linear(
            crate::NumericValues::new("quality", source),
            (0.0, 1.0),
            (0, 255),
        )
        .unwrap()
    };
    let base = Theme::publication();
    let label = base
        .typography()
        .get_guides()
        .get_legend_label()
        .clone()
        .color(crate::Color::rgba(11, 22, 33, 0))
        .unwrap();
    let guides = base
        .typography()
        .get_guides()
        .clone()
        .legend_label(label)
        .unwrap();
    let typography = base.typography().clone().guides(guides).unwrap();
    let source_theme = base.customize().typography(typography).build().unwrap();
    let points = Plot::scatter(
        crate::NumericValues::new("x", [0.5, 0.75]),
        crate::NumericValues::new("y", [0.5, 0.75]),
    )
    .theme(source_theme.clone())
    .x_limits(0.0, 1.0)
    .y_limits(0.0, 1.0)
    .with_alpha(
        AlphaTarget::Point,
        AlphaEncoding::observations(values([0.25, 0.75])),
    );
    let cells = Plot::heatmap(
        crate::CategoryValues::new("x", ["A", "B"]).unwrap(),
        crate::CategoryValues::new("y", ["row", "row"]).unwrap(),
        crate::NumericValues::new("signal", [0.0, 1.0]),
    )
    .theme(source_theme)
    .with_alpha(
        AlphaTarget::Fill,
        AlphaEncoding::observations(values([0.0, 1.0])),
    );
    let root = Canvas::grid(1, 2)
        .unwrap()
        .size(1400, 800)
        .legend_policy(CanvasLegendPolicy::Shared)
        .panel(0, 0, points)
        .unwrap()
        .panel(0, 1, cells)
        .unwrap()
        .build_preflight_plan(|prepared, _| {
            let descriptor = prepared.shared_guide.as_ref().unwrap();
            assert_eq!(descriptor.sections.len(), 3);
            assert!(matches!(
                descriptor.sections[0].body,
                GuideSectionBody::ContinuousColor(_)
            ));
            let targets = descriptor.sections[1..]
                .iter()
                .map(|section| {
                    let GuideSectionIdentity::Alpha(identity) = &section.identity else {
                        panic!("alpha follows color content")
                    };
                    identity.target
                })
                .collect::<Vec<_>>();
            assert_eq!(targets, [AlphaTarget::Point, AlphaTarget::Fill]);
            for section in &descriptor.sections[1..] {
                let GuideSectionBody::Alpha(alpha) = &section.body else {
                    unreachable!()
                };
                assert!(alpha
                    .sources
                    .iter()
                    .all(|source| source.neutral == crate::OpaqueColor::rgb(11, 22, 33)));
            }
        })
        .unwrap();
    let before = capture_counters();
    let mut svg = root.plan().take_svg_backend().unwrap();
    root.plan()
        .replay(1400, 800, &mut svg, "typed Shared color and alpha")
        .unwrap();
    let svg = String::from_utf8(svg.into_bytes().unwrap()).unwrap();
    assert_eq!(svg.matches("<linearGradient").count(), 1);
    assert_eq!(svg.matches("id=\"canvas-shared-legend\"").count(), 1);
    assert_eq!(svg.matches("fill=\"#0B1621\"").count(), 10);
    assert_eq!(capture_counters(), before);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one composed scene verifies role-local precedence, construction order and frozen replay"
)]
fn heading_instructions_are_role_local_order_independent_and_frozen() {
    use crate::layout::PhysicalTextAlignment;
    use crate::TextAlignment::{Center, End, Start};
    let base = Theme::publication();
    let composition = base
        .composition()
        .clone()
        .root_title_alignment(Start)
        .unwrap()
        .root_subtitle_alignment(None)
        .unwrap()
        .root_caption_alignment(End)
        .unwrap();
    let root_theme = base.customize().composition(composition).build().unwrap();
    let composition = base
        .composition()
        .clone()
        .root_title_alignment(End)
        .unwrap()
        .root_subtitle_alignment(Some(Start))
        .unwrap()
        .root_caption_alignment(Center)
        .unwrap();
    let local_theme = base.customize().composition(composition).build().unwrap();
    let scatter = |name: &str| {
        Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [2.0, 1.0]),
        )
        .title(format!("{name} title\nx"))
        .subtitle(format!("{name} subtitle\ny"))
        .caption(format!("{name} caption\nz"))
    };
    let mut canonical = None;
    for reversed in [false, true] {
        let explicit = if reversed {
            scatter("Explicit")
                .title_alignment(Start)
                .title_alignment(End)
                .subtitle_alignment(End)
                .subtitle_alignment(Start)
                .theme(local_theme.clone())
        } else {
            scatter("Explicit")
                .theme(local_theme.clone())
                .subtitle_alignment(Start)
                .title_alignment(End)
        };
        let canvas = Canvas::grid(1, 3)
            .unwrap()
            .size(1201, 701)
            .title("Canvas title\nx")
            .subtitle("Canvas subtitle\ny")
            .caption("Canvas caption\nz")
            .panel(0, 0, scatter("Inherited"))
            .unwrap()
            .panel(0, 1, scatter("Replaced").theme(local_theme.clone()))
            .unwrap()
            .panel(0, 2, explicit)
            .unwrap();
        let canvas = if reversed {
            canvas
                .panel_title_alignment(End)
                .panel_title_alignment(Center)
                .panel_subtitle_alignment(Start)
                .panel_subtitle_alignment(End)
                .theme(root_theme.clone())
        } else {
            canvas
                .theme(root_theme.clone())
                .panel_subtitle_alignment(End)
                .panel_title_alignment(Center)
        };
        let root = canvas.preflight_plan().unwrap();
        let retained = root.plan().retained_text();
        for (source, expected) in [
            ("Canvas title\nx", PhysicalTextAlignment::Start),
            ("Canvas subtitle\ny", PhysicalTextAlignment::Start),
            ("Canvas caption\nz", PhysicalTextAlignment::End),
            ("Inherited title\nx", PhysicalTextAlignment::Center),
            ("Inherited subtitle\ny", PhysicalTextAlignment::End),
            ("Inherited caption\nz", PhysicalTextAlignment::End),
            ("Replaced title\nx", PhysicalTextAlignment::Center),
            ("Replaced subtitle\ny", PhysicalTextAlignment::End),
            ("Replaced caption\nz", PhysicalTextAlignment::Center),
            ("Explicit title\nx", PhysicalTextAlignment::End),
            ("Explicit subtitle\ny", PhysicalTextAlignment::Start),
            ("Explicit caption\nz", PhysicalTextAlignment::Center),
        ] {
            let matching = retained
                .iter()
                .filter(|(_, text)| text.source == source)
                .collect::<Vec<_>>();
            assert_eq!(matching.len(), 1, "{source}");
            assert_eq!(matching[0].1.style.alignment, expected, "{source}");
            assert_eq!(matching[0].1.lines.len(), 2, "{source}");
        }
        let counters = capture_counters();
        let mut backend = root.plan().take_svg_backend().unwrap();
        Canvas::render_preflighted(&root, &mut backend).unwrap();
        assert_eq!(capture_counters(), counters);
        let svg = backend.into_bytes().unwrap();
        if let Some(before) = &canonical {
            assert_eq!(before, &svg);
        } else {
            canonical = Some(svg);
        }
    }
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
            Arc::make_mut(&mut rejected.placements)
                .insert(
                    placement::Footprint::cell(0, 0),
                    placement::Payload::Plot(Arc::new(plot.clone())),
                )
                .unwrap();
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
        Arc::make_mut(&mut canvas.placements)
            .insert(
                placement::Footprint::cell(index / 2, index % 2),
                placement::Payload::Plot(Arc::new(plot.clone().title(format!("{kind:?}")))),
            )
            .unwrap();
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
