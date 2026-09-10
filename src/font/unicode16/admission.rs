use super::{append, nfkc_casefold, reserve, FontError};
use crate::font::canonical::OperationFontResources;

#[test]
fn admission_precedes_input_work_and_first_allocation() {
    let mut calls = 0;
    let error = nfkc_casefold("A", &mut |_, _| {
        calls += 1;
        Err(FontError::Allocation("injected input refusal"))
    })
    .unwrap_err();
    assert!(matches!(
        error,
        FontError::Allocation("injected input refusal")
    ));
    assert_eq!(calls, 1);

    let mut output = String::new();
    let error = append(&mut output, "expanded", &mut |bytes, _| {
        if bytes == 0 {
            Ok(())
        } else {
            Err(FontError::Allocation("injected storage refusal"))
        }
    })
    .unwrap_err();
    assert!(matches!(
        error,
        FontError::Allocation("injected storage refusal")
    ));
    assert_eq!(output.capacity(), 0);
    assert!(output.is_empty());
    let mut scalars = Vec::<char>::new();
    assert!(
        reserve(&mut scalars, 1, &mut |_, _| Err(FontError::Allocation(
            "injected"
        )))
        .is_err()
    );
    assert_eq!(scalars.capacity(), 0);
}

#[test]
fn expansion_and_every_intermediate_share_exact_admission() {
    // U+FDFA expands to 18 scalars; a decomposed ring also exercises final NFC.
    let input = "\u{FDFA}A\u{030A}";
    let mut bytes = 0;
    let mut work = 0;
    let expected = nfkc_casefold(input, &mut |b, w| {
        bytes += b;
        work += w;
        Ok(())
    })
    .unwrap();
    assert!(bytes > input.len() as u64);
    assert!(work > input.len() as u64);
    for (byte_limit, work_limit, succeeds) in [
        (bytes, work, true),
        (bytes - 1, work, false),
        (bytes, work - 1, false),
    ] {
        let (mut used_bytes, mut used_work) = (0_u64, 0_u64);
        let result = nfkc_casefold(input, &mut |b, w| {
            used_bytes += b;
            used_work += w;
            if used_bytes > byte_limit || used_work > work_limit {
                Err(FontError::Limit {
                    resource: "injected Unicode budget",
                    limit: work_limit,
                })
            } else {
                Ok(())
            }
        });
        if succeeds {
            assert_eq!(result.unwrap(), expected);
        } else {
            assert!(matches!(result, Err(FontError::Limit { .. })));
        }
    }
}

#[test]
fn normalization_uses_the_existing_root_without_resetting_it() {
    let resources = OperationFontResources::new();
    resources.claim_visits(999_999).unwrap();
    let shared = resources.clone();
    assert!(matches!(
        nfkc_casefold("A", &mut |bytes, work| {
            shared.claim_bytes(bytes)?;
            shared.claim_visits(work)
        }),
        Err(FontError::Limit { .. })
    ));
    assert!(resources.claim_visits(1).is_err());
}

#[test]
fn adversarial_mark_ordering_refuses_before_quadratic_work_finishes() {
    // Both marks survive casefold: all class-232 marks must cross class-230
    // marks. Input/decomposition fits; the ordering comparisons exhaust work.
    let input = format!("{}{}", "\u{0315}".repeat(1000), "\u{0300}".repeat(1000));
    let resources = OperationFontResources::new();
    assert!(matches!(
        nfkc_casefold(&input, &mut |bytes, work| {
            resources.claim_bytes(bytes)?;
            resources.claim_visits(work)
        }),
        Err(FontError::Limit {
            resource: "root processing visits",
            ..
        })
    ));
}

#[test]
fn checked_unicode_lengths_refuse_without_reservation() {
    let mut scalars = Vec::<char>::new();
    assert!(matches!(
        reserve(&mut scalars, usize::MAX, &mut |_, _| panic!(
            "must reject arithmetic first"
        )),
        Err(FontError::Arithmetic(_))
    ));
    assert_eq!(scalars.capacity(), 0);
}
