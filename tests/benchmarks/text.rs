use super::annotations::summarize_local;
use super::{measure_case, measure_pair, CaseMeasurement};
use crate::orchestrator::measurements::{self, Counter};
use crate::{CategoryValues, HeatmapLabelFit, NumericValues, Plot};
use std::time::{Duration, Instant};

#[expect(
    clippy::too_many_lines,
    reason = "keep fixture, root-attributed work, and retained-label correctness checks together"
)]
fn heatmap_case(long: bool, wrap: bool, ambiguous: bool) -> CaseMeasurement {
    let labels = ["Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot"].map(|prefix| {
        if ambiguous {
            format!("Shared deliberately long prefix {prefix}")
        } else if long {
            format!("{prefix} deliberately long category label")
        } else {
            prefix.to_owned()
        }
    });
    let sample = measure_case(
        "heatmap_local_search",
        || {
            Plot::heatmap(
                CategoryValues::new("Column", labels.clone()).unwrap(),
                CategoryValues::new(
                    "Row",
                    ["Upper", "Upper", "Upper", "Lower", "Lower", "Lower"],
                )
                .unwrap(),
                NumericValues::new("Response", [0.0, 1.0, 2.0, 3.0, 4.0, 5.0]),
            )
            .size(1200, 500)
            .heatmap_label_fit(if wrap {
                HeatmapLabelFit::Wrap
            } else {
                HeatmapLabelFit::Elide
            })
        },
        |root, path| root.svg(path),
        !ambiguous || wrap,
        true,
    );
    let trials = sample.observations.counters[Counter::HeatmapPrefixTrial as usize];
    if wrap {
        assert_eq!(trials, 0, "Wrap is a policy, not a trial following elision");
    } else if long || ambiguous {
        assert_eq!(
            trials,
            if ambiguous { 108 } else { 328 },
            "long labels must exercise actual prefix candidates: {:?}",
            sample.observations.roots
        );
    } else {
        assert_eq!(trials, 0, "full-label width checks are not prefix trials");
    }
    if long && !wrap && !ambiguous {
        // Complete guide extents round outward to layout quanta. The resulting
        // reservation changes actual candidate work by one prefix in each round.
        // The earlier guide reservation pinned 330 candidates (115/215).
        let roots = sample
            .observations
            .roots
            .iter()
            .map(|root| (root.ordinal, root.heatmap_prefix_trials, root.successful))
            .collect::<Vec<_>>();
        assert_eq!(roots, [(0, 114, false), (1, 214, true)]);
        assert_eq!(trials, roots.iter().map(|root| root.1).sum::<u64>());
        println!("heatmap_prefix_roots,{roots:?}");
    }
    if let Some(output) = &sample.output {
        let svg = std::str::from_utf8(output.preserved_svg.as_ref().unwrap()).unwrap();
        let document = roxmltree::Document::parse(svg).unwrap();
        let texts = document
            .descendants()
            .filter(|node| node.has_tag_name("text"))
            .filter_map(|node| node.text())
            .collect::<Vec<_>>();
        if wrap {
            assert!(!texts.iter().any(|text| text.ends_with("...")));
            let retained = texts
                .concat()
                .split_whitespace()
                .collect::<String>()
                .to_ascii_lowercase();
            for label in &labels {
                let complete = label
                    .split_whitespace()
                    .collect::<String>()
                    .to_ascii_lowercase();
                assert!(
                    retained.contains(&complete),
                    "incomplete wrapped label {label}"
                );
            }
        } else if long {
            let selected = texts
                .iter()
                .filter(|text| text.ends_with("..."))
                .copied()
                .collect::<Vec<_>>();
            assert_eq!(selected.len(), 6);
            assert_eq!(
                selected
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
                6
            );
            for (source, selected) in labels.iter().zip(&selected) {
                assert!(source.starts_with(selected.strip_suffix("...").unwrap()));
            }
            println!("heatmap_selected,{selected:?}");
        }
    } else {
        assert!(sample
            .error
            .as_ref()
            .unwrap()
            .to_string()
            .contains("ambiguous after elision"));
    }
    println!(
        "heatmap_local,long={long},wrap={wrap},ambiguous={ambiguous},prefix_trials={trials},shapes={},text_exact={}",
        sample.shapes,
        sample.observations.counters[Counter::TextExactTrial as usize]
    );
    sample
}

#[expect(
    clippy::too_many_lines,
    reason = "fixture, bounded root work and complete variable-square assertions share one workload"
)]
fn matrix_case(long: bool, size: u32) -> CaseMeasurement {
    let sample = measure_case(
        "matrix_local_search",
        || {
            Plot::matrix(
                (0..3)
                    .map(|index| {
                        NumericValues::new(
                            if long {
                                format!("Variable {index} measured response over time")
                            } else {
                                format!("V{index}")
                            },
                            // Multipliers 5/8/11 avoid the constant column from 7/10/13 modulo 13.
                            (0..12).map(move |row| f64::from((row * (index * 3 + 5)) % 13)),
                        )
                    })
                    .collect(),
            )
            .size(size, size)
        },
        |root, path| {
            root.svg(path).inspect_err(|error| {
                println!("matrix_fixture_error,long={long},size={size},{error}");
            })
        },
        true,
        true,
    );
    let passes = sample.observations.counters[Counter::MatrixLayoutPass as usize];
    // Each assigned Grid call allocates once; changed gutters belong to root rounds.
    assert!(passes >= sample.observations.roots.len() as u64);
    assert!(passes <= 2 * sample.observations.roots.len() as u64);
    assert!(sample.observations.roots.last().unwrap().successful);
    assert_eq!(
        sample.observations.counters[Counter::TickSource as usize],
        6 * passes
    );
    assert_eq!(
        sample.observations.counters[Counter::TickDistinctList as usize],
        6 * passes
    );
    assert_eq!(
        sample.observations.counters[Counter::TickMetricLookup as usize],
        if size == 600 {
            18 * passes
        } else {
            36 * passes
        }
    );
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
    let texts = document
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .filter_map(|node| node.text())
        .collect::<Vec<_>>();
    println!("matrix_selected,{texts:?}");
    let cells = document
        .descendants()
        .filter(|node| {
            node.has_tag_name("rect")
                && node
                    .parent()
                    .is_some_and(|parent| parent.has_tag_name("clipPath"))
        })
        .map(|node| {
            (
                node.attribute("width").unwrap().parse::<f64>().unwrap(),
                node.attribute("height").unwrap().parse::<f64>().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(cells.len(), 9);
    assert!(cells
        .iter()
        .all(|(width, height)| *width > 0.0 && *height > 0.0));
    println!("matrix_cells,long={long},size={size},{cells:?}");
    for index in 0..3 {
        let label = if long {
            format!("Variable {index}")
        } else {
            format!("V {index}")
        };
        assert_eq!(
            texts.iter().filter(|text| text.starts_with(&label)).count(),
            2
        );
    }
    println!(
        "matrix_local,long={long},size={size},passes={passes},roots={},tick_sources={},tick_lists={},tick_metrics={},shapes={}",
        sample.observations.roots.len(),
        sample.observations.counters[Counter::TickSource as usize],
        sample.observations.counters[Counter::TickDistinctList as usize],
        sample.observations.counters[Counter::TickMetricLookup as usize],
        sample.shapes
    );
    sample
}

fn paragraph_case(multiline: bool) -> [Duration; 2] {
    let source = if multiline {
        "Measured responses remained stable across independent trials under controlled conditions. "
            .repeat(7)
            + "\nIndependent observations confirm the measured response across repeated trials."
    } else {
        "Measured response".to_owned()
    };
    let theme = crate::Theme::Publication.resolve();
    theme.prepare_font_environment().unwrap();
    let mut previous = None;
    let mut elapsed_samples = [Duration::ZERO; 2];
    for (repeat, elapsed_sample) in elapsed_samples.iter_mut().enumerate() {
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        measurements::start();
        let start = Instant::now();
        let plan = theme
            .plan_text(
                &source,
                &theme.typography.caption,
                320.0,
                crate::layout::PhysicalTextAlignment::Start,
                "benchmark paragraph",
                false,
            )
            .unwrap();
        let elapsed = start.elapsed();
        *elapsed_sample = elapsed;
        let counters = measurements::finish().counters;
        let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
        let proposals = counters[Counter::TextProposalVisit as usize];
        let exact = counters[Counter::TextExactTrial as usize];
        let glyphs = plan
            .lines
            .iter()
            .flat_map(|line| &line.runs)
            .map(|run| run.glyphs.len())
            .sum::<usize>();
        assert!(glyphs > 0);
        assert_eq!(plan.source, source);
        assert!(plan.width() <= 320.0);
        if multiline {
            assert!(plan.lines.len() > 2);
        } else {
            assert_eq!(plan.lines.len(), 1);
        }
        if let Some(previous) = &previous {
            assert!(plan.shares_payload(previous));
            assert_eq!((proposals, exact, shapes), (0, 0, 0));
        } else {
            assert!(exact > 0 && shapes > 0);
            if multiline {
                assert!(proposals > 0);
                assert!(
                    exact > u64::try_from(plan.lines.len()).unwrap() + 2,
                    "exact trials include rejected extensions, not just retained lines"
                );
            } else {
                assert_eq!(proposals, 0);
            }
        }
        println!(
            "paragraph_local,multiline={multiline},repeat={repeat},{elapsed:?},proposals={proposals},exact={exact},shapes={shapes},lines={},retained_glyphs={glyphs}",
            plan.lines.len()
        );
        previous = Some(plan);
    }
    elapsed_samples
}

#[test]
fn paragraph_search_contract() {
    paragraph_case(false);
    paragraph_case(true);
    measurements::start();
    assert_eq!(measurements::finish().counters, [0; Counter::ALL.len()]);
}

#[test]
fn specialized_text_search_contract() {
    heatmap_case(false, true, false);
    heatmap_case(false, false, false);
    heatmap_case(true, false, false);
    heatmap_case(true, true, false);
    heatmap_case(true, false, true);
    heatmap_case(true, true, true);
    matrix_case(false, 1200);
    matrix_case(true, 1200);
    let short = matrix_case(false, 600);
    let long = matrix_case(true, 600);
    assert!(
        long.observations.counters[Counter::MatrixLayoutPass as usize]
            > short.observations.counters[Counter::MatrixLayoutPass as usize]
    );
}

#[test]
#[ignore = "release local search; run with --release --ignored --nocapture --test-threads=1"]
fn specialized_text_search_measurements() {
    assert!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "requires --release"
    );
    for long in [false, true] {
        measure_pair(
            if long {
                ["heatmap_long_wrap", "heatmap_long_elide"]
            } else {
                ["heatmap_short_wrap", "heatmap_short_elide"]
            },
            [&|| heatmap_case(long, true, false), &|| {
                heatmap_case(long, false, false)
            }],
        );
    }
    measure_pair(
        [
            "heatmap_shared_prefix_wrap",
            "heatmap_shared_prefix_elide_rejected",
        ],
        [&|| heatmap_case(true, true, true), &|| {
            heatmap_case(true, false, true)
        }],
    );
    for size in [1200, 600] {
        let samples = measure_pair(
            if size == 600 {
                ["matrix_tight_short", "matrix_tight_long"]
            } else {
                ["matrix_adequate_short", "matrix_adequate_long"]
            },
            [&|| matrix_case(false, size), &|| matrix_case(true, size)],
        );
        for [short, long] in samples {
            if size == 600 {
                assert!(
                    long.observations.counters[Counter::MatrixLayoutPass as usize]
                        > short.observations.counters[Counter::MatrixLayoutPass as usize]
                );
            }
        }
    }
}

#[test]
#[ignore = "release local search; run with --release --ignored --nocapture --test-threads=1"]
fn paragraph_search_measurements() {
    assert!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "requires --release"
    );
    println!(
        "conditions,paragraph,320px,font_setup_excluded,three_runs,two_calls_per_theme,OS_cache_uncontrolled"
    );
    let samples: [[[Duration; 2]; 2]; 3] = std::array::from_fn(|run| {
        println!("paragraph_run,{run}");
        if run % 2 == 0 {
            let multiline = paragraph_case(true);
            [paragraph_case(false), multiline]
        } else {
            [paragraph_case(false), paragraph_case(true)]
        }
    });
    for (index, name) in ["paragraph_short", "paragraph_multiline"]
        .into_iter()
        .enumerate()
    {
        summarize_local(name, samples.map(|sample| sample[index]));
    }
}
