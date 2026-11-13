use super::*;
use crate::numeric::source::{axis_request_for_test, with_axis_binding_for_test};
use crate::CoordinateAxis;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

#[test]
fn warning_views_materialize_once_per_consumed_span_and_keep_complete_policy_identity() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let seen = calls.clone();
    let generation = crate::TickGeneration::function(move |context| {
        assert_eq!(context.axis(), CoordinateAxis::X);
        let (lower, upper) = context.domain().data_limits().unwrap();
        seen.lock()
            .unwrap()
            .push((lower.to_bits(), upper.to_bits(), context.span().to_bits()));
        let mut values = vec![lower];
        if lower <= 50.0 && 50.0 <= upper {
            values.push(50.0);
        }
        values.push(upper);
        Ok(values.into())
    });
    assert_eq!(generation, generation.clone());
    assert_ne!(
        generation,
        crate::TickGeneration::function(|_| Ok(Vec::new().into()))
    );
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    with_axis_binding_for_test(|binding| {
        for (view, expected) in [
            ((0.0, 120.0), vec!["0", "50", "120"]),
            ((20.0, 80.0), vec!["20", "50", "80"]),
            ((60.0, 120.0), vec!["60", "120"]),
        ] {
            for height in [300.0, 600.0] {
                let projection = Projection::new(view, (0.0, 1.0), (0.0, 900.0, 0.0, height));
                let request = axis_request_for_test(
                    binding,
                    None,
                    &generation,
                    &projection,
                    AxisSide::Bottom,
                    false,
                );
                let selected = plan(
                    view,
                    false,
                    &projection,
                    AxisSide::Bottom,
                    &theme,
                    Some(&request),
                    &FitScope::root(),
                )
                .unwrap();
                assert_eq!(
                    selected
                        .ticks
                        .iter()
                        .map(|tick| tick.label.as_str())
                        .collect::<Vec<_>>(),
                    expected
                );
            }
        }
        assert_eq!(calls.lock().unwrap().len(), 3);
        let projection = Projection::new((0.0, 120.0), (0.0, 1.0), (0.0, 1000.0, 0.0, 300.0));
        let request = axis_request_for_test(
            binding,
            None,
            &generation,
            &projection,
            AxisSide::Bottom,
            false,
        );
        plan(
            (0.0, 120.0),
            false,
            &projection,
            AxisSide::Bottom,
            &theme,
            Some(&request),
            &FitScope::root(),
        )
        .unwrap();
        assert_eq!(calls.lock().unwrap().len(), 4);
        request.binding.operation.seal_fonts().unwrap();
        assert!(plan(
            (0.0, 120.0),
            false,
            &projection,
            AxisSide::Bottom,
            &theme,
            Some(&request),
            &FitScope::root()
        )
        .is_err());
    });
}

#[test]
fn filtering_keeps_unsorted_duplicates_original_ids_and_raw_or_exponent_operands() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    let indices = Arc::new(Mutex::new(Vec::new()));
    let seen = indices.clone();
    let policy = crate::NumericFormat::default().set_function(move |context| {
        let values = context
            .values()
            .iter()
            .map(|value| {
                let crate::NumericIdentity::Tick { source_index, .. } = value.identity() else {
                    panic!("tick identity")
                };
                source_index
            })
            .collect::<Vec<_>>();
        seen.lock().unwrap().push(values);
        context.format_builtin()
    });
    with_axis_binding_for_test(|binding| {
        let generation =
            crate::TickGeneration::function(|_| Ok(vec![-10.0, 80.0, 20.0, 20.0, 120.0].into()));
        let view = (0.0, 100.0);
        let projection = Projection::new(view, (0.0, 1.0), (0.0, 900.0, 0.0, 300.0));
        let request = axis_request_for_test(
            binding,
            Some(&policy),
            &generation,
            &projection,
            AxisSide::Bottom,
            false,
        );
        let selected = plan(
            view,
            false,
            &projection,
            AxisSide::Bottom,
            &theme,
            Some(&request),
            &FitScope::root(),
        )
        .unwrap();
        assert_eq!(
            selected
                .ticks
                .iter()
                .map(|tick| tick.position.to_bits())
                .collect::<Vec<_>>(),
            [80.0_f64, 20.0, 20.0].map(f64::to_bits)
        );
        assert_eq!(*indices.lock().unwrap(), [vec![1, 2, 3]]);
        for (generation, power, expected) in [
            (
                crate::TickGeneration::function(|_| Ok(vec![1.0, 10.0, 100.0].into())),
                false,
                vec![1.0_f64, 10.0, 100.0],
            ),
            (
                crate::TickGeneration::function(|_| {
                    Ok(TickPositions::Log10Exponents(vec![0.0, 1.0, 400.0]))
                }),
                true,
                vec![0.0_f64, 1.0, 400.0],
            ),
        ] {
            let view = if power { (0.0, 400.0) } else { (0.0, 2.0) };
            let projection = Projection::new(view, (0.0, 1.0), (0.0, 120_000.0, 0.0, 300.0));
            let request = axis_request_for_test(
                binding,
                None,
                &generation,
                &projection,
                AxisSide::Bottom,
                true,
            );
            let selected = plan(
                view,
                true,
                &projection,
                AxisSide::Bottom,
                &theme,
                Some(&request),
                &FitScope::root(),
            )
            .unwrap();
            assert_eq!(
                selected.numeric.unwrap().members,
                expected
                    .iter()
                    .map(|value| Operand::number(*value, power))
                    .collect::<Vec<_>>()
            );
        }
    });
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one operation observes empty, capped and failed source refusal with its actual borrowed axis requests"
)]
fn empty_large_capped_and_failed_operations_do_not_retry_or_repair_the_locator() {
    let theme = crate::Theme::publication().resolve().unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let policy = crate::NumericFormat::default()
        .set_function(|_| panic!("empty or capped text is not consumed"));
    with_axis_binding_for_test(|binding| {
        let generation = crate::TickGeneration::function(move |_| {
            seen.fetch_add(1, Ordering::SeqCst);
            Ok(vec![-1.0, 101.0].into())
        });
        let view = (0.0, 100.0);
        let projection = Projection::new(view, (0.0, 1.0), (0.0, 900.0, 0.0, 300.0));
        let request = axis_request_for_test(
            binding,
            Some(&policy),
            &generation,
            &projection,
            AxisSide::Bottom,
            false,
        );
        assert!(plan(
            view,
            false,
            &projection,
            AxisSide::Bottom,
            &theme,
            Some(&request),
            &FitScope::root()
        )
        .unwrap()
        .ticks
        .is_empty());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let generation = crate::TickGeneration::function(|_| Ok(vec![20.0; 10_001].into()))
            .maximum_count(Some(10_000))
            .unwrap();
        let request = axis_request_for_test(
            binding,
            Some(&policy),
            &generation,
            &projection,
            AxisSide::Bottom,
            false,
        );
        assert!(plan(
            view,
            false,
            &projection,
            AxisSide::Bottom,
            &theme,
            Some(&request),
            &FitScope::root()
        )
        .is_err());
        let generation = generation.maximum_count(None).unwrap();
        let request = axis_request_for_test(
            binding,
            None,
            &generation,
            &projection,
            AxisSide::Bottom,
            false,
        );
        assert_eq!(
            plan(
                view,
                false,
                &projection,
                AxisSide::Bottom,
                &theme,
                Some(&request),
                &FitScope::root()
            )
            .unwrap()
            .ticks
            .len(),
            10_001
        );
        for generation in [
            crate::TickGeneration::function(|_| Err(Error::Data("application failure".into()))),
            crate::TickGeneration::function(|_| Ok(vec![f64::NAN].into())),
            crate::TickGeneration::function(|_| Ok(TickPositions::Log10Exponents(vec![1.0]))),
        ] {
            let request = axis_request_for_test(
                binding,
                None,
                &generation,
                &projection,
                AxisSide::Bottom,
                false,
            );
            assert!(plan(
                view,
                false,
                &projection,
                AxisSide::Bottom,
                &theme,
                Some(&request),
                &FitScope::root()
            )
            .is_err());
        }
    });
}
