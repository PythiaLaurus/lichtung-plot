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
        "compact_right" => scatter(2, true)
            .x_label("MMMMMMMMMMMMMMMMMMMM")
            .size(630, 480),
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
    for root in &observed.roots {
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
            "starvation" | "compact_right" => 2,
            "historical_reflow_640x400" => 18,
            _ => unreachable!(),
        },
    );
    if edge == LegendPosition::Auto {
        assert_outside_auto(&sample, fixture);
    }
    if !success {
        assert!(sample
            .error
            .as_ref()
            .unwrap()
            .to_string()
            .contains("x-axis title"));
    }
    sample
}

fn assert_outside_auto(sample: &CaseMeasurement, fixture: &str) {
    use GuideOutcome::{Accepted, Provisional, Rejected};
    use GuidePhase::{Body, GuideOnly};
    use LegendPosition::{OutsideRight, OutsideTop};

    let expected: &[_] = match fixture {
        "small" => &[(OutsideRight, false, Body, Accepted)],
        "starvation" => &[
            (OutsideRight, false, Body, Rejected),
            (OutsideRight, true, Body, Rejected),
            (OutsideTop, false, Body, Accepted),
        ],
        "compact_right" => &[
            (OutsideRight, false, Body, Rejected),
            (OutsideRight, true, Body, Accepted),
        ],
        "historical_reflow_640x400" => &[
            (OutsideRight, false, GuideOnly, Rejected),
            (OutsideRight, true, Body, Provisional),
            (OutsideRight, false, GuideOnly, Rejected),
            (OutsideRight, true, Body, Accepted),
        ],
        _ => unreachable!(),
    };
    let observed = &sample.observations;
    assert_eq!(
        observed.roots.len(),
        if fixture == "historical_reflow_640x400" {
            2
        } else {
            1
        }
    );
    assert_eq!(
        observed
            .guides
            .iter()
            .map(|trial| (trial.edge, trial.compact, trial.phase, trial.outcome))
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(selected(sample), expected.last().unwrap().0);
    assert_eq!(
        observed.counters[Counter::GuideBodyCheck as usize],
        u64::try_from(expected.iter().filter(|trial| trial.2 == Body).count()).unwrap()
    );
    if matches!(fixture, "compact_right" | "historical_reflow_640x400") {
        assert_eq!(
            observed.counters[Counter::TextProposalVisit as usize],
            if fixture == "compact_right" { 11 } else { 180 }
        );
    }
}

#[test]
#[ignore = "release measurements; run with --release --ignored --nocapture --test-threads=1"]
fn outside_legend_measurements() {
    assert!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "requires --release"
    );
    println!("legend_conditions,release,three_interleaved_runs,selection_probe_before_pairs,OS_cache_and_scheduling_uncontrolled,SVG_outlined,PNG_source_balanced,preserved_text_checks_outside_timing");
    println!("legend_dimensions,small=800x480,starvation=640x480,historical_reflow_640x400=640x400,compact_right=630x480");
    for fixture in [
        "small",
        "starvation",
        "historical_reflow_640x400",
        "compact_right",
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

fn shared_case(facet: bool, pressure: bool, edge: LegendPosition) -> CaseMeasurement {
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
    assert!(sample.observations.counters[Counter::GuideBodyCheck as usize] > 0);
    assert_child_reuse(&sample, facet, pressure);
    let (rounds, trials, checks) = if pressure {
        if facet {
            (3, 4, 6)
        } else {
            (3, 7, 6)
        }
    } else {
        (2, 2, 4)
    };
    assert_eq!(sample.observations.roots.len(), rounds);
    assert_eq!(
        sample.observations.counters[Counter::GuideTrial as usize],
        trials
    );
    assert_eq!(
        sample.observations.counters[Counter::GuideBodyCheck as usize],
        checks
    );
    if pressure && edge == LegendPosition::Auto {
        assert!(sample
            .observations
            .guides
            .iter()
            .any(|trial| trial.outcome == GuideOutcome::Rejected));
    }
    assert_eq!(
        selected(&sample),
        if pressure && !facet {
            LegendPosition::OutsideTop
        } else {
            LegendPosition::OutsideRight
        }
    );
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
        if pressure { 24 } else { 16 }
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
            )
            .unwrap();
        let theme = crate::Theme::Publication.resolve();
        measurements::start();
        crate::layout::plan_exact_root(
            1200,
            720,
            |_| Ok(()),
            |(), assigned, round| {
                let panel = PanelBox::new(100.0, 1100.0, 100.0, 620.0);
                let context = crate::spec::PlotRenderContext::new(plot.data(), panel, &theme);
                let first = children.measure(
                    &plot,
                    context,
                    &owner,
                    MeasurementPhase::Legend,
                    round,
                    || {
                        plot.preflight_measurement(
                            plot.kind(),
                            panel,
                            &theme,
                            &crate::layout::fit::ROOT_SCOPE,
                        )
                    },
                )?;
                let second = children.measure(
                    &plot,
                    context,
                    &owner,
                    MeasurementPhase::Legend,
                    round,
                    || panic!("identical shared legend probe must hit the actual cache"),
                )?;
                let (
                    crate::layout::fit::FitOutcome::Measured(first),
                    crate::layout::fit::FitOutcome::Measured(second),
                ) = (first, second)
                else {
                    panic!("populated legend probe must fit")
                };
                assert!(Arc::ptr_eq(&first, &second));
                let first = children.plan(
                    &plot,
                    crate::decorations::plan::LegendRole::Suppressed,
                    context,
                    None,
                    round,
                    &owner,
                )?;
                if round.child_stable(std::slice::from_ref(&owner)) {
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
                children.finish_round(round);
                Ok(MeasuredLayout::new(assigned, ()))
            },
            |(), _, (), _| Ok(()),
        )
        .unwrap();
        let observed = measurements::finish();
        assert_eq!(observed.counters[Counter::ChildCacheHit as usize], 2);
        assert_eq!(observed.counters[Counter::ChildCacheMiss as usize], 2);
        assert_eq!(observed.counters[Counter::ChildFeedbackReplay as usize], 1);
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
        "compact_right",
    ] {
        let auto = outside_case(fixture, LegendPosition::Auto, true);
        let explicit = outside_case(fixture, selected(&auto), true);
        equal_output(&auto, &explicit);
    }
    outside_case("starvation", LegendPosition::OutsideRight, false);
    outside_case(
        "historical_reflow_640x400",
        LegendPosition::OutsideRight,
        true,
    );
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
        equal_output(&auto, &uncached.unwrap());

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
        for [x, y, w, h] in panels {
            assert!(w >= 80.0 && h >= 60.0, "invalid data body: {w}x{h}");
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
fn historical_reflow_measured_gutter_control() {
    use crate::layout::fit::FitOutcome;
    use crate::layout::kernel::{FeedbackKind, FitKind};
    use crate::layout::{AddressRole, ComponentAddress, MeasuredLayout};
    use crate::orchestrator::measurements;

    let plot = outside("historical_reflow_640x400", LegendPosition::Auto).erase();
    let theme = crate::Theme::Publication.resolve();
    let cell = plot.measured_canvas_layout(&theme).unwrap().panel_box;
    let address = ComponentAddress::root()
        .child(AddressRole::PlotBody, 0)
        .unwrap();
    measurements::start();
    // Independently establish the measured-gutter input used by the public
    // root's second round, without relying on its first provisional guide.
    crate::layout::plan_exact_root(
        640,
        400,
        |_| Ok(()),
        |(), assigned, round| {
            if round.insets(&address, FeedbackKind::AxisGutter)[0] == 0.0 {
                let request = round.request_insets(
                    &address,
                    &address,
                    FeedbackKind::AxisGutter,
                    [15.0, 0.0, 0.0, 0.0],
                )?;
                round.miss(
                    &address,
                    FitKind::Tick,
                    &[request],
                    "controlled measured gutter",
                )?;
            } else {
                let FitOutcome::Measured(plan) = crate::decorations::plan::plan_panel(
                    &plot,
                    plot.kind(),
                    cell,
                    crate::decorations::plan::LegendRole::Panel,
                    &theme,
                    round,
                    &address,
                )?
                else {
                    panic!("remeasured compact guide must fit with its measured gutter")
                };
                let required = plan.result.axes.as_ref().unwrap().requirements();
                assert_eq!(required.left, 89.269_531_25);
                assert_eq!(
                    (required.left + 4.0 - plot.base_canvas_layout(&theme).insets.left).ceil(),
                    15.0
                );
                assert_eq!(plan.panel_box.width(), 80.0);
            }
            Ok(MeasuredLayout::new(assigned, ()))
        },
        |(), _, (), _| Ok(()),
    )
    .unwrap();
    let observed = measurements::finish();
    assert_eq!(observed.roots.len(), 2);
    assert!(!observed.roots[0].successful && observed.roots[1].successful);
    assert_eq!(observed.counters[Counter::GuideTrial as usize], 2);
    assert_eq!(observed.counters[Counter::GuideBodyCheck as usize], 1);
    assert_eq!(observed.guides.len(), 2);
    for (trial, compact, phase, outcome) in [
        (
            &observed.guides[0],
            false,
            GuidePhase::GuideOnly,
            GuideOutcome::Rejected,
        ),
        (
            &observed.guides[1],
            true,
            GuidePhase::Body,
            GuideOutcome::Accepted,
        ),
    ] {
        assert_eq!(trial.root_round, 1);
        assert_eq!(trial.edge, LegendPosition::OutsideRight);
        assert_eq!(
            (trial.compact, trial.phase, trial.outcome),
            (compact, phase, outcome)
        );
    }
}
