use super::*;
use crate::render::backend::TextAnchor;
use crate::theme::TitleAlignment::{Center, End, Start};

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
        let theme = crate::Theme::Publication.resolve();
        theme.prepare_font_environment().unwrap();
        let mut children = crate::decorations::plan::children::ChildPlans::facets(&source);
        crate::layout::plan_exact_root(
            source.width(),
            source.height(),
            |stage| crate::plots::shared_x::prepare_plot(&source, &stage.generation_resources()),
            |inputs, assigned, round| {
                let source = inputs.attach(source.clone())?;
                crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
                let plan = FacetPlan::new(&source, PlotKind::Scatter, &theme, round, &mut children)
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
                let (width, height) = assigned.dimensions();
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
fn legacy_parent_allocation_keeps_exact_fit_and_typed_owner() {
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [0.0]),
        crate::NumericValues::new("y", [0.0]),
    )
    .erase();
    let scope = FitScope::root()
        .child(crate::layout::AddressRole::FacetGrid, 0)
        .unwrap();
    for width in [80.0, 80.0 - 1.0 / 1024.0] {
        let outcome = planning::allocate_legacy_grid(
            plot.data(),
            crate::layout::PanelBox::new(0.0, width, 0.0, 60.0),
            1,
            1,
            &[],
            &scope,
        )
        .unwrap();
        match outcome {
            FitOutcome::Measured(cells) => {
                assert_eq!(cells[0].width().to_bits(), 80.0_f64.to_bits());
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
    use crate::font::canonical::OperationGenerationResources;
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [0.0]),
        crate::NumericValues::new("y", [0.0]),
    )
    .erase();
    let scope = FitScope::root();
    let bytes = u64::try_from(
        std::mem::size_of::<crate::layout::PanelBox>() + 6 * std::mem::size_of::<f64>(),
    )
    .unwrap();
    for (visits, storage, admitted) in [(7, bytes, true), (6, bytes, false), (7, bytes - 1, false)]
    {
        let resources = OperationGenerationResources::with_test_limits(visits, storage);
        let data = plot.data().with_axis_resources(&resources);
        let result = planning::allocate_legacy_grid(
            data,
            crate::layout::PanelBox::new(0.0, 79.0, 0.0, 60.0),
            1,
            1,
            &[],
            &scope,
        );
        assert_eq!(result.is_ok(), admitted);
        if admitted {
            assert!(matches!(result.unwrap(), FitOutcome::Miss(_)));
            assert_eq!(resources.byte_usage().unwrap(), bytes);
            assert!(planning::allocate_legacy_grid(
                data,
                crate::layout::PanelBox::new(0.0, 80.0, 0.0, 60.0),
                1,
                1,
                &[],
                &scope
            )
            .is_err());
        }
    }
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
    let theme = crate::Theme::Publication.resolve();
    theme.prepare_font_environment().unwrap();
    crate::layout::plan_exact_root(
        source.width(),
        source.height(),
        |_| Ok(()),
        |(), assigned, round| {
            crate::decorations::plan::children::CHILD_MEASUREMENTS
                .with(|calls| calls.borrow_mut().clear());
            let result = FacetPlan::new(
                &source,
                PlotKind::Scatter,
                &theme,
                round,
                &mut crate::decorations::plan::children::ChildPlans::facets(&source),
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
        |(), _, (), _| Ok(()),
    )
    .unwrap();
}
