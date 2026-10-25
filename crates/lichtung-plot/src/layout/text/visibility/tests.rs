use super::*;
use crate::LabelEndpoints;

#[test]
fn original_index_selection_and_geometric_greedy_keep_protected_endpoints() {
    let source = [(3, 50.0), (0, 0.0), (2, 10.0), (1, 0.0), (4, 100.0)];
    let candidates = source.map(|(index, position)| Candidate {
        index,
        position,
        bounds: PanelBox::new(position - 5.0, position + 5.0, 0.0, 10.0),
    });
    let eligible = [0, 1, 2, 3, 4];
    let region = PanelBox::new(0.0, 100.0, 0.0, 10.0);
    let scope = FitScope::root();
    let every = TextVisibility::every(3, 1, LabelEndpoints::Both).unwrap();
    assert_eq!(
        reduce(
            &every,
            &eligible,
            &candidates,
            region,
            TextFailure::Visible,
            &scope
        )
        .unwrap(),
        [false, true, false, true, true]
    );
    let greedy = TextVisibility::greedy(LabelEndpoints::Both);
    assert_eq!(
        reduce(
            &greedy,
            &eligible,
            &candidates,
            region,
            TextFailure::Error,
            &scope
        )
        .unwrap(),
        [true, true, true, false, true]
    );
    let bounded = greedy.boundary_tolerance(Some(0.0)).unwrap();
    assert_eq!(
        reduce(
            &bounded,
            &eligible,
            &candidates,
            region,
            TextFailure::Error,
            &scope
        )
        .unwrap(),
        [true, false, true, false, false]
    );
}

#[test]
fn parity_uses_extra_separation_and_never_silently_unprotects_a_conflict() {
    let candidates: [Candidate; 6] = std::array::from_fn(|index| {
        let position = f64::from(u32::try_from(index).unwrap()) * 10.0;
        Candidate {
            index,
            position,
            bounds: PanelBox::new(position, position + 15.0, 0.0, 10.0),
        }
    });
    let eligible = [0, 1, 2, 3, 4, 5];
    let region = PanelBox::new(0.0, 65.0, 0.0, 10.0);
    let policy = TextVisibility::parity(LabelEndpoints::Both)
        .separation(6.0)
        .unwrap();
    assert_eq!(
        reduce(
            &policy,
            &eligible,
            &candidates,
            region,
            TextFailure::Error,
            &FitScope::root()
        )
        .unwrap(),
        [true, false, false, false, false, true]
    );
    let impossible = policy.separation(40.0).unwrap();
    assert!(reduce(
        &impossible,
        &eligible,
        &candidates,
        region,
        TextFailure::Error,
        &FitScope::root()
    )
    .is_err());
    assert_eq!(
        reduce(
            &impossible,
            &eligible,
            &candidates,
            region,
            TextFailure::Hide,
            &FitScope::root()
        )
        .unwrap(),
        [false; 6]
    );
    assert!(TextVisibility::all().separation(1.0).is_err());
    assert!(TextVisibility::every(0, 0, LabelEndpoints::None).is_err());
}
