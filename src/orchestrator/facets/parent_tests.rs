use super::*;
use crate::render::backend::TextAnchor;
use crate::theme::TitleAlignment::{Center, End, Start};

#[test]
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
            .with_point_shape(crate::PointShape::TriangleUp)
            .unwrap(),
    ])
    .facet(crate::FacetSpec::column(
        crate::CategoryValues::new("facet", ["left", "right", "left", "left", "right", "left"])
            .unwrap(),
    ))
    .labels(crate::Labels::raw())
    .size(1200, 700)
    .legend_composition(LegendComposition::Separate);
    let source = source.erase();
    let operation = crate::font::canonical::OperationContext::new();
    let theme = crate::Theme::publication().resolve().unwrap();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &operation, Some(&theme)).unwrap();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let prepared =
        prepare_discrete_guide(&source, PlotKind::Interaction, &theme, &inputs, &operation)
            .unwrap();
    let guide = prepared.descriptor.unwrap();
    assert_eq!(
        (
            guide.composition,
            guide.counts.sections,
            guide.counts.entries
        ),
        (LegendComposition::Separate, 2, 5)
    );
    assert_eq!(
        guide.sections[0]
            .entries
            .iter()
            .map(|entry| entry.display_label.as_ref())
            .collect::<Vec<_>>(),
        ["one", "two"]
    );
    // First occupied facet contributes blue/green before the right facet's red.
    // The complete parent scientific hue domain is instead blue/red/green.
    assert_eq!(
        guide.sections[1]
            .entries
            .iter()
            .map(|entry| entry.display_label.as_ref())
            .collect::<Vec<_>>(),
        ["blue", "red", "green"]
    );
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
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
    reason = "complete roots exercise independent text roles and selected capture"
)]
fn outer_roles_use_one_root_producer_without_changing_centered_strips() {
    for (title, subtitle, caption, alignments) in [
        (
            "A longer first line\nShort",
            "Subtitle\nA much longer second line",
            "Common caption",
            [None, None],
        ),
        ("Title only", "", "", [Some(End), None]),
        ("", "", "", [None, None]),
        (
            "\u{5d0}\u{5d1}\n\u{5d2}",
            "Independent subtitle",
            "\u{5d0} caption",
            [Some(Start), Some(Center)],
        ),
    ] {
        let mut source = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0, 0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0, 1.0, 0.0]),
        )
        .facet(crate::FacetSpec::column(
            crate::CategoryValues::new(
                "facet",
                ["First\nShort", "First\nShort", "Second", "Second"],
            )
            .unwrap(),
        ))
        .labels(crate::Labels::raw())
        .title(title)
        .subtitle(subtitle)
        .caption(caption)
        .legend(crate::LegendPolicy::Hidden)
        .size(960, 640)
        .erase();
        if let Some(alignment) = alignments[0] {
            source = source.title_alignment(alignment);
        }
        if let Some(alignment) = alignments[1] {
            source = source.subtitle_alignment(alignment);
        }
        let theme = crate::Theme::publication().resolve().unwrap();
        theme.prepare_font_environment().unwrap();
        let mut children = crate::decorations::plan::children::ChildPlans::facets(&source);
        crate::layout::plan_exact_root(
            source.sizing_request().requested_dimensions().0,
            source.sizing_request().requested_dimensions().1,
            |stage| {
                let inputs = crate::plots::shared_x::prepare_plot(
                    &source,
                    &stage.operation(),
                    Some(&crate::theme::Theme::publication()),
                )?;
                let guide = prepare_discrete_guide(
                    &source,
                    PlotKind::Scatter,
                    &theme,
                    &inputs,
                    &stage.operation(),
                )?;
                Ok((inputs, guide))
            },
            |(inputs, guide), assigned, round| {
                let source = inputs.attach(source.clone())?;
                crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
                let plan = FacetPlan::new(
                    &source,
                    PlotKind::Scatter,
                    &theme,
                    round,
                    &mut children,
                    guide,
                    false,
                    None,
                )
                .map_err(fit::PlanError::into_error)?;
                crate::render::geometry::GEOMETRY_BACKEND_BEGINS
                    .with(|calls| assert_eq!(calls.get(), 0));
                assert_eq!(
                    plan.canvas_layout.root_text,
                    source.measured_canvas_layout(&theme)?.root_text
                );
                assert!(plan
                    .strip_text
                    .iter()
                    .all(|strip| strip.placement.anchor == TextAnchor::Middle));
                assert_eq!(plan.panel_plans.len(), 2);
                children.finish_round(round);
                Ok(crate::layout::MeasuredLayout::new(assigned, plan))
            },
            |_, assigned, plan, stage| {
                let (width, height) = assigned.dimensions()?;
                let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
                let projections = crate::layout::Projection::test_projection_calls();
                let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
                let replay = crate::orchestrator::capture_text_replay_plan(
                    width,
                    height,
                    "facet outer roles",
                    &theme,
                    stage,
                    |backend| plan.render(&source, PlotKind::Scatter, &theme, backend),
                )?;
                assert_eq!(
                    crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                    reads
                );
                assert_eq!(
                    crate::layout::Projection::test_projection_calls(),
                    projections
                );
                assert_eq!(
                    crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
                    shapes
                );
                let text = replay.retained_text();
                for role in [title, subtitle, caption]
                    .into_iter()
                    .filter(|role| !role.is_empty())
                {
                    assert_eq!(
                        text.iter().filter(|(_, text)| text.source == role).count(),
                        1
                    );
                }
                if caption == "Common caption" {
                    assert_eq!(
                        text.iter()
                            .find(|(_, text)| text.source == caption)
                            .unwrap()
                            .0
                            .anchor,
                        TextAnchor::Start
                    );
                    assert_eq!(
                        text.iter()
                            .find(|(_, text)| text.source == title)
                            .unwrap()
                            .0
                            .anchor,
                        TextAnchor::Middle
                    );
                }
                crate::orchestrator::validate_text_replay_plan(
                    width,
                    height,
                    &replay,
                    "facet outer roles",
                )?;
                Ok(())
            },
        )
        .unwrap();
    }
}

#[test]
fn measured_facet_tracks_keep_exact_fit_and_typed_owner() {
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [0.0]),
        crate::NumericValues::new("y", [0.0]),
    )
    .erase();
    let scope = FitScope::root()
        .child(crate::layout::AddressRole::FacetGrid, 0)
        .unwrap();
    let theme = crate::Theme::publication().resolve().unwrap();
    for width in [80.0, 80.0 - 1.0 / 1024.0] {
        let outcome = tracks::allocate(
            plot.data(),
            crate::layout::PanelBox::new(0.0, width, 0.0, 60.0),
            (1, 1),
            &[data::FacetPanel {
                panel_index: 0,
                row: 0,
                column: 0,
                cell: Vec::new(),
                rows: vec![0],
                source_rows: vec![0],
                panel: crate::layout::PanelBox::new(0.0, 1.0, 0.0, 1.0),
            }],
            &[(80.0, 60.0)],
            &[],
            &theme,
            &scope,
            None,
        )
        .unwrap();
        match outcome {
            FitOutcome::Measured(cells) => {
                assert_eq!(cells.cells[0].width().to_bits(), 80.0_f64.to_bits());
            }
            FitOutcome::Miss(failure) => {
                assert!(width < 80.0);
                assert_eq!(failure.reason.address, scope.address);
                assert_eq!(failure.reason.kind, FitKind::TrackOrResidual);
            }
        }
    }
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
        let mut children = ChildPlans::facets(&source);
        let mut exercised = false;
        let root = crate::layout::plan_exact_root(
            1200,
            800,
            |stage| {
                let operation = stage.operation();
                let inputs =
                    crate::plots::shared_x::prepare_plot(&source, &operation, Some(&theme))?;
                let guide = prepare_discrete_guide(
                    &source,
                    PlotKind::Scatter,
                    &theme,
                    &inputs,
                    &operation,
                )?;
                let changed_inputs = crate::plots::shared_x::prepare_plot(
                    &changed_source,
                    &operation,
                    Some(&theme),
                )?;
                let changed = prepare_discrete_guide(
                    &changed_source,
                    PlotKind::Scatter,
                    &theme,
                    &changed_inputs,
                    &operation,
                )?;
                assert_ne!(guide.descriptor, changed.descriptor);
                let requirements = crate::layout::foundational_plot_requirements(
                    Some(&inputs),
                    &source,
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
                theme.prepare_font_environment_from_census(&mut census, presence, stage)?;
                Ok((inputs, guide, changed))
            },
            |(inputs, guide, changed_guide), assigned, round| {
                let plot = inputs.attach(source.clone())?;
                let original = FacetPlan::new(
                    &plot,
                    PlotKind::Scatter,
                    &theme,
                    round,
                    &mut children,
                    guide,
                    false,
                    None,
                )
                .map_err(fit::PlanError::into_error)?;
                if !round.child_stable(&[ComponentAddress::root()]) {
                    children.finish_round(round);
                    return Ok(crate::layout::MeasuredLayout::new(assigned, original));
                }
                exercised = true;
                let same = FacetPlan::new(
                    &plot,
                    PlotKind::Scatter,
                    &theme,
                    round,
                    &mut children,
                    guide,
                    false,
                    None,
                )
                .map_err(fit::PlanError::into_error)?;
                for (first, repeated) in original.panel_plans.iter().zip(&same.panel_plans) {
                    assert_eq!(std::sync::Arc::ptr_eq(first, repeated), !disabled);
                }
                let changed = FacetPlan::new(
                    &plot,
                    PlotKind::Scatter,
                    &theme,
                    round,
                    &mut children,
                    changed_guide,
                    false,
                    None,
                )
                .map_err(fit::PlanError::into_error)?;
                for (first, replacement) in same.panel_plans.iter().zip(&changed.panel_plans) {
                    assert_eq!(first.panel_box, replacement.panel_box);
                }
                assert!(!std::sync::Arc::ptr_eq(
                    &same.panel_plans[0],
                    &changed.panel_plans[0]
                ));
                assert_eq!(
                    std::sync::Arc::ptr_eq(&same.panel_plans[1], &changed.panel_plans[1]),
                    !disabled,
                );
                assert!(changed.panel_plans[0].legend.is_some());
                assert!(changed.panel_plans[1].legend.is_none());

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
                let unstable = FacetPlan::new(
                    &plot,
                    PlotKind::Scatter,
                    &theme,
                    round,
                    &mut children,
                    changed_guide,
                    false,
                    None,
                )
                .map_err(fit::PlanError::into_error)?;
                assert!(!std::sync::Arc::ptr_eq(
                    &changed.panel_plans[0],
                    &unstable.panel_plans[0]
                ));
                assert_eq!(
                    std::sync::Arc::ptr_eq(&changed.panel_plans[1], &unstable.panel_plans[1]),
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
                Ok(crate::layout::MeasuredLayout::new(assigned, changed))
            },
            |_, _, plan, stage| {
                let replay = crate::orchestrator::capture_text_replay_plan(
                    1200,
                    800,
                    "routed facet guide dependencies",
                    &theme,
                    stage,
                    |backend| plan.render(&source, PlotKind::Scatter, &theme, backend),
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

#[test]
fn impossible_outer_title_returns_typed_fit_before_child_measurement() {
    let source = Plot::scatter(
        crate::NumericValues::new("x", [0.0]),
        crate::NumericValues::new("y", [0.0]),
    )
    .facet(crate::FacetSpec::column(
        crate::CategoryValues::new("facet", ["one"]).unwrap(),
    ))
    .title("M".repeat(200))
    .size(160, 120)
    .erase();
    let theme = crate::Theme::publication().resolve().unwrap();
    theme.prepare_font_environment().unwrap();
    crate::layout::plan_exact_root(
        source.sizing_request().requested_dimensions().0,
        source.sizing_request().requested_dimensions().1,
        |stage| {
            let inputs = crate::plots::shared_x::prepare_plot(
                &source,
                &stage.operation(),
                Some(&crate::theme::Theme::publication()),
            )?;
            prepare_discrete_guide(
                &source,
                PlotKind::Scatter,
                &theme,
                &inputs,
                &stage.operation(),
            )
        },
        |guide, assigned, round| {
            crate::decorations::plan::children::CHILD_MEASUREMENTS
                .with(|calls| calls.borrow_mut().clear());
            let result = FacetPlan::new(
                &source,
                PlotKind::Scatter,
                &theme,
                round,
                &mut crate::decorations::plan::children::ChildPlans::facets(&source),
                guide,
                false,
                None,
            );
            let Err(fit::PlanError::Fit(failure)) = result else {
                panic!("typed outer title fit");
            };
            assert_eq!(failure.reason.kind, FitKind::Text);
            assert_eq!(
                failure.reason.address,
                crate::layout::ComponentAddress::root()
                    .child(crate::layout::AddressRole::RootTitle, 0)?
            );
            crate::decorations::plan::children::CHILD_MEASUREMENTS
                .with(|calls| assert!(calls.borrow().is_empty()));
            Ok(crate::layout::MeasuredLayout::new(assigned, ()))
        },
        |_, _, (), _| Ok(()),
    )
    .unwrap();
}
