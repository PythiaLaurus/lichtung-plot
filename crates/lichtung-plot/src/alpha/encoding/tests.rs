use super::*;

fn values(bytes: impl IntoIterator<Item = u8>) -> AlphaValues {
    AlphaValues::new("confidence", bytes).unwrap()
}

#[test]
fn keyed_alpha_canonicalizes_field_order_but_preserves_raw_tuple_value_associations() {
    let group = CategoryValues::new("group", ["A  B", "A B", "unused"]).unwrap();
    let facet = CategoryValues::new("experiment", ["left", "right", "pooled"]).unwrap();
    let encoded =
        AlphaEncoding::keyed([group.clone(), facet.clone()], values([0, 230, 17])).unwrap();
    assert_eq!(
        encoded,
        AlphaEncoding::keyed([facet, group], values([0, 230, 17])).unwrap()
    );
    let prepared = encoded
        .prepare(AlphaUnitKind::Series, 100, &["group"], &["experiment"])
        .unwrap();
    assert_eq!(
        prepared
            .keyed(&[("group", "A B"), ("experiment", "right")])
            .unwrap(),
        Some(ResolvedAlpha {
            index: 1,
            alpha: 230
        })
    );
    assert_eq!(
        prepared
            .keyed(&[("experiment", "left"), ("group", "A  B"), ("group", "A  B")])
            .unwrap(),
        Some(ResolvedAlpha { index: 0, alpha: 0 })
    );
    assert!(matches!(
        prepared.keyed(&[("experiment", "left"), ("group", "A B")]),
        Err(Error::Data(_))
    ));
    assert!(prepared
        .keyed(&[("experiment", "left"), ("group", "A  B"), ("group", "A B")])
        .is_err());
    assert!(prepared.keyed(&[("group", "A B")]).is_err());
    assert!(encoded
        .prepare(
            AlphaUnitKind::Series,
            100,
            &["unknown"],
            &["experiment", "group"]
        )
        .is_err());
    assert!(encoded
        .prepare(AlphaUnitKind::Aggregate, 100, &["group"], &[])
        .is_err());
    assert!(AlphaEncoding::keyed([], values([])).is_err());
    assert!(AlphaEncoding::keyed(
        [CategoryValues::new("group", ["A", "A"]).unwrap()],
        values([0, 255])
    )
    .is_err());
    assert!(AlphaEncoding::keyed(
        [
            CategoryValues::new("group", ["A"]).unwrap(),
            CategoryValues::new("group", ["B"]).unwrap()
        ],
        values([17])
    )
    .is_err());
    assert!(AlphaEncoding::keyed(
        [CategoryValues::nullable("group", [Some("A"), None]).unwrap()],
        values([0, 255])
    )
    .is_err());
    assert!(
        AlphaEncoding::keyed([CategoryValues::new("group", ["A"]).unwrap()], values([])).is_err()
    );
    let empty = AlphaEncoding::keyed(
        [CategoryValues::new("group", std::iter::empty::<&str>()).unwrap()],
        values([]),
    )
    .unwrap();
    let empty = empty
        .prepare(AlphaUnitKind::Aggregate, 100, &["group"], &[])
        .unwrap();
    assert!(empty.keyed(&[("group", "A")]).is_err());
    let unicode = AlphaEncoding::keyed(
        [CategoryValues::new("group", ["é", "e\u{301}"]).unwrap()],
        values([90, 170]),
    )
    .unwrap()
    .prepare(AlphaUnitKind::Aggregate, 100, &["group"], &[])
    .unwrap();
    assert_eq!(unicode.keyed(&[("group", "é")]).unwrap().unwrap().alpha, 90);
    assert_eq!(
        unicode
            .keyed(&[("group", "e\u{301}")])
            .unwrap()
            .unwrap()
            .alpha,
        170
    );
}
