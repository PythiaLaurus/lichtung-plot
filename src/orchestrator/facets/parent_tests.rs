use super::*;
use crate::render::backend::TextAnchor;
use crate::theme::TitleAlignment::{Center, End, Start};

#[test]
fn ambiguous_faceted_legend_aliases_reject_before_font_discovery() {
    let source = Plot::scatter(
        crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        crate::NumericValues::new("y", [4.0, 3.0, 2.0, 1.0]),
    )
    .hue(crate::CategoryValues::new("hue", ["a", "a", "b", "b"]).unwrap())
    .facet(crate::FacetSpec::column(
        crate::CategoryValues::new("facet", ["left", "left", "right", "right"]).unwrap(),
    ))
    .labels(
        crate::Labels::humanized()
            .alias("a", "Same")
            .alias("b", "Same"),
    );
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    assert!(matches!(
        source.preflight(),
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
        source.title_alignment = alignments[0];
        source.subtitle_alignment = alignments[1];
        let theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        theme.prepare_font_environment().unwrap();
        let mut children = crate::decorations::plan::children::ChildPlans::facets(&source);
        crate::layout::plan_exact_root(
            source.width(),
            source.height(),
            |stage| {
                let inputs =
                    crate::plots::shared_x::prepare_plot(&source, &stage.generation_resources())?;
                let guide = prepare_discrete_guide(
                    &source,
                    PlotKind::Scatter,
                    &theme,
                    &inputs,
                    &stage.generation_resources(),
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
                children.finish_round(round)?;
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
        .child(
            crate::layout::AddressRole::FacetGrid,
            0,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap();
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
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
fn parent_topology_admits_exact_budget_and_does_not_refund_rejected_trials() {
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [0.0]),
        crate::NumericValues::new("y", [0.0]),
    )
    .erase();
    let scope = FitScope::root();
    let run = |limits: Option<(u128, u64)>, repeat: bool| {
        let theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        theme.prepare_font_environment().unwrap();
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        if let Some((visits, bytes)) = limits {
            resources.set_test_limits(visits, bytes);
        }
        let data = plot.data().with_axis_resources(&resources);
        let panels = [data::FacetPanel {
            panel_index: 0,
            row: 0,
            column: 0,
            cell: Vec::new(),
            rows: vec![0],
            source_rows: vec![0],
            panel: crate::layout::PanelBox::new(0.0, 1.0, 0.0, 1.0),
        }];
        let attempt = |width| {
            tracks::allocate(
                data,
                crate::layout::PanelBox::new(0.0, width, 0.0, 60.0),
                (1, 1),
                &panels,
                &[(80.0, 60.0)],
                &[],
                &theme,
                &scope,
                None,
            )
        };
        let result = attempt(79.0);
        let second = repeat.then(|| attempt(80.0));
        (resources, result, second)
    };
    let (resources, result, _) = run(None, false);
    assert!(matches!(result.unwrap(), FitOutcome::Miss(_)));
    let visits = resources.usage().unwrap().1;
    let bytes = resources.byte_usage().unwrap();
    assert!(visits > 0 && bytes > 0);
    assert!(run(Some((visits - 1, bytes)), false).1.is_err());
    assert!(run(Some((visits, bytes - 1)), false).1.is_err());
    let (_, exact, next) = run(Some((visits, bytes)), true);
    assert!(matches!(exact.unwrap(), FitOutcome::Miss(_)));
    assert!(
        next.unwrap().is_err(),
        "a rejected fit does not refund admitted work"
    );
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
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    theme.prepare_font_environment().unwrap();
    crate::layout::plan_exact_root(
        source.width(),
        source.height(),
        |stage| {
            let inputs =
                crate::plots::shared_x::prepare_plot(&source, &stage.generation_resources())?;
            prepare_discrete_guide(
                &source,
                PlotKind::Scatter,
                &theme,
                &inputs,
                &stage.generation_resources(),
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
                crate::layout::ComponentAddress::root().child(
                    crate::layout::AddressRole::RootTitle,
                    0,
                    round.resources()
                )?
            );
            crate::decorations::plan::children::CHILD_MEASUREMENTS
                .with(|calls| assert!(calls.borrow().is_empty()));
            Ok(crate::layout::MeasuredLayout::new(assigned, ()))
        },
        |_, _, (), _| Ok(()),
    )
    .unwrap();
}
