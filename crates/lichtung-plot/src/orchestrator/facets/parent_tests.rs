use super::*;

#[test]
fn fixed_facet_factory_preserves_preflight_provenance_and_ignores_coordinate_feedback() {
    use crate::decorations::plan::children::MeasurementPhase;
    use crate::layout::{AddressRole, ComponentAddress, RangeProvenance};
    use crate::orchestrator::root::{Root, RootSource};
    let source = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0, 0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0, 0.0, 1.0]),
    )
    .facet(crate::FacetSpec::column(
        crate::CategoryValues::new("facet", ["first", "first", "second", "second"]).unwrap(),
    ))
    .legend(crate::LegendPolicy::Hidden)
    .size(1200, 800);
    FACTORY_PHASES.with(|events| events.borrow_mut().clear());
    let owner = ComponentAddress::root()
        .child(AddressRole::FacetCell, 0)
        .unwrap();
    let mut rounds = 0;
    let frozen = crate::layout::plan_root(
        source.description().sizing_request(),
        |stage| Root::prepare(RootSource::Plot(source.description()), stage),
        |root, assigned, round| {
            let measured = root.measure(assigned, round)?;
            if rounds == 0 {
                round.request_domain(
                    &owner,
                    &owner,
                    crate::layout::kernel::FeedbackKind::AutomaticDomainX,
                    (-10.0, 10.0),
                )?;
            }
            rounds += 1;
            Ok(measured)
        },
        |root, size, plan, stage| {
            size.dimensions()?;
            let verifier = root.sealed_verifier()?;
            crate::orchestrator::lifecycle::capture_selected(root, plan, &verifier, stage)
        },
    )
    .unwrap();
    assert!(rounds >= 2);
    let (width, height) = frozen.dimensions();
    crate::orchestrator::validate_text_replay_plan(
        width,
        height,
        frozen.plan(),
        "fixed facet phase Null replay",
    )
    .unwrap();
    FACTORY_PHASES.with(|events| {
        let events = events.borrow();
        let probe = events
            .iter()
            .find(|(_, phase, _, _)| matches!(phase, Some(MeasurementPhase::Probe)))
            .expect("real preflight assignment");
        let full = events
            .iter()
            .find(|(id, phase, _, _)| *id == probe.0 && phase.is_none())
            .expect("same occurrence full assignment");
        assert_eq!(probe.2, full.2);
        assert_eq!(probe.3, [RangeProvenance::Automatic; 2]);
        assert_eq!(full.3, [RangeProvenance::Shared; 2]);
        assert!(events.iter().any(|(id, phase, ranges, _)| {
            *id == probe.0 && phase.is_none() && ranges[0].0 <= -10.0 && ranges[0].1 >= 10.0
        }));
        for (_, phase, ranges, _) in events.iter().filter(|(id, _, _, _)| *id == probe.0) {
            if matches!(phase, Some(MeasurementPhase::Probe)) {
                assert_eq!(
                    *ranges, probe.2,
                    "preflight must not consume coordinate feedback"
                );
            }
        }
    });
}

#[test]
fn scoped_annotation_indices_locate_sources_without_renumbering_filtered_occurrences() {
    use crate::{
        BarInput, CategoryValues, FacetCell, FacetSpec, NumericValues, Significance,
        SignificanceMark,
    };
    let source = Plot::bar(
        CategoryValues::new("category", ["a", "b", "a", "b"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [1.0, 2.0, 3.0, 4.0])),
    )
    .facet(FacetSpec::column(
        CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap(),
    ))
    .significance([
        Significance::at("a", SignificanceMark::TwoStars).facet(FacetCell::single("left")),
        Significance::at("a", SignificanceMark::OneStar).facet(FacetCell::single("right")),
    ])
    .significance([
        Significance::between("a", "b", SignificanceMark::FourStars)
            .facet(FacetCell::single("left")),
        Significance::between("a", "b", SignificanceMark::ThreeStars)
            .facet(FacetCell::single("right")),
    ])
    .erase();
    let appearance = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let request = TextRequestContext::new(&appearance, &text);
    super::super::with_prepared_plot_for_test(
        &source,
        &request,
        |_| Ok(()),
        |_plot, inputs, _request, _operation, round| {
            let registered = &inputs.plots[1];
            assert_eq!(registered.policy.significance_indices, [1, 3]);
            let super::super::registration::PreparedFamily::Bars { marks, .. } = &registered.family
            else {
                panic!("prepared Bar family")
            };
            assert_eq!(marks.significance.len(), 1);
            assert_eq!(marks.significance[0].source_index, 0);
            let measurement = inputs.assignment_factory().assign(
                source
                    .description()
                    .sizing_request()
                    .assigned_size()
                    .unwrap_or(crate::layout::RootSize::new(1, 1)),
            );
            let factory = inputs
                .plot_assignment_factory(&measurement, &registered.scope.id)
                .unwrap();
            let result = crate::layout::interior::measure_for_test(
                &factory,
                crate::layout::PixelRect::new(50.0, 750.0, 50.0, 550.0),
                round,
            )
            .unwrap();
            let brackets = &result.significance.as_ref().unwrap().brackets;
            assert_eq!(brackets.len(), 1);
            assert_eq!(brackets[0].source_index, 1);
            assert_eq!(brackets[0].label.plan.source, "***");
            assert_eq!(source.description().significance_annotations().len(), 4);
        },
    )
    .unwrap();
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one owning facet guide preserves independent parent domains and actual alpha members before fonts"
)]
fn sectioned_facets_preserve_parent_domains_and_reject_alias_collisions_before_fonts() {
    use crate::LegendComposition;
    let source = Plot::interaction(
        crate::CategoryValues::new(
            "condition",
            ["before", "before", "before", "after", "after", "after"],
        )
        .unwrap(),
        crate::NumericValues::new("estimate", [2.0, 3.0, 4.0, 5.0, 6.0, 7.0]),
        crate::CategoryValues::new("group", ["one", "two", "one", "one", "two", "one"]).unwrap(),
        crate::Bounds::new(
            crate::NumericValues::new("lower", [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
            crate::NumericValues::new("upper", [3.0, 4.0, 5.0, 6.0, 7.0, 8.0]),
        ),
    )
    .hue(
        crate::CategoryValues::new("hue", ["blue", "red", "green", "blue", "red", "green"])
            .unwrap(),
    )
    .styles([
        crate::InteractionStyle::new("one").unwrap(),
        crate::InteractionStyle::new("two")
            .unwrap()
            .point_shape(crate::PointShape::TriangleUp)
            .unwrap(),
    ])
    .facet(crate::FacetSpec::column(
        crate::CategoryValues::new("facet", ["left", "right", "left", "left", "right", "left"])
            .unwrap(),
    ))
    .labels(crate::Labels::raw())
    .size(1200, 700)
    .legend_composition(LegendComposition::Separate);
    let source = source.with_alpha(
        crate::alpha::AlphaTarget::Point,
        crate::alpha::AlphaEncoding::observations(
            crate::alpha::AlphaValues::categorical(
                crate::CategoryValues::new(
                    "confidence",
                    ["low", "high", "low", "low", "high", "low"],
                )
                .unwrap(),
                |value| if value == "low" { 0 } else { 230 },
            )
            .unwrap(),
        ),
    );
    let source = source.erase();
    let operation = crate::font::canonical::OperationContext::new();
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    let (mut inputs, _) =
        crate::plots::shared_x::prepare_plot(source.description(), &operation, &theme).unwrap();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let prepared = prepare_discrete_guide(
        source.description(),
        PlotKind::Interaction,
        &mut inputs,
        &operation,
    )
    .unwrap();
    let guide = prepared.descriptor.unwrap();
    assert_eq!(
        (
            guide.composition,
            guide.counts.sections,
            guide.counts.entries
        ),
        (LegendComposition::Separate, 3, 7)
    );
    assert_eq!(
        guide.sections[0]
            .entries()
            .iter()
            .map(|entry| entry.display_label.as_str())
            .collect::<Vec<_>>(),
        ["one", "two"]
    );
    // First occupied facet contributes blue/green before the right facet's red.
    // The complete parent scientific hue domain is instead blue/red/green.
    assert_eq!(
        guide.sections[1]
            .entries()
            .iter()
            .map(|entry| entry.display_label.as_str())
            .collect::<Vec<_>>(),
        ["blue", "red", "green"]
    );
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
    let crate::decorations::legend::discrete::GuideSectionBody::Alpha(alpha) =
        &guide.sections[2].body
    else {
        panic!("parent alpha union")
    };
    assert_eq!(
        alpha
            .entries
            .iter()
            .map(|entry| entry.display_label.as_str())
            .collect::<Vec<_>>(),
        ["low", "high"]
    );
    assert_eq!(alpha.sources.len(), 2);
    assert!(alpha.sources.iter().all(|source| source.scope.is_some()));
    let plots = inputs.finish().unwrap();
    assert_eq!(
        plots
            .iter()
            .map(|panel| panel.guide.alpha.sources[0].glyphs[0].members.len())
            .collect::<Vec<_>>(),
        [4, 2]
    );
    source.preflight().unwrap();
    let ambiguous = source.labels(
        crate::Labels::raw()
            .alias("blue", "Same")
            .alias("green", "Same"),
    );
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    assert!(matches!(
        ambiguous.preflight(),
        Err(Error::Render(message)) if message.contains("distinct raw identities")
    ));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "actual routed parent guides distinguish equal-bounds descriptor changes, selective stability and cached/uncached frozen output"
)]
fn routed_facet_guides_invalidate_only_the_cells_consuming_their_descriptor() {
    use crate::decorations::plan::children::DISABLE_CHILD_REUSE;
    use crate::layout::kernel::{FeedbackKind, TrialOutcome};
    use crate::layout::{AddressRole, ComponentAddress};
    use crate::LegendComposition;

    let make = |field| {
        let plot = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0, 0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0, 1.0, 0.0]),
        )
        .hue(crate::CategoryValues::new(field, ["a", "b", "a", "b"]).unwrap())
        .facet(crate::FacetSpec::column(
            crate::CategoryValues::new("facet", ["first", "first", "second", "second"]).unwrap(),
        ))
        .labels(crate::Labels::raw().alias(field, "Cohort"))
        .legend(crate::LegendPolicy::Visible(LegendPosition::InsideTopRight))
        .size(1200, 800)
        .legend_composition(LegendComposition::Separate);
        plot.erase()
    };
    let source = make("first raw hue");
    let changed_source = make("second raw hue");
    let guide_owner = ComponentAddress::root()
        .child(AddressRole::InsideGuide, 0)
        .unwrap();
    let mut outputs = Vec::new();
    for disabled in [false, true] {
        DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
        let mut exercised = false;
        let root = crate::layout::plan_root(
            crate::layout::RootRequest::ExactSize {
                width: 1200,
                height: 800,
            },
            |stage| {
                let operation = stage.operation();
                let theme = source.description().resolve_theme()?;
                let (inputs, composition) =
                    crate::plots::shared_x::prepare_plot(source.description(), &operation, &theme)?;
                let (mut changed_inputs, _) = crate::plots::shared_x::prepare_plot(
                    changed_source.description(),
                    &operation,
                    &theme,
                )?;
                let changed = prepare_discrete_guide(
                    changed_source.description(),
                    PlotKind::Scatter,
                    &mut changed_inputs,
                    &operation,
                )?;
                let root = super::super::lifecycle::complete_plot_root(
                    source.description(),
                    theme,
                    crate::text::TextService::default(),
                    inputs,
                    composition,
                    stage,
                )?;
                Ok((root, changed))
            },
            |(root, changed_guide), assigned, round| {
                let crate::orchestrator::root::PreparedComposition::Facets(topology) =
                    &root.composition
                else {
                    panic!("facet topology")
                };
                assert_ne!(
                    topology
                        .parent_guide
                        .as_ref()
                        .map(|guide| &guide.descriptor),
                    changed_guide.descriptor.as_ref()
                );
                let changed_topology = crate::orchestrator::root::PreparedFacetTopology {
                    rows: topology.rows,
                    columns: topology.columns,
                    slots: crate::data::source::collect_source(topology.slots.iter().map(
                        |slot| {
                            use crate::orchestrator::root::DeclaredFacetSelector as S;
                            Ok(crate::orchestrator::root::FacetSlot {
                                selector: slot.selector.as_ref().map(|selector| match selector {
                                    S::Single { level } => S::Single { level: *level },
                                    S::Grid {
                                        row_level,
                                        column_level,
                                    } => S::Grid {
                                        row_level: *row_level,
                                        column_level: *column_level,
                                    },
                                }),
                                plot_id: slot.plot_id.clone(),
                            })
                        },
                    ))?,
                    producers: topology.producers.clone(),
                    shared_axis_members: topology.shared_axis_members.clone(),
                    shared_titles_from: topology.shared_titles_from.clone(),
                    parent_guide: changed_guide
                        .descriptor
                        .as_ref()
                        .map(|descriptor| {
                            crate::decorations::legend::discrete::GuideSlot::new(
                                descriptor.clone(),
                                &ComponentAddress::root(),
                                source.description().legend_position_spec(),
                            )
                        })
                        .transpose()?,
                };
                let theme = root.request();
                let mut caches = root.caches.borrow_mut();
                let children = &mut caches.children;
                let original = OwnedFacetPlan::measure(
                    source.description(),
                    root,
                    topology,
                    root.assignment_factory().assign(assigned.assigned_size()?),
                    &theme,
                    round,
                    children,
                    false,
                )
                .map_err(fit::PlanError::into_error)?;
                if !round.child_stable(&[ComponentAddress::root()]) {
                    children.finish_round(round);
                    return Ok(crate::layout::MeasuredLayout::new(
                        assigned.assigned_size()?,
                        original,
                    ));
                }
                exercised = true;
                let same = OwnedFacetPlan::measure(
                    source.description(),
                    root,
                    topology,
                    root.assignment_factory().assign(assigned.assigned_size()?),
                    &theme,
                    round,
                    children,
                    false,
                )
                .map_err(fit::PlanError::into_error)?;
                let MeasuredRootContent::Facets(original) = original.content else {
                    panic!("measured facet content")
                };
                let MeasuredRootContent::Facets(same) = same.content else {
                    panic!("measured facet content")
                };
                assert!(original.parent_guide.is_none() && same.parent_guide.is_none());
                assert!(original.children[0].plot.legend.is_some());
                assert!(same.children[0].plot.legend.is_some());
                for (first, repeated) in original.children.iter().zip(&same.children) {
                    assert_eq!(
                        std::sync::Arc::ptr_eq(&first.plot, &repeated.plot),
                        !disabled
                    );
                    if let (Some(first_legend), Some(repeated_legend)) =
                        (&first.plot.legend, &repeated.plot.legend)
                    {
                        assert_eq!(
                            first.plot.selected.guide.edge,
                            LegendPosition::InsideTopRight
                        );
                        assert_eq!(first.plot.selected.guide.allocation, None);
                        assert_eq!(repeated.plot.selected.guide.allocation, None);
                        assert_eq!(
                            first_legend.legend.available_width(),
                            repeated_legend.legend.available_width()
                        );
                    }
                }
                let changed_plan = OwnedFacetPlan::measure(
                    source.description(),
                    root,
                    &changed_topology,
                    root.assignment_factory().assign(assigned.assigned_size()?),
                    &theme,
                    round,
                    children,
                    false,
                )
                .map_err(fit::PlanError::into_error)?;
                let MeasuredRootContent::Facets(changed) = &changed_plan.content else {
                    panic!("measured facet content")
                };
                for (first, replacement) in same.children.iter().zip(&changed.children) {
                    assert_eq!(first.plot.panel_box, replacement.plot.panel_box);
                }
                assert!(!std::sync::Arc::ptr_eq(
                    &same.children[0].plot,
                    &changed.children[0].plot
                ));
                assert_eq!(
                    std::sync::Arc::ptr_eq(&same.children[1].plot, &changed.children[1].plot),
                    !disabled,
                );
                assert!(changed.children[0].plot.legend.is_some());
                assert!(changed.children[1].plot.legend.is_none());

                // The original root-level guide slot is outside both cells.
                // Its instability invalidates the routed consumer at unchanged
                // bounds, while the other inside-guide cell remains independent.
                let checkpoint = round.begin_trial()?;
                round.request_insets(
                    &guide_owner,
                    &guide_owner,
                    FeedbackKind::OwnedInk,
                    [1.0, 0.0, 0.0, 0.0],
                )?;
                let unstable = OwnedFacetPlan::measure(
                    source.description(),
                    root,
                    &changed_topology,
                    root.assignment_factory().assign(assigned.assigned_size()?),
                    &theme,
                    round,
                    children,
                    false,
                )
                .map_err(fit::PlanError::into_error)?;
                let MeasuredRootContent::Facets(unstable) = unstable.content else {
                    panic!("measured facet content")
                };
                assert!(!std::sync::Arc::ptr_eq(
                    &changed.children[0].plot,
                    &unstable.children[0].plot
                ));
                assert_eq!(
                    std::sync::Arc::ptr_eq(&changed.children[1].plot, &unstable.children[1].plot),
                    !disabled,
                );
                round.record_failure(FitScope::new(&guide_owner, 0).failure(
                    crate::layout::kernel::FitKind::TrackOrResidual,
                    "reject guide ancestor trial",
                ))?;
                assert!(matches!(
                    round.finish_trial(checkpoint),
                    TrialOutcome::Miss(_)
                ));
                children.finish_round(round);
                Ok(crate::layout::MeasuredLayout::new(
                    assigned.assigned_size()?,
                    changed_plan,
                ))
            },
            |(root, _), size, plan, stage| {
                let MeasuredRootContent::Facets(content) = &plan.content else {
                    panic!("measured facet content")
                };
                size.dimensions()?;
                let verifier = root.sealed_verifier()?;
                let replay =
                    crate::orchestrator::capture_calls(size, &verifier, stage, |backend| {
                        content.render(
                            source.description(),
                            PlotKind::Scatter,
                            &plan.frame,
                            backend,
                        )
                    })?;
                crate::orchestrator::validate_text_replay_plan(
                    1200,
                    800,
                    &replay,
                    "routed facet guide final Null",
                )?;
                Ok(replay)
            },
        )
        .unwrap();
        DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
        assert!(exercised);
        let mut svg = root.plan().take_svg_backend().unwrap();
        root.plan()
            .replay(1200, 800, &mut svg, "routed facet guide replay")
            .unwrap();
        outputs.push(svg.into_bytes().unwrap());
    }
    assert_eq!(outputs[0], outputs[1]);
}
