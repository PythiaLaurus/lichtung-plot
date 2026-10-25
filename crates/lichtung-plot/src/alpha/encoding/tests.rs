use super::*;
use crate::alpha::{AlphaInputs, AlphaTarget};

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

#[test]
fn bin_alpha_requires_exact_intervals_and_owning_facet_keys_without_edge_adjustment() {
    let bounds = || {
        Bounds::new(
            crate::NumericValues::new("lower", [-0.0, 0.0, 1.0]),
            crate::NumericValues::new("upper", [1.0, 1.0, 2.0]),
        )
    };
    let encoded = AlphaEncoding::bins(
        bounds(),
        [CategoryValues::new("experiment", ["left", "right", "left"]).unwrap()],
        values([0, 230, 120]),
    )
    .unwrap();
    let prepared = encoded
        .prepare(AlphaUnitKind::Bin, 100, &[], &["experiment"])
        .unwrap();
    assert_eq!(
        prepared
            .bin((0.0, 1.0), &[("experiment", "right")])
            .unwrap(),
        Some(ResolvedAlpha {
            index: 1,
            alpha: 230
        })
    );
    assert_eq!(
        prepared
            .bin((-0.0, 1.0), &[("experiment", "left")])
            .unwrap()
            .unwrap()
            .alpha,
        0
    );
    assert_eq!(
        prepared
            .bin((1.0, 2.0), &[("experiment", "left")])
            .unwrap()
            .unwrap()
            .alpha,
        120
    );
    assert!(prepared
        .bin((0.0, 1.0_f64.next_up()), &[("experiment", "left")])
        .is_err());
    assert!(prepared
        .bin((1.0, 2.0), &[("experiment", "right")])
        .is_err());
    assert!(prepared.bin((0.0, 1.0), &[]).is_err());
    assert!(prepared.bin((1.0, 0.0), &[("experiment", "left")]).is_err());
    assert!(encoded.prepare(AlphaUnitKind::Bin, 100, &[], &[]).is_err());
    assert!(encoded
        .prepare(AlphaUnitKind::Series, 100, &[], &["experiment"])
        .is_err());
    let duplicate_zero = Bounds::new(
        crate::NumericValues::new("lower", [-0.0, 0.0]),
        crate::NumericValues::new("upper", [1.0, 1.0]),
    );
    assert!(AlphaEncoding::bins(duplicate_zero, [], values([17, 17])).is_err());
    for (low, high) in [
        (None, Some(1.0)),
        (Some(0.0), None),
        (Some(f64::NAN), Some(1.0)),
        (Some(0.0), Some(f64::INFINITY)),
        (Some(1.0), Some(1.0)),
        (Some(2.0), Some(1.0)),
    ] {
        let bounds = Bounds::new(
            crate::NumericValues::nullable("lower", [low]),
            crate::NumericValues::nullable("upper", [high]),
        );
        let error = AlphaEncoding::bins(bounds, [], values([0])).unwrap_err();
        assert!(matches!(error, Error::Data(_)));
        assert!(error.to_string().contains("index 0"));
    }
    assert!(AlphaEncoding::bins(bounds(), [], values([0, 17])).is_err());
    let empty = AlphaEncoding::bins(
        Bounds::new(
            crate::NumericValues::new("lower", []),
            crate::NumericValues::new("upper", []),
        ),
        [],
        values([]),
    )
    .unwrap();
    assert!(empty
        .prepare(AlphaUnitKind::Bin, 100, &[], &[])
        .unwrap()
        .bin((0.0, 1.0), &[])
        .is_err());
}

#[test]
fn explicit_unit_instructions_replace_each_target_and_survive_owned_plot_and_composed_copies() {
    let observations = AlphaEncoding::observations(values([127; 3]));
    assert_eq!(observations.values().unwrap().len(), 3);
    assert!(observations
        .prepare(AlphaUnitKind::Observation, 2, &[], &[])
        .is_err());
    assert!(observations
        .prepare(AlphaUnitKind::Series, 3, &[], &[])
        .is_err());
    assert!(observations
        .prepare(AlphaUnitKind::Aggregate, 3, &[], &[])
        .is_err());
    assert!(AlphaEncoding::single(values([])).is_err());
    assert!(AlphaEncoding::single(values([17, 17])).is_err());
    let mut targets = AlphaInputs::default();
    assert!(targets.get(AlphaTarget::Point).is_theme());
    let single = AlphaEncoding::single(values([90])).unwrap();
    targets.set(AlphaTarget::Point, observations.clone());
    targets.set(AlphaTarget::Line, single.clone());
    targets.set(
        AlphaTarget::Fill,
        AlphaEncoding::single(values([0])).unwrap(),
    );
    targets.set(AlphaTarget::Line, AlphaEncoding::theme());
    assert_eq!(targets.get(AlphaTarget::Point), &observations);
    assert!(targets.get(AlphaTarget::Line).is_theme());
    assert_eq!(
        targets.get(AlphaTarget::Fill).values().unwrap().get(0),
        Some(0)
    );
    assert_eq!(
        single
            .prepare(AlphaUnitKind::Observation, 3, &[], &[])
            .unwrap()
            .observation(2, 300)
            .unwrap(),
        Some(ResolvedAlpha {
            index: 0,
            alpha: 90
        })
    );
    assert_eq!(
        single
            .prepare(AlphaUnitKind::Single, 3, &[], &[])
            .unwrap()
            .single()
            .unwrap()
            .unwrap()
            .alpha,
        90
    );
    assert_eq!(
        AlphaEncoding::theme()
            .prepare(AlphaUnitKind::Single, 3, &[], &[])
            .unwrap()
            .single()
            .unwrap(),
        None
    );
    let source = crate::Plot::line(
        crate::NumericValues::new("x", [0.0, 1.0, 2.0]),
        crate::NumericValues::new("y", [1.0, 2.0, 3.0]),
    )
    .with_alpha(AlphaTarget::Point, observations)
    .with_alpha(AlphaTarget::Line, single.clone())
    .with_alpha(AlphaTarget::Fill, single.clone());
    let erased = source.clone().erase();
    assert_eq!(erased.alpha_inputs(), source.alpha_inputs());
    let retained = erased
        .retain_composed_inputs(
            crate::decorations::legend::discrete::scoped::ScopedGuideInputs::for_test(&erased),
        )
        .unwrap();
    assert!(erased.same_composed_inputs(&retained, &retained.guides));
    for target in [AlphaTarget::Point, AlphaTarget::Line, AlphaTarget::Fill] {
        let changed = source
            .clone()
            .with_alpha(target, AlphaEncoding::theme())
            .erase();
        assert!(!changed.same_composed_inputs(&retained, &retained.guides));
    }
    let faceted = source.facet(crate::FacetSpec::row(
        CategoryValues::new("experiment", ["left", "right", "left"]).unwrap(),
    ));
    let keyed = AlphaEncoding::keyed(
        [CategoryValues::new("experiment", ["right", "left"]).unwrap()],
        values([230, 90]),
    )
    .unwrap();
    let prepared = faceted
        .with_alpha(AlphaTarget::Line, keyed)
        .prepare_alpha(AlphaTarget::Line, AlphaUnitKind::Series, &[])
        .unwrap();
    assert_eq!(
        prepared
            .keyed(&[("experiment", "left")])
            .unwrap()
            .unwrap()
            .alpha,
        90
    );
}
