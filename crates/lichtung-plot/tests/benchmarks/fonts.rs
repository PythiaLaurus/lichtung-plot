use crate::font::canonical::{
    FontFamily, FontPairPreparationRequest, FontStack, PreparedFontEnvironment, RequestedWeight,
};
use crate::orchestrator::measurements::{self, Counter};
use std::time::{Duration, Instant};

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
