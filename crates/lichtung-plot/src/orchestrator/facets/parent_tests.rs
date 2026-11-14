use super::*;

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
        |plot, inputs, request, operation| {
            let registered = &inputs.plots[1];
            assert_eq!(registered.policy.significance_indices, [1, 3]);
            let super::super::registration::PreparedFamily::Bars { marks, .. } = &registered.family
            else {
                panic!("prepared Bar family")
            };
            assert_eq!(marks.significance.len(), 1);
            assert_eq!(marks.significance[0].source_index, 0);
            let plot = plot.clone().without_facets();
            let result = plot
                .measure_selected_interior(
                    crate::decorations::legend::discrete::scoped::GuideUse::Local,
                    crate::spec::PlotRenderContext::new(
                        registered.data(operation).unwrap(),
                        crate::layout::PixelRect::new(50.0, 750.0, 50.0, 550.0),
                        request,
                    )
                    .with_registration(registered, inputs.intrinsic(registered)),
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
    let mut inputs =
        crate::plots::shared_x::prepare_plot(source.description(), &operation, Some(&theme))
            .unwrap();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let prepared = prepare_discrete_guide(
        &source,
        PlotKind::Interaction,
        &theme,
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
    use crate::decorations::plan::children::{ChildPlans, DISABLE_CHILD_REUSE};
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
        let theme = crate::Theme::publication().resolve().unwrap();
        let text = crate::text::TextService::default();
        let theme = crate::text::TextRequestContext::new(&theme, &text);
        let mut children = ChildPlans::facets();
        let mut exercised = false;
        let root = crate::layout::plan_root(
            crate::layout::RootRequest::ExactSize {
                width: 1200,
                height: 800,
            },
            |stage| {
                let operation = stage.operation();
                let mut inputs = crate::plots::shared_x::prepare_plot(
                    source.description(),
                    &operation,
                    Some(&theme),
                )?;
                let guide = prepare_discrete_guide(
                    &source,
                    PlotKind::Scatter,
                    &theme,
                    &mut inputs,
                    &operation,
                )?;
                let mut changed_inputs = crate::plots::shared_x::prepare_plot(
                    changed_source.description(),
                    &operation,
                    Some(&theme),
                )?;
                let changed = prepare_discrete_guide(
                    &changed_source,
                    PlotKind::Scatter,
                    &theme,
                    &mut changed_inputs,
                    &operation,
                )?;
                assert_ne!(guide.descriptor, changed.descriptor);
                let inputs = crate::layout::interior::PreparedInteriorInputs::from_registry(
                    inputs,
                    crate::layout::interior::InteriorRoute::classify(source.kind()),
                    guide.descriptor.clone(),
                )?;
                let requirements = crate::layout::foundational_plot_requirements(
                    Some(&inputs),
                    source.description(),
                    PlotKind::Scatter,
                    &theme,
                    &ComponentAddress::root(),
                    true,
                    true,
                    true,
                    Some(stage),
                )?;
                let mut presence = stage.reserve_scalar_presence_map()?;
                let mut census = crate::layout::PreparedTextCensus::materialize(
                    requirements,
                    Some(stage),
                    Some(&mut presence),
                )
                .map_err(|error| Error::Render(error.to_string()))?;
                theme
                    .text
                    .prepare_font_environment_from_census(&mut census, presence, stage)?;
                Ok((inputs, guide, changed, operation))
            },
            |(inputs, guide, changed_guide, operation), assigned, round| {
                let plot = source.clone();
                let original = OwnedFacetPlan::measure(
                    &plot,
                    operation,
                    PlotKind::Scatter,
                    &theme,
                    round,
                    &mut children,
                    guide,
                    false,
                    inputs,
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
                    &plot,
                    operation,
                    PlotKind::Scatter,
                    &theme,
                    round,
                    &mut children,
                    guide,
                    false,
                    inputs,
                )
                .map_err(fit::PlanError::into_error)?;
                let MeasuredRootContent::Facets(original) = original.content else {
                    panic!("measured facet content")
                };
                let MeasuredRootContent::Facets(same) = same.content else {
                    panic!("measured facet content")
                };
                for (first, repeated) in original.children.iter().zip(&same.children) {
                    assert_eq!(
                        std::sync::Arc::ptr_eq(&first.plot, &repeated.plot),
                        !disabled
                    );
                }
                let changed_plan = OwnedFacetPlan::measure(
                    &plot,
                    operation,
                    PlotKind::Scatter,
                    &theme,
                    round,
                    &mut children,
                    changed_guide,
                    false,
                    inputs,
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
                    &plot,
                    operation,
                    PlotKind::Scatter,
                    &theme,
                    round,
                    &mut children,
                    changed_guide,
                    false,
                    inputs,
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
            |_, _, plan, stage| {
                let MeasuredRootContent::Facets(content) = &plan.content else {
                    panic!("measured facet content")
                };
                let replay = crate::orchestrator::capture_text_replay_plan(
                    1200,
                    800,
                    "routed facet guide dependencies",
                    &theme,
                    stage,
                    |backend| content.render(&source, PlotKind::Scatter, &plan.frame, backend),
                )?;
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
