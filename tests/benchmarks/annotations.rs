use crate::orchestrator::measurements::{self, Counter};
use crate::{
    BarInput, CategoryValues, NumericValues, Plot, Significance, SignificanceMark,
    SignificanceTarget,
};
use std::time::{Duration, Instant};

fn significance_case(count: usize) -> [Duration; 2] {
    let categories = (0..=count)
        .map(|index| format!("Group {index}"))
        .collect::<Vec<_>>();
    let plot = Plot::bar(
        CategoryValues::new("Group", categories.clone()).unwrap(),
        BarInput::estimates(NumericValues::new("Response", vec![10.0; categories.len()])),
    )
    .significance((1..=count).map(|index| {
        Significance::between(
            SignificanceTarget::category(&categories[0]),
            SignificanceTarget::category(&categories[index]),
            SignificanceMark::TwoStars,
        )
    }))
    .erase();
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    theme.prepare_font_environment().unwrap();
    let panel = crate::layout::PanelBox::new(0.0, 800.0, 0.0, 600.0);
    let mut expected: Option<((f64, f64), [u64; Counter::ALL.len()])> = None;
    let mut elapsed_samples = [Duration::ZERO; 2];
    for (repeat, elapsed_sample) in elapsed_samples.iter_mut().enumerate() {
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        measurements::start();
        let start = Instant::now();
        let plan = crate::significance::plan(
            &plot,
            &categories,
            &vec![10.0; categories.len()],
            (0.0, 10.0),
            panel,
            &theme,
            &crate::layout::fit::ROOT_SCOPE,
        )
        .unwrap();
        let elapsed = start.elapsed();
        *elapsed_sample = elapsed;
        let observation = measurements::finish();
        let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
        let counters = observation.counters;
        assert!(counters[Counter::SignificanceRangeGrowth as usize] > 0);
        assert_eq!(counters[Counter::SignificanceBisection as usize], 64);
        assert!(counters[Counter::SignificanceLevelCheck as usize] >= 66);
        assert_eq!(plan.brackets.len(), count);
        assert!(plan.y_range.1 > 10.0);
        assert!(plan
            .brackets
            .windows(2)
            .all(|pair| pair[0].geometry.y < pair[1].geometry.y));
        for bracket in &plan.brackets {
            assert!(bracket.geometry.x_left >= panel.x0 && bracket.geometry.x_right <= panel.x1);
            assert!(bracket.geometry.y > panel.y0);
            assert!(bracket.geometry.y + bracket.label.plan.height() <= panel.y1);
        }
        if let Some((range, previous)) = expected {
            assert_eq!(plan.y_range, range);
            for counter in [
                Counter::SignificanceRangeGrowth,
                Counter::SignificanceBisection,
                Counter::SignificanceLevelCheck,
            ] {
                assert_eq!(counters[counter as usize], previous[counter as usize]);
            }
            assert_eq!(shapes, 0, "repeated labels must use the theme cache");
        } else {
            assert!(shapes > 0);
            expected = Some((plan.y_range, counters));
        }
        println!(
            "significance_local,{count},{repeat},{elapsed:?},growth={},bisections={},level_checks={},shapes={shapes},range={:?}",
            counters[Counter::SignificanceRangeGrowth as usize],
            counters[Counter::SignificanceBisection as usize],
            counters[Counter::SignificanceLevelCheck as usize],
            plan.y_range
        );
    }
    elapsed_samples
}

pub(super) fn summarize_local(name: &str, samples: [[Duration; 2]; 3]) {
    for (repeat, phase) in ["first", "cached_repeat"].into_iter().enumerate() {
        let mut values = samples.map(|sample| sample[repeat]);
        values.sort_unstable();
        println!(
            "local_summary,{name},{phase},samples={},min_ms={:.6},median_ms={:.6},max_ms={:.6}",
            values.len(),
            values[0].as_secs_f64() * 1000.0,
            values[1].as_secs_f64() * 1000.0,
            values[2].as_secs_f64() * 1000.0,
        );
    }
}

#[test]
fn significance_search_contract() {
    significance_case(4);
    measurements::start();
    assert_eq!(measurements::finish().counters, [0; Counter::ALL.len()]);
}

#[test]
#[ignore = "release local search; run with --release --ignored --nocapture --test-threads=1"]
fn significance_search_measurements() {
    assert!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "requires --release"
    );
    println!(
        "conditions,significance,direct_planner,800x600_panel,font_setup_excluded,three_runs,two_calls_per_theme,OS_cache_uncontrolled"
    );
    let samples: [[[Duration; 2]; 3]; 3] = std::array::from_fn(|run| {
        println!("significance_run,{run}");
        [1, 4, 8].map(significance_case)
    });
    for (index, count) in [1, 4, 8].into_iter().enumerate() {
        summarize_local(
            &format!("significance_{count}"),
            samples.map(|sample| sample[index]),
        );
    }
}
