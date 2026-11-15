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
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    crate::orchestrator::with_prepared_plot_for_test(
        &plot,
        &theme,
        |_| Ok(()),
        |source, root, theme, _operation, _round| {
            let panel = crate::layout::PixelRect::new(0.0, 800.0, 0.0, 600.0);
            let mut expected: Option<((f64, f64), [u64; Counter::ALL.len()])> = None;
            let mut elapsed_samples = [Duration::ZERO; 2];
            for (repeat, elapsed_sample) in elapsed_samples.iter_mut().enumerate() {
                crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
                measurements::start();
                let start = Instant::now();
                let plan = crate::significance::plan(
                    &root.plots[0],
                    source.significance_annotations().iter(),
                    &categories,
                    &vec![10.0; categories.len()],
                    (0.0, 10.0),
                    panel,
                    theme,
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
                    assert!(
                        bracket.geometry.x_left >= panel.x0 && bracket.geometry.x_right <= panel.x1
                    );
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
        },
    )
    .unwrap()
}

#[test]
fn significance_search_contract() {
    significance_case(4);
    measurements::start();
    assert_eq!(measurements::finish().counters, [0; Counter::ALL.len()]);
}
