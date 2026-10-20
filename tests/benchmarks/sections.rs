//! Actual sectioned sources, parent/root requests and retained publication work.

use super::{measure_case, measure_pair, CaseMeasurement};
use crate::decorations::legend::discrete::{GuideCounts, PreparedDiscreteGuide};
use crate::decorations::plan::children::DISABLE_CHILD_REUSE;
use crate::orchestrator::measurements::Counter;
use crate::{
    Bounds, Canvas, CanvasLegendPolicy, CategoryValues, InteractionStyle, Labels,
    LegendComposition, LegendPolicy, LegendPosition, LinePattern, NumericValues, Plot, PointShape,
};

#[derive(Debug, Clone, Copy)]
enum Owner {
    Local,
    Facets,
    PerPlot,
    Shared,
}

#[derive(Debug, Clone, Copy)]
enum Request {
    Automatic,
    Width,
    Exact,
}

fn source(dense: bool) -> Plot<crate::kind::Interaction> {
    let pairs = if dense {
        (0..5)
            .flat_map(|group| (0..4).map(move |hue| (group, hue)))
            .collect::<Vec<_>>()
    } else {
        vec![(0, 0), (0, 1), (1, 1)]
    };
    let mut conditions = Vec::new();
    let mut groups = Vec::new();
    let mut hues = Vec::new();
    let mut estimates = Vec::new();
    for (group, hue) in pairs {
        for condition in 0..2 {
            conditions.push(if condition == 0 { "Before" } else { "After" });
            groups.push(format!("G{}", group + 1));
            hues.push(format!("H{}", hue + 1));
            estimates.push(f64::from(group * 3 + hue) + f64::from(condition) * 0.5);
        }
    }
    let lower = estimates
        .iter()
        .map(|value| value - 0.2)
        .collect::<Vec<_>>();
    let upper = estimates
        .iter()
        .map(|value| value + 0.2)
        .collect::<Vec<_>>();
    Plot::interaction(
        CategoryValues::new("condition", conditions).unwrap(),
        NumericValues::new("estimate", estimates),
        CategoryValues::new("group", groups).unwrap(),
        Bounds::new(
            NumericValues::new("lower", lower),
            NumericValues::new("upper", upper),
        ),
    )
    .hue(CategoryValues::new("hue", hues).unwrap())
    .styles((0..if dense { 5 } else { 2 }).map(|group| {
        InteractionStyle::new(format!("G{}", group + 1))
            .unwrap()
            .with_point_shape(if group % 2 == 0 {
                PointShape::Circle
            } else {
                PointShape::TriangleUp
            })
            .unwrap()
            .with_line_pattern(if group % 2 == 0 {
                LinePattern::Solid
            } else {
                LinePattern::Dashed
            })
            .unwrap()
    }))
    .labels(Labels::raw())
}

fn assert_sources(dense: bool, composition: LegendComposition) {
    let plot = source(dense).legend_composition(composition).erase();
    let pairs = if dense { 20 } else { 3 };
    let keys = if composition == LegendComposition::Combined {
        pairs
    } else if dense {
        9
    } else {
        4
    };
    let descriptor = PreparedDiscreteGuide::prepare_for_test(
        &plot,
        &crate::Theme::publication().resolve().unwrap(),
        composition,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        descriptor.counts,
        GuideCounts {
            sections: if composition == LegendComposition::Combined {
                1
            } else {
                2
            },
            entries: keys,
            points: keys,
            lines: keys,
            fills: 0,
            primary_source_keys: pairs,
            observed_pair_source_keys: pairs,
            labeled_layer_source_keys: 0,
            source_occurrences: pairs
                * if composition == LegendComposition::Combined {
                    1
                } else {
                    2
                },
        }
    );
    println!(
        "section_source_counts,dense={dense},composition={composition:?},{:?}",
        descriptor.counts
    );
    let identities = if composition == LegendComposition::Combined {
        if dense {
            (1..=5)
                .flat_map(|group| (1..=4).map(move |hue| format!("G{group}, H{hue}")))
                .collect::<Vec<_>>()
        } else {
            ["G1, H1", "G1, H2", "G2, H2"].map(str::to_owned).to_vec()
        }
    } else {
        (1..=if dense { 5 } else { 2 })
            .map(|group| format!("G{group}"))
            .chain((1..=if dense { 4 } else { 2 }).map(|hue| format!("H{hue}")))
            .collect()
    };
    assert_eq!(
        descriptor
            .sections
            .iter()
            .flat_map(|section| section.entries.iter())
            .map(|entry| entry.display_label.as_ref())
            .collect::<Vec<_>>(),
        identities
    );
}

enum Root {
    Plot(Box<Plot<crate::kind::Interaction>>),
    Canvas(Box<Canvas>),
}

impl Root {
    fn new(
        owner: Owner,
        dense: bool,
        composition: LegendComposition,
        edge: LegendPosition,
        request: Request,
    ) -> Self {
        let plot = source(dense).legend_composition(composition);
        let root = match owner {
            Owner::Local => Self::Plot(Box::new(plot.legend(LegendPolicy::Visible(edge)))),
            Owner::Facets => {
                let panels = (0..if dense { 5 } else { 2 }).flat_map(|group| {
                    std::iter::repeat_n(
                        if group % 2 == 0 { "Left" } else { "Right" },
                        if dense {
                            8
                        } else if group == 0 {
                            4
                        } else {
                            2
                        },
                    )
                });
                Self::Plot(Box::new(
                    plot.facet(crate::FacetSpec::column(
                        CategoryValues::new("panel", panels).unwrap(),
                    ))
                    .legend(LegendPolicy::Visible(edge)),
                ))
            }
            Owner::PerPlot | Owner::Shared => Self::Canvas(Box::new(
                Canvas::grid(1, 2)
                    .unwrap()
                    .panel(
                        0,
                        0,
                        plot.clone()
                            .legend_composition(if matches!(owner, Owner::Shared) {
                                LegendComposition::Combined
                            } else {
                                composition
                            }),
                    )
                    .unwrap()
                    .panel(0, 1, plot)
                    .unwrap()
                    .legend_policy(if matches!(owner, Owner::Shared) {
                        CanvasLegendPolicy::Shared
                    } else {
                        CanvasLegendPolicy::PerPlot
                    })
                    .legend_composition(composition),
            )),
        };
        match (root, request) {
            (Self::Plot(plot), Request::Automatic) => Self::Plot(plot),
            (Self::Plot(plot), Request::Width) => Self::Plot(Box::new((*plot).width(1200))),
            (Self::Plot(plot), Request::Exact) => Self::Plot(Box::new((*plot).size(1200, 800))),
            (Self::Canvas(canvas), Request::Automatic) => Self::Canvas(canvas),
            (Self::Canvas(canvas), Request::Width) => Self::Canvas(Box::new((*canvas).width(1200))),
            (Self::Canvas(canvas), Request::Exact) => {
                Self::Canvas(Box::new((*canvas).size(1200, 800)))
            }
        }
    }

    fn publish(self, path: &std::path::Path) -> crate::Result<crate::Output> {
        match self {
            Self::Plot(plot) => (*plot).svg(path),
            Self::Canvas(canvas) => (*canvas).svg(path),
        }
    }
}

fn case(
    owner: Owner,
    dense: bool,
    composition: LegendComposition,
    request: Request,
    edge: LegendPosition,
    disabled: bool,
) -> CaseMeasurement {
    DISABLE_CHILD_REUSE.with(|flag| flag.set(disabled));
    let sample = std::panic::catch_unwind(|| {
        measure_case(
            &format!("section_{owner:?}_{dense}_{composition:?}_{request:?}_{disabled}"),
            || Root::new(owner, dense, composition, edge, request),
            Root::publish,
            true,
            true,
        )
    });
    DISABLE_CHILD_REUSE.with(|flag| flag.set(false));
    let sample = sample.unwrap();
    assert_eq!(
        (sample.observations.calls[2], sample.observations.calls[3]),
        (1, 1)
    );
    assert_eq!(
        sample.observations.capture_counters,
        [0; Counter::ALL.len()]
    );
    for root in &sample.observations.roots {
        let trials = &sample.observations.guides[root.guides.clone()];
        for trial in trials {
            assert!(
                trials
                    .iter()
                    .filter(|other| other.owner == trial.owner)
                    .count()
                    <= 8
            );
        }
    }
    assert_output(&sample, owner, dense, composition, request);
    sample
}

fn assert_output(
    sample: &CaseMeasurement,
    owner: Owner,
    dense: bool,
    composition: LegendComposition,
    request: Request,
) {
    let bytes = sample.output.as_ref().unwrap();
    let svg = std::str::from_utf8(bytes.preserved_svg.as_ref().unwrap()).unwrap();
    let document = roxmltree::Document::parse(svg).unwrap();
    let guides = document
        .descendants()
        .filter(|node| {
            node.attribute("id")
                .is_some_and(|id| id.ends_with("legend"))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        guides.len(),
        if matches!(owner, Owner::PerPlot) {
            2
        } else {
            1
        }
    );
    let keys = if composition == LegendComposition::Combined {
        if dense {
            20
        } else {
            3
        }
    } else if dense {
        9
    } else {
        4
    };
    for guide in &guides {
        assert_eq!(
            guide
                .descendants()
                .filter(|node| node.has_tag_name("circle") || node.has_tag_name("polygon"))
                .count(),
            keys
        );
        assert_eq!(
            guide
                .descendants()
                .filter(|node| node.has_tag_name("polyline"))
                .count(),
            keys
        );
        let text = guide
            .descendants()
            .filter(|node| node.has_tag_name("text"))
            .filter_map(|node| node.text())
            .collect::<Vec<_>>();
        if composition == LegendComposition::Combined {
            assert_eq!(text[0], "group × hue");
            assert_eq!(text.len(), keys + 1);
        } else {
            let groups = if dense { 5 } else { 2 };
            let hues = if dense { 4 } else { 2 };
            let expected = std::iter::once("group".to_owned())
                .chain((1..=groups).map(|group| format!("G{group}")))
                .chain(std::iter::once("hue".to_owned()))
                .chain((1..=hues).map(|hue| format!("H{hue}")))
                .collect::<Vec<_>>();
            assert_eq!(text, expected);
        }
    }
    let points = document
        .descendants()
        .filter(|node| {
            (node.has_tag_name("circle") || node.has_tag_name("polygon"))
                && !node.ancestors().any(|parent| guides.contains(&parent))
        })
        .count();
    assert_eq!(
        points,
        (if dense { 40 } else { 6 })
            * if matches!(owner, Owner::PerPlot | Owner::Shared) {
                2
            } else {
                1
            }
    );
    let root = document.root_element();
    if !matches!(request, Request::Automatic) {
        assert_eq!(root.attribute("width"), Some("1200"));
    }
    if matches!(request, Request::Exact) {
        assert_eq!(root.attribute("height"), Some("800"));
    }
}

#[test]
fn sectioned_guides_preserve_all_owner_requests_and_frozen_output_without_cache() {
    use super::legends::equal_output;
    for dense in [false, true] {
        for composition in [LegendComposition::Combined, LegendComposition::Separate] {
            assert_sources(dense, composition);
        }
    }
    for owner in [Owner::Local, Owner::Facets, Owner::PerPlot, Owner::Shared] {
        for request in [Request::Automatic, Request::Width, Request::Exact] {
            let cached = case(
                owner,
                false,
                LegendComposition::Separate,
                request,
                LegendPosition::Auto,
                false,
            );
            let uncached = case(
                owner,
                false,
                LegendComposition::Separate,
                request,
                LegendPosition::Auto,
                true,
            );
            equal_output(&cached, &uncached);
        }
        let cached = case(
            owner,
            true,
            LegendComposition::Separate,
            Request::Exact,
            LegendPosition::Auto,
            false,
        );
        let uncached = case(
            owner,
            true,
            LegendComposition::Separate,
            Request::Exact,
            LegendPosition::Auto,
            true,
        );
        equal_output(&cached, &uncached);
        case(
            owner,
            true,
            LegendComposition::Combined,
            Request::Exact,
            LegendPosition::Auto,
            false,
        );
    }
    // Retain the replaced facet auto-versus-selected-edge contract on actual
    // two-section keys, rather than only the old one-section Scatter fixture.
    let auto = case(
        Owner::Facets,
        false,
        LegendComposition::Separate,
        Request::Exact,
        LegendPosition::Auto,
        false,
    );
    let edge = auto
        .observations
        .guides
        .iter()
        .rev()
        .find(|trial| trial.outcome == crate::orchestrator::measurements::GuideOutcome::Accepted)
        .unwrap()
        .edge;
    let explicit = case(
        Owner::Facets,
        false,
        LegendComposition::Separate,
        Request::Exact,
        edge,
        false,
    );
    equal_output(&auto, &explicit);
}

pub(super) fn measure() {
    use super::legends::equal_output;
    println!("section_conditions,release,dense_5x4,40_source_rows,20_pair_keys,Separate_2_sections_9_keys,three_interleaved_runs,all_owners,exact_1200x800");
    for composition in [LegendComposition::Combined, LegendComposition::Separate] {
        assert_sources(true, composition);
    }
    for owner in [Owner::Local, Owner::Facets, Owner::PerPlot, Owner::Shared] {
        let combined = || {
            case(
                owner,
                true,
                LegendComposition::Combined,
                Request::Exact,
                LegendPosition::Auto,
                false,
            )
        };
        let separate = || {
            case(
                owner,
                true,
                LegendComposition::Separate,
                Request::Exact,
                LegendPosition::Auto,
                false,
            )
        };
        let samples = measure_pair(
            [
                &format!("section_{owner:?}_combined"),
                &format!("section_{owner:?}_separate"),
            ],
            [&combined, &separate],
        );
        for (run, pair) in samples.iter().enumerate() {
            for (composition, sample) in [LegendComposition::Combined, LegendComposition::Separate]
                .iter()
                .zip(pair)
            {
                let bytes = sample.output.as_ref().unwrap();
                println!(
                    "section_bytes,{owner:?},{composition:?},{run},{},{},{},roots={},shapes={}",
                    bytes.svg.len(),
                    bytes.png.len(),
                    bytes.preserved_svg.as_ref().unwrap().len(),
                    sample.observations.roots.len(),
                    sample.shapes,
                );
            }
        }
        for pair in &samples[1..] {
            equal_output(&samples[0][0], &pair[0]);
            equal_output(&samples[0][1], &pair[1]);
        }
        let uncached = case(
            owner,
            true,
            LegendComposition::Separate,
            Request::Exact,
            LegendPosition::Auto,
            true,
        );
        equal_output(&samples[0][1], &uncached);
    }
}
