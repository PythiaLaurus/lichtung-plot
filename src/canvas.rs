//! Heterogeneous plot composition on one canvas.

use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;

use crate::Error;

use crate::decorations::legend::{Legend, LegendEntry, MeasuredContinuousGuide};
use crate::decorations::plan::children::MeasurementPhase;
use crate::layout::canvas::PanelRequirements;
use crate::layout::fit::{FitFailure, FitScope};
use crate::layout::kernel::{FeedbackKind, FitKind};
use crate::layout::{PanelBox, RootFramePlan};
use crate::render::backend::{with_layer, Backend, SurfaceRole};
use crate::scene::RenderedScene;
use crate::spec::AxisDomain;
use crate::theme::ResolvedTheme;
use crate::{CanvasLegendPolicy, Labels, Output, Plot, PlotKind, Theme};

mod plot_text;
mod sizing;
mod text_panel;
mod tracks;

#[cfg(test)]
type CoordinationTest = fn(
    usize,
    &Canvas,
    &ResolvedTheme,
    &mut crate::layout::kernel::FeedbackRound,
) -> Result<(), Error>;
#[cfg(test)]
thread_local! {
    static COORDINATION_TEST: std::cell::Cell<Option<CoordinationTest>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
mod natural_sizing_tests {
    use super::*;

    thread_local! {
        static HEIGHT_HISTORY: std::cell::RefCell<Vec<[f64; 4]>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    fn late_alignment(
        ordinal: usize,
        _source: &Canvas,
        theme: &ResolvedTheme,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        let resources = theme.prepared_font_environment()?.generation_resources();
        let cell = crate::layout::ComponentAddress::root().child(
            crate::layout::AddressRole::CanvasCell,
            0,
            &resources,
        )?;
        let grid = crate::layout::ComponentAddress::root().child(
            crate::layout::AddressRole::CanvasGrid,
            0,
            &resources,
        )?;
        let target = grid.child(crate::layout::AddressRole::CanvasCell, 0, &resources)?;
        HEIGHT_HISTORY.with(|values| -> Result<(), Error> {
            values
                .borrow_mut()
                .push(round.next_insets(&target, FeedbackKind::ContentHeight)?);
            Ok(())
        })?;
        let (owner, kind, value) = if ordinal < 2 {
            (
                cell,
                FeedbackKind::OwnedInk,
                [
                    0.0,
                    0.0,
                    0.0,
                    f64::from(u32::try_from(ordinal + 1).unwrap()),
                ],
            )
        } else {
            (
                grid.child(crate::layout::AddressRole::CanvasRangeGroup, 0, &resources)?,
                FeedbackKind::SharedAlignment,
                [120.0, 0.0, 0.0, 0.0],
            )
        };
        if let Some(request) = round.request_insets(&owner, &owner, kind, value)? {
            round.miss(
                &owner,
                crate::layout::kernel::FitKind::TrackOrResidual,
                &[Some(request)],
                "late coordinated constraint fixture",
            )?;
        }
        Ok(())
    }

    #[test]
    fn late_column_constraint_decreases_width_and_content_height_with_identical_replay() {
        let plot = || {
            Plot::scatter(
                crate::NumericValues::new("x", [0.0, 1.0]),
                crate::NumericValues::new("y", [1.0, 2.0]),
            )
            .hue(crate::CategoryValues::new("series", ["one", "two"]).unwrap())
        };
        let source = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, plot())
            .unwrap()
            .panel(0, 1, plot())
            .unwrap()
            .x_range_group([(0, 0), (0, 1)])
            .unwrap();
        let mut outputs = Vec::new();
        for disabled in [false, true] {
            HEIGHT_HISTORY.with(|values| values.borrow_mut().clear());
            COORDINATION_TEST.with(|hook| hook.set(Some(late_alignment)));
            crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
            let result = source.preflight_plan_with_sizing((Some(1200), None));
            COORDINATION_TEST.with(|hook| hook.set(None));
            crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
            let plan = result.unwrap();
            let history = HEIGHT_HISTORY.with(|values| values.borrow().clone());
            assert!(
                history.last().unwrap()[0] < history[2][0],
                "width must replace, not retain the earlier maximum: {history:?}"
            );
            assert!(
                history.last().unwrap()[2] < history[2][2],
                "preferred content height must contract: {history:?}"
            );
            let dimensions = plan.dimensions();
            let mut svg = plan.plan().take_svg_backend().unwrap();
            plan.plan()
                .replay(
                    dimensions.0,
                    dimensions.1,
                    &mut svg,
                    "late height replacement proof",
                )
                .unwrap();
            outputs.push((dimensions, svg.into_bytes().unwrap()));
        }
        assert_eq!(outputs[0], outputs[1]);
    }

    fn late_numeric_domain(
        ordinal: usize,
        _source: &Canvas,
        theme: &ResolvedTheme,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        let resources = theme.prepared_font_environment()?.generation_resources();
        let owner = crate::layout::ComponentAddress::root().child(
            crate::layout::AddressRole::CanvasCell,
            0,
            &resources,
        )?;
        if ordinal < 2 {
            if let Some(request) = round.request_insets(
                &owner,
                &owner,
                FeedbackKind::OwnedInk,
                [
                    0.0,
                    0.0,
                    0.0,
                    f64::from(u32::try_from(ordinal + 1).unwrap()),
                ],
            )? {
                round.miss(
                    &owner,
                    crate::layout::kernel::FitKind::TrackOrResidual,
                    &[Some(request)],
                    "late numeric domain fixture",
                )?;
            }
        } else if let Some(request) = round.request_domain(
            &owner,
            &owner,
            FeedbackKind::AutomaticDomainX,
            (0.0, 1_000_000.0),
        )? {
            round.miss(
                &owner,
                crate::layout::kernel::FitKind::Tick,
                &[Some(request)],
                "late numeric domain fixture",
            )?;
        }
        Ok(())
    }

    #[test]
    fn shared_numeric_feedback_is_installed_before_beeswarm_height_and_final_measurement() {
        let bee = || {
            Plot::beeswarm(
                crate::CategoryValues::new("group", ["a"; 40]).unwrap(),
                crate::NumericValues::new("x", (0..40).map(f64::from)),
                crate::NumericValues::new("color", [0.0; 40]),
            )
        };
        let source = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, bee())
            .unwrap()
            .panel(0, 1, bee())
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Hidden)
            .x_range_group([(0, 0), (0, 1)])
            .unwrap();
        let baseline = source
            .preflight_plan_with_sizing((Some(600), None))
            .unwrap()
            .dimensions();
        let mut outputs = Vec::new();
        for disabled in [false, true] {
            COORDINATION_TEST.with(|hook| hook.set(Some(late_numeric_domain)));
            crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
            let result = source.preflight_plan_with_sizing((Some(600), None));
            COORDINATION_TEST.with(|hook| hook.set(None));
            crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
            let plan = result.unwrap();
            assert!(
                plan.dimensions().1 > baseline.1,
                "coordinated occupied packing must enlarge content height"
            );
            let dimensions = plan.dimensions();
            let mut svg = plan.plan().take_svg_backend().unwrap();
            plan.plan()
                .replay(
                    dimensions.0,
                    dimensions.1,
                    &mut svg,
                    "shared numeric height proof",
                )
                .unwrap();
            outputs.push((dimensions, svg.into_bytes().unwrap()));
        }
        assert_eq!(outputs[0], outputs[1]);
    }

    #[test]
    fn coordinated_waterfall_shelves_are_reflowed_before_content_row_allocation() {
        waterfall_content_replay([100.0, -80.0, -5.0], Some(late_numeric_domain));
    }

    #[test]
    fn waterfall_shelf_root_overflow_uses_pending_mark_domain_clearance() {
        waterfall_content_replay([1.0, 2.0, 3.0], None);
    }

    fn waterfall_content_replay(deltas: [f64; 3], coordination: Option<CoordinationTest>) {
        let waterfall = || {
            Plot::waterfall(
                crate::CategoryValues::new("term", ["first", "second", "third"]).unwrap(),
                crate::NumericValues::new("delta", deltas),
                10.0,
            )
        };
        let source = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, waterfall())
            .unwrap()
            .panel(0, 1, waterfall())
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Hidden)
            .x_range_group([(0, 0), (0, 1)])
            .unwrap();
        let mut outputs = Vec::new();
        for disabled in [false, true] {
            COORDINATION_TEST.with(|hook| hook.set(coordination));
            crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
            let result = source.preflight_plan_with_sizing((Some(1000), None));
            COORDINATION_TEST.with(|hook| hook.set(None));
            crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
            let plan = result.unwrap();
            assert_eq!(plan.dimensions().0, 1000);
            let dimensions = plan.dimensions();
            let mut svg = plan.plan().take_svg_backend().unwrap();
            plan.plan()
                .replay(
                    dimensions.0,
                    dimensions.1,
                    &mut svg,
                    "coordinated Waterfall shelf proof",
                )
                .unwrap();
            outputs.push((dimensions, svg.into_bytes().unwrap()));
        }
        assert_eq!(outputs[0], outputs[1]);
    }

    #[test]
    fn private_sparse_canvas_roots_use_content_and_ignore_standalone_plot_sizes() {
        let source = Canvas::grid(2, 3).unwrap().panel(0, 0, Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]), crate::NumericValues::new("y", [1.0, 2.0])).size(99, 77)).unwrap()
            .text_panel(1, 2, TextPanel::new("Explanation", "A complete paragraph with legal break opportunities that stays in its declared trailing cell.").unwrap()).unwrap();
        for sizing in [(None, None), (Some(1200), None)] {
            let plan = source.preflight_plan_with_sizing(sizing).unwrap();
            assert!(plan.dimensions().0 > 0 && plan.dimensions().1 > 0);
            if let Some(width) = sizing.0 {
                assert_eq!(plan.dimensions().0, width);
            }
            assert_ne!(plan.dimensions(), (99, 77));
        }
        assert_eq!(source.preflight_plan().unwrap().dimensions(), (960, 640));
    }

    #[test]
    fn natural_sparse_mixed_grid_owns_spanning_headers_row_titles_and_shared_guide() {
        let source = Canvas::grid(2, 3).unwrap()
            .panel(0, 0, Plot::scatter(crate::NumericValues::new("x", [0.0, 1.0]),
                crate::NumericValues::new("y", [1.0, 2.0]))
                .hue(crate::CategoryValues::new("series", ["one", "two"]).unwrap()).size(1, 1)).unwrap()
            .text_panel(1, 2, TextPanel::new("Explanation", "First complete explanatory sentence. Second complete explanatory sentence with legal word boundaries.").unwrap()).unwrap()
            .column_header(0, 2, "A spanning column header with several legal word boundaries").unwrap()
            .row_header(0, 2, "Paired rows").unwrap()
            .row_title(1, "A complete independently owned row title").unwrap()
            .caption("First complete root caption paragraph.\nSecond complete root caption paragraph.")
            .legend_policy(crate::CanvasLegendPolicy::Shared);
        for width in [None, Some(1200)] {
            let plan = source.preflight_plan_with_sizing((width, None)).unwrap();
            if let Some(width) = width {
                assert_eq!(plan.dimensions().0, width);
            }
            let retained = plan.plan().retained_text();
            for expected in [
                "A spanning column header with several legal word boundaries",
                "Paired rows",
                "A complete independently owned row title",
                "First complete root caption paragraph.\nSecond complete root caption paragraph.",
            ] {
                assert!(retained.iter().any(|(_, text)| text.source == expected));
            }
        }
    }

    #[test]
    fn content_shared_guide_recovers_after_measured_gutters_without_reselecting_width() {
        let source = Canvas::grid(1, 1)
            .unwrap()
            .panel(
                0,
                0,
                Plot::scatter(
                    crate::NumericValues::new("x", [0.0, 1.0]),
                    crate::NumericValues::new("y", [1.0, 2.0]),
                )
                .hue(
                    crate::CategoryValues::new(
                        "series",
                        ["extraordinarycategory", "extraordinarycategory"],
                    )
                    .unwrap(),
                ),
            )
            .unwrap()
            .legend_policy(crate::CanvasLegendPolicy::Shared);
        crate::orchestrator::measurements::start();
        let plan = source
            .preflight_plan_with_sizing((Some(300), None))
            .unwrap();
        let work = crate::orchestrator::measurements::finish();
        assert_eq!(plan.dimensions().0, 300);
        assert!(work
            .guides
            .iter()
            .any(|trial| trial.edge == crate::LegendPosition::OutsideRight
                && trial.outcome == crate::orchestrator::measurements::GuideOutcome::Rejected));
        assert!(work
            .guides
            .iter()
            .any(|trial| trial.edge == crate::LegendPosition::OutsideTop
                && trial.outcome == crate::orchestrator::measurements::GuideOutcome::Accepted));
    }

    #[test]
    fn per_plot_guides_participate_in_mixed_sparse_content_roots_and_fixed_width_recovery() {
        let plot = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 2.0]),
        )
        .hue(
            crate::CategoryValues::new(
                "series",
                ["extraordinarycategory", "extraordinarycategory"],
            )
            .unwrap(),
        )
        .caption("A complete local caption with several legal break opportunities.");
        let build = |plot| {
            Canvas::grid(2, 2)
                .unwrap()
                .panel(0, 0, plot)
                .unwrap()
                .text_panel(
                    1,
                    1,
                    TextPanel::new(
                        "Explanation",
                        "A complete explanatory paragraph that reflows within its owned cell.",
                    )
                    .unwrap(),
                )
                .unwrap()
                .column_header(0, 2, "Guided scientific panel and independent prose")
                .unwrap()
                .row_title(1, "Independent second row")
                .unwrap()
        };
        let guided = build(plot.clone());
        let hidden = build(plot.clone().legend(crate::LegendPolicy::Hidden));
        CANVAS_ASSIGNED_PANEL_EVIDENCE.with(|evidence| evidence.borrow_mut().clear());
        let natural = guided.preflight_plan_with_sizing((None, None)).unwrap();
        let natural_data =
            CANVAS_ASSIGNED_PANEL_EVIDENCE.with(|evidence| evidence.borrow().last().unwrap().1);
        let natural_content = 3.0 * natural_data.width() / 4.0;
        assert!(
            natural_data.height() + 1.0 / 1024.0 >= natural_content
                && natural_data.height() < natural_content + 1.0,
            "Right guide must not keep the pre-guide scientific height: {natural_data:?}"
        );
        assert!(
            natural.dimensions().0
                > hidden
                    .preflight_plan_with_sizing((None, None))
                    .unwrap()
                    .dimensions()
                    .0
        );
        let fixed = guided
            .preflight_plan_with_sizing((Some(1000), None))
            .unwrap();
        assert_eq!(fixed.dimensions().0, 1000);
        assert!(fixed
            .plan()
            .retained_text()
            .iter()
            .any(|(_, text)| text.source == "Extraordinarycategory"));
        let narrow = Canvas::grid(1, 1).unwrap().panel(0, 0, plot).unwrap();
        CANVAS_ASSIGNED_PANEL_EVIDENCE.with(|evidence| evidence.borrow_mut().clear());
        crate::orchestrator::measurements::start();
        let plan = narrow
            .preflight_plan_with_sizing((Some(300), None))
            .unwrap();
        let work = crate::orchestrator::measurements::finish();
        assert_eq!(plan.dimensions().0, 300);
        assert!(work
            .guides
            .iter()
            .any(|trial| trial.edge == crate::LegendPosition::OutsideTop
                && trial.outcome == crate::orchestrator::measurements::GuideOutcome::Accepted));
        let data =
            CANVAS_ASSIGNED_PANEL_EVIDENCE.with(|evidence| evidence.borrow().last().unwrap().1);
        let content = 3.0 * data.width() / 4.0;
        assert!(
            data.height() + 1.0 / 1024.0 >= content && data.height() < content + 1.0,
            "the selected Top guide must leave the width-derived scientific height: {data:?}"
        );
    }

    #[test]
    fn per_plot_continuous_guides_have_complete_content_extent_in_mixed_rows() {
        let source = Canvas::grid(1, 2)
            .unwrap()
            .panel(
                0,
                0,
                Plot::beeswarm(
                    crate::CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
                    crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
                    crate::NumericValues::new("color", [0.0, 1.0, 0.0, 1.0]),
                ),
            )
            .unwrap()
            .text_panel(
                0,
                1,
                TextPanel::new(
                    "Explanation",
                    "An independent prose cell sharing the guided scientific row.",
                )
                .unwrap(),
            )
            .unwrap();
        for width in [None, Some(1400)] {
            let plan = source.preflight_plan_with_sizing((width, None)).unwrap();
            if let Some(width) = width {
                assert_eq!(plan.dimensions().0, width);
            }
            assert!(plan
                .plan()
                .retained_text()
                .iter()
                .any(|(_, text)| text.source == "Color"));
        }
    }

    #[test]
    fn nullable_heatmap_guide_wraps_its_title_and_missing_key_in_content_canvas() {
        let heatmap = Plot::heatmap(
            crate::CategoryValues::new("x", ["a", "b", "a", "b"]).unwrap(),
            crate::CategoryValues::new("y", ["first", "first", "second", "second"]).unwrap(),
            crate::NumericValues::nullable(
                "complete measured color response with independent legal word boundaries",
                [Some(1.0), None, Some(3.0), Some(4.0)],
            ),
        )
        .caption(
            "First complete local caption paragraph.\nSecond complete local caption paragraph.",
        );
        let source = Canvas::grid(2, 2)
            .unwrap()
            .panel(0, 0, heatmap)
            .unwrap()
            .text_panel(
                1,
                1,
                TextPanel::new(
                    "Explanation",
                    "Independent prose below the nullable guided grid.",
                )
                .unwrap(),
            )
            .unwrap();
        for width in [None, Some(900)] {
            let plan = source.preflight_plan_with_sizing((width, None)).unwrap();
            if let Some(width) = width {
                assert_eq!(plan.dimensions().0, width);
            }
            assert!(plan
                .plan()
                .retained_text()
                .iter()
                .any(|(_, text)| text.source == "Missing"));
        }
    }

    #[test]
    fn guided_plot_fills_a_taller_prose_row_without_inflating_its_own_preference() {
        let prose = "A complete scientific explanation with legal word boundaries and independent line reflow. ".repeat(40);
        let source = Canvas::grid(1, 2)
            .unwrap()
            .panel(
                0,
                0,
                Plot::scatter(
                    crate::NumericValues::new("x", [0.0, 1.0]),
                    crate::NumericValues::new("y", [1.0, 2.0]),
                )
                .hue(crate::CategoryValues::new("series", ["one", "two"]).unwrap()),
            )
            .unwrap()
            .text_panel(
                0,
                1,
                TextPanel::new("Explanation", prose.trim_end()).unwrap(),
            )
            .unwrap();
        for width in [None, Some(1000)] {
            CANVAS_ASSIGNED_PANEL_EVIDENCE.with(|evidence| evidence.borrow_mut().clear());
            let plan = source.preflight_plan_with_sizing((width, None)).unwrap();
            if let Some(width) = width {
                assert_eq!(plan.dimensions().0, width);
            }
            let (cell, data) =
                CANVAS_ASSIGNED_PANEL_EVIDENCE.with(|evidence| *evidence.borrow().last().unwrap());
            assert_eq!(cell.y0.to_bits(), data.y0.to_bits());
            assert_eq!(cell.y1.to_bits(), data.y1.to_bits());
            assert!(data.height() > 3.0 * data.width() / 4.0 + 100.0);
        }
    }

    #[test]
    fn content_height_feedback_replays_identical_guided_canvas_with_cache_disabled() {
        let source = Canvas::grid(1, 1)
            .unwrap()
            .panel(
                0,
                0,
                Plot::scatter(
                    crate::NumericValues::new("x", [0.0, 1.0]),
                    crate::NumericValues::new("y", [1.0, 2.0]),
                )
                .hue(
                    crate::CategoryValues::new(
                        "series",
                        ["extraordinarycategory", "extraordinarycategory"],
                    )
                    .unwrap(),
                )
                .caption("Complete local caption with legal boundaries."),
            )
            .unwrap();
        let mut outputs = Vec::new();
        for disabled in [false, true] {
            crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
            let result = source.preflight_plan_with_sizing((Some(300), None));
            crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
            let plan = result.unwrap();
            let dimensions = plan.dimensions();
            let mut svg = plan.plan().take_svg_backend().unwrap();
            plan.plan()
                .replay(
                    dimensions.0,
                    dimensions.1,
                    &mut svg,
                    "guided content cache proof",
                )
                .unwrap();
            outputs.push((dimensions, svg.into_bytes().unwrap()));
        }
        assert_eq!(outputs[0], outputs[1]);
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "all three root requests prove complete guide absence and retained scientific domains"
    )]
    fn hidden_native_continuous_guides_are_suppressed_in_all_three_requests() {
        let heatmap = Plot::heatmap(
            crate::CategoryValues::new("x", ["a", "b", "a", "b"]).unwrap(),
            crate::CategoryValues::new("y", ["first", "first", "second", "second"]).unwrap(),
            crate::NumericValues::nullable("color", [Some(1.0), None, Some(3.0), Some(4.0)]),
        );
        let beeswarm = Plot::beeswarm(
            crate::CategoryValues::new("group", ["a", "a", "b", "b"]).unwrap(),
            crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
            crate::NumericValues::new("color", [10.0, 20.0, 30.0, 40.0]),
        );
        let source = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, heatmap)
            .unwrap()
            .panel(0, 1, beeswarm)
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Hidden);
        for request in [(Some(1600), Some(1000)), (Some(1400), None), (None, None)] {
            let plan = source.preflight_plan_with_sizing(request).unwrap();
            let dimensions = plan.dimensions();
            let mut svg = plan.plan().take_svg_backend().unwrap();
            plan.plan()
                .replay(
                    dimensions.0,
                    dimensions.1,
                    &mut svg,
                    "Hidden native guide proof",
                )
                .unwrap();
            let bytes = svg.into_bytes().unwrap();
            let document =
                roxmltree::Document::parse(std::str::from_utf8(&bytes).unwrap()).unwrap();
            assert!(!document
                .descendants()
                .any(|node| node.has_tag_name("linearGradient")));
            assert_eq!(
                document
                    .descendants()
                    .filter(|node| node.has_tag_name("rect")
                        && !node
                            .parent()
                            .is_some_and(|parent| parent.has_tag_name("clipPath")))
                    .count(),
                3,
                "only the root and two Plot surfaces remain; no guide frame"
            );
            assert!(
                document
                    .descendants()
                    .filter(|node| node.has_tag_name("polygon"))
                    .all(|node| node
                        .ancestors()
                        .any(|parent| parent.attribute("clip-path").is_some())),
                "there must be no unclipped missing-key polygon"
            );
            assert!(!plan
                .plan()
                .retained_text()
                .iter()
                .any(|(_, text)| text.source == "Color" || text.source == "Missing"));
            assert!(plan
                .prepared()
                .text_census
                .requirements
                .iter()
                .all(|entry| !matches!(
                    entry.text_role,
                    crate::layout::PreparedTextRole::LegendTitle
                        | crate::layout::PreparedTextRole::LegendLabel
                )));
            let resources = plan
                .prepared()
                .themes
                .root()
                .prepared_font_environment()
                .unwrap()
                .generation_resources();
            let heat = plan.prepared().interiors[0]
                .as_ref()
                .unwrap()
                .heatmap
                .as_ref()
                .unwrap()
                .panels
                .first()
                .unwrap();
            let bee = plan.prepared().interiors[1]
                .as_ref()
                .unwrap()
                .beeswarm
                .as_ref()
                .unwrap()
                .panels
                .first()
                .unwrap();
            let data0 = source.panels[0]
                .as_ref()
                .unwrap()
                .data()
                .with_axis_resources(&resources);
            let data1 = source.panels[1]
                .as_ref()
                .unwrap()
                .data()
                .with_axis_resources(&resources);
            assert_eq!(
                heat.1
                    .guide_semantics(plan.prepared().themes.plot(0).unwrap(), data0)
                    .unwrap()
                    .domain,
                (1.0, 4.0)
            );
            assert_eq!(bee.1.guide_semantics(data1).unwrap().domain, (10.0, 40.0));
        }
    }
}

#[cfg(test)]
pub(crate) mod admission_tests;

#[cfg(test)]
thread_local! {
    pub(crate) static CANVAS_THEME_EVIDENCE: std::cell::RefCell<Vec<(usize, &'static str, usize)>> = const { std::cell::RefCell::new(Vec::new()) };
    pub(crate) static CANVAS_SHARED_LEGEND_THEME_EVIDENCE: std::cell::RefCell<Vec<(usize, &'static str)>> = const { std::cell::RefCell::new(Vec::new()) };
    pub(crate) static CANVAS_SCENE_RENDER_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    pub(crate) static CANVAS_ASSIGNED_PANEL_EVIDENCE: std::cell::RefCell<Vec<(PanelBox, PanelBox)>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
fn record_canvas_theme(owner: usize, stage: &'static str, theme: &ResolvedTheme) {
    CANVAS_THEME_EVIDENCE.with(|evidence| {
        evidence
            .borrow_mut()
            .push((owner, stage, std::ptr::from_ref(theme).addr()));
    });
}

#[cfg(test)]
fn record_shared_legend_theme(owner: usize, theme: &ResolvedPlotTheme) {
    let provenance = match theme {
        ResolvedPlotTheme::InheritedRoot => "inherited-root",
        ResolvedPlotTheme::ExplicitPlot(_) => "explicit-plot",
    };
    CANVAS_SHARED_LEGEND_THEME_EVIDENCE.with(|evidence| {
        evidence.borrow_mut().push((owner, provenance));
    });
}

struct PlannedCanvasPanel {
    index: usize,
    kind: PlotKind,
    canvas_cell: PanelBox,
    panel_plan: Arc<crate::decorations::plan::PanelPlan>,
    local_text: plot_text::PlotLocalTextPlan,
    header_layer: String,
    legend: Option<crate::decorations::legend::LegendDrawPlan>,
    layers: [String; 3],
}

type PreparedHueGroups = Vec<(String, Vec<String>)>;
use crate::decorations::legend::descriptor::{PreparedDiscreteGuide, PreparedGuideDescriptor};

struct CanvasPlan {
    dimensions: (u32, u32),
    themes: Arc<ResolvedCanvasThemes>,
    plots: Vec<Option<Plot>>,
    canvas_layout: RootFramePlan,
    #[cfg(test)]
    shared_legend: Option<crate::decorations::plan::ResolvedLegend>,
    shared_legend_draw: Option<crate::decorations::legend::LegendDrawPlan>,
    panels: Vec<PlannedCanvasPanel>,
    text_panels: Vec<TextPanelPlan>,
    headers: Vec<CanvasHeaderPlan>,
    row_titles: Vec<CanvasHeaderPlan>,
}

struct PreparedCanvasInputs {
    themes: Arc<ResolvedCanvasThemes>,
    text_census: crate::layout::PreparedTextCensus,
    root_text: crate::decorations::title_block::plan::RootTitleBlockIntrinsics,
    shared_x: Vec<Option<(f64, f64)>>,
    point_path_hue_groups: Option<Arc<PreparedHueGroups>>,
    shared_guide: Option<PreparedGuideDescriptor>,
    interiors: Vec<Option<crate::layout::interior::PreparedInteriorInputs>>,
    text_panel_intrinsics: Vec<Option<text_panel::TextPanelIntrinsics>>,
    plot_intrinsics: Vec<Option<crate::decorations::title_block::plan::RootTitleBlockIntrinsics>>,
}

struct PreparedCanvasRound<'a> {
    content_sizing: bool,
    measurements: Option<&'a sizing::HeightMeasurements>,
    interiors: &'a [Option<crate::layout::interior::PreparedInteriorInputs>],
    themes: Arc<ResolvedCanvasThemes>,
    canvas_layout: RootFramePlan,
    plots: Vec<Option<Plot>>,
    shared_legend: Option<(String, Arc<[LegendEntry]>)>,
    shared_guide: Option<&'a PreparedGuideDescriptor>,
    body_probes: Vec<Option<Arc<crate::spec::PlotRenderResult>>>,
    shared_x: &'a [Option<(f64, f64)>],
    text_panel_intrinsics: &'a [Option<text_panel::TextPanelIntrinsics>],
    plot_intrinsics:
        &'a [Option<crate::decorations::title_block::plan::RootTitleBlockIntrinsics>],
}

enum MeasuredCanvasGuide {
    Discrete(Legend),
    Continuous(MeasuredContinuousGuide),
}

impl MeasuredCanvasGuide {
    fn dimensions(&self) -> (f64, f64) {
        match self {
            Self::Discrete(legend) => (legend.compute_box_width(), legend.compute_box_height()),
            Self::Continuous(guide) => (guide.width, guide.height),
        }
    }
}

enum SharedMeasuredGuide {
    Discrete(crate::decorations::plan::ResolvedLegend),
    Continuous {
        guide: MeasuredContinuousGuide,
        bounds: PanelBox,
    },
}

impl SharedMeasuredGuide {
    fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        match self {
            Self::Discrete(legend) => legend.observe_feedback(round),
            Self::Continuous { guide, bounds } => {
                round.observe_box(*bounds)?;
                guide.observe_feedback(round)
            }
        }
    }

    fn select(
        &self,
        data: crate::data::PlotData<'_>,
    ) -> Result<crate::decorations::legend::LegendDrawPlan, Error> {
        match self {
            Self::Discrete(legend) => {
                let (x, y) = legend.bounds.anchor();
                legend.legend.select(x, y, data)
            }
            Self::Continuous { guide, bounds } => guide.select(bounds.x0, bounds.y0, data),
        }
    }
}

#[cfg(test)]
mod d1_tests {
    use super::*;

    #[test]
    fn per_plot_guides_trace_distinct_owners_and_root_rounds() {
        use crate::orchestrator::measurements::{GuideOutcome, GuidePhase};
        let plot = |offset| {
            Plot::scatter(
                crate::NumericValues::new("x", [offset, offset + 1.0]),
                crate::NumericValues::new("y", [2.0, 3.0]),
            )
            .hue(crate::CategoryValues::new("cohort", ["first", "second"]).unwrap())
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, plot(1.0))
            .unwrap()
            .panel(0, 1, plot(5.0))
            .unwrap()
            .size(480, 640)
            .legend_policy(CanvasLegendPolicy::PerPlot);
        crate::orchestrator::measurements::start();
        let result = canvas.preflight();
        let work = crate::orchestrator::measurements::finish();
        result.unwrap();
        let first = crate::layout::ComponentAddress::root()
            .child(
                crate::layout::AddressRole::CanvasCell,
                0,
                &crate::font::canonical::OperationGenerationResources::new(),
            )
            .unwrap();
        let second = crate::layout::ComponentAddress::root()
            .child(
                crate::layout::AddressRole::CanvasCell,
                1,
                &crate::font::canonical::OperationGenerationResources::new(),
            )
            .unwrap();
        // Complete provisional gutters now give the final assigned columns in
        // the first round; only publishing those root inputs needs a retry.
        assert_eq!(work.roots.len(), 2);
        assert_eq!(work.roots[0].guides, 0..2);
        assert!(!work.roots[0].successful);
        assert_eq!(work.roots[1].guides, 2..4);
        assert!(work.roots[1].successful);
        assert_eq!(work.gutters.len(), 2);
        assert!(work.gutters.iter().all(|gutter| {
            gutter.consumed.iter().all(|value| value.to_bits() == 0)
                && gutter.requested[0] > 0.0
                && gutter.requested[2] > 0.0
        }));
        for (attempt, (owner, round, edge, outcome, phase)) in work.guides.iter().zip([
            (
                &first,
                0,
                crate::LegendPosition::OutsideRight,
                GuideOutcome::Provisional,
                GuidePhase::Body,
            ),
            (
                &second,
                0,
                crate::LegendPosition::OutsideRight,
                GuideOutcome::Provisional,
                GuidePhase::Body,
            ),
            (
                &first,
                1,
                crate::LegendPosition::OutsideRight,
                GuideOutcome::Accepted,
                GuidePhase::Body,
            ),
            (
                &second,
                1,
                crate::LegendPosition::OutsideRight,
                GuideOutcome::Accepted,
                GuidePhase::Body,
            ),
        ]) {
            assert_eq!(&attempt.owner, owner);
            assert_eq!(attempt.root_round, round);
            assert_eq!(attempt.edge, edge);
            assert_eq!(attempt.outcome, outcome);
            assert_eq!(attempt.phase, phase);
            assert!(!attempt.compact);
        }
        assert_eq!(work.guides.len(), 4);
    }

    #[test]
    fn shared_policy_without_a_guide_allocates_cells_without_guide_trials() {
        let plot = || {
            Plot::scatter(
                crate::NumericValues::new("x", [1.0, 2.0]),
                crate::NumericValues::new("y", [2.0, 3.0]),
            )
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, plot())
            .unwrap()
            .panel(0, 1, plot())
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        crate::orchestrator::measurements::start();
        let result = canvas.preflight();
        let observed = crate::orchestrator::measurements::finish();
        result.unwrap();
        assert!(observed.guides.is_empty());
        assert!(observed.roots.last().is_some_and(|round| round.successful));
    }

    #[test]
    fn continuous_shared_domains_merge_before_fonts_without_choosing_one_panel() {
        let plot = |values: [f64; 2]| {
            Plot::heatmap(
                crate::CategoryValues::new("column", ["a", "b"]).unwrap(),
                crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
                crate::NumericValues::new("fill", values),
            )
        };
        let run = |first, second| {
            let canvas = Canvas::grid(1, 2)
                .unwrap()
                .panel(0, 0, first)
                .unwrap()
                .panel(0, 1, second)
                .unwrap()
                .legend_policy(CanvasLegendPolicy::Shared);
            crate::layout::plan_exact_root(
                1200,
                640,
                |stage| {
                    let themes = ResolvedCanvasThemes::new(
                        canvas.theme,
                        &canvas.panels,
                        &stage.generation_resources(),
                    )?;
                    let (interiors, _) = canvas.prepare_interiors(stage, Some(&themes))?;
                    canvas.validate_shared_guide_kinds(&interiors)?;
                    canvas.prepare_continuous_shared_guide(
                        &themes,
                        &interiors,
                        &stage.generation_resources(),
                    )
                },
                |_, assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
                |descriptor, _, (), _| Ok(descriptor.clone()),
            )
            .map(|root| root.plan().clone())
        };
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let merged = run(plot([0.0, 2.0]), plot([1.0, 5.0])).unwrap().unwrap();
        assert_eq!(merged.field, "fill");
        assert_eq!(merged.title, "Fill");
        assert_eq!(merged.domain, (0.0, 5.0));
        assert!(!merged.exact);
        assert_eq!(merged.labels, ["0", "5"]);
        let exact = run(plot([0.0, 2.0]).color_limits(0.0, 5.0), plot([1.0, 5.0]))
            .unwrap()
            .unwrap();
        assert_eq!(exact.domain, (0.0, 5.0));
        assert!(exact.exact);
        assert_eq!(exact.labels, merged.labels);
        assert!(matches!(
            run(plot([0.0, 2.0]).color_limits(0.0, 3.0), plot([1.0, 5.0])),
            Err(Error::Data(message)) if message.contains("excludes an automatic")
        ));
        assert!(matches!(
            run(plot([0.0, 2.0]).color_limits(0.0, 3.0),
                plot([1.0, 5.0]).color_limits(0.0, 5.0)),
            Err(Error::Data(message)) if message.contains("exact domains disagree")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn shared_diverging_guide_projects_one_final_center_stop_for_unequal_child_domains() {
        let plot = |values| {
            Plot::heatmap(
                crate::CategoryValues::new("column", ["a", "b"]).unwrap(),
                crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
                crate::NumericValues::new("fill", values),
            )
            .scale(crate::HeatmapScale::Diverging { center: 0.0 })
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, plot([0.0, 2.0]).color_limits(-1.0, 3.0))
            .unwrap()
            .panel(0, 1, plot([-1.0, 1.0]))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared)
            .size(1200, 640);
        let root = canvas.preflight_plan().unwrap();
        let Some(PreparedGuideDescriptor::Continuous(guide)) =
            root.prepared().shared_guide.as_ref()
        else {
            panic!("shared diverging guide")
        };
        assert_eq!(guide.domain, (-1.0, 3.0));
        assert_eq!(guide.labels, ["-1", "0", "3"]);
        assert_eq!(guide.stops.len(), 5);
        assert_eq!(guide.stops[1].offset.to_bits(), 0.25_f64.to_bits());
        assert_eq!(guide.stops[1].color, (247, 247, 247));
        assert_eq!(guide.stops[2].offset.to_bits(), 0.5_f64.to_bits());
        assert_eq!(guide.stops[3].offset.to_bits(), 0.75_f64.to_bits());
        canvas.render_scene().unwrap();
    }

    #[test]
    fn shared_diverging_guide_replays_identically_with_child_cache_disabled() {
        let plot = || {
            Plot::heatmap(
                crate::CategoryValues::new("column", ["a", "b"]).unwrap(),
                crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
                crate::NumericValues::new("fill", [-1.0, 2.0]),
            )
            .scale(crate::HeatmapScale::Diverging { center: 0.0 })
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, plot())
            .unwrap()
            .panel(0, 1, plot())
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared)
            .size(1200, 640);
        let mut outputs = Vec::new();
        for disabled in [false, true] {
            crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
            let result = canvas.render_scene();
            crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
            outputs.push(result.unwrap().svg_bytes().to_vec());
        }
        assert_eq!(outputs[0], outputs[1]);
    }

    #[test]
    fn shared_exact_constant_heatmap_retains_one_neutral_label_and_two_neutral_stops() {
        let plot = || {
            Plot::heatmap(
                crate::CategoryValues::new("column", ["a", "b"]).unwrap(),
                crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
                crate::NumericValues::new("fill", [0.0, 0.0]),
            )
            .scale(crate::HeatmapScale::Diverging { center: 0.0 })
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, plot().color_limits(0.0, 0.0))
            .unwrap()
            .panel(0, 1, plot())
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared)
            .size(1200, 640);
        let root = canvas.preflight_plan().unwrap();
        let Some(PreparedGuideDescriptor::Continuous(guide)) =
            root.prepared().shared_guide.as_ref()
        else {
            panic!("constant shared diverging guide")
        };
        assert_eq!(guide.labels, ["0"]);
        assert_eq!(guide.stops.len(), 2);
        assert!(guide.stops.iter().all(|stop| stop.color == (247, 247, 247)));
        canvas.render_scene().unwrap();
    }

    #[test]
    fn continuous_shared_guide_rejects_incompatible_gradients_before_fonts() {
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(
                0,
                0,
                Plot::heatmap(
                    crate::CategoryValues::new("column", ["a", "b"]).unwrap(),
                    crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
                    crate::NumericValues::new("fill", [1.0, 2.0]),
                ),
            )
            .unwrap()
            .panel(
                0,
                1,
                Plot::beeswarm(
                    crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
                    crate::NumericValues::new("x", [1.0, 2.0]),
                    crate::NumericValues::new("fill", [1.0, 2.0]),
                ),
            )
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let result = crate::layout::plan_exact_root(
            1200,
            640,
            |stage| {
                let themes = ResolvedCanvasThemes::new(
                    canvas.theme,
                    &canvas.panels,
                    &stage.generation_resources(),
                )?;
                let (interiors, _) = canvas.prepare_interiors(stage, Some(&themes))?;
                canvas.validate_shared_guide_kinds(&interiors)?;
                canvas.prepare_continuous_shared_guide(
                    &themes,
                    &interiors,
                    &stage.generation_resources(),
                )
            },
            |_, assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |_, _, (), _| Ok(()),
        );
        assert!(matches!(
            result,
            Err(Error::Data(message)) if message.contains("incompatible identity, title, scale")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn custom_heatmap_and_beeswarm_share_one_gradient_or_reject_conflicts_before_fonts() {
        use crate::render::backend::GradientStop;

        let gradient = [
            GradientStop::new(0.0, (0, 0, 0)),
            GradientStop::new(0.5, (255, 0, 0)),
            GradientStop::new(1.0, (255, 255, 255)),
        ];
        let canvas = |bee_gradient: &[GradientStop]| {
            Canvas::grid(1, 2)
                .unwrap()
                .panel(
                    0,
                    0,
                    Plot::heatmap(
                        crate::CategoryValues::new("column", ["a", "b"]).unwrap(),
                        crate::CategoryValues::new("row", ["only", "only"]).unwrap(),
                        crate::NumericValues::new("fill", [0.0, 1.0]),
                    )
                    .theme(Theme::Nature)
                    .with_private_gradient(gradient.to_vec()),
                )
                .unwrap()
                .panel(
                    0,
                    1,
                    Plot::beeswarm(
                        crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
                        crate::NumericValues::new("x", [0.0, 1.0]),
                        crate::NumericValues::new("fill", [1.0, 3.0]),
                    )
                    .theme(Theme::Science)
                    .with_private_gradient(bee_gradient.to_vec()),
                )
                .unwrap()
                .legend_policy(CanvasLegendPolicy::Shared)
                .size(1200, 640)
        };
        let compatible = canvas(&gradient);
        let root = compatible.preflight_plan().unwrap();
        let Some(PreparedGuideDescriptor::Continuous(guide)) =
            root.prepared().shared_guide.as_ref()
        else {
            panic!("custom continuous Shared guide")
        };
        assert_eq!(guide.domain, (0.0, 3.0));
        assert_eq!(guide.stops.first().unwrap().color, (0, 0, 0));
        assert_eq!(guide.stops.last().unwrap().color, (255, 255, 255));
        assert_eq!(guide.stops.len(), 4);
        let bee_color = root.prepared().interiors[1]
            .as_ref()
            .unwrap()
            .beeswarm
            .as_ref()
            .unwrap()
            .panels[0]
            .1
            .color_for_source_value(1.0)
            .unwrap();
        assert_eq!(guide.stops[1].color, bee_color);
        let rendered =
            String::from_utf8(compatible.render_scene().unwrap().svg_bytes().to_vec()).unwrap();
        let (r, g, b) = bee_color;
        let paint = format!("#{r:02X}{g:02X}{b:02X}");
        assert!(rendered.matches(&format!("fill=\"{paint}\"")).count() >= 2);

        let mut conflicting = gradient;
        conflicting[1].color = (0, 0, 255);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        assert!(matches!(
            canvas(&conflicting).preflight(),
            Err(Error::Data(message)) if message.contains("incompatible identity, title, scale")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn merged_continuous_guide_text_is_censused_under_the_canvas_owner() {
        let plot = |values: [f64; 2]| {
            Plot::heatmap(
                crate::CategoryValues::new("column", ["a", "b"]).unwrap(),
                crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
                crate::NumericValues::new("fill", values),
            )
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, plot([0.0, 2.0]))
            .unwrap()
            .panel(0, 1, plot([1.0, 5.0]))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared)
            .size(1200, 640);
        let root = canvas.preflight_plan().unwrap();
        let Some(PreparedGuideDescriptor::Continuous(guide)) =
            root.prepared().shared_guide.as_ref()
        else {
            panic!("merged guide must retain its numeric domain");
        };
        assert_eq!(guide.domain, (0.0, 5.0));
        let resources = crate::font::canonical::OperationGenerationResources::new();
        for (index, input) in root.prepared().interiors.iter().enumerate() {
            let marks = &input.as_ref().unwrap().heatmap.as_ref().unwrap().panels[0].1;
            assert_eq!(
                marks
                    .guide_semantics(
                        root.prepared().themes.plot(index).unwrap(),
                        canvas.panels[index]
                            .as_ref()
                            .unwrap()
                            .data()
                            .with_axis_resources(&resources),
                    )
                    .unwrap()
                    .domain,
                guide.domain
            );
            assert_eq!(marks.guide_labels().collect::<Vec<_>>(), ["0", "5"]);
        }
        let sources = root
            .prepared()
            .text_census
            .requirements
            .iter()
            .filter(|requirement| {
                requirement.component_address.leaf_role() == crate::layout::AddressRole::GuideLabel
            })
            .map(|requirement| requirement.source.as_str())
            .collect::<Vec<_>>();
        assert_eq!(sources, ["0", "5"]);
    }

    #[test]
    fn shared_beeswarm_children_select_one_color_for_the_same_numeric_value() {
        let plot = |values| {
            Plot::beeswarm(
                crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
                crate::NumericValues::new("x", [0.0, 1.0]),
                crate::NumericValues::new("fill", values),
            )
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, plot([0.0, 1.0]))
            .unwrap()
            .panel(0, 1, plot([1.0, 5.0]))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared)
            .size(1200, 640);
        let root = canvas.preflight_plan().unwrap();
        let Some(PreparedGuideDescriptor::Continuous(guide)) =
            root.prepared().shared_guide.as_ref()
        else {
            panic!("prepared continuous guide must own the shared domain");
        };
        assert_eq!(guide.domain, (0.0, 5.0));
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let colors = root
            .prepared()
            .interiors
            .iter()
            .enumerate()
            .map(|(index, input)| {
                let marks = &input.as_ref().unwrap().beeswarm.as_ref().unwrap().panels[0].1;
                assert_eq!(
                    marks
                        .guide_semantics(
                            canvas.panels[index]
                                .as_ref()
                                .unwrap()
                                .data()
                                .with_axis_resources(&resources),
                        )
                        .unwrap()
                        .domain,
                    guide.domain
                );
                assert_eq!(marks.guide_labels().collect::<Vec<_>>(), ["5", "0"]);
                marks.color_for_source_value(1.0).unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(colors[0], colors[1]);
        assert_ne!(colors[0], guide.stops.last().unwrap().color);
        let shared_value = guide
            .stops
            .iter()
            .find(|stop| stop.offset.to_bits() == 0.2_f64.to_bits())
            .expect("both children retain a sampled color for the shared value");
        assert_eq!(colors[0], shared_value.color);
    }

    #[test]
    fn constant_shared_beeswarm_uses_one_midpoint_for_marks_and_gradient() {
        let bee = || {
            Plot::beeswarm(
                crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
                crate::NumericValues::new("x", [0.0, 1.0]),
                crate::NumericValues::new("fill", [3.0, 3.0]),
            )
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, bee())
            .unwrap()
            .panel(0, 1, bee())
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared)
            .size(1200, 640);
        let root = canvas.preflight_plan().unwrap();
        let Some(PreparedGuideDescriptor::Continuous(guide)) =
            root.prepared().shared_guide.as_ref()
        else {
            panic!("constant shared Beeswarm guide")
        };
        let color = root.prepared().interiors[0]
            .as_ref()
            .unwrap()
            .beeswarm
            .as_ref()
            .unwrap()
            .panels[0]
            .1
            .color_for_source_value(3.0)
            .unwrap();
        assert_eq!(guide.domain, (3.0, 3.0));
        assert_eq!(guide.stops[0].color, color);
        assert_eq!(guide.stops[1].color, color);
    }

    #[test]
    fn scatter_shared_hue_order_is_retained_before_fonts_and_includes_hidden_marks() {
        let scatter = |hue: &str| {
            Plot::scatter(
                crate::NumericValues::new("x", [0.0, 1.0]),
                crate::NumericValues::new("y", [1.0, 2.0]),
            )
            .hue(crate::CategoryValues::new("cohort", [hue, hue]).unwrap())
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, scatter("hidden").legend(crate::LegendPolicy::Hidden))
            .unwrap()
            .panel(0, 1, scatter("visible"))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        crate::layout::plan_exact_root(
            960,
            640,
            |stage| {
                let (interiors, _) = canvas.prepare_interiors(stage, None)?;
                let groups = canvas
                    .prepared_point_path_hue_groups(&interiors, &stage.generation_resources())?
                    .unwrap();
                assert_eq!(
                    groups.as_slice(),
                    [("cohort".into(), vec!["hidden".into(), "visible".into()])]
                );
                Ok(())
            },
            |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |(), _, (), _| Ok(()),
        )
        .unwrap();
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn mixed_scatter_line_hue_order_is_prepared_before_fonts() {
        let scatter = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 2.0]),
        )
        .hue(crate::CategoryValues::new("cohort", ["first", "first"]).unwrap())
        .legend(crate::LegendPolicy::Hidden);
        let line = Plot::line(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [2.0, 3.0]),
        )
        .hue(crate::CategoryValues::new("cohort", ["second", "second"]).unwrap());
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, scatter)
            .unwrap()
            .panel(0, 1, line)
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        crate::layout::plan_exact_root(
            960,
            640,
            |stage| {
                let (interiors, _) = canvas.prepare_interiors(stage, None)?;
                let levels = canvas
                    .prepared_point_path_hue_groups(&interiors, &stage.generation_resources())?
                    .unwrap();
                assert_eq!(
                    levels.as_slice(),
                    [("cohort".into(), vec!["first".into(), "second".into()])]
                );
                Ok(())
            },
            |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
            |(), _, (), _| Ok(()),
        )
        .unwrap();
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
        let scene = canvas.render_scene().unwrap();
        let svg = String::from_utf8(scene.svg_bytes().to_vec()).unwrap();
        let guide = svg.split_once("<g id=\"canvas-shared-legend\">").unwrap().1;
        assert!(guide.contains(">Second</text>"));
        assert!(!guide.contains(">First</text>"));
    }

    #[test]
    fn mixed_point_path_shared_glyph_conflict_precedes_fonts() {
        let hue = || crate::CategoryValues::new("cohort", ["one", "one"]).unwrap();
        let scatter = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 2.0]),
        )
        .hue(hue());
        let line = Plot::line(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [2.0, 3.0]),
        )
        .hue(hue());
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, scatter)
            .unwrap()
            .panel(0, 1, line)
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        assert!(matches!(
            canvas.preflight(),
            Err(Error::Render(message)) if message.contains("glyphs must match")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn conflicting_bar_shared_glyphs_reject_before_fonts() {
        let bar = || {
            Plot::bar(
                crate::CategoryValues::new("category", ["a", "b"]).unwrap(),
                crate::BarInput::estimates(crate::NumericValues::new("value", [1.0, 2.0])),
            )
            .series(crate::CategoryValues::new("cohort", ["one", "one"]).unwrap())
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, bar().theme(Theme::Nature))
            .unwrap()
            .panel(0, 1, bar().theme(Theme::Science))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        assert!(matches!(
            canvas.preflight(),
            Err(Error::Render(message)) if message.contains("glyphs must match")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn shared_signed_bars_merge_sign_entries_instead_of_series_colors() {
        let bar = |value, series| {
            Plot::horizontal_bar(
                crate::CategoryValues::new("category", ["one"]).unwrap(),
                crate::BarInput::estimates(crate::NumericValues::new("value", [value])),
            )
            .series(crate::CategoryValues::new("series", [series]).unwrap())
            .color_by_sign()
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, bar(-2.0, "first"))
            .unwrap()
            .panel(0, 1, bar(3.0, "second"))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared)
            .size(1200, 640);
        let root = canvas.preflight_plan().unwrap();
        let Some(PreparedGuideDescriptor::Discrete((title, entries))) =
            root.prepared().shared_guide.as_ref()
        else {
            panic!("signed bars need one prepared shared guide")
        };
        assert_eq!(title, "Change");
        assert_eq!(entries.len(), 2);
        assert_eq!(
            entries[0].identity(),
            &crate::decorations::legend::LegendIdentity::bar_sign(true)
        );
        assert_eq!(
            entries[1].identity(),
            &crate::decorations::legend::LegendIdentity::bar_sign(false)
        );
        let svg = String::from_utf8(canvas.render_scene().unwrap().svg_bytes().to_vec()).unwrap();
        let guide = svg.split_once("<g id=\"canvas-shared-legend\">").unwrap().1;
        assert!(guide.contains(">Decreasing</text>"));
        assert!(guide.contains(">Nondecreasing</text>"));
        assert!(!guide.contains(">first</text>"));
        assert!(!guide.contains(">second</text>"));
    }

    #[test]
    fn conflicting_distribution_shared_glyphs_reject_before_fonts() {
        let boxplot = || {
            Plot::boxplot(
                crate::CategoryValues::new("category", ["a", "a", "b", "b"]).unwrap(),
                crate::NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]),
            )
            .series(crate::CategoryValues::new("cohort", ["one"; 4]).unwrap())
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, boxplot().theme(Theme::Nature))
            .unwrap()
            .panel(0, 1, boxplot().theme(Theme::Science))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        assert!(matches!(
            canvas.preflight(),
            Err(Error::Render(message)) if message.contains("glyphs must match")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn conflicting_density_shared_glyphs_reject_before_fonts() {
        let density = || {
            Plot::density(crate::NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]))
                .hue(crate::CategoryValues::new("cohort", ["one"; 4]).unwrap())
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, density().theme(Theme::Nature))
            .unwrap()
            .panel(0, 1, density().theme(Theme::Science))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        assert!(matches!(
            canvas.preflight(),
            Err(Error::Render(message)) if message.contains("glyphs must match")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn shared_policy_rejects_an_explicit_child_edge_before_fonts() {
        let plot = Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [2.0, 3.0]),
        )
        .hue(crate::CategoryValues::new("cohort", ["a", "a"]).unwrap())
        .legend(crate::LegendPolicy::Visible(
            crate::LegendPosition::OutsideRight,
        ));
        let canvas = Canvas::grid(1, 1)
            .unwrap()
            .panel(0, 0, plot)
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        assert!(matches!(
            canvas.preflight(),
            Err(Error::Data(message)) if message.contains("explicit per-Plot placement")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn an_ordinary_plot_without_entries_does_not_block_prepared_shared_guides() {
        let scatter = Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [2.0, 3.0]),
        )
        .hue(crate::CategoryValues::new("cohort", ["a", "a"]).unwrap());
        let histogram = Plot::histogram(crate::HistogramInput::samples(crate::NumericValues::new(
            "observation",
            [0.0, 1.0, 2.0, 3.0],
        )));
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, scatter)
            .unwrap()
            .panel(0, 1, histogram)
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        let svg = canvas.render_scene().unwrap();
        let svg = String::from_utf8(svg.svg_bytes().to_vec()).unwrap();
        assert_eq!(svg.matches("id=\"canvas-shared-legend\"").count(), 1);
        assert_eq!(svg.matches(">A</text>").count(), 1);
    }

    #[test]
    fn conflicting_scatter_glyphs_reject_before_fonts_without_body_probes() {
        let source = || {
            Plot::scatter(
                crate::NumericValues::new("x", [1.0, 2.0]),
                crate::NumericValues::new("y", [2.0, 3.0]),
            )
            .hue(crate::CategoryValues::new("cohort", ["shared", "shared"]).unwrap())
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, source().theme(Theme::Nature))
            .unwrap()
            .panel(0, 1, source().theme(Theme::Science))
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let error = canvas.preflight().unwrap_err();
        assert!(matches!(
            error,
            Error::Render(message) if message.contains("glyphs must match")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "one fixture distinguishes populated, hidden, and absent prepared contributors"
    )]
    fn mixed_shared_guide_kinds_reject_before_font_discovery() {
        let heatmap = || {
            Plot::heatmap(
                crate::CategoryValues::new("column", ["a", "b"]).unwrap(),
                crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
                crate::NumericValues::new("fill", [1.0, 2.0]),
            )
        };
        let scatter = || {
            Plot::scatter(
                crate::NumericValues::new("x", [1.0, 2.0]),
                crate::NumericValues::new("y", [2.0, 3.0]),
            )
        };
        let mixed = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, heatmap())
            .unwrap()
            .panel(
                0,
                1,
                scatter().hue(crate::CategoryValues::new("hue", ["A", "B"]).unwrap()),
            )
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        assert!(matches!(
            mixed.preflight_plan(),
            Err(Error::Data(message)) if message.contains("mixes discrete and continuous")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
        // An unlabeled Scatter and an explicitly hidden discrete legend have no
        // competing guide kind; compatibility follows contributions, not PlotKind.
        let no_guide = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, heatmap())
            .unwrap()
            .panel(0, 1, scatter())
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        let validate = |canvas: &Canvas| {
            crate::layout::plan_exact_root(
                960,
                640,
                |stage| {
                    let themes = ResolvedCanvasThemes::new(
                        canvas.theme,
                        &canvas.panels,
                        &stage.generation_resources(),
                    )?;
                    let (interiors, _) = canvas.prepare_interiors(stage, Some(&themes))?;
                    canvas.validate_shared_guide_kinds(&interiors)?;
                    Ok(())
                },
                |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
                |(), _, (), _| Ok(()),
            )
            .map(|_| ())
        };
        assert!(validate(&no_guide).is_ok());
        let hidden = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, heatmap())
            .unwrap()
            .panel(
                0,
                1,
                scatter()
                    .hue(crate::CategoryValues::new("hue", ["A", "B"]).unwrap())
                    .legend(crate::LegendPolicy::Hidden),
            )
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        assert!(validate(&hidden).is_ok());
        let grouped_without_guide = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, heatmap())
            .unwrap()
            .panel(
                0,
                1,
                Plot::line(
                    crate::NumericValues::new("x", [1.0, 2.0]),
                    crate::NumericValues::new("y", [2.0, 3.0]),
                )
                .group(crate::CategoryValues::new("trace", ["a", "a"]).unwrap()),
            )
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        assert!(validate(&grouped_without_guide).is_ok());
        let labeled_layer = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, heatmap())
            .unwrap()
            .panel(
                0,
                1,
                Plot::line(
                    crate::NumericValues::new("x", [1.0, 2.0]),
                    crate::NumericValues::new("y", [2.0, 3.0]),
                )
                .point_layer(
                    crate::PointLayer::new(
                        crate::NumericValues::new("px", [1.5]),
                        crate::NumericValues::new("py", [2.5]),
                    )
                    .legend_label("visible layer"),
                ),
            )
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        assert!(matches!(
            validate(&labeled_layer),
            Err(Error::Data(message)) if message.contains("mixes discrete and continuous")
        ));
        let marginal = Plot::marginal(
            crate::NumericValues::new("x", [1.0, 1.2, 1.4, 3.0, 3.2, 3.4]),
            crate::NumericValues::new("y", [2.0, 2.1, 2.3, 4.0, 4.2, 4.3]),
        )
        .hue(crate::CategoryValues::new("hue", ["b", "b", "b", "a", "a", "a"]).unwrap());
        let specialized_discrete = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, heatmap())
            .unwrap()
            .panel(0, 1, marginal)
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared);
        assert!(matches!(
            validate(&specialized_discrete),
            Err(Error::Data(message)) if message.contains("mixes discrete and continuous")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn odd_canvas_root_policy_is_independent_of_local_panel_alignments() {
        let mut canvas = Canvas::grid(2, 3)
            .unwrap()
            .panel(
                1,
                2,
                Plot::scatter(
                    crate::NumericValues::new("x", [0.0, 1.0]),
                    crate::NumericValues::new("y", [1.0, 2.0]),
                )
                .title("private panel title")
                .subtitle("private panel subtitle"),
            )
            .unwrap()
            .column_header(0, 3, "spanning sparse columns")
            .unwrap()
            .title("outer title\nsecond title line")
            .subtitle("outer subtitle")
            .caption("outer caption")
            .size(1201, 901);
        let initial = canvas.preflight_plan().unwrap();
        canvas.panel_title_alignment = Some(crate::theme::TitleAlignment::End);
        canvas.panel_subtitle_alignment = Some(crate::theme::TitleAlignment::Start);
        let changed = canvas.preflight_plan().unwrap();
        let initial_text = initial.plan().retained_text();
        let changed_text = changed.plan().retained_text();
        for source in [
            "outer title\nsecond title line",
            "outer subtitle",
            "outer caption",
        ] {
            let before = initial_text
                .iter()
                .find(|(_, plan)| plan.source == source)
                .unwrap();
            let after = changed_text
                .iter()
                .find(|(_, plan)| plan.source == source)
                .unwrap();
            assert_eq!(
                before, after,
                "Canvas root text {source:?} must not inherit panel policy"
            );
        }
        let before = initial_text
            .iter()
            .find(|(_, plan)| plan.source == "private panel title")
            .unwrap();
        let after = changed_text
            .iter()
            .find(|(_, plan)| plan.source == "private panel title")
            .unwrap();
        assert_ne!(before.0.anchor, after.0.anchor);
        canvas.title_alignment = Some(crate::theme::TitleAlignment::Start);
        canvas.subtitle_alignment = Some(crate::theme::TitleAlignment::End);
        let root = canvas.preflight_plan().unwrap();
        let text = root.plan().retained_text();
        for (source, anchor) in [
            (
                "outer title\nsecond title line",
                crate::render::backend::TextAnchor::Start,
            ),
            ("outer subtitle", crate::render::backend::TextAnchor::End),
            ("outer caption", crate::render::backend::TextAnchor::Start),
            (
                "private panel title",
                crate::render::backend::TextAnchor::End,
            ),
            (
                "private panel subtitle",
                crate::render::backend::TextAnchor::Start,
            ),
        ] {
            let (placement, _) = text
                .iter()
                .find(|(_, plan)| plan.source == source)
                .expect(source);
            assert_eq!(placement.anchor, anchor, "{source}");
        }
        let initial_root_title = initial_text
            .iter()
            .find(|(_, plan)| plan.source == "outer title\nsecond title line")
            .unwrap();
        let aligned_root_title = text
            .iter()
            .find(|(_, plan)| plan.source == "outer title\nsecond title line")
            .unwrap();
        assert_ne!(initial_root_title.0.anchor, aligned_root_title.0.anchor);
        let aligned = canvas.render_scene().unwrap();
        crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|disabled| disabled.set(true));
        let uncached = canvas.render_scene();
        crate::decorations::plan::children::DISABLE_CHILD_REUSE
            .with(|disabled| disabled.set(false));
        assert_eq!(aligned.svg_bytes(), uncached.unwrap().svg_bytes());
    }

    #[test]
    fn shared_x_member_rejections_precede_font_discovery() {
        let numeric = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [1.0, 2.0]),
        );
        let categorical = Plot::boxplot(
            crate::CategoryValues::new("x", ["a", "b"]).unwrap(),
            crate::NumericValues::new("y", [1.0, 2.0]),
        )
        .erase();
        for (member, expected) in [
            (Some(categorical), "categorical x axis"),
            (
                Some(numeric.clone().x_limits(0.0, 1.0).erase()),
                "explicit x limits",
            ),
            (None, "unconfigured panel"),
        ] {
            let mut canvas = Canvas::grid(1, 2)
                .unwrap()
                .panel(0, 0, numeric.clone())
                .unwrap()
                .x_range_group([(0, 0), (0, 1)])
                .unwrap();
            if let Some(member) = member {
                canvas.panels[1] = Some(member);
            }
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            let error = canvas.render_scene().unwrap_err();
            assert!(
                matches!(&error, Error::Data(message) if message.contains(expected)),
                "{error}"
            );
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
        }
    }

    #[test]
    fn shared_x_domains_are_prepared_once_and_exact_containment_is_prefont() {
        let member = |values| {
            Plot::scatter(
                crate::NumericValues::new("x", values),
                crate::NumericValues::new("y", [1.0, 2.0]),
            )
            .legend(crate::LegendPolicy::Hidden)
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, member([0.0, 1.0]))
            .unwrap()
            .panel(0, 1, member([10.0, 20.0]))
            .unwrap()
            .x_range_group([(0, 0), (0, 1)])
            .unwrap()
            .size(1200, 800);
        crate::plots::shared_x::DOMAIN_PREPARATIONS.with(|count| count.set(0));
        crate::plots::scatter::STAGES.with(|stages| stages.set([0; 4]));
        let root = canvas.preflight_plan().unwrap();
        let shared = root.prepared().shared_x[0].unwrap();
        assert_eq!(root.prepared().shared_x[1], Some(shared));
        assert!(shared.0 <= 0.0 && shared.1 >= 20.0);
        assert_eq!(
            crate::plots::shared_x::DOMAIN_PREPARATIONS.with(std::cell::Cell::get),
            0
        );
        assert_eq!(
            crate::plots::scatter::STAGES.with(std::cell::Cell::get)[0],
            2
        );

        let cached = canvas.render_scene().unwrap();
        crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|disabled| disabled.set(true));
        let uncached = canvas.render_scene();
        crate::decorations::plan::children::DISABLE_CHILD_REUSE
            .with(|disabled| disabled.set(false));
        assert_eq!(cached.svg_bytes(), uncached.unwrap().svg_bytes());

        let mut invalid = canvas;
        invalid.panels[0] = Some(
            member([0.0, 1.0])
                .x_ticks(vec![crate::AxisTick::new(1000.0, "outside")])
                .erase(),
        );
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let error = invalid.render_scene().unwrap_err();
        assert!(
            error.to_string().contains("outside resolved shared range"),
            "{error}"
        );
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn invalid_shared_x_family_domains_precede_font_discovery() {
        let invalid = [
            Plot::scatter(
                crate::NumericValues::new("x", [f64::NAN]),
                crate::NumericValues::new("y", [1.0]),
            )
            .erase(),
            Plot::density(crate::NumericValues::new("x", [1.0])).erase(),
            Plot::qq(crate::NumericValues::new("x", [1.0])).erase(),
            Plot::forest(
                crate::CategoryValues::new("term", ["a"]).unwrap(),
                crate::NumericValues::new("estimate", [0.0]),
                crate::Bounds::new(
                    crate::NumericValues::new("lower", [0.0]),
                    crate::NumericValues::new("upper", [1.0]),
                ),
                1.0,
            )
            .scale(crate::AxisScale::Log10)
            .erase(),
        ];
        for plot in invalid {
            let mut canvas = Canvas::grid(1, 2)
                .unwrap()
                .panel(
                    0,
                    1,
                    Plot::scatter(
                        crate::NumericValues::new("x", [1.0, 2.0]),
                        crate::NumericValues::new("y", [1.0, 2.0]),
                    ),
                )
                .unwrap()
                .x_range_group([(0, 0), (0, 1)])
                .unwrap();
            canvas.panels[0] = Some(plot);
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            assert!(matches!(canvas.render_scene(), Err(Error::Data(_))));
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
        }
    }

    #[test]
    fn shared_x_forest_union_is_not_log_transformed_twice() {
        let forest = || {
            Plot::forest(
                crate::CategoryValues::new("term", ["a"]).unwrap(),
                crate::NumericValues::new("estimate", [1.0]),
                crate::Bounds::new(
                    crate::NumericValues::new("lower", [0.1]),
                    crate::NumericValues::new("upper", [10.0]),
                ),
                1.0,
            )
            .scale(crate::AxisScale::Log10)
            .x_ticks(vec![
                crate::AxisTick::new(0.1, "low"),
                crate::AxisTick::new(10.0, "high"),
            ])
        };
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, forest())
            .unwrap()
            .panel(0, 1, forest())
            .unwrap()
            .x_range_group([(0, 0), (0, 1)])
            .unwrap()
            .size(1200, 800);
        let root = canvas.preflight_plan().unwrap();
        assert_eq!(
            root.prepared().shared_x[0],
            Some(
                crate::layout::continuous_auto_range(
                    (-1.0, 1.0),
                    &crate::layout::AxisSpec::default(),
                    &crate::font::canonical::OperationGenerationResources::new(),
                )
                .unwrap()
            )
        );
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "the production regression compares whole cached and uncached root lifecycles"
    )]
    fn dirty_canvas_child_reuses_the_unchanged_neighbor_with_exact_output() {
        use crate::decorations::plan::children::{
            ChildPlans, CHILD_MEASUREMENTS, DISABLE_CHILD_REUSE,
        };
        use crate::orchestrator::measurements::{self, Counter};
        let plot = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .hue(crate::CategoryValues::new("cohort", ["first", "second"]).unwrap())
        .x_ticks(vec![
            crate::layout::AxisTick::new(0.0, "zero"),
            crate::layout::AxisTick::new(1.0, "one"),
        ])
        .panel_text_annotation(
            crate::PanelTextAnnotation::new(crate::PanelCorner::TopLeft, "annotation").unwrap(),
        )
        .legend(crate::LegendPolicy::Automatic);
        let canvas = Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, plot.clone())
            .unwrap()
            .panel(0, 1, plot)
            .unwrap()
            .size(1200, 800)
            .legend_policy(CanvasLegendPolicy::PerPlot);
        let owner = crate::layout::ComponentAddress::root()
            .child(
                crate::layout::AddressRole::CanvasCell,
                0,
                &crate::font::canonical::OperationGenerationResources::new(),
            )
            .unwrap();
        let neighbor = crate::layout::ComponentAddress::root()
            .child(
                crate::layout::AddressRole::CanvasCell,
                1,
                &crate::font::canonical::OperationGenerationResources::new(),
            )
            .unwrap();
        let mut expected = None;
        for disabled in [true, false] {
            DISABLE_CHILD_REUSE.with(|value| value.set(disabled));
            CHILD_MEASUREMENTS.with(|events| events.borrow_mut().clear());
            let themes = Arc::new(
                ResolvedCanvasThemes::new(
                    canvas.theme,
                    &canvas.panels,
                    &crate::font::canonical::OperationGenerationResources::new(),
                )
                .unwrap(),
            );
            let mut children = ChildPlans::canvas(&canvas);
            let mut calls = 0;
            let mut retained_neighbor = None;
            measurements::start();
            let root = crate::layout::plan_exact_root(
                canvas.width,
                canvas.height,
                |stage| {
                    let (interiors, _) = canvas.prepare_interiors(stage, Some(&themes))?;
                    let (mut census, presence) =
                        canvas.foundational_text_census(&themes, &interiors, None, stage)?;
                    themes.prepare_font_environment(&mut census, presence, stage)?;
                    let (_, text_intrinsics, plot_intrinsics) =
                        canvas.prepare_intrinsics(&themes, &interiors)?;
                    Ok((interiors, text_intrinsics, plot_intrinsics))
                },
                |(interiors, text_intrinsics, plot_intrinsics), assigned, round| {
                    calls += 1;
                    let plan = canvas.build_canvas_plan(
                        Arc::clone(&themes),
                        assigned,
                        round,
                        &mut children,
                        &[],
                        None,
                        None,
                        text_intrinsics,
                        plot_intrinsics,
                        interiors,
                        false,
                        None,
                    )?;
                    if calls == 2 {
                        retained_neighbor = Some(Arc::clone(&plan.panels[1].panel_plan));
                        round.request_domain(
                            &owner,
                            &owner,
                            FeedbackKind::AutomaticDomainX,
                            (-1.0, 2.0),
                        )?;
                    } else if calls == 3 {
                        assert_eq!(
                            Arc::ptr_eq(
                                retained_neighbor.as_ref().unwrap(),
                                &plan.panels[1].panel_plan
                            ),
                            !disabled
                        );
                    }
                    children.finish_round(round)?;
                    Ok(crate::layout::MeasuredLayout::new(assigned, plan))
                },
                |_, assigned, plan, stage| {
                    let (width, height) = assigned.dimensions()?;
                    let panels = plan
                        .panels
                        .iter()
                        .map(|panel| {
                            let plan = &panel.panel_plan;
                            let layout = &plan.result.layout;
                            let bounds = layout.projection.panel_box();
                            (
                                plan.panel_box,
                                layout.projection.x_range(),
                                layout.projection.y_range(),
                                layout.x_axis.ticks.clone(),
                                layout.y_axis.ticks.clone(),
                                plan.result
                                    .panel_text
                                    .iter()
                                    .map(|annotation| {
                                        (
                                            annotation.bounds_in(bounds),
                                            annotation.placement_in(bounds),
                                            annotation.text_plan.clone(),
                                        )
                                    })
                                    .collect::<Vec<_>>(),
                            )
                        })
                        .collect::<Vec<_>>();
                    let replay = crate::orchestrator::capture_text_replay_plan(
                        width,
                        height,
                        "child reuse test",
                        themes.root(),
                        stage,
                        |backend| Canvas::render_planned_canvas(&plan, width, height, backend),
                    )?;
                    crate::orchestrator::validate_text_replay_plan(
                        width,
                        height,
                        &replay,
                        "child reuse test",
                    )?;
                    let text = replay
                        .retained_text()
                        .into_iter()
                        .map(|(placement, text)| (placement, text.clone()))
                        .collect::<Vec<_>>();
                    for source in ["annotation", "zero", "one"] {
                        assert!(text.iter().any(|(_, plan)| plan.source == source));
                    }
                    let mut svg = replay.take_svg_backend()?;
                    replay.replay(width, height, &mut svg, "child reuse test")?;
                    Ok((svg.into_bytes()?, text, panels))
                },
            )
            .unwrap();
            let work = measurements::finish();
            DISABLE_CHILD_REUSE.with(|value| value.set(false));
            assert_eq!(calls, 3);
            for guide_owner in [&owner, &neighbor] {
                assert!(work
                    .guides
                    .iter()
                    .any(|attempt| &attempt.owner == guide_owner));
            }
            assert_eq!(
                work.counters[Counter::ChildCacheHit as usize] > 0,
                !disabled
            );
            assert_eq!(
                work.counters[Counter::ChildFeedbackReplay as usize] > 0,
                !disabled
            );
            assert_eq!(
                work.children.iter().any(|event| {
                    event.owner == neighbor
                        && event.root_round == 2
                        && event.counter == Counter::ChildCacheHit
                }),
                !disabled
            );
            CHILD_MEASUREMENTS.with(|events| {
                let events = events.borrow();
                assert!(events.iter().all(|(_, phase)| *phase != "layout"));
                assert_eq!(
                    events
                        .iter()
                        .filter(|(address, phase)| address == &owner && *phase == "panel")
                        .count(),
                    3,
                    "dirty panel must be remeasured in each root round"
                );
                assert_eq!(
                    events
                        .iter()
                        .filter(|(address, phase)| address == &neighbor && *phase == "panel")
                        .count(),
                    if disabled { 3 } else { 2 },
                    "unchanged neighbor must be reused only when cache is enabled"
                );
            });
            if let Some(expected) = &expected {
                assert_eq!(root.plan(), expected);
            } else {
                expected = Some(root.plan().clone());
            }
        }
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "root-prepared mixed Canvas fixture checks every producer at first capture"
    )]
    fn canvas_producer_plans_survive_first_capture_without_shaping() {
        let source = format!(
            "{}AA\u{2029}A \u{202b}\u{5d0} A \u{5d0} \u{202c}A",
            "A\u{ad}".repeat(60)
        );
        for row_title in [false, true] {
            let plot = Plot::scatter(
                crate::NumericValues::new("x", [0.0, 1.0]),
                crate::NumericValues::new("y", [0.0, 1.0]),
            )
            .theme(Theme::Science);
            let plot = if row_title {
                plot
            } else {
                plot.title(&source).subtitle(&source)
            };
            let mut canvas = Canvas::grid(1, 2)
                .unwrap()
                .panel(0, 1, plot)
                .unwrap()
                .text_panel(0, 0, TextPanel::new(&source, &source).unwrap())
                .unwrap()
                .size(801, 1001)
                .title("Outer root title")
                .subtitle("Outer root subtitle")
                .caption("Outer root caption")
                .row_header(0, 1, "A\u{301} shared row")
                .unwrap();
            if row_title {
                canvas = canvas.row_title(0, &source).unwrap();
            } else {
                canvas = canvas.column_header(0, 2, "A\u{301} shared").unwrap();
            }
            let themes = Arc::new(
                ResolvedCanvasThemes::new(
                    canvas.theme,
                    &canvas.panels,
                    &crate::font::canonical::OperationGenerationResources::new(),
                )
                .unwrap(),
            );
            crate::layout::plan_exact_root(
                canvas.width,
                canvas.height,
                |stage| {
                    let (interiors, _) = canvas.prepare_interiors(stage, Some(&themes))?;
                    let (mut census, presence) =
                        canvas.foundational_text_census(&themes, &interiors, None, stage)?;
                    themes.prepare_font_environment(&mut census, presence, stage)?;
                    let (_, text_intrinsics, plot_intrinsics) =
                        canvas.prepare_intrinsics(&themes, &interiors)?;
                    Ok((interiors, text_intrinsics, plot_intrinsics))
                },
                |(interiors, text_intrinsics, plot_intrinsics), assigned, round| {
                    Ok(crate::layout::MeasuredLayout::new(
                        assigned,
                        canvas.build_canvas_plan(
                            Arc::clone(&themes),
                            assigned,
                            round,
                            &mut crate::decorations::plan::children::ChildPlans::canvas(&canvas),
                            &[],
                            None,
                            None,
                            text_intrinsics,
                            plot_intrinsics,
                            interiors,
                            false,
                            None,
                        )?,
                    ))
                },
                |_, assigned, plan, stage| {
                    let mut expected = Vec::new();
                    for panel in &plan.text_panels {
                        expected.push((panel.title_placement, panel.title.plan()));
                        expected.push((panel.body_placement, panel.body.plan()));
                    }
                    for text in plan.headers.iter().chain(&plan.row_titles) {
                        expected.push((text.text.placement, &text.text.plan));
                    }
                    for panel in &plan.panels {
                        for (placement, plan) in panel.local_text.text.placed_text() {
                            expected.push((placement, plan));
                        }
                    }
                    assert!(expected.iter().any(|(_, text)| text.source == source
                        && text
                            .lines
                            .iter()
                            .any(|line| line.selected_soft_hyphen.is_some())));
                    assert!(expected.iter().any(|(_, text)| text.source == source
                        && text
                            .lines
                            .iter()
                            .any(|line| line.runs.iter().any(|run| run.bidi_level % 2 == 1))));
                    crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
                    crate::plots::scatter::STAGES.with(|stages| stages.set([0; 4]));
                    let (width, height) = assigned.dimensions()?;
                    let replay = crate::orchestrator::capture_text_replay_plan(
                        width,
                        height,
                        "Canvas producer closure",
                        themes.root(),
                        stage,
                        |backend| Canvas::render_planned_canvas(&plan, width, height, backend),
                    )?;
                    crate::font::canonical::FONT_SHAPE_CALLS
                        .with(|calls| assert_eq!(calls.get(), 0, "first capture must not shape"));
                    assert_eq!(
                        crate::plots::scatter::STAGES.with(std::cell::Cell::get),
                        [0, 0, 0, 1]
                    );
                    let captured = replay.retained_text();
                    for (placement, text) in expected {
                        assert!(captured.iter().any(
                            |(actual_placement, actual)| *actual_placement == placement
                                && actual.shares_payload(text)
                                && *actual == text
                        ));
                    }
                    Ok(())
                },
            )
            .unwrap();
        }
    }

    #[test]
    fn shared_guide_recovery_measures_the_complete_canvas_body() {
        let plot = Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0, 2.0, 3.0]),
            crate::NumericValues::new("y", [1.0, 2.0, 1.5, 2.5]),
        )
        .hue(
            crate::CategoryValues::new(
                "hue",
                [
                    "Complete explanatory first condition",
                    "Complete explanatory first condition",
                    "Complete explanatory second condition",
                    "Complete explanatory second condition",
                ],
            )
            .unwrap(),
        )
        .x_label("MMMMMMMMMMMMMMMMMMMMMMMMMMMM");
        let canvas = Canvas::grid(1, 1)
            .unwrap()
            .panel(0, 0, plot)
            .unwrap()
            .size(640, 480)
            .legend_policy(CanvasLegendPolicy::Shared);
        let themes = Arc::new(
            ResolvedCanvasThemes::new(
                canvas.theme,
                &canvas.panels,
                &crate::font::canonical::OperationGenerationResources::new(),
            )
            .unwrap(),
        );
        let root = crate::layout::plan_exact_root(
            canvas.width,
            canvas.height,
            |stage| {
                let (interiors, _) = canvas.prepare_interiors(stage, Some(&themes))?;
                let (mut census, presence) =
                    canvas.foundational_text_census(&themes, &interiors, None, stage)?;
                themes.prepare_font_environment(&mut census, presence, stage)?;
                let (_, text_intrinsics, plot_intrinsics) =
                    canvas.prepare_intrinsics(&themes, &interiors)?;
                Ok((interiors, text_intrinsics, plot_intrinsics))
            },
            |(interiors, text_intrinsics, plot_intrinsics), assigned, round| {
                Ok(crate::layout::MeasuredLayout::new(
                    assigned,
                    canvas.build_canvas_plan(
                        Arc::clone(&themes),
                        assigned,
                        round,
                        &mut crate::decorations::plan::children::ChildPlans::canvas(&canvas),
                        &[],
                        None,
                        None,
                        text_intrinsics,
                        plot_intrinsics,
                        interiors,
                        false,
                        None,
                    )?,
                ))
            },
            |_, _, plan, _| Ok(plan),
        )
        .unwrap();
        let plan = root.plan();
        let guide = plan.shared_legend.as_ref().unwrap();
        assert!(guide.bounds.bottom > plan.panels[0].canvas_cell.y1);
        assert_eq!(
            plan.panels[0].panel_plan.result.layout.x_axis.label,
            "MMMMMMMMMMMMMMMMMMMMMMMMMMMM"
        );
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "exercise owned source variants and exact Canvas preparation budgets together"
    )]
    fn reference_canvas_owned_sources_obey_prepared_budgets() {
        use crate::{HistogramInput, HistogramKdePoint, NumericValues};

        for geometry in [false, true] {
            let mut previous = None;
            for count in [4_u32, 128] {
                let kde = (0..count)
                    .map(|index| HistogramKdePoint {
                        x: f64::from(index),
                        y: 0.1,
                    })
                    .collect();
                let plot = if geometry {
                    Plot::histogram(HistogramInput::geometry(crate::HistogramGeometry {
                        dimensions: crate::HistogramDimensions {
                            value: "sample".into(),
                            density: "density".into(),
                        },
                        bins: (0..count - 1)
                            .map(|index| crate::HistogramBin {
                                lower: f64::from(index),
                                upper: f64::from(index + 1),
                                density: 0.1,
                            })
                            .collect(),
                        kde,
                        rugs: (0..count)
                            .map(|index| crate::HistogramRug {
                                x: f64::from(index),
                                baseline: 0.0,
                                top: 0.01,
                            })
                            .collect(),
                    }))
                } else {
                    Plot::histogram(HistogramInput::samples(NumericValues::new(
                        "sample",
                        [0.0, 1.0, 2.0, 127.0],
                    )))
                    .stat(crate::HistogramStat::Density)
                    .composite(crate::HistogramComposite::KdeAndRug(
                        crate::HistogramKde::Precomputed(kde),
                    ))
                };
                let canvas = Canvas::grid(1, 2)
                    .unwrap()
                    .panel(
                        0,
                        0,
                        plot.x_ticks(vec![crate::AxisTick::new(150.0, "shared endpoint")]),
                    )
                    .unwrap()
                    .panel(
                        0,
                        1,
                        Plot::density(NumericValues::new("sample", [0.0, 100.0, 200.0])),
                    )
                    .unwrap()
                    .labels(crate::Labels::humanized().alias("sample", "Shared sample"))
                    .x_range_group([(0, 0), (0, 1)])
                    .unwrap();
                let prepare = |limits: Option<(u128, u64)>| {
                    crate::layout::plan_exact_root(
                        640,
                        480,
                        |stage| {
                            let resources = stage.generation_resources();
                            if let Some((visits, bytes)) = limits {
                                resources.set_test_limits(visits, bytes);
                            }
                            let (inputs, shared) = canvas.prepare_interiors(stage, None)?;
                            for (index, input) in inputs.iter().enumerate() {
                                let input = input.as_ref().unwrap();
                                assert_eq!(
                                    input.axes[0].0.as_deref(),
                                    Some(format!("panel-{index}").as_str())
                                );
                                assert_eq!(Some(input.axes[0].1.ranges[0]), shared[index]);
                                assert_eq!(input.axes[0].1.specs[0].limits, shared[index]);
                                assert_eq!(input.axes[0].1.labels[0], "Shared sample");
                            }
                            let usage = (
                                resources.usage().unwrap().1,
                                resources.byte_usage().unwrap(),
                            );
                            // This test bounds preparation, not the kernel's later feedback records.
                            resources.set_test_limits(u128::MAX, u64::MAX);
                            Ok(usage)
                        },
                        |_, assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
                        |usage, _, (), _| Ok(*usage),
                    )
                    .map(|root| *root.plan())
                };
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
                let (visits, bytes) = prepare(None).unwrap();
                for (limits, dimension) in [
                    ((visits - 1, bytes), "axis source work admission"),
                    ((visits, bytes - 1), "axis source storage admission"),
                ] {
                    let error = prepare(Some(limits)).unwrap_err();
                    assert!(matches!(error, Error::Render(_)), "{error}");
                    assert!(error.to_string().contains(dimension), "{error}");
                }
                assert_eq!(prepare(Some((visits, bytes))).unwrap(), (visits, bytes));
                if let Some((small_visits, small_bytes)) = previous {
                    assert!(visits > small_visits && bytes > small_bytes);
                    assert!(prepare(Some((small_visits, u64::MAX))).is_err());
                    assert!(prepare(Some((u128::MAX, small_bytes))).is_err());
                }
                previous = Some((visits, bytes));
                assert_eq!(
                    crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                    0
                );
            }
        }
    }
}

struct ResolvedCanvasThemes {
    root: Box<ResolvedTheme>,
    plots: Vec<Option<ResolvedPlotTheme>>,
}

enum ResolvedPlotTheme {
    InheritedRoot,
    ExplicitPlot(Box<ResolvedTheme>),
}

impl ResolvedCanvasThemes {
    fn new(
        root: Theme,
        plots: &[Option<Plot>],
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<Self, Error> {
        let root = Box::new(root.resolve(resources)?);
        let plots = plots
            .iter()
            .map(|plot| {
                plot.as_ref()
                    .map(|plot| match plot.theme_spec() {
                        None => Ok(ResolvedPlotTheme::InheritedRoot),
                        Some(theme) => {
                            let mut theme = Box::new(theme.resolve(resources)?);
                            theme.canvas_surface_override = Some(root.surfaces.canvas_root);
                            theme.share_font_environment_from(&root);
                            Ok(ResolvedPlotTheme::ExplicitPlot(theme))
                        }
                    })
                    .transpose()
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Self { root, plots })
    }

    fn prepare_font_environment(
        &self,
        census: &mut crate::layout::PreparedTextCensus,
        presence: Vec<u64>,
        prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<(), Error> {
        self.root
            .prepare_font_environment_from_census(census, presence, prepared_stage)
    }

    fn root(&self) -> &ResolvedTheme {
        self.root.as_ref()
    }

    fn plot(&self, index: usize) -> Option<&ResolvedTheme> {
        self.plot_selection(index).map(|theme| match theme {
            ResolvedPlotTheme::InheritedRoot => self.root(),
            ResolvedPlotTheme::ExplicitPlot(theme) => theme.as_ref(),
        })
    }

    fn plot_selection(&self, index: usize) -> Option<&ResolvedPlotTheme> {
        self.plots.get(index)?.as_ref()
    }
}

struct CanvasHeaderPlan {
    text: crate::render::backend::PlacedText,
}

/// Categorical tick-label policy for composed canvas grids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasCategoryLabels {
    /// Render categorical tick labels in every panel.
    #[default]
    EveryPlot,
    /// Render categorical tick labels only in the first column of each row.
    FirstColumnPerRow,
}

#[derive(Debug, Clone)]
struct CanvasHeader {
    start: usize,
    span: usize,
    text: String,
}

struct TextPanelPlan {
    layer: String,
    cell: PanelBox,
    title: crate::layout::TextBlock,
    body: crate::layout::TextBlock,
    title_placement: crate::render::backend::PlannedTextPlacement,
    body_placement: crate::render::backend::PlannedTextPlacement,
}

/// Validated prose content for one composed-canvas grid cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPanel {
    title: String,
    body: String,
}

impl TextPanel {
    /// Construct a text panel with nonempty title and body.
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Result<Self, Error> {
        let title = title.into();
        let body = body.into();
        validate_text_panel_value(&title, "title")?;
        validate_text_panel_value(&body, "body")?;
        Ok(Self { title, body })
    }

    /// Panel title rendered above the body.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Panel body rendered with deterministic wrapping.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }
}

fn validate_text_panel_value(value: &str, role: &str) -> Result<(), Error> {
    if value.trim().is_empty() || value.trim() != value {
        return Err(Error::Data(format!(
            "text panel {role} must be nonempty and have no surrounding whitespace"
        )));
    }
    if value.chars().any(char::is_control) {
        return Err(Error::Data(format!(
            "text panel {role} must not contain control characters"
        )));
    }
    Ok(())
}

fn canvas_header(
    start: usize,
    span: usize,
    text: String,
    limit: usize,
    role: &str,
) -> Result<CanvasHeader, Error> {
    if span == 0 || start.checked_add(span).is_none_or(|end| end > limit) {
        return Err(Error::Data(format!(
            "canvas {role} header range {start}..{} is outside 0..{limit}",
            start.saturating_add(span)
        )));
    }
    if text.trim().is_empty() || text.trim() != text || text.chars().any(char::is_control) {
        return Err(Error::Data(format!(
            "canvas {role} header must be nonempty, trimmed, and free of control characters"
        )));
    }
    Ok(CanvasHeader { start, span, text })
}

fn validate_header_overlap(
    headers: &[CanvasHeader],
    candidate: &CanvasHeader,
    role: &str,
) -> Result<(), Error> {
    let end = candidate.start + candidate.span;
    if headers.iter().any(|header| {
        let other_end = header.start + header.span;
        candidate.start < other_end && header.start < end
    }) {
        return Err(Error::Data(format!(
            "canvas {role} headers must not overlap"
        )));
    }
    Ok(())
}

/// A typed grid of independently configured charts rendered on one canvas.
///
/// Canvas owns one root theme, which defaults to [`Theme::Publication`]. Plots
/// without an explicit theme inherit that root.
pub struct Canvas {
    rows: usize,
    columns: usize,
    panels: Vec<Option<Plot>>,
    text_panels: Vec<Option<TextPanel>>,
    theme: Theme,
    legend_policy: CanvasLegendPolicy,
    shared_legend_gap: Option<f64>,
    width: u32,
    height: u32,
    title: Option<String>,
    subtitle: Option<String>,
    pub(crate) title_alignment: Option<crate::theme::TitleAlignment>,
    pub(crate) subtitle_alignment: Option<crate::theme::TitleAlignment>,
    // Stored independently of root headings; composition consumes these at T5b.
    pub(crate) panel_title_alignment: Option<crate::theme::TitleAlignment>,
    pub(crate) panel_subtitle_alignment: Option<crate::theme::TitleAlignment>,
    caption: Option<String>,
    labels: Option<Labels>,
    category_labels: CanvasCategoryLabels,
    column_headers: Vec<CanvasHeader>,
    row_headers: Vec<CanvasHeader>,
    row_titles: Vec<CanvasHeader>,
    x_range_groups: Vec<Vec<usize>>,
}

fn push_text_requirement(
    requirements: &mut Vec<crate::layout::PreparedTextRequirement>,
    address: crate::layout::ComponentAddress,
    role: crate::layout::PreparedTextRole,
    local_index: u32,
    source: &str,
    appearance: &crate::theme::TextTheme,
    prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
) -> Result<(), Error> {
    crate::layout::PreparedTextRequirement::push_into(
        requirements,
        address,
        role,
        local_index,
        source,
        &[],
        appearance,
        Some(prepared_stage),
    )
    .map_err(|error| Error::Render(format!("prepared canvas text census: {error}")))
}

impl Canvas {
    /// Bound parent-only scope copies, diagnostic framing, equal-track scratch,
    /// and topology scans. Producer-owned text/geometry retains its own charges.
    fn admit_parent_metadata(
        &self,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<(), Error> {
        let tracks = self
            .rows
            .checked_add(self.columns)
            .ok_or_else(|| Error::Render("Canvas track count overflow".into()))?;
        let headers = self
            .column_headers
            .len()
            .checked_add(self.row_headers.len())
            .and_then(|n| n.checked_add(self.row_titles.len()))
            .ok_or_else(|| Error::Render("Canvas header count overflow".into()))?;
        let owners = self
            .panels
            .len()
            .checked_add(headers)
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| Error::Render("Canvas owner count overflow".into()))?;
        let segments = owners
            .checked_mul(256)
            .ok_or_else(|| Error::Render("Canvas scope graph overflow".into()))?;
        let grid_bytes = self
            .panels
            .len()
            .checked_mul(2 * std::mem::size_of::<PanelBox>())
            .and_then(|n| {
                tracks
                    .checked_mul(6 * std::mem::size_of::<f64>())
                    .and_then(|extra| n.checked_add(extra))
            })
            .ok_or_else(|| Error::Render("Canvas grid scratch overflow".into()))?;
        let bytes = segments
            .checked_mul(std::mem::size_of::<crate::layout::kernel::AddressSegment>() + 64)
            .and_then(|n| n.checked_add(grid_bytes))
            .ok_or_else(|| Error::Render("Canvas metadata storage overflow".into()))?;
        resources
            .claim_visits(segments as u128)
            .map_err(|error| Error::Render(format!("Canvas metadata work admission: {error:?}")))?;
        resources
            .claim_bytes(
                u64::try_from(bytes)
                    .map_err(|_| Error::Render("Canvas metadata bytes overflow".into()))?,
            )
            .map_err(|error| {
                Error::Render(format!("Canvas metadata storage admission: {error}"))
            })?;
        resources
            .claim_slots(owners as u128)
            .map_err(|error| Error::Render(format!("Canvas scope graph admission: {error:?}")))?;
        for group in &self.x_range_groups {
            resources
                .claim_visits(
                    (group.len() as u128)
                        .checked_mul(self.panels.len() as u128)
                        .ok_or_else(|| Error::Render("Canvas range scan overflow".into()))?,
                )
                .map_err(|error| {
                    Error::Render(format!("Canvas range scan admission: {error:?}"))
                })?;
        }
        Ok(())
    }

    /// Construct a non-empty panel grid.
    pub fn grid(rows: usize, columns: usize) -> Result<Self, Error> {
        if rows == 0 || columns == 0 {
            return Err(Error::Data(
                "canvas grid rows and columns must both be greater than zero".into(),
            ));
        }
        let panel_count = rows
            .checked_mul(columns)
            .ok_or_else(|| Error::Data("canvas grid dimensions exceed allocation limits".into()))?;
        Ok(Self {
            rows,
            columns,
            panels: (0..panel_count).map(|_| None).collect(),
            text_panels: (0..panel_count).map(|_| None).collect(),
            theme: Theme::Publication,
            legend_policy: CanvasLegendPolicy::PerPlot,
            shared_legend_gap: None,
            width: 960,
            height: 640,
            title: None,
            subtitle: None,
            title_alignment: None,
            subtitle_alignment: None,
            panel_title_alignment: None,
            panel_subtitle_alignment: None,
            caption: None,
            labels: None,
            category_labels: CanvasCategoryLabels::EveryPlot,
            column_headers: Vec::new(),
            row_headers: Vec::new(),
            row_titles: Vec::new(),
            x_range_groups: Vec::new(),
        })
    }

    /// Place one exact-family chart in an unoccupied zero-based grid cell.
    ///
    /// An already erased plot cannot be inserted:
    ///
    /// ```compile_fail
    /// use lichtung_plot::{Canvas, NumericValues, Plot};
    /// let erased: Plot = Plot::scatter(
    ///     NumericValues::new("x", [1.0]),
    ///     NumericValues::new("y", [2.0]),
    /// ).erase();
    /// let _ = Canvas::grid(1, 1).unwrap().panel(0, 0, erased);
    /// ```
    pub fn panel<K: crate::kind::ExactFamily>(
        mut self,
        row: usize,
        column: usize,
        plot: Plot<K>,
    ) -> Result<Self, Error> {
        if row >= self.rows || column >= self.columns {
            return Err(Error::Data(format!(
                "canvas panel ({row}, {column}) is outside {}x{} grid",
                self.rows, self.columns
            )));
        }
        let index = row * self.columns + column;
        if self.panels[index].is_some() || self.text_panels[index].is_some() {
            return Err(Error::Data(format!(
                "canvas panel ({row}, {column}) is already occupied"
            )));
        }
        self.panels[index] = Some(plot.erase());
        Ok(self)
    }

    /// Place one validated text panel in an unoccupied zero-based grid cell.
    pub fn text_panel(
        mut self,
        row: usize,
        column: usize,
        panel: TextPanel,
    ) -> Result<Self, Error> {
        if row >= self.rows || column >= self.columns {
            return Err(Error::Data(format!(
                "canvas panel ({row}, {column}) is outside {}x{} grid",
                self.rows, self.columns
            )));
        }
        let index = row * self.columns + column;
        if self.panels[index].is_some() || self.text_panels[index].is_some() {
            return Err(Error::Data(format!(
                "canvas panel ({row}, {column}) is already occupied"
            )));
        }
        self.text_panels[index] = Some(panel);
        Ok(self)
    }

    /// Set per-panel, shared, or hidden legend behavior.
    #[must_use]
    pub fn legend_policy(mut self, policy: CanvasLegendPolicy) -> Self {
        self.legend_policy = policy;
        self
    }

    /// Set the exact CSS-pixel gap between the panel grid and a shared legend.
    pub fn shared_legend_gap(mut self, gap: f64) -> Result<Self, Error> {
        if !gap.is_finite() || gap < 0.0 {
            return Err(Error::Data(
                "canvas shared legend gap must be finite and nonnegative".into(),
            ));
        }
        self.shared_legend_gap = Some(gap);
        Ok(self)
    }

    /// Add one measured header spanning contiguous canvas columns.
    pub fn column_header(
        mut self,
        start_column: usize,
        span: usize,
        text: impl Into<String>,
    ) -> Result<Self, Error> {
        let header = canvas_header(start_column, span, text.into(), self.columns, "column")?;
        validate_header_overlap(&self.column_headers, &header, "column")?;
        self.column_headers.push(header);
        Ok(self)
    }

    /// Add one measured header spanning contiguous canvas rows.
    pub fn row_header(
        mut self,
        start_row: usize,
        span: usize,
        text: impl Into<String>,
    ) -> Result<Self, Error> {
        let header = canvas_header(start_row, span, text.into(), self.rows, "row")?;
        validate_header_overlap(&self.row_headers, &header, "row")?;
        self.row_headers.push(header);
        Ok(self)
    }

    /// Add one measured title centered above every panel in one canvas row.
    pub fn row_title(mut self, row: usize, text: impl Into<String>) -> Result<Self, Error> {
        let title = canvas_header(row, 1, text.into(), self.rows, "row title")?;
        validate_header_overlap(&self.row_titles, &title, "row title")?;
        self.row_titles.push(title);
        Ok(self)
    }

    /// Configure duplicate categorical-label handling across the grid.
    #[must_use]
    pub const fn category_labels(mut self, policy: CanvasCategoryLabels) -> Self {
        self.category_labels = policy;
        self
    }

    /// Couple the automatic continuous horizontal range of explicit panel cells.
    ///
    /// Grouped cells must contain configured plots without explicit x limits or
    /// categorical x domains.
    pub fn x_range_group(
        mut self,
        cells: impl IntoIterator<Item = (usize, usize)>,
    ) -> Result<Self, Error> {
        let mut indices = Vec::new();
        for (row, column) in cells {
            if row >= self.rows || column >= self.columns {
                return Err(Error::Data(format!(
                    "canvas x-range group cell ({row}, {column}) is outside {}x{} grid",
                    self.rows, self.columns
                )));
            }
            let index = row * self.columns + column;
            if indices.contains(&index)
                || self
                    .x_range_groups
                    .iter()
                    .any(|group| group.contains(&index))
            {
                return Err(Error::Data(format!(
                    "canvas x-range group repeats panel ({row}, {column})"
                )));
            }
            indices.push(index);
        }
        if indices.is_empty() {
            return Err(Error::Data(
                "canvas x-range group must contain at least one panel".into(),
            ));
        }
        self.x_range_groups.push(indices);
        Ok(self)
    }

    /// Set the composed canvas dimensions.
    #[must_use]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set one canvas-level title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set one canvas-level subtitle.
    #[must_use]
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set one canvas-level caption.
    #[must_use]
    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Apply one reusable automatic-label policy to every configured panel.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Set the Canvas root theme.
    ///
    /// Plots without an explicit theme inherit this theme. A theme selected with
    /// [`Plot::theme`] completely replaces it for that Plot. If this method is
    /// omitted, the Canvas root uses [`Theme::Publication`].
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    fn render_preflighted(
        root: &crate::layout::FrozenRoot<crate::orchestrator::TextReplayPlan, PreparedCanvasInputs>,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        let _ = &root.prepared().text_census;
        let (width, height) = root.dimensions();
        root.plan()
            .replay(width, height, backend, "prepared canvas scene replay")
    }

    /// Validate the complete composed canvas without opening a backend or creating files.
    ///
    /// This checks canvas and grid layout, every chart's plot-specific data and
    /// representability preflight, channel and interval contracts, shared legend
    /// compatibility, and nested self-managed plot restrictions.
    pub fn preflight(&self) -> Result<(), Error> {
        self.preflight_plan().map(drop)
    }

    #[cfg(test)]
    pub(crate) fn render_with_backend(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        let preflight = self.preflight_plan()?;
        Self::render_preflighted(&preflight, backend)
    }

    /// Render one immutable canonical composed SVG scene without creating a file.
    ///
    /// # Errors
    ///
    /// Returns an error when canvas preflight or SVG rendering fails.
    fn render_scene(&self) -> Result<RenderedScene, Error> {
        #[cfg(test)]
        CANVAS_SCENE_RENDER_CALLS.with(|calls| calls.set(calls.get() + 1));
        let preflight = self.preflight_plan()?;
        let (width, height) = preflight.dimensions();
        let mut backend = preflight.plan().take_svg_backend()?;
        Self::render_preflighted(&preflight, &mut backend)?;
        RenderedScene::new(width, height, backend.into_output())
    }

    /// Render once, publish an outlined SVG, and return a reusable output.
    /// The output filename is completed with `.svg` after conversion.
    pub fn svg(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.svg(path)
    }

    /// Render once, publish SVG with selectable text, and return a reusable output.
    /// The output filename is completed with `.svg` after rendering.
    pub fn svg_preserve_text(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.svg_preserve_text(path)
    }

    /// Render once, publish a balanced 300-DPI PNG, and return a reusable output.
    /// The output filename is completed with `.png` after rasterization.
    pub fn png(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.png(path)
    }

    /// Render once, publish a balanced source-sized PNG without density metadata.
    /// The output filename is completed with `.png` after rasterization.
    pub fn png_preview(self, path: impl AsRef<Path>) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.png_preview(path)
    }

    /// Render once and publish a PNG using the exact caller-provided options.
    /// The output filename is completed with `.png` after rasterization.
    pub fn png_with_options(
        self,
        path: impl AsRef<Path>,
        options: crate::PngOptions,
    ) -> Result<Output, Error> {
        let output = Output::new(self.render_scene()?);
        output.png_with_options(path, options)
    }

    fn validate_structure(&self) -> Result<(), Error> {
        if self.width < 160 || self.height < 120 {
            return Err(Error::Data("canvas must be at least 160x120 pixels".into()));
        }
        if self.panels.iter().all(Option::is_none) && self.text_panels.iter().all(Option::is_none) {
            return Err(Error::Data(
                "canvas requires at least one configured panel".into(),
            ));
        }
        if self.shared_legend_gap.is_some() && self.legend_policy != CanvasLegendPolicy::Shared {
            return Err(Error::Data(
                "canvas shared legend gap requires CanvasLegendPolicy::Shared".into(),
            ));
        }
        for plot in self.panels.iter().flatten() {
            let kind = plot.kind();
            plot.validate_configuration(kind)?;
            crate::layout::interior::CompositionChildPolicy::Canvas.validate_child(plot)?;
            if self.legend_policy == CanvasLegendPolicy::Shared
                && matches!(
                    plot.legend_policy_spec(),
                    crate::LegendPolicy::Visible(position) if position != crate::LegendPosition::Auto
                )
            {
                return Err(Error::Data(
                    "Canvas shared legend cannot honor explicit per-Plot placement".into(),
                ));
            }
        }
        for group in &self.x_range_groups {
            for &index in group {
                let plot = self.panels[index].as_ref().ok_or_else(|| {
                    Error::Data(format!(
                        "canvas x-range group references unconfigured panel {index}"
                    ))
                })?;
                if plot.user_x_limits().is_some() {
                    return Err(Error::Data(format!(
                        "canvas x-range group panel {index} has explicit x limits"
                    )));
                }
                if plot
                    .automatic_axis_category_values(plot.kind(), true)
                    .is_some()
                {
                    return Err(Error::Data(format!(
                        "canvas x-range group panel {index} has a categorical x axis"
                    )));
                }
            }
        }
        for title in &self.row_titles {
            let start = title.start * self.columns;
            let end = start + self.columns;
            if self.panels[start..end].iter().flatten().any(|plot| {
                plot.title_text().is_some_and(|text| !text.is_empty())
                    || plot.subtitle_text().is_some_and(|text| !text.is_empty())
            }) {
                return Err(Error::Data(format!(
                    "canvas row title cannot coexist with a panel title or subtitle in row {}",
                    title.start
                )));
            }
        }
        Ok(())
    }

    /// Reject incompatible semantic guide kinds before the font census or body
    /// measurement. The exact entries and continuous descriptor are prepared by
    /// their owners; an unconfigured or hidden discrete guide contributes no kind.
    fn validate_shared_guide_kinds(
        &self,
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
    ) -> Result<(), Error> {
        if self.legend_policy != CanvasLegendPolicy::Shared {
            return Ok(());
        }
        let mut selected = None;
        for (index, plot) in self.panels.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let inputs = interiors
                .get(index)
                .and_then(Option::as_ref)
                .ok_or_else(|| {
                    Error::Render("configured Canvas guide has no prepared interior".into())
                })?;
            let Some(kind) =
                crate::decorations::legend::GuideKind::contributing_prepared(plot, inputs)?
            else {
                continue;
            };
            if selected.is_some_and(|selected| selected != kind) {
                return Err(Error::Data(
                    "shared Canvas guide mixes discrete and continuous contributors".into(),
                ));
            }
            selected = Some(kind);
        }
        Ok(())
    }

    fn prepare_continuous_shared_guide(
        &self,
        themes: &ResolvedCanvasThemes,
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<Option<crate::decorations::legend::descriptor::ContinuousDescriptor>, Error> {
        if self.legend_policy != CanvasLegendPolicy::Shared {
            return Ok(None);
        }
        let Some(first) = self.panels.iter().flatten().next() else {
            return Ok(None);
        };
        let first_data = first.data().with_axis_resources(resources);
        let mut contributors = Vec::new();
        first_data.reserve_source(&mut contributors, self.panels.len())?;
        for (index, plot) in self.panels.iter().enumerate() {
            let Some(plot) = plot else { continue };
            if !matches!(plot.kind(), PlotKind::Heatmap | PlotKind::Beeswarm) {
                continue;
            }
            let inputs = interiors
                .get(index)
                .and_then(Option::as_ref)
                .ok_or_else(|| {
                    Error::Render("shared continuous guide has no prepared interior".into())
                })?;
            let data = plot.data().with_axis_resources(resources);
            let theme = themes.plot(index).ok_or_else(|| {
                Error::Render("shared continuous guide has no owning Plot theme".into())
            })?;
            let contributor =
                crate::decorations::legend::descriptor::ContinuousContributor::for_plot(
                    plot, inputs, theme, data,
                )?
                .ok_or_else(|| {
                    Error::Render("shared continuous guide has no color identity".into())
                })?;
            contributors.push(contributor);
        }
        crate::decorations::legend::descriptor::ContinuousContributor::merge(
            contributors,
            first_data,
        )
    }

    fn install_shared_continuous_domain(
        &self,
        interiors: &mut [Option<crate::layout::interior::PreparedInteriorInputs>],
        descriptor: &crate::decorations::legend::descriptor::ContinuousDescriptor,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<(), Error> {
        for (index, plot) in self.panels.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let Some(input) = interiors.get_mut(index).and_then(Option::as_mut) else {
                continue;
            };
            let data = plot.data().with_axis_resources(resources);
            match plot.kind() {
                PlotKind::Heatmap => {
                    let panels = Arc::get_mut(input.heatmap.as_mut().ok_or_else(|| {
                        Error::Render("shared Heatmap has no prepared inputs".into())
                    })?)
                    .ok_or_else(|| Error::Render("shared Heatmap inputs are aliased".into()))?;
                    for (_, marks) in &mut panels.panels {
                        marks.install_shared_domain(descriptor, data)?;
                    }
                }
                PlotKind::Beeswarm => {
                    let panels = Arc::get_mut(input.beeswarm.as_mut().ok_or_else(|| {
                        Error::Render("shared Beeswarm has no prepared inputs".into())
                    })?)
                    .ok_or_else(|| Error::Render("shared Beeswarm inputs are aliased".into()))?;
                    for (_, marks) in &mut panels.panels {
                        marks.install_shared_domain(descriptor, data)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Prepare point/path hue identities from retained semantics before fonts.
    /// Further categorical families migrate to this same owner in T5b1b2b.
    #[expect(
        clippy::too_many_lines,
        reason = "shared domain coordinates all currently prepared discrete family hue sources"
    )]
    fn prepared_point_path_hue_groups(
        &self,
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<Option<Arc<PreparedHueGroups>>, Error> {
        if self.legend_policy != CanvasLegendPolicy::Shared {
            return Ok(None);
        }
        if self.panels.iter().flatten().any(|plot| {
            plot.hue_dimension_key().is_some()
                && !matches!(
                    plot.kind(),
                    PlotKind::Scatter
                        | PlotKind::Line
                        | PlotKind::Trajectory
                        | PlotKind::LinePath
                        | PlotKind::Bar
                        | PlotKind::HorizontalBar
                        | PlotKind::StackedBar
                        | PlotKind::StackedHorizontalBar
                        | PlotKind::Boxplot
                        | PlotKind::Violin
                        | PlotKind::Raincloud
                        | PlotKind::Density
                        | PlotKind::Regression
                        | PlotKind::Dumbbell
                        | PlotKind::Interaction
                        | PlotKind::Marginal
                )
        }) {
            return Ok(None);
        }
        let Some(first) = self.panels.iter().flatten().next() else {
            return Ok(None);
        };
        let mut groups = Vec::new();
        first
            .data()
            .with_axis_resources(resources)
            .reserve_source(&mut groups, self.panels.len())?;
        for (index, plot) in self.panels.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let Some(dimension) = plot.hue_dimension_key() else {
                continue;
            };
            let data = plot.data().with_axis_resources(resources);
            let inputs = interiors
                .get(index)
                .and_then(Option::as_ref)
                .ok_or_else(|| {
                    Error::Render("Canvas hue guide is missing prepared semantics".into())
                })?;
            let scatter = inputs
                .scatter
                .as_ref()
                .and_then(|inputs| inputs.panels.first())
                .map(|(_, prepared)| prepared);
            let line = inputs
                .line
                .as_ref()
                .and_then(|inputs| inputs.panels.first())
                .map(|(_, prepared)| prepared);
            let bars = inputs
                .bars
                .as_ref()
                .and_then(|inputs| inputs.panels.first())
                .map(|(_, prepared)| prepared);
            let distribution = inputs
                .distributions
                .as_ref()
                .and_then(|inputs| inputs.panels.first())
                .map(|(_, prepared)| prepared);
            let density = inputs
                .density
                .as_ref()
                .and_then(|inputs| inputs.panels.first())
                .map(|(_, prepared)| prepared);
            let interaction = inputs
                .interaction
                .as_ref()
                .and_then(|inputs| inputs.panels.first())
                .map(|(_, prepared)| prepared);
            let marginal = inputs.marginal.as_deref();
            if scatter.is_none()
                && line.is_none()
                && bars.is_none()
                && distribution.is_none()
                && density.is_none()
                && interaction.is_none()
                && inputs.regression.is_none()
                && inputs.dumbbell.is_none()
                && marginal.is_none()
            {
                return Err(Error::Render(
                    "Canvas guide has no prepared color source".into(),
                ));
            }
            let group_index = data.source_position(&groups, |(field, _)| field == dimension)?;
            let levels = if let Some(group_index) = group_index {
                &mut groups[group_index].1
            } else {
                groups.push((data.copy_source_string(dimension)?, Vec::new()));
                &mut groups.last_mut().expect("new guide field").1
            };
            for level in scatter
                .into_iter()
                .flat_map(crate::plots::scatter::PreparedScatter::hue_levels)
                .chain(
                    line.into_iter()
                        .flat_map(crate::plots::line::PreparedLine::hue_levels),
                )
                .chain(
                    bars.into_iter()
                        .flat_map(|bars| bars.hues.iter().map(String::as_str)),
                )
                .chain(
                    distribution
                        .into_iter()
                        .flat_map(|marks| marks.hue_levels().iter().map(String::as_str)),
                )
                .chain(
                    density
                        .into_iter()
                        .flat_map(crate::plots::density::PreparedDensity::hue_levels),
                )
                .chain(
                    interaction
                        .into_iter()
                        .flat_map(crate::plots::interaction::PreparedInteraction::hue_levels),
                )
                .chain(
                    marginal
                        .into_iter()
                        .flat_map(crate::plots::marginal::PreparedMarginal::hue_levels),
                )
            {
                if data
                    .source_position(levels, |existing| existing == level)?
                    .is_none()
                {
                    data.reserve_source(levels, 1)?;
                    levels.push(data.copy_source_string(level)?);
                }
            }
        }
        first
            .data()
            .with_axis_resources(resources)
            .source_arc(groups)
            .map(Some)
    }

    /// Prepare one shared point/path guide without measuring bodies or fonts.
    /// An empty retained set is still a prepared result.
    fn prepared_discrete_shared_guide(
        &self,
        themes: &ResolvedCanvasThemes,
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
        hue_groups: Option<&[(String, Vec<String>)]>,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<Option<PreparedDiscreteGuide>, Error> {
        if self.legend_policy != CanvasLegendPolicy::Shared
            || self.panels.iter().flatten().any(|plot| {
                !crate::layout::interior::InteriorRoute::classify(plot.kind()).is_coordinate()
                    && !matches!(plot.kind(), PlotKind::Marginal | PlotKind::Waterfall)
            })
        {
            return Ok(None);
        }
        let mut entries = Vec::new();
        for (index, plot) in self.panels.iter().enumerate() {
            let Some(source) = plot else { continue };
            if source.legend_policy_spec() == crate::LegendPolicy::Hidden {
                continue;
            }
            let data = source.data().with_axis_resources(resources);
            let mut plot = source.admitted_clone(data)?;
            if let Some(labels) = &self.labels {
                plot = plot.labels(labels.clone());
            }
            if let Some((_, levels)) = hue_groups.and_then(|groups| {
                groups
                    .iter()
                    .find(|(field, _)| Some(field.as_str()) == plot.hue_dimension_key())
            }) {
                let levels =
                    data.collect_source(levels.iter().map(|level| data.copy_source_string(level)))?;
                plot = plot.with_global_hue_levels(levels);
            }
            let theme = themes
                .plot(index)
                .expect("configured Canvas panel has a resolved theme");
            let inputs = interiors
                .get(index)
                .and_then(Option::as_ref)
                .ok_or_else(|| {
                    Error::Render("Canvas point/path guide is missing prepared semantics".into())
                })?;
            let mut contribution = inputs.guide_entries(
                &plot,
                theme,
                data,
                inputs.axes.first().and_then(|(scope, _)| scope.as_deref()),
            )?;
            crate::decorations::legend::resolve_entries(&plot, &mut contribution, resources)?;
            data.reserve_source(&mut entries, contribution.len())?;
            entries.extend(contribution);
        }
        let entries = crate::decorations::legend::merge_exact_entries(
            entries,
            "shared canvas legend",
            resources,
        )?;
        let title = crate::decorations::legend::resolved_title(&entries, resources)?;
        let first = self
            .panels
            .iter()
            .flatten()
            .next()
            .ok_or_else(|| Error::Data("canvas requires a configured panel".into()))?;
        let data = first.data().with_axis_resources(resources);
        data.claim_source_bytes(
            std::alloc::Layout::array::<LegendEntry>(entries.len())
                .and_then(|entries| std::alloc::Layout::new::<[usize; 2]>().extend(entries))
                .map_err(|_| Error::Render("Canvas shared guide Arc storage overflow".into()))?
                .0
                .pad_to_align()
                .size(),
        )?;
        Ok(Some((title, entries.into())))
    }

    fn prepare_shared_x(
        &self,
        stage: &mut crate::layout::kernel::PreparedStage<'_>,
        scatter: &[Option<crate::plots::scatter::PreparedScatter>],
        line: &[Option<crate::plots::line::PreparedLine>],
        reference: &[Option<crate::plots::shared_x::PreparedReference>],
    ) -> Result<Vec<Option<(f64, f64)>>, Error> {
        let mut ranges = Vec::new();
        if self.x_range_groups.is_empty() {
            return Ok(ranges);
        }
        stage.claim_topology_slots(self.panels.len() as u128)?;
        stage.reserve_prepared(&mut ranges, self.panels.len(), "shared x domains")?;
        ranges.resize(self.panels.len(), None);
        for group in &self.x_range_groups {
            let mut shared: Option<(f64, f64)> = None;
            for &index in group {
                let plot = self.panels[index]
                    .as_ref()
                    .expect("validated shared x member");
                let range = if let Some(marks) = &reference[index] {
                    marks.x_range(
                        plot,
                        plot.data()
                            .with_axis_resources(&stage.generation_resources()),
                    )?
                } else if plot.kind() == PlotKind::Scatter {
                    crate::layout::continuous_auto_range(
                        scatter[index]
                            .as_ref()
                            .ok_or_else(|| {
                                Error::Render(
                                    "shared Scatter range requires root-prepared inputs".into(),
                                )
                            })?
                            .ranges[0],
                        plot.x_axis_spec(),
                        &stage.generation_resources(),
                    )?
                } else if matches!(
                    plot.kind(),
                    PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath
                ) {
                    let marks = line[index].as_ref().ok_or_else(|| {
                        Error::Render("shared line range requires root-prepared inputs".into())
                    })?;
                    if marks.conditions.is_some() {
                        return Err(Error::Data(
                            "canvas x-range group has a categorical x axis".into(),
                        ));
                    }
                    marks.ranges[0]
                } else {
                    crate::plots::shared_x::domain(plot, &stage.generation_resources())?
                };
                shared = Some(
                    shared.map_or(range, |prior| (prior.0.min(range.0), prior.1.max(range.1))),
                );
            }
            let shared = shared.expect("validated nonempty shared x group");
            for &index in group {
                let plot = self.panels[index]
                    .as_ref()
                    .expect("validated shared x member");
                if let Some(ticks) = plot.explicit_x_ticks() {
                    for tick in ticks {
                        let position = if plot.kind() == PlotKind::Forest
                            && plot.axis_scale_spec() == crate::AxisScale::Log10
                        {
                            crate::numeric::log10(tick.position, &stage.generation_resources())?
                        } else {
                            tick.position
                        };
                        if !position.is_finite() || position < shared.0 || position > shared.1 {
                            return Err(Error::Data(format!("x-axis explicit tick position {} is outside resolved shared range {}..{}", tick.position, shared.0, shared.1)));
                        }
                    }
                }
                ranges[index] = Some(shared);
            }
        }
        Ok(ranges)
    }

    #[expect(
        clippy::type_complexity,
        clippy::too_many_lines,
        reason = "root component inputs and their coordinated x domains are prepared together before fonts"
    )]
    fn prepare_interiors(
        &self,
        stage: &mut crate::layout::kernel::PreparedStage<'_>,
        themes: Option<&ResolvedCanvasThemes>,
    ) -> Result<
        (
            Vec<Option<crate::layout::interior::PreparedInteriorInputs>>,
            Vec<Option<(f64, f64)>>,
        ),
        Error,
    > {
        let mut interiors = Vec::new();
        stage.reserve_prepared(&mut interiors, self.panels.len(), "Canvas interior inputs")?;
        let resources = stage.generation_resources();
        let mut scatter = Vec::new();
        stage.reserve_prepared(&mut scatter, self.panels.len(), "Canvas Scatter semantics")?;
        let mut line = Vec::new();
        stage.reserve_prepared(&mut line, self.panels.len(), "Canvas line semantics")?;
        let mut reference = Vec::new();
        stage.reserve_prepared(
            &mut reference,
            self.panels.len(),
            "Canvas distribution/reference semantics",
        )?;
        for (index, plot) in self.panels.iter().enumerate() {
            reference.push(
                match plot.as_ref().filter(|plot| {
                    matches!(
                        plot.kind(),
                        PlotKind::Histogram
                            | PlotKind::Density
                            | PlotKind::Qq
                            | PlotKind::Regression
                            | PlotKind::Dumbbell
                            | PlotKind::PointInterval
                            | PlotKind::Coefficient
                            | PlotKind::Forest
                            | PlotKind::Bar
                            | PlotKind::HorizontalBar
                            | PlotKind::StackedBar
                            | PlotKind::StackedHorizontalBar
                            | PlotKind::Boxplot
                            | PlotKind::Violin
                            | PlotKind::Raincloud
                            | PlotKind::Interaction
                            | PlotKind::Heatmap
                            | PlotKind::Beeswarm
                            | PlotKind::Waterfall
                            | PlotKind::Marginal
                    )
                }) {
                    Some(plot) => {
                        let data = plot
                            .configured_source_rows()?
                            .map_or_else(|| plot.data(), |rows| plot.data().map_source_rows(rows))
                            .with_axis_resources(&resources);
                        Some(crate::plots::shared_x::PreparedReference::new(
                            plot,
                            data,
                            themes
                                .and_then(|themes| themes.plot(index))
                                .map(|theme| &theme.color_scales),
                        )?)
                    }
                    None => None,
                },
            );
            line.push(
                match plot.as_ref().filter(|plot| {
                    matches!(
                        plot.kind(),
                        PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath
                    )
                }) {
                    Some(plot) => {
                        let data = plot
                            .configured_source_rows()?
                            .map_or_else(|| plot.data(), |rows| plot.data().map_source_rows(rows))
                            .with_axis_resources(&resources);
                        Some(crate::plots::line::PreparedLine::new(plot, data, None)?)
                    }
                    None => None,
                },
            );
            scatter.push(
                match plot
                    .as_ref()
                    .filter(|plot| plot.kind() == PlotKind::Scatter)
                {
                    Some(plot) => {
                        let data = plot
                            .configured_source_rows()?
                            .map_or_else(|| plot.data(), |rows| plot.data().map_source_rows(rows))
                            .with_axis_resources(&resources);
                        Some(crate::plots::scatter::PreparedScatter::new(plot, data)?)
                    }
                    None => None,
                },
            );
        }
        let shared_x = self.prepare_shared_x(stage, &scatter, &line, &reference)?;
        for (index, plot) in self.panels.iter().enumerate() {
            interiors.push(
                match plot.as_ref().filter(|plot| {
                    matches!(
                        plot.kind(),
                        PlotKind::Scatter
                            | PlotKind::Histogram
                            | PlotKind::Density
                            | PlotKind::Qq
                            | PlotKind::Regression
                            | PlotKind::Dumbbell
                            | PlotKind::PointInterval
                            | PlotKind::Coefficient
                            | PlotKind::Forest
                            | PlotKind::Bar
                            | PlotKind::HorizontalBar
                            | PlotKind::StackedBar
                            | PlotKind::StackedHorizontalBar
                            | PlotKind::Boxplot
                            | PlotKind::Violin
                            | PlotKind::Raincloud
                            | PlotKind::Interaction
                            | PlotKind::Heatmap
                            | PlotKind::Beeswarm
                            | PlotKind::Waterfall
                            | PlotKind::Marginal
                            | PlotKind::Line
                            | PlotKind::Trajectory
                            | PlotKind::LinePath
                    )
                }) {
                    Some(plot) if reference[index].is_some() => {
                        let data = plot.data().with_axis_resources(&resources);
                        let scope_bytes = 6 + index
                            .checked_ilog10()
                            .map_or(1, |digits| digits as usize + 1);
                        let mut scope = String::new();
                        data.reserve_source_string(&mut scope, scope_bytes)?;
                        std::fmt::Write::write_fmt(&mut scope, format_args!("panel-{index}"))
                            .map_err(|error| Error::Render(format!("Canvas scope: {error}")))?;
                        Some(crate::plots::shared_x::prepare_reference_plot(
                            plot,
                            &resources,
                            reference[index]
                                .take()
                                .expect("prepared reference component"),
                            Some(&scope),
                            shared_x.get(index).copied().flatten(),
                            self.labels.as_ref(),
                        )?)
                    }
                    Some(plot) => {
                        let data = plot.data().with_axis_resources(&resources);
                        let mut scope = String::new();
                        data.reserve_source_string(&mut scope, 32)?;
                        write!(scope, "panel-{index}").expect("admitted scope");
                        let mut plot = plot.admitted_clone(data)?;
                        if let Some(labels) = &self.labels {
                            plot = plot.labels(labels.clone());
                        }
                        let mut plot = plot.with_render_scope(scope);
                        if let Some(Some(range)) = shared_x.get(index) {
                            plot = plot.with_planned_x_limits(*range);
                        }
                        Some(if plot.kind() == PlotKind::Scatter {
                            crate::plots::shared_x::prepare_scatter_plot(
                                &plot,
                                &resources,
                                scatter[index].take().expect("prepared Scatter component"),
                            )?
                        } else {
                            crate::plots::shared_x::prepare_line_plot(
                                &plot,
                                &resources,
                                line[index].take().expect("prepared line component"),
                            )?
                        })
                    }
                    None => None,
                },
            );
        }
        Ok((interiors, shared_x))
    }

    fn preflight_plan(
        &self,
    ) -> Result<
        crate::layout::FrozenRoot<crate::orchestrator::TextReplayPlan, PreparedCanvasInputs>,
        Error,
    > {
        self.preflight_plan_with_sizing((Some(self.width), Some(self.height)))
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one ordered semantic, font, intrinsic, allocation, and capture lifecycle"
    )]
    fn preflight_plan_with_sizing(
        &self,
        sizing_request: (Option<u32>, Option<u32>),
    ) -> Result<
        crate::layout::FrozenRoot<crate::orchestrator::TextReplayPlan, PreparedCanvasInputs>,
        Error,
    > {
        if self.width == 0 || self.height == 0 {
            self.validate_structure()?;
        }
        let mut children = crate::decorations::plan::children::ChildPlans::canvas(self);
        let measurements = sizing::HeightMeasurements::default();
        let root = crate::layout::plan_exact_root(
            sizing_request.0,
            sizing_request.1,
            |prepared_stage| {
                let resources = prepared_stage.generation_resources();
                self.admit_parent_metadata(&resources)?;
                let theme_bytes = self
                    .panels
                    .len()
                    .checked_add(1)
                    .and_then(|n| {
                        n.checked_mul(
                            std::mem::size_of::<ResolvedTheme>()
                                + std::mem::size_of::<Option<ResolvedPlotTheme>>()
                                + 4096,
                        )
                    })
                    .ok_or_else(|| Error::Render("Canvas theme storage overflow".into()))?;
                resources
                    .claim_bytes(
                        u64::try_from(theme_bytes)
                            .map_err(|_| Error::Render("Canvas theme storage overflow".into()))?,
                    )
                    .map_err(|error| Error::Render(format!("Canvas theme admission: {error}")))?;
                let themes = Arc::new(ResolvedCanvasThemes::new(
                    self.theme,
                    &self.panels,
                    &resources,
                )?);
                #[cfg(test)]
                {
                    record_canvas_theme(usize::MAX, "resolved", themes.root());
                    for index in 0..themes.plots.len() {
                        if let Some(theme) = themes.plot(index) {
                            record_canvas_theme(index, "resolved", theme);
                        }
                    }
                }
                self.validate_structure()?;
                let (mut interiors, shared_x) =
                    self.prepare_interiors(prepared_stage, Some(&themes))?;
                self.validate_shared_guide_kinds(&interiors)?;
                let point_path_hue_groups = self.prepared_point_path_hue_groups(
                    &interiors,
                    &prepared_stage.generation_resources(),
                )?;
                #[cfg(test)]
                if point_path_hue_groups.is_some() {
                    for (index, plot) in self.panels.iter().enumerate() {
                        if plot
                            .as_ref()
                            .is_some_and(|plot| plot.hue_dimension_key().is_some())
                        {
                            record_canvas_theme(
                                index,
                                "prepared-shared-hue",
                                themes.plot(index).expect("configured Canvas theme"),
                            );
                        }
                    }
                }
                let discrete_shared_guide = self.prepared_discrete_shared_guide(
                    &themes,
                    &interiors,
                    point_path_hue_groups.as_deref().map(Vec::as_slice),
                    &prepared_stage.generation_resources(),
                )?;
                let continuous_shared_guide = self.prepare_continuous_shared_guide(
                    &themes,
                    &interiors,
                    &prepared_stage.generation_resources(),
                )?;
                let shared_guide = match (discrete_shared_guide, continuous_shared_guide) {
                    (Some(guide), None) => Some(PreparedGuideDescriptor::Discrete(guide)),
                    (None, Some(guide)) => Some(PreparedGuideDescriptor::Continuous(guide)),
                    (None, None) => None,
                    (Some(_), Some(_)) => {
                        return Err(Error::Data(
                            "shared Canvas guide mixes discrete and continuous contributors".into(),
                        ));
                    }
                };
                if let Some(PreparedGuideDescriptor::Continuous(descriptor)) = &shared_guide {
                    self.install_shared_continuous_domain(
                        &mut interiors,
                        descriptor,
                        &prepared_stage.generation_resources(),
                    )?;
                }
                let (mut census, scalar_presence) = self.foundational_text_census(
                    &themes,
                    &interiors,
                    shared_guide.as_ref(),
                    prepared_stage,
                )?;
                themes.prepare_font_environment(&mut census, scalar_presence, prepared_stage)?;
                let (root_text, text_panel_intrinsics, plot_intrinsics) =
                    self.prepare_intrinsics(&themes, &interiors)?;
                Ok(PreparedCanvasInputs {
                    themes,
                    text_census: census,
                    root_text,
                    shared_x,
                    point_path_hue_groups,
                    shared_guide,
                    interiors,
                    text_panel_intrinsics,
                    plot_intrinsics,
                })
            },
            |prepared, assigned, round| {
                let assigned = sizing::measure(self, prepared, assigned, round, &measurements)?;
                if let Some(PreparedGuideDescriptor::Continuous(guide)) = &prepared.shared_guide {
                    round.observe_string(&guide.field)?;
                    round.observe_string(&guide.title)?;
                    round.observe_count(usize::from(guide.exact))?;
                    round.observe_number(guide.domain.0)?;
                    round.observe_number(guide.domain.1)?;
                    if let Some(center) = guide.center {
                        round.observe_number(center)?;
                    }
                    round.observe_count(guide.stops.len())?;
                    for stop in &guide.stops {
                        round.observe_number(stop.offset)?;
                        for channel in [stop.color.0, stop.color.1, stop.color.2] {
                            round.observe_count(usize::from(channel))?;
                        }
                    }
                    round.observe_count(usize::from(guide.missing.is_some()))?;
                }
                prepared
                    .root_text
                    .check_width(
                        f64::from(assigned.dimensions()?.0) - 32.0,
                        &FitScope::root(),
                        round.resources(),
                    )
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                #[cfg(test)]
                let backend_begins =
                    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get);
                let plan = self.build_canvas_plan(
                    Arc::clone(&prepared.themes),
                    assigned,
                    round,
                    &mut children,
                    &prepared.shared_x,
                    prepared.point_path_hue_groups.as_deref().map(Vec::as_slice),
                    prepared.shared_guide.as_ref(),
                    &prepared.text_panel_intrinsics,
                    &prepared.plot_intrinsics,
                    &prepared.interiors,
                    sizing_request.1.is_none(),
                    Some(&measurements),
                )?;
                #[cfg(test)]
                assert_eq!(
                    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
                    backend_begins,
                    "Canvas measurement must not open a backend"
                );
                #[cfg(test)]
                if let Some(inject) = COORDINATION_TEST.with(std::cell::Cell::get) {
                    let ordinal =
                        crate::layout::kernel::ROOT_FEEDBACK_ROUNDS.with(std::cell::Cell::get) - 1;
                    inject(ordinal, self, prepared.themes.root(), round)?;
                }
                children.finish_round(round)?;
                let assigned = crate::layout::RootSize::new(plan.dimensions.0, plan.dimensions.1);
                Ok(crate::layout::MeasuredLayout::new(assigned, plan))
            },
            |_, assigned, plan, final_stage| {
                let (width, height) = assigned.dimensions()?;
                #[cfg(test)]
                let before = admission_tests::capture_counters();
                #[cfg(test)]
                let panel_measurements =
                    crate::orchestrator::RENDER_PANEL_CALLS.with(std::cell::Cell::get);
                let replay = crate::orchestrator::capture_text_replay_plan(
                    width,
                    height,
                    "prepared canvas scene capture",
                    plan.themes.root(),
                    final_stage,
                    |backend| Self::render_planned_canvas(&plan, width, height, backend),
                )?;
                #[cfg(test)]
                assert_eq!(
                    admission_tests::capture_counters(),
                    before,
                    "Canvas first capture must not read, prepare, project, shape, or measure"
                );
                #[cfg(test)]
                assert_eq!(
                    crate::orchestrator::RENDER_PANEL_CALLS.with(std::cell::Cell::get),
                    panel_measurements,
                    "Canvas capture must not enter the panel measurement path"
                );
                Ok(replay)
            },
        )?;
        let (width, height) = root.dimensions();
        crate::orchestrator::validate_text_replay_plan(
            width,
            height,
            root.plan(),
            "prepared canvas Null replay",
        )?;
        Ok(root)
    }

    #[expect(
        clippy::type_complexity,
        reason = "one prepared root and both cell-intrinsic inventories are retained before parent allocation"
    )]
    fn prepare_intrinsics(
        &self,
        themes: &ResolvedCanvasThemes,
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
    ) -> Result<
        (
            crate::decorations::title_block::plan::RootTitleBlockIntrinsics,
            Vec<Option<text_panel::TextPanelIntrinsics>>,
            Vec<Option<crate::decorations::title_block::plan::RootTitleBlockIntrinsics>>,
        ),
        Error,
    > {
        // All summaries use the prepared labels and the actual owner theme,
        // after the font census and before any parent allocation or child probe.
        for (index, inputs) in interiors.iter().enumerate() {
            if let Some(inputs) = inputs {
                let resources = themes
                    .plot(index)
                    .expect("prepared Canvas panel theme")
                    .prepared_font_environment()?
                    .generation_resources();
                let source = self.panels[index].as_ref().expect("prepared Canvas panel");
                let plot = self.cell_visibility(
                    source.admitted_clone(source.data().with_axis_resources(&resources))?,
                    index,
                );
                inputs.prepare_intrinsics(
                    &plot,
                    themes.plot(index).expect("prepared Canvas panel theme"),
                )?;
            }
        }
        let mut text_panels = Vec::new();
        crate::layout::fit::reserve(
            themes.root(),
            &mut text_panels,
            self.text_panels.len(),
            "TextPanel intrinsic summaries",
        )?;
        for panel in &self.text_panels {
            text_panels.push(
                panel
                    .as_ref()
                    .map(|panel| text_panel::TextPanelIntrinsics::prepare(panel, themes.root()))
                    .transpose()?,
            );
        }
        let mut plot_titles = Vec::new();
        crate::layout::fit::reserve(
            themes.root(),
            &mut plot_titles,
            self.panels.len(),
            "Canvas Plot text intrinsics",
        )?;
        let resources = themes
            .root()
            .prepared_font_environment()?
            .generation_resources();
        for (index, plot) in self.panels.iter().enumerate() {
            let summary = plot
                .as_ref()
                .map(|plot| -> Result<_, Error> {
                    let data = plot.data().with_axis_resources(&resources);
                    let mut scope = String::new();
                    data.reserve_source_string(&mut scope, 32)?;
                    write!(scope, "panel-{index}").expect("reserved Canvas scope");
                    crate::decorations::title_block::plan::RootTitleBlockPlan::intrinsic_widths(
                        themes.plot(index).expect("configured Canvas Plot theme"),
                        [plot.title_text(), plot.subtitle_text(), plot.caption_text()],
                        Some(&scope),
                    )
                })
                .transpose()?;
            plot_titles.push(summary);
        }
        let root = crate::decorations::title_block::plan::RootTitleBlockPlan::intrinsic_widths(
            themes.root(),
            [
                self.title.as_deref(),
                self.subtitle.as_deref(),
                self.caption.as_deref(),
            ],
            None,
        )?;
        Ok((root, text_panels, plot_titles))
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the Canvas census keeps canonical root, composition, and panel preorder explicit"
    )]
    fn foundational_text_census(
        &self,
        themes: &ResolvedCanvasThemes,
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
        shared_guide: Option<&PreparedGuideDescriptor>,
        prepared_stage: &mut crate::layout::kernel::PreparedStage<'_>,
    ) -> Result<(crate::layout::PreparedTextCensus, Vec<u64>), Error> {
        let root = crate::layout::ComponentAddress::root();
        let mut requirements = Vec::new();
        for (source, role, address_role, appearance) in [
            (
                self.title.as_deref(),
                crate::layout::PreparedTextRole::Title,
                crate::layout::AddressRole::RootTitle,
                &themes.root().typography.root.title,
            ),
            (
                self.subtitle.as_deref(),
                crate::layout::PreparedTextRole::Subtitle,
                crate::layout::AddressRole::RootSubtitle,
                &themes.root().typography.root.subtitle,
            ),
            (
                self.caption.as_deref(),
                crate::layout::PreparedTextRole::Caption,
                crate::layout::AddressRole::RootCaption,
                &themes.root().typography.root.caption,
            ),
        ] {
            if let Some(source) = source.filter(|source| !source.is_empty()) {
                let address =
                    root.child(address_role, 0, &prepared_stage.generation_resources())?;
                push_text_requirement(
                    &mut requirements,
                    address,
                    role,
                    0,
                    source,
                    appearance,
                    prepared_stage,
                )?;
            }
        }

        if let Some(guide) = shared_guide {
            let guide_owner = root.child(
                crate::layout::AddressRole::CanvasSharedLegend,
                0,
                &prepared_stage.generation_resources(),
            )?;
            let title_appearance = &themes.root().typography.guides.legend_title;
            let label_appearance = &themes.root().typography.guides.legend_label;
            let title = match guide {
                PreparedGuideDescriptor::Discrete((title, _)) => title.as_str(),
                PreparedGuideDescriptor::Continuous(guide) => guide.title.as_str(),
            };
            if !title.is_empty() {
                let address = guide_owner.child(
                    match guide {
                        PreparedGuideDescriptor::Discrete(_) => {
                            crate::layout::AddressRole::LegendTitle
                        }
                        PreparedGuideDescriptor::Continuous(_) => {
                            crate::layout::AddressRole::GuideTitle
                        }
                    },
                    0,
                    &prepared_stage.generation_resources(),
                )?;
                push_text_requirement(
                    &mut requirements,
                    address,
                    crate::layout::PreparedTextRole::LegendTitle,
                    0,
                    title,
                    title_appearance,
                    prepared_stage,
                )?;
            }
            let (discrete, continuous) = match guide {
                PreparedGuideDescriptor::Discrete((_, entries)) => (Some(entries), None),
                PreparedGuideDescriptor::Continuous(guide) => (None, Some(&guide.labels)),
            };
            let labels = discrete
                .into_iter()
                .flat_map(|entries| {
                    entries
                        .iter()
                        .map(crate::decorations::legend::LegendEntry::display_label)
                })
                .chain(
                    continuous
                        .into_iter()
                        .flat_map(|labels| labels.iter().map(String::as_str)),
                );
            for (index, label) in labels.enumerate() {
                let ordinal = u32::try_from(index)
                    .map_err(|_| Error::Render("shared guide label ordinal overflow".into()))?;
                let address = guide_owner.child(
                    match guide {
                        PreparedGuideDescriptor::Discrete(_) => {
                            crate::layout::AddressRole::LegendLabel
                        }
                        PreparedGuideDescriptor::Continuous(_) => {
                            crate::layout::AddressRole::GuideLabel
                        }
                    },
                    ordinal,
                    &prepared_stage.generation_resources(),
                )?;
                push_text_requirement(
                    &mut requirements,
                    address,
                    crate::layout::PreparedTextRole::LegendLabel,
                    ordinal,
                    label,
                    label_appearance,
                    prepared_stage,
                )?;
            }
            if let Some(missing) = match guide {
                PreparedGuideDescriptor::Discrete(_) => None,
                PreparedGuideDescriptor::Continuous(guide) => guide.missing.as_ref(),
            } {
                let address = guide_owner.child(
                    crate::layout::AddressRole::GuideMissingBlock,
                    0,
                    &prepared_stage.generation_resources(),
                )?;
                push_text_requirement(
                    &mut requirements,
                    address,
                    crate::layout::PreparedTextRole::LegendLabel,
                    0,
                    &missing.label,
                    label_appearance,
                    prepared_stage,
                )?;
            }
        }

        let composition_title = &themes.root().typography.composition.composition_header;
        let text_panel_title = &themes.root().typography.composition.text_panel_title;
        let body = &themes.root().typography.composition.body;
        for (index, header) in self.column_headers.iter().enumerate() {
            let address = root.child(
                crate::layout::AddressRole::CanvasHeader,
                u32::try_from(index).map_err(|_| {
                    Error::Render("canvas column-header census index overflowed".into())
                })?,
                &prepared_stage.generation_resources(),
            )?;
            push_text_requirement(
                &mut requirements,
                address,
                crate::layout::PreparedTextRole::Header,
                0,
                &header.text,
                composition_title,
                prepared_stage,
            )?;
        }
        for (index, header) in self.row_headers.iter().enumerate() {
            let address = root.child(
                crate::layout::AddressRole::CanvasHeader,
                u32::try_from(self.column_headers.len() + index).map_err(|_| {
                    Error::Render("canvas row-header census index overflowed".into())
                })?,
                &prepared_stage.generation_resources(),
            )?;
            push_text_requirement(
                &mut requirements,
                address,
                crate::layout::PreparedTextRole::Header,
                0,
                &header.text,
                composition_title,
                prepared_stage,
            )?;
        }
        for (index, title) in self.row_titles.iter().enumerate() {
            let address = root.child(
                crate::layout::AddressRole::CanvasRowTitle,
                u32::try_from(index).map_err(|_| {
                    Error::Render("canvas row-title census index overflowed".into())
                })?,
                &prepared_stage.generation_resources(),
            )?;
            push_text_requirement(
                &mut requirements,
                address,
                crate::layout::PreparedTextRole::Header,
                0,
                &title.text,
                composition_title,
                prepared_stage,
            )?;
        }
        for (index, panel) in self.text_panels.iter().enumerate() {
            let Some(panel) = panel else { continue };
            let address = root.child(
                crate::layout::AddressRole::TextPanelContent,
                u32::try_from(index)
                    .map_err(|_| Error::Render("text-panel census index overflowed".into()))?,
                &prepared_stage.generation_resources(),
            )?;
            for (role, local_index, source, appearance) in [
                (
                    crate::layout::PreparedTextRole::Header,
                    0,
                    panel.title(),
                    text_panel_title,
                ),
                (crate::layout::PreparedTextRole::Body, 0, panel.body(), body),
            ] {
                let requirement_address = address.clone();
                push_text_requirement(
                    &mut requirements,
                    requirement_address,
                    role,
                    local_index,
                    source,
                    appearance,
                    prepared_stage,
                )?;
            }
        }

        for (index, plot) in self.panels.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let resources = prepared_stage.generation_resources();
            let mut plot = plot.admitted_clone(plot.data().with_axis_resources(&resources))?;
            if let Some(labels) = &self.labels {
                plot = plot.labels(labels.clone());
            }
            let plot = self.cell_visibility(plot, index);
            let theme = themes
                .plot(index)
                .expect("configured canvas panel has a resolved theme");
            let cell = root.child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("canvas text census index overflowed".into()))?,
                &prepared_stage.generation_resources(),
            )?;
            let filter_point_label_x_to_limits = !self
                .x_range_groups
                .iter()
                .any(|group| group.contains(&index));
            crate::layout::PreparedTextRequirement::append_plot(
                interiors.get(index).and_then(Option::as_ref),
                &mut requirements,
                &plot,
                plot.kind(),
                theme,
                &cell,
                true,
                filter_point_label_x_to_limits,
                true,
                Some(prepared_stage),
            )?;
        }
        crate::layout::PreparedTextRequirement::append_preference_metrics(
            &mut requirements,
            themes.root(),
            &root,
            prepared_stage,
        )?;
        for (index, plot) in self.panels.iter().enumerate() {
            if plot.is_some() {
                let owner = root.child(
                    crate::layout::AddressRole::CanvasCell,
                    u32::try_from(index).map_err(|_| {
                        Error::Render("Canvas metric census owner overflowed".into())
                    })?,
                    &prepared_stage.generation_resources(),
                )?;
                crate::layout::PreparedTextRequirement::append_preference_metrics(
                    &mut requirements,
                    themes
                        .plot(index)
                        .expect("configured Canvas Plot retains theme"),
                    &owner,
                    prepared_stage,
                )?;
            }
        }
        let text_topology =
            crate::layout::PreparedTextCensus::structural_address_count(&requirements)
                .map_err(|error| Error::Render(format!("prepared canvas text census: {error}")))?;
        let mut scalar_presence = prepared_stage.reserve_scalar_presence_map()?;
        let census = crate::layout::PreparedTextCensus::materialize(
            requirements,
            Some(prepared_stage),
            Some(&mut scalar_presence),
        )
        .map_err(|error| Error::Render(format!("prepared canvas text census: {error}")))?;
        if census.counts.structural_addresses != text_topology {
            return Err(Error::Render(format!(
                "prepared canvas text topology counted {text_topology} addresses but materialized {}",
                census.counts.structural_addresses
            )));
        }
        Ok((census, scalar_presence))
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "one Canvas round consumes separate retained domains, hue identities, and child inputs"
    )]
    fn build_canvas_plan(
        &self,
        themes: Arc<ResolvedCanvasThemes>,
        assigned: crate::layout::RootSize,
        round: &mut crate::layout::kernel::FeedbackRound,
        children: &mut crate::decorations::plan::children::ChildPlans,
        shared_x: &[Option<(f64, f64)>],
        prepared_scatter_hues: Option<&[(String, Vec<String>)]>,
        prepared_guide: Option<&PreparedGuideDescriptor>,
        text_panel_intrinsics: &[Option<text_panel::TextPanelIntrinsics>],
        plot_intrinsics: &[Option<
            crate::decorations::title_block::plan::RootTitleBlockIntrinsics,
        >],
        interiors: &[Option<crate::layout::interior::PreparedInteriorInputs>],
        content_sizing: bool,
        measurements: Option<&sizing::HeightMeasurements>,
    ) -> Result<CanvasPlan, Error> {
        use crate::layout::fit::FitOutcome;
        use crate::layout::kernel::TrialOutcome;
        let prepared = self.prepare_canvas_round(
            themes,
            assigned,
            round,
            children,
            shared_x,
            prepared_scatter_hues,
            prepared_guide,
            text_panel_intrinsics,
            plot_intrinsics,
            interiors,
            content_sizing,
            measurements,
        )?;
        let mut compact_changes = true;
        if self.legend_policy != CanvasLegendPolicy::Shared
            || (prepared.shared_legend.is_none()
                && !matches!(
                    prepared.shared_guide,
                    Some(PreparedGuideDescriptor::Continuous(_))
                ))
        {
            return match self.build_canvas_attempt(
                &prepared,
                assigned,
                round,
                None,
                &mut compact_changes,
                children,
            )? {
                FitOutcome::Measured(plan) => Ok(plan),
                FitOutcome::Miss(reason) => Err(reason.into_error()),
            };
        }
        let owner = crate::layout::ComponentAddress::root().child(
            crate::layout::AddressRole::CanvasSharedLegend,
            0,
            round.resources(),
        )?;
        let mut first_miss = None;
        for (edge_index, edge) in [
            crate::LegendPosition::OutsideRight,
            crate::LegendPosition::OutsideTop,
            crate::LegendPosition::OutsideBottom,
            crate::LegendPosition::OutsideLeft,
        ]
        .into_iter()
        .enumerate()
        {
            compact_changes = true;
            for compact in [false, true] {
                if compact && !compact_changes {
                    continue;
                }
                round.claim_guide_trial(
                    &owner,
                    8,
                    u32::try_from(edge_index * 2 + usize::from(compact))
                        .map_err(|_| Error::Render("Canvas guide ordinal overflowed".into()))?,
                )?;
                let checkpoint = round.begin_trial()?;
                #[cfg(test)]
                let guide_trial = (prepared.shared_legend.is_some()
                    || matches!(
                        prepared.shared_guide,
                        Some(PreparedGuideDescriptor::Continuous(_))
                    ))
                .then(|| crate::orchestrator::measurements::GuideTrial::new(&owner, edge, compact));
                round.observe_count(edge as usize)?;
                round.observe_count(usize::from(compact))?;
                let outcome = self.build_canvas_attempt(
                    &prepared,
                    assigned,
                    round,
                    Some((edge, compact)),
                    &mut compact_changes,
                    children,
                )?;
                match outcome {
                    FitOutcome::Measured(plan) => match round.finish_trial(checkpoint)? {
                        _outcome @ (TrialOutcome::Fit | TrialOutcome::Provisional) => {
                            #[cfg(test)]
                            #[expect(
                                clippy::used_underscore_binding,
                                reason = "outcome is observed only in tests"
                            )]
                            if let Some(trial) = &guide_trial {
                                trial.complete(&_outcome);
                            }
                            return Ok(plan);
                        }
                        TrialOutcome::Miss(reason) => {
                            #[cfg(test)]
                            if let Some(trial) = &guide_trial {
                                trial.reject();
                            }
                            FitFailure::retain_least(&mut first_miss, reason);
                        }
                    },
                    FitOutcome::Miss(reason) => {
                        #[cfg(test)]
                        if let Some(trial) = &guide_trial {
                            trial.reject();
                        }
                        round.record_failure(reason)?;
                        if let TrialOutcome::Miss(reason) = round.finish_trial(checkpoint)? {
                            FitFailure::retain_least(&mut first_miss, reason);
                        }
                    }
                }
            }
        }
        Err(first_miss
            .unwrap_or_else(|| {
                FitScope::new(&owner, 0).failure(
                    FitKind::LegendOrGuide,
                    "shared guide cannot fit its containing Canvas frame",
                )
            })
            .into_error())
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "admitted scoped configuration and shared-guide preparation form one parent round"
    )]
    fn prepare_canvas_round<'a>(
        &self,
        themes: Arc<ResolvedCanvasThemes>,
        assigned: crate::layout::RootSize,
        round: &mut crate::layout::kernel::FeedbackRound,
        children: &mut crate::decorations::plan::children::ChildPlans<'_>,
        shared_x: &'a [Option<(f64, f64)>],
        prepared_scatter_hues: Option<&[(String, Vec<String>)]>,
        prepared_guide: Option<&'a PreparedGuideDescriptor>,
        text_panel_intrinsics: &'a [Option<text_panel::TextPanelIntrinsics>],
        plot_intrinsics: &'a [Option<
            crate::decorations::title_block::plan::RootTitleBlockIntrinsics,
        >],
        interiors: &'a [Option<crate::layout::interior::PreparedInteriorInputs>],
        content_sizing: bool,
        measurements: Option<&'a sizing::HeightMeasurements>,
    ) -> Result<PreparedCanvasRound<'a>, Error> {
        let (width, height) = assigned.dimensions()?;
        #[cfg(test)]
        record_canvas_theme(usize::MAX, "layout", themes.root());
        self.admit_parent_metadata(
            &themes
                .root()
                .prepared_font_environment()?
                .generation_resources(),
        )?;
        let canvas_layout = RootFramePlan::measure(
            (width, height),
            themes.root(),
            [
                self.title.as_deref(),
                self.subtitle.as_deref(),
                self.caption.as_deref(),
            ],
            crate::decorations::title_block::plan::RootAlignments::resolve(
                themes.root(),
                [self.title_alignment, self.subtitle_alignment],
                [None; 2],
            ),
            (
                crate::layout::canvas::MINIMUM_PANEL_SPAN,
                crate::layout::canvas::MINIMUM_PANEL_SPAN,
            ),
        )
        .map_err(crate::layout::fit::PlanError::into_error)?;
        let full_outer = canvas_layout.panel_box;
        round.observe_canvas(&canvas_layout)?;
        let resources = themes
            .root()
            .prepared_font_environment()?
            .generation_resources();
        let mut plots = Vec::new();
        crate::layout::fit::reserve(
            themes.root(),
            &mut plots,
            self.panels.len(),
            "Canvas scoped configurations",
        )?;
        for (index, source) in self.panels.iter().enumerate() {
            let Some(source) = source else {
                plots.push(None);
                continue;
            };
            let data = source.data().with_axis_resources(&resources);
            let mut scope = String::new();
            data.reserve_source_string(&mut scope, 32)?;
            write!(scope, "panel-{index}").expect("admitted scope");
            let mut plot = source.admitted_clone(data)?;
            if let Some(labels) = &self.labels {
                plot = plot.labels(labels.clone());
            }
            plot = plot.with_render_scope(scope);
            if let Some(inputs) = interiors.get(index).and_then(Option::as_ref) {
                plot = inputs.attach(plot)?;
            }
            if let Some(Some(range)) = shared_x.get(index) {
                let kind = plot.kind();
                plot = plot.with_resolved_facet_ranges(kind, Some(*range), None);
            }
            plot = self.cell_visibility(plot, index);
            plots.push(Some(plot));
        }
        let fallback_hues;
        let hue_groups: &[(String, Vec<String>)] = if self.legend_policy
            != CanvasLegendPolicy::Shared
        {
            &[]
        } else if let Some(hues) = prepared_scatter_hues {
            hues
        } else {
            fallback_hues = Self::canvas_hue_groups(&plots, full_outer, &themes, children, round)?;
            &fallback_hues
        };
        for slot in &mut plots {
            if let Some(plot) = slot.take() {
                let lookup = hue_groups
                    .len()
                    .checked_mul(plot.hue_dimension_key().map_or(1, |key| key.len().max(1)))
                    .ok_or_else(|| {
                        Error::Render("Canvas coordinated hue lookup overflow".into())
                    })?;
                resources.claim_visits(lookup as u128).map_err(|error| {
                    Error::Render(format!(
                        "Canvas coordinated hue lookup admission: {error:?}"
                    ))
                })?;
                let levels = hue_groups
                    .iter()
                    .find(|(dimension, _)| Some(dimension.as_str()) == plot.hue_dimension_key());
                let plot = if let Some((_, levels)) = levels {
                    let data = plot.data().with_axis_resources(&resources);
                    let levels = data.collect_source(
                        levels.iter().map(|level| data.copy_source_string(level)),
                    )?;
                    plot.with_global_hue_levels(levels)
                } else {
                    plot
                };
                *slot = Some(
                    plot.with_canvas_size(width, height)
                        .with_parent_axis_gutters(),
                );
            }
        }
        let (shared_legend, body_probes) = if self.legend_policy == CanvasLegendPolicy::Shared {
            Self::preflight_shared_legend(
                &plots,
                &themes,
                full_outer,
                children,
                round,
                match prepared_guide {
                    Some(PreparedGuideDescriptor::Discrete(guide)) => Some(guide),
                    Some(PreparedGuideDescriptor::Continuous(_)) | None => None,
                },
            )?
        } else {
            (None, Vec::new())
        };
        if let Some((_, entries)) = &shared_legend {
            if !matches!(prepared_guide, Some(PreparedGuideDescriptor::Discrete(_))) {
                resources
                    .claim_bytes(
                        u64::try_from(
                            std::mem::size_of_val(entries.as_ref())
                                + 2 * std::mem::size_of::<usize>(),
                        )
                        .map_err(|_| Error::Render("Canvas legend Arc overflow".into()))?,
                    )
                    .map_err(|error| {
                        Error::Render(format!("Canvas legend Arc admission: {error}"))
                    })?;
            }
        }
        Ok(PreparedCanvasRound {
            content_sizing,
            measurements,
            interiors,
            themes,
            canvas_layout,
            plots,
            shared_legend,
            shared_guide: prepared_guide,
            body_probes,
            shared_x,
            text_panel_intrinsics,
            plot_intrinsics,
        })
    }

    fn cell_visibility(&self, mut plot: Plot, index: usize) -> Plot {
        if self.legend_policy == CanvasLegendPolicy::Hidden {
            plot = plot
                .with_suppressed_legend()
                .with_suppressed_continuous_guide();
        } else if self.legend_policy == CanvasLegendPolicy::Shared
            && matches!(plot.kind(), PlotKind::Heatmap | PlotKind::Beeswarm)
        {
            plot = plot.with_suppressed_continuous_guide();
        }
        if self.category_labels == CanvasCategoryLabels::FirstColumnPerRow
            && !index.is_multiple_of(self.columns)
        {
            plot = plot.with_y_tick_labels_visible(false);
        }
        plot
    }

    /// Install the same coordinated domain and visibility inputs before every
    /// width-dependent height pass and the final assigned child measurements.
    fn coordinate_plots(
        &self,
        plots: &mut [Option<Plot>],
        shared_x: &[Option<(f64, f64)>],
        round: &crate::layout::kernel::FeedbackRound,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<(), Error> {
        for group in &self.x_range_groups {
            let mut shared: Option<(f64, f64)> = None;
            for &index in group {
                resources.claim_visits(1).map_err(|error| {
                    Error::Render(format!("Canvas shared domain coordination: {error:?}"))
                })?;
                let base = shared_x[index].expect("prepared shared x member");
                let address = crate::layout::ComponentAddress::root().child(
                    crate::layout::AddressRole::CanvasCell,
                    u32::try_from(index).map_err(|_| {
                        Error::Render("Canvas coordinated address overflowed".into())
                    })?,
                    resources,
                )?;
                let range = round
                    .domain(&address, FeedbackKind::AutomaticDomainX)?
                    .map_or(base, |range| (base.0.min(range.0), base.1.max(range.1)));
                shared = Some(shared.map_or(range, |current| {
                    (current.0.min(range.0), current.1.max(range.1))
                }));
            }
            let shared = shared.expect("validated nonempty x-range group");
            if !shared.0.is_finite() || !shared.1.is_finite() || shared.0 >= shared.1 {
                return Err(Error::Render(
                    "Canvas shared x domain must be finite and ascending".into(),
                ));
            }
            for &index in group {
                let plot = plots[index].take().expect("validated grouped panel");
                let kind = plot.kind();
                plots[index] = Some(plot.with_resolved_facet_ranges(kind, Some(shared), None));
            }
        }
        for (index, slot) in plots.iter_mut().enumerate() {
            let Some(mut plot) = slot.take() else {
                continue;
            };
            let address = crate::layout::ComponentAddress::root().child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("Canvas coordinated address overflowed".into()))?,
                resources,
            )?;
            let mut grouped = false;
            for group in &self.x_range_groups {
                for &member in group {
                    resources.claim_visits(1).map_err(|error| {
                        Error::Render(format!("Canvas domain member lookup: {error:?}"))
                    })?;
                    if member == index {
                        grouped = true;
                        break;
                    }
                }
                if grouped {
                    break;
                }
            }
            if !grouped {
                if let Some(range) = round.domain(&address, FeedbackKind::AutomaticDomainX)? {
                    plot = plot.with_planned_x_limits(range);
                }
            }
            if let Some(range) = round.domain(&address, FeedbackKind::AutomaticDomainY)? {
                plot = plot.with_planned_y_limits(range);
            }
            *slot = Some(self.cell_visibility(plot, index));
        }
        Ok(())
    }

    fn complete_cell_gutters(
        plots: &[Option<Plot>],
        cells: &[PanelBox],
        themes: &ResolvedCanvasThemes,
        children: &mut crate::decorations::plan::children::ChildPlans,
        round: &crate::layout::kernel::FeedbackRound,
        requirements: &mut [(usize, usize, PanelRequirements)],
    ) -> Result<(), Error> {
        for (index, plot) in plots.iter().enumerate() {
            let Some(plot) = plot else {
                continue;
            };
            let theme = themes.plot(index).expect("configured Canvas plot theme");
            let probe = children.canvas_probe(
                plot,
                cells[index],
                theme,
                index,
                MeasurementPhase::Probe,
                round,
            )?;
            let resources = theme.prepared_font_environment()?.generation_resources();
            let scope = FitScope::root().child(
                crate::layout::AddressRole::CanvasCell,
                index,
                &resources,
            )?;
            let measured = PanelRequirements::measure_header(
                probe
                    .axes
                    .as_ref()
                    .ok_or_else(|| Error::Render("Canvas probe is missing measured axes".into()))?,
                theme,
                None,
                None,
                cells[index],
                &scope,
            )
            .map_err(crate::layout::fit::PlanError::into_error)?;
            let target = &mut requirements[index].2;
            target.left = target.left.max(measured.left);
            target.right = target.right.max(measured.right);
            target.bottom = target.bottom.max(measured.bottom);
            target.top = target.top.max(measured.top);
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_lines,
        clippy::cast_precision_loss,
        reason = "one Canvas trial allocates declared tracks and measures every affected child"
    )]
    fn build_canvas_attempt(
        &self,
        prepared: &PreparedCanvasRound,
        assigned: crate::layout::RootSize,
        round: &mut crate::layout::kernel::FeedbackRound,
        guide_trial: Option<(crate::LegendPosition, bool)>,
        compact_changes: &mut bool,
        children: &mut crate::decorations::plan::children::ChildPlans,
    ) -> Result<crate::layout::fit::FitOutcome<CanvasPlan>, Error> {
        use crate::layout::fit::{self, FitOutcome};
        let root_address = crate::layout::ComponentAddress::root();
        let guide_owner = root_address.child(
            crate::layout::AddressRole::CanvasSharedLegend,
            0,
            round.resources(),
        )?;
        let scope = FitScope::new(&root_address, round.attempt_ordinal(&guide_owner)?);
        let guide_scope = FitScope::new(&guide_owner, scope.ordinal);
        let themes = Arc::clone(&prepared.themes);
        let mut canvas_layout = prepared.canvas_layout.clone();
        let mut full_outer = canvas_layout.panel_box;
        let resources = themes
            .root()
            .prepared_font_environment()?
            .generation_resources();
        self.admit_parent_metadata(&resources)?;
        let mut plots = Vec::new();
        fit::reserve(
            themes.root(),
            &mut plots,
            prepared.plots.len(),
            "Canvas candidate configurations",
        )?;
        for plot in &prepared.plots {
            plots.push(
                plot.as_ref()
                    .map(|plot| plot.admitted_clone(plot.data().with_axis_resources(&resources)))
                    .transpose()?,
            );
        }
        self.coordinate_plots(&mut plots, prepared.shared_x, round, &resources)?;
        let natural_grid_height = |width: f64,
                                   round: &crate::layout::kernel::FeedbackRound|
         -> Result<FitOutcome<f64>, Error> {
            // Both height composition and final allocation consume the same
            // root-owned gutters. A probe at a provisional height can choose
            // different ticks and must not replace those scoped inputs locally.
            let gutters = sizing::grid_gutters(self, themes.root(), round)?;
            let grid = tracks::allocate(
                PanelBox::new(0.0, width, 0.0, 0.0),
                (self.rows, self.columns),
                &plots,
                prepared.text_panel_intrinsics,
                prepared.plot_intrinsics,
                &self.column_headers,
                &self.row_headers,
                &self.row_titles,
                &gutters,
                &themes,
                &scope,
                Some(&tracks::ContentSizing {
                    source: self,
                    interiors: prepared.interiors,
                    round,
                    measurements: prepared.measurements,
                    natural_width: false,
                    natural_height: true,
                }),
            )?;
            Ok(match grid {
                FitOutcome::Measured(grid) => FitOutcome::Measured(grid.size.1),
                FitOutcome::Miss(reason) => FitOutcome::Miss(reason),
            })
        };
        let mut outer = full_outer;
        let shared_legend = if prepared.shared_legend.is_some()
            || matches!(
                prepared.shared_guide,
                Some(PreparedGuideDescriptor::Continuous(_))
            ) {
            let (edge, compact) = guide_trial.expect("shared guide has an owned trial");
            let horizontal = matches!(
                edge,
                crate::LegendPosition::OutsideTop | crate::LegendPosition::OutsideBottom
            );
            let metrics = crate::decorations::legend::LegendMetrics::from_theme(themes.root());
            let gap = self.shared_legend_gap.unwrap_or(metrics.shared_gap);
            let grid_gap = themes.root().composition.canvas_grid_gap;
            let positive_span = crate::layout::canvas::MINIMUM_PANEL_SPAN;
            let mut columns = Vec::new();
            let mut rows = Vec::new();
            fit::reserve(
                themes.root(),
                &mut columns,
                self.columns,
                "Canvas guide column minima",
            )?;
            fit::reserve(
                themes.root(),
                &mut rows,
                self.rows,
                "Canvas guide row minima",
            )?;
            columns.resize(self.columns, positive_span);
            rows.resize(self.rows, positive_span);
            resources
                .claim_visits(plots.len() as u128)
                .map_err(|error| {
                    Error::Render(format!("Canvas guide child intrinsic visits: {error:?}"))
                })?;
            for (index, plot) in plots.iter().enumerate() {
                let hard = if let Some(plot) = plot {
                    plot.intrinsic_body_minimum(
                        themes.plot(index).expect("Plot retains its theme"),
                    )?
                } else if let Some(text) = prepared.text_panel_intrinsics[index] {
                    text.minimum
                } else {
                    continue;
                };
                columns[index % self.columns] = columns[index % self.columns].max(hard.0);
                rows[index / self.columns] = rows[index / self.columns].max(hard.1);
            }
            let minimum_width =
                columns.iter().sum::<f64>() + self.columns.saturating_sub(1) as f64 * grid_gap;
            let minimum_height =
                rows.iter().sum::<f64>() + self.rows.saturating_sub(1) as f64 * grid_gap;
            if !minimum_width.is_finite() || !minimum_height.is_finite() {
                return Err(Error::Render(
                    "Canvas shared-guide child tracks overflowed".into(),
                ));
            }
            let frame = crate::decorations::plan::AssignedGuideFrame::new(
                edge,
                (minimum_width, minimum_height),
                gap,
            );
            let available = frame.available(full_outer, [0.0; 4]);
            let measured = match prepared.shared_guide {
                Some(PreparedGuideDescriptor::Continuous(descriptor)) => {
                    match MeasuredContinuousGuide::measure(
                        descriptor,
                        (
                            available.0,
                            (!prepared.content_sizing).then_some(available.1),
                        ),
                        horizontal,
                        compact,
                        themes.root(),
                        compact_changes,
                        &guide_scope,
                    )? {
                        FitOutcome::Measured(guide) => MeasuredCanvasGuide::Continuous(guide),
                        FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
                    }
                }
                Some(PreparedGuideDescriptor::Discrete(_)) | None => {
                    let (title, entries) = prepared
                        .shared_legend
                        .as_ref()
                        .expect("prepared discrete guide");
                    match Legend::measure_assigned(
                        title,
                        Arc::clone(entries),
                        (
                            available.0,
                            (!prepared.content_sizing).then_some(available.1),
                        ),
                        horizontal,
                        compact,
                        themes.root(),
                        compact_changes,
                        &guide_scope,
                    )? {
                        FitOutcome::Measured(legend) => MeasuredCanvasGuide::Discrete(legend),
                        FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
                    }
                }
            };
            let (w, h) = measured.dimensions();
            if prepared.content_sizing {
                let grid_width = full_outer.width() - if horizontal { 0.0 } else { w + gap };
                let height = match natural_grid_height(grid_width, round)? {
                    FitOutcome::Measured(height) => height,
                    FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
                };
                let height = if horizontal {
                    height + h + gap
                } else {
                    height.max(h)
                };
                canvas_layout =
                    sizing::content_frame(self, themes.root(), assigned.dimensions()?.0, height)?;
                full_outer = canvas_layout.panel_box;
            }
            #[cfg(test)]
            crate::orchestrator::measurements::guide_body();
            outer = match frame.assign_fit(full_outer, Some((w, h)), [0.0; 4], &scope)? {
                FitOutcome::Measured(panel) => panel,
                FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
            };
            let placed = frame.place(outer, (w, h));
            let (left, bottom) = (placed.x0, placed.y0);
            Some(match measured {
                MeasuredCanvasGuide::Discrete(legend) => {
                    let bounds = legend.box_at(left, bottom);
                    SharedMeasuredGuide::Discrete(crate::decorations::plan::ResolvedLegend {
                        legend,
                        bounds,
                    })
                }
                MeasuredCanvasGuide::Continuous(guide) => SharedMeasuredGuide::Continuous {
                    guide,
                    bounds: placed,
                },
            })
        } else {
            if prepared.content_sizing {
                let height = match natural_grid_height(full_outer.width(), round)? {
                    FitOutcome::Measured(height) => height,
                    FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
                };
                canvas_layout =
                    sizing::content_frame(self, themes.root(), assigned.dimensions()?.0, height)?;
                full_outer = canvas_layout.panel_box;
                outer = full_outer;
            }
            None
        };
        let current_height = crate::layout::interior::sizing::integral(
            canvas_layout.panel_box.y1 + canvas_layout.insets.top,
        )?;
        for slot in &mut plots {
            if let Some(plot) = slot.take() {
                *slot = Some(plot.with_canvas_size(assigned.dimensions()?.0, current_height));
            }
        }
        let provisional = match tracks::allocate(
            outer,
            (self.rows, self.columns),
            &plots,
            prepared.text_panel_intrinsics,
            prepared.plot_intrinsics,
            &self.column_headers,
            &self.row_headers,
            &self.row_titles,
            &[],
            &themes,
            &scope.child(
                crate::layout::AddressRole::CanvasGrid,
                0,
                &themes
                    .root()
                    .prepared_font_environment()?
                    .generation_resources(),
            )?,
            prepared
                .content_sizing
                .then_some(tracks::ContentSizing {
                    source: self,
                    interiors: prepared.interiors,
                    round,
                    measurements: prepared.measurements,
                    natural_width: false,
                    natural_height: false,
                })
                .as_ref(),
        )? {
            FitOutcome::Measured(grid) => grid.cells,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        if self.category_labels == CanvasCategoryLabels::FirstColumnPerRow {
            for row in 0..self.rows {
                let first_index = row * self.columns;
                let first = plots[first_index].as_ref().ok_or_else(|| {
                    Error::Data(format!(
                        "canvas row {row} requires a configured first-column panel for shared category labels"
                    ))
                })?;
                let expected_theme = themes
                    .plot(first_index)
                    .expect("configured canvas panel has a resolved theme");
                #[cfg(test)]
                record_canvas_theme(
                    first_index,
                    "shared-category-label-validation",
                    expected_theme,
                );
                let expected = plot_category_domain(
                    first,
                    provisional[first_index],
                    expected_theme,
                    children,
                    first_index,
                    round,
                )?;
                for column in 1..self.columns {
                    let index = first_index + column;
                    let plot = plots[index].as_ref().ok_or_else(|| {
                        Error::Data(format!(
                            "canvas row {row} requires panel ({row}, {column}) for shared category labels"
                        ))
                    })?;
                    let theme = themes
                        .plot(index)
                        .expect("configured canvas panel has a resolved theme");
                    #[cfg(test)]
                    record_canvas_theme(index, "shared-category-label-validation", theme);
                    if plot_category_domain(
                        plot,
                        provisional[index],
                        theme,
                        children,
                        index,
                        round,
                    )? != expected
                    {
                        return Err(Error::Data(format!(
                            "canvas row {row} category domains differ across panels"
                        )));
                    }
                    plots[index] = plots[index]
                        .take()
                        .map(|plot| plot.with_y_tick_labels_visible(false));
                }
            }
        }
        let grid_address = crate::layout::ComponentAddress::root().child(
            crate::layout::AddressRole::CanvasGrid,
            0,
            &themes
                .root()
                .prepared_font_environment()?
                .generation_resources(),
        )?;
        round.begin_owner(&grid_address)?;
        round.observe_box(outer)?;
        let mut requirements = Vec::new();
        fit::reserve(
            themes.root(),
            &mut requirements,
            self.panels.len(),
            "Canvas gutter requirements",
        )?;
        for index in 0..self.panels.len() {
            let address = grid_address.child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("canvas grid address overflowed".into()))?,
                &themes
                    .root()
                    .prepared_font_environment()?
                    .generation_resources(),
            )?;
            let [left, right, bottom, top] = round.insets(&address, FeedbackKind::AxisGutter)?;
            let owned = round.insets(&address, FeedbackKind::OwnedInk)?;
            let top = top + owned[3];
            requirements.push((
                index / self.columns,
                index % self.columns,
                PanelRequirements {
                    left,
                    right,
                    bottom,
                    top,
                },
            ));
        }
        for (group_index, group) in self.x_range_groups.iter().enumerate() {
            let address = grid_address.child(
                crate::layout::AddressRole::CanvasRangeGroup,
                u32::try_from(group_index)
                    .map_err(|_| Error::Render("canvas group address overflowed".into()))?,
                &themes
                    .root()
                    .prepared_font_environment()?
                    .generation_resources(),
            )?;
            let alignment = round.insets(&address, FeedbackKind::SharedAlignment)?;
            for &index in group {
                requirements[index].2.left = requirements[index].2.left.max(alignment[0]);
                requirements[index].2.right = requirements[index].2.right.max(alignment[1]);
            }
        }
        if !prepared.content_sizing {
            Self::complete_cell_gutters(
                &plots,
                &provisional,
                &themes,
                children,
                round,
                &mut requirements,
            )?;
        }
        let grid = match tracks::allocate(
            outer,
            (self.rows, self.columns),
            &plots,
            prepared.text_panel_intrinsics,
            prepared.plot_intrinsics,
            &self.column_headers,
            &self.row_headers,
            &self.row_titles,
            &requirements,
            &themes,
            &scope.child(
                crate::layout::AddressRole::CanvasGrid,
                0,
                &themes
                    .root()
                    .prepared_font_environment()?
                    .generation_resources(),
            )?,
            prepared
                .content_sizing
                .then_some(tracks::ContentSizing {
                    source: self,
                    interiors: prepared.interiors,
                    round,
                    measurements: prepared.measurements,
                    natural_width: false,
                    natural_height: false,
                })
                .as_ref(),
        )? {
            FitOutcome::Measured(grid) => grid,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let cells = grid.cells;
        let content_insets = grid.content_insets;
        let headers = grid.headers;
        let row_titles = grid.row_titles;
        let mut measured = Vec::new();
        let mut requirements = Vec::new();
        fit::reserve(
            themes.root(),
            &mut measured,
            plots.len(),
            "Canvas measured children",
        )?;
        fit::reserve(
            themes.root(),
            &mut requirements,
            plots.len(),
            "Canvas measured requirements",
        )?;
        let mut child_miss = None;
        for (index, plot) in plots.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let row = index / self.columns;
            let column = index % self.columns;
            let kind = plot.kind();
            let theme = themes
                .plot(index)
                .expect("configured canvas panel has a resolved theme");
            #[cfg(test)]
            record_canvas_theme(index, "panel-measure", theme);
            let child_scope = scope.child(
                crate::layout::AddressRole::CanvasCell,
                index,
                &themes
                    .root()
                    .prepared_font_environment()?
                    .generation_resources(),
            )?;
            let local = match plot_text::PlotLocalTextPlan::measure(
                plot,
                cells[index],
                theme,
                [self.panel_title_alignment, self.panel_subtitle_alignment],
                prepared.plot_intrinsics[index].expect("configured Canvas Plot text intrinsics"),
                &child_scope,
            )? {
                FitOutcome::Measured(local) => local,
                FitOutcome::Miss(reason) => {
                    FitFailure::retain_least(&mut child_miss, reason);
                    continue;
                }
            };
            if shared_legend.is_some() {
                let base = prepared.body_probes[index]
                    .as_ref()
                    .expect("shared guide body probe");
                if let FitOutcome::Miss(reason) = crate::decorations::plan::check_assigned_body(
                    plot,
                    base,
                    full_outer,
                    local.body,
                    theme,
                    &scope.child(
                        crate::layout::AddressRole::CanvasCell,
                        index,
                        &themes
                            .root()
                            .prepared_font_environment()?
                            .generation_resources(),
                    )?,
                )? {
                    FitFailure::retain_least(&mut child_miss, reason);
                }
            }
            let owner = crate::layout::ComponentAddress::root().child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("canvas address overflowed".into()))?,
                &themes
                    .root()
                    .prepared_font_environment()?
                    .generation_resources(),
            )?;
            let source_rows = plot.configured_source_rows()?;
            let mut context = source_rows
                .map_or_else(
                    || crate::spec::PlotRenderContext::new(plot.data(), local.body, theme),
                    |rows| {
                        crate::spec::PlotRenderContext::with_source_rows(
                            plot.data(),
                            rows,
                            local.body,
                            theme,
                        )
                    },
                )
                .with_fit_scope(&child_scope);
            if prepared.content_sizing
                && self.legend_policy == CanvasLegendPolicy::PerPlot
                && !matches!(kind, PlotKind::Heatmap | PlotKind::Beeswarm)
            {
                context.content_height = true;
                context.fill_content_height = true;
            }
            let role = if self.legend_policy == CanvasLegendPolicy::PerPlot {
                crate::decorations::plan::LegendRole::Panel
            } else {
                crate::decorations::plan::LegendRole::Suppressed
            };
            let panel_plan = match children.plan(plot, role, context, None, round, &owner)? {
                FitOutcome::Measured(result) => result,
                FitOutcome::Miss(failure) => {
                    FitFailure::retain_least(&mut child_miss, failure);
                    continue;
                }
            };
            if context.fill_content_height {
                let target = grid_address.child(
                    crate::layout::AddressRole::CanvasCell,
                    u32::try_from(index).map_err(|_| {
                        Error::Render("Canvas content height address overflowed".into())
                    })?,
                    &themes
                        .root()
                        .prepared_font_environment()?
                        .generation_resources(),
                )?;
                let preferred = sizing::measured_preference(
                    plot,
                    &panel_plan,
                    cells[index].height() - local.body.height(),
                    round,
                    &owner,
                    theme,
                )?;
                if let Some(measurements) = prepared.measurements {
                    measurements.record(
                        plot,
                        cells[index].width(),
                        preferred,
                        content_insets[index],
                        round,
                        &target,
                        theme,
                    )?;
                }
                let request = round.request_insets(
                    &owner,
                    &target,
                    FeedbackKind::ContentHeight,
                    [cells[index].width(), 0.0, preferred, 0.0],
                )?;
                if request.is_some() || preferred > cells[index].height() + 1.0 / 1024.0 {
                    round.miss(
                        &target,
                        crate::layout::kernel::FitKind::TrackOrResidual,
                        &[request],
                        "selected Plot guide and prose require their measured content row",
                    )?;
                }
            }
            #[cfg(test)]
            CANVAS_ASSIGNED_PANEL_EVIDENCE.with(|evidence| {
                evidence
                    .borrow_mut()
                    .push((local.body, panel_plan.result.layout.projection.panel_box()));
            });
            let requirement = match fit::outcome(PanelRequirements::measure_header(
                panel_plan.result.axes.as_ref().ok_or_else(|| {
                    Error::Render("composed panel is missing its measured axes".into())
                })?,
                theme,
                None,
                None,
                local.body,
                &scope.child(
                    crate::layout::AddressRole::CanvasCell,
                    index,
                    &themes
                        .root()
                        .prepared_font_environment()?
                        .generation_resources(),
                )?,
            ))? {
                FitOutcome::Measured(requirement) => requirement,
                FitOutcome::Miss(failure) => {
                    FitFailure::retain_least(&mut child_miss, failure);
                    continue;
                }
            };
            measured.push((index, kind, panel_plan, local));
            requirements.push((row, column, requirement));
        }
        if let Some(failure) = child_miss {
            return Ok(FitOutcome::Miss(failure));
        }
        for (group_index, group) in self.x_range_groups.iter().enumerate() {
            let address = grid_address.child(
                crate::layout::AddressRole::CanvasRangeGroup,
                u32::try_from(group_index)
                    .map_err(|_| Error::Render("canvas group address overflowed".into()))?,
                &themes
                    .root()
                    .prepared_font_environment()?
                    .generation_resources(),
            )?;
            let mut alignment = [0.0_f64; 4];
            for &index in group {
                if let Some((_, _, requirement)) = requirements
                    .iter()
                    .find(|(row, column, _)| row * self.columns + column == index)
                {
                    alignment[0] = alignment[0].max(requirement.left);
                    alignment[1] = alignment[1].max(requirement.right);
                }
            }
            if let Some(request) = round.request_insets(
                &address,
                &address,
                FeedbackKind::SharedAlignment,
                alignment,
            )? {
                round.miss(
                    &address,
                    FitKind::TrackOrResidual,
                    &[Some(request)],
                    "shared range group requires aligned panel edges",
                )?;
            }
        }
        for (row, column, requirement) in requirements {
            let index = row * self.columns + column;
            let address = grid_address.child(
                crate::layout::AddressRole::CanvasCell,
                u32::try_from(index)
                    .map_err(|_| Error::Render("canvas grid address overflowed".into()))?,
                &themes
                    .root()
                    .prepared_font_environment()?
                    .generation_resources(),
            )?;
            round.observe_box(cells[index])?;
            if let Some(request) = round.request_insets(
                &address,
                &address,
                crate::layout::kernel::FeedbackKind::AxisGutter,
                [
                    requirement.left,
                    requirement.right,
                    requirement.bottom,
                    requirement.top,
                ],
            )? {
                round.miss(
                    &address,
                    crate::layout::kernel::FitKind::Tick,
                    &[Some(request)],
                    "canvas cell requires measured decoration gutters",
                )?;
            }
        }
        #[cfg(test)]
        {
            if !headers.is_empty() {
                record_canvas_theme(usize::MAX, "shared-header-plan", themes.root());
            }
            if !row_titles.is_empty() {
                record_canvas_theme(usize::MAX, "row-title-plan", themes.root());
            }
        }

        let mut text_panels = Vec::new();
        crate::render::backend::reserve_text_records(
            &mut text_panels,
            self.text_panels.iter().flatten().count(),
            themes.root().prepared_font_environment()?,
        )?;
        for (index, text_panel) in self.text_panels.iter().enumerate() {
            let Some(text_panel) = text_panel else {
                continue;
            };
            #[cfg(test)]
            record_canvas_theme(usize::MAX, "text-plan", themes.root());
            let intrinsic = prepared.text_panel_intrinsics[index]
                .expect("configured TextPanel retains prepared intrinsics");
            match TextPanelPlan::measure(
                text_panel,
                index,
                cells[index],
                themes.root(),
                intrinsic,
                &scope,
            )? {
                FitOutcome::Measured(plan) => text_panels.push(plan),
                FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
            }
        }

        for text_panel in &text_panels {
            round.observe_box(text_panel.cell)?;
            round.observe_text_block(&text_panel.title)?;
            round.observe_text_block(&text_panel.body)?;
        }
        for header in headers.iter().chain(&row_titles) {
            round.observe_text(&header.text.plan)?;
            round.observe_number(header.text.placement.position.x)?;
            round.observe_number(header.text.placement.position.y)?;
        }
        round.observe_count(usize::from(shared_legend.is_some()))?;
        if let Some(legend) = &shared_legend {
            legend.observe_feedback(round)?;
        }

        let mut panels = Vec::new();
        fit::reserve(
            themes.root(),
            &mut panels,
            measured.len(),
            "Canvas selected panels",
        )?;
        for (index, kind, panel_plan, local) in measured {
            let plot = plots[index]
                .as_ref()
                .expect("measured canvas panel remains configured");
            let cell = cells[index];
            #[cfg(test)]
            record_canvas_theme(
                index,
                "panel-plan",
                themes
                    .plot(index)
                    .expect("configured canvas panel has a resolved theme"),
            );
            let mut layers = [String::new(), String::new(), String::new(), String::new()];
            for (layer, suffix) in layers.iter_mut().zip(["grid", "axes", "legend", "title"]) {
                plot.data()
                    .with_axis_resources(&resources)
                    .reserve_source_string(layer, 40)?;
                write!(layer, "panel-{index}-{suffix}").expect("admitted Canvas layer name");
            }
            let [grid, axes, legend, header_layer] = layers;
            panels.push(PlannedCanvasPanel {
                index,
                kind,
                canvas_cell: cell,
                legend: panel_plan
                    .legend
                    .as_ref()
                    .map(|legend| {
                        let (x, y) = legend.bounds.anchor();
                        legend
                            .legend
                            .select(x, y, plot.data().with_axis_resources(&resources))
                    })
                    .transpose()?,
                panel_plan,
                local_text: local,
                header_layer,
                layers: [grid, axes, legend],
            });
        }
        let plan = CanvasPlan {
            dimensions: (
                assigned.dimensions()?.0,
                crate::layout::interior::sizing::integral(
                    canvas_layout.panel_box.y1 + canvas_layout.insets.top,
                )?,
            ),
            themes,
            canvas_layout,
            shared_legend_draw: shared_legend
                .as_ref()
                .map(|guide| {
                    guide.select(
                        plots
                            .iter()
                            .flatten()
                            .next()
                            .expect("shared guide has a source")
                            .data()
                            .with_axis_resources(&resources),
                    )
                })
                .transpose()?,
            plots,
            #[cfg(test)]
            shared_legend: shared_legend.and_then(|guide| match guide {
                SharedMeasuredGuide::Discrete(legend) => Some(legend),
                SharedMeasuredGuide::Continuous { .. } => None,
            }),
            panels,
            text_panels,
            headers,
            row_titles,
        };
        Self::observe_arranged(&plan, round)?;
        Ok(FitOutcome::Measured(plan))
    }

    fn observe_arranged(
        plan: &CanvasPlan,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new(plan.themes.root())?;
        for panel in &plan.text_panels {
            geometry.text(panel.title_placement, panel.title.plan(), None)?;
            geometry.text(panel.body_placement, panel.body.plan(), None)?;
        }
        for header in plan.row_titles.iter().chain(&plan.headers) {
            geometry.text(header.text.placement, &header.text.plan, None)?;
        }
        if let Some(legend) = &plan.shared_legend_draw {
            legend.geometry(&mut geometry)?;
        }
        geometry.observe_feedback(round)?;
        for panel in &plan.panels {
            round.observe_box(panel.canvas_cell)?;
            round.observe_box(panel.local_text.body)?;
            panel.local_text.text.observe(round)?;
            let theme = plan.themes.plot(panel.index).expect("arranged panel theme");
            let mut geometry = crate::render::geometry::GeometrySnapshot::new(theme)?;
            panel.local_text.text.geometry(&mut geometry)?;
            if let Some(legend) = &panel.legend {
                legend.geometry(&mut geometry)?;
            }
            geometry.observe_feedback(round)?;
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the immutable Canvas plan keeps theme ownership and final draw ordering explicit"
    )]
    fn render_planned_canvas(
        plan: &CanvasPlan,
        width: u32,
        height: u32,
        backend: &mut dyn Backend,
    ) -> Result<(), Error> {
        let theme = plan.themes.root();
        #[cfg(test)]
        record_canvas_theme(usize::MAX, "render", theme);
        backend.draw_surface(
            PanelBox::new(0.0, f64::from(width), 0.0, f64::from(height)),
            theme.surfaces.canvas_root,
            SurfaceRole::CanvasRoot,
        )?;
        for panel in &plan.panels {
            let theme = plan
                .themes
                .plot(panel.index)
                .expect("configured canvas panel has a resolved theme");
            backend.draw_surface(
                panel.panel_plan.panel_box,
                theme.surfaces.plot_panel,
                SurfaceRole::PlotPanel,
            )?;
        }
        for panel in &plan.panels {
            #[cfg(test)]
            let theme = plan
                .themes
                .plot(panel.index)
                .expect("configured canvas panel has a resolved theme");
            #[cfg(test)]
            record_canvas_theme(panel.index, "grid-render", theme);
            crate::render::backend::with_layer_role(
                backend,
                &panel.layers[0],
                crate::render::backend::LayerRole::Grid,
                |backend| {
                    panel
                        .panel_plan
                        .result
                        .axes
                        .as_ref()
                        .ok_or_else(|| {
                            Error::Render("Canvas plan is missing immutable axes".into())
                        })?
                        .draw_grid(backend)
                },
            )?;
        }
        for panel in &plan.panels {
            let plot = plan.plots[panel.index].as_ref().ok_or_else(|| {
                Error::Render("composed canvas panel disappeared during render".into())
            })?;
            let theme = plan
                .themes
                .plot(panel.index)
                .expect("configured canvas panel has a resolved theme");
            #[cfg(test)]
            record_canvas_theme(panel.index, "panel-render", theme);
            let source_rows = plot.configured_source_rows()?;
            let context = source_rows
                .map_or_else(
                    || {
                        crate::spec::PlotRenderContext::new(
                            plot.data(),
                            panel.panel_plan.panel_box,
                            theme,
                        )
                    },
                    |rows| {
                        crate::spec::PlotRenderContext::with_source_rows(
                            plot.data(),
                            rows,
                            panel.panel_plan.panel_box,
                            theme,
                        )
                    },
                )
                .with_retained_body(&panel.panel_plan.result);
            plot.render_panel(panel.kind, context, backend)?;
            crate::significance::render_selected(plot, &panel.panel_plan.result, theme, backend)?;
            with_layer(backend, &panel.layers[1], |backend| {
                panel
                    .panel_plan
                    .result
                    .axes
                    .as_ref()
                    .ok_or_else(|| Error::Render("Canvas plan is missing immutable axes".into()))?
                    .draw(backend)
            })?;
            if let Some(legend) = &panel.legend {
                with_layer(backend, &panel.layers[2], |backend| legend.draw(backend))?;
            }
            with_layer(backend, &panel.header_layer, |backend| {
                panel.local_text.text.draw_retained(backend)
            })?;
        }
        for panel in &plan.text_panels {
            #[cfg(test)]
            record_canvas_theme(usize::MAX, "text-render", theme);
            draw_text_panel(panel, backend)?;
        }
        if !plan.row_titles.is_empty() {
            #[cfg(test)]
            record_canvas_theme(usize::MAX, "row-title-render", theme);
            draw_canvas_row_titles(&plan.row_titles, backend)?;
        }
        if !plan.headers.is_empty() {
            #[cfg(test)]
            record_canvas_theme(usize::MAX, "shared-header-render", theme);
            draw_canvas_headers(&plan.headers, backend)?;
        }
        if let Some(legend) = &plan.shared_legend_draw {
            with_layer(backend, "canvas-shared-legend", |backend| {
                legend.draw(backend)
            })?;
        }
        plan.canvas_layout.root_text.draw_retained(backend)
    }

    fn canvas_hue_groups(
        plots: &[Option<Plot>],
        panel: PanelBox,
        themes: &ResolvedCanvasThemes,
        children: &mut crate::decorations::plan::children::ChildPlans<'_>,
        round: &crate::layout::kernel::FeedbackRound,
    ) -> Result<Vec<(String, Vec<String>)>, Error> {
        let resources = themes
            .root()
            .prepared_font_environment()?
            .generation_resources();
        let mut groups: Vec<(String, Vec<String>)> = Vec::new();
        crate::layout::fit::reserve(themes.root(), &mut groups, plots.len(), "Canvas hue groups")?;
        for (index, plot) in plots.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let data = plot.data().with_axis_resources(&resources);
            let Some(dimension) = plot.hue_dimension_key() else {
                continue;
            };
            let theme = themes
                .plot(index)
                .expect("configured canvas panel has a resolved theme");
            #[cfg(test)]
            record_canvas_theme(index, "shared-hue-discovery", theme);
            let prepared_scatter = plot
                .prepared_scatter
                .as_ref()
                .and_then(|inputs| {
                    inputs
                        .panels
                        .iter()
                        .find(|(scope, _)| scope.as_deref() == plot.axis_scope())
                })
                .map(|(_, prepared)| prepared);
            let observed = if prepared_scatter.is_none() {
                Some(children.canvas_probe(
                    plot,
                    panel,
                    theme,
                    index,
                    MeasurementPhase::Hue,
                    round,
                )?)
            } else {
                None
            };
            data.claim_source_visits(
                groups
                    .len()
                    .checked_mul(dimension.len().max(1))
                    .ok_or_else(|| Error::Render("Canvas hue lookup overflow".into()))?,
            )?;
            let levels =
                if let Some((_, levels)) = groups.iter_mut().find(|(name, _)| name == dimension) {
                    levels
                } else {
                    let index = groups.len();
                    groups.push((data.copy_source_string(dimension)?, Vec::new()));
                    &mut groups[index].1
                };
            let observed_levels = prepared_scatter
                .into_iter()
                .flat_map(crate::plots::scatter::PreparedScatter::hue_levels)
                .chain(observed.iter().flat_map(|result| {
                    result
                        .legend_entries
                        .iter()
                        .filter_map(|entry| entry.coordinated_hue_value_for(dimension))
                }));
            for level in observed_levels {
                data.claim_source_visits(
                    levels
                        .len()
                        .checked_mul(level.len().max(1))
                        .ok_or_else(|| Error::Render("Canvas hue level lookup overflow".into()))?,
                )?;
                if !levels.iter().any(|value| value == level) {
                    data.reserve_source(levels, 1)?;
                    levels.push(data.copy_source_string(level)?);
                }
            }
        }
        Ok(groups)
    }

    #[expect(
        clippy::type_complexity,
        reason = "shared descriptor preparation also retains the exact full-body probes used by frame attempts"
    )]
    fn preflight_shared_legend(
        plots: &[Option<Plot>],
        themes: &ResolvedCanvasThemes,
        panel: PanelBox,
        children: &mut crate::decorations::plan::children::ChildPlans<'_>,
        round: &crate::layout::kernel::FeedbackRound,
        prepared_discrete_guide: Option<&PreparedDiscreteGuide>,
    ) -> Result<
        (
            Option<PreparedDiscreteGuide>,
            Vec<Option<Arc<crate::spec::PlotRenderResult>>>,
        ),
        Error,
    > {
        let mut entries = Vec::new();
        let mut probes = Vec::new();
        let resources = themes
            .root()
            .prepared_font_environment()?
            .generation_resources();
        crate::layout::fit::reserve(
            themes.root(),
            &mut probes,
            plots.len(),
            "shared guide body probes",
        )?;
        probes.resize_with(plots.len(), || None);
        for (index, plot) in plots.iter().enumerate() {
            let Some(plot) = plot else { continue };
            let selection = themes
                .plot_selection(index)
                .expect("configured canvas panel has a resolved theme");
            let theme = match selection {
                ResolvedPlotTheme::InheritedRoot => themes.root(),
                ResolvedPlotTheme::ExplicitPlot(theme) => theme.as_ref(),
            };
            #[cfg(test)]
            {
                if plot.legend_policy_spec() != crate::LegendPolicy::Hidden {
                    record_canvas_theme(index, "shared-legend-preflight", theme);
                    record_shared_legend_theme(index, selection);
                }
            }
            let result = children.canvas_probe(
                plot,
                panel,
                theme,
                index,
                MeasurementPhase::Legend,
                round,
            )?;
            if prepared_discrete_guide.is_none()
                && plot.legend_policy_spec() != crate::LegendPolicy::Hidden
            {
                let data = plot.data().with_axis_resources(&resources);
                data.reserve_source(&mut entries, result.legend_entries.len())?;
                for entry in &result.legend_entries {
                    entries.push(entry.admitted_clone(data)?);
                }
            }
            probes[index] = Some(result);
        }
        if let Some((title, entries)) = prepared_discrete_guide {
            if entries.is_empty() {
                return Ok((None, probes));
            }
            let source = plots
                .iter()
                .flatten()
                .next()
                .expect("prepared guide owns a plot");
            let title = source
                .data()
                .with_axis_resources(&resources)
                .copy_source_string(title)?;
            return Ok((Some((title, Arc::clone(entries))), probes));
        }
        let entries = crate::decorations::legend::merge_exact_entries(
            entries,
            "shared canvas legend",
            &resources,
        )?;
        if entries.is_empty() {
            Ok((None, probes))
        } else {
            Ok((
                Some((
                    crate::decorations::legend::resolved_title(&entries, &resources)?,
                    entries.into(),
                )),
                probes,
            ))
        }
    }
}

fn plot_category_domain(
    plot: &Plot,
    panel: PanelBox,
    theme: &crate::theme::ResolvedTheme,
    children: &mut crate::decorations::plan::children::ChildPlans<'_>,
    index: usize,
    round: &crate::layout::kernel::FeedbackRound,
) -> Result<Vec<String>, Error> {
    let kind = plot.kind();
    if !matches!(
        kind,
        PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar
    ) {
        return Err(Error::Data(
            "shared category labels are supported only by horizontal bar families".into(),
        ));
    }
    let result =
        children.canvas_probe(plot, panel, theme, index, MeasurementPhase::Category, round)?;
    if result.y_domain != AxisDomain::Categorical {
        return Err(Error::Data(
            "shared category labels require a categorical y-axis".into(),
        ));
    }
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = plot.data().with_axis_resources(&resources);
    match &result.y_tick_keys {
        Some(keys) => data.collect_source(keys.iter().map(|key| data.copy_source_string(key))),
        None => data.collect_source(
            result
                .layout
                .y_axis
                .ticks
                .iter()
                .map(|tick| data.copy_source_string(&tick.label)),
        ),
    }
}

fn draw_canvas_headers(
    headers: &[CanvasHeaderPlan],
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    if headers.is_empty() {
        return Ok(());
    }
    with_layer(backend, "canvas-shared-headers", |backend| {
        for header in headers {
            header.text.draw(backend)?;
        }
        Ok(())
    })
}

fn draw_canvas_row_titles(
    titles: &[CanvasHeaderPlan],
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    if titles.is_empty() {
        return Ok(());
    }
    with_layer(backend, "canvas-row-titles", |backend| {
        for title in titles {
            title.text.draw(backend)?;
        }
        Ok(())
    })
}

fn draw_text_panel(panel: &TextPanelPlan, backend: &mut dyn Backend) -> Result<(), Error> {
    with_layer(backend, &panel.layer, |backend| {
        backend.draw_planned_text(panel.title_placement, panel.title.plan())?;
        backend.draw_planned_text(panel.body_placement, panel.body.plan())
    })
}
