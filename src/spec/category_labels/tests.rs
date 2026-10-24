use super::*;

#[test]
fn stride_selection_keeps_original_indices_and_unions_endpoints_without_overflow() {
    let months = CategoryLabelSelection::every(2).unwrap();
    assert_eq!(months.indices(12).collect::<Vec<_>>(), [0, 2, 4, 6, 8, 10]);
    let shifted = CategoryLabelSelection::every(3)
        .unwrap()
        .offset(1)
        .unwrap()
        .endpoints(LabelEndpoints::Both);
    assert_eq!(
        shifted.indices(12).collect::<Vec<_>>(),
        [0, 1, 4, 7, 10, 11]
    );
    assert!(shifted.indices(0).next().is_none());
    assert_eq!(shifted.indices(1).collect::<Vec<_>>(), [0]);
    let empty = CategoryLabelSelection::every(10)
        .unwrap()
        .offset(9)
        .unwrap();
    assert_eq!(empty.indices(3).len(), 0);
    assert_eq!(
        empty
            .endpoints(LabelEndpoints::Last)
            .indices(3)
            .collect::<Vec<_>>(),
        [2]
    );
    let enormous = CategoryLabelSelection::every(usize::MAX)
        .unwrap()
        .offset(usize::MAX - 1)
        .unwrap()
        .endpoints(LabelEndpoints::Both);
    assert_eq!(
        enormous.indices(usize::MAX).collect::<Vec<_>>(),
        [0, usize::MAX - 1]
    );
    assert_eq!(
        CategoryLabelSelection::every(1)
            .unwrap()
            .endpoints(LabelEndpoints::Both),
        CategoryLabelSelection::all()
    );
    assert!(CategoryLabelSelection::every(0).is_err());
    assert!(CategoryLabelSelection::all().offset(1).is_err());
    assert!(months.offset(2).is_err());
    for count in [0, 1, 2, 12] {
        let indices = shifted.indices(count).collect::<Vec<_>>();
        for index in 0..count {
            assert_eq!(shifted.includes(index, count), indices.contains(&index));
        }
    }
}
