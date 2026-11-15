use ndarray::{array, s, Array2};

use crate::{
    AdapterError, AlphaValuesInput, ArraySource, AxisInput, AxisValues, BooleanInput,
    CategoryInput, ColumnProjection, ColumnsInput, NumericInput, SourceRowsInput,
};

#[test]
fn descriptor_validation_checks_original_bounds_shapes_and_validity_without_role_admission() {
    let values = array![[1], [2]];
    let one = values.view();
    assert!(matches!(
        ColumnProjection::new((2, 10), &[]),
        Err(AdapterError::Binding {
            role: "column projection",
            ..
        })
    ));
    assert!(matches!(
        ColumnProjection::new((2, 10), &[(7, &one), (7, &one)]),
        Err(AdapterError::Binding {
            role: "column projection",
            ..
        })
    ));
    assert!(matches!(
        ColumnProjection::new((2, 10), &[(10, &one)]),
        Err(AdapterError::Column {
            index: 10,
            columns: 10,
            ..
        })
    ));
    assert!(matches!(
        ColumnProjection::new((3, 10), &[(7, &one)]),
        Err(AdapterError::Shape { expected, actual, .. })
            if expected == [3, 1] && actual == [2, 1]
    ));
    let wide = array![[1, 2]];
    let two_columns = wide.view();
    assert!(matches!(
        ColumnProjection::new((1, 10), &[(7, &two_columns)]),
        Err(AdapterError::Shape { expected, actual, .. })
            if expected == [1, 1] && actual == [1, 2]
    ));
    let wrong_validity = array![[true, true]];
    let invalid = (one, wrong_validity.view());
    assert!(matches!(
        ColumnProjection::new((2, 10), &[(7, &invalid)]),
        Err(AdapterError::Shape { operation: "source validity", expected, actual })
            if expected == [2, 1] && actual == [1, 2]
    ));
    let empty = Array2::<&str>::from_elem((0, 1), "");
    let empty_view = empty.view();
    let entries: [(usize, &dyn ArraySource); 1] = [(7, &empty_view)];
    let source = ColumnProjection::new((0, 10), &entries).unwrap();
    assert_eq!(source.shape(), (0, 10));
    assert!(source.category_column(7, "category").unwrap().is_empty());
    assert!(matches!(
        source.numeric_column(7, "numeric"),
        Err(AdapterError::UnsupportedElement {
            column: Some(7),
            ..
        })
    ));
}

#[test]
fn identity_only_projection_supports_restored_column_alpha_without_value_lookup() {
    let x = array![[0.], [1.]];
    let control = array![[1.], [2.]];
    let treated = array![[2.], [3.]];
    let x_view = x.view();
    let control_view = control.view();
    let treated_view = treated.view();
    let entries: [(usize, &dyn ArraySource); 3] =
        [(4, &x_view), (7, &control_view), (2, &treated_view)];
    let identities = [(4, "time"), (7, "control"), (2, "treated")];
    let source = ColumnProjection::new((2, 10), &entries)
        .unwrap()
        .with_identities(&identities)
        .unwrap();
    let (plot, metadata) = crate::Line::from_data(source, 4usize, [7usize, 2usize])
        .unwrap()
        .into_parts();

    let namespace = ColumnProjection::from_identities((2, 10), &identities).unwrap();
    assert_eq!(namespace.shape(), (2, 10));
    assert_eq!(
        crate::source::identity(Some(&namespace), 7, "key").unwrap(),
        "control"
    );
    assert!(matches!(
        namespace.numeric_column(7, "value"),
        Err(AdapterError::Binding { detail, .. }) if detail.contains("was not projected")
    ));
    let column_ids = array![7usize, 2usize];
    let bytes = array![55u8, 173u8];
    // Ordinary direct ID/byte views retain their original protocols. Neither
    // vector is a value entry masquerading as one of the source's ten columns.
    let _adapted = crate::Line::from_parts(plot, metadata)
        .line_alpha_columns_from_data(
            namespace,
            column_ids.view(),
            std::iter::empty::<(&str, usize)>(),
            bytes.view(),
        )
        .unwrap();

    let explicit_empty = ColumnProjection::from_identities((2, 10), &[]).unwrap();
    assert!(matches!(
        crate::source::identity(Some(&explicit_empty), 7, "key"),
        Err(AdapterError::Binding { detail, .. }) if detail.contains("no raw identity")
    ));
    assert!(matches!(
        ColumnProjection::from_identities((2, 10), &[(10, "outside")]),
        Err(AdapterError::Column {
            index: 10,
            columns: 10,
            ..
        })
    ));
}

#[test]
fn explicit_identity_tables_replace_completely_without_authorizing_value_lookup() {
    let values = array![[1], [2]];
    let one = values.view();
    let entries: [(usize, &dyn ArraySource); 1] = [(7, &one)];
    let projection = || ColumnProjection::new((2, 10), &entries).unwrap();
    assert!(matches!(
        projection().with_identities(&[]),
        Err(AdapterError::Binding {
            role: "column identities",
            ..
        })
    ));
    assert!(matches!(
        projection().with_identities(&[(2, "only-unprojected")]),
        Err(AdapterError::Binding { detail, .. }) if detail.contains("column 7")
    ));
    assert!(matches!(
        projection().with_identities(&[(7, "a"), (7, "b")]),
        Err(AdapterError::Binding { detail, .. }) if detail.contains("duplicate logical column 7")
    ));
    assert!(matches!(
        projection().with_identities(&[(7, "same"), (2, "same")]),
        Err(AdapterError::Binding { detail, .. }) if detail.contains("duplicate raw identity")
    ));
    assert!(matches!(
        projection().with_identities(&[(7, "bound"), (10, "outside")]),
        Err(AdapterError::Column {
            index: 10,
            columns: 10,
            ..
        })
    ));
    let first = [(7, "bound"), (2, "identity-only")];
    let source = projection().with_identities(&first).unwrap();
    assert_eq!(
        crate::source::identity(Some(&source), 2, "key").unwrap(),
        "identity-only"
    );
    assert!(matches!(
        source.numeric_column(2, "value"),
        Err(AdapterError::Binding { detail, .. }) if detail.contains("column 2 was not projected")
    ));
    let replacement = [(7, ""), (3, " ")];
    let source = source.with_identities(&replacement).unwrap();
    assert_eq!(7usize.raw_numeric(Some(&source), "x").unwrap().0, "");
    assert_eq!(
        crate::source::identity(Some(&source), 3, "key").unwrap(),
        " "
    );
    assert!(matches!(
        crate::source::identity(Some(&source), 2, "key"),
        Err(AdapterError::Binding { detail, .. }) if detail.contains("column 2")
    ));
    assert!(matches!(
        crate::source::identity(Some(&source), 10, "key"),
        Err(AdapterError::Column {
            index: 10,
            columns: 10,
            ..
        })
    ));
    assert!((..).bind_columns(Some(&source)).is_err());
}

#[test]
fn positional_projection_keeps_original_selection_order_and_missing_column_errors() {
    let seven = array![[70], [71]];
    let two = array![[20], [21]];
    let seven_view = seven.view();
    let two_view = two.view();
    let entries: [(usize, &dyn ArraySource); 2] = [(2, &two_view), (7, &seven_view)];
    let source = ColumnProjection::new((2, 10), &entries).unwrap();
    let (rows, selected) = [7, 2].bind_columns(Some(&source)).unwrap();
    assert_eq!(rows, 2);
    assert_eq!(
        selected,
        [
            (7, vec![Some(70.), Some(71.)]),
            (2, vec![Some(20.), Some(21.)])
        ]
    );
    assert_eq!(
        7usize.raw_numeric(Some(&source), "x").unwrap().0,
        "Column 8"
    );
    assert!(matches!(
        [7, 7].bind_columns(Some(&source)),
        Err(AdapterError::Binding { detail, .. }) if detail.contains("duplicate column 7")
    ));
    assert!([].bind_columns(Some(&source)).is_err());
    assert!(matches!(
        10usize.raw_numeric(Some(&source), "x"),
        Err(AdapterError::Column {
            index: 10,
            columns: 10,
            ..
        })
    ));
    assert!(matches!(
        0usize.raw_numeric(Some(&source), "x"),
        Err(AdapterError::Binding { detail, .. }) if detail.contains("column 0 was not projected")
    ));
    assert!((..).bind_columns(Some(&source)).is_err());
    // Positional column-alpha targets have never been constrained to source width.
    assert_eq!(
        crate::source::identity(Some(&source), usize::MAX, "key").unwrap(),
        format!("Column index {}", usize::MAX)
    );
}

#[test]
fn mixed_sparse_columns_share_raw_names_and_keep_independent_strides_nulls_and_nan() {
    let values = array![
        [Some(f64::NAN), Some(99.)],
        [Some(5.), Some(99.)],
        [None, Some(99.)],
        [Some(-0.), Some(99.)]
    ];
    let present = array![[true, true, false, true]];
    let numeric = (values.slice(s![..;-1, ..1]), present.t());
    let counts = array![[2u16], [4], [6], [8]];
    let categories = array![[Some(" A ")], [None], [Some("é")], [Some("e\u{301}")]];
    let booleans = array![[Some(true)], [None], [Some(false)], [Some(true)]];
    let bytes = array![[0u8], [64], [128], [255]];
    let counts_view = counts.view();
    let categories_view = categories.view();
    let booleans_view = booleans.view();
    let bytes_view = bytes.view();
    let entries: [(usize, &dyn ArraySource); 5] = [
        (7, &numeric),
        (2, &counts_view),
        (9, &categories_view),
        (3, &booleans_view),
        (8, &bytes_view),
    ];
    let identities = [
        (2, "count"),
        (9, "cohort"),
        (8, "confidence"),
        (7, "measure"),
        (3, "flag"),
    ];
    let source = ColumnProjection::new((4, 12), &entries)
        .unwrap()
        .with_identities(&identities)
        .unwrap();
    let (name, values) = 7usize.raw_numeric(Some(&source), "x").unwrap();
    assert_eq!(name, "measure");
    assert_eq!(values[0].unwrap().to_bits(), (-0.0_f64).to_bits());
    assert!(values[1].is_none()); // Option absence with present=true
    assert!(values[2].is_none()); // present=false hides a nonnull value
    assert!(values[3].unwrap().is_nan()); // present NaN is not a null
    assert_eq!(
        9usize.raw_category(Some(&source), "hue").unwrap(),
        (
            "cohort".into(),
            vec![
                Some(" A ".into()),
                None,
                Some("é".into()),
                Some("e\u{301}".into())
            ]
        )
    );
    assert_eq!(
        3usize.raw_boolean(Some(&source), "highlight").unwrap(),
        (
            "flag".into(),
            vec![Some(true), None, Some(false), Some(true)]
        )
    );
    assert!(matches!(
        7usize.bind_axis(Some(&source), "x").unwrap(),
        AxisValues::Numeric(values) if values.name() == "measure" && values.len() == 4
    ));
    assert!(matches!(
        9usize.bind_axis(Some(&source), "x").unwrap(),
        AxisValues::Category(values) if values.name() == "cohort" && values.len() == 4
    ));
    let alpha = 8usize.bind_alpha_values(Some(&source)).unwrap();
    assert_eq!(alpha.name(), "confidence");
    assert_eq!(source.byte_column(8, "alpha").unwrap(), [0, 64, 128, 255]);
    assert_eq!(
        2usize.raw_source_rows(Some(&source), "IDs").unwrap(),
        [2, 4, 6, 8]
    );
}

#[test]
fn delegated_dtype_and_exact_id_errors_keep_original_columns_and_logical_slots() {
    let booleans = array![[true], [false]];
    let absent = Array2::from_elem((2, 1), false);
    let boolean = (booleans.view(), absent.view());
    let ids = array![[Some(3i64)], [None]];
    let ids_view = ids.view();
    let entries: [(usize, &dyn ArraySource); 2] = [(7, &boolean), (2, &ids_view)];
    let identities = [(7, "flag"), (2, "diagnostic-id")];
    let source = ColumnProjection::new((2, 10), &entries)
        .unwrap()
        .with_identities(&identities)
        .unwrap();
    assert!(matches!(
        7usize.raw_numeric(Some(&source), "x"),
        Err(AdapterError::UnsupportedElement {
            role: "x",
            column: Some(7),
            ..
        })
    ));
    assert!(matches!(
        7usize.bind_axis(Some(&source), "axis"),
        Err(AdapterError::UnsupportedElement {
            role: "axis",
            column: Some(7),
            ..
        })
    ));
    assert!(matches!(
        2usize.raw_source_rows(Some(&source), "IDs"),
        Err(AdapterError::SourceRow {
            slot: 1,
            column: Some(2),
            ..
        })
    ));
}
