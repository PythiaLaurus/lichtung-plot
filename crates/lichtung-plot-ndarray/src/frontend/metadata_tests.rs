use std::rc::Rc;

use ndarray::{array, Array2};

use crate::{kind, AdapterError, FacetScales, Labels, Line, Plot, Raincloud, Scatter};

#[test]
fn source_free_pair_outlives_source_and_restores_validated_lookup() {
    let (plot, metadata) = {
        let source = array![[0., 1.], [1., 0.]];
        let adapted = Scatter::from_data(source.view(), 0, 1).unwrap();
        let lookup = Rc::downgrade(adapted.source.as_ref().unwrap());
        let pair = adapted.into_parts();
        assert!(lookup.upgrade().is_none());
        pair
    };
    let restored = Scatter::from_parts(plot, metadata);
    assert!(matches!(
        restored.clone().hue(0usize),
        Err(AdapterError::MissingSource { role: "hue", .. })
    ));
    let groups = array!["a", "b"];
    assert!(restored.clone().hue(groups.view()).is_ok());
    let (plot, metadata) = restored.into_parts();

    let values = Array2::<f64>::zeros((2, 2));
    let wrong_validity = Array2::from_elem((1, 4), true);
    assert!(matches!(
        Scatter::from_parts_with_data(
            plot.clone(),
            metadata.clone(),
            (values.view(), wrong_validity.view()),
        ),
        Err(AdapterError::Shape { expected, actual, .. })
            if expected == [2, 2] && actual == [1, 4]
    ));

    // Source restoration itself does not impose the old observation count.
    let longer = array![["a"], ["b"], ["c"]];
    let restored =
        Scatter::from_parts_with_data(plot.clone(), metadata.clone(), longer.view()).unwrap();
    assert!(matches!(
        restored.hue(0usize),
        Err(AdapterError::Binding { role: "hue", .. })
    ));
    let lookup = array![["a"], ["b"]];
    let restored = Scatter::from_parts_with_data(plot, metadata, lookup.view())
        .unwrap()
        .hue(0usize)
        .unwrap();
    assert_eq!(restored.kind(), crate::PlotKind::Scatter);
}

#[test]
fn restored_policy_caches_keep_replacements_and_native_validation() {
    let source = array![[0., 1., 4.], [5., 2., 5.], [20., 3., 6.]];
    let adapted = Line::from_data(source.view(), 0, [2, 1])
        .unwrap()
        .labels(Labels::humanized().alias("discarded", "Old"))
        .column_labels(["Old right", "Old left"])
        .unwrap()
        .facet_columns_row()
        .unwrap();
    let (plot, metadata) = adapted.into_parts();
    let adapted = Line::from_parts(plot, metadata)
        .facet_scales(FacetScales::FreeY)
        .unwrap()
        .labels(Labels::raw().alias("retained", "Kept"));
    let (plot, metadata) = adapted.into_parts();
    let replacement = array![Some("a"), None, Some("b")];
    let adapted = Line::from_parts(plot, metadata)
        .column_labels(["Right", "Left"])
        .unwrap()
        .facet_column(replacement.view())
        .unwrap();
    assert_eq!(
        adapted.bindings.labels,
        Labels::raw().alias("retained", "Kept").aliases([
            (crate::source::name(2), "Right"),
            (crate::source::name(1), "Left"),
        ])
    );
    let (plot, metadata) = adapted.into_parts();
    let adapted = Line::from_parts(plot, metadata)
        .facet_scales(FacetScales::FreeX)
        .unwrap();
    // Changing scales must keep the replacement facet, not resurrect the old
    // generated one. Nullable facet applicability remains a native error.
    let native: Plot<kind::Line> = adapted.into();
    assert!(
        matches!(native.preflight(), Err(crate::Error::Data(detail)) if detail.contains("facet") && detail.contains("null"))
    );
}

#[test]
fn supplied_geometry_point_alpha_retains_its_original_domain_after_roundtrip() {
    let adapted = Raincloud::geometry(
        array!["a"].view(),
        array![0.].view(),
        array![1.].view(),
        array![2.].view(),
        array!["a", "a"].view(),
        array![-1., 3.].view(),
        array![0.5, 0.5].view(),
        array!["a", "a", "a"].view(),
        array![0., 1., 2.].view(),
        array![0.25, 0.5, 0.75].view(),
        "category",
        "value",
    )
    .unwrap();
    let (plot, metadata) = adapted.into_parts();
    let adapted = Raincloud::from_parts(plot, metadata);
    assert!(adapted
        .clone()
        .point_alpha(array![0u8, 128, 255].view())
        .is_ok());
    assert!(matches!(
        adapted.point_alpha(array![64u8, 128].view()),
        Err(AdapterError::Binding { role: "alpha", detail })
            if detail.contains("expected 3 slots, got 2")
    ));
}
