use crate::font::canonical::{
    FontFamily, FontPairPreparationRequest, FontStack, PreparedFontEnvironment, RequestedWeight,
    ScalarFaceSelection, FONT_DISCOVERY_CALLS, FONT_PAIR_RESOLUTION_CALLS,
};
use crate::orchestrator::measurements::{self, Counter};
use std::time::{Duration, Instant};

fn observe<T>(name: &str, run: usize, action: impl FnOnce() -> T) -> (T, [u64; 2], Duration) {
    measurements::start();
    let start = Instant::now();
    let result = action();
    let elapsed = start.elapsed();
    let observation = measurements::finish();
    let probes = [
        observation.counters[Counter::FontScalarProbe as usize],
        observation.counters[Counter::FontUnitProbe as usize],
    ];
    assert_eq!(
        observation.counters.iter().sum::<u64>(),
        probes.iter().sum::<u64>()
    );
    println!(
        "font_sample,{name},{run},{:.6},scalar_probes={},unit_probes={}",
        elapsed.as_secs_f64() * 1000.0,
        probes[0],
        probes[1]
    );
    (result, probes, elapsed)
}

#[expect(
    clippy::too_many_lines,
    reason = "keep ordered scalar and unit cache phases with their fixture assertions"
)]
fn synthetic_case(candidates: usize, characters: u8, split: bool, run: usize) -> Duration {
    let name = format!("synthetic_c{candidates}_b{characters}_split{split}");
    let bases = (b'A'..b'A' + characters)
        .map(char::from)
        .collect::<Vec<_>>();
    let mut coverage = bases.clone();
    coverage.push('\u{301}');
    let names = (0..candidates)
        .map(|i| format!("Probe{i}"))
        .collect::<Vec<_>>();
    let faces = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let scalars: &[char] = if i == candidates - 1 {
                if split {
                    &['\u{301}']
                } else {
                    &coverage
                }
            } else if split && i == candidates - 2 {
                &bases
            } else {
                &['0']
            };
            (name.as_str(), scalars)
        })
        .collect::<Vec<_>>();
    let (environment, probes, mut total) = observe(&format!("{name}/setup"), run, || {
        PreparedFontEnvironment::synthetic_faces(&faces)
    });
    assert_eq!(probes, [0, 0]);
    let stack = FontStack::new(
        names
            .into_iter()
            .map(|name| FontFamily::named(name).unwrap()),
    )
    .unwrap();
    FONT_PAIR_RESOLUTION_CALLS.with(|calls| calls.set(0));
    let half = bases.len() / 2;
    for (phase, scalars, misses) in [
        ("scalar_first", &bases[..half], half),
        ("scalar_new", &bases[half..], bases.len() - half),
        ("scalar_cache_hit", bases.as_slice(), 0),
    ] {
        let (pair, probes, elapsed) = observe(&format!("{name}/{phase}"), run, || {
            environment
                .resolve_pair(0, &stack, RequestedWeight::Normal, scalars)
                .unwrap()
        });
        total += elapsed;
        assert_eq!(pair.provenance.candidates.len(), candidates);
        let selected = pair.provenance.candidates[candidates - 1 - usize::from(split)];
        assert!(pair
            .selected_scalars
            .iter()
            .all(|entry| entry.selection == ScalarFaceSelection::Face(selected)));
        // Scalar preparation probes the full candidate matrix before selecting first coverage.
        assert_eq!(probes, [(misses * candidates) as u64, 0]);
        assert_eq!(FONT_PAIR_RESOLUTION_CALLS.with(std::cell::Cell::get), 1);
        println!("font_coverage,{name},{run},{phase},candidates={candidates},requested_scalars={},cached_scalars={}", scalars.len(), scalars.len() - misses);
    }
    // Exercise the production batch-preparation path independently of incremental lookup.
    let ((), probes, elapsed) = observe(&format!("{name}/batch_prepare"), run, || {
        environment
            .prepare_pairs(
                [FontPairPreparationRequest::new(
                    0,
                    &stack,
                    RequestedWeight::Normal,
                    &coverage,
                )]
                .into_iter(),
                None,
            )
            .unwrap();
    });
    total += elapsed;
    assert_eq!(probes, [((bases.len() + 1) * candidates) as u64, 0]);
    let source = bases
        .iter()
        .flat_map(|base| [*base, '\u{301}'])
        .collect::<String>();
    for (phase, expected) in [
        ("unit_first", bases.len() * (candidates + 1)),
        ("unit_cache_hit", 0),
    ] {
        let ((), probes, elapsed) = observe(&format!("{name}/{phase}"), run, || {
            environment.prepare_source_units(0, &source, None).unwrap();
        });
        total += elapsed;
        assert_eq!(probes, [0, expected as u64]);
        let pair = environment
            .prepared_pair(&stack, RequestedWeight::Normal)
            .unwrap()
            .unwrap();
        assert_eq!(pair.selected_units.len(), bases.len());
        for unit in &pair.selected_units {
            assert_eq!(
                unit.face,
                (!split).then_some(pair.provenance.candidates[candidates - 1])
            );
            for scalar in unit.source.chars() {
                let index = candidates - 1 - usize::from(split && scalar != '\u{301}');
                assert_eq!(
                    pair.selection_for_unit(&unit.source, scalar).unwrap(),
                    ScalarFaceSelection::Face(pair.provenance.candidates[index])
                );
            }
        }
        println!(
            "font_coverage,{name},{run},{phase},candidates={candidates},units={},cached_units={}",
            bases.len(),
            if expected == 0 { bases.len() } else { 0 }
        );
    }
    total
}

#[test]
fn font_fixture_paths() {
    synthetic_case(2, 2, false, 0);
    synthetic_case(2, 2, true, 0);
}

#[test]
#[ignore = "release font coverage; run with --release --ignored --nocapture --test-threads=1"]
fn font_synthetic_coverage_measurements() {
    assert!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "requires --release"
    );
    println!("font_conditions,synthetic_SFNT,three_runs,no_warmup,instrumented,setup_separate,no_global_cache,OS_scheduling_uncontrolled");
    for candidates in [2, 8] {
        for characters in [2, 16] {
            for split in [false, true] {
                let mut times = (0..3)
                    .map(|run| synthetic_case(candidates, characters, split, run))
                    .collect::<Vec<_>>();
                times.sort_unstable();
                println!("font_summary,synthetic_c{candidates}_b{characters}_split{split},3,min_ms={:.6},median_ms={:.6},max_ms={:.6}", times[0].as_secs_f64() * 1000.0, times[1].as_secs_f64() * 1000.0, times[2].as_secs_f64() * 1000.0);
            }
        }
    }
}

#[test]
#[ignore = "release system fonts; run with --release --ignored --nocapture --test-threads=1"]
fn font_system_preparation_measurements() {
    assert!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "requires --release"
    );
    println!("font_conditions,system,three_fresh_environments,no_warmup,instrumented,OS_font_and_file_cache_uncontrolled,no_global_cache");
    let mut setup = Vec::new();
    for run in 0..3 {
        FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let (environment, probes, elapsed) = observe("system/discovery", run, || {
            PreparedFontEnvironment::discover_system().unwrap()
        });
        setup.push(elapsed);
        assert_eq!(probes, [0, 0]);
        assert_eq!(FONT_DISCOVERY_CALLS.with(std::cell::Cell::get), 1);
        let stack = crate::font::canonical::builtin_font_stack();
        let mut previous = None;
        for (phase, source) in [
            ("first", "Measured responses"),
            ("new", "Independent trials"),
            ("cache_hit", "Independent trials"),
        ] {
            let scalars = source.chars().collect::<Vec<_>>();
            let (pair, probes, _) = observe(&format!("system/{phase}"), run, || {
                environment
                    .resolve_pair(0, &stack, RequestedWeight::Normal, &scalars)
                    .unwrap()
            });
            assert!(!pair.provenance.candidates.is_empty());
            assert_eq!(probes[1], 0);
            if phase == "cache_hit" {
                assert_eq!(probes[0], 0);
                assert_eq!(previous.as_ref(), Some(&pair));
            } else {
                assert!(probes[0] > 0);
            }

            previous = Some(pair);
        }
        assert_eq!(FONT_DISCOVERY_CALLS.with(std::cell::Cell::get), 1);
    }
    setup.sort_unstable();
    println!(
        "font_summary,system_discovery,3,min_ms={:.6},median_ms={:.6},max_ms={:.6}",
        setup[0].as_secs_f64() * 1000.0,
        setup[1].as_secs_f64() * 1000.0,
        setup[2].as_secs_f64() * 1000.0
    );
}

#[test]
#[ignore = "release observer overhead; run with --release --ignored --nocapture --test-threads=1"]
#[expect(
    clippy::too_many_lines,
    reason = "keep paired timing boundaries and enabled/disabled equivalence assertions together"
)]
fn font_observer_overhead_measurements() {
    use crate::font::canonical::FONT_SELECTION_PROBES;

    assert!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "requires --release"
    );
    println!("font_overhead_conditions,six_pairs,32_fresh_operations_per_sample,alternating_order,no_warmup,setup_separate,observer_start_finish_excluded,test_hooks_and_legacy_counters_in_both_modes,not_production_zero_hook_overhead,OS_scheduling_uncontrolled");
    let mut coverage = ('A'..='P').collect::<Vec<_>>();
    let source = coverage
        .iter()
        .flat_map(|base| [*base, '\u{301}'])
        .collect::<String>();
    coverage.push('\u{301}');
    let names = (0..8).map(|i| format!("Probe{i}")).collect::<Vec<_>>();
    let faces = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let scalars: &[char] = if i == 7 { &coverage } else { &['0'] };
            (name.as_str(), scalars)
        })
        .collect::<Vec<_>>();
    let stack = FontStack::new(
        names
            .iter()
            .cloned()
            .map(|name| FontFamily::named(name).unwrap()),
    )
    .unwrap();
    let mut samples = [Vec::new(), Vec::new()];
    let mut deltas = Vec::new();
    let mut expected = None;
    // End a fresh empty session so the first inactive sample has explicit state.
    measurements::start();
    measurements::finish();
    for run in 0..6 {
        let mut pair = [Duration::ZERO; 2];
        for enabled in if run % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let mut setup = Duration::ZERO;
            let mut elapsed = Duration::ZERO;
            let mut observed = [0; 2];
            for _ in 0..32 {
                let start = Instant::now();
                let environment = PreparedFontEnvironment::synthetic_faces(&faces);
                setup += start.elapsed();
                FONT_SELECTION_PROBES.with(|calls| calls.set(0));
                if enabled {
                    measurements::start();
                }
                let start = Instant::now();
                environment
                    .prepare_pairs(
                        [FontPairPreparationRequest::new(
                            0,
                            &stack,
                            RequestedWeight::Normal,
                            &coverage,
                        )]
                        .into_iter(),
                        None,
                    )
                    .unwrap();
                environment.prepare_source_units(0, &source, None).unwrap();
                environment.prepare_source_units(0, &source, None).unwrap();
                let first = environment
                    .resolve_pair(0, &stack, RequestedWeight::Normal, &coverage)
                    .unwrap();
                let cached = environment
                    .resolve_pair(0, &stack, RequestedWeight::Normal, &coverage)
                    .unwrap();
                elapsed += start.elapsed();
                if enabled {
                    let observation = measurements::finish();
                    let scalar = observation.counters[Counter::FontScalarProbe as usize];
                    let unit = observation.counters[Counter::FontUnitProbe as usize];
                    assert_eq!([scalar, unit], [272, 144]);
                    assert_eq!(observation.counters.iter().sum::<u64>(), scalar + unit);
                    observed[0] += scalar;
                    observed[1] += unit;
                }
                assert_eq!(FONT_SELECTION_PROBES.with(std::cell::Cell::get), 416);
                assert_eq!(first, cached);
                let prepared = environment
                    .prepared_pair(&stack, RequestedWeight::Normal)
                    .unwrap()
                    .unwrap();
                assert_eq!(prepared.provenance.candidates.len(), 8);
                assert_eq!(prepared.selected_scalars.len(), 17);
                assert_eq!(prepared.selected_units.len(), 16);
                let selected = prepared.provenance.candidates[7];
                assert!(prepared
                    .selected_units
                    .iter()
                    .all(|unit| unit.face == Some(selected)));
                let outcome = (prepared, cached);
                if let Some(expected) = &expected {
                    assert_eq!(&outcome, expected);
                } else {
                    expected = Some(outcome);
                }
            }
            let index = usize::from(enabled);
            pair[index] = elapsed;
            samples[index].push(elapsed.as_secs_f64() * 1000.0);
            println!("font_overhead_sample,{run},enabled={enabled},operations=32,work_ms={:.6},setup_ms={:.6},observed_scalar_probes={},observed_unit_probes={},legacy_actual_probes=13312", elapsed.as_secs_f64() * 1000.0, setup.as_secs_f64() * 1000.0, observed[0], observed[1]);
        }
        let delta = (pair[1].as_secs_f64() - pair[0].as_secs_f64()) * 1000.0;
        deltas.push(delta);
        println!(
            "font_overhead_pair,{run},active_minus_inactive_ms={delta:.6},percent={:.3}",
            100.0 * delta / (pair[0].as_secs_f64() * 1000.0)
        );
    }
    for (name, mut values) in [
        ("inactive_work", samples[0].clone()),
        ("active_work", samples[1].clone()),
        ("paired_active_minus_inactive", deltas),
    ] {
        values.sort_by(f64::total_cmp);
        println!("font_overhead_summary,{name},samples=6,operations_per_sample=32,min_ms={:.6},median_ms={:.6},max_ms={:.6}", values[0], values[2].midpoint(values[3]), values[5]);
    }
}
