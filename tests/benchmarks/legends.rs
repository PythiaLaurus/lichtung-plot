use super::{measure_case, measure_pair, CaseMeasurement};
use crate::orchestrator::measurements::{Counter, GuideOutcome, GuidePhase};
use crate::{CategoryValues, LegendPolicy, LegendPosition, NumericValues, Plot};

fn scatter(entries: u32, long: bool) -> Plot<crate::kind::Scatter> {
    let count = entries * 2;
    Plot::scatter(
        NumericValues::new("x", (0..count).map(f64::from)),
        NumericValues::new("y", (0..count).map(|i| f64::from((i * 7) % 13))),
    )
    .hue(
        CategoryValues::new(
            "Condition",
            (0..count).map(|i| {
                if long {
                    format!("Complete explanatory condition {}", i / 2)
                } else {
                    format!("Condition {}", i / 2)
                }
            }),
        )
        .unwrap(),
    )
}

fn outside(fixture: &str, edge: LegendPosition) -> Plot<crate::kind::Scatter> {
    let plot = match fixture {
        "small" => scatter(3, false).size(800, 480),
        "starvation" => scatter(2, true)
            .x_label("MMMMMMMMMMMMMMMMMMMMMMMMMMMM")
            .size(640, 480),
        "historical_reflow_640x400" => scatter(18, true).size(640, 400),
        "preferred_right_control" => scatter(2, true)
            .x_label("MMMMMMMMMMMMMMMMMMMM")
            // Leave the measured hard body and complete key/frame ink enough
            // room for normal Right, independently of compact-Right recovery.
            .size(670, 480),
        "native_compact_right" => Plot::scatter(
            NumericValues::new("x", (0..16).map(f64::from)),
            NumericValues::new("y", (0..16).map(|i| f64::from((i * 7) % 13))),
        )
        .hue(
            CategoryValues::new(
                "Condition",
                (0..16).map(|i| {
                    if i / 2 == 0 {
                        "Complete explanatory condition 0".to_owned()
                    } else {
                        format!("Condition {}", i / 2)
                    }
                }),
            )
            .unwrap(),
        )
        // This real unbroken axis title supplies the body's logical text
        // allocation while the guide still requires compact two-column flow.
        .x_label("Coordinate")
        .size(520, 240),
        _ => panic!("unknown fixture"),
    };
    plot.legend(LegendPolicy::Visible(edge))
}

fn assert_observed(sample: &CaseMeasurement, entries: usize) {
    let observed = &sample.observations;
    assert_eq!(
        observed.counters[Counter::GuideTrial as usize],
        u64::try_from(observed.guides.len()).unwrap()
    );
    for (ordinal, root) in observed.roots.iter().enumerate() {
        assert_eq!(root.ordinal, ordinal);
        assert_eq!(
            root.successful,
            ordinal + 1 == observed.roots.len() && sample.error.is_none()
        );
        for attempt in &observed.guides[root.guides.clone()] {
            assert_eq!(attempt.root_round, root.ordinal);
        }
    }
    assert_eq!(
        observed.roots.iter().filter(|root| root.successful).count(),
        usize::from(sample.error.is_none())
    );
    assert_eq!(observed.calls[1], observed.roots.len());
    if let Some(output) = &sample.output {
        let svg = std::str::from_utf8(output.preserved_svg.as_ref().unwrap()).unwrap();
        let document = roxmltree::Document::parse(svg).unwrap();
        let legends = document
            .descendants()
            .filter(|node| {
                node.attribute("id")
                    .is_some_and(|id| id.ends_with("legend"))
            })
            .collect::<Vec<_>>();
        assert_eq!(legends.len(), 1, "exactly one populated guide");
        assert_eq!(
            legends[0]
                .descendants()
                .filter(|node| node.has_tag_name("circle"))
                .count(),
            entries
        );
        assert!(legends[0]
            .descendants()
            .any(|node| node.text() == Some("Condition")));
    }
}

fn selected(sample: &CaseMeasurement) -> LegendPosition {
    let root = sample.observations.roots.last().unwrap();
    assert!(root.successful);
    sample.observations.guides[root.guides.clone()]
        .iter()
        .rev()
        .find(|trial| trial.outcome == GuideOutcome::Accepted)
        .unwrap()
        .edge
}

fn equal_output(auto: &CaseMeasurement, explicit: &CaseMeasurement) {
    let auto = auto.output.as_ref().unwrap();
    let explicit = explicit.output.as_ref().unwrap();
    assert_eq!(auto.svg, explicit.svg, "outlined SVG geometry differs");
    assert_eq!(
        auto.preserved_svg, explicit.preserved_svg,
        "retained text/geometry differs"
    );
    assert_eq!(auto.png, explicit.png, "encoded PNG differs");
    let decode = |bytes: &[u8]| {
        let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
            .read_info()
            .unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();
        pixels.truncate(info.buffer_size());
        (
            info.width,
            info.height,
            info.color_type,
            info.bit_depth,
            pixels,
        )
    };
    assert_eq!(decode(&auto.png), decode(&explicit.png));
}

fn outside_case(fixture: &str, edge: LegendPosition, success: bool) -> CaseMeasurement {
    let sample = measure_case(
        &format!("legend_{fixture}_{edge:?}"),
        || outside(fixture, edge),
        |plot, path| plot.svg(path),
        success,
        true,
    );
    assert_observed(
        &sample,
        match fixture {
            "small" => 3,
            "starvation" | "preferred_right_control" => 2,
            "historical_reflow_640x400" => 18,
            "native_compact_right" => 8,
            _ => unreachable!(),
        },
    );
    assert_outside_trace(&sample, fixture, edge);
    if let Some(output) = &sample.output {
        let svg = std::str::from_utf8(output.preserved_svg.as_ref().unwrap()).unwrap();
        let title = match fixture {
            "starvation" => "MMMMMMMMMMMMMMMMMMMMMMMMMMMM",
            "preferred_right_control" => "MMMMMMMMMMMMMMMMMMMM",
            "native_compact_right" => "Coordinate",
            _ => "X",
        };
        let document = roxmltree::Document::parse(svg).unwrap();
        assert!(document
            .descendants()
            .any(|node| node.text() == Some(title)));
        assert_complete_outside_labels(&document, fixture);
        if fixture == "native_compact_right" {
            let guide = document
                .descendants()
                .find(|node| node.attribute("id") == Some("legend"))
                .unwrap();
            assert_eq!(
                document
                    .descendants()
                    .filter(|node| node.has_tag_name("circle")
                        && !node.ancestors().any(|ancestor| ancestor == guide))
                    .count(),
                16
            );
            if matches!(edge, LegendPosition::Auto | LegendPosition::OutsideRight) {
                let columns = guide
                    .descendants()
                    .filter(|node| node.has_tag_name("circle"))
                    .map(|node| node.attribute("cx").unwrap())
                    .collect::<std::collections::HashSet<_>>();
                assert_eq!(
                    columns.len(),
                    2,
                    "compact right retains two populated columns"
                );
            }
        }
    }
    if !success {
        assert!(sample
            .error
            .as_ref()
            .unwrap()
            .to_string()
            .contains("layout fit"));
    }
    sample
}

fn assert_complete_outside_labels(document: &roxmltree::Document<'_>, fixture: &str) {
    let guide = document
        .descendants()
        .find(|node| node.attribute("id") == Some("legend"))
        .unwrap();
    let retained = guide
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .filter_map(|node| node.text())
        .flat_map(str::split_whitespace)
        .collect::<String>();
    let count = match fixture {
        "small" => 3,
        "historical_reflow_640x400" => 18,
        "starvation" | "preferred_right_control" => 2,
        "native_compact_right" => 8,
        _ => unreachable!(),
    };
    for index in 0..count {
        let prefix = if fixture == "small" || (fixture == "native_compact_right" && index > 0) {
            "Condition"
        } else {
            "Completeexplanatorycondition"
        };
        assert!(
            retained.contains(&format!("{prefix}{index}")),
            "complete label {index}: {retained}"
        );
    }
}

fn assert_outside_trace(sample: &CaseMeasurement, fixture: &str, edge: LegendPosition) {
    use GuideOutcome::{Accepted, Provisional, Rejected};
    use GuidePhase::{Body, GuideOnly};
    use LegendPosition::{Auto, OutsideRight, OutsideTop};

    let right_flow = matches!(edge, Auto | OutsideRight);
    let first_edge = if edge == Auto { OutsideRight } else { edge };
    let mut expected = vec![(0, first_edge, false, Body, Provisional)];
    if fixture == "starvation" && right_flow {
        // Complete key/frame ink and the body's actual hard text minimum
        // reject Right before body allocation, even with empty feedback.
        expected = vec![
            (0, OutsideRight, false, GuideOnly, Rejected),
            (0, OutsideRight, true, GuideOnly, Rejected),
        ];
        if edge == Auto {
            expected.extend([
                (0, OutsideTop, false, Body, Provisional),
                (1, OutsideRight, false, GuideOnly, Rejected),
                (1, OutsideRight, true, GuideOnly, Rejected),
                (1, OutsideTop, false, Body, Accepted),
            ]);
        }
    } else if fixture == "historical_reflow_640x400" && right_flow {
        expected.extend([
            (1, OutsideRight, false, Body, Provisional),
            (2, OutsideRight, false, Body, Accepted),
        ]);
    } else if fixture == "native_compact_right" && right_flow {
        expected.extend([
            (1, OutsideRight, false, GuideOnly, Rejected),
            (1, OutsideRight, true, Body, Accepted),
        ]);
    } else if fixture == "native_compact_right" {
        expected.extend([
            (1, first_edge, false, Body, Provisional),
            (2, first_edge, false, Body, Accepted),
        ]);
    } else {
        expected.push((1, first_edge, false, Body, Accepted));
    }
    // Assigned-width flow no longer shapes an unrelated "0" ch probe; exact
    // work changes while the chosen frames and failed terminals remain pinned.
    // Actual-domain terminal intrinsics can add a distinct shaping query; keep
    // that work explicit while retaining the complete edge/flow/round trace.
    let (rounds, trials, checks, proposals, exact) = match (fixture, edge) {
        ("small", _) => (2, 2, 2, 0, 26),
        ("starvation", Auto) => (2, 6, 2, 10, 31),
        ("starvation", OutsideRight) => (1, 2, 0, 10, 31),
        ("starvation", _) if !right_flow => (2, 2, 2, 0, 25),
        ("preferred_right_control", _) => (2, 2, 2, 0, 31),
        ("historical_reflow_640x400", Auto | OutsideRight) => (3, 3, 3, 0, 111),
        ("historical_reflow_640x400", _) => (2, 2, 2, 0, 108),
        ("native_compact_right", Auto | OutsideRight) => (2, 3, 2, 6, 46),
        ("native_compact_right", LegendPosition::OutsideBottom) => (3, 3, 3, 0, 43),
        _ => unreachable!(),
    };
    let observed = &sample.observations;
    println!(
        "outside_trace,{fixture},{edge:?},roots={:?},guides={:?}",
        observed.roots, observed.guides
    );
    println!("outside_counts,{fixture},{edge:?},rounds={},trials={},checks={},proposals={},exact={},shapes={}", observed.roots.len(), observed.counters[Counter::GuideTrial as usize], observed.counters[Counter::GuideBodyCheck as usize], observed.counters[Counter::TextProposalVisit as usize], observed.counters[Counter::TextExactTrial as usize], sample.shapes);
    assert_eq!(observed.roots.len(), rounds);
    assert_eq!(
        observed
            .guides
            .iter()
            .map(|trial| (
                trial.root_round,
                trial.edge,
                trial.compact,
                trial.phase,
                trial.outcome
            ))
            .collect::<Vec<_>>(),
        expected
    );
    if sample.error.is_none() {
        assert_eq!(selected(sample), expected.last().unwrap().1);
    }
    assert_eq!(
        [
            Counter::GuideTrial,
            Counter::GuideBodyCheck,
            Counter::TextProposalVisit,
            Counter::TextExactTrial
        ]
        .map(|counter| observed.counters[counter as usize]),
        [trials, checks, proposals, exact]
    );
    assert_eq!(u64::try_from(sample.shapes).unwrap(), exact);
    assert_eq!(
        observed.counters[Counter::LegacyPanelConstructor as usize],
        0
    );
}

#[test]
#[ignore = "release measurements; run with --release --ignored --nocapture --test-threads=1"]
fn outside_legend_measurements() {
    assert!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "requires --release"
    );
    println!("legend_conditions,release,three_interleaved_runs,selection_probe_before_pairs,OS_cache_and_scheduling_uncontrolled,SVG_outlined,PNG_source_balanced,preserved_text_checks_outside_timing");
    println!("legend_dimensions,small=800x480,starvation=640x480,historical_reflow_640x400=640x400,preferred_right_control=670x480,native_compact_right=520x240");
    for fixture in [
        "small",
        "starvation",
        "historical_reflow_640x400",
        "preferred_right_control",
        "native_compact_right",
    ] {
        let auto = || outside_case(fixture, LegendPosition::Auto, true);
        let probe = auto();
        let edge = selected(&probe);
        let explicit = || outside_case(fixture, edge, true);
        let selected_name = format!("{fixture}_selected_{edge:?}");
        for [automatic, fixed] in measure_pair([fixture, &selected_name], [&auto, &explicit]) {
            assert_eq!(selected(&automatic), edge);
            equal_output(&automatic, &fixed);
        }
        let right = || {
            outside_case(
                fixture,
                LegendPosition::OutsideRight,
                fixture != "starvation",
            )
        };
        let bottom = || outside_case(fixture, LegendPosition::OutsideBottom, true);
        measure_pair(
            [&format!("{fixture}_right"), &format!("{fixture}_bottom")],
            [&right, &bottom],
        );
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "keep shared fixtures with their exact owner, edge/flow, and work contracts"
)]
fn shared_case(facet: bool, pressure: bool, edge: LegendPosition) -> CaseMeasurement {
    use GuideOutcome::{Accepted, Provisional, Rejected};
    use GuidePhase::{Body, GuideOnly};
    use LegendPosition::{OutsideRight, OutsideTop};

    let (width, height) = if pressure {
        (if facet { 560 } else { 480 }, 360)
    } else {
        (1200, 720)
    };
    let entries = if pressure { 12 } else { 3 };
    let sample = if facet {
        measure_case(
            "legend_facets",
            || {
                scatter(entries, pressure)
                    .facet(
                        crate::FacetSpec::wrap(
                            CategoryValues::new(
                                "Panel",
                                (0..entries * 2).map(|i| if i % 2 == 0 { "A" } else { "B" }),
                            )
                            .unwrap(),
                            2,
                        )
                        .unwrap(),
                    )
                    .legend(LegendPolicy::Visible(edge))
                    .size(width, height)
            },
            |plot, path| plot.svg(path),
            true,
            true,
        )
    } else {
        measure_case(
            "legend_canvas",
            || {
                crate::Canvas::grid(1, 2)
                    .unwrap()
                    .panel(0, 0, scatter(entries, pressure))
                    .unwrap()
                    .panel(0, 1, scatter(entries, pressure))
                    .unwrap()
                    .legend_policy(crate::CanvasLegendPolicy::Shared)
                    .size(width, height)
            },
            |canvas, path| canvas.svg(path),
            true,
            true,
        )
    };
    assert_observed(&sample, usize::try_from(entries).unwrap());
    assert!(!sample.observations.guides.is_empty());
    let owner = &sample.observations.guides[0].owner;
    assert!(
        sample
            .observations
            .guides
            .iter()
            .all(|trial| &trial.owner == owner),
        "shared guides must not become per-panel guide trials"
    );
    println!(
        "shared_trace,facet={facet},pressure={pressure},edge={edge:?},roots={:?},guides={:?},counters={:?}",
        sample.observations.roots, sample.observations.guides, sample.observations.counters
    );
    assert_child_reuse(&sample, facet, pressure);
    let (rounds, trials, checks, proposals, exact) = if pressure {
        if facet {
            // The actual-domain hard body changes the guide's assigned width:
            // this frame now settles after its first measured-gutter reflow.
            (2, 2, 4, 8, 80)
        } else {
            // Complete gutters reject the normal Right containing frame in
            // the first round; complete compact key ink rejects before body.
            (2, 6, 4, 60, 109)
        }
    } else {
        (
            2,
            2,
            4,
            if facet { 0 } else { 12 },
            if facet { 29 } else { 40 },
        )
    };
    assert_eq!(sample.observations.roots.len(), rounds);
    assert_eq!(
        [
            Counter::GuideTrial,
            Counter::GuideBodyCheck,
            Counter::TextProposalVisit,
            Counter::TextExactTrial
        ]
        .map(|counter| sample.observations.counters[counter as usize]),
        [trials, checks, proposals, exact]
    );
    assert_eq!(u64::try_from(sample.shapes).unwrap(), exact);
    let expected: &[_] = if !pressure || facet {
        &[
            (0, OutsideRight, false, Body, Provisional),
            (1, OutsideRight, false, Body, Accepted),
        ]
    } else {
        &[
            (0, OutsideRight, false, Body, Rejected),
            (0, OutsideRight, true, GuideOnly, Rejected),
            (0, OutsideTop, false, Body, Provisional),
            (1, OutsideRight, false, Body, Rejected),
            (1, OutsideRight, true, GuideOnly, Rejected),
            (1, OutsideTop, false, Body, Accepted),
        ]
    };
    assert_eq!(
        sample
            .observations
            .guides
            .iter()
            .map(|trial| (
                trial.root_round,
                trial.edge,
                trial.compact,
                trial.phase,
                trial.outcome
            ))
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(selected(&sample), expected.last().unwrap().1);
    sample
}

#[test]
#[ignore = "release measurements; run with --release --ignored --nocapture --test-threads=1"]
fn shared_legend_measurements() {
    assert!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "requires --release"
    );
    println!("legend_dimensions,shared_adequate=1200x720,canvas_pressure=480x360,facet_pressure=560x360,two_panels");
    for facet in [false, true] {
        let adequate = || shared_case(facet, false, LegendPosition::Auto);
        let pressure = || shared_case(facet, true, LegendPosition::Auto);
        let root = if facet { "facet" } else { "canvas" };
        let samples = measure_pair(
            [&format!("{root}_adequate"), &format!("{root}_pressure")],
            [&adequate, &pressure],
        );
        for pair in &samples[1..] {
            equal_output(&samples[0][0], &pair[0]);
            equal_output(&samples[0][1], &pair[1]);
        }
        if facet {
            for pressured in [false, true] {
                let auto = || shared_case(true, pressured, LegendPosition::Auto);
                let edge = selected(&auto());
                let fixed = || shared_case(true, pressured, edge);
                for [auto, fixed] in measure_pair(
                    [
                        &format!("facet_auto_pressure_{pressured}"),
                        &format!("facet_selected_pressure_{pressured}"),
                    ],
                    [&auto, &fixed],
                ) {
                    equal_output(&auto, &fixed);
                }
            }
        }
    }
}

fn assert_child_reuse(sample: &CaseMeasurement, facet: bool, pressure: bool) {
    let observed = &sample.observations;
    let events = &observed.children;
    assert_eq!(observed.counters[Counter::ChildCacheHit as usize], 0);
    assert_eq!(observed.counters[Counter::ChildFeedbackReplay as usize], 0);
    assert_eq!(
        observed.counters[Counter::ChildCacheMiss as usize],
        if facet {
            8
        } else if pressure {
            16
        } else {
            12
        }
    );
    assert!(
        events.iter().all(|event| event.phase != "layout"),
        "native parents must not probe a separate intermediate child body"
    );
    for counter in [
        Counter::ChildCacheHit,
        Counter::ChildCacheMiss,
        Counter::ChildFeedbackReplay,
    ] {
        assert_eq!(
            observed.counters[counter as usize],
            u64::try_from(
                events
                    .iter()
                    .filter(|event| event.counter == counter)
                    .count()
            )
            .unwrap()
        );
    }
    for (index, event) in events.iter().enumerate() {
        assert_eq!(
            event.owner.leaf_role(),
            if facet {
                crate::layout::AddressRole::FacetCell
            } else {
                crate::layout::AddressRole::CanvasCell
            }
        );
        assert!(observed
            .roots
            .iter()
            .any(|root| root.ordinal == event.root_round));
        assert!(!event.phase.is_empty());
        if event.counter == Counter::ChildCacheHit {
            assert!(events[..index].iter().any(|prior| {
                prior.owner == event.owner
                    && prior.phase == event.phase
                    && prior.root_round <= event.root_round
                    && prior.counter == Counter::ChildCacheMiss
            }));
        }
        if event.counter == Counter::ChildFeedbackReplay {
            let hit = &events[index - 1];
            assert_eq!(hit.counter, Counter::ChildCacheHit);
            assert_eq!(hit.owner, event.owner);
            assert_eq!(hit.phase, "panel");
            assert_eq!(event.phase, "panel");
            assert_eq!(hit.root_round, event.root_round);
        }
        println!(
            "child_cache,facet={facet},pressure={pressure},owner={:?},phase={},root_round={},event={}",
            event.owner, event.phase, event.root_round, event.counter.name()
        );
    }
    assert_eq!(
        events
            .iter()
            .filter(|event| event.phase == "panel" && event.counter == Counter::ChildCacheHit)
            .count(),
        usize::try_from(observed.counters[Counter::ChildFeedbackReplay as usize]).unwrap()
    );
    println!(
        "child_cache_totals,facet={facet},pressure={pressure},hits={},misses={},replays={}",
        observed.counters[Counter::ChildCacheHit as usize],
        observed.counters[Counter::ChildCacheMiss as usize],
        observed.counters[Counter::ChildFeedbackReplay as usize]
    );
}

// A controlled repeat isolates actual child reuse from the end-to-end fixtures,
// whose changed assignments invalidate children across root rounds.
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one controlled root lifecycle checks probe identity and body feedback replay together"
)]
fn shared_legend_child_cache_replay_contract() {
    use crate::decorations::plan::children::{ChildPlans, MeasurementPhase};
    use crate::layout::{AddressRole, ComponentAddress, MeasuredLayout, PanelBox};
    use crate::orchestrator::measurements;
    use std::sync::Arc;

    for facet in [false, true] {
        let plot = scatter(3, false).size(1200, 720).erase();
        let canvas = crate::Canvas::grid(1, 1).unwrap();
        let mut children = if facet {
            ChildPlans::facets(&plot)
        } else {
            ChildPlans::canvas(&canvas)
        };
        let owner = ComponentAddress::root()
            .child(
                if facet {
                    AddressRole::FacetCell
                } else {
                    AddressRole::CanvasCell
                },
                0,
                &crate::font::canonical::OperationGenerationResources::new(),
            )
            .unwrap();
        let theme = crate::Theme::Publication.resolve();
        measurements::start();
        crate::layout::plan_exact_root(
            1200,
            720,
            |stage| crate::plots::shared_x::prepare_plot(&plot, &stage.generation_resources()),
            |inputs, assigned, round| {
                let plot = inputs.attach(plot.clone())?;
                let panel = PanelBox::new(100.0, 1100.0, 100.0, 620.0);
                let context = crate::spec::PlotRenderContext::new(plot.data(), panel, &theme);
                let first = children.measure(
                    &plot,
                    context,
                    &owner,
                    MeasurementPhase::Legend,
                    round,
                    |call| {
                        plot.preflight_measurement_in_call(
                            plot.kind(),
                            panel,
                            &theme,
                            &crate::layout::fit::ROOT_SCOPE,
                            call,
                        )
                    },
                )?;
                let scatter_stages = crate::plots::scatter::STAGES.with(std::cell::Cell::get);
                let second = children.measure(
                    &plot,
                    context,
                    &owner,
                    MeasurementPhase::Legend,
                    round,
                    |_| panic!("identical shared legend probe must hit the actual cache"),
                )?;
                let (
                    crate::layout::fit::FitOutcome::Measured(first),
                    crate::layout::fit::FitOutcome::Measured(second),
                ) = (first, second)
                else {
                    panic!("populated legend probe must fit")
                };
                assert!(Arc::ptr_eq(&first, &second));
                assert_eq!(
                    crate::plots::scatter::STAGES.with(std::cell::Cell::get),
                    scatter_stages
                );
                assert!(Arc::ptr_eq(
                    first.marks.as_ref().unwrap(),
                    second.marks.as_ref().unwrap()
                ));
                assert!(Arc::ptr_eq(
                    first.annotations.as_ref().unwrap(),
                    second.annotations.as_ref().unwrap()
                ));
                let first = children.plan(
                    &plot,
                    crate::decorations::plan::LegendRole::Suppressed,
                    context,
                    None,
                    round,
                    &owner,
                )?;
                if round.child_stable(std::slice::from_ref(&owner))? {
                    let second = children.plan(
                        &plot,
                        crate::decorations::plan::LegendRole::Suppressed,
                        context,
                        None,
                        round,
                        &owner,
                    )?;
                    let (
                        crate::layout::fit::FitOutcome::Measured(first),
                        crate::layout::fit::FitOutcome::Measured(second),
                    ) = (first, second)
                    else {
                        panic!("shared child body must fit")
                    };
                    assert!(Arc::ptr_eq(&first, &second));
                }
                children.finish_round(round)?;
                Ok(MeasuredLayout::new(assigned, ()))
            },
            |_, _, (), _| Ok(()),
        )
        .unwrap();
        let observed = measurements::finish();
        assert_eq!(observed.roots.len(), 2);
        assert!(!observed.roots[0].successful && observed.roots[1].successful);
        assert_eq!(observed.counters[Counter::ChildCacheHit as usize], 3);
        assert_eq!(observed.counters[Counter::ChildCacheMiss as usize], 4);
        assert_eq!(observed.counters[Counter::ChildFeedbackReplay as usize], 1);
        assert_eq!(
            observed
                .children
                .iter()
                .map(|event| (event.root_round, event.phase, event.counter))
                .collect::<Vec<_>>(),
            [
                (0, "legend-probe", Counter::ChildCacheMiss),
                (0, "legend-probe", Counter::ChildCacheHit),
                (0, "panel", Counter::ChildCacheMiss),
                (1, "legend-probe", Counter::ChildCacheMiss),
                (1, "legend-probe", Counter::ChildCacheHit),
                (1, "panel", Counter::ChildCacheMiss),
                (1, "panel", Counter::ChildCacheHit),
                (1, "panel", Counter::ChildFeedbackReplay),
            ]
        );
        assert!(observed.children.iter().all(|event| event.owner == owner));
        println!(
            "controlled_shared_child,facet={facet},hits={},misses={},replays={},events={:?}",
            observed.counters[Counter::ChildCacheHit as usize],
            observed.counters[Counter::ChildCacheMiss as usize],
            observed.counters[Counter::ChildFeedbackReplay as usize],
            observed.children
        );
    }
}

fn inside_case(dense: bool) -> CaseMeasurement {
    let count = if dense { 400 } else { 6 };
    let sample = measure_case(
        if dense {
            "legend_inside_dense"
        } else {
            "legend_inside_sparse"
        },
        || {
            Plot::scatter(
                NumericValues::new(
                    "x",
                    (0..count).map(|i| {
                        if dense {
                            f64::from(i % 20) / 2.0
                        } else {
                            4.5 - f64::from(i) / 10.0
                        }
                    }),
                ),
                NumericValues::new(
                    "y",
                    (0..count).map(|i| {
                        if dense {
                            f64::from(i / 20) / 2.0
                        } else {
                            4.0 + f64::from(i) / 10.0
                        }
                    }),
                ),
            )
            .hue(
                CategoryValues::new(
                    "Condition",
                    (0..count).map(|i| format!("Condition {}", i % 3)),
                )
                .unwrap(),
            )
            .legend(LegendPolicy::Visible(LegendPosition::InsideTopRight))
            .panel_text_annotation(
                crate::PanelTextAnnotation::new(crate::PanelCorner::BottomLeft, "Caller note")
                    .unwrap(),
            )
            .size(800, 600)
        },
        |plot, path| plot.svg(path),
        true,
        true,
    );
    assert_observed(&sample, 3);
    // Explicit inside placement fixes the corner; it scans overlap and may grow
    // the range, but does not run the automatic four-corner selector.
    assert_eq!(
        sample.observations.counters[Counter::InsideLegendCandidate as usize],
        0
    );
    assert_eq!(
        sample.observations.counters[Counter::GeometryOverlapScan as usize],
        if dense { 2005 } else { 28 }
    );
    let comparisons = sample.observations.counters[Counter::AnnotationOverlapComparison as usize];
    assert_eq!(comparisons, 2);
    println!("annotation_overlap,dense={dense},comparisons={comparisons}");
    assert_eq!(sample.observations.roots.len(), 2);
    assert_inside_geometry(&sample, dense);
    sample
}

fn assert_inside_geometry(sample: &CaseMeasurement, dense: bool) {
    let svg = std::str::from_utf8(
        sample
            .output
            .as_ref()
            .unwrap()
            .preserved_svg
            .as_ref()
            .unwrap(),
    )
    .unwrap();
    let document = roxmltree::Document::parse(svg).unwrap();
    let guide = document
        .descendants()
        .find(|node| node.attribute("id") == Some("legend"))
        .unwrap();
    let rect = |node: roxmltree::Node<'_, '_>| {
        ["x", "y", "width", "height"]
            .map(|attribute| node.attribute(attribute).unwrap().parse::<f64>().unwrap())
    };
    let [x, y, width, height] = rect(
        guide
            .descendants()
            .find(|node| node.has_tag_name("rect"))
            .unwrap(),
    );
    let clip = document
        .descendants()
        .find(|node| node.has_tag_name("clipPath"))
        .unwrap();
    let [px, py, pw, ph] = rect(
        clip.descendants()
            .find(|node| node.has_tag_name("rect"))
            .unwrap(),
    );
    assert!(x > px + pw / 2.0 && y < py + ph / 2.0);
    assert!(x >= px && y >= py && x + width <= px + pw && y + height <= py + ph);
    assert!(svg.contains("Caller note"));
    let marks = document
        .descendants()
        .filter(|node| {
            node.has_tag_name("circle") && !node.ancestors().any(|ancestor| ancestor == guide)
        })
        .collect::<Vec<_>>();
    assert_eq!(marks.len(), if dense { 400 } else { 6 });
    for mark in marks {
        let [cx, cy, radius] = ["cx", "cy", "r"]
            .map(|attribute| mark.attribute(attribute).unwrap().parse::<f64>().unwrap());
        assert!(
            cx + radius <= x
                || cx - radius >= x + width
                || cy + radius <= y
                || cy - radius >= y + height,
            "inside guide must clear the retained marks"
        );
    }
    println!(
        "inside_geometry,dense={dense},corner=InsideTopRight,legend={:?},panel={:?},root_rounds={}",
        [x, y, width, height],
        [px, py, pw, ph],
        sample.observations.roots.len()
    );
}

#[test]
#[ignore = "release measurements; run with --release --ignored --nocapture --test-threads=1"]
fn inside_legend_measurements() {
    assert!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "requires --release"
    );
    println!("legend_dimensions,inside=800x600,sparse_marks=6,dense_marks=400,entries=3,annotation=BottomLeft");
    let samples = measure_pair(
        ["inside_sparse", "inside_dense"],
        [&|| inside_case(false), &|| inside_case(true)],
    );
    for pair in &samples[1..] {
        equal_output(&samples[0][0], &pair[0]);
        equal_output(&samples[0][1], &pair[1]);
    }
}

#[test]
fn populated_legend_fixture_contract() {
    for fixture in [
        "small",
        "starvation",
        "historical_reflow_640x400",
        "preferred_right_control",
        "native_compact_right",
    ] {
        let auto = outside_case(fixture, LegendPosition::Auto, true);
        let explicit = outside_case(fixture, selected(&auto), true);
        equal_output(&auto, &explicit);
        outside_case(
            fixture,
            LegendPosition::OutsideRight,
            fixture != "starvation",
        );
        outside_case(fixture, LegendPosition::OutsideBottom, true);
    }
    for facet in [false, true] {
        for pressure in [false, true] {
            shared_case(facet, pressure, LegendPosition::Auto);
        }
    }
    for dense in [false, true] {
        inside_case(dense);
    }
}

#[test]
fn shared_legend_selected_edge_contract() {
    for pressure in [false, true] {
        let auto = shared_case(true, pressure, LegendPosition::Auto);
        let explicit = shared_case(true, pressure, selected(&auto));
        equal_output(&auto, &explicit);
    }
}

#[test]
fn outside_recovery_controls_replay_identical_without_cache() {
    use crate::decorations::plan::children::DISABLE_CHILD_REUSE;
    for fixture in [
        "preferred_right_control",
        "starvation",
        "native_compact_right",
    ] {
        let cached = outside_case(fixture, LegendPosition::Auto, true);
        DISABLE_CHILD_REUSE.with(|disabled| disabled.set(true));
        let uncached =
            std::panic::catch_unwind(|| outside_case(fixture, LegendPosition::Auto, true));
        DISABLE_CHILD_REUSE.with(|disabled| disabled.set(false));
        equal_output(&cached, &uncached.unwrap());
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one fixture checks measured frame requirements, full marks, and cache-on/off byte identity"
)]
fn measured_gutter_reflow_preserves_complete_frames_without_cache() {
    use crate::decorations::plan::children::DISABLE_CHILD_REUSE;

    for facet in [false, true] {
        let render = |edge| {
            if facet {
                shared_case(true, true, edge)
            } else {
                outside_case("historical_reflow_640x400", edge, true)
            }
        };
        let auto = render(LegendPosition::Auto);
        assert_eq!(selected(&auto), LegendPosition::OutsideRight);
        let explicit = render(LegendPosition::OutsideRight);
        equal_output(&auto, &explicit);
        DISABLE_CHILD_REUSE.with(|disabled| disabled.set(true));
        let uncached = std::panic::catch_unwind(|| render(LegendPosition::OutsideRight));
        DISABLE_CHILD_REUSE.with(|disabled| disabled.set(false));
        let uncached = uncached.unwrap();
        equal_output(&auto, &uncached);
        for sample in [&auto, &explicit, &uncached] {
            assert_eq!(
                sample.observations.counters[Counter::StandardPanelIntrinsic as usize],
                if facet { 2 } else { 1 },
                "each prepared panel supplies one summary across all root rounds"
            );
            assert_eq!(
                sample.observations.capture_counters[Counter::StandardPanelIntrinsic as usize],
                0
            );
        }

        let svg = std::str::from_utf8(
            auto.output
                .as_ref()
                .unwrap()
                .preserved_svg
                .as_ref()
                .unwrap(),
        )
        .unwrap();
        let document = roxmltree::Document::parse(svg).unwrap();
        let rect = |node: roxmltree::Node<'_, '_>| {
            ["x", "y", "width", "height"]
                .map(|attribute| node.attribute(attribute).unwrap().parse::<f64>().unwrap())
        };
        let legend = document
            .descendants()
            .find(|node| {
                node.attribute("id")
                    .is_some_and(|id| id.ends_with("legend"))
            })
            .unwrap();
        let [lx, ly, lw, lh] = rect(
            legend
                .descendants()
                .find(|node| node.has_tag_name("rect"))
                .unwrap(),
        );
        let (width, height) = if facet {
            (560.0, 360.0)
        } else {
            (640.0, 400.0)
        };
        assert!(lx >= 0.0 && ly >= 0.0 && lx + lw <= width && ly + lh <= height);
        // Compare the final data frames with this fixture's scientific and
        // shaped-text requirements, rather than the retired 80x60 floor.
        let source = if facet {
            scatter(12, true)
                .facet(
                    crate::FacetSpec::wrap(
                        CategoryValues::new(
                            "Panel",
                            (0..24).map(|i| if i % 2 == 0 { "A" } else { "B" }),
                        )
                        .unwrap(),
                        2,
                    )
                    .unwrap(),
                )
                .size(560, 360)
                .erase()
        } else {
            outside("historical_reflow_640x400", LegendPosition::OutsideRight).erase()
        };
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let scopes = inputs
            .axes
            .iter()
            .map(|(scope, _)| scope.clone())
            .collect::<Vec<_>>();
        let chart = inputs.attach(source).unwrap();
        let theme = crate::Theme::Publication.resolve();
        let minimums = scopes
            .into_iter()
            .map(|scope| {
                let panel = scope.map_or_else(
                    || chart.clone(),
                    |scope| chart.clone().with_render_scope(scope),
                );
                panel.intrinsic_body_minimum(&theme).unwrap()
            })
            .collect::<Vec<_>>();
        let panels = document
            .descendants()
            .filter(|node| node.has_tag_name("clipPath"))
            .map(|clip| {
                rect(
                    clip.descendants()
                        .find(|node| node.has_tag_name("rect"))
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(panels.len(), if facet { 2 } else { 1 });
        assert_eq!(minimums.len(), panels.len());
        for ([x, y, w, h], minimum) in panels.into_iter().zip(minimums) {
            assert!(
                w >= minimum.0 && h >= minimum.1,
                "data body {w}x{h} is smaller than its traced {minimum:?}"
            );
            assert!(x >= 0.0 && y >= 0.0 && x + w <= lx && y + h <= height);
        }
        assert_eq!(
            document
                .descendants()
                .filter(|node| node.has_tag_name("circle")
                    && !node.ancestors().any(|ancestor| ancestor == legend))
                .count(),
            if facet { 24 } else { 36 }
        );
        for root in &auto.observations.roots {
            assert!(root.guides.len() <= 8);
            let trials = &auto.observations.guides[root.guides.clone()];
            assert!(trials
                .iter()
                .all(|trial| trial.edge == LegendPosition::OutsideRight));
            assert_eq!(
                trials.last().unwrap().outcome,
                if root.successful {
                    GuideOutcome::Accepted
                } else {
                    GuideOutcome::Provisional
                }
            );
        }
    }
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "exact quantized geometry and integer gutter contract"
)]
fn historical_reflow_native_measured_gutter_control() {
    use crate::layout::fit::FitOutcome;
    use crate::layout::kernel::FeedbackKind;
    use crate::layout::{AddressRole, ComponentAddress, MeasuredLayout};
    use crate::orchestrator::measurements;

    let plot = outside("historical_reflow_640x400", LegendPosition::Auto).erase();
    let theme = crate::Theme::Publication.resolve();
    let address = ComponentAddress::root()
        .child(
            AddressRole::PlotBody,
            0,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap();
    let mut rounds = Vec::new();
    measurements::start();
    // The root reserves only its own padding; axes submit absolute measured
    // gutters through feedback rather than borrowing the root's 16px inset.
    crate::layout::plan_exact_root(
        640,
        400,
        |stage| crate::plots::shared_x::prepare_plot(&plot, &stage.generation_resources()),
        |inputs, assigned, round| {
            let plot = inputs.attach(plot.clone())?;
            let canvas = plot.measured_canvas_layout(&theme)?;
            let FitOutcome::Measured(plan) = crate::decorations::plan::plan_panel(
                &plot,
                plot.kind(),
                canvas.panel_box,
                crate::decorations::plan::LegendRole::Panel,
                &theme,
                round,
                &address,
            )?
            else {
                panic!("preferred guide must fit the native frame")
            };
            let required = plan.result.axes.as_ref().unwrap().requirements();
            assert_eq!(canvas.insets.left, 16.0);
            let requested = [required.left, required.right, required.bottom, required.top]
                .map(|value| (value + 4.0).ceil());
            assert_eq!(
                round.next_insets(&address, FeedbackKind::AxisGutter)?,
                requested
            );
            let consumed = round.insets(&address, FeedbackKind::AxisGutter)?;
            assert_eq!(plan.panel_box.x0, canvas.panel_box.x0 + consumed[0]);
            assert_eq!(plan.panel_box.y0, canvas.panel_box.y0 + consumed[2]);
            assert_eq!(plan.panel_box.y1, canvas.panel_box.y1 - consumed[3]);
            rounds.push((consumed, requested, plan.panel_box.width()));
            println!(
                "native_gutter_round,consumed={consumed:?},requested={requested:?},panel={:?}",
                plan.panel_box
            );
            Ok(MeasuredLayout::new(assigned, ()))
        },
        |_, _, (), _| Ok(()),
    )
    .unwrap();
    let observed = measurements::finish();
    assert_eq!(
        rounds,
        [
            // Complete glyph/frame enclosure reserves one more spatial quantum
            // for this discrete guide. Consumed/requested gutters stay exact.
            ([0.0; 4], [79.0, 5.0, 59.0, 5.0], 367.788_085_937_5),
            (
                [79.0, 5.0, 59.0, 5.0],
                [94.0, 5.0, 59.0, 5.0],
                283.788_085_937_5
            ),
            (
                [94.0, 5.0, 59.0, 5.0],
                [94.0, 5.0, 59.0, 5.0],
                268.788_085_937_5
            ),
        ]
    );
    assert_eq!(observed.roots.len(), 3);
    assert_eq!(observed.counters[Counter::GuideTrial as usize], 3);
    assert_eq!(observed.counters[Counter::GuideBodyCheck as usize], 3);
    assert_eq!(observed.guides.len(), 3);
    for (ordinal, trial) in observed.guides.iter().enumerate() {
        let successful = ordinal == 2;
        assert_eq!(observed.roots[ordinal].ordinal, ordinal);
        assert_eq!(observed.roots[ordinal].successful, successful);
        assert_eq!(observed.roots[ordinal].guides, ordinal..ordinal + 1);
        assert_eq!(trial.root_round, ordinal);
        assert_eq!(trial.edge, LegendPosition::OutsideRight);
        assert_eq!(
            (trial.compact, trial.phase, trial.outcome),
            (
                false,
                GuidePhase::Body,
                if successful {
                    GuideOutcome::Accepted
                } else {
                    GuideOutcome::Provisional
                }
            )
        );
    }
}
