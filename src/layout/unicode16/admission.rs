use super::{linebreaks, reserve, TextPlanError};

#[test]
fn input_and_storage_refusals_precede_processing_and_pushes() {
    let mut calls = 0;
    assert!(matches!(
        linebreaks("a b", &mut |_, _| {
            calls += 1;
            Err(TextPlanError::Allocation("injected input refusal"))
        }),
        Err(TextPlanError::Allocation("injected input refusal"))
    ));
    assert_eq!(calls, 1);
    let mut values = Vec::<usize>::new();
    assert!(matches!(
        reserve(&mut values, 1, &mut |_, _| Err(TextPlanError::Allocation(
            "injected storage refusal"
        ))),
        Err(TextPlanError::Allocation("injected storage refusal"))
    ));
    assert_eq!(values.capacity(), 0);
    assert!(values.is_empty());
    assert!(matches!(
        reserve(&mut values, usize::MAX, &mut |_, _| panic!(
            "arithmetic must precede reservation"
        )),
        Err(TextPlanError::Arithmetic(_))
    ));
}

#[test]
fn linebreak_work_and_storage_boundaries_are_exact() {
    let input = "\u{4E00}\u{200D}\u{0308}\u{4E01} 1}2";
    let (mut bytes, mut work) = (0_u64, 0_u64);
    let expected = linebreaks(input, &mut |b, w| {
        bytes += b;
        work += w;
        Ok(())
    })
    .unwrap();
    for (byte_limit, work_limit, succeeds) in [
        (bytes, work, true),
        (bytes - 1, work, false),
        (bytes, work - 1, false),
    ] {
        let (mut used_bytes, mut used_work) = (0_u64, 0_u64);
        let result = linebreaks(input, &mut |b, w| {
            used_bytes += b;
            used_work += w;
            if used_bytes > byte_limit || used_work > work_limit {
                Err(TextPlanError::Limit {
                    resource: "injected Unicode budget",
                    limit: u128::from(work_limit),
                    cumulative_claim: u128::from(used_work),
                })
            } else {
                Ok(())
            }
        });
        if succeeds {
            assert_eq!(result.unwrap(), expected);
        } else {
            assert!(matches!(result, Err(TextPlanError::Limit { .. })));
        }
    }
}

#[test]
fn long_backward_contexts_have_linear_admitted_work() {
    for unit in [" ", ",", "\u{1F1E6}"] {
        let mut counts = Vec::new();
        for count in [500, 1000] {
            let input = format!("1{}2", unit.repeat(count));
            let resources = crate::font::canonical::OperationFontResources::new();
            let mut admitted_work = 0;
            linebreaks(&input, &mut |bytes, work| {
                resources.claim_bytes(bytes)?;
                resources.claim_visits(work)?;
                admitted_work += work;
                Ok(())
            })
            .unwrap();
            counts.push(admitted_work);
        }
        assert!(counts[1] < counts[0] * 3, "context {unit:?}: {counts:?}");
    }
}

#[test]
fn visits_count_scalar_passes_not_bytes_or_fixed_rules() {
    for input in ["abc", "一二三"] {
        let mut visits = 0;
        linebreaks(input, &mut |_, count| {
            visits += count;
            Ok(())
        })
        .unwrap();
        assert_eq!(visits, 3 + 3 + 2);
    }
}
